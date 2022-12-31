#![allow(rustdoc::private_intra_doc_links)]

use core::{
    marker::PhantomData,
    mem,
    result,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use static_assertions::const_assert;

use crate::memory::{
    Block,
    Page,
    Virt,
};

/// Сырая память для
/// [непрерывного циклического буфера](https://fgiesen.wordpress.com/2012/07/21/the-magic-ring-buffer/).
type RingBufferRaw = [u8; RingBuffer::MAPPED_SIZE];

/// Сырая память для
/// [непрерывного циклического буфера](https://fgiesen.wordpress.com/2012/07/21/the-magic-ring-buffer/),
/// выравненная на границу страницы.
#[repr(C, align(4096))]
pub struct RingBufferMemory([u8; RingBuffer::MAPPED_SIZE]);

const_assert!(mem::align_of::<RingBufferMemory>() == Page::SIZE);
const_assert!(mem::size_of::<RingBufferMemory>() % Page::SIZE == 0);

/// [Непрерывный циклический буфер](https://fgiesen.wordpress.com/2012/07/21/the-magic-ring-buffer/).
pub struct RingBuffer {
    /// Виртуальный адрес начала блока памяти с данными, которые хранятся в буфере.
    buf: Virt,

    /// Количество байт, прочитанных из буфера за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`RingBuffer`] значения по модулю размера буфера, чреват ошибками.
    head: AtomicUsize,

    /// Количество байт, записанных в буфер за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`RingBuffer`] значения по модулю размера буфера, чреват ошибками.
    tail: AtomicUsize,

    /// Статистики чтения и записи в буфер.
    stats: [RingBufferStats; 2],
}

impl RingBuffer {
    /// Создаёт [`RingBuffer`] поверх уже отображённой памяти `buf`.
    pub fn new(buf: &'static mut RingBufferMemory) -> Self {
        let buf = &mut buf.0;

        Self::check_continuous_mapping(buf);

        Self {
            buf: Virt::from_ref(buf),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            stats: [RingBufferStats::new(), RingBufferStats::new()],
        }
    }

    /// Создаёт недействительный [`RingBuffer`].
    pub fn zero() -> Self {
        Self {
            buf: Virt::default(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            stats: [RingBufferStats::new(), RingBufferStats::new()],
        }
    }

    /// Возвращает блок виртуальной памяти, в которую отображён буфер.
    pub fn block(&self) -> Block<Virt> {
        if self.buf == Virt::default() {
            Block::default()
        } else {
            Block::from_ref(unsafe { self.buf.try_into_ref::<RingBufferRaw>().unwrap() })
        }
    }

    /// Создаёт читающую транзакцию.
    pub fn read_tx(&mut self) -> RingBufferTx<'_, ReadTag> {
        self.tx()
    }

    /// Создаёт пишущую транзакцию.
    pub fn write_tx(&mut self) -> RingBufferTx<'_, WriteTag> {
        self.tx()
    }

    /// Возвращает статистики читающих транзакций.
    pub fn read_stats(&self) -> &RingBufferStats {
        &self.stats[ReadTag::STATS_INDEX]
    }

    /// Возвращает статистики пишущих транзакций.
    pub fn write_stats(&self) -> &RingBufferStats {
        &self.stats[WriteTag::STATS_INDEX]
    }

    /// Возвращает ссылку на буфер с данными.
    fn buf(&mut self) -> &mut RingBufferRaw {
        if self.buf == Virt::default() {
            panic!("the RingBuffer is not properly initialized");
        } else {
            unsafe { self.buf.try_into_mut().unwrap() }
        }
    }

    /// Создаёт читающую или пишущую транзакцию.
    fn tx<T: Tag>(&mut self) -> RingBufferTx<'_, T> {
        let (head, tail) = self.head_tail();

        self.stats[T::STATS_INDEX].txs.fetch_add(1, Ordering::Relaxed);

        RingBufferTx {
            ring_buffer: self,
            head,
            tail,
            bytes: 0,
            _tag: PhantomData,
        }
    }

    /// Возвращает пару из количество байт,
    /// прочитанных из буфера и записанных в буфер за всё время.
    fn head_tail(&self) -> (usize, usize) {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        (head, tail)
    }

    /// Проверяет, что `buf` отображён в виртуальную память
    /// [непрерывно](https://fgiesen.wordpress.com/2012/07/21/the-magic-ring-buffer/).
    fn check_continuous_mapping(buf: &mut RingBufferRaw) {
        let read_ptr = &buf[0] as *const u8;
        let write_ptr = &mut buf[Self::REAL_SIZE] as *mut u8;

        assert_ne!(read_ptr, write_ptr);

        unsafe {
            let new_value = !read_ptr.read_volatile();
            write_ptr.write_volatile(new_value);
            assert_eq!(
                read_ptr.read_volatile(),
                new_value,
                "RingBuffer is not mapped properly to be continuous",
            );
        }
    }

    /// Размер отображённой виртуальной памяти.
    pub const MAPPED_SIZE: usize = 2 * Self::REAL_SIZE;

    /// Ёмкость циклического буфера.
    pub const REAL_SIZE: usize = 4 * Page::SIZE;
}

