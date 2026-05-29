use core::{
    arch::asm,
    fmt,
    mem,
    result,
};

use bitflags::bitflags;
use derive_more::Display;
use duplicate::duplicate_item;
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{
        self,
        Unexpected,
        Visitor,
    },
};
use static_assertions::const_assert_eq;

use crate::{
    error::{
        Error::NoPage,
        Result,
    },
    log::warn,
};

use super::{
    addr::{
        Phys,
        Virt,
    },
    frage::{
        ElasticFrame,
        Frame,
        Page,
    },
    size,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Запись таблицы страниц.
///
/// Аналогична [`x86_64::structures::paging::page_table::PageTableEntry`].
#[derive(Clone, Copy, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    /// Флаги доступа к странице, которую описывает эта запись.
    pub fn flags(&self) -> PageTableFlags {
        let flags = self.0 & !Self::ADDRESS_MASK;
        let flags = PageTableFlags::from_bits(flags).unwrap_or_else(|| {
            panic!(
                "incorrect PageTableFlags {flags:#X} for PTE at {}",
                Virt::from_ref(self),
            )
        });

        flags ^ PageTableFlags::EXECUTABLE
    }

    /// Устанавливает флаги доступа к странице, которую описывает эта запись.
    ///
    /// При необходимости переносит флаг [`PageTableFlags::PAGE_ATTRIBUTE_TABLE`]
    /// во флаг [`PageTableFlags::HUGE`].
    pub fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) {
        self.set_phys(self.phys(), flags);
    }

    /// Физический фрейм, на который указывает эта запись.
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- эта запись не используется,
    ///   то есть, сброшен бит [`PageTableFlags::PRESENT`].
    ///
    /// # Safety
    ///
    /// Нельзя использовать эту функцию, если эта запись описывает большую страницу.
    pub unsafe fn frame(&self) -> Result<Frame> {
        if self.is_present() {
            Ok(Frame::new(self.phys()).unwrap())
        } else {
            Err(NoPage)
        }
    }

    /// Устанавливает целевой физический фрейм `frame` и
    /// флаги доступа `flags` в данной записи.
    /// Принудительно выставляет флаг [`PageTableFlags::PRESENT`].
    ///
    /// Переносит флаг [`PageTableFlags::PAGE_ATTRIBUTE_TABLE`]
    /// во флаг [`PageTableFlags::HUGE`].
    ///
    /// # Panics
    ///
    /// Паникует в отладочном режиме, если `flags` описывают большую страницу,
    /// то есть в них установлен бит [`PageTableFlags::HUGE`].
    pub fn set_frame(
        &mut self,
        frame: Frame,
        flags: PageTableFlags,
    ) {
        debug_assert!(
            !flags.contains(PageTableFlags::HUGE),
            "{}",
            Self::HUGE_PAGE_ERROR_MESSAGE,
        );

        self.set_phys(frame.address(), flags | PageTableFlags::PRESENT);
    }

    /// Большой физический фрейм, на который указывает эта запись.
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- эта запись не используется,
    ///   то есть, сброшен бит [`PageTableFlags::PRESENT`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - Эта запись описывает не большую, а обычную страницу,
    ///   то есть в ней сброшен бит [`PageTableFlags::HUGE`].
    /// - Значение `level` не соответствует уровню,
    ///   на котором можно располагать большую страницу.
    /// - Физический адрес, на который указывает эта запись не выровнен так,
    ///   как должен был бы быть для большой страницы на уровне `level`.
    ///
    /// # Safety
    ///
    /// Эту функцию можно использовать, только если эта запись описывает большую страницу.
    pub unsafe fn huge_frame<const SIZE: usize>(&self) -> Result<ElasticFrame<SIZE>> {
        assert!(self.flags().is_huge(), "{}", Self::HUGE_PAGE_ERROR_MESSAGE);

        if self.is_present() {
            Ok(ElasticFrame::new(self.phys()).expect("bad huge frame address in PageTableEntry"))
        } else {
            Err(NoPage)
        }
    }

    /// Устанавливает большой целевой физический фрейм `frame` и
    /// флаги доступа `flags` в данной записи.
    /// Принудительно выставляет флаги [`PageTableFlags::PRESENT`] и [`PageTableFlags::HUGE`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - Значение `level` не соответствует уровню,
    ///   на котором можно располагать большую страницу.
    /// - Физический адрес `frame` не выровнен так,
    ///   как должен был бы быть для большой страницы на уровне `level`.
    pub fn set_huge_frame<const SIZE: usize>(
        &mut self,
        frame: ElasticFrame<SIZE>,
        flags: PageTableFlags,
    ) {
        self.set_phys(
            frame.address(),
            flags | PageTableFlags::HUGE | PageTableFlags::PRESENT,
        );
    }

    /// Адрес физического фрейма, на который указывает эта запись.
    fn phys(&self) -> Phys {
        Phys::new(self.0 & Self::ADDRESS_MASK).unwrap()
    }

    /// Устанавливает адрес физического фрейма `phys` и флаги доступа `flags`.
    ///
    /// При необходимости переносит флаг [`PageTableFlags::PAGE_ATTRIBUTE_TABLE`]
    /// во флаг [`PageTableFlags::HUGE`].
    fn set_phys(
        &mut self,
        phys: Phys,
        mut flags: PageTableFlags,
    ) {
        if !flags.is_huge() && flags.is_pat() {
            flags ^= PageTableFlags::HUGE | PageTableFlags::PAGE_ATTRIBUTE_TABLE;
        }

        self.0 = phys.into_usize() | (flags ^ PageTableFlags::EXECUTABLE).bits();
    }

    /// Возвращает `true`, если выставлен соответствующий флаг доступа.
    #[duplicate_item(
        flag_getter;
        [is_dirty];
        [is_executable];
        [is_present];
        [is_writable];
        [is_user];
    )]
    pub fn flag_getter(&self) -> bool {
        self.flags().flag_getter()
    }

    /// Возвращает `true`, если эта запись описывает большую страницу.
    /// Аргумент `level` указывает, на каком уровне таблицы отображения страниц
    /// расположена эта запись.
    pub fn is_huge(
        &self,
        level: u32,
    ) -> bool {
        level != PAGE_TABLE_LEAF_LEVEL && self.flags().is_huge()
    }

    /// Очищает запись.
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Забирает из записи физический фрейм, на который она указывает.
    /// Сама запись очищается.
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- эта запись не используется,
    ///   то есть, сброшен бит [`PageTableFlags::PRESENT`].
    ///
    /// # Safety
    ///
    /// Нельзя использовать эту функцию, если эта запись описывает большую страницу.
    pub unsafe fn take(&mut self) -> Result<Frame> {
        if self.is_present() {
            let phys = self.phys();
            self.clear();
            Ok(Frame::new(phys).unwrap())
        } else {
            Err(NoPage)
        }
    }

    /// Маска адреса физического фрейма, на который указывает эта запись.
    const ADDRESS_MASK: usize = ((1 << Phys::BITS) - 1) & !((1 << PAGE_OFFSET_BITS) - 1);

    /// Сообщения при паниках из-за несоответствия состояния флага [`PageTableFlags::HUGE`] и
    /// метода, который используется для доступа к физическому фрейму.
    const HUGE_PAGE_ERROR_MESSAGE: &str =
        "set HUGE flag if and only if the frame is huge (not 4KiB)";
}

