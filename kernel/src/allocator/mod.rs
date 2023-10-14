/// Аллокатор памяти, предназначенный для выделения памяти блоками виртуальных страниц.
mod big;

/// Аллокатор памяти общего назначения внутри [`AddressSpace`].
mod memory_allocator;

use ku::{
    allocator::{
        Dispatcher,
        Info,
    },
    log::debug,
};

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

/// Распечатывает детальную статистику аллокатора.
pub fn dump_info() {
    debug!(allocator_info = %GLOBAL_ALLOCATOR);
}

/// Обработчик ошибок выделения памяти.
#[alloc_error_handler]
#[cold]
#[inline(never)]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("failed to allocate memory, layout = {:?}", layout)
}

/// Глобальный аллокатор памяти общего назначения для ядра.
/// Выделяет память и отображает её внутри [`BASE_ADDRESS_SPACE`].
#[global_allocator]
static GLOBAL_ALLOCATOR: Dispatcher<MemoryAllocator<'static>> =
    Dispatcher::new(MemoryAllocator::new(&BASE_ADDRESS_SPACE, KERNEL_RW));
