use lazy_static::lazy_static;
use spin::Mutex;

use crate::error::{
    Error::NoFrame,
    Result,
};

use super::{
    BootFrameAllocator,
    Frame,
};

pub enum FrameAllocator {
    Void,
    Boot(BootFrameAllocator),
}

impl FrameAllocator {
    pub fn allocate(&mut self) -> Result<Frame> {
        match self {
            FrameAllocator::Void => Err(NoFrame),
            FrameAllocator::Boot(boot_allocator) => boot_allocator.allocate(),
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
        }
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator::Void);
}
