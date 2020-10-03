use x86_64::structures::idt::InterruptStackFrame;

use super::TRAP_STATS;

pub(super) extern "x86-interrupt" fn pit(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::PIT].inc();
}

pub(super) extern "x86-interrupt" fn keyboard(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::KEYBOARD].inc();
}

pub(super) extern "x86-interrupt" fn cascade(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::CASCADE].inc();
}

pub(super) extern "x86-interrupt" fn com2(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::COM2].inc();
}

pub(super) extern "x86-interrupt" fn com1(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::COM1].inc();
}

pub(super) extern "x86-interrupt" fn lpt2(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::LPT2].inc();
}

pub(super) extern "x86-interrupt" fn floppy_disk(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::FLOPPY_DISK].inc();
}

pub(super) extern "x86-interrupt" fn lpt1(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::LPT1].inc();
}

pub(super) extern "x86-interrupt" fn rtc(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::RTC].inc();
}

pub(super) extern "x86-interrupt" fn free_29(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::FREE_29].inc();
}

pub(super) extern "x86-interrupt" fn free_2a(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::FREE_2A].inc();
}

pub(super) extern "x86-interrupt" fn free_2b(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::FREE_2B].inc();
}

pub(super) extern "x86-interrupt" fn ps2_mouse(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::PS2_MOUSE].inc();
}

pub(super) extern "x86-interrupt" fn coprocessor(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::COPROCESSOR].inc();
}

pub(super) extern "x86-interrupt" fn ata0(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::ATA0].inc();
}

pub(super) extern "x86-interrupt" fn ata1(_stack_frame: &mut InterruptStackFrame) {
    TRAP_STATS[super::ATA1].inc();
}
