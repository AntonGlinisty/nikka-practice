## Битовая карта занятых блоков


Для отслеживания какие именно блоки файловой системы заняты,
а какие свободны, служит структура
[`kernel::fs::bitmap::Bitmap`](../../doc/kernel/fs/block_bitmap/struct.Bitmap.html):

```rust
struct Bitmap {
    bitmap: &'static mut [u64],
    block_count: usize,
    cursor: usize,
}
```

Каждый элемент среза `Bitmap::bitmap` отвечает за 64 блока.
Блоку с меньшим номером соответствует бит с меньшим весом в соответствующем элементе.
Блок свободен тогда и только тогда, когда соответствующий ему бит равен `0`.


### Задача 2 --- битовая карта занятых блоков

Реализуйте метод

```rust
fn Bitmap::allocate(&mut self) -> Result<usize>
```

в файле
[`kernel/src/fs/block_bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_bitmap.rs).

Функция должна найти по `Bitmap::bitmap` свободный блок и аллоцировать его.

- Верните номер выделенного блока.
- После того как новый блок аллоцирован, сбросьте изменившийся блок самого `bitmap` на диск с помощью реализованной вами функции `BlockCache::flush_block()`. Когда меняете метаданные, лучше сразу сбрасывать их на диск, так меньше вероятность поломки файловой системы, например, при сбое питания.
- Искать каждый раз с самого начала `Bitmap::bitmap` не эффективно. Лучше, например, обходить его по циклу с того места, на котором остановились в прошлый раз. Для хранения этой позиции между вызовами `Bitmap::allocate()` служит поле `Bitmap::cursor`.
- Проверять каждый бит тоже не эффективно. Лучше сначала найти элемент `Bitmap::bitmap`, в котором есть хотя бы один свободный бит. Поэтому биты сгруппированы именно по `u64`, а не например по `u8`.
- Верните ошибку [`Error::NoDisk`](../../doc/kernel/error/enum.Error.html#variant.NoDisk), если свободных блоков не осталось.


### Проверьте себя

Теперь должен заработать тест `allocation()` в файле
[`kernel/tests/7-fs-2-bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-2-bitmap.rs):

```console
$ (cd kernel; cargo test --test 7-fs-2-bitmap)
...
7_fs_2_bitmap::allocation-----------------------------
20:37:24 0 D block_count = 8192
7_fs_2_bitmap::allocation-------------------- [passed]
20:37:30.657 0 I exit qemu; exit_code = SUCCESS
```


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/fs/block_bitmap.rs |   26 ++++++++++++++++++++++++--
 1 file changed, 24 insertions(+), 2 deletions(-)
```
