use core::fmt;

use bitflags::bitflags;

bitflags! {
    /// https://wiki.osdev.org/Exceptions#Error_code
    pub struct PageFaultInfo: usize {
        const PRESENT = 1 << 0;
        const WRITE = 1 << 1;
        const USER = 1 << 2;
        const RESERVED = 1 << 3;
        const EXECUTE = 1 << 4;
    }
}

impl fmt::Display for PageFaultInfo {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{:#b} = {} | {} | {}",
            self.bits(),
            if self.contains(Self::RESERVED) {
                "malformed page table (a reserved bit set)"
            } else if self.contains(Self::PRESENT) {
                "access mode violation"
            } else {
                "non-present page"
            },
            if self.contains(Self::EXECUTE) {
                "execute"
            } else if self.contains(Self::WRITE) {
                "write"
            } else {
                "read"
            },
            if self.contains(Self::USER) {
                "user"
            } else {
                "kernel"
            },
        )
    }
}
