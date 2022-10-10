use core::mem;

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        Block,
        KERNEL_RW,
    },
};

mod init;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn map_slice() {
    let len = 16 * MiB / mem::size_of::<usize>();
    let slice = BASE_ADDRESS_SPACE.lock().map_slice(len, KERNEL_RW, usize::default).unwrap();

    debug!(slice = ?Block::from_slice(slice));

    for (i, element) in slice.iter_mut().enumerate() {
        *element = i;
    }

    for (i, element) in slice.iter().enumerate() {
        assert_eq!(*element, i);
    }
}
