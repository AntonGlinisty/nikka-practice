use core::{
    arch::asm,
    fmt,
    mem,
    result,
};

use bitflags::bitflags;
use duplicate::duplicate_item;
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
    pub fn frame(&self) -> Result<Frame> {
        if self.is_present() {
            Ok(Frame::new(self.phys()).unwrap())
        } else {
            Err(NoPage)
        }
    }

    /// Устанавливает целевой физический фрейм `frame` и
    /// флаги доступа `flags` в данной записи.
    /// Принудительно выставляет флаг [`PageTableFlags::PRESENT`].
    pub fn set_frame(
        &mut self,
        frame: Frame,
        flags: PageTableFlags,
    ) {
        self.set_phys(frame.address(), flags | PageTableFlags::PRESENT);
    }

    /// Адрес физического фрейма, на который указывает эта запись.
    fn phys(&self) -> Phys {
        Phys::new(self.0 & Self::ADDRESS_MASK).unwrap()
    }

    /// Устанавливает адрес физического фрейма `phys` и флаги доступа `flags`.
    fn set_phys(
        &mut self,
        phys: Phys,
        flags: PageTableFlags,
    ) {
        self.0 = phys.into_usize() | (flags ^ PageTableFlags::EXECUTABLE).bits();
    }

    /// Возвращает `true`, если выставлен соответствующий флаг доступа.
    #[duplicate_item(
        flag_getter;
        [is_dirty];
        [is_executable];
        [is_huge];
        [is_present];
        [is_writable];
        [is_user];
    )]
    pub fn flag_getter(&self) -> bool {
        self.flags().flag_getter()
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
    pub fn take(&mut self) -> Result<Frame> {
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
}

impl fmt::Debug for PageTableEntry {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        if let Ok(frame) = self.frame() {
            let flags = self.flags();
            write!(formatter, "{frame} {flags}")
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

        /// Для страницы используется синхронная запись в память, без ожидания буфера записи.
        const WRITE_THROUGH = 1 << 3;

        /// Кэширование страницы запрещено.
        const NO_CACHE = 1 << 4;

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
        [is_present] [PRESENT];
        [is_writable] [WRITABLE];
        [is_user] [USER];
    )]
    pub fn flag_getter(&self) -> bool {
        self.contains(PageTableFlags::flag)
    }
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
        static FLAGS: [(PageTableFlags, char); 13] = [
            (PageTableFlags::PRESENT, 'P'),
            (PageTableFlags::EXECUTABLE, 'X'),
            (PageTableFlags::WRITABLE, 'W'),
            (PageTableFlags::USER, 'U'),
            (PageTableFlags::WRITE_THROUGH, 'T'),
            (PageTableFlags::NO_CACHE, 'C'),
            (PageTableFlags::ACCESSED, 'A'),
            (PageTableFlags::DIRTY, 'D'),
            (PageTableFlags::HUGE, 'H'),
            (PageTableFlags::GLOBAL, 'G'),
            (PageTableFlags::AVAILABLE_0, '0'),
            (PageTableFlags::AVAILABLE_1, '1'),
            (PageTableFlags::AVAILABLE_2, '2'),
        ];

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
pub const KERNEL_MMIO: PageTableFlags = PageTableFlags::from_bits_truncate(
    KERNEL_RW.bits() | PageTableFlags::NO_CACHE.bits() | PageTableFlags::WRITE_THROUGH.bits(),
);

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
