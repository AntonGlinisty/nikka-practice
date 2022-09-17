use chrono::{
    DateTime,
    Utc,
};

use crate::system_info;

use super::{
    Hz,
    Tsc,
    correlation_interval::CorrelationInterval,
};

pub const TICKS_PER_SECOND: i64 = 1;

pub struct Rtc;

impl Rtc {
    pub fn datetime<const PARTS_PER_SECOND: i64>(tsc: Tsc) -> DateTime<Utc> {
        let rtc = system_info().rtc();
        CorrelationInterval::datetime::<PARTS_PER_SECOND>(rtc, tsc)
    }

    pub fn tsc_per_second() -> Option<Hz> {
        let rtc = system_info().rtc();
        rtc.tsc_per_second()
    }
}
