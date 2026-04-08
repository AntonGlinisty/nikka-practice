#![allow(rustdoc::private_intra_doc_links)]

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use derive_more::{
    Deref,
    DerefMut,
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
use crate::collections::bounded_deque::Error;

/// Ограниченная конкурентная Single--Producer--Single--Consumer (SPSC) очередь
/// [без ожидания](https://en.wikipedia.org/wiki/Non-blocking_algorithm#Wait-freedom)
/// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
///
/// При необходимости устранить
/// [ложную конкуренцию](https://en.wikipedia.org/wiki/False_sharing)
/// за элементы очереди, можно дополнительно использовать [`CacheAligned`]:
/// ```rust
/// # use ku::sync::{
/// #     CacheAligned,
/// #     Spsc,
/// # };
/// #
/// # fn f() {
/// let spsc = Spsc::<CacheAligned<usize>, 4>::new();
/// #     drop(spsc);
/// # }
/// ```
///
/// - ["Cooperating sequential processes, section 4.3. The Bounded Buffer", Dijkstra, 1965](https://www.cs.utexas.edu/~EWD/transcriptions/EWD01xx/EWD123.html#4.3.%20The%20Bounded%20Buffer.).
/// - ["Proving the Correctness of Multiprocess Programs", Leslie Lamport, 1977](https://lamport.azurewebsites.net/pubs/proving.pdf).
/// - ["Correct and Efficient Bounded FIFO Queues", Nhat Minh Lê & Adrien Guatto & Albert Cohen & Antoniu Pop, 2013](https://www.irif.fr/~guatto/publications/sbac13.pdf).
/// - [Single Producer Single Consumer Lock-free FIFO From the Ground Up - Charles Frasch](https://www.youtube.com/watch?v=K3P_Lmq6pw0).
/// - ["Optimizing a ring buffer for throughput", Erik Rigtorp, 2021](https://rigtorp.se/ringbuffer/). [Код SPSC очереди](https://github.com/rigtorp/SPSCQueue).
/// - [Folly's SPSC](https://github.com/facebook/folly/blob/main/folly/ProducerConsumerQueue.h).
/// - [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html).
/// - [`heapless::spsc`](https://docs.rs/heapless/latest/heapless/spsc/index.html).
/// - [Проблема "производитель-потребитель"](https://en.wikipedia.org/wiki/Producer%E2%80%93consumer_problem).
#[rustfmt::skip]
pub struct Spsc<T, const CAPACITY: usize> {
    /// Циклический буфер с элементами очереди.
    // TODO: your code here.
    buf: [UnsafeCell<MaybeUninit<T>>; CAPACITY], // TODO: remove before flight.

    /// Количество элементов, прочитанных из очереди за всё время.
    /// То есть эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`Spsc`] значения по модулю размера буфера, чреват ошибками.
    // TODO: your code here.
    head: AtomicUsize, // TODO: remove before flight.

    /// Количество элементов, записанных в очередь за всё время.
    /// То есть эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`Spsc`] значения по модулю размера буфера, чреват ошибками.
    // TODO: your code here.
    tail: AtomicUsize, // TODO: remove before flight.

    // TODO: your code here.
}

impl<T, const CAPACITY: usize> Spsc<T, CAPACITY> {
    /// Создаёт ограниченную конкурентную Single--Producer--Single--Consumer (SPSC) очередь
    /// [без ожидания](https://en.wikipedia.org/wiki/Non-blocking_algorithm#Wait-freedom)
    /// на основе [циклического буфера](https://en.wikipedia.org/wiki/Circular_buffer).
    pub const fn new() -> Self {
        assert!(
            CAPACITY > 0 && CAPACITY & (CAPACITY - 1) == 0,
            "capacity should be a power of two",
        );

        Self {
            // TODO: your code here.
            buf: [const { UnsafeCell::new(MaybeUninit::uninit()) }; CAPACITY], // TODO: remove before flight.
            head: AtomicUsize::new(0), // TODO: remove before flight.
            tail: AtomicUsize::new(0), // TODO: remove before flight.
        }
    }

    /// Возвращает пару из производителя (писателя) и потребителя (читателя).
    pub fn channel(
        &self
    ) -> Option<(SpscProducer<'_, T, CAPACITY>, SpscConsumer<'_, T, CAPACITY>)> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает потребителя (читателя).
    pub fn consumer(&self) -> Option<SpscConsumer<'_, T, CAPACITY>> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает производителя (писателя).
    pub fn producer(&self) -> Option<SpscProducer<'_, T, CAPACITY>> {
        // TODO: your code here.
        unimplemented!();
    }

    // TODO: your code here.
}

