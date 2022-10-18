use core::slice;

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    log::debug,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Frame,
        KERNEL_RW,
        Page,
        Virt,
        mmu::{
            PageTableEntry,
            PageTableFlags,
        },
        test_scaffolding::{
            duplicate,
            forbid_frame_leaks,
            mapping,
        },
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn sanity_check() {
    let _guard = forbid_frame_leaks();

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
    let _guard = forbid_frame_leaks();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();
    let frame_count = init::frame_count();

    let frames = [
        frame_allocator.allocate().unwrap(),
        frame_allocator.allocate().unwrap(),
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

    let reallocate_last_freed_frame = frame_allocator.allocate().unwrap();

    debug!(?reallocate_last_freed_frame);
    assert_eq!(reallocate_last_freed_frame, frames[0]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    frame_allocator.reference(frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    frame_allocator.deallocate(frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

    frame_allocator.deallocate(frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 1);

    frame_allocator.deallocate(frames[0]);
}

#[test_case]
fn allocated_frames_are_unique() {
    let _guard = forbid_frame_leaks();

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
        *frame = frame_allocator.allocate().unwrap();
        assert!(
            frame.index() < frame_count,
            "allocated a frame {frame} outside of the physical memory of the current machine with \
             only {frame_count} frames total",
        );
    }

    debug!(free_frames = frame_allocator.count());
    assert_eq!(frame_allocator.count(), 0);
    assert!(frame_allocator.allocate().is_err());

    slice::heapsort(&mut frames[.. free_frames], |a, b| a < b);
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
    let _guard = forbid_frame_leaks();

    let frame = FRAME_ALLOCATOR.lock().allocate().expect("failed to allocate a frame");

    debug!(?frame);

    FRAME_ALLOCATOR.lock().reference(frame);

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let mut pages_iterator = address_space.allocate(2 * Page::SIZE).unwrap().into_iter();

    let pages = [
        pages_iterator.next().unwrap(),
        pages_iterator.next().unwrap(),
    ];

    debug!(?pages);

    unsafe {
        for page in pages {
            address_space
                .map_page_to_frame(page, frame, KERNEL_RW)
                .expect("failed to map a page frame");
        }
    }

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
            address_space.unmap_page(page).unwrap();
        }
    }
}

#[test_case]
fn duplicate_and_drop_mapping() {
    let _guard = forbid_frame_leaks();

    let mut variable = 0;
    let write_ptr: *mut u64 = &mut variable;

    unsafe {
        write_ptr.write_volatile(314159265);
    }

    let virt = Virt::from_ref(&variable);

    let mut address_space_copy = duplicate(&BASE_ADDRESS_SPACE.lock()).unwrap();
    let mut address_space = &mut BASE_ADDRESS_SPACE.lock();

    let ptes = [
        translate(&mut address_space, virt),
        translate(&mut address_space_copy, virt),
    ];
    let pte_addresses = [Virt::from_ref(ptes[0]), Virt::from_ref(ptes[1])];

    debug!(?ptes);
    debug!(?pte_addresses);

    assert_eq!(ptes[0], ptes[1]);
    assert_ne!(pte_addresses[0], pte_addresses[1]);

    fn translate(
        address_space: &mut AddressSpace,
        virt: Virt,
    ) -> &mut PageTableEntry {
        mapping(address_space).translate(virt, None, PageTableFlags::empty()).unwrap()
    }
}
