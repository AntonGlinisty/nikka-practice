use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    log::debug,
    memory::{
        Block,
        FRAME_ALLOCATOR,
        Frame,
        test_scaffolding::{
            forbid_frame_leaks,
            get_boot,
            is_managed,
            is_used,
        },
    },
};

mod init;

init!(Subsystems::PHYS_MEMORY);

#[test_case]
fn sanity_check() {
    let _guard = forbid_frame_leaks();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();
    let frame_allocator = get_boot(&mut frame_allocator).unwrap();

    let qemu_memory_frames = 128 * MiB / Frame::SIZE;
    let min_free_frames = qemu_memory_frames - 16 * MiB / Frame::SIZE;

    debug!(free_frames, min_free_frames, qemu_memory_frames);

    assert!(free_frames > min_free_frames);
    assert!(free_frames < qemu_memory_frames);

    let mut managed = 0;
    let mut used = 0;

    for frame in Block::<Frame>::from_index(0, qemu_memory_frames).unwrap() {
        if is_managed(&*frame_allocator, frame) {
            managed += 1;
        }
        if is_used(&*frame_allocator, frame) {
            used += 1;
        }
    }

    debug!(managed, used);

    assert_eq!(managed, free_frames);
    assert_eq!(used, 0);
}

#[test_case]
fn allocate() {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let start_free_frames = frame_allocator.count();

    let frames = [
        frame_allocator.allocate().unwrap(),
        frame_allocator.allocate().unwrap(),
    ];

    debug!(?frames);
    assert_ne!(frames[0], frames[1]);
    assert_eq!(frame_allocator.count(), start_free_frames - 2);
}

#[test_case]
fn allocated_frames_are_unique() {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let free_frames = frame_allocator.count();

    debug!(free_frames);

    let mut direction = None;
    let mut prev_frame = None;

    for i in 0 .. free_frames {
        let frame = frame_allocator.allocate().unwrap();

        if let Some(prev_frame) = prev_frame {
            if i % 10_000 == 1 {
                debug!(%prev_frame, %frame);
            }

            assert_ne!(prev_frame, frame, "allocated the same frame twice");

            if let Some(direction) = direction {
                assert_eq!(
                    direction,
                    prev_frame < frame,
                    "expected either ascending or descending order of allocations from \
                     BootFrameAllocator",
                );
            } else {
                direction = Some(prev_frame < frame);
            }
        }

        prev_frame = Some(frame);
    }

    assert_eq!(frame_allocator.count(), 0);
    assert!(frame_allocator.allocate().is_err());
}
