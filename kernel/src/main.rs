#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![deny(warnings)]
#![feature(custom_test_frameworks)]
#![no_std]
#![no_main]
#![test_runner(kernel::test_runner)]

extern crate alloc;

use bootloader_api::{
    BootInfo,
    entry_point,
};

#[cfg(not(feature = "tests"))]
use ku::time::{
    pit8254::Pit,
    rtc::Rtc,
};

#[cfg(not(feature = "conservative-backtraces"))]
use sentinel_frame::with_sentinel_frame;

use kernel::{
    self,
    BOOTLOADER_CONFIG,
};

#[cfg(feature = "tests")]
use kernel::tests;

#[cfg(not(feature = "tests"))]
use kernel::{
    ExitCode,
    log::debug,
    time,
    trap::{
        TRAP_STATS,
        Trap,
    },
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg_attr(not(feature = "conservative-backtraces"), with_sentinel_frame)]
#[cold]
#[inline(never)]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    #[cfg(feature = "tests")]
    tests::main(boot_info);

    #[cfg(not(feature = "tests"))]
    main(boot_info);
}

#[cfg(not(feature = "tests"))]
#[inline(always)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    while TRAP_STATS[Trap::Rtc].count() < 10 {
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

    kernel::exit_qemu(ExitCode::SUCCESS);
}
