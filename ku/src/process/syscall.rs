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
        const SCHED_YIELD = 1;
        const MAP = 2;
        const UNMAP = 3;
        const COPY_MAPPING = 4;
        const EXOFORK = 5;
        const SET_STATE = 6;
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
