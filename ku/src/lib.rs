#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! Общая для пространств ядра и пользователя библиотека.
//! ku --- **k**ernel && **u**ser.

#![allow(clippy::missing_safety_doc)]
#![deny(warnings)]
#![feature(allocator_api)]
#![feature(cell_update)]
#![feature(const_size_of_val)]
#![feature(slice_ptr_get)]
#![feature(strict_provenance)]
#![no_std]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

extern crate rlibc;

/// Аллокаторы памяти общего назначения.
pub mod allocator;

/// Поддержка печати трассировок стека.
pub mod backtrace;

/// Перечисление для возможных ошибок [`Error`] и соответствующий [`Result`].
pub mod error;

/// Информации о системе, доступная пользовательским процессам.
///
/// Ядро предоставляет пользовательским процессам часть информации о системе,
/// сохраняя её в памяти, доступной пользователю на чтение.
/// Эта информация собирается в виде структуры общей информации о системе [`SystemInfo`]
/// и в виде структуры с информацией о текущем процессе [`ProcessInfo`].
pub mod info;

/// [Межпроцессное взаимодействие (Inter-process communication, IPC)](https://en.wikipedia.org/wiki/Inter-process_communication).
pub mod ipc;

/// Поддержка журналирования макросами библиотеки [`tracing`].
///
/// Сериализует сообщения в [`RingBuffer`] и
/// передаёт их для журналирования в ядро через [`ProcessInfo::log()`].
/// Сериализация осуществляется с помощью [`serde`] в формате [`postcard`].
pub mod log;

/// Здесь собраны базовые примитивы для работы с памятью,
/// которые нужны и в ядре, и в пространстве пользователя.
pub mod memory;

/// Здесь собраны функции и структуры для работы с процессами,
/// которые нужны и в ядре, и в пространстве пользователя.
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
