extern crate alloc;

use alloc::{
    alloc::Layout,
    boxed::Box,
    collections::BTreeMap,
    vec::Vec,
};
use core::{
    cmp,
    mem,
};

use static_assertions::const_assert_eq;

use ku::allocator::{
    DryAllocator,
    Initialize,
};

use kernel::{
    Subsystems,
    allocator,
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        Block,
        FRAME_ALLOCATOR,
        Page,
        Virt,
        mmu::{
            PageTableEntry,
            PageTableFlags,
            USER_RW,
        },
        size::MiB,
        test_scaffolding::{
            forbid_frame_leaks,
            mapping,
        },
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn basic() {
    memory_allocator_basic();
}

#[test_case]
fn shrink_is_not_a_noop() {
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let old_layout = Layout::from_size_align(4 * Page::SIZE, Page::SIZE).unwrap();
    let old_allocation = address_space
        .allocator(USER_RW)
        .dry_allocate(old_layout, Initialize::Garbage)
        .unwrap();
    let old_block = Block::from_slice(unsafe { old_allocation.as_uninit_slice() });

    let new_layout = Layout::from_size_align(3 * Page::SIZE, Page::SIZE).unwrap();
    let new_allocation = unsafe {
        address_space
            .allocator(USER_RW)
            .dry_shrink(old_allocation.as_non_null_ptr(), old_layout, new_layout)
            .unwrap()
    };
    let new_block = Block::from_slice(unsafe { new_allocation.as_uninit_slice() });

    assert_ne!(old_block.count(), new_block.count());

    debug!(%old_block, %new_block, no_remap_on_shrink = old_block.contains_block(new_block));

    unsafe {
        address_space
            .allocator(USER_RW)
            .dry_deallocate(new_allocation.as_non_null_ptr(), new_layout);
    }
}

#[test_case]
fn grow_and_shrink() {
    memory_allocator_grow_and_shrink();
}

#[test_case]
fn paged_realloc_is_cheap() {
    let _guard = forbid_frame_leaks();

    #[repr(C, align(4096))]
    pub struct PagedType(u8);

    const_assert_eq!(mem::align_of::<PagedType>(), Page::SIZE);
    const_assert_eq!(mem::size_of::<PagedType>(), Page::SIZE);

    let mut vec = Vec::new();
    let mut prev_frames = Vec::new();

    for _ in 0 .. 5 {
        vec.push(PagedType(0));
        while vec.len() < vec.capacity() {
            vec.push(PagedType(0));
        }

        prev_frames = check_frames(&vec, prev_frames);
    }

    while !vec.is_empty() {
        vec.pop().unwrap();
        vec.shrink_to_fit();

        prev_frames = check_frames(&vec, prev_frames);
    }

    assert!(prev_frames.is_empty());
}

#[test_case]
fn stress() {
    memory_allocator_stress(10_000);
}

fn check_frames<T>(
    vec: &Vec<T>,
    prev_frames: Vec<PageTableEntry>,
) -> Vec<PageTableEntry> {
    let block = Block::from_slice(vec.as_slice()).enclosing();
    let frames: Vec<_> = block.into_iter().map(translate).collect();
    debug!(%block, ?frames);

    for pte in &frames {
        let reference_count = FRAME_ALLOCATOR.lock().reference_count(pte.frame().unwrap()).unwrap();
        assert_eq!(
            reference_count, 1,
            "after a reallocation the frames should not be shared",
        );
    }

    let len = cmp::min(frames.len(), prev_frames.len());

    assert_eq!(
        frames[.. len],
        prev_frames[.. len],
        "the paged allocator should reuse existing frames on reallocations",
    );

    frames
}

fn translate(page: Page) -> PageTableEntry {
    *mapping(&mut BASE_ADDRESS_SPACE.lock())
        .translate(page.address(), None, PageTableFlags::empty())
        .unwrap()
}

macro_rules! my_assert {
    ($cond:expr $(,)?) => {{
        assert!($cond);
    }};
}

include!("include/memory_allocator.rs");
