use core::{
    cmp::{
        self,
        Ordering,
    },
    fmt,
    iter::Filter,
    mem,
    ops::Add,
};

use chrono::{
    DateTime,
    Utc,
};
use static_assertions::const_assert;

use ku::{
    error::{
        Error::{
            FileExists,
            FileNotFound,
            InvalidArgument,
            NoDisk,
            NotDirectory,
        },
        Result,
    },
    memory::{
        Block,
        Virt,
        size::Size,
    },
    time,
};

use super::{
    BLOCK_SIZE,
    bitmap::Bitmap,
    block_cache::BlockCache,
    directory_entry::DirectoryEntry,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(usize)]
pub enum Kind {
    #[default]
    Unused = 0,
    File = 1,
    Directory = 2,
}

#[derive(Debug, Default)]
#[repr(C)]
pub(super) struct Inode {
    kind: Kind,
    modify_time: DateTime<Utc>,
    size: usize,
    root_blocks: [usize; MAX_HEIGHT],
}

impl Inode {
    pub(super) fn kind(&self) -> Kind {
        self.kind
    }

    pub(super) fn set_kind(
        &mut self,
        kind: Kind,
    ) {
        assert!(self.kind == Kind::Unused || kind == Kind::Unused);
        self.kind = kind;
    }

    pub(super) fn remove(
        &mut self,
        block_bitmap: &mut Bitmap,
    ) -> Result<()> {
        self.set_size(0, block_bitmap)?;
        assert!(self.root_blocks.iter().all(|&block| block == NO_BLOCK));
        self.set_kind(Kind::Unused);

        Ok(())
    }

    pub(super) fn size(&self) -> usize {
        self.size
    }

    pub(super) fn set_size(
        &mut self,
        size: usize,
        block_bitmap: &mut Bitmap,
    ) -> Result<()> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn max_size() -> usize {
        let max_blocks = (0 .. MAX_HEIGHT as u32)
            .map(|height| INDIRECT_BLOCK_ARITY.pow(height))
            .reduce(Add::add)
            .unwrap();

        max_blocks * BLOCK_SIZE
    }

    pub(super) fn modify_time(&self) -> DateTime<Utc> {
        self.modify_time
    }

    pub(super) fn find(
        &mut self,
        name: &str,
    ) -> Result<&mut Inode> {
        self.find_entry(name, None).map(|entry| entry.inode_mut())
    }

    pub(super) fn insert(
        &mut self,
        name: &str,
        kind: Kind,
        block_bitmap: &mut Bitmap,
    ) -> Result<&mut Inode> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn list(&mut self) -> Result<List> {
        Ok(List(
            self.iter()?.filter(|entry| entry.inode().kind() != Kind::Unused),
        ))
    }

    pub(super) fn read(
        &mut self,
        offset: usize,
        buffer: &mut [u8],
    ) -> Result<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    pub(super) fn write(
        &mut self,
        offset: usize,
        buffer: &[u8],
        block_bitmap: &mut Bitmap,
    ) -> Result<usize> {
        // TODO: your code here.
        unimplemented!();
    }

    fn find_entry(
        &mut self,
        name: &str,
        block_bitmap: Option<&mut Bitmap>,
    ) -> Result<&mut DirectoryEntry> {
        // TODO: your code here.
        unimplemented!();
    }

    fn iter(&mut self) -> Result<Iter> {
        if self.kind == Kind::Directory {
            assert_eq!(self.size % BLOCK_SIZE, 0);

            Ok(Iter {
                block: Block::default(),
                block_number: 0,
                entry: 0,
                inode: self,
            })
        } else {
            Err(NotDirectory)
        }
    }

    fn block_entry(
        &mut self,
        inode_block_number: usize,
        block_bitmap: Option<&mut Bitmap>,
    ) -> Result<&mut usize> {
        // TODO: your code here.
        unimplemented!();
    }

    fn block(
        &mut self,
        number: usize,
    ) -> Result<Block<Virt>> {
        assert!(number < self.size.div_ceil(BLOCK_SIZE));
        BlockCache::block(*self.block_entry(number, None)?)
    }

    fn extend(
        &mut self,
        block_bitmap: &mut Bitmap,
    ) -> Result<()> {
        self.set_size(self.size() + BLOCK_SIZE, block_bitmap)
    }
}

impl fmt::Display for Inode {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ {:?}, size: {} B = {}, modify: {} }}",
            self.kind(),
            self.size(),
            Size::bytes(self.size()),
            self.modify_time(),
        )
    }
}

pub(super) struct Iter<'a> {
    block: Block<Virt>,
    block_number: usize,
    entry: usize,
    inode: &'a mut Inode,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a mut DirectoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: your code here.
        unimplemented!();
    }
}

pub(super) struct List<'a>(Filter<Iter<'a>, fn(&&mut DirectoryEntry) -> bool>);

impl<'a> Iterator for List<'a> {
    type Item = &'a mut DirectoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[allow(unused_mut)] // TODO: remove before flight.
fn find_tree(
    tree_count: usize,
    mut leaf_number: usize,
) -> Result<(usize, usize, usize)> {
    // TODO: your code here.
    unimplemented!();
}

fn remove_tree(
    node: &mut usize,
    height: usize,
    block_bitmap: &mut Bitmap,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

#[allow(unused_mut)] // TODO: remove before flight.
fn traverse_tree<'a>(
    mut node: &'a mut usize,
    height: usize,
    mut leaf_count: usize,
    mut leaf_number: usize,
    mut block_bitmap: Option<&mut Bitmap>,
) -> Result<&'a mut usize> {
    // TODO: your code here.
    unimplemented!();
}

fn next_level<'a>(
    node: &'a mut usize,
    block_bitmap: &mut Option<&mut Bitmap>,
) -> Result<&'a mut [usize]> {
    // TODO: your code here.
    unimplemented!();
}

const_assert!(BLOCK_SIZE % mem::size_of::<usize>() == 0);
const_assert!(BLOCK_SIZE % mem::size_of::<Inode>() == 0);

const INDIRECT_BLOCK_ARITY: usize = BLOCK_SIZE / mem::size_of::<usize>();
const MAX_HEIGHT: usize = 4;
const NO_BLOCK: usize = 0;
