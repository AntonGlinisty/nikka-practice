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
pub struct QueueWithSentinel<T, const N: usize>
where
    [(); N + 1]:,
{
    buf: [MaybeUninit<T>; N + 1],
    head: usize,
    tail: usize,
}

impl<T, const N: usize> QueueWithSentinel<T, N>
where
    [(); N + 1]:,
{
    pub fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; N + 1],
            head: 0,
            tail: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        N - self.len()
    }

    pub fn len(&self) -> usize {
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            N + 1 + self.tail - self.head
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    pub fn dequeue(&mut self) -> Result<T, T> {
        if self.is_empty() {
            Err(Empty)
        } else {
            let old_head = self.head;
            self.head = Self::increment(self.head);

            Ok(unsafe { self.buf[old_head].as_ptr().read() })
        }
    }

    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        if self.is_full() {
            Err(Full(value))
        } else {
            self.buf[self.tail] = MaybeUninit::new(value);
            self.tail = Self::increment(self.tail);

            Ok(())
        }
    }

    #[inline(always)]
    fn increment(index: usize) -> usize {
        // criterion shows the effect of `hint::likely()`:
        //
        // invariants/sentinel<i32>/10
        //                         time:   [2.1473 µs 2.1514 µs 2.1561 µs]
        //                         change: [−23.003% −22.744% −22.472%] (p = 0.00 < 0.05)
        //                         Performance has improved.
        // Found 10 outliers among 100 measurements (10.00%)
        //   5 (5.00%) high mild
        //   5 (5.00%) high severe
        //
        // invariants/sentinel<i32>/16
        //                         time:   [2.1627 µs 2.1659 µs 2.1697 µs]
        //                         change: [−23.094% −22.879% −22.656%] (p = 0.00 < 0.05)
        //                         Performance has improved.
        // Found 6 outliers among 100 measurements (6.00%)
        //   4 (4.00%) high mild
        //   2 (2.00%) high severe
        if hint::likely(index != N) {
            index + 1
        } else {
            0
        }
    }
}
