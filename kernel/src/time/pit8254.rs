#![allow(clippy::unusual_byte_groupings)]

use x86::io;

use ku::{
    self,
    time::{
        CorrelationPoint,
        pit8254::DIVISOR,
    },
};

use crate::SYSTEM_INFO;

/// Обработчик прерываний PIT.
pub(crate) fn interrupt() {
    let pit = SYSTEM_INFO.pit();
    pit.inc_prev(ku::tsc());
}

/// Инициализация PIT.
pub(super) fn init() {
    /// Выбрать счётчик номер `0` таймера.
    const COUNTER_NUMBER_0: u8 = 0b_00 << 6;

    /// Первым передаётся младший байт [`DIVISOR`], затем старший.
    const LSB_THAN_MSB: u8 = 0b_11 << 4;

    /// Циклический режим таймера.
    const REPETITIVE_MODE: u8 = 0b_010 << 1;

    /// Использовать [двоично--десятичный](https://en.wikipedia.org/wiki/Binary-coded_decimal)
    /// формат.
    const BINARY_CODED_DECIMAL: u8 = 0b_1 << 0;

    /// Команда инициализации.
    const COMMAND_WORD: u8 =
        (COUNTER_NUMBER_0 | LSB_THAN_MSB | REPETITIVE_MODE) & !BINARY_CODED_DECIMAL;

    /// Регистр команды.
    const COMMAND_WORD_REGISTER: u16 = 0x43;

    /// Регистр счётчика номер `0` таймера.
    const COUNTER_NUMBER_0_REGISTER: u16 = 0x40;

    unsafe {
        io::outb(COMMAND_WORD_REGISTER, COMMAND_WORD);

        assert!(u16::try_from(DIVISOR).is_ok());
        io::outb(COUNTER_NUMBER_0_REGISTER, DIVISOR as u8);
        io::outb(COUNTER_NUMBER_0_REGISTER, (DIVISOR >> 8) as u8);
    }

    let now = CorrelationPoint::now(0);
    let pit = SYSTEM_INFO.pit();
    pit.init_base(now);
    pit.store_prev(now);
}
