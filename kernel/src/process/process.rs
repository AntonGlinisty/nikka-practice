use core::fmt;

use spin::MutexGuard;

use ku::{
    ProcessInfo,
    RingBuffer,
    process::MiniContext,
};

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::{
        debug,
        info,
    },
    memory::{
        AddressSpace,
        Block,
        USER_RW,
        Virt,
    },
    smp::Cpu,
};

use super::{
    Pid,
    Table,
    registers::Registers,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum State {
    Exofork,
    Runnable,
    Running,
}

#[derive(Debug)]
pub struct Process {
    address_space: AddressSpace,
    info: Block<Virt>,
    parent: Option<Pid>,
    pid: Pid,
    registers: Registers,
    state: State,
    trap_handler_context: TrapHandlerContext,
}

impl Process {
    pub(super) fn new(
        address_space: AddressSpace,
        context: MiniContext,
        info: &'static mut ProcessInfo,
    ) -> Self {
        let info = Block::from_ref(info);

        Self {
            address_space,
            info,
            parent: None,
            pid: Pid::Current,
            registers: Registers::new(context, info.start_address().into_usize()),
            state: State::Runnable,
            trap_handler_context: TrapHandlerContext::default(),
        }
    }

    pub(super) fn duplicate(
        &self,
        rax: usize,
        rdi: usize,
    ) -> Result<Self> {
        let mut address_space = self.address_space.duplicate()?;

        address_space.switch_to();
        let info = Block::from_ref(super::map_process_info(&mut address_space)?);
        self.address_space.switch_to();

        Ok(Self {
            address_space,
            info,
            parent: Some(self.pid),
            pid: Pid::Current,
            registers: self.registers.duplicate(rax, rdi, info.start_address().into_usize()),
            state: State::Exofork,
            trap_handler_context: TrapHandlerContext::default(),
        })
    }

    pub fn address_space(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }

    pub(super) fn set_context(
        &mut self,
        context: MiniContext,
    ) {
        self.registers.set_mini_context(context);
    }

    unsafe fn info(&mut self) -> Result<&mut ProcessInfo> {
        let info = self.address_space.check_permission_mut::<ProcessInfo>(self.info, USER_RW)?;

        if info.len() == 1 {
            Ok(&mut info[0])
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn log(&mut self) -> Result<&mut RingBuffer> {
        let info = unsafe { self.info()? };
        let log = Block::from_ref(info.log());

        let log = self.address_space.check_permission_mut::<RingBuffer>(log, USER_RW)?;

        if log.len() == 1 {
            self.address_space.check_permission_mut::<u8>(log[0].block(), USER_RW)?;
            Ok(&mut log[0])
        } else {
            Err(InvalidArgument)
        }
    }

    pub fn parent(&self) -> Option<Pid> {
        self.parent
    }

    pub fn pid(&self) -> Pid {
        assert_ne!(
            self.pid,
            Pid::Current,
            "the process has not been assigned a pid yet",
        );
        self.pid
    }

    pub(super) fn set_pid(
        &mut self,
        pid: Pid,
    ) {
        if let Pid::Id { .. } = pid {
            self.pid = pid;
            self.address_space.set_pid(pid);
        } else {
            panic!("assignment of a wrong pid to a process");
        }
    }

    pub(super) fn state(&self) -> State {
        self.state
    }

    pub(super) fn set_state(
        &mut self,
        state: State,
    ) {
        self.state = state
    }

    pub(crate) fn trap_handler_context(&self) -> TrapHandlerContext {
        self.trap_handler_context
    }

    pub(super) fn set_trap_handler_context(
        &mut self,
        trap_handler_context: TrapHandlerContext,
    ) {
        self.trap_handler_context = trap_handler_context;
    }

    pub fn enter_user_mode(mut process: MutexGuard<Process>) -> bool {
        let pid = process.pid;

        process.address_space.switch_to();

        if let Ok(info) = unsafe { process.info() } {
            info.set_pid(pid);
        }

        Cpu::set_current_process(Some(pid));

        let registers = &mut process.registers as *mut Registers;

        process.state = State::Running;

        debug!(%pid, registers = %process.registers, "entering the user mode");

        drop(process);

        unsafe {
            Registers::switch_to(registers);
        }

        debug!(%pid, "leaving the user mode");

        Cpu::set_current_process(None);

        // TODO: your code here.
        false // TODO: remove before flight.
    }

    /// Switches into the kernel context in which `enter_user_mode()` was called.
    /// The current kernel context is destroyed.
    pub(crate) extern "C" fn leave_user_mode() -> ! {
        unsafe {
            assert!(Cpu::current_process().is_ok());

            Registers::switch_from();
        }
    }

    pub(crate) extern "C" fn sched_yield() -> ! {
        unsafe {
            assert!(Cpu::current_process().is_ok());

            Registers::sched_yield();
        }
    }
}

impl fmt::Display for Process {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ pid: {}, address_space: {}, {} }}",
            self.pid,
            self.address_space,
            self.registers.mini_context(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TrapHandlerContext {
    mini_context: MiniContext,
    stack: Block<Virt>,
}

impl TrapHandlerContext {
    pub(super) fn new(
        rip: Virt,
        stack: Block<Virt>,
    ) -> Result<Self> {
        Ok(Self {
            mini_context: MiniContext::new(rip, stack.end_address()?),
            stack,
        })
    }

    pub(crate) fn contains(
        &self,
        rsp: Virt,
    ) -> bool {
        self.stack.contains_address(rsp)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.stack != Block::default()
    }

    pub(crate) fn mini_context(&self) -> MiniContext {
        self.mini_context
    }
}

impl fmt::Display for TrapHandlerContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ {}, stack: {} }}",
            self.mini_context, self.stack,
        )
    }
}

pub(super) mod test_scaffolding {
    use super::{
        super::registers::test_scaffolding,
        Process,
    };

    pub fn disable_interrupts(process: &mut Process) {
        test_scaffolding::disable_interrupts(&mut process.registers);
    }

    pub fn registers(process: &Process) -> [usize; 15] {
        test_scaffolding::registers(&process.registers)
    }
}