impl fmt::Debug for PageTableEntry {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let flags = self.flags();

        if self.is_present() {
            write!(formatter, "{} {flags}", Frame::containing(self.phys()))
        } else {
            write!(formatter, "<non-present>")
        }
    }
}

/// Читает из регистра `CR3` физический адрес корневого узла текущей таблицы страниц.
pub fn page_table_root() -> Frame {
    let page_table_root: usize;

    unsafe {
        asm!(
            "mov {page_table_root}, cr3",
            page_table_root = out(reg) page_table_root,
        );
    }

    let flags = PageTableFlags::from_bits(page_table_root & !PageTableEntry::ADDRESS_MASK).unwrap();
    let page_table_root =
        Frame::containing(Phys::new(page_table_root).expect("invalid physical address in CR3"));

    if !flags.is_empty() {
        warn!(
            ?page_table_root,
            ?flags,
            "non empty flags for the page table root are wrong",
        );
    }

    page_table_root
}

/// Записывает в регистра `CR3` физический адрес корневого узла таблицы страниц.
/// При этом процессор переключается в виртуальное адресное пространство,
/// задаваемое этой таблицей страниц.
///
/// # Safety
///
/// Вызывающий код должен гарантировать сохранение инвариантов работы с памятью в Rust.
/// В частности, что не осталось ссылок, ведущих в страницы, для которых целевое
/// страничное отображение отличается от текущего.
pub unsafe fn set_page_table_root(page_table_root: Frame) {
    unsafe {
        asm!(
            "mov cr3, {pte}",
            pte = in(reg) page_table_root.address().into_usize(),
        );
    }
}

