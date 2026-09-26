## Запуск ядра с отладчиком `gdb`

При запуске Nikka в эмуляторе `qemu`,
то есть когда вы запускаете `cargo run` или `cargo t` в корне репозитория,
можно подключить `gdb` к `qemu`.
Для автоматического запуска `gdb` добавьте опцию `--gdb`:
```console
nikka$ cargo t --test 1-hw-1-rtc --gdb
...
Nikka kernel binary: /.../target/x86_64-unknown-none/ci/deps/artifact/kernel-7cbfb4968139e3f7/bin/kernel-7cbfb4968139e3f7
Running: Command { std: "qemu-system-x86_64" "-cpu" "max,+x2apic" "-m" "size=256M" "-smp" "cpus=4" "-device" "isa-debug-exit,iobase=0xF4,iosize=0x04" "-drive" "if=pflash,format=raw,readonly=on,file=target/ovmf/x64/code.fd" "-drive" "if=pflash,format=raw,file=target/ovmf/x64/vars.fd" "-drive" "file=/.../target/ci/build/nikka-18bed410035f421a/out/uefi.img,index=0,media=disk,format=raw" "-drive" "file=fs.img,index=1,media=disk,format=raw" "-drive" "file=test-config.img,index=2,media=disk,format=raw" "-nographic" "-gdb" "tcp::1234" "-S", kill_on_drop: false }
GNU gdb (GDB) 17.2
...
Breakpoint 1 at 0x21618c: file kernel/src/main.rs, line 45.
(gdb) continue
...
INFO : Create bootinfo
INFO : Jumping to kernel entry point at VirtAddr(0x73f0910)
Thread 1 hit Breakpoint 1, kernel::kernel_main (boot_info=0x28000000000) at kernel/src/main.rs:45
45      #[cfg_attr(not(feature = "conservative-backtraces"), with_sentinel_frame)]
(gdb)
```

Для `cargo run` нужно ещё указать `--`, чтобы `cargo` понял, что его флаги закончились
и дальше идут флаги запускаемой программы:
```console
nikka$ cargo run -- --gdb
...
Nikka kernel binary: /.../target/x86_64-unknown-none/debug/deps/artifact/kernel-f16de538b6bd256f/bin/kernel-f16de538b6bd256f
Running: Command { std: "qemu-system-x86_64" "-cpu" "max,+x2apic" "-m" "size=256M" "-smp" "cpus=4" "-device" "isa-debug-exit,iobase=0xF4,iosize=0x04" "-drive" "if=pflash,format=raw,readonly=on,file=target/ovmf/x64/code.fd" "-drive" "if=pflash,format=raw,file=target/ovmf/x64/vars.fd" "-drive" "file=/.../target/debug/build/nikka-acb172b60b785965/out/uefi.img,index=0,media=disk,format=raw" "-drive" "file=fs.img,index=1,media=disk,format=raw" "-serial" "stdio" "-gdb" "tcp::1234" "-S", kill_on_drop: false }
GNU gdb (GDB) 17.2
...
Breakpoint 1 at 0x21618c: file kernel/src/main.rs, line 45.
(gdb) continue
0x000000000000fff0 in ?? ()
...
INFO : Create bootinfo
INFO : Jumping to kernel entry point at VirtAddr(0x2177d0)

Thread 1 hit Breakpoint 1, kernel::kernel_main (
    boot_info=0x28000000000) at kernel/src/main.rs:45
45      #[cfg_attr(not(feature = "conservative-backtraces"), with_sentinel_frame)]
(gdb)
```

При автоматическом запуске скрипт запуска
[`src/main.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/src/main.rs)
берёт начальные команды для `gdb` из файла
[`gdbinit`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/gdbinit).
Вы можете настроить его по своему вкусу.

Подключить `gdb` можно и вручную.
Удобнее всего взять печатаемую команду запуска `qemu` и путь к бинарнику ядра из строк
```console
nikka$ cargo r
...
Nikka kernel binary: /.../target/x86_64-unknown-none/debug/deps/artifact/kernel-f16de538b6bd256f/bin/kernel-f16de538b6bd256f
Running: Command { std: "qemu-system-x86_64" "-cpu" "max,+x2apic" "-m" "size=256M" "-smp" "cpus=4" "-device" "isa-debug-exit,iobase=0xF4,iosize=0x04" "-drive" "if=pflash,format=raw,readonly=on,file=target/ovmf/x64/code.fd" "-drive" "if=pflash,format=raw,file=target/ovmf/x64/vars.fd" "-drive" "file=/.../target/debug/build/nikka-acb172b60b785965/out/uefi.img,index=0,media=disk,format=raw" "-drive" "file=fs.img,index=1,media=disk,format=raw" "-serial" "stdio", kill_on_drop: false }
...
```
Дальше нужно изменить по своему усмотрению команду для `qemu`, и запустить её, добавив опции `-gdb tcp::1234 -S`.
В другой консоли нужно запустить `gdb` и подключить его к `qemu` командой `target remote localhost:1234`.
Альтернативно, можно локально поправить опции `qemu` в файле
[`src/main.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/src/main.rs).

