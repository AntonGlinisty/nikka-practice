## Загрузка процесса в память

У нас ещё нет файловой системы.
Поэтому в тестах код пользовательских программ линкуется прямо в бинарник ядра макросом
[`core::include_bytes!()`](https://doc.rust-lang.org/core/macro.include_bytes.html):

```rust
const LOOP_ELF: &[u8] = include_bytes!("../../../user/loop/target/kernel/debug/loop");
process::create(LOOP_ELF);
```

Первой нашей задачей будет распарсить
[ELF--файл](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
с помощью библиотеки
[`xmas_elf`](../../doc/xmas_elf/index.html) и построить его образ в памяти.
Нам достаточно [простейшей реализации](https://wiki.osdev.org/ELF#Loading_ELF_Binaries),
которая поддерживает только статические ELF--файлы,
без [релокаций](https://wiki.osdev.org/ELF#Relocation), обработки
[секций](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#Section_header) и символов.
Основной является функция
[`elf::load()`](../../doc/ku/process/elf/fn.load.html)
в файле
[`ku/src/process/elf.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/process/elf.rs).

{{#include ../../doc/ku/process/elf/fn.load.html.fn}}

Она принимает на вход

- Аллокатор `allocator`, реализующий типаж [`ku::allocator::big::BigAllocatorPair`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html). Вызывающая её функция гарантирует, что код работает в текущем адресном пространстве [`BigAllocatorPair::src()`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html#tymethod.src). А загружать процесс нужно в целевое адресное пространство [`BigAllocatorPair::dst()`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html#tymethod.dst). Вызывающая функция должна настроить аллокатор так, чтобы `dst().flags()` возвращал флаг доступа для пользователя, если он нужен. А также любые другие дополнительные флаги, которые не задаёт сам ELF--файл. И чтобы `src().flags()` возвращал корректные флаги для текущего кода в текущем адресном пространстве.
- Срез `file` с записанным в памяти ELF--файлом процесса.

Из `file` она создаёт объект [`xmas_elf::ElfFile`](../../doc/xmas_elf/struct.ElfFile.html),
проходится по его
[сегментам](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#Program_header) итератором
[`ElfFile::program_iter()`](../../doc/xmas_elf/struct.ElfFile.html#method.program_iter)
и загружает в память те из них, у которых тип
[`ProgramHeader::get_type()`](../../doc/xmas_elf/program/enum.ProgramHeader.html#method.get_type)
является загружаемым ---
[`Type::Load`](../../doc/xmas_elf/program/enum.Type.html#variant.Load).
После загрузки она возвращает точку входа в загруженную программу
[`HeaderPt2::entry_point()`](../../doc/xmas_elf/header/enum.HeaderPt2.html#method.entry_point)
в виде виртуального адреса
[`Virt`](../../doc/ku/memory/addr/type.Virt.html).


### Типаж [`ku::allocator::big_pair::BigAllocatorPair`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html)

Этот типаж объединяет два аллокатора
[`ku::allocator::big::BigAllocator`](../../doc/ku/allocator/big/trait.BigAllocator.html).
В общем случае, они отвечают за память разных адресных пространств
или выделяют память в разных участках адресного пространства.
Например, один --- в зарезервированном для ядра, другой --- в зарезервированном для пользователя.
Типаж
[`BigAllocatorPair`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html)
позволяет переносить отображение страниц из одного адресного пространства в другое методом
[`BigAllocatorPair::copy_mapping()`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html#tymethod.copy_mapping).
Реализует этот типаж структура
[`kernel::allocator::big_pair::BigPair`](../../doc/kernel/allocator/big_pair/struct.BigPair.html).
Её можно создать как для двух разных адресных пространств методом
[`BigPair::new_pair()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.new_pair).
Так и для одного адресного пространства методом
[`BigPair::new_single()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.new_single).
В обоих случаях, для текущего аллокатора
[`BigPair::src()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.src)
можно указать один набор флагов, а для целевого аллокатора
[`BigPair::dst()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.dst) ---
другой набор флагов.

Например, можно использовать
[`BigPair::src()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.src)
для выделения страниц в области данных ядра.
И переносить их в то же или другое целевое адресное пространство
методом
[`BigPair::copy_mapping()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.copy_mapping).
Где ими будет заведовать
[`BigPair::dst()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.dst),
например, уже в области данных пользователя.
Это нам пригодится для загрузки кодом ядра ELF--файла
в адресное пространство пользовательского процесса.
И ровно тот же код, работая в одном пользовательском процессе, сможет загружать ELF--файл
в адресное пространство другого пользовательского процесса.

Перенесите свою реализацию метода
[`BigAllocator::copy_mapping()`](../../doc/ku/allocator/big/trait.BigAllocator.html#tymethod.copy_mapping)
для структуры
[`Big`](../../doc/kernel/allocator/big/struct.Big.html)
из файла
[`kernel/src/allocator/big.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/allocator/big.rs)
в файл
[`kernel/src/allocator/big_pair.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/allocator/big_pair.rs)
в реализацию метода
[`BigAllocatorPair::copy_mapping()`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html#tymethod.copy_mapping)
для структуры
[`BigPair`](../../doc/kernel/allocator/big_pair/struct.BigPair.html).
И поправьте её с учётом того, что исходный блок `src_block` находится в текущем адресном пространстве,
получить которое можно методом
[`BigPair::src_address_space()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.src_address_space).
А целевой блок `dst_block` находится потенциально в другом, целевом адресном пространстве,
получить которое можно методом
[`BigPair::dst_address_space()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.dst_address_space).

В файле
[`kernel/src/allocator/big.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/allocator/big.rs)
в качестве реализации метода
[`Big::copy_mapping()`](../../doc/kernel/allocator/big/struct.Big.html#method.copy_mapping)
можно сделать перенаправление на
[`BigPair::copy_mapping()`](../../doc/kernel/allocator/big_pair/struct.BigPair.html#method.copy_mapping):
```rust
unsafe fn copy_mapping(
    &mut self,
    old_block: Block<Page>,
    new_block: Block<Page>,
    flags: Option<PageTableFlags>,
) -> Result<()> {
    let allocator_flags = self.flags();
    let mut allocator =
        BigPair::new_single(self.address_space, allocator_flags, allocator_flags);

    unsafe { allocator.copy_mapping(old_block, new_block, flags) }
}
```


### Методы [`BigAllocator::grow()`](../../doc/ku/allocator/big/trait.BigAllocator.html#method.grow) и [`BigAllocator::shrink()`](../../doc/ku/allocator/big/trait.BigAllocator.html#method.shrink)

Вам может пригодиться метод
[`BigAllocator::grow()`](../../doc/ku/allocator/big/trait.BigAllocator.html#method.grow)
в файле
[`ku/src/allocator/big.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/allocator/big.rs).

{{#include ../../doc/ku/allocator/big/trait.BigAllocator.html.method.grow}}

Его реализацию вы можете выделить из вашей реализации метода
[`ku::allocator::dry::DryAllocator::dry_grow()`](../../doc/ku/allocator/dry/trait.DryAllocator.html#tymethod.dry_grow)
для типажа
[`BigAllocator`](../../doc/ku/allocator/big/trait.BigAllocator.html).

Также есть и парный метод
[`BigAllocator::shrink()`](../../doc/ku/allocator/big/trait.BigAllocator.html#method.shrink).

{{#include ../../doc/ku/allocator/big/trait.BigAllocator.html.method.shrink}}

Реализацию которого вы можете выделить из вашей реализации метода
[`ku::allocator::dry::DryAllocator::dry_shrink()`](../../doc/ku/allocator/dry/trait.DryAllocator.html#tymethod.dry_shrink)
для типажа
[`BigAllocator`](../../doc/ku/allocator/big/trait.BigAllocator.html).


### Вспомогательные структуры и функции

Прочитайте документацию ко всем
вспомогательным структурам и функциям в файле
[`ku/src/process/elf.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/process/elf.rs).
Вам придётся их реализовывать и использовать.

Основной является функция [`ku::process::elf::load()`](../../doc/ku/process/elf/fn.load.html),
которая загружает сегменты ELF--файла по одному,
фактически проходя по ним
[методом сканирующей прямой](https://ru.algorithmica.org/cs/decomposition/scanline/).
Событиями, которые нужно при этом обрабатывать, являются:
- Границы сегментов ELF--файла.
- Выровненные на границы страниц границы сегментов.

System V Application Binary Interface - DRAFT - 24 April 2001 [утверждает что](https://refspecs.linuxbase.org/elf/gabi4+/ch5.pheader.html):

> `PT_LOAD`
> The array element specifies a loadable segment, described by `p_filesz` and `p_memsz`. The bytes from the file are mapped to the beginning of the memory segment. If the segment's memory size (`p_memsz`) is larger than the file size (`p_filesz`), the "extra" bytes are defined to hold the value `0` and to follow the segment's initialized area. The file size may not be larger than the memory size. Loadable segment entries in the program header table appear in ascending order, sorted on the `p_vaddr` member.

Поэтому можно полагаться на то что сегменты файла не пересекаются в памяти и расположены в таблице сегментов
ELF--файла в порядке адресов в памяти.
Если это не так, можно вернуть ошибку
[`Error::InvalidArgument`](../../doc/kernel/error/enum.Error.html#variant.InvalidArgument).

Трудности создаёт то, что границы сегментов могут быть не выровнены по границам страниц.
А разные сегменты могут иметь разные флаги, с которыми ELF--файл предписывает их отображать ---
чтение, запись, выполнение кода.
Но страничное отображение позволяет выставлять флаги только постранично.
Поэтому для страниц, в которые попадают несколько сегментов,
придётся установить такие флаги, что они удовлетворяют требованиям всех этих сегментов.

Нам придётся отслеживать пару диапазонов адресов --- `curr` и `next`,
каждый со своими флагами.
Для их хранения используется структура
[`ku::process::elf::FileRange`](../../doc/ku/process/elf/struct.FileRange.html).

{{#include ../../doc/ku/process/elf/struct.FileRange.html.declaration}}

А также её часть, связанная только с памятью ---
[`FileRange::virt_range`](../../doc/ku/process/elf/struct.FileRange.html#structfield.virt_range).

Создаётся
[`FileRange`](../../doc/ku/process/elf/struct.FileRange.html)
изначально для отдельного сегмента ELF--файла
[`xmas_elf::program::ProgramHeader`](../../doc/xmas_elf/program/enum.ProgramHeader.html).
Но в процессе загрузки ELF--файла может быть как объединена со смежными диапазонами,
так и разделена на части типа
[`ku::process::elf::PageRange`](../../doc/ku/process/elf/type.PageRange.html)
и
[`ku::process::elf::VirtRange`](../../doc/ku/process/elf/type.VirtRange.html).

Структура [`ku::process::elf::PageRange`](../../doc/ku/process/elf/type.PageRange.html)
описывает целый диапазон страниц,
который уже не будет пересекаться с последующими сегментами.
Поэтому требующиеся для него флаги отображения уже вычислены.
И для этого диапазона уже можно установить в страничном отображении финальные флаги доступа.

Типы
[`VirtRange`](../../doc/ku/process/elf/type.VirtRange.html)
и
[`PageRange`](../../doc/ku/process/elf/type.PageRange.html) ---
мономорфизации типа
[`ku::process::elf::MemoryRange`](../../doc/ku/process/elf/struct.MemoryRange.html)
для
[`Virt`](../../doc/ku/memory/addr/type.Virt.html)
и
[`Page`](../../doc/ku/memory/frage/type.Page.html)
соответственно.

{{#include ../../doc/ku/process/elf/struct.MemoryRange.html.declaration}}

Основная функция, которая объединяет или разбивает соседние диапазоны
`curr` и `next` ---
[`ku::process::elf::combine()`](../../doc/ku/process/elf/fn.combine.html).

{{#include ../../doc/ku/process/elf/fn.combine.html.fn}}

Метод
[`ku::process::elf::Loader::copy_to_memory()`](../../doc/ku/process/elf/struct.Loader.html#method.copy_to_memory)
копирует байты сегмента из ELF--файла в память.

{{#include ../../doc/ku/process/elf/struct.Loader.html.method.copy_to_memory}}

Обратите внимание на то, что размер сегмента в файле
[`ProgramHeader::file_size()`](../../doc/xmas_elf/program/enum.ProgramHeader.html#method.file_size)
может быть меньше чем его размер в памяти
[`ProgramHeader::mem_size()`](../../doc/xmas_elf/program/enum.ProgramHeader.html#method.mem_size).
Тогда дополнительные байты памяти нужно занулить.
Этого требует формат ELF --- там может, например, располагаться секция
[`.bss`](https://en.wikipedia.org/wiki/.bss),
предназначенная для неинициализированных или инициализированных нулями статических переменных.

Занулять байты имеет смысл не в методе
[`ku::process::elf::Loader::copy_to_memory()`](../../doc/ku/process/elf/struct.Loader.html#method.copy_to_memory),
а раньше, в методе
[`ku::process::elf::Loader::extend_mapping()`](../../doc/ku/process/elf/struct.Loader.html#method.extend_mapping) ---
сразу после выделения страниц.
Так как сегменты могут не покрывать выделяемые страницы полностью,
и если оставить мусор в не относящихся к сегментам местах,
в процесс пользователя может попасть закрытая от него информация.

После загрузки последнего сегмента ELF--файла методом
[`Loader::load_program_header()`](../../doc/ku/process/elf/struct.Loader.html#method.load_program_header),
функция
[`load()`](../../doc/ku/process/elf/fn.load.html)
должна вызывать метод
[`Loader::finish()`](../../doc/ku/process/elf/struct.Loader.html#method.finish).

{{#include ../../doc/ku/process/elf/struct.Loader.html.method.finish}}


### Структура загрузчика ELF--файла

Для загрузки файла используется вспомогательная структура
[`ku::process::elf::Loader`](../../doc/ku/process/elf/struct.Loader.html)
и её методы.

{{#include ../../doc/ku/process/elf/struct.Loader.html.declaration}}

Метод
[`Loader::load_program_header()`](../../doc/ku/process/elf/struct.Loader.html#method.load_program_header)
загружает в память диапазон `next` и переносит в целевое адресное пространство
заполненные физические фреймы, для которых стали известны флаги доступа.

{{#include ../../doc/ku/process/elf/struct.Loader.html.method.load_program_header}}

Метод
[`Loader::extend_mapping()`](../../doc/ku/process/elf/struct.Loader.html#method.extend_mapping)
расширяет отображение целевого адресного пространства, чтобы можно было записать
из ELF--файла в память блок, который описывает `next`.

{{#include ../../doc/ku/process/elf/struct.Loader.html.method.extend_mapping}}

Метод
[`Loader::finalize_mapping()`](../../doc/ku/process/elf/struct.Loader.html#method.finalize_mapping)
с помощью
[`BigAllocatorPair::copy_mapping()`](../../doc/ku/allocator/big_pair/trait.BigAllocatorPair.html#tymethod.copy_mapping)
переносит в целевое адресное пространство заполненные физические фреймы,
для которых стали известны флаги доступа --- диапазон `page_range`.

{{#include ../../doc/ku/process/elf/struct.Loader.html.method.finalize_mapping}}


### Задача 3 --- загрузка ELF--файла

Реализуйте указанные функции.
Вам могут пригодиться:

- Вспомогательная функция [`ku::process::elf::validate_order()`](../../doc/ku/process/elf/fn.validate_order.html).
- Метод [`PageTableFlags::try_from(ph_flags: xmas_elf::program::Flags)`](../../doc/kernel/memory/mmu/struct.PageTableFlags.html#method.try_from), который преобразовывает флаги ELF--файла в соответствующие флаги страниц. Так как [x86-64](https://en.wikipedia.org/wiki/X86-64) не поддерживает флага, который бы запрещал читать страницу, это отображение не точное. Кроме того, оно запрещает флаги, потенциально позволяющие атакующему [записать и выполнить произвольный код](https://en.wikipedia.org/wiki/Arbitrary_code_execution) в процессе работы процесса --- комбинацию флагов "доступно и на запись и на исполнение".
- Метод [`fn Block::<Virt>::enclosing() -> Block<Page>`](../../doc/ku/memory/block/struct.Block.html#method.enclosing), который для заданного блока виртуальных адресов возвращает минимальный содержащий его блок страниц виртуальной памяти.
- [`Block::<Virt>::try_into_mut_slice()`](../../doc/ku/memory/block/struct.Block.html#method.try_into_mut_slice), а также другие методы блоков, например [`Block::is_empty()`](../../doc/ku/memory/block/struct.Block.html#method.is_empty), [`Block::is_disjoint()`](../../doc/ku/memory/block/struct.Block.html#method.is_disjoint), [`Block::count()`](../../doc/ku/memory/block/struct.Block.html#method.count), [`Block::tail()`](../../doc/ku/memory/block/struct.Block.html#method.tail).
- Метод [`slice::write_filled()`](https://doc.rust-lang.org/nightly/core/primitive.slice.html#method.write_filled), инициализирующий сырую память.
- Методы [`copy_from_slice()`](https://doc.rust-lang.org/nightly/core/primitive.slice.html#method.copy_from_slice) и [`get()`](https://doc.rust-lang.org/nightly/core/primitive.slice.html#method.get) [срезов](https://doc.rust-lang.ru/book/ch04-03-slices.html).
- Арифметические операции с проверкой переполнения, такие как [`checked_add()`](https://doc.rust-lang.org/nightly/core/primitive.usize.html#method.checked_add).
- [`Result::map_err()`](https://doc.rust-lang.org/nightly/core/result/enum.Result.html#method.map_err) для преобразования одного типа ошибки в другой.
- [`Virt::new_u64()`](../../doc/ku/memory/addr/struct.Addr.html#method.new_u64).
- [`ku::memory::size::from()`](../../doc/ku/memory/size/fn.from.html).


### Дополнительные материалы про ELF--файлы

- [Executable and Linkable Format](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format).
- [ELF](https://wiki.osdev.org/ELF).
- [ELF-64 Object File Format](https://www.uclibc.org/docs/elf-64-gen.pdf).
- [ELF Format Cheatsheet](https://gist.github.com/x0nu11byt3/bcb35c3de461e5fb66173071a2379779).


### Проверьте себя

Запустите тест `4-process-3-elf` из файла
[`ku/tests/4-process-3-elf.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/4-process-3-elf.rs):

```console
$ (cd ku; cargo test --test 4-process-3-elf -- --test-threads=1)
...
running 10 tests
test t0_flags ... 2024-10-27T20:51:38.968820Z i t0_flags program_header.flags=    elf_range.flags=0--------P-(0x8000000000000001) expected_flags=0--------P-(0x8000000000000001)
...
2024-10-27T20:51:38.969256Z i t0_flags elf_range=Err(PermissionDenied) ph_flags=XWR
ok
test t1_ranges ... 2024-10-27T20:51:38.969578Z i t1_ranges elf_range=ElfRange { file_range: 0..0, flags: PageTableFlags(PRESENT | NO_EXECUTE), memory: [0v0, 0v1), size 1 B, Virt count 1, [~0 B, ~1 B) } program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 0, virtual_addr: 0, physical_addr: 0, file_size: 0, mem_size: 1, align: 0 }
...
2024-10-27T20:51:38.972547Z i t1_ranges elf_range=ElfRange { file_range: 4..8, flags: PageTableFlags(PRESENT | NO_EXECUTE), memory: [0v4, 0v8), size 4 B, Virt count 4, [~4 B, ~8 B) } program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 4, virtual_addr: 4, physical_addr: 0, file_size: 4, mem_size: 4, align: 0 }
ok
test t2_range_overflow ... 2024-10-27T20:51:38.972820Z i t2_range_overflow elf_range=Err(Overflow) program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 0, virtual_addr: 0, physical_addr: 0, file_size: 1, mem_size: 18446603336221196288, align: 0 }
2024-10-27T20:51:38.972881Z i t2_range_overflow elf_range=Err(Overflow) program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 0, virtual_addr: 18446603336221196288, physical_addr: 0, file_size: 1, mem_size: 9223372036854775807, align: 0 }
2024-10-27T20:51:38.972929Z i t2_range_overflow elf_range=Err(Overflow) program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 18446744073709551605, virtual_addr: 0, physical_addr: 0, file_size: 20, mem_size: 20, align: 0 }
2024-10-27T20:51:38.972975Z i t2_range_overflow elf_range=Err(Overflow) program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 0, virtual_addr: 0, physical_addr: 0, file_size: 21, mem_size: 20, align: 0 }
ok
test t3_out_of_bounds ... 2024-10-27T20:51:38.973160Z i  t3_out_of_bounds elf_range={ memory: [0v74E0_8FBF_F0EC, 0v74E0_8FBF_F0F8), size 12 B, flags: 0--------P-(0x8000000000000001), file_range: 0x1..0x4 } program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 1, virtual_addr: 128507833217260, physical_addr: 0, file_size: 3, mem_size: 12, align: 0 }
ok
test t4_file_offset ... 2024-10-27T20:51:38.973428Z i    t4_file_offset elf_range={ memory: [0v74E0_8FBF_F0AD, 0v74E0_8FBF_F0B0), size 3 B, flags: 0--------P-(0x8000000000000001), file_range: 0x7..0xA } program_header=ProgramHeader64 { type_: Ok(Null), flags: Flags(0), offset: 7, virtual_addr: 128507833217197, physical_addr: 0, file_size: 3, mem_size: 3, align: 0 }
2024-10-27T20:51:38.973515Z :    t4_file_offset ELF loadable program header file_block=[0v74E0_8FBF_F095, 0v74E0_8FBF_F098), size 3 B memory_block=[0v74E0_8FBF_F0AD, 0v74E0_8FBF_F0B0), size 3 B
ok
test t5_validate_order ... 2024-10-27T20:51:38.973762Z i t5_validate_order curr={ memory: [0v2, 0v3), size 1 B, flags: 0------U-PX(0x5), file_range: 0x0..0x0 } next={ memory: [0v1, 0v2), size 1 B, flags: 0------U-PX(0x5), file_range: 0x0..0x0 }
2024-10-27T20:51:38.973855Z ! t5_validate_order ELF loadable program headers intersect or are out of order by their virtual addresses curr_program_header=[0v2, 0v3), size 1 B next_program_header=[0v1, 0v2), size 1 B
...
2024-10-27T20:51:38.974546Z ! t5_validate_order ELF loadable program headers intersect or are out of order by their virtual addresses curr_program_header=[0v0, 0v3), size 3 B next_program_header=[0v1, 0v2), size 1 B
ok
test t6_stress_combine ... 2024-10-27T20:51:38.974791Z : t6_stress_combine check combine curr={ memory: [0v0, 0v1), size 1 B, flags: 0----------(0x8000000000000000), file_range: 0x0..0x0 } next={ memory: [0v1, 0v2), size 1 B, flags: 0----------(0x8000000000000000), file_range: 0x0..0x0 } curr_minus_next=None boundary=None updated_next={ memory: [0v1, 0v2), size 1 B, flags: 0----------(0x8000000000000000), file_range: 0x0..0x0 }
2024-10-27T20:51:38.974899Z : t6_stress_combine check combine curr={ memory: [0v0, 0v1), size 1 B, flags: 0----------(0x8000000000000000), file_range: 0x0..0x0 } next={ memory: [0v1, 0v2), size 1 B, flags: 0-------W--(0x8000000000000002), file_range: 0x0..0x0 } curr_minus_next=None boundary=None updated_next={ memory: [0v1, 0v2), size 1 B, flags: 0-------W--(0x8000000000000002), file_range: 0x0..0x0 }
...
2024-10-27T20:51:45.398259Z : t6_stress_combine check combine curr={ memory: [0v2001, 0v2002), size 1 B, flags: 0-------W-X(0x2), file_range: 0x0..0x0 } next={ memory: [0v4001, 0v4002), size 1 B, flags: 0-------W-X(0x2), file_range: 0x0..0x0 } curr_minus_next=Some(PageRange { flags: PageTableFlags(WRITABLE), memory: [0v2000, 0v3000), size 4.000 KiB, Page count 1, [~8.000 KiB, ~12.000 KiB) }) boundary=None updated_next={ memory: [0v4001, 0v4002), size 1 B, flags: 0-------W-X(0x2), file_range: 0x0..0x0 }
ok
test t7_finalize_mapping ... 2024-10-27T20:51:45.400811Z : t7_finalize_mapping offset_page=31373983862
2024-10-27T20:51:45.400898Z : t7_finalize_mapping reserved_fixed block=[0v74E0_9007_8000, 0v74E0_9007_9000), size 4.000 KiB
...
2024-10-27T20:51:45.423472Z : t7_finalize_mapping i=30 page=DummyPage { flags: PageTableFlags(AVAILABLE), reserved: true }
2024-10-27T20:51:45.423521Z : t7_finalize_mapping remap ELF page range page_range={ memory: [0v74E0_8802_8000, 0v74E0_8802_C000), size 16.000 KiB, flags: 0---------X(0x0) }
ok
test t8_extend_mapping ... 2024-10-27T20:51:45.425883Z :   t8_extend_mapping offset=0v74E0_8800_C000
2024-10-27T20:51:45.425977Z :   t8_extend_mapping curr=None next={ memory: [0v74E0_8800_DF61, 0v74E0_8801_0548), size 9.476 KiB, flags: 0-------W--(0x8000000000000002), file_range: 0x0..0x0 }
...
2024-10-27T20:51:45.450362Z :   t8_extend_mapping extend mapping block=[0v74E0_8802_AA2D, 0v74E0_8802_C000), size 5.456 KiB page_block=[0v74E0_8802_A000, 0v74E0_8802_C000), size 8.000 KiB flags=7-------W-X(0xE02)
ok
test t9_load_program_header ... 2024-10-27T20:51:45.451887Z i t9_load_program_header page_count=16 max_block_len=8 max_gap_len=8 iteration=0
2024-10-27T20:51:45.453051Z : t9_load_program_header offset=0v74E0_8800_C000
...
2024-10-27T20:54:14.818797Z : t9_load_program_header i=1020 page=DummyPage { flags: PageTableFlags(PRESENT | WRITABLE | AVAILABLE), reserved: true }
2024-10-27T20:54:14.818839Z : t9_load_program_header remap ELF page range page_range={ memory: [0v74E0_8842_3000, 0v74E0_8842_B000), size 32.000 KiB, flags: 0-------W-X(0x2) }
ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 155.88s
```

А также запустите тест `4-process-3-elf` из файла
[`kernel/tests/4-process-3-elf.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/4-process-3-elf.rs):

```console
$ (cd kernel; cargo test --test 4-process-3-elf)
...
4_process_3_elf::create_process-----------------------------
20:57:26.085 0 I duplicate; address_space = "process" @ 0p7E9_9000
20:57:26.091 0 I switch to; address_space = "process" @ 0p7E9_9000
20:57:26.097 0 D loading ELF from file; file_block = [0v20_2000, 0v80_1DD8), size 5.999 MiB
20:57:26.111 0 D ELF program header; next = { memory: [0v1000_0000_0000, 0v1000_0000_6DE4), size 27.473 KiB, flags: 0--------P-(0x8000000000000001), file_range: 0x1000..0x7DE4 }
20:57:26.135 0 D extend mapping; block = [0v1000_0000_0000, 0v1000_0000_6DE4), size 27.473 KiB; page_block = [0v1000_0000_0000, 0v1000_0000_7000), size 28.000 KiB; flags = 0------UWPX(0x7)
20:57:26.151 0 D ELF loadable program header; file_block = [0v20_3000, 0v20_9DE4), size 27.473 KiB; memory_block = [0v1000_0000_0000, 0v1000_0000_6DE4), size 27.473 KiB
20:57:26.165 0 D ELF program header; next = { memory: [0v1000_0000_6DF0, 0v1000_0004_AD86), size 271.896 KiB, flags: 0--------PX(0x1), file_range: 0x7DF0..0x4BD86 }
20:57:26.225 0 D extend mapping; block = [0v1000_0000_7000, 0v1000_0004_AD86), size 271.381 KiB; page_block = [0v1000_0000_7000, 0v1000_0004_B000), size 272.000 KiB; flags = 0------UWPX(0x7)
20:57:26.239 0 D ELF loadable program header; file_block = [0v20_9DF0, 0v24_DD86), size 271.896 KiB; memory_block = [0v1000_0000_6DF0, 0v1000_0004_AD86), size 271.896 KiB
20:57:26.265 0 D remap ELF page range; page_range = { memory: [0v1000_0000_0000, 0v1000_0000_6000), size 24.000 KiB, flags: 0--------P-(0x8000000000000001) }
20:57:26.277 0 D remap ELF page range; page_range = { memory: [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB, flags: 0--------PX(0x1) }
20:57:26.285 0 D ELF program header; next = { memory: [0v1000_0004_AD88, 0v1000_0004_AE58), size 208 B, flags: 0-------WP-(0x8000000000000003), file_range: 0x4BD88..0x4BE58 }
20:57:26.297 0 D ELF loadable program header; file_block = [0v24_DD88, 0v24_DE58), size 208 B; memory_block = [0v1000_0004_AD88, 0v1000_0004_AE58), size 208 B
20:57:26.363 0 D remap ELF page range; page_range = { memory: [0v1000_0000_7000, 0v1000_0004_A000), size 268.000 KiB, flags: 0--------PX(0x1) }
20:57:26.373 0 D ELF program header; next = { memory: [0v1000_0004_AE58, 0v1000_0005_18F8), size 26.656 KiB, flags: 0-------WP-(0x8000000000000003), file_range: 0x4BE58..0x528C0 }
20:57:26.387 0 D extend mapping; block = [0v1000_0004_B000, 0v1000_0005_18F8), size 26.242 KiB; page_block = [0v1000_0004_B000, 0v1000_0005_2000), size 28.000 KiB; flags = 0------UWPX(0x7)
20:57:26.401 0 D ELF loadable program header; file_block = [0v24_DE58, 0v25_48C0), size 26.602 KiB; memory_block = [0v1000_0004_AE58, 0v1000_0005_18F8), size 26.656 KiB
20:57:26.415 0 D remap ELF page range; page_range = { memory: [0v1000_0004_A000, 0v1000_0004_B000), size 4.000 KiB, flags: 0-------WPX(0x3) }
20:57:26.431 0 D remap ELF page range; page_range = { memory: [0v1000_0004_B000, 0v1000_0005_2000), size 28.000 KiB, flags: 0-------WP-(0x8000000000000003) }
20:57:26.439 0 I switch to; address_space = "base" @ 0p1000
20:57:26.445 0 I switch to; address_space = "process" @ 0p7E9_9000
20:57:26.481 0 I switch to; address_space = "base" @ 0p1000
20:57:26.487 0 D address space:
20:57:26.505 0 D     [0v1000, 0v1_6000), size 84.000 KiB, 0-------WPX(0x3)
20:57:26.531 0 D     [0vA_0000, 0vC_0000), size 128.000 KiB, 0-------WPX(0x3)
20:57:26.711 0 D     [0v20_0000, 0v81_9000), size 6.098 MiB, 0--------P-(0x8000000000000001)
20:57:26.751 0 D     [0v81_9000, 0v92_B000), size 1.070 MiB, 0--------PX(0x1)
20:57:26.779 0 D     [0v92_B000, 0vA0_8000), size 884.000 KiB, 0-------WP-(0x8000000000000003)
20:57:26.791 0 D     [0vA0_9000, 0vA2_8000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:26.803 0 D     [0vA2_9000, 0vA4_B000), size 136.000 KiB, 0-------WP-(0x8000000000000003)
20:57:26.811 0 D     [0vA4_B000, 0vA4_C000), size 4.000 KiB, 0----CT-WPX(0x1B)
20:57:26.879 0 D     [0vA4_C000, 0vA4_F000), size 12.000 KiB, 0-------WP-(0x8000000000000003)
20:57:26.887 0 D     [0v100_0000_0000, 0v100_0000_1000), size 4.000 KiB, 0-------WPX(0x3)
20:57:27.061 0 D     [0v100_0000_2000, 0v100_0020_2000), size 2.000 MiB, 0-------WPX(0x3)
20:57:27.069 0 D     [0vF7F_FFE7_B000, 0vF7F_FFE7_C000), size 4.000 KiB, 0-------WPX(0x3)
20:57:27.081 0 D     [0vF7F_FFE7_D000, 0vF7F_FFE9_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.093 0 D     [0vF7F_FFE9_D000, 0vF7F_FFEB_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.105 0 D     [0vF7F_FFEB_D000, 0vF7F_FFED_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.115 0 D     [0vF7F_FFED_D000, 0vF7F_FFEF_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.127 0 D     [0vF7F_FFEF_D000, 0vF7F_FFF1_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.139 0 D     [0vF7F_FFF1_D000, 0vF7F_FFF3_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.149 0 D     [0vF7F_FFF3_D000, 0vF7F_FFF5_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.161 0 D     [0vF7F_FFF5_D000, 0vF7F_FFF7_C000), size 124.000 KiB, 0-------WP-(0x8000000000000003)
20:57:27.185 0 D     [0vF7F_FFF7_C000, 0vF7F_FFFF_F000), size 524.000 KiB, 0-------WPX(0x3)
20:57:27.193 0 D     [0v1000_0000_0000, 0v1000_0000_6000), size 24.000 KiB, 0------U-P-(0x8000000000000005)
20:57:27.209 0 D     [0v1000_0000_6000, 0v1000_0004_A000), size 272.000 KiB, 0------U-PX(0x5)
20:57:27.217 0 D     [0v1000_0004_A000, 0v1000_0004_B000), size 4.000 KiB, 0------UWPX(0x7)
20:57:27.357 0 D     [0v1000_0004_B000, 0v1000_0005_2000), size 28.000 KiB, 0------UWP-(0x8000000000000007)
20:57:27.369 0 D     [0v7FFF_FFFD_6000, 0v7FFF_FFFF_6000), size 128.000 KiB, 0------UWP-(0x8000000000000007)
20:57:27.379 0 D     [0v7FFF_FFFF_6000, 0v7FFF_FFFF_7000), size 4.000 KiB, 0------U-P-(0x8000000000000005)
20:57:27.393 0 D     [0v7FFF_FFFF_7000, 0v7FFF_FFFF_F000), size 32.000 KiB, 0------UWP-(0x8000000000000007)
20:57:27.677 0 D     [0vFFFF_F000_0000_0000, 0vFFFF_F001_0020_0000), size 4.002 GiB, 0-H-----WPX(0x83)
20:57:27.735 0 D     [0vFFFF_FF80_0000_0000, 0vFFFF_FF80_0000_6000), size 24.000 KiB, 0-------WPX(0x3)
20:57:27.841 0 D     [0vFFFF_FF80_8000_0000, 0vFFFF_FF80_8000_2000), size 8.000 KiB, 0-------WPX(0x3)
20:57:27.849 0 D     [0vFFFF_FF87_BFFF_F000, 0vFFFF_FF87_C000_0000), size 4.000 KiB, 0-------WPX(0x3)
20:57:27.963 0 D     [0vFFFF_FF88_0000_0000, 0vFFFF_FF88_0000_1000), size 4.000 KiB, 0------UWPX(0x7)
20:57:27.979 0 D     [0vFFFF_FFBF_FFFF_F000, 0vFFFF_FFC0_0000_0000), size 4.000 KiB, 0------UWPX(0x7)
20:57:28.263 0 D     [0vFFFF_FFF8_0000_0000, 0vFFFF_FFF8_0080_1000), size 8.004 MiB, 0-H-----WPX(0x83)
20:57:28.273 0 I loaded ELF file; entry = 0v1000_0000_88D0; file_size = 5.999 MiB; process = { pid: <current>, address_space: "process" @ 0p7E9_9000, { rip: 0v1000_0000_88D0, rsp: 0v7FFF_FFFF_5000 } }
20:57:28.287 0 I user process page table entry; entry_point = 0v1000_0000_88D0; frame = Frame(32319 @ 0p7E3_F000); flags = PageTableFlags(PRESENT | USER)
20:57:28.301 0 D process_frames = 146
20:57:28.307 0 D dropping; spinlock = kernel/src/process/process.rs:85:28; stats = Stats { failures: 0, locks: 1, unlocks: 1, waits: 0 }
20:57:28.315 0 I drop; address_space = "process" @ 0p7E9_9000
4_process_3_elf::create_process-------------------- [passed]

4_process_3_elf::create_process_failure---------------------
20:57:30.103 0 I duplicate; address_space = "process" @ 0p7E9_9000
20:57:30.109 0 I switch to; address_space = "process" @ 0p7E9_9000
20:57:30.113 0 D loading ELF from file; file_block = [0v1, 0v1), size 0 B
20:57:30.119 0 I switch to; address_space = "base" @ 0p1000
20:57:30.125 0 I drop the current address space; address_space = "process" @ 0p7E9_9000; switch_to = "base" @ 0p1000
20:57:30.881 0 I expected a process creation failure; error = Elf("File is shorter than the first ELF header part")
4_process_3_elf::create_process_failure------------ [passed]
20:57:30.979 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
 ku/src/process/elf.rs | 197 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++--------
 1 file changed, 181 insertions(+), 16 deletions(-)
```
