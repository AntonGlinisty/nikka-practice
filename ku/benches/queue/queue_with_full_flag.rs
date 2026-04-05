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
pub struct QueueWithFullFlag<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    full: bool,
    head: usize,
    tail: usize,
}

impl<T, const N: usize> QueueWithFullFlag<T, N> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N],
            full: false,
            head: 0,
            tail: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        N - self.len()
    }

    pub fn len(&self) -> usize {
        if self.full {
            N
        } else if self.tail >= self.head {
            self.tail - self.head
        } else {
            N + self.tail - self.head
        }
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
            self.full = false;

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
            self.buf[self.tail] = MaybeUninit::new(value);
            self.tail = Self::increment(self.tail);
            self.full = self.tail == self.head;

            Ok(())
        }
    }

    #[inline(always)]
    fn increment(index: usize) -> usize {
        // criterion shows the effect of `hint::likely()`:
        //
        // invariants/full_flag<i32>/10
        //                         time:   [2.7036 µs 2.7082 µs 2.7132 µs]
        //                         change: [−30.303% −30.132% −29.973%] (p = 0.00 < 0.05)
        //                         Performance has improved.
        // Found 16 outliers among 100 measurements (16.00%)
        //   8 (8.00%) low mild
        //   3 (3.00%) high mild
        //   5 (5.00%) high severe
        //
        // invariants/full_flag<i32>/16
        //                         time:   [2.7110 µs 2.7201 µs 2.7297 µs]
        //                         change: [−30.560% −30.322% −30.075%] (p = 0.00 < 0.05)
        //                         Performance has improved.
        // Found 5 outliers among 100 measurements (5.00%)
        //   4 (4.00%) high mild
        //   1 (1.00%) high severe
        if hint::likely(index + 1 != N) {
            index + 1
        } else {
            0
        }
    }
}
