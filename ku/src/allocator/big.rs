use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::NonNull,
};

use crate::{
    Error::{InvalidAlignment, InvalidArgument, Overflow}, error::Result, memory::{
        Block,
        Page,
        Virt,
        mmu::PageTableFlags,
        size::SizeOf,
    },
};

use super::{
    DryAllocator,
    Initialize,
};

// Used in docs.
#[allow(unused)]
use {
    crate::error::Error,
    core::alloc::Allocator,
};

/// Интерфейс аллокатора памяти общего назначения,
/// который умеет выдавать только выровненные на границу страниц блоки памяти.
/// Работает либо в текущем адресном пространстве,
/// либо в другом адресном пространстве.
///
/// # Safety
///
/// Код, который реализует этот типаж должен гарантировать,
/// что инварианты управления памятью в Rust'е не будут нарушены.
/// В частности, [`BigAllocator::reserve()`] не должен выдавать
/// занятые в данный момент блоки виртуальных адресов.
pub unsafe trait BigAllocator {
    /// Возвращает текущие флаги отображения страниц.
    ///
    /// Они используются при выделении памяти из [`BigAllocator`] через интерфейсы
    /// [`core::alloc::Allocator`] или [`DryAllocator`],
    /// которые не принимают на вход флагов отображений страниц.
    fn flags(&self) -> PageTableFlags;

