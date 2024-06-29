#![feature(custom_test_frameworks)]
#![feature(int_roundings)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

core::include!("../src/tests/lab_8_fs_5_open.rs");
