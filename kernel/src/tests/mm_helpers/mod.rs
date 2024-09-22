#![allow(dead_code)]

use ku::{
    log::debug,
    memory::{
        FULL_ACCESS,
        KERNEL_RW,
        USER_RW,
        Virt,
        mmu::{
            PAGE_TABLE_ROOT_LEVEL,
            PageTableEntry,
            PageTableFlags,
        },
        size,
    },
};

use kernel::memory::{
    BASE_ADDRESS_SPACE,
    FRAME_ALLOCATOR,
    test_scaffolding::{
        Mapping,
        get_pte,
        mapping,
        nodes,
    },
};

pub(super) fn check_intermediate_flags(
    mapping: &mut Mapping,
    virt: Virt,
    expected_flags: PageTableFlags,
) {
    let path = mapping.path(virt);
    let intermediate_flags = unsafe {
        nodes(&path)[size::from(PAGE_TABLE_ROOT_LEVEL)]
            .try_into_ref::<PageTableEntry>()
            .unwrap()
            .flags()
    };
    assert_eq!(
        intermediate_flags, expected_flags,
        "intermediate page table flags are incorrect",
    );

    let leaf_flags = PageTableFlags::empty();
    let pte = mapping.translate(virt).unwrap();
    assert_eq!(
        pte.flags(),
        leaf_flags,
        "leaf page table flags are incorrect",
    );
}

pub(super) fn map_intermediate(virt: Virt) {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let mut path = mapping.path(virt);
    path.map_intermediate(&mut frame_allocator, PageTableFlags::empty()).unwrap();
    let pte = get_pte(&path).unwrap();

    debug!(?pte);

    let end_free_frames = frame_allocator.count();
    assert!(start_free_frames > end_free_frames);

    check_intermediate_flags(mapping, virt, PageTableFlags::PRESENT);

    for flags in [KERNEL_RW, USER_RW] {
        mapping.path(virt).map_intermediate(&mut frame_allocator, flags).unwrap();
        assert_eq!(frame_allocator.count(), end_free_frames);
        check_intermediate_flags(mapping, virt, flags);
    }
}

pub(super) fn test_no_excessive_intermediate_flags(virt: Virt) {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let flags = PageTableFlags::all().difference(FULL_ACCESS);
    mapping.path(virt).map_intermediate(&mut frame_allocator, flags).unwrap();

    let end_free_frames = frame_allocator.count();
    assert!(start_free_frames > end_free_frames);

    check_intermediate_flags(mapping, virt, PageTableFlags::PRESENT);
}
