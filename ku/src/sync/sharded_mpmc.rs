#![allow(rustdoc::private_intra_doc_links)]

use core::marker::PhantomData;

use crate::collections::bounded_deque::{
    BoundedDeque,
    Error::{
        Empty,
        Full,
    },
    Result,
};

use super::{
    CacheAligned,
    Spsc,
    spsc::{
        ConsumerTag,
        ProducerTag,
        Tag,
    },
};

// Used in docs.
#[allow(unused)]
use crate::collections::bounded_deque::Error;

/// Ограниченная конкурентная блокирующая
/// Multi--Producer--Multi--Consumer (MPMC) квазиочередь
/// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
/// Не гарантирует сохранение отношения
/// [happens before](https://en.wikipedia.org/wiki/Happened-before)
/// для извлекаемых элементов.
pub struct ShardedMpmc<T, const CAPACITY: usize, const SHARD_COUNT: usize>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Шарды, каждый из которых является ограниченной
    /// Single--Producer--Single--Consumer (SPSC) очередью.
    shards: [Spsc<T, { CAPACITY / SHARD_COUNT }>; SHARD_COUNT],
}

impl<T, const CAPACITY: usize, const SHARD_COUNT: usize> ShardedMpmc<T, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Создаёт ограниченную конкурентную
    /// [не блокирующую](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
    /// Multi--Producer--Multy--Consumer (MPMC) очередь
    /// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
    pub fn new() -> Self {
        let () = BoundedDeque::<(), CAPACITY>::ASSERT_CAPACITY_IS_POWER_OF_TWO;
        let () = Self::ASSERT_SHARD_COUNT_DIVIDES_CAPACITY;

        Self {
            shards: [const { Spsc::new() }; SHARD_COUNT],
        }
    }

    /// Возвращает пару из производителя (писателя) и потребителя (читателя).
    pub fn channel(
        &self
    ) -> (
        ShardedMpmcProducer<'_, T, CAPACITY, SHARD_COUNT>,
        ShardedMpmcConsumer<'_, T, CAPACITY, SHARD_COUNT>,
    ) {
        (
            CacheAligned(ShardedMpmcSide::new(self)),
            CacheAligned(ShardedMpmcSide::new(self)),
        )
    }

    /// Возвращает потребителя (читателя).
    pub fn consumer(&self) -> ShardedMpmcConsumer<'_, T, CAPACITY, SHARD_COUNT> {
        CacheAligned(ShardedMpmcSide::new(self))
    }

    /// Возвращает производителя (писателя).
    pub fn producer(&self) -> ShardedMpmcProducer<'_, T, CAPACITY, SHARD_COUNT> {
        CacheAligned(ShardedMpmcSide::new(self))
    }

    /// Ассоциированная константа, проверяющая, что количество шардов делит ёмкость нацело.
    /// Используется вместо inline `const { ... }`,
    /// т.к. выражения в константном контексте работают некорректно с `generic_const_exprs`.
    /// См.: [https://github.com/rust-lang/rust/issues/124167].
    ///
    /// Не допускает указание не поддерживаемого значения `SHARD_COUNT`:
    ///   - Не поддерживается `SHARD_COUNT == 0`:
    ///     ```compile_fail
    ///     # use ku::sync::ShardedMpmc;
    ///     let _ = ShardedMpmc::<u32, 1, 0>::new();
    ///     ```
    ///   - Не поддерживается `SHARD_COUNT`, не являющаяся степенью двойки:
    ///     ```compile_fail
    ///     # use ku::sync::ShardedMpmc;
    ///     let _ = ShardedMpmc::<u32, 8, 3>::new();
    ///     ```
    ///   - Не поддерживается `SHARD_COUNT > CAPACITY`:
    ///     ```compile_fail
    ///     # use ku::sync::ShardedMpmc;
    ///     let _ = ShardedMpmc::<u32, 1, 2>::new();
    ///     ```
    ///
    /// Допускает только указание `SHARD_COUNT <= CAPACITY`, и являющегося степенью двойки:
    /// ```no_run
    /// # use ku::sync::ShardedMpmc;
    /// let _ = ShardedMpmc::<u32, 16, 2>::new();
    /// ```
    const ASSERT_SHARD_COUNT_DIVIDES_CAPACITY: () = {
        assert!(
            CAPACITY % SHARD_COUNT == 0,
            "shard cound should divide capacity",
        );
    };
}

