use core::{
    fmt,
    mem,
};

use bitflags::bitflags;

use ku::memory::{
    Block,
    Phys,
    Port,
    size,
};

use super::{
    ConfigSpace,
    RoutingId,
};

/// Ширина адреса памяти.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Width {
    /// 32-битный адрес памяти.
    Memory32,

    /// 64-битный адрес памяти.
    Memory64,
}

/// Структура для работы с регистрами адресов памяти и
/// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
/// PCI--устройств --- Base Address Register (BAR).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bar {
    /// Регистр описывает диапазон физической памяти,
    /// через которое происходит взаимодействие с PCI--устройством ---
    /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O).
    Memory {
        /// Диапазон памяти
        /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O).
        block: Block<Phys>,

        /// Можно ли разрешить процессору упреждающее чтение блока.
        prefetchable: bool,

        /// Размер указателя, который поддерживает этот регистр.
        width: Width,
    },

    /// Регистр описывает диапазон
    /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O),
    /// через которые происходит взаимодействие с PCI--устройством.
    Port {
        /// Диапазон
        /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
        block: Block<Port<u8>>,
    },
}

impl Bar {
    // ANCHOR: new
    /// Читает BAR--регистр PCI--устройства, адресуемого `routing_id`,
    /// по смещению `offset` в пространстве конфигурации `config_space`.
    pub(super) fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
        offset: usize,
    ) -> Option<Self> {
        // ANCHOR_END: new
        // TODO: your code here.
        None // TODO: remove before flight.
    }

    /// Возвращает количество байт, которые занимает регистр в пространстве конфигурации.
    pub(super) fn size(bar: &Option<Self>) -> usize {
        if let &Some(Self::Memory { width, .. }) = bar &&
            width == Width::Memory64
        {
            mem::size_of::<u64>()
        } else {
            mem::size_of::<u32>()
        }
    }

    // ANCHOR: address_size
    /// Возвращает пару из адреса и размера области памяти или
    /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O),
    /// задаваемой BAR--регистром PCI--устройства, адресуемого `routing_id`,
    /// по смещению `offset` в пространстве конфигурации `config_space`.
    fn address_size(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
        offset: usize,
    ) -> (usize, usize) {
        // ANCHOR_END: address_size
        // TODO: your code here.
        unimplemented!();
    }
}

impl fmt::Display for Bar {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match *self {
            Self::Memory {
                block,
                prefetchable,
                width,
            } => {
                write!(
                    formatter,
                    "Memory {{ {block}{}{} }}",
                    if prefetchable {
                        ", prefetchable"
                    } else {
                        ""
                    },
                    if width == Width::Memory64 {
                        ", 64-bit"
                    } else {
                        ""
                    },
                )
            },

            Self::Port { block } => write!(formatter, "I/O {{ {block} }}"),
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Default, Eq, PartialEq)]
    /// Структура регистра команд и статуса PCI--устройства.
    pub struct CommandAndStatusRegister: u32 {
        /// Если установлено значение `1`, устройство может реагировать на доступ к
        /// [портам ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
        /// пространству ввода-вывода; в противном случае реакция устройства отключена.
        const IO_SPACE = 1 << 0;

        /// Если установлено значение `1`, устройство может реагировать на доступ к
        /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O).
        /// В противном случае реакция устройства отключена.
        const MEMORY_SPACE = 1 << 1;

        /// Если установлено значение `1`, устройство может вести себя как мастер шины.
        /// В противном случае устройство не может генерировать запросы PCI.
        const BUS_MASTER = 1 << 2;

        /// Если установлено значение `1`, то подача сигнала прерывания INTx# отключена.
        /// В противном случае подача сигнала прерывания включена.
        const INTERRUPT_DISABLE = 1 << 10;

        /// Состояние сигнала прерывания INTx# устройства.
        /// Если установлено значение `1` и бит запрета прерываний
        /// [`CommandAndStatusRegister::INTERRUPT_DISABLE`]
        /// установлен в `0`, сигнал прерывания будет активизирован.
        /// В противном случае сигнал прерывания будет проигнорирован.
        const INTERRUPT_STATUS = 1 << 19;
    }
}

/// Смещение регистра команд и статуса в пространстве конфигурации PCI--устройства.
pub(super) const COMMAND_ADDRESS: usize = 0x04;

/// Смещение первого BAR--регистра в пространстве конфигурации PCI--устройства.
pub(super) const BARS_START_ADDRESS: usize = 0x10;

/// Смещение следующего за последним BAR--регистром в пространстве конфигурации PCI--устройства.
pub(super) const BARS_END_ADDRESS: usize = 0x28;
