use core::result;

#[derive(Debug)]
pub enum Error {
    InvalidArgument,
    NoFrame,
}

pub type Result<T> = result::Result<T, Error>;
