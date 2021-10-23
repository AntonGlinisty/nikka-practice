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

pub(crate) type CpuId = u8;

/// <https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf>, Chapter 10.
/// <https://wiki.osdev.org/APIC#Local_APIC_registers>
#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub(crate) struct LocalApic {
    _0: [LocalApicRegister; 2],
    id: LocalApicRegister,
    _1: [LocalApicRegister; 8],
    eoi: LocalApicRegister,
    _2: [LocalApicRegister; 3],
    spurious: LocalApicRegister,
    _3: [LocalApicRegister; 32],
    interrupt_command_lo: LocalApicRegister,
    interrupt_command_hi: LocalApicRegister,
    lvt_timer: LocalApicRegister,
    lvt_termal_sensor: LocalApicRegister,
    lvt_performance_counter_overflow: LocalApicRegister,
    lvt_lint0: LocalApicRegister,
    lvt_lint1: LocalApicRegister,
    lvt_error: LocalApicRegister,
    timer_initial_count: LocalApicRegister,
    timer_current_count: LocalApicRegister,
    _4: [LocalApicRegister; 4],
    timer_divide_configuration: LocalApicRegister,
    _5: [LocalApicRegister; 1],
}

const_assert!(mem::size_of::<LocalApic>() == Page::SIZE);

impl LocalApic {
    const fn zero() -> Self {
        union LocalApicPage {
            local_apic: LocalApic,
            raw: [u8; Page::SIZE],
        }

        let clear_page = LocalApicPage {
            raw: [0; Page::SIZE],
        };

        unsafe { clear_page.local_apic }
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
        let boot_page = size::try_into_u32(Frame::index_by_address(boot_address))?;
        if boot_address.into_usize() % Page::SIZE != 0 || boot_page > MAX_BOOT_PAGE {
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
            interrupt.update(|x| x | MASK_INTERRUPT);
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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct LocalApicRegister(u32, [u32; 3]);

impl LocalApicRegister {
    fn get(&self) -> u32 {
        unsafe { (&self.0 as *const u32).read_volatile() }
    }

    fn set(
        &mut self,
        value: u32,
    ) {
        unsafe {
            (&mut self.0 as *mut u32).write_volatile(value);
        }
    }

    fn update<F: FnOnce(u32) -> u32>(
        &mut self,
        f: F,
    ) {
        self.set(f(self.get()));
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
