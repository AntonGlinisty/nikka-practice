pub mod mmu;

pub(crate) mod gdt;
pub(crate) mod phys2virt;

mod address_space;
mod boot_frame_allocator;
mod frame_allocator;
mod main_frame_allocator;
mod mapping;
mod page_allocator;
mod range;
mod stack;
mod tss;

use bootloader::BootInfo;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
};

pub use ku::memory::{
    addr,
    block,
    frage,
    size,
};

use crate::{
    log::info,
    time,
};

use boot_frame_allocator::BootFrameAllocator;
use frame_allocator::FrameAllocator;

pub use addr::{
    Phys,
    Virt,
};
pub use address_space::{
    AddressSpace,
    BASE_ADDRESS_SPACE,
};
pub use block::Block;
pub use frage::{
    Frame,
    Page,
};
pub use frame_allocator::FRAME_ALLOCATOR;
pub use mmu::{
    KERNEL_RW,
    USER_R,
    USER_RW,
};
pub use size::{
    Size,
    SizeOf,
};

pub(crate) use gdt::{
    GDT,
    Gdt,
    RealModePseudoDescriptor,
    SmallGdt,
};
pub(crate) use stack::{
    EXCEPTION_STACKS,
    Stack,
};
pub(crate) use tss::{
    DOUBLE_FAULT_IST_INDEX,
    PAGE_FAULT_IST_INDEX,
};

pub(super) fn init(boot_info: &'static BootInfo) {
    let timer = time::timer();

    unsafe {
        Efer::write(EferFlags::NO_EXECUTE_ENABLE | Efer::read());
    }

    let phys2virt = phys2virt::phys2virt(boot_info);
    info!(?phys2virt);

    *FRAME_ALLOCATOR.lock() = FrameAllocator::Boot(BootFrameAllocator::new(&boot_info.memory_map));

    *BASE_ADDRESS_SPACE.lock() = AddressSpace::new(phys2virt);

    *FRAME_ALLOCATOR.lock() =
        FrameAllocator::Main(main_frame_allocator::init(&boot_info.memory_map));

    info!(duration = %timer.elapsed(), "memory init");
}
