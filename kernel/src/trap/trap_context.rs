use core::fmt;

use volatile::Volatile;

use crate::process::ModeContext;

#[repr(transparent)]
pub struct TrapContext(ModeContext);

impl TrapContext {
    pub(super) fn is_user_mode(&self) -> bool {
        self.0.is_user_mode()
    }

    pub(super) fn get(&self) -> ModeContext {
        self.0
    }

    pub(super) fn set(
        &mut self,
        context: ModeContext,
    ) {
        Volatile::new(&mut self.0).write(context);
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
