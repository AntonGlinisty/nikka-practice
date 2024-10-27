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
        warn,
    },
    memory::{
        Block,
        Page,
        Virt,
        mmu::PageTableFlags,
        size,
    },
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

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
    // TODO: your code here.
    unimplemented!();
}

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

impl<'a, T: BigAllocator> Loader<'a, T> {
    /// Инициализирует состояние загрузчика ELF--файлов.
    fn new(
        allocator: &'a mut T,
        file: &'a [u8],
    ) -> Self {
        Self { allocator, file }
    }

    /// Загружает диапазон [`next`][ElfRange] со следующим сегментом
    /// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
    ///
    /// - Выделяет для него память с помощью [`Loader::extend_mapping()`].
    /// - Копирует его в память с помощью [`ElfRange::copy_to_memory()`].
    /// - С помощью метода [`Loader::process_boundary()`] исправляет флаги отображения
    ///   для той части `curr` и границы между `curr` и `next`,
    ///   про которые стали известны окончательные значения флагов отображения.
    /// - Возвращает часть диапазона `next`, уже скопированную в память,
    ///   но ещё не отображённую с правильными флагами.
    ///   То есть, уже не обязательно соответствующую одному сегменту ELF--файла.
    fn load_program_header(
        &mut self,
        curr: Option<ElfRange>,
        next: ElfRange,
    ) -> Result<ElfRange> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Расширяет отображение текущего адресного пространства,
    /// чтобы гарантировать что блок, который описывает `next`, отображён в память.
    ///
    /// - Уже отображённую часть памяти определяет по `curr`,
    ///   для которого вызывалась ранее. Это должен обеспечить вызывающий метод.
    /// - Выделяет для `next` память с помощью [`Loader::allocator`].
    ///   Для этого резервирует виртуальные страницы методом [`BigAllocator::reserve_fixed()`].
    ///   А после этого отображает в физическую память методом [`BigAllocator::map()`].
    /// - Флаги страниц должны быть выставлены так, чтобы в эту память мог записать метод
    ///   [`ElfRange::copy_to_memory()`], см. [`Loader::initial_flags()`].
    ///   Эти флаги могут не совпадать с флагами [`ElfRange::flags`],
    ///   с которыми этот диапазон должен быть отображён в память процесса.
    ///   Последние проставит в дальнейшем метод [`Loader::finalize_mapping()`].
    /// - Кроме нужных для копирования флагов требуется также указывать
    ///   флаги аллокатора [`Loader::allocator.flags()`][BigAllocator::flags].
    ///   Среди них, например, может быть [`PageTableFlags::USER_ACCESSIBLE`],
    ///   которого нет во флагах ELF--файла.
    /// - Новые страницы отображения заполняет нулями с помощью [`MaybeUninit::fill()`].
    fn extend_mapping(
        &mut self,
        curr: &Option<ElfRange>,
        next: &ElfRange,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

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
        curr: ElfRange,
        next: ElfRange,
    ) -> Result<ElfRange> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Исправляет флаги отображения диапазона [`page_range`][PageRange].
    ///
    /// Флаги требуется исправить только если они отличаются от
    /// использованных изначально для копирования в память содержимого из ELF--файла.
    /// Изначальные флаги выставил ранее метод [`Loader::extend_mapping()`].
    ///
    /// Кроме нужных для копирования флагов требуется также указывать
    /// флаги аллокатора [`Loader::allocator.flags()`][BigAllocator::flags].
    /// Среди них, например, может быть [`PageTableFlags::USER_ACCESSIBLE`],
    /// которого нет во флагах ELF--файла.
    fn finalize_mapping(
        &mut self,
        page_range: PageRange,
    ) -> Result<()> {
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
    const INITIAL_FLAGS: PageTableFlags = PageTableFlags::from_bits_truncate(
        PageTableFlags::PRESENT.bits() | PageTableFlags::WRITABLE.bits(),
    );
}

/// Не выровненный по границам страниц диапазон памяти,
/// который нужно скопировать из ELF--файла в адресное пространство процесса.
///
/// Так как этот диапазон не выровнен по границам страниц,
/// он ещё не готов к отображению в память с финальными флагами,
/// которые диктует ELF--файл.
///
/// Создаётся по [`ProgramHeader`] в методе [`ElfRange::try_from()`].
/// То есть, изначально соответствует одному загружаемому сегменту ELF--файла.
/// Но в процессе загрузки ELF--файла может быть как объединён со смежными сегментами,
/// так и разделён на части типа [`PageRange`] и [`ElfRange`].
/// См. функцию [`combine()`].
#[derive(Debug, Display, Eq, PartialEq)]
#[display(
    "{{ memory: {}, flags: {}, file_range: {:#X?} }}",
    memory,
    flags,
    file_range
)]
struct ElfRange {
    /// Диапазон байт файла, которые нужно скопировать в память процесса.
    ///
    /// Используется только в [`ElfRange::copy_to_memory()`].
    /// Поэтому [`combine()`], которая вызывается после вызова
    /// [`ElfRange::copy_to_memory()`], может не беспокоиться о том,
    /// какой [`ElfRange::file_range`] у её аргументов `curr` и `next` и
    /// какой [`ElfRange::file_range`] выставить для своего результата `updated_next`.
    ///
    /// Альтернативой было бы поглощать `self` в [`ElfRange::copy_to_memory()`]
    /// и возвращать из неё другую структуру, уже без этого поля.
    /// А в [`combine()`] использовать этот другой тип.
    /// Который можно было бы совместить с [`PageRange`], параметризовав его
    /// [`Virt`] или [`Page`].
    file_range: Range<usize>,

