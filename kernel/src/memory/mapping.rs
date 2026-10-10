use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{
        Bound,
        RangeBounds,
    },
    ptr::NonNull,
};

use duplicate::duplicate_item;

use crate::{
    error::{
        Error::NoPage,
        Result,
    },
    log::{
        debug,
        info,
    },
    time,
};

use super::{
    FRAME_ALLOCATOR,
    FrameGuard,
    Path,
    Phys2Virt,
    USER_R,
    Virt,
    frage::{
        Frame,
        Page,
    },
    mmu::{
        self,
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_LEAF_LEVEL,
        PAGE_TABLE_LEVEL_COUNT,
        PAGE_TABLE_ROOT_LEVEL,
        PageTable,
        PageTableEntry,
    },
    size,
};

// Used in docs.
#[allow(unused)]
use crate::{
    error::Error,
    memory::mmu::PageTableFlags,
};

/// Типаж для манипулирования трансляцией виртуальных адресов в физические.
pub trait Translate {
    #[allow(rustdoc::private_intra_doc_links)]
    /// Принимает на вход виртуальный адрес `virt`, который нужно транслировать.
    ///
    /// Возвращает максимально длинный отображённый в память префикс пути в дереве трансляции,
    /// соответствующий входному виртуальному адресу `virt`.
    /// То есть, последний из узлов [`Path::nodes`], который не равен [`None`],
    /// может указывать на [`PageTableEntry`] со сброшенным флагом [`PageTableFlags::PRESENT`],
    /// но сама эта [`PageTableEntry`] присутствует в памяти.
    /// Если же `virt` отображён в память, то либо это 4KiB-ая страница и
    /// тогда все элементы [`Path::nodes`] не равны [`None`],
    /// либо последний не равный [`None`], элемент соответствует
    /// [`PageTableEntry`] с установленным флагом [`PageTableFlags::HUGE`].
    fn path(
        &mut self,
        virt: Virt,
    ) -> Path<'_>;

    /// Выбирает в таблице страниц корневого уровня свободную запись,
    /// которую инициализирует как рекурсивную.
    /// Возвращает её номер.
    ///
    /// Если свободных записей в таблице страниц корневого уровня нет,
    /// возвращает ошибку [`Error::NoPage`].
    fn make_recursive_mapping(&mut self) -> Result<usize>;

    /// Удаляет все рекурсивные записи.
    fn remove_recursive_mappings(&mut self);

    /// Освобождает не использующиеся промежуточные узлы отображения страниц.
    fn unmap_unused_intermediate(&mut self);
}

/// Многоуровневая таблица страниц.
///
/// Фактически дерево большой арности, если игнорировать рекурсивные записи.
#[derive(Debug, Default, Eq, PartialEq)]
pub(super) struct Mapping {
    /// Фрейм с корневым узлом таблицы страниц.
    page_table_root: Frame,

    /// Для простоты работы с физической памятью,
    /// она целиком линейно отображена в некоторую область виртуальной.
    /// [`Mapping::phys2virt`] описывает это отображение.
    phys2virt: Phys2Virt,

    /// Номер рекурсивной записи в таблице страниц корневого уровня.
    /// Либо [`usize::MAX`], если рекурсивное отображение страниц не настроено.
    recursive_mapping: usize,
}

impl Mapping {
    /// Инициализирует страничное отображение по корневому узлу `page_table_root`.
    /// Аргумент [`phys2virt`][Phys2Virt] описывает линейное отображение
    /// физической памяти в виртуальную внутри этого страничного отображения.
    pub(super) fn new(
        page_table_root: Frame,
        phys2virt: Phys2Virt,
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
        result.page_table_root = result.allocate_node(PAGE_TABLE_ROOT_LEVEL)?.take();

        let page_table_root = result.page_table_root;
        self.duplicate_subtree(
            &mut result,
            page_table_root,
            self.page_table_root(),
            PAGE_TABLE_ROOT_LEVEL,
        )?;

        Ok(result)
    }

    /// Возвращает корневой узел таблицы страниц отображения.
    pub(super) fn page_table_root(&self) -> Frame {
        self.page_table_root
    }

    /// Возвращает линейное отображение физической памяти в виртуальную, см. [`Phys2Virt`].
    pub(super) fn phys2virt(&self) -> Phys2Virt {
        self.phys2virt
    }

