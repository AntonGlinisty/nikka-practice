use core::{
    cmp,
    mem,
};

use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

use crate::{
    error::{
        Error::NoFrame,
        Result,
    },
    log::{
        debug,
        info,
        trace,
    },
    time,
};

use super::{
    BASE_ADDRESS_SPACE,
    BootFrameAllocator,
    FRAME_ALLOCATOR,
    FrameAllocator,
    KERNEL_RW,
    frage::Frame,
    range,
    size,
};

// Used in docs.
#[allow(unused)]
use {
    super::Mapping,
    crate::error::Error,
};

/// Информация об одном физическом фрейме.
#[derive(Clone, Debug)]
enum FrameInfo {
    /// Фрейм не доступен --- либо находится за пределами физической памяти,
    /// либо зарезервирован аппаратурой, загрузчиком или BIOS.
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

/// Инициализирует аллокатор [`MainFrameAllocator`] по информации из `memory_map`.
/// И передаёт под его управление все фреймы,
/// которые остались свободны в [`BootFrameAllocator`].
pub(super) fn init(memory_map: &MemoryMap) -> MainFrameAllocator {
    let timer = time::timer();
    let frame_allocator = MainFrameAllocator::new(memory_map);
    info!(
        frame_allocator = "main",
        free_frame_count = frame_allocator.count(),
        duration = %timer.elapsed(),
        "init",
    );
    frame_allocator
}

/// Основной аллокатор физических фреймов.
pub struct MainFrameAllocator {
    /// Информация про все доступные физические фреймы.
    frame_info: &'static mut [FrameInfo],

    /// Голова интрузивного списка номеров свободных физических фреймов.
    free_frame: Option<usize>,

    /// Количество свободных физических фреймов.
    free_count: usize,
}

impl MainFrameAllocator {
    /// Инициализирует аллокатор [`MainFrameAllocator`] по информации из `memory_map`.
    /// И передаёт под его управление все фреймы,
    /// которые остались свободны в [`BootFrameAllocator`].
    fn new(memory_map: &MemoryMap) -> Self {
        let mut timer = time::timer();

        let frame_count = range::physical(memory_map).count();
        let frame_info = BASE_ADDRESS_SPACE
            .lock()
            .map_slice(frame_count, KERNEL_RW, || FrameInfo::Absent)
            .expect("failed to allocate memory for MainFrameAllocator metadata");

        debug!(frame_allocator = "main", duration = %timer.lap(), "frame info mapped");

        let mut main_frame_allocator = Self {
            frame_info,
            free_frame: None,
            free_count: 0,
        };

        let boot_frame_allocator = mem::replace(&mut *FRAME_ALLOCATOR.lock(), FrameAllocator::Void);

        if let FrameAllocator::Boot(boot_frame_allocator) = boot_frame_allocator {
            main_frame_allocator.init_frame_info(memory_map, &boot_frame_allocator);
            debug!(frame_allocator = "main", duration = %timer.lap(), "frame info init");

            main_frame_allocator.move_free_frames(boot_frame_allocator);
            debug!(frame_allocator = "main", duration = %timer.lap(), "moved all free frames");

            main_frame_allocator
        } else {
            panic!("FrameAllocator should be a Boot one now");
        }
    }

    /// Возвращает количество свободных физических фреймов у аллокатора [`MainFrameAllocator`].
    pub(super) fn count(&self) -> usize {
        self.free_count
    }

    /// Выделяет ровно один физический фрейм.
    ///
    /// Если свободных физических фреймов не осталось,
    /// возвращает ошибку [`Error::NoFrame`].
    pub(super) fn allocate(&mut self) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Уменьшает на единицу счётчик использований заданного физического фрейма `frame`.
    /// Физический фрейм освобождается, если на него не осталось других ссылок.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен.
    pub(super) fn deallocate(
        &mut self,
        frame: Frame,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Увеличивает на единицу счётчик использований заданного физического фрейма `frame`.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен.
    pub(super) fn reference(
        &mut self,
        frame: Frame,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает количество ссылок на `frame`.
    ///
    /// - Если `frame` свободен, возвращается `0`.
    /// - Если он отсутствует или зарезервирован --- [`Error::NoFrame`].
    pub(super) fn reference_count(
        &self,
        frame: Frame,
    ) -> Result<usize> {
        match self.frame_info[frame.index()] {
            FrameInfo::Absent => Err(NoFrame),
            FrameInfo::Free { .. } => Ok(0),
            FrameInfo::Used { reference_count } => Ok(reference_count),
        }
    }

    /// Проверяет, что заданный физический фрейм уже был выделен.
    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        frame.index() >= self.frame_info.len() ||
            !matches!(self.frame_info[frame.index()], FrameInfo::Free { .. })
    }

    /// Инициализирует аллокатор [`MainFrameAllocator`] по информации из `memory_map`.
    /// Отмечает все фреймы, которые находятся под управлением [`BootFrameAllocator`]
    /// как занятые со счётчиком использований, равным 1.
    fn init_frame_info(
        &mut self,
        memory_map: &MemoryMap,
        boot_frame_allocator: &BootFrameAllocator,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// И передаёт под управление [`MainFrameAllocator`] все фреймы,
    /// которые на данный момент принадлежат [`BootFrameAllocator`] и
    /// являются свободными.
    #[allow(unused_mut)] // TODO: remove before flight.
    fn move_free_frames(
        &mut self,
        mut src: BootFrameAllocator,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for MainFrameAllocator {
    fn drop(&mut self) {
        panic!("can not drop MainFrameAllocator");
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use bootloader::bootinfo::MemoryMap;

    use super::range;

    pub fn frame_count(memory_map: &MemoryMap) -> usize {
        range::physical(memory_map).count()
    }
}