    /// Флаги, с которыми должен быть отображён этот диапазон памяти процесса.
    flags: PageTableFlags,

    /// Не выровненный по границам страниц диапазон памяти,
    /// который нужно отобразить в адресное пространство процесса.
    memory: Block<Virt>,
}

impl ElfRange {
    /// Копирует из ELF--файла `file` в адресное пространство процесса
    /// диапазон памяти [`self`][ElfRange].
    fn copy_to_memory(
        &self,
        file: &[u8],
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl TryFrom<ProgramHeader<'_>> for ElfRange {
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
    ///     - При вычислении диапазона [`ElfRange::file_range`],
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

/// Выровненный по границам страниц диапазон памяти,
/// который нужно отобразить в адресное пространство процесса.
#[derive(Debug, Display, Eq, PartialEq)]
#[display("{{ memory: {}, flags: {} }}", memory, flags)]
struct PageRange {
    /// Флаги, с которыми должен быть отображён этот диапазон памяти процесса.
    flags: PageTableFlags,

    /// Выровненный по границам страниц диапазон памяти,
    /// который нужно отобразить в адресное пространство процесса.
    memory: Block<Page>,
}

impl From<ElfRange> for PageRange {
    /// Возвращает выровненный по границам страниц диапазон памяти [`PageRange`],
    /// содержащий [`self`][ElfRange] и имеющий те же флаги отображения страниц.
    fn from(elf_range: ElfRange) -> Self {
        Self {
            flags: elf_range.flags,
            memory: elf_range.memory.enclosing(),
        }
    }
}

/// Объединяет не выровненные по границам страниц
/// текущий обрабатываемый диапазон [`curr`][ElfRange] и
/// следующий за ним диапазон [`next`][ElfRange].
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
///     Флаги в ней должны быть подходящими как для диапазона `curr`, так и для диапазона `next`,
///     см. функцию [`combine_flags()`].
///     Поэтому они могут отличаться от флагов `curr_minus_next` и поэтому эта пограничная
///     страница не может быть обработана в составе `curr_minus_next`.
///     Часть `boundary` может содержать только одну страницу памяти.
///     Или же она равна [`None`], если никакие части `curr` и `next` не попадают в одну и ту же
///     страницу, либо в эту же страницу потенциально могут попасть последующие сегменты ELF--файла.
///   - `updated_next` содержит части `curr` и `next`,
///     которые не попали в `curr_minus_next` и `boundary`.
///     Флаги в ней должны быть подходящими для диапазона `next`.
///     В случае, если `curr` задевает те же страницы,
///     флаги `updated_next` должны быть подходящими и для `curr`,
///     см. функцию [`combine_flags()`].
///     В этом случае `updated_next` не может задевать больше одной страницы.
fn combine(
    curr: ElfRange,
    next: ElfRange,
) -> Result<(Option<PageRange>, Option<PageRange>, ElfRange)> {
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

        let mut flags = PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE;

        for (ph_flag, flag) in [
            (FLAG_R, PageTableFlags::default()),
            (FLAG_W, PageTableFlags::WRITABLE),
            (FLAG_X, PageTableFlags::NO_EXECUTE),
        ] {
            if ph_flags & ph_flag != 0 {
                flags ^= flag;
            }
        }

        Ok(flags)
    }
}

/// Объединяет флаги диапазонов памяти.
///
/// При отображении страницы в память с получившимися флагами,
/// она будет доступна как для операций, которые требуют флагов [`curr_flags`][PageTableFlags],
/// так и для операций, которые требуют флагов [`next_flags`][PageTableFlags].
fn combine_flags(
    curr_flags: PageTableFlags,
    next_flags: PageTableFlags,
) -> PageTableFlags {
    ((curr_flags ^ PageTableFlags::NO_EXECUTE) | (next_flags ^ PageTableFlags::NO_EXECUTE)) ^
        PageTableFlags::NO_EXECUTE
}

/// Требует чтобы [`curr`][ElfRange] лежал левее [`next`][ElfRange] в диапазоне адресов.
/// А также, чтобы они не пересекались.
/// См.
/// [System V Application Binary Interface](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html).
/// В противном случае, возвращает ошибку [`Error::InvalidArgument`].
fn validate_order(
    curr: &ElfRange,
    next: &ElfRange,
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