    /// Возвращает итератор по листьям дерева отображения страниц.
    pub(super) fn iter_mut(&mut self) -> MappingIterator<'_> {
        self.range_mut(..)
    }

    /// Возвращает итератор по листьям дерева отображения страниц,
    /// попадающим в заданный диапазон `range`.
    pub(super) fn range_mut<R: RangeBounds<Page>>(
        &mut self,
        range: R,
    ) -> MappingIterator<'_> {
        let curr = match range.start_bound() {
            Bound::Included(start) => start.index(),
            Bound::Excluded(start) => start.advance_index(1),
            Bound::Unbounded => Page::lower_half_start_index(),
        };

        let end = match range.end_bound() {
            Bound::Included(end) => end.index() + 1,
            Bound::Excluded(end) => end.index(),
            Bound::Unbounded => Page::higher_half_end_index(),
        };

        MappingIterator {
            _marker: PhantomData,
            curr,
            end,
            mapping: self.into(),
        }
    }

    /// Шаг рекурсии при спуске по дереву отображения страниц.
    /// Выполняет основную работу по созданию копии отображения [`Mapping`],
    /// см. [`Mapping::duplicate()`].
    fn duplicate_subtree(
        &self,
        dst: &mut Mapping,
        dst_frame: Frame,
        src_frame: Frame,
        level: u32,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Шаг рекурсии при спуске по дереву отображения страниц.
    /// Выполняет основную работу по освобождению физических фреймов
    /// как отображённых [`Mapping`], так и занятых самим отображением.
    /// Возвращает `true`, если поддерево было полностью удалено.
    ///
    /// - `node` --- физический фрейм с текущим узлом;
    /// - `level` --- уровень текущего узла в дереве отображения страниц;
    /// - `drop_used` --- равен `true`, если нужно удалить все узлы дерева,
    ///   и `false`, если нужно удалить только узлы, которые фактически не нужны.
    ///   То есть те, через которые не ведут пути для отображённых страниц.
    ///
    /// Используется в [`Mapping::drop()`] и [`Mapping::unmap_unused_intermediate()`].
    fn drop_subtree(
        &mut self,
        node: Frame,
        level: u32,
        drop_used: bool,
    ) -> bool {
        // TODO: your code here.
        false // TODO: remove before flight.
    }

    /// Возвращает физический фрейм корневого узла текущего отображения
    /// виртуальной памяти в физическую.
    pub(super) fn current_page_table_root() -> Frame {
        mmu::page_table_root()
    }

    /// Исправляет уровни всех [`PageTableEntry`] в дереве отображения на фактические.
    pub(super) fn save_node_levels(&mut self) {
        let timer = time::timer();
        let pte_count = self.save_subtree_levels(self.page_table_root, PAGE_TABLE_ROOT_LEVEL);
        info!(pte_count, duration = %timer.elapsed(), "saved node levels into PTEs themselves");
    }

    /// Шаг рекурсии при спуске по дереву отображения страниц.
    /// Выполняет основную работу по исправлению уровней [`PageTableEntry`],
    /// см. [`Mapping::save_node_levels()`].
    ///
    /// Исправляет уровни всех [`PageTableEntry`] во `frame` на `level`.
    /// Возвращает количество исправленных записей в поддереве `frame`.
    fn save_subtree_levels(
        &mut self,
        frame: Frame,
        level: u32,
    ) -> usize {
        let mut pte_count = 0;
        let page_table_root = self.page_table_root;

        for index in 0 .. PAGE_TABLE_ENTRY_COUNT {
            let pte = unsafe { &mut self.page_table_mut(frame)[index] };
            pte.set_level(level);
            pte_count += 1;

            let is_recursive = level == PAGE_TABLE_ROOT_LEVEL && pte.frame() == Ok(page_table_root);
            if !is_recursive &&
                !pte.is_leaf() &&
                let Ok(frame) = pte.frame()
            {
                pte_count += self.save_subtree_levels(frame, level - 1);
            }
        }

        pte_count
    }

    /// Возвращает ссылку на узел таблицы страниц,
    /// записанный в данном физическом фрейме.
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что:
    ///   - Во `frame` находится узел таблицы страниц.
    ///   - Инварианты управления памятью в Rust'е не будут нарушены.
    ///     В частности, нет других ссылок, которые ведут во `frame`.
    #[allow(clippy::needless_arbitrary_self_type)]
    #[allow(unused)]
    #[duplicate_item(
        page_table_getter reference(x) return_type;
        [page_table_ref] [&x] [PageTable];
        [page_table_mut] [&mut x] [PageTable];
        [page_table_uninit_mut] [&mut x] [[MaybeUninit<PageTableEntry>; PAGE_TABLE_ENTRY_COUNT]];
    )]
    pub(super) unsafe fn page_table_getter(
        self: reference([Self]),
        frame: Frame,
    ) -> reference([return_type]) {
        let page_table = self.phys2virt().map(frame.address()).expect("bad frame");
        unsafe { page_table.try_into_mut().expect("bad phys2virt or frame") }
    }

    /// Возвращает ссылку на одну запись узла таблицы страниц,
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
    #[allow(clippy::needless_arbitrary_self_type)]
    #[allow(unused)]
    #[duplicate_item(
        getter page_table_getter reference(x);
        [pte_ref] [page_table_ref] [&x];
        [pte_mut] [page_table_mut] [&mut x];
    )]
    pub(super) unsafe fn getter(
        self: reference([Self]),
        virt: Virt,
        level: u32,
        page_table_frame: Frame,
    ) -> reference([PageTableEntry]) {
        let index = virt.page_table_index(level);
        let page_table = unsafe { self.page_table_getter(page_table_frame) };

        reference([page_table[index]])
    }

    /// Создаёт новый узел --- [`PageTable`] --- дерева отображения страниц,
    /// но не провязывает его в дерево.
    ///
    /// Инициализирует записи [`PageTableEntry`] в новом узле с помощью
    /// [`MaybeUninit::new()`] элементами [`PageTableEntry`] заданного уровня `level`.
    /// Возвращает фрейм нового узла.
    pub(super) fn allocate_node(
        &mut self,
        level: u32,
    ) -> Result<FrameGuard> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        if self.page_table_root() != Frame::default() {
            assert_ne!(Self::current_page_table_root(), self.page_table_root());

            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL, true);
        }
    }
}

