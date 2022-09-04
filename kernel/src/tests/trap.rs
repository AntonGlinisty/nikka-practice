use core::panic::PanicInfo;

use bootloader::{
    BootInfo,
    entry_point,
};

use kernel::{
    log::info,
    trap::{
        BREAKPOINT,
        TRAP_STATS,
    },
};

entry_point!(test_entry);

fn test_entry(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    test_main();
    panic!("should not return to test_entry()")
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::fail_test(info)
}

fn emit_breakpoint_trap() {
    info!("emitting breakpoint interrupt (int3)");
    x86_64::instructions::interrupts::int3();
}

#[test_case]
fn traps_are_working() {
    emit_breakpoint_trap();
}

#[test_case]
fn trap_counter() {
    let breakpoint_counter = &TRAP_STATS[BREAKPOINT];
    let begin_count = breakpoint_counter.count();
    let count = 3;
    for _ in 0 .. count {
        emit_breakpoint_trap();
    }
    let end_count = breakpoint_counter.count();
    info!(begin_count, count, end_count);
    assert_eq!(begin_count + count, end_count);
}
