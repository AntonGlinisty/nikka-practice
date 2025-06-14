use core::{
    alloc::Layout,
    fmt,
    num::NonZeroUsize,
};

use static_assertions::const_assert_eq;

use crate::{
    allocator::{
        BigAllocator,
        BigAllocatorGuard,
    },
    error::Result,
    log::trace,
    memory::{
        Block,
        Page,
        Virt,
        size::{
            MiB,
            Size,
        },
    },
};

// Used in docs.
#[allow(unused)]
use super::FixedSizeAllocator;

/// Вспомогательная структура для [`FixedSizeAllocator`].
///
/// Хранит [`Quarry::SLAB_COUNT`] больших блоков размера [`Quarry::SLAB_SIZE`],
/// разбитых на выделяемые блоки памяти меньшего размера [`Quarry::size()`].
/// Резервирует в адресном пространстве большие блоки целиком при необходимости.
/// Причём последний используемый блок отображается на физическую память не полностью, ---
/// а только используемый начальный отрезок его станиц.
///
/// Не помнит количество отображённых на физическую память страниц в последнем блоке.
/// Эта информация при необходимости передаётся в методы [`Quarry`] из
/// владеющего им [`FixedSizeAllocator`].
#[derive(Debug)]
pub(super) struct Quarry {
    /// Размер выделяемых блоков памяти, на которые разбивается каждый большой блок.
    size: NonZeroUsize,

    /// Количество выделяемых блоков памяти в одном большом блоке.
    slab_allocation_count: NonZeroUsize,

    /// Первые страницы больших блоков.
    /// Каждый большой блок либо пуст и тогда его первая страница равна `Page::default()`,
    /// либо содержит [`Self::SLAB_PAGE_COUNT`] последовательных страниц.
    slabs: [Page; Self::SLAB_COUNT],
}

impl Quarry {
    /// Создаёт пустой [`Quarry`], из которого будут выделяться блоки размера `size`.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - `size` равен нулю или превышает размер блока [`Quarry::SLAB_SIZE`].
    pub(super) const fn new(size: usize) -> Self {
        let size = NonZeroUsize::new(size).expect("size is zero");
        let slab_allocation_count =
            NonZeroUsize::new(Self::SLAB_SIZE / size.get()).expect("size is too big");

        Self {
            size,
            slab_allocation_count,
            slabs: [Page::zero(); Self::SLAB_COUNT],
        }
    }

    /// Возвращает размер выделяемых блоков памяти, на которые разбивается каждый большой блок.
    pub(super) fn size(&self) -> usize {
        self.size.get()
    }

    /// Возвращает адрес выделяемого блока памяти номер `allocation_index`.
    /// Если `allocation_index` попадает в ещё не зарезервированный в памяти большой блок,
    /// либо больше или равен [`Quarry::capacity()`],
    /// возвращает [`Virt::default()`].
    ///
    /// Нумерация непрерывна от `0` до [`Quarry::capacity()`] (не включительно) и
    /// последовательно пробегает выделяемые блоки памяти
    /// из каждого большого блока [`Quarry::slabs`].
    pub(super) fn allocation(
        &mut self,
        allocation_index: usize,
    ) -> Virt {
        // TODO: your code here.
        unimplemented!();
    }

    /// Обратная операция к [`Quarry::allocation()`].
    /// Возвращает номер выделяемого блока памяти по его адресу `address` или [`None`]
    /// если выделяемого блока памяти с таким адресом в [`Quarry`] нет.
    pub(super) fn allocation_index(
        &mut self,
        address: Virt,
    ) -> Option<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает максимальное количество выделяемых блоков памяти,
    /// которое может содержаться в этом [`Quarry`].
    pub(super) fn capacity(&mut self) -> usize {
        self.slab_allocation_count.get() * self.slabs.len()
    }

