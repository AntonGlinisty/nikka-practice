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
            PAGE_OFFSET_BITS,
            PAGE_TABLE_ENTRY_COUNT,
            PAGE_TABLE_INDEX_BITS,
            PAGE_TABLE_LEAF_LEVEL,
            PAGE_TABLE_ROOT_LEVEL,
            PageTable,
            PageTableEntry,
            USER_RW,
        },
    },
    process::Pid,
};

use super::syscall;

pub fn temp_page() -> Result<Page> {
    // TODO: your code here.
    unimplemented!();
}

pub unsafe fn copy_page(
    src: Page,
    dst: Page,
) {
    // TODO: your code here.
    unimplemented!();
}

pub unsafe fn page_table(
    virt: Virt,
    level: u32,
) -> &'static PageTable {
    // TODO: your code here.
    unimplemented!();
}

pub unsafe fn pte(virt: Virt) -> PageTableEntry {
    // TODO: your code here.
    unimplemented!();
}
