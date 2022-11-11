use kernel::{
    Subsystems,
    log::info,
    memory::test_scaffolding::forbid_frame_leaks,
    process,
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY);

const LOOP_ELF: &[u8] = include_bytes!("../../target/kernel/user/loop");

#[test_case]
fn create_process() {
    let _guard = forbid_frame_leaks();

    process_helpers::make(LOOP_ELF);
}

#[test_case]
fn create_process_failure() {
    let _guard = forbid_frame_leaks();

    let bad_elf_file: &[u8] = &[];
    let error = process::create(bad_elf_file).expect_err("created a process from a bad ELF file");

    info!(?error, "expected a process creation failure");
}
