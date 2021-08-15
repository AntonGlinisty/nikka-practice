use crate::{
    error::{
        Error::{
            NoPage,
            Unimplemented,
        },
        Result,
    },
    log::warn,
};

use super::{
    FrameAllocator,
    Virt,
    frage::{
        Frame,
        Page,
    },
    mmu::{
        self,
        FULL_ACCESS,
        PAGE_OFFSET_BITS,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_INDEX_MASK,
        PAGE_TABLE_LEAF_LEVEL,
        PAGE_TABLE_ROOT_LEVEL,
        PageTable,
        PageTableEntry,
        PageTableFlags,
    },
    phys2virt,
};

#[derive(Debug)]
pub(super) struct Mapping {
    page_table_root: Frame,
    phys2virt: Page,
}

impl Mapping {
    pub(super) fn new(
        page_table_root: Frame,
        phys2virt: Page,
    ) -> Self {
        Self {
            page_table_root,
            phys2virt,
        }
    }

    pub(super) fn page_table_root(&self) -> Frame {
        self.page_table_root
    }

    pub(crate) fn translate(
        &mut self,
        virt: Virt,
        mut frame_allocator: Option<&mut FrameAllocator>,
        flags: PageTableFlags,
    ) -> Result<&mut PageTableEntry> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn current_page_table_root() -> Frame {
        let pte = mmu::page_table_root();

        let page_table_root = Frame::new(pte.phys());
        let flags = pte.flags();

        if !flags.is_empty() {
            warn!(%page_table_root, ?flags, "non empty flags for the page table root are wrong");
        }

        page_table_root
    }

    pub(super) unsafe fn page_table_ref(
        &self,
        frame: Frame,
    ) -> &PageTable {
        phys2virt::map(self.phys2virt, frame.address())
            .try_into_ref()
            .expect("bad phys2virt or frame")
    }

    unsafe fn page_table_mut(
        &mut self,
        frame: Frame,
    ) -> &mut PageTable {
        phys2virt::map(self.phys2virt, frame.address())
            .try_into_mut()
            .expect("bad phys2virt or frame")
    }

    unsafe fn pte_mut(
        &mut self,
        virt: Virt,
        level: u32,
        page_table_frame: Frame,
    ) -> &mut PageTableEntry {
        let index_shift = PAGE_OFFSET_BITS + level * PAGE_TABLE_INDEX_BITS;
        let index = (virt.into_usize() >> index_shift) & PAGE_TABLE_INDEX_MASK;
        let page_table = self.page_table_mut(page_table_frame);

        &mut page_table[index]
    }

    fn map_intermediate(
        pte: &mut PageTableEntry,
        flags: PageTableFlags,
        phys2virt: Page,
        frame_allocator: &mut FrameAllocator,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }
}
