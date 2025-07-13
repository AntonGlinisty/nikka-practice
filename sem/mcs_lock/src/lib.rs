#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_std]
#![deny(warnings)]

mod sync;

pub use sync::{
    McsLock,
    McsLockGuard,
    McsLockNode,
};
