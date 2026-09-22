#![allow(dead_code)]
#![allow(unused_features)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    env::args,
    error::Error,
    ffi::CString,
    fs::read_dir,
    os::unix::{
        fs::MetadataExt,
        prelude::RawFd,
    },
    path::{
        Path,
        PathBuf,
    },
    process::exit,
};

use nix::{
    sys::wait::{
        wait,
        waitpid,
    },
    unistd::{
        ForkResult,
        close,
        execvp,
        fork,
        pipe,
        read,
        write,
    },
};

static mut PIPE_IN: RawFd = 0;
static mut PIPE_OUT: RawFd = 0;

pub fn main() -> Result<(), Box<dyn Error>> {
    // TODO: your code here.
    unimplemented!();
}
