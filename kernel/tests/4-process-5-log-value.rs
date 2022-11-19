#![feature(custom_test_frameworks)]
#![feature(ptr_as_uninit)]
#![feature(slice_ptr_get)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

core::include!("../src/tests/lab_4_process_5_log_value.rs");
