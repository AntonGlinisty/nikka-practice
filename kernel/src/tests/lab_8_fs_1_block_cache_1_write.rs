use core::mem;

use ku::memory::{
    mmu::{
        KERNEL_RW,
        PageTableFlags,
    },
    size::MiB,
};

use kernel::{
    Subsystems,
    fs::{
        BlockCache,
        test_scaffolding::{
            BLOCK_SIZE,
            block_cache_init,
            cache,
            flush_block,
        },
    },
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        test_scaffolding::{
            get_pte,
            path,
        },
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn write() {
    let block_count = FS_SIZE / BLOCK_SIZE;
    debug!(block_count);

    block_cache_init(FS_DISK, block_count, block_count).unwrap();

    let cache = cache().unwrap();

    let len = 16 << 10;
    let slice = unsafe { cache.try_into_mut_slice().unwrap() };

    for (i, element) in slice[.. len].iter_mut().enumerate() {
        *element = i;
    }

    for block in 0 .. (len * mem::size_of_val(&slice[0])).div_ceil(BLOCK_SIZE) {
        let virt = cache.address(block * BLOCK_SIZE).unwrap();
        let flags = get_pte(&path(&mut BASE_ADDRESS_SPACE.lock(), virt)).unwrap().flags();
        assert_eq!(
            flags,
            KERNEL_RW | PageTableFlags::ACCESSED | PageTableFlags::DIRTY,
        );

        if block.is_multiple_of(2) {
            flush_block(block).unwrap();
            let flags = get_pte(&path(&mut BASE_ADDRESS_SPACE.lock(), virt)).unwrap().flags();
            debug!(block, %flags, "flush");
            assert_eq!(flags, KERNEL_RW | PageTableFlags::ACCESSED);
        }
    }

    debug!(block_cache_stats = ?BlockCache::stats());
}

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
