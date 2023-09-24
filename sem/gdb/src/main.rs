#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    fs,
    io::{
        Error,
        ErrorKind::InvalidData,
    },
};

#[inline(never)]
fn check_key(key: &[u8]) -> bool {
    // TODO: your code here.
    unimplemented!();
}

fn main() -> Result<(), Error> {
    let key = fs::read_to_string("key.txt")?;

    if check_key(key.as_bytes()) {
        println!("OK");

        Ok(())
    } else {
        Err(Error::from(InvalidData))
    }
}
