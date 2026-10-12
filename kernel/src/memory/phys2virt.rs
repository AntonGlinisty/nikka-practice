use core::{
    fmt,
    mem::MaybeUninit,
    num::NonZeroUsize,
};

use derive_getters::Getters;
use itertools::Itertools;

use ku::{
    error::{
        Error::{
            InvalidArgument,
            NoPage,
            Overflow,
            PermissionDenied,
        },
        Result,
    },
    memory::{
        Block,
        Frame,
        L2Frame,
        Page,
        Phys,
        Size,
        Virt,
        mmu::{
            KERNEL_RW,
            PAGE_TABLE_ENTRY_COUNT,
            PAGE_TABLE_ROOT_LEVEL,
            PageTable,
            PageTableEntry,
        },
    },
};

use super::{
    FRAME_ALLOCATOR,
    FrameGuard,
    range,
};

// Used in docs.
#[allow(unused)]
use {
    crate as kernel,
    crate::error::Error,
};

/// Для простоты работы с физической памятью,
/// она целиком линейно отображена в некоторую область виртуальной.
/// [`Phys2Virt`] описывает это отображение.
#[derive(Clone, Copy, Debug, Default, Eq, Getters, PartialEq)]
pub struct Phys2Virt {
    /// Блок виртуальной памяти, куда линейно отображена вся физическая память.
    mapping: Block<Page>,

    /// Блок, описывающий всю доступную физическую память
    /// (см. [`kernel::memory::range::physical()`]).
    physical_memory: Block<Frame>,

    /// Количество использованных для отображения физических фреймов.
    /// Или [`None`], если оно не известно, так как отображение построил `bootloader`.
    used_frame_count: Option<NonZeroUsize>,
}

impl Phys2Virt {
    /// Создаёт [`Phys2Virt`] по его начальной странице `start` и
    /// блоку всей физической памяти `physical_memory`
    /// (см. [`kernel::memory::range::physical()`]).
    pub(super) fn new(
        physical_memory: Block<Frame>,
        start: Page,
    ) -> Result<Self> {
        Self::new_impl(physical_memory, start, None)
    }

    /// Для заданного физического адреса `phys` возвращает виртуальный адрес
    /// внутри специальной области [`Phys2Virt`],
    /// в которую линейно отображена вся физическая память.
    ///
    /// # Errors
    ///
    /// - [`Error::Overflow`] --- адрес `phys` не попадает в физическую память,
    ///   имеющуюся в системе.
    pub fn map(
        &self,
        phys: Phys,
    ) -> Result<Virt> {
        let virt = (self.mapping.start_address() + phys.into_usize())?;
        if self.mapping.contains_address(virt) {
            Ok(virt)
        } else {
            Err(Overflow)
        }
    }

    /// Создаёт отображение [`Phys2Virt`] всей физической памяти `physical_memory`
    /// (см. [`kernel::memory::range::physical()`]),
    /// используя рекурсивную запись `recursive_mapping`.
    ///
    /// Для экономии физических фреймов под  [`Phys2Virt`]:
    ///   - Использует страницы размером по 1 GiB.
    ///   - Учитывает, что фреймы вне [`Phys2Virt::physical_memory`]
    ///     никогда не будут переданы в [`Phys2Virt::map()`].
    ///     Так как их нет в физической памяти.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidArgument`] --- номер рекурсивной записи `recursive_mapping`
    ///   выходит за пределы узла [`PageTable`] дерева отображения страниц.
    ///   Или равен нулю, чего не может быть так как под нулевой записью верхнего уровня есть
    ///   зарезервированные участки памяти.
    /// - [`Error::NoFrame`] --- не хватило свободной физической памяти.
    /// - [`Error::NoPage`] --- в корневом узле дерева отображения виртуальных страниц
    ///   не нашлось достаточного количества записей для отображения блока `physical_memory`.
    /// - [`Error::PermissionDenied`] --- рекурсивная запись `recursive_mapping` попадает в
    ///   пользовательскую часть виртуальной памяти.
    pub(super) fn make(
        physical_memory: Block<Frame>,
        recursive_mapping: usize,
    ) -> Result<Self> {
        if recursive_mapping == 0 || recursive_mapping >= PAGE_TABLE_ENTRY_COUNT {
            return Err(InvalidArgument);
        }
        if range::user_root_level_entries().contains(&recursive_mapping) {
            return Err(PermissionDenied);
        }

        // TODO: your code here.
        let root = unsafe { Self::node(recursive_mapping, recursive_mapping).try_into_mut_slice::<PageTableEntry>()? }; // TODO: remove before flight.
        let root_entry = Self::find_empty_root_entries(1, recursive_mapping)?; // TODO: remove before flight.
        Self::allocate_root_entry(root_entry, recursive_mapping)?; // TODO: remove before flight.
        let node = unsafe { Self::node(root_entry, recursive_mapping).try_into_mut_slice::<PageTableEntry>()? }; // TODO: remove before flight.
        node[0].set_huge_frame(L2Frame::default(), KERNEL_RW); // TODO: remove before flight.
        Self::new_impl(physical_memory, Page::higher_half(), NonZeroUsize::new(1)) // TODO: remove before flight.
    }

    /// Находит во второй половине корневого узла страниц. То есть, в верхней
    /// [половине адресного пространства](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details) ---
    /// последовательность из `root_entry_count` не используемых записей.
    /// Возвращает номер первой из этих записей.
    ///
    /// Использует рекурсивную запись номер `recursive_mapping`.
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- во второй половине корневого узле дерева отображения
    ///   виртуальных страниц не нашлось `root_entry_count` свободных записей.
    fn find_empty_root_entries(
        root_entry_count: usize,
        recursive_mapping: usize,
    ) -> Result<usize> {
        let lower_half_entries = PAGE_TABLE_ENTRY_COUNT / 2;

        // TODO: your code here.
        Ok(lower_half_entries) // TODO: remove before flight.
    }

