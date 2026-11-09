use core::{
    alloc::{
        Allocator,
        GlobalAlloc,
        Layout,
    }, cmp, fmt, hint, intrinsics, ptr::{
        self, NonNull, null_mut,
    },
};

use crate::{
    allocator::Initialize::Garbage, error::Result, memory::Page, sync::Spinlock,
};

use super::{
    BigAllocatorGuard,
    cache::Cache,
    dry::Initialize,
    fixed_size::{
        FixedSizeAllocator,
        MIN_SIZE,
        MaxPrimitiveType,
    },
    info::{
        AtomicInfo,
        Info,
    },
};

/// Аллокатор верхнего уровня.
/// По запрошенному размеру определяет из какого аллокатора будет выделяться память.
#[derive(Debug)]
pub struct Dispatcher<Q: Cache, T: Allocator + BigAllocatorGuard> {
    /// Кэш выделяемых блоков.
    cache: Q,

    /// Аллокатор для выделения памяти постранично, в том числе остальным аллокаторам.
    fallback: T,

    /// Статистика выделений и освобождений через [`Dispatcher::fallback`].
    fallback_info: AtomicInfo,

    /// Набор аллокаторов для разных размеров блоков.
    /// Каждый из них выделяет память блоками фиксированного размера.
    fixed_size: [Spinlock<FixedSizeAllocator>; FIXED_SIZE_COUNT],

    /// Общая статистика аллокатора.
    info: AtomicInfo,
}

impl<Q: Cache, T: Allocator + BigAllocatorGuard> Dispatcher<Q, T> {
    /// Создаёт аллокатор верхнего уровня с заданным постраничным аллокатором `fallback`.
    pub const fn new(
        cache: Q,
        fallback: T,
    ) -> Self {
        Self {
            cache,
            fallback,
            fallback_info: AtomicInfo::new(),
            fixed_size: [const { Spinlock::new(FixedSizeAllocator::new()) }; FIXED_SIZE_COUNT],
            info: AtomicInfo::new(),
        }
    }

    /// Общая статистика аллокатора.
    pub fn info(&self) -> Info {
        self.info.load()
    }

    /// Записывает в `detailed_info` статистику аллокатора.
    /// Как суммарную, так и с разбивкой по компонентам.
    ///
    /// В случае конкурентных запросов к аллокатору статистика может быть несогласованной.
    pub fn detailed_info(
        &self,
        detailed_info: &mut DetailedInfo,
    ) {
        detailed_info.total = self.info();
        detailed_info.fallback = self.fallback_info.load();

        for (index, info) in detailed_info.fixed_size.iter_mut().enumerate() {
            if Q::CACHE_AVAILABLE {
                self.cache.with_borrow_mut(index, |clip| {
                    let mut allocator = self.fixed_size[index].lock();
                    *allocator.info_mut() += core::mem::take(clip.info_mut());
                    *info = *allocator.info();
                });
            } else {
                *info = *self.fixed_size[index].lock().info();
            }
        }
    }

    /// Освобождает всю виртуальную и физическую память, выделенную аллокатором.
    ///
    /// # Panics
    ///
    /// Паникует, если остались не освобождённые блоки памяти.
    pub fn unmap(&self) {
        let info = self.info();
        assert_eq!(info.allocations().balance(), 0);
        assert_eq!(info.allocated().balance(), 0);
        assert_eq!(info.requested().balance(), 0);

        let fallback_info = self.fallback_info.load();
        assert_eq!(fallback_info.allocations().balance(), 0);
        assert_eq!(fallback_info.allocated().balance(), 0);
        assert_eq!(fallback_info.requested().balance(), 0);

        for index in 0..FIXED_SIZE_COUNT {
            if Q::CACHE_AVAILABLE {
                self.cache.with_borrow_mut(index, |clip| {
                    let mut allocator = self.fixed_size[index].lock();
                    allocator.unfill_clip(clip, 0);
                })
            }

            let mut allocator = self.fixed_size[index].lock();
            let pages_count = allocator.unmap(&self.fallback).expect("Failed to unmap fixed size allocator");
            self.info.pages_deallocation(pages_count);
        }

        assert_eq!(self.info().pages().balance(), 0);
    }

    /// Находит индекс [`FixedSizeAllocator`], который отвечает за заданный `layout`.
    ///
    /// Возвращает [`None`], если такого нет.
    /// То есть, если за этот `layout` отвечает [`Dispatcher::fallback`].
    const fn get_fixed_size_index(
        &self,
        layout: Layout,
    ) -> Option<usize> {
        let alignment = if layout.align() > MIN_SIZE {
            layout.align()
        } else {
            MIN_SIZE
        };

        let max_size = FIXED_SIZE_COUNT * MIN_SIZE;
        if layout.size() == 0 || layout.size() > max_size || alignment > max_size {
            return None;
        }

        let size = layout.size().div_ceil(alignment) * alignment;
        if  size > max_size || size % Page::SIZE == 0 {
            return None
        }

        return Some(size / MIN_SIZE - 1)
    }

