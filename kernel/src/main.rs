#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(format_args_capture)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

#[allow(unused_extern_crates)]
extern crate rlibc;
extern crate x86;

extern crate text;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

use text::{
    Attribute,
    println,
};

#[cold]
#[inline(never)]
#[no_mangle]
extern "C" fn _start() {
    text::TEXT.lock().init();

    println!("Nikka booted");

    halt();
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
