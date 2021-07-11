mod handler;
mod idt;
mod pic;
mod stats;

use x86_64::instructions::interrupts;

use log::info;

use self::idt::IDT;

pub(crate) use self::stats::TRAP_STATS;

const DIVIDE_ERROR: usize = 0x00;
const DEBUG: usize = 0x01;
const NON_MASKABLE_INTERRUPT: usize = 0x02;
const BREAKPOINT: usize = 0x03;
const OVERFLOW: usize = 0x04;
const BOUND_RANGE_EXCEEDED: usize = 0x05;
const INVALID_OPCODE: usize = 0x06;
const DEVICE_NOT_AVAILABLE: usize = 0x07;
const DOUBLE_FAULT: usize = 0x08;
const INVALID_TSS: usize = 0x0A;
const SEGMENT_NOT_PRESENT: usize = 0x0B;
const STACK_SEGMENT_FAULT: usize = 0x0C;
const GENERAL_PROTECTION_FAULT: usize = 0x0D;
const PAGE_FAULT: usize = 0x0E;
const X87_FLOATING_POINT: usize = 0x10;
const ALIGNMENT_CHECK: usize = 0x11;
const MACHINE_CHECK: usize = 0x12;
const SIMD_FLOATING_POINT: usize = 0x13;
const VIRTUALIZATION: usize = 0x14;
const SECURITY_EXCEPTION: usize = 0x1E;

const EXCEPTION_COUNT: u8 = 0x20;
const COUNT: usize = (EXCEPTION_COUNT + pic8259::PIC_INTERRUPT_COUNT) as usize;

const PIC_FIRST_INTERRUPT: u8 = EXCEPTION_COUNT;

const PIC_BASE: usize = PIC_FIRST_INTERRUPT as usize;
const PIT: usize = PIC_BASE;
const KEYBOARD: usize = PIC_BASE + 0x1;
const CASCADE: usize = PIC_BASE + 0x2;
const COM2: usize = PIC_BASE + 0x3;
const COM1: usize = PIC_BASE + 0x4;
const LPT2: usize = PIC_BASE + 0x5;
const FLOPPY_DISK: usize = PIC_BASE + 0x6;
const LPT1: usize = PIC_BASE + 0x7;
const RTC: usize = PIC_BASE + 0x8;
const FREE_29: usize = PIC_BASE + 0x9;
const FREE_2A: usize = PIC_BASE + 0xA;
const FREE_2B: usize = PIC_BASE + 0xB;
const PS2_MOUSE: usize = PIC_BASE + 0xC;
const COPROCESSOR: usize = PIC_BASE + 0xD;
const ATA0: usize = PIC_BASE + 0xE;
const ATA1: usize = PIC_BASE + 0xF;

pub(super) fn init() {
    unsafe {
        pic8259::init(PIC_FIRST_INTERRUPT);
    }

    IDT.load();

    interrupts::enable();

    info!("traps init");
}
