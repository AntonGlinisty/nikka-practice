#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    vec::Vec,
};
use core::{
    mem,
    ptr::NonNull,
};

use ku::{
    log::{
        debug,
        error,
        info,
    },
    memory::{
        Page,
        Virt,
        size::MiB,
    },
};

use lib::{
    allocator,
    entry,
};

entry!(main);

fn main() {
    info!(test_case = "basic");
    memory_allocator_basic();

    info!(test_case = "grow_and_shrink");
    memory_allocator_grow_and_shrink();

    info!(test_case = "memory_allocator_stress");
    memory_allocator_stress(10_000);
}

fn generate_page_fault() -> ! {
    unsafe {
        NonNull::<u8>::dangling().as_ptr().read_volatile();
    }

    unreachable!();
}

macro_rules! my_assert {
    ($condition:expr $(,)?) => {{
        if !$condition {
            error!(condition = stringify!($condition), "assert failed");
            generate_page_fault();
        }
    }};
}

include!("../../../kernel/src/tests/include/memory_allocator.rs");
