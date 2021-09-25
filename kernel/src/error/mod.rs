use core::{
    num::TryFromIntError,
    result,
};

#[derive(Debug)]
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

pub type Result<T> = result::Result<T, Error>;

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Int(e)
    }
}
