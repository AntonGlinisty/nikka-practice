use core::{
    cmp,
    mem,
    ops::Range,
};

use bootloader_api::info::{
    MemoryRegionKind,
    MemoryRegions,
};
use duplicate::duplicate_item;
use lazy_static::lazy_static;

use ku::sync::Spinlock;

use crate::{
    error::{
        Error::NoFrame,
        Result,
    },
    log::{
        debug,
        error,
        info,
        trace,
    },
    time,
};

use super::{
    BASE_ADDRESS_SPACE,
    Block,
    FrameGuard,
    KERNEL_RW,
    Phys,
    Virt,
    Size,
    frage::{
        Frame,
        Page,
    },
};

// Used in docs.
#[allow(unused)]
use {
    super::Mapping,
    crate::error::Error,
};

/// Информация об одном физическом фрейме.
#[derive(Clone, Copy, Debug, Default)]
enum FrameInfo {
    /// Фрейм не доступен --- либо находится за пределами физической памяти,
    /// либо зарезервирован аппаратурой, загрузчиком или BIOS.
    #[default]
    Absent,

    /// Фрейм свободен.
    Free {
        /// Номер следующего свободного фрейма.
        next_free: Option<usize>,
    },

    /// Фрейм занят.
    Used {
        /// Количество ссылок на этот фрейм.
        /// Например, из одного или разных страничных отображений [`Mapping`].
        reference_count: usize,
    },
}

/// Инициализирует аллокатор [`FrameAllocator`] по информации из `memory_regions`.
/// Использует для него временное хранилище метаданных.
#[allow(static_mut_refs)]
pub(super) fn init(memory_regions: &MemoryRegions) -> FrameAllocator {
    // ANCHOR: log_tsc
    let timer = time::timer();
    let frame_allocator = FrameAllocator::new(unsafe { &mut BOOT_FRAME_INFO.0 }, memory_regions);
    info!(
        free_frame_count = frame_allocator.count(),
        duration = %timer.elapsed(),
        "frame allocator temporary storage init",
    );
    // ANCHOR_END: log_tsc
    frame_allocator
}

/// Изменяет размер метаданных аллокатора [`FrameAllocator`] так,
/// чтобы он мог хранить информацию про
/// всю доступную в машине физическую память --- `physical_memory`.
///
/// Уже имеющиеся в [`FrameAllocator`] метаданные переносит
/// из временного хранилища в новое и освобождает временное хранилище.
/// Ещё не заполненные метаданные заполняет по информации из `memory_regions`.
pub(super) fn resize(
    physical_memory: Block<Frame>,
    memory_regions: &MemoryRegions,
) {
    let frame_info = BASE_ADDRESS_SPACE
        .lock()
        .map_slice(
            physical_memory.count(),
            KERNEL_RW,
            FrameInfo::default,
        )
        .expect("failed to allocate frame metadata");

    let old_memory = FRAME_ALLOCATOR.lock().resize(frame_info, memory_regions);

    let addresses = old_memory.as_ptr_range();
    let start = Page::new(Virt::from_ptr(addresses.start)).expect("invalid metadata block");
    let end = Page::new(Virt::from_ptr(addresses.end)).expect("invalid metadata block");

    let pages = Block::new(start, end).expect("invalid metadata block");

    unsafe {
        BASE_ADDRESS_SPACE
            .lock()
            .unmap_block(pages)
            .expect("failed to unmap metadata");
    }
}

/// Основной аллокатор физических фреймов.
#[derive(Default)]
pub struct FrameAllocator {
    /// Вспомогательная запись, которая используется при запросе номера фрейма,
    /// выходящего за границы [`field@FrameAllocator::frame_info`].
    absent: FrameInfo,

    /// Был ли доступ за границы [`field@FrameAllocator::frame_info`].
    access_beyond_frame_info: bool,

    /// Информация про все доступные физические фреймы.
    frame_info: &'static mut [FrameInfo],

    /// Количество свободных физических фреймов.
    free_count: usize,

    /// Голова интрузивного списка номеров свободных физических фреймов.
    free_frame: Option<usize>,
}

