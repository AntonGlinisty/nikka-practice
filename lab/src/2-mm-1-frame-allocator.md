## Аллокатор физических фреймов

В этой задаче реализуем аллокатор физических фреймов
[`kernel::memory::frame_allocator::FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html).

Он построен на описателе физического фрейма
[`kernel::memory::frame_allocator::FrameInfo`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html).
Это [перечисление](https://doc.rust-lang.ru/book/ch06-01-defining-an-enum.html) с вариантами:

- [`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent) --- этого физического фрейма нет в компьютере или же он зарезервирован, например [BIOS](https://en.wikipedia.org/wiki/BIOS) или загрузчиком [`bootloader`](https://docs.rs/bootloader).
- [`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free) --- этот физический фрейм свободен. Содержит поле [`FrameInfo::Free::next_free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free.field.next_free) с номером следующего свободного фрейма, либо `None` если это последний элемент списка. То есть, описатели свободных фреймов провязаны в интрузивный односвязный список с номерами вместо указателей.
- [`FrameInfo::Used`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Used) --- этот физический фрейм занят. Содержит поле [`FrameInfo::Used::reference_count`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Used.field.reference_count) с количеством ссылок на описываемый фрейм.

В
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html)
все описатели физических фреймов собраны в поле
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info).
Это срез `&[FrameInfo]`, который вначале будет инициализироваться статически массивом
[`kernel::memory::frame_allocator::BOOT_FRAME_INFO`](../../doc/kernel/memory/frame_allocator/static.BOOT_FRAME_INFO.html)
фиксированного размера:
```rust
{{#include ../../kernel/src/memory/frame_allocator.rs:log_tsc}}
```

Поле
[`FrameAllocator::free_frame`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_frame)
содержит голову списка свободных фреймов.
А поле
[`FrameAllocator::free_count`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_count) ---
их количество.

