use core::{
    alloc::Layout,
    sync::atomic::Ordering,
};

use derive_getters::Getters;
use lazy_static::lazy_static;

use ku::{
    collections::Lru,
    error::{
        Error::NoDisk,
        Result,
    },
    log::trace,
    memory::{
        Block,
        Page,
        PageFaultInfo,
        Virt,
        mmu::{
            KERNEL_RW,
            PageTableFlags,
        },
    },
    sync::FastSpinlock,
    trap::Info,
};

use crate::memory::{
    BASE_ADDRESS_SPACE,
    Translate,
};

use super::{
    BLOCK_SIZE,
    disk::{
        self,
        Disk,
        SECTOR_SIZE,
    },
};

// ANCHOR: block_cache
/// [Блочный кэш](https://en.wikipedia.org/wiki/Page_cache)
/// для ускорения работы с диском за счёт кэширования блоков файловой системы в памяти.
#[derive(Clone, Debug)]
pub struct BlockCache {
    /// Диапазон памяти для кэширования блоков.
    cache: Cache,

    /// Диск, обращения к которому кэшируются.
    disk: Disk,

    /// Политика вытеснения блоков из кэша.
    eviction_policy: Lru<usize, ()>,

    /// Статистика работы блочного кэша.
    stats: Stats,
}
// ANCHOR_END: block_cache

impl BlockCache {
    // ANCHOR: init
    /// Инициализирует блочный кэш в
    /// [синглтоне](https://en.wikipedia.org/wiki/Singleton_pattern)
    /// [`struct@BLOCK_CACHE`].
    ///
    /// Резервирует в [`BASE_ADDRESS_SPACE`] блок виртуальных страниц,
    /// достаточный для отображения один в один `block_count` блоков файловой системы.
    /// Политика вытеснения блоков из кэша ограничивает
    /// количество одновременно отображённых в память блоков параметром `capacity`.
    pub(super) fn init(
        disk: Disk,
        block_count: usize,
        capacity: usize,
    ) -> Result<()> {
        // ANCHOR_END: init
        // TODO: your code here.
        unimplemented!();
    }

    /// Диапазон памяти для кэширования блоков.
    pub(super) fn cache() -> Result<Cache> {
        if let Some(block_cache) = BLOCK_CACHE.lock().as_mut() {
            Ok(block_cache.cache)
        } else {
            Err(NoDisk)
        }
    }

    /// Статистика работы диска.
    pub fn disk_stats() -> disk::Stats {
        if let Some(block_cache) = BLOCK_CACHE.lock().as_mut() {
            block_cache.disk.stats()
        } else {
            disk::Stats::default()
        }
    }

    // ANCHOR: emplace_block
    /// Гарантирует, что блок `block_number` отображён в память блочного кэша,
    /// не читая этот блок с диска.
    /// В памяти блок может содержать как произвольный мусор,
    /// так и данные с диска, если они были загружены в блочный кэш ранее.
    /// Учитывает доступ к блоку в [`BlockCache::eviction_policy`].
    ///
    /// Возвращает этот блок.
    ///
    /// # Safety
    ///
    /// Вызывающая функция должна перезаписать блок в памяти целиком.
    pub(super) unsafe fn emplace_block(block_number: usize) -> Result<Block<Virt>> {
        // ANCHOR_END: emplace_block
        // TODO: your code here.
        unimplemented!();
    }

