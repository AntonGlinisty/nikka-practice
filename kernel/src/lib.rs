#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! Библиотека ядра.
//! Позволяет подключать ядро как при обычном запуске, так и при запуске интеграционных тестов.

#![cfg_attr(feature = "tests", feature(gen_blocks))]
#![cfg_attr(feature = "tests", feature(iter_is_partitioned))]
#![cfg_attr(feature = "tests", feature(ptr_as_uninit))]
#![cfg_attr(feature = "tests", feature(yield_expr))]
#![deny(warnings)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(custom_test_frameworks)]
#![feature(maybe_uninit_fill)]
#![feature(never_type)]
#![feature(slice_ptr_get)]
#![no_std]
#![test_runner(crate::test_runner)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

extern crate alloc;

/// Аллокаторы памяти общего назначения.
pub mod allocator;

/// Перечисление для возможных ошибок [`Error`] и соответствующий [`Result`].
pub mod error;

/// [Файловая система](https://en.wikipedia.org/wiki/File_system).
pub mod fs;

/// Поддержка журналирования макросами библиотеки [`tracing`].
pub mod log;

/// Здесь находится часть работы с памятью, которая происходит только в ядре.
pub mod memory;

/// Здесь находится часть работы с процессами, которая происходит только в ядре.
pub mod process;

/// Реализация работы с последовательным портом.
#[macro_use]
pub mod serial;

/// Поддержка симметричной многопроцессорности
/// ([Symmetric multiprocessing](https://en.wikipedia.org/wiki/Symmetric_multiprocessing), SMP).
pub mod smp;

/// Тесты, запускающиеся в режиме ядра.
#[cfg(feature = "tests")]
pub mod tests;

/// Здесь находится часть работы со временем, которая происходит только в ядре.
pub mod time;

/// Система обработки [прерываний](https://en.wikipedia.org/wiki/Interrupt).
///
/// External Interrupts in the x86 system:
///   - [Part 1. Interrupt controller evolution](https://habr.com/en/post/446312/)
///   - [Part 2. Linux kernel boot options](https://habr.com/en/post/501660/)
///   - [Part 3. Interrupt routing setup in a chipset, with the example of coreboot](https://habr.com/en/post/501912/)
pub mod trap;

use core::{
    mem,
    panic::PanicInfo,
};

use bitflags::bitflags;
use bootloader_api::{
    BootInfo,
    config::{
        BootloaderConfig,
        Mapping,
    },
};
use x86::io;

use ku::{
    self,
    SystemInfo,
    error::Error::NoPage,
    sync,
};

#[cfg(not(feature = "tests"))]
use ku::backtrace::Backtrace;

use log::info;
use memory::{
    Stack,
    gdt,
    size,
};

// Used in docs.
#[allow(unused)]
use error::Error;

bitflags! {
    /// Код выхода из qemu при запуске тестов.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ExitCode: u8 {
        /// Успешное завершение --- все тесты прошли.
        const SUCCESS = 1;

        /// Неуспешное завершение --- есть провалившийся тест.
        const FAILURE = 2;
    }
}

bitflags! {
    /// Разбивка на подсистемы для возможности инициализации только части из них в тестах.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Subsystems: usize {
        /// Память: подсистема физической памяти.
        const PHYS_MEMORY = 1 << 0;

        /// Память: подсистема виртуальной памяти.
        const VIRT_MEMORY = 1 << 1;

        /// Память: основной аллокатор физической памяти.
        const MAIN_FRAME_ALLOCATOR = 1 << 2;

        /// Все части подсистемы памяти.
        const MEMORY =
            Self::PHYS_MEMORY.bits() | Self::VIRT_MEMORY.bits() | Self::MAIN_FRAME_ALLOCATOR.bits();

        /// Процессы: системные вызовы.
        const SYSCALL = 1 << 4;

        /// Процессы: таблица процессов.
        const PROCESS_TABLE = 1 << 5;

        /// Процессы: планировщик.
        const SCHEDULER = 1 << 6;

        /// Все части подсистемы процессов.
        const PROCESS = Self::SYSCALL.bits() | Self::PROCESS_TABLE.bits() | Self::SCHEDULER.bits();

        /// Симметричной многопроцессорности: вектор структур для CPU--локальных данных.
        const CPUS = 1 << 7;

        /// Симметричной многопроцессорности: запуск Application Processors.
        const BOOT_APS = 1 << 8;

        /// Все части подсистемы симметричной многопроцессорности.
        const SMP = Self::CPUS.bits() | Self::BOOT_APS.bits();
    }
}

/// Первоначальная инициализация обязательной части подсистем ядра для тестов.
#[cold]
#[inline(never)]
fn preinit() {
    ku::set_system_info(&SYSTEM_INFO);

    smp::preinit();

    log::init();
    time::init();

    info!(now = %time::now(), tsc = ?time::timer(), "Nikka booted");

    gdt::init();
    trap::init();
}

/// Инициализация части подсистем ядра для тестов.
/// Аргумент `boot_info` содержит информацию от `bootloader`,
/// `subsystems` задаёт набор подсистем, которые нужно инициализировать.
#[cold]
#[inline(never)]
fn init_subsystems(
    boot_info: &'static BootInfo,
    subsystems: Subsystems,
) {
    let phys2virt = if subsystems.intersects(Subsystems::MEMORY) {
        memory::init(boot_info, subsystems)
    } else {
        Err(NoPage)
    };

    if subsystems.intersects(Subsystems::VIRT_MEMORY) {
        memory::fix_address_space();
    }

    if subsystems.intersects(Subsystems::SMP) &&
        let Ok(phys2virt) = phys2virt
    {
        smp::init(boot_info, phys2virt, subsystems);
    }

    if subsystems.intersects(Subsystems::PROCESS) {
        process::init(subsystems);
    }
}

/// Инициализация всех подсистем ядра.
/// Аргумент `boot_info` содержит информацию от `bootloader`.
#[cold]
#[inline(never)]
pub fn init(boot_info: &'static BootInfo) {
    preinit();
    init_subsystems(boot_info, Subsystems::all());
}

/// Выхода из qemu при запуске тестов с кодом `exit_code`.
/// Требует указания аргумента `-device isa-debug-exit,iobase=0xF4,iosize=0x04` при запуске qemu.
pub fn exit_qemu(exit_code: ExitCode) -> ! {
    info!(?exit_code, "exit qemu");

    /// [Порт ввода--вывода](https://wiki.osdev.org/Port_IO)
    /// для команды выхода из qemu.
    const EXIT_PORT: u16 = 0xF4;

    unsafe {
        io::outb(EXIT_PORT, exit_code.bits());

        ku::halt()
    }
}

/// Обработчик паники.
#[cold]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    sync::start_panicking();

    #[cfg(feature = "tests")]
    tests::fail_test(panic_info);

    #[cfg(not(feature = "tests"))]
    {
        println!("{panic_info}");

        if let Ok(backtrace) = Backtrace::current() {
            println!("{backtrace:?}");
        }

        unsafe { ku::halt() }
    }
}

/// Конфигурация `bootloader`, с которой он будет запускать ядро.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.page_table_recursive = Some(Mapping::Dynamic);
    config.kernel_stack_size = size::into_u64(2 * mem::size_of::<Stack>());

    config
};

/// Страница памяти с общей информацией о системе.
static SYSTEM_INFO: SystemInfo = SystemInfo::new();
