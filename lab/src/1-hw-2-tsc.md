## Счётчик тактов процессора

Из всех имеющихся в стандартном компьютере часов, самым большим разрешением
обладает тактовый генератор процессора.
В Nikka работа с ним собрана в модуль
[`ku::time::tsc`](../../doc/ku/time/tsc/index.html)
в файле
[`ku/src/time/tsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/tsc.rs).
Сопоставлять другие источники времени будем с ним.

Функция [`fn ku::time::tsc::tsc() -> i64`](../../doc/ku/time/tsc/fn.tsc.html)
возвращает
[номер текущего такта процессора](https://en.wikipedia.org/wiki/Time_Stamp_Counter)
в некоторый момент времени своего исполнения.
Относительно оптимальным решением для этого было бы использовать инструкцию `RDTSCP`,
позвать которую в Rust можно как
[`core::arch::x86_64::__rdtscp()`](https://doc.rust-lang.org/nightly/core/arch/x86_64/fn.__rdtscp.html).
К сожалению, она доступна только на довольно новых процессорах,
поэтому Nikka использует более старую инструкцию `RDTSC` в сочетании с `LFENCE` через функции
[`core::arch::x86_64::_rdtsc()`](https://doc.rust-lang.org/nightly/core/arch/x86_64/fn._rdtsc.html)
и
[`core::arch::x86_64::_mm_lfence()`](https://doc.rust-lang.org/nightly/core/arch/x86_64/fn._mm_lfence.html).
Кроме того, она рассчитывает, что результат `RDTSC` поместится в
[`i64`](https://doc.rust-lang.org/nightly/core/primitive.i64.html),
что удобно, например, для взятия разности при вычислении интервала времени.

```rust
#[inline(always)]
pub fn tsc() -> i64 {
    unsafe {
        x86_64::_mm_lfence();
        x86_64::_rdtsc().try_into().expect("i64 overflow when storing TSC is expected only after tens of years of uptime")
    }
}
```

В статье
[How to Benchmark Code Execution Times on Intel® IA-32 and IA-64 Instruction Set Architectures](https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/ia-32-ia-64-benchmark-code-execution-paper.pdf)
есть продвинутые рекомендации по использованию инструкций `RDTSC` и `RDTSCP` для измерений интервалов времени.
Кроме того, с использованием инструкций `RDTSC` и `RDTSCP` связаны и другие тонкости, например:

- Инвариантность в зависимости от нагрузки на процессор и его режима энергосбережения. На старых процессорах при изменении его частоты работы меняется и частота счётчика тактов, то есть она неинвариантна.
- Согласованность счётчиков тактов между разными процессорами системы.
- Вообще говоря, прикладные программы не могут рассчитывать на доступность этих инструкций. Так как ядро может запретить их использование в непривилегированном режиме.
- Ядро не спроста может хотеть запретить эти инструкции. Существует множество атак на криптографические алгоритмы, базирующихся на том, что атакующему доступно точное измерение времени работы атакуемого криптографического приложения.

Для упрощения, Nikka игнорирует эти сложности.


### Момент времени [`ku::time::tsc::Tsc`](../../doc/ku/time/tsc/struct.Tsc.html)


Структура [`Tsc`](../../doc/ku/time/tsc/struct.Tsc.html)
описывает момент времени, храня значение счётчика тактов процессора.
Она похожа на стандартную, но недоступную нам в `#[no_std]`--окружении
структуру
[`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html).
Обе описывают монотонно возрастающее время.
То есть время, не ходящее "назад" ни при переводе часов на летнее время, ни при корректировке неточно идущих часов.
Но такое время может никак не соответствовать реальному времени.


### Интервал времени [`ku::time::tsc::TscDuration`](../../doc/ku/time/tsc/struct.TscDuration.html)

Структура [`TscDuration`](../../doc/ku/time/tsc/struct.TscDuration.html) описывает
интервал между двумя моментами времени
[`Tsc`](../../doc/ku/time/tsc/struct.Tsc.html).
Она наивно предполагает, что используемый ею счётчик тактов процессора как минимум инвариантен и согласован между процессорами.

Структура [`TscDuration`](../../doc/ku/time/tsc/struct.TscDuration.html)
может быть напечатана. Например,

```rust
let timer = time::timer();
let frame_allocator = MainFrameAllocator::new(memory_map);
info!(
    frame_allocator = "main",
    free_frame_count = frame_allocator.count(),
    duration = %timer.elapsed(),
    "init",
);
```

пишет в журнал `duration = 322.017 Mtsc`:

```
15:08:28 0 I init; frame_allocator = "main frame allocator"; free_frame_count = 10601; duration = 322.017 Mtsc
```

где Mtsc --- миллионы тактов процессора.
Наша ближайшая задача, ---
научиться выражать продолжительность в гораздо более удобных для человека единицах измерения, привязанных к реальному миру.
