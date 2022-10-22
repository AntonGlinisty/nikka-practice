use alloc::vec::Vec;
use core::{
    arch::asm,
    mem,
};

use memoffset::offset_of;
use x86_64::{
    PrivilegeLevel,
    VirtAddr,
    instructions::tables,
    registers::model_specific::FsBase,
    structures::tss::TaskStateSegment,
};

use crate::{
    error::{
        Error::NoProcess,
        Result,
    },
    log::{
        debug,
        error,
    },
    memory::{
        BASE_ADDRESS_SPACE,
        DOUBLE_FAULT_IST_INDEX,
        EXCEPTION_STACKS,
        GDT,
        Gdt,
        KERNEL_RW,
        PAGE_FAULT_IST_INDEX,
        Stack,
        Virt,
    },
    process::{
        ModeContext,
        Pid,
    },
};

use super::{
    CpuId,
    LocalApic,
};

pub(super) fn init(
    cpu_count: usize,
    current_cpu: CpuId,
) -> Result<Vec<Cpu>> {
    let mut cpus = init_cpu_vec(cpu_count)?;

    let cpu = &mut cpus[usize::from(current_cpu)];
    cpu.set_fs();
    cpu.set_tss();

    Ok(cpus)
}

/// Выделяет память для структур [`Cpu`] и для содержащихся в них стеках.
fn init_cpu_vec(cpu_count: usize) -> Result<Vec<Cpu>> {
    // TODO: your code here.
    unimplemented!();
}

pub(crate) const KERNEL_RSP_OFFSET_IN_CPU: usize = offset_of!(Cpu, tss) +
    offset_of!(TaskStateSegment, privilege_stack_table) +
    PrivilegeLevel::Ring0 as usize * mem::size_of::<VirtAddr>();

/// CPU-local storage.
/// Aligned on the cache line size to avoid false sharing.
/// [Why align on 128 bytes instead of 64?](https://docs.rs/crossbeam/latest/crossbeam/utils/struct.CachePadded.html#size-and-alignment)
#[repr(align(128))]
pub(crate) struct Cpu {
    /// Исполняющийся на данном CPU в текущий момент пользовательский процесс.
    ///
    /// Пользовательские процессы выполняются независимо на разных ядрах. Следовательно, у каждого ядра текущий процесс должен быть своим.
    current_process: Option<Pid>,

    /// Идентификатор данного CPU, содержит копию идентификатора его Local APIC --- [`LocalApic::id()`].
    id: CpuId,

    /// Стек ядра данного CPU.
    ///
    /// Процессоры обрабатывают системные вызовы и прерывания независимо, так что у каждого должен быть свой стек для этого.
    kernel_stack: &'static Stack,

    /// Дополнительный стек, который используется во время обработки Page Fault.
    ///
    /// Он позволяет напечатать диагностику возникшего исключения даже в случае, когда оно вызвано исчерпанием основного стека ядра.
    page_fault_stack: &'static Stack,

    /// Адрес структуры [`Cpu`] для данного CPU.
    this: Virt,

    /// Task State Segment (TSS) для данного CPU.
    ///
    /// В TSS описано, где находится стек ядра для текущего процессора.
    /// Стеки у разных CPU разные, поэтому и TSS тоже должны быть разные.
    tss: TaskStateSegment,

    /// Временное хранилище для контекста принудительно вытесненного с этого CPU процесса.
    user_context: Option<ModeContext>,
}

impl Cpu {
    pub fn new(
        id: CpuId,
        stacks: &'static [Stack],
    ) -> Self {
        assert_eq!(stacks.len(), Self::STACKS_PER_CPU);

        let mut result = Self {
            id,
            current_process: None,
            kernel_stack: &stacks[0],
            page_fault_stack: &stacks[1],
            this: Virt::default(),
            tss: TaskStateSegment::new(),
            user_context: None,
        };

        // A double fault means there is an error in the kernel.
        // So a common double fault stack for all CPUs is sufficient as long as
        // double_fault() handler in trap/mod.rs locks and never unlocks a common mutex and does not return.
        result.tss.interrupt_stack_table[usize::from(DOUBLE_FAULT_IST_INDEX)] =
            EXCEPTION_STACKS.lock().double_fault_rsp().into();
        result.tss.interrupt_stack_table[usize::from(PAGE_FAULT_IST_INDEX)] =
            result.page_fault_stack.pointer().into();

        result
    }

    pub(crate) fn current_process() -> Result<Pid> {
        if super::test_scaffolding::is_initialized() {
            let cpu = unsafe { Self::get() };
            cpu.current_process.ok_or(NoProcess)
        } else {
            Ok(Pid::Current)
        }
    }

    pub(crate) fn set_current_process(process: Option<Pid>) {
        if super::test_scaffolding::is_initialized() {
            let cpu = unsafe { Self::get() };
            cpu.current_process = process;
        }
    }

    pub(crate) fn kernel_context() -> ModeContext {
        let cpu = unsafe { Self::get() };
        ModeContext::kernel_context(
            cpu.tss.privilege_stack_table[PrivilegeLevel::Ring0 as usize].into(),
        )
    }

    pub(crate) fn set_user_context(user_context: ModeContext) {
        let cpu = unsafe { Self::get() };
        cpu.user_context = Some(user_context);
    }

    pub(crate) fn take_user_context() -> Option<ModeContext> {
        if super::test_scaffolding::is_initialized() {
            let cpu = unsafe { Self::get() };
            cpu.user_context.take()
        } else {
            None
        }
    }

    pub(super) fn id(&self) -> CpuId {
        self.id
    }

    pub(super) fn kernel_stack(&self) -> &'static Stack {
        self.kernel_stack
    }

    /// Инициализация регистра `FS` текущего CPU.
    pub(super) fn set_fs(&mut self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Инициализация TSS и регистра TR текущего CPU.
    pub(super) fn set_tss(&self) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает структуру [`Cpu`] текущего процессора.
    unsafe fn get() -> &'static mut Cpu {
        // TODO: your code here.
        unimplemented!();
    }

    const STACKS_PER_CPU: usize = 2;
}

impl Drop for Cpu {
    fn drop(&mut self) {
        panic!("a Cpu has been dropped");
    }
}
