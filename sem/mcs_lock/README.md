# Mellor-Crummey and Scott Lock (MCS Lock)

В этой задаче вам предстоит реализовать масштабируемую честную блокировку,
которая явно поддерживает ждущие потоки в интрузивной очереди.

## Мотивация

Рассмотрим честную блокировку – [ticket lock](https://en.wikipedia.org/wiki/Ticket_lock).
Одним из её недостатков является неоптимальная работа с кэшем.
А именно, когда блокировка освобождается,
линии кэша, содержащие `next_ticket`, инвалидируются у всех ожидающих потоков.
Несмотря на то, что блокировка впоследствии захватывается только одним из ожидающих
потоков. Иными словами, если на ticket lock'е ждут $n$ потоков, то при одном освобождении
и захвате блокировки происходит $O(n)$ инвалидаций линий кэша.
Что может ухудшать масштабируемость блокировки с ростом количества потоков.
Эта проблема известна как [thundering herd problem](https://en.wikipedia.org/wiki/Thundering_herd_problem).

Выливается это в то, что при добавлении единиц ядер в систему с большим их количеством,
производительность операций, использующих
[ticket lock](https://en.wikipedia.org/wiki/Ticket_lock), отвесно падает.
Иногда даже до значений меньших, чем в системе с одним ядром.
Хорошая иллюстрация этого есть в статье
[Non-scalable locks are dangerous](https://pdos.csail.mit.edu/papers/linux:lock.pdf)
в виде графиков "Figure 2: Sudden performance collapse with ticket locks."

## Предлагаемое решение

Описанная проблема возникает из-за того, что поле `next_ticket` общее у всех ожидающих
потоков. Предлагается сделать его уникальным для каждого из ожидающих потоков.
А именно:

- Каждому ожидающему потоку будет соответствовать узел в односвязном списке.
- Головой списка является поток, владеющий блокировкой.
- У каждого узла списка будет свой флаг `owner`, который будет выставляться предыдущим
   потоком в списке при выходе из критической секции.

При попытке захватить блокировку, если она уже захвачена другим потоком,
текущий поток должен добавить себя в конец очереди ожидания.
И дождаться, пока очередь не дойдет до него, –
пока в его узле не будет установлен флаг `owner`.

<details>
<summary>
<h3>Детали реализации</h3>
</summary>

При реализации этого алгоритма вы скорее всего будете поддерживать
указатель на хвост односвязного списка.
И при вставке узла в конец возникнет проблема – нужно будет одновременно
изменить поле `next` у предыдущего узла и записать свой узел как новый хвост.
Вместо этого предлагается сначала записать свой узел как хвост, с помощью операции
[`swap`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicPtr.html#method.swap).
А затем записать себя в качестве `next` в предыдущем хвосте списка.
Тогда при отпускании блокировки, если узел, соответствующий текущему потоку,
не является хвостом списка, нужно дождаться,
пока следующий поток не запишет свой узел в `next` прежде чем обновлять `(*next).owner`.

Также при отпускании блокировки вам скорее всего пригодится метод
[`compare_exchange`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicPtr.html#method.compare_exchange),
изучите его документацию.

</details>

## Интерфейс в Rust

Так как мы хотим предоставить интерфейс похожий на стандартный
[`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock),
нам нужно понять, где выделить узел односвязного списка.
Сделать его частью возвращаемого guard'а нельзя, так как guard
может перемещаться в памяти в то время, как логика блокировки полагается
на неизменность адреса этого узла.

Самым простым вариантом было бы выделить его динамически
и передать владение над ним в `McsLockGuard`.
Но в таком случае для каждого захвата блокировки придётся совершать
относительно медленное действие --- динамическое выделение памяти.
Вместо этого, мы попросим пользователя выделить и предоставить узел самостоятельно.
У этого подхода есть сложность, которая заключается в том, что пользователю нужно
запретить перемещать и использовать переданный узел для захвата других блокировок.
Для этого сделаем сигнатуру `lock` следующей:

```rs
fn lock<'a>(&'a self, node: &'a mut McsLockNode) -> McsLockGuard<'a, T> {
    // ...
}
```

Обратите внимание на то, что эксклюзивная ссылка `&mut McsLockNode` имеет lifetime
`'a`, который также фигурирует в возвращаемом guard.
Это создает mutable borrow на `node` на все время жизни возвращаемого guard,
запрещая делать какие-либо операции с `node` пока жив guard.

Для использования блокировки можно выделить узел списка на стеке:

```rs
let mut node = McsLockNode::new();

let mut guard = lock.lock(&mut node);
```

## Запуск тестов

При запуске тестов указывайте параметр `--test-threads=1`,
чтобы тесты запускались по одному и не мешали друг другу:

```bash
cargo test -- --test-threads=1
```

## Потенциальные проблемы с miri

При запуске под miri, у вас могут возникать проблемы с нарушением правил
_stacked borrows_:

```plain
error: Undefined Behavior: trying to retag from <474372> for SharedReadWrite permission at alloc111737[0x0], but that tag does not exist in the borrow stack for this location
   |
91 |         unsafe { &*last }
   |                  ^^^^^^
   |                  |
   |                  trying to retag from <474372> for SharedReadWrite permission at alloc111737[0x0], but that tag does not exist in the borrow stack for this location
   |                  this error occurs as part of retag at alloc111737[0x0..0x10]
   |
   = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
   = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for further information
```

Это может происходить, если вы с помощью `unsafe` создали конфликтующие ссылки на
один и тот же объект (две ссылки, из которых хотя бы одна – эксклюзивная), потенциально
в разных потоках.

Проще говоря, создавая эксклюзивную ссылку на объект, удостоверьтесь, что в то же
время на этот объект не ссылается ни одна другая ссылка.

## Замеры производительности

Для оценки пропускной способности вашей реализации MCS lock'а,
сравните его с реализацией ticket lock'а и spinlock'а из библиотеки
[`spin`](https://docs.rs/spin/latest/spin/).
Для этого запустите тест производительности: `cargo bench`.
Обратите внимание на то, что он требует наличия как минимум 16 ядер.
Если ваша система имеет меньше ядер, уменьшите количество потоков в тесте производительности.
Убедитесь, что ваша реализация MCS-lock работает быстрее,
чем ticket lock при большом количестве потоков.
Так же посмотрите на счётчики производительности, связанные с протоколом когерентности
кэшей.
Для этого соберите workload:

```bash
cargo build --release --example workload
```

И запустите его с `perf stat`:

```bash
sudo perf stat -C 0-9 -e <counters> -- ../target/x86_64-unknown-linux-gnu/release/examples/workload <lock_impl> -t 10 -i 3000000
```

Сравните вывод команды для различных `lock_impl`.
Counter'ы, которые нужно указывать, зависят от вашего процессора.
Список доступных счётчиков производительности можно получить с помощью команды
`perf list` – почитайте ее вывод, там много интересного.

### AMD

На процессорах AMD предлагается посмотреть на

- `l2_request_g1.change_to_x` – All L2 Cache Requests (Breakdown 1 - Common).
   Data cache state change requests. Request change to writable, check L2 for current
   state.
- `l2_cache_req_stat.ls_rd_blk_l_hit_s` – Core to L2 cacheable request access status
   (not including L2 Prefetch). Data cache read hit on shared line in L2.
- `data_fabric/all_remote_links_outbound/` – Approximate: Outbound data bytes for
   all Remote Links for a node (die).

Обратите внимание на то, что в зависимости от конкретной модели вашего процессора
и версии ядра, некоторые метрики могут быть недоступны или иметь другие имена.

Авторское решение имеет следующие метрики:

#### MCS-lock

```bash
$ sudo perf stat -C 0-9 -e l2_request_g1.change_to_x,l2_cache_req_stat.ls_rd_blk_l_hit_s -- ../target/x86_64-unknown-linux-gnu/release/examples/workload mcs -t 10 -i 3000000

 Performance counter stats for 'CPU(s) 0-9':

         8,204,407      l2_request_g1.change_to_x
           284,727      l2_cache_req_stat.ls_rd_blk_l_hit_s

      13.541832500 seconds time elapsed
```

#### Ticket lock

```bash
$ sudo perf stat -C 0-9 -e l2_request_g1.change_to_x,l2_cache_req_stat.ls_rd_blk_l_hit_s -- ../target/x86_64-unknown-linux-gnu/release/examples/workload ticket -t 10 -i 3000000

 Performance counter stats for 'CPU(s) 0-9':

        30,199,850      l2_request_g1.change_to_x
           353,775      l2_cache_req_stat.ls_rd_blk_l_hit_s

      18.275059108 seconds time elapsed
```

#### Spinlock

```bash
$ sudo perf stat -C 0-9 -e l2_request_g1.change_to_x,l2_cache_req_stat.ls_rd_blk_l_hit_s -- ../target/x86_64-unknown-linux-gnu/release/examples/workload spin -t 10 -i 3000000

 Performance counter stats for 'CPU(s) 0-9':

        15,986,285      l2_request_g1.change_to_x
         1,048,925      l2_cache_req_stat.ls_rd_blk_l_hit_s

      11.274691803 seconds time elapsed
```

### Intel

На процессорах Intel предлагается смотреть на

- [`MEM_LOAD_L3_HIT_RETIRED.XSNP_HITM`](https://perfmon-events.intel.com/index.html?pltfrm=ahybrid.html&evnt=MEM_LOAD_L3_HIT_RETIRED.XSNP_HITM)
- [`OFFCORE_REQUESTS_OUTSTANDING.DATA_RD`](https://perfmon-events.intel.com/index.html?pltfrm=ahybrid.html&evnt=OFFCORE_REQUESTS_OUTSTANDING.DATA_RD)
- [`OFFCORE_REQUESTS_OUTSTANDING.DEMAND_DATA_RD`](https://perfmon-events.intel.com/index.html?pltfrm=ahybrid.html&evnt=OFFCORE_REQUESTS_OUTSTANDING.DEMAND_DATA_RD)

Если у вас достаточно свежая версия ядра, можете попробовать запустить `perf c2c`
и `perf mem`.

Обратите внимание на то, что в зависимости от конкретной модели вашего процессора
и версии ядра, некоторые метрики могут быть недоступны или иметь другие имена.

## Бонус

Действительно ли предложенный интерфейс согласован (sound)?
Можно ли с его помощью без использования `unsafe` спровоцировать некорректный
доступ к памяти?

## Полезные ссылки

### MCS lock

- [LWN: MCS locks and qspinlocks](https://lwn.net/Articles/590243/)
- [J.M. Mellor-Crummey and M.L. Scott. Algorithms for Scalable Synchronization on Shared-Memory Multiprocessors](https://courses.csail.mit.edu/6.852/08/papers/p21-mellor-crummey.pdf)
- [Theodore Johnson and Krishna Harathi. A simple correctness proof of the MCS contention-free lock](https://web.archive.org/web/20140411142823/http://www.cise.ufl.edu/tr/DOC/REP-1992-71.pdf)
- [Non-scalable locks are dangerous](https://pdos.csail.mit.edu/papers/linux:lock.pdf).
- [Correctness of Hierarchical MCS Locks with Timeout](https://arxiv.org/pdf/1612.09576v1).

### Aliasing

- [Stacked Borrows: An Aliasing Model for Rust](https://plv.mpi-sws.org/rustbelt/stacked-borrows/)
   – текущая модель aliasing'а, которая проверяется Miri.
