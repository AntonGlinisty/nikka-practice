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
    process,
};

use crate::forbid_frame_leaks;

#[test_case]
fn create_process() {
    forbid_frame_leaks!();

    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let elf_file = include_bytes!("../../../user/loop/target/kernel/debug/loop");
    let mut process_address_space =
        process::create("test process", elf_file).expect("failed to create the test process");

    let entry_point =
        Virt::new_u64(ElfFile::new(elf_file).unwrap().header.pt2.entry_point()).unwrap();

    let mapping_error =
        "the ELF file has not been loaded into the address space at the correct address";
    let pte = process_address_space
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
}

#[test_case]
fn create_process_failure() {
    forbid_frame_leaks!();

    let bad_elf_file: &[u8] = &[];
    let error = process::create("bad ELF process", bad_elf_file)
        .expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}
