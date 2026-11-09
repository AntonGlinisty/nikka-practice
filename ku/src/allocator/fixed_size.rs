use core::{
    alloc::Layout,
    cmp,
    hint,
    mem,
    ptr,
};

use crate::{
    allocator::BigAllocatorGuard,
    collections::DynamicBitmap,
    error::{
        Error::NoPage,
        Result,
    },
    log::{
        error,
        trace,
    },
    memory::Virt,
};

use super::{
    CLIP_SIZE,
    Clip,
    Info,
    Quarry,
};

/// Аллокатор, выделяющий память блоками одинакового размера.
///
/// Предполагает, что все блоки памяти, с которыми он работает,
/// имеют одинаковый размер.
/// Более того, этот размер и выравнивание блоков должны быть кратны [`MIN_SIZE`].
#[derive(Debug)]
pub(super) struct FixedSizeAllocator {
    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// занятости блоков из [`field@FixedSizeAllocator::quarry`].
    bitmap: DynamicBitmap,

    /// Статистика аллокатора.
    info: Info,

    /// Хранилище для блоков памяти фиксированного размера.
    quarry: Option<Quarry>,
}

impl FixedSizeAllocator {
    /// Создаёт аллокатор, выделяющий память блоками одинакового размера.
    pub(super) const fn new() -> Self {
        Self {
            bitmap: DynamicBitmap::new(),
            info: Info::new(),
            quarry: None,
        }
    }

    /// Статистика аллокатора.
    pub(super) fn info(&self) -> &Info {
        &self.info
    }

    /// Статистика аллокатора.
    pub(super) fn info_mut(&mut self) -> &mut Info {
        &mut self.info
    }

    /// Возвращает `true`, если у аллокатора больше нет свободных блоков.
    pub(super) fn is_empty(&self) -> bool {
        self.bitmap.free() == 0
    }

    /// Выделяет свободный блок памяти из [`field@FixedSizeAllocator::quarry`].
    /// Возвращает указатель на выделенный блок или
    /// [`core::ptr::null_mut()`], если свободных блоков не осталось.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - [`FixedSizeAllocator`] не подходит для операции,
    ///     которую описывает `layout`.
    #[allow(unused)]
    pub(super) fn allocate(
        &mut self,
        layout: Layout,
    ) -> *mut u8 {
        self.validate_layout(layout);

        if let Some(block_index) = self.bitmap.allocate() {
            let ptr = self.quarry().allocation(block_index).into_mut_ptr_u8();

            let size = self.quarry().size();
            self.info_mut().allocation(layout.size(), size, 0);

            return ptr
        } else {
            return core::ptr::null_mut()
        }
    }

    /// Освобождает выделенные ранее методом [`FixedSizeAllocator::allocate()`]
    /// блок памяти, начинающийся по указателю `ptr`.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать
    /// [то же самое](https://doc.rust-lang.org/nightly/core/alloc/trait.Allocator.html#safety-1),
    /// что требуется при вызове [`core::alloc::Allocator::deallocate()`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Блок уже является свободным.
    ///   - Блок не попадает в [`field@FixedSizeAllocator::quarry`].
    ///   - [`FixedSizeAllocator`] не подходит для операции,
    ///     которую описывает `layout`.
    pub(super) unsafe fn deallocate(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
    ) {
        assert!(!ptr.is_null());
        self.validate_layout(layout);

        unsafe { self.deallocate_impl(ptr); }

        let size = self.quarry().size();
        self.info_mut().deallocation(layout.size(), size, 0);
    }

    /// Заполняет `clip` до тех пор, пока либо в нём не будет `len` элементов,
    /// либо не исчерпаются свободные элементы [`FixedSizeAllocator`].
    pub(super) fn fill_clip(
        &mut self,
        clip: &mut Clip,
        len: usize,
    ) {
        assert!(len <= clip.capacity());

        *self.info_mut() += core::mem::take(clip.info_mut());

        while clip.len() < len {
            let Some(index) = self.bitmap.allocate() else {
                return
            };

            let ptr = self.quarry().allocation(index).into_mut_ptr_u8();
            clip.push(ptr).expect("Failed to push ptr to clip");
        }
    }

    /// Освобождает `clip`, пока в нём не останется не более `len` блоков.
    pub(super) fn unfill_clip(
        &mut self,
        clip: &mut Clip,
        len: usize,
    ) {
        assert!(len <= clip.capacity());

        *self.info_mut() += core::mem::take(clip.info_mut());

        while clip.len() > len {
            let ptr = clip.pop().unwrap();

            unsafe { self.deallocate_impl(ptr); }
        }
    }

    /// Выделяет из `fallback` некоторое количество новых блоков размера `size`.
    /// При необходимости, расширяет память под свои метаданные ---
    /// [`FixedSizeAllocator::bitmap`].
    ///
    /// Возвращает количество физических фреймов,
    /// которые пришлось выделить из `fallback` и под запрошенные блоки и под метаданные.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - `size` не подходит под требования размера [`FixedSizeAllocator`].
    ///   - `size` не равен ранее передававшемуся в этот метод значению.
    pub(super) fn stock_up(
        &mut self,
        size: usize,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        assert!(size.is_multiple_of(MIN_SIZE));

        let old_count = self.bitmap.len();
        let mut new_count = old_count + 1;

        if self.quarry.is_none() {
            let mut quarry = Quarry::new(size);

            self.bitmap.reserve(quarry.capacity(), FixedSizeAllocator::BITMAP_GUARD_ZONE_PAGE_COUNT, fallback)?;

            self.quarry = Some(quarry);
        } else {
            if size != self.quarry().size() {
                panic!("The size is different from the previous one.");
            }
        }

        if new_count > self.quarry().capacity() {
            return Err(NoPage)
        }

        let quarry_frames = self.quarry().map(old_count, &mut new_count, fallback)?;
        let bitmap_frames = match self.bitmap.map(new_count, fallback) {
            Ok(frames) => frames,
            Err(err) => {
                self.quarry()
                    .unmap_part(old_count, new_count, fallback)
                    .expect("failed to roll back Quarry expansion");
                return Err(err)
            },
        };

        self.info_mut().pages_allocation(quarry_frames + bitmap_frames);

        Ok(quarry_frames + bitmap_frames)
    }

