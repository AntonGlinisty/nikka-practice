use kernel::{
    Subsystems,
    log::debug,
    memory::test_scaffolding::forbid_frame_leaks,
    process::{
        Process,
        Table,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const LOOP_ELF: &[u8] = page_aligned!("../../target/kernel/user/loop");

#[test_case]
fn preemption() {
    let _guard = forbid_frame_leaks();

    let pid = process_helpers::allocate(LOOP_ELF).pid();

    let process = Table::get(pid).expect("failed to find the new process in the process table");
    Process::enter_user_mode(process);

    // If this does not happen and the test times out, the preemption is not working properly.
    debug!("returned from the user space");

    process_helpers::free(pid);
}
