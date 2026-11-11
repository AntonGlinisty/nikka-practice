use core::{
    alloc::Layout, cmp::{
        self,
        Ordering,
    },
    mem::MaybeUninit,
    ops::Range,
};

use derive_more::Display;
use xmas_elf::{
    ElfFile,
    program::{
        FLAG_R,
        FLAG_W,
        FLAG_X,
        Flags,
        ProgramHeader,
        Type,
    },
};

use crate::{
    allocator::{
        BigAllocator,
        BigAllocatorPair,
    },
    error::{
        Error::{
            Elf,
            InvalidArgument,
            NoPage,
            Overflow,
            PermissionDenied,
        },
        Result,
    },
    log::{
        debug,
        trace,
        warn,
    },
    memory::{
        Block,
        Page,
        Virt,
        block::Memory,
        mmu::PageTableFlags,
        size,
    },
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Загружает [ELF--файл](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format) `file`.
/// Выделяет для него память в целевом адресном пространстве с помощью `allocator.dst()`.
///
/// Использует разновидность алгоритма
/// [сканирующей прямой](https://ru.algorithmica.org/cs/decomposition/scanline/),
/// обрабатывая сегменты --- [`ProgramHeader`] --- загружаемого файла.
/// При этом опирается на то, что в корректном ELF--файле
/// они должны быть отсортированы по своим виртуальным адресам.
/// См.
/// [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html).
///
/// # Safety
///
/// Вызывающая функция должна гарантировать,
/// что `allocator.src()` выделяет память в текущем адресном пространстве.
pub unsafe fn load<T: BigAllocatorPair>(
    allocator: &mut T,
    file: &[u8],
) -> Result<Virt> {
    let mut loader = Loader::new(allocator, file);
    let header = ElfFile::new(file).map_err(Elf)?;

    for program_header in header.program_iter() {
        if program_header.get_type().map_err(Elf)? == Type::Load {
            let file_range = FileRange::try_from(program_header)?;
            loader.load_program_header(file_range)?;
        }
    }

    loader.finish()?;

    Virt::new_u64(header.header.pt2.entry_point())
}

/// Состояние загрузчика ELF--файлов.
struct Loader<'a, T: BigAllocatorPair> {
    /// Пара страничных аллокаторов памяти для работы как с целевым адресным пространством,
    /// куда загружается ELF--файл, так и с текущим адресным пространством.
    allocator: &'a mut T,

    /// Текущий обрабатываемый диапазон памяти загружаемого процесса.
    curr: Option<VirtRange>,

    /// Содержимое [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
    file: &'a [u8],

    /// Текущий блок отображённых на физическую память страниц в текущем адресном пространстве.
    /// В него копируются данные из ELF--файла,
    /// после чего соответствующие физические фреймы переносятся в целевое адресное пространство.
    src_block: Block<Page>,
}

impl<'a, T: BigAllocatorPair> Loader<'a, T> {
    /// Инициализирует состояние загрузчика ELF--файлов.
    fn new(
        allocator: &'a mut T,
        file: &'a [u8],
    ) -> Self {
        Self {
            allocator,
            curr: None,
            file,
            src_block: Block::default(),
        }
    }

    /// Загружает диапазон [`next`][FileRange] со следующим сегментом
    /// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
    ///
    /// - Выделяет для него память с помощью [`Loader::extend_mapping()`].
    /// - Копирует его в текущее адресное пространство с помощью [`Loader::copy_to_memory()`].
    /// - С помощью функции [`combine()`] разбивает диапазоны памяти [`Loader::curr`] и `next`
    ///   на выровненные по границам страниц части,
    ///   про которые стали известны окончательные значения флагов отображения.
    ///   И диапазон памяти, флаги которого ещё могут измениться.
    /// - С помощью метода [`Loader::finalize_mapping()`] переносит страницы в
    ///   целевое адресное пространство с правильными флагами отображения
    ///   той части [`Loader::curr`] и границы между [`Loader::curr`] и `next`,
    ///   про которые стали известны окончательные значения флагов отображения.
    /// - Сохраняет в [`Loader::curr`] часть диапазона `next`,
    ///   флаги которого ещё могут измениться.
    ///   То есть, уже не обязательно соответствующую одному сегменту ELF--файла.
    fn load_program_header(
        &mut self,
        next: FileRange,
    ) -> Result<()> {
        if let Some(curr) = &self.curr {
            validate_order(curr, &next.virt_range)?;
        }
        self.extend_mapping(&next)?;

        let next_range = self.copy_to_memory(next)?;

        if let Some(curr) = self.curr {
            let (curr_minus_next, boundary, update_next) = combine(curr, next_range)?;

            if curr_minus_next.is_some() {
                self.finalize_mapping(curr_minus_next.unwrap())?;
            }

            if boundary.is_some() {
                self.finalize_mapping(boundary.unwrap())?;
            }

            self.curr = Some(update_next);
        } else {
            self.curr = Some(next_range);
        }

        Ok(())
    }

