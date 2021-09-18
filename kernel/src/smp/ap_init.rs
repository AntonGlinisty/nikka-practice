use core::{
    arch,
    convert::TryInto,
    fmt,
    hint,
    mem,
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
};

use memoffset::offset_of;
use x86::msr::IA32_EFER;
use x86_64::{
    instructions::interrupts,
    registers::{
        control::{
            Cr0,
            Cr4,
        },
        model_specific::Efer,
    },
    structures::gdt::SegmentSelector,
};

use crate::{
    error::Result,
    log::{
        debug,
        info,
    },
    memory::{
        BASE_ADDRESS_SPACE,
        Frame,
        GDT,
        Page,
        Phys,
        RealModePseudoDescriptor,
        SmallGdt,
        Virt,
        phys2virt,
        size,
    },
    time,
    trap::IDT,
};

use super::{
    CpuId,
    LocalApic,
    cpu::Cpu,
};

/// MultiProcessor Specification 4.1 part B.4 "Application Processor Startup"
pub(super) fn boot_ap(
    phys2virt: Page,
    id: CpuId,
    cpu: &mut Cpu,
) -> Result<()> {
    let timer = time::timer();
    let initialized = AtomicBool::new(false);

    let boot_code = prepare_boot_code(phys2virt, cpu, &initialized)?;

    LocalApic::send_init(id, boot_code)?;
    wait_initialized(&initialized);

    info!(cpu = id, duration = %timer.elapsed(), "boot Application Processor");

    Ok(())
}

fn prepare_boot_code(
    phys2virt: Page,
    cpu: &mut Cpu,
    initialized: &AtomicBool,
) -> Result<Frame> {
    let boot_code = real_mode_address(BOOT_CODE, "boot code");

    copy_switch_mode_code(phys2virt::map(phys2virt, boot_code))?;

    let boot_stack_phys = real_mode_address(BOOT_STACK, "boot stack");
    let boot_stack = BootStack::new(boot_stack_phys, cpu, initialized)?;

    debug!(?boot_stack);

    unsafe {
        phys2virt::map(phys2virt, boot_stack_phys)
            .try_into_mut_ptr::<BootStack>()
            .expect("BOOT_STACK address is not suitable for BootStack type")
            .write_volatile(boot_stack);
        arch::x86_64::_mm_mfence();
    }

    Ok(Frame::new(boot_code))
}

const BOOT_CODE: usize = 7 * Page::SIZE;
const BOOT_STACK: usize = BOOT_CODE + BOOT_CODE_PLUS_STACK_SIZE - mem::size_of::<BootStack>();
const BOOT_CODE_PLUS_STACK_SIZE: usize = Page::SIZE;

#[repr(C)]
struct BootStack {
    /// <https://wiki.osdev.org/CPU_Registers_x86-64#IA32_EFER>
    efer: u32,
    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR4>
    cr4: u32,
    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR3>
    cr3: u32,
    /// <https://wiki.osdev.org/CPU_Registers_x86-64#CR0>
    cr0: u32,
    gdt_pseudo_descriptor: RealModePseudoDescriptor,
    kernel_data: SegmentSelector,
    /// `kernel_code:set_cs_rip_to_64bit` is a 16 bit far pointer for a far return `retf` instruction.
    set_cs_rip_to_64bit: u16,
    kernel_code: SegmentSelector,
    _padding: u32,
    cpu: Virt,
    initialized: Virt,
    kernel_stack: Virt,
    gdt: SmallGdt,
}

impl BootStack {
    fn new(
        boot_stack_phys: Phys,
        cpu: &mut Cpu,
        initialized: &AtomicBool,
    ) -> Result<Self> {
        let switch_mode_start: usize;
        let set_cs_rip_to_64bit: usize;
        unsafe {
            asm!(
                "
                mov {switch_mode_start}, OFFSET switch_mode_start
                mov {set_cs_rip_to_64bit}, OFFSET set_cs_rip_to_64bit
                ",
                switch_mode_start = out(reg) switch_mode_start,
                set_cs_rip_to_64bit = out(reg) set_cs_rip_to_64bit,
            )
        }

        let efer = Efer::read().bits().try_into()?;
        let cr4 = Cr4::read_raw().try_into()?;
        let cr3 = BASE_ADDRESS_SPACE.lock().page_table_root().address().try_into_u32()?;
        let cr0 = Cr0::read_raw().try_into()?;
        let gdt = SmallGdt::new();
        let gdt_pseudo_descriptor =
            SmallGdt::real_mode_pseudo_descriptor((boot_stack_phys + offset_of!(BootStack, gdt))?)?;
        let set_cs_rip_to_64bit: u16 =
            size::into_u64(BOOT_CODE + (set_cs_rip_to_64bit - switch_mode_start))
                .try_into()
                .expect(
                    "set_cs_rip_to_64bit label address should fit into 16 bit for Application \
                     Processors to be able to ret to it",
                );

        let kernel_stack = cpu.kernel_stack().pointer();

        Ok(Self {
            efer,
            cr4,
            cr3,
            cr0,
            gdt_pseudo_descriptor,
            kernel_data: SmallGdt::kernel_data(),
            set_cs_rip_to_64bit,
            kernel_code: SmallGdt::kernel_code(),
            _padding: 0,
            cpu: Virt::from_ref(cpu),
            initialized: Virt::from_ref(initialized),
            kernel_stack,
            gdt,
        })
    }
}