    /// Возвращает размер блоков памяти, за которые отвечает аллокатор
    /// по индексу `index` в массиве [`Dispatcher::fixed_size`].
    #[allow(unused)]
    const fn get_size(index: usize) -> usize {
        get_size(index)
    }

    /// Выделяет блок памяти с помощью аллокатора `fallback`.
    fn fallback_allocate(
        &self,
        layout: Layout,
        initialize: Initialize,
    ) -> *mut u8 {
        let ptr = match initialize {
            Initialize::Garbage => self.fallback.allocate(layout),
            Initialize::Zero => self.fallback.allocate_zeroed(layout),
        };

        if let Ok(ptr) = ptr {
            self.update_fallback_info(Operation::Allocation, layout);

            ptr.as_non_null_ptr().as_ptr()
        } else {
            ptr::null_mut()
        }
    }

    /// Выделяет блок памяти с помощью аллокатора
    /// по индексу `index` в массиве [`Dispatcher::fixed_size`].
    fn fixed_size_allocate(
        &self,
        index: usize,
        layout: Layout,
    ) -> *mut u8 {
        if Q::CACHE_AVAILABLE {
            return self.cache.with_borrow_mut(index, |clip| {
                if clip.is_empty() {
                    clip.bind(&self.fixed_size[index]);

                    let mut allocator = self.fixed_size[index].lock();
                    if allocator.is_empty() {
                        match allocator.stock_up(get_size(index), &self.fallback) {
                            Ok(pages) => self.info.pages_allocation(pages),
                            Err(_) => return ptr::null_mut(),
                        }
                    }

                    allocator.fill_clip(clip, clip.capacity() / 2);
                }

                let ptr = clip.pop().expect("clip should contain blocks");

                clip.info_mut().allocation(layout.size(), get_size(index), 0);
                self.info.allocation(layout.size(), get_size(index), 0);

                ptr
            });
        }

        let mut allocator = self.fixed_size[index].lock();
        if allocator.is_empty() {
            match allocator.stock_up(get_size(index), &self.fallback) {
                Ok(pages) => self.info.pages_allocation(pages),
                Err(_) => return ptr::null_mut(),
            }
        }

        let ptr = allocator.allocate(layout);
        
        if !ptr.is_null() {
            self.info.allocation(layout.size(), get_size(index), 0);
        }

        ptr
    }

    /// Освобождает блок памяти с помощью аллокатора
    /// по индексу `index` в массиве [`Dispatcher::fixed_size`].
    ///
    /// # Safety
    ///
    /// - `ptr` должен быть ранее выделен через [`Dispatcher::fixed_size_allocate()`]
    ///   с теми же `index` и `layout`.
    /// - Память, описываемая `ptr` и `layout` больше не должна использоваться.
    unsafe fn fixed_size_deallocate(
        &self,
        index: usize,
        ptr: *mut u8,
        layout: Layout,
    ) {
        if Q::CACHE_AVAILABLE {
            return self.cache.with_borrow_mut(index, |clip| {
                if !clip.is_bound() {
                    clip.bind(&self.fixed_size[index]);
                }

                if clip.len() == clip.capacity() {
                    let mut allocator = self.fixed_size[index].lock();
                    allocator.unfill_clip(clip, clip.capacity() / 2);
                }

                clip.push(ptr).expect("clip should have space");

                clip.info_mut().deallocation(layout.size(), get_size(index), 0);
                self.info.deallocation(layout.size(), get_size(index), 0);
            })
        }

        let mut allocator = self.fixed_size[index].lock();
        unsafe { allocator.deallocate(ptr, layout) }

        self.info.deallocation(layout.size(), get_size(index), 0);
    }

    /// Проверяет, что `layout` поддерживается.
    ///
    /// # Panics
    ///
    /// Паникует, если `layout` не поддерживается.
    fn validate_layout(layout: Layout) {
        debug_assert_ne!(
            layout.size(),
            0,
            "can not handle zero-sized types requested by {layout:?}",
        );
    }

