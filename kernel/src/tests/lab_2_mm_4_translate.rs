use core::mem;

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
        FrameAllocator,
        FrameGuard,
        KERNEL_RW,
        Phys,
        USER_R,
        USER_RW,
        Virt,
        mmu::PageTableFlags,
        test_scaffolding::{
            get_pte,
            iter_mut,
            path,
            phys2virt,
            translate,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn t00_path() {
    let _guard = mm_helpers::forbid_frame_leaks();

    mm_helpers::test_path(&mut BASE_ADDRESS_SPACE.lock());
}

#[test_case]
fn t01_translate() {
    let _guard = mm_helpers::forbid_frame_leaks();

    mm_helpers::test_translate(&mut BASE_ADDRESS_SPACE.lock());
}

fn do_not_loose_intermediate_flags(virt: Virt) {
    mm_helpers::map_intermediate(virt, KERNEL_RW).unwrap();
    mm_helpers::check_intermediate_flags(&mut BASE_ADDRESS_SPACE.lock(), virt, KERNEL_RW);

    mm_helpers::map_intermediate(virt, USER_R).unwrap();
    mm_helpers::check_intermediate_flags(&mut BASE_ADDRESS_SPACE.lock(), virt, USER_RW);
}

#[test_case]
fn t02_map_intermediate() {
    let _guard = mm_helpers::forbid_frame_leaks();
    mm_helpers::check_map_intermediate(mm_helpers::unique_user_virt());
    do_not_loose_intermediate_flags(mm_helpers::unique_user_virt());
}

#[test_case]
fn t03_no_excessive_intermediate_flags() {
    mm_helpers::test_no_excessive_intermediate_flags(mm_helpers::unique_user_virt());
}

#[test_case]
fn t04_huge_page() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let phys2virt = phys2virt(&address_space);
    let phys2virt_start = phys2virt.map(Phys::default()).unwrap();

    let path = path(&mut address_space, phys2virt_start);
    let result = get_pte(&path);
    assert_eq!(result, Err(Unimplemented));

    drop(address_space);
    assert_eq!(
        mm_helpers::map_intermediate(phys2virt_start, USER_R),
        Err(Unimplemented),
    );
}

#[test_case]
fn t05_no_page() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let virt = mm_helpers::unique_user_virt();
    let pte = translate(&mut address_space, virt);

    assert_eq!(pte, Err(NoPage));
}

#[test_case]
fn t06_build_map() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let flag_mask = !(PageTableFlags::ACCESSED | PageTableFlags::DIRTY);

    for ignore_frame_addresses in [false, true] {
        if ignore_frame_addresses {
            debug!("virtual address space");
        } else {
            debug!("virtual to physical mapping");
        }

        for block in iter_mut(&mut address_space)
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

    let frame_count = FRAME_ALLOCATOR.lock().count();
    let frame = FrameGuard::allocate().unwrap();
    let mut real_frame_count = 1;

    while let Ok(frame) = FrameGuard::allocate() {
        mem::forget(frame);
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
    let mut path = path(&mut address_space, virt);
    assert_eq!(
        unsafe { path.map(frame, PageTableFlags::empty()) },
        Err(NoFrame),
    );
}
