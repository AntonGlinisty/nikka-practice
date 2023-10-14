extern crate alloc;

use alloc::vec::Vec;
use core::slice;

use kernel::{
    Subsystems,
    log::debug,
    memory::test_scaffolding::forbid_frame_leaks,
    process::test_scaffolding::{
        PROCESS_SLOT_COUNT,
        dummy_process,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

#[test_case]
fn basic() {
    let _guard = forbid_frame_leaks();

    let pid_1 = dummy_process().unwrap();
    let pid_2 = dummy_process().unwrap();

    debug!(%pid_1, %pid_2);
    assert_ne!(pid_1.slot(), pid_2.slot());

    process_helpers::free(pid_1);
    process_helpers::free(pid_2);
}

#[test_case]
fn full_capacity() {
    let mut pids = Vec::with_capacity(PROCESS_SLOT_COUNT);
    let mut slots = Vec::with_capacity(PROCESS_SLOT_COUNT);

    let _guard = forbid_frame_leaks();

    while pids.len() < PROCESS_SLOT_COUNT {
        let pid = dummy_process().expect("failed to create the test process");
        pids.push(pid);
        slots.push(pid.slot());
    }

    dummy_process().expect_err("the process table is bigger than it should be");

    slice::heapsort(slots.as_mut_slice(), |a, b| a < b);
    let slots_have_expected_range =
        slots.into_iter().enumerate().all(|(index, slot)| index == slot);
    assert!(slots_have_expected_range);

    let mut prev_pid = pids[0];
    for _ in 0 .. 10 {
        process_helpers::free(pids[0]);
        let pid = dummy_process().expect("failed to create the test process");
        pids[0] = pid;

        dummy_process().expect_err("the process table is bigger than it should be");

        debug!(%prev_pid, %pid);
        assert_ne!(prev_pid, pid);

        prev_pid = pid;
    }

    while let Some(pid) = pids.pop() {
        process_helpers::free(pid);
    }
}
