use core::{
    fmt,
    mem,
};

use number_prefix::NumberPrefix;
use static_assertions::const_assert;

use crate::error::Result;

#[allow(non_upper_case_globals)]
pub const KiB: usize = 1 << 10;

#[allow(non_upper_case_globals)]
pub const MiB: usize = 1 << 20;

#[allow(non_upper_case_globals)]
pub const GiB: usize = 1 << 30;

#[allow(non_upper_case_globals)]
pub const TiB: usize = 1 << 40;

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

pub fn try_into_u32(x: usize) -> Result<u32> {
    Ok(into_u64(x).try_into()?)
}

pub trait SizeOf {
    const SIZE_OF: usize;

    fn count_up(size: usize) -> usize {
        (size + Self::SIZE_OF - 1) / Self::SIZE_OF
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Size(usize);

impl Size {
    pub const fn new<T: SizeOf>(count: usize) -> Self {
        Self(count * T::SIZE_OF)
    }

    pub const fn new_u64<T: SizeOf>(count: u64) -> Self {
        Self::new::<T>(into_usize(count))
    }

    pub const fn from_slice<T>(slice: &[T]) -> Self {
        Self(mem::size_of_val(slice))
    }

    pub const fn num_bytes(&self) -> usize {
        self.0
    }
}

impl fmt::Display for Size {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match NumberPrefix::binary(self.num_bytes() as f64) {
            NumberPrefix::Standalone(_) => {
                write!(formatter, "{} B", self.num_bytes())
            },
            NumberPrefix::Prefixed(prefix, value) => {
                write!(formatter, "{:.3} {}B", value, prefix.symbol())
            },
        }
    }
}
