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

    /// Используется в методах [`BootFrameAllocator::is_managed()`] и
    /// [`BootFrameAllocator::is_used()`] для определения,
    /// что заданный физический фрейм принадлежит этому аллокатору.
    /// Вне зависимости от того был ли он уже выделен или нет.
    initial_block: Block<Frame>,
}

impl BootFrameAllocator {
    /// Инициализирует аллокатор [`BootFrameAllocator`]
    /// самым большим блоком подряд идущих свободных физических фреймов
    /// по информации из `memory_map`.
    pub(super) fn new(memory_map: &MemoryMap) -> Self {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает количество свободных физических фреймов у аллокатора [`BootFrameAllocator`].
    pub(super) fn count(&self) -> usize {
        self.block.count()
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

    /// Выделяет ровно один физический фрейм.
    ///
    /// Если свободных физических фреймов не осталось,
    /// возвращает ошибку [`Error::NoFrame`].
    pub(super) fn allocate(&mut self) -> Result<Frame> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Проверяет, что заданный физический фрейм управляется этим аллокатором.
    pub(super) fn is_managed(
        &self,
        frame: Frame,
    ) -> bool {
        self.initial_block.contains(frame)
    }

    /// Проверяет, что заданный физический фрейм уже был выделен, причём этим аллокатором.
    pub(super) fn is_used(
        &self,
        frame: Frame,
    ) -> bool {
        self.is_managed(frame) && !self.block.contains(frame)
    }
}

impl Drop for BootFrameAllocator {
    fn drop(&mut self) {
        let leaked_frame_count = self.count();
        info!(frame_allocator = "boot", block = %self.block, leaked_frame_count, "drop");
        assert_eq!(leaked_frame_count, 0, "leaked some memory frames");
    }
}
