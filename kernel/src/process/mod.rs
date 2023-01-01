/// Загружает [ELF--файлы](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// пользовательских процессов и их адресные пространства.
mod elf;

/// Содержит структуру пользовательского процесса [`Process`].
#[allow(clippy::module_inception)]
mod process;

/// Описывает состояние регистров процесса [`Registers`] и
/// контекст исполнения содержащий уровень привилегий [`ModeContext`].
mod registers;

/// Планировщик процессов.
/// Реализует простейшее
/// [циклическое исполнение процессов](https://en.wikipedia.org/wiki/Round-robin_scheduling).
mod scheduler;

/// Реализует системные вызовы.
mod syscall;

/// Таблица процессов.
mod table;

use core::mem;

use static_assertions::const_assert;

use ku::{
    ProcessInfo,
    RingBuffer,
    SystemInfo,
    process::MiniContext,
};

use crate::{
    SYSTEM_INFO,
    Subsystems,
    error::{
        Error::NoPage,
        Result,
    },
    log::info,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        FRAME_ALLOCATOR,
        Page,
        Size,
        Stack,
        USER_R,
        USER_RW,
        Virt,
        mmu::PageTableFlags,
    },
};

use process::TrapHandlerContext;
use table::TABLE;

pub use ku::process::Pid;

pub use process::Process;
pub use scheduler::Scheduler;
pub use table::Table;

pub(crate) use registers::{
    ModeContext,
    Registers,
};

/// Инициализация подсистемы процессов.
pub fn init(subsystems: Subsystems) {
    if subsystems.contains(Subsystems::SYSCALL) {
        syscall::init();
    }

    if subsystems.contains(Subsystems::PROCESS_TABLE) {
        *TABLE.lock() = Table::new(PROCESS_SLOT_COUNT);
    }

    if subsystems.contains(Subsystems::SCHEDULER) {
        Scheduler::init(PROCESS_SLOT_COUNT);
    }
}

/// Создаёт процесс для заданного
/// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// `elf_file`,
/// вставляет его в таблицу процессов и
/// возвращает его идентификатор.
pub fn create(elf_file: &[u8]) -> Result<Pid> {
    Table::allocate(create_process(elf_file)?)
}

/// Создаёт процесс для заданного
/// [ELF--файла](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
/// `elf_file` и возвращает его.
fn create_process(elf_file: &[u8]) -> Result<Process> {
    let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate()?;

    address_space.switch_to();

    let entry = elf::load(&mut address_space, elf_file)?;

    let stack = Stack::new(&mut address_space, USER_RW)?;

    let process_info = map_process_info(&mut address_space)?;

    BASE_ADDRESS_SPACE.lock().switch_to();

    let context = MiniContext::new(entry, stack.pointer());
    let process = Process::new(address_space, context, process_info);

    info!(%context, file_size = %Size::from_slice(elf_file), %process, "loaded ELF file");

    Ok(process)
}

/// Создаёт для процесса отображение в его адресное пространство `address_space`
/// страниц с общей информацией о системе и с информацией о самом процессе.
/// См. [`SystemInfo`] и [`ProcessInfo`].
fn map_process_info(address_space: &mut AddressSpace) -> Result<&'static mut ProcessInfo> {
    let log = map_log(address_space)?;
    let recursive_mapping = address_space.mapping().make_recursive_mapping()?;
    let system_info = map_system_info(address_space)?;
    address_space.map_one(USER_RW, || {
        ProcessInfo::new(log, recursive_mapping, system_info)
    })
}

/// Создаёт для процесса отображение в его адресное пространство `address_space`
/// циклического буфера для журналирования.
/// См. [`RingBuffer`].
fn map_log(address_space: &mut AddressSpace) -> Result<RingBuffer> {
    // TODO: your code here.
    Ok(RingBuffer::zero()) // TODO: remove before flight.
}

/// Создаёт для процесса отображение в его адресное пространство `address_space`
/// страницы с общей информацией о системе.
/// См. [`SystemInfo`].
fn map_system_info(address_space: &mut AddressSpace) -> Result<*const SystemInfo> {
    let system_info_frame = address_space
        .mapping()
        .translate(Virt::from_ref(&SYSTEM_INFO), None, PageTableFlags::empty())?
        .frame()?;

    let system_info_page = address_space
        .allocate(mem::size_of_val(&SYSTEM_INFO))?
        .into_iter()
        .next()
        .ok_or(NoPage)?;

    FRAME_ALLOCATOR.lock().reference(system_info_frame);

    unsafe {
        address_space.map_page_to_frame(system_info_page, system_info_frame, USER_R)?;
    }

    system_info_page.address().try_into_ptr()
}

/// Максимальное количество одновременно работающих процессов.
const PROCESS_SLOT_COUNT: usize = 1 << 8;

#[doc(hidden)]
pub mod test_scaffolding {
    pub use super::{
        process::test_scaffolding::*,
        scheduler::test_scaffolding::*,
        syscall::test_scaffolding::*,
        table::test_scaffolding::*,
    };

    use ku::{
        RingBuffer,
        process::{
            MiniContext,
            Pid,
        },
    };

    use crate::{
        error::Result,
        memory::{
            AddressSpace,
            BASE_ADDRESS_SPACE,
        },
        process::{
            Process,
            Table,
        },
    };

    pub fn create_process(elf_file: &[u8]) -> Result<Process> {
        super::create_process(elf_file)
    }

    pub fn dummy_process() -> Result<Pid> {
        let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate()?;

        address_space.switch_to();

        let process_info = super::map_process_info(&mut address_space)?;

        BASE_ADDRESS_SPACE.lock().switch_to();

        let context = MiniContext::default();
        let process = Process::new(address_space, context, process_info);

        Table::allocate(process)
    }

    pub fn map_log(address_space: &mut AddressSpace) -> Result<RingBuffer> {
        super::map_log(address_space)
    }

    pub const PROCESS_SLOT_COUNT: usize = super::PROCESS_SLOT_COUNT;
}
