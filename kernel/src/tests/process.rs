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
        Process,
    },
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

use crate::forbid_frame_leaks;

fn make_process(
    name: &'static str,
    file: &[u8],
) -> Process {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut process = process::create(name, file).expect("failed to create the test process");

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
fn create_process() {
    forbid_frame_leaks!();

    make_process(
        "loop",
        include_bytes!("../../../user/loop/target/kernel/debug/loop"),
    );
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
fn user_mode_page_fault() {
    forbid_frame_leaks!();

    let mut process = make_process(
        "page_fault",
        include_bytes!("../../../user/page_fault/target/kernel/debug/page_fault"),
    );

    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    process.enter_user_mode();

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        1,
        "probably the user mode page faults are not handled or counted",
    );
}
