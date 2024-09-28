#![deny(warnings)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

core::include!("../src/tests/lab_1_hw_3_high_resolution_date.rs");
