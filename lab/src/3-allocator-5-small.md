## Аллокатор маленьких блоков памяти

Эта задача не обязательная, без её выполнения дальнейший код должен работать.
Её цель --- научиться тратить меньше памяти при выделениях памяти маленького размера.

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

В качестве `fallback` мы будем использовать аллокатор постраничных блоков, который реализует типаж
[`core::alloc::Allocator`](https://doc.rust-lang.org/nightly/core/alloc/trait.Allocator.html).

В качестве аллокатора нижнего уровня используем
[`ku::allocator::fixed_size::FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
Он выделяет блоки одинакового размера.
Которые хранит в структуре данных, реализующей интерфейс
[Last In, First Out](https://en.wikipedia.org/wiki/Stack_(abstract_data_type))
на основе односвязного списка записей
[`ku::allocator::fixed_size::Lifo`](../../doc/ku/allocator/fixed_size/struct.Lifo.html).
Сам
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)
не хранит размер блоков, которыми управляет.
Этот размер вычисляется на лету диспетчером
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
по индексу в массиве
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size)
и передаётся в методы
[`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)
явно.
Все размеры кратны размеру и выравниванию структуры
[`ku::allocator::fixed_size::Lifo`](../../doc/ku/allocator/fixed_size/struct.Lifo.html) ---
константе
[`ku::allocator::fixed_size::MIN_SIZE`](../../doc/ku/allocator/fixed_size/constant.MIN_SIZE.html).
Поэтому при выделении памяти мы будем округлять запрошенный размер
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size)
и запрошенное выравнивание
[`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.align)
до кратного этой константе.
Или же до размера страницы
[`Page::SIZE`](../../doc/ku/memory/frage/struct.Frage.html#associatedconstant.SIZE),
при выделении напрямую из
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).
Выравнивание большего чем страница размера поддерживать не будем.


### Задача 2 --- аллокатор маленьких блоков памяти

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
unsafe fn Dispatcher::alloc(
    &self,
    layout: Layout,
) -> *mut u8
```

Сейчас он для любого выделения памяти обращается к аллокатору
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
А значит, если
[`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size)
больше
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size),
то ориентироваться нужно на него.
По той же причине `size` должен удовлетворять
[требованиям на `align`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.from_size_align).

Если в нужном аллокаторе фиксированного размера нет свободных блоков,
то диспетчер должен прежде всего выделить блок страниц из
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).
И отдать их в
[`FixedSizeAllocator::stock_up()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.stock_up)
для нарезки на небольшие блоки.
Определить, есть ли свободные блоки, поможет метод
[`FixedSizeAllocator::is_empty()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.is_empty).
А вычислить подходящий размер большого блока страниц --- метод
[`FixedSizeAllocator::get_pack_layout()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.get_pack_layout).

Удобно вынести часть действий во вспомогательные методы, так как они нам понадобятся неоднократно:

- [`Dispatcher::get_fixed_size_index()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.get_fixed_size_index) возвращает индекс аллокатора фиксированного размера для запрошенного `layout`. Либо `None`, если за `layout` отвечает [`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback).
- [`Dispatcher::get_size()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.get_size) по индексу в массиве [`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size) возвращает размер блоков памяти, которые выдаёт соответствующий [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html).
- [`Dispatcher::fixed_size_allocate()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.fixed_size_allocate) скрывает логику выделения блока из [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html). А именно, --- проверку его непустоты, наполнение блоками, и собственно выделение блока.

Симметрично
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
доработайте метод
[`Dispatcher::dealloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.dealloc):

```rust
unsafe fn Dispatcher::dealloc(
    &self,
    ptr: *mut u8,
    layout: Layout,
)
```

Главное, что вы должны помнить --- возвращать блок нужно строго в тот аллокатор, из которого он выделялся.

Аналогично
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
доработайте метод
[`Dispatcher::alloc_zeroed()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc):

```rust
unsafe fn Dispatcher::alloc_zeroed(
    &self,
    layout: Layout,
) -> *mut u8
```

Он должен заполнить нулями выделяемый блок памяти.
Обратите внимание, что из `fallback.allocate_zeroed()`
мы уже получаем заполненный нулями блок, его повторно заполнять не нужно.

Доработайте метод
[`Dispatcher::realloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.realloc),
который изменяет размер ранее выделенной памяти:

```rust
unsafe fn Dispatcher::realloc(
    &self,
    ptr: *mut u8,
    layout: Layout,
    new_size: usize,
) -> *mut u8
```

Если и старый размер и новый относятся к
[`Dispatcher::fallback`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fallback),
то нужная логика уже написана.
Но если хотя бы один из них относится к
[`Dispatcher::fixed_size`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#structfield.fixed_size)
и при этом они относятся к разным аллокаторам, то блок памяти нужно перенести.
Что-либо делать с блоком памяти для которого не меняется выделяемый размер,
даже когда меняется запрошенный
[`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size),
не нужно.
При этом можно воспользоваться готовыми вспомогательными методами, а также методами
[`Dispatcher::alloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.alloc)
и
[`Dispatcher::dealloc()`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.dealloc).

В типаже
[`core::alloc::GlobalAlloc`](https://doc.rust-lang.org/nightly/core/alloc/trait.GlobalAlloc.html)
нет похожего метода `GlobalAlloc::realloc_zeroed()`.
Так что на этом
[`Dispatcher`](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html)
завершён.

Вам могут пригодиться функции

- [`core::ptr::copy_nonoverlapping()`](https://doc.rust-lang.org/nightly/core/ptr/fn.copy_nonoverlapping.html),
- [`core::ptr::write_bytes()`](https://doc.rust-lang.org/nightly/core/ptr/fn.write_bytes.html).


#### Реализуйте аллокатор блоков одинакового размера [`FixedSizeAllocator`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html)

Реализуйте методы
[`Lifo::pop()`](../../doc/ku/allocator/fixed_size/struct.Lifo.html#method.pop)
и
[`Lifo::push()`](../../doc/ku/allocator/fixed_size/struct.Lifo.html#method.push)
в файле
[`ku/src/allocator/fixed_size.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/fixed_size.rs):

```rust
fn Lifo::pop(
    &mut self,
) -> Virt

fn Lifo::push(
    &mut self,
    virt: Virt,
)
```

Они предназначены для работы со списком свободных блоков памяти
[`ku::allocator::fixed_size::Lifo`](../../doc/ku/allocator/fixed_size/struct.Lifo.html).
В случае отсутствия свободного блока, метод
[`Lifo::pop()`](../../doc/ku/allocator/fixed_size/struct.Lifo.html#method.pop)
возвращает нулевой адрес
[`Virt::zero()`](../../doc/ku/memory/addr/type.Virt.html#method.zero).

Реализуйте метод
[`FixedSizeAllocator::get_pack_layout()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.get_pack_layout)
в файле
[`ku/src/allocator/fixed_size.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/fixed_size.rs):

```rust
fn FixedSizeAllocator::get_pack_layout(
    size: usize,
) -> Layout
```

Он возвращает
[`Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html)
блока памяти, который

- Состоит из целого количества страниц [`Page`](../../doc/ku/memory/frage/type.Page.html).
- Подходит для нарезки на блоки размером `size`.

В методе
[`FixedSizeAllocator::stock_up()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.stock_up)
блок с этим
[`Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html)
будет нарезаться на подблоки размера `size`, которые выделяет этот аллокатор.

Реализуйте метод
[`FixedSizeAllocator::stock_up()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.stock_up):

```rust
fn FixedSizeAllocator::stock_up(
    &mut self,
    size: usize,
    pack: Block<Virt>,
)
```

Он режет блок `pack` на подблоки размера `size`, которые сохраняет себе для последующих выделений памяти.

Реализуйте метод
[`FixedSizeAllocator::allocate()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.allocate):

```rust
fn FixedSizeAllocator::allocate(
    &mut self,
    layout: Layout,
    size: usize,
) -> *mut u8
```

Он выделяет память из своего списка свободных блоков ---
[`FixedSizeAllocator::free`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.free).
А запрошенный при выделении памяти
[`Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html)
и реально выделяемый размер `size` понадобятся для учёта этой операции в
[`FixedSizeAllocator::info`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#structfield.info).

Реализуйте парный ему метод
[`FixedSizeAllocator::deallocate()`](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.deallocate):

```rust
fn FixedSizeAllocator::deallocate(
    &mut self,
    ptr: *mut u8,
    layout: Layout,
    size: usize,
)
```


### Проверьте себя

Запустите тест `3-allocator-5-small` из файла
[`kernel/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/3-allocator-5-small.rs):

```console
$ (cd kernel; cargo test --test 3-allocator-5-small)
...
3_allocator_5_small::basic-----------------------
17:27:20 0 D start_info = { allocations: 0 - 0 = 0, requested: 0 B - 0 B = 0 B, allocated: 0 B - 0 B = 0 B, pages: 0 - 0 = 0, loss: 0 B = 0.000% }
17:27:20 0 D box_contents = 2
17:27:20 0 D allocator_info = { total: { allocations: 1 - 0 = 1, requested: 4 B - 0 B = 4 B, allocated: 8 B - 0 B = 8 B, pages: 1 - 0 = 1, loss: 3.996 KiB = 99.902% }, fallback: { allocations: 0 - 0 = 0, requested: 0 B - 0 B = 0 B, allocated: 0 B - 0 B = 0 B, pages: 0 - 0 = 0, loss: 0 B = 0.000% }, size_8: { allocations: 1 - 0 = 1, requested: 4 B - 0 B = 4 B, allocated: 8 B - 0 B = 8 B, pages: 1 - 0 = 1, loss: 3.996 KiB = 99.902% } }
17:27:20 0 D info = { allocations: 1 - 0 = 1, requested: 4 B - 0 B = 4 B, allocated: 8 B - 0 B = 8 B, pages: 1 - 0 = 1, loss: 3.996 KiB = 99.902% }
17:27:20 0 D info_diff = { allocations: 1 - 0 = 1, requested: 4 B - 0 B = 4 B, allocated: 8 B - 0 B = 8 B, pages: 1 - 0 = 1, loss: 3.996 KiB = 99.902% }
17:27:20 0 D allocator_info = { total: { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, fallback: { allocations: 0 - 0 = 0, requested: 0 B - 0 B = 0 B, allocated: 0 B - 0 B = 0 B, pages: 0 - 0 = 0, loss: 0 B = 0.000% }, size_8: { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% } }
17:27:20 0 D end_info = { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }
17:27:20 0 D end_info_diff = { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }
3_allocator_5_small::basic-------------- [passed]

3_allocator_5_small::grow_and_shrink-------------
17:27:20 0 D start_info = { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }
17:27:20 0 D contents_sum = 75491328; push_sum = 75491328
17:27:20 0 D allocator_info = { total: { allocations: 14 - 13 = 1, requested: 255.973 KiB - 127.973 KiB = 128.000 KiB, allocated: 255.977 KiB - 127.977 KiB = 128.000 KiB, pages: 72 - 31 = 41, loss: 36.000 KiB = 21.951% }, fallback: { allocations: 6 - 5 = 1, requested: 252.000 KiB - 124.000 KiB = 128.000 KiB, allocated: 252.000 KiB - 124.000 KiB = 128.000 KiB, pages: 63 - 31 = 32, loss: 0 B = 0.000% }, size_8: { allocations: 1 - 1 = 0, requested: 4 B - 4 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_32: { allocations: 1 - 1 = 0, requested: 32 B - 32 B = 0 B, allocated: 32 B - 32 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_64: { allocations: 1 - 1 = 0, requested: 64 B - 64 B = 0 B, allocated: 64 B - 64 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_128: { allocations: 1 - 1 = 0, requested: 128 B - 128 B = 0 B, allocated: 128 B - 128 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_256: { allocations: 1 - 1 = 0, requested: 256 B - 256 B = 0 B, allocated: 256 B - 256 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_512: { allocations: 1 - 1 = 0, requested: 512 B - 512 B = 0 B, allocated: 512 B - 512 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_1024: { allocations: 1 - 1 = 0, requested: 1.000 KiB - 1.000 KiB = 0 B, allocated: 1.000 KiB - 1.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_2048: { allocations: 1 - 1 = 0, requested: 2.000 KiB - 2.000 KiB = 0 B, allocated: 2.000 KiB - 2.000 KiB = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% } }
17:27:20 0 D info = { allocations: 14 - 13 = 1, requested: 255.973 KiB - 127.973 KiB = 128.000 KiB, allocated: 255.977 KiB - 127.977 KiB = 128.000 KiB, pages: 72 - 31 = 41, loss: 36.000 KiB = 21.951% }
17:27:20 0 D info_diff = { allocations: 13 - 12 = 1, requested: 255.969 KiB - 127.969 KiB = 128.000 KiB, allocated: 255.969 KiB - 127.969 KiB = 128.000 KiB, pages: 71 - 31 = 40, loss: 32.000 KiB = 20.000% }
17:27:22 0 D contents_sum = 75491328; pop_sum = 75491328
17:27:22 0 D allocator_info = { total: { allocations: 28 - 28 = 0, requested: 383.965 KiB - 383.965 KiB = 0 B, allocated: 383.969 KiB - 383.969 KiB = 0 B, pages: 104 - 94 = 10, loss: 40.000 KiB = 100.000% }, fallback: { allocations: 11 - 11 = 0, requested: 376.000 KiB - 376.000 KiB = 0 B, allocated: 376.000 KiB - 376.000 KiB = 0 B, pages: 94 - 94 = 0, loss: 0 B = 0.000% }, size_8: { allocations: 2 - 2 = 0, requested: 12 B - 12 B = 0 B, allocated: 16 B - 16 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_16: { allocations: 1 - 1 = 0, requested: 16 B - 16 B = 0 B, allocated: 16 B - 16 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_32: { allocations: 2 - 2 = 0, requested: 64 B - 64 B = 0 B, allocated: 64 B - 64 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_64: { allocations: 2 - 2 = 0, requested: 128 B - 128 B = 0 B, allocated: 128 B - 128 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_128: { allocations: 2 - 2 = 0, requested: 256 B - 256 B = 0 B, allocated: 256 B - 256 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_256: { allocations: 2 - 2 = 0, requested: 512 B - 512 B = 0 B, allocated: 512 B - 512 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_512: { allocations: 2 - 2 = 0, requested: 1.000 KiB - 1.000 KiB = 0 B, allocated: 1.000 KiB - 1.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_1024: { allocations: 2 - 2 = 0, requested: 2.000 KiB - 2.000 KiB = 0 B, allocated: 2.000 KiB - 2.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_2048: { allocations: 2 - 2 = 0, requested: 4.000 KiB - 4.000 KiB = 0 B, allocated: 4.000 KiB - 4.000 KiB = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% } }
17:27:22 0 D end_info = { allocations: 28 - 28 = 0, requested: 383.965 KiB - 383.965 KiB = 0 B, allocated: 383.969 KiB - 383.969 KiB = 0 B, pages: 104 - 94 = 10, loss: 40.000 KiB = 100.000% }
3_allocator_5_small::grow_and_shrink---- [passed]

3_allocator_5_small::stress----------------------
17:27:22 0 D start_info = { allocations: 28 - 28 = 0, requested: 383.965 KiB - 383.965 KiB = 0 B, allocated: 383.969 KiB - 383.969 KiB = 0 B, pages: 104 - 94 = 10, loss: 40.000 KiB = 100.000% }
17:27:22 0 D vector_length = 0
17:27:22 0 D vector_length = 5000
17:27:23.147 0 D vector_length = 10000
17:27:23.587 0 D vector_length = 15000
17:27:24.043 0 D vector_length = 20000
17:27:24.483 0 D vector_length = 25000
17:27:24.921 0 D vector_length = 30000
17:27:25.395 0 D vector_length = 35000
17:27:25.833 0 D vector_length = 40000
17:27:26.267 0 D vector_length = 45000
17:27:26.729 0 D vector_length = 50000
17:27:27.187 0 D vector_length = 55000
17:27:27.647 0 D vector_length = 60000
17:27:28.103 0 D vector_length = 65000
17:27:28.641 0 D vector_length = 70000
17:27:29.099 0 D vector_length = 75000
17:27:29.561 0 D vector_length = 80000
17:27:30.021 0 D vector_length = 85000
17:27:30.483 0 D vector_length = 90000
17:27:30.943 0 D vector_length = 95000
17:27:31.419 0 D contents_sum = 333328333350000; push_sum = 333328333350000
17:27:31.421 0 D allocator_info = { total: { allocations: 100077 - 43 = 100034, requested: 3.144 MiB - 1.375 MiB = 1.769 MiB, allocated: 3.144 MiB - 1.375 MiB = 1.769 MiB, pages: 813 - 349 = 464, loss: 44.094 KiB = 2.376% }, fallback: { allocations: 20 - 19 = 1, requested: 2.363 MiB - 1.363 MiB = 1.000 MiB, allocated: 2.363 MiB - 1.363 MiB = 1.000 MiB, pages: 605 - 349 = 256, loss: 0 B = 0.000% }, size_8: { allocations: 100002 - 2 = 100000, requested: 781.262 KiB - 12 B = 781.250 KiB, allocated: 781.266 KiB - 16 B = 781.250 KiB, pages: 196 - 0 = 196, loss: 2.750 KiB = 0.351% }, size_16: { allocations: 1 - 1 = 0, requested: 16 B - 16 B = 0 B, allocated: 16 B - 16 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_32: { allocations: 3 - 3 = 0, requested: 96 B - 96 B = 0 B, allocated: 96 B - 96 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_64: { allocations: 3 - 3 = 0, requested: 192 B - 192 B = 0 B, allocated: 192 B - 192 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_128: { allocations: 3 - 3 = 0, requested: 384 B - 384 B = 0 B, allocated: 384 B - 384 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_192: { allocations: 28 - 0 = 28, requested: 5.250 KiB - 0 B = 5.250 KiB, allocated: 5.250 KiB - 0 B = 5.250 KiB, pages: 2 - 0 = 2, loss: 2.750 KiB = 34.375% }, size_256: { allocations: 3 - 3 = 0, requested: 768 B - 768 B = 0 B, allocated: 768 B - 768 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_288: { allocations: 5 - 0 = 5, requested: 1.406 KiB - 0 B = 1.406 KiB, allocated: 1.406 KiB - 0 B = 1.406 KiB, pages: 1 - 0 = 1, loss: 2.594 KiB = 64.844% }, size_512: { allocations: 3 - 3 = 0, requested: 1.500 KiB - 1.500 KiB = 0 B, allocated: 1.500 KiB - 1.500 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_1024: { allocations: 3 - 3 = 0, requested: 3.000 KiB - 3.000 KiB = 0 B, allocated: 3.000 KiB - 3.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_2048: { allocations: 3 - 3 = 0, requested: 6.000 KiB - 6.000 KiB = 0 B, allocated: 6.000 KiB - 6.000 KiB = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% } }
17:27:31.811 0 D info = { allocations: 100077 - 43 = 100034, requested: 3.144 MiB - 1.375 MiB = 1.769 MiB, allocated: 3.144 MiB - 1.375 MiB = 1.769 MiB, pages: 813 - 349 = 464, loss: 44.094 KiB = 2.376% }
17:27:31.821 0 D info_diff = { allocations: 100049 - 15 = 100034, requested: 2.769 MiB - 1023.969 KiB = 1.769 MiB, allocated: 2.769 MiB - 1023.969 KiB = 1.769 MiB, pages: 709 - 255 = 454, loss: 4.094 KiB = 0.225% }
17:27:32.397 0 D vector_length = 95000
17:27:32.963 0 D vector_length = 90000
17:27:33.527 0 D vector_length = 85000
17:27:34.093 0 D vector_length = 80000
17:27:34.661 0 D vector_length = 75000
17:27:35.225 0 D vector_length = 70000
17:27:35.811 0 D vector_length = 65000
17:27:36.373 0 D vector_length = 60000
17:27:36.937 0 D vector_length = 55000
17:27:37.501 0 D vector_length = 50000
17:27:38.063 0 D vector_length = 45000
17:27:38.629 0 D vector_length = 40000
17:27:39.193 0 D vector_length = 35000
17:27:39.767 0 D vector_length = 30000
17:27:40.331 0 D vector_length = 25000
17:27:40.893 0 D vector_length = 20000
17:27:41.465 0 D vector_length = 15000
17:27:42.033 0 D vector_length = 10000
17:27:42.601 0 D vector_length = 5000
17:27:43.167 0 D vector_length = 0
17:27:43.171 0 D contents_sum = 333328333350000; pop_sum = 333328333350000
17:27:43.183 0 D allocator_info = { total: { allocations: 100094 - 100094 = 0, requested: 4.144 MiB - 4.144 MiB = 0 B, allocated: 4.144 MiB - 4.144 MiB = 0 B, pages: 1068 - 860 = 208, loss: 832.000 KiB = 100.000% }, fallback: { allocations: 28 - 28 = 0, requested: 3.359 MiB - 3.359 MiB = 0 B, allocated: 3.359 MiB - 3.359 MiB = 0 B, pages: 860 - 860 = 0, loss: 0 B = 0.000% }, size_8: { allocations: 100003 - 100003 = 0, requested: 781.270 KiB - 781.270 KiB = 0 B, allocated: 781.273 KiB - 781.273 KiB = 0 B, pages: 196 - 0 = 196, loss: 784.000 KiB = 100.000% }, size_16: { allocations: 2 - 2 = 0, requested: 32 B - 32 B = 0 B, allocated: 32 B - 32 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_32: { allocations: 4 - 4 = 0, requested: 128 B - 128 B = 0 B, allocated: 128 B - 128 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_64: { allocations: 4 - 4 = 0, requested: 256 B - 256 B = 0 B, allocated: 256 B - 256 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_128: { allocations: 4 - 4 = 0, requested: 512 B - 512 B = 0 B, allocated: 512 B - 512 B = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_192: { allocations: 28 - 28 = 0, requested: 5.250 KiB - 5.250 KiB = 0 B, allocated: 5.250 KiB - 5.250 KiB = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% }, size_256: { allocations: 4 - 4 = 0, requested: 1.000 KiB - 1.000 KiB = 0 B, allocated: 1.000 KiB - 1.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_288: { allocations: 5 - 5 = 0, requested: 1.406 KiB - 1.406 KiB = 0 B, allocated: 1.406 KiB - 1.406 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_512: { allocations: 4 - 4 = 0, requested: 2.000 KiB - 2.000 KiB = 0 B, allocated: 2.000 KiB - 2.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_1024: { allocations: 4 - 4 = 0, requested: 4.000 KiB - 4.000 KiB = 0 B, allocated: 4.000 KiB - 4.000 KiB = 0 B, pages: 1 - 0 = 1, loss: 4.000 KiB = 100.000% }, size_2048: { allocations: 4 - 4 = 0, requested: 8.000 KiB - 8.000 KiB = 0 B, allocated: 8.000 KiB - 8.000 KiB = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% } }
17:27:43.571 0 D end_info = { allocations: 100094 - 100094 = 0, requested: 4.144 MiB - 4.144 MiB = 0 B, allocated: 4.144 MiB - 4.144 MiB = 0 B, pages: 1068 - 860 = 208, loss: 832.000 KiB = 100.000% }
17:27:43.581 0 D values = 100000; pages_for_values = 196
3_allocator_5_small::stress------------- [passed]
17:27:43.587 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/allocator/dispatcher.rs | 80 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 ku/src/allocator/fixed_size.rs | 66 ++++++++++++++++++++++++++++++++++++++++++++++++++++++------------
 2 files changed, 126 insertions(+), 20 deletions(-)
```
