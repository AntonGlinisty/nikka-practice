use alloc::vec::Vec;
use core::{
    fmt,
    mem,
};

use lazy_static::lazy_static;
use spin::{
    Mutex,
    MutexGuard,
};

use crate::{
    error::{
        Error::{
            NoProcess,
            NoProcessSlot,
        },
        Result,
    },
    log::info,
    time,
};

use super::{
    Pid,
    Process,
};

#[allow(clippy::large_enum_variant)]
enum Slot {
    Free {
        pid: Pid,
        next: Option<Pid>,
    },
    Used {
        process: Mutex<Process>,
    },
}

impl fmt::Display for Slot {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Slot::Free { pid, next } => write!(formatter, "Free {{ pid: {pid}, next: {next:?} }}"),
            Slot::Used { process } => write!(formatter, "Process {}", *process.lock()),
        }
    }
}

#[derive(Default)]
pub struct Table {
    free: Option<Pid>,
    process_count: usize,
    table: Vec<Slot>,
}

impl Table {
    pub(super) fn new(len: usize) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    #[allow(unused_mut)] // TODO: remove before flight.
    pub(super) fn allocate(mut process: Process) -> Result<Pid> {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn free(process: MutexGuard<Process>) {
        // TODO: your code here.
        unimplemented!();
    }

    pub fn get(pid: Pid) -> Result<MutexGuard<'static, Process>> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        assert!(
            self.table.is_empty(),
            "should not drop non-empty process table",
        );
    }
}

unsafe fn forge_static_lifetime<T>(x: &T) -> &'static T {
    &*(x as *const T) as &'static T
}

lazy_static! {
    pub(super) static ref TABLE: Mutex<Table> = Mutex::new(Table::default());
}
