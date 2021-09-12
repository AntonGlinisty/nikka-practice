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
    FRAME_ALLOCATOR,
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
        PAGE_TABLE_ENTRY_COUNT,
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
pub struct Mapping {
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

    pub(super) fn duplicate(&self) -> Result<Self> {
        let mut result = Self::new(Frame::zero(), self.phys2virt);
        result.page_table_root =
            self.duplicate_subtree(&mut result, self.page_table_root(), PAGE_TABLE_ROOT_LEVEL)?;
        Ok(result)
    }

    pub(super) fn page_table_root(&self) -> Frame {
        self.page_table_root
    }

    pub fn translate(
        &mut self,
        virt: Virt,
        mut frame_allocator: Option<&mut FrameAllocator>,
        flags: PageTableFlags,
    ) -> Result<&mut PageTableEntry> {
        // TODO: your code here.
        unimplemented!();
    }

    fn duplicate_subtree(
        &self,
        dst: &mut Mapping,
        src_frame: Frame,
        level: u32,
    ) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    fn drop_subtree(
        &mut self,
        node: Frame,
        level: u32,
    ) {
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

impl Drop for Mapping {
    fn drop(&mut self) {
        if self.page_table_root() != Frame::zero() {
            assert_ne!(Self::current_page_table_root(), self.page_table_root());

            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL);
        }
    }
}