    /// Устанавливает текущие флаги отображения страниц.
    ///
    /// Они используются при выделении памяти из [`BigAllocator`] через интерфейсы
    /// [`core::alloc::Allocator`] или [`DryAllocator`],
    /// которые не принимают на вход флагов отображений страниц.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::PermissionDenied`] если запрошенные флаги
    ///     не допускаются этим аллокатором.
    fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) -> Result<()>;

    /// Выделяет новый блок подряд идущих виртуальных страниц.
    /// Достаточный для хранения объекта, размер и выравнивание которого описывается `layout`.
    /// Ни выделения физической памяти, ни создания отображения станиц, не происходит.
    ///
    /// - Если выделить заданный размер виртуальной памяти не удалось,
    ///   возвращает ошибку [`Error::NoPage`].
    fn reserve(
        &mut self,
        layout: Layout,
    ) -> Result<Block<Page>>;

    /// Выделяет блок `block` виртуальной памяти.
    /// Ни выделения физической памяти, ни создания отображения станиц, не происходит.
    ///
    /// - Возвращает ошибку [`Error::NoPage`],
    ///   если выделить заданный блок виртуальной памяти не удалось.
    ///   То есть, если он содержит хотя бы одну уже занятую страницу.
    fn reserve_fixed(
        &mut self,
        block: Block<Page>,
    ) -> Result<()>;

    /// Освобождает блок виртуальных страниц `block`.
    ///
    /// # Safety
    ///
    /// - `block` должен был быть ранее выделен с помощью [`BigAllocator::reserve()`]
    ///   или [`BigAllocator::reserve_fixed()`].
    /// - Отображения этих станиц в физическую память уже не должно быть.
    /// - Вызывающий код должен гарантировать,
    ///   что инварианты управления памятью в Rust'е не будут нарушены.
    ///   В частности, не осталось ссылок, которые ведут в `block`.
    unsafe fn unreserve(
        &mut self,
        block: Block<Page>,
    ) -> Result<()>;

    /// Выделяет нужное количество физических фреймов
    /// и отображает в них заданный блок виртуальных страниц `block` с флагами `flags`.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::NoFrame`] если свободных физических фреймов не осталось.
    ///   - [`Error::PermissionDenied`] если запрошенные флаги
    ///     не допускаются этим аллокатором.
    ///
    /// # Safety
    ///
    /// - `block` должен был быть ранее выделен с помощью [`BigAllocator::reserve()`]
    ///   или [`BigAllocator::reserve_fixed()`].
    /// - Вызывающий код должен гарантировать, что инварианты управления памятью в Rust'е
    ///   не будут нарушены.
    ///   В частности, не осталось ссылок, которые ведут в `block`.
    unsafe fn map(
        &mut self,
        block: Block<Page>,
        flags: PageTableFlags,
    ) -> Result<()>;

    /// Удаляет отображение заданного блока виртуальных страниц `block`.
    /// Физические фреймы, на которые не осталось других ссылок, освобождаются.
    /// После работы [`BigAllocator::unmap()`] виртуальные адреса `block`
    /// становятся недоступны.
    ///
    /// # Safety
    ///
    /// - `block` должен был быть ранее отображён с помощью [`BigAllocator::map()`]
    ///   или [`BigAllocator::copy_mapping()`].
    /// - Вызывающий код должен гарантировать, что инварианты управления памятью в Rust'е
    ///   не будут нарушены.
    ///   В частности, не осталось ссылок, которые ведут в `block`.
    unsafe fn unmap(
        &mut self,
        block: Block<Page>,
    ) -> Result<()>;

    /// Копирует отображение физических фреймов из `old_block` в `new_block`.
    /// Если изначально `new_block` содержал отображённые страницы,
    /// их отображение удаляется.
    /// Физические фреймы, на которые не осталось других ссылок, освобождаются.
    /// А содержимое памяти, которое ранее было доступно через `old_block`,
    /// становится доступным и через `new_block`.
    ///
    /// Параметр `flags` задаёт флаги доступа к страницам `new_block`:
    ///   - [`None`] --- использовать те же флаги, что и в `old_block`,
    ///     индивидуально для каждой страницы.
    ///   - [`Some`] --- использовать для всех страниц флаги `flags`.
    ///
    /// В случае совпадения `old_block` и `new_block`, необходимо задать флаги `flags`.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::InvalidArgument`] если `old_block` и `new_block` имеют разный размер.
    ///   - [`Error::InvalidArgument`] если `old_block` и `new_block` пересекаются, но
    ///     не совпадают. Или совпадают и при этом `flags == None`.
    ///     (Ситуация смены флагов на одном и том же блоке является единственной допустимой
    ///     при пересечении `old_block` и `new_block`).
    ///   - [`Error::PermissionDenied`] если запрошенные флаги
    ///     не допускаются этим аллокатором.
    ///
    /// # Safety
    ///
    /// - `old_block` должен был быть ранее отображён с помощью [`BigAllocator::map()`]
    ///   или [`BigAllocator::copy_mapping()`].
    /// - `new_block` должен был быть ранее выделен
    ///   с помощью [`BigAllocator::reserve()`] или [`BigAllocator::reserve_fixed()`].
    /// - Вызывающий код должен гарантировать, что инварианты управления памятью в Rust'е
    ///   не будут нарушены.
    ///   В частности, не возникнет неуникальных изменяемых ссылок.
    unsafe fn copy_mapping(
        &mut self,
        old_block: Block<Page>,
        new_block: Block<Page>,
        flags: Option<PageTableFlags>,
    ) -> Result<()>;

    /// Аналогично [`Allocator::grow()`]
    /// увеличивает ранее отображённый блок виртуальных страниц `old_block`.
    ///
    /// При `old_block.size() == 0` выделяет полный новый блок.
    /// Метод `BigAllocator::grow(Block::default(), layout, flags)` может быть использован
    /// для выделения нового блока вместо пары вызовов
    /// [`BigAllocator::reserve()`] и [`BigAllocator::map()`].
    /// А вызов `BigAllocator::shrink(block, Page::layout_array(0))` может быть использован
    /// для полного освобождения блока вместо пары вызовов
    /// [`BigAllocator::unmap()`] и [`BigAllocator::unreserve()`].
    ///
    /// # Safety
    ///
    /// - `old_block` должен был быть пустой или должен быть ранее отображён с помощью
    ///   [`BigAllocator::map()`] или [`BigAllocator::copy_mapping()`].
    /// - Вызывающий код должен гарантировать,
    ///   что инварианты управления памятью в Rust'е не будут нарушены.
    ///   В частности, не осталось ссылок, которые ведут в освобождаемые страницы.
    ///
    /// # Panics
    ///
    /// Паникует, если `new_layout.size() < old_block.size()`.
    unsafe fn grow(
        &mut self,
        old_block: Block<Page>,
        new_layout: Layout,
        flags: PageTableFlags,
    ) -> Result<Block<Page>> {
        if new_layout.size() < old_block.size() {
            panic!("trying to grow into a smaller size");
        }

        let new_block = self.reserve(new_layout)?;
        if old_block.is_empty() {
            unsafe {
                if let Err(err) = self.map(new_block, flags) {
                    let _ = self.unreserve(new_block);
                    return Err(err)
                }
            }
        } else {
            let left_sub_block = match Block::<Page>::from_index(
                new_block.start(),
                new_block.start() + old_block.count(),
            ) {
                Ok(block) => block,
                Err(err) => {
                    let _ = unsafe { self.unreserve(new_block) };
                    return Err(err)
                }
            };

            let right_sub_block = match Block::<Page>::from_index(
                new_block.start() + old_block.count(),
                new_block.end(),
            ) {
                Ok(block) => block,
                Err(err) => {
                    let _ = unsafe { self.unreserve(new_block) };
                    return Err(err)
                }
            };

            unsafe {
                if let Err(err) = self.copy_mapping(old_block, left_sub_block, Some(flags)) {
                    let _ = self.unmap(left_sub_block);
                    let _ = self.unreserve(new_block);
                    return Err(err)
                };
            }

            unsafe {
                if let Err(err) = self.map(right_sub_block, flags) {
                    let _ = self.unmap(left_sub_block);
                    let _ = self.unreserve(new_block);
                    return Err(err)
                }
            }

            unsafe {
                self.unmap(old_block).expect("failed to unmap old block");
                self.unreserve(old_block).expect("failed to unreserve old_block")
            }
        }

        Ok(new_block)
    }

    /// Аналогично [`Allocator::shrink()`]
    /// уменьшает ранее отображённый блок виртуальных страниц `old_block`.
    /// Возвращает:
    ///   - Подблок, начинающийся с того же адреса, что и `old_block`,
    ///     если начало `old_block` удовлетворяет новому выравниванию `layout.align()`.
    ///     Для станиц в конце `old_block`, не попадающих в результирующий подблок,
    ///     вызывается [`BigAllocator::unreserve()`].
    ///   - Новый блок, не пересекающийся с `old_block`,
    ///     если начало `old_block` не удовлетворяет новому выравниванию `layout.align()`.
    ///     Для `old_block` вызывается [`BigAllocator::unreserve()`].
    ///
    /// Флаги отображения страниц нового блока постранично совпадают с флагами,
    /// с которыми были отображены страницы соответствующего префикса старого блока.
    ///
    /// При `new_layout.size() == 0` полностью освобождает `old_block`.
    /// Метод `BigAllocator::grow(Block::default(), layout, flags)` может быть использован
    /// для выделения нового блока вместо пары вызовов
    /// [`BigAllocator::reserve()`] и [`BigAllocator::map()`].
    /// А вызов `BigAllocator::shrink(block, Page::layout_array(0))` может быть использован
    /// для полного освобождения блока вместо пары вызовов
    /// [`BigAllocator::unmap()`] и [`BigAllocator::unreserve()`].
    ///
    /// # Safety
    ///
    /// - `old_block` должен был быть ранее отображён с помощью [`BigAllocator::map()`]
    ///   или [`BigAllocator::copy_mapping()`].
    /// - Вызывающий код должен гарантировать,
    ///   что инварианты управления памятью в Rust'е не будут нарушены.
    ///   В частности, не осталось ссылок, которые ведут в освобождаемые страницы.
    ///
    /// # Panics
    ///
    /// Паникует, если `new_layout.size() > old_block.size()`.
    unsafe fn shrink(
        &mut self,
        old_block: Block<Page>,
        new_layout: Layout,
    ) -> Result<Block<Page>> {
        if new_layout.size() == 0 {
            if !old_block.is_empty() {
                unsafe {
                    self.unmap(old_block).expect("failed to unmap old_block");
                    self.unreserve(old_block).expect("failed to unreserve old_block");
                }
            }
            return Ok(Block::zero())
        }

        if old_block.size() < new_layout.size() {
            panic!("trying to shrink into a bigger size")
        }

        let left_sub_block = Block::<Page>::from_index(
            old_block.start(),
            old_block.start() + new_layout.size().div_ceil(Page::SIZE),
        )?;

        if old_block.address(0)?.into_usize() % new_layout.align() == 0 {
            let right_sub_block = Block::<Page>::from_index(
                old_block.start() + (new_layout.size() + Page::SIZE - 1) / Page::SIZE,
                old_block.end(),
            )?;

            if !right_sub_block.is_empty() {
                unsafe {
                    self.unmap(right_sub_block).expect("failed to unmap right_sub_block");
                    self.unreserve(right_sub_block).expect("failed to unreserve right_sub_block");
                }
            }

            Ok(left_sub_block)
        } else {
            let new_block = self.reserve(new_layout)?;

            unsafe {
                if let Err(err) = self.copy_mapping(left_sub_block, new_block, None) {
                    let _ = self.unmap(new_block);
                    let _ = self.unreserve(new_block);
                    return Err(err)
                };

                self.unmap(old_block).expect("failed to unmap old_block");
                self.unreserve(old_block).expect("failed to unreserve old_block");
            }

            Ok(new_block)
        }
    }
}

