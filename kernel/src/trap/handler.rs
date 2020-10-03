use core::fmt::{
    Debug,
    Formatter,
    Result,
};

use x86_64::structures::idt::{
    InterruptStackFrame,
    InterruptStackFrameValue,
    PageFaultErrorCode,
};

use super::TRAP_STATS;

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

pub(super) extern "x86-interrupt" fn divide_error(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::DIVIDE_ERROR].inc();
    panic!("Divide error\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn debug(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::DEBUG].inc();
    panic!("Debug trap or fault\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::NON_MASKABLE_INTERRUPT].inc();
    panic!(
        "Non-maskable interrupt - probable hardware failure\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn breakpoint(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::BREAKPOINT].inc();
    use core::fmt::Write;
    use text::println;
    println!("breakpoint\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn overflow(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::OVERFLOW].inc();
}

pub(super) extern "x86-interrupt" fn bound_range_exceeded(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::BOUND_RANGE_EXCEEDED].inc();
    panic!("Bound range exceeded\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn invalid_opcode(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::INVALID_OPCODE].inc();
    panic!(
        "Invalid opcode - the code is compiled for a different CPU model\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn device_not_available(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::DEVICE_NOT_AVAILABLE].inc();
    panic!("Device not available\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn double_fault(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    TRAP_STATS[super::DOUBLE_FAULT].inc();
    panic!("Double fault\n{:?}", StackFrame::new(stack_frame))
}

pub(super) extern "x86-interrupt" fn invalid_tss(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::INVALID_TSS].inc();
    panic!("Invalid TSS\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn segment_not_present(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::SEGMENT_NOT_PRESENT].inc();
    panic!("Segment not present\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn stack_segment_fault(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::STACK_SEGMENT_FAULT].inc();
    panic!("Stack segment fault\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn general_protection_fault(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::GENERAL_PROTECTION_FAULT].inc();
    panic!(
        "General protection fault\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn page_fault(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    TRAP_STATS[super::PAGE_FAULT].inc();

    panic!(
        "Page fault in {} mode on {} {:?}: {}\n  error code: {:?}\n{:?}",
        if error_code.contains(PageFaultErrorCode::USER_MODE) {
            "user"
        } else {
            "kernel"
        },
        if error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) {
            "write to"
        } else {
            "read from"
        },
        Cr2::read(),
        if error_code.contains(PageFaultErrorCode::MALFORMED_TABLE) {
            "malformed page table (1-bit in a reserved field)"
        } else if error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            "protection violation"
        } else {
            "page is not present"
        },
        error_code,
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn x87_floating_point(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::X87_FLOATING_POINT].inc();
    panic!(
        "x87 floating point exception\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn alignment_check(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::ALIGNMENT_CHECK].inc();
    panic!("Alignment check\n{:?}", StackFrame::new(stack_frame));
}

pub(super) extern "x86-interrupt" fn machine_check(stack_frame: &mut InterruptStackFrame) -> ! {
    TRAP_STATS[super::MACHINE_CHECK].inc();
    panic!("Machine check\n{:?}", StackFrame::new(stack_frame))
}

pub(super) extern "x86-interrupt" fn simd_floating_point(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::SIMD_FLOATING_POINT].inc();
    panic!(
        "SIMD floating point exception\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn virtualization(stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::VIRTUALIZATION].inc();
    panic!(
        "Virtualization exception\n{:?}",
        StackFrame::new(stack_frame),
    );
}

pub(super) extern "x86-interrupt" fn security_exception(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    TRAP_STATS[super::SECURITY_EXCEPTION].inc();
    panic!("Security exception\n{:?}", StackFrame::new(stack_frame));
}
