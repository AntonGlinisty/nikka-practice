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

impl From<ku::Error> for Error {
    fn from(e: ku::Error) -> Self {
        match e {
            ku::Error::InvalidArgument => Error::NoFrame,
            ku::Error::NoFrame => Error::NoFrame,
        }
    }
}
