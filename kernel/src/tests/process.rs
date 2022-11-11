use spin::MutexGuard;
use xmas_elf::ElfFile;

use kernel::{
    Subsystems,
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
        Process,
        Scheduler,
        Table,
    },
    trap::{
        GENERAL_PROTECTION_FAULT,
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const COW_FORK_ELF: &[u8] = include_bytes!("../../target/kernel/user/cow_fork");
const EAGER_FORK_ELF: &[u8] = include_bytes!("../../target/kernel/user/eager_fork");

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
