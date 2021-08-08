use bitflags::bitflags;

use super::addr::Phys;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits(self.0 & !Self::ADDRESS_MASK).unwrap()
    }

    pub(super) fn phys(&self) -> Phys {
        Phys::new(self.0 & Self::ADDRESS_MASK).unwrap()
    }

    const ADDRESS_MASK: usize = ((1 << Phys::BITS) - 1) & !((1 << PAGE_OFFSET_BITS) - 1);
}

pub(super) fn page_table_root() -> PageTableEntry {
    let pte: usize;

    unsafe {
        asm!(
            "mov {pte}, cr3",
            pte = out(reg) pte,
        );
    }

    PageTableEntry(pte)
}

bitflags! {
    pub struct PageTableFlags: usize {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const AVAILABLE = 0b111 << 9;
        const NO_EXECUTE = 1 << 63;
    }
}

pub(super) const PAGE_OFFSET_BITS: u32 = 12;
pub(super) const PAGE_TABLE_INDEX_BITS: u32 = 9;
pub(super) const PAGE_TABLE_ROOT_LEVEL: u32 = 3;
