#![deny(warnings)]

#[path = "../common/workload.rs"]
mod workload;

use std::{
    fmt,
    time::Instant,
};

use clap::{
    Parser,
    ValueEnum,
};
use spin::mutex::{
    SpinMutex as SpinLock,
    TicketMutex as TicketLock,
};

use mcs_lock::McsLock;

use workload::{
    heavy,
    run_lock_workload,
};

use crate::workload::pin_to;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum LockImplementation {
    Spin,
    Ticket,
    Mcs,
}

impl fmt::Display for LockImplementation {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.to_possible_value().expect("no values are skipped").get_name().fmt(f)
    }
}

#[derive(Parser, Debug)]
#[command(name = "workload")]
struct Cli {
    #[arg(required = true)]
    lock_impl: LockImplementation,

    #[arg(short, long, default_value_t = 4)]
    threads: u64,

    #[arg(short, long, default_value_t = 100_000)]
    iterations: u64,

    #[arg(
        long,
        default_value_t = false,
        help = "Don't pin workload threads to cores"
    )]
    no_pin: bool,
}

fn main() {
    let args = Cli::parse();

    let setup: &(dyn Fn(u64) + Send + Sync) = if !args.no_pin {
        &pin_to
    } else {
        &|_| {}
    };

    let start = Instant::now();
    match args.lock_impl {
        LockImplementation::Spin =>
            run_lock_workload::<SpinLock<u64>>(args.threads, args.iterations, heavy, setup),
        LockImplementation::Ticket =>
            run_lock_workload::<TicketLock<u64>>(args.threads, args.iterations, heavy, setup),
        LockImplementation::Mcs =>
            run_lock_workload::<McsLock<u64>>(args.threads, args.iterations, heavy, setup),
    }
    eprintln!("Time elapsed: {:?}", start.elapsed());
}
