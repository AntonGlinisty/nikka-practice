use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use bootloader::bootinfo::MemoryMap;

use kernel::memory::test_scaffolding;

#[macro_export]
macro_rules! init {
    ($subsystems:expr) => {
        bootloader::entry_point!(test_entry);

        fn test_entry(boot_info: &'static bootloader::BootInfo) -> ! {
            kernel::init_subsystems(&boot_info, $subsystems);
            init::set_frame_count(&boot_info.memory_map);
            test_main();
            panic!("should not return to test_entry()")
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            ku::sync::start_panicking();
            kernel::fail_test(info)
        }
    };
}

#[allow(dead_code)]
pub(super) fn frame_count() -> usize {
    FRAME_COUNT.load(Ordering::Relaxed)
}

pub(super) fn set_frame_count(memory_map: &MemoryMap) {
    FRAME_COUNT.store(test_scaffolding::frame_count(memory_map), Ordering::Relaxed);
}

static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);
