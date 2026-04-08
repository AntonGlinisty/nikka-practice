#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]

use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
    thread,
};

use affinity;
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
    sync::{
        Spsc,
        spsc::{
            SpscConsumer,
            SpscProducer,
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
fn producer_and_consumer_are_unique() {
    let spsc = Spsc::<Usize, 1>::new();
    let (producer, consumer) = spsc.channel().unwrap();

    assert!(spsc.channel().is_none());
    assert!(spsc.producer().is_none());
    assert!(spsc.consumer().is_none());

    drop(producer);

    assert!(spsc.channel().is_none());

    let producer = spsc.producer().unwrap();

    assert!(spsc.channel().is_none());
    assert!(spsc.producer().is_none());
    assert!(spsc.consumer().is_none());

    drop(consumer);

    assert!(spsc.channel().is_none());

    let consumer = spsc.consumer().unwrap();

    assert!(spsc.channel().is_none());
    assert!(spsc.producer().is_none());
    assert!(spsc.consumer().is_none());

    drop(producer);
    drop(consumer);

    assert!(spsc.channel().is_some());
    assert!(spsc.producer().is_some());
    assert!(spsc.consumer().is_some());
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn singleton_queue() {
    let spsc = Spsc::<Usize, 1>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    validate_len(&producer, &consumer, 0);

    assert_eq!(consumer.dequeue(), Err(Empty));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 0);

    producer.enqueue(Usize::new(1)).unwrap();
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 1);

    assert_eq!(producer.enqueue(Usize::new(2)), Err(Full(Usize::new(2))));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 1);

    assert_eq!(consumer.dequeue(), Ok(Usize::new(1)));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 0);

    assert_eq!(consumer.dequeue(), Err(Empty));

    validate_len(&producer, &consumer, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn basic() {
    let spsc = Spsc::<Usize, 2>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    validate_len(&producer, &consumer, 0);

    assert_eq!(consumer.dequeue(), Err(Empty));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 0);

    producer.enqueue(Usize::new(1)).unwrap();
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 1);

    producer.enqueue(Usize::new(2)).unwrap();
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 2);

    assert_eq!(producer.enqueue(Usize::new(3)), Err(Full(Usize::new(3))));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 2);

    assert_eq!(consumer.dequeue(), Ok(Usize::new(1)));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 1);

    assert_eq!(consumer.dequeue(), Ok(Usize::new(2)));
    consumer.update_len();
    producer.update_capacity();

    validate_len(&producer, &consumer, 0);

    assert_eq!(consumer.dequeue(), Err(Empty));

    validate_len(&producer, &consumer, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn no_leaks() {
    enqueue_dequeue();
    assert_eq!(DropChecker::alive_count(), 0);

    enqueue_only();
    assert_eq!(DropChecker::alive_count(), 0);

    tx_commit();
    assert_eq!(DropChecker::alive_count(), 0);

    tx_abort();
    assert_eq!(DropChecker::alive_count(), 0);

    fn enqueue_dequeue() {
        let spsc = Spsc::<DropChecker, 4>::new();
        let (mut producer, mut consumer) = spsc.channel().unwrap();

        producer.enqueue(DropChecker::new("enqueue_dequeue")).unwrap();
        consumer.dequeue().unwrap();
    }

    fn enqueue_only() {
        let spsc = Spsc::<DropChecker, 4>::new();
        let (mut producer, _) = spsc.channel().unwrap();

        producer.enqueue(DropChecker::new("enqueue_only")).unwrap();
    }

    fn tx_commit() {
        let spsc = Spsc::<DropChecker, 4>::new();
        let (mut producer, _) = spsc.channel().unwrap();

        let mut tx = producer.tx();
        tx.enqueue(DropChecker::new("tx_commit")).unwrap();
        tx.commit();
    }

    fn tx_abort() {
        let spsc = Spsc::<DropChecker, 4>::new();
        let (mut producer, _) = spsc.channel().unwrap();

        let mut tx = producer.tx();
        tx.enqueue(DropChecker::new("tx_abort")).unwrap();
        tx.abort();
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn tx_operations() {
    let spsc = Spsc::<usize, 4>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    assert_eq!(consumer.dequeue(), Err(Empty));

    let mut tx = producer.tx();
    tx.enqueue(11).unwrap();
    tx.enqueue(13).unwrap();
    tx.abort();

    assert_eq!(consumer.dequeue(), Err(Empty));

    let mut tx = producer.tx();
    for i in 1 .. 4 {
        tx.enqueue(i).unwrap();
        assert_eq!(consumer.dequeue(), Err(Empty));
    }
    tx.commit();

    for round in (0 .. 3).rev() {
        let mut tx = consumer.tx();
        for i in 1 .. 4 {
            assert_eq!(tx.dequeue(), Ok(i));
        }

        if round != 0 {
            tx.abort();
        }
    }

    assert_eq!(consumer.dequeue(), Err(Empty));
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn sequential() {
    let spsc = Spsc::<Usize, 16>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut producer_values = SmallRng::seed_from_u64(SEED);
    let mut consumer_values = producer_values.clone();

    const ITERATION_COUNT: usize = if cfg!(miri) {
        1_000
    } else {
        10_000
    };

    for _iteration in 0 .. ITERATION_COUNT {
        if rng.gen_ratio(1, 2) {
            let mut tx = producer.tx();
            tx.update_capacity();
            for _ in 0 .. rng.gen_range(0 ..= tx.capacity()) {
                tx.enqueue(Usize::new(producer_values.r#gen())).unwrap();
            }
            tx.commit();
        } else {
            let mut tx = consumer.tx();
            tx.update_len();
            for _ in 0 .. rng.gen_range(0 ..= tx.len()) {
                assert_eq!(tx.dequeue(), Ok(Usize::new(consumer_values.r#gen())));
            }
            tx.commit();
        }
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn concurrent() {
    let spsc = Spsc::<u32, { 1 << 15 }>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    let mut producer_values = SmallRng::seed_from_u64(SEED);
    let mut consumer_values = producer_values.clone();

    let core_count = affinity::get_core_num();

    thread::scope(|scope| {
        let producer_thread = thread::Builder::new()
            .name("producer".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[core_count / 2]).unwrap();

                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    if producer.is_full() {
                        producer.update_capacity();
                    }
                    if !producer.is_full() {
                        producer.enqueue(producer_values.r#gen()).unwrap();
                        element_count -= 1;
                    }
                }
            })
            .unwrap();

        let consumer_thread = thread::Builder::new()
            .name("consumer".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[0]).unwrap();

                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    if consumer.is_empty() {
                        consumer.update_len();
                    }
                    if !consumer.is_empty() {
                        assert_eq!(consumer.dequeue(), Ok(consumer_values.r#gen()));
                        element_count -= 1;
                    }
                }
            })
            .unwrap();

        assert!(producer_thread.join().is_ok());
        assert!(consumer_thread.join().is_ok());
    });
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn concurrent_tx() {
    let spsc = Spsc::<u32, { 1 << 15 }>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();

    let mut producer_values = SmallRng::seed_from_u64(SEED);
    let mut consumer_values = producer_values.clone();

    let core_count = affinity::get_core_num();

    thread::scope(|scope| {
        let producer_thread = thread::Builder::new()
            .name("tx producer_thread".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[core_count / 2]).unwrap();

                let mut rng = SmallRng::seed_from_u64(SEED - 1);
                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    let mut tx = producer.tx();
                    tx.update_capacity();
                    if !tx.is_full() {
                        for _ in 0 .. rng.gen_range(0 ..= tx.capacity()) {
                            if element_count > 0 {
                                tx.enqueue(producer_values.r#gen()).unwrap();
                                element_count -= 1;
                            } else {
                                break;
                            }
                        }
                        tx.commit();
                    }
                }
            })
            .unwrap();

        let consumer_thread = thread::Builder::new()
            .name("tx consumer".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[0]).unwrap();

                let mut rng = SmallRng::seed_from_u64(SEED);
                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    let mut tx = consumer.tx();
                    if !tx.is_empty() {
                        for _ in 0 .. rng.gen_range(0 ..= tx.len()) {
                            if element_count > 0 {
                                assert_eq!(tx.dequeue(), Ok(consumer_values.r#gen()));
                                element_count -= 1;
                            } else {
                                break;
                            }
                        }
                        tx.commit();
                    }
                }
            })
            .unwrap();

        assert!(producer_thread.join().is_ok());
        assert!(consumer_thread.join().is_ok());
    });
}

#[cfg(not(miri))]
#[dtor::dtor]
fn test_leaks() {
    assert_eq!(LIVE_USIZE.load(Ordering::Relaxed), 0);
}

#[derive(Clone, Debug, PartialEq)]
struct Usize(usize);

impl Usize {
    fn new(value: usize) -> Self {
        LIVE_USIZE.fetch_add(1, Ordering::Relaxed);

        Self(value)
    }
}

impl Drop for Usize {
    fn drop(&mut self) {
        LIVE_USIZE.fetch_sub(1, Ordering::Relaxed);
    }
}

fn validate_len<T, const CAPACITY: usize>(
    producer: &SpscProducer<'_, T, CAPACITY>,
    consumer: &SpscConsumer<'_, T, CAPACITY>,
    len: usize,
) {
    assert_eq!(consumer.len(), len);
    assert_eq!(producer.len(), len);
    assert_eq!(consumer.is_empty(), len == 0);
    assert_eq!(consumer.capacity(), CAPACITY - len);
    assert_eq!(producer.capacity(), CAPACITY - len);
    assert_eq!(producer.is_full(), len == CAPACITY);
}

#[ctor::ctor]
fn init() {
    log::init();
}

const ELEMENT_COUNT: usize = if cfg!(miri) {
    100
} else if cfg!(debug_assertions) {
    100_000_000
} else {
    1_000_000_000
};

static LIVE_USIZE: AtomicUsize = AtomicUsize::new(0);

const SEED: u64 = 314159265;
