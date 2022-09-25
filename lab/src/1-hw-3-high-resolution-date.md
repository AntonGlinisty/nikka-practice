## Измерение частоты процессора и повышение разрешения часов

Структура
[`ku::time::correlation_interval::CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html)
из файла [`ku/src/time/correlation_interval.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/correlation_interval.rs)
предназначена для соотнесения частоты процессора и другого источника времени.
Она содержит поля:

- [`CorrelationInterval::base`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.base) со знакомым нам [`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) в некоторый базовый момент времени, когда тикнули отслеживаемые [`CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html) часы.
- [`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev) с [`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) на момент последнего тика отслеживаемых часов.

[`CorrelationInterval::base`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.base) и
[`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev)
содержат количество тиков отслеживаемых часов и номер такта процессора в соответствующие моменты времени.
Частота отслеживаемых часов задаётся как константный параметр `TICKS_PER_SECOND` для типа
[`struct CorrelationInterval<const TICKS_PER_SECOND: i64>`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html).

По этой информации можно:

- Вычислить частоту процессора с точки зрения отслеживаемых часов.
- Пересчитать произвольное значение счётчика тактов процессора [`ku::time::tsc::Tsc`](../../doc/ku/time/tsc/struct.Tsc.html) в показания времени отслеживаемых часов. При этом мы фактически повышаем разрешение отслеживаемых часов до частоты процессора. Разумеется, погрешность при этом превышает разрешение.

Для часов реального времени
[`ku::time::rtc::Rtc`](../../doc/ku/time/rtc/struct.Rtc.html),
количество тиков в каждом поле
[`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count)
содержит количество секунд с начала Unix--эпохи.
Методы
[`CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html)
будут опираться на это при привязке своих
[`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html)
к реальному времени.

Структура
[`ku::time::correlation_interval::AtomicCorrelationInterval`](../../doc/ku/time/correlation_interval/struct.AtomicCorrelationInterval.html)
содержит аналогичные два поля типа
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html).
Она находится в таких же отношениях с
[`CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html),
как
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
с
[`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html).
То есть,
[`CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html)
является значением, а
[`AtomicCorrelationInterval`](../../doc/ku/time/correlation_interval/struct.AtomicCorrelationInterval.html)
его конкурентным хранилищем в памяти.

> Подумайте, почему при этом
> [`AtomicCorrelationInterval`](../../doc/ku/time/correlation_interval/struct.AtomicCorrelationInterval.html)
> устроен гораздо проще чем
> [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
> с его [sequence lock](https://en.wikipedia.org/wiki/Seqlock).

Основной [метод](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime)

```rust
fn CorrelationInterval::datetime<const PARTS_PER_SECOND: i64>(
    atomic_correlation_interval: &AtomicCorrelationInterval<TICKS_PER_SECOND>,
    tsc: Tsc,
) -> DateTime<Utc>
```

должен для часов, к которым привязан `atomic_correlation_interval`, выдать время, соответствующее
такту процессора, записанному в `tsc`.
Время он возвращает в типе
[`chrono::DateTime`](../../doc/chrono/struct.DateTime.html).
Он считает, что заданные ему часы показывают количество секунд, прошедших с начала Unix--эпохи в UTC ---
[`chrono::offset::Utc`](../../doc/chrono/offset/struct.Utc.html).

Функции
[`ku::time::datetime()`](../../doc/ku/time/fn.datetime.html)
[`ku::time::datetime_ms()`](../../doc/ku/time/fn.datetime_ms.html)
используя этот метод повышают разрешение часов реального времени
[`ku::time::rtc::Rtc`](../../doc/ku/time/rtc/struct.Rtc.html):

```rust
pub fn datetime(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<NSECS_PER_SEC>(tsc)
}

pub fn datetime_ms(tsc: Tsc) -> DateTime<Utc> {
    Rtc::datetime::<MSECS_PER_SEC>(tsc)
}

const MSECS_PER_SEC: i64 = 1_000;
const NSECS_PER_SEC: i64 = 1_000_000_000;
```

Метод
[`CorrelationInterval::datetime()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime)
читает переданный ему на вход `atomic_correlation_interval`, проверяет что в результирующем
[`CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html)
оба поля ---
[`CorrelationInterval::base`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.base) и
[`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev) ---
валидны и задают разные тики базовых часов.
Если это так, он перекладывает основную работу на вспомогательный
[метод](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime_with_resolution)

