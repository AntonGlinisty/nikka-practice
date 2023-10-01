use alloc::vec::Vec;
use core::{
    arch::asm,
    str,
};

use x86_64::registers::model_specific::{
    Efer,
    EferFlags,
    LStar,
    SFMask,
    Star,
};

use ku::{
    process::{
        ExitCode,
        MiniContext,
        RFlags,
        ResultCode,
        State,
        Syscall,
        SyscallResult,
    },
    sync::spinlock::SpinlockGuard,
};

use crate::{
    allocator::MemoryAllocator,
    error::{
        Error::{
            InvalidAlignment,
            InvalidArgument,
            NoPage,
            NoProcess,
            Overflow,
            PermissionDenied,
        },
        Result,
    },
    gdt::Gdt,
    log::{
        debug,
        error,
        info,
        trace,
        warn,
    },
    memory::{
        Block,
        FRAME_ALLOCATOR,
        Frame,
        KERNEL_RW,
        Page,
        USER,
        Virt,
        mmu::PageTableFlags,
    },
    smp::{
        Cpu,
        KERNEL_RSP_OFFSET_IN_CPU,
    },
};

use super::{
    Pid,
    Process,
    Scheduler,
    Table,
    TrapHandlerContext,
};

/// Инициализация системных вызовов.
/// Подготавливает процессор к выполнению инструкций
/// [syscall](https://www.felixcloutier.com/x86/syscall) и
/// [sysret](https://www.felixcloutier.com/x86/sysret).
pub(crate) fn init() {
    // TODO: your code here.
    unimplemented!();
}

/// Получает управление при выполнении инструкции
/// [syscall](https://www.felixcloutier.com/x86/syscall).
///
/// Переключает стек на стек ядра, разрешает прерывания и
/// передаёт управление в функцию [`syscall()`].
#[naked]
extern "C" fn syscall_trampoline() -> ! {
    unsafe {
        asm!(
            "
            // TODO: your code here.

            call {syscall}
            ",

            // TODO: your code here.
            syscall = sym syscall,
            options(noreturn),
        );
    }
}

/// Выполняет диспетчеризацию системных вызовов по аргументу `number` --- номеру системного вызова.
///
/// Передаёт в функции, реализующие конкретные системные вызовы,
/// нужную часть аргументов `arg0`--`arg4`.
/// После выполнения функции конкретного системного вызова,
/// с помощью функции [`sysret()`] возвращает управление в контекст пользователя,
/// задаваемый `rip` и `rsp`.
extern "C" fn syscall(
    // https://wiki.osdev.org/System_V_ABI#x86-64:
    // Parameters to functions are passed in the registers
    // rdi, rsi, rdx, rcx, r8, r9,
    // and further values are passed on the stack in reverse order.
    number: usize, // rdi
    arg0: usize,   // rsi
    arg1: usize,   // rdx
    rip: Virt,     // rcx
    rsp: Virt,     // r8
    arg2: usize,   // r9
    // Stack, push in reverse order.
    arg3: usize,
    arg4: usize,
) -> ! {
    assert!(
        RFlags::read().contains(RFlags::INTERRUPT_FLAG),
        "enable the interrupts during the system calls",
    );

    // TODO: your code here.
    unimplemented!();
}

/// С помощью инструкции [sysret](https://www.felixcloutier.com/x86/sysret)
/// возвращает управление в контекст пользователя `context`.
///
/// Передаёт пользователю результат системного вызова в виде кода успеха или ошибки `result` и
/// полезной нагрузки `res`.
fn sysret(
    context: MiniContext,
    result: Result<SyscallResult>,
) -> ! {
    // TODO: your code here.

    unsafe {
        asm!(
            "
            // TODO: your code here.

            sysretq
            ",

            // TODO: your code here.

            options(noreturn),
        );
    }
}