Загрузчик [`bootloader`](https://docs.rs/bootloader), который мы используем,
получает от [BIOS](https://wiki.osdev.org/BIOS) или [UEFI](https://wiki.osdev.org/UEFI)
информацию о физической памяти компьютера.
Этот процесс состоит из нескольких наслоений устаревших решений.
Подробнее о том, как он устроен можно посмотреть [тут](https://wiki.osdev.org/Detecting_Memory_(x86)).
А ориентировочную карту зарезервированных областей --- [тут](https://wiki.osdev.org/Memory_Map_(x86)).
К счастью,
[`bootloader`](https://docs.rs/bootloader)
скрывает от нас эти устаревшие интерфейсы и предоставляет более удобный API ---
[`bootloader_api`](../../doc/bootloader_api/index.html)
готовую структуру
[`bootloader_api::info::MemoryRegions`](../../doc/bootloader_api/info/struct.MemoryRegions.html),
которая описывает карту доступной физической памяти.
Структура [`MemoryRegions`](../../doc/bootloader_api/info/struct.MemoryRegions.html)
[реализует](../../doc/bootloader_api/info/struct.MemoryRegions.html#impl-Deref) типаж
[`core::ops::Deref`](https://doc.rust-lang.org/nightly/core/ops/trait.Deref.html),
[возвращая из него](../../doc/bootloader_api/info/struct.MemoryRegions.html#associatedtype.Target)
массив структур
[`bootloader_api::info::MemoryRegion`](../../doc/bootloader_api/info/struct.MemoryRegion.html).
Это означает, что если мы напишем
```rust
for region in memory_regions.iter() {
    ...
}
```
переменная `region` пробежится по всем
[`MemoryRegion`](../../doc/bootloader_api/info/struct.MemoryRegion.html)
в
[`MemoryRegions`](../../doc/bootloader_api/info/struct.MemoryRegions.html).
В каждом из них есть поле
[`MemoryRegion::kind`](../../doc/bootloader_api/info/struct.MemoryRegion.html#structfield.kind),
которое равно:

- [`MemoryRegionKind::Usable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Usable) для незанятых диапазонов физической памяти.
- [`MemoryRegionKind::Kernel`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Kernel) и [`MemoryRegionKind::KernelStack`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.KernelStack) для диапазонов физической памяти, занятых ядром Nikka.
- [`PageTable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.PageTable) для фреймов, занятых на формирование таблицы страниц.

Нас будут интересовать только эти типы.
Кроме того,
[`MemoryRegion`](../../doc/bootloader_api/info/struct.MemoryRegion.html)
содержит поля
[`MemoryRegion::start`](../../doc/bootloader_api/info/struct.MemoryRegion.html#structfield.start) и
[`MemoryRegion::end`](../../doc/bootloader_api/info/struct.MemoryRegion.html#structfield.end).
Они соответствуют
[`Block::<Frame>::start`](../../doc/ku/memory/block/struct.Block.html#structfield.start) и
[`Block::<Frame>::end`](../../doc/ku/memory/block/struct.Block.html#structfield.end).
Из пары
[`MemoryRegion::start`](../../doc/bootloader_api/info/struct.MemoryRegion.html#structfield.start) и
[`MemoryRegion::end`](../../doc/bootloader_api/info/struct.MemoryRegion.html#structfield.end),
можно сделать соответствующий
[`Block::<Frame>`](../../doc/ku/memory/block/struct.Block.html)
с помощью метода
[`Block::<Frame>::from_index_u64(start, end)`](../../doc/ku/memory/block/struct.Block.html#method.from_index_u64).


### Задача 1 --- [`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html)


#### Инициализация описателей фреймов

Реализуйте метод
[`FrameAllocator::init_frame_info()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.init_frame_info)

```rust
{{#include ../../kernel/src/memory/frame_allocator.rs:init_frame_info}}
```

в файле [`kernel/src/memory/frame_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/frame_allocator.rs).
Пройдите по `memory_regions` с картой памяти [`MemoryRegions`](../../doc/bootloader_api/info/struct.MemoryRegions.html).
Обратите внимание, что на вход метод принимает также диапазон `frames`.
Он содержит номера физических фреймов, которые нужно учесть.
Не входящие в него фреймы:

- Либо лежат за пределами доступного на текущем этапе среза [`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info).
- Либо при последующих вызовах этого метода окажутся уже заполнены, --- на текущем этапе.

Заполнять метаданные нужно так:

- Пометьте фреймы из [`Usable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Usable) регионов как [`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free) и провяжите их в односвязный список.
- Пометьте фреймы из [`MemoryRegionKind::Kernel`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Kernel), [`MemoryRegionKind::KernelStack`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.KernelStack) и [`MemoryRegionKind::PageTable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.PageTable) как [`FrameInfo::Used`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Used). Их мы сможем освободить в дальнейшем. В частности, --- память под начальный массив [`kernel::memory::frame_allocator::BOOT_FRAME_INFO`](../../doc/kernel/memory/frame_allocator/static.BOOT_FRAME_INFO.html).
- Если тип региона памяти не является [`MemoryRegionKind::Usable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Usable), пометьте фрейм как [`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent). В этом случае можно просто ничего не делать. Как вы видели, [`FrameAllocator::new()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.new) при выделении памяти под срез [`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info) инициализирует его значением [`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent).

Количество найденных свободных фреймов сохраните в
[`FrameAllocator::free_count`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_count).

Вам могут пригодиться функции

- [`ku::memory::frage::Frame::from_index()`](../../doc/ku/memory/frage/struct.Frage.html#method.from_index),
- [`ku::memory::size::from()`](../../doc/ku/memory/size/fn.from.html),
- [`core::mem::replace()`](https://doc.rust-lang.org/nightly/core/mem/fn.replace.html).


#### Выделение фрейма

Реализуйте метод
[`FrameAllocator::allocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.allocate)

```rust
{{#include ../../kernel/src/memory/frame_allocator.rs:allocate}}
```

в файле [`kernel/src/memory/frame_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/frame_allocator.rs).
Он выделяет и возвращает свободный фрейм.
Количество ссылок на этот фрейм он выставляет равным единице.
Фрейм возвращается обёрнутым в RAII
[`kernel::memory::FrameGuard`](../../doc/kernel/memory/struct.FrameGuard.html),
подробнее про который можно посмотреть
[в разделе про типы](../../lab/book/2-mm-0-intro-1-types.html#raii-kernelmemoryframeguard)
и в
[его документации](../../doc/kernel/memory/struct.FrameGuard.html).
Если свободных физических фреймов не осталось,
верните ошибку
[`Error::NoFrame`](../../doc/kernel/error/enum.Error.html#variant.NoFrame),
Чтобы долго не искать, используйте фрейм из головы списка свободных ---
[`FrameAllocator::free_frame`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_frame).
Не забудьте присвоить ему следующий свободный фрейм.
Также обновите количество оставшихся свободных фреймов [`FrameAllocator::free_count`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_count).

На всякий случай можно перепроверить, что
[`FrameAllocator::free_frame`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.free_frame)
соответствует записи в
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info),
являющейся
[`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free).
И запаниковать, если это не так:
```rust
panic!("your {} message {:?}", ...);
```


#### Освобождение фрейма

Реализуйте метод
[`FrameAllocator::deallocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.deallocate)

```rust
{{#include ../../kernel/src/memory/frame_allocator.rs:deallocate}}
```

в файле [`kernel/src/memory/frame_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/frame_allocator.rs).
Это обратный метод к
[`FrameAllocator::allocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.allocate).
Он уменьшает количество ссылок на данный `frame` и помечает его как
[`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free),
если ссылок не осталось.
Если `frame` уже является
[`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free),
стоит запаниковать.

Однако освобождение фрейма, который помечен как
[`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent) ---
корректная ситуация, паниковать не нужно.
Дело в том, что зарезервированные фреймы, которые мы так помечаем в
[`FrameAllocator::init_frame_info()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.init_frame_info)
могут быть отображены в память загрузчиком
[`bootloader`](https://docs.rs/bootloader).
И при копировании и освобождении
[`Mapping`](../../doc/kernel/memory/mapping/struct.Mapping.html)
мы можем попытаться поменять количество ссылок на них.
В этом случае ничего делать не нужно, фрейм должен остаться
[`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent)
и не должен попасть в список свободных.
Так как может быть много разных причин почему он не был
[`MemoryRegionKind::Usable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Usable).
Например, он может соответствовать микросхеме
[постоянного запоминающего устройства](https://en.wikipedia.org/wiki/Read-only_memory),
которая позволяет только читать данные, и не может быть использован ядром для произвольных данных.

То же самое относится к фреймам, номера которых выходят за границы среза
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info), ---
могут иметься зарезервированные участки памяти, которые находятся по очень большим адресам.
Один из таких участков --- область ввода--вывода через память ([Memory-mapped I/O](https://en.wikipedia.org/wiki/Memory-mapped_I/O)) для управления [контроллером прерываний APIC](https://wiki.osdev.org/APIC):
```console
15:08:28 0 I acpi = Acpi { local_apic_address: Phys(0pFEE00000), bsp_id: 0, ap_ids: [1, 2, 3] }
15:08:28 0 I Local APIC init; cpu = 0; cpu_count = 4; local_apic_address = 0pFEE00000
```
Обратите внимание на физический адрес в районе 4 GiB --- `local_apic_address = 0pFEE00000`.
Поэтому функция
[`kernel::memory::range::physical()`](../../doc/kernel/memory/range/fn.physical.html)
игнорирует не
[`Usable`](../../doc/bootloader_api/info/enum.MemoryRegionKind.html#variant.Usable) регионы,
чтобы не нужно было выделять огромный срез
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info),
который может просто не поместится в память,
и будет содержать в основном
[`Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent) записи.


#### Увеличение счётчика использований фрейма

Реализуйте метод
[`FrameAllocator::reference()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.reference)

```rust
{{#include ../../kernel/src/memory/frame_allocator.rs:reference}}
```

в файле [`kernel/src/memory/frame_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/frame_allocator.rs).
Он наоборот, увеличивает количество ссылок на заданный `frame`.
Если `frame` является
[`FrameInfo::Free`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Free),
стоит запаниковать.
Как и для
[`FrameAllocator::deallocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.deallocate),
если фрейм
[`FrameInfo::Absent`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html#variant.Absent)
или выходит за границы среза
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info) ---
это допустимая ситуация, в которой ничего делать не нужно.


### Проверьте себя

Запустите тесты:

```console
$ (cd kernel; cargo test --test 2-mm-1-frame-allocator)
   Compiling kernel v0.5.0 (/.../nikka/kernel)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.57s
     Running tests/2-mm-1-frame-allocator.rs (/tmp/target/kernel/debug/deps/2_mm_1_frame_allocator-6691b6382501be55)
Building bootloader
   Compiling bootloader v0.9.33 (/.../.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bootloader-0.9.33)
    Finished `release` profile [optimized + debuginfo] target(s) in 0.73s
Running: `qemu-system-x86_64 -drive format=raw,file=/tmp/target/kernel/debug/deps/bootimage-2_mm_1_frame_allocator-6691b6382501be55.bin -no-reboot -m size=128M -smp cpus=4 -device isa-debug-exit,iobase=0xF4,iosize=0x04 -serial stdio -display none -drive file=../fs.img,index=1,media=disk,driver=raw`
...
running 3 tests

2_mm_1_frame_allocator::t1_sanity_check---------------------
14:26:35 0 D free_frames = 14668; min_free_frames = 4096; qemu_memory_frames = 32768
2_mm_1_frame_allocator::t1_sanity_check------------ [passed]

2_mm_1_frame_allocator::t2_allocate-------------------------
14:26:35 0 D frames = [FrameGuard { frame: Frame(16382 @ 0p3FF_E000) }, FrameGuard { frame: Frame(16381 @ 0p3FF_D000) }]; frame_count = 32768
2_mm_1_frame_allocator::t2_allocate---------------- [passed]

2_mm_1_frame_allocator::t3_allocated_frames_are_unique------
14:26:35 0 D free_frames = 14666; frame_count = 32768
14:26:35 0 D prev_frame = 16380 @ 0p3FF_C000; frame = 16379 @ 0p3FF_B000
14:26:35 0 D prev_frame = 6380 @ 0p18E_C000; frame = 6379 @ 0p18E_B000
2_mm_1_frame_allocator::t3_allocated_frames_are_unique [passed]
14:26:35 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Использование аллокатора физических фреймов

Ядро выделяет физические фреймы с помощью глобальной переменной
[`kernel::memory::FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html).
Это
[`kernel::memory::FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html),
обёрнутый в спин-блокировкy
[`ku::sync::Spinlock`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html) ---
[`Spinlock`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html)`<`[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html)`>`.
[`Spinlock`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html)`<`[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html)`>`.
Чтобы работать с
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html),
нужно сначала захватить блокировку методом
[`Spinlock::lock()`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html#method.lock)
а затем уже вызвать подходящий метод
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html),
например
[`FrameAllocator::allocate()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#tymethod.allocate).
Метод
[`Spinlock::lock()`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html#method.lock)
возвращает
[`ku::sync::SpinlockGuard`](../../doc/ku/sync/spinlock/struct.DebugSpinlockGuard.html),
который:

- За [счёт](../../doc/ku/sync/spinlock/struct.DebugSpinlockGuard.html#impl-Deref) [реализации](../../doc/ku/sync/spinlock/struct.DebugSpinlockGuard.html#impl-DerefMut) типажей [`core::ops::Deref`](https://doc.rust-lang.org/nightly/core/ops/trait.Deref.html) и [`core::ops::DerefMut`](https://doc.rust-lang.org/nightly/core/ops/trait.DerefMut.html) предоставляет прозрачный доступ к внутреннему [`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html).
- При разрушении, например при выходе из своей зоны видимости или [явном вызове `drop(guard)`](../../doc/ku/sync/spinlock/struct.DebugSpinlock.html#%D0%AF%D0%B2%D0%BD%D0%BE%D0%B5-%D0%BE%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B6%D0%B4%D0%B5%D0%BD%D0%B8%D0%B5-%D0%B1%D0%BB%D0%BE%D0%BA%D0%B8%D1%80%D0%BE%D0%B2%D0%BA%D0%B8), автоматически освобождает захваченную блокировку.

Пример использования с блокировками, которые освобождаются сразу, в том же выражении, есть в тестах
[`kernel/tests/memory.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/memory.rs):
```rust
    let frame = FRAME_ALLOCATOR
        .lock()
        .allocate()
        .expect("failed to allocate a frame");

    debug!(?frame);

    FRAME_ALLOCATOR
        .lock()
        .reference(frame);
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/frame_allocator.rs | 150 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
1 file changed, 140 insertions(+), 10 deletions(-)
```
