#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

/// Минимальная информация о контексте исполнения.
pub mod mini_context;

/// Идентификатор процесса.
pub mod pid;

/// Операции с [регистром флагов](https://en.wikipedia.org/wiki/FLAGS_register).
pub mod registers;

/// Константы для работы с системными вызовами.
pub mod syscall;

/// Информация об исключении процессора.
pub mod trap_info;

use bitflags::bitflags;

pub use mini_context::MiniContext;
pub use pid::Pid;
pub use registers::RFlags;
pub use syscall::{
    ExitCode,
    ResultCode,
    Syscall,
    SyscallResult,
};
pub use trap_info::{
    Info,
    RSP_OFFSET_IN_TRAP_INFO,
    TrapInfo,
};

bitflags! {
    /// Состояние пользовательского процесса.
    pub struct State: usize {
        /// Процесс только что создан и не готов к запуску.
        const EXOFORK = 0;

        /// Процесс готов к вытеснению, но не выполняется в данный момент.
        const RUNNABLE = 1;

        /// Процесс выполняется в данный момент.
        const RUNNING = 2;
    }
}
