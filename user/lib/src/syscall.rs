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
        RSP_OFFSET_IN_TRAP_INFO,
        ResultCode,
        Syscall,
        TrapInfo,
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

pub fn set_trap_handler(
    dst_pid: Pid,
    trap_handler: fn(&TrapInfo),
    trap_handler_stack: Block<Page>,
) -> Result<()> {
    unsafe {
        TRAP_HANDLER = trap_handler;
    }

    syscall(
        Syscall::SET_TRAP_HANDLER.bits(),
        dst_pid.into_usize(),
        trap_handler_trampoline as *const () as usize,
        trap_handler_stack.start_address().into_usize(),
        trap_handler_stack.size(),
        0,
    )
    .map(|_| ())
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

#[cold]
#[naked]
extern "C" fn trap_handler_trampoline() -> ! {
    unsafe {
        asm!(
            "
            // TODO: your code here.

            call {trap_handler_invoker}

            // TODO: your code here.

            // Return to the point which caused this trap.
            // This relies on that trap_handler_invoker() has called TrapInfo::prepare_for_ret()
            // to put the trap site RIP on the trap site stack.
            ret
            ",

            // TODO: your code here.
            trap_handler_invoker = sym trap_handler_invoker,

            options(noreturn),
        );
    }
}

#[cold]
#[inline(never)]
extern "C" fn trap_handler_invoker(info: &mut TrapInfo, // rdi
) {
    unsafe {
        TRAP_HANDLER(info);

        info.prepare_for_ret().unwrap();
    }
}

fn dummy_trap_handler(_info: &TrapInfo) {
}

static mut TRAP_HANDLER: fn(&TrapInfo) = dummy_trap_handler;
