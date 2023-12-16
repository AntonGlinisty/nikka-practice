/// Поведение блокировок при панике.
pub mod panic;

/// Примитив синхронизации [`SequenceLock`].
pub mod sequence_lock;

/// Примитив синхронизации [`Spinlock`].
pub mod spinlock;

pub use panic::{
    PanicStrategy,
    start_panicking,
};
pub use sequence_lock::SequenceLock;
pub use spinlock::Spinlock;
