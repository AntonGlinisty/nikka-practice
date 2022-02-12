use core::{
    convert::TryInto,
    hint,
    mem,
};

use bitflags::bitflags;
use static_assertions::const_assert;

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    memory::{
        BASE_ADDRESS_SPACE,
        Frame,
        Page,
        Phys,
        Virt,
        mmu::{
            KERNEL_RW,
            PageTableFlags,
        },
        size,
    },
    trap::{
        SPURIOUS,
        TIMER,
    },
};

use register::Register;

pub(crate) type CpuId = u8;

/// <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf>, Chapter 10.
/// <https://wiki.osdev.org/APIC#Local_APIC_registers>
#[derive(Clone)]
#[repr(C, align(4096))]
pub(crate) struct LocalApic {
    _0: [Register; 2],
    id: Register,
    _1: [Register; 8],
    eoi: Register,
    _2: [Register; 3],
    spurious: Register,
    _3: [Register; 32],
    interrupt_command_lo: Register,
    interrupt_command_hi: Register,
    lvt_timer: Register,
    lvt_termal_sensor: Register,
    lvt_performance_counter_overflow: Register,
    lvt_lint0: Register,
    lvt_lint1: Register,
    lvt_error: Register,
    timer_initial_count: Register,
    timer_current_count: Register,
    _4: [Register; 4],
    timer_divide_configuration: Register,
    _5: [Register; 1],
}

const_assert!(mem::size_of::<LocalApic>() == Page::SIZE);

impl LocalApic {
    const fn zero() -> Self {
        Self {
            _0: [Register::zero(); 2],
            id: Register::zero(),
            _1: [Register::zero(); 8],
            eoi: Register::zero(),
            _2: [Register::zero(); 3],
            spurious: Register::zero(),
            _3: [Register::zero(); 32],
            interrupt_command_lo: Register::zero(),
            interrupt_command_hi: Register::zero(),
            lvt_timer: Register::zero(),
            lvt_termal_sensor: Register::zero(),
            lvt_performance_counter_overflow: Register::zero(),
            lvt_lint0: Register::zero(),
            lvt_lint1: Register::zero(),
            lvt_error: Register::zero(),
            timer_initial_count: Register::zero(),
            timer_current_count: Register::zero(),
            _4: [Register::zero(); 4],
            timer_divide_configuration: Register::zero(),
            _5: [Register::zero(); 1],
        }
    }

    pub(super) fn init() {
        const TSCS_PER_INTERRUPT: u32 = 100_000_000;

        let local_apic = Self::get();

        local_apic.enable();
        local_apic.disable_lvts();
        local_apic.init_timer(TSCS_PER_INTERRUPT);
        Self::end_of_interrupt();
    }

    pub(super) fn map(address: Phys) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf>, Chapter 10.8.5 "Signaling Interrupt Servicing Completion"
    /// <https://wiki.osdev.org/APIC#EOI_Register>
    pub(crate) fn end_of_interrupt() {
        Self::get().eoi.set(0);
    }

    /// <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf>, Chapter 10.4.6 "Local APIC ID"
    pub(crate) fn id() -> CpuId {
        (Self::get().id.get() >> Self::ID_SHIFT).try_into().unwrap()
    }

    /// MultiProcessor Specification 4.1 part B.4 "Application Processor Startup"
    pub(super) fn send_init(
        id: CpuId,
        boot_address: Phys,
    ) -> Result<()> {
        const MAX_BOOT_PAGE: u32 = 0xFF;
        let boot_page = size::try_into_u32(Frame::new(boot_address)?.index())?;
        if boot_page > MAX_BOOT_PAGE {
            return Err(InvalidArgument);
        }

        let local_apic = Self::get();

        let init_data = InterruptCommand::INIT | InterruptCommand::TRIGGER_MODE_LEVEL;
        local_apic.send_ipi(id, (init_data | InterruptCommand::LEVEL_ASSERT).bits());
        local_apic.send_ipi(id, (init_data | InterruptCommand::LEVEL_DEASSERT).bits());

        local_apic.send_ipi(id, InterruptCommand::START_UP.bits() | boot_page);

        Ok(())
    }

    /// 10.6 "ISSUING INTERPROCESSOR INTERRUPTS"
    fn send_ipi(
        &mut self,
        id: CpuId,
        data: u32,
    ) {
        self.interrupt_command_hi.set(u32::from(id) << Self::ID_SHIFT);
        self.interrupt_command_lo.set(data);

        while (self.interrupt_command_lo.get() & InterruptCommand::SEND_PENDING.bits()) != 0 {
            hint::spin_loop();
        }
    }

    fn get() -> &'static mut Self {
        unsafe { &mut LOCAL_APIC }
    }

    /// <https://wiki.osdev.org/APIC#Spurious_Interrupt_Vector_Register>
    fn enable(&mut self) {
        const ENABLE_LOCAL_APIC: u32 = 1 << 8;
        self.spurious.set(ENABLE_LOCAL_APIC | size::try_into_u32(SPURIOUS).unwrap());
    }

    /// <https://wiki.osdev.org/APIC#Local_Vector_Table_Registers>
    fn disable_lvts(&mut self) {
        const MASK_INTERRUPT: u32 = 1 << 16;
        for interrupt in [
            &mut self.lvt_termal_sensor,
            &mut self.lvt_performance_counter_overflow,
            &mut self.lvt_lint1,
            &mut self.lvt_error,
        ] {
            interrupt.update(|x| *x |= MASK_INTERRUPT);
        }
    }

    /// <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf>, Chapter 10.5.4
    fn init_timer(
        &mut self,
        tscs_per_interrupt: u32,
    ) {
        const DIVIDE_BY_1: u32 = 0b1011;
        self.timer_divide_configuration.set(DIVIDE_BY_1);

        const PERIODIC_MODE: u32 = 0b01 << 17;
        self.lvt_timer.set(PERIODIC_MODE | size::try_into_u32(TIMER).unwrap());

        // [AMD64 Architecture Programmer’s Manual](https://docs.amd.com/v/u/en-US/40332-PUB_4.08):
        // "To avoid race conditions, software should initialize
        // the Divide Configuration Register and the Timer Local Vector Table Register
        // prior to writing the Initial Count Register to start the timer."
        self.timer_initial_count.set(tscs_per_interrupt);
    }

    const ID_SHIFT: usize = 24;
}

mod register {
    use volatile::Volatile;

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub(super) struct Register(u32, [u32; 3]);

    impl Register {
        pub(super) const fn zero() -> Self {
            Self(0, [0; 3])
        }

        pub(super) fn get(&self) -> u32 {
            Volatile::new(&self.0).read()
        }

        pub(super) fn set(
            &mut self,
            value: u32,
        ) {
            Volatile::new(&mut self.0).write(value);
        }

        pub(super) fn update<F: FnOnce(&mut u32)>(
            &mut self,
            f: F,
        ) {
            Volatile::new(&mut self.0).update(f);
        }
    }
}

bitflags! {
    /// 10.6.1 Interrupt Command Register (ICR)
    struct InterruptCommand: u32 {
        const INIT = 0b101 << 8;
        const START_UP = 0b110 << 8;
        const SEND_PENDING = 1 << 12;
        const LEVEL_ASSERT = 1 << 14;
        const LEVEL_DEASSERT = 0 << 14;
        const TRIGGER_MODE_LEVEL = 1 << 15;
    }
}

static mut LOCAL_APIC: LocalApic = LocalApic::zero();
