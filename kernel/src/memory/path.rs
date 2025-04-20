use core::{
    fmt,
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
        Error::{
            NoPage,
            Unimplemented,
        },
        Result,
    },
    log::{
        debug,
        trace,
    },
};

use super::{
    Block,
    FRAME_ALLOCATOR,
    FULL_ACCESS,
    FrameAllocator,
    FrameGuard,
    Phys2Virt,
    USER_R,
    Virt,
    frage::{
        Frame,
        Page,
    },
    mapped_block::MappedBlock,
    mmu::{
        self,
        PAGE_OFFSET_BITS,
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_INDEX_BITS,
        PAGE_TABLE_INDEX_MASK,
        PAGE_TABLE_LEAF_LEVEL,
        PAGE_TABLE_LEVEL_COUNT,
        PAGE_TABLE_ROOT_LEVEL,
        PageTable,
        PageTableEntry,
        PageTableFlags,
    },
    size,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

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

    /// Возвращает пустое отображение [`Mapping`].
    /// В отличие от [`Mapping::default()`] доступна в константном контексте.
    pub(super) const fn zero() -> Self {
        Self {
            page_table_root: Frame::zero(),
            phys2virt: Phys2Virt::zero(),
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
        result.page_table_root = self
            .duplicate_subtree(&mut result, self.page_table_root(), PAGE_TABLE_ROOT_LEVEL)?
            .take();
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

    /// Возвращает линейное отображение физической памяти в виртуальную, см. [`Phys2Virt`].
    pub(super) fn phys2virt(&self) -> Phys2Virt {
        self.phys2virt
    }

    /// Принимает на вход виртуальный адрес `virt`, который нужно транслировать.
    ///
    /// Возвращает ссылку на запись типа [`PageTableEntry`] в
    /// узле листьевого уровня таблицы страниц,
    /// соответствующую входному виртуальному адресу `virt`.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::NoPage`] если промежуточного или нужного листьевого узла таблицы страниц нет.
    ///   - [`Error::Unimplemented`] если промежуточный узел таблицы страниц
    ///     имеет флаг [`PageTableFlags::HUGE`].
    pub(super) fn translate(
        &mut self,
        virt: Virt,
    ) -> Result<&mut PageTableEntry> {
        self.path(virt).get_mut()
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Принимает на вход виртуальный адрес `virt`, который нужно транслировать.
    ///
    /// Возвращает максимально длинный отображённый в память префикс пути в дереве трансляции,
    /// соответствующий входному виртуальному адресу `virt`.
    /// То есть, последний из узлов [`Path::nodes`], который не равен [`Virt::default()`],
    /// может указывать на [`PageTableEntry`] со сброшенным флагом [`PageTableFlags::PRESENT`],
    /// но сама эта [`PageTableEntry`] присутствует в памяти.
    /// Если же `virt` отображён в память, то либо это 4KiB-ая страница и
    /// тогда все элементы [`Path::nodes`] не равны [`Virt::default()`],
    /// либо последний не равный [`Virt::default()`], элемент соответствует
    /// [`PageTableEntry`] с установленным флагом [`PageTableFlags::HUGE`].
    pub(super) fn path(
        &mut self,
        virt: Virt,
    ) -> Path {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает итератор по листьям дерева отображения страниц.
    pub(super) fn iter_mut(&mut self) -> MappingIterator {
        self.range_mut(..)
    }

    /// Возвращает итератор по листьям дерева отображения страниц,
    /// попадающим в заданный диапазон `range`.
    fn range_mut<R: RangeBounds<Page>>(
        &mut self,
        range: R,
    ) -> MappingIterator {
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
        src_frame: Frame,
        level: u32,
    ) -> Result<FrameGuard> {
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

    /// Освобождает не использующиеся промежуточные узлы отображения страниц.
    pub(super) fn unmap_unused_intermediate(&mut self) {
        if self.is_valid() &&
            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL, false)
        {
            self.page_table_root = Frame::default();
        }
    }

    /// Выбирает в таблице страниц корневого уровня свободную запись,
    /// которую инициализирует как рекурсивную.
    /// Сохраняет её номер в поле [`Mapping::recursive_mapping`] и
    /// возвращает его в вызывающую функцию.
    ///
    /// Если свободных записей в таблице страниц корневого уровня нет,
    /// возвращает ошибку [`Error::NoPage`].
    pub(super) fn make_recursive_mapping(&mut self) -> Result<usize> {
        // TODO: your code here.
        Ok(self.recursive_mapping) // TODO: remove before flight.
    }

    /// Удаляет все рекурсивные записи.
    pub(super) fn remove_recursive_mappings(&mut self) {
        let root_frame = self.page_table_root();
        let page_table_root = unsafe { self.page_table_mut(root_frame) };
        for pte in page_table_root.iter_mut() {
            if pte.frame() == Ok(root_frame) {
                debug!(?pte, "remove recursive mapping");
                pte.clear();
            }
        }
    }

    /// Удаляет ненужную часть --- находящуюся за пределами [`Mapping::phys2virt`] ---
    /// линейного отображения физической памяти, которое сделал [`bootloader`].
    pub(super) fn remove_excessive_phys2virt(&mut self) -> Result<()> {
        let last_page = Page::from_index(self.phys2virt.last_page_index().ok_or(NoPage)?)?;
        let mut cleared_pte_count = 0;

        for mut path in self.range_mut(last_page ..).skip(1) {
            let (_, pte) = path.deepest_pte_mut();

            if pte.is_huge() {
                pte.clear();
                cleared_pte_count += 1;
            } else {
                break;
            }
        }

        let start_free_frame_count = FRAME_ALLOCATOR.lock().count();

        self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL, false);

        let freed_frame_count = FRAME_ALLOCATOR.lock().count() - start_free_frame_count;

        debug!(
            freed_frame_count,
            cleared_pte_count,
            "removed excessive phys2virt{}",
            if freed_frame_count == 0 {
                ", but the frames it used are still used or reserved"
            } else {
                ""
            },
        );

        Ok(())
    }

    /// Возвращает физический фрейм корневого узла текущего отображения
    /// виртуальной памяти в физическую.
    pub(super) fn current_page_table_root() -> Frame {
        mmu::page_table_root()
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
        [page_table_uninit_mut] [&mut x] [MaybeUninit<PageTable>];
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
    unsafe fn getter(
        self: reference([Self]),
        virt: Virt,
        level: u32,
        page_table_frame: Frame,
    ) -> reference([PageTableEntry]) {
        let index_shift = PAGE_OFFSET_BITS + level * PAGE_TABLE_INDEX_BITS;
        let index = (virt.into_usize() >> index_shift) & PAGE_TABLE_INDEX_MASK;
        let page_table = unsafe { self.page_table_getter(page_table_frame) };

        reference([page_table[index]])
    }

    /// Создаёт новый узел --- [`PageTable`] --- дерева отображения страниц,
    /// но не провязывает его в дерево.
    ///
    /// Очищает записи [`PageTableEntry`] в новом узле с помощью [`MaybeUninit::zeroed()`].
    /// Возвращает фрейм нового узла.
    fn allocate_node(&mut self) -> Result<FrameGuard> {
        // TODO: your code here.
        unimplemented!();
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        if self.is_valid() {
            assert_ne!(Self::current_page_table_root(), self.page_table_root());

            self.drop_subtree(self.page_table_root(), PAGE_TABLE_ROOT_LEVEL, true);
        }
    }
}

/// Путь в дереве отображения заданного виртуального адреса.
#[derive(Debug, Eq, PartialEq)]
pub struct Path<'a> {
    /// Дерево отображения.
    mapping: &'a mut Mapping,

    /// Узлы на пути в дереве отображения,
    /// задаваемые как адреса соответствующих [`PageTableEntry`].
    /// Элемент с индексом [`PAGE_TABLE_LEAF_LEVEL`] задаёт адрес [`PageTableEntry`],
    /// отображающей виртуальную страницу адреса [`Path::virt`] на его физическую страницу.
    /// Элемент с индексом [`PAGE_TABLE_ROOT_LEVEL`] задаёт адрес [`PageTableEntry`],
    /// находящейся в корневой таблице страниц.
    /// Если какой-то из узлов не отображён в память,
    /// соответствующий элемент равен [`Virt::default()`].
    nodes: [Virt; PAGE_TABLE_LEVEL_COUNT],

    /// Адрес, для которого построен путь отображения.
    virt: Virt,
}

impl<'a> Path<'a> {
    /// Создаёт путь в дереве отображения заданного виртуального адреса.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - На корневом уровне [`PAGE_TABLE_ROOT_LEVEL`] нет [`PageTableEntry`].
    /// - Выше [`Path::deepest_pte()`] есть [`PageTableEntry`] для которой
    ///   [`PageTableEntry::is_present()`] возвращает `false`.
    /// - Глубже [`Path::deepest_pte()`] есть существующие [`PageTableEntry`].
    /// - Если самая глубокая [`PageTableEntry`] расположена выше [`PAGE_TABLE_LEAF_LEVEL`]
    ///   и при этом для неё [`PageTableEntry::is_present()`] и [`PageTableEntry::is_huge()`]
    ///   возвращают разные значения.
    fn new(
        mapping: &'a mut Mapping,
        nodes: [Virt; PAGE_TABLE_LEVEL_COUNT],
        virt: Virt,
    ) -> Self {
        let path = Self {
            mapping,
            nodes,
            virt,
        };

        path.validate();

        path
    }

    /// Возвращает отображение блока виртуальных страниц
    /// на блок физических фреймов, если текущий [`Path`]
    /// задаёт путь к листу дерева отображения.
    pub fn block(&self) -> MappedBlock {
        let (level, pte) = self.deepest_pte();

        let flags = pte.flags();
        let frame = pte.frame().unwrap_or_default();

        let pages = self.pages(level);
        let frames = Block::from_index(frame.index(), frame.index() + pages.count())
            .expect(Self::INVALID_PATH);

        MappedBlock::new(flags, frames, pages)
    }

    /// Устанавливает флаги доступа к странице, в которую ведёт этот [`Path`].
    ///
    /// Если это необходимо, корректирует промежуточные флаги.
    /// Сбрасывает кэш страничного преобразования процессора
    /// ([Translation Lookaside Buffer, TLB](https://en.wikipedia.org/wiki/Translation_lookaside_buffer))
    /// для целевой страницы `page`.
    ///
    /// Если промежуточного или нужного листьевого узла таблицы страниц нет,
    /// возвращает ошибку [`Error::NoPage`].
    pub(super) fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) -> Result<()> {
        for (level, node) in self.nodes.iter().enumerate() {
            let pte = unsafe { node.try_into_mut::<PageTableEntry>().map_err(|_| NoPage)? };
            if !pte.is_present() {
                return Err(NoPage);
            }

            let level_flags = if level == size::from(PAGE_TABLE_LEAF_LEVEL) {
                flags
            } else {
                pte.flags() | flags & FULL_ACCESS
            };

            pte.set_flags(level_flags);
        }

        unsafe {
            mmu::flush(Page::containing(self.virt));
        }

        Ok(())
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Возвращает листьевую [`PageTableEntry`], которая отвечает за отображение адреса [`Path::virt`].
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- промежуточного или нужного листьевого узла таблицы страниц нет.
    /// - [`Error::Unimplemented`] --- промежуточный узел таблицы страниц
    ///   имеет флаг [`PageTableFlags::HUGE`].
    #[allow(clippy::needless_arbitrary_self_type)]
    #[duplicate_item(
        getter self_type return_type deepest_pte_getter;
        [get] [&Self] [&'a PageTableEntry] [deepest_pte];
        [get_mut] [&mut Self] [&'a mut PageTableEntry] [deepest_pte_mut];
    )]
    pub(super) fn getter(self: self_type) -> Result<return_type> {
        let (level, pte) = self.deepest_pte_getter();

        if level == PAGE_TABLE_LEAF_LEVEL {
            Ok(pte)
        } else if pte.is_huge() {
            Err(Unimplemented)
        } else {
            Err(NoPage)
        }
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Выделяет физические фреймы под отсутствующие промежуточные таблицы страничного отображения.
    /// И исправляет флаги с которыми они отображены так, чтобы целевой адрес [`Path::virt`]
    /// можно было отобразить с эффективными флагами `flags`.
    /// Отображает заданный `frame` с флагами `flags`.
    /// Освобождает фрейм, который был ранее отображён, если он есть.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::NoFrame`] если пришлось выделить физический фрейм,
    ///     но их не осталось во [`static@FRAME_ALLOCATOR`].
    ///   - [`Error::Unimplemented`] если промежуточный узел таблицы страниц
    ///     имеет флаг [`PageTableFlags::HUGE`].
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что инварианты управления памятью в Rust'е
    /// не будут нарушены.
    /// В частности, не осталось ссылок, которые ведут в удаляемую страницу.
    pub unsafe fn map(
        &mut self,
        frame: FrameGuard,
        flags: PageTableFlags,
    ) -> Result<()> {
        // TODO: your code here.

        self.validate();

        Ok(())
    }

    /// Удаляет отображение страницы по текущему пути.
    /// Физический фрейм освобождается, если на него не осталось других ссылок.
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- промежуточного или нужного листьевого узла таблицы страниц нет.
    /// - [`Error::Unimplemented`] --- промежуточный узел таблицы страниц
    ///   имеет флаг [`PageTableFlags::HUGE`].
    ///
    /// # Safety
    ///
    /// Вызывающий код должен гарантировать, что инварианты управления памятью в Rust'е
    /// не будут нарушены.
    /// В частности, не осталось ссылок, которые ведут в удаляемую страницу.
    pub unsafe fn unmap(&mut self) -> Result<()> {
        // TODO: your code here.

        self.validate();

        Ok(())
    }

    /// Возвращает самую далёкую от корня дерева [`PageTableEntry`]
    /// в данном [`Path`] вместе с её номером уровня в дереве отображения.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - На корневом уровне [`PAGE_TABLE_ROOT_LEVEL`] нет [`PageTableEntry`].
    #[allow(clippy::needless_arbitrary_self_type)]
    #[duplicate_item(
        deepest_pte_getter self_type return_type converter;
        [deepest_pte] [&Self] [&'a PageTableEntry] [try_into_ref];
        [deepest_pte_mut] [&mut Self] [&'a mut PageTableEntry] [try_into_mut];
    )]
    fn deepest_pte_getter(self: self_type) -> (u32, return_type) {
        let level = self
            .nodes
            .iter()
            .position(|&node| node != Virt::default())
            .expect(Self::INVALID_PATH);

        let pte = unsafe { self.nodes[level].converter().expect(Self::INVALID_PATH) };
        let level = level.try_into().expect("unreasonable PTE level");

        (level, pte)
    }

    /// Возвращает блок страниц, который отображает текущий [`Path`].
    /// Аргумент `level` задаёт уровень самой глубокой PTE,
    /// его можно получить из `Path::deepest_pte()`.
    fn pages(
        &self,
        level: u32,
    ) -> Block<Page> {
        let page_count = PAGE_TABLE_ENTRY_COUNT.pow(level);

        let end = (Page::containing(self.virt).index() + 1).next_multiple_of(page_count);
        let start = end - page_count;

        Block::from_index(start, end).expect(Self::INVALID_PATH)
    }

    /// Возвращает `true` если путь идёт через рекурсивную запись корневого уровня.
    fn is_recursive_entry(&self) -> bool {
        let root_frame = self.mapping.page_table_root();
        let root_pte = unsafe {
            self.nodes[size::from(PAGE_TABLE_ROOT_LEVEL)]
                .try_into_ref::<PageTableEntry>()
                .expect(Path::INVALID_PATH)
        };

        root_pte.frame() == Ok(root_frame)
    }

    /// Проверяет инварианты [`Path`].
    ///
    /// # Panics
    ///
    /// Паникует, если:
    /// - На корневом уровне [`PAGE_TABLE_ROOT_LEVEL`] нет [`PageTableEntry`].
    /// - Выше [`Path::deepest_pte()`] есть [`PageTableEntry`] для которой
    ///   [`PageTableEntry::is_present()`] возвращает `false`.
    /// - Глубже [`Path::deepest_pte()`] есть существующие [`PageTableEntry`].
    /// - Если самая глубокая [`PageTableEntry`] расположена выше [`PAGE_TABLE_LEAF_LEVEL`]
    ///   и при этом для неё [`PageTableEntry::is_present()`] и [`PageTableEntry::is_huge()`]
    ///   возвращают разные значения.
    fn validate(&self) {
        let (level, pte) = self.deepest_pte();
        let level = size::from(level);

        assert!(self.nodes[level + 1 ..].iter().all(|pte| unsafe {
            pte.try_into_ref::<PageTableEntry>().expect(Self::INVALID_PATH).is_present()
        }));

        if level > size::from(PAGE_TABLE_LEAF_LEVEL) {
            assert!(self.nodes[.. level].iter().all(|&pte| pte == Virt::default()));
            assert_eq!(pte.is_present(), pte.is_huge());
        }
    }

    /// Сообщение паники при некорректно сформированном [`Path`].
    const INVALID_PATH: &'static str = "invalid path";
}

impl<'a> fmt::Display for Path<'a> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut frame = self.mapping.page_table_root();
        let mut level = PAGE_TABLE_ROOT_LEVEL;

        for pte in self.nodes.iter().rev().filter(|&&pte| pte != Virt::default()) {
            write!(formatter, "L{level}: {} / {frame}", pte)?;

            let pte = unsafe { pte.try_into_ref::<PageTableEntry>().expect(Self::INVALID_PATH) };

            if let Ok(next_frame) = pte.frame() {
                frame = next_frame;

                if pte.is_huge() || level == PAGE_TABLE_LEAF_LEVEL {
                    if pte.is_huge() {
                        write!(formatter, " (huge page)")?;
                    }

                    let offset_mask = (1 << (level * PAGE_TABLE_INDEX_BITS + PAGE_OFFSET_BITS)) - 1;
                    let offset = self.virt.into_usize() & offset_mask;
                    let phys =
                        (frame.address() + offset).expect("offset inside a frame is always valid");

                    write!(formatter, " => {phys}")?;
                } else {
                    write!(formatter, " -> ")?;

                    level -= 1;
                }
            } else {
                write!(formatter, " (non-present)")?;
            }
        }

        Ok(())
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

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use ku::error::Result;

    use super::super::{
        Virt,
        mmu::{
            PAGE_TABLE_LEVEL_COUNT,
            PageTableEntry,
        },
    };

    pub use super::Path;

    pub fn deepest_pte<'a>(path: &'a Path<'a>) -> (u32, &'a PageTableEntry) {
        path.deepest_pte()
    }

    pub fn get_pte<'a>(path: &'a Path<'a>) -> Result<&'a PageTableEntry> {
        path.get()
    }

    pub fn nodes(path: &Path) -> [Virt; PAGE_TABLE_LEVEL_COUNT] {
        path.nodes
    }
}
