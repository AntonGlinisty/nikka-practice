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

use crate::error::{
    Error::{
        InvalidAlignment,
        InvalidArgument,
        Null,
        Overflow,
    },
    Result,
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

    pub fn try_into_u32(self) -> Result<u32> {
        size::try_into_u32(self.0)
    }
}

impl<T: Tag> Add<usize> for Addr<T> {
    type Output = Result<Self>;

    fn add(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_add(rhs).ok_or(Overflow).and_then(Self::new).and_then(|addr| {
            if T::is_same_half(self, addr) {
                Ok(addr)
            } else {
                Err(Overflow)
            }
        })
    }
}

impl<T: Tag> Sub<usize> for Addr<T> {
    type Output = Result<Self>;

    fn sub(
        self,
        rhs: usize,
    ) -> Self::Output {
        self.0.checked_sub(rhs).ok_or(Overflow).and_then(Self::new).and_then(|addr| {
            if T::is_same_half(self, addr) {
                Ok(addr)
            } else {
                Err(Overflow)
            }
        })
    }
}

impl<T: Tag> AddAssign<usize> for Addr<T> {
    fn add_assign(
        &mut self,
        rhs: usize,
    ) {
        *self = self.add(rhs).expect("invalid arguments for Addr::add_assign()");
    }
}

impl<T: Tag> SubAssign<usize> for Addr<T> {
    fn sub_assign(
        &mut self,
        rhs: usize,
    ) {
        *self = self.sub(rhs).expect("invalid arguments for Addr::sub_assign()");
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

    fn is_same_half(
        x: Addr<Self>,
        y: Addr<Self>,
    ) -> bool;

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

    fn is_same_half(
        _x: Phys,
        _y: Phys,
    ) -> bool {
        true
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

    fn is_same_half(
        x: Virt,
        y: Virt,
    ) -> bool {
        x.is_higher_half() == y.is_higher_half()
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
            Err(InvalidArgument)
        }
    }

    pub(super) const BITS: u32 = 52;

    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl Virt {
    fn new_impl(addr: usize) -> Result<Self> {
        if addr.leading_zeros() > Self::UNUSED_BITS || addr.leading_ones() > Self::UNUSED_BITS {
            Ok(Self(addr, PhantomData))
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn is_higher_half(&self) -> bool {
        self.into_usize() >> Self::BITS != 0
    }

    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        const_assert_eq!(mem::size_of::<*const ()>(), mem::size_of::<usize>());
        Self::new(ptr as *const () as usize).expect("bad pointer")
    }

    pub fn from_ref<T: ?Sized>(x: &T) -> Self {
        Self::from_ptr(x as *const T)
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
            Err(InvalidAlignment)
        }
    }

    pub fn try_into_mut_ptr<T>(self) -> Result<*mut T> {
        const_assert_eq!(mem::size_of::<*mut ()>(), mem::size_of::<usize>());
        if self.0 % mem::align_of::<T>() == 0 {
            Ok(self.0 as *mut T)
        } else {
            Err(InvalidAlignment)
        }
    }

    pub unsafe fn try_into_ref<'a, T>(self) -> Result<&'a T> {
        let ptr = self.try_into_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
        } else {
            Ok(&*ptr)
        }
    }

    pub unsafe fn try_into_mut<'a, T>(self) -> Result<&'a mut T> {
        let ptr = self.try_into_mut_ptr::<T>()?;
        if ptr.is_null() {
            Err(Null)
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
            Err(Null)
        } else {
            Ok(slice::from_raw_parts_mut(ptr, len))
        }
    }

    const BITS: u32 = PAGE_TABLE_INDEX_BITS * (PAGE_TABLE_ROOT_LEVEL + 1) + PAGE_OFFSET_BITS;
    const UNUSED_BITS: u32 = usize::BITS - Self::BITS;
}

impl From<VirtAddr> for Virt {
    fn from(addr: VirtAddr) -> Self {
        Self::new_u64(addr.as_u64()).expect("bad VirtAddr")
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

#[cfg(test)]
mod test {
    use super::{
        Tag,
        Virt,
        VirtTag,
    };

    #[test]
    fn two_separate_halves_of_virt() {
        let lower_half_first = Virt::new(0).unwrap();
        let lower_half_last = Virt::new(0x0000_7FFF_FFFF_FFFF).unwrap();
        let higher_half_first = Virt::new(0xFFFF_8000_0000_0000).unwrap();
        let higher_half_last = Virt::new(0xFFFF_FFFF_FFFF_FFFF).unwrap();

        assert!(VirtTag::is_same_half(lower_half_first, lower_half_last));
        assert!(VirtTag::is_same_half(higher_half_first, higher_half_last));
        assert!(!VirtTag::is_same_half(lower_half_first, higher_half_first));
        assert!(!VirtTag::is_same_half(lower_half_first, higher_half_last));
        assert!(!VirtTag::is_same_half(lower_half_last, higher_half_first));
        assert!(!VirtTag::is_same_half(lower_half_last, higher_half_last));

        assert!(
            (lower_half_last + (higher_half_first.into_usize() - lower_half_last.into_usize()))
                .is_err()
        );

        for shift in [0, 1, 2, 10, 20, 30, 40, 50, 63] {
            let mut delta = (1usize << shift).wrapping_sub(0x10);
            for _ in 0 ..= 0x20 {
                if delta != 0 {
                    assert!((lower_half_first - delta).is_err());
                    assert!((lower_half_last + delta).is_err());
                    assert!((higher_half_first - delta).is_err());
                    assert!((higher_half_last + delta).is_err());
                }
                delta = delta.wrapping_add(1);
            }
        }
    }
}
