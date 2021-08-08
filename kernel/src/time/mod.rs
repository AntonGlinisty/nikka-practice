mod correlation_interval;
mod correlation_point;
mod hz;
mod pit8254;
mod rtc;
mod tsc;

use chrono::{
    DateTime,
    Utc,
};

use crate::log::info;

pub use hz::Hz;
pub use pit8254::Pit;
pub use rtc::Rtc;
pub use tsc::{
    Tsc,
    TscDuration,
};

pub(super) fn init() {
    pit8254::init();
    rtc::init();

    info!("time init");
}

pub fn now() -> DateTime<Utc> {
    now_with_precision::<NSECS_PER_SEC>()
}

pub fn now_ms() -> DateTime<Utc> {
    now_with_precision::<MSECS_PER_SEC>()
}

#[inline(always)]
pub fn timer() -> Tsc {
    Tsc::now()
}

const MSECS_PER_SEC: i64 = 1_000;
const NSECS_PER_SEC: i64 = 1_000_000_000;

fn now_with_precision<const PARTS_PER_SECOND: i64>() -> DateTime<Utc> {
    DateTime::from_utc(Rtc::now::<PARTS_PER_SECOND>(), Utc)
}
