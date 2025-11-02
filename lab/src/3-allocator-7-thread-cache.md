## Кэш блоков памяти, локальный для каждого потока

Если вы следовали предложенной архитектуре
библиотеки динамического выделения памяти общего назначения,
то уже должны были обогнать системный аллокатор,
если он использует
[ptmalloc](https://intranautic.com/posts/glibc-ptmalloc-internals/)
или
[другую производную](https://en.wikipedia.org/wiki/C_dynamic_memory_allocation#dlmalloc_and_ptmalloc)
от
[dlmalloc (Doug Lea's Memory Allocator)](https://gee.cs.oswego.edu/dl/html/malloc.html).

В однопоточном режиме это выглядит так, см. "nikka allocator (no cache)":

![](3-allocator-7-singlethreaded-performance.svg)

В многопоточном режиме:

![](3-allocator-7-multithreaded-performance.svg)

Наш аллокатор с глобальным на все потоки кэшем ---
"nikka allocator (global cache)" ---
примерно в 1.8 раза обгоняет вариант без кэша --- "nikka allocator (no cache)".
Но в многопоточном он же раза в два отстаёт от варианта с кэшем,
локальным для каждого потока --- просто "nikka allocator" на графиках.
То есть использование
[локальной памяти потоков](https://en.wikipedia.org/wiki/Thread-local_storage)
даёт дополнительный выигрыш примерно в 2.4 раза.

Некоторые аллокаторы на этом графике не справились с многопоточным режимом из-за
наличия глобальной блокировки.
Имеет смысл полностью убрать их из сравнения.
Системный аллокатор хоть и справился с многопоточным режимом,
медленнее чем
[tcmalloc](https://google.github.io/tcmalloc/design.html),
[jemalloc](https://people.freebsd.org/~jasone/jemalloc/bsdcan2006/jemalloc.pdf) и
[mimalloc](https://www.microsoft.com/en-us/research/wp-content/uploads/2019/06/mimalloc-tr-v1.pdf).
А некоторые аллокаторы даже медленнее системного.
Также уберём их из сравнения.

Смотреть имеет смысл на сумму среднего времени на выделение и освобождение блока ---
"allocation + deallocation" --- большинство блоков проходят обе операции.
На "zeroed allocation" и "reallocation" имеет смысл смотреть отдельно от обычных
"allocation" и "deallocation", так как в этих операциях есть зануление или копирование
памяти, которое доминирует над временем работы со структурами аллокатора.

В этой задаче наша цель --- отстать от лучшего из тройки современных аллокаторов
[tcmalloc](https://google.github.io/tcmalloc/design.html),
[jemalloc](https://people.freebsd.org/~jasone/jemalloc/bsdcan2006/jemalloc.pdf) и
[mimalloc](https://www.microsoft.com/en-us/research/wp-content/uploads/2019/06/mimalloc-tr-v1.pdf),
не более чем на 50%.
Как видно, по производительности пары операций "allocation + deallocation"
мы можем даже обогнать их.
Но это сравнение не учитывает производительность пользовательского кода при
использовании того или иного аллокатора.
А как мы помним с лекции, аллокатор на неё имеет влияние, и смотреть в первую очередь
стоит именно на изменение производительности приложения при смене
библиотеки динамического выделения памяти.

![](3-allocator-7-multithreaded-performance-no-system.svg)


### Задача 7 --- кэш блоков памяти

За кэширование выделенных блоков памяти отвечает
[`ku::allocator::cache::Cache`](../../doc/ku/allocator/cache/trait.Cache.html).
Для каждого маленького размера, выдаваемого
[`ku::allocator::dispatcher::Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
из
[`ku::allocator::fixed_size::FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html),
он хранит отдельный
[`ku::allocator::cache::Clip`](../../doc/ku/allocator/cache/struct.Clip.html).
В
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
есть несколько свободных блоков, которые можно быстро выдать на запросы выделения памяти.
А также несколько слотов, в которые можно быстро сохранить освобождаемые блоки памяти.

При наличии
[локальной памяти потоков](https://en.wikipedia.org/wiki/Thread-local_storage)
в этих операциях не придётся захватывать какие-либо блокировки.
Так как реализация типажа
[`Cache`](../../doc/ku/allocator/cache/trait.Cache.html)
для
[каждого потока](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/allocator/cache.rs)
имеет независимые наборы
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
всех размеров.
Также есть и реализация этого типажа, которая может использоваться в случае недоступности
[локальной памяти потоков](https://en.wikipedia.org/wiki/Thread-local_storage), ---
[`ku::allocator::cache::GlobalCache`](../../doc/ku/allocator/cache/struct.GlobalCache.html).
В ней каждый
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
имеет отдельную блокировку.
Как показывают измерения, даже такой вариант кэширования выделяемых блоков памяти
значительно улучшает производительность аллокатора.

Когда нужный
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
оказывается пустым в операции выделения памяти, необходимо заполнить этот
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
блоками.
Аналогично, если нужный
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
оказывается полным в операции освобождения памяти, необходимо освободить в нём место.
После любой из этих операций, нужно быть готовым и к последующему запросу выделения памяти
 и к последующему запросу освобождения памяти.
А значит, если нет никаких дополнительных предположений о вероятностях последующих запросов,
после обоих операций
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
должен стать наполовину заполненным и наполовину свободным.
Если же мы удаляем
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
при удалении аллокатора, его нужно опустошить полностью.

Реализуйте
[метод](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)

```rust
{{#include ../../ku/src/allocator/fixed_size.rs:fill_clip}}
```

и обратный к нему
[метод](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unfill_clip)

```rust
{{#include ../../ku/src/allocator/fixed_size.rs:unfill_clip}}
```

в файле
[`ku/src/allocator/fixed_size.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/fixed_size.rs).

В коде метода
[`FixedSizeAllocator::fill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)
вам может захотеться выделять элементы
[`ku::collections::bitmap::Bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html)
сразу целыми наборами.
Для этого реализуйте
[метод](../../doc/ku/collections/bitmap/struct.Bitmap.html#method.bulk_allocate)

```rust
{{#include ../../ku/src/collections/bitmap.rs:bulk_allocate}}
```

в файле
[`ku/src/collections/bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/collections/bitmap.rs).

При реализации
[`FixedSizeAllocator::unfill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unfill_clip)
и
[`Bitmap::bulk_allocate()`](../../doc/ku/collections/bitmap/struct.Bitmap.html#method.bulk_allocate)
постарайтесь обойтись без дублирования кода.
Для этого вам может пригодиться метод
[`FixedSizeAllocator::deallocate_impl()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.deallocate_impl),
который придётся реализовать самостоятельно.

Чтобы задействовать кэш,
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
должен выделять и освобождать блоки именно из него.
А не напрямую из
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
И, при необходимости, заполнять или опустошать кэш реализованными методами
[`FixedSizeAllocator::fill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)
и
[`FixedSizeAllocator::unfill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unfill_clip).

Доработайте
[метод](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.fixed_size_allocate)

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:fixed_size_allocate}}
```

и
[метод](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.fixed_size_deallocate)

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:fixed_size_deallocate}}
```

в файле
[`ku/src/allocator/dispatcher.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/dispatcher.rs).

Для анализа насколько кэш улучшает производительность, в тестах есть реализация типажа
[`Cache`](../../doc/ku/allocator/cache/trait.Cache.html)
без кэширования вообще.
Чтобы с ней код тоже работал, доступность кэша можно проверить по значению константы
[`Cache::CACHE_AVAILABLE`](../../doc/ku/allocator/cache/trait.Cache.html#associatedconstant.CACHE_AVAILABLE).


### Учёт статистик аллокатора в кэше

С одной стороны, блоки, находящиеся в
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html),
ещё не выделены пользователю.
Поэтому в
[`FixedSizeAllocator::fill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)
не нужно учитывать их в статистика.
Более того, между двумя последовательными обращениями к одному из методов
[`FixedSizeAllocator::fill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)
и
[`FixedSizeAllocator::unfill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unfill_clip),
из одного и того же
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
может быть выделено больше блоков, чем его полная ёмкость ---
если блоки в него периодически возвращались.
При заполнении или освобождении
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
мы не можем предсказать количество операций
выделения и освобождения памяти, которые будут обслужены.
А уж тем более не можем предсказать какие в точности размеры блоков
запросит пользователь, так как они могут быть чуть меньше размера,
обслуживаемого этим
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html).

С другой стороны, если на каждой операции блоки выделять из кэша,
а статистику обновлять в глобальном хранилище с захватом блокировки,
сводится на нет идея кэша локального для каждого потока,
к которому можно получать доступ без блокировок.

Поэтому
[`Clip`](../../doc/ku/allocator/cache/struct.Clip.html)
хранит статистики собственной работы в поле
[`Clip::info`](../../doc/ku/allocator/cache/struct.Clip.html#structfield.info).
При операциях, выполняющихся через кэш, используйте именно его для учёта операций.
А в методах
[`FixedSizeAllocator::fill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.fill_clip)
и
[`FixedSizeAllocator::unfill_clip()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unfill_clip),
которые в любом случае выполняются под блокировкой,
переносите накопленную статистику из
[`Clip::info`](../../doc/ku/allocator/cache/struct.Clip.html#structfield.info)
в соответствующее поле
[`FixedSizeAllocator::info`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.info).


### Проверьте себя

Теперь тесты в файле
[`ku/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-5-small.rs)
должны проходить с не более чем 50% отставанием от `tcmalloc`, `jemalloc` и `mimalloc`:

```console
$ (cd ku && ALLOWED_RATIO=1.50 cargo test --profile benchmark --features benchmark --test 3-allocator-5-small -- --test-threads=1)
   Compiling ku v0.5.0 (/.../nikka/ku)
    Finished `benchmark` profile [optimized] target(s) in 2.93s
     Running tests/3-allocator-5-small.rs (/.../nikka/target/benchmark/deps/3_allocator_5_small-c5d5bfe3cca771cd)

running 3 tests
test basic ... 2025-11-02T20:39:23.685943Z i basic allocator metadata size common_metadata=511.875 KiB per_thread_cache=511.750 KiB
ok
test multi_threaded ... 2025-11-02T20:39:23.717019Z : ThreadId(06) allocator="nikka allocator" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_97AF_4730, sum: Some(119266) }
2025-11-02T20:39:23.717623Z : ThreadId(07) allocator="nikka allocator" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_9E3E_BA60, sum: Some(64941) }
2025-11-02T20:39:23.719011Z : ThreadId(04) allocator="nikka allocator" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_9F71_DC00, sum: Some(46851) }
2025-11-02T20:39:23.719044Z : ThreadId(05) allocator="nikka allocator" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_9DDD_3F80, sum: Some(53480) }
2025-11-02T20:39:23.745799Z : ThreadId(06) allocator="nikka allocator" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_9EDD_2200, sum: Some(32042) }
2025-11-02T20:39:23.746213Z : ThreadId(07) allocator="nikka allocator" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_9D0F_1268, sum: Some(129154) }
2025-11-02T20:39:23.750499Z : ThreadId(04) allocator="nikka allocator" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_959F_1020, sum: Some(30652) }
2025-11-02T20:39:23.755466Z : ThreadId(05) allocator="nikka allocator" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_9EDC_9400, sum: Some(23391) }
2025-11-02T20:39:23.774895Z : ThreadId(06) allocator="nikka allocator" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_9F76_8800, sum: Some(43353) }
2025-11-02T20:39:23.781087Z : ThreadId(07) allocator="nikka allocator" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_9E2E_69C0, sum: Some(39864) }
2025-11-02T20:39:23.783507Z : ThreadId(04) allocator="nikka allocator" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_968E_CA40, sum: Some(67833) }
2025-11-02T20:39:23.802376Z : ThreadId(05) allocator="nikka allocator" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_9D50_0640, sum: Some(85075) }
2025-11-02T20:39:23.804304Z : ThreadId(06) allocator="nikka allocator" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_96F0_4C20, sum: Some(88118) }
2025-11-02T20:39:23.814920Z : ThreadId(04) allocator="nikka allocator" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_8A4C_D820, sum: Some(13436) }
2025-11-02T20:39:23.820870Z : ThreadId(07) allocator="nikka allocator" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_9F70_0400, sum: Some(116343) }
2025-11-02T20:39:23.834693Z : ThreadId(05) allocator="nikka allocator" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_9F78_6400, sum: Some(75290) }
2025-11-02T20:39:23.834914Z : ThreadId(06) allocator="nikka allocator" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_961C_AB20, sum: Some(19257) }
2025-11-02T20:39:23.847320Z : ThreadId(04) allocator="nikka allocator" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_956D_CFC0, sum: Some(5020) }
2025-11-02T20:39:23.850115Z : ThreadId(07) allocator="nikka allocator" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_9F5F_2800, sum: Some(61803) }
2025-11-02T20:39:23.865339Z : ThreadId(05) allocator="nikka allocator" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_9CEE_F600, sum: Some(118699) }
2025-11-02T20:39:23.865349Z : ThreadId(06) allocator="nikka allocator" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_9D40_4A60, sum: Some(102652) }
2025-11-02T20:39:23.878522Z : ThreadId(04) allocator="nikka allocator" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_9F61_BC00, sum: Some(45694) }
2025-11-02T20:39:23.881731Z : ThreadId(07) allocator="nikka allocator" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_9F2D_D400, sum: Some(2729) }
2025-11-02T20:39:23.894544Z : ThreadId(05) allocator="nikka allocator" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_97F0_1E98, sum: Some(89371) }
2025-11-02T20:39:23.895564Z : ThreadId(06) allocator="nikka allocator" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_9EDD_7F00, sum: Some(8252) }
2025-11-02T20:39:23.908672Z : ThreadId(04) allocator="nikka allocator" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_9F71_2400, sum: Some(89169) }
2025-11-02T20:39:23.912904Z : ThreadId(07) allocator="nikka allocator" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_9D4F_E800, sum: Some(89146) }
2025-11-02T20:39:23.924987Z : ThreadId(05) allocator="nikka allocator" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_9E5E_DC38, sum: Some(24616) }
2025-11-02T20:39:23.926063Z : ThreadId(06) allocator="nikka allocator" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_899A_8A70, sum: Some(58305) }
2025-11-02T20:39:23.938357Z : ThreadId(04) allocator="nikka allocator" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_9D3F_F2E0, sum: Some(102990) }
2025-11-02T20:39:23.945963Z : ThreadId(07) allocator="nikka allocator" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_963D_7020, sum: Some(9102) }
2025-11-02T20:39:23.963021Z : ThreadId(05) allocator="nikka allocator" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_9CF0_43C0, sum: Some(118924) }
2025-11-02T20:39:23.975353Z : ThreadId(06) allocator="nikka allocator" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_9F5F_6200, sum: Some(29007) }
2025-11-02T20:39:23.977344Z : ThreadId(04) allocator="nikka allocator" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_9ECC_E180, sum: Some(36528) }
2025-11-02T20:39:23.979005Z : ThreadId(07) allocator="nikka allocator" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_9F90_53C0, sum: Some(106866) }
2025-11-02T20:39:23.994890Z : ThreadId(05) allocator="nikka allocator" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_9E4F_4950, sum: Some(128153) }
2025-11-02T20:39:24.005645Z i ThreadId(06) performance breakdown (smaller is better) allocator_name="nikka allocator" allocation={ total: 3.152586ms, per_operation: 18.84ns, count: 167308 } allocation_zeroed={ total: 10.874233ms, per_operation: 65.17ns, count: 166850 } deallocation={ total: 2.775689ms, per_operation: 16.59ns, count: 167308 } reallocation={ total: 21.038965ms, per_operation: 63.05ns, count: 333702 } supplementary_code=280.268502ms total=318.109975ms
2025-11-02T20:39:24.005915Z i ThreadId(07) performance breakdown (smaller is better) allocator_name="nikka allocator" allocation={ total: 6.061582ms, per_operation: 36.30ns, count: 167008 } allocation_zeroed={ total: 9.607888ms, per_operation: 57.67ns, count: 166592 } deallocation={ total: 3.340637ms, per_operation: 20.00ns, count: 167008 } reallocation={ total: 23.044404ms, per_operation: 69.22ns, count: 332936 } supplementary_code=276.000914ms total=318.055425ms
2025-11-02T20:39:24.018214Z i ThreadId(04) performance breakdown (smaller is better) allocator_name="nikka allocator" allocation={ total: 4.078959ms, per_operation: 24.40ns, count: 167145 } allocation_zeroed={ total: 9.648449ms, per_operation: 57.95ns, count: 166501 } deallocation={ total: 4.16173ms, per_operation: 24.90ns, count: 167145 } reallocation={ total: 22.177844ms, per_operation: 66.51ns, count: 333462 } supplementary_code=291.077166ms total=331.144148ms
2025-11-02T20:39:24.022765Z i ThreadId(05) performance breakdown (smaller is better) allocator_name="nikka allocator" allocation={ total: 3.660217ms, per_operation: 21.85ns, count: 167532 } allocation_zeroed={ total: 11.896035ms, per_operation: 71.23ns, count: 167008 } deallocation={ total: 3.195181ms, per_operation: 19.07ns, count: 167532 } reallocation={ total: 23.78536ms, per_operation: 71.39ns, count: 333163 } supplementary_code=292.922167ms total=335.45896ms
2025-11-02T20:39:24.023333Z i multi_threaded nikka allocation + deallocation duration = 45.48ns
2025-11-02T20:39:24.053988Z : ThreadId(09) allocator="tcmalloc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v5317_3FE2_3C00, sum: Some(53480) }
2025-11-02T20:39:24.054222Z : ThreadId(10) allocator="tcmalloc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v5317_3FC5_6800, sum: Some(119266) }
2025-11-02T20:39:24.057361Z : ThreadId(08) allocator="tcmalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v5317_3FC4_1000, sum: Some(46851) }
2025-11-02T20:39:24.061142Z : ThreadId(11) allocator="tcmalloc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v5317_3FC9_A000, sum: Some(64941) }
2025-11-02T20:39:24.084164Z : ThreadId(09) allocator="tcmalloc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v5317_3FE4_0B00, sum: Some(23391) }
2025-11-02T20:39:24.084455Z : ThreadId(10) allocator="tcmalloc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v5317_3FE5_5000, sum: Some(32042) }
2025-11-02T20:39:24.097085Z : ThreadId(08) allocator="tcmalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v5317_3FE4_1200, sum: Some(30652) }
2025-11-02T20:39:24.110829Z : ThreadId(11) allocator="tcmalloc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v5317_3FCB_C400, sum: Some(129154) }
2025-11-02T20:39:24.113409Z : ThreadId(09) allocator="tcmalloc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v5317_3FD0_4580, sum: Some(85075) }
2025-11-02T20:39:24.115244Z : ThreadId(10) allocator="tcmalloc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v5317_3FD9_AC00, sum: Some(43353) }
2025-11-02T20:39:24.134844Z : ThreadId(08) allocator="tcmalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v5317_3FA0_9D40, sum: Some(67833) }
2025-11-02T20:39:24.142752Z : ThreadId(09) allocator="tcmalloc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v5317_3FC2_9000, sum: Some(75290) }
2025-11-02T20:39:24.146252Z : ThreadId(11) allocator="tcmalloc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v5317_3FE2_E9A0, sum: Some(39864) }
2025-11-02T20:39:24.152372Z : ThreadId(10) allocator="tcmalloc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v5317_3FA1_98C0, sum: Some(88118) }
2025-11-02T20:39:24.172156Z : ThreadId(09) allocator="tcmalloc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v5317_3FC4_C800, sum: Some(118699) }
2025-11-02T20:39:24.173528Z : ThreadId(08) allocator="tcmalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v5317_3FE0_6850, sum: Some(13436) }
2025-11-02T20:39:24.186623Z : ThreadId(10) allocator="tcmalloc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v5317_3FE5_9360, sum: Some(19257) }
2025-11-02T20:39:24.195088Z : ThreadId(11) allocator="tcmalloc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v5317_3FCF_4800, sum: Some(116343) }
2025-11-02T20:39:24.200969Z : ThreadId(09) allocator="tcmalloc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v5317_3FD0_D500, sum: Some(89371) }
2025-11-02T20:39:24.211358Z : ThreadId(08) allocator="tcmalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v5317_3FE1_B3C0, sum: Some(5020) }
2025-11-02T20:39:24.221036Z : ThreadId(10) allocator="tcmalloc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v5317_3FDF_2000, sum: Some(102652) }
2025-11-02T20:39:24.224667Z : ThreadId(11) allocator="tcmalloc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v5317_3FCA_4E00, sum: Some(61803) }
2025-11-02T20:39:24.230075Z : ThreadId(09) allocator="tcmalloc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v5317_3FE0_B930, sum: Some(24616) }
2025-11-02T20:39:24.262642Z : ThreadId(10) allocator="tcmalloc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v5317_3FE4_DC00, sum: Some(8252) }
2025-11-02T20:39:24.262936Z : ThreadId(08) allocator="tcmalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v5317_3FA8_3000, sum: Some(45694) }
2025-11-02T20:39:24.263010Z : ThreadId(09) allocator="tcmalloc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v5317_3FC0_5800, sum: Some(118924) }
2025-11-02T20:39:24.279597Z : ThreadId(11) allocator="tcmalloc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v5317_3FE0_9380, sum: Some(2729) }
2025-11-02T20:39:24.294391Z : ThreadId(10) allocator="tcmalloc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v5317_3FE8_7500, sum: Some(58305) }
2025-11-02T20:39:24.300170Z : ThreadId(09) allocator="tcmalloc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v5317_3FDD_6000, sum: Some(128153) }
2025-11-02T20:39:24.301176Z : ThreadId(08) allocator="tcmalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v5317_3FD8_7800, sum: Some(89169) }
2025-11-02T20:39:24.313095Z : ThreadId(11) allocator="tcmalloc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v5317_3FCA_0840, sum: Some(89146) }
2025-11-02T20:39:24.332721Z : ThreadId(10) allocator="tcmalloc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v5317_3FC9_3200, sum: Some(29007) }
2025-11-02T20:39:24.340067Z i ThreadId(09) performance breakdown (smaller is better) allocator_name="tcmalloc" allocation={ total: 4.234944ms, per_operation: 25.28ns, count: 167532 } allocation_zeroed={ total: 9.904855ms, per_operation: 59.31ns, count: 167008 } deallocation={ total: 3.486066ms, per_operation: 20.81ns, count: 167532 } reallocation={ total: 24.524694ms, per_operation: 73.61ns, count: 333163 } supplementary_code=274.362516ms total=316.513075ms
2025-11-02T20:39:24.340424Z : ThreadId(08) allocator="tcmalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v5317_3FC7_9880, sum: Some(102990) }
2025-11-02T20:39:24.348575Z : ThreadId(11) allocator="tcmalloc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v5317_3FE1_6B40, sum: Some(9102) }
2025-11-02T20:39:24.364162Z i ThreadId(10) performance breakdown (smaller is better) allocator_name="tcmalloc" allocation={ total: 4.704591ms, per_operation: 28.12ns, count: 167308 } allocation_zeroed={ total: 12.257959ms, per_operation: 73.47ns, count: 166850 } deallocation={ total: 3.450281ms, per_operation: 20.62ns, count: 167308 } reallocation={ total: 29.456324ms, per_operation: 88.27ns, count: 333702 } supplementary_code=290.715458ms total=340.584613ms
2025-11-02T20:39:24.372696Z : ThreadId(08) allocator="tcmalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v5317_3FC2_AE00, sum: Some(36528) }
2025-11-02T20:39:24.378094Z : ThreadId(11) allocator="tcmalloc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v5317_3FC8_9500, sum: Some(106866) }
2025-11-02T20:39:24.397333Z i ThreadId(08) performance breakdown (smaller is better) allocator_name="tcmalloc" allocation={ total: 4.914681ms, per_operation: 29.40ns, count: 167145 } allocation_zeroed={ total: 15.138148ms, per_operation: 90.92ns, count: 166501 } deallocation={ total: 3.69338ms, per_operation: 22.10ns, count: 167145 } reallocation={ total: 33.211293ms, per_operation: 99.60ns, count: 333462 } supplementary_code=316.913343ms total=373.870845ms
2025-11-02T20:39:24.402336Z i ThreadId(11) performance breakdown (smaller is better) allocator_name="tcmalloc" allocation={ total: 5.346048ms, per_operation: 32.01ns, count: 167008 } allocation_zeroed={ total: 11.873869ms, per_operation: 71.28ns, count: 166592 } deallocation={ total: 3.944085ms, per_operation: 23.62ns, count: 167008 } reallocation={ total: 33.976851ms, per_operation: 102.05ns, count: 332936 } supplementary_code=323.552446ms total=378.693299ms
2025-11-02T20:39:24.438427Z : ThreadId(15) allocator="jemalloc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_9481_BD80, sum: Some(64941) }
2025-11-02T20:39:24.439331Z : ThreadId(12) allocator="jemalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A5C2_D000, sum: Some(46851) }
2025-11-02T20:39:24.441026Z : ThreadId(14) allocator="jemalloc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_94C4_C800, sum: Some(119266) }
2025-11-02T20:39:24.441496Z : ThreadId(13) allocator="jemalloc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_9C20_4880, sum: Some(53480) }
2025-11-02T20:39:24.468169Z : ThreadId(12) allocator="jemalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A5C3_C460, sum: Some(30652) }
2025-11-02T20:39:24.475146Z : ThreadId(15) allocator="jemalloc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_9485_5400, sum: Some(129154) }
2025-11-02T20:39:24.475906Z : ThreadId(14) allocator="jemalloc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_94C5_C200, sum: Some(32042) }
2025-11-02T20:39:24.479573Z : ThreadId(13) allocator="jemalloc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_9C23_7900, sum: Some(23391) }
2025-11-02T20:39:24.497593Z : ThreadId(12) allocator="jemalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A5C1_3C00, sum: Some(67833) }
2025-11-02T20:39:24.507248Z : ThreadId(14) allocator="jemalloc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_94C7_B800, sum: Some(43353) }
2025-11-02T20:39:24.515831Z : ThreadId(15) allocator="jemalloc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_9488_3D80, sum: Some(39864) }
2025-11-02T20:39:24.516783Z : ThreadId(13) allocator="jemalloc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_9C22_1800, sum: Some(85075) }
2025-11-02T20:39:24.528808Z : ThreadId(12) allocator="jemalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A5C3_14D0, sum: Some(13436) }
2025-11-02T20:39:24.552432Z : ThreadId(15) allocator="jemalloc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_9485_9000, sum: Some(116343) }
2025-11-02T20:39:24.556831Z : ThreadId(13) allocator="jemalloc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_9C2B_F800, sum: Some(75290) }
2025-11-02T20:39:24.559397Z : ThreadId(12) allocator="jemalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A5C6_D100, sum: Some(5020) }
2025-11-02T20:39:24.563675Z : ThreadId(14) allocator="jemalloc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_94C5_F600, sum: Some(88118) }
2025-11-02T20:39:24.590980Z : ThreadId(12) allocator="jemalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A5CE_A600, sum: Some(45694) }
2025-11-02T20:39:24.592828Z : ThreadId(15) allocator="jemalloc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_9480_1800, sum: Some(61803) }
2025-11-02T20:39:24.596058Z : ThreadId(14) allocator="jemalloc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_94C0_F320, sum: Some(19257) }
2025-11-02T20:39:24.618973Z : ThreadId(13) allocator="jemalloc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_9C27_C800, sum: Some(118699) }
2025-11-02T20:39:24.623039Z : ThreadId(12) allocator="jemalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A5C8_F400, sum: Some(89169) }
2025-11-02T20:39:24.627061Z : ThreadId(14) allocator="jemalloc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_94C7_1580, sum: Some(102652) }
2025-11-02T20:39:24.629817Z : ThreadId(15) allocator="jemalloc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_9483_C900, sum: Some(2729) }
2025-11-02T20:39:24.653148Z : ThreadId(12) allocator="jemalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A5CC_6F00, sum: Some(102990) }
2025-11-02T20:39:24.655858Z : ThreadId(13) allocator="jemalloc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_9C2F_2500, sum: Some(89371) }
2025-11-02T20:39:24.658631Z : ThreadId(14) allocator="jemalloc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_94CB_C500, sum: Some(8252) }
2025-11-02T20:39:24.660457Z : ThreadId(15) allocator="jemalloc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_9487_BE00, sum: Some(89146) }
2025-11-02T20:39:24.684271Z : ThreadId(12) allocator="jemalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A5CB_5180, sum: Some(36528) }
2025-11-02T20:39:24.696910Z : ThreadId(13) allocator="jemalloc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_9C22_8500, sum: Some(24616) }
2025-11-02T20:39:24.699908Z : ThreadId(14) allocator="jemalloc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_94D1_CE00, sum: Some(58305) }
2025-11-02T20:39:24.700222Z : ThreadId(15) allocator="jemalloc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_9483_3120, sum: Some(9102) }
2025-11-02T20:39:24.709364Z i ThreadId(12) performance breakdown (smaller is better) allocator_name="jemalloc" allocation={ total: 4.79792ms, per_operation: 28.71ns, count: 167145 } allocation_zeroed={ total: 9.502265ms, per_operation: 57.07ns, count: 166501 } deallocation={ total: 3.071968ms, per_operation: 18.38ns, count: 167145 } reallocation={ total: 29.720513ms, per_operation: 89.13ns, count: 333462 } supplementary_code=259.688895ms total=306.781561ms
2025-11-02T20:39:24.734882Z : ThreadId(13) allocator="jemalloc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_9C24_D800, sum: Some(118924) }
2025-11-02T20:39:24.736484Z : ThreadId(15) allocator="jemalloc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_9480_3380, sum: Some(106866) }
2025-11-02T20:39:24.739400Z : ThreadId(14) allocator="jemalloc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_94CB_9C00, sum: Some(29007) }
2025-11-02T20:39:24.760741Z i ThreadId(15) performance breakdown (smaller is better) allocator_name="jemalloc" allocation={ total: 5.35984ms, per_operation: 32.09ns, count: 167008 } allocation_zeroed={ total: 9.987592ms, per_operation: 59.95ns, count: 166592 } deallocation={ total: 3.587446ms, per_operation: 21.48ns, count: 167008 } reallocation={ total: 34.056978ms, per_operation: 102.29ns, count: 332936 } supplementary_code=304.88528ms total=357.877136ms
2025-11-02T20:39:24.770113Z : ThreadId(13) allocator="jemalloc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_9C2B_A800, sum: Some(128153) }
2025-11-02T20:39:24.773684Z i ThreadId(14) performance breakdown (smaller is better) allocator_name="jemalloc" allocation={ total: 5.323076ms, per_operation: 31.82ns, count: 167308 } allocation_zeroed={ total: 11.93812ms, per_operation: 71.55ns, count: 166850 } deallocation={ total: 3.731243ms, per_operation: 22.30ns, count: 167308 } reallocation={ total: 30.595074ms, per_operation: 91.68ns, count: 333702 } supplementary_code=319.35006ms total=370.937573ms
2025-11-02T20:39:24.797616Z i ThreadId(13) performance breakdown (smaller is better) allocator_name="jemalloc" allocation={ total: 9.132713ms, per_operation: 54.51ns, count: 167532 } allocation_zeroed={ total: 17.879282ms, per_operation: 107.06ns, count: 167008 } deallocation={ total: 4.546145ms, per_operation: 27.14ns, count: 167532 } reallocation={ total: 38.217401ms, per_operation: 114.71ns, count: 333163 } supplementary_code=325.21045ms total=394.985991ms
2025-11-02T20:39:24.829069Z : ThreadId(18) allocator="mimalloc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v3BC_9A05_3C00, sum: Some(119266) }
2025-11-02T20:39:24.830504Z : ThreadId(16) allocator="mimalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v3BC_9814_1400, sum: Some(46851) }
2025-11-02T20:39:24.833285Z : ThreadId(19) allocator="mimalloc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v3BC_9C08_8C00, sum: Some(64941) }
2025-11-02T20:39:24.840298Z : ThreadId(17) allocator="mimalloc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v3BC_D803_3100, sum: Some(53480) }
2025-11-02T20:39:24.859002Z : ThreadId(18) allocator="mimalloc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v3BC_9A11_0600, sum: Some(32042) }
2025-11-02T20:39:24.860533Z : ThreadId(16) allocator="mimalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v3BC_980A_0660, sum: Some(30652) }
2025-11-02T20:39:24.862253Z : ThreadId(19) allocator="mimalloc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v3BC_9C06_9800, sum: Some(129154) }
2025-11-02T20:39:24.875845Z : ThreadId(17) allocator="mimalloc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v3BC_D803_4F00, sum: Some(23391) }
2025-11-02T20:39:24.889888Z : ThreadId(18) allocator="mimalloc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v3BC_9A02_C000, sum: Some(43353) }
2025-11-02T20:39:24.900699Z : ThreadId(16) allocator="mimalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v3BC_9809_4A00, sum: Some(67833) }
2025-11-02T20:39:24.903330Z : ThreadId(19) allocator="mimalloc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v3BC_9C0F_1FC0, sum: Some(39864) }
2025-11-02T20:39:24.911731Z : ThreadId(17) allocator="mimalloc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v3BC_D81C_0C00, sum: Some(85075) }
2025-11-02T20:39:24.920451Z : ThreadId(18) allocator="mimalloc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v3BC_9A01_CE00, sum: Some(88118) }
2025-11-02T20:39:24.932165Z : ThreadId(19) allocator="mimalloc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v3BC_9C19_5C00, sum: Some(116343) }
2025-11-02T20:39:24.934412Z : ThreadId(16) allocator="mimalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v3BC_9815_05B0, sum: Some(13436) }
2025-11-02T20:39:24.951909Z : ThreadId(18) allocator="mimalloc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v3BC_9A0B_1620, sum: Some(19257) }
2025-11-02T20:39:24.960891Z : ThreadId(17) allocator="mimalloc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v3BC_D80A_4E00, sum: Some(75290) }
2025-11-02T20:39:24.975031Z : ThreadId(19) allocator="mimalloc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v3BC_9C01_5A00, sum: Some(61803) }
2025-11-02T20:39:24.976419Z : ThreadId(16) allocator="mimalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v3BC_9807_0D60, sum: Some(5020) }
2025-11-02T20:39:24.983045Z : ThreadId(18) allocator="mimalloc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v3BC_9A1E_EC80, sum: Some(102652) }
2025-11-02T20:39:24.991415Z : ThreadId(17) allocator="mimalloc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v3BC_D802_8800, sum: Some(118699) }
2025-11-02T20:39:25.010190Z : ThreadId(16) allocator="mimalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v3BC_9804_8500, sum: Some(45694) }
2025-11-02T20:39:25.016433Z : ThreadId(19) allocator="mimalloc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v3BC_9C0E_0680, sum: Some(2729) }
2025-11-02T20:39:25.018033Z : ThreadId(18) allocator="mimalloc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v3BC_9A0C_4700, sum: Some(8252) }
2025-11-02T20:39:25.025974Z : ThreadId(17) allocator="mimalloc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v3BC_D808_B900, sum: Some(89371) }
2025-11-02T20:39:25.046707Z : ThreadId(19) allocator="mimalloc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v3BC_9C05_CB00, sum: Some(89146) }
2025-11-02T20:39:25.047464Z : ThreadId(16) allocator="mimalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v3BC_9809_A400, sum: Some(89169) }
2025-11-02T20:39:25.054929Z : ThreadId(18) allocator="mimalloc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v3BC_9A1F_1300, sum: Some(58305) }
2025-11-02T20:39:25.059913Z : ThreadId(17) allocator="mimalloc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v3BC_D80D_1B00, sum: Some(24616) }
2025-11-02T20:39:25.076334Z : ThreadId(16) allocator="mimalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v3BC_9803_2500, sum: Some(102990) }
2025-11-02T20:39:25.076875Z : ThreadId(19) allocator="mimalloc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v3BC_9C15_0F40, sum: Some(9102) }
2025-11-02T20:39:25.095352Z : ThreadId(18) allocator="mimalloc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v3BC_9A01_8400, sum: Some(29007) }
2025-11-02T20:39:25.101846Z : ThreadId(17) allocator="mimalloc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v3BC_D827_7C00, sum: Some(118924) }
2025-11-02T20:39:25.105908Z : ThreadId(16) allocator="mimalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v3BC_9802_6B80, sum: Some(36528) }
2025-11-02T20:39:25.107596Z : ThreadId(19) allocator="mimalloc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v3BC_9C1A_7380, sum: Some(106866) }
2025-11-02T20:39:25.131602Z i ThreadId(19) performance breakdown (smaller is better) allocator_name="mimalloc" allocation={ total: 4.856404ms, per_operation: 29.08ns, count: 167008 } allocation_zeroed={ total: 8.393641ms, per_operation: 50.38ns, count: 166592 } deallocation={ total: 2.612311ms, per_operation: 15.64ns, count: 167008 } reallocation={ total: 27.314433ms, per_operation: 82.04ns, count: 332936 } supplementary_code=290.178013ms total=333.354802ms
2025-11-02T20:39:25.132287Z i ThreadId(16) performance breakdown (smaller is better) allocator_name="mimalloc" allocation={ total: 5.537918ms, per_operation: 33.13ns, count: 167145 } allocation_zeroed={ total: 8.931133ms, per_operation: 53.64ns, count: 166501 } deallocation={ total: 4.666743ms, per_operation: 27.92ns, count: 167145 } reallocation={ total: 26.812797ms, per_operation: 80.41ns, count: 333462 } supplementary_code=288.250503ms total=334.199094ms
2025-11-02T20:39:25.133034Z : ThreadId(17) allocator="mimalloc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v3BC_D81A_BC00, sum: Some(128153) }
2025-11-02T20:39:25.135032Z i ThreadId(18) performance breakdown (smaller is better) allocator_name="mimalloc" allocation={ total: 6.663834ms, per_operation: 39.83ns, count: 167308 } allocation_zeroed={ total: 10.233899ms, per_operation: 61.34ns, count: 166850 } deallocation={ total: 3.282208ms, per_operation: 19.62ns, count: 167308 } reallocation={ total: 28.461231ms, per_operation: 85.29ns, count: 333702 } supplementary_code=288.211641ms total=336.852813ms
2025-11-02T20:39:25.159651Z i ThreadId(17) performance breakdown (smaller is better) allocator_name="mimalloc" allocation={ total: 5.988862ms, per_operation: 35.75ns, count: 167532 } allocation_zeroed={ total: 10.577177ms, per_operation: 63.33ns, count: 167008 } deallocation={ total: 2.762217ms, per_operation: 16.49ns, count: 167532 } reallocation={ total: 31.93531ms, per_operation: 95.85ns, count: 333163 } supplementary_code=310.252403ms total=361.515969ms
2025-11-02T20:39:25.191602Z : ThreadId(20) allocator="ring-alloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_9479_6400, sum: Some(46851) }
2025-11-02T20:39:25.192000Z : ThreadId(22) allocator="ring-alloc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_9427_6928, sum: Some(119266) }
2025-11-02T20:39:25.195208Z : ThreadId(23) allocator="ring-alloc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_8856_B420, sum: Some(64941) }
2025-11-02T20:39:25.211117Z : ThreadId(21) allocator="ring-alloc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_88BC_3580, sum: Some(53480) }
2025-11-02T20:39:25.221330Z : ThreadId(22) allocator="ring-alloc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_9004_7460, sum: Some(32042) }
2025-11-02T20:39:25.222773Z : ThreadId(20) allocator="ring-alloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_8403_3FE0, sum: Some(30652) }
2025-11-02T20:39:25.236969Z : ThreadId(23) allocator="ring-alloc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_887C_D5C0, sum: Some(129154) }
2025-11-02T20:39:25.251245Z : ThreadId(21) allocator="ring-alloc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_9806_6700, sum: Some(23391) }
2025-11-02T20:39:25.254181Z : ThreadId(22) allocator="ring-alloc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_836C_4C00, sum: Some(43353) }
2025-11-02T20:39:25.256434Z : ThreadId(20) allocator="ring-alloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_883C_6C10, sum: Some(67833) }
2025-11-02T20:39:25.283259Z : ThreadId(23) allocator="ring-alloc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_887A_2C60, sum: Some(39864) }
2025-11-02T20:39:25.285302Z : ThreadId(21) allocator="ring-alloc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_88BA_7258, sum: Some(85075) }
2025-11-02T20:39:25.285456Z : ThreadId(22) allocator="ring-alloc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_8362_FD20, sum: Some(88118) }
2025-11-02T20:39:25.286866Z : ThreadId(20) allocator="ring-alloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_8401_1420, sum: Some(13436) }
2025-11-02T20:39:25.317784Z : ThreadId(23) allocator="ring-alloc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_8877_5400, sum: Some(116343) }
2025-11-02T20:39:25.317898Z : ThreadId(22) allocator="ring-alloc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_9000_8DF0, sum: Some(19257) }
2025-11-02T20:39:25.319840Z : ThreadId(20) allocator="ring-alloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_840B_D720, sum: Some(5020) }
2025-11-02T20:39:25.321214Z : ThreadId(21) allocator="ring-alloc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_88A8_B800, sum: Some(75290) }
2025-11-02T20:39:25.348239Z : ThreadId(22) allocator="ring-alloc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_888A_0A80, sum: Some(102652) }
2025-11-02T20:39:25.350961Z : ThreadId(20) allocator="ring-alloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_8304_6500, sum: Some(45694) }
2025-11-02T20:39:25.364608Z : ThreadId(23) allocator="ring-alloc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_88E2_79D0, sum: Some(61803) }
2025-11-02T20:39:25.373622Z : ThreadId(21) allocator="ring-alloc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_88A3_A800, sum: Some(118699) }
2025-11-02T20:39:25.379960Z : ThreadId(22) allocator="ring-alloc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_9004_AC00, sum: Some(8252) }
2025-11-02T20:39:25.383237Z : ThreadId(20) allocator="ring-alloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_8318_F400, sum: Some(89169) }
2025-11-02T20:39:25.403404Z : ThreadId(23) allocator="ring-alloc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_8C04_7C00, sum: Some(2729) }
2025-11-02T20:39:25.411269Z : ThreadId(21) allocator="ring-alloc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_83AD_4180, sum: Some(89371) }
2025-11-02T20:39:25.412166Z : ThreadId(22) allocator="ring-alloc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_82E5_D448, sum: Some(58305) }
2025-11-02T20:39:25.415186Z : ThreadId(20) allocator="ring-alloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_8314_32B0, sum: Some(102990) }
2025-11-02T20:39:25.446209Z : ThreadId(20) allocator="ring-alloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_830E_7600, sum: Some(36528) }
2025-11-02T20:39:25.446509Z : ThreadId(22) allocator="ring-alloc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_8880_F800, sum: Some(29007) }
2025-11-02T20:39:25.451479Z : ThreadId(23) allocator="ring-alloc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_88E4_87B8, sum: Some(89146) }
2025-11-02T20:39:25.454184Z : ThreadId(21) allocator="ring-alloc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_9803_2888, sum: Some(24616) }
2025-11-02T20:39:25.471813Z i ThreadId(20) performance breakdown (smaller is better) allocator_name="ring-alloc" allocation={ total: 3.900642ms, per_operation: 23.34ns, count: 167145 } allocation_zeroed={ total: 11.129531ms, per_operation: 66.84ns, count: 166501 } deallocation={ total: 5.543735ms, per_operation: 33.17ns, count: 167145 } reallocation={ total: 25.017791ms, per_operation: 75.02ns, count: 333462 } supplementary_code=266.222927ms total=311.814626ms
2025-11-02T20:39:25.477843Z i ThreadId(22) performance breakdown (smaller is better) allocator_name="ring-alloc" allocation={ total: 3.938337ms, per_operation: 23.54ns, count: 167308 } allocation_zeroed={ total: 11.378017ms, per_operation: 68.19ns, count: 166850 } deallocation={ total: 5.593213ms, per_operation: 33.43ns, count: 167308 } reallocation={ total: 26.466796ms, per_operation: 79.31ns, count: 333702 } supplementary_code=270.315744ms total=317.692107ms
2025-11-02T20:39:25.492307Z : ThreadId(23) allocator="ring-alloc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_8C04_8B00, sum: Some(9102) }
2025-11-02T20:39:25.493211Z : ThreadId(21) allocator="ring-alloc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_8342_BFC0, sum: Some(118924) }
2025-11-02T20:39:25.521468Z : ThreadId(23) allocator="ring-alloc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_88FF_9100, sum: Some(106866) }
2025-11-02T20:39:25.525707Z : ThreadId(21) allocator="ring-alloc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_8233_6D90, sum: Some(128153) }
2025-11-02T20:39:25.545341Z i ThreadId(23) performance breakdown (smaller is better) allocator_name="ring-alloc" allocation={ total: 11.889785ms, per_operation: 71.19ns, count: 167008 } allocation_zeroed={ total: 10.407736ms, per_operation: 62.47ns, count: 166592 } deallocation={ total: 5.412814ms, per_operation: 32.41ns, count: 167008 } reallocation={ total: 24.367469ms, per_operation: 73.19ns, count: 332936 } supplementary_code=333.035076ms total=385.11288ms
2025-11-02T20:39:25.555295Z i ThreadId(21) performance breakdown (smaller is better) allocator_name="ring-alloc" allocation={ total: 4.996403ms, per_operation: 29.82ns, count: 167532 } allocation_zeroed={ total: 13.183482ms, per_operation: 78.94ns, count: 167008 } deallocation={ total: 6.544469ms, per_operation: 39.06ns, count: 167532 } reallocation={ total: 35.871512ms, per_operation: 107.67ns, count: 333163 } supplementary_code=334.632104ms total=395.22797ms
2025-11-02T20:39:25.590031Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_7FDF_2FC0, sum: Some(53480) }
2025-11-02T20:39:25.597031Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_820A_E220, sum: Some(64941) }
2025-11-02T20:39:25.598924Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_81CF_11D0, sum: Some(119266) }
2025-11-02T20:39:25.600168Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_83C1_B000, sum: Some(46851) }
2025-11-02T20:39:25.624880Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_825B_F000, sum: Some(23391) }
2025-11-02T20:39:25.631379Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_7F3C_4BB8, sum: Some(129154) }
2025-11-02T20:39:25.632153Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_81F0_1180, sum: Some(30652) }
2025-11-02T20:39:25.632417Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_825C_3300, sum: Some(32042) }
2025-11-02T20:39:25.678874Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_82E0_1640, sum: Some(85075) }
2025-11-02T20:39:25.679526Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_834D_B8F0, sum: Some(39864) }
2025-11-02T20:39:25.681015Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_83C6_5400, sum: Some(43353) }
2025-11-02T20:39:25.683123Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_803D_85E0, sum: Some(67833) }
2025-11-02T20:39:25.712014Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_83C1_4400, sum: Some(116343) }
2025-11-02T20:39:25.717144Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_83C1_F400, sum: Some(75290) }
2025-11-02T20:39:25.718062Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_7BCF_5138, sum: Some(13436) }
2025-11-02T20:39:25.726844Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_809E_E358, sum: Some(88118) }
2025-11-02T20:39:25.748334Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_8380_7600, sum: Some(61803) }
2025-11-02T20:39:25.758843Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_7FCF_2B00, sum: Some(5020) }
2025-11-02T20:39:25.761960Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_836E_3680, sum: Some(118699) }
2025-11-02T20:39:25.765279Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_7F6D_C860, sum: Some(19257) }
2025-11-02T20:39:25.780395Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_830C_9C00, sum: Some(2729) }
2025-11-02T20:39:25.795187Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_837F_7E00, sum: Some(45694) }
2025-11-02T20:39:25.799660Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_823B_B900, sum: Some(102652) }
2025-11-02T20:39:25.804782Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_7DAE_C0B0, sum: Some(89371) }
2025-11-02T20:39:25.812964Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_82E0_31C0, sum: Some(89146) }
2025-11-02T20:39:25.828716Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_83C0_9800, sum: Some(89169) }
2025-11-02T20:39:25.834218Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_825C_8400, sum: Some(8252) }
2025-11-02T20:39:25.839448Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_80F0_0228, sum: Some(24616) }
2025-11-02T20:39:25.844225Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_81BF_0360, sum: Some(9102) }
2025-11-02T20:39:25.862110Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_823B_B2C0, sum: Some(102990) }
2025-11-02T20:39:25.867630Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_7B7A_3F78, sum: Some(58305) }
2025-11-02T20:39:25.873270Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_836F_5B00, sum: Some(118924) }
2025-11-02T20:39:25.876180Z : ThreadId(27) allocator="nikka allocator (global cache)" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_83D0_8480, sum: Some(106866) }
2025-11-02T20:39:25.894611Z : ThreadId(24) allocator="nikka allocator (global cache)" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_829D_0080, sum: Some(36528) }
2025-11-02T20:39:25.901111Z : ThreadId(26) allocator="nikka allocator (global cache)" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_837F_8E00, sum: Some(29007) }
2025-11-02T20:39:25.902876Z i ThreadId(27) performance breakdown (smaller is better) allocator_name="nikka allocator (global cache)" allocation={ total: 9.585159ms, per_operation: 57.39ns, count: 167008 } allocation_zeroed={ total: 28.892787ms, per_operation: 173.43ns, count: 166592 } deallocation={ total: 8.167777ms, per_operation: 48.91ns, count: 167008 } reallocation={ total: 40.2105ms, per_operation: 120.78ns, count: 332936 } supplementary_code=259.084566ms total=345.940789ms
2025-11-02T20:39:25.907853Z : ThreadId(25) allocator="nikka allocator (global cache)" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_80AF_8990, sum: Some(128153) }
2025-11-02T20:39:25.921330Z i ThreadId(24) performance breakdown (smaller is better) allocator_name="nikka allocator (global cache)" allocation={ total: 10.552725ms, per_operation: 63.14ns, count: 167145 } allocation_zeroed={ total: 35.304777ms, per_operation: 212.04ns, count: 166501 } deallocation={ total: 6.322275ms, per_operation: 37.83ns, count: 167145 } reallocation={ total: 36.466554ms, per_operation: 109.36ns, count: 333462 } supplementary_code=275.919685ms total=364.566016ms
2025-11-02T20:39:25.930020Z i ThreadId(26) performance breakdown (smaller is better) allocator_name="nikka allocator (global cache)" allocation={ total: 11.794472ms, per_operation: 70.50ns, count: 167308 } allocation_zeroed={ total: 16.127606ms, per_operation: 96.66ns, count: 166850 } deallocation={ total: 7.107256ms, per_operation: 42.48ns, count: 167308 } reallocation={ total: 53.388627ms, per_operation: 159.99ns, count: 333702 } supplementary_code=284.717461ms total=373.135422ms
2025-11-02T20:39:25.937126Z i ThreadId(25) performance breakdown (smaller is better) allocator_name="nikka allocator (global cache)" allocation={ total: 8.421947ms, per_operation: 50.27ns, count: 167532 } allocation_zeroed={ total: 13.764207ms, per_operation: 82.42ns, count: 167008 } deallocation={ total: 8.310528ms, per_operation: 49.61ns, count: 167532 } reallocation={ total: 58.455431ms, per_operation: 175.46ns, count: 333163 } supplementary_code=291.355373ms total=380.307486ms
2025-11-02T20:39:25.980520Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_785C_4000, sum: Some(119266) }
2025-11-02T20:39:25.986050Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_7AB1_7C00, sum: Some(46851) }
2025-11-02T20:39:25.987629Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_78DF_CA80, sum: Some(53480) }
2025-11-02T20:39:25.988187Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_797E_7EE0, sum: Some(64941) }
2025-11-02T20:39:26.015919Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_7A1C_E000, sum: Some(32042) }
2025-11-02T20:39:26.021739Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_759E_8C40, sum: Some(30652) }
2025-11-02T20:39:26.023382Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_781B_8770, sum: Some(129154) }
2025-11-02T20:39:26.030848Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_7A1C_A000, sum: Some(23391) }
2025-11-02T20:39:26.051409Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_7AB7_AC00, sum: Some(43353) }
2025-11-02T20:39:26.063740Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_796E_3A80, sum: Some(39864) }
2025-11-02T20:39:26.067217Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_789D_4C80, sum: Some(85075) }
2025-11-02T20:39:26.076567Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_748E_AAF0, sum: Some(67833) }
2025-11-02T20:39:26.087370Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_73F0_0000, sum: Some(88118) }
2025-11-02T20:39:26.105183Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_7AB2_F800, sum: Some(75290) }
2025-11-02T20:39:26.111351Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_7AB5_0800, sum: Some(116343) }
2025-11-02T20:39:26.114002Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_73BF_1270, sum: Some(13436) }
2025-11-02T20:39:26.122136Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_74AF_10E0, sum: Some(19257) }
2025-11-02T20:39:26.144035Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_791D_E400, sum: Some(118699) }
2025-11-02T20:39:26.160554Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_7AA1_9C00, sum: Some(61803) }
2025-11-02T20:39:26.161418Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_786C_7320, sum: Some(102652) }
2025-11-02T20:39:26.170063Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_75CF_3CC0, sum: Some(5020) }
2025-11-02T20:39:26.184331Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_779D_AE98, sum: Some(89371) }
2025-11-02T20:39:26.197101Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_7A1C_9B00, sum: Some(8252) }
2025-11-02T20:39:26.203626Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_7A6D_CD00, sum: Some(2729) }
2025-11-02T20:39:26.206771Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_7AA1_C600, sum: Some(45694) }
2025-11-02T20:39:26.224748Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_799E_D170, sum: Some(24616) }
2025-11-02T20:39:26.238981Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_72BF_62E8, sum: Some(58305) }
2025-11-02T20:39:26.253318Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_789D_3100, sum: Some(89146) }
2025-11-02T20:39:26.265065Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_791D_D140, sum: Some(118924) }
2025-11-02T20:39:26.265661Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_7AB1_6C00, sum: Some(89169) }
2025-11-02T20:39:26.277128Z : ThreadId(30) allocator="nikka allocator (no cache)" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_7AA0_A800, sum: Some(29007) }
2025-11-02T20:39:26.293310Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_76F0_0180, sum: Some(9102) }
2025-11-02T20:39:26.303368Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_786C_85E0, sum: Some(102990) }
2025-11-02T20:39:26.304738Z : ThreadId(29) allocator="nikka allocator (no cache)" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_798E_A7E0, sum: Some(128153) }
2025-11-02T20:39:26.310268Z i ThreadId(30) performance breakdown (smaller is better) allocator_name="nikka allocator (no cache)" allocation={ total: 13.718043ms, per_operation: 81.99ns, count: 167308 } allocation_zeroed={ total: 18.301436ms, per_operation: 109.69ns, count: 166850 } deallocation={ total: 9.604564ms, per_operation: 57.41ns, count: 167308 } reallocation={ total: 57.96143ms, per_operation: 173.69ns, count: 333702 } supplementary_code=272.211184ms total=371.796657ms
2025-11-02T20:39:26.334525Z : ThreadId(31) allocator="nikka allocator (no cache)" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_7ACF_69C0, sum: Some(106866) }
2025-11-02T20:39:26.336160Z i ThreadId(29) performance breakdown (smaller is better) allocator_name="nikka allocator (no cache)" allocation={ total: 15.837168ms, per_operation: 94.53ns, count: 167532 } allocation_zeroed={ total: 16.553999ms, per_operation: 99.12ns, count: 167008 } deallocation={ total: 11.896624ms, per_operation: 71.01ns, count: 167532 } reallocation={ total: 55.066753ms, per_operation: 165.28ns, count: 333163 } supplementary_code=298.480428ms total=397.834972ms
2025-11-02T20:39:26.339686Z : ThreadId(28) allocator="nikka allocator (no cache)" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_7A0C_CF80, sum: Some(36528) }
2025-11-02T20:39:26.360840Z i ThreadId(31) performance breakdown (smaller is better) allocator_name="nikka allocator (no cache)" allocation={ total: 18.192847ms, per_operation: 108.93ns, count: 167008 } allocation_zeroed={ total: 20.169128ms, per_operation: 121.07ns, count: 166592 } deallocation={ total: 13.34914ms, per_operation: 79.93ns, count: 167008 } reallocation={ total: 56.291243ms, per_operation: 169.08ns, count: 332936 } supplementary_code=314.157521ms total=422.159879ms
2025-11-02T20:39:26.366674Z i ThreadId(28) performance breakdown (smaller is better) allocator_name="nikka allocator (no cache)" allocation={ total: 14.767321ms, per_operation: 88.35ns, count: 167145 } allocation_zeroed={ total: 24.59111ms, per_operation: 147.69ns, count: 166501 } deallocation={ total: 10.009112ms, per_operation: 59.88ns, count: 167145 } reallocation={ total: 59.62739ms, per_operation: 178.81ns, count: 333462 } supplementary_code=319.531377ms total=428.52631ms
2025-11-02T20:39:26.403620Z : ThreadId(33) allocator="system allocator" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_8402_3680, sum: Some(53480) }
2025-11-02T20:39:26.404707Z : ThreadId(34) allocator="system allocator" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_9003_BFF0, sum: Some(119266) }
2025-11-02T20:39:26.407355Z : ThreadId(35) allocator="system allocator" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_9801_D500, sum: Some(64941) }
2025-11-02T20:39:26.407614Z : ThreadId(32) allocator="system allocator" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_8C03_C000, sum: Some(46851) }
2025-11-02T20:39:26.440324Z : ThreadId(33) allocator="system allocator" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_8405_4F00, sum: Some(23391) }
2025-11-02T20:39:26.441425Z : ThreadId(34) allocator="system allocator" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_9002_29B0, sum: Some(32042) }
2025-11-02T20:39:26.445843Z : ThreadId(35) allocator="system allocator" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_9809_F680, sum: Some(129154) }
2025-11-02T20:39:26.446411Z : ThreadId(32) allocator="system allocator" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_8C02_ED10, sum: Some(30652) }
2025-11-02T20:39:26.477065Z : ThreadId(33) allocator="system allocator" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_8407_5070, sum: Some(85075) }
2025-11-02T20:39:26.480470Z : ThreadId(34) allocator="system allocator" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_9006_2800, sum: Some(43353) }
2025-11-02T20:39:26.484152Z : ThreadId(35) allocator="system allocator" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_980B_8E00, sum: Some(39864) }
2025-11-02T20:39:26.485257Z : ThreadId(32) allocator="system allocator" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_8C08_5620, sum: Some(67833) }
2025-11-02T20:39:26.514671Z : ThreadId(33) allocator="system allocator" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_8409_8000, sum: Some(75290) }
2025-11-02T20:39:26.534858Z : ThreadId(34) allocator="system allocator" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_9005_A400, sum: Some(88118) }
2025-11-02T20:39:26.534958Z : ThreadId(35) allocator="system allocator" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_9805_8C00, sum: Some(116343) }
2025-11-02T20:39:26.541101Z : ThreadId(32) allocator="system allocator" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_8C0B_A510, sum: Some(13436) }
2025-11-02T20:39:26.552200Z : ThreadId(33) allocator="system allocator" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_8402_97C0, sum: Some(118699) }
2025-11-02T20:39:26.573525Z : ThreadId(35) allocator="system allocator" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_9805_A580, sum: Some(61803) }
2025-11-02T20:39:26.583897Z : ThreadId(34) allocator="system allocator" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_900D_7B30, sum: Some(19257) }
2025-11-02T20:39:26.585955Z : ThreadId(32) allocator="system allocator" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_8C0A_0EC0, sum: Some(5020) }
2025-11-02T20:39:26.589235Z : ThreadId(33) allocator="system allocator" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_8409_E730, sum: Some(89371) }
2025-11-02T20:39:26.612930Z : ThreadId(35) allocator="system allocator" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_9802_0580, sum: Some(2729) }
2025-11-02T20:39:26.626882Z : ThreadId(33) allocator="system allocator" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_8415_90F0, sum: Some(24616) }
2025-11-02T20:39:26.629177Z : ThreadId(32) allocator="system allocator" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_8C08_9C00, sum: Some(45694) }
2025-11-02T20:39:26.645090Z : ThreadId(34) allocator="system allocator" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_9012_ECC0, sum: Some(102652) }
2025-11-02T20:39:26.652028Z : ThreadId(35) allocator="system allocator" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_9807_5620, sum: Some(89146) }
2025-11-02T20:39:26.665717Z : ThreadId(33) allocator="system allocator" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_8412_96E0, sum: Some(118924) }
2025-11-02T20:39:26.669677Z : ThreadId(32) allocator="system allocator" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_8C04_6E00, sum: Some(89169) }
2025-11-02T20:39:26.692244Z : ThreadId(35) allocator="system allocator" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_9809_9560, sum: Some(9102) }
2025-11-02T20:39:26.695463Z : ThreadId(34) allocator="system allocator" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_9012_A000, sum: Some(8252) }
2025-11-02T20:39:26.712466Z : ThreadId(33) allocator="system allocator" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_8409_27C0, sum: Some(128153) }
2025-11-02T20:39:26.712809Z : ThreadId(32) allocator="system allocator" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_8C04_D200, sum: Some(102990) }
2025-11-02T20:39:26.731587Z : ThreadId(35) allocator="system allocator" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_9802_EE00, sum: Some(106866) }
2025-11-02T20:39:26.746035Z i ThreadId(33) performance breakdown (smaller is better) allocator_name="system allocator" allocation={ total: 20.743645ms, per_operation: 123.82ns, count: 167532 } allocation_zeroed={ total: 26.042838ms, per_operation: 155.94ns, count: 167008 } deallocation={ total: 7.757514ms, per_operation: 46.30ns, count: 167532 } reallocation={ total: 59.895371ms, per_operation: 179.78ns, count: 333163 } supplementary_code=263.903822ms total=378.34319ms
2025-11-02T20:39:26.746267Z : ThreadId(34) allocator="system allocator" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_9004_5EA0, sum: Some(58305) }
2025-11-02T20:39:26.754323Z : ThreadId(32) allocator="system allocator" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_8C0B_5B00, sum: Some(36528) }
2025-11-02T20:39:26.763681Z i ThreadId(35) performance breakdown (smaller is better) allocator_name="system allocator" allocation={ total: 19.628109ms, per_operation: 117.53ns, count: 167008 } allocation_zeroed={ total: 28.518942ms, per_operation: 171.19ns, count: 166592 } deallocation={ total: 7.623992ms, per_operation: 45.65ns, count: 167008 } reallocation={ total: 62.55506ms, per_operation: 187.89ns, count: 332936 } supplementary_code=277.55859ms total=395.884693ms
2025-11-02T20:39:26.784451Z : ThreadId(34) allocator="system allocator" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_9003_D400, sum: Some(29007) }
2025-11-02T20:39:26.786952Z i ThreadId(32) performance breakdown (smaller is better) allocator_name="system allocator" allocation={ total: 19.26524ms, per_operation: 115.26ns, count: 167145 } allocation_zeroed={ total: 27.147045ms, per_operation: 163.04ns, count: 166501 } deallocation={ total: 7.510653ms, per_operation: 44.93ns, count: 167145 } reallocation={ total: 65.693852ms, per_operation: 197.01ns, count: 333462 } supplementary_code=299.760528ms total=419.377318ms
2025-11-02T20:39:26.818428Z i ThreadId(34) performance breakdown (smaller is better) allocator_name="system allocator" allocation={ total: 19.298493ms, per_operation: 115.35ns, count: 167308 } allocation_zeroed={ total: 37.987716ms, per_operation: 227.68ns, count: 166850 } deallocation={ total: 9.678622ms, per_operation: 57.85ns, count: 167308 } reallocation={ total: 79.834315ms, per_operation: 239.24ns, count: 333702 } supplementary_code=303.876116ms total=450.675262ms
2025-11-02T20:39:26.986748Z : ThreadId(36) allocator="dlmalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A62F_5400, sum: Some(46851) }
2025-11-02T20:39:26.997439Z : ThreadId(39) allocator="dlmalloc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_A62D_22A0, sum: Some(64941) }
2025-11-02T20:39:27.019964Z : ThreadId(37) allocator="dlmalloc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_A62E_8640, sum: Some(53480) }
2025-11-02T20:39:27.020400Z : ThreadId(38) allocator="dlmalloc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_A62E_AB00, sum: Some(119266) }
2025-11-02T20:39:27.159275Z : ThreadId(36) allocator="dlmalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A55C_FF10, sum: Some(30652) }
2025-11-02T20:39:27.181948Z : ThreadId(37) allocator="dlmalloc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_A55D_4500, sum: Some(23391) }
2025-11-02T20:39:27.196050Z : ThreadId(39) allocator="dlmalloc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_A625_5800, sum: Some(129154) }
2025-11-02T20:39:27.225144Z : ThreadId(38) allocator="dlmalloc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_A622_B600, sum: Some(32042) }
2025-11-02T20:39:27.347072Z : ThreadId(36) allocator="dlmalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A557_7AE0, sum: Some(67833) }
2025-11-02T20:39:27.369164Z : ThreadId(37) allocator="dlmalloc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_A62E_6E30, sum: Some(85075) }
2025-11-02T20:39:27.394902Z : ThreadId(39) allocator="dlmalloc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_A55C_4670, sum: Some(39864) }
2025-11-02T20:39:27.404684Z : ThreadId(38) allocator="dlmalloc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_A553_A000, sum: Some(43353) }
2025-11-02T20:39:27.527307Z : ThreadId(36) allocator="dlmalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A55B_3490, sum: Some(13436) }
2025-11-02T20:39:27.560559Z : ThreadId(37) allocator="dlmalloc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_A552_F600, sum: Some(75290) }
2025-11-02T20:39:27.589615Z : ThreadId(39) allocator="dlmalloc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_A625_2600, sum: Some(116343) }
2025-11-02T20:39:27.590486Z : ThreadId(38) allocator="dlmalloc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_A550_F530, sum: Some(88118) }
2025-11-02T20:39:27.718900Z : ThreadId(36) allocator="dlmalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A54B_B6C0, sum: Some(5020) }
2025-11-02T20:39:27.752973Z : ThreadId(37) allocator="dlmalloc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_A62A_6F00, sum: Some(118699) }
2025-11-02T20:39:27.769120Z : ThreadId(39) allocator="dlmalloc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_A62D_A9A0, sum: Some(61803) }
2025-11-02T20:39:27.790418Z : ThreadId(38) allocator="dlmalloc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_A55E_EC60, sum: Some(19257) }
2025-11-02T20:39:27.906521Z : ThreadId(36) allocator="dlmalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A626_1D00, sum: Some(45694) }
2025-11-02T20:39:27.941144Z : ThreadId(37) allocator="dlmalloc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_A550_7D40, sum: Some(89371) }
2025-11-02T20:39:27.967492Z : ThreadId(39) allocator="dlmalloc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_A55C_9700, sum: Some(2729) }
2025-11-02T20:39:27.976841Z : ThreadId(38) allocator="dlmalloc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_A62D_08C0, sum: Some(102652) }
2025-11-02T20:39:28.109392Z : ThreadId(36) allocator="dlmalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A625_9C00, sum: Some(89169) }
2025-11-02T20:39:28.127529Z : ThreadId(37) allocator="dlmalloc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_A550_CAE0, sum: Some(24616) }
2025-11-02T20:39:28.164323Z : ThreadId(38) allocator="dlmalloc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_A55F_A700, sum: Some(8252) }
2025-11-02T20:39:28.166863Z : ThreadId(39) allocator="dlmalloc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_A548_73D0, sum: Some(89146) }
2025-11-02T20:39:28.305361Z : ThreadId(36) allocator="dlmalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A556_2310, sum: Some(102990) }
2025-11-02T20:39:28.330066Z : ThreadId(37) allocator="dlmalloc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_A62D_5C00, sum: Some(118924) }
2025-11-02T20:39:28.343656Z : ThreadId(38) allocator="dlmalloc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_A62A_5F60, sum: Some(58305) }
2025-11-02T20:39:28.353585Z : ThreadId(39) allocator="dlmalloc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_A623_1DA0, sum: Some(9102) }
2025-11-02T20:39:28.514298Z : ThreadId(36) allocator="dlmalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A553_0280, sum: Some(36528) }
2025-11-02T20:39:28.522831Z : ThreadId(37) allocator="dlmalloc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_A623_A400, sum: Some(128153) }
2025-11-02T20:39:28.533149Z : ThreadId(38) allocator="dlmalloc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_A550_4600, sum: Some(29007) }
2025-11-02T20:39:28.553208Z : ThreadId(39) allocator="dlmalloc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_A556_F340, sum: Some(106866) }
2025-11-02T20:39:28.677844Z i ThreadId(36) performance breakdown (smaller is better) allocator_name="dlmalloc" allocation={ total: 254.637457ms, per_operation: 1523.45ns, count: 167145 } allocation_zeroed={ total: 261.49282ms, per_operation: 1570.52ns, count: 166501 } deallocation={ total: 220.578899ms, per_operation: 1319.69ns, count: 167145 } reallocation={ total: 573.443925ms, per_operation: 1719.67ns, count: 333462 } supplementary_code=549.008551ms total=1.859161652s
2025-11-02T20:39:28.686029Z i ThreadId(38) performance breakdown (smaller is better) allocator_name="dlmalloc" allocation={ total: 256.286209ms, per_operation: 1531.82ns, count: 167308 } allocation_zeroed={ total: 270.962115ms, per_operation: 1623.99ns, count: 166850 } deallocation={ total: 218.981236ms, per_operation: 1308.85ns, count: 167308 } reallocation={ total: 568.976915ms, per_operation: 1705.04ns, count: 333702 } supplementary_code=552.034797ms total=1.867241272s
2025-11-02T20:39:28.699829Z i ThreadId(37) performance breakdown (smaller is better) allocator_name="dlmalloc" allocation={ total: 242.531373ms, per_operation: 1447.67ns, count: 167532 } allocation_zeroed={ total: 270.926885ms, per_operation: 1622.24ns, count: 167008 } deallocation={ total: 221.926443ms, per_operation: 1324.68ns, count: 167532 } reallocation={ total: 586.955103ms, per_operation: 1761.77ns, count: 333163 } supplementary_code=558.766711ms total=1.881106515s
2025-11-02T20:39:28.699953Z i ThreadId(39) performance breakdown (smaller is better) allocator_name="dlmalloc" allocation={ total: 236.675102ms, per_operation: 1417.15ns, count: 167008 } allocation_zeroed={ total: 281.997264ms, per_operation: 1692.74ns, count: 166592 } deallocation={ total: 233.188679ms, per_operation: 1396.27ns, count: 167008 } reallocation={ total: 585.195224ms, per_operation: 1757.68ns, count: 332936 } supplementary_code=544.085885ms total=1.881142154s
2025-11-02T20:39:28.738269Z : ThreadId(40) allocator="blink" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_8403_8000, sum: Some(46851) }
2025-11-02T20:39:28.750893Z : ThreadId(42) allocator="blink" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_9805_83F0, sum: Some(119266) }
2025-11-02T20:39:28.771185Z : ThreadId(43) allocator="blink" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_9002_5DA0, sum: Some(64941) }
2025-11-02T20:39:28.772631Z : ThreadId(41) allocator="blink" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_8C05_F680, sum: Some(53480) }
2025-11-02T20:39:28.777927Z : ThreadId(40) allocator="blink" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_8405_9670, sum: Some(30652) }
2025-11-02T20:39:28.801852Z : ThreadId(42) allocator="blink" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_9807_F400, sum: Some(32042) }
2025-11-02T20:39:28.812124Z : ThreadId(41) allocator="blink" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_8C08_3500, sum: Some(23391) }
2025-11-02T20:39:28.818394Z : ThreadId(40) allocator="blink" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_8407_9520, sum: Some(67833) }
2025-11-02T20:39:28.822123Z : ThreadId(43) allocator="blink" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_900B_F0C0, sum: Some(129154) }
2025-11-02T20:39:28.840446Z : ThreadId(42) allocator="blink" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_980B_1800, sum: Some(43353) }
2025-11-02T20:39:28.851451Z : ThreadId(41) allocator="blink" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_8C02_8930, sum: Some(85075) }
2025-11-02T20:39:28.858030Z : ThreadId(40) allocator="blink" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_840E_1E90, sum: Some(13436) }
2025-11-02T20:39:28.862065Z : ThreadId(43) allocator="blink" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_9003_6A30, sum: Some(39864) }
2025-11-02T20:39:28.891845Z : ThreadId(41) allocator="blink" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_8C0E_2400, sum: Some(75290) }
2025-11-02T20:39:28.894175Z : ThreadId(42) allocator="blink" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_9804_0200, sum: Some(88118) }
2025-11-02T20:39:28.901574Z : ThreadId(43) allocator="blink" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_9003_3600, sum: Some(116343) }
2025-11-02T20:39:28.908053Z : ThreadId(40) allocator="blink" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_840A_A800, sum: Some(5020) }
2025-11-02T20:39:28.933644Z : ThreadId(41) allocator="blink" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_8C03_3680, sum: Some(118699) }
2025-11-02T20:39:28.936326Z : ThreadId(42) allocator="blink" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_9807_A560, sum: Some(19257) }
2025-11-02T20:39:28.941475Z : ThreadId(43) allocator="blink" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_9003_5A80, sum: Some(61803) }
2025-11-02T20:39:28.958519Z : ThreadId(40) allocator="blink" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_8404_2E00, sum: Some(45694) }
2025-11-02T20:39:28.981611Z : ThreadId(43) allocator="blink" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_900B_6A80, sum: Some(2729) }
2025-11-02T20:39:28.989033Z : ThreadId(41) allocator="blink" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_8C05_A3F0, sum: Some(89371) }
2025-11-02T20:39:29.004943Z : ThreadId(40) allocator="blink" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_840C_3C00, sum: Some(89169) }
2025-11-02T20:39:29.006249Z : ThreadId(42) allocator="blink" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_9806_3080, sum: Some(102652) }
2025-11-02T20:39:29.022590Z : ThreadId(43) allocator="blink" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_9001_7440, sum: Some(89146) }
2025-11-02T20:39:29.034080Z : ThreadId(41) allocator="blink" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_8C15_5400, sum: Some(24616) }
2025-11-02T20:39:29.059243Z : ThreadId(40) allocator="blink" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_840E_FC00, sum: Some(102990) }
2025-11-02T20:39:29.061398Z : ThreadId(43) allocator="blink" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_9005_FC60, sum: Some(9102) }
2025-11-02T20:39:29.065688Z : ThreadId(42) allocator="blink" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_9800_8100, sum: Some(8252) }
2025-11-02T20:39:29.075256Z : ThreadId(41) allocator="blink" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_8C16_0BA0, sum: Some(118924) }
2025-11-02T20:39:29.100776Z : ThreadId(43) allocator="blink" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_9001_CF40, sum: Some(106866) }
2025-11-02T20:39:29.107969Z : ThreadId(42) allocator="blink" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_980E_35C0, sum: Some(58305) }
2025-11-02T20:39:29.109310Z : ThreadId(40) allocator="blink" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_8411_EA80, sum: Some(36528) }
2025-11-02T20:39:29.121636Z : ThreadId(41) allocator="blink" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_8C18_4F40, sum: Some(128153) }
2025-11-02T20:39:29.133982Z i ThreadId(43) performance breakdown (smaller is better) allocator_name="blink" allocation={ total: 21.699054ms, per_operation: 129.93ns, count: 167008 } allocation_zeroed={ total: 32.625018ms, per_operation: 195.84ns, count: 166592 } deallocation={ total: 7.924127ms, per_operation: 47.45ns, count: 167008 } reallocation={ total: 66.978632ms, per_operation: 201.18ns, count: 332936 } supplementary_code=303.888192ms total=433.115023ms
2025-11-02T20:39:29.145310Z i ThreadId(40) performance breakdown (smaller is better) allocator_name="blink" allocation={ total: 25.971155ms, per_operation: 155.38ns, count: 167145 } allocation_zeroed={ total: 27.10656ms, per_operation: 162.80ns, count: 166501 } deallocation={ total: 6.958535ms, per_operation: 41.63ns, count: 167145 } reallocation={ total: 71.009924ms, per_operation: 212.95ns, count: 333462 } supplementary_code=314.151553ms total=445.197727ms
2025-11-02T20:39:29.149524Z : ThreadId(42) allocator="blink" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_9816_8A00, sum: Some(29007) }
2025-11-02T20:39:29.156709Z i ThreadId(41) performance breakdown (smaller is better) allocator_name="blink" allocation={ total: 23.998069ms, per_operation: 143.24ns, count: 167532 } allocation_zeroed={ total: 34.003344ms, per_operation: 203.60ns, count: 167008 } deallocation={ total: 10.166337ms, per_operation: 60.68ns, count: 167532 } reallocation={ total: 87.919399ms, per_operation: 263.89ns, count: 333163 } supplementary_code=300.329749ms total=456.416898ms
2025-11-02T20:39:29.183774Z i ThreadId(42) performance breakdown (smaller is better) allocator_name="blink" allocation={ total: 23.831842ms, per_operation: 142.44ns, count: 167308 } allocation_zeroed={ total: 34.249908ms, per_operation: 205.27ns, count: 166850 } deallocation={ total: 13.811907ms, per_operation: 82.55ns, count: 167308 } reallocation={ total: 75.37526ms, per_operation: 225.88ns, count: 333702 } supplementary_code=335.96778ms total=483.236697ms
2025-11-02T20:39:29.228554Z : ThreadId(46) allocator="frusa" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_8963_1800, sum: Some(119266) }
2025-11-02T20:39:29.233237Z : ThreadId(47) allocator="frusa" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_8960_F000, sum: Some(64941) }
2025-11-02T20:39:29.240779Z : ThreadId(45) allocator="frusa" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_8966_7A00, sum: Some(53480) }
2025-11-02T20:39:29.246277Z : ThreadId(44) allocator="frusa" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_895F_5800, sum: Some(46851) }
2025-11-02T20:39:29.279344Z : ThreadId(46) allocator="frusa" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_895F_9800, sum: Some(32042) }
2025-11-02T20:39:29.282454Z : ThreadId(47) allocator="frusa" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_895E_8000, sum: Some(129154) }
2025-11-02T20:39:29.296125Z : ThreadId(45) allocator="frusa" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_895A_4B00, sum: Some(23391) }
2025-11-02T20:39:29.306972Z : ThreadId(44) allocator="frusa" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_894D_0A00, sum: Some(30652) }
2025-11-02T20:39:29.332252Z : ThreadId(47) allocator="frusa" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_894E_9A00, sum: Some(39864) }
2025-11-02T20:39:29.334010Z : ThreadId(46) allocator="frusa" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_8964_C000, sum: Some(43353) }
2025-11-02T20:39:29.355976Z : ThreadId(44) allocator="frusa" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_895C_0C00, sum: Some(67833) }
2025-11-02T20:39:29.375075Z : ThreadId(45) allocator="frusa" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_8950_9000, sum: Some(85075) }
2025-11-02T20:39:29.380880Z : ThreadId(47) allocator="frusa" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_8959_B800, sum: Some(116343) }
2025-11-02T20:39:29.405401Z : ThreadId(44) allocator="frusa" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_894C_FA00, sum: Some(13436) }
2025-11-02T20:39:29.417535Z : ThreadId(46) allocator="frusa" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_894A_A800, sum: Some(88118) }
2025-11-02T20:39:29.432794Z : ThreadId(47) allocator="frusa" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_8947_5000, sum: Some(61803) }
2025-11-02T20:39:29.433571Z : ThreadId(45) allocator="frusa" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_8961_4000, sum: Some(75290) }
2025-11-02T20:39:29.458022Z : ThreadId(44) allocator="frusa" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_8963_35C0, sum: Some(5020) }
2025-11-02T20:39:29.486688Z : ThreadId(47) allocator="frusa" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_8955_AC00, sum: Some(2729) }
2025-11-02T20:39:29.493105Z : ThreadId(45) allocator="frusa" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_8943_5400, sum: Some(118699) }
2025-11-02T20:39:29.495738Z : ThreadId(46) allocator="frusa" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_895F_7400, sum: Some(19257) }
2025-11-02T20:39:29.517115Z : ThreadId(44) allocator="frusa" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_8955_2E00, sum: Some(45694) }
2025-11-02T20:39:29.539058Z : ThreadId(47) allocator="frusa" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_8944_8800, sum: Some(89146) }
2025-11-02T20:39:29.551780Z : ThreadId(45) allocator="frusa" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_8953_0400, sum: Some(89371) }
2025-11-02T20:39:29.556518Z : ThreadId(46) allocator="frusa" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_8959_9400, sum: Some(102652) }
2025-11-02T20:39:29.581137Z : ThreadId(44) allocator="frusa" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_894D_9800, sum: Some(89169) }
2025-11-02T20:39:29.595287Z : ThreadId(47) allocator="frusa" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_8955_A680, sum: Some(9102) }
2025-11-02T20:39:29.612119Z : ThreadId(45) allocator="frusa" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_8942_8700, sum: Some(24616) }
2025-11-02T20:39:29.626738Z : ThreadId(46) allocator="frusa" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_8953_D000, sum: Some(8252) }
2025-11-02T20:39:29.648029Z : ThreadId(44) allocator="frusa" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_8945_8400, sum: Some(102990) }
2025-11-02T20:39:29.652977Z : ThreadId(47) allocator="frusa" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_8944_5800, sum: Some(106866) }
2025-11-02T20:39:29.674338Z : ThreadId(45) allocator="frusa" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_895D_C400, sum: Some(118924) }
2025-11-02T20:39:29.690742Z : ThreadId(46) allocator="frusa" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_8947_E000, sum: Some(58305) }
2025-11-02T20:39:29.701110Z i ThreadId(47) performance breakdown (smaller is better) allocator_name="frusa" allocation={ total: 30.680425ms, per_operation: 183.71ns, count: 167008 } allocation_zeroed={ total: 39.022338ms, per_operation: 234.24ns, count: 166592 } deallocation={ total: 30.244197ms, per_operation: 181.09ns, count: 167008 } reallocation={ total: 127.427943ms, per_operation: 382.74ns, count: 332936 } supplementary_code=289.565801ms total=516.940704ms
2025-11-02T20:39:29.713775Z : ThreadId(44) allocator="frusa" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_894B_D000, sum: Some(36528) }
2025-11-02T20:39:29.732366Z : ThreadId(45) allocator="frusa" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_8944_A400, sum: Some(128153) }
2025-11-02T20:39:29.747643Z : ThreadId(46) allocator="frusa" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_895E_0400, sum: Some(29007) }
2025-11-02T20:39:29.757984Z i ThreadId(44) performance breakdown (smaller is better) allocator_name="frusa" allocation={ total: 34.483916ms, per_operation: 206.31ns, count: 167145 } allocation_zeroed={ total: 41.222325ms, per_operation: 247.58ns, count: 166501 } deallocation={ total: 35.00053ms, per_operation: 209.40ns, count: 167145 } reallocation={ total: 143.630999ms, per_operation: 430.73ns, count: 333462 } supplementary_code=319.626499ms total=573.964269ms
2025-11-02T20:39:29.778559Z i ThreadId(45) performance breakdown (smaller is better) allocator_name="frusa" allocation={ total: 38.5945ms, per_operation: 230.37ns, count: 167532 } allocation_zeroed={ total: 44.441208ms, per_operation: 266.10ns, count: 167008 } deallocation={ total: 31.860966ms, per_operation: 190.18ns, count: 167532 } reallocation={ total: 150.163036ms, per_operation: 450.72ns, count: 333163 } supplementary_code=329.434264ms total=594.493974ms
2025-11-02T20:39:29.787576Z i ThreadId(46) performance breakdown (smaller is better) allocator_name="frusa" allocation={ total: 40.692589ms, per_operation: 243.22ns, count: 167308 } allocation_zeroed={ total: 39.617356ms, per_operation: 237.44ns, count: 166850 } deallocation={ total: 34.614979ms, per_operation: 206.89ns, count: 167308 } reallocation={ total: 150.624778ms, per_operation: 451.38ns, count: 333702 } supplementary_code=337.920638ms total=603.47034ms
2025-11-02T20:39:29.902444Z : ThreadId(50) allocator="rlsf" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_A40F_D0F0, sum: Some(119266) }
2025-11-02T20:39:29.904398Z : ThreadId(48) allocator="rlsf" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A410_9800, sum: Some(46851) }
2025-11-02T20:39:29.905577Z : ThreadId(51) allocator="rlsf" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_A411_8FA0, sum: Some(64941) }
2025-11-02T20:39:29.907524Z : ThreadId(49) allocator="rlsf" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_A417_01C0, sum: Some(53480) }
2025-11-02T20:39:30.008773Z : ThreadId(51) allocator="rlsf" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_A40B_96B0, sum: Some(129154) }
2025-11-02T20:39:30.018178Z : ThreadId(50) allocator="rlsf" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_A415_4F50, sum: Some(32042) }
2025-11-02T20:39:30.027541Z : ThreadId(48) allocator="rlsf" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A40F_FCB0, sum: Some(30652) }
2025-11-02T20:39:30.041310Z : ThreadId(49) allocator="rlsf" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_A40B_6A00, sum: Some(23391) }
2025-11-02T20:39:30.101742Z : ThreadId(51) allocator="rlsf" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_A412_6F50, sum: Some(39864) }
2025-11-02T20:39:30.123943Z : ThreadId(50) allocator="rlsf" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_A403_3400, sum: Some(43353) }
2025-11-02T20:39:30.151906Z : ThreadId(48) allocator="rlsf" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A413_3A50, sum: Some(67833) }
2025-11-02T20:39:30.181649Z : ThreadId(49) allocator="rlsf" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_A40A_C190, sum: Some(85075) }
2025-11-02T20:39:30.202303Z : ThreadId(51) allocator="rlsf" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_A411_4200, sum: Some(116343) }
2025-11-02T20:39:30.249756Z : ThreadId(50) allocator="rlsf" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_9FDE_77D0, sum: Some(88118) }
2025-11-02T20:39:30.252287Z : ThreadId(48) allocator="rlsf" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A415_4B30, sum: Some(13436) }
2025-11-02T20:39:30.316657Z : ThreadId(49) allocator="rlsf" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_A411_6000, sum: Some(75290) }
2025-11-02T20:39:30.326111Z : ThreadId(51) allocator="rlsf" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_A417_80D0, sum: Some(61803) }
2025-11-02T20:39:30.352127Z : ThreadId(48) allocator="rlsf" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_9FD8_0180, sum: Some(5020) }
2025-11-02T20:39:30.371560Z : ThreadId(50) allocator="rlsf" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_9FD6_5DB0, sum: Some(19257) }
2025-11-02T20:39:30.434167Z : ThreadId(49) allocator="rlsf" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_A408_0E00, sum: Some(118699) }
2025-11-02T20:39:30.455468Z : ThreadId(48) allocator="rlsf" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_9FD1_C900, sum: Some(45694) }
2025-11-02T20:39:30.462161Z : ThreadId(51) allocator="rlsf" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_A416_7400, sum: Some(2729) }
2025-11-02T20:39:30.480239Z : ThreadId(50) allocator="rlsf" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_A408_5D00, sum: Some(102652) }
2025-11-02T20:39:30.560318Z : ThreadId(49) allocator="rlsf" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_9FD2_3F90, sum: Some(89371) }
2025-11-02T20:39:30.571835Z : ThreadId(51) allocator="rlsf" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_A414_F6F0, sum: Some(89146) }
2025-11-02T20:39:30.580669Z : ThreadId(48) allocator="rlsf" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_9FDA_EA00, sum: Some(89169) }
2025-11-02T20:39:30.595205Z : ThreadId(50) allocator="rlsf" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_A408_6800, sum: Some(8252) }
2025-11-02T20:39:30.673754Z : ThreadId(51) allocator="rlsf" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_A418_81C0, sum: Some(9102) }
2025-11-02T20:39:30.697454Z : ThreadId(49) allocator="rlsf" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_9FD5_1050, sum: Some(24616) }
2025-11-02T20:39:30.707419Z : ThreadId(48) allocator="rlsf" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_9FCA_AD70, sum: Some(102990) }
2025-11-02T20:39:30.710645Z : ThreadId(50) allocator="rlsf" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_9FDC_D010, sum: Some(58305) }
2025-11-02T20:39:30.785014Z : ThreadId(51) allocator="rlsf" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_9FD7_6940, sum: Some(106866) }
2025-11-02T20:39:30.812423Z : ThreadId(49) allocator="rlsf" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_9FCB_6A40, sum: Some(118924) }
2025-11-02T20:39:30.823058Z : ThreadId(48) allocator="rlsf" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A405_0400, sum: Some(36528) }
2025-11-02T20:39:30.846257Z : ThreadId(50) allocator="rlsf" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_A407_BA00, sum: Some(29007) }
2025-11-02T20:39:30.890436Z i ThreadId(51) performance breakdown (smaller is better) allocator_name="rlsf" allocation={ total: 125.924503ms, per_operation: 754.00ns, count: 167008 } allocation_zeroed={ total: 131.124984ms, per_operation: 787.10ns, count: 166592 } deallocation={ total: 126.677601ms, per_operation: 758.51ns, count: 167008 } reallocation={ total: 291.434073ms, per_operation: 875.35ns, count: 332936 } supplementary_code=427.091215ms total=1.102252376s
2025-11-02T20:39:30.906662Z i ThreadId(48) performance breakdown (smaller is better) allocator_name="rlsf" allocation={ total: 129.49878ms, per_operation: 774.77ns, count: 167145 } allocation_zeroed={ total: 136.741567ms, per_operation: 821.27ns, count: 166501 } deallocation={ total: 130.069371ms, per_operation: 778.18ns, count: 167145 } reallocation={ total: 284.672122ms, per_operation: 853.69ns, count: 333462 } supplementary_code=437.65787ms total=1.11863971s
2025-11-02T20:39:30.918125Z : ThreadId(49) allocator="rlsf" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_A417_C350, sum: Some(128153) }
2025-11-02T20:39:30.931191Z i ThreadId(50) performance breakdown (smaller is better) allocator_name="rlsf" allocation={ total: 130.060811ms, per_operation: 777.37ns, count: 167308 } allocation_zeroed={ total: 141.214789ms, per_operation: 846.36ns, count: 166850 } deallocation={ total: 131.370195ms, per_operation: 785.20ns, count: 167308 } reallocation={ total: 297.658754ms, per_operation: 891.99ns, count: 333702 } supplementary_code=442.749867ms total=1.143054416s
2025-11-02T20:39:30.954523Z i ThreadId(49) performance breakdown (smaller is better) allocator_name="rlsf" allocation={ total: 144.751918ms, per_operation: 864.03ns, count: 167532 } allocation_zeroed={ total: 141.069892ms, per_operation: 844.69ns, count: 167008 } deallocation={ total: 134.948707ms, per_operation: 805.51ns, count: 167532 } reallocation={ total: 293.218711ms, per_operation: 880.11ns, count: 333163 } supplementary_code=452.474366ms total=1.166463594s
2025-11-02T20:39:31.041864Z : ThreadId(55) allocator="talc" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7717_5200_FFE0, sum: Some(64941) }
2025-11-02T20:39:31.046785Z : ThreadId(53) allocator="talc" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7717_5202_A4C0, sum: Some(53480) }
2025-11-02T20:39:31.050402Z : ThreadId(54) allocator="talc" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_5208_FBE0, sum: Some(119266) }
2025-11-02T20:39:31.128651Z : ThreadId(55) allocator="talc" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_520E_7EB8, sum: Some(129154) }
2025-11-02T20:39:31.130772Z : ThreadId(52) allocator="talc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_5205_2C00, sum: Some(46851) }
2025-11-02T20:39:31.147546Z : ThreadId(53) allocator="talc" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7717_5203_9E00, sum: Some(23391) }
2025-11-02T20:39:31.152518Z : ThreadId(54) allocator="talc" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_520D_2F88, sum: Some(32042) }
2025-11-02T20:39:31.212176Z : ThreadId(55) allocator="talc" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7717_5202_69D0, sum: Some(39864) }
2025-11-02T20:39:31.245363Z : ThreadId(54) allocator="talc" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7717_5206_3000, sum: Some(43353) }
2025-11-02T20:39:31.295586Z : ThreadId(53) allocator="talc" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_5216_6798, sum: Some(85075) }
2025-11-02T20:39:31.299668Z : ThreadId(55) allocator="talc" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7717_5212_AE00, sum: Some(116343) }
2025-11-02T20:39:31.302948Z : ThreadId(52) allocator="talc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_5212_7DF0, sum: Some(30652) }
2025-11-02T20:39:31.359530Z : ThreadId(54) allocator="talc" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_5204_9F50, sum: Some(88118) }
2025-11-02T20:39:31.394617Z : ThreadId(55) allocator="talc" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_520D_7DB0, sum: Some(61803) }
2025-11-02T20:39:31.413267Z : ThreadId(52) allocator="talc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_5212_B180, sum: Some(67833) }
2025-11-02T20:39:31.425644Z : ThreadId(53) allocator="talc" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7717_5203_CA00, sum: Some(75290) }
2025-11-02T20:39:31.500286Z : ThreadId(55) allocator="talc" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7717_5207_5B00, sum: Some(2729) }
2025-11-02T20:39:31.507564Z : ThreadId(54) allocator="talc" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7717_520B_77E0, sum: Some(19257) }
2025-11-02T20:39:31.520637Z : ThreadId(52) allocator="talc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_5202_5B90, sum: Some(13436) }
2025-11-02T20:39:31.578655Z : ThreadId(53) allocator="talc" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7717_5216_FFC0, sum: Some(118699) }
2025-11-02T20:39:31.605576Z : ThreadId(55) allocator="talc" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_520C_A9B0, sum: Some(89146) }
2025-11-02T20:39:31.629138Z : ThreadId(52) allocator="talc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_521C_4A00, sum: Some(5020) }
2025-11-02T20:39:31.667504Z : ThreadId(54) allocator="talc" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7717_5211_1820, sum: Some(102652) }
2025-11-02T20:39:31.715689Z : ThreadId(55) allocator="talc" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7717_5207_D7A0, sum: Some(9102) }
2025-11-02T20:39:31.741701Z : ThreadId(52) allocator="talc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_520B_1400, sum: Some(45694) }
2025-11-02T20:39:31.750152Z : ThreadId(53) allocator="talc" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_5200_F520, sum: Some(89371) }
2025-11-02T20:39:31.800091Z : ThreadId(54) allocator="talc" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7717_5220_DC00, sum: Some(8252) }
2025-11-02T20:39:31.843192Z : ThreadId(55) allocator="talc" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7717_5214_C4C0, sum: Some(106866) }
2025-11-02T20:39:31.854813Z : ThreadId(52) allocator="talc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_5220_6200, sum: Some(89169) }
2025-11-02T20:39:31.895361Z : ThreadId(53) allocator="talc" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_5209_B338, sum: Some(24616) }
2025-11-02T20:39:31.926514Z : ThreadId(54) allocator="talc" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_5206_BD60, sum: Some(58305) }
2025-11-02T20:39:31.958574Z : ThreadId(52) allocator="talc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_5218_40D8, sum: Some(102990) }
2025-11-02T20:39:31.976864Z i ThreadId(55) performance breakdown (smaller is better) allocator_name="talc" allocation={ total: 138.044349ms, per_operation: 826.57ns, count: 167008 } allocation_zeroed={ total: 148.463998ms, per_operation: 891.18ns, count: 166592 } deallocation={ total: 95.860286ms, per_operation: 573.99ns, count: 167008 } reallocation={ total: 264.718504ms, per_operation: 795.10ns, count: 332936 } supplementary_code=374.118454ms total=1.021205591s
2025-11-02T20:39:32.012446Z : ThreadId(53) allocator="talc" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7717_520E_4460, sum: Some(118924) }
2025-11-02T20:39:32.025393Z : ThreadId(54) allocator="talc" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7717_5210_D000, sum: Some(29007) }
2025-11-02T20:39:32.055458Z : ThreadId(52) allocator="talc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_5220_9900, sum: Some(36528) }
2025-11-02T20:39:32.098987Z i ThreadId(54) performance breakdown (smaller is better) allocator_name="talc" allocation={ total: 155.348606ms, per_operation: 928.52ns, count: 167308 } allocation_zeroed={ total: 163.056242ms, per_operation: 977.26ns, count: 166850 } deallocation={ total: 118.312421ms, per_operation: 707.15ns, count: 167308 } reallocation={ total: 288.689957ms, per_operation: 865.11ns, count: 333702 } supplementary_code=418.665483ms total=1.144072709s
2025-11-02T20:39:32.101929Z : ThreadId(53) allocator="talc" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7717_5219_32D0, sum: Some(128153) }
2025-11-02T20:39:32.116568Z i ThreadId(52) performance breakdown (smaller is better) allocator_name="talc" allocation={ total: 148.956903ms, per_operation: 891.18ns, count: 167145 } allocation_zeroed={ total: 169.842261ms, per_operation: 1020.07ns, count: 166501 } deallocation={ total: 105.399565ms, per_operation: 630.59ns, count: 167145 } reallocation={ total: 270.031274ms, per_operation: 809.78ns, count: 333462 } supplementary_code=467.593487ms total=1.16182349s
2025-11-02T20:39:32.137437Z i ThreadId(53) performance breakdown (smaller is better) allocator_name="talc" allocation={ total: 149.34266ms, per_operation: 891.43ns, count: 167532 } allocation_zeroed={ total: 156.268622ms, per_operation: 935.70ns, count: 167008 } deallocation={ total: 124.774659ms, per_operation: 744.78ns, count: 167532 } reallocation={ total: 311.795652ms, per_operation: 935.87ns, count: 333163 } supplementary_code=440.456507ms total=1.1826381s
2025-11-02T20:39:32.306879Z : ThreadId(58) allocator="wee" iteration=101957 allocation={ size: 907, align: 1, ptr: 0v7717_9D9F_DD60, sum: Some(119266) }
2025-11-02T20:39:32.318175Z : ThreadId(57) allocator="wee" iteration=100048 allocation={ size: 406, align: 64, ptr: 0v7715_6F00_2400, sum: Some(53480) }
2025-11-02T20:39:32.322725Z : ThreadId(56) allocator="wee" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_6ED7_1000, sum: Some(46851) }
2025-11-02T20:39:32.331437Z : ThreadId(59) allocator="wee" iteration=103955 allocation={ size: 517, align: 32, ptr: 0v7715_6E9C_80E0, sum: Some(64941) }
2025-11-02T20:39:32.490567Z : ThreadId(58) allocator="wee" iteration=203246 allocation={ size: 250, align: 4, ptr: 0v7717_9504_BF30, sum: Some(32042) }
2025-11-02T20:39:32.499145Z : ThreadId(56) allocator="wee" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_69C1_BED0, sum: Some(30652) }
2025-11-02T20:39:32.513295Z : ThreadId(59) allocator="wee" iteration=204795 allocation={ size: 994, align: 4, ptr: 0v7717_9C66_0868, sum: Some(129154) }
2025-11-02T20:39:32.521825Z : ThreadId(57) allocator="wee" iteration=201923 allocation={ size: 182, align: 256, ptr: 0v7715_68FE_9200, sum: Some(23391) }
2025-11-02T20:39:32.685808Z : ThreadId(58) allocator="wee" iteration=303696 allocation={ size: 339, align: 1024, ptr: 0v7715_63E8_2C00, sum: Some(43353) }
2025-11-02T20:39:32.693667Z : ThreadId(59) allocator="wee" iteration=305095 allocation={ size: 325, align: 16, ptr: 0v7715_63AB_0E00, sum: Some(39864) }
2025-11-02T20:39:32.698917Z : ThreadId(57) allocator="wee" iteration=303775 allocation={ size: 701, align: 1, ptr: 0v7717_9D86_07A0, sum: Some(85075) }
2025-11-02T20:39:32.711273Z : ThreadId(56) allocator="wee" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_634D_0230, sum: Some(67833) }
2025-11-02T20:39:32.870097Z : ThreadId(59) allocator="wee" iteration=405974 allocation={ size: 900, align: 512, ptr: 0v7715_5E46_2000, sum: Some(116343) }
2025-11-02T20:39:32.870275Z : ThreadId(58) allocator="wee" iteration=403858 allocation={ size: 690, align: 4, ptr: 0v7717_9D90_70C8, sum: Some(88118) }
2025-11-02T20:39:32.902879Z : ThreadId(56) allocator="wee" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_9501_A588, sum: Some(13436) }
2025-11-02T20:39:32.904063Z : ThreadId(57) allocator="wee" iteration=405879 allocation={ size: 572, align: 512, ptr: 0v7715_5D58_9800, sum: Some(75290) }
2025-11-02T20:39:33.055835Z : ThreadId(59) allocator="wee" iteration=508086 allocation={ size: 511, align: 1, ptr: 0v7717_9DC1_17D0, sum: Some(61803) }
2025-11-02T20:39:33.069608Z : ThreadId(58) allocator="wee" iteration=505570 allocation={ size: 152, align: 16, ptr: 0v7715_585E_5FD0, sum: Some(19257) }
2025-11-02T20:39:33.094903Z : ThreadId(56) allocator="wee" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_57B9_55C0, sum: Some(5020) }
2025-11-02T20:39:33.096033Z : ThreadId(57) allocator="wee" iteration=508843 allocation={ size: 936, align: 64, ptr: 0v7715_579A_6200, sum: Some(118699) }
2025-11-02T20:39:33.223626Z : ThreadId(59) allocator="wee" iteration=609707 allocation={ size: 23, align: 128, ptr: 0v7715_53C5_3B80, sum: Some(2729) }
2025-11-02T20:39:33.264174Z : ThreadId(58) allocator="wee" iteration=605687 allocation={ size: 790, align: 32, ptr: 0v7715_52D1_5740, sum: Some(102652) }
2025-11-02T20:39:33.293059Z : ThreadId(57) allocator="wee" iteration=609368 allocation={ size: 707, align: 2, ptr: 0v7717_9C1D_8DF0, sum: Some(89371) }
2025-11-02T20:39:33.303680Z : ThreadId(56) allocator="wee" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_5173_8700, sum: Some(45694) }
2025-11-02T20:39:33.412180Z : ThreadId(59) allocator="wee" iteration=710058 allocation={ size: 698, align: 4, ptr: 0v7717_9D85_F3F0, sum: Some(89146) }
2025-11-02T20:39:33.445728Z : ThreadId(58) allocator="wee" iteration=708985 allocation={ size: 59, align: 256, ptr: 0v7715_4D27_4900, sum: Some(8252) }
2025-11-02T20:39:33.481304Z : ThreadId(57) allocator="wee" iteration=709374 allocation={ size: 182, align: 4, ptr: 0v7717_9D99_62D0, sum: Some(24616) }
2025-11-02T20:39:33.504062Z : ThreadId(56) allocator="wee" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_4B5A_8C00, sum: Some(89169) }
2025-11-02T20:39:33.580769Z : ThreadId(59) allocator="wee" iteration=810547 allocation={ size: 68, align: 32, ptr: 0v7715_4905_C300, sum: Some(9102) }
2025-11-02T20:39:33.647189Z : ThreadId(58) allocator="wee" iteration=810549 allocation={ size: 438, align: 4, ptr: 0v7717_9477_3710, sum: Some(58305) }
2025-11-02T20:39:33.678666Z : ThreadId(57) allocator="wee" iteration=811483 allocation={ size: 951, align: 32, ptr: 0v7715_4673_6D20, sum: Some(118924) }
2025-11-02T20:39:33.690618Z : ThreadId(56) allocator="wee" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_9509_0BC0, sum: Some(102990) }
2025-11-02T20:39:33.744034Z : ThreadId(59) allocator="wee" iteration=914317 allocation={ size: 817, align: 64, ptr: 0v7715_446A_4C00, sum: Some(106866) }
2025-11-02T20:39:33.838669Z : ThreadId(58) allocator="wee" iteration=910631 allocation={ size: 213, align: 512, ptr: 0v7715_41B4_A200, sum: Some(29007) }
2025-11-02T20:39:33.872875Z i ThreadId(59) performance breakdown (smaller is better) allocator_name="wee" allocation={ total: 211.556023ms, per_operation: 1266.74ns, count: 167008 } allocation_zeroed={ total: 218.108367ms, per_operation: 1309.24ns, count: 166592 } deallocation={ total: 162.273911ms, per_operation: 971.65ns, count: 167008 } reallocation={ total: 653.663657ms, per_operation: 1963.33ns, count: 332936 } supplementary_code=489.367658ms total=1.734969616s
2025-11-02T20:39:33.876554Z : ThreadId(56) allocator="wee" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_4079_B880, sum: Some(36528) }
2025-11-02T20:39:33.903833Z : ThreadId(57) allocator="wee" iteration=912712 allocation={ size: 996, align: 16, ptr: 0v7715_3FC2_9300, sum: Some(128153) }
2025-11-02T20:39:33.979466Z i ThreadId(58) performance breakdown (smaller is better) allocator_name="wee" allocation={ total: 232.051175ms, per_operation: 1386.97ns, count: 167308 } allocation_zeroed={ total: 238.787448ms, per_operation: 1431.15ns, count: 166850 } deallocation={ total: 168.681466ms, per_operation: 1008.21ns, count: 167308 } reallocation={ total: 698.430343ms, per_operation: 2092.98ns, count: 333702 } supplementary_code=503.639438ms total=1.84158987s
2025-11-02T20:39:33.989039Z i ThreadId(56) performance breakdown (smaller is better) allocator_name="wee" allocation={ total: 229.667599ms, per_operation: 1374.06ns, count: 167145 } allocation_zeroed={ total: 227.237605ms, per_operation: 1364.78ns, count: 166501 } deallocation={ total: 174.908186ms, per_operation: 1046.45ns, count: 167145 } reallocation={ total: 711.171162ms, per_operation: 2132.69ns, count: 333462 } supplementary_code=508.299969ms total=1.851284521s
2025-11-02T20:39:33.999715Z i ThreadId(57) performance breakdown (smaller is better) allocator_name="wee" allocation={ total: 227.584118ms, per_operation: 1358.45ns, count: 167532 } allocation_zeroed={ total: 238.809957ms, per_operation: 1429.93ns, count: 167008 } deallocation={ total: 186.54895ms, per_operation: 1113.51ns, count: 167532 } reallocation={ total: 704.090335ms, per_operation: 2113.35ns, count: 333163 } supplementary_code=504.859473ms total=1.861892833s
2025-11-02T20:39:33.999801Z i multi_threaded performance (smaller is better) nikka=162.499759ms tcmalloc=204.118069ms jemalloc=221.447576ms mimalloc=189.030118ms system=499.181407ms nikka_div_tcmalloc=0.7961066961288452 nikka_div_jemalloc=0.733806848526001 nikka_div_mimalloc=0.8596501350402832 nikka_div_system=0.325532466173172
2025-11-02T20:39:33.999824Z i multi_threaded performance breakdown of nikka allocator versus tcmalloc (smaller is better) allocation=0.8829745054244995 allocation_zeroed=0.8546364307403564 deallocation=0.9244827032089233 reallocation=0.7431476712226868 pure_operations=0.9008856415748596 mem_operations=0.7753321528434753 total=0.7961066961288452 nikka_pure_operations_ns=45.481164974820366
2025-11-02T20:39:33.999833Z i multi_threaded performance breakdown of nikka allocator versus jemalloc (smaller is better) allocation=0.6887809634208679 allocation_zeroed=0.8523411154747009 deallocation=0.9020161628723145 reallocation=0.6791356801986694 pure_operations=0.7693125009536743 mem_operations=0.7260867953300476 total=0.733806848526001 nikka_pure_operations_ns=45.481164974820366
2025-11-02T20:39:33.999839Z i multi_threaded performance breakdown of nikka allocator versus mimalloc (smaller is better) allocation=0.7355981469154358 allocation_zeroed=1.1020236015319824 deallocation=1.0112401247024536 reallocation=0.7862697839736938 pure_operations=0.8365731239318848 mem_operations=0.8651480674743652 total=0.8596501350402832 nikka_pure_operations_ns=45.481164974820366
2025-11-02T20:39:33.999845Z i multi_threaded performance breakdown of nikka allocator versus system (smaller is better) allocation=0.21477468311786652 allocation_zeroed=0.3511095941066742 deallocation=0.4136602580547333 reallocation=0.3360215425491333 pure_operations=0.2728687822818756 mem_operations=0.3406800329685211 total=0.325532466173172 nikka_pure_operations_ns=45.481164974820366
2025-11-02T20:39:33.999851Z i multi_threaded total_memory=3.535 MiB
ok
test single_threaded ... 2025-11-02T20:39:34.029749Z : single_threaded allocator="nikka allocator" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_3C4F_3400, sum: Some(46851) }
2025-11-02T20:39:34.057786Z : single_threaded allocator="nikka allocator" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_384F_1460, sum: Some(30652) }
2025-11-02T20:39:34.085900Z : single_threaded allocator="nikka allocator" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_3790_0AF0, sum: Some(67833) }
2025-11-02T20:39:34.113946Z : single_threaded allocator="nikka allocator" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_365F_21A0, sum: Some(13436) }
2025-11-02T20:39:34.142626Z : single_threaded allocator="nikka allocator" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_36ED_10C0, sum: Some(5020) }
2025-11-02T20:39:34.171394Z : single_threaded allocator="nikka allocator" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_3C3E_EC00, sum: Some(45694) }
2025-11-02T20:39:34.199998Z : single_threaded allocator="nikka allocator" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_3C50_6C00, sum: Some(89169) }
2025-11-02T20:39:34.227648Z : single_threaded allocator="nikka allocator" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_3A5F_8580, sum: Some(102990) }
2025-11-02T20:39:34.256388Z : single_threaded allocator="nikka allocator" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_3BAC_7A00, sum: Some(36528) }
2025-11-02T20:39:34.280204Z i single_threaded performance breakdown (smaller is better) allocator_name="nikka allocator" allocation={ total: 3.328201ms, per_operation: 19.91ns, count: 167145 } allocation_zeroed={ total: 9.41189ms, per_operation: 56.53ns, count: 166501 } deallocation={ total: 2.624727ms, per_operation: 15.70ns, count: 167145 } reallocation={ total: 19.237361ms, per_operation: 57.69ns, count: 333462 } supplementary_code=245.499755ms total=280.101934ms
2025-11-02T20:39:34.280625Z i single_threaded nikka allocation + deallocation duration = 35.62ns
2025-11-02T20:39:34.309595Z : single_threaded allocator="tcmalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v5317_3FDC_0800, sum: Some(46851) }
2025-11-02T20:39:34.337990Z : single_threaded allocator="tcmalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v5317_3FE6_F400, sum: Some(30652) }
2025-11-02T20:39:34.366728Z : single_threaded allocator="tcmalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v5317_3FC2_2B40, sum: Some(67833) }
2025-11-02T20:39:34.395424Z : single_threaded allocator="tcmalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v5317_3FE0_6F50, sum: Some(13436) }
2025-11-02T20:39:34.424199Z : single_threaded allocator="tcmalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v5317_3FE1_A400, sum: Some(5020) }
2025-11-02T20:39:34.453357Z : single_threaded allocator="tcmalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v5317_3FC1_8000, sum: Some(45694) }
2025-11-02T20:39:34.482702Z : single_threaded allocator="tcmalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v5317_3FA6_FC00, sum: Some(89169) }
2025-11-02T20:39:34.510991Z : single_threaded allocator="tcmalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v5317_3FAD_CA80, sum: Some(102990) }
2025-11-02T20:39:34.539908Z : single_threaded allocator="tcmalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v5317_3FB5_9600, sum: Some(36528) }
2025-11-02T20:39:34.563971Z i single_threaded performance breakdown (smaller is better) allocator_name="tcmalloc" allocation={ total: 4.054115ms, per_operation: 24.26ns, count: 167145 } allocation_zeroed={ total: 9.125118ms, per_operation: 54.81ns, count: 166501 } deallocation={ total: 3.08067ms, per_operation: 18.43ns, count: 167145 } reallocation={ total: 23.059064ms, per_operation: 69.15ns, count: 333462 } supplementary_code=244.017086ms total=283.336053ms
2025-11-02T20:39:34.593635Z : single_threaded allocator="jemalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A5C4_C400, sum: Some(46851) }
2025-11-02T20:39:34.622470Z : single_threaded allocator="jemalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A5C2_5460, sum: Some(30652) }
2025-11-02T20:39:34.651609Z : single_threaded allocator="jemalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A5C5_3C00, sum: Some(67833) }
2025-11-02T20:39:34.680443Z : single_threaded allocator="jemalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A5C1_14D0, sum: Some(13436) }
2025-11-02T20:39:34.709933Z : single_threaded allocator="jemalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A5C4_5100, sum: Some(5020) }
2025-11-02T20:39:34.739604Z : single_threaded allocator="jemalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A5CD_D400, sum: Some(45694) }
2025-11-02T20:39:34.768982Z : single_threaded allocator="jemalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A5C7_6C00, sum: Some(89169) }
2025-11-02T20:39:34.797267Z : single_threaded allocator="jemalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A5CB_8F00, sum: Some(102990) }
2025-11-02T20:39:34.826217Z : single_threaded allocator="jemalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A5C9_D180, sum: Some(36528) }
2025-11-02T20:39:34.850538Z i single_threaded performance breakdown (smaller is better) allocator_name="jemalloc" allocation={ total: 4.502124ms, per_operation: 26.94ns, count: 167145 } allocation_zeroed={ total: 9.041709ms, per_operation: 54.30ns, count: 166501 } deallocation={ total: 2.851985ms, per_operation: 17.06ns, count: 167145 } reallocation={ total: 25.705817ms, per_operation: 77.09ns, count: 333462 } supplementary_code=244.439712ms total=286.541347ms
2025-11-02T20:39:34.879266Z : single_threaded allocator="mimalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v3BC_9814_1400, sum: Some(46851) }
2025-11-02T20:39:34.907225Z : single_threaded allocator="mimalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v3BC_980A_0660, sum: Some(30652) }
2025-11-02T20:39:34.935979Z : single_threaded allocator="mimalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v3BC_9809_4A00, sum: Some(67833) }
2025-11-02T20:39:34.964337Z : single_threaded allocator="mimalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v3BC_9815_05B0, sum: Some(13436) }
2025-11-02T20:39:34.992904Z : single_threaded allocator="mimalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v3BC_9807_0D60, sum: Some(5020) }
2025-11-02T20:39:35.022385Z : single_threaded allocator="mimalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v3BC_9804_8500, sum: Some(45694) }
2025-11-02T20:39:35.050750Z : single_threaded allocator="mimalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v3BC_9809_A400, sum: Some(89169) }
2025-11-02T20:39:35.078941Z : single_threaded allocator="mimalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v3BC_9803_2500, sum: Some(102990) }
2025-11-02T20:39:35.107263Z : single_threaded allocator="mimalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v3BC_9802_6B80, sum: Some(36528) }
2025-11-02T20:39:35.131432Z i single_threaded performance breakdown (smaller is better) allocator_name="mimalloc" allocation={ total: 4.303315ms, per_operation: 25.75ns, count: 167145 } allocation_zeroed={ total: 7.705261ms, per_operation: 46.28ns, count: 166501 } deallocation={ total: 2.736763ms, per_operation: 16.37ns, count: 167145 } reallocation={ total: 23.560554ms, per_operation: 70.65ns, count: 333462 } supplementary_code=242.56637ms total=280.872263ms
2025-11-02T20:39:35.161917Z : single_threaded allocator="ring-alloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A019_A400, sum: Some(46851) }
2025-11-02T20:39:35.190725Z : single_threaded allocator="ring-alloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A042_D8E0, sum: Some(30652) }
2025-11-02T20:39:35.220999Z : single_threaded allocator="ring-alloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A061_3010, sum: Some(67833) }
2025-11-02T20:39:35.249324Z : single_threaded allocator="ring-alloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A065_EF20, sum: Some(13436) }
2025-11-02T20:39:35.280485Z : single_threaded allocator="ring-alloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A0CC_2620, sum: Some(5020) }
2025-11-02T20:39:35.310600Z : single_threaded allocator="ring-alloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A08F_C900, sum: Some(45694) }
2025-11-02T20:39:35.339563Z : single_threaded allocator="ring-alloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A0A4_6000, sum: Some(89169) }
2025-11-02T20:39:35.368193Z : single_threaded allocator="ring-alloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A09F_AEB0, sum: Some(102990) }
2025-11-02T20:39:35.397397Z : single_threaded allocator="ring-alloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A099_EE00, sum: Some(36528) }
2025-11-02T20:39:35.421504Z i single_threaded performance breakdown (smaller is better) allocator_name="ring-alloc" allocation={ total: 3.771934ms, per_operation: 22.57ns, count: 167145 } allocation_zeroed={ total: 10.472525ms, per_operation: 62.90ns, count: 166501 } deallocation={ total: 3.449162ms, per_operation: 20.64ns, count: 167145 } reallocation={ total: 22.50375ms, per_operation: 67.49ns, count: 333462 } supplementary_code=249.848447ms total=290.045818ms
2025-11-02T20:39:35.450895Z : single_threaded allocator="nikka allocator (global cache)" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_334F_3400, sum: Some(46851) }
2025-11-02T20:39:35.478637Z : single_threaded allocator="nikka allocator (global cache)" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_2F4F_1460, sum: Some(30652) }
2025-11-02T20:39:35.507715Z : single_threaded allocator="nikka allocator (global cache)" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_2E90_0AF0, sum: Some(67833) }
2025-11-02T20:39:35.535877Z : single_threaded allocator="nikka allocator (global cache)" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_2D5F_21A0, sum: Some(13436) }
2025-11-02T20:39:35.564575Z : single_threaded allocator="nikka allocator (global cache)" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_2DED_10C0, sum: Some(5020) }
2025-11-02T20:39:35.593571Z : single_threaded allocator="nikka allocator (global cache)" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_333E_EC00, sum: Some(45694) }
2025-11-02T20:39:35.622331Z : single_threaded allocator="nikka allocator (global cache)" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_3350_6C00, sum: Some(89169) }
2025-11-02T20:39:35.650662Z : single_threaded allocator="nikka allocator (global cache)" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_315F_8580, sum: Some(102990) }
2025-11-02T20:39:35.679190Z : single_threaded allocator="nikka allocator (global cache)" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_32AC_7A00, sum: Some(36528) }
2025-11-02T20:39:35.703230Z i single_threaded performance breakdown (smaller is better) allocator_name="nikka allocator (global cache)" allocation={ total: 3.768427ms, per_operation: 22.55ns, count: 167145 } allocation_zeroed={ total: 8.678142ms, per_operation: 52.12ns, count: 166501 } deallocation={ total: 3.648169ms, per_operation: 21.83ns, count: 167145 } reallocation={ total: 19.693354ms, per_operation: 59.06ns, count: 333462 } supplementary_code=245.912692ms total=281.700784ms
2025-11-02T20:39:35.734649Z : single_threaded allocator="nikka allocator (no cache)" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_2A4F_1000, sum: Some(46851) }
2025-11-02T20:39:35.763808Z : single_threaded allocator="nikka allocator (no cache)" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_264F_12A0, sum: Some(30652) }
2025-11-02T20:39:35.793766Z : single_threaded allocator="nikka allocator (no cache)" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_2590_0000, sum: Some(67833) }
2025-11-02T20:39:35.823057Z : single_threaded allocator="nikka allocator (no cache)" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_245F_21A0, sum: Some(13436) }
2025-11-02T20:39:35.853265Z : single_threaded allocator="nikka allocator (no cache)" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_24ED_1480, sum: Some(5020) }
2025-11-02T20:39:35.883574Z : single_threaded allocator="nikka allocator (no cache)" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_2A3F_A200, sum: Some(45694) }
2025-11-02T20:39:35.913129Z : single_threaded allocator="nikka allocator (no cache)" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_2A51_B000, sum: Some(89169) }
2025-11-02T20:39:35.942002Z : single_threaded allocator="nikka allocator (no cache)" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_285F_6C80, sum: Some(102990) }
2025-11-02T20:39:35.971715Z : single_threaded allocator="nikka allocator (no cache)" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_29AC_5900, sum: Some(36528) }
2025-11-02T20:39:35.996161Z i single_threaded performance breakdown (smaller is better) allocator_name="nikka allocator (no cache)" allocation={ total: 4.963966ms, per_operation: 29.70ns, count: 167145 } allocation_zeroed={ total: 9.426322ms, per_operation: 56.61ns, count: 166501 } deallocation={ total: 4.862304ms, per_operation: 29.09ns, count: 167145 } reallocation={ total: 26.207184ms, per_operation: 78.59ns, count: 333462 } supplementary_code=246.695526ms total=292.155302ms
2025-11-02T20:39:36.033570Z : single_threaded allocator="system allocator" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A0CD_A000, sum: Some(46851) }
2025-11-02T20:39:36.069682Z : single_threaded allocator="system allocator" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A0CF_2B10, sum: Some(30652) }
2025-11-02T20:39:36.106385Z : single_threaded allocator="system allocator" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A0D2_2A20, sum: Some(67833) }
2025-11-02T20:39:36.142908Z : single_threaded allocator="system allocator" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A0CD_9B90, sum: Some(13436) }
2025-11-02T20:39:36.179843Z : single_threaded allocator="system allocator" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A0D0_7B80, sum: Some(5020) }
2025-11-02T20:39:36.217011Z : single_threaded allocator="system allocator" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A0D7_5800, sum: Some(45694) }
2025-11-02T20:39:36.253711Z : single_threaded allocator="system allocator" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A0CF_6000, sum: Some(89169) }
2025-11-02T20:39:36.289662Z : single_threaded allocator="system allocator" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A0D2_B290, sum: Some(102990) }
2025-11-02T20:39:36.325581Z : single_threaded allocator="system allocator" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A0D8_0900, sum: Some(36528) }
2025-11-02T20:39:36.356523Z i single_threaded performance breakdown (smaller is better) allocator_name="system allocator" allocation={ total: 18.191323ms, per_operation: 108.84ns, count: 167145 } allocation_zeroed={ total: 25.785276ms, per_operation: 154.87ns, count: 166501 } deallocation={ total: 7.120401ms, per_operation: 42.60ns, count: 167145 } reallocation={ total: 57.304726ms, per_operation: 171.85ns, count: 333462 } supplementary_code=251.680452ms total=360.082178ms
2025-11-02T20:39:36.391884Z : single_threaded allocator="dlmalloc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A623_2000, sum: Some(46851) }
2025-11-02T20:39:36.427013Z : single_threaded allocator="dlmalloc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A625_E790, sum: Some(30652) }
2025-11-02T20:39:36.462660Z : single_threaded allocator="dlmalloc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A625_C800, sum: Some(67833) }
2025-11-02T20:39:36.498429Z : single_threaded allocator="dlmalloc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A624_C110, sum: Some(13436) }
2025-11-02T20:39:36.534836Z : single_threaded allocator="dlmalloc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A624_BAC0, sum: Some(5020) }
2025-11-02T20:39:36.571097Z : single_threaded allocator="dlmalloc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A62C_9000, sum: Some(45694) }
2025-11-02T20:39:36.607373Z : single_threaded allocator="dlmalloc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A62E_BC00, sum: Some(89169) }
2025-11-02T20:39:36.642700Z : single_threaded allocator="dlmalloc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A623_A280, sum: Some(102990) }
2025-11-02T20:39:36.678630Z : single_threaded allocator="dlmalloc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A627_1A80, sum: Some(36528) }
2025-11-02T20:39:36.709324Z i single_threaded performance breakdown (smaller is better) allocator_name="dlmalloc" allocation={ total: 16.238386ms, per_operation: 97.15ns, count: 167145 } allocation_zeroed={ total: 20.933537ms, per_operation: 125.73ns, count: 166501 } deallocation={ total: 11.002368ms, per_operation: 65.83ns, count: 167145 } reallocation={ total: 49.00147ms, per_operation: 146.95ns, count: 333462 } supplementary_code=255.588148ms total=352.763909ms
2025-11-02T20:39:36.746585Z : single_threaded allocator="blink" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A0CD_8000, sum: Some(46851) }
2025-11-02T20:39:36.782385Z : single_threaded allocator="blink" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A0D9_6CF0, sum: Some(30652) }
2025-11-02T20:39:36.819313Z : single_threaded allocator="blink" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A0D1_1000, sum: Some(67833) }
2025-11-02T20:39:36.856066Z : single_threaded allocator="blink" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A0D2_A760, sum: Some(13436) }
2025-11-02T20:39:36.892915Z : single_threaded allocator="blink" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A0D5_4640, sum: Some(5020) }
2025-11-02T20:39:36.930613Z : single_threaded allocator="blink" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A0D6_8A00, sum: Some(45694) }
2025-11-02T20:39:36.968041Z : single_threaded allocator="blink" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A0D4_DE00, sum: Some(89169) }
2025-11-02T20:39:37.004509Z : single_threaded allocator="blink" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A0D1_4080, sum: Some(102990) }
2025-11-02T20:39:37.041343Z : single_threaded allocator="blink" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A0D9_5F80, sum: Some(36528) }
2025-11-02T20:39:37.072626Z i single_threaded performance breakdown (smaller is better) allocator_name="blink" allocation={ total: 17.642448ms, per_operation: 105.55ns, count: 167145 } allocation_zeroed={ total: 25.128018ms, per_operation: 150.92ns, count: 166501 } deallocation={ total: 6.391238ms, per_operation: 38.24ns, count: 167145 } reallocation={ total: 56.643396ms, per_operation: 169.86ns, count: 333462 } supplementary_code=257.470599ms total=363.275699ms
2025-11-02T20:39:37.103465Z : single_threaded allocator="frusa" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_341A_5000, sum: Some(46851) }
2025-11-02T20:39:37.133913Z : single_threaded allocator="frusa" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_3416_9100, sum: Some(30652) }
2025-11-02T20:39:37.165166Z : single_threaded allocator="frusa" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_3415_FC00, sum: Some(67833) }
2025-11-02T20:39:37.196368Z : single_threaded allocator="frusa" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_341B_D880, sum: Some(13436) }
2025-11-02T20:39:37.228314Z : single_threaded allocator="frusa" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_341B_5680, sum: Some(5020) }
2025-11-02T20:39:37.260224Z : single_threaded allocator="frusa" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_3416_1000, sum: Some(45694) }
2025-11-02T20:39:37.292022Z : single_threaded allocator="frusa" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_3416_0400, sum: Some(89169) }
2025-11-02T20:39:37.322566Z : single_threaded allocator="frusa" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_3412_5000, sum: Some(102990) }
2025-11-02T20:39:37.354262Z : single_threaded allocator="frusa" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_3416_7E00, sum: Some(36528) }
2025-11-02T20:39:37.380813Z i single_threaded performance breakdown (smaller is better) allocator_name="frusa" allocation={ total: 6.065451ms, per_operation: 36.29ns, count: 167145 } allocation_zeroed={ total: 11.548384ms, per_operation: 69.36ns, count: 166501 } deallocation={ total: 6.362999ms, per_operation: 38.07ns, count: 167145 } reallocation={ total: 33.39724ms, per_operation: 100.15ns, count: 333462 } supplementary_code=250.771222ms total=308.145296ms
2025-11-02T20:39:37.413172Z : single_threaded allocator="rlsf" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7717_A55E_4400, sum: Some(46851) }
2025-11-02T20:39:37.444579Z : single_threaded allocator="rlsf" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7717_A55B_7230, sum: Some(30652) }
2025-11-02T20:39:37.476642Z : single_threaded allocator="rlsf" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7717_A559_4C30, sum: Some(67833) }
2025-11-02T20:39:37.508264Z : single_threaded allocator="rlsf" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7717_A55A_B990, sum: Some(13436) }
2025-11-02T20:39:37.540463Z : single_threaded allocator="rlsf" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7717_A55B_A4A0, sum: Some(5020) }
2025-11-02T20:39:37.573035Z : single_threaded allocator="rlsf" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7717_A55D_0300, sum: Some(45694) }
2025-11-02T20:39:37.604901Z : single_threaded allocator="rlsf" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7717_A55C_B000, sum: Some(89169) }
2025-11-02T20:39:37.635970Z : single_threaded allocator="rlsf" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7717_A55E_C310, sum: Some(102990) }
2025-11-02T20:39:37.668019Z : single_threaded allocator="rlsf" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7717_A554_6480, sum: Some(36528) }
2025-11-02T20:39:37.694938Z i single_threaded performance breakdown (smaller is better) allocator_name="rlsf" allocation={ total: 7.49762ms, per_operation: 44.86ns, count: 167145 } allocation_zeroed={ total: 12.568077ms, per_operation: 75.48ns, count: 166501 } deallocation={ total: 7.280408ms, per_operation: 43.56ns, count: 167145 } reallocation={ total: 23.496915ms, per_operation: 70.46ns, count: 333462 } supplementary_code=263.158949ms total=314.001969ms
2025-11-02T20:39:37.725876Z : single_threaded allocator="talc" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_01A0_5C00, sum: Some(46851) }
2025-11-02T20:39:37.756359Z : single_threaded allocator="talc" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_01A1_3F40, sum: Some(30652) }
2025-11-02T20:39:37.787573Z : single_threaded allocator="talc" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7715_01A3_4F30, sum: Some(67833) }
2025-11-02T20:39:37.818704Z : single_threaded allocator="talc" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_01A7_2410, sum: Some(13436) }
2025-11-02T20:39:37.850364Z : single_threaded allocator="talc" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7715_01A7_9600, sum: Some(5020) }
2025-11-02T20:39:37.882422Z : single_threaded allocator="talc" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_01A0_E900, sum: Some(45694) }
2025-11-02T20:39:37.913546Z : single_threaded allocator="talc" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7715_01A0_FE00, sum: Some(89169) }
2025-11-02T20:39:37.946256Z : single_threaded allocator="talc" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_01A5_2E08, sum: Some(102990) }
2025-11-02T20:39:37.978307Z : single_threaded allocator="talc" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7715_01A6_5A80, sum: Some(36528) }
2025-11-02T20:39:38.004684Z i single_threaded performance breakdown (smaller is better) allocator_name="talc" allocation={ total: 12.869846ms, per_operation: 77.00ns, count: 167145 } allocation_zeroed={ total: 18.120367ms, per_operation: 108.83ns, count: 166501 } deallocation={ total: 6.699127ms, per_operation: 40.08ns, count: 167145 } reallocation={ total: 25.485678ms, per_operation: 76.43ns, count: 333462 } supplementary_code=246.483017ms total=309.658035ms
2025-11-02T20:39:38.047257Z : single_threaded allocator="wee" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7713_214E_3C00, sum: Some(46851) }
2025-11-02T20:39:38.088982Z : single_threaded allocator="wee" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7713_1FED_5550, sum: Some(30652) }
2025-11-02T20:39:38.131765Z : single_threaded allocator="wee" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7713_1E79_D1C0, sum: Some(67833) }
2025-11-02T20:39:38.173720Z : single_threaded allocator="wee" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7715_38A5_CB30, sum: Some(13436) }
2025-11-02T20:39:38.217468Z : single_threaded allocator="wee" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7713_1B90_CDC0, sum: Some(5020) }
2025-11-02T20:39:38.260453Z : single_threaded allocator="wee" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7713_1A16_9600, sum: Some(45694) }
2025-11-02T20:39:38.303637Z : single_threaded allocator="wee" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7713_189A_1200, sum: Some(89169) }
2025-11-02T20:39:38.345305Z : single_threaded allocator="wee" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7715_37A4_FEE0, sum: Some(102990) }
2025-11-02T20:39:38.387961Z : single_threaded allocator="wee" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7713_15C1_F280, sum: Some(36528) }
2025-11-02T20:39:38.423823Z i single_threaded performance breakdown (smaller is better) allocator_name="wee" allocation={ total: 32.059548ms, per_operation: 191.81ns, count: 167145 } allocation_zeroed={ total: 37.995825ms, per_operation: 228.20ns, count: 166501 } deallocation={ total: 6.736017ms, per_operation: 40.30ns, count: 167145 } reallocation={ total: 84.390761ms, per_operation: 253.07ns, count: 333462 } supplementary_code=257.834086ms total=419.016237ms
2025-11-02T20:39:38.454375Z : single_threaded allocator="good" iteration=103641 allocation={ size: 372, align: 1024, ptr: 0v7715_016F_9C00, sum: Some(46851) }
2025-11-02T20:39:38.487583Z : single_threaded allocator="good" iteration=203658 allocation={ size: 223, align: 16, ptr: 0v7715_00E1_52D0, sum: Some(30652) }
2025-11-02T20:39:38.525977Z : single_threaded allocator="good" iteration=305252 allocation={ size: 556, align: 16, ptr: 0v7714_FFF8_3130, sum: Some(67833) }
2025-11-02T20:39:38.564291Z : single_threaded allocator="good" iteration=405496 allocation={ size: 103, align: 4, ptr: 0v7714_FF07_9D90, sum: Some(13436) }
2025-11-02T20:39:38.603824Z : single_threaded allocator="good" iteration=506630 allocation={ size: 39, align: 32, ptr: 0v7714_FE19_49C0, sum: Some(5020) }
2025-11-02T20:39:38.642932Z : single_threaded allocator="good" iteration=609597 allocation={ size: 358, align: 256, ptr: 0v7715_0198_5600, sum: Some(45694) }
2025-11-02T20:39:38.679910Z : single_threaded allocator="good" iteration=711494 allocation={ size: 701, align: 512, ptr: 0v7714_FC81_A000, sum: Some(89169) }
2025-11-02T20:39:38.718812Z : single_threaded allocator="good" iteration=811996 allocation={ size: 798, align: 1, ptr: 0v7714_E548_EF78, sum: Some(102990) }
2025-11-02T20:39:38.757949Z : single_threaded allocator="good" iteration=914603 allocation={ size: 270, align: 128, ptr: 0v7714_FAAD_DB00, sum: Some(36528) }
2025-11-02T20:39:38.790680Z i single_threaded performance breakdown (smaller is better) allocator_name="good" allocation={ total: 38.210608ms, per_operation: 228.61ns, count: 167145 } allocation_zeroed={ total: 44.544498ms, per_operation: 267.53ns, count: 166501 } deallocation={ total: 5.319628ms, per_operation: 31.83ns, count: 167145 } reallocation={ total: 29.771873ms, per_operation: 89.28ns, count: 333462 } supplementary_code=248.975277ms total=366.821884ms
2025-11-02T20:39:38.790810Z i single_threaded performance (smaller is better) nikka=34.602179ms tcmalloc=39.318967ms jemalloc=42.101635ms mimalloc=38.305893ms system=108.401726ms nikka_div_tcmalloc=0.8800378441810608 nikka_div_jemalloc=0.8218725919723511 nikka_div_mimalloc=0.9033122062683105 nikka_div_system=0.3192032277584076
2025-11-02T20:39:38.790832Z i single_threaded performance breakdown of nikka allocator versus tcmalloc (smaller is better) allocation=0.8209438920021057 allocation_zeroed=1.0314266681671143 deallocation=0.8519987463951111 reallocation=0.8342645764350891 pure_operations=0.8343528509140015 mem_operations=0.8901656270027161 total=0.8800378441810608 nikka_pure_operations_ns=35.615351939932395
2025-11-02T20:39:38.790838Z i single_threaded performance breakdown of nikka allocator versus jemalloc (smaller is better) allocation=0.7392513155937195 allocation_zeroed=1.0409414768218994 deallocation=0.9203158617019653 reallocation=0.7483660578727722 pure_operations=0.8094696402549744 mem_operations=0.8244975805282593 total=0.8218725919723511 nikka_pure_operations_ns=35.615351939932395
2025-11-02T20:39:38.790844Z i single_threaded performance breakdown of nikka allocator versus mimalloc (smaller is better) allocation=0.7734040021896362 allocation_zeroed=1.2214888334274292 deallocation=0.9590625762939453 reallocation=0.81650710105896 pure_operations=0.8455770015716553 mem_operations=0.9163123369216919 total=0.9033122062683105 nikka_pure_operations_ns=35.615351939932395
2025-11-02T20:39:38.790851Z i single_threaded performance breakdown of nikka allocator versus system (smaller is better) allocation=0.18295539915561676 allocation_zeroed=0.36501026153564453 deallocation=0.3686206638813019 reallocation=0.3357028365135193 pure_operations=0.23518460988998413 mem_operations=0.344797819852829 total=0.3192032277584076 nikka_pure_operations_ns=35.615351939932395
2025-11-02T20:39:38.790857Z i single_threaded total_memory=944.000 KiB
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.11s
```

Для сравнения своего аллокатора с тремя baseline--аллокаторами --- `tcmalloc`, `jemalloc` и `mimalloc` ---
можно смотреть только на строки со словом `versus`:
```console
$ (cd ku && ALLOWED_RATIO=1.50 cargo test --profile benchmark --features benchmark --test 3-allocator-5-small -- --test-threads=1) | grep versus
    Finished `benchmark` profile [optimized] target(s) in 0.10s
     Running tests/3-allocator-5-small.rs (/.../nikka/target/benchmark/deps/3_allocator_5_small-c5d5bfe3cca771cd)
