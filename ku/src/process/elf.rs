use core::cmp;

use xmas_elf::{
    ElfFile,
    program::{
        Flags,
        ProgramHeader,
        Type,
    },
};

use crate::{
    error::{
        Error::Elf,
        Result,
    },
    log::debug,
    memory::{
        AddressSpace,
        Block,
        Virt,
        mmu::PageTableFlags,
        size,
    },
};

pub(super) fn load(
    address_space: &mut AddressSpace,
    file: &[u8],
) -> Result<Virt> {
    // TODO: your code here.
    unimplemented!();
}

fn load_program_header(
    address_space: &mut AddressSpace,
    program_header: &ProgramHeader,
    file: &[u8],
    mapped_end: &mut Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

fn extend_mapping(
    address_space: &mut AddressSpace,
    memory_block: &Block<Virt>,
    flags: Flags,
    mapped_end: &mut Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

fn memory_block(program_header: &ProgramHeader) -> Result<Block<Virt>> {
    // TODO: your code here.
    unimplemented!();
}