    /// Отображает в физическую память необходимое количество страниц блоков [`Quarry::slabs`],
    /// с помощью страничного аллокатора `fallback`.
    ///
    /// Параметр `old_allocation_count` задаёт старое количество выделяемых блоков памяти,
    /// под которое страницы уже были ранее отображены в физическую память этим же методом.
    /// Соответствующие страницы отображать не нужно.
    /// Параметр `new_allocation_count` задаёт новое количество выделяемых блоков памяти,
    /// под которое нужно отобразить страницы, которые ещё не отображены.
    /// Перед отображением резервирует целиком блоки [`Quarry::slabs`],
    /// затронутые `new_allocation_count` выделяемыми блоками памяти.
    /// Страницы отображаются с флагами `fallback.flags()`.
    ///
    /// Так как отображает страницы в физическую память целиком, может получиться так,
    /// что отображённых страниц хватает больше чем
    /// на `new_allocation_count` выделяемых блоков памяти.
    /// В этом случае при успехе обновляет параметр `new_allocation_count`,
    /// чтобы вызывающая функция могла это учесть.
    ///
    /// Возвращает количество новых отображённых физических фреймов,
    /// на которое расширилась доступная этому [`Quarry`] память.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Обнаруживает, что уже зарезервировано меньше или больше блоков, чем требуется для
    ///     `old_allocation_count` выделяемых блоков памяти.
    ///   - `new_allocation_count` больше или равен [`Quarry::capacity()`].
    ///   - `new_allocation_count` не превышает `old_allocation_count`.
    pub(super) fn map(
        &mut self,
        old_allocation_count: usize,
        new_allocation_count: &mut usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        assert!(old_allocation_count < *new_allocation_count);
        assert!(*new_allocation_count <= self.capacity());

        self.reserve(old_allocation_count, *new_allocation_count, fallback)?;

        // TODO: your code here.
        unimplemented!();
    }

