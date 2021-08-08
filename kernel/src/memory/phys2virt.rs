use bootloader::BootInfo;

use super::{
    Virt,
    frage::Page,
};

pub(super) fn phys2virt(boot_info: &BootInfo) -> Page {
    let phys2virt =
        Virt::new_u64(boot_info.physical_memory_offset).expect("invalid phys2virt in BootInfo");

    Page::new(phys2virt)
}
