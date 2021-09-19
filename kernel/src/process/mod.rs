mod elf;
#[allow(clippy::module_inception)]
mod process;
mod registers;
mod syscall;

use crate::{
    error::Result,
    log::info,
    memory::{
        BASE_ADDRESS_SPACE,
        Size,
        Stack,
        USER_RW,
    },
};

pub use process::Process;

pub fn init() {
    syscall::init();
}

pub fn create(
    name: &'static str,
    elf_file: &[u8],
) -> Result<Process> {
    let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate(name)?;

    address_space.switch_to();

    let entry = elf::load(&mut address_space, elf_file)?;

    let stack = Stack::new(&mut address_space, USER_RW)?;

    BASE_ADDRESS_SPACE.lock().switch_to();

    let process = Process::new(address_space, entry, stack.pointer());

    info!(%entry, file_size = %Size::from_slice(elf_file), %process, "loaded ELF file");

    Ok(process)
}
