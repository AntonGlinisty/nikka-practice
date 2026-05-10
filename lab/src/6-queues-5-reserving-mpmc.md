## MPMC с резервированием элементов


### Задача 5 --- MPMC с резервированием элементов

Теперь реализуйте более сложную MPMC
в файле
[`ku/src/sync/reserving_mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/reserving_mpmc.rs).

Её идея в том, что каждый из потоков атомарной операцией над хвостом или головой очереди резервирует
элемент циклического буфера.
И дополнительно, в каждом элементе есть ещё одно атомарное поле для координации между
производителем и потребителем и для обнаружения особых случаев --- пустая или полная очередь.
Обратите внимание, что эта MPMC **не является**
[неблокирующей](https://en.wikipedia.org/wiki/Non-blocking_algorithm),
как и очередь из предыдущей задачи.
Вам может помочь статья
[Bounded MPMC queue](https://web.archive.org/web/20250810034104/https://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue).
Или же вы можете пойти своим путём.


### Сравнение производительности MPMC с резервированием элементов и SPSC очередей

Время передачи миллиона элементов:

![Время передачи миллиона элементов](8-queues-5-reserving-mpmc-time.svg)

Пропускная способность MPMC очереди с резервированием элементов
на порядок меньше пропускной способности SPSC очереди:

![Пропускная способность очереди](8-queues-5-reserving-mpmc-throughput.svg)

[Круговая задержка (round-trip time)](https://en.wikipedia.org/wiki/Round-trip_delay) очереди:

![Круговая задержка очереди](8-queues-5-reserving-mpmc-roundtrip.svg)


### Проверьте себя

Для удобства все проверки корректности и производительности в этой задаче
можно запустить одной командой:

```console
$ cargo run --manifest-path tools/check/Cargo.toml -- --dry-run --no-prerequisites-check --ci-branch-name submit/reserving-mpmc
```

Теперь должны проходить тесты в файле
[`ku/tests/8-queues-5-reserving-mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/8-queues-5-reserving-mpmc.rs):

```console
$ (cd ku && cargo test --test 8-queues-5-reserving-mpmc)
{{#include 8-queues-5-reserving-mpmc-test.md}}
```

```console
$ (cd ku && cargo bench --bench 8-queues-5-reserving-mpmc)
{{#include 8-queues-5-reserving-mpmc-bench.md}}
```

```console
$ critcmp --filter 'reserving_mpmc/' new
{{#include 8-queues-5-reserving-mpmc-critcmp.md}}
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/reserving_mpmc.rs | 154 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++---------
1 file changed, 138 insertions(+), 16 deletions(-)
```