/// Узел таблицы страниц.
/// Аналогичен [`x86_64::structures::paging::page_table::PageTable`].
pub type PageTable = [PageTableEntry; PAGE_TABLE_ENTRY_COUNT];

bitflags! {
    /// Флаги в записи таблицы страниц.
    /// Аналогичны [`x86_64::structures::paging::page_table::PageTableFlags`].
    #[derive(Clone, Copy, Default, Eq, PartialEq)]
    pub struct PageTableFlags: usize {
        /// Страница отображена в память.
        const PRESENT = 1 << 0;

        /// Страница доступна на запись.
        const WRITABLE = 1 << 1;

        /// Страница доступна в режиме пользователя.
        const USER = 1 << 2;

        /// К странице был доступ.
        const ACCESSED = 1 << 5;

        /// В страницу была запись.
        const DIRTY = 1 << 6;

        /// Большая страница вместо следующего уровня таблицы страниц.
        const HUGE = 1 << 7;

        /// Страницу не нужно сбрасывать при сбросе всего TLB.
        const GLOBAL = 1 << 8;

        /// Биты, доступные ОС для произвольных нужд.
        const AVAILABLE = Self::AVAILABLE_2.bits() | Self::AVAILABLE_1.bits() | Self::AVAILABLE_0.bits();

        /// Бит, доступный ОС для произвольных нужд.
        const AVAILABLE_0 = 0b001 << 9;

        /// Бит, доступный ОС для произвольных нужд.
        const AVAILABLE_1 = 0b010 << 9;

        /// Бит, доступный ОС для произвольных нужд.
        const AVAILABLE_2 = 0b100 << 9;

        /// Один из битов [`PageTableFlags::AVAILABLE`] используется
        /// для пометки страниц, которые должны быть скопированы в случае записи в них.
        const COPY_ON_WRITE = Self::AVAILABLE_0.bits();

        /// [Расширенное управление кэшированием](https://en.wikipedia.org/wiki/Page_attribute_table).
        /// Совместно с битами
        /// [`PageTableFlags::WRITE_THROUGH`] и [`PageTableFlags::UNCACHEABLE_MINUS`]
        /// выбирает тип кеширования виртуальной страницы.
        ///
        /// В аппаратуре это:
        /// - Либо 12-й бит для больших страниц, то есть в [`PageTableEntry`]
        ///   не на листьевом уровне [`PAGE_TABLE_LEAF_LEVEL`].
        /// - Либо это 7-й бит на листьевом уровне, где 12-й бит используется в номере
        ///   4KiB--фрейма [`L0Frame`] и его использовать нельзя.
        ///   А 7-й бит освобождается, так как на листьевом уровне не может быть
        ///   установлен флаг [`PageTableFlags::HUGE`].
        ///
        /// Перенос из 12-го бита в 7-й делают методы [`PageTableEntry::set_flags()`] и
        /// [`PageTableEntry::set_frame()`]. Но это делает невозможным корректно прочитать
        /// [`PageTableFlags::HUGE`]/[`PageTableFlags::PAGE_ATTRIBUTE_TABLE`]
        /// методом [`PageTableEntry::flags()`], в который не передаётся
        /// уровень узла таблицы страниц.
        const PAGE_ATTRIBUTE_TABLE = 1 << 12;

        /// Маска типа кэширования страницы, см. [`MemoryType`].
        const MEMORY_TYPE = Self::PAGE_ATTRIBUTE_TABLE.bits() | Self::WRITE_THROUGH.bits() | Self::UNCACHEABLE_MINUS.bits();

        /// См. [`MemoryType::WriteBack`].
        const WRITE_BACK = 0;

        /// См. [`MemoryType::WriteThrough`].
        const WRITE_THROUGH = 1 << 3;

        /// См. [`MemoryType::UncacheableMinus`].
        const UNCACHEABLE_MINUS = 1 << 4;

        /// См. [`MemoryType::Uncacheable`].
        const UNCACHEABLE = Self::UNCACHEABLE_MINUS.bits() | Self::WRITE_THROUGH.bits();

        /// См. [`MemoryType::WriteCombining`].
        const WRITE_COMBINING = Self::PAGE_ATTRIBUTE_TABLE.bits();

        /// См. [`MemoryType::WriteProtected`].
        const WRITE_PROTECTED = Self::PAGE_ATTRIBUTE_TABLE.bits() | Self::WRITE_THROUGH.bits();

        /// Страница доступна на исполнение.
        ///
        /// Процессор интерпретирует единицу в этом бите как запрет исполнения, а не разрешение.
        /// То есть, для него этот бит --- `NO_EXECUTE`.
        /// Но работать в коде с такой семантикой этого бита неудобно.
        /// Так как операции вроде `|` и `&` будут по-разному работать с битами
        /// [`PageTableFlags::WRITABLE`] и [`PageTableFlags::USER`] с одной стороны
        /// и `PageTableFlags::NO_EXECUTE` с другой.
        /// Поэтому в коде удобнее работать с [`PageTableFlags::EXECUTABLE`], который ведёт себя
        /// аналогично флагам [`PageTableFlags::WRITABLE`] и [`PageTableFlags::USER`].
        /// А при чтении и сохранении [`PageTableFlags`] в [`PageTableEntry`],
        /// а также при журналировании, значение этого флага инвертируется,
        /// см. [`PageTableEntry::flags()`] и [`PageTableEntry::set_phys()`].
        ///
        /// Чтобы работал запрет исполнения данных в страницах как кода,
        /// дополнительно требуется включить флаг
        /// [`x86_64::registers::model_specific::EferFlags::NO_EXECUTE_ENABLE`] в
        /// [`x86_64::registers::model_specific::Efer`].
        const EXECUTABLE = 1 << 63;
    }
}

