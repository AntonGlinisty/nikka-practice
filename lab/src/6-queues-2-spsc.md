## Ограниченная SPSC очередь без ожидания


### Задача 2 --- SPSC очередь без ожидания

Опираясь на свою реализацию однопоточной двухсторонней очереди из
[предыдущей задачи](../../lab/book/8-queues-1-deque.html#%D0%97%D0%B0%D0%B4%D0%B0%D1%87%D0%B0-1--%D1%8D%D1%84%D1%84%D0%B5%D0%BA%D1%82%D0%B8%D0%B2%D0%BD%D0%B0%D1%8F-%D0%B4%D0%B2%D1%83%D1%85%D1%81%D1%82%D0%BE%D1%80%D0%BE%D0%BD%D0%BD%D1%8F%D1%8F-%D0%BE%D1%87%D0%B5%D1%80%D0%B5%D0%B4%D1%8C)
[`ku::collections::BoundedDeque()`](../../doc/ku/collections/bounded_deque/struct.BoundedDeque.html)
реализуйте ограниченную конкурентная Single--Producer--Single--Consumer (SPSC) очередь
[без ожидания](https://en.wikipedia.org/wiki/Non-blocking_algorithm#Wait-freedom)
в файле
[`ku/src/sync/spsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/spsc.rs).

Вам может помочь доклад
[Single Producer Single Consumer Lock-free FIFO From the Ground Up - Charles Frasch](https://www.youtube.com/watch?v=K3P_Lmq6pw0).


### Ссылки

- ["Cooperating sequential processes, section 4.3. The Bounded Buffer", Dijkstra, 1965](https://www.cs.utexas.edu/~EWD/transcriptions/EWD01xx/EWD123.html#4.3.%20The%20Bounded%20Buffer.).
- ["Proving the Correctness of Multiprocess Programs", Leslie Lamport, 1977](https://lamport.azurewebsites.net/pubs/proving.pdf).
- ["Correct and Efficient Bounded FIFO Queues", Nhat Minh Lê & Adrien Guatto & Albert Cohen & Antoniu Pop, 2013](https://www.irif.fr/~guatto/publications/sbac13.pdf).
- [Single Producer Single Consumer Lock-free FIFO From the Ground Up - Charles Frasch](https://www.youtube.com/watch?v=K3P_Lmq6pw0).
- ["Optimizing a ring buffer for throughput", Erik Rigtorp, 2021](https://rigtorp.se/ringbuffer/). [Код SPSC очереди](https://github.com/rigtorp/SPSCQueue).
- [Folly's SPSC](https://github.com/facebook/folly/blob/main/folly/ProducerConsumerQueue.h).
- [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html).
- [`heapless::spsc`](https://docs.rs/heapless/latest/heapless/spsc/index.html).
- [`fibre::spsc`](https://docs.rs/fibre/latest/fibre/spsc/index.html).
- [Проблема "производитель--потребитель"](https://en.wikipedia.org/wiki/Producer%E2%80%93consumer_problem).


### Сравнение производительности SPSC и однопоточной очереди

Время передачи тысячи элементов в однопоточном режиме:

![Время передачи тысячи элементов](8-queues-2-sequential-time.svg)

Пропускная способность очереди в однопоточном режиме:

![Пропускная способность очереди](8-queues-2-sequential-throughput.svg)


### Сравнение производительности различных реализаций конкурентных очередей

Время передачи миллиона элементов:

![Время передачи миллиона элементов](8-queues-2-concurrent-time.svg)

Пропускная способность очереди:

![Пропускная способность очереди](8-queues-2-concurrent-throughput.svg)

[Круговая задержка (round-trip time)](https://en.wikipedia.org/wiki/Round-trip_delay) очереди:

![Круговая задержка очереди](8-queues-2-roundtrip.svg)


### Что будет, если использовать ~~`SeqCst`~~ в реализации

Если использовать ~~`SeqCst`~~ в реализации пропускная способность очереди упадёт на порядок:

![Если использовать ~~`SeqCst`~~ в реализации пропускная способность очереди упадёт на порядок](8-queues-2-concurrent-throughput-seqcst.svg)


#### Одна из известных реализаций

В
[реализации Эрика Ригторпа](https://github.com/rigtorp/SPSCQueue):

- Для циклического буфера выбраны инварианты `sentinel`.
- Применяется кэширование головы и хвоста SPSC очереди.
- Ёмкость выбирается в момент запуска, а не компиляции, --- ёмкость не является константным параметром шаблона. Но при этом используется не операция деления, а оборачивание индекса (wrap around) вручную ---
    ```cpp
    if (nextWriteIdx == capacity_) {
      nextWriteIdx = 0;
    }
    ```
- В тесте 10 миллионов `push()/pop()` в очередь на 10 миллионов элементов:
    ```cpp
    const size_t queueSize = 10000000;
    const int64_t iters = 10000000;
    ```

На том же тестовом компьютере получается близкая производительность ---
305 миллионов `push()/pop()` в секунду (в двух потоках, в очереди 32-х битные элементы):

```console
$ git clone https://github.com/rigtorp/SPSCQueue
...
$ cd SPSCQueue/
$ cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build
...
$ build/SPSCQueueBenchmark
SPSCQueue:
304580 ops/ms
96 ns RTT
```

Статистики [15 запусков](https://www.wolframalpha.com/input?i=%5B311175%2C306442%2C295490%2C309897%2C309375%2C305092%2C311316%2C298560%2C309779%2C317066%2C278723%2C280995%2C311428%2C315672%2C312400%2C281377%2C314877%2C280160%2C315192%2C280220%2C306539%2C281291%2C314495%2C312797%2C277669%5D):

Статистика | Значение
-----------|---------
mean | 301.6 MElem/sec
median | 309.4 MElem/sec
standard deviation | 0.015 MElem/sec


### Мы предусмотрели интерфейс для обмена сразу несколькими элементами

В нём --- `tx spsc` --- пропускная способность очереди значительно вырастает:

![Пропускная способность очереди](8-queues-2-concurrent-throughput-tx.svg)


### Проверьте себя

Для удобства все проверки корректности и производительности в этой задаче
можно запустить одной командой:

```console
$ cargo run --manifest-path tools/check/Cargo.toml -- --dry-run --no-prerequisites-check --ci-branch-name submit/spsc
```

Теперь не только тесты в файле
[`ku/tests/8-queues-2-spsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/8-queues-2-spsc.rs)
должны проходить:

```console
$ (cd ku && cargo test --test 8-queues-2-spsc)
{{#include 8-queues-2-spsc-test.md}}
```

Но и по производительности ваша SPSC очередь должна:

- В однопоточном поэлементном режиме быть не более чем на 25% медленнее однопоточной очереди `sentinel`.
- В многопоточном поэлементном режиме быть не более чем на 25% медленнее однопоточной очереди `sentinel`.
- В многопоточном транзакционном режиме обгонять не менее чем на 25% однопоточную очередь `sentinel`.

Проверить это можно запустив тест производительности из файла
[`ku/benches/8-queues-2-spsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/benches/8-queues-2-spsc.rs)
командами:

```console
$ (cd ku && cargo bench --bench 8-queues-2-spsc sequential)
{{#include 8-queues-2-sequential-bench.md}}
```

```console
$ (cd ku && cargo bench --bench 8-queues-2-spsc concurrent)
{{#include 8-queues-2-concurrent-bench.md}}
```

```console
$ critcmp --filter 'sequential/.*(spsc|sentinel)' new
{{#include 8-queues-2-sequential-critcmp.md}}
```

```console
$ critcmp --filter 'concurrent/1.*spsc' new
{{#include 8-queues-2-concurrent-critcmp.md}}
```

Графики производительности, в том числе распределения времени исполнения в любом из тестов,
можно посмотреть по ссылкам в файле `target/criterion/report/index.html`.
Если хотите смотреть на графики, которые строит `criterion`,
рекомендуется поставить `gnuplot` перед запуском `cargo bench`.
Указать использовать именно его можно дополнительной опцией:
`cargo bench ... -- --plotting-backend gnuplot`.
Но это не должно требоваться,
так как `criterion` по умолчанию старается по возможности использовать `gnuplot`.

Если вы хотите сравнить свою реализацию очереди с другими,
задайте опцию `--features benchmark`:

```console
$ (cd ku && cargo bench --features benchmark --bench 8-queues-2-spsc concurrent)
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/spsc.rs | 368 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------
1 file changed, 329 insertions(+), 39 deletions(-)
```
