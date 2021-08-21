use lazy_static::lazy_static;
use spin::Mutex;

use crate::error::{
    Error::NoFrame,
    Result,
};

use super::{
    BootFrameAllocator,
    Frame,
    main_frame_allocator::MainFrameAllocator,
};

pub enum FrameAllocator {
    Void,
    Boot(BootFrameAllocator),
    Main(MainFrameAllocator),
}

impl FrameAllocator {
    pub fn count(&self) -> usize {
        match self {
            FrameAllocator::Void => 0,
            FrameAllocator::Boot(boot_allocator) => boot_allocator.count(),
            FrameAllocator::Main(main_allocator) => main_allocator.count(),
        }
    }

    pub fn allocate(&mut self) -> Result<Frame> {
        match self {
            FrameAllocator::Void => Err(NoFrame),
            FrameAllocator::Boot(boot_allocator) => boot_allocator.allocate(),
            FrameAllocator::Main(main_allocator) => main_allocator.allocate(),
        }
    }

    pub fn deallocate(
        &mut self,
        frame: Frame,
    ) {
        match self {
            FrameAllocator::Main(main_allocator) => main_allocator.deallocate(frame),
            _ => panic!("can not deallocate a frame without the main frame allocator"),
        }
    }

    pub fn reference(
        &mut self,
        frame: Frame,
    ) {
        match self {
            FrameAllocator::Main(main_allocator) => main_allocator.reference(frame),
            _ => panic!("can not reference a frame without the main frame allocator"),
        }
    }

    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        match self {
            FrameAllocator::Void =>
                panic!("no frames should be marked as used by void frame allocator"),
            FrameAllocator::Boot(boot_allocator) => boot_allocator.is_used(frame),
            FrameAllocator::Main(main_allocator) => main_allocator.is_used(frame),
        }
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::Void);
}
