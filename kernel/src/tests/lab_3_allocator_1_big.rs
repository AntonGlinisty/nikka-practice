extern crate alloc;

use alloc::{
    boxed::Box,
    vec::Vec,
};
use core::mem;

use kernel::{
    Subsystems,
    allocator::test_scaffolding,
    log::debug,
    memory::{
        Page,
        test_scaffolding::forbid_frame_leaks,
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn basic() {
    let _guard = forbid_frame_leaks();

    let start_info = test_scaffolding::info();
    debug!(%start_info);

    {
        let mut a = Box::new(1_i64);
        let a_ptr = &mut *a as *mut i64;
        unsafe {
            a_ptr.write_volatile(a_ptr.read_volatile() + 1);
        }
        assert_eq!(*a, 2);

        let info = test_scaffolding::info();
        let requested = mem::size_of_val(&*a);
        debug!(%info);
        assert_eq!(info.allocations() - start_info.allocations(), 1);
        assert_eq!(info.deallocations() - start_info.deallocations(), 0);
        assert!(info.allocated() - start_info.allocated() >= requested);
        assert_eq!(info.allocated_pages() - start_info.allocated_pages(), 1);
        assert_eq!(info.deallocated_pages() - start_info.deallocated_pages(), 0);
    }

    let end_info = test_scaffolding::info();
    debug!(%end_info);
    assert_eq!(end_info.allocations() - start_info.allocations(), 1);
    assert_eq!(end_info.deallocations() - start_info.deallocations(), 1);
    assert_eq!(end_info.requested() - start_info.requested(), 0);
    assert_eq!(end_info.allocated() - start_info.allocated(), 0);
    assert_eq!(end_info.allocated_pages() - start_info.allocated_pages(), 1);
    assert_eq!(
        end_info.deallocated_pages() - start_info.deallocated_pages(),
        1,
    );
    assert_eq!(
        end_info.fragmentation_loss() - start_info.fragmentation_loss(),
        0,
    );
    assert_eq!(
        end_info.fragmentation_loss_percentage() - start_info.fragmentation_loss_percentage(),
        0.0,
    );

    assert_eq!(end_info.current_allocations(), 0);
    assert_eq!(end_info.current_pages(), 0);
    assert_eq!(end_info.fragmentation_loss(), 0);
}

#[test_case]
fn grow_and_shrink() {
    let _guard = forbid_frame_leaks();

    let start_info = test_scaffolding::info();
    debug!(%start_info);

    let mut vec = Vec::new();
    let mut push_sum = 0;

    for a in 1 .. 3 * Page::SIZE {
        vec.push(a);
        assert_eq!(vec.len(), a);
        push_sum += a;
    }

    let contents_sum = vec.iter().sum::<usize>();
    assert_eq!(contents_sum, push_sum);

    let info = test_scaffolding::info();
    debug!(%info);
    assert_eq!(
        info.fragmentation_loss() - start_info.fragmentation_loss(),
        0,
    );
    assert!(
        info.fragmentation_loss_percentage() - start_info.fragmentation_loss_percentage() <= 0.0,
    );

    let mut pop_sum = 0;

    while !vec.is_empty() {
        pop_sum += vec.pop().unwrap();
        if vec.len() <= vec.capacity() / 2 {
            vec.shrink_to_fit();
        }
    }

    assert_eq!(contents_sum, pop_sum);

    let end_info = test_scaffolding::info();
    debug!(%end_info);
    assert_eq!(end_info.current_allocations(), 0);
    assert_eq!(end_info.current_pages(), 0);
    assert_eq!(end_info.fragmentation_loss(), 0);
}
