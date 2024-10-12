# `gdb`

В директории задачи находится исполняемый файл `binary`.
Убедитесь что он последней версии, то есть что у вас актуальная копия этого репозитория: `git pull --rebase`.

Исходный код функции `main()` вам дан:

```rust
fn main() -> Result<(), Error> {
    let key = fs::read_to_string("key.txt")?;

    if check_key(&key.as_bytes()) {
        println!("OK");

        Ok(())
    } else {
        Err(Error::from(InvalidData))
    }
}
```

Исходный код функции `check_key()` вам неизвестен.
Но и писать её реализацию не требуется.
Хотя это тоже хорошее дополнительное упражнение.

Вам нужно понять, какой ключ нужно подать на вход исполняемому файлу,
чтобы он напечатал `OK`.

Чтобы проверить гипотезу, запишите её в файл `key.txt`.

```
echo -n 1234asdf > key.txt
```

Чтобы придумать гипотезу, выполните следующие шаги.

1. Установите плагин `peda` к `gdb`.

`Peda` --- это плагин для `gdb`, который наглядно показывает состояния регистров,
кода, стека и т.д. внутри сессии `gdb`.

https://github.com/longld/peda

```sh
git clone https://github.com/longld/peda.git ~/peda
echo "source ~/peda/peda.py" >> ~/.gdbinit
echo "set disassembly-flavor intel" >> ~/.gdbinit
echo "set print asm-demangle on" >> ~/.gdbinit
echo "DONE! debug your program with gdb and enjoy"
```

2. Запустите `gdb`.

Проверьте наличие динамических библиотек:

```
$ ldd binary
# должно выводить примерно следующее:
    linux-vdso.so.1 (0x00007ffe2a0ef000)
    libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007478123a3000)
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x0000747812000000)
    /lib64/ld-linux-x86-64.so.2 (0x00007478123d8000)
```

Если выводит `... => not found`, то какой-то библиотеки нет:

```
$ ldd binary
    linux-vdso.so.1 (0x00007fff9077b000)
    libgcc_s.so.1 => not found
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x000074b24c800000)
    /lib64/ld-linux-x86-64.so.2 (0x000074b24cb40000)
```

А `gdb` выдаст сообщение об ошибке:

```
$ gdb binary
gdb: error while loading shared libraries: libgcc_s.so.1: cannot open shared object file: No such file or directory
```

Если все библиотеки на месте:

```
gdb binary
```

Если у вас нет нужных динамических библиотек, можно анализировать статически собранный бинарник:

```
gdb binary.static
```

Если выйдет множество ошибок `SyntaxWarning` от `peda`, игнорируйте их.

Если возникнет ошибка `Error occurred in Python: No module named 'six.moves'`,
[выполните команды](https://github.com/longld/peda/issues/182#issuecomment-2317746602):

```
rm ~/peda/lib/six.py
sudo apt install python3-six
```

Приглашение корректно запущенного `gdb` с `peda` должен выглядеть так, подсвеченный красным:

```
gdb-peda$
```

3. Поставьте `breakpoint` на функцию `check_key()` и запустите программу.

```
b gdb::check_key
run
```

или

```
b check_key
run
```

Должна вывестись цветная табличка от `peda` с регистрами, кодом и стеком.

Если `gdb` ломается с ошибкой использования `libthread_db`

```
gdb-peda$ run
Starting program: /.../nikka/sem/gdb/binary.static
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".
Cannot find user-level thread for LWP 192000: debugger service failed
```

можно изменить путь поиска этой библиотеки, чтобы `gdb` её не нашёл:

```
gdb-peda$ set libthread-db-search-path /
gdb-peda$ run
```

Также можно попробовать использовать `lldb` вместо `gdb`.

4. Пошагайте по инструкциям и попробуйте понять что происходит.

```
ni
```

Или если хотите заходить в вызываемые функции:

```
si
```

Быстро дойти до конца функции можно командой

```
finish
```
