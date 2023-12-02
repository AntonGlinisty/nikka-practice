use core::{
    fmt,
    mem,
    str,
};

use static_assertions::const_assert;

use ku::error::{
    Error::{
        InvalidArgument,
        Medium,
    },
    Result,
};

use super::{
    BLOCK_SIZE,
    inode::Inode,
};

// Used in docs.
#[allow(unused)]
use ku::error::Error;

/// Запись [директории](https://en.wikipedia.org/wiki/Directory_(computing)) с [`Inode`],
/// который содержится в этой директории, и его именем.
#[derive(Debug)]
#[repr(C)]
pub(super) struct DirectoryEntry {
    /// [`Inode`] записи [`DirectoryEntry`].
    inode: Inode,

    /// Имя соответствующего записи файла или поддиректории.
    name: [u8; MAX_NAME_LEN],
}

impl DirectoryEntry {
    /// Возвращает неизменяемый [`Inode`] записи [`DirectoryEntry`].
    pub(super) fn inode(&self) -> &Inode {
        &self.inode
    }

    /// Возвращает изменяемый [`Inode`] записи [`DirectoryEntry`].
    pub(super) fn inode_mut(&mut self) -> &mut Inode {
        &mut self.inode
    }

    /// Возвращает имя файла или поддиректории.
    ///
    /// Возвращает ошибку [`Error::Medium`],
    /// если имя на диске не корректно.
    pub(super) fn name(&self) -> Result<&str> {
        let len = self
            .name
            .iter()
            .enumerate()
            .find(|x| *x.1 == 0)
            .map(|x| x.0)
            .unwrap_or(MAX_NAME_LEN);

        let name = str::from_utf8(&self.name[.. len]).map_err(|_| Medium)?;

        Self::validate(name).map_err(|_| Medium)?;

        Ok(name)
    }

    /// Устанавливает имя `name` для файла или поддиректории.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`],
    /// если `name` содержит не-[ASCII](https://en.wikipedia.org/wiki/ASCII) символы
    /// или разделитель пути `/`.
    pub(super) fn set_name(
        &mut self,
        name: &str,
    ) -> Result<()> {
        Self::validate(name)?;

        let bytes = name.as_bytes();
        self.name[.. bytes.len()].copy_from_slice(bytes);
        self.name[bytes.len() ..].fill(0);

        Ok(())
    }

    /// Проверяет допустимость имени файла или директории.
    ///
    /// Возвращает ошибку [`Error::InvalidArgument`],
    /// если `name` содержит не-[ASCII](https://en.wikipedia.org/wiki/ASCII) символы
    /// или разделитель пути `/`.
    fn validate(name: &str) -> Result<()> {
        let bytes = name.as_bytes();

        if bytes.is_empty() ||
            bytes.len() > MAX_NAME_LEN ||
            bytes.iter().any(|&x| x == 0 || x == b'/' || !x.is_ascii())
        {
            Err(InvalidArgument)
        } else {
            Ok(())
        }
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

/// Максимальный размер имён файлов и директорий.
pub const MAX_NAME_LEN: usize = (1 << 7) - mem::size_of::<Inode>();
