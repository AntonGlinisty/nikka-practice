extern crate alloc;

use alloc::{
    string::String,
    vec,
    vec::Vec,
};
use core::str;

use ku::memory::size::MiB;

use kernel::{
    Subsystems,
    fs::{
        File,
        FileSystem,
        Kind,
        test_scaffolding::BLOCK_SIZE,
    },
    log::info,
};

mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn fs() {
    FileSystem::format(FS_DISK, FS_SIZE).unwrap();
    let mut fs = FileSystem::mount(FS_DISK).unwrap();

    test_list(&mut fs, &[]);

    test_basic_operations(&mut fs);

    test_list(
        &mut fs,
        &[
            "/file-1",
            "/file-2",
            "/dir-1",
            "/dir-1/file-4",
            "/dir-1/file-5",
            "/dir-1/dir-2",
            "/dir-1/dir-2/dir-3",
            "/dir-1/dir-2/dir-3/file-3",
        ],
    );

    remove_all(&mut fs);

    test_list(&mut fs, &[]);
}

fn test_basic_operations(fs: &mut FileSystem) {
    let mut root = fs.open("").unwrap();

    root.insert("file-1", Kind::File).unwrap().set_size(FILE_1_ZERO_COUNT).unwrap();
    root.insert("file-to-be-erased", Kind::File).unwrap();
    root.insert("file-2", Kind::File)
        .unwrap()
        .write(FILE_2_INITIAL_ZERO_COUNT, &[b'*'; 6789])
        .unwrap();
    let mut dir1 = root.insert("dir-1", Kind::Directory).unwrap();
    dir1.insert("file-4", Kind::File).unwrap();
    dir1.insert("file-5", Kind::File).unwrap();
    let mut dir2 = dir1.insert("dir-2", Kind::Directory).unwrap();
    dir2.insert("dir-3", Kind::Directory)
        .unwrap()
        .insert("file-3", Kind::File)
        .unwrap();

    assert!(root.insert("file-1", Kind::File).is_err());

    fs.open("file-to-be-erased").unwrap().remove().unwrap();

    let mut buffer = [b'-'; 1024];

    assert!(fs.open("file-1").is_ok());
    assert!(fs.open("file-1/").is_err());
    assert!(fs.open("/file-2").is_ok());
    assert_eq!(
        fs.open("/file-1").unwrap().read(FILE_1_ZERO_COUNT - 3, &mut buffer[.. 16]),
        Ok(3),
    );
    assert_eq!(buffer[.. 3], [0; 3]);
    assert!(
        fs.open("/file-1")
            .unwrap()
            .read(FILE_1_ZERO_COUNT + 1, &mut buffer[.. 16])
            .is_err(),
    );
    assert_eq!(
        fs.open("/file-1").unwrap().read(FILE_1_ZERO_COUNT, &mut buffer[.. 16]),
        Ok(0),
    );
    assert_eq!(
        fs.open("/file-2")
            .unwrap()
            .read(FILE_2_INITIAL_ZERO_COUNT - 1, &mut buffer[.. 1]),
        Ok(1),
    );
    assert_eq!(buffer[0], 0);
    assert_eq!(
        fs.open("/file-2")
            .unwrap()
            .read(FILE_2_INITIAL_ZERO_COUNT - 2, &mut buffer[.. 4]),
        Ok(4),
    );
    assert_eq!(buffer[.. 4], [0, 0, b'*', b'*']);
    assert!(fs.open("/file-2").unwrap().set_size(FILE_2_INITIAL_ZERO_COUNT).is_ok());
    assert!(fs.open("/file-2").unwrap().set_size(FILE_2_FINAL_ZERO_COUNT).is_ok());
    assert_eq!(
        fs.open("/file-2")
            .unwrap()
            .read(FILE_2_INITIAL_ZERO_COUNT - 2, &mut buffer[.. 4]),
        Ok(4),
    );
    assert_eq!(buffer[.. 4], [0; 4]);
    assert!(fs.open("/dir-1").is_ok());
    assert!(fs.open("/dir-1/").is_ok());
    assert!(fs.open("/dir-1/file-4").is_ok());
    assert!(fs.open("/dir-1/file-5").is_ok());
    assert!(fs.open("/dir-1/file-5/").is_err());
    assert!(fs.open("/dir-1/dir-2").is_ok());
    assert!(fs.open("/dir-1/dir-2/dir-3").is_ok());
    assert!(fs.open("/dir-1/dir-2/dir-3/").is_ok());
    assert!(fs.open("/dir-1/dir-2/dir-3/file-3").is_ok());
    assert!(fs.open("file-to-be-erased").is_err());
    assert!(fs.open("no-such-file").is_err());
    assert!(fs.open("no-such-dir/file").is_err());
}

fn test_list(
    fs: &mut FileSystem,
    expected: &[&str],
) {
    let mut actual: Vec<_> = vec![];

    let root = fs.open("").unwrap();
    let usage = build_list(fs, &root, "", &mut actual);

    assert_eq!(actual, expected);

    if actual.is_empty() {
        assert_eq!(usage, 0);
        assert_eq!(fs.used_space(), 0);
    }

    assert!(usage <= fs.used_space() + POSSIBLE_HOLE_SIZE);
    assert!(fs.used_space() <= 4 * usage);
}

fn build_list(
    fs: &mut FileSystem,
    inode: &File,
    directory_path: &str,
    list: &mut Vec<String>,
) -> usize {
    let mut usage = 0;

    for entry in inode.list().unwrap() {
        let mut path = String::new();
        path.push_str(directory_path);
        path.push('/');
        path.push_str(entry.name());
        info!(path = %&path, %entry);
        list.push(path.clone());
        usage += entry.size().next_multiple_of(BLOCK_SIZE);
        if entry.kind() == Kind::Directory {
            let directory = fs.open(&path).unwrap();
            build_list(fs, &directory, &path, list);
        }
    }

    usage
}

fn remove_all(fs: &mut FileSystem) {
    let mut root = fs.open("").unwrap();
    remove_recursive(fs, &root, "");
    root.set_size(0).unwrap();
}

fn remove_recursive(
    fs: &mut FileSystem,
    inode: &File,
    directory_path: &str,
) {
    for entry in inode.list().unwrap() {
        let mut path = String::new();
        path.push_str(directory_path);
        path.push('/');
        path.push_str(entry.name());
        info!(path = %&path, %entry, "removing");
        let mut file = fs.open(&path).unwrap();
        if entry.kind() == Kind::Directory {
            remove_recursive(fs, &file, &path);
        }
        file.remove().unwrap();
    }
}

const FILE_1_ZERO_COUNT: usize = 5678;
const FILE_2_FINAL_ZERO_COUNT: usize = 9876;
const FILE_2_INITIAL_ZERO_COUNT: usize = 1234;
const FS_DISK: usize = 1;
const FS_SIZE: usize = 32 * MiB;
const POSSIBLE_HOLE_SIZE: usize = FILE_1_ZERO_COUNT.next_multiple_of(BLOCK_SIZE) +
    FILE_2_FINAL_ZERO_COUNT.next_multiple_of(BLOCK_SIZE);
