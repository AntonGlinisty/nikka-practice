use core::{
    alloc::{
        Allocator,
        GlobalAlloc,
        Layout,
    },
    cmp,
    fmt,
    ptr::{
        self,
        NonNull,
    },
};

use crate::{
    memory::{
        Block,
        Page,
    },
    sync::Spinlock,
};

use super::{
    dry::Initialize,
    fixed_size::{
        FixedSizeAllocator,
        MIN_SIZE,
    },
    info::{
        AtomicInfo,
        Info,
    },
};

/// Аллокатор верхнего уровня.
/// По запрошенному размеру определяет из какого аллокатора будет выделяться память.
#[derive(Debug)]
pub struct Dispatcher<T: Allocator> {
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

impl<T: Allocator> Dispatcher<T> {
    /// Создаёт аллокатор верхнего уровня с заданным постраничным аллокатором `fallback`.
    pub const fn new(fallback: T) -> Self {
        Self {
            fallback,
            fallback_info: AtomicInfo::new(),
            fixed_size: [FIXED_SIZE_DUMMY; FIXED_SIZE_COUNT],
            info: AtomicInfo::new(),
        }
    }

    /// Общая статистика аллокатора.
    pub fn info(&self) -> Info {
        self.info.load()
    }

    /// Находит индекс [`FixedSizeAllocator`], который отвечает за заданный `layout`.
    ///
    /// Возвращает [`None`], если такого нет.
    /// То есть, если за этот `layout` отвечает [`Dispatcher::fallback`].
    fn get_fixed_size_index(
        &self,
        layout: Layout,
    ) -> Option<usize> {
        // TODO: your code here.
        None // TODO: remove before flight.
    }

    /// Возвращает размер блоков памяти, за которые отвечает аллокатор
    /// по индексу `index` в массиве [`Dispatcher::fixed_size`].
    fn get_size(index: usize) -> usize {
        (index + 1) * MIN_SIZE
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
        // TODO: your code here.
        unimplemented!();
    }

    /// Проверяет, что `layout` поддерживается.
    ///
    /// # Panics
    ///
    /// Паникует, если `layout` не поддерживается.
    fn validate_layout(layout: Layout) {
        assert_ne!(
            layout.size(),
            0,
            "can not handle zero-sized types requested by {layout:?}",
        );

        assert!(
            layout.align() <= Page::SIZE,
            "can not handle alignment of {layout:?} that is greater than the page size {}",
            Page::SIZE,
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

unsafe impl<T: Allocator> GlobalAlloc for Dispatcher<T> {
    unsafe fn alloc(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        Self::validate_layout(layout);

        // TODO: your code here.

        self.fallback_allocate(layout, Initialize::Garbage)
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        Self::validate_layout(layout);

        // TODO: your code here.

        let ptr = NonNull::new(ptr).expect("should not get null ptr in dealloc()");

        self.fallback.deallocate(ptr, layout);

        self.update_fallback_info(Operation::Deallocation, layout);
    }

    unsafe fn alloc_zeroed(
        &self,
        layout: Layout,
    ) -> *mut u8 {
        Self::validate_layout(layout);

        // TODO: your code here.

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

        // TODO: your code here.

        let old_ptr = NonNull::new(old_ptr).expect("should not get null ptr in realloc()");

        let new_ptr = if old_layout.size() < new_layout.size() {
            self.fallback.grow(old_ptr, old_layout, new_layout)
        } else {
            self.fallback.shrink(old_ptr, old_layout, new_layout)
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

impl<T: Allocator> fmt::Display for Dispatcher<T> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ total: {}, fallback: {}",
            self.info(),
            self.fallback_info.load(),
        )?;

        for (index, fixed_size) in self.fixed_size.iter().enumerate() {
            let fixed_size = fixed_size.lock().info();

            if fixed_size.allocations().positive() > 0 {
                let size = Self::get_size(index);
                write!(formatter, ", size_{}: {}", size, fixed_size)?;
            }
        }

        write!(formatter, " }}")
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

/// Количество аллокаторов фиксированного размера [`FixedSizeAllocator`],
/// которыми управляет [`Dispatcher`].
const FIXED_SIZE_COUNT: usize = (4 * Page::SIZE - 1) / MIN_SIZE;

/// Вспомогательная константа для инициализации массива [`Dispatcher::fixed_size`]
/// в константной функции [`Dispatcher::new()`].
/// Необходима, так как [`Spinlock`] не может быть помечен типажом [`Copy`].
#[allow(clippy::declare_interior_mutable_const)]
const FIXED_SIZE_DUMMY: Spinlock<FixedSizeAllocator> = Spinlock::new(FixedSizeAllocator::new());
