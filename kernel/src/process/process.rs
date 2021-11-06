use core::fmt;

use spin::MutexGuard;

use ku::ProcessInfo;

use crate::{
    memory::{
        AddressSpace,
        Virt,
    },
    smp::Cpu,
};

use super::{
    Pid,
    registers::Registers,
};

#[derive(Debug)]
pub struct Process {
    address_space: AddressSpace,
    info: Virt,
    pid: Pid,
    registers: Registers,
}

impl Process {
    pub(super) fn new(
        address_space: AddressSpace,
        rip: Virt,
        rsp: Virt,
        info: &'static mut ProcessInfo,
    ) -> Self {
        let info = Virt::from_ref(info);

        Self {
            address_space,
            info,
            pid: Pid::Current,
            registers: Registers::new(rip, rsp, info.into_usize()),
        }
    }

    pub fn address_space(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }

    pub(super) fn set_context(
        &mut self,
        rip: Virt,
        rsp: Virt,
    ) {
        self.registers.set_context(rip, rsp);
    }

    pub fn info(&mut self) -> &mut ProcessInfo {
        unsafe { self.info.try_into_mut().unwrap() }
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
        } else {
            panic!("assignment of a wrong pid to a process");
        }
    }

    pub fn enter_user_mode(process: MutexGuard<Process>) {
        process.address_space.switch_to();

        Cpu::set_current_process(Some(process.pid));

        let registers = &process.registers as *const Registers;

        drop(process);

        unsafe {
            Registers::switch_to(registers);
        }

        Cpu::set_current_process(None);
    }

    /// Switches into the kernel context in which `enter_user_mode()` was called.
    /// The current kernel context is destroyed.
    pub(crate) extern "C" fn leave_user_mode() -> ! {
        unsafe {
            assert!(Cpu::current_process().is_ok());

            Registers::switch_from();
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
            "{{ pid: {}, address_space: {}, rip: {}, rsp: {} }}",
            self.pid,
            self.address_space,
            self.registers.rip(),
            self.registers.rsp(),
        )
    }
}
