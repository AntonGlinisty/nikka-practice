use core::result;

#[derive(Debug)]
pub enum Error {
    InvalidArgument,
    NoFrame,
    NoPage,
    Unimplemented,
}

pub type Result<T> = result::Result<T, Error>;
