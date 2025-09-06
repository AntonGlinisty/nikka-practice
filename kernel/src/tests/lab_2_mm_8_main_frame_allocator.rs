use core::mem;

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    error::Error::NoFrame,
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Frame,
        FrameAllocator,
        FrameGuard,
        KERNEL_RW,
        Page,
        USER_R,
        USER_RW,
        USER_RX,
        Virt,
        mmu::{
            self,
            PAGE_TABLE_ENTRY_COUNT,
            PAGE_TABLE_ROOT_LEVEL,
            PageTableEntry,
            PageTableFlags,
        },
        size::TiB,
        test_scaffolding::{
            duplicate,
            get_pte,
            map_page,
            path,
            physical_memory,
            switch_to,
            translate,
            unmap_page,
            user_root_level_entries,
        },
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::MEMORY);

#[test_case]
fn sanity_check() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();

    let qemu_memory_frames = 128 * MiB / Frame::SIZE;
    let min_free_frames = qemu_memory_frames - 16 * MiB / Frame::SIZE;

    debug!(free_frames, min_free_frames, qemu_memory_frames);

    assert!(free_frames > min_free_frames);
    assert!(free_frames < qemu_memory_frames);
}

#[test_case]
fn basic_frame_allocator_functions() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();
    let frame_count = init::frame_count();

    let frames = [
        take(frame_allocator.allocate().unwrap()),
        take(frame_allocator.allocate().unwrap()),
    ];

    debug!(?frames, frame_count);
    assert_ne!(frames[0], frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);
    for frame in frames {
        assert!(
            frame.index() < frame_count,
            "allocated a frame outside of the physical memory of the current machine",
        );
    }

    frame_allocator.deallocate(frames[0]);
    assert_eq!(frame_allocator.count(), start_free_frames - 1);

    let reallocate_last_freed_frame = take(frame_allocator.allocate().unwrap());

    debug!(?reallocate_last_freed_frame);
    assert_eq!(reallocate_last_freed_frame, frames[0]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    take(frame_allocator.reference(frames[1]));
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    frame_allocator.deallocate(frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    frame_allocator.deallocate(frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 1);

    frame_allocator.deallocate(frames[0]);

    let unmanaged_frame =
        Frame::containing((physical_memory().end_address().unwrap() + TiB).unwrap());
    take(frame_allocator.reference(unmanaged_frame));
    frame_allocator.deallocate(unmanaged_frame);
}

#[test_case]
fn allocated_frames_are_not_used() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let max_frames_for_intermediates: usize = PAGE_TABLE_ROOT_LEVEL.try_into().unwrap();
    let poison = 0xDEAD_BEEF_DEAD_BEEF_u64;

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let start_free_frames = FRAME_ALLOCATOR.lock().count();
    let frame_count = init::frame_count();

    let frames = address_space.map_slice(start_free_frames, USER_RW, Frame::default).unwrap();

    let page_block = address_space.allocate(Page::SIZE, KERNEL_RW).unwrap();
    let page = page_block.into_iter().next().unwrap();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();
    debug!(start_free_frames, frame_count);

    let mut frames_poisoned = 0;
    let mut prev_frame = None;

    while let Ok(frame_guard) = FrameGuard::allocate() {
        let frame = *frame_guard;

        {
            let mut frame_allocator = FRAME_ALLOCATOR.lock();
            assert_eq!(frame_allocator.reference_count(frame), Ok(1));
            take(frame_allocator.reference(frame));
            assert_eq!(frame_allocator.reference_count(frame), Ok(2));
        }

        assert!(
            frame.index() < frame_count,
            "allocated a frame {frame} outside of the physical memory of the current machine with \
             only {frame_count} frames total",
        );

        frames[frames_poisoned] = frame;

        if let Some(prev_frame) = prev_frame {
            assert_eq!(FRAME_ALLOCATOR.lock().reference_count(prev_frame), Ok(2));
        }
        unsafe {
            address_space.map_page_to_frame(page, frame_guard, KERNEL_RW).unwrap();
            mmu::flush(page);
        }
        if let Some(prev_frame) = prev_frame {
            assert_eq!(FRAME_ALLOCATOR.lock().reference_count(prev_frame), Ok(1));
        }
        prev_frame = Some(frame);

        let slice = unsafe { page_block.try_into_mut_slice().unwrap() };
        slice.fill(poison);

        if frames_poisoned % 1000 == 0 {
            debug!(
                frames_poisoned,
                %frame,
                poison = slice[frames_poisoned % slice.len()],
                "poisoning the free memory",
            );
        }

        frames_poisoned += 1;
    }

    let prev_frame = prev_frame.expect("at least one frame should be free");
    assert_eq!(FRAME_ALLOCATOR.lock().reference_count(prev_frame), Ok(2));
    unsafe {
        unmap_page(&mut address_space, page).unwrap();
    }
    assert_eq!(FRAME_ALLOCATOR.lock().reference_count(prev_frame), Ok(1));

    let end_free_frames = FRAME_ALLOCATOR.lock().count();
    debug!(start_free_frames, end_free_frames, frames_poisoned);
    assert_eq!(end_free_frames, 0);
    assert!(frames_poisoned >= start_free_frames - max_frames_for_intermediates);

    introsort::sort(&mut frames[.. frames_poisoned]);
    for (a, b) in frames.iter().zip(&frames[1 .. frames_poisoned]) {
        assert_ne!(a, b, "allocated the same frame twice");
    }

    for &mut frame in &mut frames[.. frames_poisoned] {
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        assert_eq!(frame_allocator.reference_count(frame), Ok(1));
        frame_allocator.deallocate(frame);
        assert_eq!(frame_allocator.reference_count(frame), Ok(0));
    }

    debug!(free_frames = FRAME_ALLOCATOR.lock().count());
    assert!(FRAME_ALLOCATOR.lock().count() >= start_free_frames - max_frames_for_intermediates);

    unsafe {
        address_space.unmap_slice(frames).unwrap();
    }
}

