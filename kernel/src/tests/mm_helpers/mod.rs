#![allow(dead_code)]

use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use ku::{
    log::debug,
    memory::{
        FULL_ACCESS,
        KERNEL_R,
        KERNEL_RW,
        Page,
        USER_R,
        USER_RW,
        Virt,
        mmu::{
            PAGE_OFFSET_BITS,
            PAGE_TABLE_ENTRY_COUNT,
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
        user_pages,
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
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let mut path = mapping.path(virt);
    path.map_intermediate(PageTableFlags::empty()).unwrap();
    let pte = get_pte(&path).unwrap();

    debug!(?pte);

    let end_free_frames = FRAME_ALLOCATOR.lock().count();
    assert!(start_free_frames > end_free_frames);

    check_intermediate_flags(mapping, virt, PageTableFlags::PRESENT);

    for flags in [KERNEL_RW, USER_RW] {
        mapping.path(virt).map_intermediate(flags).unwrap();
        assert_eq!(FRAME_ALLOCATOR.lock().count(), end_free_frames);
        check_intermediate_flags(mapping, virt, flags);
    }
}

pub(super) fn test_no_excessive_intermediate_flags(virt: Virt) {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let flags = PageTableFlags::all().difference(FULL_ACCESS);
    mapping.path(virt).map_intermediate(flags).unwrap();

    let end_free_frames = FRAME_ALLOCATOR.lock().count();
    assert!(start_free_frames > end_free_frames);

    check_intermediate_flags(mapping, virt, PageTableFlags::PRESENT);
}

pub(super) fn unique_kernel_virt() -> Virt {
    unique_virt(KERNEL_R)
}

pub(super) fn unique_user_virt() -> Virt {
    unique_virt(USER_R)
}

pub(super) fn unique_virt(flags: PageTableFlags) -> Virt {
    const ROOT_LEVEL_ENTRY_SIZE: usize =
        PAGE_TABLE_ENTRY_COUNT.pow(PAGE_TABLE_ROOT_LEVEL) * Page::SIZE;

    static UNIQUE_ROOT_LEVEL_ENTRY: AtomicUsize = AtomicUsize::new(1);

    let is_user = flags.contains(PageTableFlags::USER_ACCESSIBLE);
    let start = user_pages().start_address();
    let entry = UNIQUE_ROOT_LEVEL_ENTRY.fetch_add(1, Ordering::Relaxed);
    let misalignment = 1 << (entry % (size::from(PAGE_OFFSET_BITS) + 1));
    let offset = entry * ROOT_LEVEL_ENTRY_SIZE + misalignment;
    let virt = (if is_user {
        start + offset
    } else {
        start - offset
    })
    .unwrap();

    debug!(%virt, is_user, "allocated unique virtual address");

    virt
}
