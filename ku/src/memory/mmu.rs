use core::mem;

use bitflags::bitflags;
use static_assertions::const_assert_eq;

use crate::error::{
    Error::NoFrame,
    Result,
};

use super::{
    addr::Phys,
    frage::{
        Frame,
        Page,
    },
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits(self.0 & !Self::ADDRESS_MASK).unwrap()
    }

    pub fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) {
        self.set_phys(self.phys(), flags);
    }

    pub fn frame(&self) -> Result<Frame> {
        if self.is_present() {
            Ok(Frame::new(self.phys()))
        } else {
            Err(NoFrame)
        }
    }

    pub fn set_frame(
        &mut self,
        frame: Frame,
        flags: PageTableFlags,
    ) {
        self.set_phys(frame.address(), flags | PageTableFlags::PRESENT);
    }

    fn phys(&self) -> Phys {
        Phys::new(self.0 & Self::ADDRESS_MASK).unwrap()
    }

    fn set_phys(
        &mut self,
        phys: Phys,
        flags: PageTableFlags,
    ) {
        self.0 = phys.into_usize() | flags.bits();
    }

    pub fn is_present(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    const ADDRESS_MASK: usize = ((1 << Phys::BITS) - 1) & !((1 << PAGE_OFFSET_BITS) - 1);
}

pub fn page_table_root() -> Frame {
    let page_table_root: usize;

    unsafe {
        asm!(
            "mov {page_table_root}, cr3",
            page_table_root = out(reg) page_table_root,
        );
    }

    PageTableFlags::from_bits(page_table_root & !PageTableEntry::ADDRESS_MASK).unwrap();

    Frame::containing(Phys::new(page_table_root).expect("invalid physical address in CR3"))
}

pub unsafe fn set_page_table_root(page_table_root: Frame) {
    asm!(
        "mov cr3, {pte}",
        pte = in(reg) page_table_root.address().into_usize(),
    );
}

pub type PageTable = [PageTableEntry; PAGE_TABLE_ENTRY_COUNT];

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

pub const KERNEL_R: PageTableFlags = PageTableFlags::from_bits_truncate(
    PageTableFlags::PRESENT.bits() | PageTableFlags::NO_EXECUTE.bits(),
);

pub const KERNEL_RW: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_R.bits() | PageTableFlags::WRITABLE.bits());

pub const USER_RW: PageTableFlags =
    PageTableFlags::from_bits_truncate(KERNEL_RW.bits() | PageTableFlags::USER_ACCESSIBLE.bits());

pub const FULL_ACCESS: PageTableFlags = PageTableFlags::from_bits_truncate(
    PageTableFlags::PRESENT.bits() |
        PageTableFlags::WRITABLE.bits() |
        PageTableFlags::USER_ACCESSIBLE.bits(),
);

pub const PAGE_OFFSET_BITS: u32 = 12;
pub const PAGE_TABLE_ENTRY_COUNT: usize = 1 << PAGE_TABLE_INDEX_BITS;
pub const PAGE_TABLE_INDEX_BITS: u32 = 9;
pub const PAGE_TABLE_INDEX_MASK: usize = (1 << PAGE_TABLE_INDEX_BITS) - 1;
pub const PAGE_TABLE_LEAF_LEVEL: u32 = 0;
pub const PAGE_TABLE_ROOT_LEVEL: u32 = 3;

const_assert_eq!(
    mem::size_of::<PageTable>(),
    PAGE_TABLE_ENTRY_COUNT * mem::size_of::<PageTableEntry>(),
);
const_assert_eq!(mem::size_of::<PageTable>(), Page::SIZE);
