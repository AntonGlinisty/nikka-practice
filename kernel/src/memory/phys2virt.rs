use bootloader::BootInfo;

use super::{
    Page,
    Phys,
    Virt,
};

/// Возвращает виртуальную страницу `phys2virt`, с которой начинается линейное отображение
/// всей физической памяти в виртуальную.
pub(crate) fn phys2virt(boot_info: &BootInfo) -> Page {
    Virt::new_u64(boot_info.physical_memory_offset)
        .and_then(Page::new)
        .expect("invalid phys2virt in BootInfo")
}

/// Для простоты работы с ней, физическая память целиком отображена в некоторую область виртуальной.
/// Эта функция по адресу первой (виртуальной) страницы этой области `phys2virt`
/// выдаёт соответствующий виртуальный адрес для заданного физического `address`.
pub(crate) fn map(
    phys2virt: Page,
    address: Phys,
) -> Virt {
    (phys2virt.address() + address.into_usize()).expect("bad Phys address for phys2virt::map()")
}
