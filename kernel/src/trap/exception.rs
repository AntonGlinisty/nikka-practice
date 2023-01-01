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
    fs::BlockCache,
    log::{
        error,
        info,
        trace,
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

/// Общий обработчик исключений и прерываний.
///
/// - `context` --- контекст в котором возникло прерывание.
/// - `number` --- номер прерывания.
/// - `info` --- дополнительная информация для некоторых исключений процессора.
/// - `fatal` --- фатально ли исключение.
/// Если фатальное исключение вызвал процесс, то он будет остановлен и удалён.
/// Если фатальное исключение вызвало ядро, оно запаникует.
fn generic_trap(
    context: &mut TrapContext,
    number: usize,
    info: Info,
    fatal: Fatal,
) {
    trace!(trap = TRAP_STATS[number].name(), %context, %info);

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
        match BlockCache::trap_handler(&info) {
            Ok(true) => return,
            Ok(false) => {},
            Err(error) => error!(?error, "failed to handle a page fault in the block cache"),
        }

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

/// [Exceptions: Division Error](https://wiki.osdev.org/Exception#Division_Error)
pub(super) extern "x86-interrupt" fn divide_error(mut context: TrapContext) {
    generic_trap(&mut context, super::DIVIDE_ERROR, Info::None, Fatal::Yes);
}

/// [Exceptions: Debug](https://wiki.osdev.org/Exception#Debug)
pub(super) extern "x86-interrupt" fn debug(mut context: TrapContext) {
    generic_trap(&mut context, super::DEBUG, Info::None, Fatal::Yes);
}

/// [Non-maskable interrupt](https://en.wikipedia.org/wiki/Non-maskable_interrupt),
/// [Exceptions: Non-maskable interrupt](https://wiki.osdev.org/Non_Maskable_Interrupt)
pub(super) extern "x86-interrupt" fn non_maskable_interrupt(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::NON_MASKABLE_INTERRUPT,
        Info::None,
        Fatal::Yes,
    );
}

/// [Exceptions: Breakpoint](https://wiki.osdev.org/Exception#Breakpoint)
pub(super) extern "x86-interrupt" fn breakpoint(mut context: TrapContext) {
    generic_trap(&mut context, super::BREAKPOINT, Info::None, Fatal::No);
}

/// [Exceptions: Overflow](https://wiki.osdev.org/Exception#Overflow)
pub(super) extern "x86-interrupt" fn overflow(mut context: TrapContext) {
    generic_trap(&mut context, super::OVERFLOW, Info::None, Fatal::No);
}

/// [Exceptions: Bound Range Exceeded](https://wiki.osdev.org/Exception#Bound_Range_Exceeded)
pub(super) extern "x86-interrupt" fn bound_range_exceeded(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::BOUND_RANGE_EXCEEDED,
        Info::None,
        Fatal::Yes,
    );
}

/// [Exceptions: Invalid Opcode](https://wiki.osdev.org/Exception#Invalid_Opcode)
pub(super) extern "x86-interrupt" fn invalid_opcode(mut context: TrapContext) {
    generic_trap(&mut context, super::INVALID_OPCODE, Info::None, Fatal::Yes);
}

/// [Exceptions: Device Not Available](https://wiki.osdev.org/Exception#Device_Not_Available)
pub(super) extern "x86-interrupt" fn device_not_available(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::DEVICE_NOT_AVAILABLE,
        Info::None,
        Fatal::Yes,
    );
}

/// [Double Fault](https://en.wikipedia.org/wiki/Double_fault),
/// [Exceptions: Double Fault](https://wiki.osdev.org/Exception#Double_Fault)
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

/// [Exceptions: Invalid TSS](https://wiki.osdev.org/Exception#Invalid_TSS)
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

/// [Exceptions: Segment Not Present](https://wiki.osdev.org/Exception#Segment_Not_Present)
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

/// [Exceptions: Stack-Segment Fault](https://wiki.osdev.org/Exception#Stack-Segment_Fault)
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

/// [General Protection Fault](https://en.wikipedia.org/wiki/General_protection_fault)
/// [Exceptions: General Protection Fault](https://wiki.osdev.org/Exception#General_Protection_Fault)
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

/// [Page Fault](https://en.wikipedia.org/wiki/Page_fault),
/// [Exceptions: Page Fault](https://wiki.osdev.org/Exception#Page_Fault)
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

/// [Exceptions: x87 Floating-Point Exception](https://wiki.osdev.org/Exception#x87_Floating-Point_Exception)
pub(super) extern "x86-interrupt" fn x87_floating_point(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::X87_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

/// [Exceptions: Alignment Check](https://wiki.osdev.org/Exception#Alignment_Check)
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

/// [Machine-check exception](https://en.wikipedia.org/wiki/Machine-check_exception),
/// [Exceptions: Machine Check](https://wiki.osdev.org/Exception#Machine_Check)
pub(super) extern "x86-interrupt" fn machine_check(mut context: TrapContext) {
    generic_trap(&mut context, super::MACHINE_CHECK, Info::None, Fatal::Yes);
}

/// [Exceptions: SIMD Floating-Point Exception](https://wiki.osdev.org/Exception#SIMD_Floating-Point_Exception)
pub(super) extern "x86-interrupt" fn simd_floating_point(mut context: TrapContext) {
    generic_trap(
        &mut context,
        super::SIMD_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

/// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
pub(super) extern "x86-interrupt" fn virtualization(mut context: TrapContext) {
    generic_trap(&mut context, super::VIRTUALIZATION, Info::None, Fatal::Yes);
}

/// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
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
