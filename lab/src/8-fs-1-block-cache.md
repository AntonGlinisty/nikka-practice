## [Блочный кэш](https://en.wikipedia.org/wiki/Page_cache)

В Nikka диск отображается в память, чтобы удобнее было с ним работать.
Для отслеживания с какого диска, куда именно и сколько памяти отображено, служит структура
[`kernel::fs::block_cache::BlockCache`](../../doc/kernel/fs/block_cache/struct.BlockCache.html):

```rust
struct BlockCache {
    cache: Block<Page>,
    disk: Disk,
}
```

Она является [синглтоном](https://en.wikipedia.org/wiki/Singleton_pattern)
[`static ref BLOCK_CACHE: Mutex<Option<BlockCache>>`](../../doc/kernel/fs/block_cache/struct.BLOCK_CACHE.html).

В память блоки зачитываются по необходимости.
Для этого, изначально все страницы помечаются недоступными.
А при возникновении ошибки обращения к памяти, страницы нужного блока считывается с диска обработчиком
`BlockCache::trap_handler()`.
Запись блока выполняется при необходимости вызовом `BlockCache::flush_block()`.
Все блоки можно записать вызовом `BlockCache::flush()`.
Для простоты, размер блока `BLOCK_SIZE` берётся равным размеру страницы `Page::SIZE`.


### Блочный кэш в общем случае

Обычно размер кэша в памяти гораздо меньше, чем размер кэшируемого диска:

![](7-fs-1-block-cache.svg)


### Блочный кэш в Nikka

В Nikka сделано значительное упрощение.
Размер кэша в памяти равен размеру кэшируемого диска.
Поэтому размер диска ограничен размером доступной в машине памяти.

![](7-fs-1-block-cache-nikka.svg)


### Задача 1 --- блочный кэш


#### Инициализация

Реализуйте метод инициализации блочного кэша

```rust
fn BlockCache::init(disk: Disk, block_count: usize) -> Result<()>
```

в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

Он должен зарезервировать в `BASE_ADDRESS_SPACE` блок виртуальных страниц, достаточный для хранения
`block_count` блоков файловой системы.
И записать соответствующее значение в
[`BLOCK_CACHE`](../../doc/kernel/fs/block_cache/struct.BLOCK_CACHE.html).


#### Считывание блока с диска

Реализуйте метод

```rust
fn BlockCache::trap_handler(info: &Info) -> Result<bool>
```

в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

Он должен обработать Page Fault, если адрес, который его вызвал, относится к блочному кэшу.
Если это так и Page Fault успешно обработан, верните `true`.
Если адрес, вызвавший Page Fault, не относится к блочному кэшу, верните `false`.
Для чтения с диска используйте метод
[`kernel::fs::disk::Disk::pio_read()`](../../doc/kernel/fs/disk/struct.Disk.html#method.pio_read).


#### Запись блока на диск

Запись блока выполняется при необходимости вызовом `BlockCache::flush_block()`.


Реализуйте метод

```rust
fn BlockCache::flush_block(block_number: usize) -> Result<()>
```

в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

Он записывает заданный блок на диск методом
[`kernel::fs::disk::Disk::pio_write()`](../../doc/kernel/fs/disk/struct.Disk.html#method.pio_write).
Запись нужно делать только если:

- Блок отображён в память. Это означает что нему были обращения.
- И помечен как `PageTableFlags::DIRTY`. То есть, в память были записи, а значит блок на диске потенциально содержит устаревшие данные. Если обращения к блоку были только на чтение, то данные в памяти такие же как на диске, и можно их не записывать. А процессор в этом случае не установит бит `PageTableFlags::DIRTY`.
- После записи блока, сбросьте бит `PageTableFlags::DIRTY`. Он фактически означает одинаковость данных на диске и в памяти блочного кэша. Которая только что восстановлена. При этом нужно сбросить и соответствующую запись в [TLB](https://en.wikipedia.org/wiki/Translation_lookaside_buffer) с помощью функции [`kernel::memory::mmu::flush()`](../../doc/kernel/memory/mmu/fn.flush.html). Иначе процессор не узнает, что вы сбросили `PageTableFlags::DIRTY` и не проставит его в таблице страниц при следующей записи. И обновлённый блок на диск записан не будет.


### Проверьте себя

Теперь должны заработать тесты в файлaх
[`kernel/tests/7-fs-1-block-cache-1-write.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-1-block-cache-1-write.rs) и
[`kernel/tests/7-fs-1-block-cache-2-read.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-1-block-cache-2-read.rs):

```console
$ (cd kernel; cargo test --test 7-fs-1-block-cache-1-write --test 7-fs-1-block-cache-2-read)
...
7_fs_1_block_cache_1_write::write---------------------------
20:37:19 0 D block_count = 8192
7_fs_1_block_cache_1_write::write------------------ [passed]
20:37:19 0 I exit qemu; exit_code = SUCCESS
...
7_fs_1_block_cache_2_read::read_what_was_written------------
20:37:21 0 D block_count = 8192
7_fs_1_block_cache_2_read::read_what_was_written--- [passed]
20:37:21 0 I exit qemu; exit_code = SUCCESS
```


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/fs/block_cache.rs |   64 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 1 file changed, 56 insertions(+), 8 deletions(-)
```
