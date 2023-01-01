use crate::time::{
    pit8254,
    rtc,
};

use super::{
    PIC_BASE,
    TRAP_STATS,
    TrapContext,
};

/// Обработчик прерывания таймера [Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253)
/// ([programmable interval timer, PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)).
pub(super) extern "x86-interrupt" fn pit(_context: TrapContext) {
    pit8254::interrupt();
    generic_pic_interrupt(super::PIT);
}

/// Обработчик прерывания клавиатуры.
pub(super) extern "x86-interrupt" fn keyboard(_context: TrapContext) {
    generic_pic_interrupt(super::KEYBOARD);
}

/// Обработчик каскадного прерывания первого контроллера
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259),
/// к которому подключён второй такой же.
pub(super) extern "x86-interrupt" fn cascade(_context: TrapContext) {
    generic_pic_interrupt(super::CASCADE);
}

/// Обработчик прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 2 и 4.
pub(super) extern "x86-interrupt" fn com2(_context: TrapContext) {
    generic_pic_interrupt(super::COM2);
}

/// Обработчик прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 1 и 3.
pub(super) extern "x86-interrupt" fn com1(_context: TrapContext) {
    generic_pic_interrupt(super::COM1);
}

/// Обработчик прерывания второго параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub(super) extern "x86-interrupt" fn lpt2(_context: TrapContext) {
    generic_pic_interrupt(super::LPT2);
}

/// Обработчик прерывания контроллера [дискет](https://en.wikipedia.org/wiki/Floppy_disk).
pub(super) extern "x86-interrupt" fn floppy_disk(_context: TrapContext) {
    generic_pic_interrupt(super::FLOPPY_DISK);
}

/// Обработчик прерывания первого параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub(super) extern "x86-interrupt" fn lpt1(_context: TrapContext) {
    generic_pic_interrupt(super::LPT1);
}

/// Обработчик прерываний
/// [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
pub(super) extern "x86-interrupt" fn rtc(_context: TrapContext) {
    rtc::interrupt();
    generic_pic_interrupt(super::RTC);
}

/// Обработчик прерывания входа `0x9` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_29(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_29);
}

/// Обработчик прерывания входа `0xA` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_2a(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_2A);
}

/// Обработчик прерывания входа `0xB` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_2b(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_2B);
}

/// Обработчик прерывания мыши.
pub(super) extern "x86-interrupt" fn ps2_mouse(_context: TrapContext) {
    generic_pic_interrupt(super::PS2_MOUSE);
}

/// Обработчик прерывания сопроцессора.
pub(super) extern "x86-interrupt" fn coprocessor(_context: TrapContext) {
    generic_pic_interrupt(super::COPROCESSOR);
}

/// Обработчик прерывания первого контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub(super) extern "x86-interrupt" fn ata0(_context: TrapContext) {
    generic_pic_interrupt(super::ATA0);
}

/// Обработчик прерывания второго контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub(super) extern "x86-interrupt" fn ata1(_context: TrapContext) {
    generic_pic_interrupt(super::ATA1);
}

/// Выполняет общую часть обработки для всех прерываний
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
/// Аргумент `number` задаёт номер прерывания в общей нумерации таблицы обработчиков прерываний
/// ([Interrupt descriptor table](https://en.wikipedia.org/wiki/Interrupt_descriptor_table), IDT).
fn generic_pic_interrupt(number: usize) {
    TRAP_STATS[number].inc();
    unsafe {
        pic8259::end_of_interrupt(number - PIC_BASE);
    }
}
