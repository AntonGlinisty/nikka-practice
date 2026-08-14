use ku::memory::size::MiB;

use crate::{
    log::debug,
    memory::{
        FRAME_ALLOCATOR,
        Frame,
    },
};

use super::mm_helpers;

pub(super) fn sanity_check() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();

    let qemu_memory_frames = 128 * MiB / Frame::SIZE;
    let min_free_frames = qemu_memory_frames - 48 * MiB / Frame::SIZE;

    debug!(free_frames, min_free_frames, qemu_memory_frames);

    assert!(free_frames > min_free_frames);
    assert!(free_frames < qemu_memory_frames);
}
