use ku::{
    error::Error::{
        InvalidAlignment,
        InvalidArgument,
        Overflow,
        PermissionDenied,
    },
    log::debug,
    memory::{
        Block,
        Page,
        Virt,
        mmu::{
            KERNEL_R,
            KERNEL_RW,
            PageTableFlags,
            USER_R,
            USER_RW,
        },
    },
    process::{
        Pid,
        SyscallResult,
    },
    sync::spinlock::Spinlock,
};

use kernel::{
    Subsystems,
    memory::test_scaffolding::{
        forbid_frame_leaks,
        get_pte,
        mapping,
        switch_to,
    },
    process::{
        Process,
        Scheduler,
        test_scaffolding::{
            copy_mapping,
            map,
            set_pid,
            unmap,
        },
    },
    trap::{
        TRAP_STATS,
        Trap,
    },
};

mod init;
mod process_helpers;

init!(Subsystems::MEMORY | Subsystems::SMP | Subsystems::PROCESS);

const MEMORY_ALLOCATOR_ELF: &[u8] = page_aligned!("../../target/kernel/user/memory_syscalls");

#[test_case]
fn map_syscall_group() {
    let _guard = forbid_frame_leaks();

    let process = Spinlock::new(process_helpers::make(MEMORY_ALLOCATOR_ELF));
    set_pid(&mut process.lock(), Pid::new(0));
    let pid = process.lock().pid().into_usize();
    switch_to(process.lock().address_space());

    let block = Block::from_slice("some kernel memory".as_bytes()).enclosing();
    map_over_kernel(&process, block);

    let block = process.lock().address_space().allocate(2 * Page::SIZE).unwrap();
    let mut head = block;
    let tail = head.tail(1).unwrap();
    unsafe {
        process.lock().address_space().map_block(tail, KERNEL_RW).unwrap();
    }
    map_over_kernel(&process, block);
    unsafe {
        process.lock().address_space().unmap_block(tail).unwrap();
    }

    let block = process.lock().address_space().allocate(block.size()).unwrap();
    let address = block.start_address().into_usize();
    let size = block.size();
    let new_block = process.lock().address_space().allocate(block.size()).unwrap();
    let new_address = new_block.start_address().into_usize();

    for flags in [KERNEL_R, KERNEL_RW] {
        let flags = flags.bits();
        assert_eq!(
            map(process.lock(), pid, address, size, flags),
            Err(PermissionDenied),
        );
    }

    let additional_flag_sets = [
        PageTableFlags::empty(),
        PageTableFlags::ACCESSED,
        PageTableFlags::DIRTY,
        PageTableFlags::ACCESSED | PageTableFlags::DIRTY,
    ];

    for flags in [USER_R, USER_RW] {
        for additional_flags in additional_flag_sets {
            let flags = flags | additional_flags;

            assert_eq!(
                map(process.lock(), pid, address, size, flags.bits()),
                Ok(SyscallResult(address)),
            );

            validate_flags(&process, Virt::new(address).unwrap(), flags);

            let allocated_memory = unsafe { block.try_into_slice::<usize>().unwrap() };
            assert!(
                allocated_memory.iter().all(|&x| x == 0),
                "do not leak information into the user space in allocated frames",
            );
        }

        for flags in [KERNEL_R, KERNEL_RW] {
            let flags = flags.bits();
            assert_eq!(
                copy_mapping(process.lock(), pid, address, new_address, size, flags),
                Err(PermissionDenied),
            );
        }

        for additional_flags in additional_flag_sets {
            let flags = flags | additional_flags;

            assert!(
                copy_mapping(
                    process.lock(),
                    pid,
                    address,
                    new_address,
                    size,
                    flags.bits(),
                )
                .is_ok(),
            );

            validate_flags(&process, Virt::new(new_address).unwrap(), flags);
        }
    }

    let flags = USER_RW.bits();
    assert!(unmap(process.lock(), pid, address, size).is_ok());
    assert!(unmap(process.lock(), pid, new_address, size).is_ok());
    assert_eq!(
        copy_mapping(process.lock(), pid, address, new_address, size, flags),
        Err(PermissionDenied),
    );
    assert!(unmap(process.lock(), pid, address, size).is_err());

    assert!(map(process.lock(), pid, 0, size, flags).is_ok());
    assert_eq!(
        map(process.lock(), pid, 1, size, flags),
        Err(InvalidAlignment),
    );
    for address in [0, address] {
        assert_eq!(
            map(process.lock(), pid, address, 0, flags),
            Err(InvalidArgument),
        );
        for size in [1, size + 1] {
            assert_eq!(
                map(process.lock(), pid, address, size, flags),
                Err(InvalidAlignment),
            );
        }
    }

    for (address, size) in [
        (0x1_0000, 0xFFFF_FFFF_0000_0000),
        (0xFFFF_FFFF_FFFF_0000, 0x10_0000),
    ] {
        let result = map(process.lock(), pid, address, size, flags);
        assert!(
            result == Err(InvalidArgument) || result == Err(Overflow),
            "expected Err(InvalidArgument) or Err(Overflow), got {result:?}",
        );
    }

    fn validate_flags(
        process: &Spinlock<Process>,
        virt: Virt,
        expected_flags: PageTableFlags,
    ) {
        let mut process = process.lock();
        let mut address_space = process.address_space();
        let mapping = mapping(&mut address_space);
        let path = mapping.path(virt);
        let actual_flags = get_pte(&path).unwrap().flags();
        debug!(%actual_flags, %expected_flags);
        assert_eq!(actual_flags, expected_flags);
    }
}

fn map_over_kernel(
    process: &Spinlock<Process>,
    block: Block<Page>,
) {
    let pid = process.lock().pid().into_usize();

    let address = block.start_address().into_usize();
    let size = block.size();
    let new_block = process.lock().address_space().allocate(block.size()).unwrap();
    let new_address = new_block.start_address().into_usize();

    assert_eq!(
        unmap(process.lock(), pid, address, size),
        Err(PermissionDenied),
    );

    for flags in [KERNEL_R, KERNEL_RW, USER_R, USER_RW] {
        let flags = flags.bits();

        assert_eq!(
            map(process.lock(), pid, address, size, flags),
            Err(PermissionDenied),
        );
        assert_eq!(
            copy_mapping(process.lock(), pid, address, new_address, size, flags),
            Err(PermissionDenied),
        );
    }
}

#[test_case]
fn user_space_memory_allocator() {
    let _guard = forbid_frame_leaks();

    Scheduler::enqueue(process_helpers::allocate(MEMORY_ALLOCATOR_ELF).pid());

    while Scheduler::run_one() {}

    assert_eq!(
        TRAP_STATS[Trap::PageFault].count(),
        0,
        "the user mode code has detected an error in the memory allocator implementation",
    );
}
