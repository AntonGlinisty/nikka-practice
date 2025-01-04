### Задача 1 --- реализация циклического буфера


#### Сериализация и десериализация заголовка

Реализуйте
[метод](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.read_header)

```rust
fn RingBuffer::read_header(
    &mut self,
    position: usize,
) -> Header
```

и
[метод](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.write_header)

```rust
fn RingBuffer::write_header(
    &mut self,
    position: usize,
    header: Header,
)
```

в файле
[`ku/src/ipc/pipe.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/ipc/pipe.rs)
Они должен десериализовать и сериализовать заголовок `header` в циклическом буфере по позиции `position`.
Подумайте над последовательностью операций в них и необходимыми
[`Ordering`](https://doc.rust-lang.org/nightly/core/sync/atomic/enum.Ordering.html).

Также учтите, что при записи заголовка может оказаться, что в этот же заголовок конкурентно пишется значение
[`Header::Closed`](../../doc/ku/ipc/pipe/enum.Header.html#variant.Closed).
Оно имеет преимущество и не должно быть потеряно, а запись другого заголовка поверх него случиться не должна.
За исключением ситуации, когда позиция заголовка ещё не использована.
То есть, когда логически буфер не дошёл до неё.
В этом случае писатель заранее записывает заголовок
[`Header::Clear`](../../doc/ku/ipc/pipe/enum.Header.html#variant.Clear)
сразу после текущей записи с данными.
Это нужно, так как буфер всё же циклический и переиспользует одну и ту же память.
А значит, хотя логически в этой позиции ещё ничего не писалось, но из-за взятия позиции по модулю,
она попадает на какие-то старые данные в буфере.
Которые могли быть даже не данными заголовка, а передаваемыми пользовательскими.
И случайно могут выглядеть как произвольный заголовок,
в том числе как
[`Header::Closed`](../../doc/ku/ipc/pipe/enum.Header.html#variant.Closed).

Также реализуйте
[метод](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.close)

```rust
fn RingBuffer::close(
    &mut self,
)
```

который закрывает буфер на чтение и запись.

Вам пригодятся методы:

- [`RingBuffer::state()`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.state), --- возвращает [`AtomicU8`](https://doc.rust-lang.org/nightly/core/sync/atomic/struct.AtomicU8.html) с состоянием записи.
- [`RingBuffer::read_size()`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.read_size), --- читает размер записи из её заголовка.
- [`RingBuffer::write_size()`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.write_size), --- записывает размер записи в её заголовок.

В этих методах запись идентифицируется позицией своего заголовка `position` ---
это количество байт записанных до этой записи с начала работы
[`RingBuffer`](../../doc/ku/ipc/pipe/struct.RingBuffer.html).
Оно только возрастает, так как содержит значение до взятия по модулю размера буфера.


#### Получение актуальных головы и хвоста

Реализуйте
[метод](../../doc/ku/ipc/pipe/type.WriteBuffer.html#method.advance_head)

```rust
{{#include ../../ku/src/ipc/pipe.rs:advance_head}}
```

Он должен обновить поле
[`WriteBuffer::head`](../../doc/ku/ipc/pipe/type.WriteBuffer.html#structfield.head).

И симметричный
[метод](../../doc/ku/ipc/pipe/type.ReadBuffer.html#method.advance_tail)

```rust
{{#include ../../ku/src/ipc/pipe.rs:advance_tail}}
```

Он должен обновить поле
[`ReadBuffer::tail`](../../doc/ku/ipc/pipe/type.ReadBuffer.html#structfield.tail).

Для этого, отталкиваясь от их текущих значений, они читает заголовки последующих записей методом
[`RingBuffer::read_header()`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#method.read_header).
И продвигают
[`WriteBuffer::head`](../../doc/ku/ipc/pipe/type.WriteBuffer.html#structfield.head)
и
[`ReadBuffer::tail`](../../doc/ku/ipc/pipe/type.ReadBuffer.html#structfield.tail)
в соответствии с состояниями записей в прочитанных заголовках.
Если в голове находится запись в состоянии
[`Header::Closed`](../../doc/ku/ipc/pipe/enum.Header.html#variant.Closed),
это означает что буфер закрыт и методы возвращают `None`.
Кроме того, при обнаружении любых несогласованностей или нарушений инвариантов во время чтения заголовков,
буфер также считается закрытым.
Если всё хорошо, возвращается новое значение `head` или `tail` соответственно.


#### Читающая транзакция

Реализуйте [метод](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.read)

```rust
fn RingBufferTx<'_, ReadTag>::read(
    &mut self,
) -> Option<&[u8]>
```

который возвращает в виде среза полезную нагрузку очередной записи из буфера.
Или `None`, если в этой читающей транзакции больше нет записей.
При этом он обновляет поля только самой транзакции
[`RingBufferTx`](../../doc/ku/ipc/pipe/struct.RingBufferTx.html).

Реализуйте
[метод](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.commit)

```rust
fn RingBufferTx<'_, ReadTag>::commit(
    mut self,
)
```

который фиксирует читающую транзакцию, обновляя значение
[`RingBuffer::head`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#structfield.head)
и статистики
[`RingBuffer::stats`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#structfield.stats).

В разделяемый циклический буфер при этом запишите один заголовок, покрывающий все прочитанные данные.
То есть, мы не пытаемся сохранить положения и количество заголовков:
- Это ни для чего не нужно и является лишней работой.
- Заметно усложняет код.


#### Пишущая транзакция

Реализуйте [метод](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.write)

```rust
fn RingBufferTx<'_, WriteTag>::write(
    &mut self,
    data: &[u8],
) -> Result<()>
```

который копирует в буфер байты среза `data`, обновляя поля самой транзакции
[`RingBufferTx`](../../doc/ku/ipc/pipe/struct.RingBufferTx.html)
и статистику ошибок в
[`RingBuffer::stats`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#structfield.stats).
Но не трогая поля
[`RingBuffer::head`](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#structfield.head)
и
[`RingBuffer::tail`](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#structfield.tail).
Если в буфере не остаётся места под `data`, верните ошибку
[`Error::Overflow`](../../doc/ku/ipc/pipe/enum.Error.html#variant.Overflow):

```rust
{{#include ../../ku/src/ipc/pipe.rs:error}}
```

Вам может пригодиться метод
[`copy_from_slice()`](https://doc.rust-lang.org/nightly/core/primitive.slice.html#method.copy_from_slice)
срезов.

Реализуйте [метод](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.commit-1)

```rust
fn RingBufferTx<'_, WriteTag>::commit(
    mut self,
)
```

который фиксирует пишущую транзакцию, обновляя значение
[`RingBuffer::tail`](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#structfield.tail)
и статистики
[`RingBuffer::stats`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#structfield.stats).

Учтите, что читатель может прочитать внутри своей транзакции несколько записей и
зафиксировать их все внутри одной своей транзакции.


#### Сброс транзакции

Реализуйте типаж
[`core::ops::Drop`](https://doc.rust-lang.org/nightly/core/ops/trait.Drop.html)
для транзакций обоих типов ---
[метод](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.drop)

```rust
fn RingBufferTx::drop(
    &mut self,
)
```

Он обновляет статистики
[`RingBuffer::stats`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#structfield.stats),
если в транзакции был прочитан или записан хотя бы один байт.

Метод
[RingBufferTx::drop()](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.drop)
фактически отменит транзакцию, если её явно не зафиксировать методом
[RingBufferTx::commit()](../../doc/ku/ipc/pipe/struct.RingBufferTx.html#method.commit)

#### Отображение буфера в память


Реализуйте
[функцию](../../doc/ku/ipc/pipe/fn.make.html)

```rust
fn make<T: BigAllocator>(
    allocator: &mut T,
) -> error::Result<(ReadBuffer, WriteBuffer)>
```

Она должна создать двойное отображение памяти буфера в адресное пространство,
за которое отвечает `allocator`.
То есть, она должна выделить
[`RingBuffer::REAL_SIZE`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#associatedconstant.REAL_SIZE)
байт в физической памяти и
[`RingBuffer::MAPPED_SIZE`](../../doc/ku/ipc/pipe/struct.RingBuffer.html#associatedconstant.MAPPED_SIZE) ---
в виртуальной.
И построить отображение выделенных физических фреймов дважды подряд.
Также необходимо записать в начало буфера заголовок
[`Header::Clear`](../../doc/ku/ipc/pipe/enum.Header.html#variant.Clear).

Для создания
[`ReadBuffer`](../../doc/ku/ipc/pipe/type.ReadBuffer.html)
и
[`WriteBuffer`](../../doc/ku/ipc/pipe/type.WriteBuffer.html)
можно использовать конструкции

```ring_buffer
Buffer::<WriteTag>(..., PhantomData);
Buffer::<ReadTag>(..., PhantomData);
```


#### Сброс буфера

За сброс буфера с сообщениями процесса пользователя в общий журнал отвечает ядро.
Сам сброс осуществляет функция
[`Process::flush_log()`](../../doc/kernel/process/process/struct.Process.html#method.flush_log).
А вызываться она должна:

- Из метода [`Process::trap()`](../../doc/kernel/process/process/struct.Process.html#method.trap) при исключениях в коде пользователя. Это уже делается.
- Из диспетчера системных вызовов [`kernel::process::syscall::syscall()`](../../doc/kernel/process/syscall/fn.syscall.html). Это вам придётся добавить самостоятельно. Чтобы не поменялся логический порядок записей, относящихся к одному процессу, сделайте сброс до того как ядро запишет в журнал какое-либо своё сообщение при обработке системного вызова.


### Проверьте себя

Теперь должен заработать тест `6-um-1-pipe` в файле
[`ku/tests/6-um-1-pipe.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/6-um-1-pipe.rs):

```console
$ (cd ku; cargo test --test 6-um-1-pipe -- --nocapture --test-threads=1)
...
running 5 tests
test check_continuous_mapping ... 2023-11-12T19:52:21.306207Z : ThreadId(03) ring_buffer_block=[0v7F24_0DA4_A000, 0v7F24_0DA5_2000), size 32.000 KiB
ok
test concurrent ... 2023-11-12T19:52:21.808266Z : read_thread iteration=0 read_tx=69323 throughput_per_second=84.375 MiB window_bytes=44237229 read_stats=RingBufferStats { committed: 22712700, commits: 34693, dropped: 21524529, drops: 2107, errors: 0, txs: 69324 }
2023-11-12T19:52:21.808366Z : write_thread iteration=0 write_tx=3601 write_stats=RingBufferStats { committed: 22712700, commits: 2762, dropped: 15589937, drops: 5678, errors: 32775, txs: 36376 }
2023-11-12T19:52:22.308398Z :  read_thread iteration=0 read_tx=140326 throughput_per_second=81.723 MiB window_bytes=42846219 read_stats=RingBufferStats { committed: 44844551, commits: 70227, dropped: 42238897, drops: 4144, errors: 0, txs: 140327 }
2023-11-12T19:52:22.308447Z : write_thread iteration=0 write_tx=7210 write_stats=RingBufferStats { committed: 44844551, commits: 5450, dropped: 32238329, drops: 11639, errors: 65594, txs: 72804 }
2023-11-12T19:52:23.192529Z :  read_thread iteration=1 read_tx=68767 throughput_per_second=84.235 MiB window_bytes=44172197 read_stats=RingBufferStats { committed: 22660430, commits: 34439, dropped: 21511767, drops: 2092, errors: 0, txs: 68768 }
2023-11-12T19:52:23.192579Z : write_thread iteration=1 write_tx=3639 write_stats=RingBufferStats { committed: 22668544, commits: 2764, dropped: 18504829, drops: 6300, errors: 37113, txs: 40752 }
2023-11-12T19:52:23.692783Z : write_thread iteration=1 write_tx=7340 write_stats=RingBufferStats { committed: 45513134, commits: 5572, dropped: 36808688, drops: 12732, errors: 73855, txs: 81195 }
2023-11-12T19:52:23.692819Z :  read_thread iteration=1 read_tx=137101 throughput_per_second=85.916 MiB window_bytes=45058725 read_stats=RingBufferStats { committed: 45513134, commits: 68654, dropped: 43717788, drops: 4216, errors: 0, txs: 137102 }
2023-11-12T19:52:24.551556Z :  read_thread iteration=2 read_tx=69153 throughput_per_second=82.674 MiB window_bytes=43345128 read_stats=RingBufferStats { committed: 22209155, commits: 34752, dropped: 21135973, drops: 2057, errors: 0, txs: 69154 }
2023-11-12T19:52:24.551618Z : write_thread iteration=2 write_tx=3696 write_stats=RingBufferStats { committed: 22209155, commits: 2739, dropped: 19173939, drops: 6499, errors: 35893, txs: 39589 }
2023-11-12T19:52:25.051871Z :  read_thread iteration=2 read_tx=136700 throughput_per_second=83.770 MiB window_bytes=43930049 read_stats=RingBufferStats { committed: 44860375, commits: 68587, dropped: 42414802, drops: 4159, errors: 0, txs: 136701 }
2023-11-12T19:52:25.052015Z : write_thread iteration=2 write_tx=7416 write_stats=RingBufferStats { committed: 44875078, commits: 5540, dropped: 37104985, drops: 12867, errors: 72384, txs: 79800 }
2023-11-12T19:52:25.903372Z : write_thread iteration=3 write_tx=3588 write_stats=RingBufferStats { committed: 21848127, commits: 2666, dropped: 19460906, drops: 6711, errors: 38016, txs: 41604 }
2023-11-12T19:52:25.903459Z :  read_thread iteration=3 read_tx=66039 throughput_per_second=79.772 MiB window_bytes=41837923 read_stats=RingBufferStats { committed: 21848127, commits: 33096, dropped: 19989796, drops: 1954, errors: 0, txs: 66040 }
2023-11-12T19:52:26.403615Z :  read_thread iteration=3 read_tx=132322 throughput_per_second=82.199 MiB window_bytes=43101965 read_stats=RingBufferStats { committed: 43788016, commits: 66178, dropped: 41151872, drops: 3949, errors: 0, txs: 132323 }
2023-11-12T19:52:26.403678Z : write_thread iteration=3 write_tx=7118 write_stats=RingBufferStats { committed: 43798492, commits: 5294, dropped: 38380789, drops: 13220, errors: 77381, txs: 84499 }
2023-11-12T19:52:27.316664Z :  read_thread iteration=4 read_tx=68201 throughput_per_second=83.010 MiB window_bytes=43528259 read_stats=RingBufferStats { committed: 22559379, commits: 34317, dropped: 20968880, drops: 2102, errors: 0, txs: 68202 }
2023-11-12T19:52:27.316812Z : write_thread iteration=4 write_tx=3683 write_stats=RingBufferStats { committed: 22571422, commits: 2767, dropped: 18518952, drops: 6574, errors: 36349, txs: 40032 }
2023-11-12T19:52:27.816984Z :  read_thread iteration=4 read_tx=135473 throughput_per_second=83.836 MiB window_bytes=43969359 read_stats=RingBufferStats { committed: 45402091, commits: 67891, dropped: 42095527, drops: 4154, errors: 0, txs: 135474 }
2023-11-12T19:52:27.817004Z : write_thread iteration=4 write_tx=7385 write_stats=RingBufferStats { committed: 45406911, commits: 5548, dropped: 36254608, drops: 12830, errors: 72904, txs: 80289 }
2023-11-12T19:52:28.667193Z : write_thread iteration=5 write_tx=3698 write_stats=RingBufferStats { committed: 22874812, commits: 2761, dropped: 18095511, drops: 6085, errors: 36770, txs: 40468 }
2023-11-12T19:52:28.667193Z :  read_thread iteration=5 read_tx=67879 throughput_per_second=85.057 MiB window_bytes=44594564 read_stats=RingBufferStats { committed: 22874812, commits: 33926, dropped: 21719752, drops: 2093, errors: 0, txs: 67880 }
2023-11-12T19:52:29.167421Z :  read_thread iteration=5 read_tx=136354 throughput_per_second=84.950 MiB window_bytes=44538395 read_stats=RingBufferStats { committed: 45729519, commits: 68176, dropped: 43403440, drops: 4173, errors: 0, txs: 136355 }
2023-11-12T19:52:29.167594Z : write_thread iteration=5 write_tx=7375 write_stats=RingBufferStats { committed: 45729519, commits: 5543, dropped: 36346938, drops: 12472, errors: 74106, txs: 81481 }
2023-11-12T19:52:30.021125Z : write_thread iteration=6 write_tx=3727 write_stats=RingBufferStats { committed: 22404773, commits: 2806, dropped: 19147120, drops: 6733, errors: 36144, txs: 39871 }
2023-11-12T19:52:30.021120Z :  read_thread iteration=6 read_tx=68547 throughput_per_second=83.884 MiB window_bytes=43980540 read_stats=RingBufferStats { committed: 22404773, commits: 33900, dropped: 21575767, drops: 2106, errors: 0, txs: 68548 }
2023-11-12T19:52:30.521303Z : write_thread iteration=6 write_tx=7400 write_stats=RingBufferStats { committed: 45085806, commits: 5555, dropped: 38308302, drops: 13116, errors: 72358, txs: 79758 }
2023-11-12T19:52:30.521303Z :  read_thread iteration=6 read_tx=136891 throughput_per_second=82.315 MiB window_bytes=43156808 read_stats=RingBufferStats { committed: 45078305, commits: 67983, dropped: 42059043, drops: 4083, errors: 0, txs: 136892 }
2023-11-12T19:52:31.382838Z : write_thread iteration=7 write_tx=3606 write_stats=RingBufferStats { committed: 22572729, commits: 2730, dropped: 17497302, drops: 6043, errors: 35576, txs: 39182 }
2023-11-12T19:52:31.382973Z :  read_thread iteration=7 read_tx=68347 throughput_per_second=82.725 MiB window_bytes=43387343 read_stats=RingBufferStats { committed: 22572729, commits: 34383, dropped: 20814614, drops: 2036, errors: 0, txs: 68348 }
2023-11-12T19:52:31.883077Z : write_thread iteration=7 write_tx=7311 write_stats=RingBufferStats { committed: 45308170, commits: 5516, dropped: 35484786, drops: 12216, errors: 70716, txs: 78027 }
2023-11-12T19:52:31.883070Z :  read_thread iteration=7 read_tx=137032 throughput_per_second=82.480 MiB window_bytes=43243553 read_stats=RingBufferStats { committed: 45308170, commits: 68469, dropped: 41322726, drops: 4052, errors: 0, txs: 137033 }
2023-11-12T19:52:32.754459Z :  read_thread iteration=8 read_tx=69134 throughput_per_second=80.648 MiB window_bytes=42288345 read_stats=RingBufferStats { committed: 22225655, commits: 34374, dropped: 20062690, drops: 1936, errors: 0, txs: 69135 }
2023-11-12T19:52:32.754540Z : write_thread iteration=8 write_tx=3627 write_stats=RingBufferStats { committed: 22236394, commits: 2667, dropped: 19337059, drops: 6249, errors: 36119, txs: 39746 }
2023-11-12T19:52:33.254653Z :  read_thread iteration=8 read_tx=136968 throughput_per_second=82.453 MiB window_bytes=43234462 read_stats=RingBufferStats { committed: 44900680, commits: 68099, dropped: 40622127, drops: 3987, errors: 0, txs: 136969 }
2023-11-12T19:52:33.254767Z : write_thread iteration=8 write_tx=7311 write_stats=RingBufferStats { committed: 44915576, commits: 5431, dropped: 38388675, drops: 12869, errors: 71364, txs: 78675 }
2023-11-12T19:52:34.116822Z :  read_thread iteration=9 read_tx=68621 throughput_per_second=81.167 MiB window_bytes=42555082 read_stats=RingBufferStats { committed: 22477495, commits: 34435, dropped: 20077587, drops: 2029, errors: 0, txs: 68622 }
2023-11-12T19:52:34.116916Z : write_thread iteration=9 write_tx=3685 write_stats=RingBufferStats { committed: 22493863, commits: 2766, dropped: 18678299, drops: 6126, errors: 36516, txs: 40201 }
2023-11-12T19:52:34.617104Z :  read_thread iteration=9 read_tx=137632 throughput_per_second=82.408 MiB window_bytes=43217791 read_stats=RingBufferStats { committed: 44908732, commits: 68866, dropped: 40864141, drops: 4082, errors: 0, txs: 137633 }
2023-11-12T19:52:34.617219Z : write_thread iteration=9 write_tx=7264 write_stats=RingBufferStats { committed: 44914850, commits: 5438, dropped: 37398833, drops: 12528, errors: 73495, txs: 80759 }
2023-11-12T19:52:35.485207Z :  read_thread iteration=10 read_tx=68252 throughput_per_second=82.804 MiB window_bytes=43413057 read_stats=RingBufferStats { committed: 22612958, commits: 34067, dropped: 20800099, drops: 2032, errors: 0, txs: 68253 }
2023-11-12T19:52:35.485248Z : write_thread iteration=10 write_tx=3687 write_stats=RingBufferStats { committed: 22612958, commits: 2752, dropped: 18398249, drops: 6225, errors: 36618, txs: 40305 }
2023-11-12T19:52:35.985357Z :  read_thread iteration=10 read_tx=137085 throughput_per_second=82.757 MiB window_bytes=43388337 read_stats=RingBufferStats { committed: 45110463, commits: 68442, dropped: 41690931, drops: 4063, errors: 0, txs: 137086 }
2023-11-12T19:52:35.985451Z : write_thread iteration=10 write_tx=7324 write_stats=RingBufferStats { committed: 45125897, commits: 5456, dropped: 37282255, drops: 12432, errors: 73285, txs: 80609 }
2023-11-12T19:52:36.847501Z :  read_thread iteration=11 read_tx=69581 throughput_per_second=84.385 MiB window_bytes=44242447 read_stats=RingBufferStats { committed: 22587413, commits: 34742, dropped: 21655034, drops: 2124, errors: 0, txs: 69582 }
2023-11-12T19:52:36.847526Z : write_thread iteration=11 write_tx=3702 write_stats=RingBufferStats { committed: 22587413, commits: 2768, dropped: 18006864, drops: 6147, errors: 33460, txs: 37162 }
2023-11-12T19:52:37.347666Z :  read_thread iteration=11 read_tx=139317 throughput_per_second=78.768 MiB window_bytes=41297346 read_stats=RingBufferStats { committed: 44641083, commits: 69743, dropped: 40898710, drops: 4027, errors: 0, txs: 139318 }
2023-11-12T19:52:37.347762Z : write_thread iteration=11 write_tx=7309 write_stats=RingBufferStats { committed: 44651094, commits: 5439, dropped: 35093918, drops: 12370, errors: 68210, txs: 75519 }
2023-11-12T19:52:38.216991Z :  read_thread iteration=12 read_tx=64707 throughput_per_second=78.220 MiB window_bytes=41009899 read_stats=RingBufferStats { committed: 21504116, commits: 32311, dropped: 19505783, drops: 1937, errors: 0, txs: 64708 }
2023-11-12T19:52:38.217104Z : write_thread iteration=12 write_tx=3462 write_stats=RingBufferStats { committed: 21515230, commits: 2626, dropped: 18622637, drops: 6655, errors: 41772, txs: 45234 }
2023-11-12T19:52:38.717129Z :  read_thread iteration=12 read_tx=127971 throughput_per_second=80.635 MiB window_bytes=42276022 read_stats=RingBufferStats { committed: 43514035, commits: 63994, dropped: 39771886, drops: 3918, errors: 0, txs: 127972 }
2023-11-12T19:52:38.717201Z : write_thread iteration=12 write_tx=6951 write_stats=RingBufferStats { committed: 43514035, commits: 5282, dropped: 37288428, drops: 13391, errors: 82031, txs: 88982 }
2023-11-12T19:52:39.660248Z : write_thread iteration=13 write_tx=3574 write_stats=RingBufferStats { committed: 22470710, commits: 2675, dropped: 18497081, drops: 6304, errors: 36759, txs: 40333 }
2023-11-12T19:52:39.660330Z :  read_thread iteration=13 read_tx=68423 throughput_per_second=83.753 MiB window_bytes=43922211 read_stats=RingBufferStats { committed: 22470710, commits: 34357, dropped: 21451501, drops: 2086, errors: 0, txs: 68424 }
2023-11-12T19:52:40.160459Z :  read_thread iteration=13 read_tx=137104 throughput_per_second=85.794 MiB window_bytes=44984302 read_stats=RingBufferStats { committed: 45298873, commits: 68659, dropped: 43607640, drops: 4200, errors: 0, txs: 137105 }
2023-11-12T19:52:40.160518Z : write_thread iteration=13 write_tx=7108 write_stats=RingBufferStats { committed: 45307132, commits: 5362, dropped: 35940888, drops: 12278, errors: 74410, txs: 81518 }
2023-11-12T19:52:41.063212Z : write_thread iteration=14 write_tx=3640 write_stats=RingBufferStats { committed: 22263424, commits: 2746, dropped: 19372363, drops: 6568, errors: 35293, txs: 38933 }
2023-11-12T19:52:41.063219Z :  read_thread iteration=14 read_tx=68244 throughput_per_second=81.461 MiB window_bytes=42713269 read_stats=RingBufferStats { committed: 22263424, commits: 34078, dropped: 20449845, drops: 2008, errors: 0, txs: 68245 }
2023-11-12T19:52:41.563432Z :  read_thread iteration=14 read_tx=136678 throughput_per_second=85.650 MiB window_bytes=44908555 read_stats=RingBufferStats { committed: 45231269, commits: 68545, dropped: 42390555, drops: 4124, errors: 0, txs: 136679 }
2023-11-12T19:52:41.563487Z : write_thread iteration=14 write_tx=7322 write_stats=RingBufferStats { committed: 45231269, commits: 5512, dropped: 37558159, drops: 13013, errors: 72496, txs: 79818 }
2023-11-12T19:52:42.424823Z : write_thread iteration=15 write_tx=3651 write_stats=RingBufferStats { committed: 22778899, commits: 2763, dropped: 18142657, drops: 6222, errors: 35883, txs: 39534 }
2023-11-12T19:52:42.424823Z :  read_thread iteration=15 read_tx=68339 throughput_per_second=83.384 MiB window_bytes=43725941 read_stats=RingBufferStats { committed: 22778899, commits: 34394, dropped: 20947042, drops: 2032, errors: 0, txs: 68340 }
2023-11-12T19:52:42.924998Z :  read_thread iteration=15 read_tx=137238 throughput_per_second=82.913 MiB window_bytes=43470409 read_stats=RingBufferStats { committed: 45511299, commits: 68431, dropped: 41685051, drops: 4079, errors: 0, txs: 137239 }
2023-11-12T19:52:42.925101Z : write_thread iteration=15 write_tx=7337 write_stats=RingBufferStats { committed: 45511299, commits: 5555, dropped: 36062737, drops: 12508, errors: 72471, txs: 79808 }
2023-11-12T19:52:43.787817Z : write_thread iteration=16 write_tx=3386 write_stats=RingBufferStats { committed: 21102410, commits: 2518, dropped: 19332992, drops: 6863, errors: 42512, txs: 45898 }
2023-11-12T19:52:43.787806Z :  read_thread iteration=16 read_tx=62659 throughput_per_second=78.046 MiB window_bytes=40936188 read_stats=RingBufferStats { committed: 21102410, commits: 31171, dropped: 19833778, drops: 1888, errors: 0, txs: 62660 }
2023-11-12T19:52:44.287940Z : write_thread iteration=16 write_tx=6776 write_stats=RingBufferStats { committed: 41979025, commits: 5038, dropped: 39996296, drops: 14137, errors: 83781, txs: 90557 }
2023-11-12T19:52:44.287997Z :  read_thread iteration=16 read_tx=125639 throughput_per_second=79.435 MiB window_bytes=41646818 read_stats=RingBufferStats { committed: 41979025, commits: 62594, dropped: 40603981, drops: 3904, errors: 0, txs: 125640 }
2023-11-12T19:52:45.258417Z : write_thread iteration=17 write_tx=3687 write_stats=RingBufferStats { committed: 22367971, commits: 2761, dropped: 19158285, drops: 6683, errors: 36786, txs: 40473 }
2023-11-12T19:52:45.258417Z :  read_thread iteration=17 read_tx=67326 throughput_per_second=82.976 MiB window_bytes=43503269 read_stats=RingBufferStats { committed: 22367971, commits: 33452, dropped: 21135298, drops: 2108, errors: 0, txs: 67327 }
2023-11-12T19:52:45.758593Z : write_thread iteration=17 write_tx=7384 write_stats=RingBufferStats { committed: 44717857, commits: 5517, dropped: 38115595, drops: 13137, errors: 73404, txs: 80788 }
2023-11-12T19:52:45.758601Z :  read_thread iteration=17 read_tx=134661 throughput_per_second=80.273 MiB window_bytes=42086189 read_stats=RingBufferStats { committed: 44717857, commits: 67199, dropped: 40871601, drops: 4075, errors: 0, txs: 134662 }
2023-11-12T19:52:46.626783Z : write_thread iteration=18 write_tx=3662 write_stats=RingBufferStats { committed: 22395805, commits: 2737, dropped: 19335345, drops: 6429, errors: 36493, txs: 40155 }
2023-11-12T19:52:46.626877Z :  read_thread iteration=18 read_tx=67454 throughput_per_second=81.111 MiB window_bytes=42538377 read_stats=RingBufferStats { committed: 22395805, commits: 33871, dropped: 20142572, drops: 1939, errors: 0, txs: 67455 }
2023-11-12T19:52:47.127039Z :  read_thread iteration=18 read_tx=135071 throughput_per_second=83.772 MiB window_bytes=43927242 read_stats=RingBufferStats { committed: 45092959, commits: 67693, dropped: 41372660, drops: 3970, errors: 0, txs: 135072 }
2023-11-12T19:52:47.127142Z : write_thread iteration=18 write_tx=7338 write_stats=RingBufferStats { committed: 45105630, commits: 5499, dropped: 37783861, drops: 12934, errors: 74233, txs: 81571 }
2023-11-12T19:52:47.987469Z : write_thread iteration=19 write_tx=3640 write_stats=RingBufferStats { committed: 22286847, commits: 2696, dropped: 18460777, drops: 6324, errors: 37111, txs: 40751 }
2023-11-12T19:52:47.987483Z :  read_thread iteration=19 read_tx=68926 throughput_per_second=84.734 MiB window_bytes=44428050 read_stats=RingBufferStats { committed: 22286847, commits: 34308, dropped: 22141203, drops: 2121, errors: 0, txs: 68927 }
2023-11-12T19:52:48.487699Z : write_thread iteration=19 write_tx=7350 write_stats=RingBufferStats { committed: 44693096, commits: 5455, dropped: 38045762, drops: 13022, errors: 72831, txs: 80181 }
2023-11-12T19:52:48.487795Z :  read_thread iteration=19 read_tx=137412 throughput_per_second=82.442 MiB window_bytes=43235171 read_stats=RingBufferStats { committed: 44693096, commits: 68488, dropped: 42970125, drops: 4129, errors: 0, txs: 137413 }
ok
test reader_close ... 2023-11-12T19:52:48.845387Z : ThreadId(47) the RingBuffer is closed concurrently or corrupted position=0 old=1 header=Written { size: 3 }
2023-11-12T19:52:48.845799Z X ThreadId(47) state=165 position=0 RingBuffer is corrupted by the other size, treating it as closed
2023-11-12T19:52:48.845850Z X ThreadId(47) state=165 position=6 RingBuffer is corrupted by the other size, treating it as closed
2023-11-12T19:52:48.845907Z X ThreadId(47) state=165 position=0 RingBuffer is corrupted by the other size, treating it as closed
2023-11-12T19:52:48.846303Z X ThreadId(47) invalid state transition, closing the RingBuffer position=0 old=165 header=Written { size: 3 }
ok
test sequential ... 2023-11-12T19:52:48.847151Z : ThreadId(49) iteration=0 read_stats=RingBufferStats { committed: 0, commits: 0, dropped: 0, drops: 0, errors: 0, txs: 0 } write_stats=RingBufferStats { committed: 0, commits: 0, dropped: 0, drops: 0, errors: 0, txs: 0 }
2023-11-12T19:52:48.992553Z : ThreadId(49) iteration=1000 read_stats=RingBufferStats { committed: 4777049, commits: 450, dropped: 4565906, drops: 392, errors: 0, txs: 875 } write_stats=RingBufferStats { committed: 4785218, commits: 570, dropped: 2789082, drops: 376, errors: 619, txs: 1363 }
2023-11-12T19:52:49.137571Z : ThreadId(49) iteration=2000 read_stats=RingBufferStats { committed: 9660027, commits: 907, dropped: 9490403, drops: 807, errors: 0, txs: 1789 } write_stats=RingBufferStats { committed: 9660027, commits: 1143, dropped: 5401742, drops: 765, errors: 1283, txs: 2777 }
2023-11-12T19:52:49.279328Z : ThreadId(49) iteration=3000 read_stats=RingBufferStats { committed: 14364668, commits: 1352, dropped: 13764976, drops: 1184, errors: 0, txs: 2658 } write_stats=RingBufferStats { committed: 14375257, commits: 1714, dropped: 8201466, drops: 1195, errors: 1892, txs: 4126 }
2023-11-12T19:52:49.422557Z : ThreadId(49) iteration=4000 read_stats=RingBufferStats { committed: 19098946, commits: 1788, dropped: 18589163, drops: 1612, errors: 0, txs: 3554 } write_stats=RingBufferStats { committed: 19098946, commits: 2307, dropped: 11035985, drops: 1630, errors: 2552, txs: 5550 }
2023-11-12T19:52:49.563438Z : ThreadId(49) iteration=5000 read_stats=RingBufferStats { committed: 23601436, commits: 2209, dropped: 22624900, drops: 1972, errors: 0, txs: 4389 } write_stats=RingBufferStats { committed: 23612241, commits: 2868, dropped: 13984171, drops: 2060, errors: 3137, txs: 6885 }
2023-11-12T19:52:49.705967Z : ThreadId(49) iteration=6000 read_stats=RingBufferStats { committed: 28057233, commits: 2624, dropped: 28004170, drops: 2420, errors: 0, txs: 5299 } write_stats=RingBufferStats { committed: 28073548, commits: 3423, dropped: 16920182, drops: 2508, errors: 3817, txs: 8335 }
2023-11-12T19:52:49.848216Z : ThreadId(49) iteration=7000 read_stats=RingBufferStats { committed: 32660467, commits: 3074, dropped: 32279073, drops: 2792, errors: 0, txs: 6156 } write_stats=RingBufferStats { committed: 32661575, commits: 3994, dropped: 19779259, drops: 2926, errors: 4426, txs: 9696 }
2023-11-12T19:52:49.995969Z : ThreadId(49) iteration=8000 read_stats=RingBufferStats { committed: 37586632, commits: 3531, dropped: 37213399, drops: 3192, errors: 0, txs: 7056 } write_stats=RingBufferStats { committed: 37593446, commits: 4552, dropped: 22194951, drops: 3320, errors: 5066, txs: 11076 }
2023-11-12T19:52:50.136140Z : ThreadId(49) iteration=9000 read_stats=RingBufferStats { committed: 42103182, commits: 3952, dropped: 41570432, drops: 3565, errors: 0, txs: 7892 } write_stats=RingBufferStats { committed: 42113269, commits: 5109, dropped: 24716620, drops: 3697, errors: 5642, txs: 12392 }
ok
test writer_close ... 2023-11-12T19:52:50.269792Z X ThreadId(51) state=165 position=0 RingBuffer is corrupted by the other size, treating it as closed
2023-11-12T19:52:50.269961Z X ThreadId(51) state=165 position=0 RingBuffer is corrupted by the other size, treating it as closed
2023-11-12T19:52:50.270088Z X ThreadId(51) state=165 position=0 RingBuffer is corrupted by the other size, treating it as closed
ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.97s
```

Если вам придётся его отлаживать, стоит поменять в нём константу
```rust
const QUALITY: Quality = Quality::Paranoid;
```
на
```rust
const QUALITY: Quality = Quality::Debuggable;
```
а вызовы `trace!()` на `debug!()`:

```console
...
test concurrent ... 2023-11-12T20:08:42.276650Z : read_thread read commit
2023-11-12T20:08:42.276724Z : read_thread read commit
2023-11-12T20:08:42.276780Z : read_thread read commit
2023-11-12T20:08:42.276827Z :  read_thread read commit
2023-11-12T20:08:42.276819Z : write_thread write operation='o' x 9697, commit chunk_count=20
2023-11-12T20:08:42.276866Z :  read_thread read commit
2023-11-12T20:08:42.277053Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5921, exceeding_object_len: 4358 })
2023-11-12T20:08:42.277117Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 10228 })
2023-11-12T20:08:42.277166Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 8301 })
2023-11-12T20:08:42.276924Z :  read_thread read_tx data='o' x 9697, block_count = 1, total_len = 9697 len=9697
2023-11-12T20:08:42.277233Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5292, exceeding_object_len: 3624 })
2023-11-12T20:08:42.277303Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 3334, exceeding_object_len: 4674 })
2023-11-12T20:08:42.277351Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 6841 })
2023-11-12T20:08:42.277412Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 1113, exceeding_object_len: 9777 })
2023-11-12T20:08:42.277475Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5083, exceeding_object_len: 6168 })
2023-11-12T20:08:42.277522Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 9443 })
2023-11-12T20:08:42.277272Z :  read_thread read_tx data='o' x 9697, block_count = 1, total_len = 9697 len=9697
2023-11-12T20:08:42.277591Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5397, exceeding_object_len: 3839 })
2023-11-12T20:08:42.277672Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 6290, exceeding_object_len: 5389 })
2023-11-12T20:08:42.277737Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 4417, exceeding_object_len: 4514 })
2023-11-12T20:08:42.277786Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 9853 })
2023-11-12T20:08:42.277852Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5107, exceeding_object_len: 2401 })
2023-11-12T20:08:42.277921Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5246, exceeding_object_len: 2010 })
2023-11-12T20:08:42.277678Z :  read_thread read data='o' x 9697, block_count = 1, total_len = 9697 operation='o' x 9697, commit
2023-11-12T20:08:42.277970Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 0, exceeding_object_len: 8507 })
2023-11-12T20:08:42.278141Z :  read_thread read commit
2023-11-12T20:08:42.278135Z : write_thread operation='u' x 12074, commit write_error=RingBuffer(Overflow { capacity: 6680, len: 5329, exceeding_object_len: 2849 })
2023-11-12T20:08:42.278188Z :  read_thread read commit
2023-11-12T20:08:42.278242Z :  read_thread read commit
2023-11-12T20:08:42.278255Z : write_thread write operation='u' x 12074, commit chunk_count=11
2023-11-12T20:08:42.278282Z :  read_thread read commit
2023-11-12T20:08:42.278447Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 3083, exceeding_object_len: 8101 })
2023-11-12T20:08:42.278497Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 13983 })
2023-11-12T20:08:42.278557Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 4056, exceeding_object_len: 626 })
2023-11-12T20:08:42.278616Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 3042, exceeding_object_len: 10973 })
2023-11-12T20:08:42.278325Z :  read_thread read_tx data='u' x 12074, block_count = 1, total_len = 12074 len=12074
2023-11-12T20:08:42.278662Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 6234 })
2023-11-12T20:08:42.278709Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 15406 })
2023-11-12T20:08:42.278770Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 3711, exceeding_object_len: 3261 })
2023-11-12T20:08:42.278829Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 2805, exceeding_object_len: 5407 })
2023-11-12T20:08:42.278876Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 13186 })
2023-11-12T20:08:42.278921Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 14876 })
2023-11-12T20:08:42.278967Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 5486 })
2023-11-12T20:08:42.279012Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 9694 })
2023-11-12T20:08:42.279061Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 7083 })
2023-11-12T20:08:42.278780Z :  read_thread read data='u' x 12074, block_count = 1, total_len = 12074 operation='u' x 12074, commit
2023-11-12T20:08:42.279108Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 0, exceeding_object_len: 4811 })
2023-11-12T20:08:42.279164Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 2449, exceeding_object_len: 7741 })
2023-11-12T20:08:42.279186Z :  read_thread read commit
2023-11-12T20:08:42.279223Z : write_thread operation='E' x 15574, commit write_error=RingBuffer(Overflow { capacity: 4303, len: 3005, exceeding_object_len: 6764 })
2023-11-12T20:08:42.279229Z :  read_thread read commit
2023-11-12T20:08:42.279292Z :  read_thread read commit
2023-11-12T20:08:42.279329Z : write_thread write operation='E' x 15574, commit chunk_count=8
2023-11-12T20:08:42.279336Z :  read_thread read commit
2023-11-12T20:08:42.279512Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 866 })
2023-11-12T20:08:42.279560Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 7641 })
2023-11-12T20:08:42.279606Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 7534 })
2023-11-12T20:08:42.279650Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 14725 })
2023-11-12T20:08:42.279696Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 2136 })
2023-11-12T20:08:42.279742Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 1663 })
2023-11-12T20:08:42.279789Z : write_thread operation='a' x 16369, commit write_error=RingBuffer(Overflow { capacity: 803, len: 0, exceeding_object_len: 14790 })
2023-11-12T20:08:42.279402Z :  read_thread read_tx data='E' x 15574, block_count = 1, total_len = 15574 len=15574
...
```


### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/ipc/pipe.rs  | 234 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 1 file changed, 214 insertions(+), 20 deletions(-)
```
