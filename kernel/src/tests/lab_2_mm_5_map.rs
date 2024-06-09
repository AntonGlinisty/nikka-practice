use core::{
    cmp,
    mem,
};

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    error::Error::PermissionDenied,
    log::debug,
    memory::{
        AddressSpace,
        BASE_ADDRESS_SPACE,
        Block,
        FRAME_ALLOCATOR,
        Frame,
        KERNEL_R,
        KERNEL_RW,
        Page,
        USER_R,
        Virt,
        mmu::{
            PageTableEntry,
            PageTableFlags,
        },
        test_scaffolding::{
            forbid_frame_leaks,
            map_page,
            mapping,
        },
    },
};

mod init;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn map_slice() {
    let start_free_frames = FRAME_ALLOCATOR.lock().count();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let frame_count = init::frame_count();
    let len = cmp::min(frame_count * Frame::SIZE / 2, 16 * MiB) / mem::size_of::<usize>();
    let slice = address_space.map_slice(len, KERNEL_RW, usize::default).unwrap();
    let slice_frames = mem::size_of_val(slice).div_ceil(Frame::SIZE);

    let free_frames = FRAME_ALLOCATOR.lock().count();
    assert!(free_frames + slice_frames <= start_free_frames);

    debug!(slice = ?Block::from_slice(slice));
    debug!(slice_frames, frame_count);

    for (i, element) in slice.iter_mut().enumerate() {
        *element = i;
    }

    for (i, element) in slice.iter().enumerate() {
        assert_eq!(*element, i);
    }

    unsafe {
        address_space.unmap_slice(slice).unwrap();
    }

    let end_free_frames = FRAME_ALLOCATOR.lock().count();
    assert!(free_frames + slice_frames <= end_free_frames);
}

#[test_case]
fn map_readable() {
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let page = Page::containing(Virt::new(0x0000_6000_0000_0000).unwrap());
    unsafe { map_page(&mut address_space, page, KERNEL_R).unwrap() };
    let frame = translate(&mut address_space, page);
    debug!(%page, %frame);

    unsafe {
        Block::from_element(page).unwrap().try_into_slice::<u8>().unwrap().iter().max();
    }
}

#[test_case]
fn map_writable() {
    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let page = Page::containing(Virt::new(0x0000_6000_1000_0000).unwrap());
    unsafe { map_page(&mut address_space, page, PageTableFlags::WRITABLE).unwrap() };
    let frame = translate(&mut address_space, page);
    debug!(%page, %frame);

    unsafe {
        Block::from_element(page).unwrap().try_into_mut_slice::<u8>().unwrap().fill(12);
    }
}

#[test_case]
fn map_twice() {
    let virt = Virt::new(0x0000_4000_0000_0000).unwrap();
    let page = Page::containing(virt);

    unsafe {
        map_page(
            &mut BASE_ADDRESS_SPACE.lock(),
            page,
            PageTableFlags::empty(),
        )
        .unwrap()
    };
    let old_frame = translate(page.address()).frame();

    let pte = translate(virt);
    assert_eq!(KERNEL_R, pte.flags());

    let _guard = forbid_frame_leaks();

    unsafe { map_page(&mut BASE_ADDRESS_SPACE.lock(), page, KERNEL_RW).unwrap() };
    let new_frame = translate(page.address()).frame();

    let pte = translate(virt);
    assert_eq!(KERNEL_RW, pte.flags());

    let hacked_frame = unsafe { map_page(&mut BASE_ADDRESS_SPACE.lock(), page, USER_R) };
    assert_eq!(hacked_frame, Err(PermissionDenied));

    debug!(?old_frame, ?new_frame);
    assert_ne!(old_frame, new_frame);

    fn translate(virt: Virt) -> PageTableEntry {
        let mut address_space = BASE_ADDRESS_SPACE.lock();
        *mapping(&mut address_space).translate(virt).unwrap()
    }
}

fn translate(
    address_space: &mut AddressSpace,
    page: Page,
) -> Frame {
    mapping(address_space).translate(page.address()).unwrap().frame().unwrap()
}
