use std::{
    alloc::{
        AllocError,
        Allocator,
        Layout,
    },
    ffi::c_void,
    ptr::NonNull,
    result,
};

use ku::allocator::BigAllocator;

use super::Big;

pub struct Fallback;

unsafe impl Allocator for Fallback {
    fn allocate(
        &self,
        layout: Layout,
    ) -> result::Result<NonNull<[u8]>, AllocError> {
        let ptr = if layout.size() == 0 {
            layout.align() as *mut u8
        } else {
            let mut big = Big::new();
            let block = big.reserve(layout).map_err(|_| AllocError)?;
            let flags = big.flags();

            unsafe {
                big.map(block, flags).map_err(|_| AllocError)?;
            }

            big.last().start_address().try_into_mut_ptr().unwrap()
        };

        NonNull::new(ptr)
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .ok_or(AllocError)
    }

    unsafe fn deallocate(
        &self,
        mut ptr: NonNull<u8>,
        layout: Layout,
    ) {
        unsafe {
            let block =
                super::block(ptr.as_mut() as *mut u8 as *mut c_void, layout.size()).unwrap();
            Big::new().unmap(block).unwrap();
        }
    }
}
