use alloc::{
    string::String,
    vec::Vec,
};
use core::fmt;

use chrono::{
    DateTime,
    Utc,
};

use ku::{
    error::Result,
    memory::{
        Size,
        Virt,
    },
};

use super::{
    bitmap::Bitmap,
    directory_entry::DirectoryEntry,
    inode::{
        Inode,
        Kind,
    },
};

#[derive(Debug)]
pub struct File {
    block_bitmap: Bitmap,
    inode: Virt,
}

impl File {
    pub(super) fn new(
        inode: &Inode,
        block_bitmap: &Bitmap,
    ) -> Self {
        Self {
            block_bitmap: block_bitmap.clone(),
            inode: Virt::from_ref(inode),
        }
    }

    pub fn kind(&self) -> Kind {
        self.inode().kind()
    }

    pub fn set_kind(
        &mut self,
        kind: Kind,
    ) {
        self.inode().set_kind(kind)
    }

    pub fn modify_time(&self) -> DateTime<Utc> {
        self.inode().modify_time()
    }

    pub fn size(&self) -> usize {
        self.inode().size()
    }

    pub fn set_size(
        &mut self,
        size: usize,
    ) -> Result<()> {
        self.inode().set_size(size, self.block_bitmap())
    }

    pub fn find(
        &mut self,
        name: &str,
    ) -> Result<File> {
        self.inode().find(name).map(|inode| File::new(inode, &self.block_bitmap))
    }

    pub fn list(&self) -> Result<Vec<Entry>> {
        Ok(self.inode().list()?.map(Entry::from).collect())
    }

    pub fn insert(
        &mut self,
        name: &str,
        kind: Kind,
    ) -> Result<File> {
        self.inode()
            .insert(name, kind, self.block_bitmap())
            .map(|inode| File::new(inode, &self.block_bitmap))
    }

    pub fn remove(&mut self) -> Result<()> {
        self.inode().remove(self.block_bitmap())
    }

    pub fn read(
        &mut self,
        offset: usize,
        buffer: &mut [u8],
    ) -> Result<usize> {
        self.inode().read(offset, buffer)
    }

    pub fn write(
        &mut self,
        offset: usize,
        buffer: &[u8],
    ) -> Result<usize> {
        self.inode().write(offset, buffer, self.block_bitmap())
    }

    fn block_bitmap(&mut self) -> &mut Bitmap {
        &mut self.block_bitmap
    }

    fn inode(&self) -> &'static mut Inode {
        unsafe { self.inode.try_into_mut().unwrap() }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    kind: Kind,
    modify_time: DateTime<Utc>,
    name: String,
    size: usize,
}

impl Entry {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn modify_time(&self) -> DateTime<Utc> {
        self.modify_time
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl fmt::Display for Entry {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{}, {:?}, {} B = {}, {}",
            self.name(),
            self.kind(),
            self.size(),
            Size::bytes(self.size()),
            self.modify_time(),
        )
    }
}

impl From<&mut DirectoryEntry> for Entry {
    fn from(entry: &mut DirectoryEntry) -> Self {
        Self {
            kind: entry.inode().kind(),
            modify_time: entry.inode().modify_time(),
            name: String::from(entry.name()),
            size: entry.inode().size(),
        }
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use super::super::test_scaffolding::{
        Bitmap,
        Inode,
    };

    use super::File;

    pub fn make_file(
        inode: &Inode,
        block_bitmap: &Bitmap,
    ) -> File {
        File::new(&inode.0, &block_bitmap.0)
    }
}
