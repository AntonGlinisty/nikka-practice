## Запуск с отладчиком gdb

Если что-то пошло не так, для отладки можно запустить

```console
$ make run-gdb
```

Запустится эмулятор qemu, который будет ждать подключения `gdb`.
В другой консоли запустите `gdb` командой

```console
$ make gdb
```

`gdb` подключится к `qemu`.
Теперь можно, например, поставить `breakpoint`:

```console
...
0x000000000000fff0 in ?? ()
(gdb) br kernel_main
Breakpoint 1 at 0x21141f: file src/main.rs, line 36.
```

и запустить Nikka

```console
(gdb) c
Continuing.

Thread 1 hit Breakpoint 1, kernel::kernel_main (boot_info=0x10000000000) at src/main.rs:36
36    kernel::init(boot_info);
```

Запустить `qemu` без `gdb` можно командой

```console
$ make run
```

А для прогона тестов

```console
$ make test
```


### Отладка пользовательских процессов с gdb

При необходимости отлаживать пользовательский процесс,
нужно сделать несколько дополнительных вещей:

- Отредактировать файл [`gdbinit`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/gdbinit), заменив в нём команду загрузки бинарника ядра `file target/kernel/debug/kernel` на команду загрузки нужного пользовательского бинарника, например `file target/kernel/user/cow_fork`.

- Можно также добавить в `gdbinit` дополнительные команды.
    Например,

    ```gdb
    breakpoint lib::syscall::trap_handler_invoker
    ignore 1 10
    continue
    ```

    добавит точку останова на заданную функцию и пропустит первые 10 попаданий на неё.
    То есть приглашение gdb будет показано на 11-е попадание в точку останова.

- Запустить `make test-gdb` в одной консоли и несколько раз `make gdb` в другой консоли, пока не запустится нужный тестовый процесс.


### Дизассемблирование

Для получения информации о бинарниках и для их дизассемблирования можно использовать `objdump` в сочетании с `rustfilt`.
Например:
```console
$ objdump --all-headers target/kernel/debug/kernel | rustfilt
$ objdump --disassemble --disassembler-options=intel target/kernel/debug/kernel | rustfilt
$ objdump --all-headers target/kernel/user/cow_fork | rustfilt
$ objdump --disassemble --disassembler-options=intel target/kernel/user/cow_fork | rustfilt
```

Для запуска `rustfilt` и других бинарников Rust полезно добавить `$HOME/.cargo/bin/` в свой `$PATH`:
```console
$ tail -n1 ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```


### Трассировка стека

При паниках, по возможности, печатается backtrace.
Например:
```console
~/nikka$ cd kernel
~/nikka/kernel$ cargo test --test 6-um-3-eager-fork
...
23:02:48.579 0 E panicked at 'I did my best, it wasn't much', user/eager_fork/src/main.rs:47:5; backtrace = 0x10008593 0x10008479 0x100085B8 0x10008CD9; pid = 2:1
```
Его можно расшифровать с помощью `llvm-symbolizer`:
```console
~/nikka/kernel$ echo '0x10008593 0x10008479 0x100085B8 0x10008CD9' | tr ' ' '\n' | llvm-symbolizer --exe ../target/kernel/user/eager_fork
eager_fork::fork_tree::h6af1cf920637510d
/.../nikka/user/eager_fork/src/main.rs:47:5

eager_fork::main::h7eeb3104124da585
/.../nikka/user/eager_fork/src/main.rs:40:1

main
/.../nikka/user/lib/src/lib.rs:88:10

_start
/.../nikka/user/lib/src/lib.rs:60:19
```

Если вы хотите добавить журналирование backtrace, это можно сделать с помощью
[`ku::backtrace::Backtrace`](../../doc/ku/backtrace/struct.Backtrace.html):
```rust
if let Ok(backtrace) = Backtrace::current() {
    debug!(%backtrace);
}
```
Если при этом вы сталкиваетесь с оборванным backtrace,
скорее всего при его вычислении по стековым фреймам,
произошёл выход за границы страницы памяти, в которой находился стартовый стековый фрейм.
Обрыв backtrace при этом --- защитная мера от возникновения ошибок обращения к памяти --- PageFault.
В таком случае можно явно указать
[`Backtrace`](../../doc/ku/backtrace/struct.Backtrace.html)
допустимую область стека.
Например, в коде пользовательских приложений можно сделать так:
```rust
if let Ok(backtrace) = Backtrace::with_stack(ku::process_info().stack()) {
    debug!(%backtrace);
}
```
Так как ядро предоставляет пользовательскому приложению информацию о выделенном для него стеке ---
`ku::process_info().stack()`.

Также можно ограничить глубину backtrace.
[`Backtrace`](../../doc/ku/backtrace/struct.Backtrace.html)
реализует типаж
[`Iterator`](https://doc.rust-lang.org/nightly/core/iter/trait.Iterator.html).
То есть, можно ограничить его глубину, например так:
```rust
if let Ok(backtrace) = Backtrace::with_stack(ku::process_info().stack()) {
    for stack_frame in backtrace.take(3) {
        debug!(%stack_frame);
    }
}
```

Кроме того, можно распечатать backtrace через типаж
[`Debug`](https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html),
тогда переводить пробелы в переносы строк для `llvm-symbolizer` не потребуется:
```console
panicked at 'I did my best, it wasn't much', kernel/src/smp/ap_init.rs:58:5
Backtrace:
  0x27363A
  0x28F1A3
  0x28CE1A
  0x228065
  0x216648
  0x21830C
  0x9C81
```
