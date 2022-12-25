use crate::error::{
    Error,
    Result,
};

use bitflags::bitflags;

bitflags! {
    /// Код выхода пользовательской программы, передаваемый в `syscall::exit()`.
    pub struct ExitCode: usize {
        /// Пользовательская программа завершилась успешно.
        const OK = 0;

        /// Пользовательская программа запаниковала.
        const PANIC = 1;

        /// Пользовательская программа выполнила несуществующий системный вызов.
        const UNIMPLEMENTED_SYSCALL = 2;
    }
}

bitflags! {
    /// Номера системных вызовов.
    pub struct Syscall: usize {
        /// Номер системного вызова `exit()`.
        const EXIT = 0;

        /// Номер системного вызова `log_value()`.
        const LOG_VALUE = 1;

        /// Номер системного вызова `sched_yield()`.
        const SCHED_YIELD = 2;

        /// Номер системного вызова `sched_exofork()`.
        const EXOFORK = 3;

        /// Номер системного вызова `map()`.
        const MAP = 4;

        /// Номер системного вызова `unmap()`.
        const UNMAP = 5;

        /// Номер системного вызова `copy_mapping()`.
        const COPY_MAPPING = 6;

        /// Номер системного вызова `set_state()`.
        const SET_STATE = 7;

        /// Номер системного вызова `set_trap_handler()`.
        const SET_TRAP_HANDLER = 8;
    }
}

/// Данные, возвращаемые из системных вызовов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyscallResult(pub usize);

bitflags! {
    /// Код ошибки, возвращаемый из системных вызовов.
    pub struct ResultCode: usize {
        /// Код для [`Result::Ok`].
        const OK = 0;

        /// Код для всех ошибок, которые не должны возвращаться из системных вызовов.
        const UNEXPECTED = 1;

        /// Код для [`Error::InvalidArgument`].
        const INVALID_ARGUMENT = 2;

        /// Код для [`Error::NoFrame`].
        const NO_FRAME = 3;

        /// Код для [`Error::NoPage`].
        const NO_PAGE = 4;

        /// Код для [`Error::NoProcess`].
        const NO_PROCESS = 5;

        /// Код для [`Error::NoProcessSlot`].
        const NO_PROCESS_SLOT = 6;

        /// Код для [`Error::Null`].
        const NULL = 7;

        /// Код для [`Error::Overflow`].
        const OVERFLOW = 8;

        /// Код для [`Error::PermissionDenied`].
        const PERMISSION_DENIED = 9;

        /// Код для [`Error::Unimplemented`].
        const UNIMPLEMENTED = 10;

        /// Код для [`Error::InvalidAlignment`].
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
