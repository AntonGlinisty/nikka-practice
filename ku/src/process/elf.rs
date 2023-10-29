use core::cmp;

use xmas_elf::{
    ElfFile,
    program::{
        ProgramHeader,
        Type,
    },
};

use crate::{
    allocator::BigAllocator,
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
/// `file`.
/// Выделяет для него память с помощью `allocator`.
///
/// Вызывающая её функция должна гарантировать,
/// что `allocator` выделяет память в текущем адресном пространстве.
pub fn load<T: BigAllocator>(
    allocator: &mut T,
    file: &[u8],
) -> Result<Virt> {
    // TODO: your code here.
    unimplemented!();
}

/// Загружает сегмент `program_header`
/// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
/// Выделяет для него память с помощью `allocator`.
/// В аргументе `mapped_end` поддерживает адрес до которого (не включительно)
/// память для загружаемого процесса уже аллоцирована.
///
/// Вызывающая её функция должна гарантировать,
/// что `allocator` выделяет память в текущем адресном пространстве.
fn load_program_header<T: BigAllocator>(
    allocator: &mut T,
    program_header: &ProgramHeader,
    file: &[u8],
    mapped_end: &mut Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

/// Расширяет отображение текущего адресного пространства,
/// чтобы гарантировать что `memory_block` отображён в память.
/// Выделяет для него память с помощью `allocator`.
/// В аргументе `mapped_end` поддерживает адрес до которого (не включительно)
/// память для загружаемого процесса уже аллоцирована.
/// Новые страницы отображения заполняет нулями.
///
/// Вызывающая её функция должна гарантировать,
/// что `allocator` выделяет память в текущем адресном пространстве.
fn extend_mapping<T: BigAllocator>(
    allocator: &mut T,
    memory_block: &Block<Virt>,
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
