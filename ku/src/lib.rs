#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![allow(clippy::missing_safety_doc)]
#![deny(warnings)]
#![feature(cell_update)]
#![feature(const_size_of_val)]
#![no_std]

extern crate rlibc;

pub mod error;
pub mod info;
pub mod ipc;
pub mod log;

/// Здесь собраны базовые примитивы для работы с памятью, которые нужны и в ядре, и в пространстве пользователя.
pub mod memory;

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
    now,
    now_ms,
    timer,
    tsc,
};
