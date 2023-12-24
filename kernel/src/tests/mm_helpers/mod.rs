#![allow(dead_code)]

use ku::memory::{
    Virt,
    mmu::{
        PAGE_OFFSET_BITS,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_INDEX_MASK,
        PAGE_TABLE_ROOT_LEVEL,
        PageTableFlags,
    },
};

use kernel::memory::test_scaffolding::{
    Mapping,
    page_table_root,
};

pub(super) fn check_intermediate_flags(
    mapping: &mut Mapping,
    virt: Virt,
    expected_flags: PageTableFlags,
) {
    let page_table_root_index_shift =
        PAGE_TABLE_ROOT_LEVEL * PAGE_TABLE_INDEX_BITS + PAGE_OFFSET_BITS;
    let page_table_root_index =
        (virt.into_usize() >> page_table_root_index_shift) & PAGE_TABLE_INDEX_MASK;
    let intermediate_flags = page_table_root(mapping)[page_table_root_index].flags();
    assert_eq!(
        intermediate_flags, expected_flags,
        "intermediate page table flags are incorrect",
    );

    let leaf_flags = PageTableFlags::empty();
    let pte = mapping.translate(virt, None, PageTableFlags::empty()).unwrap();
    assert_eq!(
        pte.flags(),
        leaf_flags,
        "leaf page table flags are incorrect",
    );
}
