#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![allow(clippy::missing_safety_doc)]
#![deny(warnings)]
#![feature(allocator_api)]
#![feature(cell_update)]
#![feature(const_size_of_val)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![no_std]

extern crate rlibc;

/// Аллокаторы памяти общего назначения.
pub mod allocator;

pub mod error;

/// Информации о системе, доступная пользовательским процессам.
///
/// Ядро предоставляет пользовательским процессам часть информации о системе,
/// сохраняя её в памяти, доступной пользователю на чтение.
/// Эта информация собирается в виде структуры общей информации о системе [`SystemInfo`]
/// и в виде структуры с информацией о текущем процессе [`ProcessInfo`].
pub mod info;

pub mod ipc;

pub mod log;

/// Здесь собраны базовые примитивы для работы с памятью, которые нужны и в ядре, и в пространстве пользователя.
pub mod memory;

/// Здесь собраны функции и структуры для работы с процессами, которые нужны и в ядре, и в пространстве пользователя.
pub mod process;

/// Здесь собраны базовые примитивы для работы со временем, которые нужны и в ядре, и в пространстве пользователя.
pub mod time;

pub use error::{
    Error,
    Result,
};
pub use info::{
    ProcessInfo,
    SystemInfo,
    process_info,
    set_process_info,
    set_system_info,
    system_info,
};
pub use ipc::pipe::{
    self,
    RingBuffer,
    RingBufferMemory,
    RingBufferReadTx,
    RingBufferWriteTx,
};
pub use time::{
    Hz,
    Tsc,
    TscDuration,
    delay,
    now,
    now_ms,
    timer,
    tsc,
};
