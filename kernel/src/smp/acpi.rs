use alloc::vec::Vec;
use core::{
    convert::TryInto,
    ptr::NonNull,
    result,
};

use acpi::{
    AcpiError,
    AcpiHandler,
    AcpiTables,
    PhysicalMapping,
    PlatformInfo,
    platform::{
        ProcessorInfo,
        ProcessorState,
        interrupt::{
            Apic,
            InterruptModel,
        },
    },
};

use crate::{
    error::{
        Error::Unimplemented,
        Result,
    },
    log::{
        info,
        trace,
        warn,
    },
    memory::{
        Page,
        Phys,
        phys2virt,
    },
};

use super::CpuId;

#[derive(Debug)]
pub(super) struct Acpi {
    local_apic_address: Phys,
    bsp_id: CpuId,
    ap_ids: Vec<CpuId>,
}

impl Acpi {
    pub(super) fn new(phys2virt: Page) -> Result<Self> {
        let platform_info = match platform_info(phys2virt) {
            Ok(platform_info) => platform_info,
            Err(acpi_error) => {
                warn!(?acpi_error, "failed to parse ACPI tables");
                return Err(Unimplemented);
            },
        };

        let apic = apic(platform_info.interrupt_model)?;
        let cpus = platform_info.processor_info.ok_or(Unimplemented)?;

        let acpi = Self {
            local_apic_address: Phys::new_u64(apic.local_apic_address)?,
            bsp_id: cpus.boot_processor.local_apic_id.try_into()?,
            ap_ids: usable_aps(&cpus),
        };

        trace!(
            bsp = ?cpus.boot_processor,
            ap = ?cpus.application_processors,
            power_profile = ?platform_info.power_profile,
            ?apic,
            "raw ACPI info",
        );

        info!(?acpi);

        Ok(acpi)
    }

    pub(super) fn local_apic_address(&self) -> Phys {
        self.local_apic_address
    }

    pub(super) fn bsp_id(&self) -> CpuId {
        self.bsp_id
    }

    pub(super) fn ap_ids(&self) -> &[CpuId] {
        &self.ap_ids
    }
}

fn apic(interrupt_model: InterruptModel) -> Result<Apic> {
    if let InterruptModel::Apic(apic) = interrupt_model {
        Ok(apic)
    } else {
        Err(Unimplemented)
    }
}

fn platform_info(phys2virt: Page) -> result::Result<PlatformInfo, AcpiError> {
    let acpi_mapper = AcpiMapper { phys2virt };
    let acpi_tables = unsafe { AcpiTables::search_for_rsdp_bios(acpi_mapper)? };
    acpi_tables.platform_info()
}

fn usable_aps(cpus: &ProcessorInfo) -> Vec<CpuId> {
    cpus.application_processors
        .iter()
        .filter(|cpu| cpu.state == ProcessorState::WaitingForSipi)
        .filter(|cpu| cpu.local_apic_id <= CpuId::MAX.into())
        .map(|cpu| cpu.local_apic_id.try_into().unwrap())
        .collect()
}

#[derive(Clone, Copy)]
struct AcpiMapper {
    phys2virt: Page,
}

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let address = Phys::new(physical_address)
            .expect("AcpiMapper::map_physical_region() is called with an invalid physical address");

        let pointer = phys2virt::map(self.phys2virt, address).try_into_mut_ptr().expect(
            "AcpiMapper::map_physical_region<T>() is called with an unsuitable address for the \
             requested type T",
        );
        let pointer =
            NonNull::new(pointer).expect("phys2virt::map() should not return a null pointer");

        PhysicalMapping::new(physical_address, pointer, size, size, *self)
    }

    fn unmap_physical_region<T>(_: &PhysicalMapping<Self, T>) {
    }
}
