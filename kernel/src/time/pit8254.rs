#![allow(clippy::unusual_byte_groupings)]

use static_assertions::const_assert;
use x86::io;

use ku::{
    self,
    Hz,
    time::{
        AtomicCorrelationInterval,
        CorrelationPoint,
    },
};

/// Счётчик тиков PIT.
pub struct Pit(AtomicCorrelationInterval<{ TICKS_PER_SECOND as i64 }>);

impl Pit {
    /// Обработчик прерываний PIT.
    pub(crate) fn interrupt() {
        PIT.0.inc_prev(ku::tsc());
    }

    /// Оценка частоты процессора с точки зрения PIT.
    pub fn tsc_per_second() -> Option<Hz> {
        PIT.0.tsc_per_second()
    }
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
    PIT.0.init_base(now);
    PIT.0.store_prev(now);
}

/// Базовая частота PIT в герцах, из документации.
const BASE: u32 = 2 * 2 * 5 * 59659;

/// Делитель базовой частоты для конфигурирования частоты тиков.
const DIVISOR: u32 = BASE / TICKS_PER_SECOND;

/// Сконфигурированная частота тиков PIT.
///
/// Выбрана так, чтобы делить базовую частоту PIT без остатка.
const TICKS_PER_SECOND: u32 = 2 * 2 * 5;

const_assert!(TICKS_PER_SECOND * DIVISOR == BASE);

/// Счётчик тиков PIT.
static PIT: Pit = Pit(AtomicCorrelationInterval::<{ TICKS_PER_SECOND as i64 }>::new());

#[doc(hidden)]
pub(super) mod test_scaffolding {
    pub const TICKS_PER_SECOND: u32 = super::TICKS_PER_SECOND;
}
