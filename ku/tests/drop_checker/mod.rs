use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use ku::log::debug;

#[derive(Clone, Debug, PartialEq)]
pub struct DropChecker(&'static str);

impl DropChecker {
    pub fn new(value: &'static str) -> Self {
        let old_alive_count = ALIVE_COUNT.fetch_add(1, Ordering::Relaxed);

        debug!(
            alive_count = format!("{old_alive_count} -> {}", old_alive_count + 1),
            value, "new",
        );

        Self(value)
    }

    pub fn alive_count() -> usize {
        let alive_count = ALIVE_COUNT.load(Ordering::Relaxed);

        debug!(alive_count, "check");

        alive_count
    }
}

impl Drop for DropChecker {
    fn drop(&mut self) {
        let old_alive_count = ALIVE_COUNT.fetch_sub(1, Ordering::Relaxed);

        debug!(
            alive_count = format!("{old_alive_count} -> {}", old_alive_count - 1),
            value = self.0,
            "drop",
        );
    }
}

static ALIVE_COUNT: AtomicUsize = AtomicUsize::new(0);
