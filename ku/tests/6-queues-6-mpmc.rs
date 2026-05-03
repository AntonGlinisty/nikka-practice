#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(never_type)]

use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
    thread,
    vec::Vec,
};

use rand::{
    Rng,
    SeedableRng,
    rngs::SmallRng,
};
use rstest::rstest;

use ku::{
    collections::bounded_deque::Error::{
        Empty,
        Full,
    },
    log::debug,
    memory::size,
    sync::{
        Mpmc,
        test_scaffolding,
    },
};

#[cfg(not(miri))]
use std::time::Duration;

mod log;

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn singleton_queue() {
    let mpmc = Mpmc::<U32, 1>::new();
    validate_len(&mpmc, 0);
    assert_eq!(mpmc.dequeue(), Err(Empty));
    validate_len(&mpmc, 0);
    mpmc.enqueue(U32::new(1)).unwrap();
    validate_len(&mpmc, 1);
    assert_eq!(mpmc.enqueue(U32::new(2)), Err(Full(U32::new(2))));
    validate_len(&mpmc, 1);
    assert_eq!(mpmc.dequeue(), Ok(U32::new(1)));
    validate_len(&mpmc, 0);
    assert_eq!(mpmc.dequeue(), Err(Empty));
    validate_len(&mpmc, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn basic() {
    let mpmc = Mpmc::<U32, 2>::new();
    validate_len(&mpmc, 0);
    assert_eq!(mpmc.dequeue(), Err(Empty));
    validate_len(&mpmc, 0);
    mpmc.enqueue(U32::new(1)).unwrap();
    validate_len(&mpmc, 1);
    mpmc.enqueue(U32::new(2)).unwrap();
    validate_len(&mpmc, 2);
    assert_eq!(mpmc.enqueue(U32::new(3)), Err(Full(U32::new(3))));
    validate_len(&mpmc, 2);
    assert_eq!(mpmc.dequeue(), Ok(U32::new(1)));
    validate_len(&mpmc, 1);
    assert_eq!(mpmc.dequeue(), Ok(U32::new(2)));
    validate_len(&mpmc, 0);
    assert_eq!(mpmc.dequeue(), Err(Empty));
    validate_len(&mpmc, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(100)))]
fn overflow() {
    overflow::<1>();
    overflow::<2>();
    overflow::<4>();
    overflow::<8>();
    overflow::<16>();

    fn overflow<const CAPACITY: usize>() {
        let mut oveflow_count = 0;

        for start in usize::MAX - CAPACITY - 2 ..= usize::MAX {
            for count in 0 ..= u32::try_from(CAPACITY).unwrap() {
                let mpmc = unsafe { test_scaffolding::new_mpmc::<U32, CAPACITY>(start) };

                for i in 0 .. count {
                    mpmc.enqueue(U32::new(i)).unwrap();
                    validate_len(&mpmc, size::from(i + 1));
                }

                if test_scaffolding::mpmc_head(&mpmc) > test_scaffolding::mpmc_tail(&mpmc) {
                    oveflow_count += 1;
                }

                validate_len(&mpmc, size::from(count));

                for i in 0 .. count {
                    assert_eq!(mpmc.dequeue(), Ok(U32::new(i)));
                    validate_len(&mpmc, size::from(count - i - 1));
                }

                validate_len(&mpmc, 0);
            }
        }

        assert!(oveflow_count > 0);
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn sequential() {
    let mpmc = Mpmc::<U32, 16>::new();

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut producer_values = SmallRng::seed_from_u64(SEED);
    let mut consumer_values = producer_values.clone();

    const ITERATION_COUNT: usize = if cfg!(miri) {
        100
    } else {
        10_000
    };

    for _ in 0 .. ITERATION_COUNT {
        if rng.gen_ratio(1, 2) {
            for _ in 0 .. rng.gen_range(0 ..= mpmc.capacity()) {
                mpmc.enqueue(U32::new(producer_values.r#gen())).unwrap();
            }
        } else {
            for _ in 0 .. rng.gen_range(0 ..= mpmc.len()) {
                assert_eq!(mpmc.dequeue(), Ok(U32::new(consumer_values.r#gen())));
            }
        }
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn concurrent() {
    let mpmc = Mpmc::<U32, { 1 << 15 }>::new();
    let thread_count = thread::available_parallelism().unwrap().get() / 2;

    const ELEMENT_COUNT: usize = if cfg!(miri) {
        100
    } else {
        10_000_000
    };

    thread::scope(|scope| {
        let producer_threads: Vec<_> = (0 .. thread_count)
            .map(|thread| {
                let mpmc = &mpmc;

                thread::Builder::new()
                    .name(format!("mpmc #{thread}"))
                    .spawn_scoped(scope, move || {
                        let mut element_count = ELEMENT_COUNT;
                        let mut producer_values =
                            SmallRng::seed_from_u64(SEED + size::into_u64(thread));
                        let mut producer_sum: u32 = 0;
                        let mut next_producer_value = producer_values.r#gen();

                        while element_count > 0 {
                            if mpmc.enqueue(U32::new(next_producer_value)).is_ok() {
                                producer_sum =
                                    producer_sum.wrapping_add(next_producer_value.into());
                                next_producer_value = producer_values.r#gen();
                                element_count -= 1;
                            }
                        }

                        debug!(producer_sum, "finished");

                        producer_sum
                    })
                    .unwrap()
            })
            .collect();

        let consumer_threads: Vec<_> = (0 .. thread_count)
            .map(|thread| {
                let mpmc = &mpmc;

                thread::Builder::new()
                    .name(format!("mpmc #{thread}"))
                    .spawn_scoped(scope, move || {
                        let mut element_count = ELEMENT_COUNT;
                        let mut consumer_sum: u32 = 0;

                        while element_count > 0 {
                            if let Ok(U32(consumer_value)) = mpmc.dequeue() {
                                consumer_sum = consumer_sum.wrapping_add(consumer_value);
                                element_count -= 1;
                            }
                        }

                        debug!(consumer_sum, "finished");

                        consumer_sum
                    })
                    .unwrap()
            })
            .collect();

        let producer_sum = producer_threads
            .into_iter()
            .map(|thread| thread.join().expect("producers should finish successfully"))
            .fold(0, |x: u32, y: u32| x.wrapping_add(y));

        let consumer_sum = consumer_threads
            .into_iter()
            .map(|thread| thread.join().expect("consumers should finish successfully"))
            .fold(0, |x: u32, y: u32| x.wrapping_add(y));

        debug!(producer_sum, consumer_sum);
        assert_eq!(consumer_sum, producer_sum);
    });
}

#[cfg(not(miri))]
#[dtor::dtor]
fn test_leaks() {
    assert_eq!(LIVE_U32.load(Ordering::Relaxed), 0);
}

#[derive(Clone, Debug, PartialEq)]
struct U32(u32);

impl U32 {
    fn new(value: u32) -> Self {
        LIVE_U32.fetch_add(1, Ordering::Relaxed);

        Self(value)
    }
}

impl Drop for U32 {
    fn drop(&mut self) {
        LIVE_U32.fetch_sub(1, Ordering::Relaxed);
    }
}

impl TryFrom<u32> for U32 {
    type Error = !;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self::new(value))
    }
}

impl From<U32> for u32 {
    fn from(value: U32) -> Self {
        value.0
    }
}

fn validate_len<T: Into<u32> + TryFrom<u32>, const CAPACITY: usize>(
    mpmc: &Mpmc<T, CAPACITY>,
    len: usize,
) {
    assert_eq!(mpmc.len(), len);
    assert_eq!(mpmc.is_empty(), len == 0);
    assert_eq!(mpmc.capacity(), CAPACITY - len);
    assert_eq!(mpmc.is_full(), len == CAPACITY);
}

#[ctor::ctor]
fn init() {
    log::init();
}

static LIVE_U32: AtomicUsize = AtomicUsize::new(0);

const SEED: u64 = 314159265;
