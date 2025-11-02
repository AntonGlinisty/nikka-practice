## Отсутствие утечек физических фреймов

Для проверки аллокатора на утечки физических фреймов,
нам необходимо уметь полностью удалять аллокатор с освобождением всех его метаданных.
Выполнить такую операцию, естественно, можно только если все
блоки памяти, выделенные из аллокатора пользователю, были им возвращены.
Это забота тестов.

Доработайте
[метод](../../doc/ku/allocator/fixed_size/struct.FixedSizeAllocator.html#method.unmap)

```rust
{{#include ../../ku/src/allocator/fixed_size.rs:unmap}}
```

в файле
[`ku/src/allocator/fixed_size.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/fixed_size.rs).


И
[метод](../../doc/ku/allocator/dispatcher/struct.Dispatcher.html#method.unmap)

```rust
{{#include ../../ku/src/allocator/dispatcher.rs:unmap}}
```

в файле
[`ku/src/allocator/dispatcher.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/dispatcher.rs).


### Проверьте себя

Запустите тест `3-allocator-5-small` из файла
[`ku/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-5-small.rs),
указав опцию `--features forbid-leaks`:

```console
$ (cd ku && cargo test --features forbid-leaks --test 3-allocator-5-small basic -- --nocapture --test-threads=1)
...
```


### Ориентировочный объём работ этой части лабораторки

```console
src/allocator/dispatcher.rs | 11 ++++++++++-
src/allocator/fixed_size.rs | 23 +++++++++++++++++++++--
2 files changed, 31 insertions(+), 3 deletions(-)
```
