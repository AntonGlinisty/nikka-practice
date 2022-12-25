use core::{
    fmt,
    mem,
};

use memoffset::offset_of;

use crate::{
    error::Result,
    memory::{
        PageFaultInfo,
        Virt,
    },
};

use super::{
    MiniContext,
    mini_context::RSP_OFFSET_IN_MINI_CONTEXT,
};

/// Информация об исключении процессора.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapInfo {
    /// Номер исключения.
    number: usize,

    /// Информация об исключении, предоставляемая процессором.
    info: Info,

    /// Контекст, в котором возникло исключение.
    context: MiniContext,

    #[allow(rustdoc::private_intra_doc_links)]
    /// [`TrapInfo`] может быть сохранено в тот же стек,
    /// на который указывает [`field@TrapInfo::context`].
    /// Это происходит при рекурсивном исключении ---
    /// то есть когда исключение происходит внутри обработчика исключений.
    /// В этом случае функции `lib::syscall::trap_trampoline` и `lib::syscall::trap_handler_invoker`
    /// сохранят адрес возврата на тот же стек [`field@TrapInfo::context`].
    /// Что приведёт к перезаписи содержимого [`TrapInfo`].
    /// Поле [`TrapInfo::return_address_placeholder`] служит для избежания перезаписи
    /// существенных полей --- оно имеет тот же размер и расположение, как и адрес возврата.
    return_address_placeholder: [u8; Self::PLACEHOLDER_SIZE],
}

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
            number,
            info,
            context,
            return_address_placeholder: [0; Self::PLACEHOLDER_SIZE],
        }
    }

    /// Номер исключения.
    pub fn number(&self) -> usize {
        self.number
    }

    /// Информация об исключении, предоставляемая процессором.
    pub fn info(&self) -> Info {
        self.info
    }

    /// Контекст, в котором возникло исключение.
    pub fn context(&self) -> MiniContext {
        self.context
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Записывает на стек контекста исключения адрес возникновения исключения.
    /// После этого, если переключить стек в изменившийся [`field@TrapInfo::context`],
    /// и выполнить инструкцию `ret`,
    /// то регистры `rip` и `rsp` окажутся в состоянии, равном исходному [`field@TrapInfo::context`].
    pub unsafe fn prepare_for_ret(&mut self) -> Result<()> {
        *self.context.push::<Virt>()?.try_into_mut()? = self.context.rip();
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
    Code(u64),

    /// Информация об исключении обращения к странице виртуальной памяти.
    PageFault {
        /// Адрес к которому происходило обращение.
        address: Virt,

        /// Причина некорректности обращения.
        code: PageFaultInfo,
    },
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
