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
}

impl BootFrameAllocator {
    pub(super) fn new(memory_map: &MemoryMap) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    fn allocate_block(
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
}
