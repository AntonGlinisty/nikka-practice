#![allow(incomplete_features)]
#![deny(warnings)]
#![feature(adt_const_params)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(likely_unlikely)]

use std::{
    fmt::Debug,
    hint,
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

use ku::{
    memory::size,
    sync::Mpmc,
};

use producer_consumer::{
    benchmark,
    consumer_single_result,
    producer_consumer_concurrent,
    producer_single_result,
    spsc_single_concurrent,
};

#[path = "../tests/log/mod.rs"]
mod log;

mod producer_consumer;

criterion_main!(benches);

criterion_group!(benches, mpmc<u32>, roundtrip<u32>);

fn mpmc<T>(c: &mut Criterion)
where
    T: Default + Send + TryFrom<u32>,
    u32: From<T>,
{
    let mut group = c.benchmark_group("mpmc");
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
        T: Default + Send + TryFrom<u32>,
        u32: From<T>,
        [(); 1 << LB_N]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(ELEMENT_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "1. spsc",
            spsc_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "2. mpmc",
            mpmc_single_concurrent::<T, { 1 << LB_N }, ELEMENT_COUNT>,
        );
    }
}

fn roundtrip<T>(c: &mut Criterion)
where
    T: Copy + Debug + Send + PartialEq + TryFrom<u32>,
    u32: From<T>,
{
    let mut group = c.benchmark_group("roundtrip");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.throughput(Throughput::Elements(size::into_u64(1)));

    const CAPACITY: usize = 1 << 14;

    benchmark::<T, _>(
        &mut group,
        CAPACITY,
        1,
        "mpmc roundtrip",
        mpmc_roundtrip::<T, CAPACITY>,
    );

    group.finish();
}

fn mpmc_single_concurrent<T, const N: usize, const ELEMENT_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Default + Send + TryFrom<u32>,
    u32: From<T>,
{
    let mpmc = Mpmc::<T, N>::new();
    let producer = &mpmc;
    let consumer = &mpmc;
    producer_consumer_concurrent(
        id,
        iteration_count,
        move || producer_single_result(|value| producer.enqueue(value), ELEMENT_COUNT),
        move || consumer_single_result(|| consumer.dequeue(), ELEMENT_COUNT),
    )
}

fn mpmc_roundtrip<T, const N: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Copy + Debug + PartialEq + Send + TryFrom<u32>,
    u32: From<T>,
{
    let mpmc_a = Mpmc::<T, N>::new();
    let mpmc_b = Mpmc::<T, N>::new();

    let core_count = affinity::get_core_num();

    thread::scope(|scope| {
        let ping = thread::Builder::new()
            .name(format!("{id} ping"))
            .spawn_scoped(scope, || {
                affinity::set_thread_affinity(&[0]).unwrap();
                let mut timer = Instant::now();
                for iteration in 0 ..= size::from(iteration_count).try_into().unwrap() {
                    if iteration == 0 {
                        timer = Instant::now();
                    }
                    let send_value = T::try_from(iteration).map_err(|_| ()).unwrap();
                    hint::black_box(mpmc_a.enqueue(hint::black_box(send_value))).unwrap();
                    loop {
                        if let Ok(receive_value) = hint::black_box(mpmc_b.dequeue()) {
                            assert_eq!(receive_value, send_value);
                            break;
                        }
                    }
                }
                timer.elapsed()
            })
            .unwrap();

        let pong = thread::Builder::new()
            .name(format!("{id} pong"))
            .spawn_scoped(scope, || {
                affinity::set_thread_affinity(&[core_count / 2]).unwrap();
                for _ in 0 ..= iteration_count {
                    loop {
                        if let Ok(value) = hint::black_box(mpmc_a.dequeue()) {
                            hint::black_box(mpmc_b.enqueue(value)).unwrap();
                            break;
                        }
                    }
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
