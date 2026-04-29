#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(generic_const_exprs)]

use std::{
    cmp,
    thread,
};

use affinity;
use rand::{
    Rng,
    SeedableRng,
    rngs::SmallRng,
};
use rstest::rstest;

use ku::sync::Spsc;

#[cfg(not(miri))]
use std::time::Duration;

mod log;

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn sequential() {
    let spsc = Spsc::<usize, CAPACITY>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();
    let mut producer_values = vec![0; CAPACITY];
    let mut consumer_values = vec![0; CAPACITY];

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut read_values = SmallRng::seed_from_u64(SEED);
    let mut write_values = read_values.clone();

    const ITERATION_COUNT: usize = if cfg!(miri) {
        1_000
    } else {
        10_000
    };

    for _iteration in 0 .. ITERATION_COUNT {
        if rng.gen_ratio(1, 2) {
            let mut tx = producer.tx();
            tx.update_capacity();
            let len = rng.gen_range(0 ..= tx.capacity());
            for value in &mut producer_values[.. len] {
                *value = write_values.r#gen();
            }
            assert_eq!(tx.enqueue_slice(&producer_values[.. len]), len);
            tx.commit();
        } else {
            let mut tx = consumer.tx();
            tx.update_len();
            let len = rng.gen_range(0 ..= tx.len());
            assert_eq!(tx.dequeue_slice(&mut consumer_values[.. len]), len);
            for value in &consumer_values[.. len] {
                assert_eq!(*value, read_values.r#gen());
            }
            tx.commit();
        }
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(60)))]
fn concurrent() {
    type T = u32;

    let spsc = Spsc::<T, CAPACITY>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();
    let mut producer_values = vec![0; CAPACITY];
    let mut consumer_values = vec![0; CAPACITY];

    let mut read_values = SmallRng::seed_from_u64(SEED);
    let mut write_values = read_values.clone();

    let core_count = affinity::get_core_num();

    thread::scope(|scope| {
        let producer_thread = thread::Builder::new()
            .name("producer".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[core_count / 2]).unwrap();

                let mut rng = SmallRng::seed_from_u64(SEED - 1);
                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    let mut tx = producer.tx();
                    tx.update_capacity();
                    if !tx.is_full() {
                        let len = rng.gen_range(0 ..= cmp::min(element_count, tx.capacity()));
                        for value in &mut producer_values[.. len] {
                            *value = write_values.r#gen();
                        }
                        assert_eq!(tx.enqueue_slice(&producer_values[.. len]), len);
                        tx.commit();
                        element_count -= len;
                    }
                }
            })
            .unwrap();

        let consumer_thread = thread::Builder::new()
            .name("consumer".to_string())
            .spawn_scoped(scope, move || {
                affinity::set_thread_affinity(&[0]).unwrap();

                let mut rng = SmallRng::seed_from_u64(SEED);
                let mut element_count = ELEMENT_COUNT;

                while element_count > 0 {
                    let mut tx = consumer.tx();
                    tx.update_len();
                    if !tx.is_empty() {
                        let len = rng.gen_range(0 ..= cmp::min(tx.len(), element_count));
                        assert_eq!(tx.dequeue_slice(&mut consumer_values[.. len]), len);
                        for value in &consumer_values[.. len] {
                            assert_eq!(*value, read_values.r#gen());
                        }
                        tx.commit();
                        element_count -= len;
                    }
                }
            })
            .unwrap();

        assert!(consumer_thread.join().is_ok());
        assert!(producer_thread.join().is_ok());
    });
}

#[ctor::ctor]
fn init() {
    log::init();
}

const CAPACITY: usize = if cfg!(miri) {
    1 << 4
} else {
    1 << 15
};

const ELEMENT_COUNT: usize = if cfg!(miri) {
    100
} else if cfg!(debug_assertions) {
    100_000_000
} else {
    1_000_000_000
};

const SEED: u64 = 314159265;
