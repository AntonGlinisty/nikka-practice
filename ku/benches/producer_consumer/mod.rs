use std::{
    any,
    cell::UnsafeCell,
    cmp,
    hint,
    time::{
        Duration,
        Instant,
    },
};

use criterion::{
    BenchmarkGroup,
    BenchmarkId,
    measurement::WallTime,
};

use ku::collections::BoundedDeque;

use queue::{
    QueueWithFullFlag,
    QueueWithLen,
    QueueWithSentinel,
};

#[path = "../queue/mod.rs"]
mod queue;

pub(super) fn bounded_deque_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let deque = UnsafeCell::new(BoundedDeque::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).enqueue(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(|| unsafe { (&mut *consumer.get()).dequeue() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn full_flag_sequential<T, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Default,
    [(); N + 1]:,
{
    let deque = UnsafeCell::new(QueueWithFullFlag::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).enqueue(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(|| unsafe { (&mut *consumer.get()).dequeue() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn len_sequential<T, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Default,
    [(); N + 1]:,
{
    let deque = UnsafeCell::new(QueueWithLen::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).enqueue(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(|| unsafe { (&mut *consumer.get()).dequeue() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn sentinel_sequential<T, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Default,
    [(); N + 1]:,
{
    let deque = UnsafeCell::new(QueueWithSentinel::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).enqueue(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(|| unsafe { (&mut *consumer.get()).dequeue() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn reversed_stack_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let deque = UnsafeCell::new(BoundedDeque::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).push_front(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(
                || unsafe { (&mut *consumer.get()).pop_front() },
                batch_count,
            )
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn stack_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let deque = UnsafeCell::new(BoundedDeque::<T, N>::new());
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single_result(
                |value| unsafe { (&mut *producer.get()).push_back(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_result(|| unsafe { (&mut *consumer.get()).pop_back() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

pub(super) fn sequential_batch_count<const N: usize, const ELEMENT_COUNT: usize>() -> usize {
    cmp::min(cmp::max(ELEMENT_COUNT / 16, 1), N)
}

pub(super) fn benchmark<T, F: FnMut(&str, u64) -> Duration + Clone>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    capacity: usize,
    element_count: usize,
    name: &str,
    producer_consumer: F,
) {
    let id = format!("{name}<{}, {capacity}>", any::type_name::<T>());
    group.bench_with_input(
        BenchmarkId::new(&id, element_count),
        &element_count,
        |b, _| b.iter_custom(|iteration_count| producer_consumer.clone()(&id, iteration_count)),
    );
}

#[allow(unused)]
#[inline(always)]
pub(super) fn producer_single<T: Default, F: FnMut(T)>(
    mut produce: F,
    mut element_count: usize,
) {
    while element_count > 0 {
        produce(hint::black_box(T::default()));
        element_count -= 1;
    }
}

#[allow(unused)]
#[inline(always)]
pub(super) fn consumer_single_option<T: Default, F: FnMut() -> Option<T>>(
    mut consume: F,
    mut element_count: usize,
) {
    while element_count > 0 {
        if hint::black_box(consume()).is_some() {
            element_count -= 1;
        }
    }
}

#[inline(always)]
pub(super) fn producer_single_result<T: Default, R, E, F: FnMut(T) -> Result<R, E>>(
    mut produce: F,
    mut element_count: usize,
) {
    while element_count > 0 {
        if produce(hint::black_box(T::default())).is_ok() {
            element_count -= 1;
        }
    }
}

#[inline(always)]
pub(super) fn consumer_single_result<T: Default, E, F: FnMut() -> Result<T, E>>(
    mut consume: F,
    mut element_count: usize,
) {
    while element_count > 0 {
        if hint::black_box(consume()).is_ok() {
            element_count -= 1;
        }
    }
}

pub(super) fn producer_consumer_sequential<P: FnMut(usize) -> (), C: FnMut(usize) -> ()>(
    _id: &str,
    iteration_count: u64,
    mut producer: P,
    mut consumer: C,
    element_count: usize,
    batch_count: usize,
) -> Duration {
    let timer = Instant::now();

    for _ in 0 .. iteration_count {
        let mut read_element_count = element_count;
        let mut write_element_count = element_count;
        let mut empty = true;

        while read_element_count > 0 {
            if empty {
                let count = cmp::min(batch_count, write_element_count);
                producer(count);
                write_element_count -= count;
                empty = false;
            } else {
                let count = cmp::min(batch_count, read_element_count);
                consumer(count);
                read_element_count -= count;
                empty = true;
            }
        }
    }

    timer.elapsed()
}
