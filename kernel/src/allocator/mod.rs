mod info;
mod paged;

use core::{
    alloc::{
        GlobalAlloc,
        Layout,
    },
    ptr,
};

use crate::log::{
    info,
    trace,
};

pub use info::{
    AtomicInfo,
    Info,
};

unsafe impl GlobalAlloc for paged::PagedAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        let ptr = self.allocate_pages(layout).unwrap_or(ptr::null_mut());
        trace!(?ptr, ?layout, "alloc");
        ptr
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        trace!(?ptr, ?layout, "dealloc");
        self.deallocate_pages(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: paged::PagedAllocator = paged::PagedAllocator::new();

pub fn info() -> Info {
    let info = ALLOCATOR.info();

    info!(
        allocations = info.allocations(),
        deallocations = info.deallocations(),
        current_allocations = info.current_allocations(),
        allocated_pages = info.allocated_pages(),
        deallocated_pages = info.deallocated_pages(),
        current_pages = info.current_pages(),
        requested = info.requested(),
        allocated = info.allocated(),
        fragmentation_loss = info.fragmentation_loss(),
        fragmentation_loss_percentage =
            format_args!("{:.3}%", info.fragmentation_loss_percentage()),
        "allocator info",
    );

    info
}
