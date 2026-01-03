use bitflags::bitflags;
use chrono::Duration;
use num_enum::IntoPrimitive;
use x86::cpuid;
use x86_64::registers::model_specific::{
    ApicBase,
    ApicBaseFlags,
    Msr,
};

use ku::{
    log::{
        info,
        warn,
    },
    time,
};

use crate::{
    error::{
        Error::{
            self,
            InvalidArgument,
            Unimplemented,
        },
        Result,
    },
    log::error,
    memory::{
        Frame,
        size,
    },
    trap::Trap,
};

/// Идентификатор local APIC и текущего CPU.
pub(crate) type CpuId = u32;

/// Local
/// [APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
///
/// [16 Advanced Programmable Interrupt Controller (APIC)](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.998446).
pub(crate) struct LocalApic;

impl LocalApic {
    /// Инициализирует local APIC.
    ///
    /// - Переводит local APIC в
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    /// - Включает прерывание таймера.
    ///
    /// # Errors
    ///
    /// - [`Error::Unimplemented`] --- процессор не поддерживает
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    pub(crate) fn init() -> Result<()> {
        if let Err(error) = Self::enable_x2() {
            warn!(?error, "failed to switch the local APIC into x2APIC mode");

            Err(error)
        } else {
            /// Количество тиков процессора между прерываниями от таймера APIC.
            const TSCS_PER_INTERRUPT: u32 = 100_000_000;

            unsafe {
                Self::enable();
                Self::disable_lvts();
                Self::init_timer(TSCS_PER_INTERRUPT);
                Self::end_of_interrupt();
            }

            info!(cpu = Self::id(), "Local APIC init");

            Ok(())
        }
    }

    /// Возвращает `true` если local APIC инициализирован в режиме x2APIC.
    pub(crate) fn is_enabled() -> bool {
        ApicBase::read().1.contains(ApicBaseFlags::X2APIC_ENABLE)
    }

    /// Посылает local APIC сигнал о завершении работы обработчика прерывания ---
    /// [end of interrupt (EOI)](https://en.wikipedia.org/wiki/End_of_interrupt).
    ///
    /// - [16.6.4 Selecting and Handling Interrupts](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1003561).
    /// - <https://wiki.osdev.org/APIC#EOI_Register>
    ///
    /// # Safety
    ///
    /// Local APIC должен быть переведён в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    /// методом [`LocalApic::enable_x2()`].
    pub(crate) unsafe fn end_of_interrupt() {
        unsafe {
            Register::Eoi.write(0);
        }
    }

    /// Возвращает идентификатор local APIC и текущего CPU.
    /// Либо `0` если local APIC ещё не инициализирован или CPU не поддерживает
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    ///
    /// [16.3.3 Local APIC ID](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1006385>).
    pub(crate) fn id() -> CpuId {
        if Self::is_enabled() {
            unsafe { Register::Id.read() }
        } else {
            CpuId::MAX
        }
    }

