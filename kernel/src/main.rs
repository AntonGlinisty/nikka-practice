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
extern crate x86;
extern crate x86_64;

extern crate pic8259;
extern crate text;

mod memory;
mod trap;

use core::{
    fmt::Write,
    panic::PanicInfo,
};

use text::{
    Attribute,
    print,
    println,
};

use memory::gdt;
use trap::TRAP_STATS;

#[cold]
#[inline(never)]
#[no_mangle]
extern "C" fn _start() {
    text::TEXT.lock().init();

    println!("Nikka booted");

    gdt::init();
    trap::init();

    loop {
        print!("\rINT stats:");
        for stats in &TRAP_STATS {
            let count = stats.count();
            if count != 0 {
                print!(" {}:{}", stats.mnemonic(), count);
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
