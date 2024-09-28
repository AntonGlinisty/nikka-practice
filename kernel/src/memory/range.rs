use core::{
    cmp,
    ops::Range,
};

use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};
use static_assertions::const_assert;

use crate::{
    error::{
        Error::PermissionDenied,
        Result,
    },
    log::info,
};

use super::{
    block::Block,
    frage::{
        Frame,
        Page,
    },
    mmu::{
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_ROOT_LEVEL,
        PageTableFlags,
    },
    size::Size,
};

// Used in docs.
#[allow(unused)]
use crate::{
    self as kernel,
    error::Error,
};

/// Возвращает блок, описывающий всю доступную физическую память.
/// Гарантированно начинается с нулевого фрейма.
///
/// В этот блок могут не попадать некоторые зарезервированные диапазоны фреймов.
/// Поэтому он может быть немного меньше установленной в системе физической памяти.
pub(super) fn physical(memory_map: &MemoryMap) -> Block<Frame> {
    let mut total_frame_count = 0;
    let mut usable_total_frame_count = 0;

    for region in memory_map.iter() {
        let usable = region.region_type == MemoryRegionType::Usable;
        let start = region.range.start_frame_number;
        let end = region.range.end_frame_number;

        if usable {
            usable_total_frame_count += end - start;
            total_frame_count = cmp::max(total_frame_count, end);
        }
    }

    info!(
        total = %Size::new_u64::<Frame>(total_frame_count),
        usable = %Size::new_u64::<Frame>(usable_total_frame_count),
        total_frame_count,
        usable_total_frame_count,
        "available memory",
    );

    Block::from_index_u64(0, total_frame_count).expect("invalid memory map from bootloader")
}

/// Возвращает `true`, если весь блок `pages` зарезервирован для ядра.
pub(crate) fn is_kernel_block(pages: Block<Page>) -> bool {
    !is_user_block(pages) && user_pages().is_disjoint(pages)
}

/// Возвращает `true`, если весь блок `pages` зарезервирован для пространства пользователя.
pub(crate) fn is_user_block(pages: Block<Page>) -> bool {
    user_pages().contains_block(pages)
}

/// Возвращает `true`, если страница `page` зарезервирована для пространства пользователя.
fn is_user_page(page: Page) -> bool {
    user_pages().contains(page)
}

/// Возвращает диапазон записей в корневом узле таблицы страниц,
/// который зарезервирован для пространства ядра.
pub(super) const fn kernel_root_level_entries() -> Range<usize> {
    0 .. KERNEL_ROOT_LEVEL_ENTRY_COUNT
}

/// Возвращает диапазон записей в корневом узле таблицы страниц,
/// который зарезервирован для пространства пользователя.
pub(super) const fn user_root_level_entries() -> Range<usize> {
    KERNEL_ROOT_LEVEL_ENTRY_COUNT .. LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT
}

/// Возвращает блок страниц,
/// который зарезервирован для пространства пользователя.
pub(super) fn user_pages() -> Block<Page> {
    let user_root_level_entries = user_root_level_entries();
    Block::from_index(
        user_root_level_entries.start * PAGES_PER_ROOT_LEVEL_ENTRY,
        user_root_level_entries.end * PAGES_PER_ROOT_LEVEL_ENTRY,
    )
    .expect("virtual memory block reserved for user space is invalid")
}

/// Возвращает ошибку [`Error::PermissionDenied`] если в блоке есть как страницы,
/// которые лежат в зарезервированной для пользователя области,
/// так и страницы, которые лежат в зарезервированной для ядра области.
pub(super) fn validate_block(pages: Block<Page>) -> Result<()> {
    if is_kernel_block(pages) != is_user_block(pages) {
        Ok(())
    } else {
        Err(PermissionDenied)
    }
}

/// Возвращает ошибку [`Error::PermissionDenied`] если принадлежность пользователю
/// какой-нибудь страницы блока `pages` не соответствует запрошенным флагам.
///
/// То есть, адрес этой страницы лежит в зарезервированной для пользователя области
/// [`kernel::memory::range::user_pages()`], а во `flags` нет доступа пользователю ---
/// флага [`PageTableFlags::USER_ACCESSIBLE`].
/// Либо наоборот, флаг [`PageTableFlags::USER_ACCESSIBLE`] есть, но страница не
/// лежит в области [`kernel::memory::range::user_pages()`].
pub(super) fn validate_block_flags(
    pages: Block<Page>,
    flags: PageTableFlags,
) -> Result<()> {
    validate_block(pages)?;

    if is_user_block(pages) == flags.contains(PageTableFlags::USER_ACCESSIBLE) {
        Ok(())
    } else {
        Err(PermissionDenied)
    }
}

/// Возвращает ошибку [`Error::PermissionDenied`] если принадлежность пользователю
/// страницы `page` не соответствует запрошенным флагам.
///
/// То есть, адрес этой страницы лежит в зарезервированной для пользователя области
/// [`kernel::memory::range::user_pages()`], а во `flags` нет доступа пользователю ---
/// флага [`PageTableFlags::USER_ACCESSIBLE`].
/// Либо наоборот, флаг [`PageTableFlags::USER_ACCESSIBLE`] есть, но страница не
/// лежит в области [`kernel::memory::range::user_pages()`].
pub(super) fn validate_page_flags(
    page: Page,
    flags: PageTableFlags,
) -> Result<()> {
    if is_user_page(page) == flags.contains(PageTableFlags::USER_ACCESSIBLE) {
        Ok(())
    } else {
        Err(PermissionDenied)
    }
}

/// Количество записей таблицы страниц корневого уровня,
/// отвечающих одной из половин виртуального адресного пространства.
const LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT: usize = PAGE_TABLE_ENTRY_COUNT / 2;

/// Количество записей таблицы страниц корневого уровня, зарезервированных для ядра.
pub(super) const KERNEL_ROOT_LEVEL_ENTRY_COUNT: usize = 32;

/// Количество страниц, за которые отвечает одна запись таблицы страниц корневого уровня.
pub(super) const PAGES_PER_ROOT_LEVEL_ENTRY: usize =
    PAGE_TABLE_ENTRY_COUNT.pow(PAGE_TABLE_ROOT_LEVEL);

const_assert!(KERNEL_ROOT_LEVEL_ENTRY_COUNT <= LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT / 2);
const_assert!(
    kernel_root_level_entries().end <= user_root_level_entries().start ||
        user_root_level_entries().end <= kernel_root_level_entries().start,
);
const_assert!(
    kernel_root_level_entries().end <= LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT ||
        LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT <= kernel_root_level_entries().start,
);
const_assert!(
    user_root_level_entries().end <= LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT ||
        LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT <= user_root_level_entries().start,
);

#[doc(hidden)]
pub mod test_scaffolding {
    use core::ops::Range;

    use super::super::{
        Block,
        Page,
    };

    pub const fn kernel_root_level_entries() -> Range<usize> {
        super::kernel_root_level_entries()
    }

    pub const fn user_root_level_entries() -> Range<usize> {
        super::user_root_level_entries()
    }

    pub fn user_pages() -> Block<Page> {
        super::user_pages()
    }

    pub const KERNEL_ROOT_LEVEL_ENTRY_COUNT: usize = super::KERNEL_ROOT_LEVEL_ENTRY_COUNT;
    pub const LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT: usize = super::LOWER_HALF_ROOT_LEVEL_ENTRY_COUNT;
    pub const PAGES_PER_ROOT_LEVEL_ENTRY: usize = super::PAGES_PER_ROOT_LEVEL_ENTRY;
}
