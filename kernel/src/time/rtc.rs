use core::{
    convert::From,
    hint,
};

use bitflags::bitflags;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    Utc,
};
use derive_more::Display;
use scopeguard::defer;
use x86::io;

use ku::time::{
    CorrelationPoint,
    rtc::TICKS_PER_SECOND,
};

use crate::{
    SYSTEM_INFO,
    log::{
        error,
        info,
    },
};

/// Обработчик прерываний [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
pub(crate) fn interrupt() {
    if interrupt_status().contains(RegisterC::UPDATE_ENDED_INTERRUPT) &&
        let Some(timestamp) = timestamp()
    {
        let now = CorrelationPoint::now(timestamp * TICKS_PER_SECOND);
        let rtc = SYSTEM_INFO.rtc();
        rtc.init_base(now);
        rtc.store_prev(now);
    }
}

/// Инициализация микросхемы [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
pub(super) fn init() {
    const DISABLE_NMI: u8 = 1 << 7;
    const REGISTER_B: u8 = 0xB;

    defer! {
        rtc_read(!DISABLE_NMI);
    }

    let old_settings =
        unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };
    let new_settings = (old_settings & !RegisterB::DAYLIGHT_SAVING) |
        RegisterB::UPDATE_ENDED_INTERRUPT |
        RegisterB::USE_BINARY_FORMAT |
        RegisterB::USE_24_HOUR_FORMAT;
    rtc_write(DISABLE_NMI | REGISTER_B, new_settings.bits());
    let acknowledged_settings =
        unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };

    if acknowledged_settings == new_settings {
        let rtc = SYSTEM_INFO.rtc();
        rtc.store_prev(CorrelationPoint::invalid(
            timestamp().unwrap_or(0) * TICKS_PER_SECOND,
        ));

        enable_next_interrupt();

        if is_time_valid() {
            info!(?acknowledged_settings, "RTC init");
        } else {
            error!("RTC reports low battery, its time and date values are incorrect");
        }
    } else {
        error!(
            ?old_settings,
            ?new_settings,
            ?acknowledged_settings,
            "RTC did not acknowledge new settings",
        );
    }
}

bitflags! {
    /// Регистр статуса RTC.
    struct RegisterA: u8 {
        const UPDATE_IN_PROGRESS = 1 << 7;
    }
}

bitflags! {
    /// Командный регистр RTC.
    struct RegisterB: u8 {
        /// Включает переход на летнее время.
        const DAYLIGHT_SAVING = 1 << 0;

        /// Требует от микросхемы использовать 24--часовой формат времени, а не 12--часовой.
        const USE_24_HOUR_FORMAT = 1 << 1;

        /// Требует от микросхемы возвращать данные в двоичном коде, а не в [двоично--десятичном](https://en.wikipedia.org/wiki/Binary-coded_decimal).
        const USE_BINARY_FORMAT = 1 << 2;

        const SQUARE_WAVE = 1 << 3;

        /// Включает [прерывание](https://en.wikipedia.org/wiki/Interrupt), посылаемое процессору микросхемой после обновления показаний времени при тике.
        const UPDATE_ENDED_INTERRUPT = 1 << 4;

        /// Включает [прерывание](https://en.wikipedia.org/wiki/Interrupt), посылаемое процессору при срабатывании будильника.
        const ALARM_INTERRUPT = 1 << 5;

        /// Включает периодическое [прерывание](https://en.wikipedia.org/wiki/Interrupt).
        const PERIODIC_INTERRUPT = 1 << 6;

        const SET_CLOCK = 1 << 7;
    }
}

bitflags! {
    /// Регистр статуса прерывания RTC.
    struct RegisterC: u8 {
        const UPDATE_ENDED_INTERRUPT = 1 << 4;
    }
}

bitflags! {
    /// Регистр сохранности данных в памяти RTC при выключении.
    struct RegisterD: u8 {
        /// Есть заряд в батарейке.
        /// Поэтому данные в памяти RTC валидны, в том числе дата и время.
        const VALID_RAM_AND_TIME = 1 << 7;
    }
}

