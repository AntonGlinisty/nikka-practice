use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::registers::control::Cr2;

use ku::{
    memory::{
        PageFaultInfo,
        Virt,
        size,
    },
    process::{
        Info,
        Pid,
    },
};

use super::{
    TRAP_STATS,
    TrapContext,
};

use crate::{
    log::{
        debug,
        error,
        info,
    },
    process::{
        ModeContext,
        Process,
        Table,
    },
    smp::Cpu,
};

#[derive(PartialEq)]
enum Fatal {
    Yes,
    No,
}

fn generic_trap(
    context: &mut TrapContext,
    number: usize,
    info: Info,
    fatal: Fatal,
) {
    debug!(trap = TRAP_STATS[number].name(), %context, %info);

    TRAP_STATS[number].inc();

    if context.is_user_mode() {
        let pid = Cpu::current_process().expect("user mode without a process");
        if pid == Pid::Current {
            context.set(ModeContext::kernel_context(Virt::default()));
            return;
        }
        let mut process =
            Table::get(pid).expect("failed to find the current process in the process table");

        if process.trap(context, number, info) {
            return;
        }

        info!(
            trap = TRAP_STATS[number].name(),
            number,
            %info,
            %context,
            %pid,
            "user mode trap",
        );

        if fatal == Fatal::Yes {
            Table::free(process);
            Process::sched_yield();
        }
    } else {
        if fatal == Fatal::Yes {
            panic!(
                "kernel mode trap #{} - {}, context: {}, info: {}",
                number,
                TRAP_STATS[number].name(),
                context,
                info,
            );
        } else {
            error!(
                trap = TRAP_STATS[number].name(),
                number,
                %info,
                %context,
                "kernel mode trap",
            );
        }
    }
}

pub(super) extern "x86-interrupt" fn divide_error(mut context: TrapContext) {
    generic_trap(&mut context, super::DIVIDE_ERROR, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn debug(mut context: TrapContext) {
    generic_trap(&mut context, super::DEBUG, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn non_maskable_interrupt(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::NON_MASKABLE_INTERRUPT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn breakpoint(mut context: TrapContext) {
    generic_trap(&mut context, super::BREAKPOINT, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn overflow(mut context: TrapContext) {
    generic_trap(&mut context, super::OVERFLOW, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn bound_range_exceeded(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::BOUND_RANGE_EXCEEDED,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn invalid_opcode(mut context: TrapContext) {
    generic_trap(&mut context, super::INVALID_OPCODE, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn device_not_available(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::DEVICE_NOT_AVAILABLE,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn double_fault(
    mut context: TrapContext,
    error_code: u64,
) {
    lazy_static! {
        static ref STOP_ALL_CPUS: Mutex<()> = Mutex::new(());
    }

    let _ = STOP_ALL_CPUS.lock();

    generic_trap(
        &mut context,
        super::DOUBLE_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
    unreachable!();
}

pub(super) extern "x86-interrupt" fn invalid_tss(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::INVALID_TSS,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn segment_not_present(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::SEGMENT_NOT_PRESENT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn stack_segment_fault(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::STACK_SEGMENT_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn general_protection_fault(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::GENERAL_PROTECTION_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn page_fault(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::PAGE_FAULT,
        Info::PageFault {
            address: Cr2::read().into(),
            code: PageFaultInfo::from_bits_truncate(size::from(error_code)),
        },
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn x87_floating_point(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::X87_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn alignment_check(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::ALIGNMENT_CHECK,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn machine_check(mut context: TrapContext) {
    generic_trap(&mut context, super::MACHINE_CHECK, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn simd_floating_point(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::SIMD_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn virtualization(mut context: TrapContext) {
    generic_trap(&mut context, super::VIRTUALIZATION, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn security_exception(
    mut context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &mut context,
        super::SECURITY_EXCEPTION,
        Info::Code(error_code),
        Fatal::Yes,
    );
}