impl PageTableFlags {
    /// Возвращает `true`, если выставлен соответствующий флаг доступа.
    #[duplicate_item(
        flag_getter flag;
        [is_dirty] [DIRTY];
        [is_executable] [EXECUTABLE];
        [is_huge] [HUGE];
        [is_pat] [PAGE_ATTRIBUTE_TABLE];
        [is_present] [PRESENT];
        [is_writable] [WRITABLE];
        [is_user] [USER];
    )]
    pub fn flag_getter(&self) -> bool {
        self.contains(PageTableFlags::flag)
    }

    /// Возвращает номер записи в таблице [`PAGE_ATTRIBUTE_TABLE`]
    /// [расширенного управления кэшированием](https://en.wikipedia.org/wiki/Page_attribute_table).
    pub fn pat_entry(&self) -> usize {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает флаги для записи в таблице [`PAGE_ATTRIBUTE_TABLE`]
    /// [расширенного управления кэшированием](https://en.wikipedia.org/wiki/Page_attribute_table)
    /// номер `entry`.
    ///
    /// # Panics
    ///
    /// Паникует, если `entry` имеет недопустимый номер.
    pub fn from_pat_entry(entry: usize) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    // TODO: your code here.
}

impl Serialize for PageTableFlags {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> result::Result<S::Ok, S::Error> {
        serializer.serialize_u64(size::into_u64(self.bits()))
    }
}

