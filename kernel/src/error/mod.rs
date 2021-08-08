use core::result;

#[derive(Debug)]
pub enum Error {
    InvalidArgument,
}

pub type Result<T> = result::Result<T, Error>;
