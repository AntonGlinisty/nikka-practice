use core::{
    convert::TryInto,
    fmt,
};

use crate::smp::Cpu;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Pid {
    Current,
    Id {
        epoch: u32,
        slot: u16,
    },
}

impl Pid {
    pub(super) const MAX_COUNT: usize = (1 << Self::EPOCH_SHIFT);

    const EPOCH_SHIFT: u32 = u16::BITS;

    pub(super) fn new(slot: usize) -> Self {
        Self::Id {
            epoch: 0,
            slot: slot.try_into().unwrap(),
        }
    }

    pub(super) fn slot(&self) -> usize {
        let pid = match self {
            Self::Current => Cpu::current_process().expect("no current process"),
            pid => *pid,
        };

        if let Self::Id { slot, .. } = pid {
            slot.into()
        } else {
            panic!(
                "wrong pid {:?} encountered when processing pid {:?}",
                pid, self
            )
        }
    }

    pub(super) fn next_epoch(&mut self) -> Self {
        if let Self::Id { epoch, .. } = self {
            *epoch += 1
        } else {
            panic!(
                "can not increment epoch in Pid::Current when processing pid {:?}",
                self
            )
        }

        *self
    }
}

impl fmt::Debug for Pid {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(formatter)
    }
}

impl fmt::Display for Pid {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Current => {
                write!(formatter, "<current>")
            },
            Self::Id { epoch, slot } => {
                write!(formatter, "{slot}:{epoch}")
            },
        }
    }
}
