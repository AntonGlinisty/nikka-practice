mod elf;
mod pid;
#[allow(clippy::module_inception)]
mod process;
mod registers;
mod syscall;
mod table;

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

use table::TABLE;

pub use pid::Pid;
pub use process::Process;
pub use table::Table;

pub fn init() {
    syscall::init();

    const PROCESS_SLOT_COUNT: usize = 1 << 10;
    *TABLE.lock() = Table::new(PROCESS_SLOT_COUNT);
}

pub fn create(
    name: &'static str,
    elf_file: &[u8],
) -> Result<Pid> {
    let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate(name)?;

    address_space.switch_to();

    let entry = elf::load(&mut address_space, elf_file)?;

    let stack = Stack::new(&mut address_space, USER_RW)?;

    BASE_ADDRESS_SPACE.lock().switch_to();

    let process = Process::new(address_space, entry, stack.pointer());

    info!(%entry, file_size = %Size::from_slice(elf_file), %process, "loaded ELF file");

    Table::allocate(process)
}