    /// Завершает загрузку ELF--файла в адресное пространство процесса.
    /// После вызова [`Loader::finish()`] в адресное пространство процесса
    /// больше не будут добавляться новые сегменты методом [`Loader::load_program_header()`].
    ///
    /// С помощью метода [`Loader::finalize_mapping()`] переносит оставшиеся страницы из
    /// текущего адресного пространства в целевое для оставшегося диапазона [`Loader::curr`].
    fn finish(&mut self) -> Result<()> {
        if let Some(curr) = self.curr {
            let page_range = PageRange::from(curr);
            self.finalize_mapping(page_range)?;

            self.curr = None;
        }

        Ok(())
    }

    /// Копирует из ELF--файла [`Loader::file`] в текущее адресное пространство
    /// диапазон памяти [`range`][FileRange].
    ///
    /// Диапазон страниц в текущем адресном пространстве, в который осуществляется копирование,
    /// задаётся [`Loader::src_block`].
    fn copy_to_memory(
        &mut self,
        range: FileRange,
    ) -> Result<VirtRange> {
        let pages_count = range.virt_range.memory.enclosing().count();

        let start = (
            (self.src_block.end_address()? - pages_count * Page::SIZE)? +
            range.virt_range.memory.start_address().into_usize() % Page::SIZE)?;
        let end = (start + range.file_range.len())?;

        let block = Block::<Virt>::new(start, end)?;

        let slice_to_fill;
        unsafe {
            slice_to_fill = block.try_into_mut_slice::<u8>()?;
        }

        let source = self.file.get(range.file_range.clone()).ok_or(Overflow)?;
        slice_to_fill.copy_from_slice(source);

        Ok(range.virt_range)
    }

    /// Расширяет отображение целевого адресного пространства,
    /// чтобы гарантировать что блок, который описывает `next`, зарезервирован в памяти.
    ///
    /// - Уже зарезервированную часть памяти определяет по [`Loader::curr`],
    ///   для которого вызывалась ранее. Это должен обеспечить вызывающий метод.
    /// - Резервирует в целевом адресном пространстве виртуальные страницы для `next`
    ///   методом [`BigAllocator::reserve_fixed()`].
    /// - Резервирует, отображает в физическую память методом [`BigAllocator::map()`]
    ///   и зануляет с помощью [`slice::write_filled()`] необходимое количество страниц
    ///   в текущем адресном пространстве.
    ///   В них позже будут временно скопированы данные из ELF--файла.
    ///   После чего они будут перенесены в зарезервированную область
    ///   целевого адресного пространства.
    /// - Флаги страниц в текущем адресном пространстве должны быть выставлены так,
    ///   чтобы в эту память мог записать метод
    ///   [`Loader::copy_to_memory()`].
    ///   Кроме нужных для копирования флагов требуется также указывать
    ///   флаги аллокатора [`Loader::allocator.src().flags()`][BigAllocator::flags]
    ///   текущего адресного пространства.
    ///   Среди них, например, может быть [`PageTableFlags::USER`].
    fn extend_mapping(
        &mut self,
        next: &FileRange,
    ) -> Result<()> {
        let block_to_allocate;
        let requested = next.virt_range.memory.enclosing();

        if self.curr.is_some() {
            let already_allocated = self.curr.unwrap().memory.enclosing();

            block_to_allocate = Block::<Page>::from_index(
                core::cmp::max(requested.start(), already_allocated.end()),
                requested.end(),
            )?;
        } else {
            block_to_allocate = requested;
        }

        let flags: PageTableFlags = self.allocator.src().flags() | PageTableFlags::WRITABLE;

        unsafe {
            self.src_block = self.allocator.src().grow(
                self.src_block,
                Layout::from_size_align(
                    self.src_block.size().checked_add(block_to_allocate.size()).ok_or(Overflow)?,
                    Page::SIZE
                )?,
                flags
            )?;
        }

        let block_to_write = Block::<Page>::from_index(
            self.src_block.end() - block_to_allocate.count(),
            self.src_block.end()
        )?;

        unsafe {
            block_to_write.try_into_mut_slice()?.fill(0);
        }

        self.allocator.dst().reserve_fixed(block_to_allocate)?;

        Ok(())
    }

