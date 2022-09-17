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

pub fn datetime(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<NSECS_PER_SEC>(tsc)
}

pub fn datetime_ms(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<MSECS_PER_SEC>(tsc)
}

pub fn now() -> DateTime<Utc> {
    Rtc::datetime::<NSECS_PER_SEC>(Tsc::now())
}

pub fn now_ms() -> DateTime<Utc> {
    Rtc::datetime::<MSECS_PER_SEC>(Tsc::now())
}

#[inline(always)]
pub fn timer() -> Tsc {
    Tsc::now()
}

const MSECS_PER_SEC: i64 = 1_000;
const NSECS_PER_SEC: i64 = 1_000_000_000;

pub mod test_scaffolding {
    pub use super::{
        correlation_point::test_scaffolding::*,
        tsc::test_scaffolding::*,
    };
}