Также можно [запустить под gdb и отдельный тест](../../lab/book/1-hw-3-high-resolution-date.html#%D0%98%D0%BB%D0%B8-%D0%BC%D0%BE%D0%B6%D0%BD%D0%BE-%D0%B7%D0%B0%D0%BF%D1%83%D1%81%D1%82%D0%B8%D1%82%D1%8C-%D1%82%D0%B5%D1%81%D1%82-%D0%BF%D0%BE%D0%B4-gdb).


### Отладка пользовательских процессов с gdb

При необходимости отлаживать пользовательский процесс,
добавьте опцию `--exe` с бинарником отлаживаемого процесса.
Путь к бинарнику можно подсмотреть в тесте,
но нужно учитывать что он относительный от директории с самим тестом.

```console
nikka$ grep eager-fork kernel/src/tests/*.rs
kernel/src/tests/lab_7_um_3_eager_fork.rs:const EAGER_FORK_ELF: &[u8] = page_aligned!("../../../target/kernel/user/eager-fork");
nikka$ cargo t --test 7-um-3-eager-fork --gdb --exe target/kernel/user/eager-fork
...
Function "kernel::kernel_main" not defined.
Make breakpoint pending on future shared library load? (y or [n]) [answered N; input not from terminal]
(gdb) break eager_fork::main
Breakpoint 1 at 0x10000000c798: file user/eager_fork/src/main.rs, line 40.
(gdb) continue
Continuing.
...
Thread 1 hit Breakpoint 2, eager_fork::main () at user/eager_fork/src/main.rs:40
40         let mut pedigree = Vec::<Pid, DEPTH>::new();
(gdb) next
42          let mut name = String::<MAX_NAME>::new();
(gdb) n
43          name.push_str("eager_fork ").unwrap();
(gdb) n
19:33:09.005 0 D leaving the user mode; pid = 2:1
19:33:09.017 0 I the process was preempted; pid = 2:1; user_context = { mode: user, cs:rip: 0x0023:0v1000_0000_DC60, ss:rsp: 0x001B:0v7FFF_FFFF_4F30, rflags: RF IF PF }
19:33:09.055 0 I dequeue; pid = Some(2:1)
19:33:09.067 0 I switch to; address_space = "2:1" @ 0pF8C_D000
19:33:09.081 0 D entering the user mode; pid = 2:1; registers = { rax: 0x7FFFFFFF4F58, rdi: 0x7FFFFFFF4F58, rsi: 0x10000000270B, { mode: user, cs:rip: 0x0023:0v1000_0000_DC60, ss:rsp: 0x001B:0v7FFF_FFFF_4F30, rflags: RF IF PF } }
45          fork_tree(&mut pedigree, &mut name, '*');
(gdb)
```


### Дизассемблирование

Для получения информации о бинарниках и для их дизассемблирования можно использовать `objdump` в сочетании с `rustfilt`.
Например:
```console
$ objdump --all-headers target/kernel/debug/kernel | rustfilt
$ objdump --disassemble --disassembler-options=intel --source target/kernel/debug/kernel | rustfilt
$ objdump --all-headers target/kernel/user/cow_fork | rustfilt
$ objdump --disassemble --disassembler-options=intel --source target/kernel/user/cow_fork | rustfilt
```

Для запуска `rustfilt` и других бинарников Rust полезно добавить `$HOME/.cargo/bin/` в свой `$PATH`:
```console
$ tail -n1 ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```


### Трассировка стека

При паниках, по возможности, печатается трассировка стека в виде списка адресов возврата.
Также, по возможности, в него добавляются имена соответствующих функций и номера строк кода:
```console
19:58:56 ? I RTC init; acknowledged_settings = RegisterB(USE_24_HOUR_FORMAT | UPDATE_ENDED_INTERRUPT)
panicked at kernel/src/time/mod.rs:39:5:
I did my best, it wasn't much
Backtrace:
  0x74BC872 ?? at ??:0
  0x75E022C ?? at ??:0
  0x7583235 ?? at ??:0
  0x74BBE4A kernel::preinit at kernel/src/lib.rs:171
  0x749BB0C kernel::tests::tests_main at kernel/src/tests/mod.rs:130
  0x749CD18 kernel::tests::main at kernel/src/tests/mod.rs:124
  0x73EF775 ?? at ??:0
  0x73EF74D kernel::kernel_main at kernel/src/main.rs:45
```
Для этого используется `llvm-symbolizer`.
Поэтому, если он не установлен в системе, имён функций и номеров строк кода не выводится:
```console
21:02:43 ? I RTC init; acknowledged_settings = RegisterB(USE_24_HOUR_FORMAT | UPDATE_ENDED_INTERRUPT)
panicked at kernel/src/time/mod.rs:39:5:
I did my best, it wasn't much
Backtrace:
  0x74BC872 failed to run llvm-symbolizer, maybe it is not installed: No such file or directory (os error 2)
  0x75E022C
  0x7583235
  0x74BBE4A
  0x749BB0C
  0x749CD18
  0x73EF775
  0x73EF74D
```
Тогда нужно установить `llvm-symbolizer`.

При автоматической символизации в качестве файла с отладочной информацией используется
бинарный образ ядра.
Если трассировка стека относится к пользовательской программе,
нужно указать путь к бинарному файлу пользовательской программы с отладочной информацией
в опции `--exe`:
```console
~/nikka$ cargo t --test 7-um-3-eager-fork --exe target/kernel/user/eager-fork
```

Также символизацию нужно сделать вручную.
Например:
```console
~/nikka$ cd kernel
~/nikka/kernel$ cargo test --test 6-um-3-eager-fork
...
22:16:28.397 0 E panicked at user/eager_fork/src/main.rs:47:5:
I did my best, it wasn't much; backtrace = 0x10007BFD 0x10007AA7 0x10007C18 0x1000825D; pid = 2:1
```
Его можно расшифровать с помощью `llvm-symbolizer`:
```console
/.../nikka/kernel$ echo '0x10007BFD 0x10007AA7 0x10007C18 0x1000825D' | tr -d ',_' | tr 'v ' 'x\n' | llvm-symbolizer --exe ../target/kernel/user/eager_fork
??
??:0:0

eager_fork::main::hbb74cf0a448b5219
/.../nikka/user/eager_fork/src/main.rs:40:1

main
/.../nikka/user/lib/src/lib.rs:115:10

_start
/.../nikka/user/lib/src/lib.rs:69:19
```
Или же можно использовать утилиту `addr2line`:
```console
/.../nikka/kernel$ echo '0x10007BFD 0x10007AA7 0x10007C18 0x1000825D' | tr -d ',_' | tr 'v ' 'x\n' | addr2line --exe=../target/kernel/user/eager_fork --pretty-print --demangle
??:0
/.../nikka/user/eager_fork/src/main.rs:40
/.../nikka/user/lib/src/lib.rs:115
/.../nikka/user/lib/src/lib.rs:69
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

Также можно воспользоваться структурой
[`ku::backtrace::callsite::Callsite`](../../doc/ku/backtrace/callsite/struct.Callsite.html).
Кроме backtrace она позволяет сохранить ссылку на строку в коде ---
[`core::panic::Location`](https://doc.rust-lang.org/nightly/core/panic/struct.Location.html).
Например, так:
```rust
#[track_caller]
fn foo(...) {
    ...
    let callsite = Callsite::new(Location::caller());
    ...
}
```
И при печати будет показывать и номер строки в коде, из которой вызывается функция `foo()`, и ведущий туда backtrace:
```console
ku/tests/5-sync-2-spinlock.rs:86:25, backtrace: [0x4332DF 0x423B65 0x42FA55 0x411205 0x421A85 0x408F87 0x40943D 0x408B22 0x41BD55 0x40F845 0x41DA71 0x50F3D3]
```
