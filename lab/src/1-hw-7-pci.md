## Работа с конфигурацией PCI--устройств

В этом цикле задач попробуйте обойтись без подробного описания.


### Реализация

Как и в
[задаче про управление курсором](../../lab/book/1-hw-8-cursor.html#%D0%A3%D0%BF%D1%80%D0%B0%D0%B2%D0%BB%D0%B5%D0%BD%D0%B8%D0%B5-%D0%BA%D1%83%D1%80%D1%81%D0%BE%D1%80%D0%BE%D0%BC),
при реализации не работайте с
[портами ввода--вывода](../../lab/book/1-hw-1-rtc.html#%D0%9F%D0%BE%D1%80%D1%82%D1%8B-%D0%B2%D0%B2%D0%BE%D0%B4%D0%B0%D0%B2%D1%8B%D0%B2%D0%BE%D0%B4%D0%B0)
напрямую.
Для доступа к
[пространству конфигурации PCI](https://en.wikipedia.org/wiki/PCI_configuration_space)
используйте
методы
[`pci::config_space::ConfigSpace::read()`](../../doc/pci/config_space/trait.ConfigSpace.html#tymethod.read)
и
[`pci::config_space::ConfigSpace::write()`](../../doc/pci/config_space/trait.ConfigSpace.html#tymethod.write)
типажа
[`pci::config_space::ConfigSpace`](../../doc/pci/config_space/trait.ConfigSpace.html).
Это нужно как для тестов, так и для абстрагирования метода доступа к
[пространству конфигурации PCI](https://en.wikipedia.org/wiki/PCI_configuration_space)
либо через
[порты ввода--вывода](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O),
как в
[шинах PCI и PCI-X](https://en.wikipedia.org/wiki/PCI),
либо через
[Memory--mapped I/O (MMIO)](https://en.wikipedia.org/wiki/Memory-mapped_I/O),
как в
[шине PCI Express (Peripheral Component Interconnect Express, PCIe)](https://en.wikipedia.org/wiki/PCI_Express).


### Ссылки

- [PCI configuration space](https://en.wikipedia.org/wiki/PCI_configuration_space).
- [Peripheral Component Interconnect (PCI)](https://en.wikipedia.org/wiki/PCI).
- [PCI Express (Peripheral Component Interconnect Express, PCIe)](https://en.wikipedia.org/wiki/PCI_Express).
- [The Linux Documentation Project / The Linux Kernel / Chapter 6 PCI](https://tldp.org/LDP/tlk/dd/pci.html).
- [Accessing PCI Express Configuration Registers Using Intel® Chipsets](https://web.archive.org/web/20240205000601/http://www.csit-sun.pub.ro/~cpop/Documentatie_SMP/Intel_Microprocessor_Systems/Intel_ProcessorNew/Intel%20White%20Paper/Accessing%20PCI%20Express%20Configuration%20Registers%20Using%20Intel%20Chipsets.pdf).


### Идентификатор PCI--устройства

В этой задаче вам нужно реализовать метод
[`pci::device_id::DeviceId::new()`](../../doc/pci/device_id/struct.DeviceId.html#method.new):
```rust
{{#include ../../pci/src/device_id.rs:new}}
```

И вспомогательную функцию
[`pci::device_id::vendor_device()`](../../doc/pci/device_id/fn.vendor_device.html):
```rust
{{#include ../../pci/src/device_id.rs:vendor_device}}
```

Они находятся в файле
[`pci/src/device_id.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/device_id.rs).

Тесты можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib device_id -- --test-threads 1 --no-capture)
```


#### Ориентировочный объём работ этой задачи

```console
pci/src/device_id.rs | 47 +++++++++++++++++++++++++++++++++++++++++------
1 file changed, 41 insertions(+), 6 deletions(-)
```


### Класс PCI--устройства

В этой задаче вам нужно реализовать метод
[`pci::class::Class::new()`](../../doc/pci/class/struct.Class.html#method.new):
```rust
{{#include ../../pci/src/class.rs:new}}
```

Он находится в файле
[`pci/src/class.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/class.rs).

Тесты можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib class -- --test-threads 1 --no-capture)
```


#### Ориентировочный объём работ этой задачи

```console
pci/src/class.rs | 31 ++++++++++++++++++++++++++++---
1 file changed, 28 insertions(+), 3 deletions(-)
```


### Base Address Register (BAR)

В этой задаче вам нужно реализовать метод
[`pci::bar::Bar::new()`](../../doc/pci/bar/enum.Bar.html#method.new):
```rust
{{#include ../../pci/src/bar.rs:new}}
```

И вспомогательный метод
[`pci::bar::Bar::address_size()`](../../doc/pci/bar/enum.Bar.html#method.address_size):
```rust
{{#include ../../pci/src/bar.rs:address_size}}
```

Они находятся в файле
[`pci/src/bar.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/bar.rs).

Также нужно реализовать метод
[`pci::device::Device::read_bars()`](../../doc/pci/device/struct.Device.html#method.read_bars):
```rust
{{#include ../../pci/src/device.rs:read_bars}}
```

Он находится в файле
[`pci/src/device.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/device.rs).

Тесты можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib bar -- --test-threads 1 --no-capture)
```


#### Ориентировочный объём работ этой задачи

```console
pci/src/bar.rs    | 81 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----
pci/src/device.rs | 15 +++++++++++++--
2 files changed, 90 insertions(+), 6 deletions(-)
```


### Обычное (конечное) PCI--устройство

В этой задаче вам нужно доработать метод
[`pci::device::Device::new()`](../../doc/pci/device/struct.Device.html#method.new):
```rust
{{#include ../../pci/src/device.rs:new}}
```

Он находятся в файле
[`pci/src/device.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/device.rs).

Для идентификатора и производителя подустройства вам пригодится вспомогательная функция
[`pci::device_id::vendor_device()`](../../doc/pci/device_id/fn.vendor_device.html).

Тесты можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib normal -- --test-threads 1 --no-capture)
```


#### Ориентировочный объём работ этой задачи

```console
pci/src/device.rs | 26 +++++++++++++++++++++-----
1 file changed, 21 insertions(+), 5 deletions(-)
```


### PCI--PCI мост

В этой задаче вам нужно доработать метод
[`pci::device::Device::new()`](../../doc/pci/device/struct.Device.html#method.new):
```rust
{{#include ../../pci/src/device.rs:new}}
```

И реализовать вспомогательный метод
[`pci::device::Device::read_bridge()`](../../doc/pci/device/struct.Device.html#method.read_bridge):
```rust
{{#include ../../pci/src/device.rs:read_bridge}}
```

Они находится в файле
[`pci/src/device.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/pci/src/device.rs).

Тесты можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib bridge -- --test-threads 1 --no-capture)
```


#### Ориентировочный объём работ этой задачи

```console
pci/src/device.rs | 57 +++++++++++++++++++++++++++++++++++++++++++++++++++++++--
1 file changed, 55 insertions(+), 2 deletions(-)
```


### Запуск тестов

Полный набор тестов для задач про шину PCI можно запустить так:
```console
/.../nikka$ (cd pci; cargo t --lib -- --test-threads 1 --no-capture)
```


### Ориентировочный объём работ этой части лабораторки

```console
pci/src/bar.rs       |   81 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++----
pci/src/class.rs     |   31 +++++++++++++++++++++++++---
pci/src/device.rs    |   98 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++---------
pci/src/device_id.rs |   47 +++++++++++++++++++++++++++++++++++++------
4 files changed, 235 insertions(+), 22 deletions(-)
```
