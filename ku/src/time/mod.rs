#![warn(clippy::missing_docs_in_private_items)]
#![warn(missing_docs)]

/// Вспомогательные структуры [`AtomicCorrelationInterval`] и [`CorrelationInterval`] для соотнесения частоты процессора с частотой другого источника времени по двум моментам времени.
mod correlation_interval;

/// Вспомогательные структуры [`AtomicCorrelationPoint`] и [`CorrelationPoint`] для привязки тактов процессора к другому источнику времени, в один момент времени.
mod correlation_point;

/// Вспомогательная структура [`Hz`] для форматирования [частоты](https://en.wikipedia.org/wiki/Hertz) при журналировании.
mod hz;

/// [Синглтон](https://en.wikipedia.org/wiki/Singleton_pattern) [`Rtc`], позволяющий узнать показания часов реального времени как из ядра, так и из [пространства пользователя](https://en.wikipedia.org/wiki/User_space_and_kernel_space).
pub mod rtc;

/// Структура [`Tsc`] для хранения показаний счётчика тактов процессора, который является одним из источников времени в компьютере. А также структура [`TscDuration`] для хранения интервалов времени в тактах процессора.
mod tsc;

use chrono::{
    DateTime,
    Utc,
};

pub use correlation_interval::AtomicCorrelationInterval;
pub use correlation_point::CorrelationPoint;
pub use hz::Hz;
pub use tsc::{
    Tsc,
    TscDuration,
    tsc,
};

use rtc::Rtc;

// Used in docs.
#[allow(unused)]
use self::{
    correlation_interval::CorrelationInterval,
    correlation_point::AtomicCorrelationPoint,
};

/// Переводит значение счётчика тактов процессора в системное время с разрешением в наносекунды.
pub fn datetime(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<NSECS_PER_SEC>(tsc)
}

/// Переводит значение счётчика тактов процессора в системное время с разрешением в миллисекунды.
pub fn datetime_ms(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<MSECS_PER_SEC>(tsc)
}

/// Сообщает системное время в текущий момент с разрешением в наносекунды.
pub fn now() -> DateTime<Utc> {
    Rtc::datetime::<NSECS_PER_SEC>(Tsc::now())
}

/// Сообщает системное время в текущий момент с разрешением в миллисекунды.
pub fn now_ms() -> DateTime<Utc> {
    Rtc::datetime::<MSECS_PER_SEC>(Tsc::now())
}

/// Функция для получения монотонного процессорного времени, которое измеряется его тактами.
#[inline(always)]
pub fn timer() -> Tsc {
    Tsc::now()
}

/// Количество миллисекунд в одной секунде.
const MSECS_PER_SEC: i64 = 1_000;

/// Количество наносекунд в одной секунде.
const NSECS_PER_SEC: i64 = 1_000_000_000;

#[doc(hidden)]
pub mod test_scaffolding {
    pub use super::{
        correlation_point::test_scaffolding::*,
        tsc::test_scaffolding::*,
    };
}
