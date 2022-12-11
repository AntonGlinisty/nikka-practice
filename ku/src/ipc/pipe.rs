use core::{
    marker::PhantomData,
    mem,
    result,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use static_assertions::const_assert;

use crate::memory::{
    Block,
    Page,
    Virt,
};

type RingBufferRaw = [u8; RingBuffer::MAPPED_SIZE];

#[repr(C, align(4096))]
pub struct RingBufferMemory([u8; RingBuffer::MAPPED_SIZE]);

const_assert!(mem::align_of::<RingBufferMemory>() == Page::SIZE);
const_assert!(mem::size_of::<RingBufferMemory>() % Page::SIZE == 0);

/// Continuous Ring Buffer
pub struct RingBuffer {
    buf: Virt,
    head: AtomicUsize,
    tail: AtomicUsize,
    stats: [RingBufferStats; 2],
}

impl RingBuffer {
    pub fn new(buf: &'static mut RingBufferMemory) -> Self {
        let buf = &mut buf.0;

        Self::check_continuous_mapping(buf);

        Self {
            buf: Virt::from_ref(buf),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            stats: [RingBufferStats::new(), RingBufferStats::new()],
        }
    }

    pub fn zero() -> Self {
        Self {
            buf: Virt::default(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            stats: [RingBufferStats::new(), RingBufferStats::new()],
        }
    }

    pub fn block(&self) -> Block<Virt> {
        if self.buf == Virt::default() {
            Block::default()
        } else {
            Block::from_ref(unsafe { self.buf.try_into_ref::<RingBufferRaw>().unwrap() })
        }
    }

    pub fn read_tx(&mut self) -> RingBufferTx<'_, ReadTag> {
        self.tx()
    }

    pub fn write_tx(&mut self) -> RingBufferTx<'_, WriteTag> {
        self.tx()
    }

    pub fn read_stats(&self) -> &RingBufferStats {
        &self.stats[ReadTag::STATS_INDEX]
    }

    pub fn write_stats(&self) -> &RingBufferStats {
        &self.stats[WriteTag::STATS_INDEX]
    }

    fn buf(&mut self) -> &mut RingBufferRaw {
        if self.buf == Virt::default() {
            panic!("the RingBuffer is not properly initialized");
        } else {
            unsafe { self.buf.try_into_mut().unwrap() }
        }
    }

    fn tx<T: Tag>(&mut self) -> RingBufferTx<'_, T> {
        let (head, tail) = self.head_tail();

        self.stats[T::STATS_INDEX].txs.fetch_add(1, Ordering::Relaxed);

        RingBufferTx {
            ring_buffer: self,
            head,
            tail,
            bytes: 0,
            _tag: PhantomData,
        }
    }

    fn head_tail(&self) -> (usize, usize) {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        (head, tail)
    }

    fn check_continuous_mapping(buf: &mut RingBufferRaw) {
        let read_ptr = &buf[0] as *const u8;
        let write_ptr = &mut buf[Self::REAL_SIZE] as *mut u8;

        assert_ne!(read_ptr, write_ptr);

        unsafe {
            let new_value = !read_ptr.read_volatile();
            write_ptr.write_volatile(new_value);
            assert_eq!(
                read_ptr.read_volatile(),
                new_value,
                "RingBuffer is not mapped properly to be continuous",
            );
        }
    }

    pub const MAPPED_SIZE: usize = 2 * Self::REAL_SIZE;
    pub const REAL_SIZE: usize = 4 * Page::SIZE;
}

pub struct RingBufferTx<'a, T: Tag> {
    ring_buffer: &'a mut RingBuffer,
    head: usize,
    tail: usize,
    bytes: usize,
    _tag: PhantomData<T>,
}

impl RingBufferTx<'_, ReadTag> {
    pub fn read(&mut self) -> &[u8] {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl RingBufferTx<'_, WriteTag> {
    pub fn write(
        &mut self,
        data: &[u8],
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn commit(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn capacity(&self) -> usize {
        RingBuffer::REAL_SIZE - (self.tail - self.head)
    }
}

impl<T: Tag> Drop for RingBufferTx<'_, T> {
    fn drop(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }
}

pub trait Tag {
    const STATS_INDEX: usize;
}

pub type RingBufferReadTx<'a> = RingBufferTx<'a, ReadTag>;
pub type RingBufferWriteTx<'a> = RingBufferTx<'a, WriteTag>;

pub struct ReadTag;

impl Tag for ReadTag {
    const STATS_INDEX: usize = 0;
}

pub struct WriteTag;

impl Tag for WriteTag {
    const STATS_INDEX: usize = 1;
}

const_assert!(ReadTag::STATS_INDEX != WriteTag::STATS_INDEX);

#[derive(Debug)]
pub struct RingBufferStats {
    committed: AtomicUsize,
    commits: AtomicUsize,
    dropped: AtomicUsize,
    drops: AtomicUsize,
    errors: AtomicUsize,
    txs: AtomicUsize,
}

impl RingBufferStats {
    fn new() -> Self {
        Self {
            committed: AtomicUsize::new(0),
            commits: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
            drops: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            txs: AtomicUsize::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Overflow {
        capacity: usize,
        len: usize,
        exceeding_object_len: usize,
    },
}

pub type Result<T> = result::Result<T, Error>;
