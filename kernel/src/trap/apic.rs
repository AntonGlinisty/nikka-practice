use ku::process::Trap;

use crate::{
    process::Process,
    smp::LocalApic,
};

use super::{
    TRAP_STATS,
    TrapContext,
};

/// Обработчик прерывания
/// [таймера APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
pub(super) extern "x86-interrupt" fn timer(mut context: TrapContext) {
    Process::preempt(&mut context);

    generic_apic_interrupt(Trap::Timer);
}

/// Обработчик ложных прерываний
/// ([spurious interrupt](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts))
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
pub(super) extern "x86-interrupt" fn spurious(_context: TrapContext) {
    generic_apic_interrupt(Trap::Spurious);
}

/// Выполняет общую часть обработки для всех прерываний
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
fn generic_apic_interrupt(trap: Trap) {
    TRAP_STATS[trap].inc();
    LocalApic::end_of_interrupt();
}