unsafe impl<T: BigAllocator> DryAllocator for T {
    fn dry_allocate(
        &mut self,
        layout: Layout,
        initialize: Initialize,
    ) -> Result<NonNull<[u8]>> {
        let block = self.reserve(layout)?;
        unsafe {
            if let Err(err) = self.map(block, self.flags()) {
                let _ = self.unreserve(block);
                return Err(err)
            }
        }

        unsafe {
            if let Err(err) = initialize_block(block, initialize) {
                let _ = self.unmap(block);
                let _ = self.unreserve(block);
                return Err(err)
            }
        };

        unsafe {
            match block.try_into_non_null_slice() {
                Ok(slice) => Ok(slice),
                Err(err) => {
                    let _ = self.unmap(block);
                    let _ = self.unreserve(block);
                    return Err(err)
                }
            }
        }
    }

    unsafe fn dry_deallocate(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
    ) {
        let block = try_into_block(ptr, layout)
            .expect("error while casting ptr to block");
        
        unsafe {
            self.unmap(block).expect("failed to unmap block");
            self.unreserve(block).expect("failed to unreserve block");
        }
    }

    unsafe fn dry_grow(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        initialize: Initialize,
    ) -> Result<NonNull<[u8]>> {
        if new_layout.size() < old_layout.size() {
            return Err(InvalidArgument)
        }

        let old_block = try_into_block(ptr, old_layout)?;
        if ptr.addr().get() % new_layout.align() == 0 {
            let pages_count = new_layout.size().div_ceil(Page::SIZE);

            if old_block.count() == pages_count {
                unsafe {
                    initialize_block_part(
                        old_block,
                        old_layout.size(),
                        old_block.size(),
                        initialize
                    )?;
                    return old_block.try_into_non_null_slice()
                }
            }
        }

        let grow_layout = Layout::from_size_align(
            new_layout.size().max(old_block.size()),
            new_layout.align(),
        ).map_err(|_| InvalidArgument)?;

        let new_block = unsafe {
            self.grow(old_block, grow_layout, self.flags())?
        };

        unsafe {
            initialize_block_part(
                new_block,
                old_layout.size(),
                new_block.size(),
                initialize
            ).expect("failed to initialize block")
        };

        unsafe {
            Ok(new_block.try_into_non_null_slice().expect("failed to cut block into slice"))
        }
    }

