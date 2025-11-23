## Работа с local APIC

При реализации вытесняющей многозадачности для измерения кванта времени,
отведённого текущему процессу, нам нужны часы.
Микросхема [RTC](../../lab/book/1-hw-1-rtc.html) не подходит,
так как отмеряет интервалы времени равные секунде ---
слишком большие для кванта времени.
Кроме неё в компьютере есть таймер
[Intel 8253/8254](https://en.wikipedia.org/wiki/Intel_8253)
([programmable interval timer, PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)).
Можно использовать его, однако он морально устарел и обладает существенным недостатком ---
он один на всю систему.
А нам хотелось бы в конечном итоге построить многопроцессорную систему,
которая запускает процессы на каждом процессоре.
Прерывания же
[PIT](https://en.wikipedia.org/wiki/Programmable_interval_timer)
будут тогда доставляться только на один процессор.
И не понятно как ядро будет вытеснять процессы, запущенные на других процессорах.
Либо можно настроить доставку прерывания сразу на все процессоры.
Но тогда они все будут одновременно запускать планировщик,
что приведёт к высокой конкуренции на его внутренних блокировках.

Поэтому в
[x86-64](https://en.wikipedia.org/wiki/X86-64)
был добавлен более современный таймер, который есть у каждого логического процессора.
За него отвечает современный контроллер прерываний
[Advanced Programmable Interrupt Controller (APIC)](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller).
Этот контроллер прерываний пришёл на смену устаревшему
[Intel 8259](https://en.wikipedia.org/wiki/Intel_8259).
Он состоит из набора контроллеров прерываний
[local APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#Integrated_local_APICs),
по одному у каждого логического процессора,
и одного или нескольких контроллеров внешних прерываний
[I/O APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#I/O_APICs).
В каждом из
[local APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#Integrated_local_APICs)
есть
[собственный таймер](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#APIC_timer).
Его мы и будем использовать для реализации принудительного вытеснения процессов.

То есть, прежде всего нам нужно инициализировать
[local APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#Integrated_local_APICs)
и настроить его таймер на посылку периодических прерываний.
Эта настройка осуществляется через
[Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
то есть нам нужно знать физический адрес,
по которому располагается MMIO для
[local APIC](https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller#Integrated_local_APICs).
Эту информацию подготавливает BIOS, а ядро получает её от него в таблицах
[Advanced Configuration and Power Interface (ACPI)](https://en.wikipedia.org/wiki/Advanced_Configuration_and_Power_Interface).
Чтение таблицы конфигурации процессоров реализовано с помощью внешней библиотеки
[`acpi`](../../doc/acpi/index.html) и находится в методе
[`kernel::smp::acpi::Acpi::new()`](../../doc/kernel/smp/acpi/struct.Acpi.html#method.new)
в файле [`kernel/src/smp/acpi.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/smp/acpi.rs).

В Nikka функционал для работы с LAPIC реализован в структуре
[`kernel::smp::local_apic::LocalApic`](../../doc/kernel/smp/local_apic/struct.LocalApic.html)
в файле [`kernel/src/smp/local_apic.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/smp/local_apic.rs).
В частности, у неё есть такие статические методы:

- [`LocalApic::init()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.init) инициализирует local APIC, например включает прерывание таймера.
- [`LocalApic::id()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.id) позволяет узнать идентификатор local APIC и текущего CPU.
- [`LocalApic::send_init()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.send_init) пошлёт прерывание на AP, предназначенное для его инициализации.
- [`LocalApic::get()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.get) возвращает структуру [`LocalApic`](../../doc/kernel/smp/local_apic/struct.LocalApic.html) данного CPU.


### Задача 1 --- инициализация работы с `LocalApic`

Интерфейс для работы с local APIC в [x86-64](https://en.wikipedia.org/wiki/X86-64) представляет собой выровненную область размером 4 килобайта в физической памяти --- один физический фрейм.
Такой тип интерфейса называется
[Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O).
Чтобы пользоваться MMIO, нужно:

- Во-первых, отобразить его физический адрес в текущее виртуальное адресное пространство.
- А во-вторых, указать процессору, что данный диапазон памяти не нужно кэшировать. Для этого требуется выставить флаги [`PageTableFlags::NO_CACHE`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.NO_CACHE) и [`PageTableFlags::WRITE_THROUGH`](../../doc/ku/memory/mmu/struct.PageTableFlags.html#associatedconstant.WRITE_THROUGH) в [`PageTableEntry`](../../doc/ku/memory/mmu/struct.PageTableEntry.html).

Каждый процессор имеет собственный local APIC, потому он и называется локальным (local).
LAPIC находится на том же кристалле, что и процессор и связан с ним выделенным каналом.
А физические адреса всех LAPIC в системе совпадают.
Когда процессор обращается по физическому адресу, относящемуся к LAPIC, он использует этот выделенный канал.
И это обращение не видят другие процессоры и LAPIC в системе.
Поэтому конфликта из-за одинаковых физических адресов не происходит.
Наоборот, совпадение физических адресов всех LAPIC удобно для программиста.
Например, если программе нужно понять, на каком именно CPU она сейчас запущена ---
достаточно прочитать идентификатор LAPIC и CPU по определённому физическому адресу области MMIO LAPIC.
Это и делает метод
[`LocalApic::id()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.id).

Физический адрес MMIO для local APIC можно узнать, вызвав метод
[`kernel::smp::acpi::Acpi::local_apic_address()`](../../doc/kernel/smp/acpi/struct.Acpi.html#method.local_apic_address).
А виртуальный адрес можно было бы выбрать произвольно.
Но ядро должно этот адрес где-то запомнить, чтобы использовать для обращений в дальнейшем.
Для простоты, запишем этот адрес прямо в код.
Для этого, заведём статическую переменную
[`kernel::smp::local_apic::LOCAL_APIC`](../../doc/kernel/smp/local_apic/static.LOCAL_APIC.html),
изначально содержащую страницу нулей.
Компилятор выделит под неё страницу в памяти ядра, и настроит все обращения в коде к ней.
А мы при инициализации MMIO для local APIC удалим отображение для этой виртуальной страницы и вместо ненужных нам нулей отобразим на её виртуальный адрес физический фрейм local APIC.
Побочным результатом такого подхода будет то, что до того как мы инициализируем MMIO, метод
[`LocalApic::id()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.id)
будет выдавать `0`, а не ошибку.
И функции, которые им пользуются, могут обойтись без обработки ошибки в этом случае.

Реализуйте [метод](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.map)

```rust
fn LocalApic::map(address: Phys) -> Result<()>
```

в файле
[`kernel/src/smp/local_apic.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/smp/local_apic.rs).

Физический адрес MMIO local APIC он принимает на вход.
А для определения нужного виртуального адреса вам пригодится функция
[`LocalApic::get()`](../../doc/kernel/smp/local_apic/struct.LocalApic.html#method.get).
Также могут пригодиться

- [`kernel::memory::BASE_ADDRESS_SPACE`](../../doc/kernel/memory/static.BASE_ADDRESS_SPACE.html),
- [`kernel::memory::address_space::AddressSpace::map_page_to_frame()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_page_to_frame),
- [`kernel::memory::Frame`](../../doc/kernel/memory/type.Frame.html),
- [`kernel::memory::Page`](../../doc/kernel/memory/type.Page.html),
- [`kernel::memory::Virt`](../../doc/kernel/memory/type.Virt.html).


### Проверьте себя

Запустите тест `4-process-1-local-apic` из файла
[`kernel/tests/4-process-1-local-apic.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/4-process-1-local-apic.rs):

```console
$ (cd kernel; cargo test --test 4-process-1-local-apic)
...
4_process_1_local_apic::mapped_properly-------------------------
17:31:59 0 I Local APIC; local_apic = 0v3E_9000; frame = Frame(1043968 @ 0pFEE0_0000); flags = PageTableFlags(PRESENT | WRITABLE | WRITE_THROUGH | NO_CACHE | ACCESSED | DIRTY)
17:31:59 0 I cpu = 0
4_process_1_local_apic::mapped_properly---------------- [passed]
17:31:59 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
 kernel/src/smp/local_apic.rs | 19 +++++++++++++++++--
 1 file changed, 17 insertions(+), 2 deletions(-)
```
