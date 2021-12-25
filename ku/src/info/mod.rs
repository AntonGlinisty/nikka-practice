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
    memory::{
        Block,
        Page,
        Virt,
    },
    process::Pid,
    time::{
        AtomicCorrelationInterval,
        rtc::TICKS_PER_SECOND,
    },
};

#[repr(C, align(4096))]
pub struct ProcessInfo {
    log: RingBuffer,
    pid: Pid,
    recursive_mapping: usize,
    system_info: *const SystemInfo,
}

const_assert!(mem::align_of::<ProcessInfo>() == Page::SIZE);
const_assert!(mem::size_of::<ProcessInfo>() % Page::SIZE == 0);

impl ProcessInfo {
    pub fn new(
        log: RingBuffer,
        recursive_mapping: usize,
        system_info: *const SystemInfo,
    ) -> Self {
        Self {
            log,
            pid: Pid::Current,
            recursive_mapping,
            system_info,
        }
    }

    pub fn log(&mut self) -> &mut RingBuffer {
        &mut self.log
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn set_pid(
        &mut self,
        pid: Pid,
    ) {
        self.pid = pid;
    }

    pub fn recursive_mapping(&self) -> usize {
        self.recursive_mapping
    }

    pub fn contains_address(
        &self,
        address: Virt,
    ) -> bool {
        [
            Block::<Virt>::from_ref(self),
            Block::<Virt>::from_ref(unsafe { &*self.system_info }),
            Block::<Virt>::from_ref(&self.log),
            self.log.block(),
        ]
        .iter()
        .any(|block| block.enclosing().contains_address(address))
    }
}

#[repr(C, align(4096))]
pub struct SystemInfo {
    rtc: AtomicCorrelationInterval<TICKS_PER_SECOND>,
}

const_assert!(mem::align_of::<SystemInfo>() == Page::SIZE);
const_assert!(mem::size_of::<SystemInfo>() % Page::SIZE == 0);

impl SystemInfo {
    pub const fn new() -> Self {
        Self {
            rtc: AtomicCorrelationInterval::new(),
        }
    }

    pub fn rtc(&self) -> &AtomicCorrelationInterval<TICKS_PER_SECOND> {
        &self.rtc
    }
}

pub fn process_info() -> &'static mut ProcessInfo {
    let process_info = PROCESS_INFO.load(Ordering::Relaxed);
    unsafe { process_info.as_mut().expect("the process info is not initialized properly") }
}

pub fn system_info() -> &'static SystemInfo {
    let system_info = SYSTEM_INFO.load(Ordering::Relaxed);
    unsafe { system_info.as_ref().expect("the system info is not initialized properly") }
}

pub fn set_process_info(process_info: &'static mut ProcessInfo) {
    let system_info =
        unsafe { mem::transmute::<*const SystemInfo, *mut SystemInfo>(process_info.system_info) };
    PROCESS_INFO.store(process_info as *mut ProcessInfo, Ordering::Relaxed);
    SYSTEM_INFO.store(system_info, Ordering::Relaxed);
}

pub fn set_system_info(system_info: &'static SystemInfo) {
    let system_info = system_info as *const SystemInfo;
    let system_info = unsafe { mem::transmute::<*const SystemInfo, *mut SystemInfo>(system_info) };
    SYSTEM_INFO.store(system_info, Ordering::Relaxed);
}

static PROCESS_INFO: AtomicPtr<ProcessInfo> = AtomicPtr::new(ptr::null_mut());
static SYSTEM_INFO: AtomicPtr<SystemInfo> = AtomicPtr::new(ptr::null_mut());
