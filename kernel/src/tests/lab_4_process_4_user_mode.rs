use ku::{
    process::registers::RFlags,
    sync::spinlock::Spinlock,
};

use kernel::{
    Subsystems,
    log::debug,
    memory::test_scaffolding::forbid_frame_leaks,
    process::{
        Process,
        test_scaffolding,
    },
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY);

const PAGE_FAULT_ELF: &[u8] = include_bytes!("../../target/kernel/user/page_fault");

#[test_case]
fn user_mode_page_fault() {
    let _guard = forbid_frame_leaks();

    let process = Spinlock::new(process_helpers::make(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    Process::enter_user_mode(process.lock());

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        1,
        "probably the user mode page faults are not handled or counted",
    );
}

#[test_case]
fn user_context_saved() {
    let _guard = forbid_frame_leaks();

    let process = Spinlock::new(process_helpers::make(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    Process::enter_user_mode(process.lock());

    let user_registers = test_scaffolding::registers(&process.lock());
    debug!(?user_registers);
    let user_mode_context_saved = user_registers
        .into_iter()
        .enumerate()
        .all(|(i, register_value)| register_value == 77701 + i);
    assert!(user_mode_context_saved);

    assert!(
        RFlags::read().contains(RFlags::INTERRUPT_FLAG),
        "enable the interrupts after the final return to the kernel stack",
    );
}
