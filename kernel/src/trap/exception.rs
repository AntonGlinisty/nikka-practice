use core::{
    arch::asm,
    mem,
};

use sentinel_frame::with_sentinel_frame;

use ku::{
    backtrace::Backtrace,
    memory::Virt,
    process::{
        Info,
        Pid,
        Trap,
    },
    sync::{
        self,
        Spinlock,
    },
};

use super::{
    TRAP_STATS,
    TrapContext,
};

use crate::{
    fs::BlockCache,
    log::{
        error,
        info,
        warn,
    },
    process::{
        ModeContext,
        Process,
        Table,
    },
    smp::Cpu,
};

/// A helper to generate the code for exception handlers.
macro_rules! exception_with_error_code {
    ($name:ident, $trap:expr $(,)?) => {
        #[naked]
        extern "C" fn $name() {
            unsafe {
                core::arch::asm!(
                    "
                push {number}
                jmp {trap_trampoline}
                    ",

                    number = const $trap as usize,
                    trap_trampoline = sym crate::trap::handler::trap_trampoline,

                    options(noreturn),
                )
            }
        }
    };

    ($idt: ident, $name:ident, $trap:expr $(,)?) => {
        exception_with_error_code!($name, $trap);
        $idt.get_mut($trap).set_trap_handler($name);
    };

    ($idt: ident, $name:ident, $trap:expr, $stack:expr $(,)?) => {
        exception_with_error_code!($name, $trap);
        $idt.get_mut($trap)
            .set_trap_handler($name)
            .set_stack_index($stack);
    };
}

pub(super) use exception_with_error_code;

/// A helper to generate the code for exception handlers.
macro_rules! exception_without_error_code {
    ($name:ident, $trap:expr $(,)?) => {
        #[naked]
        extern "C" fn $name() {
            unsafe {
                core::arch::asm!(
                    "
                push {error_code}
                push {number}
                jmp {trap_trampoline}
                    ",

                    error_code = const 0,
                    number = const $trap as usize,
                    trap_trampoline = sym crate::trap::handler::trap_trampoline,

                    options(noreturn),
                )
            }
        }
    };

    ($idt: ident, $name:ident, $trap:expr $(,)?) => {
        {
            exception_without_error_code!($name, $trap);
            $idt.get_mut($trap).set_trap_handler($name);
        }
    };
}

pub(super) use exception_without_error_code;

/// Общий обработчик исключений и прерываний.
///
/// - `number` --- номер прерывания, также определяет фатальность исключения.
/// - `error_code` --- дополнительная информация для некоторых исключений процессора.
/// - `context` --- контекст в котором возникло прерывание.
/// - `rpb` --- значение регистра `rbp` в контексте прерывания для построения [`Backtrace`].
///
/// Если фатальное исключение вызвал процесс, то он будет остановлен и удалён.
/// Если фатальное исключение вызвало ядро, оно запаникует.
#[cfg_attr(not(feature = "conservative-backtraces"), with_sentinel_frame)]
extern "C" fn generic_trap(
    number: usize,             // rdi
    error_code: usize,         // rsi
    context: &mut TrapContext, // rdx
    rbp: usize,                // rcx
) {
    let trap = Trap::try_from(number).expect("unexpected trap number");

    if trap == Trap::DoubleFault {
        static STOP_ALL_CPUS: Spinlock<()> = Spinlock::new(());

        mem::forget(STOP_ALL_CPUS.lock());
        sync::start_panicking();
    }

    TRAP_STATS[trap].inc();

    let fatal = trap != Trap::Breakpoint && trap != Trap::Overflow;
    let info = Info::new(trap, error_code);

    if context.is_user_mode() {
        let pid = Cpu::current_process().expect("user mode without a process");
        if pid == Pid::Current {
            context.set(ModeContext::kernel_context(Virt::default()));
            return;
        }
        let mut process =
            Table::get(pid).expect("failed to find the current process in the process table");

        if process.trap(context, trap, info) {
            return;
        }

        info!(
            trap = TRAP_STATS[trap].name(),
            number,
            %info,
            %context,
            %pid,
            "user mode trap",
        );

        if fatal {
            drop(process);
            if let Err(error) = Table::free(pid) {
                warn!(
                    %pid,
                    ?error,
                    "failed to free the process, maybe it was destroyed concurrently",
                );
            }
            Process::sched_yield();
        }
    } else {
        match BlockCache::trap_handler(&info) {
            Ok(true) => return,
            Ok(false) => {},
            Err(error) => error!(?error, "failed to handle a page fault in the block cache"),
        }

        let rsp = context.get().mini_context().rsp().into_usize();
        let backtrace = Backtrace::new(rbp, rsp).unwrap_or_default();

        if fatal {
            panic!(
                "kernel mode trap #{} - {}, context: {}, info: {}, backtrace: {}",
                number,
                TRAP_STATS[trap].name(),
                context,
                info,
                backtrace,
            );
        } else {
            error!(
                trap = TRAP_STATS[trap].name(),
                number,
                %context,
                %info,
                %backtrace,
                "kernel mode trap",
            );
        }
    }
}

/// Общий трамплин для всех исключений процессора.
#[naked]
pub(super) extern "C" fn trap_trampoline() {
    unsafe {
        asm!(
            "
        // System V ABI for x86-64 caller-saved registers.
        push rax
        push rcx
        push rdx
        push rsi
        push rdi
        push r8
        push r9
        push r10
        push r11

        // The trap number is right above the caller-saved registers.
        mov rdi, [rsp + {caller_saved_registers_size}]

        // Next is the error code.
        mov rsi, [rsp + {caller_saved_registers_size} + {trap_number_size}]

        // Next is the interrupt context.
        lea rdx, [rsp + {caller_saved_registers_size} + {trap_number_size} + {error_code_size}]

        mov rcx, rbp

        call {generic_trap}

        pop r11
        pop r10
        pop r9
        pop r8
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rax

        add rsp, {trap_number_size} + {error_code_size}

        iretq
            ",

            caller_saved_registers_size = const 9 * mem::size_of::<usize>(),
            error_code_size = const mem::size_of::<usize>(),
            generic_trap = sym generic_trap,
            trap_number_size = const mem::size_of::<usize>(),

            options(noreturn),
        )
    }
}
