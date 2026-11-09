use core::{
    alloc::Layout, fmt, num::NonZeroUsize, ops::Range,
};

use static_assertions::const_assert;

use crate::{
    allocator::test_scaffolding::SLAB_SIZE, error::Result, log::trace, memory::{
        Block,
        Page,
        Virt,
        size::{
            MiB,
            Size,
        },
    },
};

use super::{
    BigAllocator,
    BigAllocatorGuard,
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
    ///
    /// Нумерация непрерывна от `0` до [`Quarry::capacity()`] (не включительно) и
    /// последовательно пробегает выделяемые блоки памяти
    /// из каждого большого блока [`Quarry::slabs`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - `allocation_index` больше или равен [`Quarry::capacity()`].
    pub(super) fn allocation(
        &mut self,
        allocation_index: usize,
    ) -> Virt {
        let (slab_index, block_index) = self.allocation_coordinates(allocation_index);
        if slab_index >= self.slabs.len() {
            panic!("Trying to allocate out of quarry capacity");
        }

        let slab_block = self.slab(slab_index);
        if slab_block == Block::default() {
            panic!("Trying to allocate at default slab");
        }

        slab_block.address(block_index * self.size()).unwrap()
    }

    /// Обратная операция к [`Quarry::allocation()`].
    /// Возвращает номер выделяемого блока памяти по его адресу `address` или [`None`]
    /// если выделяемого блока памяти с таким адресом в [`Quarry`] нет.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Смещение `address` от начала его [`Quarry::slab()`] не выровнено на [`Quarry::size()`].
    pub(super) fn allocation_index(
        &mut self,
        address: Virt,
    ) -> Option<usize> {
        for index in 0..self.slabs.len() {
            let slab_block = self.slab(index);
            if slab_block == Block::default() {
                return None
            }

            let block_offset = match slab_block.offset(address) {
                Ok(offset) => offset,
                Err(_) => continue,
            };

            if block_offset % self.size() != 0 {
                panic!("Address is not alligned to block start");
            }

            if Self::SLAB_SIZE - block_offset < self.size() {
                return None
            }

            return Some(block_offset / self.size() + index * self.slab_allocation_count.get());
        }

        None
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

        let mut result = 0;
        let mut alloc = fallback.get();

        let (new_slab_index, new_block_index) = self.allocation_coordinates(*new_allocation_count - 1);
        let (old_slab_index, old_block_index) = if old_allocation_count == 0 { (0, 0) } else {
            self.allocation_coordinates(old_allocation_count - 1)
        };

        let page_start = if old_allocation_count == 0 { 0 } else {
            ((old_block_index + 1) * self.size()).div_ceil(Page::SIZE)
        };
        let page_end = ((new_block_index + 1) * self.size()).div_ceil(Page::SIZE);

        let page_range = |index| -> Range<usize> {
            let start = if index == old_slab_index {
                page_start
            } else {
                0
            };

            let end = if index == new_slab_index {
                page_end
            } else {
                Self::SLAB_PAGE_COUNT
            };

            start..end
        };

        for slab_index in old_slab_index..=new_slab_index {
            let range = page_range(slab_index);
            if range.is_empty() {
                continue;
            }
            let page_count = range.len();

            if let Err(err) = unsafe {
                alloc.map(
                    self.slab(slab_index).slice(range).unwrap(),
                    alloc.flags()
                )
            } {
                for index in old_slab_index..slab_index {
                    let range = page_range(index);
                    if range.is_empty() {
                        continue;
                    }

                    unsafe {
                        alloc
                            .unmap(self.slab(index).slice(range).unwrap())
                            .expect("failed to roll back slab mapping")
                    };
                }

                let old_slab_count = old_allocation_count.div_ceil(self.slab_allocation_count.get());
                for index in old_slab_count..=new_slab_index {
                    unsafe {
                        alloc.unreserve(self.slab(index))
                            .expect("failed to roll back slab reservation");
                    }
                    self.slabs[index] = Page::default();
                }

                return Err(err)
            };

            result += page_count;
        }

        *new_allocation_count = new_slab_index * self.slab_allocation_count.get()
            + (page_end * Page::SIZE / self.size()).min(self.slab_allocation_count.get());

        Ok(result)
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
        assert!(allocation_count <= self.capacity());

        let slab_count = allocation_count.div_ceil(self.slab_allocation_count.get());

        for index in 0..Self::SLAB_COUNT {
            if index < slab_count &&
                self.slab(index) == Block::default() {
                    panic!("Slabs allocated less than needed");
                }

            if index >= slab_count &&
                self.slab(index) != Block::default() {
                    panic!("Slabs allocated more than needed");
                }
        }

        let mut result = 0;
        let mut alloc = fallback.get();

        let block_index = if allocation_count == 0 {
            0
        } else {
            (allocation_count - 1) % self.slab_allocation_count.get()
        };
        let page_end = ((block_index + 1) * self.size()).div_ceil(Page::SIZE); 

        for index in (0..slab_count).rev() {
            let left_bound = 0;
            let right_bound = if index == slab_count - 1 {
                page_end
            } else {
                Self::SLAB_PAGE_COUNT
            };

            unsafe {
                alloc
                    .unmap(self.slab(index).slice(left_bound..right_bound).unwrap())
                    .expect("failed to unmap Quarry slab");
                alloc
                    .unreserve(self.slab(index))
                    .expect("failed to unreserve Quarry slab");
            }

            self.slabs[index] = Page::default();
            result += right_bound - left_bound;
        }

        Ok(result)
    }

    pub(super) fn unmap_part(
        &mut self,
        old_allocation_count: usize,
        new_allocation_count: usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        assert!(old_allocation_count <= new_allocation_count);
        assert!(new_allocation_count <= self.capacity());

        if old_allocation_count == new_allocation_count {
            return Ok(0)
        }

        let old_slab_count = old_allocation_count.div_ceil(self.slab_allocation_count.get());
        
        let (first_slab, first_page) = if old_allocation_count == 0 {
            (0, 0)
        } else {
            let (slab, block) =
                self.allocation_coordinates(old_allocation_count - 1);

            (slab, ((block + 1) * self.size()).div_ceil(Page::SIZE))
        };

        let (last_slab, last_block) =
            self.allocation_coordinates(new_allocation_count - 1);
        let last_page =
            ((last_block + 1) * self.size()).div_ceil(Page::SIZE);

        let mut result = 0;
        let mut alloc = fallback.get();

        for slab_index in (first_slab..=last_slab).rev() {
            let start = if slab_index == first_slab {
                first_page
            } else {
                0
            };
            let end = if slab_index == last_slab {
                last_page
            } else {
                Self::SLAB_PAGE_COUNT
            };

            if start < end {
                unsafe {
                    alloc
                        .unmap(self.slab(slab_index).slice(start..end).unwrap())
                        .expect("failed to unmap Quarry slab");
                }

                result += end - start;
            }

            if slab_index >= old_slab_count {
                unsafe {
                    alloc
                        .unreserve(self.slab(slab_index))
                        .expect("failed to unreserve Quarry slab");
                }
                self.slabs[slab_index] = Page::default();
            }
        }

        Ok(result)
    }

    /// Возвращает пару из номера большого блока в [`Quarry::slabs`] и
    /// номера с нуля выделяемого блока памяти внутри этого большого блока,
    /// которая соответствует глобальному в [`Quarry`] номеру `allocation_index`.
    /// См. [`Quarry::allocation()`].
    fn allocation_coordinates(
        &self,
        allocation_index: usize,
    ) -> (usize, usize) {
        let slab_index = allocation_index / self.slab_allocation_count;
        let block_index = allocation_index % self.slab_allocation_count;

        (slab_index, block_index)
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
    ///   - `new_allocation_count` не превышает `old_allocation_count`.
    fn reserve(
        &mut self,
        old_allocation_count: usize,
        new_allocation_count: usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<()> {
        assert!(old_allocation_count < new_allocation_count);

        if new_allocation_count > self.capacity() {
            panic!("Trying to reserve more blocks than capacity");
        }

        let old_slab_count = old_allocation_count.div_ceil(self.slab_allocation_count.get());

        for slab_index in 0..Self::SLAB_COUNT {
            if slab_index < old_slab_count &&
                self.slab(slab_index) == Block::default() {
                    panic!("Slabs allocated less than needed");
                }

            if slab_index >= old_slab_count &&
                self.slab(slab_index) != Block::default() {
                    panic!("Slabs allocated more than needed");
                }
        }

        let new_slab_count = new_allocation_count.div_ceil(self.slab_allocation_count.get());

        for slab_index in old_slab_count..new_slab_count {
            let layout = Layout::from_size_align(Self::SLAB_SIZE, Page::SIZE)?;
            let new_block = match fallback.get().reserve(layout) {
                Ok(block) => block,
                Err(err) => {
                    for index in old_slab_count..slab_index {
                        unsafe {
                            fallback.get().unreserve(self.slab(index))
                                .expect("failed to roll back slab reservation");
                        }
                        self.slabs[index] = Page::default();
                    }
                    return Err(err)
                }
            };
            self.slabs[slab_index] = new_block.start_element();
        }

        Ok(())
    }

    /// Возвращает блок номер `slab_index` или [`Block::default()`],
    /// если `slab_index` выходит за границы [`Quarry::slabs`].
    fn slab(
        &self,
        slab_index: usize,
    ) -> Block<Page> {
        Self::slab_at(self.slabs[slab_index])
    }

    /// Возвращает блок, начинающийся со страницы `start` или [`Block::default()`],
    /// если `start` равен [`Page::default()`].
    fn slab_at(start: Page) -> Block<Page> {
        if start == Page::default() {
            Block::default()
        } else {
            unsafe { Block::from_count_unchecked(start, Self::SLAB_PAGE_COUNT) }
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

const_assert!(Quarry::SLAB_SIZE.is_multiple_of(Page::SIZE));

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
