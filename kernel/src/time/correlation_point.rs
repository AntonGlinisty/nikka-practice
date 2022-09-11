use core::{
    hint,
    sync::atomic::{
        AtomicI64,
        Ordering,
    },
};

use super::tsc;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CorrelationPoint {
    count: i64,
    tsc: i64,
}

impl CorrelationPoint {
    fn new(
        count: i64,
        tsc: i64,
    ) -> Self {
        assert!(count >= 0);
        assert!(tsc >= 0);

        Self { count, tsc }
    }

    pub fn invalid(count: i64) -> Self {
        Self::new(count, 0)
    }

    pub fn now(count: i64) -> Self {
        Self::new(count, tsc::tsc())
    }

    pub fn is_valid(&self) -> bool {
        self.tsc != 0
    }

    pub fn count(&self) -> i64 {
        self.count
    }

    pub fn tsc(&self) -> i64 {
        self.tsc
    }
}

/// Single writer [Sequence Lock (AKA Seqlock)](https://en.wikipedia.org/wiki/Seqlock).
/// The protected data is a couple of atomic variables which simplifies the Seqlock a lot.
/// See also:
/// - [Writing a seqlock in Rust.](https://pitdicker.github.io/Writing-a-seqlock-in-Rust/)
/// - [Can Seqlocks Get Along With Programming Language Memory Models?](https://www.hpl.hp.com/techreports/2012/HPL-2012-68.pdf)
/// - [Crate seqlock.](https://docs.rs/seqlock/0.1.2/seqlock/)
#[derive(Debug, Default)]
pub struct AtomicCorrelationPoint {
    /// An odd `count` means that an update is in progress.
    count: AtomicI64,
    tsc: AtomicI64,
}

impl AtomicCorrelationPoint {
    pub const fn new() -> Self {
        Self {
            count: AtomicI64::new(0),
            tsc: AtomicI64::new(0),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.tsc.load(Ordering::Relaxed) != 0
    }

    pub fn inc(
        &self,
        tsc: i64,
    ) {
        // TODO: your code here.
        self.count.fetch_add(1, Ordering::Relaxed); // TODO: remove before flight.
        self.tsc.store(tsc, Ordering::Relaxed); // TODO: remove before flight.
    }

    pub fn store(
        &self,
        correlation_point: CorrelationPoint,
    ) {
        // TODO: your code here.
        self.count.store(correlation_point.count(), Ordering::Relaxed); // TODO: remove before flight.
        self.tsc.store(correlation_point.tsc(), Ordering::Relaxed); // TODO: remove before flight.
    }

    pub fn load(&self) -> CorrelationPoint {
        loop {
            if let Some(point) = self.try_load() {
                return point;
            } else {
                hint::spin_loop();
            }
        }
    }

    fn try_load(&self) -> Option<CorrelationPoint> {
        // TODO: your code here.
        Some(CorrelationPoint { count: self.count.load(Ordering::Relaxed), tsc: self.tsc.load(Ordering::Relaxed) }) // TODO: remove before flight.
    }
}

pub(super) mod test_scaffolding {
    use super::CorrelationPoint;

    pub use super::AtomicCorrelationPoint;

    pub fn new_point(
        count: i64,
        tsc: i64,
    ) -> CorrelationPoint {
        CorrelationPoint::new(count, tsc)
    }

    pub fn try_load(point: &AtomicCorrelationPoint) -> Option<CorrelationPoint> {
        point.try_load()
    }
}
