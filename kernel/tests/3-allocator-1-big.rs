#![deny(warnings)]
#![feature(allocator_api)]
#![feature(custom_test_frameworks)]
#![feature(maybe_uninit_fill, maybe_uninit_slice)]
#![feature(ptr_as_uninit)]
#![feature(slice_ptr_get)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

core::include!("../src/tests/lab_3_allocator_1_big.rs");
