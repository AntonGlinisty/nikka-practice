#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! Библиотека для пользовательских процессов.

#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(slice_ptr_get)]
#![no_std]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

/// Аллокатор памяти общего назначения в пространстве пользователя, реализованный через
/// системные вызовы [`syscall::map()`], [`syscall::unmap()`] и [`syscall::copy_mapping()`].
pub mod allocator;

/// Вспомогательные функции для работы с виртуальными страницами
/// [`memory::copy_page`] и [`memory::temp_page()`],
/// а также с таблицами страниц [`memory::page_table()`].
pub mod memory;

/// Системные вызовы.
pub mod syscall;

use core::panic::PanicInfo;

use tracing_core::{
    dispatch,
    dispatch::Dispatch,
};

use ku::{
    backtrace::Backtrace,
    info,
    info::ProcessInfo,
    log::{
        LOG_COLLECTOR,
        error,
    },
    process::ExitCode,
};

/// Точка входа в процесс пользователя.
/// Получает от ядра ссылку `process_info` на информацию о текущем процессе.
#[no_mangle]
pub extern "C" fn _start(process_info: &'static mut ProcessInfo) -> ! {
    info::set_process_info(process_info);

    LOG_COLLECTOR.set_flush(syscall::sched_yield);

    dispatch::set_global_default(Dispatch::from_static(&LOG_COLLECTOR)).unwrap();

    extern "Rust" {
        fn main();
    }

    unsafe {
        main();
    }

    syscall::exit(ExitCode::OK.bits());
}

/// Обработчик паники.
#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Ok(backtrace) = Backtrace::with_stack(ku::process_info().stack()) {
        error!(message = %panic_info, %backtrace);
    } else {
        error!(message = %panic_info);
    }
    syscall::exit(ExitCode::PANIC.bits());
}

/// Задаёт функцию `main()` пользовательского процесса.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn check_main_signature() {
            let main: fn() = $path;

            main()
        }
    };
}