    /// Обновляет статистики [`Dispatcher::info`] и [`Dispatcher::fallback_info`],
    /// записывая в них операцию `operation` с заданным `layout`,
    /// которая была обслужена аллокатором [`Dispatcher::fallback`].
    fn update_fallback_info(
        &self,
        operation: Operation,
        layout: Layout,
    ) {
        let allocated_pages = layout.size().div_ceil(Page::SIZE);
        let allocated = allocated_pages * Page::SIZE;

        for info in [&self.fallback_info, &self.info] {
            match operation {
                Operation::Allocation => {
                    info.allocation(layout.size(), allocated, allocated_pages);
                },
                Operation::Deallocation => {
                    info.deallocation(layout.size(), allocated, allocated_pages);
                },
            }
        }
    }
}

unsafe impl<Q: Cache, T: Allocator + BigAllocatorGuard> GlobalAlloc for Dispatcher<Q, T> {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        Self::validate_layout(layout);

        let fixed_size_index = self.get_fixed_size_index(layout);
        if fixed_size_index.is_some() {
            return self.fixed_size_allocate(fixed_size_index.unwrap(), layout) 
        }

        self.fallback_allocate(layout, Initialize::Garbage)
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        Self::validate_layout(layout);

        let fixed_size_index = self.get_fixed_size_index(layout);
        if fixed_size_index.is_some() {
            unsafe {
                return self.fixed_size_deallocate(fixed_size_index.unwrap(), ptr, layout)
            }
        }

        let ptr = NonNull::new(ptr).expect("should not get null ptr in dealloc()");

        unsafe {
            self.fallback.deallocate(ptr, layout);
        }

        self.update_fallback_info(Operation::Deallocation, layout);
    }

    unsafe fn alloc_zeroed(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        Self::validate_layout(layout);

        let fixed_size_index = self.get_fixed_size_index(layout);
        if fixed_size_index.is_some() {
            let ptr = self.fixed_size_allocate(fixed_size_index.unwrap(), layout);
            if ptr != null_mut() {
                unsafe { ptr.write_bytes(0, layout.size()) };
            }
            
            return ptr
        }

        self.fallback_allocate(layout, Initialize::Zero)
    }

    unsafe fn realloc(
        &self,
        old_ptr: *mut u8,
        old_layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let new_layout = Layout::from_size_align(new_size, old_layout.align())
            .expect("bad `old_layout` or `new_size` for realloc");

        Self::validate_layout(new_layout);

        let old_index = self.get_fixed_size_index(old_layout);
        let new_index = self.get_fixed_size_index(new_layout);
        if old_index.is_some() || new_index.is_some() {
            if old_index == new_index {
                let index = old_index.unwrap();
                let class_size = get_size(index);

                let mut allocator = self.fixed_size[index].lock();
                let info = allocator.info_mut();

                self.info.deallocation(old_layout.size(), class_size, 0);
                self.info.allocation(new_size, class_size, 0);
                info.deallocation(old_layout.size(), class_size, 0);
                info.allocation(new_size, class_size, 0);

                return old_ptr
            }

            let new_ptr;
            unsafe {
                new_ptr = self.alloc(new_layout);
                if new_ptr.is_null() {
                    return new_ptr;
                }

                core::ptr::copy_nonoverlapping(old_ptr, new_ptr, core::cmp::min(old_layout.size(), new_layout.size()));

                self.dealloc(old_ptr, old_layout)
            }

            return new_ptr
        }

        let old_ptr = NonNull::new(old_ptr).expect("should not get null ptr in realloc()");

        let new_ptr = if old_layout.size() < new_layout.size() {
            unsafe { self.fallback.grow(old_ptr, old_layout, new_layout) }
        } else {
            unsafe { self.fallback.shrink(old_ptr, old_layout, new_layout) }
        };

        if let Ok(new_ptr) = new_ptr {
            self.update_fallback_info(Operation::Allocation, new_layout);
            self.update_fallback_info(Operation::Deallocation, old_layout);

            new_ptr.as_non_null_ptr().as_ptr()
        } else {
            ptr::null_mut()
        }
    }
}

impl<Q: Cache, T: Allocator + BigAllocatorGuard> Drop for Dispatcher<Q, T> {
    fn drop(&mut self) {
        self.unmap();
    }
}

/// Детальная статистика аллокатора [`Dispatcher`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetailedInfo {
    /// Общая статистика аллокатора [`Dispatcher`].
    total: Info,

    /// Статистика выделений и освобождений через [`Dispatcher::fallback`].
    fallback: Info,

    /// Статистика аллокаторов [`FixedSizeAllocator`] для разных размеров блоков.
    fixed_size: [Info; FIXED_SIZE_COUNT],
}

