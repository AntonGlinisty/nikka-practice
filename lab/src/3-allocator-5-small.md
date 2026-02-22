## Аллокатор маленьких блоков памяти

Структура глобального аллокатора будет напоминать описанную Андреем Александреску
иерархическую схему:

- [CppCon 2015: Andrei Alexandrescu "std::allocator..."](https://www.youtube.com/watch?v=LIb3L4vKZ7U)

Аллокатор верхнего уровня называется
[`ku::allocator::dispatcher::Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html).
Сам он память не выделяет, а только диспетчеризует обращение к подходящему аллокатору
нижнего уровня.
Для понимания кто владеет блоком, используем размер этого блока, ---
Rust предоставляет
[`core::alloc::Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html)
с размером
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size)
при освобождении памяти.

В качестве `fallback` мы будем использовать аллокатор постраничных блоков, который реализует типажи
[`core::alloc::Allocator`](https://doc.rust-lang.org/nightly/core/alloc/trait.Allocator.html)
и
[`ku::allocator::big::BigAllocatorGuard`](../../doc/ku/allocator/big/trait.BigAllocatorGuard.html).
В качестве аллокатора нижнего уровня используем
[`ku::allocator::fixed_size::FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
Он выделяет блоки одинакового размера.
Которые хранит в структуре данных
[`ku::allocator::quarry::Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html).
А их занятость отслеживает с помощью битовой карты
[`FixedSizeAllocator::bitmap`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.bitmap).

Сам
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)
не хранит размер блоков, которыми управляет.
Он либо обращается к уже инициализированному
[`FixedSizeAllocator::quarry`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.quarry),
используя метод
[`Quarry::size()`](../../doc/ku/allocator/quarry/struct.Quarry.html#method.size).
Либо этот размер вычисляется на лету диспетчером
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
по индексу в массиве
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size)
и передаётся в метод
[`FixedSizeAllocator::stock_up()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.stock_up)
явно.
В котором выполняется начальная инициализация
[`FixedSizeAllocator::quarry`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.quarry),
если это требуется.

Все размеры кратны константе
[`ku::allocator::fixed_size::MIN_SIZE`](../../doc/ku/allocator/fixed_size/constant.MIN_SIZE.html).
Поэтому при выделении памяти мы будем округлять запрошенный размер
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size)
и запрошенное выравнивание
[`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.align)
до кратного этой константе.
Или же до размера, кратного странице
[`Page::SIZE`](../../doc/ku/memory/frage/struct.Frage.html#associatedconstant.SIZE),
при выделении напрямую из
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).


### Задача 5 --- аллокатор маленьких блоков памяти

#### Доработайте диспетчер аллокаторов [`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)

Диспетчер аллокаторов
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
устанавливается в качестве глобального аллокатора
[`kernel::allocator::GLOBAL_ALLOCATOR`](../../doc/kernel/allocator/static.GLOBAL_ALLOCATOR.html)
и должен реализовывать соответствующий типаж
[`core::alloc::GlobalAlloc`](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html).

Кроме того, он отвечает за статистики выделения памяти
[`ku::allocator::info::Info`](../../doc/ku/allocator/info/struct.Info.html).
А именно, за полную статистику
[`Dispatcher::info`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.info)
и за статистику аллокатора
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback) ---
[`Dispatcher::fallback_info`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback_info).
Это сделано, чтобы не дублировать учёт статистики в каждом из возможных `fallback`.
Также есть статистика у каждого из аллокаторов
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html) ---
[`FixedSizeAllocator::info`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.info).
За неё они отвечают самостоятельно. Кроме того, она имеет тип
[`ku::allocator::info::Info`](../../doc/ku/allocator/info/struct.Info.html),
а не
[`ku::allocator::info::AtomicInfo`](../../doc/ku/allocator/info/struct.AtomicInfo.html).
Так как работу с каждым из
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)
диспетчер выполняет под соответствующей спин-блокировкой `Spinlock<FixedSizeAllocator>`.
Эти статистики должны поддерживать такой инвариант:
полная статистика
[`Dispatcher::info`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.info)
равна сумме
[`Dispatcher::fallback_info`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback_info)
и всех
[`FixedSizeAllocator::info`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.info).
Разбиение полной статистики на статистики разных аллокаторов сделано для удобства отладки.