#[test_case]
fn allocated_frames_are_unique() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();
    let frame_count = init::frame_count();
    let frames = BASE_ADDRESS_SPACE
        .lock()
        .map_slice(start_free_frames, KERNEL_RW, Frame::default)
        .unwrap();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();
    debug!(free_frames, frame_count);

    for frame in &mut frames[.. free_frames] {
        *frame = take(frame_allocator.allocate().unwrap());
        assert!(
            frame.index() < frame_count,
            "allocated a frame {frame} outside of the physical memory of the current machine with \
             only {frame_count} frames total",
        );
    }

    debug!(free_frames = frame_allocator.count());
    assert_eq!(frame_allocator.count(), 0);
    assert_eq!(frame_allocator.allocate(), Err(NoFrame));

    introsort::sort(&mut frames[.. free_frames]);
    for (a, b) in frames.iter().zip(&frames[1 .. free_frames]) {
        assert_ne!(a, b, "allocated the same frame twice");
    }

    for &frame in &frames[.. free_frames] {
        frame_allocator.deallocate(frame);
    }

    debug!(free_frames = frame_allocator.count());
    assert_eq!(frame_allocator.count(), free_frames);

    drop(frame_allocator);

    unsafe {
        BASE_ADDRESS_SPACE.lock().unmap_slice(frames).unwrap();
    }
}

#[test_case]
fn shared_memory() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let frame = FrameGuard::allocate().expect("failed to allocate a frame");

    debug!(%frame);

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let mut pages_iterator = address_space.allocate(2 * Page::SIZE, KERNEL_RW).unwrap().into_iter();

    let pages = [
        pages_iterator.next().unwrap(),
        pages_iterator.next().unwrap(),
    ];

    debug!(?pages);

    for page in pages {
        let frame = FrameGuard::reference(*frame);
        unsafe {
            address_space
                .map_page_to_frame(page, frame, KERNEL_RW)
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
            unmap_page(&mut address_space, page).unwrap();
        }
    }
}

#[test_case]
fn garbage_after_unmap() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let virt = Virt::new(0x0000_4000_0000_0000).unwrap();
    let mut path = path(&mut address_space, virt);
    unsafe {
        let frame = FrameGuard::allocate().unwrap();
        path.map(frame, PageTableFlags::PRESENT).unwrap();
        path.unmap().unwrap();
    }
    let pte = get_pte(&path).unwrap();

    let expected_pte = PageTableEntry::default();

    debug!(?pte, ?expected_pte);
    assert_eq!(*pte, expected_pte);

    let end_free_frames = FRAME_ALLOCATOR.lock().count();

    assert_ne!(start_free_frames, end_free_frames);

    for neighbour in 1 .. PAGE_TABLE_ENTRY_COUNT {
        let neighbour_virt = (virt + neighbour * Page::SIZE).unwrap();
        let neighbour_pte = translate(&mut address_space, neighbour_virt).unwrap();
        if *neighbour_pte != expected_pte {
            debug!(?neighbour_pte, ?expected_pte);
        }
        assert_eq!(*neighbour_pte, expected_pte);
    }
}

