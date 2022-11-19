use kernel::{
    Subsystems,
    memory::test_scaffolding::forbid_frame_leaks,
    process::Scheduler,
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const MEMORY_ALLOCATOR_ELF: &[u8] = include_bytes!("../../target/kernel/user/memory_syscalls");

#[test_case]
fn user_space_memory_allocator() {
    let _guard = forbid_frame_leaks();

    Scheduler::enqueue(process_helpers::allocate(MEMORY_ALLOCATOR_ELF).pid());

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        0,
        "the user mode code has detected an error in the memory allocator implementation",
    );
}
