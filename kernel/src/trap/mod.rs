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

use crate::{
    log::info,
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

/// [Exceptions: Division Error](https://wiki.osdev.org/Exception#Division_Error)
pub const DIVIDE_ERROR: usize = 0x00;

/// [Exceptions: Debug](https://wiki.osdev.org/Exception#Debug)
pub const DEBUG: usize = 0x01;

/// [Non-maskable interrupt](https://en.wikipedia.org/wiki/Non-maskable_interrupt),
/// [Exceptions: Non-maskable interrupt](https://wiki.osdev.org/Non_Maskable_Interrupt)
pub const NON_MASKABLE_INTERRUPT: usize = 0x02;

/// [Exceptions: Breakpoint](https://wiki.osdev.org/Exception#Breakpoint)
pub const BREAKPOINT: usize = 0x03;

/// [Exceptions: Overflow](https://wiki.osdev.org/Exception#Overflow)
pub const OVERFLOW: usize = 0x04;

/// [Exceptions: Bound Range Exceeded](https://wiki.osdev.org/Exception#Bound_Range_Exceeded)
pub const BOUND_RANGE_EXCEEDED: usize = 0x05;

/// [Exceptions: Invalid Opcode](https://wiki.osdev.org/Exception#Invalid_Opcode)
pub const INVALID_OPCODE: usize = 0x06;

/// [Exceptions: Device Not Available](https://wiki.osdev.org/Exception#Device_Not_Available)
pub const DEVICE_NOT_AVAILABLE: usize = 0x07;

/// [Double Fault](https://en.wikipedia.org/wiki/Double_fault),
/// [Exceptions: Double Fault](https://wiki.osdev.org/Exception#Double_Fault)
pub const DOUBLE_FAULT: usize = 0x08;

/// [Exceptions: Invalid TSS](https://wiki.osdev.org/Exception#Invalid_TSS)
pub const INVALID_TSS: usize = 0x0A;

/// [Exceptions: Segment Not Present](https://wiki.osdev.org/Exception#Segment_Not_Present)
pub const SEGMENT_NOT_PRESENT: usize = 0x0B;

/// [Exceptions: Stack-Segment Fault](https://wiki.osdev.org/Exception#Stack-Segment_Fault)
pub const STACK_SEGMENT_FAULT: usize = 0x0C;

/// [General Protection Fault](https://en.wikipedia.org/wiki/General_protection_fault),
/// [Exceptions: General Protection Fault](https://wiki.osdev.org/Exception#General_Protection_Fault)
pub const GENERAL_PROTECTION_FAULT: usize = 0x0D;

/// [Page Fault](https://en.wikipedia.org/wiki/Page_fault),
/// [Exceptions: Page Fault](https://wiki.osdev.org/Exception#Page_Fault)
pub const PAGE_FAULT: usize = 0x0E;

/// [Exceptions: x87 Floating-Point Exception](https://wiki.osdev.org/Exception#x87_Floating-Point_Exception)
pub const X87_FLOATING_POINT: usize = 0x10;

/// [Exceptions: Alignment Check](https://wiki.osdev.org/Exception#Alignment_Check)
pub const ALIGNMENT_CHECK: usize = 0x11;

/// [Machine-check exception](https://en.wikipedia.org/wiki/Machine-check_exception),
/// [Exceptions: Machine Check](https://wiki.osdev.org/Exception#Machine_Check)
pub const MACHINE_CHECK: usize = 0x12;

/// [Exceptions: SIMD Floating-Point Exception](https://wiki.osdev.org/Exception#SIMD_Floating-Point_Exception)
pub const SIMD_FLOATING_POINT: usize = 0x13;

/// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
pub const VIRTUALIZATION: usize = 0x14;

/// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
pub const SECURITY_EXCEPTION: usize = 0x1E;

/// Количество
/// [исключений процессора](https://en.wikipedia.org/wiki/Interrupt_descriptor_table#Processor-generated_exceptions).
const EXCEPTION_COUNT: u8 = 0x20;

/// Первое прерывание
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
/// [Стандартная последовательность](https://wiki.osdev.org/Interrupts#Standard_ISA_IRQs)
/// подключения
/// [прерываний в x86](https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture))
/// для устройств шины
/// [Industry Standard Architecture](https://en.wikipedia.org/wiki/Industry_Standard_Architecture)
/// (ISA).
const PIC_FIRST_INTERRUPT: u8 = EXCEPTION_COUNT;

/// Первое прерывание
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
/// [Стандартная последовательность](https://wiki.osdev.org/Interrupts#Standard_ISA_IRQs)
/// подключения
/// [прерываний в x86](https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture))
/// для устройств шины
/// [Industry Standard Architecture](https://en.wikipedia.org/wiki/Industry_Standard_Architecture)
/// (ISA).
const PIC_BASE: usize = PIC_FIRST_INTERRUPT as usize;

/// Номер прерывания таймера [Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253)
/// ([programmable interval timer, PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)).
pub const PIT: usize = PIC_BASE;

/// Номер прерывания клавиатуры.
pub const KEYBOARD: usize = PIC_BASE + 0x1;

/// Номер входа первого контроллера
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259),
/// к которому каскадно подключён второй такой же.
pub const CASCADE: usize = PIC_BASE + 0x2;