/// Выполняет системный вызов
/// [`lib::syscall::exit(code)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.exit.html).
///
/// Освобождает слот таблицы процессов и возвращается в контекст ядра,
/// из которого пользовательский процесс был запущен.
fn exit(
    process: SpinlockGuard<Process>,
    code: usize,
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::log_value(message, value)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.log_value.html).
///
/// Записывает в журнал строку `message` типа `&str`, заданную началом `start` и длиной `len`,
/// а также число `value`.
#[allow(unused_mut)] // TODO: remove before flight.
fn log_value(
    mut process: SpinlockGuard<Process>,
    start: usize,
    len: usize,
    value: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::sched_yield()`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.sched_yield.html).
///
/// Перепланирует процесс в конец очереди готовых к исполнению процессов и
/// забирает у него CPU функцией [`Process::sched_yield()`],
/// которая вернёт управление в другой контекст ядра ---
/// в контекст из которого была вызвана функция [`Process::enter_user_mode()`].
/// Текущий контекст исходного процесса --- `context` --- записывает в него,
/// чтобы в дальнейшем в него можно было вернуться через [`Process::enter_user_mode()`].
#[allow(unused_mut)] // TODO: remove before flight.
fn sched_yield(
    mut process: SpinlockGuard<Process>,
    context: MiniContext,
) -> ! {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::exofork()`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.exofork.html).
///
/// Создаёт копию вызывающего процесса `process` и возвращает исходному процессу [`Pid`] копии.
/// Внутри копии возвращает [`Pid::Current`].
/// При этом новый процесс создаётся практически без адресного пространства и не готовый к работе.
/// Поэтому он, в частности, не ставится в очередь планировщика.
/// Текущий контекст исходного процесса --- `context` --- записывает в копию, чтобы в копии
/// вернуться туда же, куда происходит возврат из системного вызова для вызывающего процесса.
#[allow(unused_mut)] // TODO: remove before flight.
fn exofork(
    mut process: SpinlockGuard<Process>,
    context: MiniContext,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::map(dst_pid, dst_block, flags)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.map.html).
///
/// Отображает в памяти процесса, заданного `dst_pid`, блок страниц размера `dst_size` байт
/// начиная с виртуального адреса `dst_address` с флагами доступа `flags`.
/// Если `dst_address` равен нулю,
/// сам выбирает свободный участок адресного пространства размера `dst_size`.
/// Выбранный виртуальный адрес возвращается как результирующее значение системного вызова.
/// Размер при этом не возвращается, так как он должен быть равен аргументу `dst_address`.
///
/// Если во `flags` нет флага [`PageTableFlags::PRESENT`],
/// происходит только выделение адресного пространства --- виртуальных страниц.
/// То есть, не происходит ни выделения физических фреймов, ни их отображения.
fn map(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
    dst_address: usize,
    dst_size: usize,
    flags: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::unmap(dst_pid, dst_block)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.unmap.html).
///
/// Удаляет из виртуальной памяти целевого процесса `dst_pid` блок страниц
/// размера `dst_size` байт начиная с виртуального адреса `dst_address`.
fn unmap(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
    dst_address: usize,
    dst_size: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::copy_mapping(dst_pid, src_block, dst_block, flags)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.copy_mapping.html).
///
/// Создаёт копию отображения виртуальной памяти из вызывающего процесса `process`
/// в процесс, заданный `dst_pid`.
/// Исходный диапазон начинается с виртуального адреса `src_address`,
/// целевой --- с виртуального адреса `dst_address`.
/// Размер диапазона --- `dst_size` байт.
/// В целевом процессе диапазон должен быть отображён с флагами `flags`.
/// Не допускает целевое отображение с более широким набором флагов, чем исходное.
/// После выполнения у процессов появляется область
/// [разделяемой памяти](https://en.wikipedia.org/wiki/Shared_memory).
/// При этом флаги [`PageTableFlags::WRITABLE`] и [`PageTableFlags::COPY_ON_WRITE`]
/// с точки зрения ядра имеют одинаковые логические привилегии, --- доступность страницы на запись.
fn copy_mapping(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
    src_address: usize,
    dst_address: usize,
    dst_size: usize,
    flags: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Проверяет, что заданный блок виртуальных страниц `block` отображён в
/// адресное пространство процесса `process` с корректно заданными флагами `flags`.
/// Возвращает вектор физических фреймов, в которые отображены эти страницы.
fn check_frames<'a>(
    process: &'a SpinlockGuard<Process>,
    block: Block<Page>,
    flags: PageTableFlags,
) -> Result<Vec<Frame, MemoryAllocator<'a>>> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет отображение `src_frames` в `dst_pages` с флагами `flags`
/// в адресное пространство процесса `process`.
/// Количества элементов в `src_frames` и `dst_pages` должны совпадать.
fn map_pages_to_frames(
    process: &SpinlockGuard<Process>,
    src_frames: Vec<Frame, MemoryAllocator>,
    dst_pages: Block<Page>,
    flags: PageTableFlags,
) -> Result<()> {
    assert_eq!(src_frames.len(), dst_pages.count());

    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::set_state(dst_pid, state)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.set_state.html).
