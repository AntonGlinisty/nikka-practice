use kernel::{
    Subsystems,
    error::Error::NoPage,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        FrameAllocator,
        FrameGuard,
        USER_R,
        Virt,
        test_scaffolding::{
            duplicate,
            forbid_frame_leaks,
            path,
            switch_to,
            translate,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn duplicate_and_drop() {
    let _guard = forbid_frame_leaks();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let a = mm_helpers::unique_user_virt();
    let b = mm_helpers::unique_user_virt();

    {
        let frame = FrameGuard::allocate().unwrap();
        let mut path = path(&mut address_space, a);
        unsafe {
            path.map(frame, USER_R).unwrap();
            path.unmap().unwrap();
        }
        mm_helpers::check_intermediate_flags(&mut address_space, a, USER_R);
    }

    let free_frames = map_in_duplicate(&mut address_space, false, true, a);
    assert!(start_free_frames > free_frames);

    {
        let pte = translate(&mut address_space, b);
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
        let mut address_space_copy = duplicate(address_space).unwrap();

        {
            let start_free_frames = FRAME_ALLOCATOR.lock().count();
            let frame = FrameGuard::allocate().unwrap();
            let mut path = path(&mut address_space_copy, virt);
            unsafe {
                path.map(frame, USER_R).unwrap();
                path.unmap().unwrap();
            }
            let end_free_frames = FRAME_ALLOCATOR.lock().count();
            assert!(allocate_intermediate || end_free_frames == start_free_frames);
        }

        mm_helpers::check_intermediate_flags(&mut address_space_copy, virt, USER_R);
        if mapped_in_original {
            mm_helpers::check_intermediate_flags(address_space, virt, USER_R);
        }

        let free_frames = FRAME_ALLOCATOR.lock().count();

        free_frames
    }
}

#[test_case]
fn duplicate_path() {
    let _guard = forbid_frame_leaks();

    let mut address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    switch_to(&address_space_copy);

    mm_helpers::test_path(&mut address_space_copy);
}

#[test_case]
fn duplicate_translate() {
    let _guard = forbid_frame_leaks();

    let mut address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    switch_to(&address_space_copy);

    mm_helpers::test_translate(&mut address_space_copy);
}
