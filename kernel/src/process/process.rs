use core::fmt;

use crate::{
    gdt::Gdt,
    memory::{
        AddressSpace,
        Virt,
    },
    smp::Cpu,
};

use super::registers::Registers;

#[derive(Debug)]
pub struct Process {
    address_space: AddressSpace,
    registers: Registers,
}

impl Process {
    pub(super) fn new(
        address_space: AddressSpace,
        rip: Virt,
        rsp: Virt,
    ) -> Self {
        Self {
            address_space,
            registers: Registers::new(Gdt::user_code(), Gdt::user_data(), rip, rsp),
        }
    }

    pub fn address_space(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }

    pub fn enter_user_mode(&mut self) {
        self.address_space.switch_to();

        Cpu::set_current_process(Virt::from_ref(self));

        unsafe {
            Registers::switch_to(&self.registers);
        }

        Cpu::set_current_process(Virt::zero());
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
            "{{ address_space: {}, rip: {}, rsp: {} }}",
            self.address_space,
            self.registers.rip(),
            self.registers.rsp(),
        )
    }
}
