use core::{
    fmt::{
        Debug,
        Display,
        Formatter,
        Result,
    },
    iter::IntoIterator,
    marker::PhantomData,
    ops::Range,
    option::Option,
};

use crate::log::debug;

use super::{
    addr::Tag,
    frage::Frage,
    size,
    size::{
        Size,
        SizeOf,
    },
};

#[derive(Clone, Copy)]
pub struct Block<T: SizeOf> {
    start: usize,
    end: usize,
    tag: PhantomData<T>,
}

impl<T: SizeOf> Block<T> {
    pub const fn new(
        start: usize,
        end: usize,
    ) -> Self {
        assert!(start <= end);
        Self {
            start,
            end,
            tag: PhantomData,
        }
    }

    pub const fn new_u64(
        start: u64,
        end: u64,
    ) -> Self {
        Self::new(size::into_usize(start), size::into_usize(end))
    }

    pub const fn zero() -> Self {
        Self {
            start: 0,
            end: 0,
            tag: PhantomData,
        }
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn end(&self) -> usize {
        self.end
    }

    pub const fn count(&self) -> usize {
        self.end - self.start
    }

    pub const fn size(&self) -> usize {
        self.count() * T::SIZE_OF
    }

    pub fn tail(
        &mut self,
        count: usize,
    ) -> Option<Self> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl<T: SizeOf> Debug for Block<T> {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        write!(
            formatter,
            "[{:#x}, {:#x}), size {}",
            self.start,
            self.end,
            Size::new::<T>(self.count()),
        )
    }
}

impl<T: SizeOf> Display for Block<T> {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
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
        self.0.next().map(Frage::from_index)
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::Frame,
        Block,
    };

    #[test_case]
    fn block() {
        for start in 0 .. 100 {
            for count in 0 .. 100 {
                let end = start + count;
                let mut a = Block::<Frame>::new(start, end);
                assert_eq!(a.count(), count);

                for requested_count in count + 1 .. count + 3 {
                    assert!(a.tail(requested_count).is_none());
                }

                let mut requested_count = 0;

                while requested_count < a.count() {
                    let old_count = a.count();
                    let old_end = a.end();

                    a.tail(0).unwrap();
                    assert_eq!(a.count(), old_count);

                    let b = a.tail(requested_count).unwrap();

                    assert_eq!(a.count(), old_count - requested_count);
                    assert_eq!(b.count(), requested_count);

                    assert_eq!(a.start(), start);
                    assert!(a.start() < a.end());
                    assert_eq!(a.start() + old_count - requested_count, a.end());
                    assert_eq!(a.end(), b.start());
                    assert!(b.start() <= b.end());
                    assert_eq!(b.start() + requested_count, b.end());
                    assert!(b.end() <= end);
                    assert_eq!(b.end(), old_end);

                    requested_count += 1;
                }

                if count > 0 {
                    let remaining_count = a.count();
                    assert!(remaining_count > 0);

                    let c = a.tail(remaining_count).unwrap();

                    assert_eq!(a.count(), 0);
                    assert_eq!(c.count(), remaining_count);

                    assert_eq!(a.start(), start);
                    assert_eq!(a.start(), a.end());
                    assert_eq!(a.end(), c.start());
                    assert!(c.start() < c.end());
                    assert_eq!(c.start() + remaining_count, c.end());
                    assert!(c.end() <= end);
                }

                a.tail(0).unwrap();
                assert_eq!(a.count(), 0);
                assert!(a.tail(1).is_none());
            }
        }
    }
}
