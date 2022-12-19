#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![allow(clippy::missing_safety_doc)]
#![cfg_attr(test, no_main)]
#![deny(warnings)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(const_refs_to_cell)]
#![feature(custom_test_frameworks)]
#![feature(int_roundings)]
#![feature(maybe_uninit_slice)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(crate::test_runner)]

extern crate alloc;

/// Аллокаторы памяти общего назначения.
pub mod allocator;

pub mod error;

/// [Файловая система](https://en.wikipedia.org/wiki/File_system).
pub mod fs;

pub mod log;

/// Здесь находится часть работы с памятью, которая происходит только в ядре.
pub mod memory;

/// Здесь находится часть работы с процессами, которая происходит только в ядре.
pub mod process;

/// Поддержка симметричной многопроцессорности
/// ([Symmetric multiprocessing](https://en.wikipedia.org/wiki/Symmetric_multiprocessing), SMP).
pub mod smp;

/// Здесь находится часть работы со временем, которая происходит только в ядре.
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

bitflags! {
    pub struct Subsystems: usize {
        const PHYS_MEMORY = 1 << 0;
        const VIRT_MEMORY = 1 << 1;
        const MAIN_FRAME_ALLOCATOR = 1 << 2;

        const MEMORY =
            Self::PHYS_MEMORY.bits | Self::VIRT_MEMORY.bits | Self::MAIN_FRAME_ALLOCATOR.bits;

        const SYSCALL = 1 << 4;
        const PROCESS_TABLE = 1 << 5;
        const SCHEDULER = 1 << 6;

        const PROCESS = Self::SYSCALL.bits | Self::PROCESS_TABLE.bits | Self::SCHEDULER.bits;

        const LOCAL_APIC = 1 << 7;
        const CPUS = 1 << 8;
        const BOOT_APS = 1 << 9;

        const SMP = Self::LOCAL_APIC.bits | Self::CPUS.bits | Self::BOOT_APS.bits;
    }
}

#[cold]
#[inline(never)]
pub fn init_subsystems(
    boot_info: &'static BootInfo,
    subsystems: Subsystems,
) {
    set_system_info(&SYSTEM_INFO);

    text::TEXT.lock().init();
    log::init();
    time::init();

    info!(now = %time::now(), tsc = ?time::timer(), "Nikka booted");

    gdt::init();
    trap::init();

    if subsystems & Subsystems::MEMORY != Subsystems::empty() {
        memory::init(boot_info, subsystems);
    }

    if subsystems.contains(Subsystems::MEMORY) {
        if let Err(error) = EXCEPTION_STACKS.lock().make_guard_zones() {
            warn!(?error, "failed to make guard zones in exception stacks")
        }
    }

    if subsystems & Subsystems::SMP != Subsystems::empty() {
        smp::init(boot_info, subsystems);
    }

    if subsystems & Subsystems::PROCESS != Subsystems::empty() {
        process::init(subsystems);
    }
}

#[cold]
#[inline(never)]
pub fn init(boot_info: &'static BootInfo) {
    init_subsystems(boot_info, Subsystems::all());
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
#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    fail_test(panic_info)
}

static SYSTEM_INFO: SystemInfo = SystemInfo::new();
