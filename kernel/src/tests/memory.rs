use kernel::{
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        KERNEL_RW,
        Page,
    },
};

#[test_case]
fn allocate_frame() {
    let frame = FRAME_ALLOCATOR.lock().allocate().expect("failed to allocate a memory frame");

    debug!(?frame);
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
    let frame = FRAME_ALLOCATOR.lock().allocate().expect("failed to allocate a frame");

    debug!(?frame);

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
}
