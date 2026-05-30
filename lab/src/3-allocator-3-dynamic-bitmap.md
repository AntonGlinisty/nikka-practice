## Динамическое изменение размера битовой карты

В предыдущей задаче мы реализовали битовую карту фиксированного размера
[`ku::collections::bitmap::Bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html).
Но нам также потребуется резервировать место в адресном пространстве под битовую карту,
увеличивать размер битовой карты и удалять выделенную под битовую карту память.
Всё это мы будем делать через постраничный аллокатор
[`ku::allocator::big::BigAllocator`](../../doc/ku/allocator/big/trait.BigAllocator.html).

Для динамического управления размером
[`ku::collections::bitmap::Bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html)
обернём её в структуру
[`ku::collections::dynamic_bitmap::DynamicBitmap`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html):

```rust
{{#include ../../ku/src/collections/dynamic_bitmap.rs:bitmap}}
```

В дополнение к знакомой нам битовой карте
[`DynamicBitmap::bitmap`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#structfield.bitmap)
эта структура хранит зарезервированный под неё диапазон страниц
[`DynamicBitmap::pages`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#structfield.pages).
Причём, он включает дополнительные защитные области по
[`DynamicBitmap::guard_zone_page_count`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#structfield.guard_zone_page_count)
страниц с обоих сторон от битовой карты.
Эти зоны не будут отображены в физическую память.
И предназначены для защиты битовой карты --- метаданных аллокатора ---
от переполнения пользователем аллокатора своих буферов.

В этой задаче мы не будем экономить виртуальное адресное пространство ---
в 64-х битном режиме оно практически неограниченно и является виртуальным ресурсом.
Поэтому в виртуальном адресном пространстве будем резервировать блок максимального
размера, который нам понадобится.
Это позволит расширять битовую карту без необходимости переносить её данные
по другим виртуальным адресам.
В прочем, с помощью
[`BigAllocator::copy_mapping()`](../../doc/ku/allocator/big/trait.BigAllocator.html#tymethod.copy_mapping)
это несложно было бы сделать.

Реализуйте метод
[`DynamicBitmap::reserve()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.reserve)

```rust
{{#include ../../ku/src/collections/dynamic_bitmap.rs:reserve}}
```

в файле
[`ku/src/collections/dynamic_bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/collections/dynamic_bitmap.rs).

Получить
[`BigAllocator`](../../doc/ku/allocator/big/trait.BigAllocator.html)
из аргумента `allocator` можно операцией `allocator.get()`.
Также вам могут пригодиться методы

- [`BigAllocator::reserve()`](../../doc/ku/allocator/big/trait.BigAllocator.html#tymethod.reserve),
- [`DynamicBitmap::page_count()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.page_count),
- [`Page::layout()`](../../doc/ku/memory/frage/struct.Frage.html#method.layout),
- [`Page::layout_array()`](../../doc/ku/memory/frage/struct.Frage.html#method.layout_array).

А вот физические страницы под битовую карту мы будем выделять экономно, ---
в минимальном количестве, которое нужно для данной битовой карты в данный момент.
Поэтому метод
[DynamicBitmap::map()](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.map)
может вызываться несколько раз с динамически увеличивающимся размером.
Уменьшение размера можно не поддерживать.
Также можно паниковать, если размер превышает возможности зарезервированного
в предыдущем методе блока виртуальной памяти.

Реализуйте метод
[`DynamicBitmap::map()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.map)

```rust
{{#include ../../ku/src/collections/dynamic_bitmap.rs:map}}
```

Этот метод возвращает количество физических фреймов,
которые ему пришлось выделить дополнительно к имевшимся ранее.
Это число может быть нулевое, даже если новый размер больше старого.
А используется оно в
[статистиках аллокатора](../../lab/book/3-allocator-1-big.html#%D0%A1%D1%82%D0%B0%D1%82%D0%B8%D1%81%D1%82%D0%B8%D0%BA%D0%B8-%D0%B0%D0%BB%D0%BB%D0%BE%D0%BA%D0%B0%D1%82%D0%BE%D1%80%D0%B0-kuallocatorinfoinfo).

Вам могут пригодиться методы

- [`Block::slice()`](../../doc/ku/memory/block/struct.Block.html#method.slice),
- [`DynamicBitmap::page_count()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.page_count),
- [`DynamicBitmap::pages()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.pages).

Удаляться битовая карта будет только один раз,
вместе с удалением аллокатора, который её использует.
Это понадобится, в частности, для проверки что в аллокаторе нет утечек физических фреймов.

Реализуйте метод
[`DynamicBitmap::unmap()`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#method.unmap)

```rust
{{#include ../../ku/src/collections/dynamic_bitmap.rs:unmap}}
```

Не забудьте удалить и зарезервированный под битовую карту блок виртуальной памяти
[`DynamicBitmap::pages`](../../doc/ku/collections/dynamic_bitmap/struct.DynamicBitmap.html#structfield.pages).
Верните количество освободившихся физических страниц.


### Проверьте себя

Теперь должны заработать тесты в файле
[`ku/tests/3-allocator-3-dynamic-bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-3-dynamic-bitmap.rs):

```console
$ (cd ku && cargo test --test 3-allocator-3-dynamic-bitmap)
...
running 1 test
2025-11-02T19:38:28.376526Z : basic len=1 free_bits=1
2025-11-02T19:38:28.376543Z : basic len=2 free_bits=1
2025-11-02T19:38:28.376547Z : basic len=3 free_bits=2
2025-11-02T19:38:28.376549Z : basic len=4 free_bits=3
2025-11-02T19:38:28.376683Z : basic len=32763 free_bits=32762
2025-11-02T19:38:28.377459Z : basic len=32764 free_bits=31878
2025-11-02T19:38:28.378130Z : basic len=32765 free_bits=31879
2025-11-02T19:38:28.378787Z : basic len=32766 free_bits=31880
2025-11-02T19:38:28.379572Z : basic len=32767 free_bits=31881
2025-11-02T19:38:28.380236Z : basic len=32768 free_bits=31882
2025-11-02T19:38:28.380921Z : basic len=32769 free_bits=31883
2025-11-02T19:38:28.381579Z : basic len=32770 free_bits=31884
2025-11-02T19:38:28.382230Z : basic len=32771 free_bits=31885
2025-11-02T19:38:28.382884Z : basic len=32772 free_bits=31886
2025-11-02T19:38:28.383677Z : basic len=65531 free_bits=64645
2025-11-02T19:38:28.385124Z : basic len=65532 free_bits=63760
2025-11-02T19:38:28.386463Z : basic len=65533 free_bits=63761
2025-11-02T19:38:28.387867Z : basic len=65534 free_bits=63762
2025-11-02T19:38:28.389173Z : basic len=65535 free_bits=63763
2025-11-02T19:38:28.390487Z : basic len=65536 free_bits=63764
2025-11-02T19:38:28.391866Z : basic len=65537 free_bits=63765
2025-11-02T19:38:28.393193Z : basic len=65538 free_bits=63766
2025-11-02T19:38:28.394497Z : basic len=65539 free_bits=63767
2025-11-02T19:38:28.395846Z : basic len=65540 free_bits=63768
2025-11-02T19:38:28.397292Z : basic len=98299 free_bits=96527
2025-11-02T19:38:28.399508Z : basic len=98300 free_bits=95643
2025-11-02T19:38:28.401462Z : basic len=98301 free_bits=95644
2025-11-02T19:38:28.403414Z : basic len=98302 free_bits=95645
2025-11-02T19:38:28.405316Z : basic len=98303 free_bits=95646
2025-11-02T19:38:28.407180Z : basic len=98304 free_bits=95647
2025-11-02T19:38:28.409095Z : basic len=98305 free_bits=95648
2025-11-02T19:38:28.411088Z : basic len=98306 free_bits=95649
2025-11-02T19:38:28.413033Z : basic len=98307 free_bits=95650
2025-11-02T19:38:28.414947Z : basic len=98308 free_bits=95651
2025-11-02T19:38:28.416975Z : basic len=131067 free_bits=128410
2025-11-02T19:38:28.419807Z : basic len=131068 free_bits=127525
2025-11-02T19:38:28.422416Z : basic len=131069 free_bits=127526
2025-11-02T19:38:28.425089Z : basic len=131070 free_bits=127527
2025-11-02T19:38:28.427742Z : basic len=131071 free_bits=127528
2025-11-02T19:38:28.430323Z : basic len=131072 free_bits=127529
test basic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```


### Ориентировочный объём работ этой части лабораторки

```console
src/collections/dynamic_bitmap.rs | 86 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++------
1 file changed, 80 insertions(+), 6 deletions(-)
```
