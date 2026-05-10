## Шардированная MPMC


### Задача 4 --- шардированная MPMC

Используя свою реализацию SPSC из
[задачи](../../lab/book/8-queues-2-spsc.html#%D0%97%D0%B0%D0%B4%D0%B0%D1%87%D0%B0-2--spsc-%D0%BE%D1%87%D0%B5%D1%80%D0%B5%D0%B4%D1%8C-%D0%B1%D0%B5%D0%B7-%D0%BE%D0%B6%D0%B8%D0%B4%D0%B0%D0%BD%D0%B8%D1%8F)
в качестве строительного кирпича, реализуйте простую MPMC
в файле
[`ku/src/sync/sharded_mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/sync/sharded_mpmc.rs).
Обратите внимание, что эта MPMC **не является**
[неблокирующей](https://en.wikipedia.org/wiki/Non-blocking_algorithm).

Вам могут помочь:

- "Live Lock-Free or Deadlock (Practical Lock-free Programming)" - Fedor Pikus: [part 1](https://www.youtube.com/watch?v=lVBvHbJsg5Y), [part 2](https://www.youtube.com/watch?v=1obZeHnAwz4).
- [`moodycamel::ConcurrentQueue`](https://github.com/cameron314/concurrentqueue), [обзор](https://web.archive.org/web/20250323171445/https://moodycamel.com/blog/2014/a-fast-general-purpose-lock-free-queue-for-c++.htm).

Или же вы можете пойти своим путём.


### Проверьте себя

Для удобства все проверки корректности и производительности в этой задаче
можно запустить одной командой:

```console
$ cargo run --manifest-path tools/check/Cargo.toml -- --dry-run --no-prerequisites-check --ci-branch-name submit/sharded-mpmc
```

Теперь должны проходить тесты в файле
[`ku/tests/8-queues-4-sharded-mpmc.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/8-queues-4-sharded-mpmc.rs):

```console
$ (cd ku && cargo test --test 8-queues-4-sharded-mpmc)
{{#include 8-queues-4-sharded-mpmc-test.md}}
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/sync/sharded_mpmc.rs | 87 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------
1 file changed, 77 insertions(+), 10 deletions(-)
```
