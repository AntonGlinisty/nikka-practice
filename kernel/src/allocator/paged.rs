use core::alloc::Layout;

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::{
        error,
        warn,
    },
    memory::{
        BASE_ADDRESS_SPACE,
        Block,
        KERNEL_RW,
        Page,
        SizeOf,
    },
};

use super::{
    AtomicInfo,
    Info,
};

pub(super) struct PagedAllocator {
    info: AtomicInfo,
}

impl PagedAllocator {
    pub(super) const fn new() -> Self {
        Self {
            info: AtomicInfo::new(),
        }
    }

    pub(super) fn info(&self) -> Info {
        self.info.load()
    }

    pub(super) unsafe fn allocate_pages(
        &self,
        layout: Layout,
    ) -> Result<*mut u8> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) unsafe fn deallocate_pages(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}
