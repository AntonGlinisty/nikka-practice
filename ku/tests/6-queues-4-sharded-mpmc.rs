#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]

use std::{
    cmp,
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
        ShardedMpmc,
        sharded_mpmc::{
            ShardedMpmcConsumer,
            ShardedMpmcProducer,
        },
    },
};

use drop_checker::DropChecker;

#[cfg(not(miri))]
use std::time::Duration;

mod drop_checker;
mod log;

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn producer_and_consumer_are_multiple() {
    let mpmc = ShardedMpmc::<U64, 1, 1>::new();
    let (producer_1, consumer_1) = mpmc.channel();
    let (producer_2, consumer_2) = mpmc.channel();
    let producer_3 = mpmc.producer();
    let consumer_3 = mpmc.consumer();
    drop(producer_1);
    drop(producer_2);
    drop(producer_3);
    drop(consumer_1);
    drop(consumer_2);
    drop(consumer_3);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn singleton_queue() {
    let mpmc = ShardedMpmc::<U64, 1, 1>::new();
    let (mut producer, mut consumer) = mpmc.channel();
    validate_len(&producer, &consumer, 0);
    assert_eq!(consumer.dequeue(), Err(Empty));
    validate_len(&producer, &consumer, 0);
    producer.enqueue(U64::new(1)).unwrap();
    validate_len(&producer, &consumer, 1);
    assert_eq!(producer.enqueue(U64::new(2)), Err(Full(U64::new(2))));
    validate_len(&producer, &consumer, 1);
    assert_eq!(consumer.dequeue(), Ok(U64::new(1)));
    validate_len(&producer, &consumer, 0);
    assert_eq!(consumer.dequeue(), Err(Empty));
    validate_len(&producer, &consumer, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn basic() {
    let mpmc = ShardedMpmc::<U64, 2, 1>::new();
    let (mut producer, mut consumer) = mpmc.channel();
    validate_len(&producer, &consumer, 0);
    assert_eq!(consumer.dequeue(), Err(Empty));
    validate_len(&producer, &consumer, 0);
    producer.enqueue(U64::new(1)).unwrap();
    validate_len(&producer, &consumer, 1);
    producer.enqueue(U64::new(2)).unwrap();
    validate_len(&producer, &consumer, 2);
    assert_eq!(producer.enqueue(U64::new(3)), Err(Full(U64::new(3))));
    validate_len(&producer, &consumer, 2);
    assert_eq!(consumer.dequeue(), Ok(U64::new(1)));
    validate_len(&producer, &consumer, 1);
    assert_eq!(consumer.dequeue(), Ok(U64::new(2)));
    validate_len(&producer, &consumer, 0);
    assert_eq!(consumer.dequeue(), Err(Empty));
    validate_len(&producer, &consumer, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn sequential() {
    let mpmc = ShardedMpmc::<U64, 16, 4>::new();
    let (mut producer, mut consumer) = mpmc.channel();

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut producer_values = SmallRng::seed_from_u64(SEED);

    let mut producer_element_count = if cfg!(miri) {
        100
    } else {
        1_000_000
    };
    let mut consumer_element_count = producer_element_count;
    let mut producer_sum: u64 = 0;
    let mut consumer_sum: u64 = 0;

    while producer_element_count > 0 || consumer_element_count > 0 {
        if rng.gen_ratio(1, 2) {
            let count = rng.gen_range(0 ..= cmp::min(producer.capacity(), producer_element_count));
            for _ in 0 .. count {
                let value = producer_values.r#gen();
                producer_sum = producer_sum.wrapping_add(value);
                producer.enqueue(U64::new(value)).unwrap();
            }
            producer_element_count -= count;
        } else {
            let count = rng.gen_range(0 ..= cmp::min(consumer.len(), consumer_element_count));
            for _ in 0 .. count {
                let value = consumer.dequeue().unwrap().0;
                consumer_sum = consumer_sum.wrapping_add(value);
            }
            consumer_element_count -= count;
        }
    }

    debug!(producer_sum, consumer_sum);
    assert_eq!(producer_sum, consumer_sum);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn concurrent() {
    let mpmc = ShardedMpmc::<U64, { 1 << 14 }, 16>::new();
    let (producer, consumer) = mpmc.channel();
    let thread_count = thread::available_parallelism().unwrap().get() / 2;

    const ELEMENT_COUNT: usize = if cfg!(miri) {
        100
    } else {
        10_000_000
    };

    thread::scope(|scope| {
        let producer_threads: Vec<_> = (0 .. thread_count)
            .map(|thread| {
                let mut producer = producer.clone();

                thread::Builder::new()
                    .name(format!("producer #{thread}"))
                    .spawn_scoped(scope, move || {
                        let mut element_count = ELEMENT_COUNT;
                        let mut producer_values =
                            SmallRng::seed_from_u64(SEED + size::into_u64(thread));
                        let mut producer_sum: u64 = 0;
                        let mut next_producer_value = producer_values.r#gen();

                        while element_count > 0 {
                            if producer.enqueue(U64::new(next_producer_value)).is_ok() {
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
                let mut consumer = consumer.clone();

                thread::Builder::new()
                    .name(format!("consumer #{thread}"))
                    .spawn_scoped(scope, move || {
                        let mut element_count = ELEMENT_COUNT;
                        let mut consumer_sum: u64 = 0;

                        while element_count > 0 {
                            if let Ok(U64(consumer_value)) = consumer.dequeue() {
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
            .fold(0, |x: u64, y: u64| x.wrapping_add(y));

        let consumer_sum = consumer_threads
            .into_iter()
            .map(|thread| thread.join().expect("consumers should finish successfully"))
            .fold(0, |x: u64, y: u64| x.wrapping_add(y));

        debug!(producer_sum, consumer_sum);
        assert_eq!(consumer_sum, producer_sum);
    });
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn no_leaks() {
    enqueue_dequeue();
    assert_eq!(DropChecker::alive_count(), 0);

    enqueue_only();
    assert_eq!(DropChecker::alive_count(), 0);

    fn enqueue_dequeue() {
        let mpmc = ShardedMpmc::<DropChecker, 4, 2>::new();
        let (mut producer, mut consumer) = mpmc.channel();

        producer.enqueue(DropChecker::new("enqueue_dequeue")).unwrap();
        consumer.dequeue().unwrap();
    }

    fn enqueue_only() {
        let mpmc = ShardedMpmc::<DropChecker, 4, 2>::new();
        let (mut producer, _) = mpmc.channel();

        producer.enqueue(DropChecker::new("enqueue_only")).unwrap();
    }
}

#[cfg(not(miri))]
#[dtor::dtor]
fn test_leaks() {
    assert_eq!(LIVE_U64.load(Ordering::Relaxed), 0);
}

#[derive(Clone, Debug, PartialEq)]
struct U64(u64);

impl U64 {
    fn new(value: u64) -> Self {
        LIVE_U64.fetch_add(1, Ordering::Relaxed);

        Self(value)
    }
}

impl Drop for U64 {
    fn drop(&mut self) {
        LIVE_U64.fetch_sub(1, Ordering::Relaxed);
    }
}

fn validate_len<T, const CAPACITY: usize, const SHARD_COUNT: usize>(
    producer: &ShardedMpmcProducer<'_, T, CAPACITY, SHARD_COUNT>,
    consumer: &ShardedMpmcConsumer<'_, T, CAPACITY, SHARD_COUNT>,
    len: usize,
) where
    [(); CAPACITY / SHARD_COUNT]:,
{
    assert_eq!(consumer.len(), len);
    assert_eq!(consumer.is_empty(), len == 0);
    assert_eq!(producer.capacity(), CAPACITY - len);
    assert_eq!(producer.is_full(), len == CAPACITY);
}

#[ctor::ctor]
fn init() {
    log::init();
}

static LIVE_U64: AtomicUsize = AtomicUsize::new(0);

const SEED: u64 = 314159265;
