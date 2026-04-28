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
    ops::Add,
    thread,
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

use affinity;

use ku::{
    memory::size,
    sync::Spsc,
};

use producer_consumer::{
    benchmark,
    bounded_deque_sequential,
    consumer_single_result,
    producer_consumer_sequential,
    producer_single_result,
    sentinel_sequential,
    spsc_single_concurrent,
    spsc_tx_concurrent,
};

#[cfg(feature = "benchmark")]
use {
    bounded_spsc_queue,
    crossbeam_channel,
    gil,
    heapless,
    producer_consumer::{
        consumer_single_option,
        producer_consumer_concurrent,
        producer_single_option,
    },
    ringbuffer_spsc,
    rtrb,
    smallring,
    std::{
        num::NonZero,
        sync::mpsc,
    },
};

#[path = "../tests/log/mod.rs"]
mod log;

mod producer_consumer;

criterion_main!(benches);

criterion_group!(
    benches,
    sequential<u32>,
    concurrent<u32>,
    crates<u32>,
    roundtrip<u32>,
    tx<u32>,
);

fn sequential<T: Copy + Debug + Default>(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");
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
        T: Copy + Debug + Default,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "1. sequential spsc",
            spsc_single_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "2. infinite_indices",
            bounded_deque_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "3. sentinel",
            sentinel_sequential::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );
    }
}

fn concurrent<T: Add<T, Output = T> + Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
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
        T: Add<T, Output = T> + Copy + Debug + Default + Send,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "1. spsc",
            spsc_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        #[cfg(feature = "benchmark")]
        {
            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "2. bounded_spsc_queue",
                bounded_spsc_queue_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "3. gil",
                gil_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "4. heapless",
                heapless_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "5. rtrb",
                rtrb_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "6. smallring",
                smallring_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "7. std mpsc::sync_channel",
                std_mpsc_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "8. crossbeam::bounded",
                crossbeam_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );
        }
    }
}

fn crates<T: Add<T, Output = T> + Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("crates");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    compare::<T, 14, 1>(&mut group);
    compare::<T, 14, 10>(&mut group);
    compare::<T, 14, 100>(&mut group);
    compare::<T, 14, 1_000>(&mut group);
    compare::<T, 14, 10_000>(&mut group);
    compare::<T, 14, 100_000>(&mut group);
    compare::<T, 14, 1_000_000>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize, const ELEMENT_COUNT: usize>(
        group: &mut BenchmarkGroup<'_, WallTime>
    ) where
        T: Add<T, Output = T> + Copy + Debug + Default + Send,
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
                "bounded_spsc_queue",
                bounded_spsc_queue_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "gil",
                gil_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "rtrb",
                rtrb_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "smallring",
                smallring_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );

            benchmark::<T, _>(
                group,
                1 << LB_N,
                ELEMENT_COUNT,
                "ringbuffer_spsc",
                ringbuffer_spsc_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
            );
        }
    }
}

fn roundtrip<T: Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    compare::<T, 14>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize>(group: &mut BenchmarkGroup<'_, WallTime>)
    where
        T: Copy + Debug + Default + Send,
        [(); 1 << LB_N]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(1)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            1,
            "spsc roundtrip",
            spsc_roundtrip::<T, { 1 << LB_N }>,
        );
    }
}

fn tx<T: Add<T, Output = T> + Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("tx");
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
        T: Add<T, Output = T> + Copy + Debug + Default + Send,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) + 1]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

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
}

fn spsc_single_sequential<T: Copy + Debug + Default, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let spsc = Spsc::<T, N>::new();
    let (mut producer, mut consumer) = spsc.channel().unwrap();
    producer_consumer_sequential(
        id,
        iteration_count,
        move |batch_count| producer_single_result(|value| producer.enqueue(value), batch_count),
        move |batch_count| consumer_single_result(|| consumer.dequeue(), batch_count),
        ELEMENT_COUNT,
        cmp::min(cmp::max(ELEMENT_COUNT / 16, 1), N),
    )
}