impl Translate for Mapping {
    fn path(
        &mut self,
        virt: Virt,
    ) -> Path<'_> {
        // TODO: your code here.
        unimplemented!();
    }

    fn make_recursive_mapping(&mut self) -> Result<usize> {
        // TODO: your code here.
        Ok(self.recursive_mapping) // TODO: remove before flight.
    }

    fn remove_recursive_mappings(&mut self) {
        let root_frame = self.page_table_root();
        let page_table_root = unsafe { self.page_table_mut(root_frame) };
        for pte in page_table_root.iter_mut() {
            if pte.frame() == Ok(root_frame) {
                debug!(?pte, "remove recursive mapping");
                pte.clear();
            }
        }
    }

    fn unmap_unused_intermediate(&mut self) {
        // At least `Phys2Virt` should remain.
        assert!(!self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL, false));
    }
}

/// Итератор по листьям дерева отображения страниц.
pub struct MappingIterator<'a> {
    /// Маркер, привязывающий время жизни [`MappingIterator`]
    /// ко времени жизни соответствующего [`Mapping`].
    _marker: PhantomData<&'a mut Mapping>,

    /// Текущий номер виртуальной страницы, задающий позицию итератора.
    /// Значение, больше либо равное [`MappingIterator::end`], означает, что достигнут конец.
    curr: usize,

    /// Номер виртуальной страницы, задающий позицию за концом диапазона, который пробегает итератор.
    /// Значение, равное [`Page::higher_half_end_index()`], означает, что достигнут конец.
    /// Значения, больше [`Page::higher_half_end_index()`] не допустимы.
    end: usize,

    /// Дерево отображения.
    mapping: NonNull<Mapping>,
}

impl<'a> Iterator for MappingIterator<'a> {
    type Item = Path<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.end <= Page::higher_half_end_index());

        loop {
            if self.curr >= self.end {
                return None;
            }

            let mapping = unsafe { &mut (*self.mapping.as_ptr()) };
            let page = Page::from_index(self.curr)
                .expect("incorrect initialization or advancement of MappingIterator");
            let path = mapping.path(page.address());

            if path.is_recursive_entry() {
                let pages = path.pages(PAGE_TABLE_ROOT_LEVEL);
                self.curr = pages.start_element().advance_index(pages.count());
            } else {
                let (level, _) = path.deepest_pte();
                let pages = path.pages(level);

                self.curr = pages.start_element().advance_index(pages.count());

                return Some(path);
            }
        }
    }
}
