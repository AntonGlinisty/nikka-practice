#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(asm)]
#![no_main]
#![no_std]

use core::panic::PanicInfo;

use ku::process::Syscall;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    exit(3141592653589793238);
}

fn exit(code: usize) -> ! {
    unsafe {
        asm!(
            "
            // Test that the kernel switches into its own stack.
            xor rsp, rsp

            syscall
            ",

            in("rdi") Syscall::EXIT.bits(),
            in("rsi") code,

            options(noreturn),
        );
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
