use derive_more::{
    Deref,
    DerefMut,
};

/// Выравнивает аргумент на удвоенный размер линии кэша, чтобы избежать
/// [ложной конкуренции](https://en.wikipedia.org/wiki/False_sharing)
/// за данные.
///
/// Выравнивание делается на удвоенный размер аналогично
/// [`crossbeam::utils::CachePadded`](https://docs.rs/crossbeam/latest/crossbeam/utils/struct.CachePadded.html#size-and-alignment).
/// для учёта
/// [упреждающей загрузки](https://en.wikipedia.org/wiki/Prefetching)
/// линий кэша.
/// Или
/// [упреждающей загрузки](https://en.wikipedia.org/wiki/Prefetching)
/// линий кэша парами, как делается в некоторых современных процессорах.
#[derive(Clone, Copy, Default, Deref, DerefMut, Eq, Ord, PartialEq, PartialOrd)]
#[repr(align(128))]
pub struct CacheAligned<T>(pub T);

impl<T> CacheAligned<T> {
    /// Выравнивает аргумент на удвоенный размер линии кэша, чтобы избежать
    /// [ложной конкуренции](https://en.wikipedia.org/wiki/False_sharing)
    /// за данные.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}
