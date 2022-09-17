## Часы реального времени

Краткое описание возможностей микросхемы RTC и её низкоуровневого интерфейса можно посмотреть в статьях
[RTC](https://wiki.osdev.org/RTC) и [CMOS](https://wiki.osdev.org/CMOS)
на сайте [OSDev.org](https://wiki.osdev.org/).
Подробное описание есть в
[спецификации микросхемы Motorola MC146818](https://pdf1.alldatasheet.com/datasheet-pdf/view/122156/MOTOROLA/MC146818.html).
Современные микросхемы RTC поддерживают совместимость с ней.

Основная работа с микросхемой
[часов реального времени (Real-time clock, RTC)](https://en.wikipedia.org/wiki/Real-time_clock)
собрана в модуле
[`kernel::time::rtc`](../../doc/kernel/time/rtc/index.html),
который находится в файле
[`kernel/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/rtc.rs).


### Инициализация микросхемы часов реального времени

Функция
[`kernel::time::rtc::init()`](../../doc/kernel/time/rtc/fn.init.html)
выполняет инициализацию микросхемы:

```rust
let old_settings =
    unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };
let new_settings = (old_settings & !RegisterB::DAYLIGHT_SAVING) |
    RegisterB::UPDATE_ENDED_INTERRUPT |
    RegisterB::USE_BINARY_FORMAT |
    RegisterB::USE_24_HOUR_FORMAT;
unsafe { rtc_write(DISABLE_NMI | REGISTER_B, new_settings.bits()) };
```

То есть, она

- Выключает переход на летнее время --- `!`[`RegisterB::DAYLIGHT_SAVING`](../../doc/kernel/time/rtc/struct.RegisterB.html#associatedconstant.DAYLIGHT_SAVING).
- Включает [прерывание](https://en.wikipedia.org/wiki/Interrupt), посылаемое процессору микросхемой после обновления показаний времени при тике, --- [`RegisterB::UPDATE_ENDED_INTERRUPT`](../../doc/kernel/time/rtc/struct.RegisterB.html#associatedconstant.UPDATE_ENDED_INTERRUPT).
- Требует от микросхемы возвращать данные в двоичном коде, а не в [двоично--десятичном](https://en.wikipedia.org/wiki/Binary-coded_decimal), --- [`RegisterB::USE_BINARY_FORMAT`](../../doc/kernel/time/rtc/struct.RegisterB.html#associatedconstant.USE_BINARY_FORMAT).
- Требует от микросхемы использовать 24--часовой формат времени, а не 12--часовой, --- [`RegisterB::USE_24_HOUR_FORMAT`](../../doc/kernel/time/rtc/struct.RegisterB.html#associatedconstant.USE_24_HOUR_FORMAT).
- Остальные настройки оставляет без изменений.

Далее
[`rtc::init()`](../../doc/kernel/time/rtc/fn.init.html)
проверяет что микросхема подтвердила изменение конфигурации и читает текущие показания времени из микросхемы

```rust
let acknowledged_settings = unsafe { RegisterB::from_bits_unchecked(rtc_read(DISABLE_NMI | REGISTER_B)) };

if acknowledged_settings == new_settings {
    let rtc = SYSTEM_INFO.rtc();
    rtc.store_prev(CorrelationPoint::invalid(
        timestamp().unwrap_or(0) * TICKS_PER_SECOND,
    ));

    enable_next_interrupt();

    ...
}
```

Количество тиков в секунду ---
[`ku::time::rtc::TICKS_PER_SECOND`](../../doc/ku/time/rtc/constant.TICKS_PER_SECOND.html) ---
для RTC равно 1.
Про то, что такое
[`ku::info::SYSTEM_INFO`](../../doc/ku/info/static.SYSTEM_INFO.html) и
[`ku::time::correlation_point::CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html)
узнаем чуть позже.

Микросхема RTC выдаёт текущее время, разбитое на компоненты --- год, месяц, день, час, минута, секунда.
Оно переводится в
[секунды с момента начала Unix--эпохи](https://en.wikipedia.org/wiki/Unix_time)
функцией
[`kernel::time::rtc::timestamp()`](../../doc/kernel/time/rtc/fn.timestamp.html):

```rust
fn timestamp() -> Option<i64> {
    Date::now().map(|now| DateTime::<Utc>::from_utc(now, Utc).timestamp())
}
```

В ней используется внешняя библиотека
[`chrono`](../../doc/chrono/index.html),
которая реализует работу со временем и календарём.

При этом код считает, что микросхема RTC хранит
[всемирное координированное время (Coordinated Universal Time, UTC)](https://en.wikipedia.org/wiki/Coordinated_Universal_Time).
Это так в тестах, которые запускают Nikka в эмуляторе [`qemu`](https://en.wikipedia.org/wiki/QEMU).
По умолчанию в [`qemu`](https://en.wikipedia.org/wiki/QEMU) используется UTC.

Функция [`enable_next_interrupt()`](../../doc/kernel/time/rtc/fn.enable_next_interrupt.html)
говорит микросхеме RTC, что процессор обработал
[прерывание](https://en.wikipedia.org/wiki/Interrupt) от неё.
Пока процессор этого не сделает, микросхема не пошлёт следующее прерывание,
считая что процессор ещё занят обработкой предыдущего.
То есть, если забыть это сделать, то "время остановится".
Такой сигнал оборудованию от процессора иногда называется
[end of interrupt (EOI)](https://en.wikipedia.org/wiki/End_of_interrupt).
Он встречается не только для RTC, но и для другого оборудования.
Например, контроллер прерываний можно сконфигурировать так,
чтобы он тоже ждал от процессора подтверждения каждого прерывания.
Посылать EOI соответствующему оборудованию нужно из каждого прерывания.
Мы также посылаем его один раз при инициализации микросхемы RTC ---
на всякий случай стоит перевести микросхему в определённое состояние.
Кроме того, возможно она уже успела послать прерывание до того как ядро инициализировало их обработку.
Тогда мы его потеряли, но должны разблокировать последующие.
В частном случае микросхемы RTC, отправка ей EOI и чтение её регистра статуса прерывания функцией
[`interrupt_status()`](../../doc/kernel/time/rtc/fn.interrupt_status.html), ---
это одно и то же действие:

```rust
fn enable_next_interrupt() {
    interrupt_status();
}
```


### Задача 1-hw-1-rtc

#### Несогласованное чтение даты и времени из микросхемы RTC

Реализуйте статический [метод](../../doc/kernel/time/rtc/struct.Date.html#method.read_inconsistent)

```rust
fn Date::read_inconsistent() -> Date
```

в файле [`kernel/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/rtc.rs),
который считывает из микросхемы RTC показания даты и времени и возвращает их в виде
[структуры](../../doc/kernel/time/rtc/struct.Date.html)

```rust
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
```

Метод называется `...inconsistent()` и помечен `unsafe`, потому что может вернуть некорректное значение
[`Date`](../../doc/kernel/time/rtc/struct.Date.html),
если во время его работы произошёл тик RTC и микросхема конкурентно
обновляла содержимое соответствующих полей в своей памяти.

Вам могут пригодиться:

- Функция [`kernel::time::rtc::rtc_read(address)`](../../doc/kernel/time/rtc/fn.rtc_read.html), которая читает из микросхемы один байт [`u8`](https://doc.rust-lang.org/nightly/core/primitive.u8.html), расположенный по адресу `address` в памяти микросхемы RTC. Адрес `address` не имеет отношения к основной памяти компьютера, он адресует внутреннюю память микросхемы RTC.
- [Таблица адресов памяти RTC](http://www.bioscentral.com/misc/cmosmap.htm) или её [подходящий фрагмент](https://wiki.osdev.org/CMOS#Getting_Current_Date_and_Time_from_RTC). Либо вы можете найти эту таблицу в первоисточнике --- [спецификации микросхемы Motorola MC146818](https://pdf1.alldatasheet.com/datasheet-pdf/view/122156/MOTOROLA/MC146818.html).
- Метод [`u16::from(u8)`](https://doc.rust-lang.org/nightly/core/primitive.u16.html#impl-From%3Cu8%3E-for-u16) для перевода значения типа [`u8`](https://doc.rust-lang.org/nightly/core/primitive.u8.html) в значение типа [`u16`](https://doc.rust-lang.org/nightly/core/primitive.u16.html).


#### Согласованное чтение даты и времени из микросхемы RTC

Реализуйте статический [метод](../../doc/kernel/time/rtc/struct.Date.html#method.read)

```rust
fn Date::read() -> Option<Date>
```

Он
[пытается несколько раз прочитать данные из микросхемы RTC](https://wiki.osdev.org/CMOS#RTC_Update_In_Progress)
уже реализованным вами методом
[`Date::read_inconsistent()`](../../doc/kernel/time/rtc/struct.Date.html#method.read_inconsistent).
Перед каждым чтением он в цикле ждёт, пока в регистре `A` микросхемы RTC флаг
[`kernel::time::rtc::RegisterA::UPDATE_IN_PROGRESS`](../../doc/kernel/time/rtc/struct.RegisterA.html#associatedconstant.UPDATE_IN_PROGRESS)
установлен, то есть пока микросхема обновляет данные в своей памяти.

Если

- Два чтения подряд вернут одинаковое значение структуры [`Date`](../../doc/kernel/time/rtc/struct.Date.html).
- Перед чтениями флаг [`RegisterA::UPDATE_IN_PROGRESS`](../../doc/kernel/time/rtc/struct.RegisterA.html#associatedconstant.UPDATE_IN_PROGRESS) был сброшен.
- Предполагаем, что микросхема обновляет поля всегда в одном порядке.
- Чтение одного [`u8`](https://doc.rust-lang.org/nightly/core/primitive.u8.html) из её памяти атомарно.

То можно считать совпавшее значение структуры
[`Date`](../../doc/kernel/time/rtc/struct.Date.html)
согласованным и вернуть его,
[обернув](https://doc.rust-lang.ru/book/ch06-01-defining-an-enum.html#%D0%9F%D0%B5%D1%80%D0%B5%D1%87%D0%B8%D1%81%D0%BB%D0%B5%D0%BD%D0%B8%D0%B5-option-%D0%B8-%D0%B5%D0%B3%D0%BE-%D0%BF%D1%80%D0%B5%D0%B8%D0%BC%D1%83%D1%89%D0%B5%D1%81%D1%82%D0%B2%D0%B0-%D0%BF%D0%B5%D1%80%D0%B5%D0%B4-null-%D0%B7%D0%BD%D0%B0%D1%87%D0%B5%D0%BD%D0%B8%D1%8F%D0%BC%D0%B8) в
[`core::option::Option::Some`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.Some):

```rust
return Some(date);
```

Если же за несколько попыток прочитать дважды одинаковые значения
[`Date`](../../doc/kernel/time/rtc/struct.Date.html)
не получилось,
стоит сдаться и вернуть
[`core::option::Option::None`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None):

```rust
return None;
```


Вам могут пригодиться:

- Метод [`Date::default()`](../../doc/kernel/time/rtc/struct.Date.html#method.default), который возвращает [`Date`](../../doc/kernel/time/rtc/struct.Date.html), заполненную нулями. Что не соответствует разумной дате настоящего времени.
- Функция [`core::hint::spin_loop()`](https://doc.rust-lang.org/nightly/core/hint/fn.spin_loop.html), которая сообщает процессору, что он находится в цикле ожидания внешнего события. И может, например, снизить частоту и энергопотребление.
- Операторы равенства `==` и неравенства `!=` для [`Date`](../../doc/kernel/time/rtc/struct.Date.html).

В этом методе нужно писать код очень внимательно, так как он не покрывается тестами.
А использование в будущем журнала с несогласованной датой может привести к потере времени при отладке.

> Полные пути `core::option::Option::` можно не указывать, так как `Option`, `None`, `Some` и другие стандартные вещи
> импортируются по умолчанию прелюдией
> [`core::prelude`](https://doc.rust-lang.org/nightly/core/prelude/index.html).
> У нас [редакция 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html),
> соответствующая прелюдия ---
> [`core::prelude::rust_2021`](https://doc.rust-lang.org/nightly/core/prelude/rust_2021/index.html).
> А она перекладывает основную работу на
> [`core::prelude::v1`](https://doc.rust-lang.org/nightly/core/prelude/v1/index.html),
> где и импортируются `Option`, `None`, `Some` и многое другое.
>
> Метод [`Date::default()`](../../doc/kernel/time/rtc/struct.Date.html#method.default) определён в
> [типаже](https://doc.rust-lang.ru/book/ch10-02-traits.html)
> [`core::default::Default`](https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default),
> который реализован для [`Date`](../../doc/kernel/time/rtc/struct.Date.html) автоматически за счёт атрибута
> [`derive`](https://doc.rust-lang.ru/book/appendix-03-derivable-traits.html) ---
> [`#[derive(Default)]`](https://doc.rust-lang.ru/book/appendix-03-derivable-traits.html#default-%D0%B4%D0%BB%D1%8F-%D0%B7%D0%BD%D0%B0%D1%87%D0%B5%D0%BD%D0%B8%D0%B9-%D0%BF%D0%BE-%D1%83%D0%BC%D0%BE%D0%BB%D1%87%D0%B0%D0%BD%D0%B8%D1%8E).
> Всю соответствующую работу выполняет макрос
> [`core::prelude::v1::derive`](https://doc.rust-lang.org/nightly/core/prelude/v1/macro.derive.html).
>
> Операторы равенства и неравенства для [`Date`](../../doc/kernel/time/rtc/struct.Date.html) определены в
> [типажах](https://doc.rust-lang.ru/book/ch10-02-traits.html)
> [`core::cmp::PartialEq`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html) и
> [`core::cmp::Eq`](https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html),
> которые реализованы для [`Date`](../../doc/kernel/time/rtc/struct.Date.html) автоматически за счёт атрибута
> [`#[derive(..., Eq, PartialEq)]`](https://doc.rust-lang.ru/book/appendix-03-derivable-traits.html#partialeq-%D0%B8-eq-%D0%B4%D0%BB%D1%8F-%D1%81%D1%80%D0%B0%D0%B2%D0%BD%D0%B5%D0%BD%D0%B8%D1%8F-%D1%80%D0%B0%D0%B2%D0%B5%D0%BD%D1%81%D1%82%D0%B2%D0%B0).
> Типаж [`core::cmp::PartialEq`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html) определяет методы
> [`PartialEq::eq()`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq) и
> [`PartialEq::ne()`](https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne)
> для проверки на равенство или неравенство соответственно.
> Rust рассахаривает операторы `==` и `!=` в обращения к этим методам.
> А дополнительный типаж
> [`core::cmp::Eq`](https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html)
> сигнализирует, что операции `PartialEq` для
> [`Date`](../../doc/kernel/time/rtc/struct.Date.html) задают
> [отношение эквивалентности](https://en.wikipedia.org/wiki/Equivalence_relation), то есть
> [рефлексивное](https://en.wikipedia.org/wiki/Reflexive_relation),
> [симметричное](https://en.wikipedia.org/wiki/Symmetric_relation) и
> [транзитивное](https://en.wikipedia.org/wiki/Transitive_relation).


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/time/rtc.rs |   30 +++++++++++++++++++++++++++---
 1 file changed, 27 insertions(+), 3 deletions(-)
```