```rust
fn CorrelationInterval::datetime_with_resolution<const PARTS_PER_SECOND: i64>(
    &self,
    tsc: Tsc,
) -> NaiveDateTime
```

В тех редких случаях, когда в `atomic_correlation_interval` ещё не прошло два тика часов, метод
[`CorrelationInterval::datetime()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime)
возвращает момент времени
[`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev)
в виде
[`DateTime`](../../doc/chrono/struct.DateTime.html).
Мы заметим это на старте системы по низкому --- секундному --- разрешению в записях журнала.


### Задача 3 --- реализация основного метода повышения разрешения часов

Для этой задачи понадобится готовое решение [задачи 1](../../lab/book/1-hw-1-rtc.html#%D0%97%D0%B0%D0%B4%D0%B0%D1%87%D0%B0-1--%D1%87%D1%82%D0%B5%D0%BD%D0%B8%D0%B5-%D1%80%D0%B5%D0%B0%D0%BB%D1%8C%D0%BD%D0%BE%D0%B3%D0%BE-%D0%B2%D1%80%D0%B5%D0%BC%D0%B5%D0%BD%D0%B8-%D0%B8%D0%B7-%D0%BC%D0%B8%D0%BA%D1%80%D0%BE%D1%81%D1%85%D0%B5%D0%BC%D1%8B-rtc).

Реализуйте метод
[`fn CorrelationInterval::datetime_with_resolution()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime_with_resolution).

- Считайте, что `self.base.count()` и `self.prev.count()` задают разные секунды с начала Unix--эпохи.
- Необходимо вернуть реальное время, которое соответствует такту номер `tsc.get()` процессора. Он может быть как больше `self.prev.tsc()`, так и меньше `self.base.tsc()`, а может лежать где-то между ними. Чтобы все эти варианты можно было поддержать, такты процессора хранятся в знаковом типе [`i64`](https://doc.rust-lang.org/nightly/core/primitive.i64.html).
- Результат возвращается в типе [`chrono::naive::NaiveDateTime`](../../doc/chrono/naive/struct.NaiveDateTime.html), который не содержит данных о часовом поясе. Идентификатор часового пояса добавит вызывающая функция.
- Запрошенное разрешение `PARTS_PER_SECOND` задаётся в единицах в секунду, например для миллисекунд `PARTS_PER_SECOND = 1_000`. Оно не будет выше наносекунд --- `PARTS_PER_SECOND = 1_000_000_000`, --- которые поддерживает [`NaiveDateTime`](../../doc/chrono/naive/struct.NaiveDateTime.html).
- [Високосные секунды](https://en.wikipedia.org/wiki/Leap_second) не поддерживаем.


### Запуск тестов

Тесты можно запустить командой `cargo test --test 1-hw-3-high-resolution-date` в директории `kernel` репозитория.

После выполнения задачи должен проходить тест `high_resolution_date()`
в файле
[`kernel/tests/time.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/time.rs):

```console
$ (cd kernel; cargo test --test 1-hw-3-high-resolution-date)
...
time::high_resolution_date----------------------------------
10:39:50.562 0 D waiting for the RTC to tick twice
10:39:52.001 0 D measuring; timer_ticks = 102
10:39:57.015 0 D measured; samples = 100; min = PT0.049911511S; max = PT0.050046701S; mean = PT0.049999155S; deviation = PT0.000020516S
10:39:57.024 0 D restrictions; min_delta = PT0.040S; max_delta = PT0.060S; min_mean = PT0.049500S; max_mean = PT0.050500S; min_deviation = PT0.000000100S; max_deviation = PT0.005S
time::high_resolution_date------------------------- [passed]
```

### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/time/correlation_interval.rs |   26 ++++++++++++++++++++++++--
 1 file changed, 24 insertions(+), 2 deletions(-)
```
