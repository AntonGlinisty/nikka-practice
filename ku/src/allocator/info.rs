use core::{
    cmp,
    fmt,
    hint,
    mem,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use crate::memory::{
    Page,
    Size,
    Virt,
};

/// Статистика аллокатора общего назначения.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Info {
    allocated: Counter,
    allocations: Counter,
    pages: Counter,
    requested: Counter,
}

impl Info {
    pub fn allocated(&self) -> usize {
        self.allocated.balance()
    }

    pub fn allocations(&self) -> usize {
        self.allocations.positive()
    }

    pub fn deallocations(&self) -> usize {
        self.allocations.negative()
    }

    pub fn current_allocations(&self) -> usize {
        self.allocations.balance()
    }

    pub fn allocated_pages(&self) -> usize {
        self.pages.positive()
    }

    pub fn deallocated_pages(&self) -> usize {
        self.pages.negative()
    }

    pub fn current_pages(&self) -> usize {
        self.pages.balance()
    }

    pub fn requested(&self) -> usize {
        self.requested.balance()
    }

    pub fn fragmentation_loss(&self) -> usize {
        self.allocated() - self.requested()
    }

    pub fn fragmentation_loss_percentage(&self) -> f64 {
        self.fragmentation_loss() as f64 / cmp::max(self.allocated(), 1) as f64 * 100.0
    }

    fn is_valid(&self) -> bool {
        self.allocated.is_valid() &&
            self.allocations.is_valid() &&
            self.pages.is_valid() &&
            self.requested.is_valid() &&
            self.requested.balance() <= self.allocated.balance() &&
            self.requested.negative() <= self.allocated.negative() &&
            self.allocated.balance() <= self.pages.balance() * Page::SIZE
    }
}

/// Предназначена для конкурентного доступа к статистике аллокатора.
///
/// Реализует [неблокирующую синхронизацию](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
/// для согласованного доступа к полям [`Info`].
/// Использует упрощённый [sequence lock](https://en.wikipedia.org/wiki/Seqlock).
///
/// См. также:
/// - [Writing a seqlock in Rust.](https://pitdicker.github.io/Writing-a-seqlock-in-Rust/)
/// - [Can Seqlocks Get Along With Programming Language Memory Models?](https://www.hpl.hp.com/techreports/2012/HPL-2012-68.pdf)
/// - [Crate seqlock.](https://docs.rs/seqlock/0.1.2/seqlock/)
#[derive(Debug, Default)]
pub struct AtomicInfo {
    allocated: AtomicCounter,
    allocations: AtomicCounter,
    pages: AtomicCounter,
    requested: AtomicCounter,
    sequence: AtomicUsize,
}

impl AtomicInfo {
    pub const fn new() -> Self {
        Self {
            allocated: AtomicCounter::new(),
            allocations: AtomicCounter::new(),
            pages: AtomicCounter::new(),
            requested: AtomicCounter::new(),
            sequence: AtomicUsize::new(0),
        }
    }

    pub fn allocation(
        &self,
        requested: usize,
        allocated: usize,
        allocated_pages: usize,
    ) {
        self.sequence.fetch_add(1, Ordering::Acquire);
        self.allocated.add(allocated);
        self.allocations.add(1);
        if allocated_pages > 0 {
            self.pages.add(allocated_pages);
        }
        self.requested.add(requested);
        self.sequence.fetch_add(1, Ordering::Release);
    }

    pub fn deallocation(
        &self,
        requested: usize,
        allocated: usize,
        deallocated_pages: usize,
    ) {
        self.sequence.fetch_add(1, Ordering::Acquire);
        self.allocations.sub(1);
        self.requested.sub(requested);
        self.allocated.sub(allocated);
        if deallocated_pages > 0 {
            self.pages.sub(deallocated_pages);
        }
        self.sequence.fetch_add(1, Ordering::Release);
    }

    pub fn load(&self) -> Info {
        let mut allocated;
        let mut allocations;
        let mut pages;
        let mut requested;

        loop {
            let mut sequence = self.sequence.load(Ordering::Acquire);

            while sequence % 2 == 1 {
                hint::spin_loop();
                sequence = self.sequence.load(Ordering::Acquire);
            }

            allocated = self.allocated.load();
            allocations = self.allocations.load();
            pages = self.pages.load();
            requested = self.requested.load();

            // Use the 'read-dont-modify-write' trick to be able to `Release`
            // the read section of the sequence lock.
            // See <https://www.hpl.hp.com/techreports/2012/HPL-2012-68.pdf>.
            let curr_sequence = self.sequence.fetch_add(0, Ordering::Release);

            if curr_sequence == mem::replace(&mut sequence, curr_sequence) {
                break;
            }
        }

        let info = Info {
            allocated,
            allocations,
            pages,
            requested,
        };

        assert!(info.is_valid());

        info
    }
}

impl fmt::Display for Info {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ allocations: {}, requested: {}, allocated: {}, pages: {}, loss: {} = {:.3}% }}",
            self.allocations,
            SizeCounter(self.requested),
            SizeCounter(self.allocated),
            self.pages,
            Size::new::<Virt>(self.fragmentation_loss()),
            self.fragmentation_loss_percentage(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counter {
    negative: usize,
    positive: usize,
}

impl Counter {
    fn negative(&self) -> usize {
        self.negative
    }

    fn positive(&self) -> usize {
        self.positive
    }

    fn balance(&self) -> usize {
        self.positive.checked_sub(self.negative).expect("negative balance detected")
    }

    fn is_valid(&self) -> bool {
        self.negative <= self.positive
    }
}

impl fmt::Display for Counter {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{} - {} = ", self.positive(), self.negative())?;

        if self.is_valid() {
            write!(formatter, "{}", self.balance())
        } else {
            write!(formatter, "error")
        }
    }
}

#[derive(Debug, Default)]
struct AtomicCounter {
    negative: AtomicUsize,
    positive: AtomicUsize,
}

impl AtomicCounter {
    const fn new() -> Self {
        Self {
            positive: AtomicUsize::new(0),
            negative: AtomicUsize::new(0),
        }
    }

    fn load(&self) -> Counter {
        Counter {
            positive: self.positive(),
            negative: self.negative(),
        }
    }

    fn negative(&self) -> usize {
        self.negative.load(Ordering::Relaxed)
    }

    fn positive(&self) -> usize {
        self.positive.load(Ordering::Relaxed)
    }

    fn add(
        &self,
        value: usize,
    ) {
        self.positive.fetch_add(value, Ordering::Relaxed);
    }

    fn sub(
        &self,
        value: usize,
    ) {
        self.negative.fetch_add(value, Ordering::Relaxed);
    }
}

struct SizeCounter(Counter);

impl fmt::Display for SizeCounter {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{} - {} = ",
            Size::new::<Virt>(self.0.positive()),
            Size::new::<Virt>(self.0.negative()),
        )?;

        if self.0.is_valid() {
            write!(formatter, "{}", Size::new::<Virt>(self.0.balance()))
        } else {
            write!(formatter, "error")
        }
    }
}
