/// Аллокатор памяти, предназначенный для выделения памяти блоками виртуальных страниц.
mod big;

/// Аллокатор памяти общего назначения внутри [`AddressSpace`].
mod memory_allocator;

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

use crate::memory::{
    BASE_ADDRESS_SPACE,
    KERNEL_RW,
};

pub(crate) use big::Big;
pub(crate) use memory_allocator::MemoryAllocator;

// Used in docs.
#[allow(unused)]
use crate::memory::AddressSpace;

/// Статистика глобального аллокатора памяти общего назначения для ядра.
pub fn info() -> Info {
    GLOBAL_ALLOCATOR.info()
}

/// Глобальный аллокатор памяти общего назначения для ядра.
/// Выделяет память и отображает её внутри [`struct@BASE_ADDRESS_SPACE`].
struct GlobalAllocator;

impl GlobalAllocator {
    /// Статистика аллокатора общего назначения.
    fn info(&self) -> Info {
        BASE_ADDRESS_SPACE.lock().info()
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        MemoryAllocator::new(&BASE_ADDRESS_SPACE, KERNEL_RW)
            .allocate(layout)
            .map(|x| x.as_non_null_ptr().as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        MemoryAllocator::new(&BASE_ADDRESS_SPACE, KERNEL_RW)
            .deallocate(NonNull::new(ptr).unwrap(), layout);
    }

    unsafe fn alloc_zeroed(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        MemoryAllocator::new(&BASE_ADDRESS_SPACE, KERNEL_RW)
            .allocate_zeroed(layout)
            .map(|x| x.as_non_null_ptr().as_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let allocator = MemoryAllocator::new(&BASE_ADDRESS_SPACE, KERNEL_RW);
        let ptr = NonNull::new(ptr).unwrap();

        if let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) {
            let new_ptr = if layout.size() < new_layout.size() {
                allocator.grow(ptr, layout, new_layout)
            } else {
                allocator.shrink(ptr, layout, new_layout)
            };

            new_ptr.map(|x| x.as_non_null_ptr().as_ptr()).unwrap_or(ptr::null_mut())
        } else {
            ptr::null_mut()
        }
    }
}

/// Глобальный аллокатор памяти общего назначения для ядра.
/// Выделяет память и отображает её внутри [`struct@BASE_ADDRESS_SPACE`].
#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;
