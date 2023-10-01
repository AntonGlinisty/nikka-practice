use crate::error::{
    Error::{
        NoPage,
        Unimplemented,
    },
    Result,
};

use super::{
    FRAME_ALLOCATOR,
    FrameAllocator,
    Virt,
    frage::{
        Frame,
        Page,
    },
    mmu::{
        self,
        FULL_ACCESS,
        PAGE_OFFSET_BITS,
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_INDEX_MASK,
        PAGE_TABLE_LEAF_LEVEL,
        PAGE_TABLE_ROOT_LEVEL,
        PageTable,
        PageTableEntry,
        PageTableFlags,
        USER_R,
    },
    phys2virt,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Многоуровневая таблица страниц.
///
/// Фактически дерево большой арности, если игнорировать рекурсивные записи.
#[derive(Debug, Default)]
pub struct Mapping {
    /// Фрейм с корневым узлом таблицы страниц.
    page_table_root: Frame,

    /// Первая страница отображённой полностью физической памяти.
    ///
    /// Через область, которая с неё начинается, можно работать с любым участком
    /// физической памяти, не отображая его предварительно в виртуальное адресное пространство.
    /// См. [`phys2virt::map()`].
    phys2virt: Page,

    /// Номер рекурсивной записи в таблице страниц корневого уровня.
    /// Либо [`usize::MAX`], если рекурсивное отображение страниц не настроено.
    recursive_mapping: usize,
}

impl Mapping {
    /// Запоминает корневой узел таблицы страниц `page_table_root`.
    /// И начало "окна", в которое отображена вся физическая память, --- `phys2virt`.
    pub(super) fn new(
        page_table_root: Frame,
        phys2virt: Page,
    ) -> Self {
        Self {
            page_table_root,
            phys2virt,
            recursive_mapping: usize::MAX,
        }
    }

    /// Создаёт копию отображения виртуальных страниц в физические фреймы [`Mapping`],
    /// которая указывает на те же целевые физические фреймы,
    /// то есть разделяет отображённую память.
    /// Но при этом само отображение для копии и оригинала хранится в разных физических фреймах.
    /// Поэтому копия может быть модифицирована независимо от оригинала.
    pub(super) fn duplicate(&self) -> Result<Self> {
        let mut result = Self::new(Frame::default(), self.phys2virt);
        result.page_table_root =
            self.duplicate_subtree(&mut result, self.page_table_root(), PAGE_TABLE_ROOT_LEVEL)?;
        Ok(result)
    }

    /// Возвращает `true` если [`Mapping`] содержит непустое
    /// отображение виртуальной памяти в физическую.
    pub(super) fn is_valid(&self) -> bool {
        self.page_table_root() != Frame::default()
    }

    /// Возвращает корневой узел таблицы страниц отображения.
    pub(super) fn page_table_root(&self) -> Frame {
        self.page_table_root
    }

    /// Принимает на вход
    ///   - Виртуальный адрес `virt`, который нужно транслировать.
    ///   - Опциональный аллокатор фреймов `frame_allocator`.
    ///   - Флаги `flags` для промежуточных записей таблиц страниц, если их придётся создавать.
    ///
    /// Возвращает ссылку на запись типа [`PageTableEntry`] в
    /// узле листьевого уровня таблицы страниц,
    /// соответствующую входному виртуальному адресу `virt`.
    ///
    /// Если входной `frame_allocator` равен [`Some`], тогда [`Mapping::translate()`]
    /// аллоцирует с его помощью физические фреймы для недостающих узлов таблицы страниц.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::NoPage`] если промежуточного или нужного листьевого узла таблицы страниц нет,
    ///     а `frame_allocator` равен [`None`].
    ///   - [`Error::NoFrame`] если промежуточного или нужного листьевого узла таблицы страниц нет,
    ///     а `frame_allocator` не смог выделить требуемую физическую память.
    #[allow(unused_mut)] // TODO: remove before flight.
    pub fn translate(
        &mut self,
        virt: Virt,
        mut frame_allocator: Option<&mut FrameAllocator>,
        flags: PageTableFlags,
    ) -> Result<&mut PageTableEntry> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Шаг рекурсии при спуске по дереву отображения страниц.
    /// Выполняет основную работу по созданию копии отображения [`Mapping`],
    /// см. [`Mapping::duplicate()`].
    fn duplicate_subtree(
        &self,
        dst: &mut Mapping,
        src_frame: Frame,
        level: u32,
    ) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Шаг рекурсии при спуске по дереву отображения страниц.
    /// Выполняет основную работу по освобождению физических фреймов
    /// как отображённых [`Mapping`], так и занятых самим отображением.
    /// См. [`Mapping::drop()`].
    fn drop_subtree(
        &mut self,
        node: Frame,
        level: u32,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Выбирает в таблице страниц корневого уровня свободную запись,
    /// которую инициализирует как рекурсивную.
    /// Сохраняет её номер в поле [`Mapping::recursive_mapping`] и
    /// возвращает его в вызывающую функцию.
    ///
    /// Если свободных записей в таблице страниц корневого уровня нет,
    /// возвращает ошибку [`Error::NoPage`].
    pub(crate) fn make_recursive_mapping(&mut self) -> Result<usize> {
        // TODO: your code here.
        Ok(self.recursive_mapping) // TODO: remove before flight.
    }

    /// Возвращает физический фрейм корневого узла текущего отображения
    /// виртуальной памяти в физическую.
    pub(super) fn current_page_table_root() -> Frame {
        mmu::page_table_root()
    }

    /// Возвращает неизменяемую ссылку на узел таблицы страниц,
    /// записанный в данном физическом фрейме.
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что:
    ///   - Во `frame` находится узел таблицы страниц.
    ///   - Инварианты управления памятью в Rust'е не будут нарушены.
    ///     В частности, нет изменяемых ссылок, которые ведут во `frame`.
    pub(super) unsafe fn page_table_ref(
        &self,
        frame: Frame,
    ) -> &PageTable {
        phys2virt::map(self.phys2virt, frame.address())
            .try_into_ref()
            .expect("bad phys2virt or frame")
    }

    /// Возвращает изменяемую ссылку на узел таблицы страниц,
    /// записанный в данном физическом фрейме.
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что:
    ///   - Во `frame` находится узел таблицы страниц.
    ///   - Инварианты управления памятью в Rust'е не будут нарушены.
    ///     В частности, нет других ссылок, которые ведут во `frame`.
    unsafe fn page_table_mut(
        &mut self,
        frame: Frame,
    ) -> &mut PageTable {
        phys2virt::map(self.phys2virt, frame.address())
            .try_into_mut()
            .expect("bad phys2virt or frame")
    }

    /// Возвращает изменяемую ссылку на одну запись узла таблицы страниц,
    /// записанного в физическом фрейме `page_table_frame`.
    /// Запись соответствует виртуальному адресу `virt`,
    /// а `level` указывает уровень узла в дереве отображения.
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что:
    ///   - Во `frame` находится узел таблицы страниц уровня `level`.
    ///   - Инварианты управления памятью в Rust'е не будут нарушены.
    ///     В частности, нет других ссылок, которые ведут в ту же запись [`PageTableEntry`].
    unsafe fn pte_mut(
        &mut self,
        virt: Virt,
        level: u32,
        page_table_frame: Frame,
    ) -> &mut PageTableEntry {
        let index_shift = PAGE_OFFSET_BITS + level * PAGE_TABLE_INDEX_BITS;
        let index = (virt.into_usize() >> index_shift) & PAGE_TABLE_INDEX_MASK;
        let page_table = self.page_table_mut(page_table_frame);

        &mut page_table[index]
    }

    /// Выделяет физический фрейм под отсутствующую промежуточную таблицу
    /// страничного отображения.
    /// См. [`Mapping::translate()`] в котором этот метод
    /// используется как вспомогательный.
    fn map_intermediate(
        pte: &mut PageTableEntry,
        flags: PageTableFlags,
        phys2virt: Page,
        frame_allocator: &mut FrameAllocator,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        if self.is_valid() {
            assert_ne!(Self::current_page_table_root(), self.page_table_root());

            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL);
        }
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use super::{
        super::Page,
        Mapping,
    };

    pub fn phys2virt(mapping: &Mapping) -> Page {
        mapping.phys2virt
    }
}