impl<'de> Deserialize<'de> for PageTableFlags {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> result::Result<Self, D::Error> {
        /// [`Visitor`] для десериализации [`PageTableFlags`].
        struct PageTableFlagsVisitor;

        impl<'de> Visitor<'de> for PageTableFlagsVisitor {
            type Value = PageTableFlags;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                formatter.write_str("a valid usize value")
            }

            fn visit_u64<E: de::Error>(
                self,
                value: u64,
            ) -> result::Result<PageTableFlags, E> {
                PageTableFlags::from_bits(size::from(value))
                    .ok_or(de::Error::invalid_value(Unexpected::Unsigned(value), &self))
            }
        }

        deserializer.deserialize_u64(PageTableFlagsVisitor)
    }
}

impl fmt::Debug for PageTableFlags {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{:#03X}",
            (*self ^ PageTableFlags::EXECUTABLE).bits(),
        )
    }
}

impl fmt::Display for PageTableFlags {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        /// Сокращения для флагов отображения страниц.
        static FLAGS: [(PageTableFlags, char); 11] = [
            (PageTableFlags::PRESENT, 'P'),
            (PageTableFlags::EXECUTABLE, 'X'),
            (PageTableFlags::WRITABLE, 'W'),
            (PageTableFlags::USER, 'U'),
            (PageTableFlags::ACCESSED, 'A'),
            (PageTableFlags::DIRTY, 'D'),
            (PageTableFlags::HUGE, 'H'),
            (PageTableFlags::GLOBAL, 'G'),
            (PageTableFlags::AVAILABLE_0, '0'),
            (PageTableFlags::AVAILABLE_1, '1'),
            (PageTableFlags::AVAILABLE_2, '2'),
        ];

        write!(formatter, "{} ", PAGE_ATTRIBUTE_TABLE[self.pat_entry()])?;

        for (flag, flag_symbol) in FLAGS.iter().rev() {
            write!(
                formatter,
                "{}",
                if self.contains(*flag) {
                    *flag_symbol
                } else {
                    '-'
                },
            )?;
        }

        write!(formatter, "({self:?})")
    }
}

/// Флаги для страниц, предназначенных для взаимодействия с устройствами
/// ([Memory--mapped I/O](https://en.wikipedia.org/wiki/Memory-mapped_I/O), MMIO).
pub const KERNEL_MMIO: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_RW.bits() | PageTableFlags::UNCACHEABLE.bits());

/// Флаги для страниц, предназначенных для взаимодействия с
/// [видеобуфером](https://en.wikipedia.org/wiki/Framebuffer)
/// и подобными устройствами.
pub const KERNEL_WC: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_RW.bits() | PageTableFlags::WRITE_COMBINING.bits());

/// Флаги для страниц, доступных ядру только на чтение.
pub const KERNEL_R: PageTableFlags =
    PageTableFlags::from_bits_truncate(PageTableFlags::PRESENT.bits());

/// Флаги для страниц, доступных ядру на чтение и запись.
pub const KERNEL_RW: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_R.bits() | PageTableFlags::WRITABLE.bits());

/// Флаги для страниц, доступных ядру только на чтение и исполнение.
pub const KERNEL_RX: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_R.bits() | PageTableFlags::EXECUTABLE.bits());

