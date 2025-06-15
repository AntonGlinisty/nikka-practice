use core::{
    cmp::{
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
    allocator::BigAllocator,
    error::{
        Error::{
            Elf,
            InvalidArgument,
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
        KERNEL_RW,
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

// ANCHOR: load
/// Загружает [ELF--файл](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format) `file`.
/// Выделяет для него память с помощью `allocator`.
///
/// Вызывающая функция должна гарантировать,
/// что `allocator` выделяет память в текущем адресном пространстве.
///
/// Использует разновидность алгоритма
/// [сканирующей прямой](https://ru.algorithmica.org/cs/decomposition/scanline/),
/// обрабатывая сегменты --- [`ProgramHeader`] --- загружаемого файла.
/// При этом опирается на то, что в корректном ELF--файле
/// они должны быть отсортированы по своим виртуальным адресам.
/// См.
/// [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html).
pub fn load<T: BigAllocator>(
    allocator: &mut T,
    file: &[u8],
) -> Result<Virt> {
    // ANCHOR_END: load
    // TODO: your code here.
    unimplemented!();
}

// ANCHOR: loader
/// Состояние загрузчика ELF--файлов.
struct Loader<'a, T: BigAllocator> {
    /// Страничный аллокатор памяти.
    ///
    /// Выделяет память в текущем адресном пространстве,
    /// которое совпадает с адресным пространством загружаемого процесса.
    allocator: &'a mut T,

    /// Содержимое [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
    file: &'a [u8],
}
// ANCHOR_END: loader

impl<'a, T: BigAllocator> Loader<'a, T> {
    /// Инициализирует состояние загрузчика ELF--файлов.
    fn new(
        allocator: &'a mut T,
        file: &'a [u8],
    ) -> Self {
        Self { allocator, file }
    }

    // ANCHOR: load_program_header
    /// Загружает диапазон [`next`][FileRange] со следующим сегментом
    /// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
    ///
    /// - Выделяет для него память с помощью [`Loader::extend_mapping()`].
    /// - Копирует его в память с помощью [`FileRange::copy_to_memory()`].
    /// - С помощью метода [`Loader::process_boundary()`] исправляет флаги отображения
    ///   для той части `curr` и границы между `curr` и `next`,
    ///   про которые стали известны окончательные значения флагов отображения.
    /// - Возвращает часть диапазона `next`, уже скопированную в память,
    ///   но ещё не отображённую с правильными флагами.
    ///   То есть, уже не обязательно соответствующую одному сегменту ELF--файла.
    fn load_program_header(
        &mut self,
        curr: Option<VirtRange>,
        next: FileRange,
    ) -> Result<VirtRange> {
        // ANCHOR_END: load_program_header
        // TODO: your code here.
        unimplemented!();
    }

    // ANCHOR: extend_mapping
    /// Расширяет отображение текущего адресного пространства,
    /// чтобы гарантировать что блок, который описывает `next`, отображён в память.
    ///
    /// - Уже отображённую часть памяти определяет по `curr`,
    ///   для которого вызывалась ранее. Это должен обеспечить вызывающий метод.
    /// - Выделяет для `next` память с помощью [`Loader::allocator`].
    ///   Для этого резервирует виртуальные страницы методом [`BigAllocator::reserve_fixed()`].
    ///   А после этого отображает в физическую память методом [`BigAllocator::map()`].
    /// - Флаги страниц должны быть выставлены так, чтобы в эту память мог записать метод
    ///   [`FileRange::copy_to_memory()`], см. [`Loader::initial_flags()`].
    ///   Эти флаги могут не совпадать с флагами [`VirtRange::flags`],
    ///   с которыми этот диапазон должен быть отображён в память процесса.
    ///   Последние проставит в дальнейшем метод [`Loader::finalize_mapping()`].
    /// - Кроме нужных для копирования флагов требуется также указывать
    ///   флаги аллокатора [`Loader::allocator.flags()`][BigAllocator::flags].
    ///   Среди них, например, может быть [`PageTableFlags::USER`],
    ///   которого нет во флагах ELF--файла.
    /// - Новые страницы отображения заполняет нулями с помощью [`slice::write_filled()`].
    fn extend_mapping(
        &mut self,
        curr: &Option<VirtRange>,
        next: &FileRange,
    ) -> Result<()> {
        // ANCHOR_END: extend_mapping
        // TODO: your code here.
        unimplemented!();
    }

    // ANCHOR: process_boundary
    /// Обрабатывает очередную границу между диапазонами памяти `curr` и `next`.
    ///
    /// - С помощью функции [`combine()`] разбивает диапазоны памяти `curr` и `next`
    ///   на выровненные по границам страниц части,
    ///   про которые стали известны окончательные значения флагов отображения.
    ///   И диапазон памяти, флаги которого ещё могут измениться.
    /// - С помощью метода [`Loader::finalize_mapping()`] исправляет флаги отображения
    ///   для той части `curr` и границы между `curr` и `next`,
    ///   про которые стали известны окончательные значения флагов отображения.
    /// - Возвращает диапазон памяти, флаги которого ещё могут измениться.
    fn process_boundary(
        &mut self,
        curr: VirtRange,
        next: VirtRange,
    ) -> Result<VirtRange> {
        // ANCHOR_END: process_boundary
        // TODO: your code here.
        unimplemented!();
    }

    // ANCHOR: finalize_mapping
    /// Исправляет флаги отображения диапазона [`page_range`][PageRange].
    ///
    /// Флаги требуется исправить только если они отличаются от
    /// использованных изначально для копирования в память содержимого из ELF--файла.
    /// Изначальные флаги выставил ранее метод [`Loader::extend_mapping()`].
    ///
    /// Кроме нужных для копирования флагов требуется также указывать
    /// флаги аллокатора [`Loader::allocator.flags()`][BigAllocator::flags].
    /// Среди них, например, может быть [`PageTableFlags::USER`],
    /// которого нет во флагах ELF--файла.
    fn finalize_mapping(
        &mut self,
        page_range: PageRange,
    ) -> Result<()> {
        // ANCHOR_END: finalize_mapping
        // TODO: your code here.
        unimplemented!();
    }

    /// Флаги, используемые изначально для копирования в память содержимого из ELF--файла,
    /// которые выставляет метод [`Loader::extend_mapping()`].
    fn initial_flags(&self) -> PageTableFlags {
        Self::INITIAL_FLAGS | self.allocator.flags()
    }

    /// Флаги, используемые изначально для копирования в память содержимого из ELF--файла,
    /// которые выставляет метод [`Loader::extend_mapping()`].
    const INITIAL_FLAGS: PageTableFlags = KERNEL_RW;
}

// ANCHOR: memory_range
/// Диапазон памяти процесса с заданными флагами доступа.
#[derive(Debug, Display, Eq, PartialEq)]
#[display("{{ memory: {}, flags: {} }}", memory, flags)]
struct MemoryRange<T: Memory<Address = Virt>> {
    /// Флаги, с которыми должен быть отображён этот диапазон памяти процесса.
    flags: PageTableFlags,

    /// Диапазон памяти процесса, который нужно отобразить в адресное пространство процесса.
    memory: Block<T>,
}
// ANCHOR_END: memory_range

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

// ANCHOR: file_range
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
// ANCHOR_END: file_range

impl FileRange {
    // ANCHOR: copy_to_memory
    /// Копирует из ELF--файла `file` в адресное пространство процесса
    /// диапазон памяти [`self`][FileRange].
    fn copy_to_memory(
        self,
        file: &[u8],
    ) -> Result<VirtRange> {
        // ANCHOR_END: copy_to_memory
        // TODO: your code here.
        unimplemented!();
    }
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
        // TODO: your code here.
        unimplemented!();
    }
}

// ANCHOR: combine
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
    // ANCHOR_END: combine
    // TODO: your code here.
    unimplemented!();
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
        allocator::BigAllocator,
        error::Result,
        memory::{
            Block,
            Page,
            Virt,
            block::Memory,
            mmu::PageTableFlags,
        },
    };

    pub struct Loader<'a, T: BigAllocator>(super::Loader<'a, T>);

    impl<'a, T: BigAllocator> Loader<'a, T> {
        pub fn new(
            allocator: &'a mut T,
            file: &'a [u8],
        ) -> Self {
            Self(super::Loader::new(allocator, file))
        }

        pub fn load_program_header(
            &mut self,
            curr: Option<VirtRange>,
            next: FileRange,
        ) -> Result<VirtRange> {
            let curr = curr.map(|x| x.into());
            let next = next.into();
            self.0.load_program_header(curr, next).map(|x| x.into())
        }

        pub fn extend_mapping(
            &mut self,
            curr: &Option<VirtRange>,
            next: &FileRange,
        ) -> Result<()> {
            let curr = curr.map(|x| x.into());
            let next = next.clone().into();
            self.0.extend_mapping(&curr, &next)
        }

        pub fn finalize_mapping(
            &mut self,
            page_range: PageRange,
        ) -> Result<()> {
            self.0.finalize_mapping(page_range.into())
        }

        pub fn initial_flags(&self) -> PageTableFlags {
            self.0.initial_flags()
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

        pub fn copy_to_memory(
            self,
            file: &[u8],
        ) -> Result<VirtRange> {
            Ok(super::FileRange::from(self.clone()).copy_to_memory(file)?.into())
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
