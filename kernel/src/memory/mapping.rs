use crate::log::warn;

use super::{
    frage::{
        Frame,
        Page,
    },
    mmu::{
        self,
        PageTable,
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
}
