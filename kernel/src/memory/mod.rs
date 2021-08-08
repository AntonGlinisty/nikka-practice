pub mod mmu;
pub mod size;

pub(crate) mod gdt;

mod addr;
mod address_space;
mod frage;
mod mapping;
mod phys2virt;
mod stack;
mod tss;

use bootloader::BootInfo;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
};

use crate::log::info;

use address_space::{
    AddressSpace,
    BASE_ADDRESS_SPACE,
};

pub use addr::{
    Phys,
    Virt,
};
pub use frage::Page;

pub(crate) use tss::{
    DOUBLE_FAULT_IST_INDEX,
    PAGE_FAULT_IST_INDEX,
};

pub(super) fn init(boot_info: &'static BootInfo) {
    unsafe {
        Efer::write(EferFlags::NO_EXECUTE_ENABLE | Efer::read());
    }

    let phys2virt = phys2virt::phys2virt(boot_info);
    info!(?phys2virt);

    *BASE_ADDRESS_SPACE.lock() = AddressSpace::new("base");
}
