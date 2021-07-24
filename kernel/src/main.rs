#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(panic_info_message)]
#![no_std]
#![no_main]

extern crate rlibc;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

use text::{
    Attribute,
    println,
};

use kernel::{
    log::debug,
    time,
    time::{
        Pit,
        Rtc,
    },
    trap::TRAP_STATS,
};

#[cold]
#[inline(never)]
#[no_mangle]
extern "C" fn _start() {
    kernel::init();

    loop {
        if let Some(frequency) = Pit::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by PIT");
        }

        if let Some(frequency) = Rtc::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by RTC");
        }

        let start = time::now();
        let stop = time::now();
        let time_precision = stop - start;
        debug!(%time_precision);

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
    text::TEXT.lock().set_attribute(Attribute::PANIC);

    println!("{}", panic_info);

    kernel::halt();
}
