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
        Virt,
        mmu::{
            PAGE_OFFSET_BITS,
            PageTableFlags,
            USER,
        },
        test_scaffolding::{
            forbid_frame_leaks,
            mapping,
            phys2virt,
            phys2virt_map,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

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
    let page_virt = phys2virt_map(phys2virt, frame.address());
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

#[test_case]
fn t02_map_intermediate() {
    let _guard = forbid_frame_leaks();
    mm_helpers::map_intermediate(Virt::new(0xFFFF_FFFF_FFFF_FFFF).unwrap());
}

#[test_case]
fn t03_no_excessive_intermediate_flags() {
    mm_helpers::test_no_excessive_intermediate_flags(Virt::new(0xFFFF_8FFF_FFFF_FFFF).unwrap());
}

#[test_case]
fn t04_huge_page() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let phys2virt = phys2virt(mapping);
    let result = mapping.path(phys2virt.address()).map_intermediate(USER);

    assert_eq!(result, Err(Unimplemented));
}

#[test_case]
fn t05_no_page() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);

    let virt = Virt::new(0xFFFF_FFFF_FFDF_FFFF).unwrap();
    let pte = mapping.translate(virt);

    assert_eq!(pte, Err(NoPage));
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

    let virt = Virt::new(0xFFFF_FFFF_FFAF_FFFF).unwrap();
    let pte = mapping.path(virt).map_intermediate(PageTableFlags::empty());
    assert_eq!(pte, Err(NoFrame));
}
