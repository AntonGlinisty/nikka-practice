## Аллокатор виртуальных страниц адресного пространства

За выделение последовательно идущих виртуальных страниц отвечает
[`kernel::memory::page_allocator::PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html).
В качестве структуры данных для учёта свободных страниц
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)
использует поле
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block)
типа
[`Block`](../../doc/ku/memory/block/struct.Block.html)`<`[`Page`](../../doc/ku/memory/frage/type.Page.html)`>`.
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block)
содержит полуоткрытый интервал свободных страниц.
Для выделения свободной виртуальной памяти
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)
отщепляет запрошенный объём с конца
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block).

Поле
[`PageAllocator::initial_block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.initial_block)
содержит копию стартового значение поля
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block).
И не меняется при выделениях страниц, в отличие от
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block).
Оно используется при создании копии
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)
в методе
[`PageAllocator::duplicate()`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#method.is_managed).
Копии создаются при порождении новых процессов, и в новом процессе
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html),
отвечающий за пользовательскую часть адресного пространства,
сбрасывается в исходное состояние.

Параметр `T` обобщённого типа [`Block<T>`](../../doc/ku/memory/block/struct.Block.html)
и соответствующее фантомное, не занимающее памяти, поле
[`Block::tag`](../../doc/ku/memory/block/struct.Block.html#structfield.tag)
служат той же цели, что и в
[`Addr`](../../doc/ku/memory/addr/struct.Addr.html)`<T: `[`Tag`](../../doc/ku/memory/addr/trait.Tag.html)`>` ---
сделать блоки с разными параметрами `T` несовместимыми.
Поэтому `Block<Frame>` будет несовместим с `Block<Page>` и компилятор не даст их перепутать.
Подробнее про тип этого фантомного тега можно посмотреть в документации ---
[`core::marker::PhantomData`](https://doc.rust-lang.org/nightly/core/marker/struct.PhantomData.html).

Инициализировать
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block)
предлагается следующим способом.
Пройдём по таблице страниц корневого уровня.
Пусть, например, оказалось, что запись номер \\( 17 \\) корневого уровня свободна.
Тогда свободны все виртуальные адреса в полуинтервале
\\( [17 \cdot 2^{9 + 9 + 9 + 12}, 18 \cdot 2^{9 + 9 + 9 + 12}) \\).
Это дает \\( 2^{9 + 9 + 9} \\) свободных виртуальных страниц,
то есть 512 GiB адресного пространства на каждую такую запись.
Действительно, каждая запись в узле таблицы страниц на уровне
[`ku::memory::mmu::PAGE_TABLE_LEAF_LEVEL`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_LEAF_LEVEL.html)`== 0`
ссылается на виртуальную страницу, то есть ссылается на блок памяти размера 4KiB или \\( 2^{12} \\) байт.
Каждая запись в узле более высокого уровня ссылается на
[`ku::memory::mmu::PAGE_TABLE_ENTRY_COUNT`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ENTRY_COUNT.html)
записей более низкого уровня.
То есть через неё адресуется в
[`ku::memory::mmu::PAGE_TABLE_ENTRY_COUNT`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ENTRY_COUNT.html)`== 512`,
или в \\( 2^{9} \\),
раз больше байт.
Поэтому, если
[`ku::memory::mmu::PAGE_TABLE_ROOT_LEVEL`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ROOT_LEVEL.html)`== 3`,
то всего выходит 512 * 512 * 512 * 4KiB = 512 GiB, или \\( 2^{9 + 9 + 9 + 12} \\) байт.
Если найдем в таблице страниц корневого уровня самую длинную последовательность подряд идущих свободных записей ---
добудем много подряд идущих свободных виртуальных страниц.
Почти все \\( 2^9 \\) записи корневого уровня свободны,
а значит получаем почти по 128 TiB непрерывного свободного адресного пространства в каждой из
[двух половин](../../lab/book/2-mm-0-intro-1-types.html#%D0%94%D0%B2%D0%B5-%D0%BF%D0%BE%D0%BB%D0%BE%D0%B2%D0%B8%D0%BD%D1%8B-%D0%B2%D0%B8%D1%80%D1%82%D1%83%D0%B0%D0%BB%D1%8C%D0%BD%D0%BE%D0%B3%D0%BE-%D0%B0%D0%B4%D1%80%D0%B5%D1%81%D0%BD%D0%BE%D0%B3%D0%BE-%D0%BF%D1%80%D0%BE%D1%81%D1%82%D1%80%D0%B0%D0%BD%D1%81%D1%82%D0%B2%D0%B0).
При этом рассматривать будем только записи корневого узла таблицы страниц,
спускаться на следующие уровни не будем.

Для простоты реализации
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)
не умеет ничего освобождать.
Виртуальных страниц достаточно много --- сотни TiB, и они не потребляют память.
Кроме того, адресное пространство будет отдельное на каждый процесс.
И даже если процесс исчерпает все виртуальные страницы своего адресного пространства,
на другие процессы это не повлияет.
Поэтому с такой реализацией аллокатора страниц проблем не возникнет.


### Граница между страницами для ядра и для пространства пользователя

Удобно провести в виртуальном адресном пространстве чёткую границу
между адресами, которые выделяются для ядра, и адресами, которые выделяются
для пространства пользователя.
Причём удобнее всего,
если эта граница будет проходить по границе записи корневого уровня таблицы страниц.

В Nikka мы зарезервируем
[`kernel::memory::range::KERNEL_ROOT_LEVEL_ENTRY_COUNT`](../../doc/kernel/memory/range/constant.KERNEL_ROOT_LEVEL_ENTRY_COUNT.html)
первых записей корневого уровня таблицы страниц для ядра.
Остальные записи в нижней половине адресного пространства останутся
для пространства пользователя.
А
[верхнюю половину](2-mm-0-intro-1-types.html#%D0%94%D0%B2%D0%B5-%D0%BF%D0%BE%D0%BB%D0%BE%D0%B2%D0%B8%D0%BD%D1%8B-%D0%B2%D0%B8%D1%80%D1%82%D1%83%D0%B0%D0%BB%D1%8C%D0%BD%D0%BE%D0%B3%D0%BE-%D0%B0%D0%B4%D1%80%D0%B5%D1%81%D0%BD%D0%BE%D0%B3%D0%BE-%D0%BF%D1%80%D0%BE%D1%81%D1%82%D1%80%D0%B0%D0%BD%D1%81%D1%82%D0%B2%D0%B0)
зарезервируем для дополнительных нужд.
В частности, в ней будет находиться линейное отображение физической памяти ---
[`kernel::memory::phys2virt::Phys2Virt`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html).

Это разделение фиксируют функции
[`kernel::memory::range::kernel_root_level_entries()`](../../doc/kernel/memory/range/fn.kernel_root_level_entries.html)
и
[`kernel::memory::range::user_root_level_entries()`](../../doc/kernel/memory/range/fn.user_root_level_entries.html).
Они возвращают диапазоны ---
[`core::ops::Range<usize>`](https://doc.rust-lang.org/nightly/core/ops/struct.Range.html) ---
номеров записей корневого уровня таблицы страниц,
зарезервированных для ядра и пользовательского пространства соответственно.
А функции
[`kernel::memory::range::is_user_page(page: Page)`](../../doc/kernel/memory/range/fn.is_user_page.html)
и
[`kernel::memory::range::is_user_block(block: Block<Page>)`](../../doc/kernel/memory/range/fn.is_user_block.html)
проверяют, что заданная страница или блок страниц целиком лежит в зарезервированной для
пространства пользователя области.
В свою очередь функции
[`kernel::memory::range::validate_page_flags(page: Page, flags: PageTableFlags)`](../../doc/kernel/memory/range/fn.validate_page_flags.html)
и
[`kernel::memory::range::validate_block_flags(block: Block<Page>, flags: PageTableFlags)`](../../doc/kernel/memory/range/fn.is_user_block.html)
проверяют соответствие между диапазоном, в котором находится страница или блок,
и `flags.is_user()`.
В случае несоответствия они возвращают ошибку
[`Error::PermissionDenied`](../../doc/kernel/error/enum.Error.html#variant.PermissionDenied).

За выделение страниц в каждой из этих областей, будет отвечать свой экземпляр аллокатора страниц.
Выделять страницы для ядра будет глобальный
[`kernel::memory::KERNEL_PAGE_ALLOCATOR`](../../doc/kernel/memory/struct.KERNEL_PAGE_ALLOCATOR.html).
А страницы для пространства пользователя --- локальный для каждого адресного пространства
[`kernel::memory::address_space::AddressSpace::user_page_allocator`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#structfield.user_page_allocator).
Функции, которые выделяют виртуальные адреса,
должны будут по передаваемым им на вход флагам для отображения страницы,
выбрать соответствующий аллокатор.
А функции, принимающие на вход и флаги и уже выделенный адрес страницы,
и отображающие её в физическую память,
будут проверять, что страница попадает в область, допустимую для заданных флагов.


### Задача 2 --- [`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)


#### Инициализация аллокатора страниц

Реализуйте вспомогательную функцию
[`PageAllocator::find_unused_block()`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#method.find_unused_block)
в файле [`kernel/src/memory/page_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/page_allocator.rs).

{{#include ../../doc/kernel/memory/page_allocator/struct.PageAllocator.html.method.find_unused_block}}

Эта функция принимает на вход таблицу страниц
[`ku::memory::mmu::PageTable`](../../doc/ku/memory/mmu/type.PageTable.html)
корневого уровня и `allowed_elements` ---
допустимый диапазон записей этой таблицы страниц,
которые может использовать инициализируемый аллокатор.
А вернуть она должна блок страниц
[`Block<Page>`](../../doc/ku/memory/block/struct.Block.html),
соответствующий самой длинной последовательности пустых записей в `page_table_root`,
ограниченной заданным `allowed_elements`.
Как вы уже догадались, в качестве `allowed_elements` будет передаваться
[`range::kernel_root_level_entries()`](../../doc/kernel/memory/range/fn.kernel_root_level_entries.html)
или
[`range::user_root_level_entries()`](../../doc/kernel/memory/range/fn.user_root_level_entries.html).
Если все записи заняты, верните `None`.

Узел дерева многоуровневой таблицы страниц
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html)
это просто массив из
[`ku::memory::mmu::PAGE_TABLE_ENTRY_COUNT`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ENTRY_COUNT.html)
элементов типа
[`ku::memory::mmu::PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html).
С
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
мы будем ещё много работать.
Пока что скорее всего хватит её метода
[`fn PageTableEntry::is_present(&self) -> bool`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.is_present).
Он возвращает `true`, если запись используется.

Вам может пригодиться константа
[`kernel::memory::range::PAGES_PER_ROOT_LEVEL_ENTRY`](../../doc/kernel/memory/range/constant.PAGES_PER_ROOT_LEVEL_ENTRY.html) ---
количество виртуальных страниц, за которые отвечает одна запись корневого узла отображения страниц.
Также могут пригодиться и
[другие константы](../../doc/ku/memory/mmu/index.html#constants)
из файла [`ku/src/memory/mmu.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/memory/mmu.rs).

При инициализации ядра
[`PageAllocator`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html)
запишет в журнал примерно такое сообщение про выделенное для ядра пространство страниц:
```console
20:44:19 0 I page allocator init; free_page_count = 3758096384; block = [0v180_0000_0000, 0vF80_0000_0000), size 14.000 TiB
```


#### Выделение виртуальных страниц

Реализуйте метод
[`Block::tail()`](../../doc/ku/memory/block/struct.Block.html#method.tail)

```rust
fn Block::tail(&mut self, count: usize) -> Option<Self>
```

в файле [`ku/src/memory/block.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/memory/block.rs).
Он должен разделить блок `self` на две дизъюнктные части:
- новое значение `self` и
- новый [`Block`](../../doc/ku/memory/block/struct.Block.html) размером `count` единиц.

То есть, от существующего блока он откусывает хвост с заданным количеством единиц, и возвращает его в виде нового блока `Self::new(...)`, завернув в
[`core::option::Option::Some`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.Some):
`Some(Self::new(...))`.
Если в исходном блоке нет запрошенного количества, не меняйте его и верните
[`core::option::Option::None`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None), например
```rust
pub fn tail(&mut self, count: usize) -> Option<Self> {
    ...
    return None;
    ...
}
```
или просто `None`, если это последнее выражение в функции:
```rust
pub fn tail(&mut self, count: usize) -> Option<Self> {
    ...
    None
}
```
Проверьте свою реализацию запустив тест:
```console
$ (cd kernel; cargo test --test 2-mm-2-block)
...
2_mm_2_block::block-----------------------------------------
17:21:00 0 D start = 0; end = 0; block = [0v0, 0v0), size 0 B
17:21:00 0 D start = 0; end = 33; block = [0v0, 0v21), size 33 B
17:21:00 0 D start = 0; end = 66; block = [0v0, 0v42), size 66 B
17:21:00 0 D start = 0; end = 99; block = [0v0, 0v63), size 99 B
17:21:00 0 D start = 25; end = 33; block = [0v19, 0v21), size 8 B
17:21:00 0 D start = 25; end = 66; block = [0v19, 0v42), size 41 B
17:21:00 0 D start = 25; end = 99; block = [0v19, 0v63), size 74 B
17:21:01 0 D start = 50; end = 66; block = [0v32, 0v42), size 16 B
17:21:01 0 D start = 50; end = 99; block = [0v32, 0v63), size 49 B
17:21:01 0 D start = 50; end = 132; block = [0v32, 0v84), size 82 B
17:21:01 0 D start = 75; end = 99; block = [0v4B, 0v63), size 24 B
17:21:01 0 D start = 75; end = 132; block = [0v4B, 0v84), size 57 B
17:21:01 0 D start = 75; end = 165; block = [0v4B, 0vA5), size 90 B
2_mm_2_block::block-------------------------------- [passed]
17:21:01 0 I exit qemu; exit_code = SUCCESS
```

Применив [`Block::tail()`](../../doc/ku/memory/block/struct.Block.html#method.tail)
к полю
[`PageAllocator::block`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#structfield.block)
реализуйте метод
[`PageAllocator::allocate()`](../../doc/kernel/memory/page_allocator/struct.PageAllocator.html#method.allocate)
в файле [`kernel/src/memory/page_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/page_allocator.rs).

{{#include ../../doc/kernel/memory/page_allocator/struct.PageAllocator.html.method.allocate}}

Она принимает размер и выравнивание запрошенного блока в виде структуры
[`core::alloc::Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html).
И возвращает подходящий блок свободных виртуальных страниц
[`Block<Page>`](../../doc/ku/memory/block/struct.Block.html):

- Размер блока должен совпадать с запрошенным размером [`Layout::size()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.size).
- А его выравнивание --- с [`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.align). То есть, начальный адрес блока должен делиться нацело на [`Layout::align()`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html#method.align).

Если свободных страниц не хватает, верните ошибку
[`kernel::error::Error::NoPage`](../../doc/kernel/error/enum.Error.html#variant.NoPage).

Превратить
[`Option::None`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None)
в
[`Error::NoFrame`](../../doc/kernel/error/enum.Error.html#variant.NoFrame)
можно с помощью метода
[`Option::ok_or()`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#method.ok_or):
```rust
fn allocate_block(&mut self, size: usize) -> Result<Block<Frame>> {
    ...
    tail.ok_or(NoFrame)
}
```
Если вы забудете превратить `Option` в `Result` и используете оператор `?`, Rust выдаст сообщение об ошибке "the `?` operator can only be used in a function that returns `Result` or `Option`".
```
error[E0277]: the `?` operator can only be used in a method that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> kernel/src/memory/boot_frame_allocator.rs:61:41
   |
59 | /     pub(super) fn allocate_block(&mut self, size: usize) -> Result<Block<Frame>> {
60 | |         ...
61 | |         ...?
   | |            ^ cannot use the `?` operator in a method that returns `Result<Block<Frage<PhysTag>>, ku::Error>`
62 | |     }
   | |_____- this function should return `Result` or `Option` to accept `?`
   |
   = help: the trait `FromResidual<core::option::Option<Infallible>>` is not implemented for `Result<Block<Frage<PhysTag>>, ku::Error>`
   = help: the following other types implement trait `FromResidual<R>`:
             <Result<T, F> as FromResidual<Result<Infallible, E>>>
             <Result<T, F> as FromResidual<Yeet<E>>>

For more information about this error, try `rustc --explain E0277`.
```
Аналогичная ошибка будет при применении оператора `?` к `Result`, если функция возвращает `Option`.

Вам может пригодиться функция
[`fn Page::count_up(size: usize) -> usize`](../../doc/kernel/memory/frage/struct.Frage.html#method.count_up).
Она реализует формулу \\( \lceil \frac{\texttt{size}}{\texttt{Page::SIZE}} \rceil \\),
возвращая количество страниц, которое требуется для хранения `size` байт.
Также могут пригодиться:
- Метод [`Option::ok_or()`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#method.ok_or):
`tail.ok_or(NoPage)`.
- Вычитание с проверкой переполнения [`usize::checked_sub()`](https://doc.rust-lang.org/nightly/core/primitive.usize.html#method.checked_sub).
- Методы структуры [`Layout`](https://doc.rust-lang.org/nightly/core/alloc/struct.Layout.html).


### Проверьте себя

Запустите тесты:

```console
$ (cd kernel; cargo test --test 2-mm-2-block --test 2-mm-2-page-allocator)
...
2_mm_2_block::block-----------------------------------------
18:00:54 0 D start = 0; end = 0; block = [0v0, 0v0), size 0 B
18:00:54 0 D start = 0; end = 33; block = [0v0, 0v21), size 33 B
18:00:54 0 D start = 0; end = 66; block = [0v0, 0v42), size 66 B
18:00:54 0 D start = 0; end = 99; block = [0v0, 0v63), size 99 B
18:00:54 0 D start = 25; end = 33; block = [0v19, 0v21), size 8 B
18:00:54 0 D start = 25; end = 66; block = [0v19, 0v42), size 41 B
18:00:54 0 D start = 25; end = 99; block = [0v19, 0v63), size 74 B
18:00:55 0 D start = 50; end = 66; block = [0v32, 0v42), size 16 B
18:00:55 0 D start = 50; end = 99; block = [0v32, 0v63), size 49 B
18:00:55 0 D start = 50; end = 132; block = [0v32, 0v84), size 82 B
18:00:55 0 D start = 75; end = 99; block = [0v4B, 0v63), size 24 B
18:00:55 0 D start = 75; end = 132; block = [0v4B, 0v84), size 57 B
18:00:55 0 D start = 75; end = 165; block = [0v4B, 0vA5), size 90 B
2_mm_2_block::block-------------------------------- [passed]
18:00:55 0 I exit qemu; exit_code = SUCCESS
...
2_mm_2_page_allocator::sanity_check-------------------------
18:00:57 0 D page_allocator_block = [0v18000000000, 0v7FFFFFFFF000), size 126.500 TiB, Page[~1.500 TiB, ~128.000 TiB)
2_mm_2_page_allocator::sanity_check---------------- [passed]

2_mm_2_page_allocator::allocate_page------------------------
18:00:57 0 D page = Page(34359738366 @ 0v7FFFFFFFE000)
2_mm_2_page_allocator::allocate_page--------------- [passed]

2_mm_2_page_allocator::allocate_two_pages-------------------
18:00:57 0 D pages = [Page(34359738365 @ 0v7FFFFFFFD000), Page(34359738364 @ 0v7FFFFFFFC000)]
2_mm_2_page_allocator::allocate_two_pages---------- [passed]

2_mm_2_page_allocator::allocate_block-----------------------
18:00:57 0 D requested_size = 38.000 KiB; block = [0v7FFFFFFF2000, 0v7FFFFFFFC000), size 40.000 KiB, Page[~128.000 TiB, ~128.000 TiB)
2_mm_2_page_allocator::allocate_block-------------- [passed]
18:00:57 0 I exit qemu; exit_code = SUCCESS
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/page_allocator.rs | 45 +++++++++++++++++++++++++++++++++++++++++----
ku/src/memory/block.rs              | 12 ++++++++++--
2 files changed, 51 insertions(+), 6 deletions(-)
```
