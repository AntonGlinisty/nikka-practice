use core::mem;

use derive_getters::Getters;
use derive_more::Display;

use ku::memory::{
    Block,
    KiB,
    MiB,
    Phys,
    Port,
    size,
};

use super::{
    BARS_END_ADDRESS,
    BARS_START_ADDRESS,
    Bar,
    Class,
    ConfigSpace,
    DeviceId,
    Id,
    RoutingId,
    vendor_device,
};

/// Тип PCI--устройства.
#[derive(Clone, Copy, Debug, Display)]
pub enum Kind {
    /// Обычное (конечное) PCI--устройство.
    #[display("Normal device")]
    Normal {
        /// Регистры адресов памяти и
        /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
        /// PCI--устройства --- Base Address Registers (BARs).
        bars: [Option<Bar>; 6],
    },

    /// PCI--PCI мост.
    #[display(
        "PCI-to-PCI Bridge {{ primary: {:02X}, secondary: {:02X}, subordinate: {:02X} }}",
        primary,
        secondary,
        subordinate
    )]
    PciPciBridge {
        /// Регистры адресов памяти и
        /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
        /// PCI--PCI моста --- Base Address Registers (BARs).
        bars: [Option<Bar>; 2],

        /// Номер шины, расположенной непосредственно перед PCI--PCI мостом.
        primary: u8,

        /// Номер шины, расположенной непосредственно после PCI--PCI моста.
        secondary: u8,

        /// Наибольший номер шины из всех шин,
        /// к которым возможен доступ через PCI--PCI мост.
        subordinate: u8,

        /// Диапазон
        /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
        /// для всех PCI--устройств за PCI--PCI мостом.
        io_behind_bridge: Block<Port<u8>>,

        /// Диапазон физической памяти для
        /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O)
        /// всех PCI--устройств за PCI--PCI мостом.
        memory_behind_bridge: Block<Phys>,

        /// Диапазон физической памяти для
        /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O)
        /// всех PCI--устройств за PCI--PCI мостом,
        /// для которой разрешено упреждающее чтение.
        prefetchable_memory_behind_bridge: Block<Phys>,
    },

    /// PCI--[CardBus](https://en.wikipedia.org/wiki/PC_Card#CardBus) мост.
    #[display("PCI-to-CardBus Bridge")]
    CardBusBridge,

    /// Неизвестное устройство.
    Unknown,
}

/// Структура, описывающая PCI--устройство.
#[derive(Clone, Copy, Debug, Display, Getters)]
#[display(
    "{{ vendor: {}, device: {}, revision: {}, class: {}, subclass: {}, interface: {} }}",
    id.vendor(),
    id.device(),
    id.revision(),
    class.class(),
    class.subclass(),
    class.interface()
)]
pub struct Device {
    /// Класс PCI--устройства.
    class: Class,

    /// Идентификатор PCI--устройства.
    id: DeviceId,

    /// Поддерживает ли устройство несколько функций.
    is_multi_function: bool,

    /// Тип PCI--устройства.
    kind: Kind,

    /// Идентификатор подустройства.
    /// Например, конкретной платы, основанной на микросхеме,
    /// задаваемой основным идентификатором устройства.
    subvendor: Option<Id<u16>>,

    /// Идентификатор производителя подустройства.
    /// Например, производителя конкретной платы, основанной на микросхеме,
    /// задаваемой основным идентификатором устройства.
    subdevice: Option<Id<u16>>,
}

impl Device {
    /// Читает описание PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    /// Если по адресу `routing_id` нет устройства, возвращает [`None`].
    pub fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Option<Self> {
        let id = DeviceId::new(config_space, routing_id)?;
        let class = Class::new(config_space, routing_id);

        // TODO: your code here.
        let subvendor = None; // TODO: remove before flight.
        let subdevice = None; // TODO: remove before flight.
        let is_multi_function = false; // TODO: remove before flight.
        let kind = Self::read_normal(config_space, routing_id); // TODO: remove before flight.

        Some(Self {
            class,
            id,
            is_multi_function,
            kind,
            subvendor,
            subdevice,
        })
    }

    /// Возвращает тип [`Kind`] обычного PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    fn read_normal(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Kind {
        let bars = Self::read_bars(config_space, routing_id);

        assert_eq!(
            bars.len(),
            (BARS_END_ADDRESS - BARS_START_ADDRESS) / mem::size_of::<u32>(),
        );

        Kind::Normal { bars }
    }

    /// Возвращает тип [`Kind`] PCI--PCI моста, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    fn read_bridge(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Kind {
        // TODO: your code here.
        unimplemented!();
    }

    /// Читает `N` регистров адресов памяти и
    /// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
    /// --- Base Address Registers (BARs) ---
    /// PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    fn read_bars<const N: usize>(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> [Option<Bar>; N] {
        // TODO: your code here.
        [None; _] // TODO: remove before flight.
    }
}

/// Смещение регистра размера линии кэша в пространстве конфигурации PCI--устройства.
const CACHE_LINE_SIZE_ADDRESS: usize = 0x0C;

/// Смещение идентификатора производителя подустройства
/// в пространстве конфигурации PCI--устройства.
const SUBSYSTEM_VENDOR_ID_ADDRESS: usize = 0x2C;

/// Смещение регистра номера шины в пространстве конфигурации моста PCI--PCI.
const PRIMARY_BUS_NUMBER_ADDRESS: usize = 0x18;

/// Смещение регистра младших разрядов диапазона
/// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
/// в пространстве конфигурации моста PCI--PCI.
const IO_BASE_ADDRESS: usize = 0x1C;

/// Смещение регистра старших разрядов диапазона
/// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O)
/// в пространстве конфигурации моста PCI--PCI.
const IO_BASE_UPPER_ADDRESS: usize = 0x30;

/// Гранулярность регистров диапазона
/// [портов ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
const BRIDGE_IO_GRANULARITY: usize = 4 * KiB;

/// Смещение регистра диапазона физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O)
/// в пространстве конфигурации моста PCI--PCI.
const MEMORY_BASE_ADDRESS: usize = 0x20;

/// Гранулярность регистров диапазона физической памяти для
/// в пространстве конфигурации моста PCI--PCI.
const BRIDGE_MEMORY_GRANULARITY: usize = MiB;

/// Смещение регистра младших битов диапазона физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
/// для которой разрешено упреждающее чтение,
/// в пространстве конфигурации моста PCI--PCI.
const PREFETCHABLE_MEMORY_BASE_ADDRESS: usize = 0x24;

/// Смещение регистра старших битов адреса первого байта физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
/// для которой разрешено упреждающее чтение,
/// в пространстве конфигурации моста PCI--PCI.
const PREFETCHABLE_MEMORY_BASE_UPPER_ADDRESS: usize = 0x28;

/// Смещение регистра старших битов адреса последнего байта физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
/// для которой разрешено упреждающее чтение,
/// в пространстве конфигурации моста PCI--PCI.
const PREFETCHABLE_MEMORY_LIMIT_UPPER_ADDRESS: usize = 0x2C;

/// Маска ширины адресов физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
/// для которой разрешено упреждающее чтение.
const PREFETCHABLE_MEMORY_TYPE_MASK: usize = 0xF_0000;

/// Признак 64-разрядности адресов физической памяти для
/// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
/// для которой разрешено упреждающее чтение.
const PREFETCHABLE_MEMORY_TYPE_64: usize = 0x1_0000;

/// Смещение следующего за последним BAR--регистром в пространстве конфигурации моста PCI--PCI.
const BARS_END_ADDRESS_BRIDGE: usize = 0x18;
