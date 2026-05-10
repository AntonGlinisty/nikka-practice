## Интерфейс обмена срезами для SPSC


### Задача 3 --- интерфейс обмена срезами для SPSC

Добавьте в свою реализацию SPSC из
[предыдущей задачи](../../lab/book/8-queues-2-spsc.html#%D0%97%D0%B0%D0%B4%D0%B0%D1%87%D0%B0-2--spsc-%D0%BE%D1%87%D0%B5%D1%80%D0%B5%D0%B4%D1%8C-%D0%B1%D0%B5%D0%B7-%D0%BE%D0%B6%D0%B8%D0%B4%D0%B0%D0%BD%D0%B8%D1%8F)
возможность записывать и читать из очереди целыми срезами за одну операцию.
Для этого вам нужно будет реализовать методы

- [`SpscSide::enqueue_slice()`](../../doc/ku/sync/spsc/struct.SpscSide.html#method.enqueue_slice),
- [`SpscTx::enqueue_slice()`](../../doc/ku/sync/spsc/struct.SpscTx.html#method.enqueue_slice),
- [`SpscSide::dequeue_slice()`](../../doc/ku/sync/spsc/struct.SpscSide.html#method.dequeue_slice) и
- [`SpscTx::dequeue_slice()`](../../doc/ku/sync/spsc/struct.SpscTx.html#method.dequeue_slice)

в файле
[`ku/src/sync/spsc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/spsc.rs).


### Проверьте себя

Для удобства все проверки корректности и производительности в этой задаче
можно запустить одной командой:

```console
$ cargo run --manifest-path tools/check/Cargo.toml -- --dry-run --no-prerequisites-check --ci-branch-name submit/spsc-slice
```

Теперь не только тесты в файле
[`ku/tests/8-queues-3-spsc-slice.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/8-queues-3-spsc-slice.rs)
должны проходить:

```console
$ (cd ku && cargo test --test 8-queues-3-spsc-slice)
{{#include 8-queues-3-spsc-slice-test.md}}
```

Но и по производительности в режиме обмена срезами
ваша SPSC очередь должна быть не более чем в два раза медленнее чем
двойное копирование памяти --- `double memory copy`.

Проверить это можно запустив тест производительности из файла
[`ku/benches/8-queues-3-spsc-slice.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/benches/8-queues-3-spsc-slice.rs)
командами:

```console
$ (cd ku && cargo bench --bench 8-queues-3-spsc-slice)
{{#include 8-queues-3-spsc-slice-bench.md}}
```

```console
$ critcmp --filter 'slice/.*(slice spsc|double)' new
{{#include 8-queues-3-spsc-slice-critcmp.md}}
```

Пропускная способность очереди при обмене срезами:

![Пропускная способность очереди при обмене срезами](8-queues-3-spsc-slice-throughput.svg)


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/spsc.rs | 140 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------
1 file changed, 126 insertions(+), 14 deletions(-)
```
