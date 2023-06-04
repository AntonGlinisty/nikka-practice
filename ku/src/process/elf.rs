use core::cmp;

use xmas_elf::{
    ElfFile,
    program::{
        Flags,
        ProgramHeader,
        Type,
    },
};

use ku::allocator::BigAllocator;

use crate::{
    error::{
        Error::Elf,
        Result,
    },
    log::debug,
    memory::{
        Block,
        Virt,
        size,
    },
};

/// Загружает [ELF--файл](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// `file` в адресное пространство `address_space`.
///
/// Вызывающая её функция должна гарантировать,
/// что `address_space` является текущим адресным пространством.
pub(super) fn load(
    address_space: &mut dyn BigAllocator,
    file: &[u8],
) -> Result<Virt> {
    // TODO: your code here.
    unimplemented!();
}

/// Загружает сегмент `program_header`
/// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// в адресное пространство `address_space`.
/// В аргументе `mapped_end` поддерживает адрес до которого (не включительно)
/// память для загружаемого процесса уже аллоцирована.
///
/// Вызывающая её функция должна гарантировать,
/// что `address_space` является текущим адресным пространством.
fn load_program_header(
    address_space: &mut dyn BigAllocator,
    program_header: &ProgramHeader,
    file: &[u8],
    mapped_end: &mut Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

/// Расширяет отображение адресного пространства `address_space`,
/// чтобы гарантировать что `memory_block` отображён в память с доступами,
/// которые соответствуют флагам из ELF--файла `flags`.
/// В аргументе `mapped_end` поддерживает адрес до которого (не включительно)
/// память для загружаемого процесса уже аллоцирована.
/// Новые страницы отображения заполняет нулями.
///
/// Вызывающая её функция должна гарантировать,
/// что `address_space` является текущим адресным пространством.
fn extend_mapping(
    address_space: &mut dyn BigAllocator,
    memory_block: &Block<Virt>,
    flags: Flags,
    mapped_end: &mut Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

/// Для сегмента `program_header`
/// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// возвращает соответствующий ему описатель блока памяти.
fn memory_block(program_header: &ProgramHeader) -> Result<Block<Virt>> {
    // TODO: your code here.
    unimplemented!();
}
