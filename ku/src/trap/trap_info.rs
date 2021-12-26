use core::{
    fmt,
    mem,
};

use memoffset::offset_of;

use crate::{
    error::Result,
    memory::{
        PageFaultInfo,
        Virt,
    },
};

use super::{
    MiniContext,
    mini_context::RSP_OFFSET_IN_MINI_CONTEXT,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapInfo {
    number: usize,
    info: Info,
    context: MiniContext,

    /// `TrapInfo` can be pushed onto the same stack the `context` is pointing to.
    /// Eg. if the trap is recursive - the trap has happened inside a trap handler.
    /// In this case `lib::syscall::trap_trampoline()` and `lib::syscall::trap_handler_invoker()`
    /// will push a return address onto the `context` stack.
    /// This field exists only to protect meaningfull fields of the `TrapInfo` from beeing overwritten.
    return_address_placeholder: [u8; Self::PLACEHOLDER_SIZE],
}

impl TrapInfo {
    const PLACEHOLDER_SIZE: usize = mem::size_of::<Virt>();

    pub fn new(
        number: usize,
        info: Info,
        context: MiniContext,
    ) -> Self {
        Self {
            number,
            info,
            context,
            return_address_placeholder: [0; Self::PLACEHOLDER_SIZE],
        }
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn info(&self) -> Info {
        self.info
    }

    pub fn context(&self) -> MiniContext {
        self.context
    }

    pub unsafe fn prepare_for_ret(&mut self) -> Result<()> {
        *self.context.push::<Virt>()?.try_into_mut()? = self.context.rip();
        Ok(())
    }
}

impl fmt::Display for TrapInfo {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ #{}, {}, {} }}",
            self.number, self.info, self.context
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Info {
    None,
    Code(u64),
    PageFault {
        address: Virt,
        code: PageFaultInfo,
    },
}

impl fmt::Display for Info {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Info::None => write!(formatter, "{{ }}"),
            Info::Code(code) => write!(formatter, "{{ code: {code} }}"),
            Info::PageFault { address, code } => {
                write!(formatter, "{{ address: {address}, code: {code} }}")
            },
        }
    }
}

pub const RSP_OFFSET_IN_TRAP_INFO: usize =
    offset_of!(TrapInfo, context) + RSP_OFFSET_IN_MINI_CONTEXT;
