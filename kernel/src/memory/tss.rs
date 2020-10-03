use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;

use crate::make_static_stack;

use super::stack::Stack;

lazy_static! {
    pub(super) static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[usize::from(DOUBLE_FAULT_IST_INDEX)] = make_static_stack!();
        tss.interrupt_stack_table[usize::from(PAGE_FAULT_IST_INDEX)] = make_static_stack!();

        tss
    };
}

pub(crate) const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub(crate) const PAGE_FAULT_IST_INDEX: u16 = 1;
