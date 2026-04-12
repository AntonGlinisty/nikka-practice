## Вытеснение блоков из блочного кэша


### Сравнительный анализ LRU и идеальной политики вытеснения

Сравнительный анализ LRU и идеальной политики вытеснения приведён в статье
["Amortized Efficiency of List Update and Paging Rules", Daniel D. Sleator & Robert E. Tarjan, 1985](https://www.cs.princeton.edu/courses/archive/fall07/cos521/handouts/amortized.pdf).


### Задача 6 --- LRU

Реализуйте политику вытеснения давно неиспользуемых данных ---
[Least Recently Used (LRU)](https://en.wikipedia.org/wiki/Cache_replacement_policies#LRU).
Её код находится в файле
[`ku/src/lru/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/lru/mod.rs).
Вы можете следовать предложенному скелету реализации или же сделать всё по-своему.

Добавьте вытеснение блоков в вашу реализацию
[`BlockCache::register_usage()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.register_usage):

```rust
{{#include ../../kernel/src/fs/block_cache.rs:register_usage}}
```

Используйте этот метод в
[`BlockCache::trap_handler()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.trap_handler)
и
[`BlockCache::emplace_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.emplace_block).
Убедитесь, что вызов метода
[`BlockCache::fill_block()`](../../doc/kernel/fs/block_cache/struct.BlockCache.html#method.fill_block)
также приведёт к учёту доступа к блоку и возможному вытеснению другого блока из кэша.

Добавьте работу с кэшем путей файлов
[`FileSystem::resolve_cache`](../../doc/kernel/fs/struct.FileSystem.html#structfield.resolve_cache)
в вашу реализацию метода
[`FileSystem::open()`](../../doc/kernel/fs/file_system/struct.FileSystem.html#method.open).


#### Статистики политики вытеснения

Для интроспекции LRU поддерживает статистики:

```rust
{{#include ../../ku/src/collections/lru.rs:stats}}
```

С помощью этих статистик в тестах выполняются проверки,
что файловая система использует LRU для блочного кэша и кэширования путей.
Чтобы тесты проходили, не забывайте обновлять статистики LRU в соответствующих случаях.


### Проверьте себя

Теперь должен заработать тесты в файле
[`ku/tests/7-fs-6-lru/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/7-fs-6-lru/mod.rs):

```console
$ (cd ku; cargo test --test 7-fs-6-lru)
...
running 3 tests
2023-12-03T17:40:32.893231Z : basic_lru lru=head: Some(0), tail: Some(4), lru: [{id: 0, key: 0, value: 0, prev: None, next: Some(1)}, {id: 1, key: 1, value: 2, prev: Some(0), next: Some(2)}, {id: 2, key: 2, value: 4, prev: Some(1), next: Some(3)}, {id: 3, key: 3, value: 6, prev: Some(2), next: Some(4)}, {id: 4, key: 4, value: 8, prev: Some(3), next: None}], map: {0 -> 0 [0], 1 -> 2 [1], 2 -> 4 [2], 3 -> 6 [3], 4 -> 8 [4]}
2023-12-03T17:40:32.893233Z : basic_map lru=head: Some(0), tail: Some(4), lru: [{id: 0, key: 0, value: 0, prev: None, next: Some(1)}, {id: 1, key: 1, value: 2, prev: Some(0), next: Some(2)}, {id: 2, key: 2, value: 4, prev: Some(1), next: Some(3)}, {id: 3, key: 3, value: 6, prev: Some(2), next: Some(4)}, {id: 4, key: 4, value: 8, prev: Some(3), next: None}], map: {0 -> 0 [0], 1 -> 2 [1], 2 -> 4 [2], 3 -> 6 [3], 4 -> 8 [4]}
2023-12-03T17:40:32.893489Z : basic_lru lru=head: Some(1), tail: Some(4), lru: [{id: 1, key: 5, value: 10, prev: None, next: Some(2)}, {id: 2, key: 6, value: 12, prev: Some(1), next: Some(3)}, {id: 3, key: 7, value: 14, prev: Some(2), next: Some(0)}, {id: 0, key: 8, value: 16, prev: Some(3), next: Some(4)}, {id: 4, key: 9, value: 18, prev: Some(0), next: None}], map: {5 -> 10 [1], 6 -> 12 [2], 7 -> 14 [3], 8 -> 16 [0], 9 -> 18 [4]}
2023-12-03T17:40:32.893507Z : basic_map lru=head: Some(1), tail: Some(4), lru: [{id: 1, key: 0, value: 0, prev: None, next: Some(2)}, {id: 2, key: 1, value: 3, prev: Some(1), next: Some(3)}, {id: 3, key: 2, value: 6, prev: Some(2), next: Some(0)}, {id: 0, key: 3, value: 9, prev: Some(3), next: Some(4)}, {id: 4, key: 4, value: 12, prev: Some(0), next: None}], map: {0 -> 0 [1], 1 -> 3 [2], 2 -> 6 [3], 3 -> 9 [0], 4 -> 12 [4]}
2023-12-03T17:40:32.893572Z : basic_lru lru=head: Some(0), tail: Some(4), lru: [{id: 0, key: 9, value: 27, prev: None, next: Some(3)}, {id: 3, key: 8, value: 24, prev: Some(0), next: Some(2)}, {id: 2, key: 7, value: 21, prev: Some(3), next: Some(1)}, {id: 1, key: 6, value: 18, prev: Some(2), next: Some(4)}, {id: 4, key: 5, value: 15, prev: Some(1), next: None}], map: {5 -> 15 [4], 6 -> 18 [1], 7 -> 21 [2], 8 -> 24 [3], 9 -> 27 [0]}
2023-12-03T17:40:32.893634Z : basic_lru lru=head: Some(3), tail: Some(4), lru: [{id: 3, key: 14, value: 28, prev: None, next: Some(2)}, {id: 2, key: 13, value: 26, prev: Some(3), next: Some(1)}, {id: 1, key: 12, value: 24, prev: Some(2), next: Some(0)}, {id: 0, key: 11, value: 22, prev: Some(1), next: Some(4)}, {id: 4, key: 10, value: 20, prev: Some(0), next: None}], map: {10 -> 20 [4], 11 -> 22 [0], 12 -> 24 [1], 13 -> 26 [2], 14 -> 28 [3]}
test basic_map ... ok
test basic_lru ... ok
test stress ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.54s
```

Также должны отработать тесты собственно файловой системы,
которые проверяют использование политики вытеснения в блочном кэше и кэше путей файлов:

```console
$ (cd kernel; cargo test --features cache-eviction --test 7-fs-3-inode)
...
7_fs_3_inode::big_file--------------------------------------
22:32:09.141 0 I formatted the file system; free_space = 31.859 MiB; disk = { id: 1, io_port: 0x1F0, io_disk: 1 }; block_size = 4.000 KiB; block_count = 8192; blocks = 36..8192; inode_size = 64 B; inodes = 3..2048; directory_entry_size = 128 B; max_file_size = 513.002 GiB
22:33:36.215 0 D file_block_count = 8074; file_size = 31.539 MiB
22:35:15.605 0 D free_block_count = 65; used_block_count = 8091
22:35:24.057 0 D block_cache_stats = Stats { discards: 8105, emplacements: 8091, evictions: 15215, reads: 8148, writes: 8134 }
22:35:24.075 0 D disk_stats = Stats { errors: 0, flushes: 1, reads: 65184, timeouts: 0, writes: 65072 }
22:35:32.399 0 D unmount; block_cache_stats = Stats { discards: 9129, emplacements: 8091, evictions: 15215, reads: 8148, writes: 8134 }; resolve_cache_stats = Stats { evictions: 0, hits: 0, misses: 0 }
7_fs_3_inode::big_file----------------------------- [passed]
...
```

```console
$ (cd kernel; cargo test --features cache-eviction --test 7-fs-5-open)
...
7_fs_5_open::fs---------------------------------------------
22:37:09.339 0 I formatted the file system; free_space = 31.859 MiB; disk = { id: 1, io_port: 0x1F0, io_disk: 1 }; block_size = 4.000 KiB; block_count = 8192; blocks = 36..8192; inode_size = 64 B; inodes = 3..2048; directory_entry_size = 128 B; max_file_size = 513.002 GiB
22:37:18.639 0 I path = /file-1; entry = 3, file-1, File, 5678 B = 5.545 KiB, 2026-04-15 22:37:18.321 UTC
22:37:18.689 0 I path = /file-2; entry = 5, file-2, File, 9876 B = 9.645 KiB, 2026-04-15 22:37:18.463 UTC
22:37:18.721 0 I path = /dir-1; entry = 6, dir-1, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.369 UTC
22:37:18.761 0 I path = /dir-1/file-4; entry = 7, file-4, File, 0 B = 0 B, 2026-04-15 22:37:18.365 UTC
22:37:18.789 0 I path = /dir-1/file-5; entry = 8, file-5, File, 0 B = 0 B, 2026-04-15 22:37:18.367 UTC
22:37:18.821 0 I path = /dir-1/dir-2; entry = 9, dir-2, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.375 UTC
22:37:18.863 0 I path = /dir-1/dir-2/dir-3; entry = 10, dir-3, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.379 UTC
22:37:18.927 0 I path = /dir-1/dir-2/dir-3/file-3; entry = 11, file-3, File, 0 B = 0 B, 2026-04-15 22:37:18.381 UTC
22:37:18.997 0 I removing; path = /file-1; entry = 3, file-1, File, 5678 B = 5.545 KiB, 2026-04-15 22:37:18.321 UTC
22:37:19.043 0 I removing; path = /file-2; entry = 5, file-2, File, 9876 B = 9.645 KiB, 2026-04-15 22:37:18.463 UTC
22:37:19.085 0 I removing; path = /dir-1; entry = 6, dir-1, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.369 UTC
22:37:19.127 0 I removing; path = /dir-1/file-4; entry = 7, file-4, File, 0 B = 0 B, 2026-04-15 22:37:18.365 UTC
22:37:19.169 0 I removing; path = /dir-1/file-5; entry = 8, file-5, File, 0 B = 0 B, 2026-04-15 22:37:18.367 UTC
22:37:19.213 0 I removing; path = /dir-1/dir-2; entry = 9, dir-2, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.375 UTC
22:37:19.257 0 I removing; path = /dir-1/dir-2/dir-3; entry = 10, dir-3, Directory, 4096 B = 4.000 KiB, 2026-04-15 22:37:18.379 UTC
22:37:19.309 0 I removing; path = /dir-1/dir-2/dir-3/file-3; entry = 11, file-3, File, 0 B = 0 B, 2026-04-15 22:37:18.381 UTC
22:37:19.401 0 D resolve_cache_stats = Stats { evictions: 4, hits: 35, misses: 16 }
22:37:28.001 0 D unmount; block_cache_stats = Stats { discards: 1, emplacements: 11, evictions: 0, reads: 5, writes: 15 }; resolve_cache_stats = Stats { evictions: 4, hits: 35, misses: 16 }
7_fs_5_open::fs------------------------------------ [passed]
...
```

### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/fs/block_cache.rs |  13 ++++++++++-
 ku/src/lru/mod.rs            | 103 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++------------
 2 files changed, 101 insertions(+), 15 deletions(-)
```
