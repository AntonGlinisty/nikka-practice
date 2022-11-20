use ku::{
    error::Error::{
        InvalidArgument,
        PermissionDenied,
    },
    process::{
        Pid,
        State,
    },
};

use kernel::{
    Subsystems,
    log::debug,
    memory::test_scaffolding::{
        forbid_frame_leaks,
        switch_to,
    },
    process::{
        Scheduler,
        Table,
        test_scaffolding::{
            exofork,
            set_state,
        },
    },
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const EAGER_FORK_ELF: &[u8] = include_bytes!("../../target/kernel/user/eager_fork");
const EXIT_ELF: &[u8] = include_bytes!("../../target/kernel/user/exit");

#[test_case]
fn exofork_syscall() {
    let _guard = forbid_frame_leaks();

    let mut parent = process_helpers::allocate(EXIT_ELF);
    let parent_pid = parent.pid();
    let unrelated_process_pid = process_helpers::allocate(EXIT_ELF).pid();

    switch_to(parent.address_space());

    let child_pid = Pid::from_usize(exofork(parent).expect("exofork() failed").0)
        .expect("wrong child pid from exofork()");

    debug!(%child_pid);

    let child =
        Table::get(child_pid).expect("failed to find the child process in the process table");

    debug!(%child);

    drop(child);

    assert_eq!(
        set_state(
            Table::get(parent_pid).unwrap(),
            unrelated_process_pid.into_usize(),
            State::RUNNABLE.bits(),
        ),
        Err(PermissionDenied),
    );

    assert_eq!(
        set_state(Table::get(parent_pid).unwrap(), child_pid.into_usize(), 42),
        Err(InvalidArgument),
    );

    assert_eq!(
        set_state(
            Table::get(parent_pid).unwrap(),
            child_pid.into_usize(),
            State::RUNNING.bits(),
        ),
        Err(InvalidArgument),
    );

    let result = set_state(
        Table::get(parent_pid).unwrap(),
        child_pid.into_usize(),
        State::RUNNABLE.bits(),
    );
    assert!(result.is_ok());

    assert_eq!(
        set_state(
            Table::get(parent_pid).unwrap(),
            child_pid.into_usize(),
            State::RUNNABLE.bits(),
        ),
        Err(PermissionDenied),
    );

    for pid in [parent_pid, unrelated_process_pid] {
        Table::free(Table::get(pid).unwrap());
    }

    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        1,
        "the child process should page fault",
    );

    Table::get(child_pid).expect_err("the child process has not run up to its completion");
}

#[test_case]
fn eager_fork() {
    let start_page_fault_count = TRAP_STATS[PAGE_FAULT].count();

    let pid = process_helpers::allocate(EAGER_FORK_ELF).pid();

    Scheduler::enqueue(pid);

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        start_page_fault_count,
        "eager_fork should not page fault",
    );
}