    /// Посылает прерывание на Application Processor с идентификатором `dst`,
    /// предназначенное для его инициализации.
    /// Физический адрес процедуры инициализации задаёт `boot_address`.
    ///
    /// MultiProcessor Specification 4.1 part B.4 "Application Processor Startup"
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidArgument`] --- `boot_address` выходит за допустимую в
    ///   реальном режиме границу.
    /// - [`Error::LocalApic`] --- local APIC вернул ошибку.
    /// - [`Error::Unimplemented`] --- процессор не поддерживает
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    pub(super) fn send_init(
        dst: CpuId,
        boot_address: Frame,
    ) -> Result<()> {
        /// Максимальный номер физического фрейма, который может задавать адрес
        /// кода загрузки процессора.
        const MAX_BOOT_FRAME: u32 = 0xFF;

        let boot_page = size::try_into::<u32>(boot_address.index())?;
        if boot_page > MAX_BOOT_FRAME {
            return Err(InvalidArgument);
        }

        if !Self::is_enabled() {
            return Err(Unimplemented);
        }

        let init_data = InterruptCommand::DESTINATION_PHYSICAL |
            InterruptCommand::INIT |
            InterruptCommand::TRIGGER_MODE_LEVEL;

        unsafe {
            Self::send_ipi(dst, (init_data | InterruptCommand::LEVEL_ASSERT).bits())?;
        }
        time::delay(Duration::microseconds(200));

        unsafe {
            Self::send_ipi(dst, (init_data | InterruptCommand::LEVEL_DEASSERT).bits())?;
        }
        time::delay(Duration::microseconds(200));

        for _ in 0 .. 2 {
            unsafe {
                Self::send_ipi(dst, InterruptCommand::START_UP.bits() | boot_page)?;
            }
            time::delay(Duration::microseconds(200));
        }

        Ok(())
    }

    /// Посылает процессору `dst` прерывание
    /// ([inter-processor interrupt](https://en.wikipedia.org/wiki/Inter-processor_interrupt), IPI)
    /// с дополнительными данными `data`.
    ///
    /// [16.5 Interprocessor Interrupts (IPI)](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1001655).
    ///
    /// # Errors
    ///
    /// - [`Error::LocalApic`] --- local APIC вернул ошибку.
    /// - [`Error::Unimplemented`] --- процессор не поддерживает
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    ///
    /// # Safety
    ///
    /// Local APIC должен быть переведён в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    /// методом [`LocalApic::enable_x2()`].
    unsafe fn send_ipi(
        dst: CpuId,
        data: u32,
    ) -> Result<()> {
        /// Сдвиг для [`CpuId`] внутри
        /// [Interrupt Command Register](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1050595).
        const ID_SHIFT: u32 = u32::BITS;

        let mut interrupt_command = Msr::new(0x830);

        unsafe {
            interrupt_command.write((u64::from(dst) << ID_SHIFT) | u64::from(data));
        }

        let error = unsafe { Register::ErrorStatus.read() };
        if error == 0 {
            Ok(())
        } else {
            error!(dst, data, error, "error sending IPI");

            Err(Error::LocalApic)
        }
    }

    /// Запрещает ненужные нам локальные прерывания.
    ///
    /// <https://wiki.osdev.org/APIC#Local_Vector_Table_Registers>
    ///
    /// # Safety
    ///
    /// Local APIC должен быть переведён в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    /// методом [`LocalApic::enable_x2()`].
    unsafe fn disable_lvts() {
        /// Отключает получение прерывания.
        const MASK_INTERRUPT: u32 = 1 << 16;

        for mut interrupt in [
            Register::LvtTermalSensor,
            Register::LvtPerformanceCounterOverflow,
            Register::LvtLocalInterrupt1,
            Register::LvtError,
        ] {
            unsafe {
                interrupt.write(interrupt.read() | MASK_INTERRUPT);
            }
        }
    }

    /// Разрешает использование local APIC.
    ///
    /// <https://wiki.osdev.org/APIC#Spurious_Interrupt_Vector_Register>
    ///
    /// # Safety
    ///
    /// Local APIC должен быть переведён в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    /// методом [`LocalApic::enable_x2()`].
    unsafe fn enable() {
        /// Включает использование local APIC.
        const ENABLE_LOCAL_APIC: u32 = 1 << 8;

        unsafe {
            Register::Spurious.write(ENABLE_LOCAL_APIC | u32::from(Trap::Spurious));
        }
    }

    /// Переводит local APIC в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    ///
    /// [16.9 Detecting and Enabling x2APIC Mode](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1046719).
    ///
    /// # Errors
    ///
    /// - [`Error::Unimplemented`] --- процессор не поддерживает
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    fn enable_x2() -> Result<()> {
        let (frame, mut flags) = ApicBase::read();

        flags |= ApicBaseFlags::LAPIC_ENABLE;

        unsafe {
            ApicBase::write(frame, flags);
        }

        if cpuid::CpuId::new()
            .get_feature_info()
            .map(|feature_info| feature_info.has_x2apic())
            .unwrap_or(false)
        {
            unsafe {
                ApicBase::write(frame, flags | ApicBaseFlags::X2APIC_ENABLE);
            }

            Ok(())
        } else {
            Err(Unimplemented)
        }
    }

    /// Инициализирует таймер local APIC в периодическом режиме с делителем 1
    /// и прерыванием номер [`Trap::Timer`].
    ///
    /// [16.4.1 APIC Timer Interrupt](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1000026).
    ///
    /// # Safety
    ///
    /// Local APIC должен быть переведён в
    /// [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    /// методом [`LocalApic::enable_x2()`].
    unsafe fn init_timer(tscs_per_interrupt: u32) {
        /// Задаёт делитель таймера равный 1.
        const DIVIDE_BY_1: u32 = 0b1011;

        /// Задаёт периодический режим таймера.
        const PERIODIC_MODE: u32 = 0b01 << 17;

        unsafe {
            Register::TimerDivideConfiguration.write(DIVIDE_BY_1);
            Register::LvtTimer.write(PERIODIC_MODE | u32::from(Trap::Timer));

            // [AMD64 Architecture Programmer’s Manual](https://docs.amd.com/v/u/en-US/40332-PUB_4.08):
            // "To avoid race conditions, software should initialize
            // the Divide Configuration Register and the Timer Local Vector Table Register
            // prior to writing the Initial Count Register to start the timer."
            Register::TimerInitialCount.write(tscs_per_interrupt);
        }
    }
}

bitflags! {
    /// Поля регистра Interrupt Command Register (ICR) для отправки
    /// [inter-processor interrupt](https://en.wikipedia.org/wiki/Inter-processor_interrupt) (IPI).
    ///
    /// [Figure 16-18. Interrupt Command Register (APIC Offset 300h–310h)](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1009018).
    /// 10.6.1 Interrupt Command Register (ICR)
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct InterruptCommand: u32 {
        /// Выполнить инициализацию процессора.
        const INIT = 0b101 << 8;

        /// Выполнить код загрузки по заданному адресу.
        const START_UP = 0b110 << 8;

        /// Используется физический идентификатор целевого CPU.
        const DESTINATION_PHYSICAL = 0 << 11;

        /// Высокий уровень сигнала для прерывания [`InterruptCommand::INIT`].
        const LEVEL_ASSERT = 1 << 14;

        /// Низкий уровень сигнала для прерывания [`InterruptCommand::INIT`].
        const LEVEL_DEASSERT = 0 << 14;

        /// Выбрать режим "по уровню" для прерывания [`InterruptCommand::INIT`].
        const TRIGGER_MODE_LEVEL = 1 << 15;
    }
}

/// Регистр local APIC в
/// [режиме x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
///
/// Задаёт номера
/// [моделезависимых регистров](https://en.wikipedia.org/wiki/Model-specific_register)
/// в соответствии с
/// [таблицей](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1047581).
///
/// <https://wiki.osdev.org/APIC#Local_APIC_registers>
#[derive(Clone, Copy, Debug, IntoPrimitive)]
#[repr(u32)]
enum Register {
    /// Возвращает идентификатор local APIC и текущего CPU.
    Id = 0x802,

    /// Посылает local APIC сигнал о завершении работы обработчика прерывания ---
    /// [end of interrupt (EOI)](https://en.wikipedia.org/wiki/End_of_interrupt).
    Eoi = 0x80B,

    /// Регистр таймера.
    LvtTimer = 0x832,

    /// Обработчик "фантомных прерываний"
    /// ([Spurious interrupts](https://en.wikipedia.org/wiki/Interrupt#Spurious_interrupts)).
    ///
    /// <https://wiki.osdev.org/APIC#Spurious_Interrupt_Vector_Register>
    Spurious = 0x80F,

    /// Регистр ошибок.
    ErrorStatus = 0x828,

    /// Регистр сенсора температуры.
    LvtTermalSensor = 0x833,

    /// Регистр переполнения
    /// [hardware performance counter](https://en.wikipedia.org/wiki/Hardware_performance_counter)
    /// (HPC).
    LvtPerformanceCounterOverflow = 0x834,

    #[doc(hidden)]
    LvtLocalInterrupt1 = 0x836,

    #[doc(hidden)]
    LvtError = 0x837,

    /// Начальное значение счётчика таймера.
    TimerInitialCount = 0x838,

    /// Делитель счётчика таймера.
    TimerDivideConfiguration = 0x83E,
}

impl Register {
    /// Читает значение из регистра `register` local APIC в
    /// [режиме x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    ///
    /// # Safety
    ///
    /// - Local APIC должен быть переведён в
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    ///   методом [`LocalApic::enable_x2()`].
    /// - Читать можно только из
    ///   [определённых регистров local APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1047581).
    unsafe fn read(&self) -> u32 {
        unsafe {
            u32::try_from(Msr::new((*self).into()).read() & u64::from(u32::MAX))
                .expect("should fit")
        }
    }

    /// Записывает значение в регистр `register` local APIC в
    /// [режиме x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352).
    ///
    /// # Safety
    ///
    /// - Local APIC должен быть переведён в
    ///   [режим x2APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1044352)
    ///   методом [`LocalApic::enable_x2()`].
    /// - Записывать можно только в
    ///   [определённые регистры local APIC](https://docs.amd.com/v/u/en-US/40332_4.10_APM_Vol1-5_PUB#G22.1047581)
    ///   и в определённом для них формате.
    unsafe fn write(
        &mut self,
        value: u32,
    ) {
        unsafe {
            Msr::new((*self).into()).write(value.into());
        }
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use super::{
        CpuId,
        LocalApic,
    };

    pub fn id() -> CpuId {
        LocalApic::id()
    }
}
