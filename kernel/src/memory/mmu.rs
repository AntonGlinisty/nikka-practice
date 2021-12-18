use x86::tlb;

pub use ku::memory::mmu::{
    FULL_ACCESS,
    KERNEL_R,
    KERNEL_RW,
    PAGE_OFFSET_BITS,
    PAGE_TABLE_ENTRY_COUNT,
    PAGE_TABLE_INDEX_BITS,
    PAGE_TABLE_INDEX_MASK,
    PAGE_TABLE_LEAF_LEVEL,
    PAGE_TABLE_ROOT_LEVEL,
    PageTable,
    PageTableEntry,
    PageTableFlags,
    USER,
    USER_R,
    USER_RW,
    page_table_root,
    set_page_table_root,
};

use ku::memory::Page;

pub(super) unsafe fn flush(page: Page) {
    tlb::flush(page.address().into_usize());
}
