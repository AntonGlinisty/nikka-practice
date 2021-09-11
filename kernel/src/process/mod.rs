mod elf;

use crate::{
    error::Result,
    log::info,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        Size,
    },
};

pub fn create(
    name: &'static str,
    elf_file: &[u8],
) -> Result<AddressSpace> {
    let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate(name)?;

    address_space.switch_to();

    let entry = elf::load(&mut address_space, elf_file)?;

    BASE_ADDRESS_SPACE.lock().switch_to();

    info!(%entry, file_size = %Size::from_slice(elf_file), "loaded ELF file");

    Ok(address_space)
}