/// Структура для хранения даты и времени, прочитанных из микросхемы [часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock).
#[derive(Clone, Copy, Default, Display, Eq, PartialEq)]
#[display(
    fmt = "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
    year,
    month,
    day,
    hour,
    minute,
    second
)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl Date {
    /// [Пытается несколько раз прочитать данные из микросхемы RTC](https://wiki.osdev.org/CMOS#RTC_Update_In_Progress)
    /// методом [`Date::read_inconsistent()`].
    /// Перед каждым чтением в цикле ждёт, пока в регистре `A` микросхемы RTC флаг
    /// [`RegisterA::UPDATE_IN_PROGRESS`] установлен, то есть пока микросхема обновляет данные в своей памяти.
    /// Возвращает [`Some`], если два чтения подряд вернут одинаковое значение структуры [`Date`].
    fn read() -> Option<Self> {
        if !is_time_valid() {
            return None;
        }

        const REGISTER_A: u8 = 0xA;

        // TODO: your code here.
        Some(Self::read_inconsistent()) // TODO: remove before flight.
    }

    /// Считывает из микросхемы RTC показания даты и времени и возвращает их в виде [`Date`].
    ///
    /// Может вернуть некорректное значение [`Date`],
    /// если во время его работы произошёл тик RTC и микросхема конкурентно
    /// обновляла содержимое соответствующих полей в своей памяти.
    fn read_inconsistent() -> Self {
        // TODO: your code here.
        Self::default() // TODO: remove before flight.
    }
}

impl From<Date> for NaiveDateTime {
    fn from(date: Date) -> NaiveDateTime {
        if date == Date::default() {
            Self::default()
        } else {
            NaiveDate::from_ymd(date.year.into(), date.month.into(), date.day.into()).and_hms(
                date.hour.into(),
                date.minute.into(),
                date.second.into(),
            )
        }
    }
}

const ADDRESS_PORT: u16 = 0x0070;
const DATA_PORT: u16 = 0x0071;

/// Читает значение, которое находится в байте номер `address` внутренней памяти микросхемы RTC.
///
/// Адрес `address` не имеет отношения к основной памяти компьютера, он адресует внутреннюю память микросхемы RTC.
fn rtc_read(address: u8) -> u8 {
    unsafe {
        io::outb(ADDRESS_PORT, address);
        io::inb(DATA_PORT)
    }
}

/// Записывает значение `data`, в байт номер `address` внутренней памяти микросхемы RTC.
///
/// Адрес `address` не имеет отношения к основной памяти компьютера, он адресует внутреннюю память микросхемы RTC.
fn rtc_write(
    address: u8,
    data: u8,
) {
    unsafe {
        io::outb(ADDRESS_PORT, address);
        io::outb(DATA_PORT, data);
    }
}

/// Читает регистр статуса прерывания RTC.
fn interrupt_status() -> RegisterC {
    const REGISTER_C: u8 = 0xC;
    unsafe { RegisterC::from_bits_unchecked(rtc_read(REGISTER_C)) }
}

/// Проверяет правильность данных даты и времени в RTC.
///
/// На самом деле просто заряд в батарейке.
fn is_time_valid() -> bool {
    const REGISTER_D: u8 = 0xD;
    let register_d = unsafe { RegisterD::from_bits_unchecked(rtc_read(REGISTER_D)) };
    register_d.contains(RegisterD::VALID_RAM_AND_TIME)
}

/// Говорит микросхеме RTC, что процессор обработал [прерывание](https://en.wikipedia.org/wiki/Interrupt) от неё.
/// То есть, посылает ей [end of interrupt (EOI)](https://en.wikipedia.org/wiki/End_of_interrupt).
///
/// См. [Interrupts and Register C](https://wiki.osdev.org/RTC#Interrupts_and_Register_C).
fn enable_next_interrupt() {
    interrupt_status();
}

/// Переводит текущее время RTC в [секунды с момента начала Unix--эпохи](https://en.wikipedia.org/wiki/Unix_time).
/// Предполагает, что микросхема RTC хранит [всемирное координированное время (Coordinated Universal Time, UTC)](https://en.wikipedia.org/wiki/Coordinated_Universal_Time).
fn timestamp() -> Option<i64> {
    Date::read().map(|date| DateTime::<Utc>::from_utc(date.into(), Utc).timestamp())
}
