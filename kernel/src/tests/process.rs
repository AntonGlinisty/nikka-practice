use spin::MutexGuard;
use xmas_elf::ElfFile;

use kernel::{
    log::{
        debug,
        info,
    },
    memory::{
        FRAME_ALLOCATOR,
        Virt,
        mmu::PageTableFlags,
        test_scaffolding::forbid_frame_leaks,
    },
    process::{
        self,
        Pid,
        Process,
        Scheduler,
        Table,
        test_scaffolding,
    },
    trap::{
        GENERAL_PROTECTION_FAULT,
        INVALID_OPCODE,
        PAGE_FAULT,
        TRAP_STATS,
    },
};

const COW_FORK_ELF: &[u8] = include_bytes!("../../../target/kernel/user/cow_fork");
const EAGER_FORK_ELF: &[u8] = include_bytes!("../../../target/kernel/user/eager_fork");
const EXIT_ELF: &[u8] = include_bytes!("../../../target/kernel/user/exit");
const LOOP_ELF: &[u8] = include_bytes!("../../../target/kernel/user/loop");
const PAGE_FAULT_ELF: &[u8] = include_bytes!("../../../target/kernel/user/page_fault");
const SCHED_YIELD_ELF: &[u8] = include_bytes!("../../../target/kernel/user/sched_yield");

fn make_process(file: &[u8]) -> MutexGuard<'static, Process> {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let pid = process::create(file).expect("failed to create the test process");
    let mut process = Table::get(pid).expect("failed to find the new process in the process table");

    let entry_point = Virt::new_u64(ElfFile::new(file).unwrap().header.pt2.entry_point()).unwrap();

    let mapping_error =
        "the ELF file has not been loaded into the address space at the correct address";
    let pte = process
        .address_space()
        .mapping()
        .translate(entry_point, None, PageTableFlags::empty())
        .expect(mapping_error);

    info!(%entry_point, frame = ?pte.frame().expect(mapping_error), flags = ?pte.flags(), "user process page table entry");

    assert!(pte.is_present(), "{}", mapping_error);
    assert!(
        pte.flags().contains(PageTableFlags::USER_ACCESSIBLE),
        "the ELF file is not accessible from the user space",
    );
    assert!(
        !pte.flags().contains(PageTableFlags::NO_EXECUTE),
        "the entry point is not executable",
    );

    let process_frames = start_free_frames - FRAME_ALLOCATOR.lock().count();
    debug!(process_frames);
    assert!(process_frames > 0, "created process uses no memory");

    process
}

fn free_process(pid: Pid) {
    let process = Table::get(pid).expect("failed to find the new process in the process table");
    Table::free(process);
}

#[test_case]
fn create_process() {
    let _guard = forbid_frame_leaks();

    let pid = make_process(LOOP_ELF).pid();

    free_process(pid);
}

#[test_case]
fn create_process_failure() {
    let _guard = forbid_frame_leaks();

    let bad_elf_file: &[u8] = &[];
    let error = process::create(bad_elf_file).expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}

#[test_case]
fn syscall_exit() {
    let _guard = forbid_frame_leaks();

    let mut process = make_process(EXIT_ELF);
    let pid = process.pid();

    test_scaffolding::disable_interrupts(&mut process);

    Process::enter_user_mode(process);

    assert_eq!(
        TRAP_STATS[INVALID_OPCODE].count(),
        0,
        "probably the `syscall` instruction is not initialized",
    );
    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        0,
        "probably the kernel has not switched to its own stack",
    );

    Table::get(pid).expect_err("the exit() syscall has not freed the process slot");
}

#[test_case]
fn syscall_sched_yield() {
    let _guard = forbid_frame_leaks();

    let mut process = make_process(SCHED_YIELD_ELF);
    let pid = process.pid();

    test_scaffolding::disable_interrupts(&mut process);

    Process::enter_user_mode(process);

    for _ in 0 .. 2 {
        let process = Table::get(pid).expect("failed to find the new process in the process table");

        let user_registers = test_scaffolding::registers(&process);
        debug!(?user_registers, "returned from the user space");

        Process::enter_user_mode(process);
    }

    free_process(pid);
}

#[test_case]
fn user_mode_page_fault() {
    let _guard = forbid_frame_leaks();

    let mut process = make_process(PAGE_FAULT_ELF);
    let pid = process.pid();

    test_scaffolding::disable_interrupts(&mut process);

    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    Process::enter_user_mode(process);

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        1,
        "probably the user mode page faults are not handled or counted",
    );

    Table::get(pid).expect_err("the page fault handler has not freed the process slot");
}

#[test_case]
fn preemption() {
    let _guard = forbid_frame_leaks();

    let pid = make_process(LOOP_ELF).pid();

    let process = Table::get(pid).expect("failed to find the new process in the process table");
    Process::enter_user_mode(process);

    // If this does not happen and the test times out, the preemption is not working properly.
    debug!("returned from the user space");

    free_process(pid);
}

#[test_case]
fn user_context_saved() {
    let _guard = forbid_frame_leaks();

    let pid = make_process(LOOP_ELF).pid();

    let mut user_mode_context_saved = false;

    while !user_mode_context_saved {
        let process = Table::get(pid).expect("failed to find the new process in the process table");
        Process::enter_user_mode(process);

        let process = Table::get(pid).expect("failed to find the new process in the process table");

        let user_registers = test_scaffolding::registers(&process);

        debug!(?user_registers);

        user_mode_context_saved = true;
        for (i, register_value) in user_registers.iter().enumerate() {
            if *register_value != 77701 + i {
                user_mode_context_saved = false;
                break;
            }
        }
    }

    // If this does not happen and the test times out, the user space context was not saved properly.
    debug!("the user context is saved properly");

    free_process(pid);
}

#[test_case]
fn scheduler() {
    let exit_pid = make_process(EXIT_ELF).pid();
    let page_fault_pid = make_process(PAGE_FAULT_ELF).pid();

    Scheduler::enqueue(exit_pid);
    Scheduler::enqueue(page_fault_pid);

    while Scheduler::run_one() {}

    Table::get(exit_pid).expect_err("the 'exit' process was not run up to its completion");
    Table::get(page_fault_pid)
        .expect_err("the 'page_fault' process was not run up to its completion");
}

#[test_case]
fn eager_fork() {
    let start_page_fault_count = TRAP_STATS[PAGE_FAULT].count();

    let pid = make_process(EAGER_FORK_ELF).pid();

    Scheduler::enqueue(pid);

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        start_page_fault_count,
        "eager_fork should not page fault",
    );
}

#[test_case]
fn cow_fork() {
    let start_page_fault_count = TRAP_STATS[PAGE_FAULT].count();

    let pid = make_process(COW_FORK_ELF).pid();

    Scheduler::enqueue(pid);

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[GENERAL_PROTECTION_FAULT].count(),
        0,
        "cow_fork should not trigger the general protection fault",
    );
    assert!(
        TRAP_STATS[PAGE_FAULT].count() + 10 > start_page_fault_count,
        "cow_fork should page fault many times",
    );
}
