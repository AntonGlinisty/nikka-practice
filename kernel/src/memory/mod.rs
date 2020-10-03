pub(crate) mod gdt;

mod stack;
mod tss;

pub(crate) use self::tss::{
    DOUBLE_FAULT_IST_INDEX,
    PAGE_FAULT_IST_INDEX,
};