/// Номер прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 2 и 4.
pub const COM2: usize = PIC_BASE + 0x3;

/// Номер прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 1 и 3.
pub const COM1: usize = PIC_BASE + 0x4;

/// [Номер прерывания](https://en.wikipedia.org/wiki/Parallel_port#Port_addresses)
/// второго параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub const LPT2: usize = PIC_BASE + 0x5;

/// Номер прерывания контроллера [дискет](https://en.wikipedia.org/wiki/Floppy_disk).
pub const FLOPPY_DISK: usize = PIC_BASE + 0x6;

/// [Номер прерывания](https://en.wikipedia.org/wiki/Parallel_port#Port_addresses)
/// первого и третьего параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub const LPT1: usize = PIC_BASE + 0x7;

/// Номер обработчика прерываний
/// [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
pub const RTC: usize = PIC_BASE + 0x8;

/// Номер прерывания входа `0x9` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub const FREE_29: usize = PIC_BASE + 0x9;

/// Номер прерывания входа `0xA` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub const FREE_2A: usize = PIC_BASE + 0xA;

/// Номер прерывания входа `0xB` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub const FREE_2B: usize = PIC_BASE + 0xB;

/// Номер прерывания мыши.
pub const PS2_MOUSE: usize = PIC_BASE + 0xC;

/// Номер прерывания сопроцессора.
pub const COPROCESSOR: usize = PIC_BASE + 0xD;

/// Номер прерывания первого контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub const ATA0: usize = PIC_BASE + 0xE;

/// Номер прерывания второго контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub const ATA1: usize = PIC_BASE + 0xF;

/// Первое прерывание
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
const APIC_BASE: usize = ATA1 + 1;

/// Номер прерывания
/// [таймера APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
pub const TIMER: usize = APIC_BASE;

/// Номер ложных прерываний
/// ([spurious interrupt](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts))
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
pub const SPURIOUS: usize = APIC_BASE + 0x1;

/// Количество исключений и прерываний.
const COUNT: usize = SPURIOUS + 1;

/// Инициализирует таблицу обработчиков прерываний [`struct@IDT`].
pub(super) fn init() {
    unsafe {
        pic8259::init(PIC_FIRST_INTERRUPT);
    }

    IDT.load();

    interrupts::enable();

    rtc::enable_next_interrupt();

    info!("traps init");
}

#[doc(hidden)]
pub mod test_scaffolding {
    pub use super::{
        idt::test_scaffolding::*,
        local_apic::test_scaffolding::*,
    };
}
