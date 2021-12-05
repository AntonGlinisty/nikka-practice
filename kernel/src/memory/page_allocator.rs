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
    stack::GUARD_ZONE_SIZE,
};

#[derive(Debug, Default)]
pub(super) struct PageAllocator {
    block: Block<Page>,
}

impl PageAllocator {
    pub(super) fn new(page_table_root: &PageTable) -> Self {
        let timer = time::timer();

        if let Some(block) = Self::find_unused_block(page_table_root) {
            let mut page_allocator = Self { block };

            info!(
                free_page_count = page_allocator.block.count(),
                block = %page_allocator.block,
                duration = %timer.elapsed(),
                "page allocator init",
            );

            // Detect stack overruns that corrupt memory allocated by PageAllocator.
            page_allocator
                .allocate(GUARD_ZONE_SIZE)
                .expect("failed to create guard zone for PageAllocator");

            page_allocator
        } else {
            error!(?page_table_root);
            panic!(
                "failed to find a free entry in the page table root for the virtual page allocator",
            );
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
