use core::{
    cmp,
    mem,
};

use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

use crate::{
    error::{
        Error::NoFrame,
        Result,
    },
    log::{
        debug,
        info,
        trace,
    },
    time,
};

use super::{
    BASE_ADDRESS_SPACE,
    BootFrameAllocator,
    FRAME_ALLOCATOR,
    FrameAllocator,
    KERNEL_RW,
    frage::Frame,
    range,
    size,
};

#[derive(Clone, Debug)]
enum FrameInfo {
    Absent,
    Free {
        next_free: Option<usize>,
    },
    Used {
        reference_count: usize,
    },
}

pub(super) fn init(memory_map: &MemoryMap) -> MainFrameAllocator {
    let timer = time::timer();
    let frame_allocator = MainFrameAllocator::new(memory_map);
    info!(
        frame_allocator = "main",
        free_frame_count = frame_allocator.count(),
        duration = %timer.elapsed(),
        "init",
    );
    frame_allocator
}

pub struct MainFrameAllocator {
    frame_info: &'static mut [FrameInfo],
    free_frame: Option<usize>,
    free_count: usize,
}

impl MainFrameAllocator {
    fn new(memory_map: &MemoryMap) -> Self {
        let mut timer = time::timer();

        let frame_count = range::physical(memory_map).count();
        let frame_info = BASE_ADDRESS_SPACE
            .lock()
            .map_slice(frame_count, KERNEL_RW, FrameInfo::Absent)
            .expect("failed to allocate memory for MainFrameAllocator metadata");

        debug!(frame_allocator = "main", duration = %timer.lap(), "frame info mapped");

        let mut main_frame_allocator = Self {
            frame_info,
            free_frame: None,
            free_count: 0,
        };

        let boot_frame_allocator = mem::replace(&mut *FRAME_ALLOCATOR.lock(), FrameAllocator::Void);

        if let FrameAllocator::Boot(boot_frame_allocator) = boot_frame_allocator {
            main_frame_allocator.init_frame_info(memory_map, &boot_frame_allocator);
            debug!(frame_allocator = "main", duration = %timer.lap(), "frame info init");

            main_frame_allocator.move_free_frames(boot_frame_allocator);
            debug!(frame_allocator = "main", duration = %timer.lap(), "moved all free frames");

            main_frame_allocator
        } else {
            panic!("FrameAllocator should be a Boot one now");
        }
    }

    pub(super) fn count(&self) -> usize {
        self.free_count
    }

    pub(super) fn allocate(&mut self) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn deallocate(
        &mut self,
        frame: Frame,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn reference(
        &mut self,
        frame: Frame,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        frame.index() >= self.frame_info.len() ||
            !matches!(self.frame_info[frame.index()], FrameInfo::Free { .. })
    }

    fn init_frame_info(
        &mut self,
        memory_map: &MemoryMap,
        boot_frame_allocator: &BootFrameAllocator,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    fn move_free_frames(
        &mut self,
        mut src: BootFrameAllocator,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for MainFrameAllocator {
    fn drop(&mut self) {
        panic!("can not drop MainFrameAllocator");
    }
}
