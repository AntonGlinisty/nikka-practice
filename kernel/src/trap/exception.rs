use core::fmt::{
    Display,
    Formatter,
    Result,
};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    registers::control::Cr2,
    structures::idt::PageFaultErrorCode,
};

use super::{
    TRAP_STATS,
    TrapContext,
};

use crate::{
    log,
    log::{
        error,
        info,
        trace,
    },
    memory::Virt,
    process::{
        Process,
        Table,
    },
    smp::Cpu,
};

enum Info {
    None,
    Code(u64),
    PageFault {
        address: Virt,
        code: PageFaultErrorCode,
    },
}

impl Display for Info {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        match self {
            Info::None => write!(formatter, "{{ }}"),
            Info::Code(code) => write!(formatter, "{{ code: {code} }}"),
            Info::PageFault { address, code } =>
                write!(formatter, "{{ address: {}, code: {:?} }}", address, code),
        }
    }
}

#[derive(PartialEq)]
enum Fatal {
    Yes,
    No,
}

fn generic_trap(
    context: &TrapContext,
    exception_number: usize,
    info: Info,
    fatal: Fatal,
) {
    TRAP_STATS[exception_number].inc();

    if let Ok(pid) = Cpu::current_process() {
        let mut process =
            Table::get(pid).expect("failed to find the current process in the process table");

        let log = process.info().log();
        log::user_events(pid, log);
        trace!(read_stats = ?log.read_stats(), write_stats = ?log.write_stats());

        info!(
            trap = TRAP_STATS[exception_number].name(),
            exception_number,
            %info,
            %context,
            %process,
            "{}",
            if context.is_user_mode() {
                "user mode trap"
            } else {
                "kernel mode trap on behalf of the user mode"
            }
        );

        if fatal == Fatal::Yes {
            Table::free(process);
            Process::leave_user_mode();
        }
    } else {
        error!(
            trap = TRAP_STATS[exception_number].name(),
            exception_number,
            %info,
            %context,
            "kernel mode trap",
        );

        if fatal == Fatal::Yes {
            panic!(
                "kernel mode trap #{} - {}, context: {}",
                exception_number,
                TRAP_STATS[exception_number].name(),
                context,
            );
        }
    }
}

pub(super) extern "x86-interrupt" fn divide_error(context: TrapContext) {
    generic_trap(&context, super::DIVIDE_ERROR, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn debug(context: TrapContext) {
    generic_trap(&context, super::DEBUG, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn non_maskable_interrupt(context: TrapContext) {
    generic_trap(
        &context,
        super::NON_MASKABLE_INTERRUPT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn breakpoint(context: TrapContext) {
    generic_trap(&context, super::BREAKPOINT, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn overflow(context: TrapContext) {
    generic_trap(&context, super::OVERFLOW, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn bound_range_exceeded(context: TrapContext) {
    generic_trap(
        &context,
        super::BOUND_RANGE_EXCEEDED,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn invalid_opcode(context: TrapContext) {
    generic_trap(&context, super::INVALID_OPCODE, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn device_not_available(context: TrapContext) {
    generic_trap(
        &context,
        super::DEVICE_NOT_AVAILABLE,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn double_fault(
    context: TrapContext,
    error_code: u64,
) {
    lazy_static! {
        static ref STOP_ALL_CPUS: Mutex<()> = Mutex::new(());
    }

    let _ = STOP_ALL_CPUS.lock();

    generic_trap(
        &context,
        super::DOUBLE_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
    unreachable!();
}

pub(super) extern "x86-interrupt" fn invalid_tss(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::INVALID_TSS,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn segment_not_present(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::SEGMENT_NOT_PRESENT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn stack_segment_fault(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::STACK_SEGMENT_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn general_protection_fault(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::GENERAL_PROTECTION_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn page_fault(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::PAGE_FAULT,
        Info::PageFault {
            address: Cr2::read().into(),
            code: PageFaultErrorCode::from_bits_truncate(error_code),
        },
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn x87_floating_point(context: TrapContext) {
    generic_trap(&context, super::X87_FLOATING_POINT, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn alignment_check(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::ALIGNMENT_CHECK,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn machine_check(context: TrapContext) {
    generic_trap(&context, super::MACHINE_CHECK, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn simd_floating_point(context: TrapContext) {
    generic_trap(&context, super::SIMD_FLOATING_POINT, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn virtualization(context: TrapContext) {
    generic_trap(&context, super::VIRTUALIZATION, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn security_exception(
    context: TrapContext,
    error_code: u64,
) {
    generic_trap(
        &context,
        super::SECURITY_EXCEPTION,
        Info::Code(error_code),
        Fatal::Yes,
    );
}
