#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![deny(warnings)]
#![feature(custom_test_frameworks)]
#![feature(maybe_uninit_slice)]
#![feature(panic_info_message)]
#![feature(sort_internals)]
#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

extern crate alloc;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

use bootloader::{
    BootInfo,
    entry_point,
};

use ku::time::rtc::Rtc;
use text::{
    Attribute,
    println,
};

use kernel::{
    Subsystems,
    log::debug,
    time,
    time::Pit,
    trap::TRAP_STATS,
};

entry_point!(kernel_main);

#[cold]
#[inline(never)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    kernel::init_subsystems(boot_info, Subsystems::empty());

    #[cfg(test)]
    test_main();

    loop {
        if let Some(frequency) = Pit::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by PIT");
        }

        if let Some(frequency) = Rtc::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by RTC");
        }

        let time_precision = time::timer().lap();
        debug!(%time_precision, time_precision_in_tsc = ?time_precision);

        for (number, stats) in TRAP_STATS.iter().enumerate() {
            let count = stats.count();
            if count != 0 {
                let mnemonic = stats.mnemonic();
                debug!(number, %mnemonic, count, "trap stats");
            }
        }

        x86_64::instructions::hlt();
    }
}

#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if cfg!(test) {
        kernel::fail_test(panic_info)
    } else {
        text::TEXT.lock().set_attribute(Attribute::PANIC);

        println!("{panic_info}");

        kernel::halt()
    }
}
