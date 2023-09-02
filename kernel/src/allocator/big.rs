use core::alloc::Layout;

use ku::{
    allocator::BigAllocator,
    error::{
        Error::{
            InvalidAlignment,
            InvalidArgument,
        },
        Result,
    },
    memory::{
        Block,
        Page,
        mmu::PageTableFlags,
    },
};

use crate::{
    log::warn,
    memory::{
        AddressSpace,
        FRAME_ALLOCATOR,
    },
};

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
    ) -> Big {
        Big {
            address_space,
            flags,
        }
    }
}

unsafe impl BigAllocator for Big<'_> {
    fn reserve(
        &mut self,
        layout: Layout,
    ) -> Result<Block<Page>> {
        if layout.align() > Page::SIZE {
            warn!(
                ?layout,
                page_size = %Page::SIZE,
                "can not handle alignments greater than the page size",
            );
            return Err(InvalidAlignment);
        }

        let block = self.address_space.allocate(layout.size())?;
        self.address_space
            .info_mut()
            .allocation(block.size(), block.size(), block.count());

        Ok(block)
    }

    unsafe fn unreserve(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.address_space
            .info_mut()
            .deallocation(block.size(), block.size(), block.count());

        Ok(())
    }

    unsafe fn rereserve(
        &mut self,
        old_block: Block<Page>,
        sub_block: Block<Page>,
    ) -> Result<()> {
        if old_block.contains_block(sub_block) {
            self.address_space.info_mut().allocation(
                sub_block.size(),
                sub_block.size(),
                sub_block.count(),
            );
            self.address_space.info_mut().deallocation(
                old_block.size(),
                old_block.size(),
                old_block.count(),
            );

            Ok(())
        } else {
            Err(InvalidArgument)
        }
    }

    unsafe fn map(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.address_space.map_block(block, self.flags)
    }

    unsafe fn unmap(
        &mut self,
        block: Block<Page>,
    ) -> Result<()> {
        self.address_space.unmap_block(block)
    }

    unsafe fn copy_mapping(
        &mut self,
        old_block: Block<Page>,
        new_block: Block<Page>,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }
}
