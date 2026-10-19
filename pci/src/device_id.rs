use derive_getters::Getters;
use derive_more::Display;
use pci_ids::{
    Device,
    FromId,
    Vendor,
};

use super::{
    ConfigSpace,
    Id,
    RoutingId,
};

/// Идентификатор PCI--устройства.
#[derive(Clone, Copy, Debug, Default, Display, Getters)]
#[display("{}, rev. {} ({})", device, revision, vendor)]
pub struct DeviceId {
    /// Идентификатор устройства.
    device: Id<u16>,

    /// Идентификатор производителя устройства.
    vendor: Id<u16>,

    /// Номер ревизии устройства.
    revision: u8,
}

impl DeviceId {
    /// Читает идентификатор PCI--устройства, адресуемого `routing_id`,
    /// из пространства конфигурации `config_space`.
    /// Если по адресу `routing_id` нет устройства, возвращает [`None`].
    pub(super) fn new(
        config_space: &mut impl ConfigSpace,
        routing_id: RoutingId,
    ) -> Option<Self> {
        let vendor_device_id = unsafe { config_space.read(routing_id, VENDOR_ID_ADDRESS) };
        let (device, vendor) = vendor_device(vendor_device_id)?;

        const BAD_CAST_MESSAGE: &str = "Bad cast";

        let revision_id = unsafe { config_space.read(routing_id, REVISION_ADDRESS) };
        let revision = u8::try_from(revision_id & 0xFF).expect(BAD_CAST_MESSAGE);

        Some(DeviceId {
            device,
            vendor,
            revision,
        })
    }
}

/// Возвращает пару из идентификатора устройства и идентификатора производителя устройства,
/// закодированные в значении `data`.
/// Также подходит для декодирования пары из
/// идентификатора подустройства и идентификатора производителя подустройства.
pub(super) fn vendor_device(data: u32) -> Option<(Id<u16>, Id<u16>)> {
    const BAD_CAST_MESSAGE: &str = "Bad cast";

    let vendor = u16::try_from(data & 0xFFFF).expect(BAD_CAST_MESSAGE);
    if vendor == NO_DEVICE {
        return None;
    }
    let vendor_name = Vendor::from_id(vendor).map(|info| info.name());

    let device = u16::try_from(data >> 16).expect(BAD_CAST_MESSAGE);
    let device_name = Device::from_vid_pid(vendor, device).map(|info| info.name());

    Some((Id::new(device, device_name), Id::new(vendor, vendor_name)))
}

/// Смещение номера ревизии устройства в пространстве конфигурации PCI--устройства.
pub(super) const REVISION_ADDRESS: usize = 0x08;

/// Смещение идентификатора производителя устройства в пространстве конфигурации PCI--устройства.
const VENDOR_ID_ADDRESS: usize = 0x00;

/// Признак отсутствия устройства.
const NO_DEVICE: u16 = u16::MAX;
