use core::{
    cell::UnsafeCell,
    fmt,
    hint,
    ops::{
        Deref,
        DerefMut,
    },
    sync::atomic::{
        AtomicU64,
        Ordering,
        fence,
    },
};

use crossbeam::utils::CachePadded;

pub struct TicketLock<T> {
    next_ticket: CachePadded<AtomicU64>,
    current: CachePadded<AtomicU64>,
    value: UnsafeCell<T>,
}

impl<T> TicketLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            next_ticket: CachePadded::new(AtomicU64::new(0)),
            current: CachePadded::new(AtomicU64::new(0)),
            value: UnsafeCell::new(value),
        }
    }
}

unsafe impl<T: Send> Send for TicketLock<T> {
}

unsafe impl<T: Send> Sync for TicketLock<T> {
}

unsafe impl<T: Sync> Sync for TicketLockGuard<'_, T> {
}

pub struct TicketLockGuard<'a, T> {
    lock: &'a TicketLock<T>,
}

impl<'a, T> TicketLockGuard<'a, T> {
    fn new(lock: &'a TicketLock<T>) -> Self {
        Self { lock }
    }
}

impl<T> Drop for TicketLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.release();
    }
}

impl<T> Deref for TicketLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for TicketLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

#[derive(Debug)]
pub struct WouldBlock;

impl fmt::Display for WouldBlock {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        "try_lock failed because the operation would block".fmt(f)
    }
}

pub type TryLockResult<'a, T> = Result<TicketLockGuard<'a, T>, WouldBlock>;

impl<T> TicketLock<T> {
    pub fn lock(&self) -> TicketLockGuard<'_, T> {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn try_lock(&self) -> TryLockResult<'_, T> {
        // TODO: your code here.
        unimplemented!();
    }

    fn release(&self) {
        // TODO: your code here.
        unimplemented!();
    }
}
