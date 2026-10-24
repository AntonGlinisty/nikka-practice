use core::alloc::Layout;

use ku::{
    allocator::{
        BigAllocator,
        BigAllocatorPair,
    },
    error::{
        Error::InvalidArgument,
        Result,
    },
    memory::{
        Block,
        Page,
        mmu::PageTableFlags,
    },
};

use crate::memory::{AddressSpace, FrameGuard, Translate};

use super::BigPair;

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Аллокатор памяти, предназначенный для выделения памяти блоками виртуальных страниц.
pub struct Big<'a> {
    /// Адресное пространство, внутри которого аллокатор выделяет память.
    address_space: &'a mut AddressSpace,

    /// Флаги доступа к выделяемой аллокатором памяти.
    flags: PageTableFlags,
}

impl Big<'_> {
    /// Возвращает аллокатор памяти для постраничного выделения памяти внутри `address_space`.
    /// Выделяемая им память будет отображена с флагами `flags`.
    pub fn new(
        address_space: &mut AddressSpace,
        flags: PageTableFlags,
    ) -> Big<'_> {
        Big {
            address_space,
            flags,
        }
    }
}

unsafe impl BigAllocator for Big<'_> {
    fn flags(&self) -> PageTableFlags {
        self.flags
    }

    fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) -> Result<()> {
        self.flags = flags;
        Ok(())
    }

    fn reserve(
        &mut self,
        layout: Layout,
    ) -> Result<Block<Page>> {
        self.address_space.allocate(layout, self.flags)
    }

    fn reserve_fixed(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.address_space.reserve(block, self.flags)
    }

    unsafe fn unreserve(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.address_space.deallocate(block)
    }

    unsafe fn map(
        &mut self,
        block: Block<Page>,
        flags: PageTableFlags,
    ) -> Result<()> {
        unsafe { self.address_space.map_block(block, flags) }
    }

    unsafe fn unmap(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        unsafe { self.address_space.unmap_block(block) }
    }

    unsafe fn copy_mapping(
        &mut self,
        old_block: Block<Page>,
        new_block: Block<Page>,
        flags: Option<PageTableFlags>,
    ) -> Result<()> {
        if old_block.count() != new_block.count() ||
            old_block != new_block && !old_block.intersection(new_block).is_empty() ||
            old_block == new_block && flags.is_none() {
            return Err(InvalidArgument)
        }

        for (old_page, new_page) in old_block.into_iter().zip(new_block) {
            let old_path = self.address_space.path(old_page.address());
            let old_pte = old_path.pte()?;

            let old_frame = old_pte.frame()?;
            let old_frame_guard = FrameGuard::reference(old_frame);

            let old_flags = old_pte.flags();
            let new_flags = flags.unwrap_or(old_flags);

            unsafe {
                self.address_space.map_page_to_frame(new_page, old_frame_guard, new_flags)?;
            }
        }

        Ok(())
    }
}
