mod big;
mod big_for_pipe;
mod caching_big;
mod fallback;

#[allow(unused)]
pub use big::Big;

#[allow(unused)]
pub use big_for_pipe::BigForPipe;

#[allow(unused)]
pub use caching_big::CachingBig;

#[allow(unused)]
pub use fallback::Fallback;

use std::ffi::c_void;

use ku::{
    error::Result,
    memory::{
        Block,
        Page,
        Virt,
    },
};

fn block(
    ptr: *mut c_void,
    len: usize,
) -> Result<Block<Page>> {
    let virt = Virt::from_ptr(ptr);
    let start = Page::new(virt)?;
    let end = (start + len.div_ceil(Page::SIZE))?;

    Block::new(start, end)
}
