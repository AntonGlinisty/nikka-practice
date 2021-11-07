mod apic;
mod handler;
mod idt;
mod pic;
mod stats;
mod trap_context;

use x86_64::instructions::interrupts;

use crate::log::info;

use self::trap_context::TrapContext;

pub(crate) use self::idt::IDT;

pub use self::stats::TRAP_STATS;

pub const DIVIDE_ERROR: usize = 0x00;
pub const DEBUG: usize = 0x01;
pub const NON_MASKABLE_INTERRUPT: usize = 0x02;
pub const BREAKPOINT: usize = 0x03;
pub const OVERFLOW: usize = 0x04;
pub const BOUND_RANGE_EXCEEDED: usize = 0x05;
pub const INVALID_OPCODE: usize = 0x06;
pub const DEVICE_NOT_AVAILABLE: usize = 0x07;
pub const DOUBLE_FAULT: usize = 0x08;
pub const INVALID_TSS: usize = 0x0A;
pub const SEGMENT_NOT_PRESENT: usize = 0x0B;
pub const STACK_SEGMENT_FAULT: usize = 0x0C;
pub const GENERAL_PROTECTION_FAULT: usize = 0x0D;
pub const PAGE_FAULT: usize = 0x0E;
pub const X87_FLOATING_POINT: usize = 0x10;
pub const ALIGNMENT_CHECK: usize = 0x11;
pub const MACHINE_CHECK: usize = 0x12;
pub const SIMD_FLOATING_POINT: usize = 0x13;
pub const VIRTUALIZATION: usize = 0x14;
pub const SECURITY_EXCEPTION: usize = 0x1E;

const EXCEPTION_COUNT: u8 = 0x20;

const PIC_FIRST_INTERRUPT: u8 = EXCEPTION_COUNT;

const PIC_BASE: usize = PIC_FIRST_INTERRUPT as usize;
pub const PIT: usize = PIC_BASE;
pub const KEYBOARD: usize = PIC_BASE + 0x1;
pub const CASCADE: usize = PIC_BASE + 0x2;
pub const COM2: usize = PIC_BASE + 0x3;
pub const COM1: usize = PIC_BASE + 0x4;
pub const LPT2: usize = PIC_BASE + 0x5;
pub const FLOPPY_DISK: usize = PIC_BASE + 0x6;
pub const LPT1: usize = PIC_BASE + 0x7;
pub const RTC: usize = PIC_BASE + 0x8;
pub const FREE_29: usize = PIC_BASE + 0x9;
pub const FREE_2A: usize = PIC_BASE + 0xA;
pub const FREE_2B: usize = PIC_BASE + 0xB;
pub const PS2_MOUSE: usize = PIC_BASE + 0xC;
pub const COPROCESSOR: usize = PIC_BASE + 0xD;
pub const ATA0: usize = PIC_BASE + 0xE;
pub const ATA1: usize = PIC_BASE + 0xF;

const APIC_BASE: usize = ATA1 + 1;
pub const TIMER: usize = APIC_BASE;
pub const SPURIOUS: usize = APIC_BASE + 0x1;

const COUNT: usize = SPURIOUS + 1;

pub(super) fn init() {
    unsafe {
        pic8259::init(PIC_FIRST_INTERRUPT);
    }

    IDT.load();

    interrupts::enable();

    info!("traps init");
}
