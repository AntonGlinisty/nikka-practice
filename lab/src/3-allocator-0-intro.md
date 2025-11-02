## Аллокатор памяти общего назначения

В этой лабораторке нужно будет реализовать аллокатор памяти общего назначения.
Чтобы можно было пользоваться, например, стандартными контейнерами.
Только первая задача является обязательной.
От остальных задач почти никакие последующие лабораторные работы не зависят.
Исключение --- `static-bitmap`, которая пригодится нам при реализации файловой системы.

В первой, обязательной, задаче вам нужно будет провязать
Rust'овский интерфейс аллокатора памяти общего назначения с реализованными во
[второй лабораторной работе](../../lab/book/2-mm-0-intro.html)
методами адресного пространства
[`kernel::memory::address_space::AddressSpace`](../../doc/kernel/memory/address_space/struct.AddressSpace.html).

После этого станет доступна
[`alloc`](https://doc.rust-lang.org/nightly/alloc/index.html)
часть стандартной библиотеки.
И в последующих задачах лабораторок мы сможем использовать стандартные контейнеры, такие как
[`alloc::vec::Vec`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html)
и
[`alloc::collections::vec_deque::VecDeque`](https://doc.rust-lang.org/nightly/alloc/collections/vec_deque/struct.VecDeque.html).
Что гораздо удобнее, чем методы семейства
[`AddressSpace::map_slice()`](../../doc/kernel/memory/address_space/struct.AddressSpace.html#method.map_slice).

Однако, при каждом динамическом выделении памяти будет расходоваться целое количество физических фреймов.
Даже если запрошено было всего несколько байт.
Это безумно расточительно, и в последующих задачах этой лабораторной работы нужно будет
сделать библиотеку динамического выделения памяти общего назначения.
Которая тратит меньше памяти при выделениях памяти маленького размера.
Она получает память фреймами из сделанного в первой задаче интерфейса,
нарезает полученную память небольшими блоками и динамически выделяет эти блоки.
В результате в один физический фрейм чаще всего попадают данные большого числа выделенных блоков
и физическая память расходуется экономно.

Дизайн библиотеки динамической памяти --- задача достаточно творческая.
Вы можете пойти по пути, проложенному в необязательных задачах.
Или выбрать собственный путь и реализовать аллокатор так, как вам хочется.
При этом можно вдохновляться дизайном хороших современных аллокаторов:

- Thread-caching malloc (Google):
  - [Обзор дизайна TCMalloc](https://google.github.io/tcmalloc/design.html).
  - [Temeraire: Hugepage-Aware Allocator](https://google.github.io/tcmalloc/temeraire.html).
  - [Beyond malloc efficiency to fleet efficiency: a hugepage-aware memory allocator](https://storage.googleapis.com/gweb-research2023-media/pubtools/6170.pdf).
  - [TCMalloc : Thread-Caching Malloc](https://pages.cs.wisc.edu/~danb/google-perftools-0.98/tcmalloc.html).
- Jason Evans' malloc:
  - [A Scalable Concurrent malloc(3) Implementation for FreeBSD](https://people.freebsd.org/~jasone/jemalloc/bsdcan2006/jemalloc.pdf).
  - [https://engineering.fb.com/2011/01/03/core-data/scalable-memory-allocation-using-jemalloc/](https://engineering.fb.com/2011/01/03/core-data/scalable-memory-allocation-using-jemalloc/).
- Mimalloc (Microsoft):
  - [Mimalloc: Free List Sharding in Action](https://www.microsoft.com/en-us/research/wp-content/uploads/2019/06/mimalloc-tr-v1.pdf).
- Threading Building Blocks (Intel)
  - [The Foundations for Scalable Multi-core Software in Intel® Threading Building Blocks](http://www.intel.com/content/dam/www/public/us/en/documents/research/2007-vol11-iss-4-intel-technology-journal.pdf).
- Ещё полезные ссылки:
  - [Hoard: A Scalable Memory Allocator for Multithreaded Applications](https://www.cs.utexas.edu/~mckinley/papers/asplos-2000.pdf).
  - [Scalable Lock-Free Dynamic Memory Allocation](https://www.cs.tufts.edu/~nr/cs257/archive/maged-michael/pldi-2004.pdf).
  - [Scalable Locality-Conscious Multithreaded Memory Allocation](https://pureadmin.qub.ac.uk/ws/files/3429262/ISMM_2006.pdf).
  - Обзор старых аллокаторов --- [Dynamic Storage Allocation: A Survey and Critical Review](https://www.cs.hmc.edu/~oneill/gc-library/Wilson-Alloc-Survey-1995.pdf).
  - [На ARM можно занулить линию кэша без чтения её из памяти](https://www.systemonchips.com/cortex-a53-cache-line-allocation-without-memory-read-solutions-and-workarounds/).

Наша реализация будет доступна и в ядре и в пространстве пользователя.
Помимо прочего, это упрощает её тестирование.
Так как при запуске тестов `ku/tests/3-allocator-*.rs`
вам будут доступны все возможности тестирования пользовательского приложения под linux.


### Зависимости задач этой лабораторной работы

![Зависимости задач этой лабораторной работы](3-allocator-0-intro-lab-dependencies.svg)


### Ориентировочный объём работ этой лабораторки

```console
kernel/src/allocator/big.rs          |  23 +++++++-
ku/src/allocator/big.rs              | 108 +++++++++++++++++++++++++++++++++++-----
ku/src/allocator/dispatcher.rs       | 198 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
ku/src/allocator/fixed_size.rs       | 147 +++++++++++++++++++++++++++++++++++++++++++++++++------
ku/src/allocator/quarry.rs           | 182 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-----
ku/src/collections/bitmap.rs         |  40 +++++++++++++--
ku/src/collections/dynamic_bitmap.rs |  86 +++++++++++++++++++++++++++++---
7 files changed, 723 insertions(+), 61 deletions(-)
```
