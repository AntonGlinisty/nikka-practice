## [Блочный кэш](https://en.wikipedia.org/wiki/Page_cache)

В Nikka диск отображается в память, чтобы с ним было удобнее работать.
Для отслеживания с какого диска, сколько блоков и куда именно в памяти отображено, служит структура
[блочного кэша](https://en.wikipedia.org/wiki/Page_cache)
[`kernel::fs::block_cache::BlockCache`](../../doc/kernel/fs/block_cache/struct.BlockCache.html).

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.declaration}}

Она является [синглтоном](https://en.wikipedia.org/wiki/Singleton_pattern)
[`static ref BLOCK_CACHE: FastSpinlock<Option<BlockCache>>`](../../doc/kernel/fs/block_cache/struct.BLOCK_CACHE.html).

В память блоки зачитываются по необходимости.
Для этого, изначально все страницы помечаются недоступными.
А при возникновении ошибки обращения к памяти, страницы нужного блока считывается с диска обработчиком
[`BlockCache::trap_handler()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.trap_handler).
Запись блока выполняется при необходимости вызовом
[`BlockCache::flush_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.flush_block).
Все блоки можно записать вызовом
[`BlockCache::flush()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.flush).
Для простоты, размер блока
[`kernel::fs::BLOCK_SIZE`](../../doc/kernel/fs/constant.BLOCK_SIZE.html)
берётся равным размеру страницы
[`Page::SIZE`](../../doc/ku/memory/frage/struct.Frage.html#associatedconstant.SIZE).


### Блочный кэш в общем случае

Обычно размер кэша в памяти гораздо меньше, чем размер кэшируемого диска:

![](7-fs-1-block-cache.svg)


### Блочный кэш в Nikka

В Nikka сделано значительное упрощение.
Под кэш в адресном пространстве зарезервирован блок по размеру равный кэшируемому диску.
Поэтому размер диска ограничен размером адресного пространства.
И, если не реализовать вытеснение блоков из кэша, то он также будет ограничен
размером доступной в машине физической памяти.

![](7-fs-1-block-cache-nikka.svg)


### Задача 1 --- блочный кэш


#### Инициализация

Реализуйте метод
[`BlockCache::init()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.init)
инициализации блочного кэша
в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.method.init}}


#### Статистики блочного кэша

Для интроспекции блочный кэш поддерживает статистики
[`kernel::fs::block_cache::Stats`](../../doc/kernel/fs/block_cache/struct.Stats.html).

