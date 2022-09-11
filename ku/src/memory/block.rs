use core::{
    fmt,
    iter::IntoIterator,
    marker::PhantomData,
    mem,
    ops::Range,
    option::Option,
};

use crate::error::{
    Error::InvalidArgument,
    Result,
};

use super::{
    addr::{
        Addr,
        IsVirt,
        Tag,
        Virt,
    },
    frage::Frage,
    size,
    size::{
        Size,
        SizeOf,
    },
};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Block<T: Memory> {
    start: usize,
    end: usize,
    tag: PhantomData<T>,
}

impl<T: Memory> Block<T> {
    pub fn new(
        start: T,
        end: T,
    ) -> Result<Self> {
        Self::from_index(T::index(start), T::index(end))
    }

    pub fn from_index(
        start: usize,
        end: usize,
    ) -> Result<Self> {
        let start_address = T::address_by_index(start)?;
        let last_address = T::address_by_index(
            if start < end {
                end - 1
            } else {
                end
            },
        )?;

        if start <= end && T::is_same_half(start_address, last_address) {
            Ok(Self {
                start,
                end,
                tag: PhantomData,
            })
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn from_index_u64(
        start: u64,
        end: u64,
    ) -> Result<Self> {
        Self::from_index(size::from(start), size::from(end))
    }

    pub const fn count(&self) -> usize {
        self.end - self.start
    }

    pub const fn size(&self) -> usize {
        self.count() * T::SIZE_OF
    }

    pub fn start_address(&self) -> T::Address {
        T::address_by_index(self.start).unwrap()
    }

    pub fn end_address(&self) -> Result<T::Address> {
        T::address_by_index(self.end)
    }

    pub fn contains(
        &self,
        element: T,
    ) -> bool {
        self.contains_address(T::address(element))
    }

    pub fn contains_address(
        &self,
        addr: T::Address,
    ) -> bool {
        if addr < self.start_address() {
            false
        } else if let Ok(end_address) = self.end_address() {
            addr < end_address
        } else {
            true
        }
    }

    pub fn tail(
        &mut self,
        count: usize,
    ) -> Option<Self> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl<T: Tag> Block<Addr<T>> {
    pub fn enclosing(&self) -> Block<Frage<T>> {
        let start_frage = Frage::<T>::index_by_address(self.start_address());
        let end_frage = if self.start == self.end {
            start_frage
        } else {
            Frage::<T>::index_by_address((self.end_address().unwrap() - 1).unwrap()) + 1
        };

        Block::from_index(start_frage, end_frage).unwrap()
    }
}

impl<T: Memory> Block<T>
where
    T::Address: IsVirt,
{
    pub fn into_ptr_u8(self) -> *const u8 {
        self.start_address().into_ptr_u8()
    }

    pub fn into_mut_ptr_u8(self) -> *mut u8 {
        self.start_address().into_mut_ptr_u8()
    }

    pub fn try_into_ptr<Q>(self) -> Result<*const Q> {
        if self.size() == mem::size_of::<Q>() {
            self.start_address().try_into_ptr::<Q>()
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn try_into_mut_ptr<Q>(self) -> Result<*mut Q> {
        if self.size() == mem::size_of::<Q>() {
            self.start_address().try_into_mut_ptr::<Q>()
        } else {
            Err(InvalidArgument)
        }
    }

    pub unsafe fn try_into_ref<'a, Q>(self) -> Result<&'a Q> {
        if self.size() == mem::size_of::<Q>() {
            self.start_address().try_into_ref::<Q>()
        } else {
            Err(InvalidArgument)
        }
    }

    pub unsafe fn try_into_mut<'a, Q>(self) -> Result<&'a mut Q> {
        if self.size() == mem::size_of::<Q>() {
            self.start_address().try_into_mut::<Q>()
        } else {
            Err(InvalidArgument)
        }
    }

    pub unsafe fn try_into_slice<'a, Q>(self) -> Result<&'a [Q]> {
        if self.size() % mem::size_of::<Q>() == 0 {
            self.start_address().try_into_slice::<Q>(self.size() / mem::size_of::<Q>())
        } else {
            Err(InvalidArgument)
        }
    }

    pub unsafe fn try_into_mut_slice<'a, Q>(self) -> Result<&'a mut [Q]> {
        if self.size() % mem::size_of::<Q>() == 0 {
            self.start_address().try_into_mut_slice::<Q>(self.size() / mem::size_of::<Q>())
        } else {
            Err(InvalidArgument)
        }
    }
}

impl Block<Virt> {
    pub fn from_ptr<T>(x: *const T) -> Self {
        let start = Virt::from_ptr(x);
        let end = (start + mem::size_of::<T>()).unwrap();
        Self::new(start, end).unwrap()
    }

    pub fn from_ref<T>(x: &T) -> Self {
        Self::from_ptr(x)
    }

    pub fn from_slice<T>(x: &[T]) -> Self {
        let range = x.as_ptr_range();

        Self::new(Virt::from_ptr(range.start), Virt::from_ptr(range.end)).unwrap()
    }
}

impl<T: Tag> From<Block<Frage<T>>> for Block<Addr<T>> {
    fn from(block: Block<Frage<T>>) -> Self {
        Self::new(block.start_address(), block.end_address().unwrap()).unwrap()
    }
}

impl<T: Memory> fmt::Debug for Block<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "[{:#x}, {:#x}), size {}",
            self.start,
            self.end,
            Size::new::<T>(self.count()),
        )
    }
}

impl<T: Memory> fmt::Display for Block<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "[{}, {}), size {}",
            Size::new::<T>(self.start),
            Size::new::<T>(self.end),
            Size::new::<T>(self.count()),
        )
    }
}