Доработайте метод
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
в файле
[`ku/src/allocator/dispatcher.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/dispatcher.rs):

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:alloc}}
```

[Описание этого метода](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html#tymethod.alloc),
как и остальных реализуемых нами методов типажа
[`core::alloc::GlobalAlloc`](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html)
есть в
[документации на этот типаж](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html).

Сейчас
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
для любого выделения памяти обращается к аллокатору
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).
Сделайте так, чтобы диспетчер обращался к нему, только когда выделяемый размер:

- Слишком большой и для него нет [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
- Или же кратен странице, --- в этом случае [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html) не выполнял бы никакой полезной работы.

Запрошенный при выделении памяти размер
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size)
при этом может быть и чуть меньше, но при округлении должен давать кратный размеру страницы результат.
Обратите внимание, что в
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size)
может быть запись, которая соответствует такому же размеру.
Но использовать её мы никогда не будем.
Она там есть, только чтобы было удобно пересчитывать индексы в этом массиве в размеры выделяемых блоков памяти.
С другой стороны, в этом массиве нет записи для размера равного нулю,
так как она не сильно упрощает такой пересчёт.

В остальных случаях, выделяйте блок памяти одним из
[`ku::allocator::fixed_size::FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html),
которые собраны в массив
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size).
Каждый из них отвечает за отдельный размер выделяемых блоков.
Обратите внимание, что
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)
выделяет блоки размера и выравнивания `size`.
И подходящий `size` нужно определять с учётом
[`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size),
а не только
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size).

Если в нужном аллокаторе фиксированного размера нет свободных блоков,
то диспетчер должен прежде всего пополнить запас из
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).
с помощью метода
[`FixedSizeAllocator::stock_up()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.stock_up).
Определить, есть ли свободные блоки, поможет метод
[`FixedSizeAllocator::is_empty()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.is_empty).

Удобно вынести часть действий во вспомогательные методы, так как они нам понадобятся неоднократно:

- [`Dispatcher::get_fixed_size_index()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.get_fixed_size_index) возвращает индекс аллокатора фиксированного размера для запрошенного `layout`. Либо `None`, если за `layout` отвечает [`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback). Функция помечена `const`, т.к. функция `Layout::new::<T>` является константной, а значит размерный класс для любых типов может быть посчитан на этапе компиляции.
- [`Dispatcher::get_size()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.get_size) по индексу в массиве [`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size) возвращает размер блоков памяти, которые выдаёт соответствующий [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
- [`Dispatcher::fixed_size_allocate()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.fixed_size_allocate) скрывает логику выделения блока из [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html). А именно, --- проверку его непустоты, наполнение блоками, и собственно выделение блока.

Симметрично
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
доработайте метод
[`Dispatcher::dealloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.dealloc):

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:dealloc}}
```

Главное, что вы должны помнить --- возвращать блок нужно строго в тот аллокатор, из которого он выделялся.

Аналогично
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
доработайте метод
[`Dispatcher::alloc_zeroed()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc_zeroed):

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:alloc_zeroed}}
```

Он должен заполнить нулями выделяемый блок памяти.
Обратите внимание, что из `fallback.allocate_zeroed()`
мы уже получаем заполненный нулями блок, его повторно заполнять не нужно.

Доработайте метод
[`Dispatcher::realloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.realloc),
который изменяет размер ранее выделенной памяти:

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:realloc}}
```

Если и старый и новый размеры относятся к
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback),
то нужная логика уже написана.
Но если хотя бы один из них относится к
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size)
и при этом они относятся к разным аллокаторам, то блок памяти нужно перенести.
Для этого можно воспользоваться готовыми вспомогательными методами, а также методами
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
и
[`Dispatcher::dealloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.dealloc).
С блоком памяти, для которого не меняется выделяемый размер, даже когда меняется запрошенный
[`Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html),
не нужно производить подобных действий.
Но нужно учесть освобождение его старого размера и выделение его нового размера в статистиках.
Для этого вам пригодится метод
[`FixedSizeAllocator::info_mut()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.info_mut).

В типаже
[`core::alloc::GlobalAlloc`](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html)
нет похожего метода `GlobalAlloc::realloc_zeroed()`.
Так что на этом
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
завершён.

Вам могут пригодиться функции

