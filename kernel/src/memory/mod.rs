/// Дополнительные функции для работы с
/// [Memory Management Unit](https://en.wikipedia.org/wiki/Memory_management_unit),
/// доступные только ядру.
pub mod mmu;

/// Глобальная таблица дескрипторов [`Gdt`]
/// ([Global Descriptor Table](https://en.wikipedia.org/wiki/Global_Descriptor_Table), GDT).
pub(crate) mod gdt;

/// Для простоты работы с физической памятью,
/// она целиком замаплена в некоторую область виртуальной.
pub(crate) mod phys2virt;

/// Абстракция адресного пространства [`AddressSpace`].
mod address_space;

/// Временный аллокатор физических фреймов [`BootFrameAllocator`].
mod boot_frame_allocator;

/// Аллокатор физических фреймов [`FrameAllocator`],
/// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
mod frame_allocator;

/// Основной аллокатор физических фреймов [`MainFrameAllocator`].
mod main_frame_allocator;

/// Реализация отображения виртуальной памяти в физическую [`Mapping`].
mod mapping;

/// Аллокатор страниц виртуальной памяти [`PageAllocator`].
mod page_allocator;

/// Диапазоны памяти в системе.
mod range;

/// Работа со стеками [`Stack`] и
/// выделенные стеки для непредвиденных исключений [`ExceptionStacks`].
mod stack;

/// Сегмент состояния задачи
/// ([Task State Segment](https://en.wikipedia.org/wiki/Task_state_segment), TSS).
mod tss;

use bootloader::BootInfo;
use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
};

pub use ku::{
    memory::{
        addr,
        block,
        frage,
        size,
    },
    sync::spinlock::Spinlock,
};

use crate::{
    Subsystems,
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
    KERNEL_R,
    KERNEL_RW,
    USER,
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

// Used in docs.
#[allow(unused)]
use self::{
    main_frame_allocator::MainFrameAllocator,
    mapping::Mapping,
    page_allocator::PageAllocator,
    stack::ExceptionStacks,
};

/// Инициализация подсистемы памяти.
pub(super) fn init(
    boot_info: &'static BootInfo,
    subsystems: Subsystems,
) {
    let timer = time::timer();

    unsafe {
        Efer::write(EferFlags::NO_EXECUTE_ENABLE | Efer::read());
    }

    let phys2virt = phys2virt::phys2virt(boot_info);
    info!(?phys2virt);

    if subsystems.contains(Subsystems::PHYS_MEMORY) {
        *FRAME_ALLOCATOR.lock() =
            FrameAllocator::Boot(BootFrameAllocator::new(&boot_info.memory_map));
    }

    if subsystems.contains(Subsystems::VIRT_MEMORY) {
        *BASE_ADDRESS_SPACE.lock() = AddressSpace::new(phys2virt);
    }

    if subsystems.contains(Subsystems::MAIN_FRAME_ALLOCATOR) {
        *FRAME_ALLOCATOR.lock() =
            FrameAllocator::Main(main_frame_allocator::init(&boot_info.memory_map));
    }

    info!(duration = %timer.elapsed(), "memory init");
}

#[doc(hidden)]
pub mod test_scaffolding {
    use super::{
        BootFrameAllocator,
        FrameAllocator,
    };

    pub use super::{
        address_space::test_scaffolding::*,
        boot_frame_allocator::test_scaffolding::*,
        frame_allocator::test_scaffolding::*,
        main_frame_allocator::test_scaffolding::*,
        mapping::test_scaffolding::*,
        phys2virt::test_scaffolding::*,
    };

    pub fn get_boot(frame_allocator: &mut FrameAllocator) -> Option<&mut BootFrameAllocator> {
        match frame_allocator {
            FrameAllocator::Boot(boot_allocator) => Some(boot_allocator),
            FrameAllocator::Main(..) | FrameAllocator::Void => None,
        }
    }
}
