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
    /// Читает BAR--регистр PCI--устройства, адресуемого `routing_id`,
    /// по смещению `offset` в пространстве конфигурации `config_space`.
    pub(super) fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
        offset: usize,
    ) -> Option<Self> {
        let (address, size) = Self::address_size(config_space, routing_id, offset);
        if size == 0 {
            return None
        }

        let original_address = u64::from(unsafe { config_space.read(routing_id, offset) });

        let is_memory = original_address & 0x1 == 0;
        if is_memory {
            let block: Block<Phys> = Block::from_index(address, address.checked_add(size)?).ok()?;

            let prefetchable = original_address & 0x8 == 8;
            let width = if original_address & 0x6 == 0 {
                Width::Memory32
            } else {
                Width::Memory64
            };
            return Some(Bar::Memory { block: block, prefetchable, width })
        } else {
            let block: Block<Port<u8>> = Block::from_index(address, address.checked_add(size)?).ok()?;
            return Some(Bar::Port { block });
        }
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

    /// Возвращает пару из адреса и размера области памяти или
    /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O),
    /// задаваемой BAR--регистром PCI--устройства, адресуемого `routing_id`,
    /// по смещению `offset` в пространстве конфигурации `config_space`.
    fn address_size(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
        offset: usize,
    ) -> (usize, usize) {
        let original_address = u64::from(unsafe { config_space.read(routing_id, offset) });

        let is_memory = original_address & 0x1 == 0;
        let is_32_bit = !is_memory || original_address & 0x6 == 0;

        let mut result_address = original_address;
        let result_address_mask = if is_memory {
            0xFFFF_FFFF_FFFF_FFF0
        } else {
            0xFFFF_FFFF_FFFF_FFFC
        };

        let command_register = unsafe { config_space.read(routing_id, COMMAND_ADDRESS) };
        let new_command_register = command_register & 0xFFFF_FFFC;
        if is_memory {
            let is_32_bit = original_address & 0x6 == 0;
            let is_64_bit = original_address & 0x6 == 4;

            if is_64_bit {
                let original_next_address = unsafe { config_space.read(routing_id, offset + 4) };
                result_address = (u64::from(original_next_address) << 32) + original_address;
            } else if !is_32_bit {
                return (0, 0)
            }
        }

        if command_register != new_command_register {
            unsafe { config_space.write(routing_id, COMMAND_ADDRESS, new_command_register); }
        }
        let result_size = Self::get_size(config_space, routing_id, offset, result_address, is_memory, is_32_bit);
        if command_register != new_command_register {
            unsafe { config_space.write(routing_id, COMMAND_ADDRESS, command_register); }
        }

        result_address &= result_address_mask;

        const BAD_CAST_MESSAGE: &str = "BAD CAST";
        (usize::try_from(result_address).expect(BAD_CAST_MESSAGE), result_size)
    }

    fn get_size(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
        offset: usize,
        original_address: u64,
        is_memory: bool,
        is_32_bit: bool,
    ) -> usize {
        const SIZE_REQUEST_VALUE: u32 = 0xFFFF_FFFF;
        const BAD_CAST_MESSAGE: &str = "BAD_CAST";

        let result: u64;
        if is_32_bit {
            unsafe { config_space.write(routing_id, offset, SIZE_REQUEST_VALUE) };
            let size_response = unsafe { config_space.read(routing_id, offset) };

            unsafe { config_space.write(routing_id, offset, u32::try_from(original_address).expect(BAD_CAST_MESSAGE)) };

            let size_without_flags = if is_memory {
                size_response & 0xFFFF_FFF0
            } else {
                size_response & 0xFFFF_FFFC
            };
            result = u64::from((!size_without_flags).wrapping_add(1));
        } else {
            unsafe { config_space.write(routing_id, offset, SIZE_REQUEST_VALUE) };
            unsafe { config_space.write(routing_id, offset + 4, SIZE_REQUEST_VALUE) };

            let size_lower_response = unsafe { config_space.read(routing_id, offset) };
            let size_higher_response = unsafe { config_space.read(routing_id, offset + 4) };

            unsafe { config_space.write(routing_id, offset, u32::try_from(original_address & 0xFFFF_FFFF).expect(BAD_CAST_MESSAGE)) };
            unsafe { config_space.write(routing_id, offset + 4, u32::try_from(original_address >> 32).expect(BAD_CAST_MESSAGE)) };

            let size_without_flags = ((u64::from(size_higher_response) << 32) + u64::from(size_lower_response)) & 0xFFFF_FFFF_FFFF_FFF0;
            result = (!size_without_flags).wrapping_add(1);
        }

        return usize::try_from(result).expect(BAD_CAST_MESSAGE)
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
