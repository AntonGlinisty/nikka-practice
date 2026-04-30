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
    sync::ShardedMpmc,
};

use producer_consumer::{
    benchmark,
    consumer_single_result,
    producer_consumer_multiple_concurrent,
    producer_single_result,
    spsc_single_concurrent,
};

#[path = "../tests/log/mod.rs"]
mod log;

mod producer_consumer;

criterion_main!(benches);

criterion_group!(benches, sharded_mpmc<u32>, roundtrip<u32>);

fn sharded_mpmc<T: Clone + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("sharded_mpmc");
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
        T: Clone + Default + Send,
        [(); 1 << LB_N]:,
        [(); (1 << LB_N) / 1]:,
        [(); (1 << LB_N) / 4]:,
        [(); (1 << LB_N) / 16]:,
        [(); (1 << LB_N) / 64]:,
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
            "2. sharded<1> mpmc",
            sharded_mpmc_single_concurrent::<T, { 1 << LB_N }, 1, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "3. sharded<4> mpmc",
            sharded_mpmc_single_concurrent::<T, { 1 << LB_N }, 4, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "4. sharded<16> mpmc",
            sharded_mpmc_single_concurrent::<T, { 1 << LB_N }, 16, ELEMENT_COUNT>,
        );

        benchmark::<T, _>(
            group,
            1 << LB_N,
            ELEMENT_COUNT,
            "5. sharded<64> mpmc",
            sharded_mpmc_single_concurrent::<T, { 1 << LB_N }, 64, ELEMENT_COUNT>,
        );
    }
}

fn roundtrip<T: Copy + Debug + Default + Send>(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    compare::<T, 14, 0>(&mut group);
    compare::<T, 14, 1>(&mut group);
    compare::<T, 14, 2>(&mut group);
    compare::<T, 14, 3>(&mut group);
    compare::<T, 14, 4>(&mut group);
    compare::<T, 14, 5>(&mut group);
    compare::<T, 14, 6>(&mut group);

    group.finish();

    fn compare<T, const LB_N: usize, const LB_SHARD_COUNT: usize>(
        group: &mut BenchmarkGroup<'_, WallTime>
    ) where
        T: Copy + Debug + Default + Send,
        [(); 1 << LB_N]:,
        [(); 1 << LB_SHARD_COUNT]:,
        [(); (1 << LB_N) / (1 << LB_SHARD_COUNT)]:,
    {
        group.throughput(Throughput::Elements(size::into_u64(1 << LB_SHARD_COUNT)));

        benchmark::<T, _>(
            group,
            1 << LB_N,
            1,
            &format!(
                "{}. sharded<{}> mpmc roundtrip",
                LB_SHARD_COUNT + 1,
                1 << LB_SHARD_COUNT,
            ),
            sharded_mpmc_roundtrip::<T, { 1 << LB_N }, { 1 << LB_SHARD_COUNT }>,
        );
    }
}

fn sharded_mpmc_single_concurrent<
    T,
    const N: usize,
    const SHARD_COUNT: usize,
    const ELEMENT_COUNT: usize,
>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Clone + Default + Send,
    [(); N / SHARD_COUNT]:,
{
    let mpmc = ShardedMpmc::<T, N, SHARD_COUNT>::new();
    let (mut producer, mut consumer) = mpmc.channel();
    producer_consumer_multiple_concurrent(
        id,
        SHARD_COUNT,
        iteration_count,
        move || {
            producer_single_result(|value| producer.enqueue(value), ELEMENT_COUNT / SHARD_COUNT)
        },
        move || consumer_single_result(|| consumer.dequeue(), ELEMENT_COUNT / SHARD_COUNT),
    )
}

fn sharded_mpmc_roundtrip<T, const N: usize, const SHARD_COUNT: usize>(
    id: &str,
    iteration_count: u64,
) -> Duration
where
    T: Copy + Debug + Default + Send,
    [(); N / SHARD_COUNT]:,
{
    let mpmc_a = ShardedMpmc::<T, N, SHARD_COUNT>::new();
    let mpmc_b = ShardedMpmc::<T, N, SHARD_COUNT>::new();
    let (mut producer_a, mut consumer_a) = mpmc_a.channel();
    let (mut producer_b, mut consumer_b) = mpmc_b.channel();

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
                    while hint::black_box(consumer_b.dequeue()).is_err() {}
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
                        if let Ok(value) = hint::black_box(consumer_a.dequeue()) {
                            hint::black_box(producer_b.enqueue(value)).unwrap();
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
