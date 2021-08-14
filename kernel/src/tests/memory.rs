use kernel::{
    log::debug,
    memory::FRAME_ALLOCATOR,
};

#[test_case]
fn allocate_frame() {
    let frame = FRAME_ALLOCATOR.lock().allocate().expect("failed to allocate a memory frame");

    debug!(?frame);
}
