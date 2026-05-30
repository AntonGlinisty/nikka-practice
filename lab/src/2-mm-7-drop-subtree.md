## Удаление виртуального отображения

### Задача 6 --- удаление виртуального отображения

Раз есть способ создать новое отображение, которое расходует физические фреймы, значит должен быть и способ удалить его, вернув эти фреймы в систему.
Это делает реализация типажа [`core::ops::Drop`](https://doc.rust-lang.org/nightly/core/ops/trait.Drop.html):
```rust
impl Drop for Mapping {
    fn drop(&mut self) {
        assert_ne!(Self::current_page_table_root(), self.page_table_root);

        if self.is_valid() {
            self.drop_subtree(self.page_table_root, PAGE_TABLE_ROOT_LEVEL);
        }
    }
}
```
В строчке `assert_ne!(Self::current_page_table_root(), self.page_table_root)` проверяется, что мы не пытаемся удалить отображение, которое в данный момент загружено в регистр
[`CR3`](https://wiki.osdev.org/CPU_Registers_x86#CR3)
и является активным виртуальным пространством.
Далее вся работа перекладывается на рекурсивный метод
[`Mapping::drop_subtree()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.drop_subtree).

Реализуйте метод
[`Mapping::drop_subtree()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.drop_subtree)
```rust
fn Mapping::drop_subtree(
    &mut self,
    node: Frame,
    level: u32,
    drop_used: bool,
) -> bool
```
в файле [`kernel/src/memory/mapping.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/mapping.rs).

Он удаляет узлы дерева отображения.

- Если `drop_used = true`, удаляются все узлы. Это как раз нужно в [`Mapping::drop()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.drop).
- Если `drop_used = false`, удаляются только узлы, через которые не отображена ни одна страница. То есть, узлы в которых все элементы равны [`PageTableEntry::default()`](../../doc/ku/memory/mmu/struct.PageTableEntry.html#method.default), либо становятся таковыми после удаления ненужных узлов следующих уровней. Это нужно для очистки не использующейся памяти методом [`Mapping::unmap_unused_intermediate()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.unmap_unused_intermediate) и корректной работы функции `forbid_frame_leaks()`, которая следит за отсутствием утечек физических фреймов в тестах. В этом режиме используется возвращаемое значение, которое говорит о том, был ли удалён текущий узел.

Для освобождения фреймов используйте
[`FrameAllocator::deallocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.deallocate),
он уменьшит количество ссылок на фрейм и освободит его, когда ссылок не останется.
Перед освобождением фрейма, занятого под одну из `PageTable`, то есть из записей `PageTableEntry` уровней L3--L1,
нужно спуститься в этот фрейм, рекурсивно вызвав `Mapping::drop_subtree()`.
Пользовательские фреймы нужно освобождать так же как и фреймы, занятые под `PageTable`.
А вот с записями у которых включён флаг `PageTableFlags::HUGE` делать ничего не нужно --- ни освобождать, ни спускаться в них рекурсивно.


### Проверьте себя

Запустите тест `2-mm-7-drop-subtree`:

```console
$ (cd kernel; cargo test --test 2-mm-7-drop-subtree)
...
running 1 tests

2_mm_7_drop_subtree::drop_subtree---------------------------
2_mm_7_drop_subtree::drop_subtree------------------ [passed]
18:46:22 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/mapping.rs | 46 ++++++++++++++++++++++++++++++++++++++++++++--
1 file changed, 44 insertions(+), 2 deletions(-)
```
