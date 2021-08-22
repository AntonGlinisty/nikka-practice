use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;

use super::{
    EXCEPTION_STACKS,
    size,
};

lazy_static! {
    pub(super) static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        let stacks = EXCEPTION_STACKS.lock();

        tss.interrupt_stack_table[size::from(DOUBLE_FAULT_IST_INDEX)] =
            stacks.double_fault_rsp().into();
        tss.interrupt_stack_table[size::from(PAGE_FAULT_IST_INDEX)] =
            stacks.page_fault_rsp().into();

        tss
    };
}

pub(crate) const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub(crate) const PAGE_FAULT_IST_INDEX: u16 = 1;
