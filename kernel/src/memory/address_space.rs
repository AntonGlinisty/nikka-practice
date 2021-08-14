use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    error::Result,
    log::info,
};

use super::{
    block::Block,
    frage::{
        Frame,
        Page,
    },
    mapping::Mapping,
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
