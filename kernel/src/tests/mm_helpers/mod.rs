#![allow(dead_code)]

use ku::memory::{
    Virt,
    mmu::{
        PAGE_TABLE_ROOT_LEVEL,
        PageTableEntry,
        PageTableFlags,
    },
    size,
};

use kernel::memory::test_scaffolding::{
    Mapping,
    nodes,
};

pub(super) fn check_intermediate_flags(
    mapping: &mut Mapping,
    virt: Virt,
    expected_flags: PageTableFlags,
) {
    let path = mapping.path(virt);
    let intermediate_flags = unsafe {
        nodes(&path)[size::from(PAGE_TABLE_ROOT_LEVEL)]
            .try_into_ref::<PageTableEntry>()
            .unwrap()
            .flags()
    };
    assert_eq!(
        intermediate_flags, expected_flags,
        "intermediate page table flags are incorrect",
    );

    let leaf_flags = PageTableFlags::empty();
    let pte = mapping.translate(virt).unwrap();
    assert_eq!(
        pte.flags(),
        leaf_flags,
        "leaf page table flags are incorrect",
    );
}