#[test_case]
fn duplicate_and_drop_mapping() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let variable =
        unsafe { BASE_ADDRESS_SPACE.lock().map_slice_zeroed::<usize>(1, KERNEL_RW).unwrap() };
    let virt = Virt::from_ref(&variable[0]);
    let frame = translate(&mut BASE_ADDRESS_SPACE.lock(), virt).unwrap().frame().unwrap();

    let reference_count = FRAME_ALLOCATOR.lock().reference_count(frame).unwrap();
    debug!(%virt, %frame, %reference_count, "one address space");
    assert_eq!(reference_count, 1);

    let mut address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let reference_count = FRAME_ALLOCATOR.lock().reference_count(frame).unwrap();
    debug!(%virt, %frame, %reference_count, "two identical address spaces");
    assert_eq!(reference_count, 2);

    let ptes = [
        translate(&mut address_space, virt).unwrap(),
        translate(&mut address_space_copy, virt).unwrap(),
    ];
    let pte_addresses = [Virt::from_ref(ptes[0]), Virt::from_ref(ptes[1])];

    debug!(?ptes);
    debug!(?pte_addresses);

    assert_eq!(ptes[0], ptes[1]);
    assert_ne!(pte_addresses[0], pte_addresses[1]);

    drop(address_space_copy);

    let reference_count = FRAME_ALLOCATOR.lock().reference_count(frame).unwrap();
    debug!(%virt, %frame, %reference_count, "back to one address space");
    assert_eq!(reference_count, 1);

    unsafe {
        address_space.unmap_slice(variable).unwrap();
    }
}

#[test_case]
fn all_frames_are_freed_on_drop() {
    const PAGES_PER_ROOT_LEVEL_ENTRY: usize = PAGE_TABLE_ENTRY_COUNT.pow(PAGE_TABLE_ROOT_LEVEL);

    let _guard = mm_helpers::forbid_frame_leaks();

    let mut address_space = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();

    for (i, flags) in [USER_R, USER_RW, USER_RX].into_iter().enumerate() {
        let page =
            Page::from_index((user_root_level_entries().start + i) * PAGES_PER_ROOT_LEVEL_ENTRY)
                .unwrap();
        unsafe {
            map_page(&mut address_space, page, flags).unwrap();
        }
        let frame = translate(&mut address_space, page.address()).unwrap().frame().unwrap();
        debug!(%page, %frame, ?flags);
    }
}

#[test_case]
fn garbage_in_duplicate() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let virt = Virt::new(0x0000_0300_0000_0000).unwrap();
    let page = Page::containing(virt);

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    unsafe {
        map_page(&mut address_space, page, KERNEL_RW).unwrap();
    }

    let mut address_space_copy = duplicate(&address_space).unwrap();

    let ptes = [
        translate(&mut address_space, virt).unwrap(),
        translate(&mut address_space_copy, virt).unwrap(),
    ];
    let pte_addresses = [Virt::from_ref(ptes[0]), Virt::from_ref(ptes[1])];

    debug!(?ptes);
    debug!(?pte_addresses);

    assert_eq!(ptes[0], ptes[1]);
    assert_ne!(pte_addresses[0], pte_addresses[1]);

    for neighbour in 1 .. PAGE_TABLE_ENTRY_COUNT {
        let neighbour_virt = (virt + neighbour * Page::SIZE).unwrap();
        let expected_neighbour_pte = translate(&mut address_space, neighbour_virt).unwrap();
        let neighbour_pte = translate(&mut address_space_copy, neighbour_virt).unwrap();
        if *neighbour_pte != *expected_neighbour_pte {
            debug!(?neighbour_pte, ?expected_neighbour_pte);
        }
        assert_eq!(*neighbour_pte, *expected_neighbour_pte);
    }

    drop(address_space_copy);

    unsafe {
        unmap_page(&mut address_space, page).unwrap();
    }
}

