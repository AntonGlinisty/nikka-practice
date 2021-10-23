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
    rdi: usize,

    // The following layout is dictated by the 'iret' instruction which is used to jump into the user space.
    rip: Virt,
    cs: usize,
    rflags: RFlags,
    rsp: Virt,
    ss: usize,
}

impl Registers {
    pub(super) fn new(
        code: SegmentSelector,
        data: SegmentSelector,
        rip: Virt,
        rsp: Virt,
        arg0: usize,
    ) -> Self {
        Self {
            // https://wiki.osdev.org/System_V_ABI#x86-64:
            // Parameters to functions are passed in the registers rdi, rsi, rdx, rcx, r8, r9, and further values are passed on the stack in reverse order.
            rdi: arg0,

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
            pop rdi
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
