/// Интерфейс аллокатора памяти [`BigAllocator`],
/// который умеет выдавать только выровненные на границу страниц блоки памяти.
pub mod big;

/// Определяет [`DryAllocator`], который аналогичен [`core::alloc::Allocator`],
/// и позволяет не дублировать код.
pub mod dry;

/// Статистика аллокатора общего назначения.
mod info;

pub use big::BigAllocator;
pub use dry::{
    DryAllocator,
    Initialize,
};
pub use info::{
    AtomicInfo,
    Info,
};
