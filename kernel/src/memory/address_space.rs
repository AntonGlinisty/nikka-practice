use core::{
    any,
    fmt,
    mem,
    mem::MaybeUninit,
    ptr,
};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::{
        info,
        trace,
        warn,
    },
};

use super::{
    FRAME_ALLOCATOR,
    Virt,
    block::Block,
    frage::{
        Frame,
        Page,
    },
    mapping::Mapping,
    mmu,
    mmu::PageTableFlags,
    page_allocator::PageAllocator,
};

#[derive(Debug)]
pub struct AddressSpace {
    name: &'static str,
    mapping: Mapping,
    page_allocator: PageAllocator,
}

impl AddressSpace {
    pub(super) fn new(
        name: &'static str,
        phys2virt: Page,
    ) -> Self {
        let mapping = Mapping::new(Mapping::current_page_table_root(), phys2virt);
        let page_allocator = PageAllocator::new(name, unsafe {
            mapping.page_table_ref(mapping.page_table_root())
        });

        let address_space = Self {
            name,
            mapping,
            page_allocator,
        };

        info!(%address_space, "init");

        address_space
    }

    fn zero(name: &'static str) -> Self {
        Self {
            name,
            mapping: Mapping::new(Frame::zero(), Page::zero()),
            page_allocator: PageAllocator::zero(name),
        }
    }

    pub fn allocate(
        &mut self,
        size: usize,
    ) -> Result<Block<Page>> {
        self.page_allocator.allocate(size)
    }

    pub unsafe fn map_page_to_frame(
        &mut self,
        page: Page,
        frame: Frame,
        flags: PageTableFlags,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    unsafe fn map_page(
        &mut self,
        page: Page,
        flags: PageTableFlags,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub unsafe fn unmap_page(
        &mut self,
        page: Page,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub unsafe fn map_block(
        &mut self,
        pages: Block<Page>,
        flags: PageTableFlags,
    ) -> Result<()> {
        for page in pages {
            self.map_page(page, flags)?;
        }

        Ok(())
    }

    pub unsafe fn unmap_block(
        &mut self,
        pages: Block<Page>,
    ) -> Result<()> {
        for page in pages {
            self.unmap_page(page)?;
        }

        Ok(())
    }

    pub fn map_slice<T: Clone>(
        &mut self,
        len: usize,
        flags: PageTableFlags,
        default: T,
    ) -> Result<&'static mut [T]> {
        if Page::SIZE % mem::align_of::<T>() != 0 {
            warn!(
                alignment = mem::align_of::<T>(),
                page_size = Page::SIZE,
                "can not handle alignments greater than the page size",
            );
            return Err(InvalidArgument);
        }

        let block = self.page_allocator.allocate(len * mem::size_of::<T>())?;

        unsafe {
            self.map_block(block, flags)?;
        }

        let addr = Page::address_by_index(block.start());
        let len = block.size() / mem::size_of::<T>();
        let slice = unsafe { addr.try_into_mut_slice::<MaybeUninit<T>>(len)? };

        for element in slice.iter_mut() {
            element.write(default.clone());
        }

        trace!(
            %addr,
            page_count = block.count(),
            slice = format_args!("[{}; {}]", any::type_name::<T>(), len),
            "mapped a slice",
        );

        Ok(unsafe { MaybeUninit::slice_assume_init_mut(slice) })
    }

    pub unsafe fn unmap_slice<T>(
        &mut self,
        slice: &mut [T],
    ) -> Result<()> {
        let ptr_range = slice.as_ptr_range();
        let start = Virt::from_ptr(ptr_range.start).into_usize();
        let end = Virt::from_ptr(ptr_range.end).into_usize();

        if start % Page::SIZE != 0 || end % Page::SIZE != 0 {
            warn!(
                start,
                end,
                page_size = Page::SIZE,
                "can not unmap slice that does not start and end on page boundary",
            );
            return Err(InvalidArgument);
        }

        let block = Block::<Page>::new(start / Page::SIZE, end / Page::SIZE);
        let page_count = block.count();

        for element in slice.iter_mut() {
            ptr::drop_in_place(element as *mut T);
        }

        self.unmap_block(block)?;

        trace!(
            addr = %Virt::new(start)?,
            page_count,
            slice = format_args!("[{}; {}]", any::type_name::<T>(), slice.len()),
            "unmapped a slice",
        );

        Ok(())
    }
}

impl fmt::Display for AddressSpace {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "\"{}\" @ {}",
            self.name,
            self.mapping.page_table_root().address()
        )
    }
}

lazy_static! {
    pub static ref BASE_ADDRESS_SPACE: Mutex<AddressSpace> =
        Mutex::new(AddressSpace::zero("invalid"));
}
