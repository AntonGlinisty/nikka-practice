use crate::error::{
    Error,
    Result,
};

use bitflags::bitflags;

bitflags! {
    pub struct ExitCode: usize {
        const OK = 0;
        const PANIC = 1;
        const UNIMPLEMENTED_SYSCALL = 2;
    }
}

bitflags! {
    pub struct Syscall: usize {
        const EXIT = 0;
        const LOG_VALUE = 1;
        const SCHED_YIELD = 2;
        const EXOFORK = 3;
        const MAP = 4;
        const UNMAP = 5;
        const COPY_MAPPING = 6;
        const SET_STATE = 7;
        const SET_TRAP_HANDLER = 8;
    }
}

bitflags! {
    pub struct ResultCode: usize {
        const OK = 0;
        const UNIMPLEMENTED = 2;
        const INVALID_ARGUMENT = 3;
        const PERMISSION_DENIED = 4;
        const NO_FRAME = 5;
        const NO_PAGE = 6;
        const NO_PROCESS = 7;
        const NO_PROCESS_SLOT = 8;
        const UNEXPECTED = 9;
    }
}

impl From<ResultCode> for Result<()> {
    fn from(result: ResultCode) -> Result<()> {
        match result {
            ResultCode::OK => Ok(()),
            ResultCode::NO_FRAME => Err(Error::NoFrame),
            _ => Err(Error::InvalidArgument),
        }
    }
}