    /// Переносит заполненные физические фреймы для целевого диапазона
    /// [`page_range`][PageRange] из текущего адресного пространства в целевое
    /// с окончательными флагами.
    ///
    /// Кроме заданных в ELF--файле флагов требуется также указывать
    /// флаги аллокатора [`Loader::allocator.dst().flags()`][BigAllocator::flags].
    /// Среди них, например, может быть [`PageTableFlags::USER`],
    /// которого нет во флагах ELF--файла.
    fn finalize_mapping(
        &mut self,
        page_range: PageRange,
    ) -> Result<()> {
        let page_count = page_range.memory.count();
        if page_count > self.src_block.count() {
            return Err(InvalidArgument)
        }

        let start = self.src_block.start_address();
        let end = (self.src_block.start_address() + page_count * Page::SIZE)?;

        let block = Block::<Page>::new(
            Page::containing(start),
            Page::containing(end)
        )?;

        let flags = self.allocator.dst().flags() | page_range.flags;

        unsafe {
            self.allocator.copy_mapping(block, page_range.memory, Some(flags))?;
            self.allocator.src().unmap(block)?;
            self.allocator.src().unreserve(block)?;
        }

        self.src_block = Block::<Page>::from_index(
            block.end(),
            self.src_block.end(),
        )?;

        Ok(())
    }
}

impl<'a, T: BigAllocatorPair> Drop for Loader<'a, T> {
    fn drop(&mut self) {
        if self.src_block.is_empty() {
            return
        }

        unsafe {
            self.allocator.src().unmap(self.src_block).expect("Failed to unmap src_block in drop");
            self.allocator.src().unreserve(self.src_block).expect("Failed to unreserve src_block in drop");
        }
    }
}

/// Диапазон памяти процесса с заданными флагами доступа.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
#[display("{{ memory: {}, flags: {} }}", memory, flags)]
struct MemoryRange<T: Memory<Address = Virt>> {
    /// Флаги, с которыми должен быть отображён этот диапазон памяти процесса.
    flags: PageTableFlags,

    /// Диапазон памяти процесса, который нужно отобразить в адресное пространство процесса.
    memory: Block<T>,
}

/// Не выровненный по границам страниц диапазон памяти,
/// который нужно отобразить в адресное пространство процесса.
type VirtRange = MemoryRange<Virt>;

/// Выровненный по границам страниц диапазон памяти,
/// который нужно отобразить в адресное пространство процесса.
type PageRange = MemoryRange<Page>;

impl From<VirtRange> for PageRange {
    /// Возвращает выровненный по границам страниц диапазон памяти [`PageRange`],
    /// содержащий [`self`][VirtRange] и имеющий те же флаги отображения страниц.
    fn from(virt_range: VirtRange) -> Self {
        Self {
            flags: virt_range.flags,
            memory: virt_range.memory.enclosing(),
        }
    }
}

/// Не выровненный по границам страниц диапазон памяти,
/// который нужно скопировать из ELF--файла в адресное пространство процесса.
///
/// Так как этот диапазон не выровнен по границам страниц,
/// он ещё не готов к отображению в память с финальными флагами,
/// которые диктует ELF--файл.
///
/// Создаётся по [`ProgramHeader`] в методе [`FileRange::try_from()`].
/// То есть, изначально соответствует одному загружаемому сегменту ELF--файла.
/// Но в процессе загрузки ELF--файла может быть как объединён со смежными сегментами,
/// так и разделён на части типа [`PageRange`] и [`VirtRange`].
/// См. функцию [`combine()`].
#[derive(Debug, Display, Eq, PartialEq)]
#[display("{{ {}, file_range: {:#X?} }}", virt_range, file_range)]
struct FileRange {
    /// Диапазон байт файла, которые нужно скопировать в память процесса.
    file_range: Range<usize>,

