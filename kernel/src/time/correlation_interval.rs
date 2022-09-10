#![forbid(unsafe_code)]

use chrono::NaiveDateTime;

use super::{
    Hz,
    NSECS_PER_SEC,
    Tsc,
    correlation_point::{
        AtomicCorrelationPoint,
        CorrelationPoint,
    },
};

pub struct CorrelationInterval<const TICKS_PER_SECOND: i64> {
    base: CorrelationPoint,
    prev: CorrelationPoint,
}

impl<const TICKS_PER_SECOND: i64> CorrelationInterval<TICKS_PER_SECOND> {
    pub fn datetime<const PARTS_PER_SECOND: i64>(
        atomic_correlation_interval: &AtomicCorrelationInterval<TICKS_PER_SECOND>,
        tsc: Tsc,
    ) -> NaiveDateTime {
        let correlation_interval = atomic_correlation_interval.load();

        if correlation_interval.elapsed_count() > 0 {
            correlation_interval.datetime_with_resolution::<PARTS_PER_SECOND>(tsc)
        } else {
            NaiveDateTime::from_timestamp(correlation_interval.prev.count() / TICKS_PER_SECOND, 0)
        }
    }

    fn datetime_with_resolution<const PARTS_PER_SECOND: i64>(
        &self,
        tsc: Tsc,
    ) -> NaiveDateTime {
        // TODO: your code here.
        NaiveDateTime::from_timestamp(self.prev.count() / TICKS_PER_SECOND, 0) // TODO: remove before flight.
    }

    fn tsc_per_second(&self) -> i64 {
        let elapsed_count = self.elapsed_count();
        if elapsed_count > 0 {
            TICKS_PER_SECOND * self.elapsed_tsc() / elapsed_count
        } else {
            0
        }
    }

    fn elapsed_count(&self) -> i64 {
        if self.base.is_valid() && self.prev.is_valid() {
            self.prev.count() - self.base.count()
        } else {
            0
        }
    }

    fn elapsed_tsc(&self) -> i64 {
        assert!(self.base.is_valid());
        assert!(self.prev.is_valid());

        self.prev.tsc() - self.base.tsc()
    }
}

pub struct AtomicCorrelationInterval<const TICKS_PER_SECOND: i64> {
    base: AtomicCorrelationPoint,
    prev: AtomicCorrelationPoint,
}

impl<const TICKS_PER_SECOND: i64> AtomicCorrelationInterval<TICKS_PER_SECOND> {
    pub const fn new() -> Self {
        Self {
            base: AtomicCorrelationPoint::new(),
            prev: AtomicCorrelationPoint::new(),
        }
    }

    pub fn load(&self) -> CorrelationInterval<TICKS_PER_SECOND> {
        CorrelationInterval {
            base: self.base.load(),
            prev: self.prev.load(),
        }
    }

    pub fn init_base(
        &self,
        base: CorrelationPoint,
    ) {
        if !self.base.is_valid() {
            self.base.store(base);
        }
    }

    pub fn store_prev(
        &self,
        prev: CorrelationPoint,
    ) {
        self.prev.store(prev);
    }

    pub fn inc_prev(
        &self,
        tsc: i64,
    ) {
        self.prev.inc(tsc);
    }

    pub fn tsc_per_second(&self) -> Option<Hz> {
        self.load().tsc_per_second().try_into().ok().and_then(Hz::new)
    }
}
