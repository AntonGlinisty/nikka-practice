use core::fmt::{
    Debug,
    Display,
    Formatter,
    Result,
};

use super::{
    addr::{
        Addr,
        PhysTag,
        Tag,
        VirtTag,
    },
    size::KiB,
};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Frage<T>(Addr<T>);

impl<T: Tag> Frage<T> {
    pub const SIZE: usize = 4 * KiB;

    pub fn new(addr: Addr<T>) -> Self {
        Self(addr)
    }

    pub fn zero() -> Self {
        Self(Addr::zero())
    }

    pub fn address(&self) -> Addr<T> {
        self.0
    }

    pub fn index(&self) -> usize {
        self.address().into_usize() / Self::SIZE
    }
}

impl<T: Tag> Debug for Frage<T> {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        write!(
            formatter,
            "{}({} @ {})",
            T::FRAGE_NAME,
            self.index(),
            self.0,
        )
    }
}

impl<T: Tag> Display for Frage<T> {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        write!(formatter, "{} @ {}", self.index(), self.0)
    }
}

pub type Frame = Frage<PhysTag>;

pub type Page = Frage<VirtTag>;
