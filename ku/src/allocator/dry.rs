use core::{
    alloc::Layout,
    ptr::NonNull,
};

use crate::error::Result;

// Used in docs.
#[allow(unused)]
use core::alloc::Allocator;

/// Аналогичен [`core::alloc::Allocator`].
///
/// Но при выделении нового и увеличении ранее
/// выделенного блока памяти принимает аргумент типа [`Initialize`],
/// чтобы не дублировать код
/// ([DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)).
pub unsafe trait DryAllocator {
    /// Выделяет память как [`Allocator::allocate()`] и [`Allocator::allocate_zeroed()`].
    /// Зануляет память как [`Allocator::allocate_zeroed()`], если
    /// `initialize` равен [`Initialize::Zero`].
    fn dry_allocate(
        &mut self,
        layout: Layout,
        initialize: Initialize,
    ) -> Result<NonNull<[u8]>>;

    /// Освобождает память как [`Allocator::deallocate()`].
    unsafe fn dry_deallocate(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
    );

    /// Увеличивает выделенный блок памяти как [`Allocator::grow()`] и [`Allocator::grow_zeroed()`].
    /// Зануляет новую память как [`Allocator::grow_zeroed()`], если
    /// `initialize` равен [`Initialize::Zero`].
    unsafe fn dry_grow(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        initialize: Initialize,
    ) -> Result<NonNull<[u8]>>;

    /// Уменьшает выделенный блок памяти как [`Allocator::shrink()`].
    unsafe fn dry_shrink(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>>;
}

/// Необходимо ли инициализировать выделяемую память.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Initialize {
    /// Инициализировать выделяемую память не требуется.
    Garbage,

    /// Выделяемую память нужно инициализировать нулями.
    Zero,
}
