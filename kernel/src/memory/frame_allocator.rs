use lazy_static::lazy_static;
use spin::Mutex;

use crate::error::{
    Error::NoFrame,
    Result,
};

use super::{
    BootFrameAllocator,
    Frame,
    main_frame_allocator::MainFrameAllocator,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Аллокатор физических фреймов [`FrameAllocator`],
/// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
pub enum FrameAllocator {
    /// Пустой аллокатор физических фреймов.
    Void,

    /// Временный аллокатор физических фреймов, используется при инициализации подсистемы памяти.
    Boot(BootFrameAllocator),

    /// Основной аллокатор физических фреймов.
    Main(MainFrameAllocator),
}

#[allow(rustdoc::private_intra_doc_links)]
impl FrameAllocator {
    /// Возвращает количество свободных физических фреймов у аллокатора [`FrameAllocator`].
    pub fn count(&self) -> usize {
        match self {
            FrameAllocator::Void => 0,
            FrameAllocator::Boot(boot_allocator) => boot_allocator.count(),
            FrameAllocator::Main(main_allocator) => main_allocator.count(),
        }
    }

    /// Выделяет ровно один физический фрейм.
    ///
    /// Если свободных физических фреймов не осталось,
    /// возвращает ошибку [`Error::NoFrame`].
    pub fn allocate(&mut self) -> Result<Frame> {
        match self {
            FrameAllocator::Void => Err(NoFrame),
            FrameAllocator::Boot(boot_allocator) => boot_allocator.allocate(),
            FrameAllocator::Main(main_allocator) => main_allocator.allocate(),
        }
    }

    /// Уменьшает на единицу счётчик использований заданного физического фрейма `frame`.
    /// Физический фрейм освобождается, если на него не осталось других ссылок.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен или
    /// если в данный момент за выделение физических фреймов отвечает не [`MainFrameAllocator`].
    pub fn deallocate(
        &mut self,
        frame: Frame,
    ) {
        match self {
            FrameAllocator::Main(main_allocator) => main_allocator.deallocate(frame),
            _ => panic!("can not deallocate a frame without the main frame allocator"),
        }
    }

    /// Увеличивает на единицу счётчик использований заданного физического фрейма `frame`.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен или
    /// если в данный момент за выделение физических фреймов отвечает не [`MainFrameAllocator`].
    pub fn reference(
        &mut self,
        frame: Frame,
    ) {
        match self {
            FrameAllocator::Main(main_allocator) => main_allocator.reference(frame),
            _ => panic!("can not reference a frame without the main frame allocator"),
        }
    }

    /// Проверяет, что заданный физический фрейм уже был выделен.
    ///
    /// # Panics
    ///
    /// Паникует, если в данный момент ни [`BootFrameAllocator`] ни [`MainFrameAllocator`]
    /// ещё не используются.
    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        match self {
            FrameAllocator::Void =>
                panic!("no frames should be marked as used by void frame allocator"),
            FrameAllocator::Boot(boot_allocator) => boot_allocator.is_used(frame),
            FrameAllocator::Main(main_allocator) => main_allocator.is_used(frame),
        }
    }
}

lazy_static! {
    /// Аллокатор физических фреймов [`FrameAllocator`],
    /// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::Void);
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use crate::log::error;

    use super::FRAME_ALLOCATOR;

    #[must_use]
    pub fn forbid_frame_leaks() -> impl Drop {
        scopeguard::guard(FRAME_ALLOCATOR.lock().count(), |start_free_frames| {
            let end_free_frames = FRAME_ALLOCATOR.lock().count();
            let (message, affected_frames) = if start_free_frames <= end_free_frames {
                (
                    "freed more memory than allocated",
                    end_free_frames - start_free_frames,
                )
            } else {
                ("leaked some memory", start_free_frames - end_free_frames)
            };
            if affected_frames != 0 {
                error!(start_free_frames, end_free_frames, affected_frames, message);
                panic!("{}", message);
            }
        })
    }
}
