use core::mem;

use ku::memory::{
    Virt,
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
            SECTORS_PER_BLOCK,
            block_cache_init,
            cache,
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
fn read_what_was_written() {
    let block_count = FS_SIZE / BLOCK_SIZE;
    debug!(block_count);

    block_cache_init(FS_DISK, block_count, block_count).unwrap();

    let cache = cache().unwrap();

    let len = 16 << 10;
    let slice = unsafe { cache.try_into_mut_slice::<usize>().unwrap() };
    let read_count = (len * mem::size_of_val(&slice[0])).div_ceil(BLOCK_SIZE);

    for (i, actual) in slice[.. len].iter_mut().enumerate() {
        let block = i * mem::size_of_val(actual) / BLOCK_SIZE;
        let expected = if block.is_multiple_of(2) {
            i
        } else {
            INVALID_DISK_PATTERN
        };
        let virt = Virt::from_ref(actual);
        let actual = unsafe { virt.try_into_ptr::<usize>().unwrap().read_volatile() };
        let flags = get_pte(&path(&mut BASE_ADDRESS_SPACE.lock(), virt)).unwrap().flags();
        if actual != expected || i.is_multiple_of(777) {
            debug!(i, block, %flags, actual, expected);
        }
        assert_eq!(actual, expected);
        assert_eq!(flags, KERNEL_RW | PageTableFlags::ACCESSED);
    }

    let block_cache_stats = BlockCache::stats();
    let disk_stats = BlockCache::disk_stats();
    debug!(?block_cache_stats);
    debug!(?disk_stats);
    assert_eq!(*block_cache_stats.emplacements(), 0);
    assert_eq!(*block_cache_stats.discards(), 0);
    assert_eq!(*block_cache_stats.reads(), read_count);
    assert_eq!(*block_cache_stats.writes(), 0);
    assert_eq!(*disk_stats.flushes(), 0);
    assert_eq!(*disk_stats.reads(), read_count * SECTORS_PER_BLOCK);
    assert_eq!(*disk_stats.writes(), 0);
}

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
const INVALID_DISK_PATTERN: usize = 0xAAAA_AAAA_AAAA_AAAA;
