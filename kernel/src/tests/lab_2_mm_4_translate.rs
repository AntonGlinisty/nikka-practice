use itertools::Itertools;

use kernel::{
    Subsystems,
    error::Error::{
        NoFrame,
        NoPage,
        Unimplemented,
    },
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Frame,
        Page,
        Phys,
        Virt,
        mmu::{
            KERNEL_RW,
            PAGE_OFFSET_BITS,
            PAGE_TABLE_LEAF_LEVEL,
            PAGE_TABLE_ROOT_LEVEL,
            PageTableEntry,
            PageTableFlags,
            USER,
            USER_RW,
        },
        size::{
            self,
            MiB,
        },
        test_scaffolding::{
            Mapping,
            deepest_pte,
            forbid_frame_leaks,
            get_pte,
            mapping,
            nodes,
            page_table_root,
            phys2virt,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn t00_path() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

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

    let non_present_virt = mm_helpers::unique_user_virt();
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
    assert!(pte.flags().contains(PageTableFlags::HUGE));
    assert_eq!(get_pte(&huge_path), Err(Unimplemented));
    check_path(mapping, huge_virt);
}

#[test_case]
fn t01_translate() {
    let _guard = forbid_frame_leaks();

    let mut variable = 314159265;
    let write_ptr: *mut u64 = &mut variable;

    let virt = Virt::from_ref(&variable);

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let pte = mapping.translate(virt).unwrap();

    debug!(?pte);

    let frame = pte.frame().unwrap();
    let expected_flags = PageTableFlags::PRESENT |
        PageTableFlags::WRITABLE |
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

fn do_not_loose_intermediate_flags(virt: Virt) {
    map_intermediate(virt, KERNEL_RW);
    check_intermediate(virt, KERNEL_RW);

    map_intermediate(virt, USER);
    check_intermediate(virt, USER_RW);

    fn check_intermediate(
        virt: Virt,
        flags: PageTableFlags,
    ) {
        let mut address_space = BASE_ADDRESS_SPACE.lock();
        let mapping = mapping(&mut address_space);

        mm_helpers::check_intermediate_flags(mapping, virt, flags);
    }

    fn map_intermediate(
        virt: Virt,
        flags: PageTableFlags,
    ) {
        let mut address_space = BASE_ADDRESS_SPACE.lock();
        let mapping = mapping(&mut address_space);

        let mut path = mapping.path(virt);
        path.map_intermediate(flags).unwrap();
    }
}

#[test_case]
fn t02_map_intermediate() {
    let _guard = forbid_frame_leaks();
    mm_helpers::map_intermediate(mm_helpers::unique_user_virt());
    do_not_loose_intermediate_flags(mm_helpers::unique_user_virt());
}

#[test_case]
fn t03_no_excessive_intermediate_flags() {
    mm_helpers::test_no_excessive_intermediate_flags(mm_helpers::unique_user_virt());
}

#[test_case]
fn t04_huge_page() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let phys2virt = phys2virt(mapping);
    let phys2virt_start = phys2virt.map(Phys::default()).unwrap();

    let mut path = mapping.path(phys2virt_start);
    let result = get_pte(&path);
    assert_eq!(result, Err(Unimplemented));

    let result = path.map_intermediate(USER);
    assert_eq!(result, Err(Unimplemented));
}

#[test_case]
fn t05_no_page() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let virt = mm_helpers::unique_user_virt();
    let pte = mapping.translate(virt);

    assert_eq!(pte, Err(NoPage));
}

#[test_case]
fn t06_build_map() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let flag_mask = !(PageTableFlags::ACCESSED | PageTableFlags::DIRTY);

    for ignore_frame_addresses in [false, true] {
        if ignore_frame_addresses {
            debug!("virtual address space");
        } else {
            debug!("virtual to physical mapping");
        }

        for block in mapping
            .iter_mut()
            .map(|path| path.block())
            .filter(|block| block.is_present())
            .coalesce(|a, b| a.coalesce(b, ignore_frame_addresses, flag_mask))
        {
            debug!(%block);
        }
    }
}

#[test_case]
fn t99_no_frame() {
    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let frame_count = FRAME_ALLOCATOR.lock().count();
    let mut real_frame_count = 0;

    while FRAME_ALLOCATOR.lock().allocate().is_ok() {
        real_frame_count += 1;
    }

    debug!(
        frame_count,
        real_frame_count,
        "real free frame count can be less than free frame count reported by boot frame allocator \
         since it records reference and deallocation counts but does not do that actually",
    );
    assert!(real_frame_count <= frame_count);

    let virt = mm_helpers::unique_user_virt();
    let pte = mapping.path(virt).map_intermediate(PageTableFlags::empty());
    assert_eq!(pte, Err(NoFrame));
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
        assert!(!pte.is_present() || pte.flags().contains(PageTableFlags::HUGE));
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
