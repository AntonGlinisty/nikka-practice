#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use lib::{
    entry,
    syscall,
};

entry!(main);

fn main() {
    syscall::sched_yield();
}
