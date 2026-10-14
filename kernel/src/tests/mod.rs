mod fs_helpers;

mod mm_helpers;

#[macro_use]
mod process_helpers;

mod lab_1_hw_1_rtc;

mod lab_1_hw_3_high_resolution_date;

mod lab_1_hw_4_interrupts;

mod lab_2_mm_1_frame_allocator;

mod lab_2_mm_2_phys2virt;

mod lab_2_mm_3_block;

mod lab_2_mm_3_page_allocator;

mod lab_2_mm_4_translate;

mod lab_2_mm_5_map;

mod lab_2_mm_6_duplicate_subtree;

mod lab_2_mm_7_drop_subtree;

mod lab_2_mm_8_resize_frame_allocator;

mod lab_3_allocator_1_big;

mod lab_3_allocator_5_small;

mod lab_4_process_1_cpus;

mod lab_4_process_2_elf;

mod lab_4_process_3_permission_checks;

mod lab_4_process_4_user_mode;

mod lab_4_process_5_syscall;

mod lab_4_process_5_log_value;

mod lab_4_process_6_table;

mod lab_4_process_7_preemption;

mod lab_4_process_8_scheduler;

mod lab_5_sync_4_correlation_point;

// mod lab_5_sync_5_ap_init;

mod lab_7_um_2_memory_syscalls;

mod lab_7_um_3_eager_fork;

mod lab_7_um_4_trap_handler;

mod lab_7_um_5_cow_fork;

mod lab_8_fs_1_block_cache_1_write;

mod lab_8_fs_1_block_cache_2_read;

mod lab_8_fs_2_bitmap;

mod lab_8_fs_3_inode;

mod lab_8_fs_4_directory;

mod lab_8_fs_5_open;

use core::{
    any,
    cmp,
    mem,
    panic::PanicInfo,
    str,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use bootloader_api::{
    BootInfo,
    info::MemoryRegions,
};
use heapless::String;
use static_assertions::const_assert_eq;

use ku::{
    backtrace::Backtrace,
    error::{
        Error::{
            InvalidArgument,
            Overflow,
        },
        Result,
    },
    log::error,
    memory::mmu::PAGE_TABLE_ENTRY_COUNT,
};

use crate::{
    self as kernel,
    ExitCode,
    Subsystems,
    exit_qemu,
    fs::{
        Disk,
        SECTOR_SIZE,
    },
    memory::test_scaffolding,
};

/// Точка входа для запуска интеграционных тестов.
pub fn main(boot_info: &'static BootInfo) -> ! {
    let Err(error) = tests_main(boot_info);
    panic!("failed to run tests: {error:?}");
}

/// Запускает интеграционные тесты в режиме ядра.
fn tests_main(boot_info: &'static BootInfo) -> Result<!> {
    kernel::preinit();

    let config = read_config()?;

    let mut subsystems = None;
    for test in TESTS.iter().filter(|x| x.is_enabled(config.as_str())) {
        if let Some(subsystems) = subsystems &&
            subsystems != test.subsystems()
        {
            error!("incompatible test requirements for kernel subsystems");
            for test in TESTS.iter().filter(|x| x.is_enabled(config.as_str())) {
                error!(test = test.name(), requirement = ?test.subsystems());
            }
            exit_qemu(ExitCode::FAILURE);
        } else {
            subsystems = Some(test.subsystems());
        }
    }

    kernel::init_subsystems(boot_info, subsystems.unwrap_or(Subsystems::empty()));

    set_frame_count(&boot_info.memory_regions);
    if let Some(recursive_index) = boot_info.recursive_index.as_ref() {
        set_recursive_mapping(ku::memory::size::from(*recursive_index));
    }

    run_tests(config.as_str(), &TESTS);
}

/// Конфигурация теста.
struct Test<'a> {
    /// Необходимые для тесты подсистемы.
    subsystems: Subsystems,

    /// Код и имя теста.
    test: &'a dyn Testable,
}

impl<'a> Test<'a> {
    /// Создаёт конфигурацию теста.
    const fn new(
        test: &'a dyn Testable,
        subsystems: Subsystems,
    ) -> Self {
        Self { test, subsystems }
    }

    /// Имя теста.
    fn name(&self) -> &'static str {
        self.test.name()
    }

    /// Необходимые для тесты подсистемы.
    fn subsystems(&self) -> Subsystems {
        self.subsystems
    }

    /// Возвращает `true`, если `config` определяет, что тест нужно запустить.
    fn is_enabled(
        &self,
        config: &str,
    ) -> bool {
        let test_name = self.test.name();

        config.split('\n').take_while(|x| !x.is_empty()).any(|x| test_name.contains(x))
    }

    /// Запускает тест.
    fn run(&self) {
        self.test.run();
    }
}

