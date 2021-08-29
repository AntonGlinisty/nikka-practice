mod acpi;

use bootloader::BootInfo;

use crate::{
    log::{
        info,
        warn,
    },
    memory::phys2virt,
};

use self::acpi::Acpi;

pub(super) fn init(boot_info: &'static BootInfo) {
    let phys2virt = phys2virt::phys2virt(boot_info);

    let acpi = Acpi::new(phys2virt);
    if let Ok(acpi) = acpi {
        info!(
            cpu_count = 1 + acpi.ap_ids().len(),
            bootstrap_processor = %acpi.bsp_id(),
            local_apic = %acpi.local_apic_address(),
            "SMP init",
        );
    } else {
        warn!(error = ?acpi.unwrap_err(), "SMP init");
    }
}
