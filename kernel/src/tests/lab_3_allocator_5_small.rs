extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    vec::Vec,
};
use core::mem;

use ku::memory::{
    Page,
    Virt,
    size::MiB,
};

use kernel::{
    Subsystems,
    allocator,
    log::debug,
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn basic() {
    memory_allocator_basic();
}

#[test_case]
fn grow_and_shrink() {
    memory_allocator_grow_and_shrink();
}

#[test_case]
fn stress() {
    let values = 100_000;
    let pages_for_values = memory_allocator_stress(values);
    debug!(values, pages_for_values);
    assert!(pages_for_values < 2 * values / mem::size_of::<usize>());
}

macro_rules! my_assert {
    ($cond:expr $(,)?) => {{
        assert!($cond);
    }};
}

include!("include/memory_allocator.rs");
