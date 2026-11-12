use alloc::vec::Vec;
use core::fmt;

use lazy_static::lazy_static;

use ku::{process, sync::spinlock::{
    Spinlock,
    SpinlockGuard,
}};

use crate::{
    error::{
        Error::{
            InvalidArgument,
            NoProcess,
            NoProcessSlot,
        },
        Result,
    }, log::info, smp::Cpu, time,
};

use super::{
    Pid,
    Process,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Слот таблицы процессов.
#[allow(clippy::large_enum_variant)]
enum Slot {
    /// Слот свободен.
    Free {
        /// Хранит эпоху последнего процесса, занимавшего этот слот.
        pid: Pid,

        /// Провязывает свободные слоты в интрузивный список.
        next: Option<Pid>,
    },

    /// Слот занят.
    Used {
        /// Процесс, находящийся в этом слоте таблицы процессов.
        process: Spinlock<Process>,
    },
}

impl fmt::Display for Slot {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Slot::Free { pid, next } => write!(formatter, "Free {{ pid: {pid}, next: {next:?} }}"),
            Slot::Used { process } => write!(formatter, "Process {}", *process.lock()),
        }
    }
}

/// Таблица процессов.
#[derive(Default)]
pub struct Table {
    /// Голова списка свободных слотов таблицы.
    free: Option<Pid>,

    /// Количество процессов в таблице.
    process_count: usize,

    /// Слоты таблицы процессов.
    table: Vec<Slot>,
}

impl Table {
    /// Он создаёт таблицу процессов [`Table`] размера `len` элементов, заполняя её
    /// пустыми слотами [`Slot::Free`] с соответствующими индексам слотов полями [`Pid::Id::slot`].
    /// Эти пустые слоты провязывает в односвязный список с головой в поле [`Table::free`].
    pub(super) fn new(len: usize) -> Self {
        if len == 0 {
            return Self::default();
        }

        let mut table = alloc::vec::Vec::with_capacity(len);

        for index in 0..(len - 1) {
            table.push(Slot::Free { pid: Pid::new(index), next: Some(Pid::new(index + 1)) })
        }
        table.push(Slot::Free { pid: Pid::new(len - 1), next: None });

        Self {
            free: Some(Pid::new(0)),
            process_count: 0,
            table,
        }
    }

    /// Выделяет процессу `process` свободный слот таблицы и возвращает соответствующий [`Pid`].
    /// Если свободного слота нет, возвращает ошибку [`Error::NoProcessSlot`].
    pub(super) fn allocate(mut process: Process) -> Result<Pid> {
        let mut table = TABLE.lock();

        let pid = table.free.ok_or(NoProcessSlot)?;

        process.set_pid(pid);

        table.free = match &table.table[pid.slot()] {
            Slot::Free { next, .. } => *next,
            Slot::Used { .. } => panic!("The process table has broken down"),
        };
        
        table.process_count += 1;
        table.table[pid.slot()] = Slot::Used { process: Spinlock::new(process) };

        Ok(pid)
    }

    /// Удаляет процесс с заданным `pid`.
    /// При этом:
    ///   - Инкрементирует эпоху в освободившемся слоте.
    ///   - Вставляет слот в голову списка свободных слотов [`Table::free`].
    pub fn free(mut pid: Pid) -> Result<()> {
        let mut table = TABLE.lock();

        if pid == Pid::Current {
            pid = Cpu::current_process()?;
        }
        
        match table.table.get(pid.slot()).ok_or(NoProcess)? {
            Slot::Free { .. } => return Err(NoProcess),
            Slot::Used { process } => {
                if process.lock().pid() != pid {
                    return Err(NoProcess)
                }
            },
        };
        
        pid.next_epoch();

        table.table[pid.slot()] = Slot::Free { pid: pid, next: table.free };
        table.free = Some(pid);
        table.process_count -= 1;

        Ok(())
    }

    /// Возвращает захваченную спин-блокировку [`SpinlockGuard`] со структурой [`Process`]
    /// соответствующей идентификатору `pid`.
    /// Если процесса по указанному `pid` нет или тот же слот занят уже другим процессом,
    /// возвращает ошибку [`Error::NoProcess`].
    pub fn get(pid: Pid) -> Result<SpinlockGuard<'static, Process>> {
        let table = TABLE.lock();

        let new_pid = if pid == Pid::Current {
            Cpu::current_process()?
        } else {
            pid
        };

        match table.table.get(new_pid.slot()).ok_or(NoProcess)? {
            Slot::Free { .. } => return Err(NoProcess),
            Slot::Used { process } => {
                let lock = unsafe { forge_static_lifetime(process).lock() };
                if lock.pid() != new_pid {
                    return Err(NoProcess)
                }

                Ok(lock)
            },
        }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        assert!(
            self.table.is_empty(),
            "should not drop non-empty process table",
        );
    }
}

/// Обещает Rust, что слоты таблицы процессов [`Table`] имеют время жизни `'static`,
/// так как Rust не может проверить это самостоятельно.
///
/// Так как размер таблицы процессов [`Table`] после инициализации мы никогда не меняем,
/// и в частности не уменьшаем, время жизни каждого её слота --- практически `'static`.
unsafe fn forge_static_lifetime<T>(x: &T) -> &'static T {
    unsafe { &*(x as *const T) as &'static T }
}

lazy_static! {
    /// Таблица процессов.
    pub(super) static ref TABLE: Spinlock<Table> = Spinlock::new(Table::default());
}

#[doc(hidden)]
pub mod test_scaffolding {
    use crate::error::Result;

    use super::{
        Pid,
        Process,
        TABLE,
        Table,
    };

    pub fn init() {
        if TABLE.lock().table.is_empty() {
            *TABLE.lock() = Table::new(1);
        }
    }

    pub fn allocate(process: Process) -> Result<Pid> {
        Table::allocate(process)
    }
}
