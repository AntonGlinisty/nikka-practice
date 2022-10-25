use core::{
    arch::asm,
    str,
};

use spin::MutexGuard;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
    LStar,
    SFMask,
    Star,
};

use ku::process::{
    ExitCode,
    MiniContext,
    RFlags,
    ResultCode,
    Syscall,
    SyscallResult,
};

use crate::{
    error::{
        Error::{
            InvalidAlignment,
            InvalidArgument,
            NoPage,
            NoProcess,
            Overflow,
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
    Scheduler,
    State,
    Table,
    TrapHandlerContext,
};

/// Инициализация системных вызовов.
/// Подготавливает процессор к выполнению инструкций
/// [syscall](https://www.felixcloutier.com/x86/syscall) и
/// [sysret](https://www.felixcloutier.com/x86/sysret).
pub(crate) fn init() {
    // TODO: your code here.
    unimplemented!();
}

/// Получает управление при выполнении инструкции
/// [syscall](https://www.felixcloutier.com/x86/syscall).
/// Переключает стек на стек ядра, разрешает прерывания и
/// передаёт управление в функцию [`syscall()`].
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

/// Выполняет диспетчеризацию системных вызовов по аргументу `number` --- номеру системного вызова.
/// Передаёт в функции, реализующие конкретные системные вызовы,
/// нужную часть аргументов `arg0`–`arg4`.
/// После выполнения функции конкретного системного вызова,
/// с помощью функции [`sysret()`] возвращает управление в контекст пользователя,
/// задаваемый `rip` и `rsp`.
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
    assert!(
        RFlags::read().contains(RFlags::INTERRUPT_FLAG),
        "enable the interrupts during the system calls",
    );

    // TODO: your code here.
    unimplemented!();
}

/// С помощью инструкции [sysret](https://www.felixcloutier.com/x86/sysret)
/// возвращает управление в контекст пользователя `context`.
/// Передаёт пользователю результат системного вызова в виде кода успеха или ошибки `result` и
/// полезной нагрузки `res`.
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

/// Выполняет системный вызов `syscall::exit(code)`.
/// Освобождает слот таблицы процессов и возвращается в контекст ядра,
/// из которого пользовательский процесс был запущен.
fn exit(
    process: MutexGuard<Process>,
    code: usize,
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов `syscall::log_value(string, value)`.
/// Записывает в журнал строку типа `&str`, заданную началом `start` и длиной `len`, а также число `value`.
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
