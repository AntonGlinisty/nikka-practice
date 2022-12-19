use core::mem;

use static_assertions::const_assert;

use ku::error::{
    Error::Medium,
    Result,
};

use super::{
    BLOCK_SIZE,
    SUPERBLOCK_BLOCK,
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
    pub(super) fn format(block_count: usize) -> Result<()> {
        unsafe {
            BlockCache::block(SUPERBLOCK_BLOCK)?
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

        Ok(())
    }

    /// [Корневая директория](https://en.wikipedia.org/wiki/Root_directory)
    /// файловой системы.
    pub(super) fn root(&mut self) -> &mut Inode {
        &mut self.root
    }

    /// Количество блоков в файловой системе.
    pub(super) fn block_count(&self) -> usize {
        self.block_count
    }

    /// Создаёт [`Superblock`], не проверяя корректность его данных на диске.
    fn new_unchecked() -> &'static mut Self {
        unsafe { BlockCache::block(SUPERBLOCK_BLOCK).unwrap().try_into_mut::<Self>().unwrap() }
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
}

/// Значение индикатора [направления байт](https://en.wikipedia.org/wiki/Endianness).
const ENDIAN: u64 = 0x0102_0304_0506_0708;

/// [Сигнатура](https://en.wikipedia.org/wiki/Magic_number_(programming)#Format_indicators)
/// файловой системы.
const MAGIC: &str = "Nikka's simple file system";