impl FrameAllocator {
    /// Инициализирует аллокатор [`FrameAllocator`] по информации из `memory_regions`.
    fn new(
        frame_info: &'static mut [FrameInfo],
        memory_regions: &MemoryRegions,
    ) -> Self {
        let mut frame_allocator = Self {
            absent: FrameInfo::Absent,
            access_beyond_frame_info: false,
            frame_info,
            free_count: 0,
            free_frame: None,
        };

        let frame_count = frame_allocator.frame_info.len();

        frame_allocator.init_frame_info(memory_regions, 0 .. frame_count);

        debug!(
            frame_count,
            physical_memory = %Size::new::<Frame>(frame_count),
            "frame allocator temporary storage",
        );

        frame_allocator
    }

    /// Возвращает количество свободных физических фреймов у аллокатора.
    pub fn count(&self) -> usize {
        self.free_count
    }

    /// Выделяет ровно один физический фрейм.
    /// Возвращает [`FrameGuard`], владеющий ссылкой на этот фрейм.
    ///
    /// Если свободных физических фреймов не осталось,
    /// возвращает ошибку [`Error::NoFrame`].
    pub fn allocate(&mut self) -> Result<FrameGuard> {
        if self.free_count == 0 {
            return Err(Error::NoFrame);
        }

        let free_frame_index = self.free_frame.ok_or(Error::NoFrame)?;
        let new_frame = Frame::from_index(free_frame_index)?;

        let next_free_frame_index = match self.frame_info[free_frame_index] {
            FrameInfo::Free { next_free } => {
                next_free
            },
            _ => {
                panic!("free list points to a non-free frame")
            }
        };

        self.frame_info[free_frame_index] = FrameInfo::Used { reference_count: 1 };
        self.free_count -= 1;
        self.free_frame = next_free_frame_index;

        Ok(FrameGuard::new(new_frame))
    }

    /// Уменьшает на единицу счётчик использований заданного физического фрейма `frame`.
    /// Физический фрейм освобождается, если на него не осталось других ссылок.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен.
    pub fn deallocate(
        &mut self,
        frame: Frame,
    ) {
        match self.frame_info_mut(frame) {
            FrameInfo::Used { reference_count } => {
                if *reference_count == 1 {
                    self.frame_info[frame.index()] = FrameInfo::Free { next_free: self.free_frame };
                    self.free_count += 1;
                    self.free_frame = Some(frame.index());
                } else if *reference_count == 0 {
                    panic!("FrameInfo::Used reference_count == 0")
                } else {
                    *reference_count -= 1;
                }
            },
            FrameInfo::Free { .. } => {
                panic!("trying deallocate free frame")
            },
            _ => {}
        }
    }

    /// Увеличивает на единицу счётчик использований заданного физического фрейма `frame`.
    /// Возвращает [`FrameGuard`], владеющий новой ссылкой на этот фрейм.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен.
    pub fn reference(
        &mut self,
        frame: Frame,
    ) -> FrameGuard {
        match self.frame_info_mut(frame) {
            FrameInfo::Used { reference_count } => {
                *reference_count += 1;
            },
            FrameInfo::Free { .. } => {
                panic!("trying to take reference to the free frame");
            },
            _ => {}
        }

        FrameGuard::new(frame)
    }

    /// Возвращает количество ссылок на `frame`.
    ///
    /// - Если `frame` свободен, возвращается `0`.
    /// - Если он отсутствует или зарезервирован --- [`Error::NoFrame`].
    pub fn reference_count(
        &self,
        frame: Frame,
    ) -> Result<usize> {
        match *self.frame_info(frame) {
            FrameInfo::Absent => Err(NoFrame),
            FrameInfo::Free { .. } => Ok(0),
            FrameInfo::Used { reference_count } => Ok(reference_count),
        }
    }

