use core::{
    hint,
    mem::MaybeUninit,
};

use ku::collections::bounded_deque::{
    Error::{
        Empty,
        Full,
    },
    Result,
};

#[derive(Debug)]
pub struct QueueWithLen<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> QueueWithLen<T, N> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            head: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        N - self.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    #[allow(unused)]
    pub fn dequeue(&mut self) -> Result<T, T> {
        if self.is_empty() {
            Err(Empty)
        } else {
            let old_head = self.head;
            self.head = Self::increment(self.head);
            self.len -= 1;

            Ok(unsafe { self.buf[old_head].as_ptr().read() })
        }
    }

    #[allow(unused)]
    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        if self.is_full() {
            Err(Full(value))
        } else {
            self.buf[Self::wrap(self.head + self.len)] = MaybeUninit::new(value);
            self.len += 1;

            Ok(())
        }
    }

    #[inline(always)]
    fn increment(index: usize) -> usize {
        if hint::likely(index + 1 != N) {
            index + 1
        } else {
            0
        }
    }

    #[inline(always)]
    fn wrap(index: usize) -> usize {
        if index < N {
            index
        } else {
            index - N
        }
    }
}
