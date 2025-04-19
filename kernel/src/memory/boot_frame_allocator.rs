#![forbid(unsafe_code)]

use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

use crate::{
    error::{
        Error::NoFrame,
        Result,
    },
    log::{
        info,
        trace,
    },
};

use super::{
    FrameAllocator,
    FrameGuard,
    block::Block,
    frage::Frame,
    size::SizeOf,
};

// Used in docs.
#[allow(unused)]
use crate::error::Error;

/// Вспомогательный аллокатор физических фреймов,
/// использующийся до инициализации основного [`super::main_frame_allocator::MainFrameAllocator`].
/// Выделяет фреймы из блока подряд и не способен забирать их назад (освобождать).
pub struct BootFrameAllocator {
    /// Блок свободных физических фреймов.
    block: Block<Frame>,

    /// Количество операций освобождения фреймов.
    deallocations: usize,

    /// Используется в методах [`BootFrameAllocator::is_managed()`] и
    /// [`BootFrameAllocator::is_used()`] для определения,
    /// что заданный физический фрейм принадлежит этому аллокатору.
    /// Вне зависимости от того был ли он уже выделен или нет.
    initial_block: Block<Frame>,

    /// Количество увеличений ссылок на уже выделенные фреймы.
    references: usize,
}

impl BootFrameAllocator {
    /// Инициализирует аллокатор [`BootFrameAllocator`]
    /// самым большим блоком подряд идущих свободных физических фреймов
    /// по информации из `memory_map`.
    pub(super) fn new(memory_map: &MemoryMap) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    /// Выделяет блок подряд идущих фреймов, достаточный для хранения объекта
    /// размером `size` **байт**.
    ///
    /// Если выделить заданный размер физической памяти не удалось,
    /// возвращает ошибку [`Error::NoFrame`].
    pub(super) fn allocate_block(
        &mut self,
        size: usize,
    ) -> Result<Block<Frame>> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает `true`, если не было вызовов
    /// [`BootFrameAllocator::deallocate()`] или [`BootFrameAllocator::reference()`],
    /// после которых информация об использовании физических фреймов может быть неконсистентна.
    pub(super) fn is_consistent(&self) -> bool {
        self.deallocations == 0 && self.references == 0
    }

    /// Проверяет, что заданный физический фрейм управляется этим аллокатором.
    pub(super) fn is_managed(
        &self,
        frame: Frame,
    ) -> bool {
        self.initial_block.contains(frame)
    }
}

impl Drop for BootFrameAllocator {
    fn drop(&mut self) {
        let leaked_frame_count = self.count();
        info!(frame_allocator = "boot", block = %self.block, leaked_frame_count, "drop");
        assert!(leaked_frame_count == 0, "leaked some memory frames");
    }
}

impl FrameAllocator for BootFrameAllocator {
    /// Возвращает количество "свободных" физических фреймов, считая, в том числе:
    ///   - утёкшие при освобождении методом [`BootFrameAllocator::deallocate()`],
    ///   - фреймы для которых было "увеличено количество ссылок" в [`BootFrameAllocator::reference()`].
    fn count(&self) -> usize {
        self.block.count() + self.deallocations - self.references
    }

    fn allocate(&mut self) -> Result<FrameGuard> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Увеличивает на единицу общий счётчик освобождённых фреймов.
    /// Счетчик учитывается в [`BootFrameAllocator::count()`].
    /// Физический фрейм `_frame` утекает.
    /// Поэтому этот метод должен использоваться только в тестах.
    fn deallocate(
        &mut self,
        _frame: Frame,
    ) {
        self.deallocations += 1;
    }

    /// Увеличивает на единицу общий счётчик ссылок на фреймы.
    /// Счетчик учитывается в [`BootFrameAllocator::count()`].
    /// Не увеличивает счётчик использования самого фрейма `_frame`.
    /// Поэтому этот метод должен использоваться только в тестах.
    fn reference(
        &mut self,
        frame: Frame,
    ) -> FrameGuard {
        self.references += 1;
        FrameGuard::new(frame)
    }

    fn reference_count(
        &self,
        _frame: Frame,
    ) -> Result<usize> {
        panic!("boot frame allocator does not support frame reference count");
    }

    /// Проверяет, что заданный физический фрейм уже был выделен, причём этим аллокатором.
    fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        self.is_managed(frame) && !self.block.contains(frame)
    }
}

#[doc(hidden)]
pub(super) mod test_scaffolding {
    use crate::error::Result;

    use super::{
        super::{
            Block,
            Frame,
            FrameAllocator,
        },
        BootFrameAllocator,
    };

    pub fn allocate_block(
        allocator: &mut BootFrameAllocator,
        size: usize,
    ) -> Result<Block<Frame>> {
        allocator.allocate_block(size)
    }

    pub fn is_managed(
        allocator: &BootFrameAllocator,
        frame: Frame,
    ) -> bool {
        allocator.is_managed(frame)
    }

    pub fn is_used(
        allocator: &BootFrameAllocator,
        frame: Frame,
    ) -> bool {
        allocator.is_used(frame)
    }
}
