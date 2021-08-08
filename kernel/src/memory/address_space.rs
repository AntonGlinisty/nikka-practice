use core::fmt::{
    Display,
    Formatter,
    Result,
};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::log::info;

use super::{
    frage::Frame,
    mapping::Mapping,
};

#[derive(Debug)]
pub(super) struct AddressSpace {
    name: &'static str,
    mapping: Mapping,
}

impl AddressSpace {
    pub(super) fn new(name: &'static str) -> Self {
        let address_space = Self {
            name,
            mapping: Mapping::new(Mapping::current_page_table_root()),
        };

        info!(%address_space, "init");

        address_space
    }

    fn zero(name: &'static str) -> Self {
        Self {
            name,
            mapping: Mapping::new(Frame::zero()),
        }
    }
}

impl Display for AddressSpace {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        write!(
            formatter,
            "\"{}\" @ {}",
            self.name,
            self.mapping.page_table_root().address()
        )
    }
}

lazy_static! {
    pub(super) static ref BASE_ADDRESS_SPACE: Mutex<AddressSpace> =
        Mutex::new(AddressSpace::zero("invalid"));
}
