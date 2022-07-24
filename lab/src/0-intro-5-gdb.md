## Запуск, в том числе с отладчиком gdb

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