    /// Освобождает как всю использованную [`Quarry`] физическую память,
    /// так и всё использованное им адресное пространство.
    /// Возвращает их в страничный аллокатор `fallback`.
    ///
    /// Параметр `allocation_count` задаёт количество выделяемых блоков памяти,
    /// под которое была зарезервирована память в [`Quarry`].
    ///
    /// Возвращает количество освобождённых физических фреймов.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Обнаруживает, что уже зарезервировано меньше или больше блоков, чем требуется для
    ///     `allocation_count` выделяемых блоков памяти.
    ///   - `allocation_count` больше `[Quarry::capacity()]`.
    pub(super) fn unmap(
        &mut self,
        allocation_count: usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает пару из номера большого блока в [`Quarry::slabs`] и
    /// номера с нуля выделяемого блока памяти внутри этого большого блока,
    /// которая соответствует глобальному в [`Quarry`] номеру `allocation_index`.
    /// См. [`Quarry::allocation()`].
    fn allocation_coordinates(
        &self,
        allocation_index: usize,
    ) -> (usize, usize) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Резервирует адресное пространство под необходимое количество блоков [`Quarry::slabs`],
    /// с помощью страничного аллокатора `fallback`.
    /// Параметр `old_allocation_count` задаёт старое количество выделяемых блоков памяти,
    /// под которое большие блоки уже были ранее зарезервированы этим же методом.
    /// Соответствующие большие блоки резервировать не нужно.
    /// Параметр `new_allocation_count` задаёт новое количество выделяемых блоков памяти,
    /// под которое нужно зарезервировать большие блоки, которые ещё не зарезервированы.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Обнаруживает, что уже зарезервировано меньше или больше блоков, чем требуется для
    ///     `old_allocation_count` выделяемых блоков памяти.
    ///   - `new_allocation_count` больше или равен [`Quarry::capacity()`].
    ///   - `old_allocation_count` превышает `new_allocation_count`.
    fn reserve(
        &mut self,
        old_allocation_count: usize,
        new_allocation_count: usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<()> {
        assert!(old_allocation_count < new_allocation_count);

        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает блок номер `slab_index` или [`Block::default()`],
    /// если `slab_index` выходит за границы [`Quarry::slabs`].
    fn slab(
        &self,
        slab_index: usize,
    ) -> Block<Page> {
        if slab_index < self.slabs.len() {
            Self::slab_at(self.slabs[slab_index])
        } else {
            Block::default()
        }
    }

    /// Возвращает блок, начинающийся со страницы `start` или [`Block::default()`],
    /// если `start` равен [`Page::default()`].
    fn slab_at(start: Page) -> Block<Page> {
        if start == Page::default() {
            Block::default()
        } else {
            let end = (start + Self::SLAB_PAGE_COUNT).expect(Self::BAD_SIZE_MESSAGE);

            Block::new(start, end).expect(Self::BAD_SIZE_MESSAGE)
        }
    }

    /// Возвращает количество отображённых на физическую память страниц в большом блоке,
    /// в котором выделена память на `mapped_allocation_count` выделяемых блоков памяти.
    fn slab_page_count(
        &self,
        mapped_allocation_count: usize,
    ) -> usize {
        (mapped_allocation_count * self.size()).div_ceil(Page::SIZE)
    }

    /// Возвращает суммарный доступный размер выделяемых блоков памяти,
    /// отображённых на физическую память, --- `mapped_allocation_count`.
    fn total_size(
        &self,
        mapped_allocation_count: usize,
    ) -> Size {
        Size::bytes(self.size() * mapped_allocation_count)
    }

    /// Сообщение для паник, когда [`Quarry`] обнаруживает нарушение своих инвариантов.
    const BAD_SIZE_MESSAGE: &str = "bad Quarry size constants or calculations";

    /// Максимальное количество блоков в [`Quarry::slabs`], из которых выделяется память.
    const SLAB_COUNT: usize = 16;

    /// Полное количество страниц в каждом блоке [`Quarry::slabs`], из которых выделяется память.
    const SLAB_PAGE_COUNT: usize = Self::SLAB_SIZE / Page::SIZE;

    /// Размер каждого блока в [`Quarry::slabs`], из которых выделяется память.
    const SLAB_SIZE: usize = MiB;
}

impl fmt::Display for Quarry {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let active_slab_count =
            self.slabs.iter().position(|&start| start == Page::default()).unwrap_or(0);

        write!(
            formatter,
            "{{ slab_allocation_count: {:?}, active_slab_count: {}, active_slabs: [",
            self.slab_allocation_count, active_slab_count,
        )?;

        let mut separator = "";
        for slab_index in 0 .. active_slab_count {
            write!(formatter, "{separator}{}", self.slab(slab_index))?;
            separator = ", ";
        }

        write!(formatter, "] }}")
    }
}

impl Drop for Quarry {
    fn drop(&mut self) {
        assert!(self.slabs.iter().all(|&start| start == Page::default()));
    }
}

const_assert_eq!(Quarry::SLAB_SIZE % Page::SIZE, 0);

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use crate::{
        allocator::BigAllocatorGuard,
        error::Result,
        memory::Virt,
    };

    pub struct Quarry(super::Quarry);

    impl Quarry {
        pub const fn new(size: usize) -> Self {
            Self(super::Quarry::new(size))
        }

        pub fn allocation(
            &mut self,
            allocation_index: usize,
        ) -> Virt {
            self.0.allocation(allocation_index)
        }

        pub fn allocation_index(
            &mut self,
            address: Virt,
        ) -> Option<usize> {
            self.0.allocation_index(address)
        }

        pub fn capacity(&mut self) -> usize {
            self.0.capacity()
        }

        pub fn map(
            &mut self,
            old_allocation_count: usize,
            new_allocation_count: &mut usize,
            fallback: &impl BigAllocatorGuard,
        ) -> Result<usize> {
            self.0.map(old_allocation_count, new_allocation_count, fallback)
        }

        pub fn unmap(
            &mut self,
            allocation_count: usize,
            fallback: &impl BigAllocatorGuard,
        ) -> Result<usize> {
            self.0.unmap(allocation_count, fallback)
        }
    }

    pub const SLAB_SIZE: usize = super::Quarry::SLAB_SIZE;
}
