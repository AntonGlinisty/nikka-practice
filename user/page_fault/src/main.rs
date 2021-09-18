#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use core::{
    panic::PanicInfo,
    ptr::NonNull,
};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        NonNull::<u8>::dangling().as_ptr().read_volatile();
    }

    unreachable!();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
