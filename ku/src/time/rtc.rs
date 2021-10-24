use chrono::NaiveDateTime;

use crate::system_info;

use super::{
    Hz,
    correlation_interval::CorrelationInterval,
};

pub const TICKS_PER_SECOND: i64 = 1;

pub struct Rtc;

impl Rtc {
    pub fn now<const PARTS_PER_SECOND: i64>() -> NaiveDateTime {
        let rtc = system_info().rtc();
        CorrelationInterval::now::<PARTS_PER_SECOND>(rtc)
    }

    pub fn tsc_per_second() -> Option<Hz> {
        let rtc = system_info().rtc();
        rtc.tsc_per_second()
    }
}
