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
Обработчик прерываний устроен так:

```rust
{{#include ../../kernel/src/trap/pic.rs:rtc}}

{{#include ../../kernel/src/trap/pic.rs:generic_pic_interrupt}}
```

он увеличивает счётчик прерываний от RTC в структуре
[`kernel::trap::TRAP_STATS`](../../doc/kernel/trap/static.TRAP_STATS.html),
которая является массивом элементов типа
[`kernel::trap::Stats`](../../doc/kernel/trap/struct.Stats.html).
И кроме счётчика прерываний
[`Stats::count`](../../doc/kernel/trap/struct.Stats.html#structfield.count),
содержит также имя и короткую мнемонику конкретного прерывания.
Достать значение счётчика из неё можно методом
[`Stats::count()`](../../doc/kernel/trap/struct.Stats.html#method.count).

{{#include ../../doc/kernel/trap/struct.Stats.html.declaration}}

Основную работу
[`kernel::trap::pic::rtc()`](../../doc/kernel/trap/pic/fn.rtc.html)
перекладывает на
[`kernel::time::rtc::interrupt()`](../../doc/kernel/time/rtc/fn.interrupt.html),
которая определена в файле [`kernel/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/rtc.rs):

```rust
{{#include ../../kernel/src/time/rtc.rs:interrupt}}
```

Вы уже знакомы почти со всеми кусочками, которые использует этот код.
Краткое напоминание:

- [`timestamp()`](../../doc/kernel/time/rtc/fn.timestamp.html) читает дату и время из микросхемы RTC и переводит их в секунды от начала Unix--эпохи.
- [`CorrelationPoint::now()`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#method.now) добавляет к этому значению текущий номер такта процессора.
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
{{#include ../../kernel/src/time/rtc.rs:first_correlation_point}}
```

При этом
[`CorrelationPoint::invalid()`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#method.invalid)
помечает, что этим значением нельзя пользоваться для оценки частоты процессора и текущего времени высокого разрешения.
Так как оно взято в случайный момент между тиками RTC.
А не привязано по времени к определённому тику за счёт вызова из прерывания RTC.


### Обработчики прерываний и блокировки

Просто так захватывать блокировки в обработчиках прерываний нельзя.
Представьте себе, что основной код уже захватил блокировку вне прерывания.
И после этого происходит прерывание.
Тогда обработчик прерываний будет ждать освобождения блокировки,
то есть будет ждать возврата в основной код, который должен её освободить.
Но он не завершится, пока не дождётся, а значит и переключения на основной код не произойдёт.
Можно сказать, что основной код тоже ждёт обработчика прерывания.
Получается [взаимоблокировка](https://en.wikipedia.org/wiki/Deadlock).

Чтобы её избежать есть такие варианты:
- Использовать неблокирующую синхронизацию.
- Перед захватом блокировки в основном коде запретить прерывания. А после освобождения блокировки разрешить их, если до запрета они были разрешены. Этот вариант проще, но не так привлекателен --- запрет прерываний ухудшает их латентность. И может приводить, в том числе, даже к потерям прерываний. Ведь если за время запрета возникнет несколько одинаковых прерываний, после их разрешения мы получим уведомление только об одном.


### Реализация [`IrqSpinlock`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlock.html)

В этой задаче вам предлагается сделать вспомогательную структуру
[`ku::sync::irq_spinlock::IrqGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqGuard.html).
Она используется в
[`ku::sync::irq_spinlock::IrqSpinlock`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlock.html).
А именно,
[`IrqSpinlock`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlock.html)
строится из обычного
[`ku::sync::spinlock::Spinlock`](../../doc/ku/sync/spinlock/struct.FastSpinlock.html)
следующим способом.
При захвате в методе
[`IrqSpinlock::lock()`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlock.html#method.lock)
он сначала создаёт
[`IrqGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqGuard.html),
который должен запомнить включены ли прерывания и отключить их.
А потом захватывает базовый
[`Spinlock`](../../doc/ku/sync/spinlock/struct.FastSpinlock.html):
```rust
{{#include ../../ku/src/sync/irq_spinlock.rs:lock}}
```

Аналогичным образом действует и
[`IrqSpinlock::try_lock()`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlock.html#method.try_lock):
```rust
{{#include ../../ku/src/sync/irq_spinlock.rs:try_lock}}
```

В нём дополнительно можно заметить, что благодаря механизму
["получение ресурса есть инициализация"](https://en.wikipedia.org/wiki/Resource_acquisition_is_initialization)
(Resource Acquisition Is Initialization, RAII),
[`IrqGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqGuard.html)
будет разрушен, а значит прерывания будут включены назад,
даже если захватить базовый
[`Spinlock`](../../doc/ku/sync/spinlock/struct.FastSpinlock.html)
не получится.

Возвращает этот метод захваченную блокировку --- специальную структуру
[`ku::sync::irq_spinlock::IrqSpinlockGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlockGuard.html).
При её разрушении вызовется неявной реализации типажа
[`core::ops::Drop`](https://doc.rust-lang.org/nightly/core/ops/trait.Drop.html)
для неё.
Которая сначала разрушит
[`IrqSpinlockGuard::spinlock_guard`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlockGuard.html#structfield.spinlock_guard)
типа
[`ku::sync::spinlock::SpinlockGuard`](../../doc/ku/sync/spinlock/struct.FastSpinlockGuard.html),
так как это поле объявлено в нём первым.
Это действие освободит блокировку.

А после этого разрушит
[`IrqSpinlockGuard::irq_guard`](../../doc/ku/sync/irq_spinlock/struct.IrqSpinlockGuard.html#structfield.irq_guard).
В свою очередь он, в реализации типажа
[`core::ops::Drop`](https://doc.rust-lang.org/nightly/core/ops/trait.Drop.html),
которую вы напишете,
должен разрешить прерывания, если они были разрешены на момент его создания.

Реализуйте
[`IrqGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqGuard.html),
определённый в файле
[`ku/src/sync/irq_spinlock.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/irq_spinlock.rs).
Вам могут пригодиться высокоуровневые функции для работы с прерываниями, определённые в модуле
[`x86_64::instructions::interrupts`](../../doc/x86_64/instructions/interrupts/index.html)
используемой нами внешней библиотеки
[`x86_64`](../../doc/x86_64/index.html)
для низкоуровневой работы с архитектурой [x86-64](https://en.wikipedia.org/wiki/X86-64).

Наш алгоритм для
[`IrqGuard`](../../doc/ku/sync/irq_spinlock/struct.IrqGuard.html) ---
несовершенный.
Подумайте, какие у него есть проблемы.


### Запуск тестов

Тесты можно запустить командой `cargo test --test 1-hw-4-interrupts` в директории `kernel` репозитория.

```console
/.../nikka$ (cd kernel; cargo test --test 1-hw-4-interrupts)
running 4 tests

1_hw_4_interrupts::failed_try_lock--------------------------
14:56:45 0 D start; interrupts_enabled = true
14:56:45 0 D locked; interrupts_enabled = false
14:56:45 0 D enabled interrupts; interrupts_enabled = true
14:56:45 0 D try_lock() failed; interrupts_enabled = true
14:56:45 0 D unlocked; interrupts_enabled = true
1_hw_4_interrupts::failed_try_lock----------------- [passed]

1_hw_4_interrupts::interrupts_disabled----------------------
14:56:45 0 D start; interrupts_enabled = false
14:56:45 0 D locked; interrupts_enabled = false
14:56:45 0 D unlocked; interrupts_enabled = false
14:56:45 0 D try_lock() succeeded; interrupts_enabled = false
14:56:45 0 D unlocked; interrupts_enabled = false
1_hw_4_interrupts::interrupts_disabled------------- [passed]

1_hw_4_interrupts::interrupts_enabled-----------------------
14:56:45 0 D start; interrupts_enabled = true
14:56:45 0 D locked; interrupts_enabled = false
14:56:45 0 D unlocked; interrupts_enabled = true
14:56:45 0 D try_lock() locked; interrupts_enabled = false
14:56:45 0 D try_lock() succeeded; interrupts_enabled = false
14:56:45 0 D unlocked; interrupts_enabled = true
1_hw_4_interrupts::interrupts_enabled-------------- [passed]

1_hw_4_interrupts::nested-----------------------------------
14:56:45 0 D start; interrupts_enabled = true
14:56:45 0 D locked outer; interrupts_enabled = false
14:56:45 0 D locked inner; interrupts_enabled = false
14:56:45 0 D unlocked inner; interrupts_enabled = false
14:56:45 0 D unlocked outer; interrupts_enabled = true
1_hw_4_interrupts::nested-------------------------- [passed]
14:56:45 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/irq_spinlock.rs | 20 ++++++++++++++++----
1 file changed, 16 insertions(+), 4 deletions(-)
```
