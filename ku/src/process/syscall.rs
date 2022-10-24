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

#[derive(Clone, Copy, Debug)]
pub struct SyscallResult(pub usize);

bitflags! {
    pub struct ResultCode: usize {
        const OK = 0;
        const INVALID_ARGUMENT = 1;
        const NO_FRAME = 2;
        const NO_PAGE = 3;
        const NO_PROCESS = 4;
        const NO_PROCESS_SLOT = 5;
        const NULL = 6;
        const OVERFLOW = 7;
        const PERMISSION_DENIED = 8;
        const UNEXPECTED = 9;
        const UNIMPLEMENTED = 10;
        const INVALID_ALIGNMENT = 11;
    }
}

impl From<ResultCode> for Result<()> {
    fn from(result: ResultCode) -> Result<()> {
        match result {
            ResultCode::OK => Ok(()),

            ResultCode::INVALID_ARGUMENT => Err(Error::InvalidArgument),
            ResultCode::NO_FRAME => Err(Error::NoFrame),
            ResultCode::NO_PAGE => Err(Error::NoPage),
            ResultCode::NO_PROCESS => Err(Error::NoProcess),
            ResultCode::NO_PROCESS_SLOT => Err(Error::NoProcessSlot),
            ResultCode::NULL => Err(Error::Null),
            ResultCode::OVERFLOW => Err(Error::Overflow),
            ResultCode::PERMISSION_DENIED => Err(Error::PermissionDenied),
            ResultCode::UNIMPLEMENTED => Err(Error::Unimplemented),
            ResultCode::INVALID_ALIGNMENT => Err(Error::InvalidAlignment),

            _ => panic!("unexpected error"),
        }
    }
}

impl<T> From<Result<T>> for ResultCode {
    fn from(result: Result<T>) -> ResultCode {
        match result {
            Ok(_) => ResultCode::OK,

            Err(error) => match error {
                Error::Elf(_) => ResultCode::UNEXPECTED,
                Error::Fmt(_) => ResultCode::UNEXPECTED,
                Error::Int(_) => ResultCode::UNEXPECTED,
                Error::InvalidArgument => ResultCode::INVALID_ARGUMENT,
                Error::NoFrame => ResultCode::NO_FRAME,
                Error::NoPage => ResultCode::NO_PAGE,
                Error::NoProcess => ResultCode::NO_PROCESS,
                Error::NoProcessSlot => ResultCode::NO_PROCESS_SLOT,
                Error::Null => ResultCode::NULL,
                Error::Overflow => ResultCode::OVERFLOW,
                Error::PermissionDenied => ResultCode::PERMISSION_DENIED,
                Error::Pipe(_) => ResultCode::UNEXPECTED,
                Error::Postcard(_) => ResultCode::UNEXPECTED,
                Error::Unimplemented => ResultCode::UNIMPLEMENTED,
                Error::InvalidAlignment => ResultCode::INVALID_ALIGNMENT,

                _ => ResultCode::UNEXPECTED,
            },
        }
    }
}
