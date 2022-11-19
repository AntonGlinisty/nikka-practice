#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

/// Аллокатор памяти общего назначения в пространстве пользователя, реализованный через
/// системные вызовы [`syscall::map()`], [`syscall::unmap()`] и [`syscall::copy_mapping()`].
mod map;

use core::{
    alloc::{
        Allocator,
        GlobalAlloc,
        Layout,
    },
    ptr::{
        self,
        NonNull,
    },
};

use ku::allocator::Info;

use map::MapAllocator;

// Used in docs.
#[allow(unused)]
use crate::syscall;

/// Статистика глобального аллокатора памяти общего назначения в пространстве пользователя.
pub fn info() -> Info {
    GLOBAL_ALLOCATOR.info()
}

#[alloc_error_handler]
#[cold]
#[inline(never)]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("failed to allocate memory, layout = {:?}", layout)
}

unsafe impl GlobalAlloc for MapAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        self.allocate(layout)
            .map(|x| x.as_non_null_ptr().as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        self.deallocate(NonNull::new(ptr).unwrap(), layout);
    }

    unsafe fn alloc_zeroed(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        self.allocate_zeroed(layout)
            .map(|x| x.as_non_null_ptr().as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let ptr = NonNull::new(ptr).unwrap();

        if let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) {
            let new_ptr = if layout.size() < new_layout.size() {
                self.grow(ptr, layout, new_layout)
            } else {
                self.shrink(ptr, layout, new_layout)
            };

            new_ptr.map(|x| x.as_non_null_ptr().as_ptr()).unwrap_or(ptr::null_mut())
        } else {
            ptr::null_mut()
        }
    }
}

/// Глобальный аллокатор памяти общего назначения в пространстве пользователя, реализованный через
/// системные вызовы [`syscall::map()`], [`syscall::unmap()`] и [`syscall::copy_mapping()`].
#[global_allocator]
static GLOBAL_ALLOCATOR: MapAllocator = MapAllocator::new();
