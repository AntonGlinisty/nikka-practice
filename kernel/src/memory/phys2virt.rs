use bootloader::BootInfo;

use super::{
    Page,
    Phys,
    Virt,
};

pub(crate) fn phys2virt(boot_info: &BootInfo) -> Page {
    let phys2virt =
        Virt::new_u64(boot_info.physical_memory_offset).expect("invalid phys2virt in BootInfo");

    Page::new(phys2virt)
}

pub(crate) fn map(
    phys2virt: Page,
    address: Phys,
) -> Virt {
    (phys2virt.address() + address.into_usize()).expect("bad Phys address for phys2virt::map()")
}
