use core::fmt;

use crate::process::ModeContext;

#[allow(rustdoc::private_intra_doc_links)]
/// Обёртка для [`ModeContext`].
/// Она делает запись в него через [`core::ptr::write_volatile()`],
/// чтобы компилятор такую операцию записи не выкинул.
#[repr(transparent)]
pub struct TrapContext(ModeContext);

impl TrapContext {
    /// Возвращает `true`, если контекст имеет привилегии пользователя.
    pub(crate) fn is_user_mode(&self) -> bool {
        self.0.is_user_mode()
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Возвращает [`ModeContext`], содержащийся в этом [`TrapContext`].
    pub fn get(&self) -> ModeContext {
        self.0
    }

    /// Записывает `context` в этот [`TrapContext`].
    ///
    /// Делает это через [`core::ptr::write_volatile()`],
    /// чтобы компилятор такую операцию записи не выкинул.
    pub(crate) fn set(
        &mut self,
        context: ModeContext,
    ) {
        unsafe {
            (&mut self.0 as *mut ModeContext).write_volatile(context);
        }
    }
}

impl fmt::Debug for TrapContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{:?}", self.get())
    }
}

impl fmt::Display for TrapContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get())
    }
}
