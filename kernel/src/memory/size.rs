use core::mem;

use static_assertions::const_assert;

#[allow(non_upper_case_globals)]
pub const KiB: usize = 1 << 10;

#[allow(non_upper_case_globals)]
pub const MiB: usize = 1 << 20;

#[allow(non_upper_case_globals)]
pub const GiB: usize = 1 << 30;

pub fn from<T>(x: T) -> usize
where
    u64: From<T>,
{
    into_usize(u64::from(x))
}

pub const fn into_usize(x: u64) -> usize {
    const_assert!(mem::size_of::<u64>() == mem::size_of::<usize>());
    x as usize
}

pub const fn into_u64(x: usize) -> u64 {
    const_assert!(mem::size_of::<u64>() == mem::size_of::<usize>());
    x as u64
}