/// Флаги для страниц, доступных коду пользователя только на чтение.
pub const USER_R: PageTableFlags = PageTableFlags::from_bits_truncate(
    PageTableFlags::PRESENT.bits() | PageTableFlags::USER.bits(),
);

/// Флаги для страниц, доступных коду пользователя на чтение и запись.
pub const USER_RW: PageTableFlags =
    PageTableFlags::from_bits_truncate(USER_R.bits() | PageTableFlags::WRITABLE.bits());

/// Флаги для страниц, доступных коду пользователя только на чтение и исполнение.
pub const USER_RX: PageTableFlags =
    PageTableFlags::from_bits_truncate(USER_R.bits() | PageTableFlags::EXECUTABLE.bits());

/// Шаблон флагов для страниц, не ограничивающий доступ.
pub const FULL_ACCESS: PageTableFlags = PageTableFlags::from_bits_truncate(
    PageTableFlags::EXECUTABLE.bits() |
        PageTableFlags::PRESENT.bits() |
        PageTableFlags::USER.bits() |
        PageTableFlags::WRITABLE.bits(),
);

/// Шаблон флагов для страниц, который пользователь может задавать в системных вызовах.
pub const SYSCALL_ALLOWED_FLAGS: PageTableFlags = PageTableFlags::from_bits_truncate(
    FULL_ACCESS.bits() |
        PageTableFlags::ACCESSED.bits() |
        PageTableFlags::COPY_ON_WRITE.bits() |
        PageTableFlags::DIRTY.bits(),
);

/// Режимы кэширования для
/// [page attribute table (PAT)](https://en.wikipedia.org/wiki/Page_attribute_table).
///
/// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
/// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459)
#[derive(Clone, Copy, Debug, Display, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum MemoryType {
    /// Кэширование страницы запрещено.
    ///
    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > All accesses are uncacheable. Write combining is not allowed.
    /// > Speculative accesses are not allowed.
    #[display("UC")]
    Uncacheable = 0,

    /// Режим кэширования с
    /// [объединением записей (write combining)](https://en.wikipedia.org/wiki/Write_combining).
    /// Данные записываются в один из специальных write combining буферов процессора,
    /// а не в кэш процессора.
    /// Этот буфер асинхронно в фоне записывает данные в память или устройство.
    /// Никаких гарантий модели памяти для таких областей не предоставляется.
    ///
    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > All accesses are uncacheable. Write combining is allowed. Speculative reads are allowed.
    #[display("WC")]
    WriteCombining = 1,

    /// Для страницы используется синхронная запись в память, без ожидания буфера записи.
    ///
    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > Reads allocate cache lines on a cache miss, but only to the shared state.
    /// > Cache lines are not allocated on a write miss.
    /// > Write hits update the cache and main memory.
    #[display("WT")]
    WriteThrough = 4,

    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > Reads allocate cache lines on a cache miss, but only to the shared state.
    /// > All writes update main memory. Cache lines are not allocated on a write miss.
    /// > Write hits invalidate the cache line and update main memory.
    #[display("WP")]
    WriteProtected = 5,

    /// Обычный механизм кэширования данных страницы.
    /// Новые данные записываются в кэш процессора,
    /// а кэш асинхронно в фоне записывает их в память.
    ///
    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > Reads allocate cache lines on a cache miss, and can allocate to
    /// > either the shared or exclusive state.
    /// > Writes allocate to the modified state on a cache miss.
    #[display("WB")]
    WriteBack = 6,

    /// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
    /// [Table 7-8. PAT Type Encodings](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G13.931459):
    /// > All accesses are uncacheable. Write combining is not allowed.
    /// > Speculative accesses are not allowed.
    /// > Can be overridden by an MTRR with the WC type.
    #[display("Uc")]
    UncacheableMinus = 7,
}

