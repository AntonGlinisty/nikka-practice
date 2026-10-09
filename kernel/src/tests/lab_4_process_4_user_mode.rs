use ku::{
    process::registers::RFlags,
    sync::spinlock::Spinlock,
};

use crate::{
    log::debug,
    process::{
        Process,
        test_scaffolding,
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

const PAGE_FAULT_ELF: &[u8] = page_aligned!("../../../target/kernel/user/page-fault.stripped");

pub(super) fn user_mode_page_fault() {
    let _trap_guard = process_helpers::forbid_traps_except(&[Trap::PageFault]);
    let _guard = mm_helpers::forbid_frame_leaks();

    let process = Spinlock::new(process_helpers::make(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    let start_page_faults = TRAP_STATS[Trap::PageFault].count();

    Process::enter_user_mode(process.lock());

    assert_eq!(
        TRAP_STATS[Trap::PageFault].count(),
        start_page_faults + 1,
        "probably the user mode page faults are not handled or counted",
    );
}

pub(super) fn user_context_saved() {
    let _trap_guard = process_helpers::forbid_traps_except(&[Trap::PageFault]);
    let _guard = mm_helpers::forbid_frame_leaks();

    let process = Spinlock::new(process_helpers::make(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    Process::enter_user_mode(process.lock());

    let user_registers = test_scaffolding::registers(&process.lock());
    debug!(?user_registers);
    let user_mode_context_sum = user_registers.into_iter().sum::<usize>();
    let expected_sum = (77701 ..= 77715).sum();
    assert_eq!(user_mode_context_sum, expected_sum);

    assert!(
        RFlags::read().contains(RFlags::INTERRUPT_FLAG),
        "enable the interrupts after the final return to the kernel stack",
    );
}