#[test_case]
fn duplicate_only_kernel() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let page = BASE_ADDRESS_SPACE
        .lock()
        .allocate(Page::SIZE, USER_R)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let virt = page.address();

    debug!(?page);

    unsafe {
        map_page(&mut BASE_ADDRESS_SPACE.lock(), page, USER_R).unwrap();
    }

    let mut address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let ptes = [
        translate(&mut address_space, virt).unwrap(),
        translate(&mut address_space_copy, virt).unwrap(),
    ];
    debug!(?ptes);
    assert!(ptes[0].is_present());
    assert!(!ptes[1].is_present(), "do not copy mappings to user pages");

    drop(address_space_copy);

    unsafe {
        unmap_page(&mut address_space, page).unwrap();
    }
}

#[test_case]
fn duplicate_works() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let variable =
        unsafe { BASE_ADDRESS_SPACE.lock().map_slice_zeroed::<usize>(1, KERNEL_RW).unwrap() };
    variable[0] = 0;
    debug!(
        variable = variable[0],
        "original address space initializes the variable",
    );

    let address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    switch_to(&address_space_copy);

    debug!("inside the duplicate address space");
    variable[0] = 1;
    debug!(
        variable = variable[0],
        "duplicate address space modifies the variable",
    );

    switch_to(&address_space);

    drop(address_space_copy);

    debug!(
        variable = variable[0],
        "original address space reads the variable",
    );
    assert_eq!(variable[0], 1);

    unsafe {
        address_space.unmap_slice(variable).unwrap();
    }
}

#[test_case]
fn tlb_flush() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let old_mark = 1111111;
    let new_mark = 2222222;

    let old_frame_guard = get_marked_frame(old_mark);
    let new_frame_guard = get_marked_frame(new_mark);
    let old_frame = *old_frame_guard;
    let new_frame = *new_frame_guard;

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let page = address_space
        .allocate(Page::SIZE, KERNEL_RW)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    unsafe {
        address_space
            .map_page_to_frame(page, old_frame_guard, KERNEL_RW)
            .expect("failed to map a page frame");
    }

    let current_mark = get_page_mark(page);
    debug!(
        %page,
        %old_frame,
        current_mark,
        old_mark,
        "the page is currently mapped to the old frame and should contain its old mark",
    );
    assert_eq!(current_mark, old_mark);

    unsafe {
        address_space
            .map_page_to_frame(page, new_frame_guard, KERNEL_RW)
            .expect("failed to map a page frame");
    }

    let current_mark = get_page_mark(page);
    debug!(
        %page,
        %new_frame,
        current_mark,
        new_mark,
        "the page is currently mapped to the new frame and should contain its new mark",
    );
    assert_eq!(current_mark, new_mark);

    unsafe {
        unmap_page(&mut address_space, page).unwrap();
    }

    FRAME_ALLOCATOR.lock().deallocate(old_frame);
    FRAME_ALLOCATOR.lock().deallocate(new_frame);

    fn get_marked_frame(mark: u64) -> FrameGuard {
        let frame = FrameGuard::allocate().expect("failed to allocate a frame");
        take(FrameGuard::reference(*frame));

        let mut address_space = BASE_ADDRESS_SPACE.lock();

        let page = address_space
            .allocate(Page::SIZE, KERNEL_RW)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        unsafe {
            let frame = FrameGuard::reference(*frame);
            address_space
                .map_page_to_frame(page, frame, KERNEL_RW)
                .expect("failed to map a page frame");
        }

        unsafe {
            page.address().try_into_mut_ptr::<u64>().unwrap().write_volatile(mark);
        }

        let frame_mark = get_page_mark(page);
        debug!(%frame, %page, frame_mark);
        assert_eq!(mark, get_page_mark(page));

        unsafe {
            unmap_page(&mut address_space, page).unwrap();
        }

        frame
    }

    fn get_page_mark(page: Page) -> u64 {
        unsafe { page.address().try_into_ptr::<u64>().unwrap().read_volatile() }
    }
}

fn take(frame_guard: FrameGuard) -> Frame {
    let frame = *frame_guard;
    mem::forget(frame_guard);

    frame
}
