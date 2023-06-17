#![allow(rustdoc::private_intra_doc_links)]

use core::{
    cmp,
    marker::PhantomData,
    mem,
    result,
    sync::atomic::{
        AtomicU8,
        Ordering,
    },
};

use static_assertions::const_assert;

use crate::{
    error::Error::{
        InvalidAlignment,
        Null,
    },
    log::error,
    memory::{
        Block,
        Page,
        Virt,
        size,
    },
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

/// Создаёт [`RingBuffer`] поверх уже отображённой памяти `buf`.
///
/// Возвращает пару из [`ReadBuffer`] и [`WriteBuffer`] ---
/// интерфейсом читателя и интерфейсом писателя для этого буфера.
pub fn make(buf: &'static mut RingBufferMemory) -> (ReadBuffer, WriteBuffer) {
    // TODO: your code here.
    Ok((ReadBuffer::default(), WriteBuffer::default())) // TODO: remove before flight.
}

/// [Непрерывный циклический буфер](https://fgiesen.wordpress.com/2012/07/21/the-magic-ring-buffer/).
#[derive(Debug, Default)]
pub struct RingBuffer {
    /// Виртуальный адрес начала блока памяти с данными, которые хранятся в буфере.
    buf: Virt,

    /// Количество байт, прочитанных из буфера за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`RingBuffer`] значения по модулю размера буфера, чреват ошибками.
    ///
    /// Аналогично, все методы работают с позициями в буфере,
    /// измеряя их от момента инициализации буфера.
    /// Единственное преобразование таких позиций в смещения в памяти, ---
    /// взятие по модулю `Self::REAL_SIZE`, --- выполняется в реализации метода [`RingBuffer::into()`].
    head: usize,

    /// Количество байт, записанных в буфер за всё время.
    /// То есть, эта величина потенциально больше размера буфера.
    /// Вариант хранить в [`RingBuffer`] значения по модулю размера буфера, чреват ошибками.
    ///
    /// Аналогично, все методы работают с позициями в буфере,
    /// измеряя их от момента инициализации буфера.
    /// Единственное преобразование таких позиций в смещения в памяти, ---
    /// взятие по модулю `Self::REAL_SIZE`, --- выполняется в реализации метода [`RingBuffer::into()`].
    tail: usize,

    /// Статистики чтения или записи в буфер.
    stats: RingBufferStats,
}

impl RingBuffer {
    /// Инициализирует буфер по адресу `buf`.
    fn new(buf: Virt) -> Self {
        assert_ne!(buf, Virt::default());

        Self {
            buf,
            head: 0,
            tail: 0,
            stats: RingBufferStats::default(),
        }
    }

    /// Возвращает блок виртуальной памяти, в которую отображён буфер.
    fn block(&self) -> Block<Virt> {
        if self.buf == Virt::default() {
            Block::default()
        } else {
            Block::from_ref(unsafe { self.buf.try_into_ref::<RingBufferRaw>().unwrap() })
        }
    }

    /// Создаёт читающую или пишущую транзакцию.
    ///
    /// Возвращает [`None`], если [`RingBuffer`] был уже закрыт методом [`RingBuffer::close()`].
    fn tx<T: Tag>(&mut self) -> Option<RingBufferTx<'_, T>> {
        let (head, tail) = self.head_tail()?;

        self.stats.txs += 1;

        Some(RingBufferTx {
            ring_buffer: self,
            header: 0,
            head,
            tail,
            bytes: 0,
            _tag: PhantomData,
        })
    }

    /// Закрывает [`RingBuffer`].
    /// После вызова этой функции любым из участников никакие транзакции больше создаваться не могут.
    /// А соотвествующий метод [`RingBuffer::tx()`] возвращает [`None`].
    fn close(&mut self) {
        self.write_header(self.tail, Header::Closed);
    }

    /// Читает заголовок записи, находящейся на позиции `position`.
    /// В случае, если заголовок записи некорректен,
    /// считает что противоположная сторона закрыла буфер и возвращает
    /// [`Header::Closed`].
    fn read_header(
        &mut self,
        position: usize,
    ) -> Header {
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает заголовок записи, находящейся на позиции `position`.
    fn write_header(
        &mut self,
        position: usize,
        header: Header,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Читает из заголовка записи, находящейся на позиции `position`, её размер.
    fn read_size(
        &mut self,
        position: usize,
    ) -> usize {
        let mut size = 0;
        for x in self.header(position) {
            size = (size << u8::BITS) | size::from(*x);
        }
        size
    }

    /// Записывает в заголовок записи, находящейся на позиции `position`, её размер `size`.
    fn write_size(
        &mut self,
        position: usize,
        size: usize,
    ) {
        let mut size = size;
        for x in self.header(position).iter_mut().rev() {
            *x = size as u8;
            size >>= u8::BITS;
        }
        assert_eq!(size, 0);
    }

    /// Возвращает ссылку на флаг состояния записи,
    /// находящейся на позиции `position`.
    ///
    /// Принимает `&mut self`, чтобы гарантировать отсутствие алиасинга.
    fn state(
        &mut self,
        position: usize,
    ) -> &AtomicU8 {
        self.into(position)
    }

    /// Возвращает ссылку на заголовок с метаданными записи,
    /// находящейся на позиции `position`.
    ///
    /// Принимает `&mut self`, чтобы гарантировать отсутствие алиасинга.
    fn header(
        &mut self,
        position: usize,
    ) -> &mut [u8; Self::HEAD_SIZE - mem::size_of::<AtomicU8>()] {
        self.into(position + mem::size_of::<AtomicU8>())
    }

    /// Возвращает ссылку на буфер с данными,
    /// находящийся на позиции `position`.
    ///
    /// Принимает `&mut self`, чтобы гарантировать отсутствие алиасинга.
    fn buf(
        &mut self,
        position: usize,
    ) -> &mut [u8; Self::REAL_SIZE] {
        self.into(position)
    }

    /// Возвращает ссылку на тип `T`, записанный на позиции `position`.
    /// Позиция `position` измеряется от момента инициализации буфера,
    /// то есть не учитывает переполнения размера буфера.
    /// Тип `T` должен влезать в буфер и требовать тривиального выравнивания.
    ///
    /// # Panics
    ///
    /// Паникует:
    /// - Если память под буфер не выделена.
    /// - Тип `T` не влезает в буфер.
    /// - Тип `T` требует нетривиального выравнивания.
    ///
    /// Принимает `&mut self`, чтобы гарантировать отсутствие алиасинга.
    fn into<T>(
        &mut self,
        position: usize,
    ) -> &mut T {
        assert!(mem::size_of::<T>() <= Self::REAL_SIZE);

        let offset = position % Self::REAL_SIZE;
        let virt =
            (self.buf + offset).expect("the RingBuffer does not fit into address space properly");

        match unsafe { virt.try_into_mut() } {
            Ok(value) => value,
            Err(Null) => panic!("the RingBuffer is not properly initialized"),
            Err(InvalidAlignment) => panic!("type T should be trivially aligned"),
            Err(error) => panic!("unexpected error {:?}", error),
        }
    }

    /// Возвращает пару из количество байт,
    /// прочитанных из буфера и записанных в буфер за всё время.
    /// Если буфер закрыт, возвращает [`None`].
    /// Если замечает нарушение инвариантов буфера,
    /// считает что противоположная сторона испортила буфер и
    /// расценивает его как закрытый.
    fn head_tail(&mut self) -> Option<(usize, usize)> {
        // TODO: your code here.
        unimplemented!();
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

    /// Вычисляет размер заголовка одной записи для `RingBuffer` размера `Self::REAL_SIZE`.
    const fn head_size() -> usize {
        let mut head_size = mem::size_of::<AtomicU8>();
        let mut size = Self::REAL_SIZE;

        while size > 0 {
            head_size += mem::size_of::<u8>();
            size >>= u8::BITS;
        }

        head_size
    }

    /// Размер отображённой виртуальной памяти.
    pub const MAPPED_SIZE: usize = 2 * Self::REAL_SIZE;

    /// Ёмкость циклического буфера.
    pub const REAL_SIZE: usize = 4 * Page::SIZE;

    /// Размер заголовка одной записи в буфере.
    pub const HEAD_SIZE: usize = Self::head_size();

    /// Размер хвоста после конца записи в буфере --- флага состояния следующей записи.
    /// Используется при записи в буфер, чтобы отметить следующую запись свободной.
    /// То есть, чтобы читающая сторона не пыталась читать следующую запись до тех пор,
    /// пока та не будет записана на самом деле.
    pub const TAIL_SIZE: usize = mem::size_of::<AtomicU8>();

    /// Максимальный размер полезной нагрузки в одной записи.
    pub const MAX_CAPACITY: usize = Self::REAL_SIZE - Self::HEAD_SIZE - Self::TAIL_SIZE;
}

/// Читающая сторона [`RingBuffer`].
/// Позволяет создавать только читающие транзакции.
#[derive(Debug, Default)]
pub struct ReadBuffer(RingBuffer);

impl ReadBuffer {
    /// Создаёт читающую транзакцию.
    pub fn read_tx(&mut self) -> Option<RingBufferReadTx<'_>> {
        let mut tx = self.0.tx()?;
        tx.header = tx.head;
        Some(tx)
    }

    /// Закрывает [`RingBuffer`].
    /// После вызова этой функции никакие транзакции больше создаваться не могут.
    pub fn close(&mut self) {
        self.0.close();
    }

    /// Возвращает блок виртуальной памяти, в которую отображён буфер.
    pub fn block(&self) -> Block<Virt> {
        self.0.block()
    }

    /// Возвращает статистики читающих транзакций.
    pub fn read_stats(&self) -> &RingBufferStats {
        &self.0.stats
    }
}

/// Пишущая сторона [`RingBuffer`].
/// Позволяет создавать только пишущие транзакции.
#[derive(Debug, Default)]
pub struct WriteBuffer(RingBuffer);

impl WriteBuffer {
    /// Создаёт пишущую транзакцию.
    pub fn write_tx(&mut self) -> Option<RingBufferWriteTx<'_>> {
        let mut tx = self.0.tx()?;
        tx.header = tx.tail;
        Some(tx)
    }

    /// Закрывает [`RingBuffer`].
    /// После вызова этой функции никакие транзакции больше создаваться не могут.
    pub fn close(&mut self) {
        self.0.close();
    }

    /// Возвращает блок виртуальной памяти, в которую отображён буфер.
    pub fn block(&self) -> Block<Virt> {
        self.0.block()
    }

    /// Возвращает статистики пишущих транзакций.
    pub fn write_stats(&self) -> &RingBufferStats {
        &self.0.stats
    }
}

/// Заголовок одной записи [`RingBuffer`].
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
enum Header {
    /// Запись свободна, то есть в неё ещё ничего не записывалось.
    Clear = Self::CLEAR,

    /// Запись сообщает о закрытие буфера.
    Closed = Self::CLOSED,

    /// Запись захвачена писателем.
    /// То есть, он начал в неё писать данные, но пока ещё не закончил.
    Locked = Self::LOCKED,

    /// Запись зафиксирована писателем, но ещё не зафиксирована читателем.
    /// Так как она ещё нужна читателю, поверх неё пока нельзя писать новые данные,
    /// и она занимает место в буфере.
    Written {
        /// Размер полезных данных в записи.
        size: usize,
    } = Self::WRITTEN,

    /// Запись зафиксирована и писателем и читателем.
    /// Теперь поверх неё можно писать новые данные.
    Read {
        /// Размер полезных данных в записи.
        size: usize,
    } = Self::READ,
}

impl Header {
    /// Значение флага для свободной записи,
    /// см. [`Header::Clear`].
    const CLEAR: u8 = 0;

    /// Значение флага для записи, закрывающей [`RingBuffer`],
    /// см. [`Header::Closed`].
    const CLOSED: u8 = 1;

    /// Значение флага для захваченной писателем записи,
    /// см. [`Header::Locked`].
    const LOCKED: u8 = 2;

    /// Значение флага для зафиксированной писателем и не зафиксированной читателем записи,
    /// см. [`Header::Written`].
    const WRITTEN: u8 = 3;

    /// Значение флага для зафиксированной и писателем и читателем записи,
    /// см. [`Header::Written`].
    const READ: u8 = 4;
}

/// Читающая или пишущая транзакция.
pub struct RingBufferTx<'a, T: Tag> {
    /// Ссылка на исходный [`RingBuffer`].
    ring_buffer: &'a mut RingBuffer,

    /// Позиция заголовка записи, которую обрабатывает эта транзакция.
    header: usize,

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
    ///
    /// # Safety
    ///
    /// Так как возвращается срез, расположенный в разделяемой памяти,
    /// его содержимое может конкурентно меняться другой стороной.
    /// То есть, нельзя проверить его на какие-либо инварианты,
    /// и после этого использовать, опираясь на них.
    /// Так как в промежутке эти инварианты могут быть сломаны другой стороной.
    ///
    /// То есть, нужно:
    /// - Либо гарантировать, что другая сторона не запускается конкурентно.
    /// - Либо прежде всего скопировать срез, и дальше пользоваться только копией.
    pub unsafe fn read(&mut self) -> Option<&[u8]> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Фиксирует читающую транзакцию, записывая обновлённое значение [`RingBuffer::head`] и
    /// статистику [`RingBuffer::stats`] в поля [`RingBuffer`].
    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl RingBufferTx<'_, WriteTag> {
    /// Копирует в буфер байты среза `data`.
    /// Обновляет поля самой транзакции [`RingBufferTx`] и статистики [`RingBuffer::stats`],
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
    /// статистику [`RingBuffer::stats`] в полях [`RingBuffer`].
    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Ёмкость, оставшаяся в буфере транзакции на текущий момент.
    pub fn capacity(&self) -> usize {
        let skip_header = if self.header == self.tail {
            RingBuffer::HEAD_SIZE
        } else {
            0
        };
        if RingBuffer::REAL_SIZE - (self.tail - self.head) >= skip_header + RingBuffer::TAIL_SIZE {
            RingBuffer::REAL_SIZE - (self.tail + skip_header - self.head) - RingBuffer::TAIL_SIZE
        } else {
            0
        }
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
pub trait Tag {}

/// Читающая транзакция.
pub type RingBufferReadTx<'a> = RingBufferTx<'a, ReadTag>;

/// Пишущая транзакция.
pub type RingBufferWriteTx<'a> = RingBufferTx<'a, WriteTag>;

/// Тег читающих транзакций.
pub struct ReadTag;

impl Tag for ReadTag {
}

/// Тег пишущих транзакций.
pub struct WriteTag;

impl Tag for WriteTag {
}

/// Статистики чтения или записи в буфер.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RingBufferStats {
    /// Количество байт, прочитанных или записанных в зафиксированных транзакциях.
    committed: usize,

    /// Количество зафиксированных транзакций соответствующего типа.
    commits: usize,

    /// Количество байт, прочитанных или записанных в оборванных транзакциях.
    dropped: usize,

    /// Количество отменённых транзакций соответствующего типа.
    drops: usize,

    /// Количество ошибок в транзакциях.
    errors: usize,

    /// Количество транзакций чтения либо записи соответственно.
    txs: usize,
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
