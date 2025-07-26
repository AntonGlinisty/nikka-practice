/// Интерфейс аллокатора памяти [`BigAllocator`],
/// который умеет выдавать только выровненные на границу страниц блоки памяти.
pub mod big;

/// Кэш выделяемых блоков.
pub mod cache;

/// Определяет [`DryAllocator`], который аналогичен [`core::alloc::Allocator`],
/// и позволяет не дублировать код.
pub mod dry;

/// Статистика аллокатора общего назначения.
#[cfg_attr(not(feature = "allocator-statistics"), path = "dummy_info.rs")]
mod info;

/// Аллокатор верхнего уровня.
/// По запрошенному размеру определяет из какого аллокатора будет выделяться память.
mod dispatcher;

/// Аллокатор, выделяющий память блоками одинакового размера.
mod fixed_size;

/// Вспомогательная структура [`Quarry`] для [`FixedSizeAllocator`].
mod quarry;

pub use big::{
    BigAllocator,
    BigAllocatorGuard,
};
pub use cache::{
    Cache,
    Clip,
    GlobalCache,
};
pub use dispatcher::{
    DetailedInfo,
    Dispatcher,
    FIXED_SIZE_COUNT,
};
pub use dry::{
    DryAllocator,
    Initialize,
};
pub use info::{
    AtomicInfo,
    Info,
};

use fixed_size::FixedSizeAllocator;
use quarry::Quarry;

#[doc(hidden)]
pub mod test_scaffolding {
    pub use super::quarry::test_scaffolding::*;
}
