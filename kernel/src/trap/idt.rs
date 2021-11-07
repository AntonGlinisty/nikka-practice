use core::{
    convert::TryInto,
    mem,
};

use lazy_static::lazy_static;
use x86_64::{
    VirtAddr,
    instructions::tables,
    structures::{
        DescriptorTablePointer,
        idt::{
            Entry,
            EntryOptions,
        },
    },
};

use crate::memory::{
    DOUBLE_FAULT_IST_INDEX,
    PAGE_FAULT_IST_INDEX,
    Virt,
};

use super::{
    COUNT,
    TrapContext,
    apic,
    handler,
    pic,
};

pub(crate) struct Idt([IdtEntry; COUNT]);

impl Idt {
    fn new() -> Self {
        let mut idt = [IdtEntry::missing(); COUNT];

        unsafe {
            idt[super::DOUBLE_FAULT]
                .set_handler_with_error(handler::double_fault)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);

            idt[super::PAGE_FAULT]
                .set_handler_with_error(handler::page_fault)
                .set_stack_index(PAGE_FAULT_IST_INDEX);
        }

        idt[super::DIVIDE_ERROR].set_handler(handler::divide_error);
        idt[super::DEBUG].set_handler(handler::debug);
        idt[super::NON_MASKABLE_INTERRUPT].set_handler(handler::non_maskable_interrupt);
        idt[super::BREAKPOINT].set_handler(handler::breakpoint);
        idt[super::OVERFLOW].set_handler(handler::overflow);
        idt[super::BOUND_RANGE_EXCEEDED].set_handler(handler::bound_range_exceeded);
        idt[super::INVALID_OPCODE].set_handler(handler::invalid_opcode);
        idt[super::DEVICE_NOT_AVAILABLE].set_handler(handler::device_not_available);
        idt[super::INVALID_TSS].set_handler_with_error(handler::invalid_tss);
        idt[super::SEGMENT_NOT_PRESENT].set_handler_with_error(handler::segment_not_present);
        idt[super::STACK_SEGMENT_FAULT].set_handler_with_error(handler::stack_segment_fault);
        idt[super::GENERAL_PROTECTION_FAULT]
            .set_handler_with_error(handler::general_protection_fault);
        idt[super::X87_FLOATING_POINT].set_handler(handler::x87_floating_point);
        idt[super::ALIGNMENT_CHECK].set_handler_with_error(handler::alignment_check);
        idt[super::MACHINE_CHECK].set_handler(handler::machine_check);
        idt[super::SIMD_FLOATING_POINT].set_handler(handler::simd_floating_point);
        idt[super::VIRTUALIZATION].set_handler(handler::virtualization);
        idt[super::SECURITY_EXCEPTION].set_handler_with_error(handler::security_exception);

        idt[super::PIT].set_handler(pic::pit);
        idt[super::KEYBOARD].set_handler(pic::keyboard);
        idt[super::CASCADE].set_handler(pic::cascade);
        idt[super::COM2].set_handler(pic::com2);
        idt[super::COM1].set_handler(pic::com1);
        idt[super::LPT2].set_handler(pic::lpt2);
        idt[super::FLOPPY_DISK].set_handler(pic::floppy_disk);
        idt[super::LPT1].set_handler(pic::lpt1);
        idt[super::RTC].set_handler(pic::rtc);
        idt[super::FREE_29].set_handler(pic::free_29);
        idt[super::FREE_2A].set_handler(pic::free_2a);
        idt[super::FREE_2B].set_handler(pic::free_2b);
        idt[super::PS2_MOUSE].set_handler(pic::ps2_mouse);
        idt[super::COPROCESSOR].set_handler(pic::coprocessor);
        idt[super::ATA0].set_handler(pic::ata0);
        idt[super::ATA1].set_handler(pic::ata1);

        idt[super::TIMER].set_handler(apic::timer);
        idt[super::SPURIOUS].set_handler(apic::spurious);

        Self(idt)
    }

    pub(crate) fn load(&self) {
        unsafe {
            let pseudo_descriptor = DescriptorTablePointer {
                base: Virt::from_ref(&self.0).into(),
                limit: (mem::size_of_val(&self.0) - 1).try_into().expect("the IDT is too large"),
            };
            tables::lidt(&pseudo_descriptor);
        }
    }
}

lazy_static! {
    pub(crate) static ref IDT: Idt = Idt::new();
}

type InterruptHandler = fn(TrapContext);

#[derive(Clone, Copy)]
struct IdtEntry(Entry<InterruptHandler>);

impl IdtEntry {
    const fn missing() -> Self {
        Self(Entry::missing())
    }

    fn set_handler(
        &mut self,
        handler: extern "x86-interrupt" fn(TrapContext),
    ) -> &mut EntryOptions {
        unsafe { self.0.set_handler_addr(VirtAddr::from_ptr(handler as *const ())) }
    }

    fn set_handler_with_error(
        &mut self,
        handler: extern "x86-interrupt" fn(TrapContext, u64),
    ) -> &mut EntryOptions {
        unsafe { self.0.set_handler_addr(VirtAddr::from_ptr(handler as *const ())) }
    }
}
