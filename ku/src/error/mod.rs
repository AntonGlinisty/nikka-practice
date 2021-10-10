use core::result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidArgument,
    NoFrame,
}

pub type Result<T> = result::Result<T, Error>;