#[cfg(feature = "benchmark")]
fn bounded_spsc_queue_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (producer, consumer) = bounded_spsc_queue::make::<T>(N);
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_option(|value| producer.try_push(value), ELEMENT_COUNT),
        move || consumer_single_option(|| consumer.try_pop(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn smallring_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (mut producer, mut consumer) = smallring::spsc::new::<T, N>(NonZero::new(N).unwrap());
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.push(value), ELEMENT_COUNT),
        move || consumer_single_result(|| consumer.pop(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn ringbuffer_spsc_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (mut producer, mut consumer) = ringbuffer_spsc::ringbuffer::<T>(N);
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_option(|value| producer.push(value), ELEMENT_COUNT),
        move || consumer_single_option(|| consumer.pull(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn rtrb_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (mut producer, mut consumer) = rtrb::RingBuffer::<T>::new(N);
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.push(value), ELEMENT_COUNT),
        move || consumer_single_result(|| consumer.pop(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn gil_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (mut producer, mut consumer) = gil::spsc::channel::<T>(NonZero::new(N).unwrap());
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.try_send(value), ELEMENT_COUNT),
        move || consumer_single_option(|| consumer.try_recv(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn heapless_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let mut spsc = heapless::spsc::Queue::<T, N>::new();
    let (mut producer, mut consumer) = spsc.split();
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.enqueue(value), ELEMENT_COUNT),
        move || consumer_single_option(|| consumer.dequeue(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn std_mpsc_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (producer, consumer) = mpsc::sync_channel::<T>(N);
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.try_send(value), ELEMENT_COUNT),
        move || consumer_single_result(|| consumer.try_recv(), ELEMENT_COUNT),
    )
}

#[cfg(feature = "benchmark")]
fn crossbeam_single_concurrent<
    T: Copy + Debug + Default + Send,
    const N: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let (producer, consumer) = crossbeam_channel::bounded::<T>(N);
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.send(value), ELEMENT_COUNT),
        move || consumer_single_result(|| consumer.try_recv(), ELEMENT_COUNT),
    )
}

fn spsc_roundtrip<T: Copy + Debug + Default + Send, const N: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration {
    let spsc_a = Spsc::<T, N>::new();
    let spsc_b = Spsc::<T, N>::new();
    let (mut producer_a, mut consumer_a) = spsc_a.channel().unwrap();
    let (mut producer_b, mut consumer_b) = spsc_b.channel().unwrap();

    let core_count = affinity::get_core_num();

    thread::scope(|scope| {
        let ping = thread::Builder::new()
            .name(format!("{id} ping"))
            .spawn_scoped(scope, || {
                affinity::set_thread_affinity(&[0]).unwrap();
                let mut timer = Instant::now();
                for iteration in 0 ..= iteration_count {
                    if iteration == 0 {
                        timer = Instant::now();
                    }
                    hint::black_box(producer_a.enqueue(hint::black_box(T::default()))).unwrap();
                    while consumer_b.is_empty() {
                        consumer_b.update_len();
                    }
                    hint::black_box(consumer_b.dequeue()).unwrap();
                }
                timer.elapsed()
            })
            .unwrap();

        let pong = thread::Builder::new()
            .name(format!("{id} pong"))
            .spawn_scoped(scope, || {
                affinity::set_thread_affinity(&[core_count / 2]).unwrap();
                for _ in 0 ..= iteration_count {
                    while consumer_a.is_empty() {
                        consumer_a.update_len();
                    }
                    hint::black_box(
                        producer_b.enqueue(hint::black_box(consumer_a.dequeue()).unwrap()),
                    )
                    .unwrap();
                }
                Instant::now()
            })
            .unwrap();

        let duration = ping.join().unwrap();
        pong.join().unwrap();

        duration
    })
}

#[ctor::ctor]
fn init() {
    log::init();
}
