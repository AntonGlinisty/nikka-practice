## Отображение виртуальных страниц в физические фреймы

За это отображение отвечает структура
[`kernel::memory::mapping::Mapping`](../../doc/kernel/memory/mapping/struct.Mapping.html).
Фактически она реализует дерево большой арности --- 512, если закрыть глаза на возможность сослаться в узле на любой узел.
Такая возможность используется только в специфических случаях, например для реализации [рекурсивного отображения](https://wiki.osdev.org/Page_Tables#Recursive_mapping).

[`Mapping`](../../doc/kernel/memory/mapping/struct.Mapping.html) содержит уже немного знакомые нам поля:

- [`Mapping::page_table_root`](../../doc/kernel/memory/mapping/struct.Mapping.html#structfield.page_table_root) типа [`Frame`](../../doc/ku/memory/frage/type.Frame.html) --- физический фрейм с таблицей страниц самого верхнего уровня, корневой в дереве.
- [`Mapping::phys2virt`](../../doc/kernel/memory/mapping/struct.Mapping.html#structfield.phys2virt) --- "окно", в которое отображена вся физическая память.

[`Mapping::phys2virt`](../../doc/kernel/memory/mapping/struct.Mapping.html#structfield.phys2virt)
нужен, чтобы с помощью функции
[`Phys2Virt::map(&self, phys: Phys) -> Result<Virt>`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html#method.map).
по заданному физическому адресу, найти куда он отображён в "окне".
Это потребуется когда нам нужно будет записать в какую-нибудь
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html).
Так как в
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html) таблица
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html) следующего вниз уровня
задаётся именно физическим адресом.

> Подумайте, а можно ли было бы в
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
> хранить виртуальный адрес
> [`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html),
> на которую ссылается данная
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)?

Логически отображение виртуальных страниц на физические фреймы
можно изобразить в виде огромной таблицы:

![](2-mm-4-virt-to-phys.svg)

Про отображение страниц можно почитать на сайте [osdev](https://wiki.osdev.org/), нас интересует 64-битный вариант для x86-64:

- [Memory management](https://wiki.osdev.org/Memory_management).
- [Paging](https://wiki.osdev.org/Paging).

Ещё более подробный и обстоятельный разбор есть в блоге [Writing an OS in Rust](https://os.phil-opp.com/):

- [Introduction to Paging](https://os.phil-opp.com/paging-introduction/), [перевод](https://habr.com/ru/post/436606/).
- [Paging Implementation](https://os.phil-opp.com/paging-implementation/), [перевод](https://habr.com/ru/post/445618/).


### Задача 4 --- отображение виртуальных страниц на физические фреймы

Страничное преобразование устроено как показано на схеме.
Стрелки ведут из физических адресов, хранящихся в регистре
[`CR3`](https://wiki.osdev.org/CPU_Registers_x86#CR3)
и в элементах
[`ku::memory::mmu::PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
таблиц
[`ku::memory::mmu::PageTable`](../../doc/ku/memory/mmu/type.PageTable.html),
в целевые фреймы и целевой байт.
А пунктиром показано какая часть битового представления виртуального адреса
([`Virt`](../../doc/ku/memory/addr/type.Virt.html))
используется как индекс в одной из таблиц
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html),
либо как смещение внутри целевого фрейма.
Каждая [`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html) занимает ровно один фрейм физической памяти.

![](2-mm-4-translate.svg)

Вооружившись этими знаниями, реализуйте метод
[`Mapping::path()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.path)
```rust
fn Mapping::path(
    &mut self,
    virt: Virt,
) -> Path
```

в файле [`kernel/src/memory/mapping.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/mapping.rs).
Он принимает на вход виртуальный адрес `virt`, который нужно транслировать.
И возвращает максимально длинный отображённый в память префикс пути в дереве отображения страниц
[`kernel::memory::path::Path`](../../doc/kernel/memory/path/struct.Path.html),
соответствующий входному виртуальному адресу `virt`.
Элементы
[`Path::nodes`](../../doc/kernel/memory/path/struct.Path.html#structfield.nodes)
заполняются указателями
[`core::ptr::NonNull`](https://doc.rust-lang.org/nightly/core/ptr/struct.NonNull.html)
на соответствующие
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html):

- В `nodes[i]` лежит указатель на [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html) из [`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html) `i`-ого уровня. То есть из Li на картинке.
- Если путь не ведёт до самого низа, в не задействованных элементах [`Path::nodes`](../../doc/kernel/memory/path/struct.Path.html#structfield.nodes) содержится `None`.
- Полезно изучить инварианты [`Path`](../../doc/kernel/memory/path/struct.Path.html),
которые проверяет метод [`Path::validate()`](../../doc/kernel/memory/path/struct.Path.html#method.validate).

[`Mapping::path()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.path)
должен пройти от корневой L3 по физическим фреймам, которые возвращает метод
[`fn PageTableEntry::frame() -> Result<Frame>`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.frame)
для существующих узлов дерева
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html).

Если какая-то из этих
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
не отображена на физическую память или содержит флаг
[`PageTableFlags::HUGE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.HUGE),
на этом построение пути заканчивается.

Теперь должны проходить тесты `t00_path()` и `t01_translate()`
из файла
[`kernel/tests/2-mm-4-translate.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/2-mm-4-translate.rs):

```
$ (cd kernel; cargo test --test 2-mm-4-translate)
...
running 8 tests

2_mm_4_translate::t00_path----------------------------------
16:50:10 0 D present:
16:50:10 0 D     virt = 0v2F_4498
16:50:10 0 D     path = L3: 0vFFFF_8000_0000_1000 / 1 @ 0p1000 -> L2: 0vFFFF_8000_0000_2000 / 2 @ 0p2000 -> L1: 0vFFFF_8000_0000_3008 / 3 @ 0p3000 -> L0: 0vFFFF_8000_0085_F7A0 / 2143 @ 0p85_F000 => 0p4F_1498
16:50:10 0 D     level = 0
16:50:10 0 D     pte = 1265 @ 0p4F_1000 -----DA---W-P(0x8000000000000063)
16:50:10 0 D allocated unique virtual address; virt = 0v1080_0000_0000; is_user = true
16:50:10 0 D non-present:
16:50:10 0 D     virt = 0v1080_0000_0000
16:50:10 0 D     path = L3: 0vFFFF_8000_0000_1108 / 1 @ 0p1000 (non-present)
16:50:10 0 D     level = 3
16:50:10 0 D     pte = <non-present>
16:50:10 0 D huge:
16:50:10 0 D     virt = 0vFFFF_8000_0070_0000
16:50:10 0 D     path = L3: 0vFFFF_8000_0000_1800 / 1 @ 0p1000 -> L2: 0vFFFF_8000_03FF_F000 / 16383 @ 0p3FF_F000 (1 GiB huge page) => 0p70_0000
16:50:10 0 D     level = 2
16:50:10 0 D     pte = 0 @ 0p0 ----HDA---W-P(0x80000000000000E3)
2_mm_4_translate::t00_path------------------------- [passed]

2_mm_4_translate::t01_translate-----------------------------
16:50:10 0 D pte = 1265 @ 0p4F_1000 -----DA---W-P(0x8000000000000063)
16:50:10 0 D read_ptr = 0xffff8000004f1190; write_ptr = 314159266
16:50:10 0 D write_value = 0; read_value = 0; variable = 0
16:50:10 0 D write_value = 1; read_value = 1; variable = 1
16:50:10 0 D write_value = 2; read_value = 2; variable = 2
16:50:10 0 D write_value = 3; read_value = 3; variable = 3
16:50:10 0 D write_value = 4; read_value = 4; variable = 4
2_mm_4_translate::t01_translate-------------------- [passed]

...
```

Далее реализуйте метод
[`Path::map()`](../../doc/kernel/memory/path/struct.Path.html#method.map)
```rust
pub fn Path::map(
    &mut self,
    frame: FrameGuard,
    flags: PageTableFlags,
) -> Result<()>
```
в файле [`kernel/src/memory/path.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/path.rs).

Он должен достроить путь в дереве отображения страниц,
соответствующий виртуальному адресу
[`Path::virt`](../../doc/kernel/memory/path/struct.Path.html#structfield.virt),
до листьевого уровня (L0 на схеме) и отобразить
[`Path::virt`](../../doc/kernel/memory/path/struct.Path.html#structfield.virt)
в физический фрейм, задаваемый `frame` с флагами `flags`.

Недостающие узлы этот метод выделяет с помощью глобального
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html).
Только что выданную
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html)
нужно обязательно очистить ---
все записи в ней должны быть равны значению, которое возвращает метод
[`PageTableEntry::default()`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.default).
Только после этого можно установить этот фрейм в промежуточную
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
методом
[`PageTableEntry::set_frame()`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.set_frame).
Теперь промежуточная таблица отображена в память.

Если в промежуточной таблице встретилась запись, в которой
[`PageTableEntry::flags()`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.flags)
содержит флаг
[`PageTableFlags::HUGE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.HUGE),
верните
[`Error::Unimplemented`](../../doc/kernel/error/enum.Error.html#variant.Unimplemented).
Такая запись не является промежуточной, а задаёт большую страницу, ---
[x86-64 поддерживает](https://en.wikipedia.org/wiki/X86-64#Virtual_address_space_details) страницы
[размеров `4 KiB`, `2 MiB` и `1 GiB`](https://en.wikipedia.org/wiki/Page_(computer_memory)#Multiple_page_sizes).
Мы не будем их поддерживать.
Но они нам будут попадаться, потому что с их помощью
[`kernel::memory::phys2virt::Phys2Virt`](../../doc/kernel/memory/phys2virt/struct.Phys2Virt.html)
отображает всю физическую память в "окно" в виртуальной.
Делается это для экономии физических фреймов на само отображение.

Кроме того, есть ещё такой момент.
Если например, во `flags` стоит флаг
[`PageTableFlags::WRITABLE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITABLE),
и вы установите его только на уровне таблицы L0, этого может оказаться недостаточно.
Дело в том, что процессор пересечёт флаги [`PageTableFlags::WRITABLE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITABLE)
из всех промежуточных
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
по схеме `И`.
И если в какой-то из промежуточных
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
флаг
[`PageTableFlags::WRITABLE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITABLE)
будет сброшен, то запись по виртуальному адресу `virt` будет запрещена,
несмотря на то что в
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
таблицы L0, она разрешена.
Поэтому во всех промежуточных, не только выделяемых,
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
функция
[`Path::map()`](../../doc/kernel/memory/path/struct.Path.html#method.map)
должна будет включить флаги, заданные ей в аргументе `flags`.
И никакие флаги промежуточных
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
она не должна случайно сбросить при этой операции.
Альтернативой было бы либо сразу включать все доступы в промежуточных
[`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html).
Также заметьте, что включать нужно только флаги доступа, они задаются константой
[`ku::memory::mmu::FULL_ACCESS`](../../doc/ku/memory/mmu/constant.FULL_ACCESS.html).
То есть, включать нужно флаги которые есть в пересечении --- `flags & FULL_ACCESS`.

> Про флаг `NO_EXECUTE`
> Intel Software Developer's Manual утверждает, что:
> > Instruction fetches from supervisor-mode addresses:
> >
> > ...
> >
> > For other paging modes with IA32_EFER.NXE = 1, instructions may be fetched from any supervisor-mode address with
> > a translation for which the XD flag (bit 63) is 0 in every paging-structure entry controlling the translation;
> > instructions may not be fetched from any supervisor-mode address with a translation for which the XD flag is 1
> > in any paging-structure entry controlling the translation.
> >
> > --- <cite>Section 5.6.1, Volume 3A</cite>
>
> Для чтения инструкций по
> [`USER`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.USER)
> адресам из кода ядра и кода пользователя там же описаны аналогичные условия. То есть для вычисления эффективного
> значения флага `NO_EXECUTE`
> берется логическое `ИЛИ` всех
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html),
> которые участвуют в трансляции виртуального адреса в физический.
>
> Это неудобно, поэтому в
> [`PageTableFlags`](../../doc/ku/memory/mmu/struct.PageTableFlags.html)
> мы храним обратный флаг
> [`EXECUTABLE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.EXECUTABLE).
> Который ведёт себя по тем же правилам, что флаги
> [`PageTableFlags::USER`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.USER)
> и
> [`PageTableFlags::WRITABLE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITABLE).
> А при чтении и записи в
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
> значение этого флага обращается.
> Это происходит в методах
> [`PageTableEntry::flags()`](file:///.../nikka/doc/ku/memory/mmu/struct.PageTableEntry.html#method.flags)
> и
> [`PageTableEntry::set_phys()`](file:///.../nikka/doc/ku/memory/mmu/struct.PageTableEntry.html#method.set_phys).
>
> Про флаги
> [`NO_CACHE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.NO_CACHE)
> (Page Cache Disable) и
> [`WRITE_THROUGH`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITE_THROUGH)
> (Page Write Through) Intel Software Developer's Manual утверждает, что:
> > For any access to a physical address, the table combines the memory type specified for that physical address by the
> > MTRRs with a memory type selected from the PAT. ... Specifically, it comes from entry i of the PAT, where i is
> > defined as follows:
> >
> > ...
> >
> > For an access to the physical address that is the translation of a linear address, \\( i = 4 \times PAT + 2 \times PCD + PWT \\), where
> > the PAT, PCD, and PWT values come from the relevant PTE.
> >
> > --- <cite>Section 5.9.2, Volume 3A</cite>
>
> В Nikka используются значения PAT по умолчанию, которые обеспечивают поведение совместимое со старыми процессорами,
> а именно эффективное значение флагов
> [`NO_CACHE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.NO_CACHE)
> и
> [`WRITE_THROUGH`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITE_THROUGH)
> берется из последней, листовой
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html).
>
> Поэтому в Nikka мы будем выставлять флаги
> [`NO_CACHE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.NO_CACHE)
> и
> [`WRITE_THROUGH`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITE_THROUGH)
> только в листовые
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html) --- выставлять
> [`NO_CACHE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.NO_CACHE)
> и
> [`WRITE_THROUGH`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITE_THROUGH)
> в промежуточные
> [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html)
> не имеет смысла.

Выделение физического фрейма под отсутствующую промежуточную таблицу
[`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html)
и его зануление
рекомендуется вынести во вспомогательную [функцию](../../doc/kernel/memory/mapping/struct.Mapping.html#method.allocate_node)

```rust
fn allocate_node(
    &mut self,
) -> Result<Frame>
```

в файле [`kernel/src/memory/mapping.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/mapping.rs).

Теперь реализуйте обратный метод
[`Path::unmap()`](../../doc/kernel/memory/path/struct.Path.html#method.unmap)
```rust
pub unsafe fn Path::unmap(&mut self) -> Result<()>
```
в файле [`kernel/src/memory/path.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/path.rs).

Он удаляет отображение страницы по текущему пути.
Физический фрейм освобождается, если на него не осталось других ссылок.
Удалять промежуточные узлы не требуется, даже если они больше не используются.

**Важно** сбросить запись в буфере ассоциативной трансляции
([Translation lookaside buffer](https://en.wikipedia.org/wiki/Translation_lookaside_buffer),
[TLB](https://wiki.osdev.org/TLB))
касающуюся
[`Path::virt`](../../doc/kernel/memory/path/struct.Path.html#structfield.virt).
Иначе процессор может использовать старую закэшерованную в TLB запись,
когда обратится по адресу внутри той же страницы в следующий раз.
А это может быть как уже совершенно другой физический фрейм,
так и намеренно не отображённая на физическую память страница.
Выглядеть с вашей точки зрения это будет так: вы обращаетесь по некоторому адресу,
а там находится совсем не то что вы ожидаете.
Даже отладчик будет показывать вам то же самое, что вы ожидаете.
А вот процессор будет читать и писать совсем в другую физическую память.
Это приведёт к трудно отлаживаемым
[гейзенбагам](https://en.wikipedia.org/wiki/Heisenbug).
Для сброса нужной записи TLB поможет функция
[`kernel::memory::mmu::flush()`](../../doc/kernel/memory/mmu/fn.flush.html).

При реализации `Mapping::path()`, `Mapping::allocate_node()`,
`Path::map()` и `Path::unmap()` вам также могут пригодиться:

- Поле [`Mapping::page_table_root`](../../doc/kernel/memory/mapping/struct.Mapping.html#structfield.page_table_root), оно содержит адрес фрейма корневой таблицы L3, то есть фактически копию регистра [`CR3`](https://wiki.osdev.org/CPU_Registers_x86#CR3).
- Константа [`ku::memory::mmu::PAGE_TABLE_ROOT_LEVEL`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ROOT_LEVEL.html) --- та самая 3 из обозначения L3 для корневой таблицы страниц --- третий уровень считая с нуля.
- Константа [`ku::memory::mmu::PAGE_TABLE_LEAF_LEVEL`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_LEAF_LEVEL.html) --- та самая 0 из обозначения L0 для листьевой таблицы страниц.
- Конструкция `unsafe { virt_addr.try_into_mut::<PageTable>()? }`, которая превращает [`Virt`](../../doc/ku/memory/addr/type.Virt.html) в `&mut PageTable`.
- Метод [`core::mem::MaybeUninit::zeroed()`](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html#method.zeroed) для зануления выделенной памяти под выделяемую таблицу страниц. Нулевое значение подойдёт для [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html), --- с точки зрения компилятора сгодится любое значение оборачиваемого ею типа [`usize`](https://doc.rust-lang.org/nightly/core/primitive.usize.html), а с точки зрения процессора, подойдёт любое значение, если [`PageTableFlags::PRESENT`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.PRESENT) будет сброшен.
- Цикл, чтобы обойти уровни таблицы. Для уменьшения потенциальных возможностей совершить ошибку, можно было бы сделать таблицы страниц разных уровней не совместимыми в системе типов, [как здесь](https://os.phil-opp.com/page-tables/#some-clever-solution). Тогда компилятор не дал бы их перепутать в коде. Но такой подход, к сожалению, не даст написать обычный цикл по уровням, только рекурсию на обобщённых функциях или копипасту. Поэтому в Nikka выбор сделан в пользу одного и того же типа для узлов всех уровней.
- Итерирование в обратном порядке: `for i in (5..10).rev()`, или же `for i in (5..=10).rev()` если `10` должно быть включительно.
- Методы [`FrameGuard::load()`](../../doc/kernel/memory/struct.FrameGuard.html#method.load) и [`FrameGuard::store()`](../../doc/kernel/memory/struct.FrameGuard.html#method.store).


### Проверьте себя

Теперь тест `2-mm-4-translate` должен проходить полностью:

```console
$ (cd kernel; cargo test --test 2-mm-4-translate)
...
running 8 tests

2_mm_4_translate::t00_path----------------------------------
17:30:01 0 D present:
17:30:01 0 D     virt = 0v2F_7498
17:30:01 0 D     path = L3: 0vFFFF_8000_0000_1000 / 1 @ 0p1000 -> L2: 0vFFFF_8000_0000_2000 / 2 @ 0p2000 -> L1: 0vFFFF_8000_0000_3008 / 3 @ 0p3000 -> L0: 0vFFFF_8000_0086_37B8 / 2147 @ 0p86_3000 => 0p4F_4498
17:30:01 0 D     level = 0
17:30:01 0 D     pte = 1268 @ 0p4F_4000 -----DA---W-P(0x8000000000000063)
17:30:01 0 D allocated unique virtual address; virt = 0v1080_0000_0000; is_user = true
17:30:01 0 D non-present:
17:30:01 0 D     virt = 0v1080_0000_0000
17:30:01 0 D     path = L3: 0vFFFF_8000_0000_1108 / 1 @ 0p1000 (non-present)
17:30:01 0 D     level = 3
17:30:01 0 D     pte = <non-present>
17:30:01 0 D huge:
17:30:01 0 D     virt = 0vFFFF_8000_0070_0000
17:30:01 0 D     path = L3: 0vFFFF_8000_0000_1800 / 1 @ 0p1000 -> L2: 0vFFFF_8000_03FF_F000 / 16383 @ 0p3FF_F000 (1 GiB huge page) => 0p70_0000
17:30:01 0 D     level = 2
17:30:01 0 D     pte = 0 @ 0p0 ----HDA---W-P(0x80000000000000E3)
2_mm_4_translate::t00_path------------------------- [passed]

2_mm_4_translate::t01_translate-----------------------------
17:30:01 0 D pte = 1268 @ 0p4F_4000 -----DA---W-P(0x8000000000000063)
17:30:01 0 D read_ptr = 0xffff8000004f4190; write_ptr = 314159266
17:30:01 0 D write_value = 0; read_value = 0; variable = 0
17:30:01 0 D write_value = 1; read_value = 1; variable = 1
17:30:01 0 D write_value = 2; read_value = 2; variable = 2
17:30:01 0 D write_value = 3; read_value = 3; variable = 3
17:30:01 0 D write_value = 4; read_value = 4; variable = 4
2_mm_4_translate::t01_translate-------------------- [passed]

2_mm_4_translate::t02_map_intermediate----------------------
17:30:01 0 D allocated unique virtual address; virt = 0v1100_0000_0000; is_user = true
17:30:01 0 D pte = <non-present>
17:30:01 0 D allocated unique virtual address; virt = 0v1180_0000_0000; is_user = true
2_mm_4_translate::t02_map_intermediate------------- [passed]

2_mm_4_translate::t03_no_excessive_intermediate_flags-------
17:30:01 0 D allocated unique virtual address; virt = 0v1200_0000_0000; is_user = true
2_mm_4_translate::t03_no_excessive_intermediate_flags [passed]

2_mm_4_translate::t04_huge_page-----------------------------
2_mm_4_translate::t04_huge_page-------------------- [passed]

2_mm_4_translate::t05_no_page-------------------------------
17:30:01 0 D allocated unique virtual address; virt = 0v1280_0000_0000; is_user = true
2_mm_4_translate::t05_no_page---------------------- [passed]

2_mm_4_translate::t06_build_map-----------------------------
17:30:01 0 D virtual to physical mapping
17:30:01 0 D block = [0v1000, 0v1_6000), size 84 KiB -> [0p1000, 0p1_6000), size 84 KiB, ----------WXP(0x3)
17:30:01 0 D block = [0vA_0000, 0vC_0000), size 128 KiB -> [0pA_0000, 0pC_0000), size 128 KiB, ----------WXP(0x3)
17:30:01 0 D block = [0v20_0000, 0v21_5000), size 84 KiB -> [0p40_0000, 0p41_5000), size 84 KiB, ------------P(0x8000000000000001)
17:30:02.067 0 D block = [0v21_5000, 0v2E_0000), size 812 KiB -> [0p41_4000, 0p4D_F000), size 812 KiB, -----------XP(0x1)
17:30:02.105 0 D block = [0v2E_0000, 0v2F_5000), size 84 KiB -> [0p4D_E000, 0p4F_3000), size 84 KiB, ----------W-P(0x8000000000000003)
17:30:02.125 0 D block = [0v2F_5000, 0v2F_6000), size 4 KiB -> [0p1_7000, 0p1_8000), size 4 KiB, ----------W-P(0x8000000000000003)
17:30:02.733 0 D block = [0v2F_7000, 0v5B_A000), size 2.762 MiB -> [0p4F_4000, 0p7B_7000), size 2.762 MiB, ----------W-P(0x8000000000000003)
17:30:03.341 0 D block = [0v5B_A000, 0v5C_0000), size 24 KiB -> [0p1_8000, 0p1_E000), size 24 KiB, ----------W-P(0x8000000000000003)
17:30:03.363 0 D block = [0v100_0000_0000, 0v100_0000_1000), size 4 KiB -> [0p1_6000, 0p1_7000), size 4 KiB, ----------WXP(0x3)
17:30:03.491 0 D block = [0v100_0000_2000, 0v100_0008_3000), size 516 KiB -> [0p1_E000, 0p9_F000), size 516 KiB, ----------WXP(0x3)
17:30:08.221 0 D block = [0v100_0008_3000, 0v100_0020_2000), size 1.496 MiB -> [0p10_0000, 0p27_F000), size 1.496 MiB, ----------WXP(0x3)
17:30:08.619 0 D block = [0vFFFF_8000_0000_0000, 0vFFFF_8000_4000_0000), size 1 GiB -> [0p0, 0p4000_0000), size 1 GiB, ----H-----W-P(0x8000000000000083)
17:30:08.639 0 D virtual address space
17:30:08.747 0 D block = [0v1000, 0v1_6000), size 84 KiB, ----------WXP(0x3)
17:30:08.973 0 D block = [0vA_0000, 0vC_0000), size 128 KiB, ----------WXP(0x3)
17:30:09.001 0 D block = [0v20_0000, 0v21_5000), size 84 KiB, ------------P(0x8000000000000001)
17:30:09.185 0 D block = [0v21_5000, 0v2E_0000), size 812 KiB, -----------XP(0x1)
17:30:09.221 0 D block = [0v2E_0000, 0v2F_6000), size 88 KiB, ----------W-P(0x8000000000000003)
17:30:10.417 0 D block = [0v2F_7000, 0v5C_0000), size 2.785 MiB, ----------W-P(0x8000000000000003)
17:30:10.435 0 D block = [0v100_0000_0000, 0v100_0000_1000), size 4 KiB, ----------WXP(0x3)
17:30:15.281 0 D block = [0v100_0000_2000, 0v100_0020_2000), size 2 MiB, ----------WXP(0x3)
17:30:15.681 0 D block = [0vFFFF_8000_0000_0000, 0vFFFF_8000_4000_0000), size 1 GiB, ----H-----W-P(0x8000000000000083)
2_mm_4_translate::t06_build_map-------------------- [passed]

2_mm_4_translate::t99_no_frame------------------------------
17:30:15.779 0 D real free frame count can be less than free frame count reported by boot frame allocator since it records reference and deallocation counts but does not do that actually; frame_count = 14606; real_frame_count = 14606
17:30:15.797 0 D allocated unique virtual address; virt = 0v1300_0000_0000; is_user = true
2_mm_4_translate::t99_no_frame--------------------- [passed]
17:30:15.817 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/mapping.rs | 32 ++++++++++++++++++++++++++++----
kernel/src/memory/path.rs    | 68 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
2 files changed, 94 insertions(+), 6 deletions(-)
```