    unsafe fn dry_shrink(
        &mut self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>> {
        if new_layout.size() > old_layout.size() {
            return Err(InvalidArgument)
        }

        let old_block = try_into_block(ptr, old_layout)?;

        if new_layout.size() == 0 {
            let empty = NonNull::slice_from_raw_parts(new_layout.dangling_ptr(), 0);

            unsafe {
                self.shrink(old_block, new_layout)?;
            }

            return Ok(empty)
        }

        if ptr.addr().get() % new_layout.align() == 0 {
            let pages_count = new_layout.size().div_ceil(Page::SIZE);

            if old_block.count() == pages_count {
                return unsafe { old_block.try_into_non_null_slice() }
            }
        }
        
        let new_block = unsafe {
            self.shrink(old_block, new_layout)?
        };

        unsafe {
            Ok(new_block.try_into_non_null_slice().expect("failed to cut block into slice"))
        }
    }
}

/// Типаж, который позволяет захватывать [`BigAllocator`].
pub trait BigAllocatorGuard {
    /// Захватывает [`BigAllocator`] и возвращает его,
    /// возможно обёрнутым во вспомогательный объект.
    ///
    /// Если это требуется в реализации,
    /// может захватывать блокировку на возвращаемый [`BigAllocator`].
    /// В этом случае, блокировка освобождается при вызове [`Drop::drop()`]
    /// для возвращённого объекта.
    fn get(&self) -> impl BigAllocator;
}

/// Преобразует `ptr` и `layout` в соответствующий блок страниц.
///
/// Возвращает ошибки:
///   - [`Error::InvalidAlignment`], если `ptr` не выровнен на границу страницы.
///   - [`Error::Overflow`] или [`Error::InvalidArgument`], если получающийся блок
///     пересекает границу одной из половин адресного пространства.
fn try_into_block(
    ptr: NonNull<u8>,
    layout: Layout,
) -> Result<Block<Page>> {
    if ptr.addr().get() % Page::SIZE != 0 {
        return Err(InvalidAlignment)
    }

    let start_index = ptr.addr().get() / Page::SIZE;
    let pages_count = layout.size().div_ceil(Page::SIZE);

    let block = Block::<Page>::from_index(
        start_index,
        start_index.checked_add(pages_count).ok_or(Overflow)?,
    )?;
    Ok(block)
}

/// Инициализирует `block` так, как предписывает `initialize`.
///
/// # Safety
///
/// - `block` должен быть отображён в память.
/// - Вызывающий код должен гарантировать,
///   что инварианты управления памятью в Rust'е не будут нарушены.
///   В частности, нет ссылок, которые ведут в `block`.
unsafe fn initialize_block(
    block: Block<Page>,
    initialize: Initialize,
) -> Result<()> {
    if initialize == Initialize::Zero {
        let slice = unsafe { block.try_into_mut_slice::<MaybeUninit<u8>>()? };
        for item in slice.iter_mut() {
            *item = MaybeUninit::zeroed();
        }
    }

    Ok(())
}

unsafe fn initialize_block_part(
    block: Block<Page>,
    start: usize,
    end: usize,
    initialize: Initialize,
) -> Result<()> {
    if start > end || end > block.size() {
        return Err(InvalidArgument)
    }

    if initialize == Initialize::Zero {
        let slice = unsafe { block.try_into_mut_slice::<MaybeUninit<u8>>()? };
        for index in start..end {
            slice[index] = MaybeUninit::zeroed();
        }
    }

    Ok(())
}
