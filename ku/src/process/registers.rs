use core::{
    fmt,
    ops::Not,
};

use derive_more::{
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
};
use x86_64::registers::rflags;

use crate::memory::size;

/// [Регистр флагов](https://en.wikipedia.org/wiki/FLAGS_register).
#[derive(
    Clone,
    Copy,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Debug,
    Default,
    Eq,
    PartialEq,
)]
#[repr(transparent)]
pub struct RFlags(usize);

impl RFlags {
    pub const ALL: RFlags = RFlags(rflags::RFlags::all().bits() as usize);

    pub const INTERRUPT_FLAG: RFlags = RFlags(rflags::RFlags::INTERRUPT_FLAG.bits() as usize);
    pub const TRAP_FLAG: RFlags = RFlags(rflags::RFlags::TRAP_FLAG.bits() as usize);

    pub fn contains(
        &self,
        flags: RFlags,
    ) -> bool {
        self.0 & flags.0 == flags.0
    }

    pub fn into_usize(&self) -> usize {
        self.0
    }

    pub fn read() -> Self {
        Self(size::from(rflags::read_raw()))
    }

    pub unsafe fn write(&self) {
        rflags::write_raw(size::into_u64(self.0));
    }
}

impl fmt::Display for RFlags {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        static FLAGS: [&str; 22] = [
            "CF",
            "",
            "PF",
            "",
            "AF",
            "",
            "ZF",
            "SF",
            "TF",
            "IF",
            "DF",
            "OF",
            "",
            "",
            "NT",
            "",
            "RF",
            "VM",
            "AC",
            "VF",
            "VP",
            "ID",
        ];

        let mut separator = "";

        for i in (0 .. FLAGS.len()).rev() {
            if (self.0 >> i) & 1 != 0 && !FLAGS[i].is_empty() {
                write!(formatter, "{}{}", separator, FLAGS[i])?;
                separator = " ";
            }
        }

        Ok(())
    }
}

impl From<RFlags> for rflags::RFlags {
    fn from(rflags: RFlags) -> rflags::RFlags {
        rflags::RFlags::from_bits(size::into_u64(rflags.0))
            .expect("an incorrect flag is set in RFlags")
    }
}

impl Not for RFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(Self::ALL.0 & !self.0)
    }
}
