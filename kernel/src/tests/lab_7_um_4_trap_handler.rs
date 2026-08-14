use crate::{
    process::{
        Scheduler,
        Table,
        test_scaffolding::disable_interrupts,
    },
    trap::{
        TRAP_STATS,
        Trap,
    },
};

use super::{
    mm_helpers,
    process_helpers,
};

const TRAP_HANDLER_ELF: &[u8] = page_aligned!("../../../target/kernel/user/trap_handler");

pub(super) fn trap_handler() {
    let _trap_guard = process_helpers::forbid_traps_except(&[Trap::PageFault]);
    let _guard = mm_helpers::forbid_frame_leaks();

    let pid = process_helpers::allocate(TRAP_HANDLER_ELF).pid();

    Scheduler::enqueue(pid);

    while Scheduler::run_one() {
        if let Ok(mut process) = Table::get(pid) {
            disable_interrupts(&mut process);
        }
    }

    let expected_page_faults = 1 + MAX_RECURSION_LEVEL + 2;

    assert_eq!(
        TRAP_STATS[Trap::PageFault].count(),
        expected_page_faults,
        "trap_handler should page fault once on a non-recursive page fault and \
         MAX_RECURSION_LEVEL + 2 on a recursive one",
    );
}

const MAX_RECURSION_LEVEL: usize = 8;