/// [Page attribute table (PAT)](https://en.wikipedia.org/wiki/Page_attribute_table),
/// которую нужно загрузить в процессор, чтобы:
///   - Были доступны режимы кэширования `PageTableFlags::WRITE_COMBINING` и
///     `PageTableFlags::WRITE_PROTECTED`.
///   - Для остальных режимов кэширования не поменялось соответствие флагам [`PageTableFlags`].
///
/// [AMD64 Architecture Programmer's Manual Volume 2: System Programming](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf),
/// [Table 7-9. PAT-Register PA-Field Indexing](https://github.com/tpn/pdfs/blob/master/AMD%20-%20AMD64%20Programmer's%20Manual%20-%20Volume%202%20-%20System%20Programming%20-%20Rev%203.30%20-%20Sept%202018%20(24593).pdf#G13.1049568).
pub const PAGE_ATTRIBUTE_TABLE: [MemoryType; 8] = [
    MemoryType::WriteBack,
    MemoryType::WriteThrough,
    MemoryType::UncacheableMinus,
    MemoryType::Uncacheable,
    MemoryType::WriteCombining,
    MemoryType::WriteProtected,
    MemoryType::UncacheableMinus,
    MemoryType::Uncacheable,
];

/// Количество бит в смещении внутри виртуальной страницы или физического фрейма.
/// Равно двоичному логарифму их размера.
pub const PAGE_OFFSET_BITS: u32 = 12;

/// Количество записей в одном узле таблицы страниц.
pub const PAGE_TABLE_ENTRY_COUNT: usize = 1 << PAGE_TABLE_INDEX_BITS;

/// Количество бит в номере записи внутри одного узла таблицы страниц.
pub const PAGE_TABLE_INDEX_BITS: u32 = 9;

/// Маска для вырезания из адреса индекса в каком-нибудь из узлов таблицы страниц.
pub const PAGE_TABLE_INDEX_MASK: usize = (1 << PAGE_TABLE_INDEX_BITS) - 1;

/// Номер листьевого уровня таблицы страниц.
pub const PAGE_TABLE_LEAF_LEVEL: u32 = 0;

/// Номер корневого уровня таблицы страниц.
pub const PAGE_TABLE_ROOT_LEVEL: u32 = 3;

/// Количество уровней в дереве отображения.
pub const PAGE_TABLE_LEVEL_COUNT: usize = PAGE_TABLE_ROOT_LEVEL as usize + 1;

const_assert_eq!(
    mem::size_of::<PageTable>(),
    PAGE_TABLE_ENTRY_COUNT * mem::size_of::<PageTableEntry>(),
);
const_assert_eq!(mem::size_of::<PageTable>(), Page::SIZE);

#[cfg(test)]
mod test {
    use super::{
        MemoryType,
        PAGE_ATTRIBUTE_TABLE,
        PageTableFlags,
    };

    #[test]
    fn page_attribute_table() {
        for (flags, memory_type) in [
            (PageTableFlags::WRITE_BACK, MemoryType::WriteBack),
            (PageTableFlags::WRITE_THROUGH, MemoryType::WriteThrough),
            (
                PageTableFlags::UNCACHEABLE_MINUS,
                MemoryType::UncacheableMinus,
            ),
            (PageTableFlags::UNCACHEABLE, MemoryType::Uncacheable),
            (PageTableFlags::WRITE_COMBINING, MemoryType::WriteCombining),
            (PageTableFlags::WRITE_PROTECTED, MemoryType::WriteProtected),
        ] {
            assert_eq!(flags & PageTableFlags::MEMORY_TYPE, flags);
            assert_eq!(PAGE_ATTRIBUTE_TABLE[flags.pat_entry()], memory_type);
        }
    }

    #[test]
    fn pat_entry_conversions() {
        for entry in 0 .. PAGE_ATTRIBUTE_TABLE.len() {
            assert_eq!(PageTableFlags::from_pat_entry(entry).pat_entry(), entry);
        }
    }
}
