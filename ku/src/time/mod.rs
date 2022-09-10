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
    datetime_with_precision::<NSECS_PER_SEC>(tsc)
}

pub fn datetime_ms(tsc: Tsc) -> DateTime<Utc> {
    datetime_with_precision::<MSECS_PER_SEC>(tsc)
}

pub fn now() -> DateTime<Utc> {
    datetime_with_precision::<NSECS_PER_SEC>(Tsc::now())
}

pub fn now_ms() -> DateTime<Utc> {
    datetime_with_precision::<MSECS_PER_SEC>(Tsc::now())
}

#[inline(always)]
pub fn timer() -> Tsc {
    Tsc::now()
}

const MSECS_PER_SEC: i64 = 1_000;
const NSECS_PER_SEC: i64 = 1_000_000_000;

fn datetime_with_precision<const PARTS_PER_SECOND: i64>(tsc: Tsc) -> DateTime<Utc> {
    DateTime::from_utc(Rtc::datetime::<PARTS_PER_SECOND>(tsc), Utc)
}

pub mod test_scaffolding {
    pub use super::{
        correlation_point::test_scaffolding::*,
        tsc::test_scaffolding::*,
    };
}
