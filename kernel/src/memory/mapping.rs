use crate::error::{
    Error::{
        NoPage,
        Unimplemented,
    },
    Result,
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
        USER_R,
    },
    phys2virt,
};

#[derive(Debug, Default)]
pub struct Mapping {
    page_table_root: Frame,
    phys2virt: Page,
    recursive_mapping: usize,
}

impl Mapping {
    pub(super) fn new(
        page_table_root: Frame,
        phys2virt: Page,
    ) -> Self {
        Self {
            page_table_root,
            phys2virt,
            recursive_mapping: usize::MAX,
        }
    }

    pub(super) fn duplicate(&self) -> Result<Self> {
        let mut result = Self::new(Frame::default(), self.phys2virt);
        result.page_table_root =
            self.duplicate_subtree(&mut result, self.page_table_root(), PAGE_TABLE_ROOT_LEVEL)?;
        Ok(result)
    }

    pub(super) fn is_valid(&self) -> bool {
        self.page_table_root() != Frame::default()
    }

    pub(super) fn page_table_root(&self) -> Frame {
        self.page_table_root
    }

    #[allow(unused_mut)] // TODO: remove before flight.
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

    pub(crate) fn make_recursive_mapping(&mut self) -> Result<usize> {
        // TODO: your code here.
        Ok(self.recursive_mapping) // TODO: remove before flight.
    }

    pub(super) fn current_page_table_root() -> Frame {
        mmu::page_table_root()
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
        if self.is_valid() {
            assert_ne!(Self::current_page_table_root(), self.page_table_root());

            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL);
        }
    }
}