    /// Освобождает как всю использованную [`FixedSizeAllocator`] физическую память,
    /// так и всё использованное им адресное пространство.
    /// Возвращает их в страничный аллокатор `fallback`.
    /// Переводит [`FixedSizeAllocator::bitmap`] и [`field@FixedSizeAllocator::quarry`]
    /// в неинициализированное состояние,
    /// такое же как создаваемое методом [`FixedSizeAllocator::new()`].
    /// Но накопленные статистики [`FixedSizeAllocator::info`] при этом не сбрасываются.
    ///
    /// Возвращает количество освобождённых физических фреймов.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Не все блоки этого [`FixedSizeAllocator`] освобождены.
    pub fn unmap(
        &mut self,
        fallback: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        assert_eq!(self.info.allocations().balance(), 0);
        assert_eq!(self.info.allocated().balance(), 0);
        assert_eq!(self.info.requested().balance(), 0);

        if self.quarry.is_none() {
            return Ok(0)
        }

        let allocation_count = self.bitmap.len();
        
        let bitmap_frames = self.bitmap.unmap(fallback).expect("Failed to unmap bitmap");
        let quarry_frames = self.quarry().unmap(allocation_count, fallback).expect("Failed to unmap quarry");

        self.quarry = None;

        self.info_mut().pages_deallocation(quarry_frames + bitmap_frames);

        Ok(quarry_frames + bitmap_frames)
    }

    /// Освобождает выделенные ранее методом [`FixedSizeAllocator::allocate()`]
    /// блок памяти, начинающийся по указателю `ptr`.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать
    /// [то же самое](https://doc.rust-lang.org/nightly/core/alloc/trait.Allocator.html#safety-1),
    /// что требуется при вызове [`core::alloc::Allocator::deallocate()`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Блок уже является свободным.
    ///   - Блок не попадает в [`field@FixedSizeAllocator::quarry`].
    pub(super) unsafe fn deallocate_impl(
        &mut self,
        ptr: *mut u8,
    ) {
        debug_assert!(!ptr.is_null());

        let address = Virt::from_mut_ptr(ptr);
        if let Some(number) = self.quarry().allocation_index(address) {
            if self.bitmap.is_free(number) {
                panic!("Block is already free");
            }
            self.bitmap.set_free(number);
        } else {
            panic!("The address does not belong to quarry");
        }
    }

    /// Возвращает хранилище блоков памяти фиксированного размера.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - [`field@FixedSizeAllocator::quarry`] ещё не инициализирован,
    ///     то есть ещё не было сделано ни одной операции выделения или освобождения памяти.
    fn quarry(&mut self) -> &mut Quarry {
        self.quarry.as_mut().expect(Self::UNINITIALIZED_MESSAGE)
    }

    /// Проверяет, что [`FixedSizeAllocator`] для размера `size`
    /// поддерживает выделение блоков памяти заданного `layout`.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - [`FixedSizeAllocator`] не подходит для операции,
    ///     которую описывает `layout`.
    fn validate_layout(
        &self,
        layout: Layout,
    ) {
        let size = self.quarry.as_ref().expect(Self::UNINITIALIZED_MESSAGE).size();
        debug_assert!(layout.size() <= size);
        debug_assert!(size.is_multiple_of(layout.align()));
    }

    /// Количество страниц в каждой из не отображённых в память защитных зон,
    /// которыми битовая карта [`FixedSizeAllocator::bitmap`]
    /// окружается в адресном пространстве с обоих сторон.
    /// Это необходимо, чтобы защитить метаданные аллокатора от перезаписи неверным
    /// использующим аллокатор кодом, обращающимся за пределы выданного ему блока памяти.
    const BITMAP_GUARD_ZONE_PAGE_COUNT: usize = 1;

    /// Сообщение для паник, когда [`FixedSizeAllocator`] ещё не инициализирован.
    const UNINITIALIZED_MESSAGE: &str = "FixedSizeAllocator is not initialized";
}

impl Drop for FixedSizeAllocator {
    fn drop(&mut self) {
        assert!(self.bitmap.is_empty());
        assert!(self.quarry.is_none());
    }
}

/// Максимальный тип, подходящий для зануления и копирования блоков памяти,
/// которыми управляет [`FixedSizeAllocator`].
pub(super) type MaxPrimitiveType = u64;

/// Минимальный размер и выравнивание для блоков памяти,
/// которыми управляет [`FixedSizeAllocator`].
pub(super) const MIN_SIZE: usize = mem::size_of::<MaxPrimitiveType>();

#[doc(hidden)]
pub(super) mod test_scaffolding {
    pub const MIN_SIZE: usize = super::MIN_SIZE;
}
