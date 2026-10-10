use core::{
    cmp,
    fmt,
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
    log::trace,
};

use super::{
    Block,
    FULL_ACCESS,
    FrameGuard,
    Mapping,
    Virt,
    frage::{
        L0_SIZE,
        L1_SIZE,
        L2_SIZE,
        Page,
    },
    mapped_block::MappedBlock,
    mmu::{
        self,
        PAGE_TABLE_ENTRY_COUNT,
        PAGE_TABLE_LEAF_LEVEL,
        PAGE_TABLE_LEVEL_COUNT,
        PAGE_TABLE_ROOT_LEVEL,
        PTE_LEVEL_CAP,
        PageTableEntry,
        PageTableFlags,
    },
    size::{
        self,
        Size,
    },
};

// Used in docs.
#[allow(unused)]
use {
    super::FRAME_ALLOCATOR,
    ku::error::Error,
};

/// Путь в дереве отображения заданного виртуального адреса.
#[derive(Debug, Eq, PartialEq)]
pub struct Path<'a> {
    /// Дерево отображения.
    mapping: &'a mut Mapping,

    /// Узлы на пути в дереве отображения,
    /// задаваемые как указатели на соответствующие [`PageTableEntry`].
    /// Элемент с индексом [`PAGE_TABLE_LEAF_LEVEL`] задаёт указатель на [`PageTableEntry`],
    /// отображающей виртуальную страницу адреса [`Path::virt`] на его физическую страницу.
    /// Элемент с индексом [`PAGE_TABLE_ROOT_LEVEL`] задаёт указатель на [`PageTableEntry`],
    /// находящуюся в корневой таблице страниц.
    /// Если какой-то из узлов не отображён в память,
    /// соответствующий элемент равен [`None`].
    nodes: [Option<NonNull<PageTableEntry>>; PAGE_TABLE_LEVEL_COUNT],

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
    pub(super) fn new(
        mapping: &'a mut Mapping,
        nodes: [Option<NonNull<PageTableEntry>>; PAGE_TABLE_LEVEL_COUNT],
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
        let l0_frame_index = pte.phys().into_usize() / L0_SIZE;

        let pages = self.pages(level);
        let frames = Block::from_index(l0_frame_index, l0_frame_index + pages.count())
            .expect(Self::INVALID_PATH);

        MappedBlock::new(flags, frames, pages)
    }

    /// Листьевые флаги доступа к странице, которую описывает этот [`Path`].
    pub fn flags(&self) -> Result<PageTableFlags> {
        self.pte().map(|pte| pte.flags())
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
    pub(crate) fn set_flags(
        &mut self,
        flags: PageTableFlags,
    ) -> Result<()> {
        for (level, node) in self.nodes.iter().enumerate() {
            let pte = unsafe { node.ok_or(NoPage)?.as_mut() };
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

    /// Возвращает адрес, для которого построен путь отображения.
    pub(super) fn virt(&self) -> Virt {
        self.virt
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Возвращает листьевую [`PageTableEntry`], которая отвечает за отображение адреса [`Path::virt()`].
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- промежуточного или нужного листьевого узла таблицы страниц нет.
    /// - [`Error::Unimplemented`] --- промежуточный узел таблицы страниц
    ///   имеет флаг [`PageTableFlags::HUGE`].
    pub fn pte(&self) -> Result<&'a PageTableEntry> {
        self.get_pte()
    }

    /// Возвращает листьевую [`PageTableEntry`], которая отвечает за отображение адреса [`Path::virt`].
    ///
    /// # Errors
    ///
    /// - [`Error::NoPage`] --- промежуточного или нужного листьевого узла таблицы страниц нет.
    /// - [`Error::Unimplemented`] --- промежуточный узел таблицы страниц
    ///   имеет флаг [`PageTableFlags::HUGE`].
    fn pte_mut(&mut self) -> Result<&'a mut PageTableEntry> {
        self.get_pte_mut()
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
        [get_pte] [&Self] [&'a PageTableEntry] [deepest_pte];
        [get_pte_mut] [&mut Self] [&'a mut PageTableEntry] [deepest_pte_mut];
    )]
    fn getter(self: self_type) -> Result<return_type> {
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
        [deepest_pte] [&Self] [&'a PageTableEntry] [as_ref];
        [deepest_pte_mut] [&mut Self] [&'a mut PageTableEntry] [as_mut];
    )]
    pub(super) fn deepest_pte_getter(self: self_type) -> (u32, return_type) {
        let level = self.nodes.iter().position(Option::is_some).expect(Self::INVALID_PATH);

        let pte = unsafe { self.nodes[level].expect(Self::INVALID_PATH).converter() };
        let level = level.try_into().expect("unreasonable PTE level");

        (level, pte)
    }

    /// Возвращает блок страниц, который отображает текущий [`Path`].
    /// Аргумент `level` задаёт уровень самой глубокой PTE,
    /// его можно получить из `Path::deepest_pte()`.
    pub(super) fn pages(
        &self,
        level: u32,
    ) -> Block<Page> {
        let page_count = PAGE_TABLE_ENTRY_COUNT.pow(level);

        let end = (Page::containing(self.virt).index() + 1).next_multiple_of(page_count);
        let start = end - page_count;

        Block::from_index(start, end).expect(Self::INVALID_PATH)
    }

    /// Возвращает `true` если путь идёт через рекурсивную запись корневого уровня.
    pub(super) fn is_recursive_entry(&self) -> bool {
        let root_frame = self.mapping.page_table_root();
        let root_pte = unsafe {
            self.nodes[size::from(PAGE_TABLE_ROOT_LEVEL)]
                .expect(Path::INVALID_PATH)
                .as_ref()
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

        assert!(
            self.nodes[size::from(level) + 1 ..]
                .iter()
                .all(|pte| unsafe { pte.expect(Self::INVALID_PATH).as_ref().is_present() }),
        );

        if !self.is_recursive_entry() {
            assert!(
                (level ..)
                    .zip(self.nodes[size::from(level) ..].iter())
                    .all(|(level, pte)| cmp::min(level, PTE_LEVEL_CAP) ==
                        unsafe { pte.expect(Self::INVALID_PATH).as_ref().level() })
            );
        }

        if level > PAGE_TABLE_LEAF_LEVEL {
            assert!(self.nodes[.. size::from(level)].iter().all(Option::is_none));
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

        for pte in self.nodes.iter().rev().flatten().map(|node| unsafe { node.as_ref() }) {
            write!(formatter, "L{level}: {} / {frame}", Virt::from_ref(pte))?;

            if !pte.is_present() {
                return write!(formatter, " (non-present)");
            }

            let is_present = "failed to get frame for a present page";

            if pte.is_huge() {
                let (phys, huge_frame_size) = match level {
                    1 => {
                        let huge_frame = pte.huge_frame::<L1_SIZE>().expect(is_present);
                        (huge_frame.offset(self.virt), huge_frame.size())
                    },
                    2 => {
                        let huge_frame = pte.huge_frame::<L2_SIZE>().expect(is_present);
                        (huge_frame.offset(self.virt), huge_frame.size())
                    },
                    _ => panic!("unexpected page table level for a huge page - {}", level),
                };

                return write!(
                    formatter,
                    " ({} huge page) => {phys}",
                    Size::bytes(huge_frame_size),
                );
            };

            frame = pte.frame().expect(is_present);

            if level == PAGE_TABLE_LEAF_LEVEL {
                let phys = frame.offset(self.virt);
                write!(formatter, " => {phys}")?;
            } else {
                write!(formatter, " -> ")?;

                level -= 1;
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use core::ptr::NonNull;

    use super::super::mmu::{
        PAGE_TABLE_LEVEL_COUNT,
        PageTableEntry,
    };

    pub use super::Path;

    pub fn deepest_pte<'a>(path: &'a Path<'a>) -> (u32, &'a PageTableEntry) {
        path.deepest_pte()
    }

    pub fn nodes(path: &Path) -> [Option<NonNull<PageTableEntry>>; PAGE_TABLE_LEVEL_COUNT] {
        path.nodes
    }
}
