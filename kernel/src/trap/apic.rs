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
///
/// # Safety
///
/// Local APIC должен быть переведён в
/// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
/// методом [`LocalApic::enable_x2()`].
pub(super) unsafe extern "x86-interrupt" fn timer(mut context: TrapContext) {
    Process::preempt(&mut context);

    unsafe {
        generic_apic_interrupt(Trap::Timer);
    }
}

/// Обработчик ложных прерываний
/// ([spurious interrupt](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts))
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
///
/// # Safety
///
/// Local APIC должен быть переведён в
/// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
/// методом [`LocalApic::enable_x2()`].
pub(super) unsafe extern "x86-interrupt" fn spurious(_context: TrapContext) {
    unsafe {
        generic_apic_interrupt(Trap::Spurious);
    }
}

/// Выполняет общую часть обработки для всех прерываний
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
///
/// # Safety
///
/// Local APIC должен быть переведён в
/// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
/// методом [`LocalApic::enable_x2()`].
unsafe fn generic_apic_interrupt(trap: Trap) {
    TRAP_STATS[trap].inc();
    unsafe {
        LocalApic::end_of_interrupt();
    }
}
