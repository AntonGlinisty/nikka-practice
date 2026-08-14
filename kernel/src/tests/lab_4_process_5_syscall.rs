use crate::{
    process::{
        Process,
        test_scaffolding,
    },
    trap::{
        TRAP_STATS,
        Trap,
    },
};

use super::process_helpers;

const EXIT_ELF: &[u8] = page_aligned!("../../../target/kernel/user/exit");
const LOG_VALUE_ELF: &[u8] = page_aligned!("../../../target/kernel/user/log_value");

pub(super) fn syscall_exit() {
    let _trap_guard = process_helpers::forbid_traps();

    let mut process = process_helpers::dummy_allocate(EXIT_ELF);

    test_scaffolding::disable_interrupts(&mut process);

    Process::enter_user_mode(process);

    assert_eq!(
        TRAP_STATS[Trap::InvalidOpcode].count(),
        0,
        "probably the `syscall` instruction is not initialized",
    );

    assert_eq!(
        TRAP_STATS[Trap::PageFault].count(),
        0,
        concat!(
            "if the Page Fault was in the kernel mode, ",
            "probably the `syscall` instruction is not initialized or ",
            "the kernel has not switched to its own stack; ",
            "if it was in the user mode, maybe the time functions from the first lab ",
            "use `read-dont-modify-write` construction",
        ),
    );
}

pub(super) fn syscall_log_value() {
    let _trap_guard = process_helpers::forbid_traps();

    let mut process = process_helpers::dummy_allocate(LOG_VALUE_ELF);

    test_scaffolding::disable_interrupts(&mut process);

    Process::enter_user_mode(process);

    assert_eq!(
        TRAP_STATS[Trap::PageFault].count(),
        0,
        "the user mode code has detected an error in syscall::log_value() implementation",
    );
}
