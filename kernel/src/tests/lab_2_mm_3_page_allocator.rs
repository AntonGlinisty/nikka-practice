use ku::memory::size::{
    Size,
    TiB,
};

use kernel::{
    Subsystems,
    log::debug,
    memory::{
        BASE_ADDRESS_SPACE,
        Page,
        Virt,
        test_scaffolding::{
            forbid_frame_leaks,
            page_allocator_allocate,
            page_allocator_block,
        },
    },
};

mod init;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn sanity_check() {
    let _guard = forbid_frame_leaks();

    let page_allocator_block = page_allocator_block(&BASE_ADDRESS_SPACE.lock());

    debug!(?page_allocator_block);

    assert!(page_allocator_block.size() > 100 * TiB);
    assert!(page_allocator_block.size() < 128 * TiB);
}

#[test_case]
fn allocate_block() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    for size in (1 ..= 3 * Page::SIZE).filter(|x| (x + 3) % Page::SIZE < 6) {
        let free_page_count = page_allocator_block(&address_space).count();
        let allocated = page_allocator_allocate(&mut address_space, size).unwrap();
        let allocated_page_count = free_page_count - page_allocator_block(&address_space).count();

        debug!(requested = %Size::new::<Virt>(size), %allocated, allocated_page_count);

        assert!(
            allocated.size() - size < Page::SIZE,
            "allocated excessive memory",
        );
        assert_eq!(allocated.count(), allocated_page_count, "lost some pages");
    }
}

#[test_case]
fn allocate_page() {
    let _guard = forbid_frame_leaks();

    let page = BASE_ADDRESS_SPACE
        .lock()
        .allocate(Page::SIZE)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    debug!(?page);

    assert_ne!(page.index(), 0);
}

#[test_case]
fn allocate_two_pages() {
    let _guard = forbid_frame_leaks();

    let mut address_space = BASE_ADDRESS_SPACE.lock();

    let pages = [
        address_space.allocate(Page::SIZE).unwrap().into_iter().next().unwrap(),
        address_space.allocate(Page::SIZE).unwrap().into_iter().next().unwrap(),
    ];

    debug!(?pages);

    assert_ne!(pages[0], pages[1]);
}
