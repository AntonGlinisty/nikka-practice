#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use core::arch::asm;

use lib::entry;

entry!(main);

fn main() {
    unsafe {
        asm!(
            "
            // Test that the kernel stores the user mode context properly.
            mov rax, 77701
            mov rbx, 77702
            mov rcx, 77703
            mov rdx, 77704
            mov rdi, 77705
            mov rsi, 77706
            mov rbp, 77707
            mov r8, 77708
            mov r9, 77709
            mov r10, 77710
            mov r11, 77711
            mov r12, 77712
            mov r13, 77713
            mov r14, 77714
            mov r15, 77715

            // Test that the kernel switches into its own stack.
            xor rsp, rsp

            // Test that the user mode Page Fault is handled.
            mov rsp, [rsp]
            ",
            options(noreturn),
        );
    }
}
