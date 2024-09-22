use ku::process::Trap;

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
    generic_pic_interrupt(Trap::Pit);
}

/// Обработчик прерывания клавиатуры.
pub(super) extern "x86-interrupt" fn keyboard(_context: TrapContext) {
    generic_pic_interrupt(Trap::Keyboard);
}

/// Обработчик каскадного прерывания первого контроллера
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259),
/// к которому подключён второй такой же.
pub(super) extern "x86-interrupt" fn cascade(_context: TrapContext) {
    generic_pic_interrupt(Trap::Cascade);
}

/// Обработчик прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 2 и 4.
pub(super) extern "x86-interrupt" fn com2(_context: TrapContext) {
    generic_pic_interrupt(Trap::Com2);
}

/// Обработчик прерывания
/// [последовательных портов](https://en.wikipedia.org/wiki/Serial_port) номер 1 и 3.
pub(super) extern "x86-interrupt" fn com1(_context: TrapContext) {
    generic_pic_interrupt(Trap::Com1);
}

/// Обработчик прерывания второго параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub(super) extern "x86-interrupt" fn lpt2(_context: TrapContext) {
    generic_pic_interrupt(Trap::Lpt2);
}

/// Обработчик прерывания контроллера [дискет](https://en.wikipedia.org/wiki/Floppy_disk).
pub(super) extern "x86-interrupt" fn floppy_disk(_context: TrapContext) {
    generic_pic_interrupt(Trap::FloppyDisk);
}

/// Обработчик прерывания первого и третьего параллельного порта
/// ([Parallel port](https://en.wikipedia.org/wiki/Parallel_port)).
/// Так как через параллельные порты чаще всего подключались принтеры
/// ([Line printer](https://en.wikipedia.org/wiki/Line_printer)),
/// сохранилось их сокращение LPT.
pub(super) extern "x86-interrupt" fn lpt1(_context: TrapContext) {
    generic_pic_interrupt(Trap::Lpt1);
}

// ANCHOR: rtc
/// Обработчик прерываний
/// [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
pub(super) extern "x86-interrupt" fn rtc(_context: TrapContext) {
    rtc::interrupt();
    generic_pic_interrupt(Trap::Rtc);
}
// ANCHOR_END: rtc

/// Обработчик прерывания входа `0x9` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_29(_context: TrapContext) {
    generic_pic_interrupt(Trap::Free29);
}

/// Обработчик прерывания входа `0xA` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_2a(_context: TrapContext) {
    generic_pic_interrupt(Trap::Free2A);
}

/// Обработчик прерывания входа `0xB` каскадной пары
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
pub(super) extern "x86-interrupt" fn free_2b(_context: TrapContext) {
    generic_pic_interrupt(Trap::Free2B);
}

/// Обработчик прерывания мыши.
pub(super) extern "x86-interrupt" fn ps2_mouse(_context: TrapContext) {
    generic_pic_interrupt(Trap::Ps2Mouse);
}

/// Обработчик прерывания сопроцессора.
pub(super) extern "x86-interrupt" fn coprocessor(_context: TrapContext) {
    generic_pic_interrupt(Trap::Coprocessor);
}

/// Обработчик прерывания первого контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub(super) extern "x86-interrupt" fn ata0(_context: TrapContext) {
    generic_pic_interrupt(Trap::Ata0);
}

/// Обработчик прерывания второго контроллера
/// [PATA](https://en.wikipedia.org/wiki/Parallel_ATA).
pub(super) extern "x86-interrupt" fn ata1(_context: TrapContext) {
    generic_pic_interrupt(Trap::Ata1);
}

// ANCHOR: generic_pic_interrupt
/// Выполняет общую часть обработки для всех прерываний
/// [PIC 8259](https://en.wikipedia.org/wiki/Intel_8259).
fn generic_pic_interrupt(trap: Trap) {
    TRAP_STATS[trap].inc();
    unsafe {
        pic8259::end_of_interrupt(usize::from(trap) - PIC_BASE);
    }
}
// ANCHOR_END: generic_pic_interrupt