impl<T, const CAPACITY: usize> Default for Spsc<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> Drop for Spsc<T, CAPACITY> {
    fn drop(&mut self) {
        let (_, mut consumer) = self
            .channel()
            .expect("Spsc has been dropped whilst either producer or consumer is alive");
        while consumer.dequeue().is_ok() {}
    }
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<T: Send, const CAPACITY: usize> Send for Spsc<T, CAPACITY> {
}

/// Производитель (писатель) или потребитель (читатель).
///
/// Кэширует позиции и собственного и противоположного конца очереди ---
/// [`Spsc::head`] и [`Spsc::tail`].
/// Чтобы уменьшить количество относительно медленных атомарных операций с
/// разделяемым состоянием [`Spsc`].
/// Которые приводят к передачам сообщений
/// [протокола поддержки когерентности кэшей](https://en.wikipedia.org/wiki/Cache_coherency_protocols_(examples)),
/// и расходуют пропускную способность
/// [интерконнекта](https://en.wikipedia.org/wiki/HyperTransport).
#[rustfmt::skip]
pub struct SpscSide<'side, T, Q: Tag, const CAPACITY: usize> {
    /// Ссылка на конкурентную [`Spsc`] очередь,
    /// разделяемую между производителем (писателем) и потребителем (читателем).
    spsc: &'side Spsc<T, CAPACITY>,

    // TODO: your code here.

    /// Тег, отличающий производителя (писателя) от потребителя (читателя).
    _tag: PhantomData<Q>,
}

impl<'side, T, Q: Tag, const CAPACITY: usize> SpscSide<'side, T, Q, CAPACITY> {
    /// Создаёт производителя (писателя) или потребителя (читателя)
    /// для конкурентной очереди `spsc`.
    fn new(spsc: &'side Spsc<T, CAPACITY>) -> Self {
        Self {
            spsc,
            // TODO: your code here.
            _tag: PhantomData,
        }
    }

    /// Возвращает количество свободных слотов под новые элементы в очереди
    /// с точки зрения закэшированной позиции противоположного конца.
    /// То есть, может вернуть не актуальное значение.
    ///
    /// См. [`SpscSide::update_len()`] или [`SpscSide::update_capacity()`] для
    /// актуализации позиции противоположного конца ---
    /// производителя (писателя) или потребителя (читателя).
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        CAPACITY - self.len()
    }

    /// Возвращает количество элементов в очереди
    /// с точки зрения закэшированной позиции противоположного конца.
    /// То есть, может вернуть не актуальное значение.
    ///
    /// См. [`SpscSide::update_len()`] или [`SpscSide::update_capacity()`] для
    /// актуализации позиции противоположного конца ---
    /// производителя (писателя) или потребителя (читателя).
    #[inline(always)]
    pub fn len(&self) -> usize {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает `true`, если очередь пуста --- в ней не осталось элементов ---
    /// с точки зрения закэшированной позиции противоположного конца.
    /// То есть, может вернуть не актуальное значение.
    ///
    /// См. [`SpscSide::update_len()`] или [`SpscSide::update_capacity()`] для
    /// актуализации позиции противоположного конца ---
    /// производителя (писателя) или потребителя (читателя).
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Возвращает `true`, если очередь полна ---
    /// в ней не осталось свободных слотов под новые элементы ---
    /// с точки зрения закэшированной позиции противоположного конца.
    /// То есть, может вернуть не актуальное значение.
    ///
    /// См. [`SpscSide::update_len()`] или [`SpscSide::update_capacity()`] для
    /// актуализации позиции противоположного конца ---
    /// производителя (писателя) или потребителя (читателя).
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.capacity() == 0
    }

    /// Фиксирует читающую или пишущую транзакцию.
    ///
    /// Записывает обновлённое значение [`Spsc::head`] или [`Spsc::tail`] соответственно.
    fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Создаёт читающую или пишущую транзакцию.
    #[inline(always)]
    fn start_tx<'tx>(&'tx mut self) -> SpscTx<'side, 'tx, T, Q, CAPACITY> {
        SpscTx(self)
    }

    // TODO: your code here.
}

impl<'side, T, Q: Tag, const CAPACITY: usize> Drop for SpscSide<'side, T, Q, CAPACITY> {
    fn drop(&mut self) {
        // TODO: your code here.
    }
}

/// См. [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl<'side, T: Send, Q: Tag, const CAPACITY: usize> Send
    for SpscSide<'side, T, Q, CAPACITY>
{
}

