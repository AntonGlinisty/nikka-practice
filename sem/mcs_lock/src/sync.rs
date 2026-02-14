use core::{
    cell::UnsafeCell,
    fmt,
    hint,
    ops::{
        Deref,
        DerefMut,
    },
    ptr,
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

unsafe impl<T: Sync> Sync for McsLockGuard<'_, T> {
}

#[derive(Default)]
pub struct McsLockNode {
    owner: CachePadded<AtomicBool>,
    next: CachePadded<AtomicPtr<McsLockNode>>,
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

pub type TryLockResult<'a, T> = Result<McsLockGuard<'a, T>, WouldBlock>;

impl McsLockNode {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct McsLockGuard<'a, T> {
    node: &'a McsLockNode,
    lock: &'a McsLock<T>,
}

impl<'a, T> McsLockGuard<'a, T> {
    fn new(
        lock: &'a McsLock<T>,
        node: &'a McsLockNode,
    ) -> Self {
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
        let node = &*node;
        self.acquire(node);
        McsLockGuard::new(self, node)
    }

    pub fn try_lock<'a>(
        &'a self,
        node: &'a mut McsLockNode,
    ) -> TryLockResult<'a, T> {
        let node = &*node;
        self.try_acquire(node).then(|| McsLockGuard::new(self, node)).ok_or(WouldBlock)
    }

    fn try_acquire(
        &self,
        node: &McsLockNode,
    ) -> bool {
        // TODO: your code here.
        unimplemented!();
    }

    fn acquire(
        &self,
        node: &McsLockNode,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    fn release(
        &self,
        node: &McsLockNode,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}
