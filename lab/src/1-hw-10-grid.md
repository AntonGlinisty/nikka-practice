## Печать на экране

Для этой задачи требуется сначала решить
[предыдущую](../../lab/book/1-hw-8-cursor.html).

Печать на экран в текстовом режиме графического контроллера
[Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array)
осуществляется с помощью
[ввода--вывода через операции с памятью (Memory--mapped I/O, MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O).
То есть, через инструкции `mov` ассемблера, операции с переменными или массивами языка высокого уровня,
функции вроде
[`copy_from_slice()`](https://doc.rust-lang.org/nightly/core/primitive.slice.html#method.copy_from_slice),
[`std::copy()`](https://en.cppreference.com/w/cpp/algorithm/copy.html) или
[`memcpy()`](https://en.cppreference.com/w/cpp/string/byte/memcpy).
Но данные при этом передаются не между процессором и памятью, а между процессором и внешним устройством.
В нашем случае, --- между процессором и видеокартой.

Организация видеопамяти для текстового режима в контроллере
[Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array)
выглядит следующим образом.
Экран представляется матрицей, элементы которой --- 16-битные структуры.
Записывается матрица по строкам слева направо сверху вниз.
Каждый элемент этой матрицы является парой байт:
- Сначала идёт символ [ASCII (American Standard Code for Information Interchange)](https://en.wikipedia.org/wiki/ASCII), который нужно отобразить в соответствующей позиции экрана.
- После него идёт байт атрибутов этого символа. Этот байт разбит на два полубайта в одинаковом формате:
  - Старший полубайт задаёт цвет фона.
  - Младший --- цвет текста.
  - Цвет задаётся как YRGB, то есть по одному биту в порядке от младшего к старшему: синий, зелёный, красный и яркость.
  - Имеется дополнительная настройка через [порты ввода--вывода](../../lab/book/1-hw-1-rtc.html#%D0%9F%D0%BE%D1%80%D1%82%D1%8B-%D0%B2%D0%B2%D0%BE%D0%B4%D0%B0%D0%B2%D1%8B%D0%B2%D0%BE%D0%B4%D0%B0), которая [меняет смысл бита яркости фона на мигание символа](https://www.reddit.com/r/osdev/comments/70fcig/blinking_text/).

В наиболее распространённом режиме, которым мы и воспользуемся,
экран имеет текстовое разрешение 25 строк и 80 столбцов.
А адресуется его содержимое, начиная с физического адреса `0xB8000`.


### Реализация

Вам нужно будет прочитать и доработать код в файле
[`text/src/grid.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/text/src/grid.rs).

За элемент матрицы текстового содержимого экрана отвечает структура
[`text::grid::Glyph`](../../doc/text/grid/struct.Glyph.html):

```rust
{{#include ../../text/src/grid.rs:glyph}}
```

Для того, чтобы она имела в памяти такое же представление, которое контроллер
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array)
ожидает для элемента текста, она снабжена
[атрибутом `repr(C)`](https://doc.rust-lang.org/nomicon/other-reprs.html#reprc).
Он указывает компилятору располагать поля структуры в памяти так же,
как это делает язык C.

Вспомогательная структура
[`text::grid::GlyphWrapper`](../../doc/text/grid/struct.GlyphWrapper.html)
нужна чтобы обернуть
[`Glyph`](../../doc/text/grid/struct.Glyph.html) в
[`volatile::Volatile`](../../doc/volatile/struct.Volatile.html).
В свою очередь,
[`Volatile`](../../doc/volatile/struct.Volatile.html)
нужен для того, чтобы компилятор при оптимизации не выкинул
чтения и записи символов в память графического контроллера
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array).

Структура
[`text::grid::Grid`](../../doc/text/grid/struct.Grid.html)
задаёт буфер текстового режима
[`Grid::buffer`](../../doc/text/grid/struct.Grid.html#structfield.buffer)
произвольного текстового разрешения:

```rust
{{#include ../../text/src/grid.rs:grid}}
```

И содержит дополнительные поля, позволяющие работать с ним:
- [`Grid::column_count`](../../doc/text/grid/struct.Grid.html#structfield.column_count) --- разрешение в символах по горизонтали, то есть количество символов в одной строке.
- [`Grid::row_start`](../../doc/text/grid/struct.Grid.html#structfield.row_start) --- индекс в [`Grid::buffer`](../../doc/text/grid/struct.Grid.html#structfield.buffer), с которого начинается строка символов,  содержащая текущее положение. То есть, положение в котором находится курсор и в котором будет напечатан следующий символ.
- [`Grid::column`](../../doc/text/grid/struct.Grid.html#structfield.column) --- номер колонки текущего положения. То есть, положения в котором находится курсор и в котором будет напечатан следующий символ.
- [`Grid::tab_width`](../../doc/text/grid/struct.Grid.html#structfield.tab_width) --- количество пробелов в символе табуляции --- `\t`.
- [`Grid::attribute`](../../doc/text/grid/struct.Grid.html#structfield.attribute) --- текущие атрибуты при печати. Они будут использованы при печати следующего символа.

Вам нужно будет реализовать основные методы:

```rust
{{#include ../../text/src/grid.rs:clear}}
```

```rust
{{#include ../../text/src/grid.rs:print_character}}
```

```rust
{{#include ../../text/src/grid.rs:scroll}}
```

Рекомендуется вынести часть их работы во вспомогательные методы:

```rust
{{#include ../../text/src/grid.rs:adjust_position}}
```

```rust
{{#include ../../text/src/grid.rs:character}}
```

```rust
{{#include ../../text/src/grid.rs:newline}}
```

```rust
{{#include ../../text/src/grid.rs:tab}}
```

Для чтения из элемента
[`Grid::buffer`](../../doc/text/grid/struct.Grid.html#structfield.buffer)
используйте метод самого элемента
[`Volatile::read()`](../../doc/volatile/struct.Volatile.html#method.read),
а для записи --- его метод
[`Volatile::write()`](../../doc/volatile/struct.Volatile.html#method.write).

При очистке экрана, заполнения табуляции или оставшейся части строки после `\n` используйте пробелы.

В методе
[`Grid::clear()`](../../doc/text/grid/struct.Grid.html#method.clear)
передаётся параметр `range` типа
[`core::ops::Range<usize>`](https://doc.rust-lang.org/nightly/core/ops/struct.Range.html),
который задаёт очищаемый диапазон в смысле индексов в буфере экрана
[`Grid::buffer`](../../doc/text/grid/struct.Grid.html#structfield.buffer).
Это позволяет не только очищать весь экран, но и, например, очищать одну строку.
Что будет полезно в реализации
[`Grid::scroll()`](../../doc/text/grid/struct.Grid.html#method.scroll).
Передать этот диапазон можно таким же синтаксисом, каким диапазоны записываются в `for`:
```rust
self.clear(start .. end);
```
А при реализации
[`Grid::clear()`](../../doc/text/grid/struct.Grid.html#method.clear)
этот диапазон можно использовать так:
```rust
for i in range {
    ...
}
```

Вам могут пригодиться:
- Метод [`usize::next_multiple_of()`](https://doc.rust-lang.org/nightly/core/primitive.usize.html#method.next_multiple_of).
- Функция [`core::cmp::min()`](https://doc.rust-lang.org/nightly/core/cmp/fn.min.html).


### Ссылки

- [VGA Text Mode / Writing an OS in Rust](https://os.phil-opp.com/vga-text-mode/).
- [Printing To Screen](https://wiki.osdev.org/Printing_To_Screen).
- [VGA Text Mode Operation](http://www.osdever.net/FreeVGA/vga/vgatext.htm).
- [FreeVGA Project](http://www.osdever.net/FreeVGA/freevga.htm).
- [VGA Hardware](https://wiki.osdev.org/VGA_Hardware).


### Запуск тестов

Тесты можно запустить командой `cargo test --lib` в директории `text` репозитория:

```console
/.../nikka$ (cd text; cargo test --lib)
bash: cd: text: No such file or directory
   Compiling text v0.5.0 (/.../nikka/text)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running unittests src/lib.rs (/.../nikka/target/debug/deps/text-eccaabaface7f26b)

running 10 tests
test test::cursor ... ok
test test::explicit_newline_after_an_implicit_one ... ok
test test::explicit_newline ... ok
test test::carriage_return ... ok
test test::out_of_bounds_on_init ... ok
test test::position_is_always_less_than_size ... ok
test test::out_of_bounds_on_tab ... ok
test test::out_of_bounds ... ok
test test::scroll ... ok
test test::printing ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.28s
```


### Ориентировочный объём работ этой части лабораторки

```console
text/src/grid.rs | 100 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----------
1 file changed, 89 insertions(+), 11 deletions(-)
```
