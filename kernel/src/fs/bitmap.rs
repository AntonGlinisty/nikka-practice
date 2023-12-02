use core::{
    mem,
    ops::{
        Add,
        Range,
    },
};

use ku::{
    error::{
        Error::{
            Medium,
            NoDisk,
        },
        Result,
    },
    memory::size,
};

use super::{
    BLOCK_SIZE,
    block_cache::BlockCache,
    superblock::Superblock,
};

// Used in docs.
#[allow(unused)]
use ku::error::Error;

/// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
/// для отслеживания какие именно блоки
/// [файловой системы](https://en.wikipedia.org/wiki/File_system) заняты, а какие --- свободны.
#[derive(Debug)]
pub(super) struct Bitmap {
    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap),
    /// каждый элемент этого среза отвечает за [`Self::BITS_PER_ENTRY`] блоков.
    /// Сам срез хранится в памяти блочного кэша [`BlockCache`].
    bitmap: &'static mut [u64],

    /// Диапазон блоков для пользовательских данных файловой системы.
    /// Полное количество блоков в [файловой системе](https://en.wikipedia.org/wiki/File_system),
    /// то есть количество используемых бит в битовой карте, равно `Bitmap::blocks.end`.
    /// Блоки с нулевого до начала возвращаемого диапазона (не включительно),
    /// то есть диапазон `0..Bitmap::blocks.start`,
    /// зарезервированы под метаданные самой файловой системы.
    blocks: Range<usize>,

    /// Место последней аллокации из битовой карты.
    /// Служит для ускорения аллокаций и равномерного распределения занятых блоков по диску.
    cursor: usize,
}

impl Bitmap {
    /// Возвращает битовую карту файловой системы или ошибку [`Error::Medium`],
    /// если отведённое под битовую карту место диска содержит данные,
    /// которые не похожи на корректную битовую карту.
    pub(super) fn new(blocks: Range<usize>) -> Result<Self> {
        let bitmap = Self::new_unchecked(blocks);

        bitmap.validate()
    }

    /// Форматирует часть диска,
    /// отведённую под битовую карту для `blocks.end` блоков,
    /// его начальным состоянием.
    pub(super) fn format(blocks: Range<usize>) -> Result<()> {
        let mut bitmap = Self::new_unchecked(blocks.clone());

        bitmap.bitmap.fill(0);

        for reserved_element in 0 .. bitmap.blocks.start {
            assert_eq!(
                bitmap.allocate().expect("the bitmap has been cleared just yet"),
                reserved_element,
            );
        }

        if blocks.end < bitmap.bitmap.len() * Self::BITS_PER_ENTRY {
            bitmap.bitmap[blocks.end / Self::BITS_PER_ENTRY] =
                !0 << (blocks.end % Self::BITS_PER_ENTRY);
        }

        bitmap.validate()?;

        Ok(())
    }

    /// Возвращает `true`, если блок `number` свободен.
    ///
    /// # Panics
    ///
    /// Паникует, если `number` выходит за пределы диска --- `Bitmap::blocks.end`.
    pub(super) fn is_free(
        &self,
        number: usize,
    ) -> bool {
        assert!(number < self.blocks.end);
        self.bitmap[number / Self::BITS_PER_ENTRY] & (1 << (number % Self::BITS_PER_ENTRY)) == 0
    }

    /// Помечает блок `number` как свободный.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Значение `number` выходит за пределы файловой системы --- `Bitmap::block_count.end`.
    ///   - Блок `number` зарезервирован.
    ///   - Блок `number` уже помечен как свободный.
    pub(super) fn set_free(
        &mut self,
        number: usize,
    ) {
        assert!(self.blocks.contains(&number));
        assert!(!self.is_free(number));
        self.bitmap[number / Self::BITS_PER_ENTRY] &= !(1 << (number % Self::BITS_PER_ENTRY));
    }

