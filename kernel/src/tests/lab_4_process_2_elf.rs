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
        test_scaffolding::forbid_frame_leaks,
    },
    process::{
        self,
        Process,
        test_scaffolding,
    },
};

mod init;

init!(Subsystems::MEMORY);

const LOOP_ELF: &[u8] = include_bytes!("../../target/kernel/user/loop");

fn make_process(file: &[u8]) -> Process {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut process =
        test_scaffolding::create_process(file).expect("failed to create the test process");

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

    let process_frames = start_free_frames - FRAME_ALLOCATOR.lock().count();
    debug!(process_frames);
    assert!(process_frames > 0, "created process uses no memory");

    process
}

#[test_case]
fn create_process() {
    let _guard = forbid_frame_leaks();

    make_process(LOOP_ELF);
}

#[test_case]
fn create_process_failure() {
    let _guard = forbid_frame_leaks();

    let bad_elf_file: &[u8] = &[];
    let error = process::create(bad_elf_file).expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}