impl<T: Tag> IntoIterator for Block<Frage<T>> {
    type Item = Frage<T>;
    type IntoIter = IntoIter<Frage<T>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.start .. self.end, PhantomData)
    }
}

/// A separate struct for iterating over a block since Block is Copy and copyable iterators are a source of bugs.
#[derive(Clone)]
pub struct IntoIter<T>(Range<usize>, PhantomData<T>);

impl<T: Tag> Iterator for IntoIter<Frage<T>> {
    type Item = Frage<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().and_then(|index| Frage::from_index(index).ok())
    }
}

pub trait Memory: Clone + Copy + SizeOf {
    type Address: Ord;

    fn address(element: Self) -> Self::Address;
    fn address_by_index(index: usize) -> Result<Self::Address>;
    fn index(element: Self) -> usize;
    fn is_same_half(
        x: Self::Address,
        y: Self::Address,
    ) -> bool;
}

impl<T: Tag> Memory for Addr<T> {
    type Address = Addr<T>;

    fn address(element: Self) -> Self::Address {
        element
    }

    fn address_by_index(index: usize) -> Result<Self::Address> {
        Self::new(index)
    }

    fn index(element: Self) -> usize {
        element.into_usize()
    }

    fn is_same_half(
        x: Self::Address,
        y: Self::Address,
    ) -> bool {
        T::is_same_half(x, y)
    }
}

impl<T: Tag> Memory for Frage<T> {
    type Address = Addr<T>;

    fn address(element: Self) -> Self::Address {
        element.address()
    }

    fn address_by_index(index: usize) -> Result<Self::Address> {
        Self::address_by_index(index)
    }

    fn index(element: Self) -> usize {
        element.index()
    }

    fn is_same_half(
        x: Self::Address,
        y: Self::Address,
    ) -> bool {
        T::is_same_half(x, y)
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{
            Page,
            Phys,
            Virt,
        },
        Block,
    };

    #[test]
    fn grow_forward() {
        assert!(Block::<Phys>::from_index(0, 1).is_ok());
        assert!(Block::<Phys>::from_index(1, 1).is_ok());
        assert!(Block::<Phys>::from_index(1, 0).is_err());
    }

