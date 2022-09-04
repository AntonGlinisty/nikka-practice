## Собирая всё вместе

В файле [`kernel/src/main.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/main.rs) определена функция `kernel_main()`.
Вы можете использовать её по своему усмотрению, например проверяя какую-нибудь функциональность.

Например:

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);

    loop {
        if let Some(frequency) = Pit::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by PIT");
        }

        if let Some(frequency) = Rtc::tsc_per_second() {
            debug!(%frequency, "CPU frequency measured by RTC");
        }

        debug!(time_precision = %time::timer().lap());

        for (number, stats) in TRAP_STATS.iter().enumerate() {
            let count = stats.count();
            if count != 0 {
                let mnemonic = stats.mnemonic;
                debug!(number, %mnemonic, count, "trap stats");
            }
        }

        x86_64::instructions::hlt();
    }
}
```

Сначала выполняется инициализация системы функцией
[`kernel::init()`](../../doc/kernel/fn.init.html)
из файла [`kernel/src/lib.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/lib.rs).
А затем мы в цикле выводим в журнал статистику счётчиков прерываний и
вычисленную с помощью RTC и PIT частоту процессора.
Инструкция `HLT`, которую мы используем через функцию
[`x86_64::instructions::hlt()`](../../doc/x86_64/instructions/fn.hlt.html),
выключает процессор до прихода следующего прерывания.
Реализация PIT устроена также, как и изученная вами реализация RTC.
Только он тикает 20 раз в секунду и не привязан к реальному времени.

Запускаем:

```console
$ make run
```

В файле `kernel/serial.out` появляется журнал, время должно соответствовать команде `date -u`:
```console
13:31:48 0 I RTC init; acknowledged_settings = USE_24_HOUR_FORMAT | USE_BINARY_FORMAT | UPDATE_ENDED_INTERRUPT
13:31:48 0 I time init
13:31:48 0 I Nikka booted; now = 2022-09-04 13:31:48 UTC; tsc = Tsc(5913674560)
13:31:48 0 I GDT init
13:31:48 0 I traps init
...
```
Видно, что RTC инициализировался, и выдаёт разумное время --- `now = 2022-09-04 13:31:48 UTC`.
А время в журнале пока имеет разрешение в секунду.
Дальше продолжается инициализация, после чего запускается наш цикл сбора статистик прерываний:
```
13:31:49 0 D CPU frequency measured by PIT; frequency = 3.742 GHz
13:31:49 0 D time_precision = 61.685 ktsc
13:31:49 0 D trap stats; number = 32; mnemonic = #TI; count = 11
13:31:49 0 D trap stats; number = 40; mnemonic = #RT; count = 1
13:31:49 0 D trap stats; number = 48; mnemonic = #AT; count = 7
```
PIT уже тикнул 11 раз --- `trap stats; number = 32; mnemonic = #TI; count = 11`, поэтому он уже может оценить частоту процессора --- `CPU frequency measured by PIT; frequency = 3.742 GHz`.
RTC тикнул пока только один раз --- `trap stats; number = 40; mnemonic = #RT; count = 1`, ждём дальше.

RTC тикает во второй раз, и на основе его показаний уже можно оценить частоту процессора.
Также увеличивается разрешение системного времени в сообщениях журнала:
```
13:31:50.001 0 D CPU frequency measured by PIT; frequency = 3.591 GHz
13:31:50.029 0 D CPU frequency measured by RTC; frequency = 3.420 GHz
13:31:50.033 0 D time_precision = 18.033 us
13:31:50.040 0 D trap stats; number = 32; mnemonic = #TI; count = 24
13:31:50.045 0 D trap stats; number = 40; mnemonic = #RT; count = 2
13:31:50.050 0 D trap stats; number = 48; mnemonic = #AT; count = 31
```
И вычисление интервалов времени начинает опираться на секунды --- `time_precision = 18.033 us`.
То есть, между последовательными обращениями к `RDTSC` проходит около 18 микросекунд.
Такая низкая точность объясняется отладочной сборкой без оптимизаций.

Если же собрать релизную, то получим более разумную цифру.
Которая, правда, всё ещё далека от идеала:
```console
$ cd kernel; cargo run --release
...
14:36:08.714 0 D time_precision = 220.695 ns
```
