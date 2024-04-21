#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(ffi_returns_twice)]

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct Context {
    rip: usize,
    rsp: usize,
    rbp: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i64)]
pub enum ContextIs {
    Saved = 0,
    Loaded = 1,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }
}

extern "C" {
    #[ffi_returns_twice]
    pub fn save(context: &mut Context) -> ContextIs;

    pub fn load(context: &Context) -> !;
}