    #[test]
    fn enforce_same_virt_half() {
        let lower_half_last = 0x0000_7FFF_FFFF_FFFF;
        assert!(Block::<Virt>::from_index(0, lower_half_last).is_ok());
        assert!(Block::<Virt>::from_index(0, lower_half_last + 1).is_ok());

        let inside_lower_half = Virt::new(lower_half_last / 2).unwrap();
        let inside_higher_half = Virt::new(0xFFFF_FFFF_0000_0000).unwrap();
        assert!(Block::new(inside_lower_half, inside_higher_half).is_err());

        let lower_half_last =
            Page::new(Virt::new(lower_half_last - (Page::SIZE - 1)).unwrap()).unwrap();
        let inside_lower_half = Page::new(
            Virt::new((lower_half_last.address().into_usize() + Page::SIZE) / 2).unwrap(),
        )
        .unwrap();
        let inside_higher_half = Page::new(inside_higher_half).unwrap();
        assert!(Block::<Page>::from_index(0, lower_half_last.index()).is_ok());
        assert!(Block::<Page>::from_index(0, lower_half_last.index() + 1).is_ok());
        assert!(Block::<Page>::from_index(0, lower_half_last.index() + 2).is_err());
        assert!(Block::new(inside_lower_half, inside_higher_half).is_err());
    }

    #[test]
    fn enclosing() {
        let lower_half_first = 0x0;
        let lower_half_last = 0x0000_7FFF_FFFF_FFFF;
        let higher_half_first = 0xFFFF_8000_0000_0000;
        let higher_half_last = 0xFFFF_FFFF_FFFF_FFFF;
        let inside_lower_half = (lower_half_last / (2 * Page::SIZE)) * Page::SIZE;
        let inside_higher_half = 0xFFFF_FFFF_0000_0000;

        for base in [
            lower_half_first,
            higher_half_first,
            inside_lower_half,
            inside_higher_half,
        ] {
            for shift in [0, 1] {
                let start_virt = base + shift;
                let start_page = start_virt / Page::SIZE;
                for (end_virt, end_page) in [
                    (start_virt, start_page),
                    (start_virt + 1, start_page + 1),
                    (start_virt + (Page::SIZE - shift) - 1, start_page + 1),
                    (start_virt + (Page::SIZE - shift), start_page + 1),
                    (start_virt + (Page::SIZE - shift) + 1, start_page + 2),
                ] {
                    assert!(
                        Block::<Virt>::from_index(start_virt, end_virt).unwrap().enclosing() ==
                            Block::<Page>::from_index(start_page, end_page).unwrap(),
                    );
                }
            }
        }

        for base in [
            lower_half_first + 2 * Page::SIZE - 1,
            lower_half_last,
            higher_half_first + 2 * Page::SIZE - 1,
            higher_half_last,
        ] {
            for shift in [0, 1] {
                let end_virt = base - shift;
                let end_page = end_virt / Page::SIZE + 1;
                for (start_virt, start_page) in [
                    (end_virt - 1, end_page - 1),
                    (end_virt - (Page::SIZE - shift) + 1, end_page - 1),
                    (end_virt - (Page::SIZE - shift), end_page - 2),
                    (end_virt - (Page::SIZE - shift) - 1, end_page - 2),
                ] {
                    assert!(
                        Block::<Virt>::from_index(start_virt, end_virt).unwrap().enclosing() ==
                            Block::<Page>::from_index(start_page, end_page).unwrap(),
                    );
                }
                assert!(
                    Block::<Virt>::from_index(end_virt, end_virt).unwrap().enclosing() ==
                        Block::<Page>::from_index(end_page - 1, end_page - 1).unwrap(),
                );
            }
        }
    }

    #[test]
    fn bad_address() {
        let phys_end = 1 << 52;
        assert!(Block::<Phys>::from_index(phys_end - 1, phys_end).is_ok());
        assert!(Block::<Phys>::from_index(phys_end, phys_end).is_err());
        assert!(Block::<Phys>::from_index(phys_end - 1, phys_end + 1).is_err());

        let bad_virt = 1 << 49;
        assert!(Block::<Virt>::from_index(bad_virt, bad_virt).is_err());
    }
}
