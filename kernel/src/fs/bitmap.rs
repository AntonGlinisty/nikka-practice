use core::{
    mem,
    ops::Add,
};

use ku::error::{
    Error::{
        Medium,
        NoDisk,
    },
    Result,
};

use super::{
    BLOCK_SIZE,
    FIRST_BITMAP_BLOCK,
    block_cache::BlockCache,
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

    /// Количество блоков в [файловой системе](https://en.wikipedia.org/wiki/File_system),
    /// то есть количество используемых бит в битовой карте.
    block_count: usize,

    /// Место последней аллокации из битовой карты.
    /// Служит для ускорения аллокаций и равномерного распределения занятых блоков по диску.
    cursor: usize,
}

impl Bitmap {
    /// Возвращает битовую карту файловой системы или ошибку [`Error::Medium`],
    /// если отведённое под битовую карту место диска содержит данные,
    /// которые не похожи на корректную битовую карту.
    pub(super) fn new(block_count: usize) -> Result<Self> {
        let bitmap = Self::new_unchecked(block_count);

        bitmap.validate()
    }

    /// Форматирует часть диска,
    /// отведённую под битовую карту для `block_count` блоков,
    /// его начальным состоянием.
    pub(super) fn format(block_count: usize) -> Result<()> {
        let mut bitmap = Self::new_unchecked(block_count);

        bitmap.bitmap.fill(0);

        for reserved_element in 0 .. bitmap.reserved_block_count() {
            assert_eq!(
                bitmap.allocate().expect("the bitmap has been cleared just yet"),
                reserved_element,
            );
        }

        if block_count < bitmap.bitmap.len() * Self::BITS_PER_ENTRY {
            bitmap.bitmap[block_count / Self::BITS_PER_ENTRY] =
                !0 << (block_count % Self::BITS_PER_ENTRY);
        }

        bitmap.validate()?;

        Ok(())
    }

    /// Возвращает `true`, если блок `number` свободен.
    ///
    /// # Panics
    ///
    /// Паникует, если `number` выходит за пределы диска --- [`Bitmap::block_count`].
    pub(super) fn is_free(
        &self,
        number: usize,
    ) -> bool {
        assert!(number < self.block_count);
        self.bitmap[number / Self::BITS_PER_ENTRY] & (1 << (number % Self::BITS_PER_ENTRY)) == 0
    }

    /// Помечает блок `number` как свободный.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - Значение `number` выходит за пределы файловой системы --- [`Bitmap::block_count`].
    /// - Блок `number` зарезервирован.
    /// - Блок `number` уже помечен как свободный.
    pub(super) fn set_free(
        &mut self,
        number: usize,
    ) {
        assert!(self.reserved_block_count() <= number && number < self.block_count);
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

    /// Возвращает количество зарезервированных блоков в файловой системе.
    pub(super) fn reserved_block_count(&self) -> usize {
        FIRST_BITMAP_BLOCK + self.size_in_blocks()
    }

    /// Создаёт [`Bitmap`], не проверяя корректность его данных на диске.
    fn new_unchecked(block_count: usize) -> Self {
        Self {
            bitmap: Self::bitmap(block_count),
            block_count,
            cursor: 0,
        }
    }

    /// Проверяет корректность данных [`Bitmap`] на диске.
    /// Возвращает ошибку [`Error::Medium`],
    /// если данные на диске заведомо не содержат корректный [`Bitmap`],
    /// или сам [`Bitmap`] иначе.
    fn validate(self) -> Result<Self> {
        let bit_count = self.bitmap.len() * Self::BITS_PER_ENTRY;
        if bit_count < self.block_count || self.block_count < self.reserved_block_count() {
            return Err(Medium);
        }

        let unused_elements = self.block_count != self.bitmap.len() * Self::BITS_PER_ENTRY;
        if unused_elements {
            let unused_elements_expected_bitmap = !0 << (self.block_count % Self::BITS_PER_ENTRY);
            let unused_elements_actual_bitmap =
                self.bitmap[self.bitmap.len() - 1] & unused_elements_expected_bitmap;
            if unused_elements_actual_bitmap != unused_elements_expected_bitmap {
                return Err(Medium);
            }
        }

        if (0 .. self.reserved_block_count())
            .all(|reserved_element| !self.is_free(reserved_element))
        {
            Ok(self)
        } else {
            Err(Medium)
        }
    }

    /// Возвращает количество блоков, которые занимает [`Bitmap`] на диске.
    fn size_in_blocks(&self) -> usize {
        mem::size_of_val(self.bitmap).div_ceil(BLOCK_SIZE)
    }

    /// Возвращает срез элементов битовой карты в памяти блочного кэша [`Bitmap`].
    fn bitmap(block_count: usize) -> &'static mut [u64] {
        unsafe {
            BlockCache::block(FIRST_BITMAP_BLOCK)
                .unwrap()
                .start_address()
                .try_into_mut_slice::<u64>(block_count.div_ceil(Self::BITS_PER_ENTRY))
                .unwrap()
        }
    }

    /// Количество блоков, за которые отвечает один элемент среза [`Bitmap::bitmap`].
    const BITS_PER_ENTRY: usize = u64::BITS as usize;
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        Self {
            bitmap: Self::bitmap(self.block_count),
            block_count: self.block_count,
            cursor: self.cursor,
        }
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use ku::error::Result;

    pub struct Bitmap(pub(in super::super) super::Bitmap);

    impl Bitmap {
        pub fn new(block_count: usize) -> Result<Self> {
            Ok(Self(super::Bitmap::new(block_count)?))
        }

        pub fn format(block_count: usize) -> Result<()> {
            super::Bitmap::format(block_count)
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
