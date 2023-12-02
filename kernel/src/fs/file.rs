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
    error::{
        Error,
        Result,
    },
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

/// Интерфейс к файлaм и директориям файловой системы.
#[derive(Debug)]
pub struct File {
    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap) файловой системы.
    block_bitmap: Bitmap,

    /// [Inode](https://en.wikipedia.org/wiki/Inode) файла.
    inode: Virt,
}

impl File {
    /// Создаёт [`File`] для доступа к заданному `inode`.
    pub(super) fn new(
        inode: &Inode,
        block_bitmap: &Bitmap,
    ) -> Self {
        Self {
            block_bitmap: block_bitmap.clone(),
            inode: Virt::from_ref(inode),
        }
    }

    /// Тип --- файл или директория.
    pub fn kind(&self) -> Kind {
        self.inode().kind()
    }

    /// Время последней модификации файла или директории.
    pub fn modify_time(&self) -> DateTime<Utc> {
        self.inode().modify_time()
    }

    /// Размер данных в байтах.
    pub fn size(&self) -> usize {
        self.inode().size()
    }

    /// Устанавливает размер данных в байтах.
    /// Если файл расширяется, то новые блоки с данными содержат нули.
    /// При необходимости выделяет или освобождает блоки.
    /// Обновляет время последней модификации файла.
    pub fn set_size(
        &mut self,
        size: usize,
    ) -> Result<()> {
        self.inode().set_size(size, self.block_bitmap())
    }

    /// Находит файл или поддиректорию с именем `name` в директории.
    /// Возвращает ошибку [`Error::FileNotFound`], если такого файла нет.
    pub fn find(
        &mut self,
        name: &str,
    ) -> Result<File> {
        self.inode().find(name).map(|inode| File::new(inode, &self.block_bitmap))
    }

    /// Возвращает список файлов и поддиректорий в директории.
    pub fn list(&self) -> Result<Vec<Entry>> {
        let mut list = Vec::new();
        for inode in self.inode().list()? {
            list.push(Entry::try_from(inode)?);
        }

        Ok(list)
    }

    /// Вставляет в директорию запись с именем `name` и типом `kind`.
    /// Обновляет как время модификации выделенной записи, так и время модификации самой директории.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::FileExists`] если запись с таким именем уже есть.
    ///   - [`Error::InvalidArgument`] если `kind` равен [`Kind::Unused`].
    pub fn insert(
        &mut self,
        name: &str,
        kind: Kind,
    ) -> Result<File> {
        self.inode()
            .insert(name, kind, self.block_bitmap())
            .map(|inode| File::new(inode, &self.block_bitmap))
    }

    /// Удаляет файл.
    pub fn remove(&mut self) -> Result<()> {
        self.inode().remove(self.block_bitmap())
    }

    /// Читает из файла по смещению `offset` в буфер `buffer` столько байт,
    /// сколько остаётся до конца файла или до конца буфера.
    ///
    /// Возвращает количество прочитанных байт.
    /// Если `offset` равен размеру файла, возвращает `0` прочитанных байт.
    ///
    /// Возвращает ошибки:
    ///   - [`Error::NotFile`] если [Inode](https://en.wikipedia.org/wiki/Inode) не является файлом.
    ///   - [`Error::InvalidArgument`] если `offset` превышает размер файла.
    pub fn read(
        &mut self,
        offset: usize,
        buffer: &mut [u8],
    ) -> Result<usize> {
        self.inode().read(offset, buffer)
    }

    /// Записывает в файл по смещению `offset` байты из буфера `buffer`.
    /// При необходимости расширяет размер файла.
    ///
    /// Возвращает количество записанных байт.
    /// Если `offset` превышает размер файла, расширяет файл нулями до заданного `offset`.
    ///
    /// Возвращает ошибку [`Error::NotFile`]
    /// если [Inode](https://en.wikipedia.org/wiki/Inode) не является файлом.
    pub fn write(
        &mut self,
        offset: usize,
        buffer: &[u8],
    ) -> Result<usize> {
        self.inode().write(offset, buffer, self.block_bitmap())
    }

    /// Возвращает [битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap) файловой системы.
    fn block_bitmap(&mut self) -> &mut Bitmap {
        &mut self.block_bitmap
    }

    /// Возвращает [inode](https://en.wikipedia.org/wiki/Inode) файла.
    fn inode(&self) -> &'static mut Inode {
        unsafe { self.inode.try_into_mut().unwrap() }
    }
}

/// Элемент списка файлов и поддиректорий в директории.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Тип --- файл или поддиректория.
    kind: Kind,

    /// Время последней модификации файла или поддиректории.
    modify_time: DateTime<Utc>,

    /// Имя файла или поддиректории.
    name: String,

    /// Размер файла или поддиректории в байтах.
    size: usize,
}

impl Entry {
    /// Тип --- файл или поддиректория.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Время последней модификации файла или поддиректории.
    pub fn modify_time(&self) -> DateTime<Utc> {
        self.modify_time
    }

    /// Имя файла или поддиректории.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Размер файла или поддиректории в байтах.
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

impl TryFrom<&mut DirectoryEntry> for Entry {
    type Error = Error;

    fn try_from(entry: &mut DirectoryEntry) -> Result<Self> {
        Ok(Self {
            kind: entry.inode().kind(),
            modify_time: entry.inode().modify_time(),
            name: String::from(entry.name()?),
            size: entry.inode().size(),
        })
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
