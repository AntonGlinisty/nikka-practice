use core::mem;

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    fs::test_scaffolding::{
        BLOCK_SIZE,
        block_cache_init,
        cache,
        flush_block,
    },
    log::debug,
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn write() {
    let block_count = FS_SIZE / BLOCK_SIZE;
    debug!(block_count);

    block_cache_init(FS_DISK, block_count).unwrap();

    let cache = cache().unwrap();

    let len = 16 << 10;
    let slice = unsafe { cache.try_into_mut_slice().unwrap() };

    for (i, element) in slice[.. len].iter_mut().enumerate() {
        *element = i;
    }

    for block in 0 .. (len * mem::size_of_val(&slice[0])).div_ceil(BLOCK_SIZE) {
        if block % 2 == 0 {
            debug!(block, "flush");
            flush_block(block).unwrap();
        }
    }
}

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