/// Читающая или пишущая транзакция.
pub struct RingBufferTx<'a, T: Tag> {
    /// Ссылка на исходный [`RingBuffer`].
    ring_buffer: &'a mut RingBuffer,

    /// Актуальное в рамках транзакции значение количества байт,
    /// прочитанных из буфера за всё время.
    /// В момент старта транзакции инициализируется из поля [`RingBuffer::head`].
    head: usize,

    /// Актуальное в рамках транзакции значение количества байт,
    /// записанных в буфер за всё время.
    /// В момент старта транзакции инициализируется из поля [`RingBuffer::tail`].
    tail: usize,

    /// Количество байт, прочитанных или записанных на текущий момент в данной транзакции.
    bytes: usize,

    /// Тег, отличающий пишущие транзакции от читающих.
    _tag: PhantomData<T>,
}

impl RingBufferTx<'_, ReadTag> {
    /// Возвращает в виде среза все доступные на момент запуска читающей транзакции байты из буфера.
    /// Обновляет поля только самой транзакции [`RingBufferTx`].
    pub fn read(&mut self) -> &[u8] {
        // TODO: your code here.
        unimplemented!();
    }

    /// Фиксирует читающую транзакцию, записывая обновлённое значение [`RingBuffer::head`] и
    /// статистику [`RingBuffer::read_stats()`] в поля [`RingBuffer`].
    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl RingBufferTx<'_, WriteTag> {
    /// Копирует в буфер байты среза `data`.
    /// Обновляет поля самой транзакции [`RingBufferTx`] и статистики [`RingBuffer::write_stats()`],
    /// но не трогает поля [`RingBuffer::head`] и [`RingBuffer::tail`].
    /// Если в буфере не остаётся места под `data`, возвращает ошибку [`Error::Overflow`].
    pub fn write(
        &mut self,
        data: &[u8],
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Фиксирует пишущую транзакцию, обновляя значение [`RingBuffer::tail`] и
    /// статистику [`RingBuffer::write_stats()`] в полях [`RingBuffer`].
    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Ёмкость, оставшаяся в буфере транзакции на текущий момент.
    pub fn capacity(&self) -> usize {
        RingBuffer::REAL_SIZE - (self.tail - self.head)
    }
}

impl<T: Tag> Drop for RingBufferTx<'_, T> {
    /// Обрывает читающую или пишущую транзакцию.
    /// Обновляет соответствующие статистики [`RingBuffer::stats`],
    /// если в транзакции был прочитан или записан хотя бы один байт.
    fn drop(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

/// Тег, отличающий пишущие транзакции от читающих.
pub trait Tag {
    /// Индекс соответствующих статистик в массиве [`RingBuffer::stats`].
    const STATS_INDEX: usize;
}

/// Читающая транзакция.
pub type RingBufferReadTx<'a> = RingBufferTx<'a, ReadTag>;

/// Пишущая транзакция.
pub type RingBufferWriteTx<'a> = RingBufferTx<'a, WriteTag>;

/// Тег читающих транзакций.
pub struct ReadTag;

impl Tag for ReadTag {
    const STATS_INDEX: usize = 0;
}

/// Тег пишущих транзакций.
pub struct WriteTag;

impl Tag for WriteTag {
    const STATS_INDEX: usize = 1;
}

const_assert!(ReadTag::STATS_INDEX != WriteTag::STATS_INDEX);

/// Статистики чтения или записи в буфер.
#[derive(Debug)]
pub struct RingBufferStats {
    /// Количество байт, прочитанных или записанных в зафиксированных транзакциях.
    committed: AtomicUsize,

    /// Количество зафиксированных транзакций соответствующего типа.
    commits: AtomicUsize,

    /// Количество байт, прочитанных или записанных в оборванных транзакциях.
    dropped: AtomicUsize,

    /// Количество отменённых транзакций соответствующего типа.
    drops: AtomicUsize,

    /// Количество ошибок в транзакциях.
    errors: AtomicUsize,

    /// Количество транзакций чтения либо записи соответственно.
    txs: AtomicUsize,
}

impl RingBufferStats {
    /// Создаёт статистики чтения или записи в буфер.
    fn new() -> Self {
        Self {
            committed: AtomicUsize::new(0),
            commits: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
            drops: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            txs: AtomicUsize::new(0),
        }
    }
}

/// Ошибки, которые могут возникать при работе с [`RingBuffer`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Буфер транзакции переполнен.
    Overflow {
        /// Остававшееся в буфере место на момент старта транзакции.
        /// То есть, полная доступная для транзакции ёмкость.
        capacity: usize,

        /// Уже записанный ранее в рамках транзакции объём.
        len: usize,

        /// Размер объекта, при попытке записи которого буфер транзакции переполнился.
        /// Должно выполняться неравенство `exceeding_object_len > capacity - len`.
        exceeding_object_len: usize,
    },
}

/// Тип возвращаемого результата `T` или ошибки [`Error`] ---
/// мономорфизация [`result::Result`] по типу ошибки.
pub type Result<T> = result::Result<T, Error>;
