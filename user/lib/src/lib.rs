#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(asm)]
#![no_std]

pub mod syscall;

use core::panic::PanicInfo;

use ku::process::ExitCode;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    extern "Rust" {
        fn main();
    }

    unsafe {
        main();
    }

    syscall::exit(ExitCode::OK.bits());
}

#[cold]
#[inline(never)]
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    syscall::exit(ExitCode::PANIC.bits());
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn check_main_signature() {
            let main: fn() = $path;

            main()
        }
    };
}