impl<'side, T, const CAPACITY: usize> SpscSide<'side, T, ConsumerTag, CAPACITY> {
    /// Извлекает элемент из головы очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не осталось элементов
    ///     с точки зрения закэшированной позиции противоположного конца.
    #[inline(always)]
    pub fn dequeue(&mut self) -> Result<T, T> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Создаёт читающую транзакцию.
    #[inline(always)]
    pub fn tx<'tx>(&'tx mut self) -> SpscTx<'side, 'tx, T, ConsumerTag, CAPACITY> {
        if self.is_empty() {
            self.update_len();
        }

        self.start_tx()
    }

    /// Обновляет закэшированную позицию `SpscSide::tail` противоположного конца
    /// разделяемой очереди --- производителя (писателя).
    /// После этого вызова потребителю (читателю) становятся доступными новые элементы.
    #[inline(always)]
    pub fn update_len(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    // TODO: your code here.
}

impl<'side, T: Copy, const CAPACITY: usize> SpscSide<'side, T, ConsumerTag, CAPACITY> {
    /// Обрывает читающую транзакцию.
    ///
    /// См. также [`SpscTx::abort()`].
    fn abort(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl<'side, T, const CAPACITY: usize> SpscSide<'side, T, ProducerTag, CAPACITY> {
    /// Обрывает пишущую транзакцию.
    ///
    /// При необходимости вызывает `T::drop()` для элементов,
    /// вставленных в обрываемой транзакции.
    fn abort(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает элемент `value` в хвост очереди.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] если в очереди не осталось места
    ///     с точки зрения закэшированной позиции противоположного конца.
    #[inline(always)]
    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Создаёт пишущую транзакцию.
    #[inline(always)]
    pub fn tx<'tx>(&'tx mut self) -> SpscTx<'side, 'tx, T, ProducerTag, CAPACITY> {
        if self.is_full() {
            self.update_capacity();
        }

        self.start_tx()
    }

    /// Актуализирует закэшированную позицию противоположного конца --- потребителя (читателя).
    /// Обновляет закэшированную позицию `SpscSide::head` противоположного конца
    /// разделяемой очереди --- потребителя (читателя).
    /// После этого вызова производителю (писателю) становятся доступными
    /// новые свободные слоты под элементы.
    #[inline(always)]
    pub fn update_capacity(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    // TODO: your code here.
}

/// Читающая или пишущая транзакция.
#[derive(Deref, DerefMut)]
pub struct SpscTx<'side: 'tx, 'tx, T, Q: Tag, const CAPACITY: usize>(
    &'tx mut SpscSide<'side, T, Q, CAPACITY>,
);

impl<T, Q: Tag, const CAPACITY: usize> SpscTx<'_, '_, T, Q, CAPACITY> {
    /// Фиксирует читающую или пишущую транзакцию.
    ///
    /// Записывает обновлённое значение [`Spsc::head`] или [`Spsc::tail`] соответственно.
    #[inline(always)]
    pub fn commit(self) {
        self.0.commit();
    }
}

impl<T, const CAPACITY: usize> SpscTx<'_, '_, T, ConsumerTag, CAPACITY> {
    /// Извлекает элемент из головы очереди.
    /// Обновляет поля потребителя (читателя) [`SpscSide`],
    /// но не трогает поля [`Spsc::head`] и [`Spsc::tail`].
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Empty`] если в очереди не осталось элементов
    ///     с точки зрения закэшированной позиции противоположного конца.
    #[inline(always)]
    pub fn dequeue(&mut self) -> Result<T, T> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Обновляет закэшированную позицию `SpscSide::tail` противоположного конца
    /// разделяемой очереди --- производителя (писателя).
    /// После этого вызова потребителю (читателю) становятся доступными новые элементы.
    #[inline(always)]
    pub fn update_len(&mut self) {
        self.0.update_len();
    }
}

impl<T: Copy, const CAPACITY: usize> SpscTx<'_, '_, T, ConsumerTag, CAPACITY> {
    /// Обрывает читающую транзакцию.
    ///
    /// Это можно делать только если тип элемента `T` поддерживает типаж [`Copy`].
    /// Иначе, можно было бы получить две копии одного элемента в таком коде:
    /// ```rust
    /// # use std::fmt::Debug;
    /// #
    /// # use ku::sync::{
    /// #     CacheAligned,
    /// #     Spsc,
    /// # };
    /// #
    /// # #[derive(Clone, Copy, Debug)]
    /// # struct T;
    /// #
    /// # impl T {
    /// #     fn new() -> Self {
    /// #         Self
    /// #     }
    /// # }
    /// #
    /// # fn f() {
    /// let spsc = Spsc::<T, 4>::new();
    /// let (mut producer, mut consumer) = spsc.channel().unwrap();
    /// producer.enqueue(T::new()).unwrap();
    ///
    /// let mut tx = consumer.tx();
    /// let element = tx.dequeue().unwrap();
    /// tx.abort();
    ///
    /// let element_copy = consumer.dequeue().unwrap();
    /// # }
    /// ```
    #[inline(always)]
    pub fn abort(self) {
        self.0.abort();
    }
}

impl<T, const CAPACITY: usize> SpscTx<'_, '_, T, ProducerTag, CAPACITY> {
    /// Обрывает пишущую транзакцию.
    #[inline(always)]
    pub fn abort(self) {
        self.0.abort();
    }

