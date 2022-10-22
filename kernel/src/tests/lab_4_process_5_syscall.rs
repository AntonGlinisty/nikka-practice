use spin::MutexGuard;
use xmas_elf::ElfFile;

use kernel::{
    Subsystems,
    log::info,
    memory::{
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

mod init;

init!(Subsystems::MEMORY | Subsystems::SYSCALL);

const EXIT_ELF: &[u8] = include_bytes!("../../target/kernel/user/exit");
const LOG_VALUE_ELF: &[u8] = include_bytes!("../../target/kernel/user/log_value");

fn make_process(file: &[u8]) -> MutexGuard<'static, Process> {
    test_scaffolding::set_process(
        process::create_process(file).expect("failed to create the test process"),
    );
    let mut process =
        Table::get(Pid::Current).expect("failed to find the new process in the process table");

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

    process
}

#[test_case]
fn syscall_exit() {
    let mut process = make_process(EXIT_ELF);

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
        concat!(
            "if the Page Fault was in the kernel mode, ",
            "probably the `syscall` instruction is not initialized or ",
            "the kernel has not switched to its own stack; ",
            "if it was in the user mode, maybe the time functions from the first lab ",
            "use `read-dont-modify-write` construction",
        ),
    );
}

#[test_case]
fn syscall_log_value() {
    let mut process = make_process(LOG_VALUE_ELF);

    test_scaffolding::disable_interrupts(&mut process);

    Process::enter_user_mode(process);

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        0,
        "the user mode code has detected an error in syscall::log_value() implementation",
    );
}
