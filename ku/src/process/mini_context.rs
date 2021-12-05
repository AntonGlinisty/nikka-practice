use core::{
    fmt,
    mem,
};

use crate::{
    error::Result,
    memory::{
        Block,
        Virt,
    },
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct MiniContext {
    rip: Virt,
    rsp: Virt,
}

impl MiniContext {
    pub fn new(
        rip: Virt,
        rsp: Virt,
    ) -> Self {
        Self { rip, rsp }
    }

    pub fn rip(&self) -> Virt {
        self.rip
    }

    pub fn rsp(&self) -> Virt {
        self.rsp
    }

    pub fn push<T>(&mut self) -> Result<Block<Virt>> {
        let old_rsp = self.rsp;
        self.rsp = (old_rsp - mem::size_of::<T>())?;

        Block::new(self.rsp, old_rsp)
    }
}

impl fmt::Display for MiniContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{{ rip: {}, rsp: {} }}", self.rip, self.rsp)
    }
}
