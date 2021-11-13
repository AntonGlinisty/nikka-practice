use alloc::collections::VecDeque;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions;

use crate::{
    log::info,
    smp::LocalApic,
};

use super::{
    Pid,
    process::Process,
    table::Table,
};

pub struct Scheduler {
    queue: VecDeque<Pid>,
}

impl Scheduler {
    pub fn init(count: usize) {
        *SCHEDULER.lock() = Scheduler {
            queue: VecDeque::with_capacity(count),
        }
    }

    pub fn run_one() -> bool {
        // TODO: your code here.

        false
    }

    pub(crate) fn run() -> ! {
        test_scaffolding::scheduler_wait();

        let cpu = LocalApic::id();

        loop {
            if !Scheduler::run_one() {
                info!(cpu, "nothing to do");
                instructions::hlt();
            }
        }
    }

    pub fn enqueue(pid: Pid) {
        // TODO: your code here.
        unimplemented!();
    }

    fn dequeue() -> Option<Pid> {
        // TODO: your code here.
        unimplemented!();
    }
}

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler {
        queue: VecDeque::new()
    });
}

pub mod test_scaffolding {
    use core::sync::atomic::{
        AtomicBool,
        Ordering,
    };

    use x86_64::instructions;

    pub fn scheduler_enable() {
        ENABLED.store(true, Ordering::Release);
    }

    pub(super) fn scheduler_wait() {
        loop {
            if ENABLED.load(Ordering::Acquire) {
                return;
            }

            instructions::hlt();
        }
    }

    static ENABLED: AtomicBool = AtomicBool::new(false);
}
