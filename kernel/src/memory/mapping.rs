use crate::log::warn;

use super::{
    frage::Frame,
    mmu,
};

#[derive(Debug)]
pub(super) struct Mapping {
    page_table_root: Frame,
}

impl Mapping {
    pub(super) fn new(page_table_root: Frame) -> Self {
        Self { page_table_root }
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
}