impl<T, const CAPACITY: usize, const SHARD_COUNT: usize> Default
    for ShardedMpmc<T, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize, const SHARD_COUNT: usize> Drop
    for ShardedMpmc<T, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    fn drop(&mut self) {
        let mut consumer = self.consumer();
        while consumer.dequeue().is_ok() {}
    }
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T, const CAPACITY: usize, const SHARD_COUNT: usize> Send
    for ShardedMpmc<T, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
}

/// Производитель (писатель) или потребитель (читатель).
#[derive(Clone, Copy)]
pub struct ShardedMpmcSide<'side, T, Q: Tag, const CAPACITY: usize, const SHARD_COUNT: usize>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Ссылка на конкурентную [`ShardedMpmc`] очередь,
    /// разделяемую между производителями (писателями) и потребителями (читателями).
    mpmc: &'side ShardedMpmc<T, CAPACITY, SHARD_COUNT>,

    /// Текущий шард.
    shard: usize,

    /// Тег, отличающий производителя (писателя) от потребителя (читателя).
    _tag: PhantomData<Q>,
}

impl<'side, T, Q: Tag, const CAPACITY: usize, const SHARD_COUNT: usize>
    ShardedMpmcSide<'side, T, Q, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Создаёт производителя (писателя) или потребителя (читателя)
    /// для конкурентной очереди `mpmc`.
    fn new(mpmc: &'side ShardedMpmc<T, CAPACITY, SHARD_COUNT>) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает оценку количества свободных слотов под новые элементы в очереди.
    /// Может вернуть не верное значение, но оно гарантированно находится
    /// в диапазоне `0 .. CAPACITY`.
    pub fn capacity(&self) -> usize {
        // TODO: your code here.
        unimplemented!();
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
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<'side, T, Q: Tag, const CAPACITY: usize, const SHARD_COUNT: usize> Send
    for ShardedMpmcSide<'side, T, Q, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
}

impl<'side, T, const CAPACITY: usize, const SHARD_COUNT: usize>
    ShardedMpmcSide<'side, T, ConsumerTag, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Извлекает элемент из головы очереди.
    /// Не гарантирует сохранение отношения
    /// [happens before](https://en.wikipedia.org/wiki/Happened-before)
    /// для извлекаемых элементов.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не оказалось элементов.
    pub fn dequeue(&mut self) -> Result<T, T> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl<'side, T, const CAPACITY: usize, const SHARD_COUNT: usize>
    ShardedMpmcSide<'side, T, ProducerTag, CAPACITY, SHARD_COUNT>
where
    [(); CAPACITY / SHARD_COUNT]:,
{
    /// Записывает элемент `value` в хвост очереди.
    /// Не гарантирует сохранение отношения
    /// [happens before](https://en.wikipedia.org/wiki/Happened-before)
    /// для извлекаемых элементов.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] в очереди не оказалось свободного места.
    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        // TODO: your code here.
        unimplemented!();
    }
}

/// Потребитель (читатель)
/// ограниченной конкурентной
/// Multi--Producer--Multi--Consumer (MPMC) квазиочереди [`ShardedMpmc`].
pub type ShardedMpmcConsumer<'side, T, const CAPACITY: usize, const SHARD_COUNT: usize> =
    CacheAligned<ShardedMpmcSide<'side, T, ConsumerTag, CAPACITY, SHARD_COUNT>>;

/// Производитель (писатель)
/// ограниченной конкурентной
/// Multi--Producer--Multi--Consumer (MPMC) квазиочереди [`ShardedMpmc`].
pub type ShardedMpmcProducer<'side, T, const CAPACITY: usize, const SHARD_COUNT: usize> =
    CacheAligned<ShardedMpmcSide<'side, T, ProducerTag, CAPACITY, SHARD_COUNT>>;
