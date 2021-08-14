pub mod block;
pub mod mmu;
pub mod size;

pub(crate) mod gdt;

mod addr;
mod address_space;
mod boot_frame_allocator;
mod frage;
mod frame_allocator;
mod mapping;
mod page_allocator;
mod phys2virt;
mod stack;
mod tss;

use bootloader::BootInfo;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
};

use crate::log::info;

use address_space::AddressSpace;
use boot_frame_allocator::BootFrameAllocator;
use frame_allocator::FrameAllocator;

pub use addr::{
    Phys,
    Virt,
};
pub use address_space::BASE_ADDRESS_SPACE;
pub use frage::{
    Frame,
    Page,
};
pub use frame_allocator::FRAME_ALLOCATOR;

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

    *FRAME_ALLOCATOR.lock() = FrameAllocator::Boot(BootFrameAllocator::new(&boot_info.memory_map));

    *BASE_ADDRESS_SPACE.lock() = AddressSpace::new("base", phys2virt);
}
