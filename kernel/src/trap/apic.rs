use crate::smp::{
    Cpu,
    LocalApic,
};

use super::{
    TRAP_STATS,
    TrapContext,
};

pub(super) extern "x86-interrupt" fn timer(mut context: TrapContext) {
    // TODO: your code here.

    generic_apic_interrupt(super::TIMER);
}

pub(super) extern "x86-interrupt" fn spurious(_context: TrapContext) {
    generic_apic_interrupt(super::SPURIOUS);
}

fn generic_apic_interrupt(number: usize) {
    TRAP_STATS[number].inc();
    LocalApic::end_of_interrupt();
}
