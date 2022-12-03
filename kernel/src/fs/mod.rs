mod bitmap;
mod block_cache;
mod directory_entry;
mod disk;
mod file;
mod file_system;
mod inode;
mod superblock;

use ku::memory::Page;

pub(crate) use block_cache::BlockCache;

pub use directory_entry::MAX_NAME_LEN;
pub use file::File;
pub use file_system::FileSystem;
pub use inode::Kind;

const BLOCK_SIZE: usize = Page::SIZE;
const SUPERBLOCK_BLOCK: usize = 1;
const FIRST_BITMAP_BLOCK: usize = SUPERBLOCK_BLOCK + 1;