    /// Не выровненный по границам страниц диапазон памяти,
    /// который нужно отобразить в адресное пространство процесса.
    virt_range: VirtRange,
}

impl TryFrom<ProgramHeader<'_>> for FileRange {
    type Error = Error;

    /// Преобразует сегмент [`program_header`][ProgramHeader]
    /// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
    /// в соответствующий ему описатель блока памяти,
    /// которую нужно скопировать в адресное пространство процесса.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::InvalidArgument`] если какие-то адреса в сегменте не являются
    ///     [каноническими](https://en.wikipedia.org/wiki/X86-64#Canonical_form_addresses).
    ///   - [`Error::Overflow`] если:
    ///     - Диапазон в памяти не корректен или меньше чем диапазон в файле, см.
    ///       [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html).
    ///     - При вычислении диапазона [`FileRange::file_range`],
    ///       отвечающего содержимому сегмента в ELF--файле,
    ///       возникает переполнение.
    ///   - [`Error::PermissionDenied`] если входные флаги содержат одновременно
    ///     [`FLAG_W`] и [`FLAG_X`] --- то есть открывают возможность атакующему
    ///     [записать и выполнить произвольный код](https://en.wikipedia.org/wiki/Arbitrary_code_execution)
    ///     внутри создаваемого процесса.
    fn try_from(program_header: ProgramHeader) -> Result<Self> {
        if program_header.mem_size() < program_header.file_size() {
            return Err(Overflow)
        }

        let file_offset = usize::try_from(program_header.offset()).map_err(|_| Overflow)?;
        let file_size = usize::try_from(program_header.file_size()).map_err(|_| Overflow)?;

        let file_range = file_offset..file_offset.checked_add(file_size).ok_or(Overflow)?;
        let virt_range = VirtRange {
            flags: PageTableFlags::try_from(program_header.flags())?,
            memory: Block::<Virt>::new(
                Virt::new_u64(program_header.virtual_addr())?,
                Virt::new_u64(
                    program_header
                        .virtual_addr()
                        .checked_add(program_header.mem_size())
                        .ok_or(Overflow)?
                    )?,
            ).map_err(|_| Overflow)?,
        };
        
        Ok(FileRange { file_range, virt_range })
    }
}

