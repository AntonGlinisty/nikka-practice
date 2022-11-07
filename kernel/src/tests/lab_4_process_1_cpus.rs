extern crate alloc;

use alloc::vec::Vec;
use core::slice;

use kernel::{
    Subsystems,
    log::info,
    memory::{
        BASE_ADDRESS_SPACE,
        Page,
        mmu::PageTableFlags,
        test_scaffolding::mapping,
    },
    smp::test_scaffolding::{
        cpu_count,
        cpu_id,
        id,
        kernel_stack_zones,
    },
};

mod init;

init!(Subsystems::MEMORY | Subsystems::SYSCALL | Subsystems::LOCAL_APIC | Subsystems::CPUS);

#[test_case]
fn initialized() {
    let cpu = cpu_id();
    let local_apic_id = id();
    info!(cpu, local_apic_id);

    assert_eq!(cpu, local_apic_id);
}

#[test_case]
fn kernel_stacks() {
    let mut stacks = Vec::with_capacity(cpu_count());

    for cpu in 0 .. cpu_count() {
        let (stack_guard, stack) = kernel_stack_zones(cpu);

        info!(cpu, %stack_guard, %stack);

        stacks.push(stack);

        assert_eq!(stack.start_address().into_usize() % Page::SIZE, 0);

        let mut address_space = BASE_ADDRESS_SPACE.lock();
        let mapping = mapping(&mut address_space);

        let stack_mapping_error = "kernel stack is not mapped";

        for stack_page in stack.enclosing() {
            let stack_pte = mapping
                .translate(stack_page.address(), None, PageTableFlags::empty())
                .expect(stack_mapping_error);
            let stack_frame = stack_pte.frame().expect(stack_mapping_error);
            let stack_flags = stack_pte.flags();

            if stack_page.address() == stack.start_address() {
                info!(cpu, %stack_page, ?stack_frame, ?stack_flags);
            }

            assert!(stack_pte.is_present(), "{stack_mapping_error}");
            assert_ne!(stack_page, Page::default(), "{stack_mapping_error}");
            assert_eq!(
                stack_flags & (PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE),
                PageTableFlags::WRITABLE,
                "wrong flags for the stack",
            );
        }

        if cpu != cpu_id().into() {
            unsafe {
                stack.try_into_mut_slice::<usize>().unwrap().fill(0);
            }
        }

        for stack_guard_page in stack_guard.enclosing() {
            let stack_guard_mapping_error = "kernel stack guard is not unmapped";
            let stack_guard_pte = mapping
                .translate(stack_guard_page.address(), None, PageTableFlags::empty())
                .expect(stack_mapping_error);
            let stack_guard_frame = stack_guard_pte.frame().expect_err(stack_guard_mapping_error);
            let stack_guard_flags = stack_guard_pte.flags();

            info!(cpu, %stack_guard_page, ?stack_guard_frame, ?stack_guard_flags);
            assert!(!stack_guard_pte.is_present(), "{stack_guard_mapping_error}");
        }
    }

    slice::heapsort(stacks.as_mut_slice(), |a, b| {
        a.start_address() < b.start_address()
    });

    for (stack_a, stack_b) in stacks.iter().zip(stacks.iter().skip(1)) {
        info!(%stack_a, %stack_b);
        assert!(stack_a.end_address().unwrap() < stack_b.start_address());
    }
}
