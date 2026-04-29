#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(likely_unlikely)]

use std::{
    cmp,
    fmt::Debug,
    hint,
    time::{
        Duration,
        Instant,
    },
};

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

use ku::{
    memory::size,
    sync::Spsc,
};

use producer_consumer::{
    benchmark,
    consumer_tx,
    producer_consumer_concurrent,
    producer_tx,
};

#[cfg(feature = "benchmark")]
use producer_consumer::{
    bounded_deque_sequential,
    sentinel_sequential,
    spsc_single_concurrent,
    spsc_tx_concurrent,
};

mod producer_consumer;

#[path = "../tests/log/mod.rs"]
mod log;

criterion_main!(benches);

criterion_group!(benches, slice<u32>);

fn slice<T: Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("slice");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    #[cfg(feature = "benchmark")]
    {
        compare::<T, 14, 1>(&mut group);
        compare::<T, 14, 10>(&mut group);
        compare::<T, 14, 100>(&mut group);
    }

    compare::<T, 14, 1_000>(&mut group);
    compare::<T, 14, 10_000>(&mut group);
    compare::<T, 14, 100_000>(&mut group);
    compare::<T, 14, 1_000_000>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize, const ELEMENT_COUNT: usize>(
        group: &mut BenchmarkGroup<'_, WallTime>
    ) where
        T: Copy + Debug + Default + Send,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        #[cfg(feature = "benchmark")]
        {
            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "1. single spsc",
                spsc_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "2. tx spsc",
                spsc_tx_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "3. infinite_indices",
                bounded_deque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "4. sentinel",
                sentinel_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );
        }

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "5. slice spsc",
            spsc_slice_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        #[cfg(feature = "benchmark")]
        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "6. memory copy",
            memory_copy::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "7. double memory copy",
            double_memory_copy::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );
    }
}

fn spsc_slice_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let spsc = Spsc::<T, N>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();
    let producer_values = vec![T::default(); N];
    let mut consumer_values = vec![T::default(); N];
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || {
            producer_tx(
                |element_count| {
                    let mut tx = producer.tx();
                    let count = cmp::min(element_count, N);
                    let count = tx.enqueue_slice(hint::black_box(&producer_values[.. count]));
                    tx.commit();
                    count
                },
                ELEMENT_COUNT,
            )
        },
        move || {
            consumer_tx(
                |element_count| {
                    let mut tx = consumer.tx();
                    let count = cmp::min(element_count, N);
                    let count = tx.dequeue_slice(hint::black_box(&mut consumer_values[.. count]));
                    tx.commit();
                    count
                },
                ELEMENT_COUNT,
            )
        },
    )
}

#[cfg(feature = "benchmark")]
fn memory_copy<T: Copy + Debug + Default + Send, const N: usize, const ELEMENT_COUNT: usize>(
    _id: &str,
    iteration_count: u64,
) -> Duration {
    let producer_values = vec![T::default(); N];
    let mut consumer_values = vec![T::default(); N];
    let timer = Instant::now();
    for _ in 0 .. iteration_count {
        let mut element_count = ELEMENT_COUNT;
        while element_count > 0 {
            let count = cmp::min(element_count, N);
            hint::black_box(&mut consumer_values[.. count])
                .copy_from_slice(hint::black_box(&producer_values[.. count]));
            element_count -= count
        }
    }
    timer.elapsed()
}

fn double_memory_copy<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    _id: &str,
    iteration_count: u64,
) -> Duration {
    let producer_values = vec![T::default(); N];
    let mut queue = vec![T::default(); N];
    let mut consumer_values = vec![T::default(); N];
    let timer = Instant::now();
    for _ in 0 .. iteration_count {
        let mut element_count = ELEMENT_COUNT;
        while element_count > 0 {
            let count = cmp::min(element_count, N);
            hint::black_box(&mut queue[.. count])
                .copy_from_slice(hint::black_box(&producer_values[.. count]));
            hint::black_box(&mut consumer_values[.. count])
                .copy_from_slice(hint::black_box(&queue[.. count]));
            element_count -= count
        }
    }
    timer.elapsed()
}

#[ctor::ctor]
fn init() {
    log::init();
}
