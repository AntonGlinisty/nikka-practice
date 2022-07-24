## Журналирование и файл журнала

При обычном запуске сообщения журнала записываются в файл `kernel/serial.out`:

```console
$ make run
...
<Ctrl-C>
$ head -n20 kernel/serial.out
...
15:08:28 0 I RTC init; acknowledged_settings = USE_24_HOUR_FORMAT | USE_BINARY_FORMAT | UPDATE_ENDED_INTERRUPT
15:08:28 0 I time init
15:08:28 0 I Nikka booted; now = 2022-08-28 15:08:28 UTC; tsc = Tsc(10571359859)
15:08:28 0 I GDT init
15:08:28 0 I traps init
15:08:28 0 I phys2virt = Page(402653184 @ 0v18000000000)
15:08:28 0 I init; frame_allocator = "boot frame allocator"; block = [9.664 MiB, 49.875 MiB), size 40.211 MiB; free_frame_count = 10294
15:08:28 0 I page allocator init; free_page_count = 33822867456; block = [2.000 TiB, 128.000 TiB), size 126.000 TiB
15:08:28 0 I init; address_space = "base" @ 0p1000
15:08:28 0 I drop; address_space = "invalid" @ 0p0
15:08:28 0 I available memory; total = 49.875 MiB; usable = 41.715 MiB; frame_count = 12768; usable_frame_count = 10679
15:08:28 0 D frame info mapped; duration = 52.040 Mtsc
15:08:28 0 D frame info init; duration = 82.218 Mtsc
15:08:28 0 D move free frames; dst = "main frame allocator"; src = "boot frame allocator"; free_frame_count = 10216
15:08:28 0 I drop; frame_allocator = "boot frame allocator"; block = [9.664 MiB, 9.664 MiB), size 0 B; leaked_frame_count = 0
15:08:28 0 D moved all free frames; duration = 180.033 Mtsc
15:08:28 0 I init; frame_allocator = "main frame allocator"; free_frame_count = 10601; duration = 322.017 Mtsc
15:08:28 0 I memory init; duration = 461.240 Mtsc
15:08:28 0 I acpi = Acpi { local_apic_address: Phys(0pFEE00000), bsp_id: 0, ap_ids: [1, 2, 3] }
15:08:28 0 I Local APIC init; cpu = 0; cpu_count = 4; local_apic_address = 0pFEE00000
```

Для журналирования используйте макросы `error`, `warn`, `info`, `debug` и `trace` из файла [`kernel/src/log/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/log/mod.rs).
Это реэкспорты макросов библиотеки [tracing](https://docs.rs/tracing/).

Например, вызов

```rust
info!(free_page_count = page_allocator.block.count(), block = %page_allocator.block, "page allocator init");
```

приводит к печати
```console
15:08:28 0 I page allocator init; free_page_count = 33822867456; block = [2.000 TiB, 128.000 TiB), size 126.000 TiB
```

Здесь:

- `15:08:28` --- время сообщения.
- `0` --- номер процессора, на котором работал код.
- `I` --- признак уровня журналирования `INFO`.
- `page allocator init` --- текст сообщения, за которым идут значения полей, разделённые знаком `;`.

Запись

```rust
info!(..., free_page_count = page_allocator.block.count(), ...);
```

означает, что `free_page_count` --- имя поля в структурированной части сообщения, а `page_allocator.block.count()` --- значение этого поля.
Если в коде есть переменная `free_page_count` с нужным значением, `= ...` можно опустить:

```rust
info!(..., free_page_count, ...);
```

Если вы забудете указать имя поля при печати сложного выражения, а не просто переменной, возникнет синтаксическая ошибка

```console
error: expected expression, found `%`
  --> src/tests/memory.rs:21:12
   |
21 |     debug!(%frames[0]);
   |            ^ expected expression
```

Просто добавьте имя поля:
```rust
debug!(frame = %frames[0]);
```

Tracing поддерживает печать встроенных типов напрямую.
А вот для пользовательских типов нужно указать `%` или `?`, как в

```rust
trace!(page_allocator = %self, ..., ?block, ...);
```

`%` означает печать типа через [типаж](https://doc.rust-lang.ru/book/ch10-02-traits.html) [`core::fmt::Display`](https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html), а `?` --- через типаж [`core::fmt::Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html).
Если забыть указать `%` или `?` --- возникнет ошибка, говорящая, что для типа не реализован типаж [`tracing::Value`](../../doc/tracing/trait.Value.html):

```console
error[E0277]: the trait bound `PageAllocator: Value` is not satisfied
  --> src/memory/page_allocator.rs:82:17
   |
82 |                 trace!(page_allocator = self, size, page_count, ?block, "allocate pages");
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Value` is not implemented for `PageAllocator`
   |
   = note: required because of the requirements on the impl of `Value` for `&mut PageAllocator`
   = note: required for the cast to the object type `dyn Value`
   = note: this error originates in the macro `$crate::valueset` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
```

Если же тип не реализует запрошенный через `%` или `?` типаж, возникнет похожая ошибка, но уже про типаж [`Display`](https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html) или [`Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html):

```console
error[E0277]: `PageAllocator` doesn't implement `Display`
  --> src/memory/page_allocator.rs:82:17
   |
82 |                 trace!(page_allocator = %self, size, page_count, ?block, "allocate pages");
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `PageAllocator` cannot be formatted with the default formatter
   |
   = help: the trait `Display` is not implemented for `PageAllocator`
   = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
   = note: required because of the requirements on the impl of `Display` for `&mut PageAllocator`
   = note: 1 redundant requirements hidden
   = note: required because of the requirements on the impl of `Display` for `&&mut PageAllocator`
   = note: required because of the requirements on the impl of `Value` for `DisplayValue<&&mut PageAllocator>`
   = note: required for the cast to the object type `dyn Value`
   = note: this error originates in the macro `$crate::valueset` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
```

Самый простой способ это починить, --- использовать [`Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html) с автоматической реализацией через макрос `derive` в месте объявления типа, например:

```rust
#[derive(Debug)]
struct PageAllocator {
    ...
}
```

Это потребует реализацию [`Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html) от всех полей структуры, которую при необходимости можно добавить аналогичным способом в местах объявления типов полей.

[`Display`](https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html) тоже можно реализовать с помощью макроса из внешней библиотеки [derive_more](https://docs.rs/derive_more/0.99.16/derive_more/), при этом можно указать [формат печати](https://jeltef.github.io/derive_more/derive_more/display.html):

```rust
use derive_more::Display;
...
#[derive(Display)]
#[display(fmt = "{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}
```

Также можно реализовать [`Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html) или [`Display`](https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html) вручную, например:

```rust
impl fmt::Display for Size {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match NumberPrefix::binary(self.size() as f64) {
            NumberPrefix::Standalone(_) => {
                write!(formatter, "{} B", self.size())
            }
            NumberPrefix::Prefixed(prefix, value) => {
                write!(formatter, "{:.3} {}B", value, prefix.symbol())
            }
        }
    }
}
```

Эта реализация объясняет почему размеры в записи журнала имели суффиксы `TiB`: `... block = [2.000 TiB, 128.000 TiB), size 126.000 TiB`.

Поменять глобальный уровень журналирования можно в самом конце файла [`kernel/src/log/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/log/mod.rs):

```rust
static LOG_COLLECTOR: ... = LogCollector::new(..., Level::INFO);
```

Другие примеры использования можно посмотреть в документации tracing, например [для `info!()`](../../doc/tracing/macro.info.html#examples).
Рядом есть и [более подробная документация по синтаксису макросов журналирования](../../doc/tracing/index.html#using-the-macros).
