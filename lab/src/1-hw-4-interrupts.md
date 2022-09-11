## Обработка прерываний RTC

Как вы помните, при инициализации RTC функция
[`kernel::time::rtc::init()`](../../doc/kernel/time/rtc/fn.init.html)
включает [прерывание](https://en.wikipedia.org/wiki/Interrupt),
которое сигнализирует о том, что RTC обновил свою информацию о времени и дате:

```rust
let new_settings = ...
    RegisterB::UPDATE_ENDED_INTERRUPT |
    ...
```

Оно приходит каждую секунду.

[Прерывание](https://en.wikipedia.org/wiki/Interrupt) --- это событие, которое останавливает текущую активность
процессора и заставляет его переключиться на выполнение специального кода обработки возникшего прерывания.
Прерывания происходят:

- Либо асинхронно, в произвольный момент с точки зрения текущего исполняющегося кода. Обычно --- когда оборудование требует от него внимания.
- Либо синхронно, на фиксированной инструкции исполняющегося кода. Обычно, если возникла особая ситуация при выполнении текущего кода.
  - Это может быть ошибка в программе. Например, деление на ноль или [недопустимое обращение к памяти (Page Fault)](https://en.wikipedia.org/wiki/Page_fault).
  - А может быть предусмотренное действие. Например, чтобы из режима пользователя переключиться в режим ядра для выполнения [системного вызова](https://en.wikipedia.org/wiki/System_call), такого как чтение файла.

За настройки прерываний отвечает функция
[`kernel::trap::init()`](../../doc/kernel/trap/fn.init.html)
в файле [`kernel/src/trap/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/trap/mod.rs).
Она инициализирует таблицу обработчиков прерываний
[`kernel::trap::idt::IDT`](../../doc/kernel/trap/idt/struct.IDT.html)
находящимися в том же файле функциями.
Для RTC выделен номер прерывания
[`kernel::trap::RTC`](../../doc/kernel/trap/constant.RTC.html),
а его обработчик --- функция
[`kernel::trap::pic::rtc()`](../../doc/kernel/trap/pic/fn.rtc.html).

Она помечена как [`extern "x86-interrupt"`](https://github.com/rust-lang/rust/issues/40180),
поэтому Rust знает что это обработчик прерывания, то есть код который может запуститься в произвольный момент
времени и прервать исполняющийся в этот момент обычный код.
Поэтому он сохраняет все нужные регистры на стек при входе в функцию
[`kernel::trap::pic::rtc()`](../../doc/kernel/trap/pic/fn.rtc.html).
Есть надежда, что компилятор достаточно умён, и сохранит на стек только те регистры,
которые реально используются кодом обработчика прерываний.
А не вообще все регистры процессора, как сделали бы мы, если бы сохраняли регистры вручную.

Обработчик прерываний устроен так:

```rust
extern "x86-interrupt" fn rtc(_context: TrapContext) {
    TRAP_STATS[RTC].inc();
    rtc::interrupt();
}
```

он увеличивает счётчик прерываний от RTC в структуре
[`kernel::trap::TRAP_STATS`](../../doc/kernel/trap/static.TRAP_STATS.html),
которая является массивом элементов типа
[`kernel::trap::Stats`](../../doc/kernel/trap/struct.Stats.html):

```rust
pub struct Stats {
    count: AtomicUsize,
    mnemonic: &'static str,
    name: &'static str,
}
```

И кроме счётчика прерываний
[`Stats::count`](../../doc/kernel/trap/struct.Stats.html#structfield.count),
содержит также имя и короткую мнемонику конкретного прерывания.
Достать значение счётчика из неё можно методом
[`Stats::count()`](../../doc/kernel/trap/struct.Stats.html#method.count).

Основную работу
[`kernel::trap::pic::rtc()`](../../doc/kernel/trap/pic/fn.rtc.html)
перекладывает на
[`kernel::time::rtc::interrupt()`](../../doc/kernel/time/rtc/fn.interrupt.html),
которая определена в файле [`kernel/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/rtc.rs):

```rust
pub(crate) fn interrupt() {
    if interrupt_status().contains(RegisterC::UPDATE_ENDED_INTERRUPT) {
        if let Some(timestamp) = timestamp() {
            let now = CorrelationPoint::now(timestamp * TICKS_PER_SECOND);
            let rtc = SYSTEM_INFO.rtc();
            rtc.init_base(now);
            rtc.store_prev(now);
        }
    }
}
```

Вы уже знакомы почти со всеми кусочками, которые использует этот код.
Краткое напоминание:

- [`timestamp()`](../../doc/kernel/time/rtc/fn.timestamp.html) читает дату и время из микросхемы RTC и переводит их в секунды от начала Unix--эпохи.
- [`CorrelationPoint::now()`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#method.now) добавляет к этому счётчику секунд текущий номер такта процессора.
- `rtc.init_base(now)` сохраняет получившуюся пару `now` в поле [`CorrelationInterval::base`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.base), если оно ещё не было инициализировано. То есть, это происходит только на первом прерывании.
- А `rtc.store_prev(now)` сохраняет пару `now` в поле [`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev) на каждом прерывании.

Таким образом,
[`CorrelationInterval::datetime()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime)
может прочитать эти сохранённые значения и,
если RTC уже дважды тикнул, то оценить текущее время по нему и одновременно по счётчику тактов процессора,
с разрешением выше секунды.
Если же RTC ещё не тикнул дважды, то
[`CorrelationInterval::datetime()`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#method.datetime)
может выдать последнее прочитанное из RTC значение, имеющее разрешение в секунду.
Для того чтобы не ждать значение
[`CorrelationInterval::prev`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html#structfield.prev)
до первого тика RTC, функция
[`kernel::time::rtc::init()`](../../doc/kernel/time/rtc/fn.init.html)
на старте записывает в него значение даты и времени из RTC:

```rust
let rtc = SYSTEM_INFO.rtc();
rtc.store_prev(CorrelationPoint::invalid(timestamp() * TICKS_PER_SECOND));
```

[`CorrelationPoint::invalid()`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#method.invalid) помечает, что этим значением нельзя пользоваться для оценки частоты процессора и текущего времени высокого разрешения.
Так как оно взято в случайный момент между тиками RTC.
А не привязано по времени к определённому тику за счёт вызова из прерывания RTC.


Остаётся понять, что же такое [`SYSTEM_INFO`](../../doc/ku/info/static.SYSTEM_INFO.html).
