#[macro_export]
macro_rules! init {
    () => {
        bootloader::entry_point!(test_entry);

        fn test_entry(boot_info: &'static bootloader::BootInfo) -> ! {
            kernel::init(boot_info);
            test_main();
            panic!("should not return to test_entry()")
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            kernel::fail_test(info)
        }
    };
}
