#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(likely_unlikely)]
#![feature(stmt_expr_attributes)]

use std::fmt;

use criterion::{
    AxisScale,
    BenchmarkGroup,
    Criterion,
    PlotConfiguration,
    Throughput,
    criterion_group,
    criterion_main,
    measurement::WallTime,
};

use ku::memory::size;

use producer_consumer::{
    benchmark,
    bounded_deque_sequential,
    full_flag_sequential,
    len_sequential,
    reversed_stack_sequential,
    sentinel_sequential,
    stack_sequential,
};

#[cfg(feature = "benchmark")]
use {
    heapless,
    producer_consumer::{
        consumer_single_option,
        producer_consumer_sequential,
        producer_single,
        producer_single_result,
        sequential_batch_count,
    },
    std::{
        cell::UnsafeCell,
        collections::VecDeque,
        time::Duration,
        vec::Vec,
    },
};

#[path = "../tests/log/mod.rs"]
mod log;

mod producer_consumer;

criterion_main!(benches);

criterion_group!(benches, invariants<u32>, cache_effects<u32>);

fn invariants<T: fmt::Debug + Default>(c: &mut Criterion) {
    let mut group = c.benchmark_group("invariants");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    #[cfg(feature = "benchmark")]
    {
        compare::<T, 8, 1>(&mut group);
        compare::<T, 8, 2>(&mut group);
        compare::<T, 8, 5>(&mut group);
        compare::<T, 8, 10>(&mut group);
    }

    compare::<T, 8, 100>(&mut group);
    compare::<T, 8, 1_000>(&mut group);
    compare::<T, 8, 10_000>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize, const ELEMENT_COUNT: usize>(
        group: &mut BenchmarkGroup<'_, WallTime>
    ) where
        T: fmt::Debug + Default,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "1. infinite_indices",
            bounded_deque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "2. sentinel",
            sentinel_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "3. full_flag",
            full_flag_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "4. len",
            len_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        #[cfg(feature = "benchmark")]
        {
            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "5. stack",
                stack_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "6. Vec",
                vec_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "7. VecDeque",
                vecdeque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "8. heapless::Deque",
                heapless_deque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );
        }
    }
}

fn cache_effects<T: fmt::Debug + Default>(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_effects");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    compare::<T, 8, 1>(&mut group);
    compare::<T, 8, 2>(&mut group);
    compare::<T, 8, 5>(&mut group);
    compare::<T, 8, 10>(&mut group);
    compare::<T, 8, 100>(&mut group);
    compare::<T, 8, 1_000>(&mut group);
    compare::<T, 8, 10_000>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize, const ELEMENT_COUNT: usize>(
        group: &mut BenchmarkGroup<WallTime>
    ) where
        T: fmt::Debug + Default,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "1. queue",
            bounded_deque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "2. stack",
            stack_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "3. reversed_stack",
            reversed_stack_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );
    }
}

#[cfg(feature = "benchmark")]
pub fn vec_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let vec = UnsafeCell::new(Vec::<T>::with_capacity(N));
    let producer = &vec;
    let consumer = &vec;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single(
                |value| unsafe { (&mut *producer.get()).push(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_option(|| unsafe { (&mut *consumer.get()).pop() }, batch_count)
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

#[cfg(feature = "benchmark")]
pub fn vecdeque_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let deque = UnsafeCell::new(VecDeque::<T>::with_capacity(N));
    let producer = &deque;
    let consumer = &deque;

    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| {
            producer_single(
                |value| unsafe { (&mut *producer.get()).push_back(value) },
                batch_count,
            )
        },
        move |batch_count| {
            consumer_single_option(
                || unsafe { (&mut *consumer.get()).pop_front() },
                batch_count,
            )
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

#[cfg(feature = "benchmark")]
pub fn heapless_deque_sequential<T: Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let deque = UnsafeCell::new(heapless::Deque::<T, N>::new());
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
            consumer_single_option(
                || unsafe { (&mut *consumer.get()).pop_front() },
                batch_count,
            )
        },
        ELEMENT_COUNT,
        sequential_batch_count::<N, ELEMENT_COUNT>(),
    )
}

#[ctor::ctor]
fn init() {
    log::init();
}