/// Объединяет не выровненные по границам страниц
/// текущий обрабатываемый диапазон [`curr`][VirtRange] и
/// следующий за ним диапазон [`next`][VirtRange].
///
/// Требует, чтобы `curr` лежал левее `next` в диапазоне адресов.
/// А также, чтобы они не пересекались.
/// См.
/// [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html)
/// и функцию [`validate_order()`].
/// В противном случае, возвращает ошибку [`Error::InvalidArgument`].
///
/// Возвращает `(curr_minus_next, boundary, updated_next)`, где:
///   - `curr_minus_next` --- выровненная по границам страниц максимальная часть `curr`,
///     которая не пересекается с `next` и последующими сегментами ELF--файла.
///     То есть, `curr_minus_next` гарантированно не попадает в те же страницы,
///     в которые попадает `next` или могут попасть последующие сегменты ELF--файла.
///     Про неё уже известны окончательные флаги отображения, и они совпадают с флагами `curr`.
///     Может быть равна [`None`].
///   - `boundary` --- пограничная страница, в которую попадает как `curr`,
///     так и `next`. Но при этом не могут попасть никакие последующие сегменты ELF--файла.
///     Флаги в ней должны быть подходящими как для диапазона `curr`, так и для диапазона `next`.
///     Поэтому они могут отличаться от флагов `curr_minus_next` и поэтому эта пограничная
///     страница не может быть обработана в составе `curr_minus_next`.
///     Часть `boundary` может содержать только одну страницу памяти.
///     Или же она равна [`None`], если никакие части `curr` и `next` не попадают в одну и ту же
///     страницу, либо в эту же страницу потенциально могут попасть последующие сегменты ELF--файла.
///   - `updated_next` содержит части `curr` и `next`,
///     которые не попали в `curr_minus_next` и `boundary`.
///     Флаги в ней должны быть подходящими для диапазона `next`.
///     В случае, если `curr` задевает те же страницы,
///     флаги `updated_next` должны быть подходящими и для `curr`.
///     В этом случае `updated_next` не может задевать больше одной страницы.
fn combine(
    curr: VirtRange,
    next: VirtRange,
) -> Result<(Option<PageRange>, Option<PageRange>, VirtRange)> {
    validate_order(&curr, &next)?;

    let curr_pages = curr.memory.enclosing();
    let next_pages = next.memory.enclosing();

    let curr_minus_next_block = Block::<Page>::from_index(
        curr_pages.start(),
        core::cmp::min(curr_pages.end(), next_pages.start()),
    )?;

    let curr_minus_next = if curr_minus_next_block.is_empty() {
        None
    } else {
        Some(PageRange { flags: curr.flags, memory: curr_minus_next_block })
    };

    let boundary_block;
    let boundary_flags = curr.flags | next.flags;

    let updated_next_block;
    let mut updated_next_flags = next.flags;

    if curr_pages.end() <= next_pages.start() {
        boundary_block = None;
        updated_next_block = next.memory;
    } else if curr_pages.end() < next_pages.end() {
        boundary_block = Some(Block::<Page>::from_index(
            next_pages.start(),
            curr_pages.end()
        )?);

        updated_next_block = Block::<Virt>::new(
            curr_pages.end_address()?,
            next.memory.end_address()?
        )?;
    } else {
        boundary_block = None;

        updated_next_block = Block::<Virt>::new(
            core::cmp::max((next_pages.end_address()? - Page::SIZE)?, curr.memory.start_address()),
            next.memory.end_address()?
        )?;
        updated_next_flags = curr.flags | next.flags;
    }

    let boundary: Option<PageRange>;
    if let Some(block) = boundary_block {
        boundary = Some(PageRange {
            flags: boundary_flags,
            memory: block,
        });
    } else {
        boundary = None
    }

    let updated_next = VirtRange {
        flags: updated_next_flags,
        memory: updated_next_block,
    };

    Ok((curr_minus_next, boundary, updated_next))
}

impl TryFrom<Flags> for PageTableFlags {
    type Error = Error;

    /// Переводит флаги доступа сегментов ELF--файла в соответствующие [`PageTableFlags`].
    ///
    /// Возвращает ошибку [`Error::PermissionDenied`] если:
    ///   - Входные флаги содержат одновременно [`FLAG_W`] и [`FLAG_X`] ---
    ///     то есть открывают возможность атакующему
    ///     [записать и выполнить произвольный код](https://en.wikipedia.org/wiki/Arbitrary_code_execution)
    ///     внутри создаваемого процесса.
    fn try_from(ph_flags: Flags) -> Result<Self> {
        let Flags(ph_flags) = ph_flags;

        let insecure_flags = FLAG_W | FLAG_X;
        if ph_flags & insecure_flags == insecure_flags {
            return Err(PermissionDenied);
        }

        let mut flags = PageTableFlags::PRESENT;

        for (ph_flag, flag) in [
            (FLAG_R, PageTableFlags::default()),
            (FLAG_W, PageTableFlags::WRITABLE),
            (FLAG_X, PageTableFlags::EXECUTABLE),
        ] {
            if ph_flags & ph_flag != 0 {
                flags ^= flag;
            }
        }

        Ok(flags)
    }
}

