extern crate alloc;

use alloc::format;
use core::{
    slice,
    str,
};

use ku::{
    error::Error::{
        FileExists,
        FileNotFound,
        InvalidArgument,
        NotDirectory,
        NotFile,
    },
    memory::size::Size,
};

use kernel::{
    Subsystems,
    fs::{
        Kind,
        MAX_NAME_LEN,
        test_scaffolding::make_file,
    },
    log::debug,
};

mod fs_helpers;
mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn basic_operations() {
    let (block_bitmap, inode) = fs_helpers::single_inode_fs(Kind::Directory);
    let mut directory = make_file(&inode, &block_bitmap);

    let mut buffer = [0; 1];
    assert_eq!(directory.read(0, &mut buffer).unwrap_err(), NotFile);
    assert_eq!(directory.write(0, &buffer).unwrap_err(), NotFile);

    assert!(directory.list().unwrap().is_empty());

    assert_eq!(directory.find("file-1").unwrap_err(), FileNotFound);
    assert_eq!(
        directory.insert("file-1", Kind::Unused).unwrap_err(),
        InvalidArgument,
    );
    let mut file_1 = directory.insert("file-1", Kind::File).unwrap();
    assert!(directory.find("file-1").is_ok());

    assert_eq!(file_1.list().unwrap_err(), NotDirectory);
    assert_eq!(
        file_1.insert("file-1", Kind::File).unwrap_err(),
        NotDirectory,
    );
    assert_eq!(file_1.find("file-1").unwrap_err(), NotDirectory);

    let list = directory.list().unwrap();
    debug!(?list);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].kind(), Kind::File);
    assert_eq!(list[0].name(), "file-1");
    assert_eq!(list[0].size(), 0);

    assert_eq!(file_1.write(0, &buffer), Ok(buffer.len()));

    let list = directory.list().unwrap();
    debug!(?list);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].kind(), Kind::File);
    assert_eq!(list[0].name(), "file-1");
    assert_eq!(list[0].size(), buffer.len());

    for kind in [Kind::Directory, Kind::File] {
        assert_eq!(directory.insert("file-1", kind).unwrap_err(), FileExists);
    }

    let offset_2 = 1234;
    let mut file_2 = directory.insert("file-2", Kind::File).unwrap();
    assert_eq!(file_2.write(offset_2, &buffer), Ok(buffer.len()));

    let mut list = directory.list().unwrap();
    slice::heapsort(&mut list, |a, b| a.name() < b.name());
    debug!(?list);

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].kind(), Kind::File);
    assert_eq!(list[0].name(), "file-1");
    assert_eq!(list[0].size(), buffer.len());
    assert_eq!(list[1].kind(), Kind::File);
    assert_eq!(list[1].name(), "file-2");
    assert_eq!(list[1].size(), offset_2 + buffer.len());

    file_1.remove().unwrap();
    assert_eq!(directory.find("file-1").unwrap_err(), FileNotFound);
    file_2.remove().unwrap();
    assert_eq!(directory.find("file-2").unwrap_err(), FileNotFound);
}

#[test_case]
fn big_directory() {
    let (block_bitmap, inode) = fs_helpers::single_inode_fs(Kind::Directory);
    let mut directory = make_file(&inode, &block_bitmap);
    let start_free_block_count = block_bitmap.free_block_count();

    for file_count in 1 ..= 1000 {
        directory.insert(&format!("file-{file_count}"), Kind::File).unwrap();
        if file_count % 100 == 0 {
            debug!(file_count, directory_size = %Size::bytes(directory.size()));
        }
    }

    let free_block_count = block_bitmap.free_block_count();
    let used_block_count = start_free_block_count - free_block_count;
    debug!(free_block_count, used_block_count);

    directory.remove().unwrap();

    let end_free_block_count = block_bitmap.free_block_count();
    let leaked_block_count = start_free_block_count - end_free_block_count;
    assert_eq!(leaked_block_count, 0);
}

#[test_case]
fn max_name_len() {
    let (block_bitmap, inode) = fs_helpers::single_inode_fs(Kind::Directory);
    let mut directory = make_file(&inode, &block_bitmap);

    let name = str::from_utf8(&[b'x'; MAX_NAME_LEN]).unwrap();
    let mut file = directory.insert(name, Kind::File).unwrap();

    assert!(directory.insert(name, Kind::File).is_err());

    let list = directory.list().unwrap();
    debug!(?list);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name().len(), MAX_NAME_LEN);
    assert_eq!(list[0].name(), name);

    file.remove().unwrap();

    let name = str::from_utf8(&[b'x'; MAX_NAME_LEN + 1]).unwrap();
    assert_eq!(
        directory.insert(name, Kind::File).unwrap_err(),
        InvalidArgument,
    );
}
