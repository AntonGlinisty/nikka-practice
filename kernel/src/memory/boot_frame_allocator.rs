use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

use crate::{
    error::{
        Error::NoFrame,
        Result,
    },
    log::{
        info,
        trace,
    },
};

use super::{
    block::Block,
    frage::Frame,
    size::SizeOf,
};

pub struct BootFrameAllocator {
    block: Block<Frame>,
    initial_block: Block<Frame>,
}

impl BootFrameAllocator {
    pub(super) fn new(memory_map: &MemoryMap) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn count(&self) -> usize {
        self.block.count()
    }

    pub(super) fn allocate_block(
        &mut self,
        size: usize,
    ) -> Result<Block<Frame>> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn allocate(&mut self) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn is_managed(
        &self,
        frame: Frame,
    ) -> bool {
        self.initial_block.contains(frame.index())
    }

    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        self.is_managed(frame) && !self.block.contains(frame.index())
    }
}

impl Drop for BootFrameAllocator {
    fn drop(&mut self) {
        let leaked_frame_count = self.count();
        info!(frame_allocator = "boot", block = %self.block, leaked_frame_count, "drop");
        assert_eq!(leaked_frame_count, 0, "leaked some memory frames");
    }
}
