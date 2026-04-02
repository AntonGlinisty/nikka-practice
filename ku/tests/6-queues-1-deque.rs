#![deny(warnings)]
#![feature(adt_const_params)]

use std::collections::VecDeque;

use rand::{
    Rng,
    SeedableRng,
    rngs::SmallRng,
};
use rstest::rstest;

use ku::{
    collections::{
        BoundedDeque,
        bounded_deque::Error::{
            Empty,
            Full,
        },
        test_scaffolding,
    },
    memory::size,
};

use drop_checker::DropChecker;

#[cfg(not(miri))]
use std::time::Duration;

mod drop_checker;
mod log;

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn singleton_queue() {
    let mut deque = BoundedDeque::<usize, 1>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.dequeue(), Err(Empty));
    validate_len(&deque, 0);
    deque.enqueue(1).unwrap();
    validate_len(&deque, 1);
    assert_eq!(deque.enqueue(2), Err(Full(2)));
    validate_len(&deque, 1);
    assert_eq!(deque.dequeue(), Ok(1));
    validate_len(&deque, 0);
    assert_eq!(deque.dequeue(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn singleton_reverse_queue() {
    let mut deque = BoundedDeque::<usize, 1>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
    deque.push_front(1).unwrap();
    validate_len(&deque, 1);
    assert_eq!(deque.push_front(2), Err(Full(2)));
    validate_len(&deque, 1);
    assert_eq!(deque.pop_back(), Ok(1));
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn queue() {
    let mut deque = BoundedDeque::<usize, 2>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.dequeue(), Err(Empty));
    validate_len(&deque, 0);
    deque.enqueue(1).unwrap();
    validate_len(&deque, 1);
    deque.enqueue(2).unwrap();
    validate_len(&deque, 2);
    assert_eq!(deque.enqueue(3), Err(Full(3)));
    validate_len(&deque, 2);
    assert_eq!(deque.dequeue(), Ok(1));
    validate_len(&deque, 1);
    assert_eq!(deque.dequeue(), Ok(2));
    validate_len(&deque, 0);
    assert_eq!(deque.dequeue(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn reverse_queue() {
    let mut deque = BoundedDeque::<usize, 2>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
    deque.push_front(1).unwrap();
    validate_len(&deque, 1);
    deque.push_front(2).unwrap();
    validate_len(&deque, 2);
    assert_eq!(deque.push_front(3), Err(Full(3)));
    validate_len(&deque, 2);
    assert_eq!(deque.pop_back(), Ok(1));
    validate_len(&deque, 1);
    assert_eq!(deque.pop_back(), Ok(2));
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn stack() {
    let mut deque = BoundedDeque::<usize, 2>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
    deque.push_back(1).unwrap();
    validate_len(&deque, 1);
    deque.push_back(2).unwrap();
    validate_len(&deque, 2);
    assert_eq!(deque.push_back(3), Err(Full(3)));
    validate_len(&deque, 2);
    assert_eq!(deque.pop_back(), Ok(2));
    validate_len(&deque, 1);
    assert_eq!(deque.pop_back(), Ok(1));
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn reverse_stack() {
    let mut deque = BoundedDeque::<usize, 2>::new();
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
    deque.push_back(1).unwrap();
    validate_len(&deque, 1);
    deque.push_back(2).unwrap();
    validate_len(&deque, 2);
    assert_eq!(deque.push_back(3), Err(Full(3)));
    validate_len(&deque, 2);
    assert_eq!(deque.pop_back(), Ok(2));
    validate_len(&deque, 1);
    assert_eq!(deque.pop_back(), Ok(1));
    validate_len(&deque, 0);
    assert_eq!(deque.pop_back(), Err(Empty));
    validate_len(&deque, 0);
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn overflow() {
    overflow::<1>();
    overflow::<2>();
    overflow::<4>();
    overflow::<8>();
    overflow::<16>();

    fn overflow<const CAPACITY: usize>() {
        let mut oveflow_count = 0;

        for start in usize::MAX - CAPACITY - 2 ..= usize::MAX {
            for count in 0 ..= CAPACITY {
                let mut deque = BoundedDeque::<usize, CAPACITY>::new();
                test_scaffolding::set_head_tail(&mut deque, start);

                for i in 0 .. count {
                    deque.enqueue(i).unwrap();
                }

                if test_scaffolding::head(&deque) > test_scaffolding::tail(&deque) {
                    oveflow_count += 1;
                }

                assert_eq!(deque.capacity(), CAPACITY - count);
                assert_eq!(deque.len(), count);

                for i in 0 .. count {
                    assert_eq!(deque.dequeue(), Ok(i));
                }

                assert_eq!(deque.capacity(), CAPACITY);
                assert_eq!(deque.len(), 0);
            }
        }

        assert!(oveflow_count > 0);
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_secs(10)))]
fn random() {
    random::<1, 1_000>();
    random::<2, 1_000>();
    random::<4, 1_000>();
    random::<8, 1_000>();
    random::<16, 1_000>();

    if !cfg!(miri) {
        random::<{ 1 << 10 }, 10_000_000>();
        random::<{ 1 << 15 }, 10_000_000>();

        if !cfg!(debug_assertions) {
            random::<{ 1 << 10 }, 100_000_000>();
        }
    }

    fn random<const CAPACITY: usize, const ITERATION_COUNT: usize>() {
        let mut deque = BoundedDeque::<usize, CAPACITY>::new();
        let mut vecdeque = VecDeque::with_capacity(CAPACITY);

        const SEED: u64 = 314159265;
        let mut rng = SmallRng::seed_from_u64(SEED + size::into_u64(CAPACITY + ITERATION_COUNT));

        validate_len(&deque, vecdeque.len());

        for _ in 0 .. ITERATION_COUNT {
            match rng.gen_range(0 .. 4) {
                0 =>
                    if !deque.is_full() {
                        let value = rng.r#gen();
                        deque.push_front(value).unwrap();
                        vecdeque.push_front(value);
                    },

                1 =>
                    if !deque.is_full() {
                        let value = rng.r#gen();
                        deque.push_back(value).unwrap();
                        vecdeque.push_back(value);
                    },

                2 =>
                    if !deque.is_empty() {
                        assert_eq!(deque.pop_front().unwrap(), vecdeque.pop_front().unwrap());
                    },

                3 =>
                    if !deque.is_empty() {
                        assert_eq!(deque.pop_back().unwrap(), vecdeque.pop_back().unwrap());
                    },

                _ => {},
            }

            validate_len(&deque, vecdeque.len());
        }
    }
}

#[rstest]
#[cfg_attr(not(miri), timeout(Duration::from_millis(10)))]
fn no_leaks() {
    enqueue_dequeue();
    assert_eq!(DropChecker::alive_count(), 0);

    enqueue_only();
    assert_eq!(DropChecker::alive_count(), 0);

    fn enqueue_dequeue() {
        let mut deque = BoundedDeque::<DropChecker, 4>::new();

        deque.enqueue(DropChecker::new("enqueue_dequeue")).unwrap();
        deque.dequeue().unwrap();
    }

    fn enqueue_only() {
        let mut deque = BoundedDeque::<DropChecker, 4>::new();

        deque.enqueue(DropChecker::new("enqueue_only")).unwrap();
    }
}

fn validate_len<T, const CAPACITY: usize>(
    deque: &BoundedDeque<T, CAPACITY>,
    len: usize,
) {
    assert_eq!(deque.capacity(), CAPACITY - len);
    assert_eq!(deque.len(), len);
    assert_eq!(deque.is_empty(), len == 0);
    assert_eq!(deque.is_full(), len == CAPACITY);
}

#[ctor::ctor]
fn init() {
    log::init();
}
