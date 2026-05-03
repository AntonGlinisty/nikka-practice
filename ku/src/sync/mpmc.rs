#![allow(rustdoc::private_intra_doc_links)]

use core::{
    cmp,
    marker::PhantomData,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use crate::{
    collections::bounded_deque::{
        Error::{
            Empty,
            Full,
        },
        Result,
    },
    memory::size,
};

use super::CacheAligned;

// Used in docs.
#[allow(unused)]
use crate::collections::bounded_deque::Error;

/// Ограниченная конкурентная
/// [не блокирующая](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
/// Multi--Producer--Multi--Consumer (MPMC) очередь
/// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
///
/// - [User API & C++ Implementation of a Multi Producer, Multi Consumer, Lock Free, Atomic Queue](https://www.youtube.com/watch?v=bjz_bMNNWRk).
pub struct Mpmc<T, const CAPACITY: usize>(MpmcU32<CAPACITY>, PhantomData<T>)
where
    T: Into<u32>,
    u32: TryInto<T>;

impl<T, const CAPACITY: usize> Mpmc<T, CAPACITY>
where
    T: Into<u32>,
    u32: TryInto<T>,
{
    /// Создаёт ограниченную конкурентную
    /// [не блокирующую](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
    /// Multi--Producer--Multy--Consumer (MPMC) очередь
    /// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
    pub fn new() -> Self {
        Self(MpmcU32::new(), PhantomData)
    }

    /// Возвращает оценку количества свободных слотов под новые элементы в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Возвращает оценку количества элементов в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Возвращает `true`, если очередь пуста --- в ней не осталось элементов.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Возвращает `true`, если очередь полна ---
    /// в ней не осталось свободных слотов под новые элементы.
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    /// Извлекает элемент из головы очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не оказалось элементов.
    ///
    /// # Panics
    ///
    /// При попытке извлечения был найден элемент `value`,
    /// для которого преобразование `u32::try_into::<T>(T::into::<u32>(value))` возвращает ошибку.
    pub fn dequeue(&self) -> Result<T, ()> {
        self.0.dequeue().map(Self::map_value)
    }

    /// Записывает элемент `value` в хвост очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] в очереди не оказалось свободного места.
    ///
    /// # Panics
    ///
    /// В очереди не оказалось свободного места а
    /// преобразование `u32::try_into::<T>(T::into::<u32>(value))` возвращает ошибку.
    pub fn enqueue(
        &self,
        value: T,
    ) -> Result<(), T> {
        self.0.enqueue(value.into()).map_err(Self::map_error)
    }

    /// Преоборазует хранившееся в [`MpmcU32`] значение `value` в тип `T` элементов [`Mpmc`].
    ///
    /// # Panics
    ///
    /// Паникует, если преобразование `u32::try_into::<T>(T::into::<u32>(value))`
    /// возвращает ошибку.
    fn map_value(value: u32) -> T {
        u32::try_into(value).unwrap_or_else(|_| panic!("{}", Self::INVALID_CONVERSION))
    }

    /// Преоборазует ошибку [`MpmcU32`] из типа [`Error<u32>`] в тип [`Error<T>`] ошибок [`Mpmc`].
    ///
    /// # Panics
    ///
    /// Паникует, если `error` является ошибкой [`Error::Full`]`(value)` и
    /// преобразование `u32::try_into::<T>(T::into::<u32>(value))` возвращает ошибку.
    fn map_error(error: Error<u32>) -> Error<T> {
        match error {
            Full(value) => Full(Self::map_value(value)),
            Empty => Empty,
        }
    }

    /// Сообщение паники при ошибке преобразования `u32::try_into::<T>(T::into::<u32>(...))`.
    const INVALID_CONVERSION: &'static str = "invalid T->u32->T conversion";
}

impl<T, const CAPACITY: usize> Default for Mpmc<T, CAPACITY>
where
    T: Into<u32>,
    u32: TryInto<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> Drop for Mpmc<T, CAPACITY>
where
    T: Into<u32>,
    u32: TryInto<T>,
{
    fn drop(&mut self) {
        while self.dequeue().is_ok() {}
    }
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T, const CAPACITY: usize> Send for Mpmc<T, CAPACITY>
where
    T: Into<u32> + Send,
    u32: TryInto<T>,
{
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T, const CAPACITY: usize> Sync for Mpmc<T, CAPACITY>
where
    T: Into<u32> + Send,
    u32: TryInto<T>,
{
}

/// Ограниченная конкурентная
/// [не блокирующая](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
/// Multi--Producer--Multi--Consumer (MPMC) очередь
/// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
///
/// - [User API & C++ Implementation of a Multi Producer, Multi Consumer, Lock Free, Atomic Queue](https://www.youtube.com/watch?v=bjz_bMNNWRk).
struct MpmcU32<const CAPACITY: usize> {
    /// Циклический буфер с элементами очереди.
    buf: CacheAligned<[AtomicUsize; CAPACITY]>,

    /// Количество элементов, прочитанных из очереди за всё время.
    /// То есть эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`MpmcU32`] значения по модулю размера буфера, чреват ошибками.
    head: CacheAligned<AtomicUsize>,

    /// Количество элементов, записанных в очередь за всё время.
    /// То есть эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`MpmcU32`] значения по модулю размера буфера, чреват ошибками.
    tail: CacheAligned<AtomicUsize>,
}

impl<const CAPACITY: usize> MpmcU32<CAPACITY> {
    /// Создаёт ограниченную конкурентную
    /// [не блокирующую](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
    /// Multi--Producer--Multy--Consumer (MPMC) очередь
    /// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
    fn new() -> Self {
        assert!(
            CAPACITY > 0 && CAPACITY & (CAPACITY - 1) == 0,
            "capacity should be a power of two",
        );
        assert!(CAPACITY <= 1 << Entry::PAYLOAD_BIT_COUNT);

        Self {
            buf: CacheAligned([const { AtomicUsize::new(0) }; CAPACITY]),
            head: CacheAligned(AtomicUsize::new(0)),
            tail: CacheAligned(AtomicUsize::new(0)),
        }
    }

    /// Возвращает оценку количества свободных слотов под новые элементы в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    fn capacity(&self) -> usize {
        CAPACITY - self.len()
    }

    /// Возвращает оценку количества элементов в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    fn len(&self) -> usize {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает `true`, если очередь пуста --- в ней не осталось элементов.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает `true`, если очередь полна ---
    /// в ней не осталось свободных слотов под новые элементы.
    fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    /// Извлекает элемент из головы очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не оказалось элементов.
    fn dequeue(&self) -> Result<u32, ()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает элемент `value` в хвост очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] в очереди не оказалось свободного места.
    fn enqueue(
        &self,
        value: u32,
    ) -> Result<(), u32> {
        // TODO: your code here.
        unimplemented!();
    }

    /// По номеру элемента в воображаемом бесконечном массиве
    /// возвращает эпоху этого элемента в циклическом буфере очереди.
    #[inline(always)]
    fn epoch(x: usize) -> usize {
        (x / CAPACITY) & (Entry::EPOCH_MASK >> Entry::PAYLOAD_BIT_COUNT)
    }

    // TODO: your code here.
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<const CAPACITY: usize> Send for MpmcU32<CAPACITY> {
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<const CAPACITY: usize> Sync for MpmcU32<CAPACITY> {
}

//#[derive(Clone, Copy, Debug, Default)]
/// Элемент или свободный слот циклического буфера [`MpmcU32`].
#[derive(Debug)]
struct Entry(usize);

impl Entry {
    /// Создаёт элемент или свободный слот циклического буфера [`MpmcU32`]
    /// по его значению `entry`.
    #[inline(always)]
    fn new(entry: usize) -> Self {
        Self(entry)
    }

    /// Создаёт элемент циклического буфера [`MpmcU32`]
    /// по его полезной нагрузке `payload` и эпохе `epoch`.
    #[inline(always)]
    fn new_element(
        payload: u32,
        epoch: usize,
    ) -> Self {
        let element =
            Self(size::from(payload) | (epoch << Self::PAYLOAD_BIT_COUNT) | Self::PRESENT_MASK);
        debug_assert_eq!(element.payload(), payload);
        debug_assert_eq!(element.epoch(), epoch);

        element
    }

    /// Создаёт свободный слот для элемента циклического буфера [`MpmcU32`]
    /// по его эпохе `epoch`.
    #[inline(always)]
    fn new_slot(epoch: usize) -> Self {
        Self((epoch << Self::PAYLOAD_BIT_COUNT) & !Self::PRESENT_MASK)
    }

    /// Возвращает значение элемента или свободного слота циклического буфера [`MpmcU32`].
    #[inline(always)]
    fn get(&self) -> usize {
        self.0
    }

    /// Возвращает эпоху элемента или свободного слота циклического буфера [`MpmcU32`].
    #[inline(always)]
    fn epoch(&self) -> usize {
        (self.0 & !Self::PRESENT_MASK) >> Self::PAYLOAD_BIT_COUNT
    }

    /// Возвращает полезную нагрузку элемента циклического буфера [`MpmcU32`].
    #[inline(always)]
    fn payload(&self) -> u32 {
        u32::try_from(self.0 & Self::PAYLOAD_MASK).expect("payload does not fit into u32")
    }

    /// Возвращает `true`, если это элемент, а не свободный слот циклического буфера [`MpmcU32`].
    #[inline(always)]
    fn is_present(&self) -> bool {
        self.0 & Self::PRESENT_MASK != 0
    }

    /// Значение, которое нужно добавить к элементу, чтобы увеличить его эпоху на единицу.
    const EPOCH_INCREMENT: usize = 1 << Self::PAYLOAD_BIT_COUNT;

    /// Маска эпохи.
    const EPOCH_MASK: usize = usize::MAX & !Self::PAYLOAD_MASK & !Self::PRESENT_MASK;

    /// Маска бита, отличающего элемент от свободного слота.
    const PRESENT_MASK: usize = 1 << (usize::BITS - 1);

    /// Количество битов полезной нагрузки.
    const PAYLOAD_BIT_COUNT: u32 = u32::BITS;

    /// Маска полезной нагрузки.
    const PAYLOAD_MASK: usize = Self::EPOCH_INCREMENT - 1;
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use core::sync::atomic::Ordering;

    use super::{
        Entry,
        Mpmc,
        MpmcU32,
    };

    pub unsafe fn new_mpmc<T, const CAPACITY: usize>(start: usize) -> Mpmc<T, CAPACITY>
    where
        T: Into<u32>,
        u32: TryInto<T>,
    {
        type S = MpmcU32<1>;
        let start_epoch = S::epoch(start / CAPACITY) << Entry::PAYLOAD_BIT_COUNT;
        let next_epoch = S::epoch((start / CAPACITY).wrapping_add(1)) << Entry::PAYLOAD_BIT_COUNT;
        let start_index = start % CAPACITY;

        let mpmc = Mpmc::new();

        mpmc.0.head.store(start, Ordering::Relaxed);
        mpmc.0.tail.store(start, Ordering::Relaxed);

        for index in 0 .. CAPACITY {
            let entry = if index < start_index {
                next_epoch
            } else {
                start_epoch
            };
            mpmc.0.buf[index].store(entry, Ordering::Relaxed);
        }

        mpmc
    }

    pub fn mpmc_head<T, const CAPACITY: usize>(mpmc: &Mpmc<T, CAPACITY>) -> usize
    where
        T: Into<u32>,
        u32: TryInto<T>,
    {
        mpmc.0.head.load(Ordering::Relaxed)
    }

    pub fn mpmc_tail<T, const CAPACITY: usize>(mpmc: &Mpmc<T, CAPACITY>) -> usize
    where
        T: Into<u32>,
        u32: TryInto<T>,
    {
        mpmc.0.tail.load(Ordering::Relaxed)
    }
}
