use crate::time::{
    pit8254,
    rtc,
};

use super::{
    PIC_BASE,
    TRAP_STATS,
    TrapContext,
};

pub(super) extern "x86-interrupt" fn pit(_context: TrapContext) {
    pit8254::interrupt();
    generic_pic_interrupt(super::PIT);
}

pub(super) extern "x86-interrupt" fn keyboard(_context: TrapContext) {
    generic_pic_interrupt(super::KEYBOARD);
}

pub(super) extern "x86-interrupt" fn cascade(_context: TrapContext) {
    generic_pic_interrupt(super::CASCADE);
}

pub(super) extern "x86-interrupt" fn com2(_context: TrapContext) {
    generic_pic_interrupt(super::COM2);
}

pub(super) extern "x86-interrupt" fn com1(_context: TrapContext) {
    generic_pic_interrupt(super::COM1);
}

pub(super) extern "x86-interrupt" fn lpt2(_context: TrapContext) {
    generic_pic_interrupt(super::LPT2);
}

pub(super) extern "x86-interrupt" fn floppy_disk(_context: TrapContext) {
    generic_pic_interrupt(super::FLOPPY_DISK);
}

pub(super) extern "x86-interrupt" fn lpt1(_context: TrapContext) {
    generic_pic_interrupt(super::LPT1);
}

pub(super) extern "x86-interrupt" fn rtc(_context: TrapContext) {
    rtc::interrupt();
    generic_pic_interrupt(super::RTC);
}

pub(super) extern "x86-interrupt" fn free_29(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_29);
}

pub(super) extern "x86-interrupt" fn free_2a(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_2A);
}

pub(super) extern "x86-interrupt" fn free_2b(_context: TrapContext) {
    generic_pic_interrupt(super::FREE_2B);
}

pub(super) extern "x86-interrupt" fn ps2_mouse(_context: TrapContext) {
    generic_pic_interrupt(super::PS2_MOUSE);
}

pub(super) extern "x86-interrupt" fn coprocessor(_context: TrapContext) {
    generic_pic_interrupt(super::COPROCESSOR);
}

pub(super) extern "x86-interrupt" fn ata0(_context: TrapContext) {
    generic_pic_interrupt(super::ATA0);
}

pub(super) extern "x86-interrupt" fn ata1(_context: TrapContext) {
    generic_pic_interrupt(super::ATA1);
}

fn generic_pic_interrupt(number: usize) {
    TRAP_STATS[number].inc();
    unsafe {
        pic8259::end_of_interrupt(number - PIC_BASE);
    }
}
