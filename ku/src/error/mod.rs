use core::{
    num::TryFromIntError,
    result,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Elf(&'static str),
    Int(TryFromIntError),
    InvalidArgument,
    NoFrame,
    NoPage,
    NoProcess,
    NoProcessSlot,
    Unimplemented,
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Int(e)
    }
}

pub type Result<T> = result::Result<T, Error>;
