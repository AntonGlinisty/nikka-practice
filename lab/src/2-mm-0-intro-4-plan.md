## План лабораторной работы №2

Для управления памятью ядру нужна структура данных, хранящая информацию про имеющиеся физические фреймы.
Когда будет поступать запрос на выделение памяти, эта структура должна будет выдать запрошенное количество фреймов.
При освобождении фреймов же, структура должна будет обработать этот факт.
То есть, ядру понадобится аллокатор физических фреймов.

Однако, самой такой структуре нужна память.
Эту память нужно как-то выделить и учесть в самом аллокаторе памяти.
Причём выделять память под аллокатор нужно до того как он сам будет инициализирован.
Получаем замкнутый круг.

Выбираться из него будем в три этапа.

Сначала
[реализуем аллокатор](../../lab/book/2-mm-1-frame-allocator.html)
физических фреймов
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html).
Под его метаданные выделим немного памяти фиксированного размера на этапе компиляции.
Выделять из него на этом этапе можно будет только ограниченную часть физической памяти.
Про каждый фрейм он будет помнить некоторую информацию, собранную в
[`FrameInfo`](../../doc/kernel/memory/frame_allocator/enum.FrameInfo.html).
Соответственно, ему потребуется целый
[срез](https://doc.rust-lang.ru/book/ch04-03-slices.html)
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info).
Полный размер этого среза зависит от доступного количества памяти в компьютере,
именно поэтому он не может быть инициализирован статически.
То есть, на первом этапе аллокатору будет доступен фактически
только начальный участок этого среза.

На втором этапе нам нужно будет
[поддержать работу с виртуальным адресным пространством](../../lab/book/2-mm-2-address-space.html),
инкапсулированную в структуру
[`AddressSpace`](../../doc/kernel/memory/struct.AddressSpace.html).
В ней вы реализуете выделение виртуальных страниц в адресном пространстве и страничное преобразование.
С помощью этих операций в дальнейшем можно будет выделять память под структуры,
размер которых не известен на этапе компиляции.

После этого
[реализуем изменение размера метаданных](../../lab/book/2-mm-8-resize-frame-allocator.html)
имеющегося у нас аллокатора физических фреймов
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html).
Теперь мы можем использовать полный
[срез](https://doc.rust-lang.ru/book/ch04-03-slices.html)
[`FrameAllocator::frame_info`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#structfield.frame_info),
описывающий всю доступную физическую память компьютера.
Память под новый срез мы выделим с помощью
[`AddressSpace`](../../doc/kernel/memory/struct.AddressSpace.html)
из самого же
[`FrameAllocator`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html).
Из старого среза скопируем имеющуюся метаинформацию в новый,
после чего память под старый срез можно будет освободить.

Отражением этого плана в коде является функция
[`kernel::memory::init()`](../../doc/kernel/memory/fn.init.html):
```rust
fn init(boot_info: &'static BootInfo) {
    ...

    *FRAME_ALLOCATOR.lock() = frame_allocator::init(&boot_info.memory_map);

    ...

    *BASE_ADDRESS_SPACE.lock() = AddressSpace::new(page_table_root, phys2virt);

    ...

    frame_allocator::resize(physical_memory, &boot_info.memory_map);

    ...
}
```
Сначала она инициализирует глобальный аллокатор физических фреймов
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html)
с заданным на этапе компиляции размером метаданных.
Затем она инициализирует глобальную переменную
[`BASE_ADDRESS_SPACE`](../../doc/kernel/memory/static.BASE_ADDRESS_SPACE.html)
для управления адресным пространством структурой
[`AddressSpace`](../../doc/kernel/memory/struct.AddressSpace.html).
Затем размер глобального аллокатора физических фреймов
[`FRAME_ALLOCATOR`](../../doc/kernel/memory/struct.FRAME_ALLOCATOR.html)
меняется на необходимый для данного компьютера.
