use core::{
    fmt,
    marker::PhantomData,
    mem,
    ops::{
        Add,
        AddAssign,
        Sub,
        SubAssign,
    },
    slice,
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
    size::{
        self,
        SizeOf,
    },
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

    pub fn into_ptr_u8(self) -> *const u8 {
        const_assert_eq!(mem::align_of::<u8>(), 1);
        const_assert_eq!(mem::size_of::<*const u8>(), mem::size_of::<usize>());
        self.0 as *const u8
    }

    pub fn into_mut_ptr_u8(self) -> *mut u8 {
        const_assert_eq!(mem::align_of::<u8>(), 1);
        const_assert_eq!(mem::size_of::<*mut u8>(), mem::size_of::<usize>());
        self.0 as *mut u8
    }

    pub fn try_into_ptr<T>(self) -> Result<*const T> {
        const_assert_eq!(mem::size_of::<*const ()>(), mem::size_of::<usize>());
        if self.0 % mem::align_of::<T>() == 0 {
            Ok(self.0 as *const T)
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn try_into_mut_ptr<T>(self) -> Result<*mut T> {
        const_assert_eq!(mem::size_of::<*mut ()>(), mem::size_of::<usize>());
        if self.0 % mem::align_of::<T>() == 0 {
            Ok(self.0 as *mut T)
        } else {
            Err(InvalidArgument)
        }
    }

    pub unsafe fn try_into_ref<'a, T>(self) -> Result<&'a T> {
        let ptr = self.try_into_ptr::<T>()?;
        if ptr.is_null() {
            Err(InvalidArgument)
        } else {
            Ok(&*ptr)
        }
    }

    pub unsafe fn try_into_mut<'a, T>(self) -> Result<&'a mut T> {
        let ptr = self.try_into_mut_ptr::<T>()?;
        if ptr.is_null() {
            Err(InvalidArgument)
        } else {
            Ok(&mut *ptr)
        }
    }

    pub unsafe fn try_into_mut_slice<'a, T>(
        self,
        len: usize,
    ) -> Result<&'a mut [T]> {
        let ptr = self.try_into_mut_ptr::<T>()?;
        if ptr.is_null() {
            Err(InvalidArgument)
        } else {
            Ok(slice::from_raw_parts_mut(ptr, len))
        }
    }

    const BITS: u32 = PAGE_TABLE_INDEX_BITS * (PAGE_TABLE_ROOT_LEVEL + 1) + PAGE_OFFSET_BITS;
    const NEGATIVE_SIGN_EXTEND: usize = !0 << Self::BITS;
    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl Add<usize> for Virt {
    type Output = Result<Self>;

    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_add(rhs).ok_or(InvalidArgument).and_then(Self::new)
    }
}

impl AddAssign<usize> for Virt {
    fn add_assign(
        &mut self,
        rhs: usize,
    ) {
        *self = self.add(rhs).expect("invalid arguments for Virt::add_assign()");
    }
}

impl Sub<usize> for Virt {
    type Output = Result<Self>;

    fn sub(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_sub(rhs).ok_or(InvalidArgument).and_then(Self::new)
    }
}

impl SubAssign<usize> for Virt {
    fn sub_assign(
        &mut self,
        rhs: usize,
    ) {
        *self = self.sub(rhs).expect("invalid arguments for Virt::sub_assign()");
    }
}

impl From<Virt> for VirtAddr {
    fn from(addr: Virt) -> Self {
        VirtAddr::new(addr.into_u64())
    }
}

impl<T: Tag> SizeOf for Addr<T> {
    const SIZE_OF: usize = 1;
}
