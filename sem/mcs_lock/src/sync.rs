use core::{
    cell::UnsafeCell,
    hint,
    ops::{
        Deref,
        DerefMut,
    },
    ptr::{
        self,
        NonNull,
    },
    sync::atomic::{
        self,
        AtomicBool,
        AtomicPtr,
        Ordering,
    },
};

use crossbeam::utils::CachePadded;

pub struct McsLock<T> {
    last: AtomicPtr<McsLockNode>,
    value: UnsafeCell<T>,
}

impl<T> McsLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            last: AtomicPtr::new(ptr::null_mut()),
            value: UnsafeCell::new(v),
        }
    }
}

unsafe impl<T: Send> Send for McsLock<T> {
}

unsafe impl<T: Send> Sync for McsLock<T> {
}

#[derive(Default)]
pub struct McsLockNode {
    owner: CachePadded<AtomicBool>,
    next: CachePadded<AtomicPtr<McsLockNode>>,
}

impl McsLockNode {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct McsLockGuard<'a, T> {
    node: NonNull<McsLockNode>,
    lock: &'a McsLock<T>,
}

impl<'a, T> McsLockGuard<'a, T> {
    fn new(
        lock: &'a McsLock<T>,
        node: &'a mut McsLockNode,
    ) -> Self {
        let node = NonNull::from(node);
        lock.acquire(node);
        Self { node, lock }
    }
}

impl<T> Drop for McsLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.release(self.node);
    }
}

impl<T> Deref for McsLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for McsLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> McsLock<T> {
    pub fn section<R, F: FnOnce(&mut T) -> R>(
        &self,
        critical: F,
    ) -> R {
        let mut node = McsLockNode::new();
        let mut guard = self.lock(&mut node);
        critical(&mut *guard)
    }

    pub fn lock<'a>(
        &'a self,
        node: &'a mut McsLockNode,
    ) -> McsLockGuard<'a, T> {
        McsLockGuard::new(self, node)
    }

    fn acquire(
        &self,
        node: NonNull<McsLockNode>,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    fn release(
        &self,
        node: NonNull<McsLockNode>,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}
