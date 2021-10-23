use core::{
    mem,
    ptr,
    sync::atomic::{
        AtomicPtr,
        Ordering,
    },
};

use static_assertions::const_assert;

use super::{
    RingBuffer,
    memory::Page,
};

#[repr(C, align(4096))]
pub struct ProcessInfo {
    log: RingBuffer,
}

const_assert!(mem::align_of::<ProcessInfo>() == Page::SIZE);
const_assert!(mem::size_of::<ProcessInfo>() % Page::SIZE == 0);

impl ProcessInfo {
    pub fn new(log: RingBuffer) -> Self {
        Self { log }
    }

    pub fn log(&mut self) -> &mut RingBuffer {
        &mut self.log
    }
}

pub fn process_info() -> &'static mut ProcessInfo {
    let process_info = PROCESS_INFO.load(Ordering::Relaxed);
    unsafe { process_info.as_mut().expect("the process info is not initialized properly") }
}

pub fn set_process_info(process_info: &'static mut ProcessInfo) {
    PROCESS_INFO.store(process_info as *mut ProcessInfo, Ordering::Relaxed);
}

static PROCESS_INFO: AtomicPtr<ProcessInfo> = AtomicPtr::new(ptr::null_mut());
