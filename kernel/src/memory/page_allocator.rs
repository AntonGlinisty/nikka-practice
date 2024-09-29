use core::ops::Range;

use itertools::Itertools;

use crate::{
    error::{
        Error::{
            InvalidArgument,
            NoPage,
        },
        Result,
    },
    log::{
        error,
        info,
        trace,
    },
    time,
};

use super::{
    PAGES_PER_ROOT_LEVEL_ENTRY,
    block::Block,
    frage::Page,
    mmu::PageTable,
    size::SizeOf,
    stack::Stack,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Простой аллокатор виртуальных страниц адресного пространства.
#[derive(Debug, Default)]
pub(super) struct PageAllocator {
    /// Блок свободных виртуальных страниц.
    block: Block<Page>,

    /// Первоначальное состояние блока [`PageAllocator::block`], сразу после отведения
    /// защитной зоны и до первого выделения виртуальных страниц вызывающему коду.
    /// Используется при дублировании аллокатора в методе [`PageAllocator::duplicate()`].
    initial_block: Block<Page>,
}

impl PageAllocator {
    /// Инициализирует аллокатор [`PageAllocator`]
    /// большим блоком подряд идущих свободных виртуальных страниц
    /// по информации из корневого узла таблицы страниц `page_table_root`.
    /// При этом `allowed_elements` задаёт диапазон записей корневого узла таблицы страниц,
    /// который можно использовать данному аллокатору.
    pub(super) fn new(
        page_table_root: &PageTable,
        allowed_elements: Range<usize>,
    ) -> Self {
        let timer = time::timer();

        if let Some(block) = Self::find_unused_block(page_table_root, allowed_elements) {
            let mut page_allocator = Self {
                block,
                initial_block: Block::default(),
            };

            // Detect stack overruns that corrupt memory allocated by PageAllocator.
            page_allocator
                .allocate(Stack::GUARD_ZONE_SIZE)
                .expect("failed to create guard zone for PageAllocator");

            page_allocator.initial_block = page_allocator.block;

            info!(
                free_page_count = page_allocator.block.count(),
                block = %page_allocator.block,
                duration = %timer.elapsed(),
                "page allocator init",
            );

            page_allocator
        } else {
            error!(?page_table_root);
            panic!(
                "failed to find a free entry in the page table root for the virtual page allocator",
            );
        }
    }

    /// Возвращает пустой [`PageAllocator`].
    /// В отличие от [`PageAllocator::default()`] доступна в константном контексте.
    pub(super) const fn zero() -> Self {
        Self {
            block: Block::zero(),
            initial_block: Block::zero(),
        }
    }

    /// Создаёт копию [`PageAllocator`] на момент его инициализации.
    pub(super) fn duplicate(&self) -> Self {
        Self {
            block: self.initial_block,
            initial_block: self.initial_block,
        }
    }

    /// Устанавливает [`PageAllocator`] в состояние эквивалентное `original`.
    /// Текущий [`PageAllocator`] должен был быть получен
    /// из `original` методом [`PageAllocator::duplicate()`].
    /// И из него не должно было быть выделено больше страниц, чем из `original`.
    /// Если это не так, возвращается ошибка [`Error::InvalidArgument`].
    pub(super) fn duplicate_allocator_state(
        &mut self,
        original: &Self,
    ) -> Result<()> {
        if self.block.contains_block(original.block) {
            self.block = original.block;
            Ok(())
        } else {
            Err(InvalidArgument)
        }
    }

    /// Выделяет блок подряд идущих виртуальных страниц, достаточный для хранения объекта
    /// размером `size` **байт**.
    /// Ни выделения физической памяти, ни создания отображения станиц, не происходит.
    ///
    /// Если выделить заданный размер виртуальной памяти не удалось,
    /// возвращает ошибку [`Error::NoPage`].
    pub(super) fn allocate(
        &mut self,
        size: usize,
    ) -> Result<Block<Page>> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает блок виртуальных страниц,
    /// соответствующий самой длинной последовательности пустых записей в
    /// диапазоне `allowed_elements` корневого узла таблицы страниц `page_table_root`.
    ///
    /// Для упрощения не смотрит в другие узлы таблицы страниц,
    /// и резервирует только страницы, соответствующие полностью свободным
    /// записям таблицы страниц корневого уровня.
    ///
    /// Если все записи заняты, возвращает `None`.
    fn find_unused_block(
        page_table_root: &PageTable,
        allowed_elements: Range<usize>,
    ) -> Option<Block<Page>> {
        // TODO: your code here.
        unimplemented!();
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use core::ops::Range;

    use super::{
        super::{
            Block,
            Page,
            mmu::PageTable,
        },
        PageAllocator,
    };

    pub(in super::super) fn block(page_allocator: &PageAllocator) -> Block<Page> {
        page_allocator.block
    }

    pub fn find_unused_block(
        page_table_root: &PageTable,
        allowed_elements: Range<usize>,
    ) -> Option<Block<Page>> {
        PageAllocator::find_unused_block(page_table_root, allowed_elements)
    }
}
