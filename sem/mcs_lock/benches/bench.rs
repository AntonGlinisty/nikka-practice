#![deny(warnings)]

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use spin::mutex::{
    SpinMutex as SpinLock,
    TicketMutex as TicketLock,
};

#[path = "../common/workload.rs"]
mod workload;

use mcs_lock::McsLock;
use workload::{
    Lockable,
    heavy,
    pin_to,
    run_lock_workload,
};

const NUM_ITERATIONS: u64 = 100_000;

fn lock_benchmark<L: Lockable<u64>>(c: &mut Criterion) {
    for threads in [1, 2, 4, 6, 8, 12, 16] {
        c.bench_function(
            &format!("{}-{}", std::any::type_name::<L>(), threads),
            |b| b.iter(|| run_lock_workload::<L>(threads, NUM_ITERATIONS, heavy, pin_to)),
        );
    }
}

criterion_group!(
    benches,
    lock_benchmark::<McsLock<u64>>,
    lock_benchmark::<SpinLock<u64>>,
    lock_benchmark::<TicketLock<u64>>,
);
criterion_main!(benches);
