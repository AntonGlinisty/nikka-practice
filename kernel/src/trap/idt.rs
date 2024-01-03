use core::{
    convert::TryInto,
    mem,
    ops::{
        Index,
        IndexMut,
    },
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

use ku::process::Trap;

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
        let mut idt = Self([IdtEntry::missing(); COUNT]);

        unsafe {
            exception_with_error_code!(
                idt,
                double_fault,
                Trap::DoubleFault,
                DOUBLE_FAULT_IST_INDEX,
            );

            exception_with_error_code!(idt, page_fault, Trap::PageFault, PAGE_FAULT_IST_INDEX);
        }

        exception_without_error_code!(idt, divide_error, Trap::DivideError);
        exception_without_error_code!(idt, debug, Trap::Debug);
        exception_without_error_code!(idt, non_maskable_interrupt, Trap::NonMaskableInterrupt);
        exception_without_error_code!(idt, breakpoint, Trap::Breakpoint);
        exception_without_error_code!(idt, overflow, Trap::Overflow);
        exception_without_error_code!(idt, bound_range_exceeded, Trap::BoundRangeExceeded);
        exception_without_error_code!(idt, invalid_opcode, Trap::InvalidOpcode);
        exception_without_error_code!(idt, device_not_available, Trap::DeviceNotAvailable);
        exception_with_error_code!(idt, invalid_tss, Trap::InvalidTss);
        exception_with_error_code!(idt, segment_not_present, Trap::SegmentNotPresent);
        exception_with_error_code!(idt, stack_segment_fault, Trap::StackSegmentFault);
        exception_with_error_code!(idt, general_protection_fault, Trap::GeneralProtectionFault);
        exception_without_error_code!(idt, x87_floating_point, Trap::X87FloatingPoint);
        exception_with_error_code!(idt, alignment_check, Trap::AlignmentCheck);
        exception_without_error_code!(idt, machine_check, Trap::MachineCheck);
        exception_without_error_code!(idt, simd_floating_point, Trap::SimdFloatingPoint);
        exception_without_error_code!(idt, virtualization, Trap::Virtualization);
        exception_with_error_code!(idt, security_exception, Trap::SecurityException);

        idt[Trap::Pit].set_handler(pic::pit);
        idt[Trap::Keyboard].set_handler(pic::keyboard);
        idt[Trap::Cascade].set_handler(pic::cascade);
        idt[Trap::Com2].set_handler(pic::com2);
        idt[Trap::Com1].set_handler(pic::com1);
        idt[Trap::Lpt2].set_handler(pic::lpt2);
        idt[Trap::FloppyDisk].set_handler(pic::floppy_disk);
        idt[Trap::Lpt1].set_handler(pic::lpt1);
        idt[Trap::Rtc].set_handler(pic::rtc);
        idt[Trap::Free29].set_handler(pic::free_29);
        idt[Trap::Free2A].set_handler(pic::free_2a);
        idt[Trap::Free2B].set_handler(pic::free_2b);
        idt[Trap::Ps2Mouse].set_handler(pic::ps2_mouse);
        idt[Trap::Coprocessor].set_handler(pic::coprocessor);
        idt[Trap::Ata0].set_handler(pic::ata0);
        idt[Trap::Ata1].set_handler(pic::ata1);

        idt[Trap::Timer].set_handler(apic::timer);
        idt[Trap::Spurious].set_handler(apic::spurious);

        idt
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

    /// Возвращает запись таблицы обработчиков прерываний для прерывания `trap`.
    fn get_mut(
        &mut self,
        index: Trap,
    ) -> &mut IdtEntry {
        &mut self.0[usize::from(index)]
    }
}

impl Index<Trap> for Idt {
    type Output = IdtEntry;

    fn index(
        &self,
        index: Trap,
    ) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<Trap> for Idt {
    fn index_mut(
        &mut self,
        index: Trap,
    ) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
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
pub(crate) struct IdtEntry(Entry<InterruptHandler>);

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

    use super::{
        COUNT,
        Idt,
        IdtEntry,
        Trap,
        TrapContext,
    };

    static IDT: Spinlock<Idt> = Spinlock::new(Idt([IdtEntry::missing(); COUNT]));

    pub fn set_debug_handler(handler: extern "x86-interrupt" fn(TrapContext)) {
        let mut idt = IDT.lock();

        *idt = Idt::new();
        idt.0[usize::from(Trap::Debug)].set_handler(handler);
        idt.load();
    }
}
