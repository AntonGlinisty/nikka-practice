use core::fmt;

use itertools::Itertools;

use crate::{
    error::{
        Error::NoPage,
        Result,
    },
    log::{
        error,
        info,
        trace,
    },
    time,
};

use super::{
    block::Block,
    frage::Page,
    mmu::{
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_ROOT_LEVEL,
        PageTable,
    },
    size::SizeOf,
};

#[derive(Debug)]
pub(super) struct PageAllocator {
    name: &'static str,
    block: Block<Page>,
}

impl PageAllocator {
    pub(super) fn new(
        name: &'static str,
        page_table_root: &PageTable,
    ) -> Self {
        let timer = time::timer();

        if let Some(block) = Self::find_unused_block(page_table_root) {
            let page_allocator = Self { name, block };

            info!(
                %page_allocator,
                free_page_count = page_allocator.block.count(),
                block = %page_allocator.block,
                duration = %timer.elapsed(),
                "init",
            );

            page_allocator
        } else {
            error!(?page_table_root);
            panic!(
                "failed to find a free entry in the page table root for the virtual page allocator",
            );
        }
    }

    pub(super) fn zero(name: &'static str) -> Self {
        Self {
            name,
            block: Block::zero(),
        }
    }

    pub(super) fn allocate(
        &mut self,
        size: usize,
    ) -> Result<Block<Page>> {
        // TODO: your code here.
        unimplemented!();
    }

    fn find_unused_block(page_table_root: &PageTable) -> Option<Block<Page>> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl fmt::Display for PageAllocator {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "\"{}\"", self.name)
    }
}
