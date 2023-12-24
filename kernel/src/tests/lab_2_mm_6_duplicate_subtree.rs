use kernel::{
    Subsystems,
    error::Error::NoPage,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Virt,
        mmu::{
            PageTableFlags,
            USER,
        },
        test_scaffolding::{
            duplicate,
            forbid_frame_leaks,
            mapping,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn duplicate_and_drop() {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let a = Virt::new(0xFFFF_9FFF_FFFF_FFFF).unwrap();
    let b = Virt::new(0xFFFF_AFFF_FFFF_FFFF).unwrap();

    {
        let original_mapping = mapping(&mut address_space);
        original_mapping.translate(a, Some(&mut FRAME_ALLOCATOR.lock()), USER).unwrap();
        mm_helpers::check_intermediate_flags(original_mapping, a, USER);
    }

    let free_frames = map_in_duplicate(&mut address_space, false, true, a);
    assert!(start_free_frames > free_frames);

    {
        let original_mapping = mapping(&mut address_space);
        let pte = original_mapping.translate(b, None, PageTableFlags::empty());
        assert_eq!(pte, Err(NoPage));
    }

    let end_free_frames = map_in_duplicate(&mut address_space, true, false, b);
    assert!(free_frames > end_free_frames);

    fn map_in_duplicate(
        address_space: &mut AddressSpace,
        allocate_intermediate: bool,
        mapped_in_original: bool,
        virt: Virt,
    ) -> usize {
        let _guard = forbid_frame_leaks();

        let mut address_space_copy = duplicate(address_space).unwrap();
        let mapping_copy = mapping(&mut address_space_copy);
        let original_mapping = mapping(address_space);

        if allocate_intermediate {
            mapping_copy.translate(virt, Some(&mut FRAME_ALLOCATOR.lock()), USER).unwrap();
        } else {
            mapping_copy.translate(virt, None, USER).unwrap();
        }

        mm_helpers::check_intermediate_flags(mapping_copy, virt, USER);
        if mapped_in_original {
            mm_helpers::check_intermediate_flags(original_mapping, virt, USER);
        }

        let free_frames = FRAME_ALLOCATOR.lock().count();

        free_frames
    }
}
