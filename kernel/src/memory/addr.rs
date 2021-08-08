use core::{
    fmt,
    marker::PhantomData,
    mem,
};

use static_assertions::const_assert_eq;
use x86_64::addr::VirtAddr;

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::warn,
};

use super::{
    mmu::{
        PAGE_OFFSET_BITS,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_ROOT_LEVEL,
    },
    size,
};

#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Addr<T>(usize, PhantomData<T>);

const_assert_eq!(mem::size_of::<Addr<()>>(), mem::size_of::<usize>());
const_assert_eq!(mem::size_of::<Addr<()>>(), mem::size_of::<u64>());

impl<T: Tag> Addr<T> {
    pub fn new(addr: usize) -> Result<Self> {
        T::new(addr)
    }

    pub fn new_u64(addr: u64) -> Result<Self> {
        Self::new(size::from(addr))
    }

    pub fn zero() -> Self {
        Self(0, PhantomData)
    }

    pub fn into_usize(self) -> usize {
        self.0
    }

    pub fn into_u64(self) -> u64 {
        size::into_u64(self.0)
    }
}

impl<T: Tag> fmt::Debug for Addr<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}({}{:X})", T::ADDR_NAME, T::HEX_PREFIX, self.0)
    }
}

impl<T: Tag> fmt::Display for Addr<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{}{:X}", T::HEX_PREFIX, self.0)
    }
}

pub trait Tag: Clone + Copy + Eq + Ord {
    fn new(addr: usize) -> Result<Addr<Self>>;

    const ADDR_NAME: &'static str;
    const FRAGE_NAME: &'static str;
    const HEX_PREFIX: &'static str;
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysTag;

impl Tag for PhysTag {
    fn new(addr: usize) -> Result<Phys> {
        Phys::new_impl(addr)
    }

    const ADDR_NAME: &'static str = "Phys";
    const FRAGE_NAME: &'static str = "Frame";
    const HEX_PREFIX: &'static str = "0p";
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct VirtTag;

impl Tag for VirtTag {
    fn new(addr: usize) -> Result<Virt> {
        Virt::new_impl(addr)
    }

    const ADDR_NAME: &'static str = "Virt";
    const FRAGE_NAME: &'static str = "Page";
    const HEX_PREFIX: &'static str = "0v";
}

pub type Phys = Addr<PhysTag>;

pub type Virt = Addr<VirtTag>;

impl Phys {
    fn new_impl(addr: usize) -> Result<Self> {
        let zeros = addr.leading_zeros();
        if zeros >= Self::UNUSED_BITS {
            Ok(Self(addr, PhantomData))
        } else {
            warn!(
                addr,
                zeros,
                unused_bits = Self::UNUSED_BITS,
                "invalid address casted to Phys",
            );
            Err(InvalidArgument)
        }
    }

    pub(super) const BITS: u32 = 52;

    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl Virt {
    fn new_impl(addr: usize) -> Result<Self> {
        let zeros = addr.leading_zeros();
        if zeros > Self::UNUSED_BITS {
            Ok(Self(addr, PhantomData))
        } else if zeros == Self::UNUSED_BITS {
            Ok(Self(addr | Self::NEGATIVE_SIGN_EXTEND, PhantomData))
        } else if zeros == 0 && addr & Self::NEGATIVE_SIGN_EXTEND == Self::NEGATIVE_SIGN_EXTEND {
            Ok(Self(addr, PhantomData))
        } else {
            warn!(
                addr,
                zeros,
                unused_bits = Self::UNUSED_BITS,
                "invalid address casted to Virt",
            );
            Err(InvalidArgument)
        }
    }

    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        const_assert_eq!(mem::size_of::<*const ()>(), mem::size_of::<usize>());
        Self::new(ptr as *const () as usize).expect("bad pointer")
    }

    const BITS: u32 = PAGE_TABLE_INDEX_BITS * (PAGE_TABLE_ROOT_LEVEL + 1) + PAGE_OFFSET_BITS;
    const NEGATIVE_SIGN_EXTEND: usize = !0 << Self::BITS;
    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl From<Virt> for VirtAddr {
    fn from(addr: Virt) -> Self {
        VirtAddr::new(addr.into_u64())
    }
}
