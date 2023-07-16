/// Примитив синхронизации [`SequenceLock`].
pub mod sequence_lock;

/// Примитив синхронизации [`Spinlock`].
pub mod spinlock;

pub use sequence_lock::SequenceLock;
pub use spinlock::Spinlock;
