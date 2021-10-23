#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(asm)]
#![no_std]

pub mod syscall;

use core::panic::PanicInfo;

use tracing_core::{
    dispatch,
    dispatch::Dispatch,
};

use ku::{
    info,
    info::ProcessInfo,
    log::{
        LOG_COLLECTOR,
        error,
    },
    process::ExitCode,
};

#[no_mangle]
pub extern "C" fn _start(process_info: &'static mut ProcessInfo) -> ! {
    info::set_process_info(process_info);

    dispatch::set_global_default(Dispatch::from_static(&LOG_COLLECTOR)).unwrap();

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
fn panic(panic_info: &PanicInfo) -> ! {
    error!(message = %panic_info);
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
