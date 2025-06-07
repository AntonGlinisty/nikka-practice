use std::{
    alloc::Layout,
    fs::File,
    num::NonZeroUsize,
};

use nix::sys::mman::{
    self,
    MRemapFlags,
    MapFlags,
    ProtFlags,
};

use ku::{
    allocator::BigAllocator,
    error::{
        Error::PermissionDenied,
        Result,
    },
    memory::{
        Block,
        Page,
        USER_RW,
        mmu::PageTableFlags,
    },
};

pub struct Big {
    flags: PageTableFlags,
    last: Block<Page>,
}

impl Big {
    pub fn new() -> Self {
        Self {
            flags: USER_RW,
            last: Block::default(),
        }
    }

    pub(super) fn last(&self) -> Block<Page> {
        self.last
    }

    unsafe fn mmap(
        block: Block<Page>,
        prot_flags: ProtFlags,
    ) -> Block<Page> {
        let address = NonZeroUsize::new(block.start_address().into_usize());
        let size = NonZeroUsize::new(block.size()).expect("useless call to map an empty block");

        let mut flags = MapFlags::MAP_ANONYMOUS;
        flags |= if prot_flags == ProtFlags::PROT_NONE {
            MapFlags::MAP_PRIVATE
        } else {
            MapFlags::MAP_SHARED
        };
        if address.is_some() {
            flags |= MapFlags::MAP_FIXED;
        }

        let ptr = unsafe { mman::mmap::<File>(address, size, prot_flags, flags, None, 0).unwrap() };

        super::block(ptr, size.into()).unwrap()
    }

    pub(super) const GARBAGE: u8 = 0xFF;
}

unsafe impl BigAllocator for Big {
    fn flags(&self) -> PageTableFlags {
        self.flags
    }

    fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) -> Result<()> {
        if flags.is_user() {
            self.flags = flags;

            Ok(())
        } else {
            Err(PermissionDenied)
        }
    }

    fn reserve(
        &mut self,
        layout: Layout,
    ) -> Result<Block<Page>> {
        let block = Block::from_index(0, layout.pad_to_align().size()).unwrap().enclosing();

        Ok(unsafe { Self::mmap(block, ProtFlags::PROT_NONE) })
    }

    fn reserve_fixed(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        assert!(!block.is_empty());

        let reserved_block = unsafe { Self::mmap(block, ProtFlags::PROT_NONE) };

        assert_eq!(block, reserved_block);

        Ok(())
    }

    unsafe fn unreserve(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        assert!(!block.is_empty());

        unsafe {
            mman::munmap(
                block.start_address().try_into_mut_ptr().unwrap(),
                block.size(),
            )
            .unwrap();
        }

        Ok(())
    }

    unsafe fn rereserve(
        &mut self,
        _old_block: Block<Page>,
        _sub_block: Block<Page>,
    ) -> Result<()> {
        unimplemented!();
    }

    unsafe fn map(
        &mut self,
        block: Block<Page>,
        flags: PageTableFlags,
    ) -> Result<()> {
        assert!(!block.is_empty());

        let mut prot_flags = ProtFlags::PROT_READ;
        if flags.is_writable() {
            prot_flags |= ProtFlags::PROT_WRITE;
        }

        unsafe {
            self.last = Self::mmap(block, prot_flags);

            self.last.try_into_mut_slice().unwrap().fill(Self::GARBAGE);
        }

        Ok(())
    }

    unsafe fn unmap(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.reserve_fixed(block)
    }

    unsafe fn copy_mapping(
        &mut self,
        old_block: Block<Page>,
        new_block: Block<Page>,
        flags: Option<PageTableFlags>,
    ) -> Result<()> {
        assert!(!old_block.is_empty());
        assert_eq!(old_block.size(), new_block.size());
        assert_eq!(flags, None);

        let ptr = unsafe {
            mman::mremap(
                old_block.start_address().try_into_mut_ptr().unwrap(),
                0,
                new_block.size(),
                MRemapFlags::MREMAP_MAYMOVE | MRemapFlags::MREMAP_FIXED,
                Some(new_block.start_address().try_into_mut_ptr().unwrap()),
            )
            .unwrap()
        };

        self.last = super::block(ptr, new_block.size()).unwrap();

        Ok(())
    }
}
