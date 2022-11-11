#![allow(dead_code)]

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
        Pid,
        Process,
        Table,
        test_scaffolding,
    },
};

pub(super) fn make(file: &[u8]) -> Process {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut process =
        test_scaffolding::create_process(file).expect("failed to create the test process");

    check(file, &mut process);

    let process_frames = start_free_frames - FRAME_ALLOCATOR.lock().count();
    debug!(process_frames);
    assert!(process_frames > 0, "created process uses no memory");

    process
}

pub(super) fn dummy_allocate(file: &[u8]) -> MutexGuard<'static, Process> {
    test_scaffolding::set_process(
        test_scaffolding::create_process(file).expect("failed to create the test process"),
    );

    let mut process =
        Table::get(Pid::Current).expect("failed to find the new process in the process table");

    check(file, &mut process);

    process
}

fn check(
    file: &[u8],
    process: &mut Process,
) {
    let entry_point = Virt::new_u64(ElfFile::new(file).unwrap().header.pt2.entry_point()).unwrap();

    let mapping_error =
        "the ELF file has not been loaded into the address space at the correct address";
    let pte = process
        .address_space()
        .mapping()
        .translate(entry_point, None, PageTableFlags::empty())
        .expect(mapping_error);

    info!(
        %entry_point,
        frame = ?pte.frame().expect(mapping_error),
        flags = ?pte.flags(),
        "user process page table entry",
    );

    assert!(pte.is_present(), "{}", mapping_error);
    assert!(
        pte.flags().contains(PageTableFlags::USER_ACCESSIBLE),
        "the ELF file is not accessible from the user space",
    );
    assert!(
        !pte.flags().contains(PageTableFlags::NO_EXECUTE),
        "the entry point is not executable",
    );
}
