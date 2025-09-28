use derive_getters::Getters;
use derive_more::Display;
use pci_ids::{
    self,
    FromId,
    Subclass,
};

use super::{
    ConfigSpace,
    Id,
    REVISION_ADDRESS,
    RoutingId,
};

/// Класс PCI--устройства.
#[derive(Clone, Copy, Debug, Default, Display, Getters)]
#[display("{} / {} / {}", class, subclass, interface)]
pub struct Class {
    /// Идентификатор класса PCI--устройства.
    class: Id<u8>,

    /// Идентификатор подкласса PCI--устройства.
    subclass: Id<u8>,

    /// Идентификатор интерфейса PCI--устройства.
    interface: Id<u8>,
}

impl Class {
    // ANCHOR: new
    /// Читает класс PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    pub(super) fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Self {
        // ANCHOR_END: new
        // TODO: your code here.
        Self::default() // TODO: remove before flight.
    }
}
