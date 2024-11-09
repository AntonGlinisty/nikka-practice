/// Дополнительные функции для работы с
/// [Memory Management Unit](https://en.wikipedia.org/wiki/Memory_management_unit),
/// доступные только ядру.
pub mod mmu;

/// Глобальная таблица дескрипторов [`Gdt`]
/// ([Global Descriptor Table](https://en.wikipedia.org/wiki/Global_Descriptor_Table), GDT).
pub(crate) mod gdt;

/// Абстракция адресного пространства [`AddressSpace`].
mod address_space;

/// Временный аллокатор физических фреймов [`BootFrameAllocator`].
mod boot_frame_allocator;

/// Аллокатор физических фреймов [`FrameAllocator`],
/// хранящий и оборачивающий либо [`BootFrameAllocator`], либо [`MainFrameAllocator`].
mod frame_allocator;

/// RAII для операции выделения одного [`Frame`].
mod frame_guard;

/// Основной аллокатор физических фреймов [`MainFrameAllocator`].
mod main_frame_allocator;

/// Отображённый блок адресного пространства [`MappedBlock`].
mod mapped_block;

/// Реализация отображения виртуальной памяти в физическую [`Mapping`].
mod mapping;

/// Аллокатор страниц виртуальной памяти [`PageAllocator`].
mod page_allocator;

/// Диапазоны памяти в системе.
mod range;

/// Для простоты работы с физической памятью,
/// она целиком отображена в некоторую область виртуальной.
/// [`Phys2Virt`] описывает это отображение.
mod phys2virt;

/// Работа со стеками [`Stack`] и
/// выделенные стеки для непредвиденных исключений [`ExceptionStacks`].
mod stack;

/// Сегмент состояния задачи
/// ([Task State Segment](https://en.wikipedia.org/wiki/Task_state_segment), TSS).
mod tss;

use bootloader::BootInfo;
use lazy_static::lazy_static;
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
    error::Result,
    log::{
        error,
        info,
    },
    time,
};

use boot_frame_allocator::BootFrameAllocator;
use frame_allocator::FrameAllocator;
use mapping::Mapping;
use range::PAGES_PER_ROOT_LEVEL_ENTRY;

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
pub(crate) use phys2virt::Phys2Virt;
pub(crate) use range::{
    is_kernel_block,
    is_user_block,
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
use {
    crate::{
        self as kernel,
        error::Error,
    },
    main_frame_allocator::MainFrameAllocator,
    mapped_block::MappedBlock,
    page_allocator::PageAllocator,
    stack::ExceptionStacks,
};

/// Инициализация подсистемы памяти.
pub(super) fn init(
    boot_info: &'static BootInfo,
    subsystems: Subsystems,
) -> Result<Phys2Virt> {
    let timer = time::timer();

    unsafe {
        Efer::write(EferFlags::NO_EXECUTE_ENABLE | Efer::read());
    }

    let physical_memory = range::physical(&boot_info.memory_map);

    if subsystems.contains(Subsystems::PHYS_MEMORY) {
        *FRAME_ALLOCATOR.lock() =
            FrameAllocator::Boot(BootFrameAllocator::new(&boot_info.memory_map));
    }

    let phys2virt = if cfg!(feature = "bootloader-phys2virt") {
        Phys2Virt::new(physical_memory, phys2virt::phys2virt(boot_info))
    } else {
        Phys2Virt::make(physical_memory, size::from(boot_info.recursive_index()))
    }?;

    info!(%phys2virt);

    if subsystems.contains(Subsystems::VIRT_MEMORY) {
        let page_table_root = Mapping::current_page_table_root();
        let mut address_space = AddressSpace::new(page_table_root, phys2virt);

        let page_table_root = unsafe { address_space.mapping().page_table_ref(page_table_root) };
        *KERNEL_PAGE_ALLOCATOR.lock() =
            PageAllocator::new(page_table_root, range::kernel_root_level_entries());

        *BASE_ADDRESS_SPACE.lock() = address_space;
    }

    if subsystems.contains(Subsystems::MAIN_FRAME_ALLOCATOR) {
        if let FrameAllocator::Boot(boot_frame_allocator) = &*FRAME_ALLOCATOR.lock() {
            assert!(boot_frame_allocator.is_consistent());
        }

        *FRAME_ALLOCATOR.lock() = FrameAllocator::Main(main_frame_allocator::init(
            physical_memory,
            &boot_info.memory_map,
        ));
    }

    if subsystems.contains(Subsystems::VIRT_MEMORY | Subsystems::MAIN_FRAME_ALLOCATOR) &&
        let Err(error) = EXCEPTION_STACKS.lock().make_guard_zones()
    {
        error!(?error, "failed to make guard zones for the trap stacks");
    }

    info!(duration = %timer.elapsed(), "memory init");

    Ok(phys2virt)
}

lazy_static! {
    /// Аллокатор виртуальных страниц для ядра.
    static ref KERNEL_PAGE_ALLOCATOR: Spinlock<PageAllocator> =
        Spinlock::new(PageAllocator::default());
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
        mapping::test_scaffolding::*,
        page_allocator::test_scaffolding::*,
        phys2virt::test_scaffolding::*,
        range::test_scaffolding::*,
    };

    pub fn get_boot(frame_allocator: &mut FrameAllocator) -> Option<&mut BootFrameAllocator> {
        match frame_allocator {
            FrameAllocator::Boot(boot_allocator) => Some(boot_allocator),
            FrameAllocator::Main(..) | FrameAllocator::Void => None,
        }
    }
}
