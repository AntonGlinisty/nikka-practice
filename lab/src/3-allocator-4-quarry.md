## Нарезка больших блоков на маленькие

Маленькими блоками в аллокаторе будет управлять структура
[`ku::allocator::quarry::Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html):

```rust
{{#include ../../ku/src/allocator/quarry.rs:quarry}}
```

Как и в предыдущей задаче, будем экономить физические фреймы и
более свободно расходовать виртуальное адресное пространство.
Поэтому резервировать в виртуальной памяти будем сразу блоками по
[`Quarry::SLAB_PAGE_COUNT`](../../doc/ku/allocator/quarry/struct.Quarry.html#associatedconstant.SLAB_PAGE_COUNT)
страниц, которые в этом файле называются
[плитами (slab)](https://en.wikipedia.org/wiki/Slab_allocation).
Все плиты имеют одинаковый размер и выровнены на границу страницы,
поэтому для каждого в поле
[`Quarry::slabs`](../../doc/ku/allocator/quarry/struct.Quarry.html#structfield.slabs)
хранится только номер первой его страницы.
Или
[`Page::default()`](../../doc/ku/memory/frage/struct.Frage.html#method.default),
если плита не зарезервирована.
Чтобы вычислить весь блок виртуальной памяти, который зарезервирован под плиту,
есть вспомогательные методы
[`Quarry::slab_at()`](../../doc/ku/allocator/quarry/struct.Quarry.html#method.slab_at)
и
[`Quarry::slab()`](../../doc/ku/allocator/quarry/struct.Quarry.html#method.slab).

У каждого
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html)
есть от нуля до
[`Quarry::SLAB_COUNT`](../../doc/ku/allocator/quarry/struct.Quarry.html#associatedconstant.SLAB_COUNT)
уже зарезервированных плит.
Столько, сколько нужно для маленьких блоков, которые он выдал.
Эти маленькие блоки, выдаваемые пользователю аллокатора, в файле называются `allocation`.
Каждая плита резервируется в адресном пространстве целиком.
Если бы мы зарезервировали все плиты сразу, то даже в виртуальном адресном пространстве
заняли бы слишком большой объём.
Дело в том, что есть очень много
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html) ---
по количеству размеров маленьких блоков, которые выдаёт аллокатор.
Полный размер каждого
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html)
тоже достаточно большой, чтобы гарантированно вместить все
маленькие блоки соответствующего размера.

Размер маленьких блоков, на который
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html)
разрезает каждую плиту, хранится в поле
[`Quarry::size`](../../doc/ku/allocator/quarry/struct.Quarry.html#structfield.size).
А количество таких блоков, которые помещаются в одну плиту --- в поле
[`Quarry::slab_allocation_count`](../../doc/ku/allocator/quarry/struct.Quarry.html#structfield.slab_allocation_count).

Реализуйте
[метод](../../doc/ku/allocator/quarry/struct.Quarry.html#method.reserve)

```rust
{{#include ../../ku/src/allocator/quarry.rs:reserve}}
```

в файле
[`ku/src/allocator/quarry.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/quarry.rs).

Как и в предыдущей задаче, физические фреймы мы будем выделять максимально экономно.
Доработайте
[метод](../../doc/ku/allocator/quarry/struct.Quarry.html#method.map)

```rust
{{#include ../../ku/src/allocator/quarry.rs:map}}
```

Этот метод возвращает количество физических фреймов,
которые ему пришлось выделить дополнительно к имевшимся ранее.
Возвращаемый результат используется в
[статистиках аллокатора](../../lab/book/3-allocator-1-big.html#%D0%A1%D1%82%D0%B0%D1%82%D0%B8%D1%81%D1%82%D0%B8%D0%BA%D0%B8-%D0%B0%D0%BB%D0%BB%D0%BE%D0%BA%D0%B0%D1%82%D0%BE%D1%80%D0%B0-kuallocatorinfoinfo).
При этом может оказаться, что после расширения
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html)
новое количество маленьких блоков, которое помещается в его физическую память,
больше запрошенного `new_allocation_count`.
В этом случае поправьте значение аргумента `new_allocation_count`,
это позволит уменьшить общее количество вызовов
[`Quarry::map()`](../../doc/ku/allocator/quarry/struct.Quarry.html#method.map).

Вам может пригодиться вспомогательный метод
[`Quarry::allocation_coordinates()`](../../doc/ku/allocator/quarry/struct.Quarry.html#method.allocation_coordinates),
который придётся реализовать самостоятельно.

Как и битовые карты, удаляться
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html)
будет только один раз, вместе с удалением аллокатора.
Это понадобится, в частности, для проверки что в аллокаторе нет утечек физических фреймов.

Реализуйте
[метод](../../doc/ku/allocator/quarry/struct.Quarry.html#method.unmap)

```rust
{{#include ../../ku/src/allocator/quarry.rs:unmap}}
```

Не забудьте удалить и зарезервированные под плиты блоки виртуальной памяти.
Верните количество освободившихся физических страниц.

Нам также потребуется находить адрес выделяемого маленького блока по его индексу в
[`Quarry`](../../doc/ku/allocator/quarry/struct.Quarry.html).
И наоборот, по адресу освобождаемого маленького блока найти его индекс.
Этот индекс сквозной по всем плитам --- нулевой индекс отвечает нулевому маленькому блоку,
выделенному из нулевой плиты, и не относится ни к какой другой плите.

Реализуйте
[метод](../../doc/ku/allocator/quarry/struct.Quarry.html#method.allocation)

```rust
{{#include ../../ku/src/allocator/quarry.rs:allocation}}
```

и обратный к нему
[метод](../../doc/ku/allocator/quarry/struct.Quarry.html#method.allocation_index)

```rust
{{#include ../../ku/src/allocator/quarry.rs:allocation_index}}
```


### Проверьте себя

Теперь должны заработать тесты в файле
[`ku/tests/3-allocator-4-quarry.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-4-quarry.rs):

```console
$ (cd ku && cargo test --test 3-allocator-4-quarry)
...
running 3 tests
2025-11-02T19:53:50.672432Z : stress size=1
test out_of_bounds - should panic ... ok
test wrong_alignment - should panic ... ok
2025-11-02T19:53:51.688121Z : stress size=19
2025-11-02T19:53:52.673358Z : stress size=46
2025-11-02T19:53:53.701866Z : stress size=75
2025-11-02T19:53:54.695949Z : stress size=102
2025-11-02T19:53:55.696378Z : stress size=1021
2025-11-02T19:53:56.678761Z : stress size=3075
2025-11-02T19:53:57.703558Z : stress size=6146
2025-11-02T19:53:58.688383Z : stress size=9217
2025-11-02T19:53:59.683702Z : stress size=12288
2025-11-02T19:54:00.723047Z : stress size=15360
2025-11-02T19:54:01.678857Z : stress size=18430
2025-11-02T19:54:02.714064Z : stress size=21502
2025-11-02T19:54:03.686254Z : stress size=23555
2025-11-02T19:54:04.680555Z : stress size=26626
2025-11-02T19:54:05.690807Z : stress size=29697
2025-11-02T19:54:06.695952Z : stress size=32768
2025-11-02T19:54:07.715965Z : stress size=35839
2025-11-02T19:54:08.715033Z : stress size=38910
2025-11-02T19:54:09.693635Z : stress size=41981
2025-11-02T19:54:10.674277Z : stress size=44035
2025-11-02T19:54:11.720738Z : stress size=47107
2025-11-02T19:54:12.679654Z : stress size=50177
2025-11-02T19:54:13.673641Z : stress size=53248
2025-11-02T19:54:14.704415Z : stress size=56320
2025-11-02T19:54:15.680981Z : stress size=59391
2025-11-02T19:54:16.691047Z : stress size=62462
2025-11-02T19:54:17.683728Z : stress size=64512
2025-11-02T19:54:18.677293Z : stress size=73727
2025-11-02T19:54:19.683981Z : stress size=86014
2025-11-02T19:54:20.696860Z : stress size=98301
2025-11-02T19:54:21.685623Z : stress size=106499
2025-11-02T19:54:22.690226Z : stress size=118786
2025-11-02T19:54:23.745323Z : stress size=126978
2025-11-02T19:54:24.679912Z : stress size=139261
2025-11-02T19:54:25.678891Z : stress size=147459
2025-11-02T19:54:26.693065Z : stress size=159747
2025-11-02T19:54:27.698986Z : stress size=172034
2025-11-02T19:54:28.712874Z : stress size=184322
2025-11-02T19:54:29.692671Z : stress size=196609
2025-11-02T19:54:30.709041Z : stress size=208896
2025-11-02T19:54:31.677238Z : stress size=221183
2025-11-02T19:54:32.705294Z : stress size=233472
2025-11-02T19:54:33.680905Z : stress size=245759
2025-11-02T19:54:34.775711Z : stress size=253953
2025-11-02T19:54:35.692846Z : stress size=262145
2025-11-02T19:54:36.675853Z : stress size=274433
2025-11-02T19:54:37.705931Z : stress size=286722
2025-11-02T19:54:38.707089Z : stress size=299010
2025-11-02T19:54:39.677083Z : stress size=311297
2025-11-02T19:54:40.719414Z : stress size=323585
2025-11-02T19:54:41.715971Z : stress size=335872
2025-11-02T19:54:42.698173Z : stress size=348158
2025-11-02T19:54:43.702432Z : stress size=360447
2025-11-02T19:54:44.709051Z : stress size=372737
2025-11-02T19:54:45.689507Z : stress size=380930
2025-11-02T19:54:46.690325Z : stress size=393216
2025-11-02T19:54:47.705059Z : stress size=405505
2025-11-02T19:54:48.679126Z : stress size=417793
2025-11-02T19:54:49.682446Z : stress size=430081
test stress has been running for over 60 seconds
2025-11-02T19:54:50.675290Z : stress size=442369
2025-11-02T19:54:51.713806Z : stress size=454657
2025-11-02T19:54:52.690811Z : stress size=466944
2025-11-02T19:54:53.675692Z : stress size=479231
2025-11-02T19:54:54.715683Z : stress size=491519
2025-11-02T19:54:55.702838Z : stress size=503806
2025-11-02T19:54:56.748611Z : stress size=507907
2025-11-02T19:54:57.699155Z : stress size=520191
2025-11-02T19:54:58.708265Z : stress size=532481
2025-11-02T19:54:59.693516Z : stress size=548862
2025-11-02T19:55:00.695256Z : stress size=561153
2025-11-02T19:55:01.672980Z : stress size=573442
2025-11-02T19:55:02.690446Z : stress size=589822
2025-11-02T19:55:03.686448Z : stress size=602112
2025-11-02T19:55:04.673230Z : stress size=614403
2025-11-02T19:55:05.717843Z : stress size=630784
2025-11-02T19:55:06.720341Z : stress size=638976
2025-11-02T19:55:07.690557Z : stress size=651264
2025-11-02T19:55:08.716447Z : stress size=663555
2025-11-02T19:55:09.683847Z : stress size=679933
2025-11-02T19:55:10.706647Z : stress size=692223
2025-11-02T19:55:11.719027Z : stress size=704510
2025-11-02T19:55:12.672594Z : stress size=716798
2025-11-02T19:55:13.707388Z : stress size=729087
2025-11-02T19:55:14.691010Z : stress size=741376
2025-11-02T19:55:15.715758Z : stress size=753666
2025-11-02T19:55:16.698645Z : stress size=761858
2025-11-02T19:55:17.715344Z : stress size=774142
2025-11-02T19:55:18.700770Z : stress size=786430
2025-11-02T19:55:19.680515Z : stress size=798718
2025-11-02T19:55:20.683268Z : stress size=811006
2025-11-02T19:55:21.697366Z : stress size=823294
2025-11-02T19:55:22.705988Z : stress size=831490
2025-11-02T19:55:23.713792Z : stress size=843778
2025-11-02T19:55:24.688046Z : stress size=856064
2025-11-02T19:55:25.697618Z : stress size=868352
2025-11-02T19:55:26.701520Z : stress size=880640
2025-11-02T19:55:27.773575Z : stress size=888833
2025-11-02T19:55:28.707570Z : stress size=897023
2025-11-02T19:55:29.724488Z : stress size=909310
2025-11-02T19:55:30.709627Z : stress size=921597
2025-11-02T19:55:31.721140Z : stress size=929795
2025-11-02T19:55:32.713697Z : stress size=942081
2025-11-02T19:55:33.708737Z : stress size=954366
2025-11-02T19:55:34.680018Z : stress size=962561
2025-11-02T19:55:35.679507Z : stress size=974847
2025-11-02T19:55:36.720030Z : stress size=987133
2025-11-02T19:55:37.685672Z : stress size=995330
2025-11-02T19:55:38.711406Z : stress size=1007617
2025-11-02T19:55:39.689269Z : stress size=1015808
2025-11-02T19:55:40.700864Z : stress size=1023997
2025-11-02T19:55:41.691594Z : stress size=1032193
2025-11-02T19:55:42.687925Z : stress size=1044479
test stress ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 112.51s
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/allocator/quarry.rs | 182 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++------
1 file changed, 170 insertions(+), 12 deletions(-)
```
