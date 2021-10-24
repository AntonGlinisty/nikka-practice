mod correlation_interval;
mod correlation_point;
mod hz;
pub mod rtc;
mod tsc;

use chrono::{
    DateTime,
    Utc,
};

pub use correlation_interval::AtomicCorrelationInterval;
pub use correlation_point::CorrelationPoint;
pub use hz::Hz;
pub use tsc::{
    Tsc,
    TscDuration,
    tsc,
};

use rtc::Rtc;

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
