#![deny(warnings)]
#![feature(cfg_select)]
#![feature(const_ops)]
#![feature(const_trait_impl)]

use rstest::*;
use std::{
    sync::{
        Barrier,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};

use ticket_lock::{
    TicketLock,
    TicketLockGuard,
    TryLockResult,
};

#[derive(Default)]
struct LockNode;

impl LockNode {
    fn new() -> Self {
        Self
    }
}

struct Lock<T>(TicketLock<T>);

impl<T> Lock<T> {
    fn new(value: T) -> Self {
        Self(TicketLock::new(value))
    }

    fn lock(
        &self,
        _node: &mut LockNode,
    ) -> TicketLockGuard<'_, T> {
        self.0.lock()
    }

    fn try_lock(
        &self,
        _node: &mut LockNode,
    ) -> TryLockResult<'_, T> {
        self.0.try_lock()
    }

    fn section<R, F: FnOnce(&mut T) -> R>(
        &self,
        critical: F,
    ) -> R {
        let mut guard = self.0.lock();
        critical(&mut *guard)
    }
}

include!("../../mcs_lock/tests/include/lock_tests.rs");
include!("../../mcs_lock/tests/include/fair_lock_tests.rs");
include!("../../mcs_lock/tests/include/try_lock_tests.rs");
