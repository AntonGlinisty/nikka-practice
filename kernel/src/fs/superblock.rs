use core::{
    mem,
    ops::Range,
};

use static_assertions::const_assert;

use ku::error::{
    Error::Medium,
    Result,
};

use super::{
    BLOCK_SIZE,
    bitmap::Bitmap,
    block_cache::BlockCache,
    inode::{
        Inode,
        Kind,
    },
};

// Used in docs.
#[allow(unused)]
use ku::error::Error;

/// Суперблок
/// ([superblock](https://en.wikipedia.org/wiki/Unix_File_System#Design))
/// [файловой системы](https://en.wikipedia.org/wiki/File_system).
///
/// Содержит метаинформацию, которая нужна для работы с файловой системой в целом.
#[derive(Debug)]
#[repr(C, align(4096))]
pub struct Superblock {
    /// [Сигнатура](https://en.wikipedia.org/wiki/Magic_number_(programming)#Format_indicators)
    /// файловой системы.
    /// Позволяет убедиться, что на диске действительно хранится
    /// инициализированная файловая система нужного формата.
    magic: [u8; MAGIC.len()],

    /// Индикатор [направления байт](https://en.wikipedia.org/wiki/Endianness)
    /// компьютера, создавшего файловую систему.
    /// Проверяется как и [`Superblock::magic`] чтобы не интерпретировать
    /// данные на диске заведомо неправильным образом.
    endian: u64,

    /// Количество блоков в файловой системе.
    block_count: usize,

    /// [Корневая директория](https://en.wikipedia.org/wiki/Root_directory)
    /// файловой системы.
    root: Inode,
}

const_assert!(mem::align_of::<Superblock>() == BLOCK_SIZE);
const_assert!(mem::size_of::<Superblock>() == BLOCK_SIZE);

impl Superblock {
    /// Возвращает суперблок файловой системы или ошибку [`Error::Medium`],
    /// если отведённое под суперблок место диска содержит данные,
    /// которые не похожи на корректный суперблок.
    pub(super) fn new() -> Result<&'static mut Self> {
        Self::new_unchecked().validate()
    }

    /// Форматирует часть диска,
    /// отведённую под суперблок для файловой системы размером `block_count` блоков,
    /// его начальным состоянием.
    pub(super) fn format(block_count: usize) -> Result<&'static Self> {
        unsafe {
            BlockCache::block(Self::SUPERBLOCK_BLOCK)?
                .try_into_mut_slice::<usize>()
                .unwrap()
                .fill(0);
        }

        let superblock = Self::new_unchecked();

        superblock.magic.copy_from_slice(MAGIC.as_bytes());
        superblock.endian = ENDIAN;
        superblock.block_count = block_count;
        superblock.root.set_kind(Kind::Directory);

        superblock.validate()?;

        Ok(superblock)
    }

    /// Возвращает диапазон блоков для пользовательских данных и директорий.
    ///
    /// Полное количество блоков в [файловой системе](https://en.wikipedia.org/wiki/File_system)
    /// равно `Superblock::blocks().end`.
    /// Блоки с нулевого до начала возвращаемого диапазона (не включительно),
    /// то есть диапазон `0..Superblock::blocks().start`,
    /// зарезервированы под метаданные самой файловой системы.
    pub(super) fn blocks(&self) -> Range<usize> {
        Self::FIRST_BITMAP_BLOCK + Bitmap::size_in_blocks(self.block_count) .. self.block_count
    }

    /// [Корневая директория](https://en.wikipedia.org/wiki/Root_directory)
    /// файловой системы.
    pub(super) fn root(&mut self) -> &mut Inode {
        &mut self.root
    }

    /// Создаёт [`Superblock`], не проверяя корректность его данных на диске.
    fn new_unchecked() -> &'static mut Self {
        unsafe {
            BlockCache::block(Self::SUPERBLOCK_BLOCK)
                .unwrap()
                .try_into_mut::<Self>()
                .unwrap()
        }
    }

    /// Проверяет корректность данных [`Superblock`] на диске.
    /// Возвращает ошибку [`Error::Medium`],
    /// если данные на диске заведомо не содержат корректный [`Superblock`],
    /// или сам [`Superblock`] иначе.
    fn validate(&mut self) -> Result<&mut Self> {
        let is_valid = self.magic == MAGIC.as_bytes() &&
            self.endian == ENDIAN &&
            self.block_count > 0 &&
            self.root.kind() == Kind::Directory;

        if is_valid {
            Ok(self)
        } else {
            Err(Medium)
        }
    }

    /// Номер первого блока, хранящего
    /// [битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// занятых блоков --- [`Bitmap`].
    pub(super) const FIRST_BITMAP_BLOCK: usize = Self::SUPERBLOCK_BLOCK + 1;

    /// Номер блока, в котором хранится суперблок
    /// ([superblock](https://en.wikipedia.org/wiki/Unix_File_System#Design))
    /// файловой системы --- [`Superblock`].
    pub(super) const SUPERBLOCK_BLOCK: usize = 1;
}

/// Значение индикатора [направления байт](https://en.wikipedia.org/wiki/Endianness).
const ENDIAN: u64 = 0x0102_0304_0506_0708;

/// [Сигнатура](https://en.wikipedia.org/wiki/Magic_number_(programming)#Format_indicators)
/// файловой системы.
const MAGIC: &str = "Nikka's simple file system";

#[doc(hidden)]
pub mod test_scaffolding {
    use core::ops::Range;

    use ku::error::Result;

    pub struct Superblock(&'static super::Superblock);

    impl Superblock {
        pub fn format(block_count: usize) -> Result<Self> {
            Ok(Superblock(super::Superblock::format(block_count)?))
        }

        pub fn blocks(&self) -> Range<usize> {
            self.0.blocks()
        }
    }
}
