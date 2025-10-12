## Создание полной копии виртуального отображения

### Задача 6 --- создание полной копии виртуального отображения

В будущем, при создании нового процесса, нам понадобится скопировать существующий
[`Mapping`](../../doc/kernel/memory/mapping/struct.Mapping.html)
в новый.
Это делает метод
[`Mapping::duplicate()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.duplicate):
```rust
fn duplicate(&self) -> Result<Self> {
    let mut result = Self::new(Frame::zero(), self.phys2virt);
    result.page_table_root = Self::duplicate_subtree(&mut result, self, self.page_table_root, PAGE_TABLE_ROOT_LEVEL)?;
    Ok(result)
}
```
Основную работу он перекладывает на рекурсивный метод
[`Mapping::duplicate_subtree()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.duplicate_subtree),
запуская корневой вызов рекурсии.

Реализуйте [метод](../../doc/kernel/memory/mapping/struct.Mapping.html#method.duplicate_subtree)
```rust
fn Mapping::duplicate_subtree(
    &self,
    dst: &mut Mapping,
    src_frame: Frame,
    level: u32,
) -> Result<Frame>
```

в файле [`kernel/src/memory/mapping.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/mapping.rs)
сейчас, пока помните как устроено страничное отображение.
Он принимает на вход:

- Новое отображение `dst`, которое мы создаём как копию исходного.
- Исходное отображение `self`.
- Исходный узел `PageTable` таблицы страниц в виде содержащего его фрейма `src_frame`.
- Уровень этого узла `level` от [`PAGE_TABLE_ROOT_LEVEL = 3`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_ROOT_LEVEL.html)
до [`PAGE_TABLE_LEAF_LEVEL = 0`](../../doc/ku/memory/mmu/constant.PAGE_TABLE_LEAF_LEVEL.html).

И должен создать копию заданного `src_frame` узла.
Возвращает он либо эту копию, либо возникшую в процессе работы ошибку.
Этот метод копирует только страничное отображение, то есть все `PageTable` уровней L3--L0,
но не фреймы, на которые указывает это отображение.
То есть, должно получиться отображение,
которое переводит те же виртуальные адреса в те же физические адреса.
Разделяя таким образом отображённую часть физической памяти,
но дублируя физическую память под само страничное отображение (`PageTable` уровней L3--L0).
Если бы `PageTable` разделяли ту же физическую память, то уже после того как копирование отработало,
модификации исходного `self` приводили бы к модификации `dst` и наоборот.
А это не то что нам нужно.
Это означает, что

- `PageTable` уровней L3--L1 включительно нужно пересоздать, заполняя их `PageTableEntry` результатами рекурсивных вызовов `Mapping::duplicate_subtree()`.
- А вот `PageTable` уровня L0 нужно просто скопировать как есть, их записи `PageTableEntry` должны вести в те же физические фреймы, что и в исходном отображении `self`. Но только если соответствующая запись ведёт на страницу, принадлежащую ядру. Ссылки уровня L0 на страницы, принадлежащие пользователю --- `PageTableFlags::USER`, --- копировать не нужно.
- На этот раз `PageTableFlags::HUGE` нужно корректно обработать. Это означает, что рекурсивно спускаться в записи `PageTableEntry` для которых этот флаг включён не нужно. С этими записями нужно поступить как с записями на уровне L0, --- скопировать в точности из `self`.
- Не используемые записи в новых [`PageTable`](../../doc/ku/memory/mmu/type.PageTable.html) нужно почистить, как и при реализации вспомогательного метода [`Mapping::allocate_node()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.allocate_node) для [`Path::map()`](../../doc/kernel/memory/path/struct.Path.html#method.map).

Для физических фреймов, на которые указывают скопированные записи листьевого уровня L0, нужно вызвать
[`FrameAllocator::reference()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.reference).
Этот метод увеличивает число ссылок на физический фрейм.
И позволяет понять, что физический фрейм нельзя считать свободным пока оба отображения `self` и `dst` существуют.
А ведь именно указанные фреймы мы разделяем, используя сразу в обоих отображениях.
Для выделения новых физических фреймов используйте
[`FrameAllocator::allocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.allocate).
В обоих случаях обращайтесь к глобальному
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html).

Также вам могут пригодиться методы

- [`unsafe fn Mapping::page_table_ref(&self, frame: Frame) -> &PageTable`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.page_table_ref) и
- [`unsafe fn Mapping::page_table_mut(&mut self, frame: Frame) -> &mut PageTable`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.page_table_mut).

Они позволяют интерпретировать заданный физический фрейм `frame` как узел `PageTable` таблицы страниц.
И различаются только возможностью менять эту таблицу.

Учтите, что в рекурсивный вызов передаётся изменяемая ссылка `dst: &mut Mapping`.
А значит, в момент рекурсивного вызова не должно быть живых локальных переменных ссылающихся на `dst` прямо или косвенно.
Rust не даст скомпилировать такой код из-за нарушения [владения](https://doc.rust-lang.ru/book/ch04-00-understanding-ownership.html) `dst`, ---
только одна из функций, вызывающая или вызываемая, может эксклюзивно заимствовать `dst` в каждый момент времени.
А [изменяемая ссылка и означает эксклюзивность заимствования](https://doc.rust-lang.ru/book/ch04-02-references-and-borrowing.html#%D0%98%D0%B7%D0%BC%D0%B5%D0%BD%D1%8F%D0%B5%D0%BC%D1%8B%D0%B5-%D1%81%D1%81%D1%8B%D0%BB%D0%BE%D1%87%D0%BD%D1%8B%D0%B5-%D0%BF%D0%B5%D1%80%D0%B5%D0%BC%D0%B5%D0%BD%D0%BD%D1%8B%D0%B5).


### Проверьте себя

Теперь должен проходить тест `2-mm-6-duplicate-subtree`:

```console
$ (cd kernel; cargo test --test 2-mm-6-duplicate-subtree)
...
running 3 tests

2_mm_6_duplicate_subtree::duplicate_and_drop----------------
18:40:23 0 D allocated unique virtual address; virt = 0v1080_0000_0000; is_user = true
18:40:23 0 D allocated unique virtual address; virt = 0v1100_0000_0000; is_user = true
18:40:23 0 I duplicate; address_space = "process" @ 0p3FF_E000
18:40:23 0 I drop; address_space = "process" @ 0p3FF_E000
18:40:23 0 I duplicate; address_space = "process" @ 0p3FE_A000
18:40:23 0 I drop; address_space = "process" @ 0p3FE_A000
2_mm_6_duplicate_subtree::duplicate_and_drop------- [passed]

2_mm_6_duplicate_subtree::duplicate_path--------------------
18:40:23 0 I duplicate; address_space = "process" @ 0p3FD_9000
18:40:23 0 I switch to; address_space = "process" @ 0p3FD_9000
18:40:23 0 D present:
18:40:23 0 D     virt = 0v2E_F3A8
18:40:23 0 D     path = L3: 0vFFFF_8000_03FD_9000 / 16345 @ 0p3FD_9000 -> L2: 0vFFFF_8000_03FD_5000 / 16341 @ 0p3FD_5000 -> L1: 0vFFFF_8000_03FD_4008 / 16340 @ 0p3FD_4000 -> L0: 0vFFFF_8000_03FD_2778 / 16338 @ 0p3FD_2000 => 0p4E_C3A8
18:40:23 0 D     level = 0
18:40:23 0 D     pte = 1260 @ 0p4E_C000 -----DA---W-P(0x8000000000000063)
18:40:23 0 D allocated unique virtual address; virt = 0v1180_0000_0000; is_user = true
18:40:23 0 D non-present:
18:40:23 0 D     virt = 0v1180_0000_0000
18:40:23 0 D     path = L3: 0vFFFF_8000_03FD_9118 / 16345 @ 0p3FD_9000 (non-present)
18:40:24.007 0 D     level = 3
18:40:24.015 0 D     pte = <non-present>
18:40:24.023 0 D huge:
18:40:24.029 0 D     virt = 0vFFFF_8000_0070_0000
18:40:24.037 0 D     path = L3: 0vFFFF_8000_03FD_9800 / 16345 @ 0p3FD_9000 -> L2: 0vFFFF_8000_03FC_6000 / 16326 @ 0p3FC_6000 (1 GiB huge page) => 0p70_0000
18:40:24.055 0 D     level = 2
18:40:24.061 0 D     pte = 0 @ 0p0 ----HDA---W-P(0x80000000000000E3)
18:40:24.071 0 I switch to; address_space = "base" @ 0p1000
18:40:24.077 0 I drop the current address space; address_space = "process" @ 0p3FD_9000; switch_to = "base" @ 0p1000
2_mm_6_duplicate_subtree::duplicate_path----------- [passed]

2_mm_6_duplicate_subtree::duplicate_translate---------------
18:40:24.355 0 I duplicate; address_space = "process" @ 0p3FC_5000
18:40:24.363 0 I switch to; address_space = "process" @ 0p3FC_5000
18:40:24.371 0 D pte = 1260 @ 0p4E_C000 -----DA---W-P(0x8000000000000063)
18:40:24.379 0 D read_ptr = 0xffff8000004ec0d0; write_ptr = 314159266
18:40:24.387 0 D write_value = 0; read_value = 0; variable = 0
18:40:24.395 0 D write_value = 1; read_value = 1; variable = 1
18:40:24.403 0 D write_value = 2; read_value = 2; variable = 2
18:40:24.409 0 D write_value = 3; read_value = 3; variable = 3
18:40:24.417 0 D write_value = 4; read_value = 4; variable = 4
18:40:24.425 0 I switch to; address_space = "base" @ 0p1000
18:40:24.431 0 I drop the current address space; address_space = "process" @ 0p3FC_5000; switch_to = "base" @ 0p1000
2_mm_6_duplicate_subtree::duplicate_translate------ [passed]
18:40:24.453 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/mapping.rs | 32 ++++++++++++++++++++++++++++++--
1 file changed, 30 insertions(+), 2 deletions(-)
```
