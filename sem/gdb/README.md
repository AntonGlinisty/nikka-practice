# `gdb`

В директории задачи находится исполняемый файл `binary` (Убедитесь что он последней версии!).

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

0. Запишите в файл `key.txt` вашу гипотезу.

```
echo -n 1234asdf > key.txt
```

1. Установите плагин `peda` к `gdb`.

```sh
git clone https://github.com/longld/peda.git ~/peda
echo "source ~/peda/peda.py" >> ~/.gdbinit
echo "set disassembly-flavor intel" >> ~/.gdbinit
echo "set print asm-demangle on" >> ~/.gdbinit
echo "DONE! debug your program with gdb and enjoy"
```

2. Запустите `gdb`.

```
gdb binary
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
