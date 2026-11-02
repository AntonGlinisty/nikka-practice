use core::alloc::Layout;

use derive_more::{
    Deref,
    DerefMut,
};

use crate::{
    Error::InvalidArgument, allocator::{
        BigAllocator,
        BigAllocatorGuard,
    }, error::{
        Error::{
            InvalidAlignment,
            PermissionDenied,
        },
        Result,
    }, memory::{
        Block,
        Page,
        size,
    },
};

use super::Bitmap;

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
/// расширяемого размера
/// для отслеживания какие именно элементы заняты, а какие --- свободны.
#[derive(Debug, Deref, DerefMut)]
pub struct DynamicBitmap {
    /// Битовая карта текущего фиксированного размера.
    #[deref]
    #[deref_mut]
    bitmap: Bitmap<'static>,

    /// Количество страниц в каждой из не отображённых в память защитных зон,
    /// которыми битовая карта окружается в адресном пространстве с обеих сторон.
    guard_zone_page_count: usize,

    /// Зарезервированный под битовую карту и её защитные зоны блок страниц.
    /// В память отображена только часть этого блока,
    /// достаточная для хранения [`DynamicBitmap::bitmap`] текущего размера.
    pages: Block<Page>,
}

#[allow(rustdoc::private_intra_doc_links)]
impl DynamicBitmap {
    /// Возвращает пустой [`DynamicBitmap`].
    /// В отличие от [`DynamicBitmap::default()`] доступна в константном контексте.
    pub const fn new() -> Self {
        Self {
            bitmap: Bitmap::zero(),
            guard_zone_page_count: 0,
            pages: Block::zero(),
        }
    }

    /// С помощью `allocator` резервирует в адресном пространстве
    /// блок страниц [`field@DynamicBitmap::pages`], достаточный для хранения
    /// битовой карты [`DynamicBitmap::bitmap`] на `capacity` элементов (бит).
    ///
    /// Битовая карта окружается в адресном пространстве с обоих сторон не отображёнными в память
    /// защитными зонами размера `guard_zone_page_count` страниц.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::NoPage`] если выделить необходимый размер виртуальной памяти не удалось.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Метод [`DynamicBitmap::reserve()`] уже вызывался, то есть [`DynamicBitmap`] не пуст.
    ///   - Ёмкость `capacity == 0`.
    pub fn reserve(
        &mut self,
        capacity: usize,
        guard_zone_page_count: usize,
        allocator: &impl BigAllocatorGuard,
    ) -> Result<()> {
        if capacity == 0 {
            panic!("Trying to reserve pages for 0 capacity bitmap");
        }

        if !self.pages.is_empty() {
            panic!("Current pages are not empty");
        }

        let guard_pages_count = guard_zone_page_count * Page::SIZE;
        let bitmap_pages_count = Self::page_count(capacity) * Page::SIZE;

        let layout = Layout::from_size_align(2 * guard_pages_count + bitmap_pages_count, Page::SIZE)?;
        let result_pages = allocator.get().reserve(layout)?;

        self.bitmap = Bitmap::zero();
        self.guard_zone_page_count = guard_zone_page_count;
        self.pages = result_pages;

        Ok(())
    }

    /// С помощью `allocator` отображает в память
    /// часть блока страниц [`field@DynamicBitmap::pages`], достаточную для хранения
    /// битовой карты [`DynamicBitmap::bitmap`] на `new_len` элементов (бит).
    /// Новые элементы битовой карты инициализируются нулями.
    /// Страницы отображаются с флагами `allocator.flags()`.
    ///
    /// Возвращает количество выделенных физических фреймов памяти.
    ///
    /// Возвращает ошибку:
    ///   - [`Error::NoFrame`] если свободных физических фреймов не осталось.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Новый размер `new_len` не превышает старый размер [`DynamicBitmap::bitmap`].
    pub fn map(
        &mut self,
        new_len: usize,
        allocator: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        if new_len <= self.bitmap.len() {
            panic!("Trying to map pages for new_len <= current_len bitmap");
        }

        let old_len = self.bitmap.len();
        let pages = self.pages();

        if Self::page_count(new_len) != Self::page_count(old_len) {
            let mut alloc = allocator.get();

            let new_pages = pages
                .slice(Self::page_count(old_len)..Self::page_count(new_len))
                .unwrap();

            unsafe {
                alloc.map(new_pages, alloc.flags())?;

                new_pages
                    .start_address()
                    .into_mut_ptr_u8()
                    .write_bytes(0, new_pages.size());
            };
        }

        let new_free = self.bitmap.free() + (new_len - old_len);

        let mapped_pages = pages.slice(0..Self::page_count(new_len)).unwrap();
        let data: &mut [u64] = unsafe { mapped_pages.try_into_mut_slice::<u64>()? };

        self.bitmap = Bitmap::new(data, new_len, Some(new_free));

        Ok(Self::page_count(new_len) - Self::page_count(old_len))
    }

    /// С помощью `allocator`:
    ///   - Удаляет отображение в память битовой карты [`DynamicBitmap::bitmap`].
    ///   - Разрезервирует блок [`field@DynamicBitmap::pages`],
    ///     который был ранее зарезервирован в адресном пространстве.
    ///
    /// После этого присваивает `self` пустое значение,
    /// которое возвращает [`DynamicBitmap::new()`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - Не все элементы свободны,
    ///     то есть для поля [`DynamicBitmap::bitmap`] значения
    ///     [`Bitmap::free()`] и [`Bitmap::len()`] различаются.
    pub fn unmap(
        &mut self,
        allocator: &impl BigAllocatorGuard,
    ) -> Result<usize> {
        assert_eq!(self.free(), self.len());

        let mut alloc = allocator.get();

        let mapped_pages_count = Self::page_count(self.bitmap.len());
        let pages_to_unmap = self.pages().slice(0..mapped_pages_count).unwrap();
        
        unsafe { alloc.unmap(pages_to_unmap)? };
        self.bitmap = Bitmap::zero();

        unsafe { alloc.unreserve(self.pages)? };

        *self = DynamicBitmap::new();
        Ok(mapped_pages_count)
    }

    /// Возвращает количество страниц, которые занимает битовая карта [`DynamicBitmap::bitmap`]
    /// на `len` элементов (бит).
    fn page_count(len: usize) -> usize {
        len.div_ceil(size::from(u8::BITS) * Page::SIZE)
    }

    /// Возвращает зарезервированный под битовую карту блок страниц.
    fn pages(&self) -> Block<Page> {
        self.pages
            .slice(self.guard_zone_page_count .. self.pages.count() - self.guard_zone_page_count)
            .expect("failed to make guard zones")
    }
}

impl Default for DynamicBitmap {
    fn default() -> Self {
        Self::new()
    }
}
