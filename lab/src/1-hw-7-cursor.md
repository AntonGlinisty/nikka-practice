## Управление курсором

Управление курсором в текстовом режиме графического контроллера
[Video Graphics Array (VGA)](https://en.wikipedia.org/wiki/Video_Graphics_Array)
осуществляется через
[порты ввода--вывода](../../lab/book/1-hw-1-rtc.html#%D0%9F%D0%BE%D1%80%D1%82%D1%8B-%D0%B2%D0%B2%D0%BE%D0%B4%D0%B0%D0%B2%D1%8B%D0%B2%D0%BE%D0%B4%D0%B0).
Как и в случае RTC, это пара 8-битных портов индекс--данные с номерами `0x03D4` и `0x03D5`.

Прочитайте исходный код в файле
[`text/src/cursor.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/text/src/cursor.rs),
он очень простой.
Нам нужно будет реализовать четыре метода.

{{#include ../../doc/text/cursor/trait.Cursor.html.method.get}}

{{#include ../../doc/text/cursor/trait.Cursor.html.method.set}}

{{#include ../../doc/text/cursor/trait.Cursor.html.method.set_disable}}

{{#include ../../doc/text/cursor/trait.Cursor.html.method.set_height}}


### Управление позицией курсора

Методы
[`text::cursor::VgaCursor::get()`](../../doc/text/cursor/trait.Cursor.html#tymethod.get)
и
[`text::cursor::VgaCursor::set()`](../../doc/text/cursor/trait.Cursor.html#tymethod.set)
работают с позицией курсора на экране.
Эта позиция --- одномерная, она фактически является номером позиции символа на экране.
Этот номер считается от нуля из левого верхнего угла по направлению слева направо.
При достижении правой границы осуществляется переход в самую левую позицию следующей строки.
Содержимое экрана хранится в массиве, индексы которого пробегают позиции символов на экране
в этом же порядке.

В контроллере
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array)
позиция курсора хранится как 16-битная величина в паре внутренних 8-битных регистров с индексами:
- [`text::cursor::POSITION_HIGH = 0x0E`](../../doc/text/cursor/constant.POSITION_HIGH.html) для старших 8 бит.
- [`text::cursor::POSITION_LOW = 0x0F`](../../doc/text/cursor/constant.POSITION_LOW.html) для младших 8 бит.

То есть, методы
[`VgaCursor::get()`](../../doc/text/cursor/trait.Cursor.html#tymethod.get)
и
[`VgaCursor::set()`](../../doc/text/cursor/trait.Cursor.html#tymethod.set)
осуществляют конвертацию между `usize`, в котором позицию курсора удобно хранить программе,
и этой парой 8-битных регистров контроллера
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array).

При реализации не работайте с
[портами ввода--вывода](../../lab/book/1-hw-1-rtc.html#%D0%9F%D0%BE%D1%80%D1%82%D1%8B-%D0%B2%D0%B2%D0%BE%D0%B4%D0%B0%D0%B2%D1%8B%D0%B2%D0%BE%D0%B4%D0%B0)
напрямую.
Используйте методы `self.0.read()` и `self.0.write()`.
Запись `.0` означает нулевой элемент кортежа --- тип
[`text::cursor::VgaCursor`](../../doc/text/cursor/struct.VgaCursor.html)
является
[кортежем](https://doc.rust-lang.ru/book/ch05-01-defining-structs.html#%D0%9A%D0%BE%D1%80%D1%82%D0%B5%D0%B6%D0%BD%D1%8B%D0%B5-%D1%81%D1%82%D1%80%D1%83%D0%BA%D1%82%D1%83%D1%80%D1%8B-%D1%81%D1%82%D1%80%D1%83%D0%BA%D1%82%D1%83%D1%80%D1%8B-%D0%B1%D0%B5%D0%B7-%D0%B8%D0%BC%D0%B5%D0%BD%D0%BE%D0%B2%D0%B0%D0%BD%D0%BD%D1%8B%D1%85-%D0%BF%D0%BE%D0%BB%D0%B5%D0%B9-%D0%B4%D0%BB%D1%8F-%D1%81%D0%BE%D0%B7%D0%B4%D0%B0%D0%BD%D0%B8%D1%8F-%D1%80%D0%B0%D0%B7%D0%BD%D1%8B%D1%85-%D1%82%D0%B8%D0%BF%D0%BE%D0%B2)
из одного элемента типа `T`.
Это --- использование
[паттерна Newtype](https://doc.rust-lang.ru/book/ch19-04-advanced-types.html#%D0%98%D1%81%D0%BF%D0%BE%D0%BB%D1%8C%D0%B7%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D0%B5-%D0%BF%D0%B0%D1%82%D1%82%D0%B5%D1%80%D0%BD%D0%B0-newtype-%D0%B4%D0%BB%D1%8F-%D0%BE%D0%B1%D0%B5%D1%81%D0%BF%D0%B5%D1%87%D0%B5%D0%BD%D0%B8%D1%8F-%D0%B1%D0%B5%D0%B7%D0%BE%D0%BF%D0%B0%D1%81%D0%BD%D0%BE%D1%81%D1%82%D0%B8-%D1%82%D0%B8%D0%BF%D0%BE%D0%B2-%D0%B8-%D1%81%D0%BE%D0%B7%D0%B4%D0%B0%D0%BD%D0%B8%D1%8F-%D0%B0%D0%B1%D1%81%D1%82%D1%80%D0%B0%D0%BA%D1%86%D0%B8%D0%B9).
А тип `T` должен удовлетворять типажу
[`ku::memory::port::IndexDataPair<u8>`](../../doc/ku/memory/port/trait.IndexDataPair.html),
который описывает пару индекс--данные 8-битных
[портов ввода--вывода](../../lab/book/1-hw-1-rtc.html#%D0%9F%D0%BE%D1%80%D1%82%D1%8B-%D0%B2%D0%B2%D0%BE%D0%B4%D0%B0%D0%B2%D1%8B%D0%B2%D0%BE%D0%B4%D0%B0).
При обращении к регистру оборудования, этот типаж записывает индекс регистра в индексный порт пары,
а данные записывает или читает из порта данных пары.
То есть, как раз то, что нам нужно.
Чтение осуществляет метод
[`IndexDataPair::<u8>::read(index: u8) -> u8`](../../doc/ku/memory/port/trait.IndexDataPair.html#tymethod.read),
а запись --- метод
[`IndexDataPair::<u8>::write(index: u8, data: u8)`](../../doc/ku/memory/port/trait.IndexDataPair.html#tymethod.write).
Нам остаётся только передать в эти методы правильные номера регистров контроллера
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array)
в качестве `index` и правильные данные в `data`.

Почему
[`IndexDataPair`](../../doc/ku/memory/port/trait.IndexDataPair.html) ---
типаж, а не просто тип?
Это используется для того, чтобы в тестах можно было создать эмуляцию
[портов ввода-вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
И проверить корректность работы кода с портами.
Для этого в файле
[`text/src/test.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/text/src/test.rs),
определяется структура `MockCursor`.
Она реализует типаж
[`IndexDataPair`](../../doc/ku/memory/port/trait.IndexDataPair.html),
но не пишет в настоящие
[порты ввода-вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).
Вместо этого она позволяет проверить правильность работы с портами вашего решения задачи.
Это ---
[один из стандартных приёмов тестирования кода](https://en.wikipedia.org/wiki/Mock_object).
В обычном же режиме работы, используется реализация
[`ku::memory::port::IndexDataPortPair`](../../doc/ku/memory/port/struct.IndexDataPortPair.html)
типажа
[`ku::memory::port::IndexDataPair`](../../doc/ku/memory/port/trait.IndexDataPair.html).
Она работает с настоящими
[порты ввода-вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O).


### Управление формой курсора

Контроллер
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array)
позволяет задать вертикальный размер курсора как отрезок номеров горизонтальных линий,
формирующих изображение курсора.
В стандартном текстовом режиме разрешением 80x25 символов сами символы имеют высоту в 16 пикселей.
Такое же количество горизонтальных линий длиной в ширину символа может быть у курсора ---
[`text::cursor::MAX_LINES = 16`](../../doc/text/cursor/constant.MAX_LINES.html).

Регистр
[`text::cursor::START_LINE`](../../doc/text/cursor/constant.START_LINE.html)
контроллера
[VGA](https://en.wikipedia.org/wiki/Video_Graphics_Array)
содержит номер первой, считая сверху, линии курсора.
А регистр
[`text::cursor::END_LINE`](../../doc/text/cursor/constant.END_LINE.html) ---
последней.
Если номер первой линии больше чем последней, то возможны варианты.
Например, курсор отображаться не будет, или будет отображаться как два блока.

Рисовать курсор над символом или где-то посередине слишком экзотично,
поэтому метод
[`text::cursor::VgaCursor::set_height()`](../../doc/text/cursor/trait.Cursor.html#tymethod.set_height)
позволяет указать только количество линий курсора.
Тесты ожидают, что последняя его линия --- самая нижняя.


### Включение и выключение курсора

Для выключения курсора предназначен бит `5` регистра
[`START_LINE`](../../doc/text/cursor/constant.START_LINE.html).
Курсор не будет отображаться, если в этом бите записана единица.
Так как регистр тот же, что и для номера первой линии курсора,
работать с ним нужно аккуратно.
А именно, при изменении номера первой линии курсора
нужно оставить бит выключения в исходном состоянии.
И наоборот, при манипуляциях с битом выключения,
нужно оставить номер первой линии курсора в исходном состоянии.

Для этого придётся:
- Сначала прочитать регистр [`START_LINE`](../../doc/text/cursor/constant.START_LINE.html).
- Поменять в прочитанном значении только нужные биты.
- Записать получившееся значение в регистр.

При этом вам могут пригодиться битовые маски
[`CURSOR_DISABLE`](file:///.../nikka/doc/text/cursor/constant.CURSOR_DISABLE.html)
и
[`CURSOR_LINE_MASK`](file:///.../nikka/doc/text/cursor/constant.CURSOR_DISABLE.html).

При изменении регистра
[`END_LINE`](../../doc/text/cursor/constant.END_LINE.html)
нужно делать то же самое.
Так как в нём биты, не относящиеся к номеру последней линии курсора,
имеют свой собственный смысл.


### Ссылки

- [Text Mode Cursor](https://wiki.osdev.org/Text_Mode_Cursor).
- [Manipulating the Text-mode Cursor](http://www.osdever.net/FreeVGA/vga/textcur.htm).


### Запуск тестов

Тесты можно запустить командой `cargo test --lib cursor` в директории `text` репозитория:

```console
/.../nikka$ (cd text; cargo test --lib cursor)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (/.../nikka/target/debug/deps/text-eccaabaface7f26b)

running 1 test
test test::cursor ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
```


#### Отладка в `gdb`

Тесты в данном случае --- пользовательское приложение,
и ваше решение можно отлаживать стандартным отладчиком.
Для этого обратим внимание на одну из строк, которые печатает `cargo test`:
```console
     Running unittests src/lib.rs (/.../nikka/target/debug/deps/text-eccaabaface7f26b)
```
Тут нас интересует путь к бинарнику теста --- `/.../nikka/target/debug/deps/text-eccaabaface7f26b`.
Запускаем `gdb`, указав этот путь:
```console
$ gdb /.../nikka/target/debug/deps/text-eccaabaface7f26b
...
(gdb) break src/cursor.rs:73
Breakpoint 1 at 0x3e64d: file text/src/cursor.rs, line 73.
(gdb) run cursor --test-threads=1
The program being debugged has been started already.
Start it from the beginning? (y or n) y
Starting program: /.../nikka/target/debug/deps/text-eccaabaface7f26b cursor --test-threads=1
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".

running 1 test
test test::cursor ... [New Thread 0x7ffff7bff640 (LWP 161669)]
[Switching to Thread 0x7ffff7bff640 (LWP 161669)]

Thread 2 "test::cursor" hit Breakpoint 1, text::cursor::{impl#1}::get<&text::test::MockCursor> (self=0x7ffff7bfe468) at text/src/cursor.rs:73
73              // TODO: your code here.
...
```


#### Журналирование

Макросы журналирования ---
`error`, `warn`, `info`, `debug` и `trace` из файла
[`ku/src/log/mod.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/log/mod.rs) ---
должны работать в тесте.
Поменять уровень и другие настройки журналирования можно в самом конце файла теста
[`text/src/test.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/text/src/test.rs).
Аналогично, должен работать отладочный макрос
[`ku::dbg`](../../doc/ku/macro.dbg.html).
[Подробнее про журналирование можно почитать во введении](../../lab/book/0-intro-6-log.html).


### Ориентировочный объём работ этой части лабораторки

```console
text/src/cursor.rs | 54 ++++++++++++++++++++++++++++++++++++++++++++++++------
1 file changed, 48 insertions(+), 6 deletions(-)
```
