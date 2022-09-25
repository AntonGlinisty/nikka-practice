use core::{
    convert::From,
    hint,
    sync::atomic::{
        AtomicU8,
        Ordering,
    },
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
use x86_64::instructions::interrupts;

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
///
/// Во время изменения настроек RTC
/// запрещает все [прерывания](https://en.wikipedia.org/wiki/Interrupt),
/// в том числе [немаскируемые](https://en.wikipedia.org/wiki/Non-maskable_interrupt).
/// Иначе она может остаться в
/// [некоррекном состоянии](https://wiki.osdev.org/RTC#Avoiding_NMI_and_Other_Interrupts_While_Programming).
pub(super) fn init() {
    let mut old_settings = RegisterB::empty();
    let mut new_settings = RegisterB::empty();
    let mut acknowledged_settings = RegisterB::empty();

    interrupts::without_interrupts(|| {
        defer! {
            rtc_read(!DISABLE_NMI);
        }

        old_settings = RegisterB::from_bits(rtc_read(DISABLE_NMI | REGISTER_B)).unwrap();
        new_settings =
            (old_settings & !RegisterB::DAYLIGHT_SAVING) | RegisterB::UPDATE_ENDED_INTERRUPT;
        rtc_write(DISABLE_NMI | REGISTER_B, new_settings.bits());
        acknowledged_settings = RegisterB::from_bits(rtc_read(DISABLE_NMI | REGISTER_B)).unwrap();

        SETTINGS.store(acknowledged_settings.bits(), Ordering::Relaxed);
    });

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
    /// Регистр настроек RTC.
    pub struct RegisterB: u8 {
        /// Включает переход на летнее время.
        const DAYLIGHT_SAVING = 1 << 0;

        /// Время в микросхеме хранится
        /// в [24-часовом формате](https://en.wikipedia.org/wiki/24-hour_clock),
        /// а не в [12--часовом](https://en.wikipedia.org/wiki/12-hour_clock).
        const USE_24_HOUR_FORMAT = 1 << 1;

        /// Время в микросхеме хранится
        /// в [двоичном коде](https://en.wikipedia.org/wiki/Binary_number),
        /// а не в [двоично--десятичном](https://en.wikipedia.org/wiki/Binary-coded_decimal).
        const USE_BINARY_FORMAT = 1 << 2;

        /// Генерировать сигнал с конфигурируемой частотой но отдельном выходе микросхемы.
        const SQUARE_WAVE = 1 << 3;

        /// Включает [прерывание](https://en.wikipedia.org/wiki/Interrupt), посылаемое процессору микросхемой после обновления показаний времени при тике.
        const UPDATE_ENDED_INTERRUPT = 1 << 4;

        /// Включает [прерывание](https://en.wikipedia.org/wiki/Interrupt), посылаемое процессору при срабатывании будильника.
        const ALARM_INTERRUPT = 1 << 5;

        /// Включает периодическое [прерывание](https://en.wikipedia.org/wiki/Interrupt) с конфигурируемой частотой.
        const PERIODIC_INTERRUPT = 1 << 6;

        /// Сообщает микросхеме, что процессор меняет дату и время.
        /// Пока процессор не сбросит этот бит, микросхема не будет их обновлять.
        const SET_CLOCK = 1 << 7;
    }
}

bitflags! {
    /// Регистр статуса прерывания RTC. Сбрасывается при чтении.
    struct RegisterC: u8 {
        /// Микросхема сгенерировала прерывание как минимум одного из типов.
        const INTERRUPT = 1 << 7;

        /// Сгенерировано периодическое прерывание.
        const PERIODIC_INTERRUPT = 1 << 6;

        /// Сгенерировано прерывание будильника.
        const ALARM_INTERRUPT = 1 << 5;

        /// Сгенерировано прерывание после обновления показаний времени.
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

/// Номер порта для выбора адреса в памяти микросхемы RTC.
const ADDRESS_PORT: u16 = 0x0070;

/// Номер порта для обмена данными с памятью микросхемы RTC.
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
    RegisterC::from_bits_truncate(rtc_read(REGISTER_C))
}

/// Проверяет правильность данных даты и времени в RTC.
///
/// На самом деле просто заряд в батарейке.
fn is_time_valid() -> bool {
    let register_d = RegisterD::from_bits_truncate(rtc_read(REGISTER_D));
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

/// Переводит значение `x` из формата RTC `format` в двоичный.
///
/// `format` может быть как
/// [двоично--десятичным](https://en.wikipedia.org/wiki/Binary-coded_decimal), так и
/// [двоичным](https://en.wikipedia.org/wiki/Binary_number), см. [`RegisterB`].
///
/// [Подробнее про формат времени в RTC](https://wiki.osdev.org/CMOS#Format_of_Bytes).
fn parse_value(
    x: u8,
    format: RegisterB,
) -> u8 {
    // TODO: your code here.
    x // TODO: remove before flight.
}

/// Переводит `hour` из формата RTC `format` в двоичный 24-ти часовой формат.
///
/// `format` может иметь четыре варианта, независимо задаваемые двумя битами, см. [`RegisterB`]:
///   - [двоично--десятичный](https://en.wikipedia.org/wiki/Binary-coded_decimal) или
///     [двоичный](https://en.wikipedia.org/wiki/Binary_number);
///   - [12-ти часовой](https://en.wikipedia.org/wiki/12-hour_clock) или
///     [24-часовой](https://en.wikipedia.org/wiki/24-hour_clock).
///
/// [Подробнее про формат времени в RTC](https://wiki.osdev.org/CMOS#Format_of_Bytes).
fn parse_hour(
    hour: u8,
    format: RegisterB,
) -> u8 {
    // TODO: your code here.
    hour // TODO: remove before flight.
}

/// Копия текущих настроек микросхемы --- [`RegisterB`].
static SETTINGS: AtomicU8 = AtomicU8::new(0);

/// Запрет [немаскируемых прерываний](https://en.wikipedia.org/wiki/Non-maskable_interrupt).
/// Разделяет тот же номер [порта ввода--вывода](https://wiki.osdev.org/Port_IO), что и [`ADDRESS_PORT`].
const DISABLE_NMI: u8 = 1 << 7;

/// Адрес регистра статуса RTC.
const REGISTER_A: u8 = 0xA;

/// Адрес регистра настроек RTC.
const REGISTER_B: u8 = 0xB;

/// Адрес регистра статуса прерывания RTC.
const REGISTER_C: u8 = 0xC;

/// Адрес регистра сохранности данных в памяти RTC при выключении.
const REGISTER_D: u8 = 0xD;

#[doc(hidden)]
pub(super) mod test_scaffolding {
    pub use super::RegisterB;

    pub fn parse_hour(
        hour: u8,
        format: RegisterB,
    ) -> u8 {
        super::parse_hour(hour, format)
    }
}
