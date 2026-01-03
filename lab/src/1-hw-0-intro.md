## Время и прерывания

В этой лабораторке нужно будет реализовать работу со временем в операционной системе Nikka.
Лабораторка вводная, предназначена для ознакомления с системой и предварительной оценки своих сил.
Кода потребуется написать немного.
Кроме того, этот код не будет необходим для выполнения последующих лабораторок.
Все задачи в этот раз необязательные и независимы между собой.
Но, если вы их выполните, в дальнейшем вам будет доступно системное время.
Что как минимум приятно при изучении сообщений журнала.

В этом описании вы встретите много разных специфичных вещей.
[Прерывания](https://en.wikipedia.org/wiki/Interrupt),
[`RDTSC`](https://en.wikipedia.org/wiki/Time_Stamp_Counter),
[атомарные переменные](https://doc.rust-lang.org/nightly/core/sync/atomic/index.html),
спецификации аппаратного обеспечения, и т.д.
Не пугайтесь, предварительных знаний не должно потребоваться.
Они упоминаются на тот случай,
если вы уже хорошо знаете базовую часть материала или же захотите углубиться в какую-либо из таких областей.
Это может оказаться очень увлекательно!
Большие блоки интересного, но не обязательного материала выделены цветом, точно так же как цитаты:

> Time is an illusion. Lunchtime doubly so.
> - Douglas Adams, The Hitchhiker's Guide to the Galaxy

В современном компьютере есть несколько источников времени с разными характеристиками.
Мы познакомимся с парой из них и сделаем комбинированное по их характеристикам [системное время](https://en.wikipedia.org/wiki/System_time).
А именно, из часов, показывающих время в реальном мире и имеющих низкое --- секундное --- разрешение,
и источника времени с высоким разрешением, но не привязанного ко времени в реальном мире,
построим часы, показывающие время в реальном мире с высоким разрешением.


### Код работы со временем в Nikka собран в модули

- [`kernel::time`](../../doc/kernel/time/index.html) в директории `kernel/src/time`. Здесь находится часть работы со временем, которая происходит только в ядре.
  - [`kernel/src/time/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/mod.rs) --- корневая часть модуля [`kernel::time`](../../doc/kernel/time/index.html).
  - [`kernel/src/time/pit8254.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/pit8254.rs) --- драйвер устаревшего таймера [Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253) ([programmable interval timer, PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)). Не представляет большого интереса, так как мы будем использовать более современный [таймер в APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
  - [`kernel/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/time/rtc.rs) --- драйвер [часов реального времени](https://en.wikipedia.org/wiki/Real-time_clock). Они обычно условно независимы по питанию, так как снабжены [батарейкой](https://en.wikipedia.org/wiki/Nonvolatile_BIOS_memory#CMOS_battery). Отслеживают дату и время в реальном мире с точностью до секунды.

- [`ku::time`](../../doc/ku/time/index.html) в директории `ku/src/time`. Здесь собраны базовые примитивы для работы со временем, которые нужны и в ядре, и в пространстве пользователя.
  - [`ku/src/time/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/mod.rs) --- корневая часть модуля [`ku::time`](../../doc/ku/time/index.html). Содержит:
    - Функцию [`ku::time::timer()`](../../doc/ku/time/fn.timer.html) для получения монотонного процессорного времени, которое измеряется его тактами.
    - Функции [`ku::time::datetime()`](../../doc/ku/time/fn.datetime.html) и [`ku::time::datetime_ms()`](../../doc/ku/time/fn.datetime_ms.html), которые переводят значение счётчика тактов процессора в системное время с разрешением в наносекунды и миллисекунды соответственно.
    - Функции [`ku::time::now()`](../../doc/ku/time/fn.now.html) и [`ku::time::now_ms()`](../../doc/ku/time/fn.now_ms.html), которые сообщают системное время в текущий момент с разрешением в наносекунды и миллисекунды соответственно.
  - [`ku/src/time/correlation_point.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/correlation_point.rs) --- вспомогательные структуры [`ku::time::correlation_point::AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html) и [`ku::time::correlation_point::CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) для привязки тактов процессора к другому источнику времени, в один момент времени.
  - [`ku/src/time/correlation_interval.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/correlation_interval.rs) --- вспомогательные структуры [`ku::time::correlation_interval::AtomicCorrelationInterval`](../../doc/ku/time/correlation_interval/struct.AtomicCorrelationInterval.html) и [`ku::time::correlation_interval::CorrelationInterval`](../../doc/ku/time/correlation_interval/struct.CorrelationInterval.html) для соотнесения частоты процессора с частотой другого источника времени по двум моментам времени.
  - [`ku/src/time/hz.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/hz.rs) --- вспомогательная структура [`ku::time::hz::Hz`](../../doc/ku/time/hz/struct.Hz.html) для форматирования [частоты](https://en.wikipedia.org/wiki/Hertz) при журналировании.
  - [`ku/src/time/rtc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/rtc.rs) --- [синглтон](https://en.wikipedia.org/wiki/Singleton_pattern) [`ku::time::rtc::Rtc`](../../doc/ku/time/rtc/struct.Rtc.html), позволяющий узнать показания часов реального времени как из ядра, так и из [пространства пользователя](https://en.wikipedia.org/wiki/User_space_and_kernel_space).
  - [`ku/src/time/tsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/tsc.rs) --- структура [`ku::time::tsc::Tsc`](../../doc/ku/time/tsc/struct.Tsc.html) для хранения показаний счётчика тактов процессора, который является одним из источников времени в компьютере. А также структура [`ku::time::tsc::TscDuration`](../../doc/ku/time/tsc/struct.TscDuration.html) для хранения интервалов времени в тактах процессора.


### Ориентировочный объём работ этой лабораторки

```console
kernel/src/time/rtc.rs              |   70 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
ku/src/sync/irq_spinlock.rs         |   20 ++++++++++++++++----
ku/src/time/correlation_interval.rs |   30 ++++++++++++++++++++++++++++--
3 files changed, 106 insertions(+), 14 deletions(-)
```
