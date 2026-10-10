use x86::{
    msr::IA32_PAT,
    tlb,
};
use x86_64::registers::model_specific::Msr;

pub use ku::memory::mmu::{
    FULL_ACCESS,
    KERNEL_MMIO,
    KERNEL_R,
    KERNEL_RW,
    KERNEL_RX,
    KERNEL_WC,
    PAGE_OFFSET_BITS,
    PAGE_TABLE_ENTRY_COUNT,
    PAGE_TABLE_INDEX_BITS,
    PAGE_TABLE_INDEX_MASK,
    PAGE_TABLE_LEAF_LEVEL,
    PAGE_TABLE_LEVEL_COUNT,
    PAGE_TABLE_ROOT_LEVEL,
    PTE_LEVEL_CAP,
    PageTable,
    PageTableEntry,
    PageTableFlags,
    SYSCALL_ALLOWED_FLAGS,
    USER_R,
    USER_RW,
    USER_RX,
    page_table_root,
    set_page_table_root,
};

use ku::{
    log::info,
    memory::{
        Page,
        mmu::PAGE_ATTRIBUTE_TABLE,
        size,
    },
};

/// Инициализирует
/// [page attribute table (PAT)](https://en.wikipedia.org/wiki/Page_attribute_table)
/// процессора в соответствии с [`PAGE_ATTRIBUTE_TABLE`].
pub(super) fn init_page_attribute_table() {
    let mut pat = Msr::new(IA32_PAT);
    let old_pat = unsafe { pat.read() };

    let mut new_pat = 0;

    info!("page attribute table:");
    for (entry, memory_type) in (0 ..).into_iter().zip(PAGE_ATTRIBUTE_TABLE.iter()) {
        new_pat |= u64::from(u8::from(*memory_type)) << (entry * u8::BITS);
        let flags = PageTableFlags::from_pat_entry(size::from(entry));
        info!(%flags, ?memory_type, "    PA{entry}");
    }

    assert_eq!(
        old_pat & u64::from(u32::MAX),
        new_pat & u64::from(u32::MAX),
        "it is better to leave the first half of the page attribute table at the default settings",
    );

    unsafe {
        pat.write(new_pat);
    }
}

/// Сбрасывает кэш страничного преобразования процессора
/// ([Translation Lookaside Buffer, TLB](https://en.wikipedia.org/wiki/Translation_lookaside_buffer))
/// для заданной страницы `page`.
///
/// # Safety
///
/// Вызывающий код должен работать в режиме ядра
/// (в [кольце защиты](https://en.wikipedia.org/wiki/Protection_ring) 0).
pub unsafe fn flush(page: Page) {
    unsafe {
        tlb::flush(page.address().into_usize());
    }
}
