use core::{
    hint,
    sync::atomic::{
        AtomicI64,
        Ordering,
    },
};

use super::tsc;

#[derive(Clone, Copy)]
pub(super) struct CorrelationPoint {
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

    pub(super) fn invalid(count: i64) -> Self {
        Self::new(count, 0)
    }

    pub(super) fn now(count: i64) -> Self {
        Self::new(count, tsc::tsc())
    }

    pub(super) fn is_valid(&self) -> bool {
        self.tsc != 0
    }

    pub(super) fn count(&self) -> i64 {
        self.count
    }

    pub(super) fn tsc(&self) -> i64 {
        self.tsc
    }
}

/// Single writer [Sequence Lock (AKA Seqlock)](https://en.wikipedia.org/wiki/Seqlock).
/// The protected data is a couple of atomic variables which simplifies the Seqlock a lot.
/// See also:
/// - [Writing a seqlock in Rust.](https://pitdicker.github.io/Writing-a-seqlock-in-Rust/)
/// - [Can Seqlocks Get Along With Programming Language Memory Models?](https://www.hpl.hp.com/techreports/2012/HPL-2012-68.pdf)
/// - [Crate seqlock.](https://docs.rs/seqlock/0.1.2/seqlock/)
pub(super) struct AtomicCorrelationPoint {
    /// An odd `count` means that an update is in progress.
    count: AtomicI64,
    tsc: AtomicI64,
}

impl AtomicCorrelationPoint {
    pub(super) const fn new() -> Self {
        Self {
            count: AtomicI64::new(0),
            tsc: AtomicI64::new(0),
        }
    }

    pub(super) fn is_valid(&self) -> bool {
        self.tsc.load(Ordering::Relaxed) != 0
    }

    pub(super) fn load(&self) -> CorrelationPoint {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn store(
        &self,
        correlation_point: CorrelationPoint,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn inc(
        &self,
        tsc: i64,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}
