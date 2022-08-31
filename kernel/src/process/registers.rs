use core::{
    arch::asm,
    fmt,
    mem,
};

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
    rax: usize,
    gpr1: [usize; 3],
    rdi: usize,
    rsi: usize,
    gpr2: [usize; 9],
    user_context: ModeContext,
}

impl Registers {
    pub(super) fn new(
        context: MiniContext,
        rdi: usize,
    ) -> Self {
        Self {
            // https://wiki.osdev.org/System_V_ABI#x86-64:
            // Parameters to functions are passed in the registers rdi, rsi, rdx, rcx, r8, r9, and further values are passed on the stack in reverse order.
            rax: 0,
            gpr1: [0; 3],
            rdi,
            rsi: 0,
            gpr2: [0; 9],
            user_context: ModeContext::user_context(context),
        }
    }

    pub(super) fn duplicate(
        &self,
        rax: usize,
        rdi: usize,
        rsi: usize,
    ) -> Self {
        Self {
            rax,
            gpr1: self.gpr1,
            rdi,
            rsi,
            gpr2: self.gpr2,
            user_context: self.user_context,
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

    pub(super) fn set_mode_context(
        &mut self,
        context: ModeContext,
    ) {
        self.user_context = context;
    }

    #[allow(named_asm_labels)]
    #[inline(never)] // For the named label to be a unique link symbol.
    pub(super) unsafe fn switch_to(registers: *mut Registers) {
        asm!(
            "
            // TODO: your code here.

            iretq

        store_user_mode_context:

            // TODO: your code here.

        switch_to_kernel_mode:

            // TODO: your code here.
            ",

            // TODO: your code here.
        );
    }

    #[naked]
    pub(super) unsafe extern "C" fn sched_yield() -> ! {
        asm!("jmp switch_to_kernel_mode", options(noreturn));
    }

    #[naked]
    pub(super) unsafe extern "C" fn switch_from() -> ! {
        asm!("jmp store_user_mode_context", options(noreturn));
    }
}

impl fmt::Display for Registers {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ rax: {:#X}, rdi: {:#X}, rsi: {:#X}, {} }}",
            self.rax, self.rdi, self.rsi, self.user_context,
        )
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

    pub(crate) fn kernel_context(rsp: Virt) -> Self {
        Self::new(
            Gdt::kernel_code(),
            Gdt::kernel_data(),
            MiniContext::new(Virt::from_ptr(Registers::switch_from as *const ()), rsp),
            RFlags::INTERRUPT_FLAG,
        )
    }

    pub(crate) fn mini_context(&self) -> MiniContext {
        MiniContext::new(self.rip, self.rsp)
    }

    pub(crate) fn set_mini_context(
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
pub(super) struct RFlags(pub(super) usize);

impl RFlags {
    pub(super) const INTERRUPT_FLAG: RFlags = RFlags(1 << 9);
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

pub(super) mod test_scaffolding {
    use super::{
        RFlags,
        Registers,
    };

    pub(in super::super) fn disable_interrupts(registers: &mut Registers) {
        registers.user_context.rflags =
            RFlags(registers.user_context.rflags.0 & !RFlags::INTERRUPT_FLAG.0);
    }

    pub(in super::super) fn registers(registers: &Registers) -> [usize; 15] {
        let mut result = [0; 15];
        result[0] = registers.rax;
        result[1 .. 4].copy_from_slice(&registers.gpr1);
        result[4] = registers.rdi;
        result[5] = registers.rsi;
        result[6 .. 15].copy_from_slice(&registers.gpr2);
        result
    }
}
