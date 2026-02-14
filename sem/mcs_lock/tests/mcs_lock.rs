#![feature(test)]
#![feature(cfg_select)]
#![feature(const_ops)]
#![feature(const_trait_impl)]
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
    sync::{
        Barrier,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    thread,
    time::{
        Duration,
        Instant,
    },
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

#[test]
fn fairness() {
    cfg_select! {
        miri => {
            const THREADS: usize = 4;
            const BASIC_BLOCK: Duration = Duration::from_micros(5);
            const THREAD_DISADVANTAGE: Duration = BASIC_BLOCK * 1500;
            const DURATION: Duration = THREAD_DISADVANTAGE * (THREADS - 1) as u32 * 600;
        }
        _ => {
            const THREADS: usize = 5;
            const THREAD_DISADVANTAGE: Duration = Duration::from_millis(2);
            const DURATION: Duration = THREAD_DISADVANTAGE * (THREADS - 1) as u32 * 600;
        }
    }
    let lock = McsLock::new(vec![]);
    let finish = AtomicBool::new(false);
    let barrier_start = Barrier::new(THREADS + 1);
    let barrier_end = Barrier::new(THREADS + 1);

    let (cnt_mismatch, iterations) = thread::scope(|s| {
        for t in 0 .. THREADS {
            let lock = &lock;
            let finish = &finish;
            let barrier_start = &barrier_start;
            let barrier_end = &barrier_end;

            s.spawn(move || {
                loop {
                    barrier_start.wait();
                    if finish.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(THREAD_DISADVANTAGE * t as u32);
                    let mut node = McsLockNode::new();
                    let mut guard = lock.lock(&mut node);
                    guard.push(t);
                    if t == 0 {
                        thread::sleep(THREAD_DISADVANTAGE * (THREADS - 1) as u32);
                    }
                    drop(guard);
                    barrier_end.wait();
                }
            });
        }

        let mut cnt_mismatch = 0;
        let mut iterations = 0;
        let start = Instant::now();
        while start.elapsed() < DURATION {
            barrier_start.wait();
            barrier_end.wait();

            let mut node = McsLockNode::new();
            let history = core::mem::take(&mut *lock.try_lock(&mut node).unwrap());
            let expected = (0 .. THREADS).collect::<Vec<_>>();
            if history != expected {
                println!("Unexpected order: {history:?}");
                cnt_mismatch += 1;
            }
            iterations += 1;
        }
        finish.store(true, Ordering::Relaxed);
        barrier_start.wait();

        (cnt_mismatch, iterations)
    });

    let ratio = cnt_mismatch as f64 / iterations as f64;
    println!(
        "cnt_mismatch: {}, iterations: {}, ratio: {:.3}",
        cnt_mismatch, iterations, ratio,
    );
    assert!(ratio < 0.02, "ratio: {ratio}");
}

#[rstest]
#[timeout(Duration::from_millis(500))]
fn try_lock_basic() {
    let lock = McsLock::new(0);

    let mut node_primary = McsLockNode::new();
    let mut guard_primary = lock.lock(&mut node_primary);

    let mut node_secondary = McsLockNode::new();
    assert!(lock.try_lock(&mut node_secondary).is_err());

    *guard_primary += 1;
    drop(guard_primary);

    let mut guard_secondary = lock.try_lock(&mut node_secondary).unwrap();
    *guard_secondary += 1;
    drop(guard_secondary);

    assert_eq!(*lock.try_lock(&mut node_primary).unwrap(), 2);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn try_lock_busy_polling() {
    const TARGET: usize = if cfg!(miri) {
        300
    } else {
        30_000
    };

    let lock = McsLock::new(0usize);

    thread::scope(|s| {
        let lock = &lock;

        let handle_remainder_try_lock = |rem| {
            s.spawn(move || {
                let mut node = McsLockNode::new();
                loop {
                    if let Ok(mut guard) = lock.try_lock(&mut node) {
                        let value = *guard;
                        if value >= TARGET {
                            break;
                        }
                        if value % 3 == rem {
                            *guard += 1;
                        }
                    }
                    core::hint::spin_loop();
                }
            })
        };

        handle_remainder_try_lock(0);
        handle_remainder_try_lock(1);

        s.spawn(move || {
            let mut node = McsLockNode::new();
            loop {
                let mut guard = lock.lock(&mut node);
                let value = *guard;
                if value >= TARGET {
                    break;
                }
                if value % 3 == 2 {
                    *guard += 1;
                }
                drop(guard);
                core::hint::spin_loop();
            }
        });
    });

    assert_eq!(lock.section(|v| *v), TARGET);
}
