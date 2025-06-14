use core::alloc::Layout;

use derive_more::{
    Deref,
    DerefMut,
};

use crate::{
    allocator::{
        BigAllocator,
        BigAllocatorGuard,
    },
    error::{
        Error::{
            InvalidAlignment,
            PermissionDenied,
        },
        Result,
    },
    memory::{
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
        // TODO: your code here.
        unimplemented!();
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
        // TODO: your code here.
        unimplemented!();
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

        // TODO: your code here.
        unimplemented!();
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
