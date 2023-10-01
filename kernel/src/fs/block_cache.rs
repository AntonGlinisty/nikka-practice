use lazy_static::lazy_static;
use spin::Mutex;

use ku::{
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
    process::Info,
};

use crate::memory::{
    BASE_ADDRESS_SPACE,
    mmu,
};

use super::{
    BLOCK_SIZE,
    disk::{
        Disk,
        SECTOR_SIZE,
    },
};

/// [Блочный кэш](https://en.wikipedia.org/wiki/Page_cache)
/// для ускорения работы с диском за счёт кэширования блоков файловой системы в памяти.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BlockCache {
    /// Диапазон памяти для кэширования блоков.
    cache: Block<Page>,

    /// Диск, обращения к которому кэшируются.
    disk: Disk,
}

impl BlockCache {
    /// Инициализирует блочный кэш в
    /// [синглтоне](https://en.wikipedia.org/wiki/Singleton_pattern)
    /// [`struct@BLOCK_CACHE`].
    ///
    /// Резервирует в [`struct@BASE_ADDRESS_SPACE`] блок виртуальных страниц,
    /// достаточный для отображения один в один `block_count` блоков файловой системы.
    pub(super) fn init(
        disk: Disk,
        block_count: usize,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Возвращает блок памяти блочного кэша [`struct@BLOCK_CACHE`],
    /// который отвечает блоку `block_number` диска.
    pub(super) fn block(block_number: usize) -> Result<Block<Virt>> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Записывает блок `block_number` на диск, если:
    ///
    /// - Блок отображён в память. Это означает что нему были обращения.
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
    /// И обновлённый блок на диск записан не будет.
    pub(super) fn flush_block(block_number: usize) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    /// Сбрасывает первые `count` блоков на диск методом [`BlockCache::flush_block()`].
    pub(super) fn flush(count: usize) -> Result<()> {
        for block_number in 0 .. count {
            Self::flush_block(block_number)?;
        }

        Ok(())
    }

    /// Обрабатывает Page Fault, если адрес, который его вызвал, относится к блочному кэшу.
    /// Если это так и Page Fault успешно обработан, возвращает `true`.
    /// Если адрес, вызвавший Page Fault, не относится к блочному кэшу, возвращает `false`.
    pub(crate) fn trap_handler(info: &Info) -> Result<bool> {
        // TODO: your code here.
        Ok(false) // TODO: remove before flight.
    }

    /// Возвращает синглтон [`struct@BLOCK_CACHE`], если `address` относится к блочному кэшу.
    fn get(address: Virt) -> Option<Self> {
        BLOCK_CACHE
            .lock()
            .filter(|block_cache| block_cache.cache.contains_address(address))
    }
}

lazy_static! {
    /// Блочный кэш для ускорения работы с диском
    /// за счёт кэширования блоков файловой системы в памяти.
    pub(super) static ref BLOCK_CACHE: Mutex<Option<BlockCache>> = Mutex::new(None);
}

/// Количество секторов диска в одном блоке файловой системы.
const SECTORS_PER_BLOCK: usize = BLOCK_SIZE / SECTOR_SIZE;

#[doc(hidden)]
pub mod test_scaffolding {
    use ku::{
        error::{
            Error::NoDisk,
            Result,
        },
        memory::{
            Block,
            Page,
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
    ) -> Result<()> {
        BlockCache::init(Disk::new(disk)?, block_count)
    }

    pub fn cache() -> Result<Block<Page>> {
        Ok(BLOCK_CACHE.lock().ok_or(NoDisk)?.cache)
    }

    pub fn flush_block(block_number: usize) -> Result<()> {
        BlockCache::flush_block(block_number)
    }
}