/// Требует чтобы [`curr`][VirtRange] лежал левее [`next`][VirtRange] в диапазоне адресов.
/// А также, чтобы они не пересекались.
/// См.
/// [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html).
/// В противном случае, возвращает ошибку [`Error::InvalidArgument`].
fn validate_order(
    curr: &VirtRange,
    next: &VirtRange,
) -> Result<()> {
    if curr.memory.partial_cmp(&next.memory) != Some(Ordering::Less) {
        warn!(
            curr_program_header = %curr.memory,
            next_program_header = %next.memory,
            "ELF loadable program headers intersect or are out of order by their virtual addresses",
        );
        Err(InvalidArgument)
    } else {
        Ok(())
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use core::ops::Range;

    use derive_more::Display;
    use serde::{
        Deserialize,
        Serialize,
    };
    use xmas_elf::program::{
        ProgramHeader,
        ProgramHeader64,
    };

    use crate::{
        allocator::BigAllocatorPair,
        error::Result,
        memory::{
            Block,
            Page,
            Virt,
            block::Memory,
            mmu::PageTableFlags,
        },
    };

    pub struct Loader<'a, T: BigAllocatorPair>(super::Loader<'a, T>);

    impl<'a, T: BigAllocatorPair> Loader<'a, T> {
        pub fn new(
            allocator: &'a mut T,
            file: &'a [u8],
        ) -> Self {
            Self(super::Loader::new(allocator, file))
        }

        pub fn load_program_header(
            &mut self,
            next: FileRange,
        ) -> Result<()> {
            let next = next.into();
            self.0.load_program_header(next)
        }

        pub fn finish(&mut self) -> Result<()> {
            self.0.finish()
        }

        pub fn extend_mapping(
            &mut self,
            next: &FileRange,
        ) -> Result<()> {
            let next = next.clone().into();
            self.0.extend_mapping(&next)
        }

        pub fn finalize_mapping(
            &mut self,
            page_range: PageRange,
        ) -> Result<()> {
            self.0.finalize_mapping(page_range.into())
        }
    }

    #[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, PartialEq, Serialize)]
    #[display("{{ memory: {}, flags: {} }}", memory, flags)]
    pub struct MemoryRange<T: Memory<Address = Virt>> {
        pub flags: PageTableFlags,
        pub memory: Block<T>,
    }

    impl<T: Memory<Address = Virt>> MemoryRange<T> {
        pub fn new(
            memory: Block<T>,
            flags: PageTableFlags,
        ) -> Self {
            Self { flags, memory }
        }

        pub fn start_address(&self) -> Virt {
            self.memory.start_address()
        }

        pub fn end_address(&self) -> Result<Virt> {
            self.memory.end_address()
        }
    }

    impl<T: Memory<Address = Virt>> From<MemoryRange<T>> for super::MemoryRange<T> {
        fn from(memory_range: MemoryRange<T>) -> Self {
            Self {
                flags: memory_range.flags,
                memory: memory_range.memory,
            }
        }
    }

    impl<T: Memory<Address = Virt>> From<super::MemoryRange<T>> for MemoryRange<T> {
        fn from(memory_range: super::MemoryRange<T>) -> Self {
            Self {
                flags: memory_range.flags,
                memory: memory_range.memory,
            }
        }
    }

    pub type VirtRange = MemoryRange<Virt>;

    pub type PageRange = MemoryRange<Page>;

    impl From<VirtRange> for PageRange {
        fn from(virt_range: VirtRange) -> Self {
            super::PageRange::from(super::VirtRange::from(virt_range)).into()
        }
    }

    #[derive(Clone, Debug, Default, Display, Eq, PartialEq)]
    #[display("{{ {}, file_range: {:#X?} }}", virt_range, file_range)]
    pub struct FileRange {
        pub file_range: Range<usize>,
        pub virt_range: VirtRange,
    }

    impl FileRange {
        pub fn new(
            memory: Block<Virt>,
            flags: PageTableFlags,
            file_range: Range<usize>,
        ) -> Self {
            Self {
                file_range,
                virt_range: VirtRange::new(memory, flags),
            }
        }
    }

    impl From<FileRange> for super::FileRange {
        fn from(file_range: FileRange) -> Self {
            Self {
                file_range: file_range.file_range,
                virt_range: super::VirtRange {
                    flags: file_range.virt_range.flags,
                    memory: file_range.virt_range.memory,
                },
            }
        }
    }

    impl From<super::FileRange> for FileRange {
        fn from(file_range: super::FileRange) -> Self {
            Self {
                virt_range: VirtRange::new(
                    file_range.virt_range.memory,
                    file_range.virt_range.flags,
                ),
                file_range: file_range.file_range,
            }
        }
    }

    pub fn program_header_to_file_range(program_header: &ProgramHeader64) -> Result<FileRange> {
        Ok(super::FileRange::try_from(ProgramHeader::Ph64(program_header))?.into())
    }

    pub fn combine(
        curr: VirtRange,
        next: VirtRange,
    ) -> Result<(Option<PageRange>, Option<PageRange>, VirtRange)> {
        let (curr_minus_next, boundary, updated_next) = super::combine(curr.into(), next.into())?;

        Ok((
            curr_minus_next.map(|x| x.into()),
            boundary.map(|x| x.into()),
            updated_next.into(),
        ))
    }
}
