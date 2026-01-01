/// Информация об исключении процессора.
mod trap_info;

use enum_iterator::Sequence;
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};

pub use trap_info::{
    Info,
    RSP_OFFSET_IN_TRAP_INFO,
    TrapInfo,
};

use super::memory::size;

/// Исключение или прерывание.
#[derive(
    Clone, Copy, Debug, Eq, IntoPrimitive, Ord, PartialEq, PartialOrd, Sequence, TryFromPrimitive,
)]
#[repr(usize)]
pub enum Trap {
    /// [Exceptions: Division Error](https://wiki.osdev.org/Exception#Division_Error)
    DivideError = 0x00,

    /// [Exceptions: Debug](https://wiki.osdev.org/Exception#Debug)
    Debug = 0x01,

    /// [Non-maskable interrupt](https://en.wikipedia.org/wiki/Non-maskable_interrupt),
    /// [Exceptions: Non-maskable interrupt](https://wiki.osdev.org/Non_Maskable_Interrupt)
    NonMaskableInterrupt = 0x02,

    /// [Exceptions: Breakpoint](https://wiki.osdev.org/Exception#Breakpoint)
    Breakpoint = 0x03,

    /// [Exceptions: Overflow](https://wiki.osdev.org/Exception#Overflow)
    Overflow = 0x04,

    /// [Exceptions: Bound Range Exceeded](https://wiki.osdev.org/Exception#Bound_Range_Exceeded)
    BoundRangeExceeded = 0x05,

    /// [Exceptions: Invalid Opcode](https://wiki.osdev.org/Exception#Invalid_Opcode)
    InvalidOpcode = 0x06,

    /// [Exceptions: Device Not Available](https://wiki.osdev.org/Exception#Device_Not_Available)
    DeviceNotAvailable = 0x07,

    /// [Double Fault](https://en.wikipedia.org/wiki/Double_fault),
    /// [Exceptions: Double Fault](https://wiki.osdev.org/Exception#Double_Fault)
    DoubleFault = 0x08,

    /// [Exceptions: Invalid TSS](https://wiki.osdev.org/Exception#Invalid_TSS)
    InvalidTss = 0x0A,

    /// [Exceptions: Segment Not Present](https://wiki.osdev.org/Exception#Segment_Not_Present)
    SegmentNotPresent = 0x0B,

    /// [Exceptions: Stack-Segment Fault](https://wiki.osdev.org/Exception#Stack-Segment_Fault)
    StackSegmentFault = 0x0C,

    /// [General Protection Fault](https://en.wikipedia.org/wiki/General_protection_fault),
    /// [Exceptions: General Protection Fault](https://wiki.osdev.org/Exception#General_Protection_Fault)
    GeneralProtectionFault = 0x0D,

    /// [Page Fault](https://en.wikipedia.org/wiki/Page_fault),
    /// [Exceptions: Page Fault](https://wiki.osdev.org/Exception#Page_Fault)
    PageFault = 0x0E,

    /// [Exceptions: x87 Floating-Point Exception](https://wiki.osdev.org/Exception#x87_Floating-Point_Exception)
    X87FloatingPoint = 0x10,

    /// [Exceptions: Alignment Check](https://wiki.osdev.org/Exception#Alignment_Check)
    AlignmentCheck = 0x11,

    /// [Machine-check exception](https://en.wikipedia.org/wiki/Machine-check_exception),
    /// [Exceptions: Machine Check](https://wiki.osdev.org/Exception#Machine_Check)
    MachineCheck = 0x12,

    /// [Exceptions: SIMD Floating-Point Exception](https://wiki.osdev.org/Exception#SIMD_Floating-Point_Exception)
    SimdFloatingPoint = 0x13,

    /// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
    Virtualization = 0x14,

    /// [Exceptions: Security Exception](https://wiki.osdev.org/Exception#Security_Exception)
    SecurityException = 0x1E,

    /// Номер прерывания таймера [Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253)
    /// ([programmable interval timer, PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)).
    Pit = 0x20,

    /// Номер прерывания клавиатуры.
    Keyboard,

    /// Номер входа первого контроллера
    /// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259),
    /// к которому каскадно подключён второй такой же.
    Cascade,

    /// Номер прерывания
    /// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 2 и 4.
    Com2,

    /// Номер прерывания
    /// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 1 и 3.
    Com1,

    /// [Номер прерывания](https://en.wikipedia.org/wiki/Parallel_port#Port_addresses)
    /// второго параллельного порта
    /// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
    /// Так как через параллельные порты чаще всего подключались принтеры
    /// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
    /// сохранилось их сокращение LPT.
    Lpt2,

    /// Номер прерывания контроллера [дискет](https://en.wikipedia.org/wiki/Floppy_disk).
    FloppyDisk,

    /// [Номер прерывания](https://en.wikipedia.org/wiki/Parallel_port#Port_addresses)
    /// первого и третьего параллельного порта
    /// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
    /// Так как через параллельные порты чаще всего подключались принтеры
    /// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
    /// сохранилось их сокращение LPT.
    Lpt1,

    /// Номер обработчика прерываний
    /// [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
    Rtc,

    /// Номер прерывания входа `0x9` каскадной пары
    /// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
    Free29,

    /// Номер прерывания входа `0xA` каскадной пары
    /// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
    Free2A,

    /// Номер прерывания входа `0xB` каскадной пары
    /// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
    Free2B,

    /// Номер прерывания мыши.
    Ps2Mouse,

    /// Номер прерывания сопроцессора.
    Coprocessor,

    /// Номер прерывания первого контроллера
    /// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
    Ata0,

    /// Номер прерывания второго контроллера
    /// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
    Ata1,

    /// Номер прерывания
    /// [таймера APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
    Timer,

    /// Номер ложных прерываний
    /// ([spurious interrupt](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts))
    /// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
    Spurious,
}

impl Trap {
    /// Номер прерывания или исключения слишком большой.
    const INVALID_TRAP_NUMBER: &str = "too big trap number";
}

impl From<Trap> for u32 {
    fn from(trap: Trap) -> Self {
        size::try_into::<u32>(trap.into()).expect(Trap::INVALID_TRAP_NUMBER)
    }
}
