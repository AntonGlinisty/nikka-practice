use core::{
    fmt,
    num::TryFromIntError,
    result,
};

use super::pipe;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Elf(&'static str),
    FileExists,
    FileNotFound,
    Fmt(fmt::Error),
    Int(TryFromIntError),
    InvalidAlignment,
    InvalidArgument,
    LocalApic,
    Medium,
    NoData,
    NoDisk,
    NoFrame,
    NoPage,
    NoProcess,
    NoProcessSlot,
    NotDirectory,
    NotFile,
    Null,
    Overflow,
    PermissionDenied,
    Pipe(pipe::Error),
    Postcard(postcard::Error),
    Timeout,
    Unimplemented,
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Self {
        Error::Fmt(e)
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Int(e)
    }
}

impl From<pipe::Error> for Error {
    fn from(e: pipe::Error) -> Self {
        Error::Pipe(e)
    }
}

impl From<postcard::Error> for Error {
    fn from(e: postcard::Error) -> Self {
        Error::Postcard(e)
    }
}

pub type Result<T> = result::Result<T, Error>;
