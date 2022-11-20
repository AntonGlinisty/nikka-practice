pub mod mini_context;
pub mod pid;
pub mod registers;
pub mod syscall;
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

pub fn pid() -> Pid {
    super::process_info().pid()
}
