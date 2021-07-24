#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(abi_x86_interrupt)]
#![feature(format_args_capture)]
#![no_std]

extern crate rlibc;

pub mod log;
pub mod memory;
pub mod time;
pub mod trap;

use log::info;
use memory::gdt;

#[cold]
#[inline(never)]
pub fn init() {
    text::TEXT.lock().init();
    log::init();
    time::init();

    info!(now = %time::now(), tsc = time::tsc(), "Nikka booted");

    gdt::init();
    trap::init();
}

#[cold]
#[inline(never)]
pub fn halt() -> ! {
    loop {
        unsafe {
            x86::irq::disable();
            x86::halt();
        }
    }
}
