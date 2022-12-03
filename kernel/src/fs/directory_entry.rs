use core::{
    fmt,
    mem,
    str,
};

use static_assertions::const_assert;

use ku::error::{
    Error::InvalidArgument,
    Result,
};

use super::{
    BLOCK_SIZE,
    inode::Inode,
};

#[derive(Debug)]
#[repr(C)]
pub(super) struct DirectoryEntry {
    inode: Inode,
    name: [u8; MAX_NAME_LEN],
}

impl DirectoryEntry {
    pub(super) fn inode(&self) -> &Inode {
        &self.inode
    }

    pub(super) fn inode_mut(&mut self) -> &mut Inode {
        &mut self.inode
    }

    pub(super) fn name(&self) -> &str {
        let len = self
            .name
            .iter()
            .enumerate()
            .find(|x| *x.1 == 0)
            .map(|x| x.0)
            .unwrap_or(MAX_NAME_LEN);

        assert!(len > 0);

        str::from_utf8(&self.name[.. len]).unwrap()
    }

    pub(super) fn set_name(
        &mut self,
        name: &str,
    ) -> Result<()> {
        let bytes = name.as_bytes();
        if bytes.is_empty() ||
            bytes.len() > MAX_NAME_LEN ||
            bytes.iter().any(|&x| x == 0 || x == b'/' || !x.is_ascii())
        {
            return Err(InvalidArgument);
        }

        self.name[.. bytes.len()].copy_from_slice(bytes);
        self.name[bytes.len() ..].fill(0);

        Ok(())
    }
}

impl fmt::Display for DirectoryEntry {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(formatter, "{:?} {}", self.name(), self.inode())
    }
}

const_assert!(BLOCK_SIZE % mem::size_of::<DirectoryEntry>() == 0);

pub const MAX_NAME_LEN: usize = (1 << 7) - mem::size_of::<Inode>();
