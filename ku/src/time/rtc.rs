use core::convert::From;

use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    Utc,
};
use derive_more::Display;
use scopeguard::defer;
use x86::io;

use crate::log::{
    error,
    info,
};

use super::{
    Hz,
    correlation_interval::{
        AtomicCorrelationInterval,
        CorrelationInterval,
    },
    correlation_point::CorrelationPoint,
};

pub struct Rtc(AtomicCorrelationInterval<TICKS_PER_SECOND>);

impl Rtc {
    pub(crate) fn interrupt() {
        if interrupt_status() & UPDATE_ENDED_INTERRUPT != 0 {
            let now = CorrelationPoint::now(rtc_timestamp() * TICKS_PER_SECOND);
            RTC.0.init_base(now);
            RTC.0.store_prev(now);
        }
    }

    pub fn now<const PARTS_PER_SECOND: i64>() -> NaiveDateTime {
        CorrelationInterval::now::<PARTS_PER_SECOND>(&RTC.0)
    }

    pub fn tsc_per_second() -> Option<Hz> {
        RTC.0.tsc_per_second()
    }
}

pub(super) fn init() {
    const DISABLE_NMI: u8 = 1 << 7;
    const REGISTER_B: u8 = 0xB;
    const ENABLE_DAYLIGHT_SAVING: u8 = 1 << 0;
    const SET_24_HOUR_FORMAT: u8 = 1 << 1;
    const DISABLE_BINARY_CODED_DECIMAL: u8 = 1 << 2;
    const ENABLE_UPDATE_ENDED_INTERRUPT: u8 = 1 << 4;

    defer! {
        unsafe { rtc_read(!DISABLE_NMI); }
    }

    let old_settings = unsafe { rtc_read(DISABLE_NMI | REGISTER_B) };
    let new_settings = !ENABLE_DAYLIGHT_SAVING &
        (old_settings |
            ENABLE_UPDATE_ENDED_INTERRUPT |
            DISABLE_BINARY_CODED_DECIMAL |
            SET_24_HOUR_FORMAT);
    unsafe {
        rtc_write(DISABLE_NMI | REGISTER_B, new_settings);
    }
    let acknowledged_settings = unsafe { rtc_read(DISABLE_NMI | REGISTER_B) };

    if acknowledged_settings == new_settings {
        RTC.0.store_prev(CorrelationPoint::invalid(
            rtc_timestamp() * TICKS_PER_SECOND,
        ));

        enable_next_interrupt();

        info!(new_settings, "RTC init");
    } else {
        error!(
            old_settings,
            new_settings, acknowledged_settings, "RTC did not acknowledge new settings",
        );
    }
}

#[derive(Clone, Copy, Display, PartialEq)]
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
        Self::rtc_read().map(|now| now.into())
    }

    fn rtc_read() -> Option<Self> {
        // TODO: your code here.

        None
    }

    unsafe fn rtc_read_inconsistent() -> Self {
        // TODO: your code here.
        unimplemented!();
    }
}

impl From<Date> for NaiveDateTime {
    fn from(date: Date) -> NaiveDateTime {
        NaiveDate::from_ymd(date.year.into(), date.month.into(), date.day.into()).and_hms(
            date.hour.into(),
            date.minute.into(),
            date.second.into(),
        )
    }
}

const ADDRESS_PORT: u16 = 0x70;
const DATA_PORT: u16 = 0x71;

const TICKS_PER_SECOND: i64 = 1;

const UPDATE_ENDED_INTERRUPT: u8 = 1 << 4;

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

fn interrupt_status() -> u8 {
    const REGISTER_C: u8 = 0xC;
    unsafe { rtc_read(REGISTER_C) }
}

/// See [Interrupts and Register C](https://wiki.osdev.org/RTC#Interrupts_and_Register_C).
fn enable_next_interrupt() {
    interrupt_status();
}

fn rtc_timestamp() -> i64 {
    if let Some(now) = Date::now() {
        DateTime::<Utc>::from_utc(now, Utc).timestamp()
    } else {
        0
    }
}

static RTC: Rtc = Rtc(AtomicCorrelationInterval::<TICKS_PER_SECOND>::new());
