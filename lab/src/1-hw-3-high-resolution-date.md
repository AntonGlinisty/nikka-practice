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
) -> NaiveDateTime
```

должен для часов, к которым привязан `atomic_correlation_interval`, выдать время, соответствующее
такту процессора, записанному в `tsc`.
Время он возвращает в типе
[`chrono::naive::NaiveDateTime`](../../doc/chrono/naive/struct.NaiveDateTime.html),
который не содержит данных о часовом поясе.

Функция
[`ku::time::datetime_with_precision()`](../../doc/ku/time/fn.datetime_with_precision.html)
используя этот метод повышает разрешение часов реального времени
[`ku::time::rtc::Rtc`](../../doc/ku/time/rtc/struct.Rtc.html)
и добавляет к возвращаемому типу информацию о часовом поясе ---
[`chrono::offset::Utc`](../../doc/chrono/offset/struct.Utc.html):

```rust
fn datetime_with_precision<const PARTS_PER_SECOND: i64>(tsc: Tsc) -> DateTime<Utc> {
    DateTime::from_utc(Rtc::datetime::<PARTS_PER_SECOND>(tsc), Utc)
}
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
[`NaiveDateTime`](../../doc/chrono/naive/struct.NaiveDateTime.html).
Мы заметим это на старте системы по низкому --- секундному --- разрешению в записях журнала.


### Задача 1 --- реализация основного метода повышения разрешения часов

Реализуйте метод
[`fn CorrelationInterval::datetime_with_resolution()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime_with_resolution).

- Считайте, что `self.base.count()` и `self.prev.count()` задают разные секунды с начала Unix--эпохи.
- Необходимо вернуть реальное время, которое соответствует такту номер `tsc.get()` процессора. Он может быть как больше `self.prev.tsc()`, так и меньше `self.base.tsc()`, а может лежать где-то между ними. Чтобы все эти варианты можно было поддержать, такты процессора хранятся в знаковом типе [`i64`](https://doc.rust-lang.org/nightly/core/primitive.i64.html).
- Запрошенное разрешение `PARTS_PER_SECOND` не будет выше наносекунд, которые поддерживает [`NaiveDateTime`](../../doc/chrono/naive/struct.NaiveDateTime.html).
- [Високосные секунды](https://en.wikipedia.org/wiki/Leap_second) не поддерживаем.


### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/time/correlation_interval.rs |   20 +++++++++++++++++++-
 1 file changed, 19 insertions(+), 1 deletion(-)
```
