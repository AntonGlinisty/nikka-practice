#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![allow(clippy::missing_safety_doc)]
#![cfg_attr(test, no_main)]
#![deny(warnings)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(const_refs_to_cell)]
#![feature(custom_test_frameworks)]
#![feature(derive_default_enum)]
#![feature(format_args_capture)]
#![feature(maybe_uninit_slice)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

extern crate alloc;

pub mod allocator;
pub mod error;
pub mod log;
pub mod memory;
pub mod process;
pub mod smp;
pub mod time;
pub mod trap;

use core::{
    any,
    fmt::Write,
    panic::PanicInfo,
};

use bitflags::bitflags;
use bootloader::BootInfo;
use x86::io;

use ku::{
    SystemInfo,
    set_system_info,
};
use text::println;

use log::{
    info,
    warn,
};
use memory::{
    EXCEPTION_STACKS,
    gdt,
};

#[allow(dead_code)]
const PASS: text::Color = text::Color::LIGHT_GREEN;

#[allow(dead_code)]
const FAIL: text::Color = text::Color::LIGHT_RED;

bitflags! {
    pub struct ExitCode: u8 {
        const SUCCESS = 1;
        const FAILURE = 2;
    }
}

#[cold]
#[inline(never)]
pub fn init(boot_info: &'static BootInfo) {
    set_system_info(&SYSTEM_INFO);

    text::TEXT.lock().init();
    log::init();
    time::init();

    info!(now = %time::now(), tsc = ?time::timer(), "Nikka booted");

    gdt::init();
    trap::init();

    memory::init(boot_info);

    if let Err(error) = EXCEPTION_STACKS.lock().make_guard_zones() {
        warn!(?error, "failed to make guard zones in exception stacks")
    }

    smp::init(boot_info);

    process::init();
}

#[cold]
#[inline(never)]
pub fn halt() -> ! {
    loop {
        x86_64::instructions::interrupts::without_interrupts(x86_64::instructions::hlt)
    }
}

pub fn exit_qemu(exit_code: ExitCode) -> ! {
    info!(?exit_code, "exit qemu");

    const EXIT_PORT: u16 = 0xF4;

    unsafe {
        io::outb(EXIT_PORT, exit_code.bits);
    }

    halt()
}

#[alloc_error_handler]
#[cold]
#[inline(never)]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("failed to allocate memory, layout = {:?}", layout)
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        println!("\n{:-<60}", any::type_name::<T>());
        self();
        println!(color(PASS), "{:-<51} [passed]", any::type_name::<T>());
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    exit_qemu(ExitCode::SUCCESS);
}

pub fn fail_test(panic_info: &PanicInfo) -> ! {
    println!(color(FAIL), "{}\n{:-<51} [failed]", panic_info, "");

    exit_qemu(ExitCode::FAILURE)
}

pub fn pass_test() -> ! {
    println!(color(PASS), "{:-<51} [passed]", "");

    exit_qemu(ExitCode::SUCCESS)
}

#[cfg(test)]
#[no_mangle]
extern "C" fn _start() -> ! {
    test_main();
    exit_qemu(ExitCode::FAILURE)
}

#[cfg(test)]
#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    fail_test(panic_info)
}

static SYSTEM_INFO: SystemInfo = SystemInfo::new();
