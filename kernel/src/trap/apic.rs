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

    generic_apic_interrupt(super::TIMER);
}

/// Обработчик ложных прерываний
/// ([spurious interrupt](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts))
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
pub(super) extern "x86-interrupt" fn spurious(_context: TrapContext) {
    generic_apic_interrupt(super::SPURIOUS);
}

/// Выполняет общую часть обработки для всех прерываний
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
/// Аргумент `number` задаёт номер прерывания в общей нумерации таблицы обработчиков прерываний
/// ([Interrupt descriptor table](https://en.wikipedia.org/wiki/Interrupt_descriptor_table), IDT).
fn generic_apic_interrupt(number: usize) {
    TRAP_STATS[number].inc();
    LocalApic::end_of_interrupt();
}
