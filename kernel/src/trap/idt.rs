use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

use crate::memory::{
    DOUBLE_FAULT_IST_INDEX,
    PAGE_FAULT_IST_INDEX,
};

use super::{
    apic,
    handler,
    pic,
};

lazy_static! {
    pub(crate) static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.divide_error.set_handler_fn(handler::divide_error);
        idt.debug.set_handler_fn(handler::debug);
        idt.non_maskable_interrupt.set_handler_fn(handler::non_maskable_interrupt);
        idt.breakpoint.set_handler_fn(handler::breakpoint);
        idt.overflow.set_handler_fn(handler::overflow);
        idt.bound_range_exceeded.set_handler_fn(handler::bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(handler::invalid_opcode);
        idt.device_not_available.set_handler_fn(handler::device_not_available);

        unsafe {
            idt.double_fault
                .set_handler_fn(handler::double_fault)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(handler::invalid_tss);
        idt.segment_not_present.set_handler_fn(handler::segment_not_present);
        idt.stack_segment_fault.set_handler_fn(handler::stack_segment_fault);
        idt.general_protection_fault.set_handler_fn(handler::general_protection_fault);

        unsafe {
            idt.page_fault
                .set_handler_fn(handler::page_fault)
                .set_stack_index(PAGE_FAULT_IST_INDEX);
        }

        idt.x87_floating_point.set_handler_fn(handler::x87_floating_point);
        idt.alignment_check.set_handler_fn(handler::alignment_check);
        idt.machine_check.set_handler_fn(handler::machine_check);
        idt.simd_floating_point.set_handler_fn(handler::simd_floating_point);
        idt.virtualization.set_handler_fn(handler::virtualization);
        idt.security_exception.set_handler_fn(handler::security_exception);

        idt[super::PIT].set_handler_fn(pic::pit);
        idt[super::KEYBOARD].set_handler_fn(pic::keyboard);
        idt[super::CASCADE].set_handler_fn(pic::cascade);
        idt[super::COM2].set_handler_fn(pic::com2);
        idt[super::COM1].set_handler_fn(pic::com1);
        idt[super::LPT2].set_handler_fn(pic::lpt2);
        idt[super::FLOPPY_DISK].set_handler_fn(pic::floppy_disk);
        idt[super::LPT1].set_handler_fn(pic::lpt1);
        idt[super::RTC].set_handler_fn(pic::rtc);
        idt[super::FREE_29].set_handler_fn(pic::free_29);
        idt[super::FREE_2A].set_handler_fn(pic::free_2a);
        idt[super::FREE_2B].set_handler_fn(pic::free_2b);
        idt[super::PS2_MOUSE].set_handler_fn(pic::ps2_mouse);
        idt[super::COPROCESSOR].set_handler_fn(pic::coprocessor);
        idt[super::ATA0].set_handler_fn(pic::ata0);
        idt[super::ATA1].set_handler_fn(pic::ata1);

        idt[super::TIMER].set_handler_fn(apic::timer);
        idt[super::SPURIOUS].set_handler_fn(apic::spurious);

        idt
    };
}