    /// Инициализирует записи 1 GiB страниц для отображения [`Phys2Virt`].
    ///
    /// Начинает с корневой записи `start_root_entry` и
    /// инициализирует `root_entry_count` корневых записей.
    /// А также инициализирует `entry_count` записей следующего уровня,
    /// каждая из которых отвечает за 1 GiB виртуальной памяти.
    ///
    /// Использует рекурсивную запись номер `recursive_mapping`.
    ///
    /// # Errors
    ///
    /// - [`Error::NoFrame`] --- не хватило свободной физической памяти.
    fn fill_entries(
        start_root_entry: usize,
        root_entry_count: usize,
        entry_count: usize,
        recursive_mapping: usize,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Выделяет фрейм для дочернего узла номер `root_entry` корневого узла таблицы страниц,
    /// инициализирует его и записывает его в соответствующую [`PageTableEntry`].
    ///
    /// Использует рекурсивную запись номер `recursive_mapping`.
    ///
    /// # Errors
    ///
    /// - [`Error::NoFrame`] --- не хватило свободной физической памяти.
    /// - [`Error::NoPage`] --- корневая запись номер `root_entry` уже занята.
    fn allocate_root_entry(
        root_entry: usize,
        recursive_mapping: usize,
    ) -> Result<()> {
        let root = unsafe {
            Self::node(recursive_mapping, recursive_mapping).try_into_mut::<PageTable>()?
        };
        let pte = &mut root[root_entry];

        if !pte.is_present() {
            let frame = FrameGuard::allocate()?;
            unsafe {
                frame.replace(pte, KERNEL_RW);
            }

            let node = unsafe {
                Self::node(root_entry, recursive_mapping)
                    .try_into_mut_slice::<MaybeUninit<PageTableEntry>>()?
            };
            node.write_filled(PageTableEntry::new(PAGE_TABLE_ROOT_LEVEL - 1));

            Ok(())
        } else {
            Err(NoPage)
        }
    }

    /// Возвращает узел таблицы страниц,
    /// на который ссылается запись номер `index` корневого узла таблицы страниц.
    ///   - При `index == recursive_mapping` этот узел --- сам корневой.
    ///   - При `index != recursive_mapping` это узел следующего уровня,
    ///     каждая запись в нём отвечает за 1 GiB адресного пространства.
    ///
    /// Возвращает блок размером в одну 4 KiB страницу,
    /// который удобно преобразовывать с помощью `Block::try_into*()`
    /// в подходящий тип для работы с узлом дерева отображения.
    ///
    /// Использует рекурсивную запись номер `recursive_mapping`.
    fn node(
        index: usize,
        recursive_mapping: usize,
    ) -> Block<Page> {
        let virt = Virt::from_page_table_indexes(
            [
                index,
                recursive_mapping,
                recursive_mapping,
                recursive_mapping,
            ],
            0,
        );
        let message = "wrong calculations through recursive entry";

        let page = Page::new(virt).expect(message);

        Block::new(page, (page + 1).expect(message)).expect(message)
    }

    /// Создаёт [`Phys2Virt`] по его начальной странице `start` и
    /// блоку всей физической памяти `physical_memory`
    /// (см. [`kernel::memory::range::physical()`]).
    ///
    /// Сохраняет `used_frame_count` --- количество физических фреймов,
    /// которые были потрачены на создание отображения `Phys2Virt`.
    /// Или [`None`], если это количество не известно --- отображение создано `bootloader`.
    fn new_impl(
        physical_memory: Block<Frame>,
        start: Page,
        used_frame_count: Option<NonZeroUsize>,
    ) -> Result<Self> {
        Ok(Self {
            mapping: Block::from_index(start.index(), start.index() + physical_memory.count())?,
            physical_memory,
            used_frame_count,
        })
    }
}

impl fmt::Display for Phys2Virt {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{} -> {}", self.mapping, self.physical_memory)?;

        if let Some(used_frame_count) = self.used_frame_count {
            write!(
                formatter,
                "; uses {} for the mapping itself",
                Size::new::<Frame>(used_frame_count.get()),
            )?;
        }

        Ok(())
    }
}

/// Для простоты работы с ней, физическая память целиком отображена в некоторую область виртуальной.
/// Эта функция по адресу первой (виртуальной) страницы этой области `phys2virt`
/// выдаёт соответствующий виртуальный адрес для заданного физического `address`.
#[allow(unused)]
pub(crate) fn map(
    phys2virt: Page,
    address: Phys,
) -> Virt {
    let message = "bad phys2virt";
    let frame = Frame::containing(address);
    let pseudo_physical_memory =
        Block::new(Frame::default(), (frame + 1).expect(message)).expect(message);

    Phys2Virt::new(pseudo_physical_memory, phys2virt)
        .expect(message)
        .map(address)
        .expect(message)
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use ku::error::Result;

    use super::{
        super::{
            Block,
            Frame,
            Page,
            Phys,
            Virt,
        },
        Phys2Virt,
    };

    pub fn make_phys2virt(
        physical_memory: Block<Frame>,
        recursive_mapping: usize,
    ) -> Result<Phys2Virt> {
        Phys2Virt::make(physical_memory, recursive_mapping)
    }

    pub fn phys2virt_map(
        phys2virt: Page,
        address: Phys,
    ) -> Virt {
        super::map(phys2virt, address)
    }
}