#[allow(rustdoc::private_intra_doc_links)]
impl DetailedInfo {
    /// Инициализирует детальную статистику аллокатора [`Dispatcher`] нулями.
    pub const fn new() -> Self {
        Self {
            total: Info::new(),
            fallback: Info::new(),
            fixed_size: [Info::new(); FIXED_SIZE_COUNT],
        }
    }

    /// Общая статистика аллокатора [`Dispatcher`].
    pub fn total(&self) -> &Info {
        &self.total
    }

    /// Статистика выделений и освобождений через [`Dispatcher::fallback`].
    pub fn fallback(&self) -> &Info {
        &self.fallback
    }

    /// Статистика аллокаторов [`FixedSizeAllocator`] для разных размеров блоков.
    pub fn fixed_size(&self) -> &[Info] {
        &self.fixed_size
    }

    /// Проверяет инварианты статистики аллокатора.
    ///
    /// Требует эксклюзивного доступа к аллокатору в момент снятия детальной статистики.
    /// Иначе из-за конкурентных операций статистики разойдутся.
    pub fn is_valid(&self) -> bool {
        if !self.total.is_valid() || !self.fallback.is_valid() {
            return false;
        }

        if let Ok(mut balance) = self.total - self.fallback {
            // `.iter()` is required to avoid copying `self.fixed_size`
            // which is too large to fit on the stack.
            // See also:
            //   - <https://github.com/rust-lang/rust/issues/45683>
            //   - <https://crates.io/crates/cargo-call-stack>
            for fixed_size in self.fixed_size.iter() {
                if !fixed_size.is_valid() {
                    return false;
                }

                if let Ok(new_balance) = balance - *fixed_size {
                    balance = new_balance;
                } else {
                    return false;
                }
            }

            balance == Info::new()
        } else {
            false
        }
    }
}

impl Default for DetailedInfo {
    fn default() -> Self {
        Self::new()
    }
}

// Avoid changing the info during formatting it due to requests to the global allocator.
impl fmt::Display for DetailedInfo {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let separator = if formatter.alternate() {
            "\n    "
        } else {
            " "
        };

        write!(
            formatter,
            "{{{0}valid: {1},{0}total: {2},{0}fallback: {3}",
            separator,
            self.is_valid(),
            self.total,
            self.fallback,
        )?;

        for (index, fixed_size) in self.fixed_size.iter().enumerate() {
            if fixed_size.allocations().positive() > 0 {
                let size = get_size(index);
                write!(formatter, ",{separator}size_{size}: {fixed_size}")?;
            }
        }

        if formatter.alternate() && self.total.allocated().positive() > 0 {
            assert!(self.total.allocations().positive() > 0);
            assert!(self.total.pages().positive() > 0);

            let total_allocations = self.total.allocations().positive() as f64;
            let total_allocated = self.total.allocated().positive() as f64;
            let total_pages = self.total.pages().positive() as f64;

            let format_ratio = |formatter: &mut fmt::Formatter, info: &Info| {
                write!(
                    formatter,
                    "{{ allocations: {:.3}%, allocated: {:.3}%, pages: {:.3}% }}",
                    info.allocations().positive() as f64 / total_allocations * 100.0,
                    info.allocated().positive() as f64 / total_allocated * 100.0,
                    info.pages().positive() as f64 / total_pages * 100.0,
                )
            };

            write!(
                formatter,
                ",{separator}allotment: {{{separator}    fallback: ",
            )?;
            format_ratio(formatter, &self.fallback)?;

            for (index, fixed_size) in self.fixed_size.iter().enumerate() {
                if fixed_size.allocations().positive() > 0 {
                    let size = get_size(index);
                    write!(formatter, ",{separator}    size_{size}: ")?;
                    format_ratio(formatter, fixed_size)?;
                }
            }

            write!(formatter, ",{separator}}}")?;
        }

        let separator = if formatter.alternate() {
            ",\n"
        } else {
            " "
        };

        write!(formatter, "{separator}}}")
    }
}

/// Операция, которая была выполнена.
#[derive(Debug)]
enum Operation {
    /// Выделение памяти.
    Allocation,

    /// Освобождение памяти.
    Deallocation,
}

/// Возвращает размер блоков памяти, за которые отвечает аллокатор
/// по индексу `index` в массиве [`Dispatcher::fixed_size`].
#[inline(always)]
const fn get_size(index: usize) -> usize {
    (index + 1) * MIN_SIZE
}

#[allow(rustdoc::private_intra_doc_links)]
/// Количество аллокаторов фиксированного размера [`FixedSizeAllocator`],
/// которыми управляет [`Dispatcher`].
pub const FIXED_SIZE_COUNT: usize = (4 * Page::SIZE - 1) / MIN_SIZE;
