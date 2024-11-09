use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use x86_64::instructions::{
    self,
    interrupts,
};

use kernel::{
    Subsystems,
    log::info,
    memory::{
        BASE_ADDRESS_SPACE,
        Virt,
        test_scaffolding::{
            mapping,
            phys2virt,
        },
    },
    process::test_scaffolding::set_handler,
    smp::test_scaffolding::{
        cpu_id,
        init_smp,
    },
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn ap_init() {
    set_handler(ap_loop);

    let phys2virt = phys2virt(mapping(&mut BASE_ADDRESS_SPACE.lock()));
    init_smp(phys2virt, Subsystems::SMP).unwrap();

    let cpu_count = 4;

    barrier(&BARRIER_START, cpu_count);
    race();
    barrier(&BARRIER_FINISH, cpu_count);

    let racy_counter = unsafe { RACY_COUNTER };
    let race_free_counter = RACE_FREE_COUNTER.load(Ordering::Relaxed);
    info!(racy_counter, race_free_counter);
    assert_ne!(racy_counter, race_free_counter);
    assert_eq!(race_free_counter, CPU_COUNTER * cpu_count);
}

fn ap_loop() {
    let cpu_count = 4;

    barrier(&BARRIER_START, cpu_count);
    race();
    barrier(&BARRIER_FINISH, cpu_count);

    let cpu = cpu_id();
    info!(cpu, "AP halted");

    loop {
        interrupts::without_interrupts(instructions::hlt)
    }
}

fn barrier(
    barrier: &AtomicUsize,
    cpu_count: usize,
) {
    let barrier_id = Virt::from_ref(barrier);
    let cpu = cpu_id();
    let cpus_waiting = barrier.fetch_add(1, Ordering::Relaxed);
    let last = cpus_waiting == cpu_count - 1;
    info!(%barrier_id, cpu, cpu_count, cpus_waiting, last, "arrived at the barrier");

    while barrier.load(Ordering::Relaxed) != cpu_count {
        instructions::hlt();
    }

    info!(%barrier_id, cpu, cpu_count, "all CPUs have arrived at the barrier");
}

fn race() {
    for _ in 0 .. CPU_COUNTER {
        RACE_FREE_COUNTER.fetch_add(1, Ordering::Relaxed);
        unsafe {
            RACY_COUNTER += 1;
        }
    }
}

static BARRIER_START: AtomicUsize = AtomicUsize::new(0);
static BARRIER_FINISH: AtomicUsize = AtomicUsize::new(0);
static RACE_FREE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static mut RACY_COUNTER: usize = 0;

const CPU_COUNTER: usize = 5_000_000;