{{#include ../../doc/kernel/fs/block_cache/struct.Stats.html.declaration}}

С помощью этих статистик в тестах выполняются проверки,
что файловая система при общении с диском не читает и не пишет лишние данные.
Чтобы тесты проходили, не забывайте обновлять статистики блочного кэша в соответствующих случаях.


#### Обращение к блоку в блочном кэше

Реализуйте метод
[`Cache::block()`](../../doc/kernel/fs/block_cache/struct.Cache.html#method.block)
в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

{{#include ../../doc/kernel/fs/block_cache/struct.Cache.html.method.block}}


#### Считывание блока с диска

Реализуйте метод
[`BlockCache::trap_handler()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.trap_handler)
в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.method.trap_handler}}

Для чтения с диска используйте метод
[`kernel::fs::disk::Disk::pio_read()`](../../doc/kernel/fs/disk/struct.Disk.html#method.pio_read).


#### Пропуск чтения блока с диска

Чтобы не читать с диска блок, который планируется перезаписать целиком, реализуйте методы
[`BlockCache::emplace_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.emplace_block)
и
[`BlockCache::fill_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.fill_block).

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.method.emplace_block}}

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.method.fill_block}}


#### Запись блока на диск

Запись блока выполняется при необходимости вызовом
[`BlockCache::flush_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.flush_block).

Реализуйте метод
[`BlockCache::flush_block_impl()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.flush_block_impl)
в файле
[`kernel/src/fs/block_cache.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/fs/block_cache.rs).

{{#include ../../doc/kernel/fs/block_cache/struct.BlockCache.html.method.flush_block_impl}}

Он записывает заданный блок на диск методом
[`kernel::fs::disk::Disk::pio_write()`](../../doc/kernel/fs/disk/struct.Disk.html#method.pio_write).


### Проверьте себя

Теперь должны заработать тесты в файлaх
[`kernel/tests/7-fs-1-block-cache-1-write.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-1-block-cache-1-write.rs) и
[`kernel/tests/7-fs-1-block-cache-2-read.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/7-fs-1-block-cache-2-read.rs):

```console
$ (cd kernel; cargo test --test 7-fs-1-block-cache-1-write --test 7-fs-1-block-cache-2-read)
...
7_fs_1_block_cache_1_write::write---------------------------
17:12:05 0 D block_count = 8192
17:12:05 0 D flush; block = 0
17:12:05 0 D flush; block = 2
17:12:05 0 D flush; block = 4
17:12:05 0 D flush; block = 6
17:12:05 0 D flush; block = 8
17:12:05 0 D flush; block = 10
17:12:05 0 D flush; block = 12
17:12:05 0 D flush; block = 14
17:12:05 0 D flush; block = 16
17:12:05 0 D flush; block = 18
17:12:05 0 D flush; block = 20
17:12:05 0 D flush; block = 22
17:12:05 0 D flush; block = 24
17:12:05 0 D flush; block = 26
17:12:05 0 D flush; block = 28
17:12:05 0 D flush; block = 30
17:12:05 0 D block_cache_stats = Stats { discards: 0, evictions: 0, reads: 32, writes: 16 }
7_fs_1_block_cache_1_write::write------------------ [passed]
17:12:05 0 I exit qemu; exit_code = ExitCode(SUCCESS)
...
7_fs_1_block_cache_2_read::read_what_was_written------------
17:12:09 0 D block_count = 8192
17:12:09 0 D i = 0; block = 0; actual = 0; expected = 0
17:12:09 0 D i = 777; block = 1; actual = 0; expected = 0
17:12:09 0 D i = 1554; block = 3; actual = 0; expected = 0
17:12:09 0 D i = 2331; block = 4; actual = 2331; expected = 2331
17:12:09 0 D i = 3108; block = 6; actual = 3108; expected = 3108
17:12:09 0 D i = 3885; block = 7; actual = 0; expected = 0
17:12:09 0 D i = 4662; block = 9; actual = 0; expected = 0
17:12:09 0 D i = 5439; block = 10; actual = 5439; expected = 5439
17:12:09 0 D i = 6216; block = 12; actual = 6216; expected = 6216
17:12:09 0 D i = 6993; block = 13; actual = 0; expected = 0
17:12:09 0 D i = 7770; block = 15; actual = 0; expected = 0
17:12:09 0 D i = 8547; block = 16; actual = 8547; expected = 8547
17:12:09 0 D i = 9324; block = 18; actual = 9324; expected = 9324
17:12:09 0 D i = 10101; block = 19; actual = 0; expected = 0
17:12:09 0 D i = 10878; block = 21; actual = 0; expected = 0
17:12:09 0 D i = 11655; block = 22; actual = 11655; expected = 11655
17:12:09 0 D i = 12432; block = 24; actual = 12432; expected = 12432
17:12:09 0 D i = 13209; block = 25; actual = 0; expected = 0
17:12:09 0 D i = 13986; block = 27; actual = 0; expected = 0
17:12:09 0 D i = 14763; block = 28; actual = 14763; expected = 14763
17:12:09 0 D i = 15540; block = 30; actual = 15540; expected = 15540
17:12:09 0 D i = 16317; block = 31; actual = 0; expected = 0
17:12:09 0 D block_cache_stats = Stats { discards: 0, evictions: 0, reads: 32, writes: 0 }
7_fs_1_block_cache_2_read::read_what_was_written--- [passed]
17:12:09 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/fs/block_cache.rs | 77 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 1 file changed, 69 insertions(+), 8 deletions(-)
```
