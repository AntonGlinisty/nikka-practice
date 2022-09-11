use core::slice;

use kernel::{
    Subsystems,
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Frame,
        KERNEL_RW,
        Page,
        test_scaffolding::forbid_frame_leaks,
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn basic_frame_allocator_functions() {
    let _guard = forbid_frame_leaks();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();

    let frames = [
        frame_allocator.allocate().unwrap(),
        frame_allocator.allocate().unwrap(),
    ];

    debug!(?frames);
    assert_ne!(frames[0], frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);

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
    let frames = BASE_ADDRESS_SPACE
        .lock()
        .map_slice(start_free_frames, KERNEL_RW, Frame::default)
        .unwrap();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();
    debug!(free_frames);

    for frame in &mut frames[.. free_frames] {
        *frame = frame_allocator.allocate().unwrap();
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
fn allocate_page() {
    let page = BASE_ADDRESS_SPACE
        .lock()
        .allocate(Page::SIZE)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    debug!(?page);
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
