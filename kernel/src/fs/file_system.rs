use ku::{
    error::{
        Error::{
            FileNotFound,
            Medium,
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

    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// для отслеживания какие именно
    /// [inode](https://en.wikipedia.org/wiki/Inode)
    /// файловой системы заняты, а какие --- свободны.
    inode_bitmap: Bitmap,

    /// Массив
    /// [inode](https://en.wikipedia.org/wiki/Inode)
    /// файловой системы.
    inodes: &'static mut [Inode],

    /// Суперблок
    /// ([superblock](https://en.wikipedia.org/wiki/Unix_File_System#Design))
    /// файловой системы.
    superblock: &'static mut Superblock,
}

impl FileSystem {
    /// [Монтирует](https://en.wikipedia.org/wiki/Mount_(computing))
    /// файловую систему с диска номер `disk`.
    /// Параметр `block_cache_capacity` задаёт ограничение на размер кэша блоков.
    pub fn mount(
        disk: usize,
        block_cache_capacity: usize,
    ) -> Result<FileSystem> {
        let disk = Disk::new(disk)?;

        BlockCache::init(disk, Superblock::SUPERBLOCK_BLOCK + 1, 1)?;

        let superblock = Superblock::new()?;

        BlockCache::init(disk, superblock.blocks().end, block_cache_capacity)?;

        let inodes = unsafe {
            BlockCache::block(superblock.inode_table().start)?
                .start_address()
                .try_into_mut_slice::<Inode>(superblock.inodes().end)?
        };

        if inodes[superblock.root()].kind() == Kind::Directory {
            Ok(Self {
                block_bitmap: Bitmap::new(superblock.block_bitmap().start, superblock.blocks())?,
                inode_bitmap: Bitmap::new(superblock.inode_bitmap().start, superblock.inodes())?,
                inodes,
                superblock,
            })
        } else {
            Err(Medium)
        }
    }

    /// Форматирует файловую систему размером `size` байт на диске номер `disk`.
    ///
    /// Если количество
    /// [inode](https://en.wikipedia.org/wiki/Inode)
    /// не указано, то есть `inode_count.is_none()`,
    /// оно подбирается под размер `size` автоматически.
    pub fn format(
        disk: usize,
        size: usize,
        inode_count: Option<usize>,
    ) -> Result<()> {
        let disk = Disk::new(disk)?;

        let block_count = size / BLOCK_SIZE;
        let default_blocks_per_inode = 4;
        let block_cache_capacity = 1 << 10;
        let inode_count = inode_count.unwrap_or(block_count / default_blocks_per_inode);

        BlockCache::init(disk, block_count, block_cache_capacity)?;

        let superblock = Superblock::format(block_count, inode_count)?;
        Bitmap::format(superblock.block_bitmap().start, superblock.blocks())?;
        Bitmap::format(superblock.inode_bitmap().start, superblock.inodes())?;

        let inodes = unsafe {
            BlockCache::block(superblock.inode_table().start)?
                .start_address()
                .try_into_mut_slice::<Inode>(superblock.inodes().end)?
        };
        for inode in inodes.iter_mut() {
            *inode = Inode::default();
        }
        inodes[superblock.root()].init(Kind::Directory);

        let block_size = Size::bytes(BLOCK_SIZE);
        let blocks = superblock.blocks();
        let inodes = superblock.inodes();
        let inode_size = Size::of::<Inode>();
        let directory_entry_size = Size::of::<DirectoryEntry>();
        let free_space = Size::bytes(blocks.clone().count() * BLOCK_SIZE);
        let max_file_size = Size::bytes(Inode::max_size());
        info!(
            %free_space,
            %disk,
            %block_size,
            block_count,
            ?blocks,
            %inode_size,
            ?inodes,
            %directory_entry_size,
            %max_file_size,
            "formatted the file system",
        );

        BlockCache::flush(superblock.blocks().start)
    }

    /// Записывает на диск все обновления файловой системы.
    pub fn flush(&self) -> Result<()> {
        BlockCache::flush(self.superblock.blocks().end)
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
        self.block_bitmap.free_count() * BLOCK_SIZE
    }

    /// Возвращает размер занятого места файловой системы в байтах.
    pub fn used_space(&self) -> usize {
        self.superblock.blocks().count() * BLOCK_SIZE - self.free_space()
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            error!(?error, "error on the file system unmount");
        }
    }
}
