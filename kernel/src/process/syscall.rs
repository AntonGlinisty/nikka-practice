use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
    LStar,
    Star,
};

use ku::process::{
    ExitCode,
    Syscall,
};

use crate::{
    gdt::Gdt,
    log::{
        debug,
        error,
        info,
        trace,
    },
    memory::Virt,
    smp::{
        Cpu,
        KERNEL_RSP_OFFSET_IN_CPU,
    },
};

use super::Process;

pub(crate) fn init() {
    // TODO: your code here.
    unimplemented!();
}

#[naked]
extern "C" fn syscall_trampoline() -> ! {
    unsafe {
        asm!(
            "
            // TODO: your code here.

            jmp {syscall}
            ",

            // TODO: your code here.
            syscall = sym syscall,
            options(noreturn),
        );
    }
}

extern "C" fn syscall(
    // https://wiki.osdev.org/System_V_ABI#x86-64:
    // Parameters to functions are passed in the registers rdi, rsi, rdx, rcx, r8, r9, and further values are passed on the stack in reverse order.
    number: usize, // rdi
    arg0: usize,   // rsi
    arg1: usize,   // rdx
    rip: Virt,     // rcx
    rsp: Virt,     // r8
    arg2: usize,   // r9
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

fn exit(code: usize) -> ! {
    // TODO: your code here.
    unimplemented!();
}
