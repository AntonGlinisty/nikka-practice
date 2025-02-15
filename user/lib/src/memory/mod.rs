use core::{
    mem,
    ptr,
};

use ku::{
    error::{
        Error::NoPage,
        Result,
    },
    memory::{
        Block,
        Page,
        Virt,
        mmu::{
            FULL_ACCESS,
            PAGE_OFFSET_BITS,
            PAGE_TABLE_INDEX_BITS,
            PAGE_TABLE_LEAF_LEVEL,
            PAGE_TABLE_ROOT_LEVEL,
            PageTable,
            PageTableEntry,
        },
    },
    process::Pid,
};

use super::syscall;

// Used in docs.
#[allow(unused)]
use crate as lib;

/// Заводит в адресном пространстве страницу памяти для временных нужд.
/// Использует системный вызов [`lib::syscall::map()`].
pub fn temp_page() -> Result<Page> {
    // TODO: your code here.
    unimplemented!();
}

/// Копирует содержимое страницы `src` в страницу `dst` с помощью
/// [`core::ptr::copy_nonoverlapping()`].
///
/// # Safety
///
/// Страницы должны быть отображены в память и различны.
pub unsafe fn copy_page(
    src: Page,
    dst: Page,
) {
    assert_ne!(src, dst);

    // TODO: your code here.
    unimplemented!();
}

/// Пользуясь рекурсивной записью таблицы страниц, выдаёт ссылку
/// на таблицу страниц заданного уровня `level` для заданного виртуального адреса `virt`.
///
/// # Safety
///
/// - Уровень `level` должен быть равен [`PAGE_TABLE_ROOT_LEVEL`] или
///   адрес `virt` должен быть отображён и доступен пользователю.
/// - Время жизни возвращаемой ссылки не `'static`.
///   Оно может закончится, если делаются системные вызовы, меняющие адресное пространство.
pub unsafe fn page_table(
    virt: Virt,
    level: u32,
) -> &'static PageTable {
    // TODO: your code here.
    unimplemented!();
}

/// Пользуясь рекурсивной записью таблицы страниц, возвращает
/// `PageTableEntry` листьевого уровня для заданного виртуального адреса `virt`.
///
/// # Safety
///
/// Адрес `virt` должен быть отображён и доступен пользователю.
pub unsafe fn pte(virt: Virt) -> PageTableEntry {
    // TODO: your code here.
    unimplemented!();
}
