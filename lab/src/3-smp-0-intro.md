## Конкурентное выполнение задач

В этой лабораторке нужно будет реализовать конкурентное исполнение на уровне железа, то есть на разных процессорах одновременно.
А также, конкурентное исполнение пользовательских процессов на одном процессоре за счёт разделения его времени работы между ними.

### Ориентировочный объём работ этой лабораторки

```console
 kernel/src/memory/address_space.rs |   22 ++++++
 kernel/src/process/process.rs      |   24 ++++++-
 kernel/src/process/scheduler.rs    |   26 ++++++--
 kernel/src/process/syscall.rs      |   13 +++-
 kernel/src/process/table.rs        |  119 ++++++++++++++++++++++++++++++++++---
 kernel/src/smp/ap_init.rs          |   55 ++++++++++++++++-
 kernel/src/smp/cpu.rs              |   54 ++++++++++++++--
 kernel/src/smp/local_apic.rs       |   19 +++++
 ku/src/allocator/big.rs            |   87 +++++++++++++++++++++++----
 9 files changed, 374 insertions(+), 45 deletions(-)
```
