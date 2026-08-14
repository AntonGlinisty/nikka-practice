use core::mem;

use itertools::Itertools;

use ku::{
    memory::{
        Frame,
        GiB,
    },
    sync::Spinlock,
};

use crate::{
    error::Error::{
        NoFrame,
        NoPage,
        Unimplemented,
    },
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        FrameGuard,
        KERNEL_RW,
        Page,
        Phys,
        USER_R,
        USER_RW,
        Virt,
        mmu::{
            PAGE_TABLE_ENTRY_COUNT,
            PageTableEntry,
            PageTableFlags,
        },
        test_scaffolding::{
            iter_mut,
            path,
            phys2virt,
        },
    },
};

use super::mm_helpers;

pub(super) fn check_path() {
    let _guard = mm_helpers::forbid_frame_leaks();

    mm_helpers::test_path(&mut BASE_ADDRESS_SPACE.lock());
}

pub(super) fn check_translate() {
    let _guard = mm_helpers::forbid_frame_leaks();

    mm_helpers::test_translate(&mut BASE_ADDRESS_SPACE.lock());
}

fn do_not_loose_intermediate_flags(virt: Virt) {
    mm_helpers::map_intermediate(virt, KERNEL_RW).unwrap();
    mm_helpers::check_intermediate_flags(&mut BASE_ADDRESS_SPACE.lock(), virt, KERNEL_RW);

    mm_helpers::map_intermediate(virt, USER_R).unwrap();
    mm_helpers::check_intermediate_flags(&mut BASE_ADDRESS_SPACE.lock(), virt, USER_RW);
}

pub(super) fn map_intermediate() {
    let _guard = mm_helpers::forbid_frame_leaks();
    mm_helpers::check_map_intermediate(mm_helpers::unique_user_virt());
    do_not_loose_intermediate_flags(mm_helpers::unique_user_virt());
}

pub(super) fn no_excessive_intermediate_flags() {
    let _guard = mm_helpers::forbid_frame_leaks();

    mm_helpers::test_no_excessive_intermediate_flags(mm_helpers::unique_user_virt());
}

pub(super) fn huge_page() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let phys2virt = phys2virt(&address_space);
    let phys2virt_start = phys2virt.map(Phys::default()).unwrap();

    let result = path(&mut address_space, phys2virt_start).flags();
    assert_eq!(result, Err(Unimplemented));

    drop(address_space);
    assert_eq!(
        mm_helpers::map_intermediate(phys2virt_start, USER_R),
        Err(Unimplemented),
    );
}

pub(super) fn no_page() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let virt = mm_helpers::unique_user_virt();
    let result = path(&mut address_space, virt).flags();

    assert_eq!(result, Err(NoPage));
}

pub(super) fn build_map() {
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

pub(super) fn shared_memory() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let frame = FrameGuard::allocate().expect("failed to allocate a frame");

    debug!(%frame);

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let mut pages_iterator =
        address_space.allocate(Page::layout_array(2), KERNEL_RW).unwrap().into_iter();

    let pages = [
        pages_iterator.next().unwrap(),
        pages_iterator.next().unwrap(),
    ];

    debug!(?pages);

    for page in pages {
        let frame = FrameGuard::reference(*frame);
        unsafe {
            path(&mut address_space, page.address())
                .map(frame, KERNEL_RW)
                .expect("failed to map a page frame");
        }
    }

    drop(frame);

    let read_ptr: *const u64 = pages[0].address().try_into_ptr().unwrap();
    let write_ptr: *mut u64 = pages[1].address().try_into_mut_ptr().unwrap();
    debug!(?write_ptr, ?read_ptr);
    assert_ne!(read_ptr, write_ptr);

    const ITERATIONS: u64 = 3;

    for write_value in 0 .. ITERATIONS {
        let read_value = unsafe {
            write_ptr.write_volatile(write_value);
            read_ptr.read_volatile()
        };
        debug!(write_value, read_value);
        assert_eq!(read_value, write_value);
    }

    for page in pages {
        unsafe {
            path(&mut address_space, page.address()).unmap().unwrap();
        }
    }
}