    // ANCHOR: fill_block
    /// Заполняет блок `block_number` в памяти блочного кэша значениями `value`,
    /// не читая этот блок с диска.
    /// Учитывает доступ к блоку в [`BlockCache::eviction_policy`].
    ///
    /// Возвращает этот блок.
    pub(super) fn fill_block<T: Clone>(
        block_number: usize,
        value: T,
    ) -> Result<Block<Virt>> {
        // ANCHOR_END: fill_block
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает блок `block_number` на диск.
    ///
    /// См. также [`BlockCache::flush_block_impl()`].
    pub(super) fn flush_block(block_number: usize) -> Result<()> {
        if !test_scaffolding::FLUSH_ENABLED.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Some(block_cache) = BLOCK_CACHE.lock().as_mut() {
            block_cache.flush_block_impl(block_number)?;
            block_cache.disk.flush()
        } else {
            Err(NoDisk)
        }
    }

    /// Сбрасывает первые `count` блоков на диск.
    ///
    /// См. также [`BlockCache::flush_block_impl()`].
    pub(super) fn flush(count: usize) -> Result<()> {
        if let Some(block_cache) = BLOCK_CACHE.lock().as_mut() {
            for block_number in 0 .. count {
                block_cache.flush_block_impl(block_number)?;
            }

            block_cache.disk.flush()
        } else {
            Err(NoDisk)
        }
    }

    // ANCHOR: trap_handler
    /// Обрабатывает Page Fault, если адрес, который его вызвал, относится к блочному кэшу.
    /// Если это так и Page Fault успешно обработан, возвращает `true`.
    /// Учитывает доступ к блоку в [`BlockCache::eviction_policy`].
    /// Если адрес, вызвавший Page Fault, не относится к блочному кэшу, возвращает `false`.
    pub(crate) fn trap_handler(info: &Info) -> Result<bool> {
        // ANCHOR_END: trap_handler
        // TODO: your code here.
        Ok(false) // TODO: remove before flight.
    }

    /// Статистика работы блочного кэша.
    pub fn stats() -> Stats {
        if let Some(block_cache) = BLOCK_CACHE.lock().as_ref() {
            block_cache.stats
        } else {
            Stats::default()
        }
    }

    // ANCHOR: flush_block_impl
    /// Записывает блок `block_number` на диск, если:
    ///
    /// - Блок отображён в память. Это означает, что к нему были обращения.
    /// - И помечен как [`PageTableFlags::DIRTY`].
    ///   То есть, в память были записи, а значит блок на диске потенциально содержит
    ///   устаревшие данные.
    ///   Если обращения к блоку были только на чтение, то данные в памяти такие же как на диске,
    ///   и можно их не записывать.
    ///   А процессор в этом случае не установит бит [`PageTableFlags::DIRTY`].
    ///
    /// После записи блока, сбрасывает бит [`PageTableFlags::DIRTY`].
    /// Он фактически означает одинаковость данных на диске и в памяти блочного кэша.
    /// Которая только что восстановлена.
    /// При этом сбрасывает и соответствующую запись в
    /// [TLB](https://en.wikipedia.org/wiki/Translation_lookaside_buffer)
    /// с помощью функции [`mmu::flush()`].
    /// Иначе процессор не узнает, что сброшен [`PageTableFlags::DIRTY`]
    /// и не проставит его в таблице страниц при следующей записи.
    /// В результате, обновлённый блок на диск записан не будет.
    fn flush_block_impl(
        &mut self,
        block_number: usize,
    ) -> Result<()> {
        // ANCHOR_END: flush_block_impl
        // TODO: your code here.
        unimplemented!();
    }

    // ANCHOR: register_usage
    /// Учитывает в [`BlockCache::eviction_policy`] доступ к блоку `block_number`.
    /// При необходимости вытесняет из кэша другой блок и записывает вытесненный блок на диск.
    fn register_usage(
        &mut self,
        block_number: usize,
    ) -> Result<()> {
        // ANCHOR_END: register_usage
        // TODO: your code here.
        Ok(()) // TODO: remove before flight.
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        let block_count = self.cache.0.count() * Page::SIZE / BLOCK_SIZE;

        for block_number in 0 .. block_count {
            self.flush_block_impl(block_number).expect("failed to flush the block cache");
        }
    }
}

/// Диапазон памяти для кэширования блоков.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cache(Block<Page>);

impl Cache {
    // ANCHOR: block
    /// Возвращает блок памяти блочного кэша,
    /// который отвечает блоку `block_number` диска.
    pub(super) fn block(
        &self,
        block_number: usize,
    ) -> Block<Virt> {
        // ANCHOR_END: block
        // TODO: your code here.
        unimplemented!();
    }
}

// ANCHOR: stats
/// Статистика работы блочного кэша.
#[derive(Clone, Copy, Default, Debug, Eq, Getters, PartialEq)]
pub struct Stats {
    /// Количество блоков, которые не пришлось записывать на диск в [`BlockCache::flush_block_impl()`].
    discards: usize,

    /// Количество блоков, которые были внесены в кэш без чтения с диска в
    /// [`BlockCache::emplace_block()`] или [`BlockCache::fill_block()`].
    emplacements: usize,

    /// Количество блоков, которые были вытеснены из кэша в [`BlockCache::register_usage()`]
    /// при вызовах [`BlockCache::trap_handler()`],
    /// [`BlockCache::emplace_block()`] или [`BlockCache::fill_block()`].
    evictions: usize,

    /// Количество блоков, которые были прочитаны с диска в [`BlockCache::trap_handler()`].
    reads: usize,

    /// Количество блоков, которые были записаны на диск в [`BlockCache::flush_block_impl()`].
    writes: usize,
}
// ANCHOR_END: stats

lazy_static! {
    /// Блочный кэш для ускорения работы с диском
    /// за счёт кэширования блоков файловой системы в памяти.
    pub(super) static ref BLOCK_CACHE: FastSpinlock<Option<BlockCache>> = FastSpinlock::new(None);
}

/// Количество секторов диска в одном блоке файловой системы.
pub(super) const SECTORS_PER_BLOCK: usize = BLOCK_SIZE / SECTOR_SIZE;

#[doc(hidden)]
pub mod test_scaffolding {
    use core::sync::atomic::{
        AtomicBool,
        Ordering,
    };

    use ku::{
        error::{
            Error::NoDisk,
            Result,
        },
        memory::{
            Block,
            Page,
            Virt,
        },
    };

    use super::{
        BLOCK_CACHE,
        BlockCache,
        Disk,
    };

    pub fn block_cache_init(
        disk: usize,
        block_count: usize,
        capacity: usize,
    ) -> Result<()> {
        BlockCache::init(Disk::new(disk)?, block_count, capacity)
    }

    pub fn cache() -> Result<Block<Page>> {
        Ok(BLOCK_CACHE.lock().as_ref().ok_or(NoDisk)?.cache.0)
    }

    pub unsafe fn emplace_block(block_number: usize) -> Result<Block<Virt>> {
        unsafe { BlockCache::emplace_block(block_number) }
    }

    pub fn flush_block(block_number: usize) -> Result<()> {
        BlockCache::flush_block(block_number)
    }

    pub fn disable_flush() {
        FLUSH_ENABLED.store(false, Ordering::Relaxed);
    }

    pub(super) static FLUSH_ENABLED: AtomicBool = AtomicBool::new(true);

    pub static SECTORS_PER_BLOCK: usize = super::SECTORS_PER_BLOCK;
}
