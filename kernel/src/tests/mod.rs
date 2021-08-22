mod allocator;
mod memory;
mod trap;

#[macro_export]
macro_rules! forbid_frame_leaks {
    () => {
        let _leak_guard = scopeguard::guard(
            kernel::memory::FRAME_ALLOCATOR.lock().count(),
            |start_free_frames| {
                let end_free_frames = kernel::memory::FRAME_ALLOCATOR.lock().count();
                let (message, affected_frames) = if start_free_frames <= end_free_frames {
                    (
                        "freed more memory than allocated",
                        end_free_frames - start_free_frames,
                    )
                } else {
                    ("leaked some memory", start_free_frames - end_free_frames)
                };
                if affected_frames != 0 {
                    kernel::log::error!(
                        start_free_frames,
                        end_free_frames,
                        affected_frames,
                        message,
                    );
                    panic!("{}", message);
                }
            },
        );
    };
}
