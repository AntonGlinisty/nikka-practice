use ku::memory::size::MiB;

use kernel::{
    fs::{
        Kind,
        test_scaffolding::{
            BLOCK_SIZE,
            Bitmap,
            Inode,
            block_cache_init,
        },
    },
    log::debug,
};

pub(super) fn single_inode_fs(kind: Kind) -> (Bitmap, Inode) {
    debug!(block_count = BLOCK_COUNT);

    block_cache_init(FS_DISK, BLOCK_COUNT).unwrap();

    Bitmap::format(BLOCK_COUNT).unwrap();

    let block_bitmap = Bitmap::new(BLOCK_COUNT).unwrap();
    let inode = Inode::new(kind);

    (block_bitmap, inode)
}

pub(super) const BLOCK_COUNT: usize = FS_SIZE / BLOCK_SIZE;

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
