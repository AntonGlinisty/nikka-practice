use core::fmt;

use spin::{
    Mutex,
    MutexGuard,
};

use ku::{
    ProcessInfo,
    RingBuffer,
    process::MiniContext,
};

use crate::{
    allocator::MemoryAllocator,
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::{
        debug,
        info,
    },
    memory::{
        AddressSpace,
        Block,
        USER_RW,
        Virt,
    },
    smp::Cpu,
};

use super::{
    Pid,
    Table,
    registers::Registers,
};

/// Состояние пользовательского процесса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum State {
    /// Процесс только что создан и не готов к запуску.
    Exofork,

    /// Процесс готов к вытеснению, но не выполняется в данный момент.
    Runnable,

    /// Процесс выполняется в данный момент.
    Running,
}

/// Описывает пользовательский процесс.
#[derive(Debug)]
pub struct Process {
    /// Виртуальное адресное пространство процесса.
    address_space: Mutex<AddressSpace>,

    /// Блок памяти, через который ядро предоставляет процессу информацию о нём.
    /// В этом блоке находится структура типа [`ProcessInfo`].
    info: Block<Virt>,

    /// Идентификатор процесса--родителя, который создал данный процесс.
    parent: Option<Pid>,

    /// Идентификатор процесса.
    pid: Pid,

    /// Состояние регистров процесса.
    registers: Registers,

    /// Состояние процесса.
    state: State,

    /// Контекст пользователя, в который передаются исключения и прерывания,
    /// относящиеся к данному процессу.
    /// Например, Page Fault при некорректном доступе к памяти в коде пользователя.
    trap_handler_context: TrapHandlerContext,
}

impl Process {
    /// Создаёт новый процесс.
    pub(super) fn new(
        address_space: AddressSpace,
        context: MiniContext,
        info: &'static mut ProcessInfo,
    ) -> Self {
        let info = Block::from_ref(info);

        Self {
            address_space: Mutex::new(address_space),
            info,
            parent: None,
            pid: Pid::Current,
            registers: Registers::new(context, info.start_address().into_usize()),
            state: State::Runnable,
            trap_handler_context: TrapHandlerContext::default(),
        }
    }

    /// Дублирует существующий процесс.
    pub(super) fn duplicate(
        &self,
        rax: usize,
        rdi: usize,
    ) -> Result<Self> {
        let mut address_space = self.address_space.lock().duplicate()?;

        address_space.switch_to();
        let info = Block::from_ref(super::map_process_info(&mut address_space)?);
        self.address_space.lock().switch_to();

        Ok(Self {
            address_space: Mutex::new(address_space),
            info,
            parent: Some(self.pid),
            pid: Pid::Current,
            registers: self.registers.duplicate(rax, rdi, info.start_address().into_usize()),
            state: State::Exofork,
            trap_handler_context: TrapHandlerContext::default(),
        })
    }

    /// Возвращает виртуальное адресное пространство процесса.
    pub fn address_space(&mut self) -> &mut AddressSpace {
        self.address_space.get_mut()
    }

