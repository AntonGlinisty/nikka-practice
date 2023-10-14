use core::{
    alloc::Layout,
    mem,
    ptr::NonNull,
};

use crate::memory::{
    Block,
    Page,
    Virt,
};

use super::Info;

/// Аллокатор, выделяющий память блоками одинакового размера.
///
/// Предполагает, что все блоки памяти, с которыми он работает,
/// имеют одинаковый размер.
/// Причём тот же размер передаётся как аргумент `size` в
/// некоторые из его методов.
/// Более того, этот размер и выравнивание блоков должны быть кратны [`MIN_SIZE`].
#[derive(Debug)]
pub(super) struct FixedSizeAllocator {
    /// Список свободных блоков памяти.
    free: Lifo,

    /// Статистика аллокатора.
    info: Info,
}

impl FixedSizeAllocator {
    /// Создаёт аллокатор, выделяющий память блоками одинакового размера.
    pub(super) const fn new() -> Self {
        Self {
            free: Lifo::new(),
            info: Info::new(),
        }
    }

    /// Статистика аллокатора.
    pub(super) fn info(&self) -> Info {
        self.info
    }

    /// Возвращает `true`, если у аллокатора больше нет свободных блоков.
    pub(super) fn is_empty(&self) -> bool {
        self.free.is_empty()
    }

    /// Выделяет блок памяти из списка свободных.
    /// Возвращает указатель на выделенный блок или
    /// [`core::ptr::null_mut()`], если список свободных блоков пуст.
    pub(super) fn allocate(
        &mut self,
        layout: Layout,
        size: usize,
    ) -> *mut u8 {
        // TODO: your code here.
        unimplemented!();
    }

    /// Сохраняет в список свободных блок памяти по указателю `ptr`.
    ///
    /// # Safety
    ///
    /// Вызывающая сторона должна гарантировать
    /// [то же самое](https://doc.rust-lang.org/nightly/core/alloc/trait.Allocator.html#safety-1),
    /// что требуется при вызове [`core::alloc::Allocator::deallocate()`].
    pub(super) unsafe fn deallocate(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        size: usize,
    ) {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает [`Layout`] блока памяти,
    /// который состоит из целого количества страниц [`Page`].
    ///
    /// В методе [`FixedSizeAllocator::stock_up()`] блок с этим `Layout`
    /// будет нарезаться на подблоки размера `size`, которые выделяет этот аллокатор.
    pub(super) fn get_pack_layout(size: usize) -> Layout {
        // TODO: your code here.
        unimplemented!();
    }

    /// Режет блок `pack` на подблоки размера `size`,
    /// которые сохраняет себе для последующих аллокаций.
    ///
    /// # Safety
    ///
    /// Блок `pack` не должен пересекаться ни с какой другой памятью.
    ///
    /// # Panics
    ///
    /// Паникует, если:
    ///   - `pack` не выровнен по границам страниц;
    ///   - `size` не подходит под требования размера и выравнивания структуры [`Lifo`].
    pub(super) unsafe fn stock_up(
        &mut self,
        size: usize,
        pack: Block<Virt>,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}

/// Список свободных блоков памяти.
///
/// Узел списка является этой же структурой.
/// Фактически, узел задаёт подсписок от текущего узла до последнего.
///
/// На блоки памяти, сохраняемые в [`Lifo`] накладываются требования, см. метод [`Lifo::push()`]:
///   - [`Lifo`] реализован интрузивно, поэтому сохраняемые блоки должны
///     позволять хранить в их начале структуру [`Lifo`]. Для этого они должны иметь
///     размер не меньше размера [`Lifo`] и выравнивание не меньше выравнивания [`Lifo`].
///   - Сохраняемые блоки не должны пересекаться ни с какой другой памятью.
///     В том числе, это требуется для того чтобы [`Lifo`] не имело разделяемого состояния,
///     а значит могло быть помечено типажом [`Send`].
///
/// В названии используется аббревиатура LIFO
/// ([Last In, First Out](https://en.wikipedia.org/wiki/Stack_(abstract_data_type))),
/// а не `Stack`, чтобы не путать со стеком программы.
#[derive(Clone, Copy, Debug)]
struct Lifo {
    /// Указатель на следующий свободный блок или [`None`].
    next: Option<NonNull<Lifo>>,
}

impl Lifo {
    /// Создаёт пустой список свободных блоков памяти.
    const fn new() -> Self {
        Self { next: None }
    }

    /// Возвращает `true`, если в списке больше нет свободных блоков.
    fn is_empty(&self) -> bool {
        self.next.is_none()
    }

    /// Выделяет блок памяти из списка свободных.
    /// Возвращает адрес выделенного блока или
    /// [`Virt::default()`], если список свободных блоков пуст.
    fn pop(&mut self) -> Virt {
        // TODO: your code here.
        unimplemented!();
    }

    /// Сохраняет в список свободных блок памяти с адресом `virt`.
    ///
    /// # Safety
    ///
    /// - Сохраняемый блок должен позволять хранить в его начале структуру [`Lifo`].
    ///   Для этого он должен иметь размер не меньше размера [`Lifo`] и
    ///   выравнивание не меньше выравнивания [`Lifo`].
    /// - Сохраняемый блок не должен пересекаться ни с какой другой памятью.
    ///
    /// # Panics
    ///
    /// Паникует, если `virt` не является валидным адресом для хранения структуры [`Lifo`].
    unsafe fn push(
        &mut self,
        virt: Virt,
    ) {
        // TODO: your code here.
        unimplemented!();
    }
}

/// Сохраняемые в [`Lifo`] блоки памяти не должны пересекаться ни с какой другой памятью.
/// Поэтому [`Lifo`] не имеет разделяемого состояния.
/// См. требования метода [`Lifo::push()`] и
/// [The Rustonomicon, "Send and Sync"](https://doc.rust-lang.org/nomicon/send-and-sync.html).
unsafe impl Send for Lifo {
}

/// Минимальный размер и выравнивание для блоков памяти, которыми управляет [`FixedSizeAllocator`].
pub(super) const MIN_SIZE: usize = if mem::size_of::<Lifo>() > mem::align_of::<Lifo>() {
    mem::size_of::<Lifo>()
} else {
    mem::align_of::<Lifo>()
};
