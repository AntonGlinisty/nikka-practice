use ku::{
    error::{
        Error::{
            FileNotFound,
            NotDirectory,
        },
        Result,
    },
    log::{
        error,
        info,
    },
    memory::size::Size,
};

use super::{
    BLOCK_SIZE,
    bitmap::Bitmap,
    block_cache::BlockCache,
    directory_entry::DirectoryEntry,
    disk::Disk,
    file::File,
    inode::{
        Inode,
        Kind,
    },
    superblock::Superblock,
};

/// Интерфейс к файловой системе.
#[derive(Debug)]
pub struct FileSystem {
    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// для отслеживания какие именно блоки файловой системы заняты, а какие --- свободны.
    block_bitmap: Bitmap,

    /// Суперблок
    /// ([superblock](https://en.wikipedia.org/wiki/Unix_File_System#Design))
    /// файловой системы.
    superblock: &'static mut Superblock,
}

impl FileSystem {
    /// [Монтирует](https://en.wikipedia.org/wiki/Mount_(computing))
    /// файловую систему с диска номер `disk`.
    pub fn mount(disk: usize) -> Result<FileSystem> {
        let disk = Disk::new(disk)?;

        BlockCache::init(disk, Superblock::SUPERBLOCK_BLOCK + 1)?;

        let superblock = Superblock::new()?;

        BlockCache::init(disk, superblock.block_count())?;

        let block_bitmap =
            Bitmap::new(superblock.block_count(), superblock.reserved_block_count())?;

        Ok(Self {
            block_bitmap,
            superblock,
        })
    }

    /// Форматирует файловую систему размером `size` байт на диске номер `disk`.
    pub fn format(
        disk: usize,
        size: usize,
    ) -> Result<()> {
        let disk = Disk::new(disk)?;

        let block_count = size / BLOCK_SIZE;

        BlockCache::init(disk, block_count)?;

        let superblock = Superblock::format(block_count)?;
        Bitmap::format(block_count, superblock.reserved_block_count())?;

        let reserved_block_count = superblock.reserved_block_count();
        let inode_size = Size::of::<Inode>();
        let directory_entry_size = Size::of::<DirectoryEntry>();
        let free_space = Size::bytes((block_count - reserved_block_count) * BLOCK_SIZE);
        let max_file_size = Size::bytes(Inode::max_size());
        info!(
            %free_space,
            %disk,
            block_count,
            reserved_block_count,
            %inode_size,
            %directory_entry_size,
            %max_file_size,
            "formatted the file system",
        );

        BlockCache::flush(block_count)
    }

    /// Записывает на диск все обновления файловой системы.
    pub fn flush(&self) -> Result<()> {
        BlockCache::flush(self.superblock.block_count())
    }

    /// Проходит от корня файловой системы по заданному полному пути `path`.
    /// Возвращает [`File`], соответствующий этому `path`.
    pub fn open(
        &mut self,
        path: &str,
    ) -> Result<File> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает размер свободного места файловой системы в байтах.
    pub fn free_space(&self) -> usize {
        self.block_bitmap.free_block_count() * BLOCK_SIZE
    }

    /// Возвращает размер зарезервированного места файловой системы в байтах.
    pub fn reserved_space(&self) -> usize {
        self.superblock.reserved_block_count() * BLOCK_SIZE
    }

    /// Возвращает размер занятого места файловой системы в байтах.
    pub fn used_space(&self) -> usize {
        self.superblock.block_count() * BLOCK_SIZE - self.free_space() - self.reserved_space()
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            error!(?error, "error on the file system unmount");
        }
    }
}
