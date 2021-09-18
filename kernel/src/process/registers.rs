use core::mem;

use static_assertions::const_assert;
use x86_64::{
    registers::rflags::RFlags,
    structures::gdt::SegmentSelector,
};

use crate::{
    memory::Virt,
    smp::KERNEL_RSP_OFFSET_IN_CPU,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct Registers {
    // The following layout is dictated by the 'iret' instruction which is used to jump into the user space.
    rip: Virt,
    cs: usize,
    rflags: RFlags,
    rsp: Virt,
    ss: usize,
}

const_assert!(mem::size_of::<Registers>() == 5 * mem::size_of::<usize>());

impl Registers {
    pub(super) fn new(
        code: SegmentSelector,
        data: SegmentSelector,
        rip: Virt,
        rsp: Virt,
    ) -> Self {
        Self {
            rip,
            cs: code.0.into(),
            rflags: RFlags::INTERRUPT_FLAG,
            rsp,
            ss: data.0.into(),
        }
    }

    pub(super) fn rip(&self) -> Virt {
        self.rip
    }

    pub(super) fn rsp(&self) -> Virt {
        self.rsp
    }

    #[allow(named_asm_labels)]
    #[inline(never)] // For the named label to be a unique link symbol.
    pub(super) unsafe fn switch_to(registers: *const Registers) {
        asm!(
            "
            // TODO: your code here.

            // Load the user space context and switch into the user space.
            iretq

        switch_to_kernel_mode:

            // TODO: your code here.
            ",

            // TODO: your code here.
        );
    }

    #[naked]
    pub(super) unsafe extern "C" fn switch_from() -> ! {
        asm!("jmp switch_to_kernel_mode", options(noreturn));
    }
}
