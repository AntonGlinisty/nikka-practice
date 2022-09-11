## Счётчики тиков различных источников времени

Структура
[`ku::time::correlation_point::CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html)
из файла [`ku/src/time/correlation_point.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/correlation_point.rs)
предназначена для привязки тактов процессора к другим часам в один момент времени.
Она содержит поля:

- [`CorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.tsc) со значением счётчика тактов процессора.
- [`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count) со значением счётчика тиков другого источника времени в тот же момент.

Если значение
[`CorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.tsc)
равно нулю, то структура в целом считается невалидной ---
[`CorrelationPoint::invalid()`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#method.invalid).
Это означает, что
[`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count)
не привязан ни к какому значению
[`CorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.tsc).
И значение структуры относится не к самому тику, а к промежутку после тика
[`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count)
и до следующего.
При этом значение
[`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count)
может быть использовано как текущее время с низким разрешением --- частотой соответствующих часов.

Посмотрите код реализации структуры
[`ku::time::correlation_point::CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html),
он достаточно прост.

Более интересная структура
[`ku::time::correlation_point::AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
из файла [`ku/src/time/correlation_point.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/time/correlation_point.rs) предназначена для конкурентного доступа к значениям
[`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html).
То есть [`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) и
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
соотносятся также как примитивный тип
[`i64`](https://doc.rust-lang.org/nightly/core/primitive.i64.html)
и атомарный
[`core::sync::atomic::AtomicI64`](https://doc.rust-lang.org/nightly/core/sync/atomic/struct.AtomicI64.html).

Атомарность нужна для того, чтобы конкурентно

- в обработчике прерывания обновлять счётчики [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html);
- а в обычном коде читать эти счётчики, чтобы "посмотреть на часы".

В [`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) два числа типа
[`i64`](https://doc.rust-lang.org/nightly/core/primitive.i64.html),
а в
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html) два соответствующих им
[`AtomicI64`](https://doc.rust-lang.org/nightly/core/sync/atomic/struct.AtomicI64.html).
Нам нужно читать и записывать эти два значения согласованно, поэтому и возникают небольшие сложности.

Можно было бы завести блокировку на доступ к полям
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html),
но это потребовало бы захватывать её из прерывания.
Это практически гарантированно приведёт к проблемам:

- Захват этой блокировки должен ещё и запрещать прерывания.
- Иногда прерывание будет ждать другой код, пока он не отпустит эту блокировку. Обычно прерывания являются более приоритетной деятельностью, поэтому возникнет нежелательная [инверсия приоритетов](https://en.wikipedia.org/wiki/Priority_inversion).
- Писать мы хотим из прерывания в режиме ядра, а читать --- из режима пользователя. При этом с одной стороны, ядро должно обеспечить для кода пользователя согласованность чтения. А с другой --- оно не должны допустить чтобы злонамеренный код из пользовательского режима мог заблокировать ядро, в том числе навечно.

Вообще, захват блокировок из прерываний --- очень плохая идея.

Поэтому реализуем
[неблокирующую синхронизацию](https://en.wikipedia.org/wiki/Non-blocking_algorithm)
для согласованного доступа к полям
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html).
Которая будет отдавать предпочтение писателю, так как это обработчик прерывания в ядре, ---
он всегда сможет завершить работу по обновлению структуры за фиксированное количество тактов.
И никогда не будет ждать читателей, которые могут быть запущены как в режиме ядра, так и в режиме пользователя.
А вот читатели будут максимально пессимизированы ---
они будут вынуждены ждать пока писатель завершит свою работу, если им не повезло запуститься конкурентно с писателем.
Впрочем, для них это не будет страшно, так как:

- Писатель запускается редко, один раз в секунду для микросхемы RTC или двадцать раз в секунду для микросхемы PIT [Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253). Эти два писателя пишут в разные экземпляры структуры, поэтому они никогда не конфликтуют между собой.
- Писатель отрабатывает очень быстро, всего за несколько инструкций процессора.

В качестве инструмента такой неблокирующей синхронизации предлагается использовать упрощённый
[sequence lock](https://en.wikipedia.org/wiki/Seqlock).
В нашем случае его идея в следующем.
Мы жертвуем одним младшим битом в поле
[`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count)
для хранения признака, что сейчас структура
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
находится в несогласованном состоянии из-за текущей активности писателя.
То есть:

- Нечётное значение в [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) означает, что писатель начал обновлять структуру [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html), но ещё не закончил. Если читатель обнаруживает структуру в таком состоянии, он должен подождать пока писатель закончит обновление.
- Чётное значение в [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) означает, что значение структуры [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html) согласованно и читатель может его использовать.

Мы пожертвовали одним младшим битом в
[`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count),
и при этом не хотим терять нечётные значения
[`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count).
Например, можно сделать чтобы значения в этих полях отличались вдвое когда
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
находится в согласованном состоянии:

```rust
AtomicCorrelationPoint::count == 2 * CorrelationPoint::count
```

В нашем случае писатель один, что дополнительно упрощает дело.
Алгоритм его действий:
- Раз он один, значит до начала обновления им структуры [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html), её значение заведомо согласованно. А [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) содержит чётное число.
- Писатель атомарно инкрементирует поле [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count). Оно становится нечётным, символизируя что идёт обновление.
- После этого писатель заполняет поле [`AtomicCorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.tsc).
- Последним действием писатель атомарно обновляет поле [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) на целевое значение. Оно одновременно должно быть чётным. Тогда это действие сигнализирует всем читателям, что обновление завершено и структура находится в согласованном состоянии.

Алгоритм действий читателя чуть сложнее:

- Читатель атомарно загружает значения поля [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count).
- Далее он читает значение поля [`AtomicCorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.tsc).
- После чего он повторно атомарно загружает значение поля [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count).
- Если при обеих загрузках значения поля [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) совпадают и являются чётными, то структура [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html) была прочитана в согласованном состоянии. Читатель делит значение [`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count) пополам, чтобы получить соответствующее значение [`CorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html#structfield.count). И возвращает значение получившейся структуры [`CorrelationPoint`](../../doc/ku/time/correlation_point/struct.CorrelationPoint.html) в вызывающую функцию.
- Если же это не так, читатель повторяет все действия с самого начала.

Перекос сложности в сторону читателя и наличие в нём цикла ожидания, чего нет в писателе, ---
проявление большего приоритета писателя.


### Задача 1 --- реализация [`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)

Реализуйте описанные алгоритмы чтения и записи.
Читает из структуры
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
её [метод](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.load)

```rust
fn AtomicCorrelationPoint::load(&self) -> CorrelationPoint
```

Точнее вам достаточно реализовать вспомогательный [метод](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.try_load)

```rust
fn AtomicCorrelationPoint::try_load(&self) -> Option<CorrelationPoint>
```

Он пытается прочитать значение только один раз.
И возвращает его, завернув в
[`core::option::Option::Some`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.Some),
если оно было прочитано согласованно.

Пишут в
[`AtomicCorrelationPoint`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html)
два других метода.
[Метод](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.store)

```rust
fn AtomicCorrelationPoint::store(&self, correlation_point: CorrelationPoint)
```

записывает заданное `correlation_point` значение. А
[метод](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.inc)

```rust
fn AtomicCorrelationPoint::inc(&self, tsc: i64)
```

увеличивает поле
[`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count)
на минимальное неслужебное значение и записывает в поле
[`AtomicCorrelationPoint::tsc`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.tsc)
значение аргумента `tsc`.

Методом
[`AtomicCorrelationPoint::store()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.store)
пользуется обработчик прерываний RTC.
С его помощью он сохраняет текущее значение секунд, прошедших с начала Unix--эпохи.
А методом
[`AtomicCorrelationPoint::inc()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.inc) ---
обработчик прерываний PIT.
Он сохраняет просто счётчик своих тиков, который никак не привязан к реальному времени.
Поэтому ему достаточно инкрементировать логическое значение своего счётчика при каждом прерывании.

Используйте поле
[`AtomicCorrelationPoint::count`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#structfield.count)
как атомарную переменную, синхронизующую между собой
[`load()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.load) и
[`store()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.store)/[`inc()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.inc).
То есть, правильно расставьте сами атомарные операции доступа к обоим полям и
[`core::sync::atomic::Ordering`](https://doc.rust-lang.org/nightly/core/sync/atomic/enum.Ordering.html)
в них.
К сожалению, тест не может проверить корректность расстановки
[`Ordering`](https://doc.rust-lang.org/nightly/core/sync/atomic/enum.Ordering.html).
Поэтому ей стоит уделить повышенное внимание.

Вы можете опираться на тот факт, что писатель один и методы записи ---
[`AtomicCorrelationPoint::store()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.store) и
[`AtomicCorrelationPoint::inc()`](../../doc/ku/time/correlation_point/struct.AtomicCorrelationPoint.html#method.inc) ---
гарантированно не вызываются конкурентно ни каждый сам с собой, ни друг с другом.


### Theory and Practice of Concurrency

Более подробно познакомиться с конкурентностью, атомарными переменными, синхронизациями, моделями памяти и т.п.,
вы можете на курсе Ромы Липовского "Theory and Practice of Concurrency":

- Публичный доступ:
  - [Плейлист с лекциями](https://youtube.com/playlist?list=PL4_hYwCyhAva37lNnoMuBcKRELso5nvBm).
  - [Репозиторий курса](https://gitlab.com/Lipovsky/concurrency-course).
- [Личный кабинет ШАД, весна 2022](https://lk.yandexdataschool.ru/courses/2022-spring/7.1013-theory-and-practice-of-concurrency/about/).
- [Wiki yandex-team, запись 2020 года](https://wiki.yandex-team.ru/hr/volnoslushateli/lekcii-shad/chetvertyjj-semestr/#zapis2020goda9).


### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/time/correlation_point.rs |   24 +++++++++++++++++++++---
 1 file changed, 21 insertions(+), 3 deletions(-)
```
