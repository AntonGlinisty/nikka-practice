extern crate alloc;

use alloc::vec::Vec;
use core::slice;

use ku::error::Error::NoDisk;

use kernel::{
    Subsystems,
    fs::Kind,
    log::debug,
};

mod fs_helpers;
mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn allocation() {
    let (mut block_bitmap, _) = fs_helpers::single_inode_fs(Kind::File);

    let free_block_count = (0 .. fs_helpers::BLOCK_COUNT)
        .map(|i| {
            if block_bitmap.is_free(i) {
                1
            } else {
                0
            }
        })
        .sum();
    assert!(free_block_count >= fs_helpers::BLOCK_COUNT - 16);

    let mut allocated: Vec<_> =
        (0 .. free_block_count).map(|_| block_bitmap.allocate().unwrap()).collect();
    debug!(
        allocated_head = ?allocated[.. 10],
        allocated_tail = ?allocated[allocated.len() - 10 ..],
    );

    assert_eq!(block_bitmap.allocate(), Err(NoDisk));
    block_bitmap.set_free(33);
    block_bitmap.allocate().unwrap();
    for _ in (1 ..= fs_helpers::BLOCK_COUNT).rev() {
        block_bitmap.set_free(fs_helpers::BLOCK_COUNT - 33);
        block_bitmap.allocate().unwrap();
    }

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
