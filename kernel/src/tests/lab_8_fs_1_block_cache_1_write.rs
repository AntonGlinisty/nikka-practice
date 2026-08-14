use core::mem;

use ku::memory::{
    mmu::{
        KERNEL_RW,
        PageTableFlags,
    },
    size::MiB,
};

use crate::{
    fs::{
        BlockCache,
        test_scaffolding::{
            BLOCK_SIZE,
            SECTORS_PER_BLOCK,
            block_cache_init,
            cache,
            emplace_block,
            flush_block,
        },
    },
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        test_scaffolding::path,
    },
};

pub(super) fn write() {
    let block_count = FS_SIZE / BLOCK_SIZE;
    debug!(block_count);

    block_cache_init(FS_DISK, block_count, block_count).unwrap();

    let cache = cache().unwrap();

    let len = 16 << 10;
    let slice = unsafe { cache.try_into_mut_slice().unwrap() };

    let mut emplacement_count = 0;
    let mut read_count = 0;
    let mut write_count = 0;

    for block in 0 .. (len * mem::size_of_val(&slice[0])).div_ceil(BLOCK_SIZE) {
        if block % 3 == 0 {
            unsafe {
                emplace_block(block).unwrap();
            }
            emplacement_count += 1;
        } else {
            read_count += 1;
        }
    }

    for (i, element) in slice[.. len].iter_mut().enumerate() {
        *element = i;
    }

    for block in 0 .. (len * mem::size_of_val(&slice[0])).div_ceil(BLOCK_SIZE) {
        let virt = cache.address(block * BLOCK_SIZE).unwrap();
        let flags = path(&mut BASE_ADDRESS_SPACE.lock(), virt).flags().unwrap();
        assert_eq!(
            flags,
            KERNEL_RW | PageTableFlags::ACCESSED | PageTableFlags::DIRTY,
        );

        if block.is_multiple_of(2) {
            flush_block(block).unwrap();
            let flags = path(&mut BASE_ADDRESS_SPACE.lock(), virt).flags().unwrap();
            debug!(block, %flags, "flush");
            assert_eq!(flags, KERNEL_RW | PageTableFlags::ACCESSED);
            write_count += 1;
        }
    }

    let block_cache_stats = BlockCache::stats();
    let disk_stats = BlockCache::disk_stats();
    debug!(?block_cache_stats);
    debug!(?disk_stats);
    assert_eq!(*block_cache_stats.emplacements(), emplacement_count);
    assert_eq!(*block_cache_stats.discards(), 0);
    assert_eq!(*block_cache_stats.reads(), read_count);
    assert_eq!(*block_cache_stats.writes(), write_count);
    assert_eq!(*disk_stats.flushes(), write_count);
    assert_eq!(*disk_stats.reads(), read_count * SECTORS_PER_BLOCK);
    assert_eq!(*disk_stats.writes(), write_count * SECTORS_PER_BLOCK);
}

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
