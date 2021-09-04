use core::mem;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::error::Result;

use super::{
    AddressSpace,
    BASE_ADDRESS_SPACE,
    KERNEL_RW,
    Page,
    Virt,
};

#[repr(C, align(4096))]
pub(crate) struct Stack([u8; STACK_SIZE]);

impl Stack {
    const fn new_static() -> Self {
        Self([0; mem::size_of::<Self>()])
    }

    pub(crate) fn new() -> Result<&'static mut Self> {
        let address_space = &mut *BASE_ADDRESS_SPACE.lock();
        let stack = address_space.map_slice::<u8>(STACK_SIZE, KERNEL_RW, 0)?;
        let stack = unsafe { &mut *(stack as *mut [u8] as *mut Stack) };

        unsafe {
            stack.make_guard_zone(address_space)?;
        }

        Ok(stack)
    }

    pub(crate) fn pointer(&self) -> Virt {
        Virt::from_ptr(self.0.as_ptr_range().end)
    }

    unsafe fn make_guard_zone(
        &mut self,
        address_space: &mut AddressSpace,
    ) -> Result<()> {
        address_space.unmap_slice(&mut self.0[.. GUARD_ZONE_SIZE])
    }
}

macro_rules! make_static_stack {
    () => {{
        #[link_section = ".stack"]
        static mut STACK: Stack = Stack::new_static();

        unsafe { &mut STACK }
    }};
}

pub(crate) struct ExceptionStacks {
    double_fault_stack: &'static mut Stack,
    page_fault_stack: &'static mut Stack,
}

impl ExceptionStacks {
    fn new() -> Self {
        Self {
            double_fault_stack: make_static_stack!(),
            page_fault_stack: make_static_stack!(),
        }
    }

    pub(crate) fn make_guard_zones(&mut self) -> Result<()> {
        let address_space = &mut BASE_ADDRESS_SPACE.lock();
        unsafe {
            self.double_fault_stack.make_guard_zone(address_space)?;
            self.page_fault_stack.make_guard_zone(address_space)
        }
    }

    pub(super) fn double_fault_rsp(&self) -> Virt {
        self.double_fault_stack.pointer()
    }

    pub(super) fn page_fault_rsp(&self) -> Virt {
        self.page_fault_stack.pointer()
    }
}

lazy_static! {
    pub(crate) static ref EXCEPTION_STACKS: Mutex<ExceptionStacks> =
        Mutex::new(ExceptionStacks::new());
}

pub const GUARD_ZONE_SIZE: usize = Page::SIZE;

const STACK_SIZE: usize = 32 * Page::SIZE;
