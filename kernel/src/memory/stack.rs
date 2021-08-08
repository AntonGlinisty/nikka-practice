use core::mem;

use super::{
    Page,
    Virt,
};

#[repr(C, align(4096))]
pub(super) struct Stack([u8; STACK_SIZE]);

impl Stack {
    pub(super) const fn new() -> Self {
        Self([0; mem::size_of::<Self>()])
    }

    pub(super) fn pointer(&self) -> Virt {
        Virt::from_ptr(self.0.as_ptr_range().end)
    }
}

#[macro_export]
macro_rules! make_static_stack {
    () => {{
        #[link_section = ".stack"]
        static mut STACK: Stack = Stack::new();

        unsafe { STACK.pointer() }
    }};
}

const STACK_SIZE: usize = 32 * Page::SIZE;
