#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![no_main]
#![no_std]

use core::ptr::NonNull;

use ku::{
    error::{
        Error::{
            InvalidArgument,
            Null,
            Overflow,
            PermissionDenied,
        },
        Result,
    },
    memory::{
        Block,
        Virt,
    },
    process::{
        ResultCode,
        Syscall,
    },
    time,
};

use lib::{
    entry,
    syscall,
};

entry!(main);

fn main() {
    let now = time::now();
    let timestamp = now.timestamp().try_into().unwrap();

    if syscall::log_value("user space can read the system time", timestamp).is_err() {
        generate_page_fault();
    }

    let block = Block::from_index(1, 1);
    if block.is_err() || log_block(block.unwrap(), 0).is_err() {
        generate_page_fault();
    }

    let block = Block::from_index(1, 2);
    if block.is_err() {
        syscall::log_value("the block should be ok", 0).unwrap();
        generate_page_fault();
    }
    let result = log_block(block.unwrap(), 0);
    match result {
        Ok(_) => {
            syscall::log_value("expected Err(PermissionDenied), got Ok", 0).unwrap();
            generate_page_fault();
        },
        Err(PermissionDenied) => {},
        Err(_) => {
            syscall::log_value(
                "expected Err(PermissionDenied), got another error",
                ResultCode::from(result).bits(),
            )
            .unwrap();
            generate_page_fault();
        },
    }

    let block = Block::from_index(0, 0);
    if block.is_err() {
        syscall::log_value("the block should be ok", 0).unwrap();
        generate_page_fault();
    }
    let result = log_block(block.unwrap(), 0);
    match result {
        Ok(_) => {
            syscall::log_value("expected Err(InvalidArgument) or Err(Null), got Ok", 0).unwrap();
            generate_page_fault();
        },
        Err(InvalidArgument | Null) => {},
        Err(_) => {
            syscall::log_value(
                "expected Err(InvalidArgument) or Err(Null), got another error",
                ResultCode::from(result).bits(),
            )
            .unwrap();
            generate_page_fault();
        },
    }

    let invalid_utf8 = b"\xFF";
    let result = log_block(Block::from_slice(invalid_utf8), 0);
    match result {
        Ok(_) => {
            syscall::log_value("expected Err(InvalidArgument), got Ok", 0).unwrap();
            generate_page_fault();
        },
        Err(InvalidArgument) => {},
        Err(_) => {
            syscall::log_value(
                "expected Err(InvalidArgument), got another error",
                ResultCode::from(result).bits(),
            )
            .unwrap();
            generate_page_fault();
        },
    }

    let result = log_something(0x1_0000, 0xFFFF_FFFF_0000_0000, 0);
    match result {
        Ok(_) => {
            syscall::log_value("expected Err(InvalidArgument), got Ok", 0).unwrap();
            generate_page_fault();
        },
        Err(InvalidArgument) => {},
        Err(_) => {
            syscall::log_value(
                "expected Err(InvalidArgument), got another error",
                ResultCode::from(result).bits(),
            )
            .unwrap();
            generate_page_fault();
        },
    }

    let result = log_something(0xFFFF_FFFF_FFFF_0000, 0x10_0000, 0);
    match result {
        Ok(_) => {
            syscall::log_value("expected Err(InvalidArgument) or Err(Overflow), got Ok", 0)
                .unwrap();
            generate_page_fault();
        },
        Err(InvalidArgument | Overflow) => {},
        Err(_) => {
            syscall::log_value(
                "expected Err(InvalidArgument) or Err(Overflow), got another error",
                ResultCode::from(result).bits(),
            )
            .unwrap();
            generate_page_fault();
        },
    }
}

fn generate_page_fault() -> ! {
    unsafe {
        NonNull::<u8>::dangling().as_ptr().read_volatile();
    }

    unreachable!();
}

fn log_block(
    block: Block<Virt>,
    value: usize,
) -> Result<()> {
    log_something(block.start_address().into_usize(), block.size(), value)
}

fn log_something(
    start: usize,
    size: usize,
    value: usize,
) -> Result<()> {
    syscall::syscall(Syscall::LOG_VALUE.bits(), start, size, value, 0, 0).map(|_| ())
}
