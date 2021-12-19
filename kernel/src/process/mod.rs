mod elf;
#[allow(clippy::module_inception)]
mod process;
mod registers;
mod scheduler;
mod syscall;
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

use process::State;
use registers::RFlags;
use table::TABLE;

pub use ku::process::Pid;

pub use process::Process;
pub use scheduler::Scheduler;
pub use table::Table;

pub(crate) use registers::ModeContext;

pub fn init() {
    syscall::init();

    const PROCESS_SLOT_COUNT: usize = 1 << 10;
    *TABLE.lock() = Table::new(PROCESS_SLOT_COUNT);

    Scheduler::init(PROCESS_SLOT_COUNT);
}

pub fn create(elf_file: &[u8]) -> Result<Pid> {
    let mut address_space = BASE_ADDRESS_SPACE.lock().duplicate()?;

    address_space.switch_to();

    let entry = elf::load(&mut address_space, elf_file)?;

    let stack = Stack::new(&mut address_space, USER_RW)?;

    let process_info = map_process_info(&mut address_space)?;

    BASE_ADDRESS_SPACE.lock().switch_to();

    let context = MiniContext::new(entry, stack.pointer());
    let process = Process::new(address_space, context, process_info);

    info!(%context, file_size = %Size::from_slice(elf_file), %process, "loaded ELF file");

    Table::allocate(process)
}

fn map_process_info(address_space: &mut AddressSpace) -> Result<&'static mut ProcessInfo> {
    let log = map_log(address_space)?;
    let system_info = map_system_info(address_space)?;
    address_space.map_one(USER_RW, || ProcessInfo::new(log, system_info))
}

fn map_log(address_space: &mut AddressSpace) -> Result<RingBuffer> {
    // TODO: your code here.
}

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

pub mod test_scaffolding {
    pub use super::{
        process::test_scaffolding::*,
        scheduler::test_scaffolding::*,
    };
}
