use kernel::{
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
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
