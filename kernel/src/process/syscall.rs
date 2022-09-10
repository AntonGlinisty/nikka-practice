use core::{
    arch::asm,
    str,
};

use bitflags::bitflags;
use spin::MutexGuard;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
    LStar,
    Star,
};

use ku::process::{
    ExitCode,
    MiniContext,
    Syscall,
};

use crate::{
    error::{
        Error,
        Error::{
            InvalidAlignment,
            InvalidArgument,
            NoPage,
            NoProcess,
            PermissionDenied,
        },
        Result,
    },
    gdt::Gdt,
    log,
    log::{
        debug,
        error,
        info,
        trace,
        warn,
    },
    memory::{
        Block,
        FRAME_ALLOCATOR,
        Frame,
        Page,
        USER,
        Virt,
        mmu::PageTableFlags,
    },
    smp::{
        Cpu,
        KERNEL_RSP_OFFSET_IN_CPU,
    },
};

use super::{
    Pid,
    Process,
    RFlags,
    Scheduler,
    State,
    Table,
    TrapHandlerContext,
};

pub(crate) fn init() {
    // TODO: your code here.
    unimplemented!();
}

#[derive(Clone, Copy, Debug)]
struct SyscallResult(usize);

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
                Error::Postcard(_) => ResultCode::UNEXPECTED,
                Error::Pipe(_) => ResultCode::UNEXPECTED,
                Error::Unimplemented => ResultCode::UNIMPLEMENTED,
                Error::InvalidAlignment => ResultCode::INVALID_ALIGNMENT,
            },
        }
    }
}

#[naked]
extern "C" fn syscall_trampoline() -> ! {
    unsafe {
        asm!(
            "
            // TODO: your code here.

            call {syscall}
            ",

            // TODO: your code here.
            syscall = sym syscall,
            options(noreturn),
        );
    }
}

extern "C" fn syscall(
    // https://wiki.osdev.org/System_V_ABI#x86-64:
    // Parameters to functions are passed in the registers rdi, rsi, rdx, rcx, r8, r9, and further values are passed on the stack in reverse order.
    number: usize, // rdi
    arg0: usize,   // rsi
    arg1: usize,   // rdx
    rip: Virt,     // rcx
    rsp: Virt,     // r8
    arg2: usize,   // r9
    // Stack, push in reverse order.
    arg3: usize,
    arg4: usize,
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

fn sysret(
    context: MiniContext,
    result: Result<SyscallResult>,
) -> ! {
    // TODO: your code here.

    unsafe {
        asm!(
            "
            // TODO: your code here.

            sysretq
            ",

            // TODO: your code here.

            options(noreturn),
        );
    }
}

fn exit(
    process: MutexGuard<Process>,
    code: usize,
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

#[allow(unused_mut)] // TODO: remove before flight.
fn log_value(
    mut process: MutexGuard<Process>,
    start: usize,
    len: usize,
    value: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn sched_yield(process: MutexGuard<Process>) -> ! {
    // TODO: your code here.
    unimplemented!();
}

fn exofork(process: MutexGuard<Process>) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn map(
    process: MutexGuard<Process>,
    dst_pid: usize,
    dst_address: usize,
    dst_size: usize,
    flags: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn unmap(
    process: MutexGuard<Process>,
    dst_pid: usize,
    dst_address: usize,
    dst_size: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn copy_mapping(
    process: MutexGuard<Process>,
    dst_pid: usize,
    src_address: usize,
    dst_address: usize,
    dst_size: usize,
    flags: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn set_state(
    process: MutexGuard<Process>,
    dst_pid: usize,
    state: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn set_trap_handler(
    process: MutexGuard<Process>,
    dst_pid: usize,
    rip: usize,
    stack_address: usize,
    stack_size: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

fn check_block(
    address: usize,
    size: usize,
) -> Result<Block<Page>> {
    // TODO: your code here.
    unimplemented!();
}

fn check_frame(
    process: &mut MutexGuard<Process>,
    page: Page,
    flags: PageTableFlags,
) -> Result<Frame> {
    // TODO: your code here.
    unimplemented!();
}

fn check_page_flags(flags: usize) -> Result<PageTableFlags> {
    // TODO: your code here.
    unimplemented!();
}

fn check_process_permissions(
    process: MutexGuard<Process>,
    dst_pid: usize,
) -> Result<MutexGuard<Process>> {
    // TODO: your code here.
    unimplemented!();
}
