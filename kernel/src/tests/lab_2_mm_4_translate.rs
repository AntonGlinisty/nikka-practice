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
        KERNEL_RW,
        Phys,
        USER_R,
        USER_RW,
        Virt,
        mmu::PageTableFlags,
        test_scaffolding::{
            forbid_frame_leaks,
            get_pte,
            mapping,
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

    mm_helpers::test_path(&mut BASE_ADDRESS_SPACE.lock());
}

#[test_case]
fn t01_translate() {
    let _guard = forbid_frame_leaks();

    mm_helpers::test_translate(&mut BASE_ADDRESS_SPACE.lock());
}

fn do_not_loose_intermediate_flags(virt: Virt) {
    map_intermediate(virt, KERNEL_RW);
    check_intermediate(virt, KERNEL_RW);

    map_intermediate(virt, USER_R);
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

    let result = path.map_intermediate(USER_R);
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
