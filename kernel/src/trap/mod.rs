/// Обработчики прерываний, маршрутизируемых через
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
mod apic;

/// Общий код обработчиков исключений и прерываний.
mod handler;

/// Таблица обработчиков исключений и прерываний
/// ([Interrupt descriptor table](https://en.wikipedia.org/wiki/Interrupt_descriptor_table), IDT).
mod idt;

/// Обработчики прерываний, маршрутизируемых через
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
mod pic;

/// Информация обо всех исключениях и прерываниях.
mod stats;

/// Обёртка для [`ModeContext`].
/// Она делает запись в него через [`core::ptr::write_volatile()`],
/// чтобы компилятор такую операцию записи не выкинул.
mod trap_context;

use x86_64::instructions::interrupts;

pub use ku::process::Trap;

use crate::{
    log::info,
    smp::LocalApic,
    time::rtc,
};

pub(crate) use self::idt::IDT;

pub use self::{
    stats::TRAP_STATS,
    trap_context::TrapContext,
};

// Used in docs.
#[allow(unused)]
use crate::process::ModeContext;

/// Первое прерывание
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
/// [Стандартная последовательность](https://wiki.osdev.org/Interrupts#Standard_ISA_IRQs)
/// подключения
/// [прерываний в x86](https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture))
/// для устройств шины
/// [Industry Standard Architecture](https://en.wikipedia.org/wiki/Industry_Standard_Architecture)
/// (ISA).
const PIC_BASE: usize = Trap::Pit as usize;

/// Количество исключений и прерываний.
const COUNT: usize = Trap::Spurious as usize + 1;

// ANCHOR: init
/// Инициализирует таблицу обработчиков прерываний [`struct@IDT`].
pub(super) fn init() {
    unsafe {
        pic8259::init(PIC_BASE.try_into().expect("too many interrupt numbers"));
    }

    IDT.load();

    LocalApic::init().expect("failed to initialize the local APIC");

    interrupts::enable();

    rtc::enable_next_interrupt();

    info!("traps init");
}
// ANCHOR_END: init

#[doc(hidden)]
pub mod test_scaffolding {
    pub use super::{
        idt::test_scaffolding::*,
        local_apic::test_scaffolding::*,
    };
}
