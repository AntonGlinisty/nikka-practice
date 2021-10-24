#![allow(clippy::unusual_byte_groupings)]

use core::convert::TryFrom;

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

pub struct Pit(AtomicCorrelationInterval<{ TICKS_PER_SECOND as i64 }>);

impl Pit {
    pub(crate) fn interrupt() {
        PIT.0.inc_prev(ku::tsc());
    }

    pub fn tsc_per_second() -> Option<Hz> {
        PIT.0.tsc_per_second()
    }
}

pub(super) fn init() {
    const COUNTER_NUMBER_0: u8 = 0b_00 << 6;
    const LSB_THAN_MSB: u8 = 0b_11 << 4;
    const REPETITIVE_MODE: u8 = 0b_010 << 1;
    const BINARY_CODED_DECIMAL: u8 = 0b_1 << 0;
    const COMMAND_WORD: u8 =
        (COUNTER_NUMBER_0 | LSB_THAN_MSB | REPETITIVE_MODE) & !BINARY_CODED_DECIMAL;

    const COMMAND_WORD_REGISTER: u16 = 0x43;
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

const BASE: u32 = 2 * 2 * 5 * 59659;
const DIVISOR: u32 = BASE / TICKS_PER_SECOND;

const TICKS_PER_SECOND: u32 = 2 * 2 * 5;

const_assert!(TICKS_PER_SECOND * DIVISOR == BASE);

static PIT: Pit = Pit(AtomicCorrelationInterval::<{ TICKS_PER_SECOND as i64 }>::new());
