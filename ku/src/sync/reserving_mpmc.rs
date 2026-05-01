#![allow(rustdoc::private_intra_doc_links)]

use core::{
    cell::UnsafeCell,
    cmp,
    mem::MaybeUninit,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use crate::collections::bounded_deque::{
    Error::{
        Empty,
        Full,
    },
    Result,
};

use super::CacheAligned;

// Used in docs.
#[allow(unused)]
use {
    super::Mpmc,
    crate::collections::bounded_deque::Error,
};

/// Ограниченная конкурентная
/// Multi--Producer--Multi--Consumer (MPMC) очередь
/// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
/// **Не является**
/// [не блокирующей](https://en.wikipedia.org/wiki/Non-blocking_algorithm),
/// в отличие от [`Mpmc`].
pub struct ReservingMpmc<T, const CAPACITY: usize> {
    // TODO: your code here.
    _remove_before_flight: core::marker::PhantomData<T>, // TODO: remove before flight.
}

impl<T, const CAPACITY: usize> ReservingMpmc<T, CAPACITY> {
    /// Создаёт ограниченную конкурентную
    /// [не блокирующую](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
    /// Multi--Producer--Multy--Consumer (MPMC) очередь
    /// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
    pub fn new() -> Self {
        assert!(
            CAPACITY > 1 && CAPACITY & (CAPACITY - 1) == 0,
            "capacity should be a power of two greater than one",
        );

        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает оценку количества свободных слотов под новые элементы в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    pub fn capacity(&self) -> usize {
        CAPACITY - self.len()
    }

    /// Возвращает оценку количества элементов в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    pub fn len(&self) -> usize {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает `true`, если очередь пуста --- в ней не осталось элементов.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает `true`, если очередь полна ---
    /// в ней не осталось свободных слотов под новые элементы.
    pub fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    /// Извлекает элемент из головы очереди.
    /// Не гарантирует сохранение отношения
    /// [happens before](https://en.wikipedia.org/wiki/Happened-before)
    /// для извлекаемых элементов.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не оказалось элементов.
    pub fn dequeue(&self) -> Result<T, T> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает элемент `value` в хвост очереди.
    /// Не гарантирует сохранение отношения
    /// [happens before](https://en.wikipedia.org/wiki/Happened-before)
    /// для извлекаемых элементов.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] в очереди не оказалось свободного места.
    pub fn enqueue(
        &self,
        value: T,
    ) -> Result<(), T> {
        // TODO: your code here.
        unimplemented!();
    }

    // TODO: your code here.
}

impl<T, const CAPACITY: usize> Default for ReservingMpmc<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> Drop for ReservingMpmc<T, CAPACITY> {
    fn drop(&mut self) {
        while self.dequeue().is_ok() {}
    }
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T, const CAPACITY: usize> Send for ReservingMpmc<T, CAPACITY> {
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T, const CAPACITY: usize> Sync for ReservingMpmc<T, CAPACITY> {
}

// TODO: your code here.
