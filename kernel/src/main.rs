#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(abi_x86_interrupt)]
#![feature(format_args_capture)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

#[allow(unused_extern_crates)]
extern crate lazy_static;
extern crate rlibc;
extern crate tracing;
extern crate tracing_core;
extern crate x86;
extern crate x86_64;

extern crate pic8259;
extern crate text;

mod log;
mod memory;
mod trap;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

use text::{
    Attribute,
    println,
};

use log::{
    debug,
    info,
};
use memory::gdt;
use trap::TRAP_STATS;

#[cold]
#[inline(never)]
#[no_mangle]
extern "C" fn _start() {
    text::TEXT.lock().init();
    log::init();

    info!("Nikka booted");

    gdt::init();
    trap::init();

    loop {
        for (number, stats) in TRAP_STATS.iter().enumerate() {
            let count = stats.count();
            if count != 0 {
                let mnemonic = stats.mnemonic();
                debug!(number, %mnemonic, count, "trap stats");
            }
        }

        unsafe {
            x86::halt();
        }
    }
}

#[cold]
#[inline(never)]
fn halt() -> ! {
    loop {
        unsafe {
            x86::irq::disable();
            x86::halt();
        }
    }
}

#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    text::TEXT.lock().set_attribute(Attribute::PANIC);

    println!("{panic_info}");

    halt();
}
