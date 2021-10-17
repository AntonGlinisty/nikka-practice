#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![allow(clippy::missing_safety_doc)]
#![deny(warnings)]
#![feature(asm)]
#![feature(const_fn_trait_bound)]
#![feature(const_panic)]
#![feature(const_size_of_val)]
#![no_std]

extern crate rlibc;

pub mod error;
pub mod memory;
pub mod process;

pub use error::{
    Error,
    Result,
};
