## Отображение всей физической памяти в виртуальную

В этой задаче мы реализуем отображение всей физической памяти в виртуальную,
которое понадобится нам в дальнейшем для работы с
деревом отображения виртуальных страниц в физические фреймы.
Для построения этого отображения у нас будет только
[рекурсивная запись](https://os.phil-opp.com/page-tables/#recursive-mapping).
Но её достаточно, чтобы модифицировать всё дерево отображения.
В том числе, её достаточно для того,
чтобы построить линейное отображение всей физической памяти в виртуальную.
За это отвечает структура
[`kernel::memory::phys2virt::Phys2Virt`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html).

Эту задачу можно пропустить, так как минимальное отображение
[`Phys2Virt`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html)
есть в открытом коде.
Тем не менее, её стоит решить для дальнейшего понимания задач про отображение страниц.
В частности, о том откуда берутся большие фреймы и что с ними делать,
когда вы будете натыкаться на них в последующих задачах.


### Задача 3 --- [`Phys2Virt::make()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.make)

Напишите реализацию метода
[`Phys2Virt::make()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.make).

{{#include ../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html.method.make}}

Вы можете придерживаться структуры кода эталонного решения, используя вспомогательные методы:

- [`Phys2Virt::find_empty_root_entries()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.find_empty_root_entries),
- [`Phys2Virt::fill_entries()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.fill_entries),
- [`Phys2Virt::allocate_root_entry()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.allocate_root_entry),
- [`Phys2Virt::node()`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.node).

Или сделать всё по-своему.

Вам могут пригодиться методы:
- [`usize::div_ceil()`](https://doc.rust-lang.org/nightly/core/primitive.usize.html#method.div_ceil),
- [`MaybeUninit::fill()`](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html#method.fill),
- [`Virt::canonize()`](../../doc/ku/memory/addr/type.Virt.html#method.canonize).


### Проверьте себя

Запустите тест:

```console
$ (cd kernel; cargo test --test 2-mm-3-phys2virt)
...
running 1 tests

2_mm_3_phys2virt::basic-------------------------------------
16:15:44 0 D iteration = 0; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_8080_0800_0000), size 128 MiB -> [0p0, 0p800_0000), size 128 MiB; uses 4 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 258; entry_count = 1
16:15:49.193 0 D iteration = 1000; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_8080_C380_0000), size 3.055 GiB -> [0p0, 0pC380_0000), size 3.055 GiB; uses 4 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 258; entry_count = 4
16:15:55.115 0 D iteration = 2000; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_80D5_A000_0000), size 342.500 GiB -> [0p0, 0p55_A000_0000), size 342.500 GiB; uses 4 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 258; entry_count = 343
16:15:58.191 0 D iteration = 2340; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_8140_0000_0000), size 768 GiB -> [0p0, 0pC0_0000_0000), size 768 GiB; uses 8 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 259; entry_count = 768
16:15:59.113 0 D iteration = 2360; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_8640_0000_0000), size 5.750 TiB -> [0p0, 0p5C0_0000_0000), size 5.750 TiB; uses 48 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 269; entry_count = 5888
16:16:01.277 0 D iteration = 2380; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_8B40_0000_0000), size 10.750 TiB -> [0p0, 0pAC0_0000_0000), size 10.750 TiB; uses 88 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 279; entry_count = 11008
16:16:04.679 0 D iteration = 2400; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_9040_0000_0000), size 15.750 TiB -> [0p0, 0pFC0_0000_0000), size 15.750 TiB; uses 128 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 289; entry_count = 16128
16:16:09.323 0 D iteration = 2420; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_9540_0000_0000), size 20.750 TiB -> [0p0, 0p14C0_0000_0000), size 20.750 TiB; uses 168 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 299; entry_count = 21248
16:16:15.227 0 D iteration = 2440; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_9A40_0000_0000), size 25.750 TiB -> [0p0, 0p19C0_0000_0000), size 25.750 TiB; uses 208 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 309; entry_count = 26368
16:16:22.341 0 D iteration = 2460; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_9F40_0000_0000), size 30.750 TiB -> [0p0, 0p1EC0_0000_0000), size 30.750 TiB; uses 248 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 319; entry_count = 31488
16:16:30.683 0 D iteration = 2480; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_A440_0000_0000), size 35.750 TiB -> [0p0, 0p23C0_0000_0000), size 35.750 TiB; uses 288 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 329; entry_count = 36608
16:16:40.321 0 D iteration = 2500; phys2virt = [0vFFFF_8080_0000_0000, 0vFFFF_AE80_0000_0000), size 46 TiB -> [0p0, 0p2E00_0000_0000), size 46 TiB; uses 368 KiB for the mapping itself; root_entries_start = 257; root_entries_end = 349; entry_count = 47104
2_mm_3_phys2virt::basic---------------------------- [passed]
16:16:46.225 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/phys2virt.rs | 90 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----------
1 file changed, 79 insertions(+), 11 deletions(-)
```
