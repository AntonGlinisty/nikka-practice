#![allow(dead_code)]

use core::mem;

use derive_more::{
    Deref,
    Display,
};

use crate::error::Result;

use super::{
    FRAME_ALLOCATOR,
    frage::Frame,
};

// Used in docs.
#[allow(unused)]
use {
    super::FrameAllocator,
    ku::error::Error,
};

/// RAII для операции выделения одного [`Frame`].
///
/// Вызывает [`FrameAllocator::deallocate()`] для [frame][Frame] при
/// [`FrameGuard::drop()`], если [frame][Frame]
/// не был забран ранее с помощью [`FrameGuard::take()`].
///
/// # Examples
///
/// ## Автоматическое освобождение фрейма в случае ошибки
/// ```ignore
/// # fn fallible_initialization(_frame: Frame) -> Result<()> {
/// #     Ok(())
/// # }
/// #
/// fn get_initialized_frame() -> Result<Frame> {
///     let frame = FrameGuard::allocate()?;
///
///     // Благодаря тому, что FrameGuard реализует типаж Deref<Target = Frame>,
///     // принадлежащий ему Frame доступен через разыменование.
///     // В случае, если этот вызов возвращает Err(...), утечки не происходит и Frame освобождается
///     // автоматически.
///     fallible_initialization(*frame)?;
///     Ok(frame.take())
/// }
/// ```
#[derive(Debug, Deref, Display, Eq, Ord, PartialEq, PartialOrd)]
#[display("{}", frame)]
#[must_use]
pub struct FrameGuard {
    /// Фрейм, защищаемый от утечки.
    frame: Frame,
}

impl FrameGuard {
    /// Создает новый [`FrameGuard`] для данного [`frame`][Frame].
    ///
    /// Ограничение видимости `pub(super)` требуется для того, чтобы внешние
    /// (по отношению к `memory`) модули не могли создавать [`FrameGuard`] самостоятельно.
    /// И тем самым обойти контроль за утечками фреймов.
    pub(super) fn new(frame: Frame) -> Self {
        Self { frame }
    }

    /// Создает новый [`FrameGuard`] от выражения
    /// [`FRAME_ALLOCATOR.lock().allocate()`][FrameAllocator::allocate].
    ///
    /// # Errors
    ///
    /// - [`Error::NoFrame`] --- свободных физических фреймов не осталось.
    pub fn allocate() -> Result<Self> {
        Ok(Self::new(FRAME_ALLOCATOR.lock().allocate()?))
    }

    /// Забирает [`Frame`] из данного [`FrameGuard`].
    pub fn take(self) -> Frame {
        let frame = self.frame;
        mem::forget(self);
        frame
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().deallocate(self.frame);
    }
}
