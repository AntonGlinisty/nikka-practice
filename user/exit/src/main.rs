#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use core::arch::asm;

use ku::process::Syscall;

use lib::entry;

entry!(main);

fn main() {
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