    /// Находит по [`Bitmap::bitmap`] свободный блок и аллоцирует его.
    /// Возвращает номер выделенного блока или
    /// ошибку [`Error::NoDisk`], если свободных блоков не осталось.
    ///
    /// При поиске свободного блока стартует с номера [`Bitmap::cursor`] в
    /// срезе [`Bitmap::bitmap`] и запоминает в неё номер элемента,
    /// из которого выделен свободный блок.
    pub(super) fn allocate(&mut self) -> Result<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает количество свободных блоков в файловой системе.
    pub(super) fn free_block_count(&self) -> usize {
        self.bitmap
            .iter()
            .map(|bits| bits.count_zeros().try_into().unwrap())
            .reduce(Add::add)
            .unwrap_or(0)
    }

    /// Возвращает количество блоков, которые занимает [`Bitmap`] на диске.
    pub(super) fn size_in_blocks(block_count: usize) -> usize {
        block_count.div_ceil(size::from(u8::BITS) * BLOCK_SIZE)
    }

    /// Создаёт [`Bitmap`], не проверяя корректность его данных на диске.
    fn new_unchecked(blocks: Range<usize>) -> Self {
        Self {
            bitmap: Self::bitmap(blocks.clone()),
            blocks,
            cursor: 0,
        }
    }

    /// Проверяет корректность данных [`Bitmap`] на диске.
    /// Возвращает ошибку [`Error::Medium`],
    /// если данные на диске заведомо не содержат корректный [`Bitmap`],
    /// или сам [`Bitmap`] иначе.
    fn validate(self) -> Result<Self> {
        let bit_count = self.bitmap.len() * Self::BITS_PER_ENTRY;
        if self.blocks.start >= self.blocks.end || bit_count < self.blocks.end {
            return Err(Medium);
        }

        let unused_elements = self.blocks.end != self.bitmap.len() * Self::BITS_PER_ENTRY;
        if unused_elements {
            let unused_elements_expected_bitmap = !0 << (self.blocks.end % Self::BITS_PER_ENTRY);
            let unused_elements_actual_bitmap =
                self.bitmap[self.bitmap.len() - 1] & unused_elements_expected_bitmap;
            if unused_elements_actual_bitmap != unused_elements_expected_bitmap {
                return Err(Medium);
            }
        }

        if (0 .. self.blocks.start).all(|reserved_element| !self.is_free(reserved_element)) {
            Ok(self)
        } else {
            Err(Medium)
        }
    }

    /// Возвращает срез элементов битовой карты в памяти блочного кэша [`Bitmap`].
    fn bitmap(blocks: Range<usize>) -> &'static mut [u64] {
        unsafe {
            BlockCache::block(Superblock::FIRST_BITMAP_BLOCK)
                .unwrap()
                .start_address()
                .try_into_mut_slice::<u64>(blocks.end.div_ceil(Self::BITS_PER_ENTRY))
                .unwrap()
        }
    }

    /// Количество блоков, за которые отвечает один элемент среза [`Bitmap::bitmap`].
    const BITS_PER_ENTRY: usize = u64::BITS as usize;
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        Self {
            bitmap: Self::bitmap(self.blocks.clone()),
            blocks: self.blocks.clone(),
            cursor: self.cursor,
        }
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use core::ops::Range;

    use ku::error::Result;

    pub struct Bitmap(pub(in super::super) super::Bitmap);

    impl Bitmap {
        pub fn new(blocks: Range<usize>) -> Result<Self> {
            Ok(Self(super::Bitmap::new(blocks)?))
        }

        pub fn format(blocks: Range<usize>) -> Result<()> {
            super::Bitmap::format(blocks)
        }

        pub fn is_free(
            &self,
            number: usize,
        ) -> bool {
            self.0.is_free(number)
        }

        pub fn set_free(
            &mut self,
            number: usize,
        ) {
            self.0.set_free(number)
        }

        pub fn allocate(&mut self) -> Result<usize> {
            self.0.allocate()
        }

        pub fn free_block_count(&self) -> usize {
            self.0.free_block_count()
        }
    }
}
