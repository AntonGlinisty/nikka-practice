extern crate alloc;
use alloc::{
    boxed::Box,
    vec::Vec,
};

use core::mem;

use kernel::{
    allocator,
    memory::Page,
};

mod init;

init!();

#[test_case]
fn basic() {
    let start_info = allocator::info();

    {
        let mut a = Box::new(1_i64);
        let a_ptr = &mut *a as *mut i64;
        unsafe {
            a_ptr.write_volatile(a_ptr.read_volatile() + 1);
        }
        assert_eq!(*a, 2);

        let info = allocator::info();
        let requested = mem::size_of_val(&*a);
        assert_eq!(info.allocations() - start_info.allocations(), 1);
        assert_eq!(info.deallocations() - start_info.deallocations(), 0);
        assert_eq!(info.requested() - start_info.requested(), requested);
        assert!(info.allocated() - start_info.allocated() >= requested);
        assert_eq!(info.allocated_pages() - start_info.allocated_pages(), 1);
        assert_eq!(info.deallocated_pages() - start_info.deallocated_pages(), 0);
    }

    let info = allocator::info();
    assert_eq!(info.allocations() - start_info.allocations(), 1);
    assert_eq!(info.deallocations() - start_info.deallocations(), 1);
    assert_eq!(info.requested() - start_info.requested(), 0);
    assert_eq!(info.allocated() - start_info.allocated(), 0);
    assert_eq!(info.allocated_pages() - start_info.allocated_pages(), 1);
    assert_eq!(info.deallocated_pages() - start_info.deallocated_pages(), 1);
    assert_eq!(
        info.fragmentation_loss() - start_info.fragmentation_loss(),
        0,
    );
    assert_eq!(
        info.fragmentation_loss_percentage() - start_info.fragmentation_loss_percentage(),
        0.0,
    );
}

#[test_case]
fn realloc() {
    let start_info = allocator::info();

    let mut vec = Vec::new();
    let mut sum = 0;

    for a in 1 .. 3 * Page::SIZE {
        vec.push(a);
        assert_eq!(vec.len(), a);
        sum += a;
    }

    assert!(vec.iter().sum::<usize>() == sum);

    let info = allocator::info();
    let requested = vec.capacity() * mem::size_of_val(&sum);
    assert_eq!(info.requested() - start_info.requested(), requested);
    assert_eq!(info.allocated() - start_info.allocated(), requested);
    assert_eq!(
        info.fragmentation_loss() - start_info.fragmentation_loss(),
        0,
    );
    assert!(
        info.fragmentation_loss_percentage() - start_info.fragmentation_loss_percentage() <= 0.0,
    );
}
