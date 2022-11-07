use kernel::{
    Subsystems,
    log::info,
    memory::{
        BASE_ADDRESS_SPACE,
        Phys,
        mmu::PageTableFlags,
        test_scaffolding::mapping,
    },
    smp::test_scaffolding::{
        id,
        local_apic,
    },
};

mod init;

init!(Subsystems::MEMORY | Subsystems::SYSCALL | Subsystems::LOCAL_APIC);

#[test_case]
fn mapped_properly() {
    let local_apic = local_apic();
    let expected_local_apic_address = Phys::new(0xFEE00000).unwrap();

    let mut address_space = BASE_ADDRESS_SPACE.lock();
    let mapping = mapping(&mut address_space);
    let mapping_error = "Local APIC is not mapped";
    let pte = mapping
        .translate(local_apic, None, PageTableFlags::empty())
        .expect(mapping_error);
    let frame = pte.frame().expect(mapping_error);
    let flags = pte.flags();

    info!(%local_apic, ?frame, ?flags, "Local APIC");

    assert!(pte.is_present(), "{}", mapping_error);
    assert_eq!(
        frame.address(),
        expected_local_apic_address,
        "wrong physical address for the Local APIC",
    );
    assert!(
        flags.contains(PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH),
        "wrong flags for the Local APIC virtual page",
    );

    info!(cpu = id());
}
