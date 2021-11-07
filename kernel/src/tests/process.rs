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
    },
    process::{
        self,
        Pid,
        Process,
        Table,
        test_scaffolding,
    },
    trap::{
        INVALID_OPCODE,
        PAGE_FAULT,
        TRAP_STATS,
    },
};

use crate::forbid_frame_leaks;

const EXIT_ELF: &[u8] = include_bytes!("../../../user/exit/target/kernel/debug/exit");
const LOOP_ELF: &[u8] = include_bytes!("../../../user/loop/target/kernel/debug/loop");
const PAGE_FAULT_ELF: &[u8] =
    include_bytes!("../../../user/page_fault/target/kernel/debug/page_fault");
const SCHED_YIELD_ELF: &[u8] =
    include_bytes!("../../../user/sched_yield/target/kernel/debug/sched_yield");

fn make_process(
    name: &'static str,
    file: &[u8],
) -> MutexGuard<'static, Process> {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let pid = process::create(name, file).expect("failed to create the test process");
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
    forbid_frame_leaks!();

    let pid = make_process("loop", LOOP_ELF).pid();

    free_process(pid);
}

#[test_case]
fn create_process_failure() {
    forbid_frame_leaks!();

    let bad_elf_file: &[u8] = &[];
    let error = process::create("bad ELF process", bad_elf_file)
        .expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}

#[test_case]
fn syscall_exit() {
    forbid_frame_leaks!();

    let mut process = make_process("exit", EXIT_ELF);
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
    forbid_frame_leaks!();

    let mut process = make_process("sched_yield", SCHED_YIELD_ELF);
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
    forbid_frame_leaks!();

    let mut process = make_process("page_fault", PAGE_FAULT_ELF);
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
    forbid_frame_leaks!();

    let pid = make_process("loop", LOOP_ELF).pid();

    for _ in 0 .. 2 {
        let process = Table::get(pid).expect("failed to find the new process in the process table");
        Process::enter_user_mode(process);
    }

    // If this does not happen and the test times out, the preemption is not working properly.
    debug!("returned from the user space");

    {
        let process = Table::get(pid).expect("failed to find the new process in the process table");

        let user_registers = test_scaffolding::registers(&process);

        debug!(?user_registers);

        for i in 0 .. user_registers.len() {
            assert_eq!(user_registers[i], 77701 + i);
        }
    }

    free_process(pid);
}
