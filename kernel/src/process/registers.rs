use core::fmt;

use x86_64::structures::gdt::SegmentSelector;

use ku::process::MiniContext;

use crate::{
    gdt::Gdt,
    memory::Virt,
    smp::KERNEL_RSP_OFFSET_IN_CPU,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct Registers {
    rdi: usize,
    user_context: ModeContext,
}

impl Registers {
    pub(super) fn new(
        context: MiniContext,
        arg0: usize,
    ) -> Self {
        Self {
            // https://wiki.osdev.org/System_V_ABI#x86-64:
            // Parameters to functions are passed in the registers rdi, rsi, rdx, rcx, r8, r9, and further values are passed on the stack in reverse order.
            rdi: arg0,
            user_context: ModeContext::user_context(context),
        }
    }

    pub(super) fn mini_context(&self) -> MiniContext {
        self.user_context.mini_context()
    }

    pub(super) fn set_mini_context(
        &mut self,
        context: MiniContext,
    ) {
        self.user_context.set_mini_context(context);
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

/// `iret` instruction layout.
/// It is the minimal register context that is necessary for switching between the kernel mode and the user mode.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub(crate) struct ModeContext {
    rip: Virt,
    cs: usize,
    rflags: RFlags,
    rsp: Virt,
    ss: usize,
}

impl ModeContext {
    fn new(
        code: SegmentSelector,
        data: SegmentSelector,
        context: MiniContext,
        rflags: RFlags,
    ) -> Self {
        Self {
            rip: context.rip(),
            cs: code.0.into(),
            rflags,
            rsp: context.rsp(),
            ss: data.0.into(),
        }
    }

    fn user_context(context: MiniContext) -> Self {
        Self::new(
            Gdt::user_code(),
            Gdt::user_data(),
            context,
            RFlags::INTERRUPT_FLAG,
        )
    }

    fn mini_context(&self) -> MiniContext {
        MiniContext::new(self.rip, self.rsp)
    }

    fn set_mini_context(
        &mut self,
        context: MiniContext,
    ) {
        self.rip = context.rip();
        self.rsp = context.rsp();
    }

    pub(crate) fn is_user_mode(&self) -> bool {
        assert_eq!(
            Self::is_user_mode_segment(self.cs),
            Self::is_user_mode_segment(self.ss),
        );

        Self::is_user_mode_segment(self.cs)
    }

    fn is_user_mode_segment(segment_register: usize) -> bool {
        segment_register & Self::CPL_MASK != Self::RING_0
    }

    fn mode(&self) -> &'static str {
        if self.is_user_mode() {
            "user"
        } else {
            "kernel"
        }
    }

    const CPL_MASK: usize = 0x3;
    const RING_0: usize = 0x0;
}

impl fmt::Display for ModeContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ mode: {}, cs:rip: {:#06X}:{}, ss:rsp: {:#06X}:{}, rflags: {} }}",
            self.mode(),
            self.cs,
            self.rip,
            self.ss,
            self.rsp,
            self.rflags,
        )
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct RFlags(usize);

impl RFlags {
    const INTERRUPT_FLAG: RFlags = RFlags(1 << 9);
}

impl fmt::Display for RFlags {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        static FLAGS: [&str; 22] = [
            "CF",
            "",
            "PF",
            "",
            "AF",
            "",
            "ZF",
            "SF",
            "TF",
            "IF",
            "DF",
            "OF",
            "",
            "",
            "NT",
            "",
            "RF",
            "VM",
            "AC",
            "VF",
            "VP",
            "ID",
        ];

        let mut separator = "";

        for i in (0 .. FLAGS.len()).rev() {
            if (self.0 >> i) & 1 != 0 && !FLAGS[i].is_empty() {
                write!(formatter, "{}{}", separator, FLAGS[i])?;
                separator = " ";
            }
        }

        Ok(())
    }
}
