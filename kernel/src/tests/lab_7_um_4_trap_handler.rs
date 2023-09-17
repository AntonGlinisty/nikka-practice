use kernel::{
    Subsystems,
    memory::test_scaffolding::forbid_frame_leaks,
    process::{
        Scheduler,
        Table,
        test_scaffolding::disable_interrupts,
    },
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const TRAP_HANDLER_ELF: &[u8] = page_aligned!("../../target/kernel/user/trap_handler");

#[test_case]
fn trap_handler() {
    let _guard = forbid_frame_leaks();

    let pid = process_helpers::allocate(TRAP_HANDLER_ELF).pid();

    Scheduler::enqueue(pid);

    while Scheduler::run_one() {
        if let Ok(mut process) = Table::get(pid) {
            disable_interrupts(&mut process);
        }
    }

    let expected_page_faults = 1 + MAX_RECURSION_LEVEL + 2;

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        expected_page_faults,
        "trap_handler should page fault once on a non-recursive page fault and \
         MAX_RECURSION_LEVEL + 2 on a recursive one",
    );
}

const MAX_RECURSION_LEVEL: usize = 8;
