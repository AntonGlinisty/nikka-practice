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
        test_scaffolding::run_handler();

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
    use core::{
        mem,
        sync::atomic::{
            AtomicBool,
            AtomicPtr,
            Ordering,
        },
    };

    use static_assertions::const_assert;
    use x86_64::instructions;

    pub fn scheduler_enable() {
        ENABLED.store(true, Ordering::Release);
    }

    pub fn set_handler(handler: fn()) {
        HANDLER.store(handler as *mut _, Ordering::Relaxed);
    }

    pub(super) fn run_handler() {
        let handler = HANDLER.load(Ordering::Relaxed);
        if !handler.is_null() {
            unsafe {
                const_assert!(mem::size_of::<*const ()>() == mem::size_of::<fn()>());
                let handler = mem::transmute::<*const (), fn()>(handler);
                (handler)();
            }
        }
    }

    fn scheduler_wait() {
        loop {
            if ENABLED.load(Ordering::Acquire) {
                return;
            }

            instructions::hlt();
        }
    }

    static ENABLED: AtomicBool = AtomicBool::new(false);
    static HANDLER: AtomicPtr<()> = AtomicPtr::new(scheduler_wait as *mut _);
}