    /// Записывает элемент `value` в хвост очереди.
    ///
    /// Обновляет поля производителя (писателя) [`SpscSide`] и
    /// разделяемый циклический буфер статистики [`Spsc::buf`],
    /// но не трогает поля [`Spsc::head`] и [`Spsc::tail`].
    ///
    /// Возвращает ошибку:
    ///   - [`Error::Full`] если в очереди не осталось места
    ///     с точки зрения закэшированной позиции противоположного конца.
    #[inline(always)]
    pub fn enqueue(
        &mut self,
        value: T,
    ) -> Result<(), T> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl<T, Q: Tag, const CAPACITY: usize> Drop for SpscTx<'_, '_, T, Q, CAPACITY> {
    /// Фиксирует читающую или пишущую транзакцию.
    fn drop(&mut self) {
        self.0.commit();
    }
}

/// Один из концов очереди --- либо производитель (писатель), либо потребитель (читатель).
pub enum Side {
    /// Потребитель (читатель).
    Consumer,

    /// Производитель (писатель).
    Producer,
}

/// Тег, отличающий производителя (писателя) от потребителя (читателя).
pub trait Tag {
    /// Соответствующий конец очереди ---
    /// либо производитель (писатель), либо потребитель (читатель).
    const SIDE: Side;
}

/// Потребитель (читатель)
/// ограниченной конкурентной Single--Producer--Single--Consumer (SPSC) очереди [`Spsc`].
///
/// [`CacheAligned`] нужен чтобы можно было перевать [`SpscConsumer`] в поток по ссылке и
/// не наступить на грабли при этом.
/// Иначе один забытый `move` у замыкания может на порядок просадить производительность.
/// В целом, чтобы пользовательский код был проще, --- чтобы он не заботился
/// о таких моментах самостоятельно и был менее подвержен ошибкам.
pub type SpscConsumer<'side, T, const CAPACITY: usize> =
    CacheAligned<SpscSide<'side, T, ConsumerTag, CAPACITY>>;

/// Производитель (писатель)
/// ограниченной конкурентной Single--Producer--Single--Consumer (SPSC) очереди [`Spsc`].
///
/// [`CacheAligned`] нужен чтобы можно было перевать [`SpscProducer`] в поток по ссылке и
/// не наступить на грабли при этом.
/// Иначе один забытый `move` у замыкания может на порядок просадить производительность.
/// В целом, чтобы пользовательский код был проще, --- чтобы он не заботился
/// о таких моментах самостоятельно и был менее подвержен ошибкам.
pub type SpscProducer<'side, T, const CAPACITY: usize> =
    CacheAligned<SpscSide<'side, T, ProducerTag, CAPACITY>>;

/// Читающая транзакция
/// ограниченной конкурентной Single--Producer--Single--Consumer (SPSC) очереди [`Spsc`].
pub type SpscConsumeTx<'side, 'tx, T, const CAPACITY: usize> =
    SpscTx<'side, 'tx, T, ConsumerTag, CAPACITY>;

/// Пишущая транзакция
/// ограниченной конкурентной Single--Producer--Single--Consumer (SPSC) очереди [`Spsc`].
pub type SpscProduceTx<'side, 'tx, T, const CAPACITY: usize> =
    SpscTx<'side, 'tx, T, ProducerTag, CAPACITY>;

/// Тег читающих транзакций.
#[derive(Clone)]
pub struct ConsumerTag;

impl Tag for ConsumerTag {
    const SIDE: Side = Side::Consumer;
}

/// Тег пишущих транзакций.
#[derive(Clone)]
pub struct ProducerTag;

impl Tag for ProducerTag {
    const SIDE: Side = Side::Producer;
}

// TODO: your code here.
