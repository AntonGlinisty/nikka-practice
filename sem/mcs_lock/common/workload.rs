#![deny(warnings)]

use std::{
    hint::black_box,
    thread,
};

use spin::mutex::{
    SpinMutex as SpinLock,
    TicketMutex as TicketLock,
};

use mcs_lock::{
    McsLock,
    McsLockNode,
};

pub trait Lockable<T>: Send + Sync {
    fn critical_section<R, F: FnOnce(&mut T) -> R>(
        &self,
        section: F,
    ) -> R;

    fn new(value: T) -> Self;
}

impl<T: Send> Lockable<T> for McsLock<T> {
    fn critical_section<R, F: FnOnce(&mut T) -> R>(
        &self,
        section: F,
    ) -> R {
        section(&mut *self.lock(&mut McsLockNode::new()))
    }

    fn new(value: T) -> Self {
        McsLock::new(value)
    }
}

impl<T: Send> Lockable<T> for SpinLock<T> {
    fn critical_section<R, F: FnOnce(&mut T) -> R>(
        &self,
        section: F,
    ) -> R {
        section(&mut *self.lock())
    }

    fn new(value: T) -> Self {
        SpinLock::new(value)
    }
}

impl<T: Send> Lockable<T> for TicketLock<T> {
    fn critical_section<R, F: FnOnce(&mut T) -> R>(
        &self,
        section: F,
    ) -> R {
        section(&mut *self.lock())
    }

    fn new(value: T) -> Self {
        TicketLock::new(value)
    }
}

pub fn heavy() {
    black_box((0 .. black_box(75)).fold(1, |a, b| a ^ b));
}

pub fn pin_to(core: u64) {
    let mut cpuset = rustix::process::CpuSet::new();
    cpuset.set(core as usize);
    rustix::process::sched_setaffinity(None, &cpuset).unwrap();
}

pub fn run_lock_workload<L: Lockable<u64>>(
    threads: u64,
    iterations: u64,
    critical: impl Fn() + Send + Sync,
    setup: impl Fn(u64) + Send + Sync,
) {
    let lock = L::new(0);
    let lock_ref = &lock;

    thread::scope(|s| {
        for i in 0 .. threads {
            let critical_ref = &critical;
            let setup_ref = &setup;
            s.spawn(move || {
                setup_ref(i);
                for _ in 0 .. iterations {
                    lock_ref.critical_section(|r| {
                        critical_ref();
                        *r += 1;
                    });
                }
            });
        }
    });

    assert_eq!(lock.critical_section(|r| *r), threads * iterations);
}
