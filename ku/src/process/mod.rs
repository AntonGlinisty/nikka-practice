pub mod mini_context;
pub mod pid;
pub mod registers;
pub mod syscall;
pub mod trap_info;

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

pub fn pid() -> Pid {
    super::process_info().pid()
}
