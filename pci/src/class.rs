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
    /// Читает класс PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    pub(super) fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Self {
        let mut class_id = unsafe { config_space.read(routing_id, REVISION_ADDRESS) };
        const BAD_CAST_MESSAGE: &str = "Bad cast";

        class_id >>= 8;
        let interface = u8::try_from(class_id & 0xFF).expect(BAD_CAST_MESSAGE);

        class_id >>= 8;
        let subclass = u8::try_from(class_id & 0xFF).expect(BAD_CAST_MESSAGE);

        class_id >>= 8;
        let class = u8::try_from(class_id & 0xFF).expect(BAD_CAST_MESSAGE);
        let class_name = pci_ids::Class::from_id(class).map(|info|info.name());

        let subclass_obj = Subclass::from_cid_sid(class, subclass);
        let subclass_name = subclass_obj.map(|info| info.name());

        let interface_name = if let Some(found_subclass) = subclass_obj {
            found_subclass
                .prog_ifs()
                .find(|info| info.id() == interface)
                .map(|info| info.name())
        } else {
            None
        };

        Class {
            class: Id::new(class, class_name),
            subclass: Id::new(subclass, subclass_name),
            interface: Id::new(interface, interface_name),
        }
    }
}
