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
    inode::{
        Inode,
        Kind,
    },
};

// Used in docs.
#[allow(unused)]
use ku::error::Error;

/// Интерфейс к файлaм и директориям файловой системы.
#[derive(Debug)]
pub struct File {
    /// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// блоков файловой системы.
    block_bitmap: Bitmap,

    /// [Inode](https://en.wikipedia.org/wiki/Inode)
    /// директории, содержащей файл.
    directory: usize,

    /// Номер [inode](https://en.wikipedia.org/wiki/Inode) файла.
    inode: usize,

    /// [Inode](https://en.wikipedia.org/wiki/Inode)
    /// [битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
    /// файловой системы.
    inode_bitmap: Bitmap,

    /// Массив
    /// [inode](https://en.wikipedia.org/wiki/Inode)
    /// файловой системы.
    inodes: Virt,

    /// Имя файла.
    name: String,
}

impl File {
    /// Создаёт [`File`] для доступа к заданному `inode`.
    pub(super) fn new(
        name: &str,
        directory: usize,
        inode: usize,
        inodes: &[Inode],
        block_bitmap: &Bitmap,
        inode_bitmap: &Bitmap,
    ) -> Self {
        Self {
            block_bitmap: block_bitmap.clone(),
            directory,
            inode,
            inode_bitmap: inode_bitmap.clone(),
            inodes: Virt::from_ptr(inodes.as_ptr()),
            name: String::from(name),
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
    ) -> Result<Self> {
        self.inode().find(name).map(|directory_entry| {
            Self::new(
                name,
                self.inode,
                directory_entry.inode(),
                self.inodes(directory_entry.inode()),
                &self.block_bitmap,
                &self.inode_bitmap,
            )
        })
    }

    /// Возвращает список файлов и поддиректорий в директории.
    pub fn list(&self) -> Result<Vec<Entry>> {
        let mut list = Vec::new();
        for directory_entry in self.inode().list()? {
            let inode = &self.inodes(directory_entry.inode())[directory_entry.inode()];
            let entry = Entry {
                inode: directory_entry.inode(),
                kind: inode.kind(),
                modify_time: inode.modify_time(),
                name: String::from(directory_entry.name()?),
                size: inode.size(),
            };
            list.push(entry);
        }

        Ok(list)
    }

    /// Вставляет в директорию запись с именем `name` и типом `kind`.
    /// Обновляет как время модификации выделенной записи, так и время модификации самой директории.
    ///
    /// Возвращает ошибку [`Error::FileExists`] если запись с таким именем уже есть.
    pub fn insert(
        &mut self,
        name: &str,
        kind: Kind,
    ) -> Result<Self> {
        let entry = self.inode().insert(name, self.block_bitmap())?;
        let inode = self.inode_bitmap.allocate()?;
        entry.set_inode(inode);
        self.inodes(inode)[inode].init(kind);

        Ok(Self::new(
            name,
            self.inode,
            inode,
            self.inodes(inode),
            &self.block_bitmap,
            &self.inode_bitmap,
        ))
    }

    /// Удаляет файл.
    pub fn remove(&mut self) -> Result<()> {
        self.inodes(self.directory)[self.directory].find(&self.name)?.set_free();
        self.inode_bitmap.set_free(self.inode);
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
        &mut self.inodes(self.inode)[self.inode]
    }

    /// Возвращает срез
    /// [inode](https://en.wikipedia.org/wiki/Inode)
    /// файловой системы до `inode` включительно.
    fn inodes(
        &self,
        inode: usize,
    ) -> &'static mut [Inode] {
        unsafe { self.inodes.try_into_mut_slice(inode + 1).unwrap() }
    }
}

/// Элемент списка файлов и поддиректорий в директории.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Номер [inode](https://en.wikipedia.org/wiki/Inode) файла.
    inode: usize,

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
            "{}, {}, {:?}, {} B = {}, {}",
            self.inode,
            self.name(),
            self.kind(),
            self.size(),
            Size::bytes(self.size()),
            self.modify_time(),
        )
    }
}

#[doc(hidden)]
pub mod test_scaffolding {
    use core::mem;

    use super::super::test_scaffolding::{
        Bitmap,
        Inode,
    };

    use super::File;

    pub fn make_file(
        inodes: &[Inode],
        block_bitmap: &Bitmap,
        inode_bitmap: &Bitmap,
    ) -> File {
        unsafe {
            File::new(
                "",
                0,
                0,
                mem::transmute(inodes),
                mem::transmute(block_bitmap),
                mem::transmute(inode_bitmap),
            )
        }
    }
}
