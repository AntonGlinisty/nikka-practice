## Высокоуровневый интерфейс управления адресным пространством

Всё управление адресным пространством инкапсулировано в структуре
[`kernel::memory::address_space::AddressSpace`](../../doc/kernel/memory/address_space/struct.AddressSpace.html).
Её поле
[`AddressSpace::mapping`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#structfield.mapping)
служит для модификации отображения страниц.
А поле
[`AddressSpace::page_allocator`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#structfield.page_allocator) ---
для выделения виртуальных страниц внутри этого адресного пространства.


### Задача 5 --- высокоуровневый интерфейс управления адресным пространством

#### Отображение виртуальной страницы на физический фрейм

Реализуйте метод
[`AddressSpace::map_page_to_frame()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page_to_frame)

```rust
unsafe fn AddressSpace::map_page_to_frame(
    &mut self,
    page: Page,
    frame: Frame,
    flags: PageTableFlags,
) -> Result<()>
```

в файле [`kernel/src/memory/address_space.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/address_space.rs).
Он должен отобразить заданную виртуальную страницу `page` на заданный физический фрейм `frame` с указанными флагами доступа `flags`.

Если во `flags` включён флаг
[`PageTableFlags::USER`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.USER),
а `page` не принадлежит
[пользовательской части виртуального адресного пространства](../../lab/book/2-mm-2-page-allocator.html#%D0%93%D1%80%D0%B0%D0%BD%D0%B8%D1%86%D0%B0-%D0%BC%D0%B5%D0%B6%D0%B4%D1%83-%D1%81%D1%82%D1%80%D0%B0%D0%BD%D0%B8%D1%86%D0%B0%D0%BC%D0%B8-%D0%B4%D0%BB%D1%8F-%D1%8F%D0%B4%D1%80%D0%B0-%D0%B8-%D0%B4%D0%BB%D1%8F-%D0%BF%D1%80%D0%BE%D1%81%D1%82%D1%80%D0%B0%D0%BD%D1%81%D1%82%D0%B2%D0%B0-%D0%BF%D0%BE%D0%BB%D1%8C%D0%B7%D0%BE%D0%B2%D0%B0%D1%82%D0%B5%D0%BB%D1%8F),
возвращается ошибка
[`Error::PermissionDenied`](../../doc/kernel/error/enum.Error.html#variant.PermissionDenied).
Если этот флаг выключен, но `page` принадлежит пользовательской части адресного пространства ---
тоже нужно вернуть ошибку.
Проверить куда попадает `page` можно функцией
[`kernel::memory::range::validate_page_flags()`](../../doc/kernel/memory/range/fn.validate_page_flags.html).

Используйте
[`Mapping::path()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.path),
[`Path::map()`](../../doc/kernel/memory/path/struct.Path.html#method.map)
и
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html).
Вам может пригодиться метод
[`PageTableEntry::set_frame()`](../../doc/kernel/memory/mmu/struct.PageTableEntry.html#method.set_frame).
Увеличивать количество ссылок на `frame` не нужно, это задача вызывающей функции.
Дело в том, что нам иногда захочется использовать
[`AddressSpace::map_page_to_frame()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page_to_frame)
для отображения зарезервированных физических фреймов, про которые
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html)
ничего не знает и не отслеживает количество ссылок на них.
Например, это делает метод
[`kernel::smp::local_apic::LocalApic::map()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.map),
инициализирующий работу с
[контроллером прерываний APIC](https://wiki.osdev.org/APIC)
методом ввода--вывода через память
([Memory-mapped I/O](https://en.wikipedia.org/wiki/Memory-mapped_I/O)).

Если `page` уже была отображена, то старое отображение удаляется,
а старый физический фрейм освобождается,
если на него не осталось других ссылок.
Удалить старое отображение можно с помощью метода
[`Path::unmap()`](../../doc/kernel/memory/path/struct.Path.html#method.unmap).

Используя
[`FrameAllocator::allocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.allocate)
из глобального
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html)
и
[`AddressSpace::map_page_to_frame()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page_to_frame)
реализуйте метод
[`AddressSpace::map_page()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page)

```rust
unsafe fn AddressSpace::map_page(
    &mut self,
    page: Page,
    flags: PageTableFlags,
) -> Result<Frame>
```

в файле [`kernel/src/memory/address_space.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/address_space.rs).
Он отличается от
[`AddressSpace::map_page_to_frame()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page_to_frame)
только тем, что не принимает `frame`, а выделяет его сам.


#### Удаление отображения заданной страницы

Реализуйте метод
[`AddressSpace::unmap_page()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.unmap_page)

```rust
unsafe fn AddressSpace::unmap_page(
    &mut self,
    page: Page,
) -> Result<()>
```

в файле
[`kernel/src/memory/address_space.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/address_space.rs).

Воспользуйтесь
[`Mapping::path()`](../../doc/kernel/memory/mapping/struct.Mapping.html#method.path)
и
[`Path::unmap()`](../../doc/kernel/memory/path/struct.Path.html#method.unmap).

Также вам пригодятся

- [`FrameAllocator::deallocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.deallocate) и
- [`PageTableEntry::clear()`](../../doc/kernel/memory/mmu/struct.PageTableEntry.html#method.clear).


### Изучите как устроены отображения целых блоков

Посмотрите на методы

- [`unsafe fn AddressSpace::map_block(&mut self, pages: Block<Page>, flags: PageTableFlags) -> Result<()>`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_block) и
- [`unsafe fn AddressSpace::unmap_block(&mut self, pages: Block<Page>) -> Result<()>`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.unmap_block).

Их код и назначение должны быть уже понятны.


### Изучите как устроены отображения срезов

С помощью реализованных вами функций метод
[`AddressSpace::map_slice_uninit()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_slice_uninit)

```rust
fn AddressSpace::map_slice_uninit<T>(
    &mut self,
    len: usize,
    flags: PageTableFlags,
) -> Result<&'static mut [MaybeUninit<T>]>
```

выделяет срез неинициализированной памяти
[`core::mem::MaybeUninit`](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html),
достаточный для `len` элементов типа `T`.
Так как он пользуется выделением виртуальных страниц и физических фреймов,
каждый вызов расходует целое число фреймов.
То есть, выделение через него срезов маленького размера в байтах неэффективно.
В результате вернётся срез с большим, чем было запрошено, количеством элементов ---
сколько поместилось в выделенном наборе страниц.

[Метод](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_slice)

```rust
fn AddressSpace::map_slice<T, F: Fn() -> T>(
    &mut self,
    len: usize,
    flags: PageTableFlags,
    default: F,
) -> Result<&'static mut [T]>
```

дополнительно параметризован
[замыканием](https://doc.rust-lang.ru/book/ch13-01-closures.html)
типажа
[`core::ops::Fn`](https://doc.rust-lang.org/nightly/core/ops/trait.Fn.html).
Возвращаемый им срез уже инициализирован значениями, которые возвращает замыкание `default`.

[Метод](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_slice_zeroed)

```rust
unsafe fn AddressSpace::map_slice_zeroed<T>(
    &mut self,
    len: usize,
    flags: PageTableFlags,
) -> Result<&'static mut [T]>
```

не принимает значение для инициализации, а инициализирует память нулями.
`unsafe` означает, что вызывающая функция должна гарантировать, что [нулевые байты являются корректным значением](https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html#example-2) для типа `T`.

[Метод](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.unmap_slice)

```rust
unsafe fn AddressSpace::unmap_slice<T>(
    &mut self,
    slice: &mut [T],
) -> Result<()>
```

удаляет отображение страниц под заданным срезом.
Он проверяет, что срез действительно занимает свои страницы целиком.
`unsafe` означает, что вызывающая функция должна гарантировать, что `slice` больше не будет использоваться.


### Проверьте себя

Запустите тест:

```console
$ (cd kernel; cargo test --test 2-mm-5-map)
...
running 6 tests

2_mm_5_map::kernel_and_user_are_separated-------------------
2_mm_5_map::kernel_and_user_are_separated---------- [passed]

2_mm_5_map::map_readable------------------------------------
18:36:33 0 D allocated unique virtual address; virt = 0vF80_0000_0000; is_user = false
18:36:33 0 D page = 4160749568 @ 0vF80_0000_0000; frame = 16382 @ 0p3FF_E000
2_mm_5_map::map_readable--------------------------- [passed]

2_mm_5_map::map_slice---------------------------------------
18:36:35.709 0 D slice = [0vF7F_FEFF_F000, 0vF7F_FFFF_F000), size 16 MiB, Virt count 16777216, [~15.500 TiB, ~15.500 TiB)
18:36:35.729 0 D slice_frames = 4096; frame_count = 32768
2_mm_5_map::map_slice------------------------------ [passed]

2_mm_5_map::map_twice---------------------------------------
18:36:37.871 0 D allocated unique virtual address; virt = 0vF00_0000_0000; is_user = false
18:36:37.897 0 D old_frame = Ok(Frame(12274 @ 0p2FF_2000)); new_frame = Ok(Frame(12277 @ 0p2FF_5000))
2_mm_5_map::map_twice------------------------------ [passed]

2_mm_5_map::map_writable------------------------------------
18:36:37.931 0 D allocated unique virtual address; virt = 0vE80_0000_0000; is_user = false
18:36:37.961 0 D page = 3892314112 @ 0vE80_0000_0000; frame = 12277 @ 0p2FF_5000
18:36:37.975 0 D allocated unique virtual address; virt = 0vE00_0000_0000; is_user = false
18:36:38.005 0 D page = 3758096384 @ 0vE00_0000_0000; frame = 12277 @ 0p2FF_5000
18:36:38.019 0 D allocated unique virtual address; virt = 0vD80_0000_0000; is_user = false
18:36:38.049 0 D page = 3623878656 @ 0vD80_0000_0000; frame = 12286 @ 0p2FF_E000
18:36:38.063 0 D allocated unique virtual address; virt = 0vD00_0000_0000; is_user = false
18:36:38.093 0 D page = 3489660928 @ 0vD00_0000_0000; frame = 12289 @ 0p300_1000
2_mm_5_map::map_writable--------------------------- [passed]

2_mm_5_map::unmap_nonmapped_page----------------------------
18:36:38.125 0 D allocated unique virtual address; virt = 0vC80_0000_0000; is_user = false
2_mm_5_map::unmap_nonmapped_page------------------- [passed]
18:36:38.163 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/address_space.rs | 35 +++++++++++++++++++++++++++++------
1 file changed, 29 insertions(+), 6 deletions(-)
```
