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
    handler::{
        exception_with_error_code,
        exception_without_error_code,
    },
    pic,
};

/// Таблица обработчиков прерываний.
pub(crate) struct Idt([IdtEntry; COUNT]);

impl Idt {
    /// Создаёт таблицу обработчиков прерываний.
    fn new() -> Self {
        let mut idt = [IdtEntry::missing(); COUNT];

        unsafe {
            exception_with_error_code!(
                idt,
                double_fault,
                super::DOUBLE_FAULT,
                DOUBLE_FAULT_IST_INDEX,
            );

            exception_with_error_code!(idt, page_fault, super::PAGE_FAULT, PAGE_FAULT_IST_INDEX);
        }

        exception_without_error_code!(idt, divide_error, super::DIVIDE_ERROR);
        exception_without_error_code!(idt, debug, super::DEBUG);
        exception_without_error_code!(idt, non_maskable_interrupt, super::NON_MASKABLE_INTERRUPT);
        exception_without_error_code!(idt, breakpoint, super::BREAKPOINT);
        exception_without_error_code!(idt, overflow, super::OVERFLOW);
        exception_without_error_code!(idt, bound_range_exceeded, super::BOUND_RANGE_EXCEEDED);
        exception_without_error_code!(idt, invalid_opcode, super::INVALID_OPCODE);
        exception_without_error_code!(idt, device_not_available, super::DEVICE_NOT_AVAILABLE);
        exception_with_error_code!(idt, invalid_tss, super::INVALID_TSS);
        exception_with_error_code!(idt, segment_not_present, super::SEGMENT_NOT_PRESENT);
        exception_with_error_code!(idt, stack_segment_fault, super::STACK_SEGMENT_FAULT);
        exception_with_error_code!(
            idt,
            general_protection_fault,
            super::GENERAL_PROTECTION_FAULT,
        );
        exception_without_error_code!(idt, x87_floating_point, super::X87_FLOATING_POINT);
        exception_with_error_code!(idt, alignment_check, super::ALIGNMENT_CHECK);
        exception_without_error_code!(idt, machine_check, super::MACHINE_CHECK);
        exception_without_error_code!(idt, simd_floating_point, super::SIMD_FLOATING_POINT);
        exception_without_error_code!(idt, virtualization, super::VIRTUALIZATION);
        exception_with_error_code!(idt, security_exception, super::SECURITY_EXCEPTION);

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

    /// Загружает ссылку на таблицу обработчиков прерываний в
    /// [регистр `IDTR`](https://wiki.osdev.org/Interrupt_Descriptor_Table#IDTR)
    /// процессора.
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
    /// Таблица обработчиков прерываний.
    pub(crate) static ref IDT: Idt = Idt::new();
}

/// Тип функции обработчиков исключений и прерываний.
type InterruptHandler = fn(TrapContext);

/// Запись таблицы исключений и прерываний
/// ([Interrupt descriptor table](https://en.wikipedia.org/wiki/Interrupt_descriptor_table), IDT).
#[derive(Clone, Copy)]
struct IdtEntry(Entry<InterruptHandler>);

impl IdtEntry {
    /// Создаёт запись таблицы исключений и прерываний без обработчика.
    const fn missing() -> Self {
        Self(Entry::missing())
    }

    /// Устанавливает обработчик `handler` исключения процессора.
    fn set_trap_handler(
        &mut self,
        handler: extern "C" fn(),
    ) -> &mut EntryOptions {
        unsafe { self.0.set_handler_addr(VirtAddr::from_ptr(handler as *const ())) }
    }

    /// Устанавливает обработчик `handler` прерывания.
    fn set_handler(
        &mut self,
        handler: extern "x86-interrupt" fn(TrapContext),
    ) -> &mut EntryOptions {
        unsafe { self.0.set_handler_addr(VirtAddr::from_ptr(handler as *const ())) }
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use ku::sync::Spinlock;

    use super::super::DEBUG;

    use super::{
        COUNT,
        Idt,
        IdtEntry,
        TrapContext,
    };

    static IDT: Spinlock<Idt> = Spinlock::new(Idt([IdtEntry::missing(); COUNT]));

    pub fn set_debug_handler(handler: extern "x86-interrupt" fn(TrapContext)) {
        let mut idt = IDT.lock();

        *idt = Idt::new();
        idt.0[DEBUG].set_handler(handler);
        idt.load();
    }
}
