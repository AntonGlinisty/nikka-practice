use lazy_static::lazy_static;
use spin::Mutex;

use ku::{
    error::{
        Error::NoDisk,
        Result,
    },
    log::trace,
    memory::{
        Block,
        Page,
        PageFaultInfo,
        Virt,
        mmu::{
            KERNEL_RW,
            PageTableFlags,
        },
    },
    process::Info,
};

use crate::memory::{
    BASE_ADDRESS_SPACE,
    mmu,
};

use super::{
    BLOCK_SIZE,
    disk::{
        Disk,
        SECTOR_SIZE,
    },
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockCache {
    cache: Block<Page>,
    disk: Disk,
}

impl BlockCache {
    pub(super) fn init(
        disk: Disk,
        block_count: usize,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn block(block_number: usize) -> Result<Block<Virt>> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn flush_block(block_number: usize) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn flush(count: usize) -> Result<()> {
        for block_number in 0 .. count {
            Self::flush_block(block_number)?;
        }

        Ok(())
    }

    pub(crate) fn trap_handler(info: &Info) -> Result<bool> {
        // TODO: your code here.
        Ok(false) // TODO: remove before flight.
    }

    fn get(address: Virt) -> Option<Self> {
        BLOCK_CACHE
            .lock()
            .filter(|block_cache| block_cache.cache.contains_address(address))
    }
}

lazy_static! {
    pub(super) static ref BLOCK_CACHE: Mutex<Option<BlockCache>> = Mutex::new(None);
}

const SECTORS_PER_BLOCK: usize = BLOCK_SIZE / SECTOR_SIZE;