2025-11-02T20:44:14.418183Z i multi_threaded performance breakdown of nikka allocator versus tcmalloc (smaller is better) allocation=0.76795494556427 allocation_zeroed=0.8983904719352722 deallocation=0.9945341944694519 reallocation=0.7985100746154785 pure_operations=0.8580058217048645 mem_operations=0.8295041918754578 total=0.834296703338623 nikka_pure_operations_ns=49.48181221627132
2025-11-02T20:44:14.418188Z i multi_threaded performance breakdown of nikka allocator versus jemalloc (smaller is better) allocation=0.7008309364318848 allocation_zeroed=0.9962031245231628 deallocation=0.8042026162147522 reallocation=0.7336801886558533 pure_operations=0.7449429035186768 mem_operations=0.8049730062484741 total=0.7939096093177795 nikka_pure_operations_ns=49.48181221627132
2025-11-02T20:44:14.418192Z i multi_threaded performance breakdown of nikka allocator versus mimalloc (smaller is better) allocation=0.6773990392684937 allocation_zeroed=1.1055552959442139 deallocation=0.9222507476806641 reallocation=0.8222017884254456 pure_operations=0.7717950344085693 mem_operations=0.8996998071670532 total=0.8746340274810791 nikka_pure_operations_ns=49.48181221627132
2025-11-02T20:44:14.418196Z i multi_threaded performance breakdown of nikka allocator versus system (smaller is better) allocation=0.20172207057476044 allocation_zeroed=0.4155324697494507 deallocation=0.4057723581790924 reallocation=0.3465903401374817 pure_operations=0.26254287362098694 mem_operations=0.36705756187438965 total=0.3434164226055145 nikka_pure_operations_ns=49.48181221627132
2025-11-02T20:44:19.245430Z i single_threaded performance breakdown of nikka allocator versus tcmalloc (smaller is better) allocation=0.7659674286842346 allocation_zeroed=1.02800452709198 deallocation=0.8468436598777771 reallocation=0.8244423866271973 pure_operations=0.8009668588638306 mem_operations=0.8822652697563171 total=0.8673936724662781 nikka_pure_operations_ns=35.05160190253971
2025-11-02T20:44:19.245434Z i single_threaded performance breakdown of nikka allocator versus jemalloc (smaller is better) allocation=0.7260000109672546 allocation_zeroed=1.0301110744476318 deallocation=0.9882825613021851 reallocation=0.7403726577758789 pure_operations=0.8263404369354248 mem_operations=0.8163718581199646 total=0.8180387616157532 nikka_pure_operations_ns=35.05160190253971
2025-11-02T20:44:19.245438Z i single_threaded performance breakdown of nikka allocator versus mimalloc (smaller is better) allocation=0.741452157497406 allocation_zeroed=1.1906237602233887 deallocation=0.9546205997467041 reallocation=0.8128373026847839 pure_operations=0.8258262872695923 mem_operations=0.9082180261611938 total=0.8931657671928406 nikka_pure_operations_ns=35.05160190253971
2025-11-02T20:44:19.245442Z i single_threaded performance breakdown of nikka allocator versus system (smaller is better) allocation=0.17658765614032745 allocation_zeroed=0.363079309463501 deallocation=0.36896851658821106 reallocation=0.3328494727611542 pure_operations=0.23191335797309875 mem_operations=0.3422817587852478 total=0.3168136477470398 nikka_pure_operations_ns=35.05160190253971
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/allocator/dispatcher.rs | 88 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------------
ku/src/allocator/fixed_size.rs | 30 ++++++++++++++++++++++++++--
2 files changed, 101 insertions(+), 17 deletions(-)
```
