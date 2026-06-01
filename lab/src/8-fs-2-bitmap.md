## Битовые карты занятых блоков и inode


Для отслеживания какие именно элементы --- блоки или inode --- файловой системы заняты,
а какие свободны, служит структура
[`kernel::fs::bitmap::Bitmap`](../../doc/kernel/fs/bitmap/struct.Bitmap.html).

{{#include ../../doc/kernel/fs/bitmap/struct.Bitmap.html.declaration}}

Каждое значение в срезе
[`Bitmap::bitmap.bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html#structfield.bitmap)
отвечает за 64 элемента.
Элементу с меньшим номером соответствует бит с меньшим весом в соответствующем элементе.
Элемент свободен тогда и только тогда, когда соответствующий ему бит равен `0`.


### Задача 2 --- битовая карта занятых блоков или inode

Доделайте метод
[`Bitmap::format()`](../../doc/kernel/fs/bitmap/struct.Bitmap.html#method.format)
в файле
[`kernel/src/fs/bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/bitmap.rs).

{{#include ../../doc/kernel/fs/bitmap/struct.Bitmap.html.method.format}}

У вас уже должна быть решена [задача про битовую карту](../../lab/book/3-allocator-2-bitmap.html#%D0%97%D0%B0%D0%B4%D0%B0%D1%87%D0%B0-2--%D0%B2%D1%8B%D0%B4%D0%B5%D0%BB%D0%B5%D0%BD%D0%B8%D0%B5-%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B4%D0%BD%D0%BE%D0%B3%D0%BE-%D1%8D%D0%BB%D0%B5%D0%BC%D0%B5%D0%BD%D1%82%D0%B0-%D1%81-%D0%BF%D0%BE%D0%BC%D0%BE%D1%89%D1%8C%D1%8E-%D0%B1%D0%B8%D1%82%D0%BE%D0%B2%D0%BE%D0%B9-%D0%BA%D0%B0%D1%80%D1%82%D1%8B). Если это не так, сделайте её.


### Проверьте себя

Теперь должен заработать тест `allocation()` в файле
[`kernel/tests/7-fs-2-bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-2-bitmap.rs):

```console
$ (cd kernel; cargo test --test 7-fs-2-bitmap)
...
7_fs_2_bitmap::allocation-----------------------------------
17:12:11 0 D block_count = 8192
17:12:16.879 0 D allocated_head = [36, 37, 38, 39, 40, 41, 42, 43, 44, 45]; allocated_tail = [8182, 8183, 8184, 8185, 8186, 8187, 8188, 8189, 8190, 8191]
17:12:23.939 0 D reallocated_head = [8128, 8130, 8132, 8134, 8136, 8138, 8140, 8142, 8144, 8146]; reallocated_tail = [8108, 8110, 8112, 8114, 8116, 8118, 8120, 8122, 8124, 8126]
17:12:24.051 0 D block_cache_stats = Stats { discards: 0, evictions: 0, reads: 3, writes: 20464 }
7_fs_2_bitmap::allocation-------------------------- [passed]
17:12:24.065 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/fs/bitmap.rs | 22 ++++++++++++++++++++--
 1 file changed, 20 insertions(+), 2 deletions(-)
```
