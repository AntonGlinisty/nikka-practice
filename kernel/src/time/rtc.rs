use core::{
    convert::From,
    hint,
};

use bitflags::bitflags;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    Utc,
};
use derive_more::Display;
use scopeguard::defer;
use x86::io;

use ku::time::{
    CorrelationPoint,
    rtc::TICKS_PER_SECOND,
};

use crate::{
    SYSTEM_INFO,
    log::{
        error,
        info,
    },
};

pub(crate) fn interrupt() {
    if interrupt_status().contains(RegisterC::UPDATE_ENDED_INTERRUPT) &&
        let Some(timestamp) = timestamp()
    {
        let now = CorrelationPoint::now(timestamp * TICKS_PER_SECOND);
        let rtc = SYSTEM_INFO.rtc();
        rtc.init_base(now);
        rtc.store_prev(now);
    }
}

pub(super) fn init() {
    const DISABLE_NMI: u8 = 1 << 7;
    const REGISTER_B: u8 = 0xB;

    defer! {
        unsafe { rtc_read(!DISABLE_NMI); }
    }

    let old_settings =
        unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };
    let new_settings = (old_settings & !RegisterB::DAYLIGHT_SAVING) |
        RegisterB::UPDATE_ENDED_INTERRUPT |
        RegisterB::USE_BINARY_FORMAT |
        RegisterB::USE_24_HOUR_FORMAT;
    unsafe { rtc_write(DISABLE_NMI | REGISTER_B, new_settings.bits()) };
    let acknowledged_settings =
        unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };

    if acknowledged_settings == new_settings {
        let rtc = SYSTEM_INFO.rtc();
        rtc.store_prev(CorrelationPoint::invalid(
            timestamp().unwrap_or(0) * TICKS_PER_SECOND,
        ));

        enable_next_interrupt();

        info!(?acknowledged_settings, "RTC init");
    } else {
        error!(
            ?old_settings,
            ?new_settings,
            ?acknowledged_settings,
            "RTC did not acknowledge new settings",
        );
    }
}

bitflags! {
    /// https://pdf1.alldatasheet.com/datasheet-pdf/view/122156/MOTOROLA/MC146818.html
    struct RegisterA: u8 {
        const UPDATE_IN_PROGRESS = 1 << 7;
    }
}

bitflags! {
    /// https://pdf1.alldatasheet.com/datasheet-pdf/view/122156/MOTOROLA/MC146818.html
    struct RegisterB: u8 {
        const DAYLIGHT_SAVING = 1 << 0;
        const USE_24_HOUR_FORMAT = 1 << 1;
        const USE_BINARY_FORMAT = 1 << 2;
        const SQUARE_WAVE = 1 << 3;
        const UPDATE_ENDED_INTERRUPT = 1 << 4;
        const ALARM_INTERRUPT = 1 << 5;
        const PERIODIC_INTERRUPT = 1 << 6;
        const SET_CLOCK = 1 << 7;
    }
}

bitflags! {
    /// https://pdf1.alldatasheet.com/datasheet-pdf/view/122156/MOTOROLA/MC146818.html
    struct RegisterC: u8 {
        const UPDATE_ENDED_INTERRUPT = 1 << 4;
    }
}

#[derive(Clone, Copy, Default, Display, Eq, PartialEq)]
#[display(
    fmt = "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
    year,
    month,
    day,
    hour,
    minute,
    second
)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl Date {
    fn now() -> Option<NaiveDateTime> {
        Self::read().map(|now| now.into())
    }

    fn read() -> Option<Self> {
        const REGISTER_A: u8 = 0xA;

        // TODO: your code here.
        Some(unsafe { Self::read_inconsistent() }) // TODO: remove before flight.
    }

    unsafe fn read_inconsistent() -> Self {
        // TODO: your code here.
        Self::default() // TODO: remove before flight.
    }
}

impl From<Date> for NaiveDateTime {
    fn from(date: Date) -> NaiveDateTime {
        if date == Date::default() {
            Self::default()
        } else {
            NaiveDate::from_ymd(date.year.into(), date.month.into(), date.day.into()).and_hms(
                date.hour.into(),
                date.minute.into(),
                date.second.into(),
            )
        }
    }
}

const ADDRESS_PORT: u16 = 0x0070;
const DATA_PORT: u16 = 0x0071;

unsafe fn rtc_read(address: u8) -> u8 {
    io::outb(ADDRESS_PORT, address);
    io::inb(DATA_PORT)
}

unsafe fn rtc_write(
    address: u8,
    data: u8,
) {
    io::outb(ADDRESS_PORT, address);
    io::outb(DATA_PORT, data)
}

fn interrupt_status() -> RegisterC {
    const REGISTER_C: u8 = 0xC;
    unsafe { RegisterC::from_bits_unchecked(rtc_read(REGISTER_C)) }
}

/// See [Interrupts and Register C](https://wiki.osdev.org/RTC#Interrupts_and_Register_C).
fn enable_next_interrupt() {
    interrupt_status();
}

fn timestamp() -> Option<i64> {
    Date::now().map(|now| DateTime::<Utc>::from_utc(now, Utc).timestamp())
}
