use x86_64::structures::idt::InterruptStackFrame;

use crate::smp::LocalApic;

use super::TRAP_STATS;

pub(super) extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    generic_apic_interrupt(super::TIMER);
}

pub(super) extern "x86-interrupt" fn spurious(_stack_frame: InterruptStackFrame) {
    generic_apic_interrupt(super::SPURIOUS);
}

fn generic_apic_interrupt(number: usize) {
    TRAP_STATS[number].inc();
    LocalApic::end_of_interrupt();
}
