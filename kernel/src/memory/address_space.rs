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
        Error::InvalidAlignment,
        Result,
    },
    log::{
        info,
        trace,
        warn,
    },
    process::Pid,
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

#[derive(Debug, Default)]
pub struct AddressSpace {
    kind: Kind,
    mapping: Mapping,
    page_allocator: PageAllocator,
    pid: Option<Pid>,
}

impl AddressSpace {
    pub(super) fn new(phys2virt: Page) -> Self {
        let mapping = Mapping::new(Mapping::current_page_table_root(), phys2virt);
        let page_allocator =
            PageAllocator::new(unsafe { mapping.page_table_ref(mapping.page_table_root()) });

        let address_space = Self {
            kind: Kind::Base,
            mapping,
            page_allocator,
            pid: None,
        };

        info!(%address_space, "init");

        address_space
    }

    pub(crate) fn duplicate(&self) -> Result<Self> {
        let mapping = self.mapping.duplicate()?;

        let page_allocator =
            PageAllocator::new(unsafe { mapping.page_table_ref(mapping.page_table_root()) });

        let address_space = Self {
            kind: Kind::Process,
            mapping,
            page_allocator,
            pid: None,
        };

        info!(%address_space, "duplicate");

        Ok(address_space)
    }

    pub fn mapping(&mut self) -> &mut Mapping {
        &mut self.mapping
    }

    pub fn page_table_root(&self) -> Frame {
        self.mapping.page_table_root()
    }

    pub(crate) fn set_pid(
        &mut self,
        pid: Pid,
    ) {
        self.pid = Some(pid)
    }

    pub(crate) fn switch_to(&self) {
        info!(address_space = %self, "switch to");

        unsafe {
            assert_ne!(self.page_table_root(), Frame::default());
            mmu::set_page_table_root(self.page_table_root());
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

    fn map_slice_uninit<T>(
        &mut self,
        len: usize,
        flags: PageTableFlags,
    ) -> Result<&'static mut [MaybeUninit<T>]> {
        if Page::SIZE % mem::align_of::<T>() != 0 {
            warn!(
                alignment = mem::align_of::<T>(),
                page_size = Page::SIZE,
                "can not handle alignments greater than the page size",
            );
            return Err(InvalidAlignment);
        }

        let block = self.page_allocator.allocate(len * mem::size_of::<T>())?;

        unsafe {
            self.map_block(block, flags)?;
        }

        let slice = unsafe { block.try_into_mut_slice()? };

        trace!(
            %block,
            page_count = block.count(),
            slice = format_args!("[{}; {}]", any::type_name::<T>(), slice.len()),
            "mapped a slice",
        );

        Ok(slice)
    }

    pub fn map_slice<T, F: Fn() -> T>(
        &mut self,
        len: usize,
        flags: PageTableFlags,
        default: F,
    ) -> Result<&'static mut [T]> {
        let slice = self.map_slice_uninit(len, flags)?;

        for element in slice.iter_mut() {
            element.write(default());
        }

        Ok(unsafe { MaybeUninit::slice_assume_init_mut(slice) })
    }

    pub unsafe fn map_slice_zeroed<T>(
        &mut self,
        len: usize,
        flags: PageTableFlags,
    ) -> Result<&'static mut [T]> {
        let slice = self.map_slice_uninit(len, flags)?;

        for element in slice.iter_mut() {
            *element = MaybeUninit::zeroed();
        }

        Ok(MaybeUninit::slice_assume_init_mut(slice))
    }

    pub unsafe fn unmap_slice<T>(
        &mut self,
        slice: &mut [T],
    ) -> Result<()> {
        let ptr_range = slice.as_ptr_range();
        let start = Page::new(Virt::from_ptr(ptr_range.start))?;
        let end = Page::new(Virt::from_ptr(ptr_range.end))?;

        let block = Block::new(start, end)?;
        let page_count = block.count();

        for element in slice.iter_mut() {
            ptr::drop_in_place(element as *mut T);
        }

        self.unmap_block(block)?;

        trace!(
            addr = %start.address(),
            page_count,
            slice = format_args!("[{}; {}]", any::type_name::<T>(), slice.len()),
            "unmapped a slice",
        );

        Ok(())
    }

    pub fn map_one<T, F: FnOnce() -> T>(
        &mut self,
        flags: PageTableFlags,
        default: F,
    ) -> Result<&'static mut T> {
        let slice = self.map_slice_uninit(1, flags)?;
        slice[0].write(default());
        Ok(unsafe { MaybeUninit::assume_init_mut(&mut slice[0]) })
    }

    pub unsafe fn unmap_one<T>(
        &mut self,
        element: &'static mut T,
    ) -> Result<()> {
        self.unmap_slice(Block::<Virt>::from_ref(element).try_into_mut_slice::<T>()?)
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        if Mapping::current_page_table_root() == self.page_table_root() {
            let base_address_space = BASE_ADDRESS_SPACE.lock();
            assert_ne!(self.page_table_root(), base_address_space.page_table_root());
            base_address_space.switch_to();
            info!(
                address_space = %self,
                switch_to = %base_address_space,
                "drop the current address space",
            );
        } else {
            info!(address_space = %self, "drop");
        }
    }
}

impl fmt::Display for AddressSpace {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let page_table_root = self.page_table_root().address();
        match self.kind {
            Kind::Invalid => write!(formatter, "\"invalid\" @ {page_table_root}"),
            Kind::Base => write!(formatter, "\"base\" @ {page_table_root}"),
            Kind::Process => match self.pid {
                Some(pid) => write!(formatter, "\"{pid}\" @ {page_table_root}"),
                None => write!(formatter, "\"process\" @ {page_table_root}"),
            },
        }
    }
}

#[derive(Debug, Default)]
enum Kind {
    #[default]
    Invalid,
    Base,
    Process,
}

lazy_static! {
    pub static ref BASE_ADDRESS_SPACE: Mutex<AddressSpace> = Mutex::new(AddressSpace::default());
}
