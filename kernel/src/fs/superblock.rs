use core::mem;

use static_assertions::const_assert;

use ku::error::{
    Error::Medium,
    Result,
};

use super::{
    BLOCK_SIZE,
    SUPERBLOCK_BLOCK,
    block_cache::BlockCache,
    inode::{
        Inode,
        Kind,
    },
};

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct Superblock {
    magic: [u8; MAGIC.len()],
    endian: u64,
    block_count: usize,
    root: Inode,
}

const_assert!(mem::align_of::<Superblock>() == BLOCK_SIZE);
const_assert!(mem::size_of::<Superblock>() == BLOCK_SIZE);

impl Superblock {
    pub(super) fn new() -> Result<&'static mut Self> {
        Self::new_unchecked().validate()
    }

    pub(super) fn format(block_count: usize) -> Result<()> {
        unsafe {
            BlockCache::block(SUPERBLOCK_BLOCK)?
                .try_into_mut_slice::<usize>()
                .unwrap()
                .fill(0);
        }

        let superblock = Self::new_unchecked();

        superblock.magic.copy_from_slice(MAGIC.as_bytes());
        superblock.endian = ENDIAN;
        superblock.set_block_count(block_count);
        superblock.root.set_kind(Kind::Directory);

        superblock.validate()?;

        Ok(())
    }

    pub(super) fn root(&mut self) -> &mut Inode {
        &mut self.root
    }

    pub(super) fn block_count(&self) -> usize {
        self.block_count
    }

    fn new_unchecked() -> &'static mut Self {
        unsafe { BlockCache::block(SUPERBLOCK_BLOCK).unwrap().try_into_mut::<Self>().unwrap() }
    }

    fn set_block_count(
        &mut self,
        block_count: usize,
    ) {
        self.block_count = block_count;
    }

    fn validate(&mut self) -> Result<&mut Self> {
        let is_valid = self.magic == MAGIC.as_bytes() &&
            self.endian == ENDIAN &&
            self.block_count > 0 &&
            self.root.kind() == Kind::Directory;

        if is_valid {
            Ok(self)
        } else {
            Err(Medium)
        }
    }
}

const ENDIAN: u64 = 0x0102_0304_0506_0708;
const MAGIC: &str = "Nikka's simple file system";