/// Определяет интерфейс запуска теста.
trait Testable {
    /// Имя теста.
    fn name(&self) -> &'static str;

    /// Запускает тест.
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn name(&self) -> &'static str {
        any::type_name::<T>()
            .strip_prefix("kernel::tests::lab_")
            .expect("unexpected test location")
    }

    fn run(&self) {
        let test_name = self.name();
        let test_name = test_name.get(0 .. cmp::min(test_name.len(), 50)).expect("invalid range");
        println!("\n{test_name:-<60}");
        self();
        println!("{test_name:-<51} [passed]");
    }
}

/// Отмечает интеграционный тест как проваленный.
pub(super) fn fail_test(panic_info: &PanicInfo) -> ! {
    println!("{}", panic_info);

    if let Ok(backtrace) = Backtrace::current() {
        println!("{backtrace:?}");
    }

    println!("{:-<51} [failed]", "");

    exit_qemu(ExitCode::FAILURE)
}

/// Читает конфигурацию тестов с диска.
fn read_config() -> Result<String<SECTOR_SIZE>> {
    const SECTOR_SIZE_IN_U32: usize = SECTOR_SIZE / mem::size_of::<u32>();
    let mut buffer = [0_u32; SECTOR_SIZE_IN_U32];
    Disk::new(TEST_CONFIG_DISK)?.pio_read(0 .. 1, &mut buffer)?;

    let buffer_u8 = unsafe {
        type From = [u32; SECTOR_SIZE_IN_U32];
        type To = [u8; SECTOR_SIZE];
        const_assert_eq!(mem::size_of::<From>(), mem::size_of::<To>());
        mem::transmute::<&[u32; SECTOR_SIZE_IN_U32], &[u8; SECTOR_SIZE]>(&buffer)
    };
    let buffer_str = str::from_utf8(buffer_u8).map_err(|_| InvalidArgument)?;

    let mut config = String::new();
    config.push_str(buffer_str).map_err(|_| Overflow)?;

    Ok(config)
}

/// Запускает набор тестов `tests`.
fn run_tests(
    config: &str,
    tests: &[Test],
) -> ! {
    let count = tests.iter().filter(|x| x.is_enabled(config)).map(|x| x.subsystems()).count();
    println!("running {count} tests");

    for test in tests {
        if test.is_enabled(config) {
            test.run();
        }
    }

    exit_qemu(ExitCode::SUCCESS);
}

fn frame_count() -> usize {
    FRAME_COUNT.load(Ordering::Relaxed)
}

fn set_frame_count(memory_regions: &MemoryRegions) {
    FRAME_COUNT.store(
        test_scaffolding::physical(memory_regions).end(),
        Ordering::Relaxed,
    );
}

fn recursive_mapping() -> Option<usize> {
    let recursive_mapping = RECURSIVE_MAPPING.load(Ordering::Relaxed);
    if recursive_mapping < PAGE_TABLE_ENTRY_COUNT {
        Some(recursive_mapping)
    } else {
        None
    }
}

fn set_recursive_mapping(recursive_mapping: usize) {
    RECURSIVE_MAPPING.store(recursive_mapping, Ordering::Relaxed);
}

static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);
static RECURSIVE_MAPPING: AtomicUsize = AtomicUsize::new(usize::MAX);

const TEST_CONFIG_DISK: usize = 2;

