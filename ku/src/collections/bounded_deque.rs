use core::{
    mem::MaybeUninit,
    result,
};

use Error::{
    Empty,
    Full,
};

/// Ограниченная двухсторонняя очередь на циклическом буфере.
#[derive(Debug)]
#[rustfmt::skip]
pub struct BoundedDeque<T, const CAPACITY: usize> {
    /// Циклический буфер с элементами очереди.
    buf: [MaybeUninit<T>; CAPACITY],

    /// Количество элементов, прочитанных из очереди за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`BoundedDeque`] значения по модулю размера буфера, чреват ошибками.
    head: usize,

    /// Количество элементов, записанных в очередь за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`BoundedDeque`] значения по модулю размера буфера, чреват ошибками.
    tail: usize,

    full: bool, // TODO: remove before flight.
}

impl<T, const CAPACITY: usize> BoundedDeque<T, CAPACITY> {
    /// Создаёт ограниченную двухстороннюю очередь на циклическом буфере.
    pub fn new() -> Self {
        assert!(
            CAPACITY > 0 && CAPACITY & (CAPACITY - 1) == 0,
            "capacity should be a power of two",
        );

        Self {
            buf: [const { MaybeUninit::uninit() }; CAPACITY],
            head: 0,
            tail: 0,
            full: false, // TODO: remove before flight.
        }
    }

    /// Количество свободных слотов под новые элементы в очереди на текущий момент.
    pub fn capacity(&self) -> usize {
        CAPACITY - self.len()
    }

    /// Количество элементов в очереди на текущий момент.
    pub fn len(&self) -> usize {
        // TODO: your code here.
        if self.full { CAPACITY } else if self.tail >= self.head { self.tail - self.head } else { CAPACITY + self.tail - self.head } // TODO: remove before flight.
    }

    /// Возвращает `true`, если очередь пуста --- в ней не осталось элементов.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает `true`, если очередь полна --- в ней не осталось свободных слотов под новые элементы.
    pub fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    /// Извлекает элемент из головы очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не осталось элементов.
    #[inline(always)]
    pub fn dequeue(&mut self) -> Result<T, T> {
        self.pop_front()
    }

    /// Вставляет элемент в хвост очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] если в очереди не осталось слотов для новых элементов.
    #[inline(always)]
    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        self.push_back(value)
    }

    /// Извлекает элемент из хвоста очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не осталось элементов.
    pub fn pop_back(&mut self) -> Result<T, T> {
        if self.is_empty() {
            Err(Empty)
        } else {
            // TODO: your code here.
            self.tail = Self::decrement(self.tail); self.full = false; Ok(unsafe { self.buf[self.tail].as_ptr().read() }) // TODO: remove before flight.
        }
    }

    /// Извлекает элемент из головы очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не осталось элементов.
    pub fn pop_front(&mut self) -> Result<T, T> {
        if self.is_empty() {
            Err(Empty)
        } else {
            // TODO: your code here.
            let old_head = self.head; self.head = Self::increment(self.head); self.full = false; // TODO: remove before flight.

            Ok(unsafe { self.buf[old_head].as_ptr().read() })
        }
    }

    /// Вставляет элемент в хвост очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] если в очереди не осталось слотов для новых элементов.
    pub fn push_back(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        if self.is_full() {
            Err(Full(value))
        } else {
            // TODO: your code here.
            self.buf[self.tail] = MaybeUninit::new(value); self.tail = Self::increment(self.tail); self.full = self.tail == self.head; // TODO: remove before flight.

            Ok(())
        }
    }

    /// Вставляет элемент в голову очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] если в очереди не осталось слотов для новых элементов.
    pub fn push_front(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        if self.is_full() {
            Err(Full(value))
        } else {
            // TODO: your code here.
            self.head = Self::decrement(self.head); self.full = self.tail == self.head; self.buf[self.head] = MaybeUninit::new(value); // TODO: remove before flight.

            Ok(())
        }
    }

    #[inline(always)] fn decrement(index: usize) -> usize { if core::hint::likely(index != 0) { index - 1 } else { CAPACITY - 1 } } // TODO: remove before flight.

    #[inline(always)] fn increment(index: usize) -> usize { if core::hint::likely(index != CAPACITY - 1) { index + 1 } else { 0 } } // TODO: remove before flight.

    // TODO: your code here.
}

impl<T, const CAPACITY: usize> Default for BoundedDeque<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> Drop for BoundedDeque<T, CAPACITY> {
    fn drop(&mut self) {
        while self.dequeue().is_ok() {}
    }
}

/// Ошибки, которые могут возникать при работе с ограниченной очередью.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<T> {
    /// Очередь пуста --- в ней не осталось элементов.
    Empty,

    /// Очередь переполнена --- в ней не осталось свободных слотов под новые элементы.
    /// Содержит объект, попытка вставки которого в очередь из-за этого не удалась.
    Full(T),
}

/// Тип возвращаемого результата `T` или ошибки [`Error`] ---
/// мономорфизация [`result::Result`] по типу ошибки.
pub type Result<Q, T> = result::Result<Q, Error<T>>;

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use super::BoundedDeque;

    pub fn head<T, const CAPACITY: usize>(bounded_queue: &BoundedDeque<T, CAPACITY>) -> usize {
        bounded_queue.head
    }

    pub fn tail<T, const CAPACITY: usize>(bounded_queue: &BoundedDeque<T, CAPACITY>) -> usize {
        bounded_queue.tail
    }

    pub fn set_head_tail<T, const CAPACITY: usize>(
        bounded_queue: &mut BoundedDeque<T, CAPACITY>,
        start: usize,
    ) {
        bounded_queue.head = start;
        bounded_queue.tail = start;
    }
}