///
/// Переводит целевой процесс, заданный идентификатором `dst_pid`, в заданное состояние `state`.
/// Ставит его в очередь планировщика в случае [`State::RUNNABLE`].
fn set_state(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
    state: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Выполняет системный вызов
/// [`lib::syscall::set_trap_handler(dst_pid, trap_handler, trap_stack)`](https://sergey-v-galtsev.gitlab.io/labs-description/doc/lib/syscall/fn.set_trap_handler.html).
///
/// Устанавливает для целевого процесса, заданного идентификатором `dst_pid`,
/// пользовательский обработчик прерывания с виртуальным адресом `rip` и стеком,
/// который задаётся блоком виртуальных адресов начиная с `stack_address` и размера `stack_size`.
/// Стек может быть не выровнен по границе страниц.
fn set_trap_handler(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
    rip: usize,
    stack_address: usize,
    stack_size: usize,
) -> Result<SyscallResult> {
    // TODO: your code here.
    unimplemented!();
}

/// Проверяет, что `address` и `size` задают корректно выровненный диапазон страниц,
/// целиком лежащий внутри одной из
/// [двух непрерывных половин](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details)
/// адресного пространства.
/// Возвращает блок соответствующих виртуальных страниц.
fn check_block(
    address: usize,
    size: usize,
) -> Result<Block<Page>> {
    // TODO: your code here.
    unimplemented!();
}

/// Проверяет, что заданная виртуальная страница `page` отображена в
/// адресное пространство процесса `process` с корректно заданными флагами `flags`.
/// Возвращает физический фрейм, в который она отображена.
fn check_frame(
    process: &SpinlockGuard<Process>,
    page: Page,
    flags: PageTableFlags,
) -> Result<Frame> {
    // TODO: your code here.
    unimplemented!();
}

/// Проверяет, что `flags` задаёт валидный набор флагов отображения страниц,
/// в котором обязательно должны быть включены флаги присутствия страницы в памяти и
/// разрешения доступа со стороны кода пользователя.
/// Возвращает входные `flags` в виде [`PageTableFlags`].
fn check_page_flags(flags: usize) -> Result<PageTableFlags> {
    // TODO: your code here.
    unimplemented!();
}

/// Проверяет, что процесс `process` имеет право модифицировать целевой процесс,
/// заданный своим идентификатором `dst_pid`.
/// Поглощает блокировку `process`, выдавая взамен блокировку на целевой процесс.
/// Целевой процесс может совпадать с `process`.
///
/// Модифицировать можно:
///   - Либо самого себя, задавая [`Pid::Current`] или явно собственный идентификатор [`Pid::Id`].
///   - Либо свой непосредственно дочерний процесс, задавая его идентификатор.
fn check_process_permissions(
    process: SpinlockGuard<Process>,
    dst_pid: usize,
) -> Result<SpinlockGuard<Process>> {
    // TODO: your code here.
    unimplemented!();
}

#[doc(hidden)]
pub mod test_scaffolding {
    use ku::{
        process::{
            MiniContext,
            SyscallResult,
        },
        sync::spinlock::SpinlockGuard,
    };

    use crate::error::Result;

    use super::super::Process;

    pub fn log_value(
        process: SpinlockGuard<Process>,
        start: usize,
        len: usize,
        value: usize,
    ) -> Result<SyscallResult> {
        super::log_value(process, start, len, value)
    }

    pub fn exofork(process: SpinlockGuard<Process>) -> Result<SyscallResult> {
        super::exofork(process, MiniContext::default())
    }

    pub fn map(
        process: SpinlockGuard<Process>,
        dst_pid: usize,
        dst_address: usize,
        dst_size: usize,
        flags: usize,
    ) -> Result<SyscallResult> {
        super::map(process, dst_pid, dst_address, dst_size, flags)
    }

    pub fn unmap(
        process: SpinlockGuard<Process>,
        dst_pid: usize,
        dst_address: usize,
        dst_size: usize,
    ) -> Result<SyscallResult> {
        super::unmap(process, dst_pid, dst_address, dst_size)
    }

    pub fn copy_mapping(
        process: SpinlockGuard<Process>,
        dst_pid: usize,
        src_address: usize,
        dst_address: usize,
        dst_size: usize,
        flags: usize,
    ) -> Result<SyscallResult> {
        super::copy_mapping(process, dst_pid, src_address, dst_address, dst_size, flags)
    }

    pub fn set_state(
        process: SpinlockGuard<Process>,
        dst_pid: usize,
        state: usize,
    ) -> Result<SyscallResult> {
        super::set_state(process, dst_pid, state)
    }
}
