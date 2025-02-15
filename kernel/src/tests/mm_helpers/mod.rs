#![allow(dead_code)]

use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use ku::{
    error::Error::{
        NoPage,
        Unimplemented,
    },
    log::debug,
    memory::{
        FULL_ACCESS,
        Frame,
        KERNEL_R,
        KERNEL_RW,
        Page,
        Phys,
        USER_R,
        USER_RW,
        Virt,
        mmu::{
            PAGE_OFFSET_BITS,
            PAGE_TABLE_ENTRY_COUNT,
            PAGE_TABLE_LEAF_LEVEL,
            PAGE_TABLE_ROOT_LEVEL,
            PageTableEntry,
            PageTableFlags,
        },
        size::{
            self,
            MiB,
        },
    },
};

use kernel::memory::{
    AddressSpace,
    BASE_ADDRESS_SPACE,
    FRAME_ALLOCATOR,
    test_scaffolding::{
        Mapping,
        deepest_pte,
        get_pte,
        mapping,
        nodes,
        page_table_root,
        phys2virt,
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

    let leaf_flags = PageTableFlags::EXECUTABLE;
    let pte = mapping.translate(virt).unwrap();
    assert_eq!(
        pte.flags(),
        leaf_flags,
        "leaf page table flags are incorrect",
    );
}

fn check_path(
    mapping: &mut Mapping,
    virt: Virt,
) {
    let page_table_root = Page::new(Virt::from_ref(page_table_root(mapping))).unwrap();
    let path = mapping.path(virt);

    assert_eq!(
        Page::containing(nodes(&path)[size::from(PAGE_TABLE_ROOT_LEVEL)]),
        page_table_root,
        "PTE at PAGE_TABLE_ROOT_LEVEL should be the inside the root page table in any Path",
    );

    assert!(
        nodes(&path).iter().is_partitioned(|&pte| pte == Virt::default()),
        "only lower PTEs can be absent in a Path",
    );

    let (level, pte) = deepest_pte(&path);
    if level > 0 {
        assert!(!pte.is_present() || pte.is_huge());
    }

    for (&child, &parent) in nodes(&path).iter().zip(nodes(&path)[1 ..].iter()) {
        if child != Virt::default() {
            let child_page = Page::containing(child);
            let phys2virt_start_page =
                Page::containing(phys2virt(mapping).map(Phys::default()).unwrap());
            let child_frame_index = (child_page - phys2virt_start_page).unwrap();
            let parent_points_to =
                unsafe { parent.try_into_ref::<PageTableEntry>().unwrap().frame().unwrap() };
            assert_eq!(child_frame_index, parent_points_to.index());
        }
    }
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

pub(super) fn test_path(address_space: &mut AddressSpace) {
    let mapping = mapping(address_space);

    let variable = 314159265;
    let present_virt = Virt::from_ref(&variable);
    let present_path = mapping.path(present_virt);
    let (level, pte) = deepest_pte(&present_path);
    debug!("present:");
    debug!("    virt = {present_virt}");
    debug!("    path = {present_path}");
    debug!("    level = {level}");
    debug!("    pte = {pte:?}");
    assert_eq!(level, PAGE_TABLE_LEAF_LEVEL);
    assert!(pte.is_present());
    assert!(get_pte(&present_path).is_ok());
    check_path(mapping, present_virt);

    let non_present_virt = unique_user_virt();
    let non_present_path = mapping.path(non_present_virt);
    let (level, pte) = deepest_pte(&non_present_path);
    debug!("non-present:");
    debug!("    virt = {non_present_virt}");
    debug!("    path = {non_present_path}");
    debug!("    level = {level}");
    debug!("    pte = {pte:?}");
    assert!(level > PAGE_TABLE_LEAF_LEVEL);
    assert_ne!(
        Virt::from_ref(pte),
        Page::containing(Virt::from_ref(pte)).address(),
        "nodes should be addresses of PageTableEntries, not addresses of PageTables",
    );
    assert!(!pte.is_present());
    assert_eq!(get_pte(&non_present_path), Err(NoPage));
    check_path(mapping, non_present_virt);

    let huge_virt = phys2virt(mapping).map(Phys::new(7 * MiB).unwrap()).unwrap();
    let huge_path = mapping.path(huge_virt);
    let (level, pte) = deepest_pte(&huge_path);
    debug!("huge:");
    debug!("    virt = {huge_virt}");
    debug!("    path = {huge_path}");
    debug!("    level = {level}");
    debug!("    pte = {pte:?}");
    assert_ne!(
        level, PAGE_TABLE_LEAF_LEVEL,
        "a Path should not descend deeper into a huge page",
    );
    assert!(pte.is_present());
    assert!(pte.is_huge());
    assert_eq!(get_pte(&huge_path), Err(Unimplemented));
    check_path(mapping, huge_virt);
}

pub(super) fn test_translate(address_space: &mut AddressSpace) {
    let mut variable = 314159265;
    let write_ptr: *mut u64 = &mut variable;

    let virt = Virt::from_ref(&variable);

    let mapping = mapping(address_space);

    let pte = mapping.translate(virt).unwrap();

    debug!(?pte);

    let frame = pte.frame().unwrap();
    let expected_flags = PageTableFlags::PRESENT |
        PageTableFlags::WRITABLE |
        PageTableFlags::EXECUTABLE |
        PageTableFlags::ACCESSED |
        PageTableFlags::DIRTY;

    assert_eq!(pte.flags(), expected_flags);
    assert_ne!(frame, Frame::from_index(0).unwrap());

    let phys2virt = phys2virt(mapping);

    let page_offset_mask = (1 << PAGE_OFFSET_BITS) - 1;
    let page_offset = virt.into_usize() & page_offset_mask;
    let page_virt = phys2virt.map(frame.address()).unwrap();
    let alternative_virt = (page_virt + page_offset).unwrap();
    let read_ptr: *const u64 = alternative_virt.try_into_ptr().unwrap();

    debug!(?read_ptr, ?write_ptr);
    assert_ne!(read_ptr, write_ptr);

    const ITERATIONS: u64 = 5;

    for write_value in 0 .. ITERATIONS {
        let read_value = unsafe {
            write_ptr.write_volatile(write_value);
            read_ptr.read_volatile()
        };
        debug!(write_value, read_value, variable);
        assert_eq!(read_value, write_value);
        assert_eq!(read_value, variable);
    }
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

    let is_user = flags.is_user();
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
