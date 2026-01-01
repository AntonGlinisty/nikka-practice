use core::{
    fmt,
    mem,
};

use memoffset::offset_of;
use x86_64::registers::control::Cr2;

use crate::{
    error::Result,
    memory::{
        PageFaultInfo,
        Virt,
    },
    process::MiniContext,
};

use crate::process::RSP_OFFSET_IN_MINI_CONTEXT;

use super::Trap;

// ANCHOR: trap_info
/// Информация об исключении процессора.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapInfo {
    /// Контекст, в котором возникло исключение.
    context: MiniContext,

    /// Информация об исключении, предоставляемая процессором.
    info: Info,

    /// Номер исключения.
    number: usize,

    #[allow(rustdoc::private_intra_doc_links)]
    /// [`TrapInfo`] может быть сохранено в тот же стек,
    /// на который указывает [`field@TrapInfo::context`].
    /// Это происходит при рекурсивном исключении ---
    /// то есть когда исключение происходит внутри обработчика исключений.
    /// В этом случае функции `lib::syscall::trap_trampoline` и
    /// `lib::syscall::trap_handler_invoker` сохранят адрес возврата на тот же стек
    /// [`field@TrapInfo::context`].
    /// Что приведёт к перезаписи содержимого [`TrapInfo`].
    /// Поле [`TrapInfo::return_address_placeholder`] служит для избежания перезаписи
    /// существенных полей --- оно имеет тот же размер и расположение, как и адрес возврата.
    return_address_placeholder: [u8; Self::PLACEHOLDER_SIZE],
}
// ANCHOR_END: trap_info

impl TrapInfo {
    /// Размер адреса возврата из функции, см. [`TrapInfo::return_address_placeholder`].
    const PLACEHOLDER_SIZE: usize = mem::size_of::<Virt>();

    /// Создаёт информацию об исключении.
    ///
    /// - number --- номер исключения,
    /// - info --- информация об исключении, предоставляемая процессором.
    /// - context --- контекст, в котором возникло исключение.
    pub fn new(
        number: usize,
        info: Info,
        context: MiniContext,
    ) -> Self {
        Self {
            context,
            info,
            number,
            return_address_placeholder: [0; Self::PLACEHOLDER_SIZE],
        }
    }

    /// Контекст, в котором возникло исключение.
    pub fn context(&self) -> MiniContext {
        self.context
    }

    /// Информация об исключении, предоставляемая процессором.
    pub fn info(&self) -> Info {
        self.info
    }

    /// Номер исключения.
    pub fn number(&self) -> usize {
        self.number
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Записывает на стек контекста исключения адрес возникновения исключения.
    /// После этого, если переключить стек в изменившийся [`field@TrapInfo::context`],
    /// и выполнить инструкцию `ret`,
    /// то регистры `rip` и `rsp` окажутся в состоянии, равном исходному [`field@TrapInfo::context`].
    ///
    /// # Safety
    ///
    /// - Контекст исключения, сохранённый в [`field@TrapInfo::context`] должен быть корректен.
    /// - В стеке контекста исключения должно быть достаточно места для адреса возврата ---
    ///   [`Virt`].
    pub unsafe fn prepare_for_ret(&mut self) -> Result<()> {
        unsafe {
            *self.context.push::<Virt>()?.try_into_mut()? = self.context.rip();
        }
        Ok(())
    }
}

impl fmt::Display for TrapInfo {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ #{}, {}, {} }}",
            self.number, self.info, self.context
        )
    }
}

/// Информация об исключении, предоставляемая процессором.
#[derive(Clone, Copy, Debug)]
pub enum Info {
    /// Исключение не имеет дополнительной информации.
    None,

    /// Код ошибки, сохраняемый процессором на стеке для некоторых исключений.
    Code(usize),

    /// Информация об исключении обращения к странице виртуальной памяти.
    PageFault {
        /// Адрес к которому происходило обращение.
        address: Virt,

        /// Причина некорректности обращения.
        code: PageFaultInfo,
    },
}

impl Info {
    /// Информация об исключении `trap`, предоставляемая процессором в виде кода `error_code`.
    pub fn new(
        trap: Trap,
        error_code: usize,
    ) -> Self {
        match trap {
            Trap::AlignmentCheck |
            Trap::DoubleFault |
            Trap::GeneralProtectionFault |
            Trap::InvalidTss |
            Trap::SecurityException |
            Trap::SegmentNotPresent |
            Trap::StackSegmentFault => Info::Code(error_code),

            Trap::PageFault => Info::PageFault {
                address: Cr2::read().expect("read bad value from %cr2").into(),
                code: PageFaultInfo::from_bits_truncate(error_code),
            },

            _ => Info::None,
        }
    }
}

impl fmt::Display for Info {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Info::None => write!(formatter, "{{ }}"),
            Info::Code(code) => write!(formatter, "{{ code: {code} }}"),
            Info::PageFault { address, code } => {
                write!(formatter, "{{ address: {address}, code: {code} }}")
            },
        }
    }
}

/// Смещение поля для регистра `rsp` контекста исключения в структуре [`TrapInfo`].
/// Позволяет обращаться к этому полю из ассемблерных вставок.
pub const RSP_OFFSET_IN_TRAP_INFO: usize =
    offset_of!(TrapInfo, context) + RSP_OFFSET_IN_MINI_CONTEXT;
