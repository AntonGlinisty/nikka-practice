use alloc::vec::Vec;
use core::{
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

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Данные из таблиц
/// [Advanced Configuration and Power Interface](https://en.wikipedia.org/wiki/ACPI) (ACPI)
/// которые используются при инициализации симметричной многопроцессорности
/// ([Symmetric multiprocessing](https://en.wikipedia.org/wiki/Symmetric_multiprocessing), SMP).
///
/// [Спецификация ACPI](https://uefi.org/sites/default/files/resources/ACPI_Spec_6_5_Aug29.pdf).
#[derive(Debug)]
pub(super) struct Acpi {
    /// Физический адрес
    /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O)
    /// local [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
    local_apic_address: Phys,

    /// Идентификатор Bootstrap Processor.
    bsp_id: CpuId,

    /// Идентификаторы доступных Application Processor.
    ap_ids: Vec<CpuId>,
}

impl Acpi {
    /// Заполняет структуру [`Acpi`] по таблицам ACPI, которые BIOS сохранил в памяти.
    /// Аргумент `phys2virt` --- виртуальная страница, с которой начинается линейное отображение
    /// всей физической памяти в виртуальную, см. [`phys2virt::map()`].
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

    /// Физический адрес
    /// [Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O)
    /// local [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
    pub(super) fn local_apic_address(&self) -> Phys {
        self.local_apic_address
    }

    /// Идентификатор Bootstrap Processor.
    pub(super) fn bsp_id(&self) -> CpuId {
        self.bsp_id
    }

    /// Идентификаторы доступных Application Processor.
    pub(super) fn ap_ids(&self) -> &[CpuId] {
        &self.ap_ids
    }
}

/// Возвращает [`Apic`] если в `interrupt_model` из таблиц ACPI указан соответствующий
/// контроллер прерываний.
/// Иначе возвращает ошибку [`Error::Unimplemented`].
fn apic(interrupt_model: InterruptModel) -> Result<Apic> {
    if let InterruptModel::Apic(apic) = interrupt_model {
        Ok(apic)
    } else {
        Err(Unimplemented)
    }
}

/// Выполняет поиск в памяти таблиц ACPI с помощью библиотеки [`acpi`].
/// И возвращает найденную информацию.
/// Аргумент `phys2virt` --- виртуальная страница, с которой начинается линейное отображение
/// всей физической памяти в виртуальную, см. [`phys2virt::map()`].
fn platform_info(phys2virt: Page) -> result::Result<PlatformInfo, AcpiError> {
    let acpi_mapper = AcpiMapper { phys2virt };
    let acpi_tables = unsafe { AcpiTables::search_for_rsdp_bios(acpi_mapper)? };
    acpi_tables.platform_info()
}

/// Возвращает идентификаторы доступных Application Processor по входной структуре `cpus`.
fn usable_aps(cpus: &ProcessorInfo) -> Vec<CpuId> {
    cpus.application_processors
        .iter()
        .filter(|cpu| cpu.state == ProcessorState::WaitingForSipi)
        .filter(|cpu| cpu.local_apic_id <= CpuId::MAX.into())
        .map(|cpu| cpu.local_apic_id.try_into().unwrap())
        .collect()
}

/// Структура для отображения физической памяти в виртуальную, которая нужна библиотеке [`acpi`].
#[derive(Clone, Copy)]
struct AcpiMapper {
    /// Виртуальная страница, с которой начинается линейное отображение
    /// всей физической памяти в виртуальную, см. [`phys2virt::map()`].
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
