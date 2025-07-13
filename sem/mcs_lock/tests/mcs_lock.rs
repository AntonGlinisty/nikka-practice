#![feature(test)]
#![deny(warnings)]

extern crate test;

use mcs_lock::{
    McsLock,
    McsLockNode,
};
use rand::{
    distr::{
        Bernoulli,
        Distribution,
    },
    seq::SliceRandom,
};
use rstest::*;
use std::{
    thread,
    time::Duration,
};

#[rstest]
#[timeout(Duration::from_millis(500))]
fn basic() {
    let lock = McsLock::new(0);

    let mut node = McsLockNode::new();
    lock.section(|v| *v += 1);
    *lock.lock(&mut node) += 1;
    lock.section(|v| *v += 1);
    *lock.lock(&mut McsLockNode::new()) += 1;
    assert_eq!(lock.section(|v| *v), 4);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn simple_multithreaded() {
    let max_threads = thread::available_parallelism().unwrap().get();
    let num_threads = if cfg!(miri) {
        3
    } else {
        6
    }
    .min(max_threads);
    const NUM_ITERATIONS: usize = if cfg!(miri) {
        100
    } else {
        100_000
    };

    let lock = McsLock::new(0);

    thread::scope(|s| {
        for _ in 0 .. num_threads {
            s.spawn(|| {
                let mut node = McsLockNode::new();
                for _ in 0 .. NUM_ITERATIONS {
                    *lock.lock(&mut node) += 1;
                }
            });
        }
    });

    assert_eq!(lock.section(|v| *v), num_threads * NUM_ITERATIONS);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn long_sections() {
    let max_threads = thread::available_parallelism().unwrap().get();
    let num_threads = 3.min(max_threads);
    const NUM_ITERATIONS: usize = if cfg!(miri) {
        100
    } else {
        1000
    };

    let lock = McsLock::new(0);

    thread::scope(|s| {
        for _ in 0 .. num_threads {
            s.spawn(|| {
                let mut node = McsLockNode::new();
                for _ in 0 .. NUM_ITERATIONS {
                    let mut guard = lock.lock(&mut node);
                    std::thread::yield_now();
                    *guard += 1;
                }
            });
        }
    });

    assert_eq!(lock.section(|v| *v), num_threads * NUM_ITERATIONS);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
fn multiple_locks() {
    let max_threads = thread::available_parallelism().unwrap().get();
    let num_threads = if cfg!(miri) {
        3
    } else {
        6
    }
    .min(max_threads);
    const NUM_ITERATIONS: usize = if cfg!(miri) {
        25
    } else {
        100_000
    };
    const LOCKS: usize = 5;

    let locks = (0 .. LOCKS).map(|_| McsLock::new(0)).collect::<Vec<_>>();

    let expected = thread::scope(|s| {
        (0 .. num_threads)
            .map(|_| s.spawn(|| multi_writer(&locks, NUM_ITERATIONS)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum::<usize>()
    });

    let real = locks.iter().map(|v| v.section(|v| *v)).sum::<usize>();
    assert_eq!(expected, real);
}

fn multi_writer(
    locks: &[McsLock<usize>],
    iters: usize,
) -> usize {
    let should_lock = Bernoulli::from_ratio(2, 3).unwrap();
    let should_shuffle_guards = Bernoulli::from_ratio(1, 3).unwrap();
    let should_shuffle_nodes = Bernoulli::from_ratio(1, 4).unwrap();

    let mut rng = rand::rng();
    let mut nodes: Vec<_> = (0 .. locks.len()).map(|_| McsLockNode::new()).collect();
    let mut node_ptrs = nodes.iter_mut().collect::<Vec<_>>();

    for _ in 0 .. iters {
        let mut guards = Vec::with_capacity(node_ptrs.len());
        for (node, lock) in node_ptrs.iter_mut().zip(locks) {
            if should_lock.sample(&mut rng) {
                guards.push(lock.lock(node));
            }
        }
        if should_shuffle_guards.sample(&mut rng) {
            guards.shuffle(&mut rng);
        }
        drop(guards);
        if should_shuffle_nodes.sample(&mut rng) {
            node_ptrs.shuffle(&mut rng);
        }
    }
    0
}
