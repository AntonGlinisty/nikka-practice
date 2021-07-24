use core::{
    panic::PanicInfo,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use kernel::{
    log::info,
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

#[no_mangle]
extern "C" fn _start() {
    kernel::init();
    test_main();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let page_fault_count = TRAP_STATS[PAGE_FAULT].count();
    info!(page_fault_count);
    if page_fault_count == 1 {
        kernel::pass_test()
    } else {
        kernel::fail_test(info)
    }
}

#[test_case]
fn stack_overflow() {
    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    static PREVENT_OPTIMISATION: AtomicUsize = AtomicUsize::new(0);

    deep_recursion(1_000_000_000);

    panic!("no page fault generated");

    fn deep_recursion(iteration: usize) {
        if iteration > 0 {
            if PREVENT_OPTIMISATION.load(Ordering::Relaxed) % 2 == 0 {
                PREVENT_OPTIMISATION.fetch_add(1, Ordering::Relaxed);
            } else {
                PREVENT_OPTIMISATION.fetch_sub(1, Ordering::Relaxed);
            }

            deep_recursion(iteration - 1);
            deep_recursion(iteration - 1);
        }
    }
}
