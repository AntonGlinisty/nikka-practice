mod acpi;
mod ap_init;
mod cpu;
mod local_apic;

use core::{
    cmp,
    mem,
};

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

pub(crate) use cpu::{
    Cpu,
    KERNEL_RSP_OFFSET_IN_CPU,
};
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

    let max_cpu_id = cmp::max(
        bootstrap_processor,
        *acpi.ap_ids().iter().max().unwrap_or(&bootstrap_processor),
    );
    let cpu_count = usize::from(max_cpu_id) + 1;
    info!(cpu = current_cpu, cpu_count, %local_apic_address, "Local APIC init");

    let mut cpus = cpu::init(cpu_count, current_cpu)?;

    info!(cpu = current_cpu, "Bootstrap Processor");

    for id in acpi.ap_ids() {
        ap_init::boot_ap(phys2virt, &mut cpus[usize::from(*id)])?;
    }

    // Intentionally leak Cpu stuctures as they should be 'static.
    mem::forget(cpus);

    test_scaffolding::set_initialized();

    Ok(())
}

#[doc(hidden)]
pub mod test_scaffolding {
    use core::sync::atomic::{
        AtomicBool,
        Ordering,
    };

    pub fn is_initialized() -> bool {
        INITIALIZED.load(Ordering::Relaxed)
    }

    pub(super) fn set_initialized() {
        INITIALIZED.store(true, Ordering::Relaxed);
    }

    static INITIALIZED: AtomicBool = AtomicBool::new(false);
}
