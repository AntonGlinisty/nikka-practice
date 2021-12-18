use core::{
    convert::TryInto,
    fmt,
    mem,
};

use static_assertions::const_assert;

use crate::error::{
    Error::InvalidArgument,
    Result,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Pid {
    Current,
    Id {
        epoch: u32,
        slot: u16,
    },
}

impl Pid {
    pub const MAX_COUNT: usize = (1 << Self::EPOCH_SHIFT);

    const EPOCH_SHIFT: u32 = u16::BITS;
    const SLOT_MASK: usize = Self::MAX_COUNT - 1;

    pub fn new(slot: usize) -> Self {
        Self::Id {
            epoch: 0,
            slot: slot.try_into().unwrap(),
        }
    }

    pub fn from_usize(pid: usize) -> Result<Self> {
        if pid == usize::MAX - 1 {
            Ok(Self::Current)
        } else {
            let result = Self::Id {
                epoch: (pid >> Self::EPOCH_SHIFT) as u32,
                slot: (pid & Self::SLOT_MASK) as u16,
            };

            if result.into_usize() == pid {
                Ok(result)
            } else {
                Err(InvalidArgument)
            }
        }
    }

    pub fn into_usize(&self) -> usize {
        const_assert!(mem::size_of::<Pid>() <= mem::size_of::<usize>());
        const_assert!(mem::size_of::<Option<Pid>>() <= mem::size_of::<usize>());

        match self {
            Self::Current => usize::MAX - 1,
            Self::Id { epoch, slot } => (*epoch as usize) << Self::EPOCH_SHIFT | (*slot as usize),
        }
    }

    pub fn slot(&self) -> usize {
        let pid = match self {
            Self::Current => {
                unimplemented!();
            },
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

    pub fn next_epoch(&mut self) {
        if let Self::Id { epoch, .. } = self {
            *epoch = epoch.checked_add(1).expect("epoch overflow");
        } else {
            panic!(
                "can not increment epoch in Pid::Current when processing pid {:?}",
                self,
            );
        }
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