- [`core::ptr::copy_nonoverlapping()`](https://doc.rust-lang.org/nightly/core/ptr/fn.copy_nonoverlapping.html),
- [`core::ptr::write_bytes()`](https://doc.rust-lang.org/nightly/core/ptr/fn.write_bytes.html).


#### Реализуйте выделение и освобождение блоков одинакового размера в [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)

Реализуйте
[метод](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.allocate)

```rust
{{#include ../../ku/src/allocator/fixed_size.rs:allocate}}
```

и обратный к нему
[метод](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.deallocate)

```rust
{{#include ../../ku/src/allocator/fixed_size.rs:deallocate}}
```

в файле
[`ku/src/allocator/fixed_size.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/fixed_size.rs).


### Проверьте себя

Тесты можно запустить командой `cargo test --test 3-allocator-5-small`
в директориях `kernel` и `ku` репозитория.
В проверочной системе запускаются тесты из обеих этих директорий.

#### Базовый тест из [`kernel/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/3-allocator-5-small.rs)

Запустите тест `3-allocator-5-small` из файла
[`kernel/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/3-allocator-5-small.rs):

```console
$ (cd kernel && cargo test --test 3-allocator-5-small)
...
running 4 tests

3_allocator_5_small::alignment------------------------------
17:30:17 0 D align = 1
17:30:17 0 D align = 2
17:30:17 0 D align = 4
17:30:17 0 D align = 8
17:30:17 0 D align = 16
17:30:17 0 D align = 32
17:30:17 0 D align = 64
17:30:17 0 D align = 128
17:30:17 0 D align = 256
17:30:17 0 D align = 512
17:30:17 0 D align = 1024
17:30:17 0 D align = 2048
17:30:17 0 D align = 4096
3_allocator_5_small::alignment--------------------- [passed]

3_allocator_5_small::basic----------------------------------
17:30:17 0 D box_contents = 2
3_allocator_5_small::basic------------------------- [passed]

3_allocator_5_small::grow_and_shrink------------------------
17:30:17 0 D contents_sum = 75491328; push_sum = 75491328
17:30:17 0 D contents_sum = 75491328; pop_sum = 75491328
3_allocator_5_small::grow_and_shrink--------------- [passed]

3_allocator_5_small::stress---------------------------------
17:30:18 0 D contents_sum = 333328333350000; push_sum = 333328333350000
17:30:18 0 D contents_sum = 333328333350000; pop_sum = 333328333350000
17:30:18 0 D values = 100000; pages_for_values = 196
3_allocator_5_small::stress------------------------ [passed]
17:30:18 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


#### Стресс--тест из [`ku/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-5-small.rs)

Также запустите тест `3-allocator-5-small` из файла
[`ku/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-5-small.rs):

```console
$ (cd ku && cargo test --test 3-allocator-5-small -- --nocapture --test-threads=1)
...
running 3 tests
test basic ... 2025-11-02T20:18:46.240560Z i basic allocator metadata size common_metadata=1023.875 KiB per_thread_cache=511.750 KiB
ok
test multi_threaded ... 2025-11-02T20:18:46.393272Z : ThreadId(05) allocator="nikka allocator" iteration=100000 allocation={ size: 1, align: 512, ptr: 0v7724_25DB_4800, sum: Some(153) }
2025-11-02T20:18:46.402460Z : ThreadId(07) allocator="nikka allocator" iteration=100000 allocation={ size: 287, align: 128, ptr: 0v7724_0F2E_0000, sum: Some(37257) }
2025-11-02T20:18:46.417694Z : ThreadId(06) allocator="nikka allocator" iteration=100000 allocation={ size: 268, align: 512, ptr: 0v7724_25DA_DE00, sum: Some(33577) }
2025-11-02T20:18:46.433763Z : ThreadId(04) allocator="nikka allocator" iteration=100000 allocation={ size: 682, align: 256, ptr: 0v7724_25EA_D300, sum: Some(85328) }
2025-11-02T20:18:46.537829Z : ThreadId(05) allocator="nikka allocator" iteration=200000 allocation={ size: 543, align: 16, ptr: 0v7724_0E8B_4760, sum: Some(69864) }
2025-11-02T20:18:46.555512Z : ThreadId(07) allocator="nikka allocator" iteration=200000 allocation={ size: 120, align: 8, ptr: 0v7724_0A53_B1E0, sum: Some(15210) }
2025-11-02T20:18:46.574197Z : ThreadId(06) allocator="nikka allocator" iteration=200000 allocation={ size: 681, align: 256, ptr: 0v7724_25EB_D800, sum: Some(87013) }
2025-11-02T20:18:46.610591Z : ThreadId(04) allocator="nikka allocator" iteration=200000 allocation={ size: 756, align: 32, ptr: 0v7724_25EB_FC00, sum: Some(96185) }
2025-11-02T20:18:46.680406Z : ThreadId(05) allocator="nikka allocator" iteration=300000 allocation={ size: 457, align: 2, ptr: 0v7724_0F8F_8570, sum: Some(57627) }
2025-11-02T20:18:46.713868Z : ThreadId(07) allocator="nikka allocator" iteration=300000 allocation={ size: 253, align: 32, ptr: 0v7724_25AA_0C00, sum: Some(33472) }
2025-11-02T20:18:46.732972Z : ThreadId(06) allocator="nikka allocator" iteration=300000 allocation={ size: 115, align: 512, ptr: 0v7724_25DB_C800, sum: Some(14998) }
2025-11-02T20:18:46.780758Z : ThreadId(04) allocator="nikka allocator" iteration=300000 allocation={ size: 91, align: 256, ptr: 0v7724_25A9_F700, sum: Some(11125) }
2025-11-02T20:18:46.823930Z : ThreadId(05) allocator="nikka allocator" iteration=400000 allocation={ size: 755, align: 128, ptr: 0v7724_25EB_A500, sum: Some(98618) }
2025-11-02T20:18:46.870278Z : ThreadId(07) allocator="nikka allocator" iteration=400000 allocation={ size: 47, align: 64, ptr: 0v7724_2527_A200, sum: Some(6209) }
2025-11-02T20:18:46.890128Z : ThreadId(06) allocator="nikka allocator" iteration=400000 allocation={ size: 482, align: 128, ptr: 0v7724_25DC_2600, sum: Some(60595) }
2025-11-02T20:18:46.945695Z : ThreadId(04) allocator="nikka allocator" iteration=400000 allocation={ size: 392, align: 64, ptr: 0v7724_0F9F_B100, sum: Some(47903) }
2025-11-02T20:18:46.975143Z : ThreadId(05) allocator="nikka allocator" iteration=500000 allocation={ size: 766, align: 256, ptr: 0v7724_25EC_7100, sum: Some(95233) }
2025-11-02T20:18:47.031406Z : ThreadId(07) allocator="nikka allocator" iteration=500000 allocation={ size: 298, align: 2, ptr: 0v7724_2507_25F0, sum: Some(38485) }
2025-11-02T20:18:47.055152Z : ThreadId(06) allocator="nikka allocator" iteration=500000 allocation={ size: 685, align: 1024, ptr: 0v7724_2610_DC00, sum: Some(87886) }
2025-11-02T20:18:47.111440Z : ThreadId(04) allocator="nikka allocator" iteration=500000 allocation={ size: 849, align: 128, ptr: 0v7724_2589_9500, sum: Some(107202) }
2025-11-02T20:18:47.119825Z : ThreadId(05) allocator="nikka allocator" iteration=600000 allocation={ size: 613, align: 128, ptr: 0v7724_2579_4980, sum: Some(79489) }
2025-11-02T20:18:47.192766Z : ThreadId(07) allocator="nikka allocator" iteration=600000 allocation={ size: 602, align: 256, ptr: 0v7724_25EB_4500, sum: Some(74872) }
2025-11-02T20:18:47.211434Z : ThreadId(06) allocator="nikka allocator" iteration=600000 allocation={ size: 921, align: 2, ptr: 0v7724_24D6_CAE0, sum: Some(115163) }
2025-11-02T20:18:47.262628Z : ThreadId(05) allocator="nikka allocator" iteration=700000 allocation={ size: 853, align: 16, ptr: 0v7724_2599_86C0, sum: Some(107058) }
2025-11-02T20:18:47.271181Z : ThreadId(04) allocator="nikka allocator" iteration=600000 allocation={ size: 719, align: 16, ptr: 0v7724_0D95_A5A0, sum: Some(91977) }
2025-11-02T20:18:47.351569Z : ThreadId(07) allocator="nikka allocator" iteration=700000 allocation={ size: 874, align: 4, ptr: 0v7724_0B16_5260, sum: Some(108529) }
2025-11-02T20:18:47.368030Z : ThreadId(06) allocator="nikka allocator" iteration=700000 allocation={ size: 682, align: 64, ptr: 0v7724_2517_7080, sum: Some(87875) }
2025-11-02T20:18:47.409943Z : ThreadId(05) allocator="nikka allocator" iteration=800000 allocation={ size: 9, align: 2, ptr: 0v7724_0B36_9060, sum: Some(1172) }
2025-11-02T20:18:47.454336Z : ThreadId(04) allocator="nikka allocator" iteration=700000 allocation={ size: 914, align: 4, ptr: 0v7724_2442_A000, sum: Some(113657) }
2025-11-02T20:18:47.516208Z : ThreadId(07) allocator="nikka allocator" iteration=800000 allocation={ size: 439, align: 256, ptr: 0v7724_25DC_3000, sum: Some(57391) }
2025-11-02T20:18:47.518749Z : ThreadId(06) allocator="nikka allocator" iteration=800000 allocation={ size: 922, align: 2, ptr: 0v7724_24D6_F2C0, sum: Some(117048) }
2025-11-02T20:18:47.554071Z : ThreadId(05) allocator="nikka allocator" iteration=900000 allocation={ size: 375, align: 8, ptr: 0v7724_0BCA_BD38, sum: Some(47356) }
2025-11-02T20:18:47.621710Z : ThreadId(04) allocator="nikka allocator" iteration=800000 allocation={ size: 288, align: 512, ptr: 0v7724_25DC_F600, sum: Some(35522) }
2025-11-02T20:18:47.668618Z : ThreadId(06) allocator="nikka allocator" iteration=900000 allocation={ size: 673, align: 1, ptr: 0v7724_2422_57D8, sum: Some(87033) }
2025-11-02T20:18:47.679645Z : ThreadId(07) allocator="nikka allocator" iteration=900000 allocation={ size: 464, align: 4, ptr: 0v7724_0F8F_A610, sum: Some(59472) }
2025-11-02T20:18:47.697686Z : ThreadId(05) allocator="nikka allocator" iteration=1000000 allocation={ size: 257, align: 4, ptr: 0v7724_0FF0_0000, sum: Some(32038) }
2025-11-02T20:18:47.780005Z : ThreadId(04) allocator="nikka allocator" iteration=900000 allocation={ size: 326, align: 1, ptr: 0v7724_24F7_23D8, sum: Some(42724) }
2025-11-02T20:18:47.819685Z : ThreadId(06) allocator="nikka allocator" iteration=1000000 allocation={ size: 182, align: 1024, ptr: 0v7724_2615_F800, sum: Some(22186) }
2025-11-02T20:18:47.825849Z : ThreadId(07) allocator="nikka allocator" iteration=1000000 allocation={ size: 660, align: 32, ptr: 0v7724_0F3E_A4C0, sum: Some(85472) }
2025-11-02T20:18:47.918375Z : ThreadId(04) allocator="nikka allocator" iteration=1000000 allocation={ size: 93, align: 64, ptr: 0v7724_2463_2600, sum: Some(12539) }
2025-11-02T20:18:47.920717Z i multi_threaded nikka allocation + deallocation duration = 178.84ns
ok
test single_threaded ... 2025-11-02T20:18:48.066298Z : single_threaded allocator="nikka allocator" iteration=100000 allocation={ size: 682, align: 256, ptr: 0v7724_0981_B300, sum: Some(85328) }
2025-11-02T20:18:48.204795Z : single_threaded allocator="nikka allocator" iteration=200000 allocation={ size: 756, align: 32, ptr: 0v7724_0981_B000, sum: Some(96185) }
2025-11-02T20:18:48.342212Z : single_threaded allocator="nikka allocator" iteration=300000 allocation={ size: 91, align: 256, ptr: 0v7724_0788_9500, sum: Some(11125) }
2025-11-02T20:18:48.480746Z : single_threaded allocator="nikka allocator" iteration=400000 allocation={ size: 392, align: 64, ptr: 0v7724_05AD_1C00, sum: Some(47903) }
2025-11-02T20:18:48.617843Z : single_threaded allocator="nikka allocator" iteration=500000 allocation={ size: 849, align: 128, ptr: 0v7724_0723_7380, sum: Some(107202) }
2025-11-02T20:18:48.756112Z : single_threaded allocator="nikka allocator" iteration=600000 allocation={ size: 719, align: 16, ptr: 0v7724_0798_C000, sum: Some(91977) }
2025-11-02T20:18:48.896489Z : single_threaded allocator="nikka allocator" iteration=700000 allocation={ size: 914, align: 4, ptr: 0v7724_0352_9000, sum: Some(113657) }
2025-11-02T20:18:49.037506Z : single_threaded allocator="nikka allocator" iteration=800000 allocation={ size: 288, align: 512, ptr: 0v7724_086D_3400, sum: Some(35522) }
2025-11-02T20:18:49.176602Z : single_threaded allocator="nikka allocator" iteration=900000 allocation={ size: 326, align: 1, ptr: 0v7724_0961_1A40, sum: Some(42724) }
2025-11-02T20:18:49.315931Z : single_threaded allocator="nikka allocator" iteration=1000000 allocation={ size: 93, align: 64, ptr: 0v7724_092F_0580, sum: Some(12539) }
2025-11-02T20:18:49.317827Z i single_threaded nikka allocation + deallocation duration = 121.37ns
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.08s
```


### Ориентировочный объём работ этой части лабораторки

```console
src/allocator/dispatcher.rs | 129 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-------
src/allocator/fixed_size.rs |  92 +++++++++++++++++++++++++++++++++++++++++++++++++++++------
2 files changed, 203 insertions(+), 18 deletions(-)
```
