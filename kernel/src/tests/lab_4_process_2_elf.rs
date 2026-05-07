use ku::sync::Spinlock;

use kernel::{
    Subsystems,
    log::{
        debug,
        info,
    },
    memory::{
        FRAME_ALLOCATOR,
        Frame,
        FrameGuard,
        size::GiB,
    },
    process,
};

mod init;
mod mm_helpers;
mod process_helpers;

init!(Subsystems::MEMORY);

const LOOP_ELF: &[u8] = page_aligned!("../../target/kernel/user/loop");

#[test_case]
fn create_process() {
    let _guard = mm_helpers::forbid_frame_leaks();

    process_helpers::make(LOOP_ELF);
}

#[test_case]
fn create_process_failure() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let bad_elf_file: &[u8] = &[];
    let error = process::create(bad_elf_file).expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}

#[test_case]
fn create_process_no_frame() {
    let _guard = mm_helpers::forbid_frame_leaks();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let process = process_helpers::make(LOOP_ELF);

    let required_free_frames_for_elf =
        start_free_frames.checked_sub(FRAME_ALLOCATOR.lock().count()).unwrap();
    drop(process);
    debug!(required_free_frames_for_elf);

    static FRAMES: Spinlock<[Option<FrameGuard>; GiB / Frame::SIZE]> =
        Spinlock::new([const { None }; _]);

    for free_frames in 0 .. required_free_frames_for_elf {
        let _guard = mm_helpers::forbid_frame_leaks();

        let mut frames = FRAMES.lock();
        assert!(
            frames.len() >= start_free_frames,
            "reserve more memory for frames",
        );

        let mut frames_stolen = 0;
        while FRAME_ALLOCATOR.lock().count() > free_frames {
            frames[frames_stolen] = Some(FRAME_ALLOCATOR.lock().allocate().unwrap());
            frames_stolen += 1;
        }

        debug!(free_frames = FRAME_ALLOCATOR.lock().count());

        {
            let _guard = mm_helpers::forbid_frame_leaks();
            process::create(LOOP_ELF).unwrap_err();
        }

        for i in 0 .. frames_stolen {
            frames[i] = None;
        }
    }
}
