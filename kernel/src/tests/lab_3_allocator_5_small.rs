use alloc::{
    alloc::{
        Allocator,
        Global,
        Layout,
    },
    boxed::Box,
    collections::BTreeMap,
    vec::Vec,
};
use core::{
    cmp,
    mem,
};

use ku::{
    allocator::Info,
    memory::{
        Page,
        Virt,
        size::{
            MiB,
            Size,
        },
    },
};

use crate::{
    allocator,
    log::debug,
};

pub(super) fn basic() {
    memory_allocator_basic();
}

pub(super) fn alignment() {
    memory_allocator_alignment();
}

pub(super) fn grow_and_shrink() {
    memory_allocator_grow_and_shrink();
}

pub(super) fn stress() {
    let values = 100_000;
    let max_fragmentation_loss = |values| cmp::max(6 * values, 2 * MiB);
    let pages_for_values = memory_allocator_stress(values, max_fragmentation_loss);
    debug!(values, pages_for_values);
    assert!(pages_for_values < 2 * values / mem::size_of::<usize>());
}

macro_rules! my_assert {
    ($cond:expr $(,)?) => {{
        assert!($cond);
    }};
}

include!("include/memory_allocator.rs");
