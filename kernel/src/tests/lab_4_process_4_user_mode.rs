use spin::Mutex;
use xmas_elf::ElfFile;

use ku::process::registers::RFlags;

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
    trap::{
        PAGE_FAULT,
        TRAP_STATS,
    },
};

mod init;

init!(Subsystems::MEMORY);

const PAGE_FAULT_ELF: &[u8] = include_bytes!("../../target/kernel/user/page_fault");

fn make_process(file: &[u8]) -> Process {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut process = process::create_process(file).expect("failed to create the test process");

    let entry_point = Virt::new_u64(ElfFile::new(file).unwrap().header.pt2.entry_point()).unwrap();

    let mapping_error =
        "the ELF file has not been loaded into the address space at the correct address";
    let pte = process
        .address_space()
        .mapping()
        .translate(entry_point, None, PageTableFlags::empty())
        .expect(mapping_error)
        .clone();

    info!(%entry_point, frame = ?pte.frame().expect(mapping_error), flags = ?pte.flags(), "user process page table entry");

    assert!(pte.is_present(), "{}", mapping_error);
    assert!(
        pte.flags().contains(PageTableFlags::USER_ACCESSIBLE),
        "the ELF file is not accessible from the user space",
    );

    let process_frames = start_free_frames - FRAME_ALLOCATOR.lock().count();
    debug!(process_frames);
    assert!(process_frames > 0, "created process uses no memory");

    process
}

#[test_case]
fn user_mode_page_fault() {
    let _guard = forbid_frame_leaks();

    let process = Mutex::new(make_process(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    assert_eq!(TRAP_STATS[PAGE_FAULT].count(), 0);

    Process::enter_user_mode(process.lock());

    assert_eq!(
        TRAP_STATS[PAGE_FAULT].count(),
        1,
        "probably the user mode page faults are not handled or counted",
    );
}

#[test_case]
fn user_context_saved() {
    let _guard = forbid_frame_leaks();

    let process = Mutex::new(make_process(PAGE_FAULT_ELF));

    test_scaffolding::disable_interrupts(&mut process.lock());

    Process::enter_user_mode(process.lock());

    let user_registers = test_scaffolding::registers(&process.lock());
    debug!(?user_registers);
    let user_mode_context_saved = user_registers
        .into_iter()
        .enumerate()
        .all(|(i, register_value)| register_value == 77701 + i);
    assert!(user_mode_context_saved);

    assert!(
        RFlags::read().contains(RFlags::INTERRUPT_FLAG),
        "enable the interrupts after the final return to the kernel stack",
    );
}
