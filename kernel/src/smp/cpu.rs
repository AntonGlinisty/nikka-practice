use alloc::vec::Vec;

use x86_64::{
    instructions::tables,
    structures::tss::TaskStateSegment,
};

use crate::{
    error::Result,
    memory::{
        BASE_ADDRESS_SPACE,
        DOUBLE_FAULT_IST_INDEX,
        EXCEPTION_STACKS,
        GDT,
        Gdt,
        PAGE_FAULT_IST_INDEX,
        Stack,
    },
};

use super::{
    CpuId,
    LocalApic,
};

pub(super) fn init(
    cpu_count: usize,
    current_cpu: CpuId,
) -> Result<Vec<Cpu>> {
    let cpus = init_cpu_vec(cpu_count)?;

    cpus[usize::from(current_cpu)].set_tss();

    Ok(cpus)
}

fn init_cpu_vec(cpu_count: usize) -> Result<Vec<Cpu>> {
    // TODO: your code here.
    unimplemented!();
}

pub(super) struct Cpu {
    kernel_stack: &'static Stack,
    launch_escape_stack: &'static Stack,
    tss: TaskStateSegment,
}

impl Cpu {
    pub fn new(stacks: &'static [Stack]) -> Self {
        assert_eq!(stacks.len(), Self::STACKS_PER_CPU);

        let mut result = Self {
            kernel_stack: &stacks[0],
            launch_escape_stack: &stacks[1],
            tss: TaskStateSegment::new(),
        };

        // A double fault means there is an error in the kernel.
        // So a common double fault stack for all CPUs is sufficient as long as
        // double_fault() handler in trap/mod.rs locks and never unlocks a common mutex and does not return.
        result.tss.interrupt_stack_table[usize::from(DOUBLE_FAULT_IST_INDEX)] =
            EXCEPTION_STACKS.lock().double_fault_rsp().into();
        result.tss.interrupt_stack_table[usize::from(PAGE_FAULT_IST_INDEX)] =
            result.launch_escape_stack.pointer().into();

        result
    }

    pub(super) fn kernel_stack(&self) -> &'static Stack {
        self.kernel_stack
    }

    pub(super) fn set_tss(&self) {
        // TODO: your code here.
        unimplemented!();
    }

    const STACKS_PER_CPU: usize = 2;
}

impl Drop for Cpu {
    fn drop(&mut self) {
        panic!("a Cpu has been dropped");
    }
}
