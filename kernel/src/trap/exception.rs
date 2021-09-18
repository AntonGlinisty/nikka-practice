use core::fmt::{
    Debug,
    Display,
    Formatter,
    Result,
};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    registers::{
        control::Cr2,
        rflags::RFlags,
    },
    structures::idt::{
        InterruptStackFrame,
        InterruptStackFrameValue,
        PageFaultErrorCode,
    },
};

use super::TRAP_STATS;

use crate::{
    log::{
        error,
        info,
    },
    memory::Virt,
    process::Process,
    smp::Cpu,
};

struct StackFrame<'frame> {
    frame: &'frame InterruptStackFrameValue,
}

impl<'frame> StackFrame<'frame> {
    fn new(stack_frame: &'frame InterruptStackFrame) -> Self {
        Self { frame: stack_frame }
    }
}

impl<'frame> Debug for StackFrame<'frame> {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        const CPL_MASK: u64 = 0x3;
        const IOPL_MASK: u64 = CPL_MASK;
        const IOPL_SHIFT: u64 = 12;

        write!(
            formatter,
            concat!(
                "  cs:rip = {:#06X}:{:#018X}\t",
                "ss:rsp = {:#06X}:{:#018X}\n",
                "  privilege level = {}\t\t\t",
                "IO privilege level = {}\n",
                "  flags = {:#X} = ",
            ),
            self.frame.code_segment,
            self.frame.instruction_pointer.as_u64(),
            self.frame.stack_segment,
            self.frame.stack_pointer.as_u64(),
            self.frame.code_segment & CPL_MASK,
            (self.frame.code_segment >> IOPL_SHIFT) & IOPL_MASK,
            self.frame.cpu_flags,
        )?;

        static FLAGS: [&str; 22] = [
            "CF",
            "",
            "PF",
            "",
            "AF",
            "",
            "ZF",
            "SF",
            "TF",
            "IF",
            "DF",
            "OF",
            "",
            "",
            "NT",
            "",
            "RF",
            "VM",
            "AC",
            "VF",
            "VP",
            "ID",
        ];

        let mut separator = "";

        for i in (0 .. FLAGS.len()).rev() {
            if (self.frame.cpu_flags >> i) & 1 != 0 && !FLAGS[i].is_empty() {
                write!(formatter, "{}{}", separator, FLAGS[i])?;
                separator = " ";
            }
        }

        Ok(())
    }
}

const CPL_MASK: u64 = 0x3;
const RING_0: u64 = 0x0;

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
    stack_frame: &InterruptStackFrame,
    exception_number: usize,
    info: Info,
    fatal: Fatal,
) {
    TRAP_STATS[exception_number].inc();

    if stack_frame.code_segment & CPL_MASK != RING_0 {
        let process =
            Cpu::current_process().expect("running in the user mode without a current process");

        info!(
            trap = TRAP_STATS[exception_number].name(),
            exception_number,
            %info,
            cs = format_args!("{:#06X}", stack_frame.code_segment),
            rip = %Virt::from(stack_frame.instruction_pointer),
            ss = format_args!("{:#06X}", stack_frame.stack_segment),
            rsp = %Virt::from(stack_frame.stack_pointer),
            rflags = ?RFlags::from_bits_truncate(stack_frame.cpu_flags),
            %process,
            "user mode trap",
        );

        if fatal == Fatal::Yes {
            Process::leave_user_mode();
        }
    } else {
        error!(
            trap = TRAP_STATS[exception_number].name(),
            exception_number,
            %info,
            cs = format_args!("{:#06X}", stack_frame.code_segment),
            rip = %Virt::from(stack_frame.instruction_pointer),
            ss = format_args!("{:#06X}", stack_frame.stack_segment),
            rsp = %Virt::from(stack_frame.stack_pointer),
            rflags = ?RFlags::from_bits_truncate(stack_frame.cpu_flags),
            "kernel mode trap",
        );

        if fatal == Fatal::Yes {
            panic!(
                "kernel mode trap #{} - {}, context: {:?}",
                exception_number,
                TRAP_STATS[exception_number].name(),
                StackFrame::new(stack_frame),
            );
        }
    }
}

pub(super) extern "x86-interrupt" fn divide_error(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::DIVIDE_ERROR, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn debug(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::DEBUG, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: InterruptStackFrame) {
    generic_trap(
        &stack_frame,
        super::NON_MASKABLE_INTERRUPT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::BREAKPOINT, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::OVERFLOW, Info::None, Fatal::No);
}

pub(super) extern "x86-interrupt" fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
    generic_trap(
        &stack_frame,
        super::BOUND_RANGE_EXCEEDED,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::INVALID_OPCODE, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn device_not_available(stack_frame: InterruptStackFrame) {
    generic_trap(
        &stack_frame,
        super::DEVICE_NOT_AVAILABLE,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn double_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    lazy_static! {
        static ref STOP_ALL_CPUS: Mutex<()> = Mutex::new(());
    }

    let _ = STOP_ALL_CPUS.lock();

    generic_trap(
        &stack_frame,
        super::DOUBLE_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
    unreachable!();
}

pub(super) extern "x86-interrupt" fn invalid_tss(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::INVALID_TSS,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn segment_not_present(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::SEGMENT_NOT_PRESENT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn stack_segment_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::STACK_SEGMENT_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn general_protection_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::GENERAL_PROTECTION_FAULT,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    generic_trap(
        &stack_frame,
        super::PAGE_FAULT,
        Info::PageFault {
            address: Cr2::read().into(),
            code: error_code,
        },
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn x87_floating_point(stack_frame: InterruptStackFrame) {
    generic_trap(
        &stack_frame,
        super::X87_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn alignment_check(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::ALIGNMENT_CHECK,
        Info::Code(error_code),
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn machine_check(stack_frame: InterruptStackFrame) -> ! {
    generic_trap(&stack_frame, super::MACHINE_CHECK, Info::None, Fatal::Yes);
    unreachable!();
}

pub(super) extern "x86-interrupt" fn simd_floating_point(stack_frame: InterruptStackFrame) {
    generic_trap(
        &stack_frame,
        super::SIMD_FLOATING_POINT,
        Info::None,
        Fatal::Yes,
    );
}

pub(super) extern "x86-interrupt" fn virtualization(stack_frame: InterruptStackFrame) {
    generic_trap(&stack_frame, super::VIRTUALIZATION, Info::None, Fatal::Yes);
}

pub(super) extern "x86-interrupt" fn security_exception(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    generic_trap(
        &stack_frame,
        super::SECURITY_EXCEPTION,
        Info::Code(error_code),
        Fatal::Yes,
    );
}
