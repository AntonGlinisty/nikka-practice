use crate::time::{
    Pit,
    rtc,
};

use super::{
    TRAP_STATS,
    TrapContext,
};

pub(super) extern "x86-interrupt" fn pit(_context: TrapContext) {
    TRAP_STATS[super::PIT].inc();
    Pit::interrupt();
}

pub(super) extern "x86-interrupt" fn keyboard(_context: TrapContext) {
    TRAP_STATS[super::KEYBOARD].inc();
}

pub(super) extern "x86-interrupt" fn cascade(_context: TrapContext) {
    TRAP_STATS[super::CASCADE].inc();
}

pub(super) extern "x86-interrupt" fn com2(_context: TrapContext) {
    TRAP_STATS[super::COM2].inc();
}

pub(super) extern "x86-interrupt" fn com1(_context: TrapContext) {
    TRAP_STATS[super::COM1].inc();
}

pub(super) extern "x86-interrupt" fn lpt2(_context: TrapContext) {
    TRAP_STATS[super::LPT2].inc();
}

pub(super) extern "x86-interrupt" fn floppy_disk(_context: TrapContext) {
    TRAP_STATS[super::FLOPPY_DISK].inc();
}

pub(super) extern "x86-interrupt" fn lpt1(_context: TrapContext) {
    TRAP_STATS[super::LPT1].inc();
}

pub(super) extern "x86-interrupt" fn rtc(_context: TrapContext) {
    TRAP_STATS[super::RTC].inc();
    rtc::interrupt();
}

pub(super) extern "x86-interrupt" fn free_29(_context: TrapContext) {
    TRAP_STATS[super::FREE_29].inc();
}

pub(super) extern "x86-interrupt" fn free_2a(_context: TrapContext) {
    TRAP_STATS[super::FREE_2A].inc();
}

pub(super) extern "x86-interrupt" fn free_2b(_context: TrapContext) {
    TRAP_STATS[super::FREE_2B].inc();
}

pub(super) extern "x86-interrupt" fn ps2_mouse(_context: TrapContext) {
    TRAP_STATS[super::PS2_MOUSE].inc();
}

pub(super) extern "x86-interrupt" fn coprocessor(_context: TrapContext) {
    TRAP_STATS[super::COPROCESSOR].inc();
}

pub(super) extern "x86-interrupt" fn ata0(_context: TrapContext) {
    TRAP_STATS[super::ATA0].inc();
}

pub(super) extern "x86-interrupt" fn ata1(_context: TrapContext) {
    TRAP_STATS[super::ATA1].inc();
}