    /// Захватывает и возвращает блокировку на виртуальное адресное пространство процесса.
    /// В отличие от [`Process::address_space()`] не требует эксклюзивную ссылку на [`Process`].
    /// Это позволяет одновременно держать ссылки и на адресное пространство процесса и
    /// на его аллокатор памяти [`MemoryAllocator`].
    pub(super) fn lock_address_space(&self) -> MutexGuard<'_, AddressSpace> {
        self.address_space.lock()
    }

    /// Возвращает аллокатор памяти общего назначения внутри адресного пространства процесса.
    pub(super) fn allocator(&self) -> MemoryAllocator {
        MemoryAllocator::new(&self.address_space)
    }

    /// Устанавливает минимальный контекст процесса.
    pub(super) fn set_context(
        &mut self,
        context: MiniContext,
    ) {
        self.registers.set_mini_context(context);
    }

    /// Возвращает ссылку на структуру [`ProcessInfo`],
    /// через которую ядро предоставляет процессу информацию о нём.
    unsafe fn info(&mut self) -> Result<&mut ProcessInfo> {
        let info = self
            .address_space
            .get_mut()
            .check_permission_mut::<ProcessInfo>(self.info, USER_RW)?;

        if info.len() == 1 {
            Ok(&mut info[0])
        } else {
            Err(InvalidArgument)
        }
    }

    /// Возвращает [`RingBuffer`] в который код пользователя записывает свои сообщения журнала.
    pub fn log(&mut self) -> Result<&mut RingBuffer> {
        let info = unsafe { self.info()? };
        let log = Block::from_ref(info.log());
        let address_space = self.address_space.get_mut();

        let log = address_space.check_permission_mut::<RingBuffer>(log, USER_RW)?;

        if log.len() == 1 {
            address_space.check_permission_mut::<u8>(log[0].block(), USER_RW)?;
            Ok(&mut log[0])
        } else {
            Err(InvalidArgument)
        }
    }

    /// Возвращает идентификатор процесса--родителя, который создал данный процесс.
    pub fn parent(&self) -> Option<Pid> {
        self.parent
    }

    /// Возвращает идентификатор процесса.
    pub fn pid(&self) -> Pid {
        assert_ne!(
            self.pid,
            Pid::Current,
            "the process has not been assigned a pid yet",
        );
        self.pid
    }

    /// Устанавливает идентификатор процесса.
    pub(super) fn set_pid(
        &mut self,
        pid: Pid,
    ) {
        if let Pid::Id { .. } = pid {
            self.pid = pid;
            self.address_space.get_mut().set_pid(pid);
        } else {
            panic!("assignment of a wrong pid to a process");
        }
    }

    /// Возвращает состояние процесса.
    pub(super) fn state(&self) -> State {
        self.state
    }

    /// Устанавливает состояние процесса.
    pub(super) fn set_state(
        &mut self,
        state: State,
    ) {
        self.state = state
    }

    /// Возвращает контекст пользователя, в который передаются исключения и прерывания,
    /// относящиеся к данному процессу.
    /// Например, Page Fault при некорректном доступе к памяти в коде пользователя.
    pub(crate) fn trap_handler_context(&self) -> TrapHandlerContext {
        self.trap_handler_context
    }

    /// Устанавливает контекст пользователя, в который передаются исключения и прерывания,
    /// относящиеся к данному процессу.
    /// Например, Page Fault при некорректном доступе к памяти в коде пользователя.
    pub(super) fn set_trap_handler_context(
        &mut self,
        trap_handler_context: TrapHandlerContext,
    ) {
        self.trap_handler_context = trap_handler_context;
    }

    /// Переходит в режим пользователя.
    /// Возвращает `true` если возврат из режима пользователя
    /// был совершён принудительно (по прерыванию таймера `trap::timer()`).
    pub fn enter_user_mode(mut process: MutexGuard<Process>) -> bool {
        let pid = process.pid;

        process.address_space.get_mut().switch_to();

        if let Ok(info) = unsafe { process.info() } {
            info.set_pid(pid);
        }

        Cpu::set_current_process(Some(pid));

        let registers = &mut process.registers as *mut Registers;

        process.state = State::Running;

        debug!(%pid, registers = %process.registers, "entering the user mode");

        drop(process);

        unsafe {
            Registers::switch_to(registers);
        }

        debug!(%pid, "leaving the user mode");

        Cpu::set_current_process(None);

        // TODO: your code here.
        false // TODO: remove before flight.
    }

    /// Вытесняет текущий исполняющийся процесс с процессора принудительно.
    ///
    /// Сохраняет контекст процесса пользователя,
    /// так как он не готов к вытеснению и порче регистров.
    /// Возвращается в контекст ядра, из которого этот процесс был запущен.
    /// Текущий контекст ядра уничтожается.
    pub(crate) extern "C" fn leave_user_mode() -> ! {
        unsafe {
            assert!(Cpu::current_process().is_ok());

            Registers::switch_from();
        }
    }

    /// Вытесняет текущий исполняющийся процесс с процессора по его собственному запросу.
    ///
    /// Не сохраняет контекст процесса пользователя, пользователь должен
    /// сохранить его сам в процедуре системного вызова `syscall::sched_yield()`.
    /// Возвращается в контекст ядра, из которого этот процесс был запущен.
    /// Текущий контекст ядра уничтожается.
    pub(crate) extern "C" fn sched_yield() -> ! {
        unsafe {
            assert!(Cpu::current_process().is_ok());

            Registers::sched_yield();
        }
    }
}

impl fmt::Display for Process {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ pid: {}, address_space: {}, {} }}",
            self.pid,
            self.address_space.lock(),
            self.registers.mini_context(),
        )
    }
}

/// Контекст пользователя, в который передаются исключения и прерывания,
/// относящиеся к данному процессу.
/// Например, Page Fault при некорректном доступе к памяти в коде пользователя.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TrapHandlerContext {
    /// Контекст пользовательского обработчика исключений и прерываний.
    mini_context: MiniContext,

    /// Блок памяти, отведённый под стек для
    /// пользовательского обработчика исключений и прерываний.
    stack: Block<Virt>,
}

impl TrapHandlerContext {
    /// Создаёт контекст пользовательского обработчика исключений и прерываний.
    pub(super) fn new(
        rip: Virt,
        stack: Block<Virt>,
    ) -> Result<Self> {
        Ok(Self {
            mini_context: MiniContext::new(rip, stack.end_address()?),
            stack,
        })
    }

    /// Возвращает `true` если указатель стека `rsp` указывает в стек
    /// пользовательского обработчика исключений и прерываний.
    pub(crate) fn contains(
        &self,
        rsp: Virt,
    ) -> bool {
        self.stack.contains_address(rsp)
    }

    /// Возвращает `true` если пользователь установил свой обработчик исключений.
    pub(crate) fn is_valid(&self) -> bool {
        self.stack != Block::default()
    }

    /// Возвращает контекст пользовательского обработчика исключений и прерываний.
    pub(crate) fn mini_context(&self) -> MiniContext {
        self.mini_context
    }
}

impl fmt::Display for TrapHandlerContext {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ {}, stack: {} }}",
            self.mini_context, self.stack,
        )
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use super::{
        super::registers::test_scaffolding,
        Process,
    };

    pub fn disable_interrupts(process: &mut Process) {
        test_scaffolding::disable_interrupts(&mut process.registers);
    }

    pub fn registers(process: &Process) -> [usize; 15] {
        test_scaffolding::registers(&process.registers)
    }
}
