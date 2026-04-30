/// [`CacheAligned`] --- выравнивание на удвоенный размер линии кэша, чтобы избежать
/// [ложной конкуренции за данные](https://en.wikipedia.org/wiki/False_sharing).
pub mod cache_aligned;

/// Примитив синхронизации [`IrqSpinlock`].
pub mod irq_spinlock;

/// Примитив синхронизации [`OnceLock`] для данных,
/// которые один раз записываются, а потом только читаются.
pub mod once_lock;

/// Поведение блокировок при панике.
pub mod panic;

/// Примитив синхронизации [`SequenceLock`].
pub mod sequence_lock;

/// MPMC квазиочередь [`ShardedMpmc`].
pub mod sharded_mpmc;

/// Примитив синхронизации [`Spinlock`].
pub mod spinlock;

/// SPSC очередь без ожидания [`Spsc`].
pub mod spsc;

pub use cache_aligned::CacheAligned;
pub use irq_spinlock::{
    IrqSpinlock,
    IrqSpinlockGuard,
};
pub use once_lock::OnceLock;
pub use panic::{
    PanicStrategy,
    start_panicking,
};
pub use sequence_lock::SequenceLock;
pub use sharded_mpmc::ShardedMpmc;
pub use spinlock::{
    FastSpinlock,
    FastSpinlockGuard,
    Spinlock,
    SpinlockGuard,
};
pub use spsc::Spsc;
