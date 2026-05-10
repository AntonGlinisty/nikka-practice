## Не блокирующая MPMC очередь


### Задача 6 --- Не блокирующая MPMC очередь

Наконец, реализуйте
[не блокирующую](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
MPMC
в файле
[`ku/src/sync/mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/mpmc.rs).
Она немного похожа на очередь из предыдущей задачи.
Но накладывает ограничение на тип элементов --- его должно быть можно сериализовать в `u32`.
Вам может помочь доклад
[User API & C++ Implementation of a Multi Producer, Multi Consumer, Lock Free, Atomic Queue](https://www.youtube.com/watch?v=bjz_bMNNWRk).
Или же вы можете пойти своим путём.


### Сравнение производительности не блокирующих MPMC и SPSC очередей

Время передачи миллиона элементов:

![Время передачи миллиона элементов](8-queues-6-mpmc-time.svg)

Пропускная способность не блокирующей MPMC очереди
на порядок меньше пропускной способности SPSC очереди:

![Пропускная способность очереди](8-queues-6-mpmc-throughput.svg)

[Круговая задержка (round-trip time)](https://en.wikipedia.org/wiki/Round-trip_delay) очереди:

![Круговая задержка очереди](8-queues-6-mpmc-roundtrip.svg)


### Проверьте себя

Для удобства все проверки корректности и производительности в этой задаче
можно запустить одной командой:

```console
$ cargo run --manifest-path tools/check/Cargo.toml -- --dry-run --no-prerequisites-check --ci-branch-name submit/mpmc
```

Теперь должны проходить тесты в файле
[`ku/tests/8-queues-6-mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/8-queues-6-mpmc.rs):

```console
$ (cd ku && cargo test --test 8-queues-6-mpmc)
{{#include 8-queues-6-mpmc-test.md}}
```

```console
$ (cd ku && cargo bench --bench 8-queues-6-mpmc)
{{#include 8-queues-6-mpmc-bench.md}}
```

```console
$ critcmp --filter 'mpmc/' new
{{#include 8-queues-6-mpmc-critcmp.md}}
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/mpmc.rs | 111 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++------
1 file changed, 104 insertions(+), 7 deletions(-)
```
