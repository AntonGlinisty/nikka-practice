mod acpi;
mod ap_init;
mod local_apic;

use bootloader::BootInfo;

use crate::{
    error::{
        Error::Unimplemented,
        Result,
    },
    log::{
        info,
        warn,
    },
    memory::{
        Page,
        phys2virt,
    },
    time,
};

use self::acpi::Acpi;

pub(crate) use local_apic::{
    CpuId,
    LocalApic,
};

pub(super) fn init(boot_info: &'static BootInfo) {
    let timer = time::timer();
    let phys2virt = phys2virt::phys2virt(boot_info);

    if let Err(error) = init_smp(phys2virt) {
        warn!(?error, duration = %timer.elapsed(), "SMP init");
    } else {
        info!(duration = %timer.elapsed(), "SMP init");
    }
}

fn init_smp(phys2virt: Page) -> Result<()> {
    let acpi = Acpi::new(phys2virt)?;
    let local_apic_address = acpi.local_apic_address();

    LocalApic::map(local_apic_address)?;
    LocalApic::init();

    let bootstrap_processor = acpi.bsp_id();
    let current_cpu = LocalApic::id();
    if bootstrap_processor != current_cpu {
        warn!(
            bootstrap_processor,
            current_cpu, "ACPI Bootstrap Processor is not the current processor",
        );
        return Err(Unimplemented);
    }

    info!(cpu = current_cpu, cpu_count = 1 + acpi.ap_ids().len(), %local_apic_address, "Local APIC init");

    info!(cpu = current_cpu, "Bootstrap Processor");

    for id in acpi.ap_ids() {
        ap_init::boot_ap(phys2virt, *id)
            .unwrap_or_else(|_| panic!("failed to boot the Application Processor #{}", id));
    }

    Ok(())
}
