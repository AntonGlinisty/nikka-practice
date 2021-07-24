use core::{
    panic::PanicInfo,
    ptr::NonNull,
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
fn page_fault() {
    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    unsafe {
        NonNull::<u8>::dangling().as_ptr().read_volatile();
    }

    panic!("no page fault generated");
}
