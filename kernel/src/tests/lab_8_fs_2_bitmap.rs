extern crate alloc;

use alloc::vec::Vec;
use core::slice;

use ku::{
    error::Error::NoDisk,
    memory::size::MiB,
};

use kernel::{
    Subsystems,
    fs::test_scaffolding::{
        BLOCK_SIZE,
        Bitmap,
        block_cache_init,
    },
    log::debug,
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn allocation() {
    let block_count = FS_SIZE / BLOCK_SIZE;
    debug!(block_count);

    block_cache_init(FS_DISK, block_count).unwrap();

    Bitmap::format(block_count).unwrap();
    let mut block_bitmap = Bitmap::new(block_count).unwrap();

    let free_block_count = (0 .. block_count)
        .map(|i| {
            if block_bitmap.is_free(i) {
                1
            } else {
                0
            }
        })
        .sum();
    assert!(free_block_count >= block_count - 16);

    let mut allocated: Vec<_> =
        (0 .. free_block_count).map(|_| block_bitmap.allocate().unwrap()).collect();
    debug!(
        allocated_head = ?allocated[.. 10],
        allocated_tail = ?allocated[allocated.len() - 10 ..],
    );

    assert_eq!(block_bitmap.allocate(), Err(NoDisk));

    slice::heapsort(&mut allocated, |a, b| a < b);
    for (i, j) in allocated.iter().zip(allocated.iter().skip(1)) {
        assert_ne!(*i, *j, "allocated the same block twice");
    }

    let mut free_block_count = 0;
    for i in allocated.iter().step_by(2) {
        if i % 10_000 == 0 {
            debug!(block = i, free_block_count);
        }
        assert!(!block_bitmap.is_free(*i));
        block_bitmap.set_free(*i);
        free_block_count += 1;
    }

    let mut reallocated: Vec<_> =
        (0 .. free_block_count).map(|_| block_bitmap.allocate().unwrap()).collect();
    debug!(
        reallocated_head = ?reallocated[.. 10],
        reallocated_tail = ?reallocated[reallocated.len() - 10 ..],
    );
    assert_eq!(block_bitmap.allocate(), Err(NoDisk));

    slice::heapsort(&mut reallocated, |a, b| a < b);
    for (i, j) in allocated.iter().step_by(2).zip(reallocated) {
        assert_eq!(*i, j);
    }
}

const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
