## Конкурентные очереди

Целью этого цикла задач является реализация ограниченных
[неблокирующих](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
конкурентных очередей для взаимодействия разных потоков исполнения.


### Зависимости задач этой лабораторной работы

![Зависимости задач этой лабораторной работы](8-queues-0-intro-lab-dependencies.svg)


### Ориентировочный объём работ этой лабораторки

```console
ku/src/collections/bounded_deque.rs |  52 ++++---
ku/src/sync/mpmc.rs                 | 111 ++++++++++++++--
ku/src/sync/reserving_mpmc.rs       | 154 ++++++++++++++++++++---
ku/src/sync/sharded_mpmc.rs         |  87 +++++++++++--
ku/src/sync/spsc.rs                 | 508 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
5 files changed, 811 insertions(+), 101 deletions(-)
```
