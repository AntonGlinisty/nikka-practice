use duplicate::duplicate_item;
use lazy_static::lazy_static;

use ku::sync::Spinlock;

use crate::error::Result;

use super::{
    BootFrameAllocator,
    Frame,
    FrameGuard,
    main_frame_allocator::MainFrameAllocator,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Типаж аллокатора физических фреймов.
#[allow(rustdoc::private_intra_doc_links)]
pub trait FrameAllocator {
    /// Возвращает количество свободных физических фреймов у аллокатора.
    fn count(&self) -> usize;

    /// Выделяет ровно один физический фрейм.
    ///
    /// Если свободных физических фреймов не осталось,
    /// возвращает ошибку [`Error::NoFrame`].
    fn allocate(&mut self) -> Result<FrameGuard>;

    /// Уменьшает на единицу счётчик использований заданного физического фрейма `frame`.
    /// Физический фрейм освобождается, если на него не осталось других ссылок.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен или
    /// если в данный момент за выделение физических фреймов отвечает не [`MainFrameAllocator`].
    fn deallocate(
        &mut self,
        frame: Frame,
    );

    /// Увеличивает на единицу счётчик использований заданного физического фрейма `frame`.
    ///
    /// # Panics
    ///
    /// Паникует, если фрейм свободен или
    /// если в данный момент за выделение физических фреймов отвечает не [`MainFrameAllocator`].
    fn reference(
        &mut self,
        frame: Frame,
    ) -> FrameGuard;

    /// Возвращает количество ссылок на `frame`.
    ///
    /// - Если `frame` свободен, возвращается `0`.
    /// - Если он отсутствует или зарезервирован --- [`Error::NoFrame`].
    ///
    /// # Panics
    ///
    /// Паникует, если в данный момент за выделение физических фреймов
    /// отвечает не [`MainFrameAllocator`].
    fn reference_count(
        &self,
        frame: Frame,
    ) -> Result<usize>;

    /// Проверяет, что заданный физический фрейм уже был выделен.
    ///
    /// # Panics
    ///
    /// Паникует, если в данный момент ни [`BootFrameAllocator`] ни [`MainFrameAllocator`]
    /// ещё не используются.
    fn is_used(
        &self,
        frame: Frame,
    ) -> bool;
}

/// Аллокатор физических фреймов [`CurrentFrameAllocator`],
/// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
pub enum CurrentFrameAllocator {
    /// Пустой аллокатор физических фреймов.
    Void,

    /// Временный аллокатор физических фреймов, используется при инициализации подсистемы памяти.
    Boot(BootFrameAllocator),

    /// Основной аллокатор физических фреймов.
    Main(MainFrameAllocator),
}

#[allow(rustdoc::private_intra_doc_links)]
impl CurrentFrameAllocator {
    /// Возвращает ссылку на текущий аллокатор фреймов.
    ///
    /// # Panics
    ///
    /// Паникует, если в данный момент ни [`BootFrameAllocator`] ни [`MainFrameAllocator`]
    /// ещё не используются.
    #[allow(clippy::needless_arbitrary_self_type)]
    #[duplicate_item(
        getter reference(type);
        [get] [&type];
        [get_mut] [&mut type];
    )]
    fn getter(self: reference([Self])) -> reference([dyn FrameAllocator]) {
        match self {
            Self::Void => panic!("no current frame allocator"),
            Self::Boot(boot_allocator) => boot_allocator as reference([dyn FrameAllocator]),
            Self::Main(main_allocator) => main_allocator as reference([dyn FrameAllocator]),
        }
    }
}

impl FrameAllocator for CurrentFrameAllocator {
    #[allow(clippy::needless_arbitrary_self_type)]
    #[duplicate_item(
        method getter reference(type) result;
        [count] [get] [&type] [usize];
        [allocate] [get_mut] [&mut type] [Result<FrameGuard>];
    )]
    fn method(self: reference([Self])) -> result {
        self.getter().method()
    }

    #[allow(clippy::needless_arbitrary_self_type)]
    #[duplicate_item(
        method getter reference(type) result;
        [deallocate] [get_mut] [&mut type] [()];
        [reference] [get_mut] [&mut type] [FrameGuard];
        [reference_count] [get] [&type] [Result<usize>];
        [is_used] [get] [&type] [bool];
    )]
    fn method(
        self: reference([Self]),
        frame: Frame,
    ) -> result {
        self.getter().method(frame)
    }
}

lazy_static! {
    /// Аллокатор физических фреймов [`CurrentFrameAllocator`],
    /// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
    pub static ref FRAME_ALLOCATOR: Spinlock<CurrentFrameAllocator> =
        Spinlock::new(CurrentFrameAllocator::Void);
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use crate::log::error;

    use super::{
        super::BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        FrameAllocator,
    };

    #[must_use]
    pub fn forbid_frame_leaks() -> impl Drop {
        BASE_ADDRESS_SPACE.lock().mapping().unmap_unused_intermediate();

        scopeguard::guard(FRAME_ALLOCATOR.lock().count(), |start_free_frames| {
            BASE_ADDRESS_SPACE.lock().mapping().unmap_unused_intermediate();

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
