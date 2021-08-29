use core::{
    num::TryFromIntError,
    result,
};

#[derive(Debug)]
pub enum Error {
    Int(TryFromIntError),
    InvalidArgument,
    NoFrame,
    NoPage,
    Unimplemented,
}

pub type Result<T> = result::Result<T, Error>;

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Int(e)
    }
}