    /// Проверяет, что заданный физический фрейм уже был выделен.
    pub fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        !matches!(self.frame_info(frame), FrameInfo::Free { .. })
    }

    /// Инициализирует метаданные аллокатора в диапазоне номеров
    /// физических фреймов `frames` по информации из `memory_regions`.
    fn init_frame_info(
        &mut self,
        memory_regions: &MemoryRegions,
        frames: Range<usize>,
    ) {
        if frames.is_empty() {
            return;
        }

        for region in memory_regions.iter() {
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }

            let start = core::cmp::max(frames.start,
                ku::memory::size::from(region.start).div_ceil(Frame::SIZE));
            let end = core::cmp::min(frames.end,
                ku::memory::size::from(region.end) / Frame::SIZE);

            for frame in start..end {
                self.frame_info[frame] = FrameInfo::Free { next_free: self.free_frame };
                self.free_count += 1;
                self.free_frame = Some(frame);
            }
        }
    }

    /// Переносит уже имеющиеся в [`FrameAllocator`] метаданные
    /// из временного хранилища в новое --- `frame_info`.
    /// Ещё не заполненные метаданные заполняет по информации из `memory_regions`.
    /// Переключает [`field@FrameAllocator::frame_info`] на новое хранилище.
    /// Возвращает старое временное хранилище метаданных.
    fn resize(
        &mut self,
        frame_info: &'static mut [FrameInfo],
        memory_regions: &MemoryRegions,
    ) -> &'static mut [FrameInfo] {
        assert!(
            !self.access_beyond_frame_info,
            "resize() after reference() of a frame beyond frame_info can break FrameAllocator's \
             invariants",
        );

        for index in 0..self.frame_info.len() {
            frame_info[index] = self.frame_info[index];
        }

        let last_index = self.frame_info.len();

        for memory_region in memory_regions.iter() {
            if memory_region.kind != MemoryRegionKind::Usable {
                continue;
            }

            let start = core::cmp::max(
                last_index,
                ku::memory::size::from(memory_region.start).div_ceil(Frame::SIZE)
            );

            let end = core::cmp::min(
                frame_info.len(),
                ku::memory::size::from(memory_region.end) / Frame::SIZE
            );

            for index in start..end {
                frame_info[index] = FrameInfo::Free { next_free: self.free_frame };
                self.free_frame = Some(index);
                self.free_count += 1;
            }
        }

        core::mem::replace(&mut self.frame_info, frame_info)
    }

    /// Возвращает ссылку на [`FrameInfo`], соответствующую физическому фрейму `frame`.
    ///
    /// Если номер фрейма `frame` попадает в диапазон [`field@FrameAllocator::frame_info`],
    /// то эта ссылка указывает на соответствующий элемент этого среза.
    ///
    /// Иначе, она указывает на специальную запись [`FrameAllocator::absent`],
    /// содержащую значение [`FrameInfo::Absent`].
    /// В этом случае, вызывающий код не должен менять эту запись.
    /// Кроме того, если запрос был на изменяемую ссылку,
    /// а возвращается [`FrameAllocator::absent`],
    /// дополнительно [`FrameAllocator::access_beyond_frame_info`] выставляется в `true`.
    /// После этого нельзя вызывать [`FrameAllocator::resize()`],
    /// так как это может нарушить инварианты [`FrameAllocator`]:
    /// ```ignore
    /// let frame = frame_beyond_old_frame_info_but_inside_new_frame_info;
    /// let guard = frame_allocator.reference(frame); // no reference increase
    /// frame_allocator.resize(...);
    /// drop(guard); // unbalanced reference decrease
    /// ```
    #[allow(clippy::needless_arbitrary_self_type)]
    #[allow(unused)]
    #[duplicate_item(
        frame_info_getter reference(x) access_beyond_frame_info(x);
        [frame_info] [&x] [];
        [frame_info_mut] [&mut x] [x.access_beyond_frame_info = true;];
    )]
    fn frame_info_getter(
        self: reference([Self]),
        frame: Frame,
    ) -> reference([FrameInfo]) {
        if frame.index() < self.frame_info.len() {
            reference([self.frame_info[frame.index()]])
        } else {
            access_beyond_frame_info([self]);
            reference([self.absent])
        }
    }
}

impl Drop for FrameAllocator {
    fn drop(&mut self) {
        if !self.frame_info.is_empty() {
            panic!("can not drop non-empty FrameAllocator");
        }
    }
}

lazy_static! {
    /// Аллокатор физических фреймов.
    pub static ref FRAME_ALLOCATOR: Spinlock<FrameAllocator> =
        Spinlock::new(FrameAllocator::default());
}

/// Временное хранилище метаданных для [`FrameAllocator`].
#[repr(align(4096))]
struct BootFrameInfo([FrameInfo; BOOT_FRAME_INFO_LEN]);

/// Временное хранилище метаданных для [`FrameAllocator`].
static mut BOOT_FRAME_INFO: BootFrameInfo = BootFrameInfo([FrameInfo::Absent; _]);

/// Количество описателей фреймов [`FrameInfo`] во
/// временном хранилище метаданных для [`FrameAllocator`].
const BOOT_FRAME_INFO_LEN: usize = 64 * Frame::SIZE / mem::size_of::<FrameInfo>();