const TESTS: [Test<'_>; 108] = [
    Test::new(
        &lab_1_hw_1_rtc::different_rtc_formats as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_1_rtc::rtc_read_inconsistent as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_3_high_resolution_date::points_beyond_interval as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_3_high_resolution_date::stress as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_3_high_resolution_date::high_resolution_date as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_4_interrupts::interrupts_enabled as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_4_interrupts::interrupts_disabled as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_1_hw_4_interrupts::failed_try_lock as &dyn Testable,
        NONE,
    ),
    Test::new(&lab_1_hw_4_interrupts::nested as &dyn Testable, NONE),
    Test::new(
        &lab_2_mm_1_frame_allocator::sanity_check as &dyn Testable,
        PHYS,
    ),
    Test::new(&lab_2_mm_1_frame_allocator::basic as &dyn Testable, PHYS),
    Test::new(
        &lab_2_mm_1_frame_allocator::allocated_frames_are_not_used as &dyn Testable,
        PHYS,
    ),
    Test::new(
        &lab_2_mm_1_frame_allocator::operations_on_absent_frames_are_ok as &dyn Testable,
        PHYS,
    ),
    Test::new(&lab_2_mm_2_phys2virt::basic as &dyn Testable, PHYS),
    Test::new(&lab_2_mm_3_block::block_page as &dyn Testable, NONE),
    Test::new(&lab_2_mm_3_block::block_virt as &dyn Testable, NONE),
    Test::new(
        &lab_2_mm_3_page_allocator::sanity_check as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_3_page_allocator::some_used_entries as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_3_page_allocator::allocate_block as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_3_page_allocator::allocate_page as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_3_page_allocator::allocate_two_pages as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_3_page_allocator::alignment as &dyn Testable,
        PHYS_PAGE_ALLOCATOR,
    ),
    Test::new(
        &lab_2_mm_4_translate::check_path as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_4_translate::check_translate as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_4_translate::map_intermediate as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_4_translate::no_excessive_intermediate_flags as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(&lab_2_mm_4_translate::huge_page as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_4_translate::no_page as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_4_translate::build_map as &dyn Testable, PHYS_VIRT),
    Test::new(
        &lab_2_mm_4_translate::shared_memory as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_4_translate::garbage_after_unmap as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(&lab_2_mm_4_translate::tlb_flush as &dyn Testable, PHYS_VIRT),
    Test::new(
        &lab_2_mm_4_translate::path_after_map as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(&lab_2_mm_4_translate::no_frame as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_5_map::map_slice as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_5_map::map_readable as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_5_map::map_writable as &dyn Testable, PHYS_VIRT),
    Test::new(&lab_2_mm_5_map::map_twice as &dyn Testable, PHYS_VIRT),
    Test::new(
        &lab_2_mm_5_map::unmap_nonmapped_page as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_5_map::kernel_and_user_are_separated as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::basic as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_and_drop as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_error as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_path as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_translate as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::all_frames_are_freed_on_drop as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::garbage_in_duplicate as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_only_kernel as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_6_duplicate_subtree::duplicate_works as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_7_drop_subtree::drop_subtree as &dyn Testable,
        PHYS_VIRT,
    ),
    Test::new(
        &lab_2_mm_8_resize_frame_allocator::sanity_check as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_3_allocator_1_big::basic as &dyn Testable, MEMORY),
    Test::new(&lab_3_allocator_1_big::alignment as &dyn Testable, MEMORY),
    Test::new(
        &lab_3_allocator_1_big::grow_and_shrink_unmap_old_block as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::shrink_is_not_a_noop as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::copy_mapping_frame_references as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::copy_mapping_same_block as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::copy_mapping_corner_cases as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::grow_and_shrink as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_3_allocator_1_big::paged_realloc_is_cheap as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_3_allocator_1_big::stress as &dyn Testable, MEMORY),
    Test::new(&lab_3_allocator_5_small::basic as &dyn Testable, MEMORY),
    Test::new(&lab_3_allocator_5_small::alignment as &dyn Testable, MEMORY),
    Test::new(
        &lab_3_allocator_5_small::grow_and_shrink as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_3_allocator_5_small::stress as &dyn Testable, MEMORY),
    Test::new(
        &lab_4_process_1_cpus::initialized as &dyn Testable,
        MEMORY_CPUS,
    ),
    Test::new(
        &lab_4_process_1_cpus::kernel_stacks as &dyn Testable,
        MEMORY_CPUS,
    ),
    Test::new(
        &lab_4_process_2_elf::create_process as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_2_elf::create_process_failure as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_2_elf::create_process_no_frame as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_3_permission_checks::user_rw as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_3_permission_checks::non_present as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_3_permission_checks::stress as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_4_process_4_user_mode::user_mode_page_fault as &dyn Testable,
        MEMORY_SMP,
    ),
    Test::new(
        &lab_4_process_4_user_mode::user_context_saved as &dyn Testable,
        MEMORY_SMP,
    ),
    Test::new(
        &lab_4_process_5_syscall::syscall_exit as &dyn Testable,
        MEMORY_SMP_SYSCALL,
    ),
    Test::new(
        &lab_4_process_5_syscall::syscall_log_value as &dyn Testable,
        MEMORY_SMP_SYSCALL,
    ),
    Test::new(
        &lab_4_process_5_log_value::log_value_implementation as &dyn Testable,
        MEMORY_SMP_SYSCALL,
    ),
    Test::new(
        &lab_4_process_6_table::basic as &dyn Testable,
        MEMORY_PROCESS,
    ),
    // Test::new(&lab_4_process_6_table::full_capacity as &dyn Testable, MEMORY_PROCESS),
    Test::new(
        &lab_4_process_7_preemption::preemption as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_4_process_7_preemption::flags_and_registers_are_saved as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_4_process_8_scheduler::syscall_sched_yield as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_4_process_8_scheduler::scheduler as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_4_process_8_scheduler::sched_yield_reschedules as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_5_sync_4_correlation_point::correlation_point_reader as &dyn Testable,
        NONE,
    ),
    Test::new(
        &lab_5_sync_4_correlation_point::correlation_point_writer as &dyn Testable,
        NONE,
    ),
    // Test::new(&lab_5_sync_4_correlation_point::down_the_rabbit_hole as &dyn Testable, NONE), // Uncomment it if you wish.
    // Test::new(&lab_5_sync_5_ap_init::ap_init as &dyn Testable, MEMORY),
    Test::new(
        &lab_7_um_2_memory_syscalls::map_syscall_group as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_2_memory_syscalls::copy_mapping_of_intersecting_blocks as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_2_memory_syscalls::copy_mapping_of_enormous_blocks as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_2_memory_syscalls::user_space_memory_allocator as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_3_eager_fork::exofork_syscall as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_3_eager_fork::eager_fork as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_4_trap_handler::trap_handler as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_7_um_5_cow_fork::cow_fork as &dyn Testable,
        MEMORY_SMP_PROCESS,
    ),
    Test::new(
        &lab_8_fs_1_block_cache_1_write::write as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_8_fs_1_block_cache_2_read::read_what_was_written as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_8_fs_2_bitmap::reserved_elements as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_8_fs_2_bitmap::allocation as &dyn Testable, MEMORY),
    Test::new(
        &lab_8_fs_2_bitmap::allocation_has_amortized_constant_complexity as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_8_fs_3_inode::block_entry as &dyn Testable, MEMORY),
    Test::new(&lab_8_fs_3_inode::read_write_speed as &dyn Testable, MEMORY),
    Test::new(&lab_8_fs_3_inode::write_read as &dyn Testable, MEMORY),
    Test::new(&lab_8_fs_3_inode::big_file as &dyn Testable, MEMORY),
    Test::new(&lab_8_fs_3_inode::set_size as &dyn Testable, MEMORY),
    Test::new(
        &lab_8_fs_4_directory::basic_operations as &dyn Testable,
        MEMORY,
    ),
    Test::new(
        &lab_8_fs_4_directory::big_directory as &dyn Testable,
        MEMORY,
    ),
    Test::new(&lab_8_fs_4_directory::max_name_len as &dyn Testable, MEMORY),
    Test::new(&lab_8_fs_5_open::fs as &dyn Testable, MEMORY),
];

const MEMORY: Subsystems = Subsystems::MEMORY;
const MEMORY_CPUS: Subsystems =
    Subsystems::from_bits_truncate(Subsystems::MEMORY.bits() | Subsystems::CPUS.bits());
const MEMORY_PROCESS: Subsystems =
    Subsystems::from_bits_truncate(Subsystems::MEMORY.bits() | Subsystems::PROCESS.bits());
const MEMORY_SMP: Subsystems =
    Subsystems::from_bits_truncate(Subsystems::MEMORY.bits() | Subsystems::SMP.bits());
const MEMORY_SMP_SYSCALL: Subsystems = Subsystems::from_bits_truncate(
    Subsystems::MEMORY.bits() | Subsystems::SMP.bits() | Subsystems::SYSCALL.bits(),
);
const MEMORY_SMP_PROCESS: Subsystems = Subsystems::from_bits_truncate(
    Subsystems::MEMORY.bits() | Subsystems::SMP.bits() | Subsystems::PROCESS.bits(),
);
const NONE: Subsystems = Subsystems::empty();
const PHYS: Subsystems = Subsystems::PHYS_MEMORY;
const PHYS_PAGE_ALLOCATOR: Subsystems =
    Subsystems::from_bits_truncate(Subsystems::PHYS_MEMORY.bits() | Subsystems::PAGE_ALLOCATOR.bits());
const PHYS_VIRT: Subsystems =
    Subsystems::from_bits_truncate(Subsystems::PHYS_MEMORY.bits() | Subsystems::VIRT_MEMORY.bits());
