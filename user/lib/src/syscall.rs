use ku::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    info,
    info::ProcessInfo,
    memory::{
        Block,
        Page,
        Virt,
        mmu::PageTableFlags,
    },
    process::{
        Pid,
        ResultCode,
        Syscall,
    },
};

pub fn exit(code: usize) -> ! {
    syscall(Syscall::EXIT.bits(), code, 0, 0, 0, 0).unwrap();

    unreachable!();
}

#[allow(unused_must_use)]
pub fn sched_yield() {
    syscall(Syscall::SCHED_YIELD.bits(), 0, 0, 0, 0, 0);
}

// Inline is needed for the correctness of exofork().
#[inline(always)]
pub fn exofork() -> Result<Pid> {
    let (child_pid, process_info) = syscall(Syscall::EXOFORK.bits(), 0, 0, 0, 0, 0)?;
    let child_pid = Pid::from_usize(child_pid)?;
    if child_pid == Pid::Current {
        info::set_process_info(unsafe { Virt::new(process_info)?.try_into_mut::<ProcessInfo>()? });
    }
    Ok(child_pid)
}

pub fn map(
    dst_pid: Pid,
    dst_block: Block<Page>,
    flags: PageTableFlags,
) -> Result<Block<Page>> {
    let (address, _) = syscall(
        Syscall::MAP.bits(),
        dst_pid.into_usize(),
        dst_block.start_address().into_usize(),
        dst_block.size(),
        flags.bits(),
        0,
    )?;

    let start = Virt::new(address)?;
    let end = (start + dst_block.size())?;

    Block::new(Page::new(start)?, Page::new(end)?)
}

pub fn unmap(
    dst_pid: Pid,
    dst_block: Block<Page>,
) -> Result<()> {
    syscall(
        Syscall::UNMAP.bits(),
        dst_pid.into_usize(),
        dst_block.start_address().into_usize(),
        dst_block.size(),
        0,
        0,
    )
    .map(|_| ())
}

pub fn copy_mapping(
    dst_pid: Pid,
    src_block: Block<Page>,
    dst_block: Block<Page>,
    flags: PageTableFlags,
) -> Result<()> {
    if src_block.count() == dst_block.count() {
        syscall(
            Syscall::COPY_MAPPING.bits(),
            dst_pid.into_usize(),
            src_block.start_address().into_usize(),
            dst_block.start_address().into_usize(),
            dst_block.size(),
            flags.bits(),
        )
        .map(|_| ())
    } else {
        Err(InvalidArgument)
    }
}

pub fn set_state(dst_pid: Pid) -> Result<()> {
    syscall(Syscall::SET_STATE.bits(), dst_pid.into_usize(), 0, 0, 0, 0).map(|_| ())
}

// Inline is needed for the correctness of exofork().
#[inline(always)]
fn syscall(
    number: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> Result<(usize, usize)> {
    // TODO: your code here.
    unimplemented!();
}