impl fmt::Debug for BootStack {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            concat!(
                "{{ ",
                "efer: {:#X}, cr4: {:#X}, cr3: {:#X}, cr0: {:#X}, ",
                "gdt: {:?}, kernel_code: {:?}, kernel_data: {:?}, set_cs_rip_to_64bit: 0p{:04X}, ",
                "kernel_stack: {}, cpu: {}, initialized: {} ",
                "}}",
            ),
            self.efer,
            self.cr4,
            self.cr3,
            self.cr0,
            self.gdt_pseudo_descriptor,
            self.kernel_code,
            self.kernel_data,
            self.set_cs_rip_to_64bit,
            self.kernel_stack,
            self.cpu,
            self.initialized,
        )
    }
}

#[allow(named_asm_labels)]
#[cold]
#[naked]
extern "C" fn switch_from_real_mode_to_long_mode() -> ! {
    unsafe {
        asm!(
            "
        .code16

        switch_mode_start:

            // TODO: your code here.

        .code64

            // Perform a far return to `set_cs_rip_to_64bit` in order to set `CS:RIP` right.
            retf

        set_cs_rip_to_64bit:

            // TODO: your code here.

            // The destination address of this jump should be absolute
            // since this code is moved to another position in the memory.
            mov rax, OFFSET {ap_kernel_main}
            jmp rax

        switch_mode_end:
            ",

            // TODO: your code here.
            ap_kernel_main = sym ap_kernel_main,

            options(noreturn),
        );
    }
}

#[cold]
#[inline(never)]
extern "C" fn ap_kernel_main(
    cpu: &mut Cpu,            // rdi
    initialized: &AtomicBool, // rsi
) -> ! {
    GDT.lock().load();

    LocalApic::init();

    cpu.set_fs();
    cpu.set_tss();

    IDT.load();
    interrupts::enable();

    signal_initialized(initialized);

    info!(cpu = LocalApic::id(), "report for duty");

    crate::halt();
}

fn copy_switch_mode_code(boot_code_address: Virt) -> Result<()> {
    let switch_mode_code_address = Virt::from_ptr(switch_from_real_mode_to_long_mode as *const ());
    let switch_mode_code_slice =
        unsafe { switch_mode_code_address.try_into_mut_slice::<u8>(switch_mode_code_size())? };
    let boot_code_slice =
        unsafe { boot_code_address.try_into_mut_slice::<u8>(switch_mode_code_slice.len())? };

    boot_code_slice.clone_from_slice(switch_mode_code_slice);

    Ok(())
}

fn switch_mode_code_size() -> usize {
    let switch_mode_start: usize;
    let switch_mode_end: usize;
    unsafe {
        asm!(
            "
            mov {switch_mode_start}, OFFSET switch_mode_start
            mov {switch_mode_end}, OFFSET switch_mode_end
            ",
            switch_mode_start = out(reg) switch_mode_start,
            switch_mode_end = out(reg) switch_mode_end,
        )
    }

    let switch_mode_code_size = switch_mode_end - switch_mode_start;
    assert!(switch_mode_code_size + mem::size_of::<BootStack>() <= BOOT_CODE_PLUS_STACK_SIZE);

    switch_mode_code_size
}

fn real_mode_address(
    address: usize,
    what: &'static str,
) -> Phys {
    let address: u32 = address.try_into().unwrap_or_else(|_| {
        panic!(
            "{} address {:#X} should fit into 32 bit for Application Processors to be able to \
             boot from it",
            what, address,
        )
    });
    Phys::new_u64(address.into()).expect("any u32 value should be a valid physical address")
}

fn signal_initialized(initialized: &AtomicBool) {
    initialized
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .expect("double or concurrent CPU initialization");
}

fn wait_initialized(initialized: &AtomicBool) {
    while !initialized.load(Ordering::Acquire) {
        hint::spin_loop();
    }
}
