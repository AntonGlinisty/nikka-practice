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
    SUPERBLOCK_BLOCK,
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

#[derive(Debug)]
pub struct FileSystem {
    block_bitmap: Bitmap,
    superblock: &'static mut Superblock,
}

impl FileSystem {
    pub fn mount(disk: usize) -> Result<FileSystem> {
        let disk = Disk::list()[disk];

        BlockCache::init(disk, SUPERBLOCK_BLOCK + 1)?;

        let superblock = Superblock::new()?;

        BlockCache::init(disk, superblock.block_count())?;

        let block_bitmap = Bitmap::new(superblock.block_count())?;

        Ok(Self {
            block_bitmap,
            superblock,
        })
    }

    pub fn format(
        disk: usize,
        size: usize,
    ) -> Result<()> {
        let disk = Disk::list()[disk];
        let block_count = size / BLOCK_SIZE;

        BlockCache::init(disk, block_count)?;

        Superblock::format(block_count)?;
        Bitmap::format(block_count)?;

        let reserved_block_count = Bitmap::new(block_count)?.reserved_block_count();
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

    pub fn flush(&self) -> Result<()> {
        BlockCache::flush(self.superblock.block_count())
    }

    pub fn open(
        &mut self,
        path: &str,
    ) -> Result<File> {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn free_space(&self) -> usize {
        self.block_bitmap.free_block_count() * BLOCK_SIZE
    }

    pub fn reserved_space(&self) -> usize {
        self.block_bitmap.reserved_block_count() * BLOCK_SIZE
    }

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
