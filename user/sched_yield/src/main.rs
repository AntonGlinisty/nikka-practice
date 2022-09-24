#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use ku::{
    log::info,
    time,
};

use lib::{
    entry,
    syscall,
};

entry!(main);

fn main() {
    info!(now = %time::now());
    syscall::sched_yield();

    let message = "trigger the sched_yield() syscall by overflowing the log buffer";

    for message_number in 0 .. {
        info!(
            message_number,
            "{}",
            &message[.. (message_number % message.len()) + 1],
        );
    }
}