pub(super) fn garbage_after_unmap() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let virt = Virt::new(0x0000_4000_0000_0000).unwrap();
    let mut virt_path = path(&mut address_space, virt);
    unsafe {
        let frame = FrameGuard::allocate().unwrap();
        virt_path.map(frame, PageTableFlags::PRESENT).unwrap();
        virt_path.unmap().unwrap();
    }
    let pte = virt_path.pte().unwrap();

    let expected_pte = PageTableEntry::default();

    debug!(?pte, ?expected_pte);
    assert_eq!(*pte, expected_pte);

    let end_free_frames = FRAME_ALLOCATOR.lock().count();

    assert_ne!(start_free_frames, end_free_frames);

    for neighbour in 1 .. PAGE_TABLE_ENTRY_COUNT {
        let neighbour_virt = (virt + neighbour * Page::SIZE).unwrap();
        let neighbour_pte = path(&mut address_space, neighbour_virt).pte().unwrap();
        if *neighbour_pte != expected_pte {
            debug!(?neighbour_pte, ?expected_pte);
        }
        assert_eq!(*neighbour_pte, expected_pte);
    }
}

pub(super) fn tlb_flush() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let old_mark = 1111111;
    let new_mark = 2222222;

    let old_frame_guard = get_marked_frame(old_mark);
    let new_frame_guard = get_marked_frame(new_mark);
    let old_frame = *old_frame_guard;
    let new_frame = *new_frame_guard;

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let page = address_space
        .allocate(Page::layout_array(1), KERNEL_RW)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    unsafe {
        path(&mut address_space, page.address())
            .map(old_frame_guard, KERNEL_RW)
            .expect("failed to map a page frame");
    }

    let mark_before_remap = get_page_mark(page);

    unsafe {
        path(&mut address_space, page.address())
            .map(new_frame_guard, KERNEL_RW)
            .expect("failed to map a page frame");
    }

    let mark_after_remap = get_page_mark(page);

    debug!(
        %page,
        %old_frame,
        mark_before_remap,
        old_mark,
        "the page is currently mapped to the old frame and should contain its old mark",
    );
    assert_eq!(mark_before_remap, old_mark);

    debug!(
        %page,
        %new_frame,
        mark_after_remap,
        new_mark,
        "the page is currently mapped to the new frame and should contain its new mark",
    );
    assert_eq!(mark_after_remap, new_mark);

    unsafe {
        path(&mut address_space, page.address()).unmap().unwrap();
    }

    FRAME_ALLOCATOR.lock().deallocate(old_frame);
    FRAME_ALLOCATOR.lock().deallocate(new_frame);

    fn get_marked_frame(mark: u64) -> FrameGuard {
        let frame = FrameGuard::allocate().expect("failed to allocate a frame");
        mem::forget(FrameGuard::reference(*frame));

        let mut address_space = BASE_ADDRESS_SPACE.lock();

        let page = address_space
            .allocate(Page::layout_array(1), KERNEL_RW)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        unsafe {
            let frame = FrameGuard::reference(*frame);
            path(&mut address_space, page.address())
                .map(frame, KERNEL_RW)
                .expect("failed to map a page frame");
        }

        unsafe {
            page.address().try_into_mut_ptr::<u64>().unwrap().write_volatile(mark);
        }

        let frame_mark = get_page_mark(page);
        debug!(%frame, %page, frame_mark);
        assert_eq!(mark, get_page_mark(page));

        unsafe {
            path(&mut address_space, page.address()).unmap().unwrap();
        }

        frame
    }

    fn get_page_mark(page: Page) -> u64 {
        unsafe { page.address().try_into_ptr::<u64>().unwrap().read_volatile() }
    }
}

pub(super) fn path_after_map() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let unique_l3_index = mm_helpers::unique_user_virt().page_table_index(3);
    let virt = Virt::from_page_table_indexes([2, 3, 4, unique_l3_index], 0);

    let mut path_a = path(&mut address_space, virt);
    debug!(path = %path_a);
    unsafe {
        path_a.map(FrameGuard::allocate().unwrap(), USER_R).unwrap();
    }
    debug!(path_after_map = %path_a);
    let pte_a = *path_a.pte().unwrap();

    let mut path_b = path(&mut address_space, virt);
    debug!(%path_b);
    let pte_b = *path_b.pte().unwrap();

    unsafe {
        path_b.unmap().unwrap();
    }

    assert_eq!(pte_a, pte_b);
}

pub(super) fn no_frame() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let frame_count = FRAME_ALLOCATOR.lock().count();
    let frame = FrameGuard::allocate().unwrap();
    let mut real_frame_count = 1;

    static FRAMES: Spinlock<[Option<FrameGuard>; GiB / Frame::SIZE]> =
        Spinlock::new([const { None }; _]);
    let mut frames = FRAMES.lock();
    assert!(
        frames.len() >= frame_count,
        "reserve more memory for frames",
    );

    let mut frames_stolen = 0;

    while let Ok(frame) = FrameGuard::allocate() {
        frames[frames_stolen] = Some(frame);
        real_frame_count += 1;
        frames_stolen += 1;
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

    frames.fill_with(|| None);
}
