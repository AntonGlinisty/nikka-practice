extern crate alloc;

use alloc::{
    format,
    vec,
    vec::Vec,
};
use core::{
    cmp,
    mem,
};

use chrono::Duration;

use ku::memory::size::{
    MiB,
    Size,
};

use kernel::{
    Subsystems,
    fs::{
        File,
        Kind,
        test_scaffolding::{
            BLOCK_SIZE,
            flush_block,
            make_file,
        },
    },
    log::debug,
    memory::Virt,
    time::{
        self,
        TscDuration,
    },
};

mod fs_helpers;
mod init;

init!(Subsystems::MEMORY);

#[test_case]
fn block_entry() {
    let (mut block_bitmap, _, mut inodes) = fs_helpers::simple_fs(Kind::File);
    let inode = &mut inodes[0];

    const ENTRY_SIZE: usize = mem::size_of::<usize>();
    const ENTRIES_PER_BLOCK: usize = BLOCK_SIZE / ENTRY_SIZE;
    const FIRST_ENTRY_THROUGH_NON_DIRECT_BLOCK: usize = 1;

    let mut prev_entry: Option<Virt> = None;
    let check_entries = 3 * ENTRIES_PER_BLOCK.pow(2);

    for i in 0 .. check_entries {
        let entry = inode.block_entry(i, &mut block_bitmap).unwrap();

        let current_entry = Virt::from_ref(entry);
        let entry_order_violation = if let Some(prev_entry) = prev_entry {
            prev_entry >= current_entry
        } else {
            false
        };
        if entry_order_violation || i % 100_000 == 0 {
            debug!(i, ?prev_entry, %current_entry, entry_order_violation);
        }
        assert!(!entry_order_violation);

        if let Some(prev_entry) = prev_entry {
            let prev_entry_inc = (prev_entry + ENTRY_SIZE).unwrap();
            assert_eq!(current_entry, prev_entry_inc);
        }

        prev_entry = if i >= FIRST_ENTRY_THROUGH_NON_DIRECT_BLOCK &&
            (i + 1 - FIRST_ENTRY_THROUGH_NON_DIRECT_BLOCK) % ENTRIES_PER_BLOCK != 0
        {
            Some(current_entry)
        } else {
            None
        };

        *entry = i;
    }

    debug!("block entry allocation is done, checking the entries");

    for i in 0 .. check_entries {
        let entry = inode.block_entry(i, &mut block_bitmap).unwrap();
        if *entry != i || i % 100_000 == 0 {
            debug!(i, actual = *entry, expected = i);
        }
        assert_eq!(*entry, i);
    }
}

#[test_case]
fn read_write_speed() {
    let (block_bitmap, inode_bitmap, inodes) = fs_helpers::simple_fs(Kind::File);
    let mut file = make_file(&inodes, &block_bitmap, &inode_bitmap);

    let mut buffer = vec![0x77_u8; 10 * MiB];
    let size = Size::from_slice(&buffer);
    let timer = time::timer();
    file.write(0, &buffer).unwrap();
    let elapsed = timer.elapsed();
    debug!(
        throughput_per_second = %throughput_per_second(size, elapsed),
        %size,
        %elapsed,
        "disk read speed",
    );

    buffer.fill(0);

    let timeout = TscDuration::try_from(Duration::seconds(2)).unwrap();

    let timer = time::timer();
    file.read(0, &mut buffer).unwrap();
    let elapsed = timer.elapsed();
    debug!(
        throughput_per_second = %throughput_per_second(size, elapsed),
        %size,
        %elapsed,
        %timeout,
        "file system read speed",
    );
    assert!(elapsed < timeout);

    assert_eq!(buffer[123456], 0x77);
    buffer.fill(0x33);

    let timer = time::timer();
    file.write(0, &buffer).unwrap();
    let elapsed = timer.elapsed();
    debug!(
        throughput_per_second = %throughput_per_second(size, elapsed),
        %size,
        %elapsed,
        %timeout,
        "file system write speed",
    );
    assert!(elapsed < timeout);

    let timer = time::timer();
    for block in 0 .. fs_helpers::BLOCK_COUNT {
        flush_block(block).unwrap();
    }
    let elapsed = timer.elapsed();
    debug!(
        throughput_per_second = %throughput_per_second(size, elapsed),
        %size,
        %elapsed,
        "disk write speed",
    );

    fn throughput_per_second(
        size: Size,
        elapsed: TscDuration,
    ) -> Size {
        let elapsed: Duration = elapsed.try_into().unwrap();
        let throughput_per_second = u128::try_from(size.num_bytes()).unwrap() * 1_000_000 /
            u128::try_from(elapsed.num_microseconds().unwrap()).unwrap();
        Size::bytes(throughput_per_second.try_into().unwrap())
    }
}

#[test_case]
fn write_read() {
    let (block_bitmap, inode_bitmap, inodes) = fs_helpers::simple_fs(Kind::File);
    let mut file = make_file(&inodes, &block_bitmap, &inode_bitmap);

    let data: Vec<_> = (0 .. MiB).map(|x| (x + x * x) as u8).collect();
    let mut offset = 0;

    while offset < data.len() {
        for size in (0 ..= 3 * BLOCK_SIZE).filter(|x| (x + 3) % BLOCK_SIZE < 6) {
            let real_size = cmp::min(offset + size, data.len()) - offset;
            assert_eq!(
                file.write(offset, &data[offset .. offset + real_size]),
                Ok(real_size),
            );
            offset += real_size;
        }
    }

    let mut buffer = [0; MiB];
    offset = 0;

    while offset < data.len() {
        for size in (0 ..= 4 * BLOCK_SIZE).filter(|x| (x + 4) % BLOCK_SIZE < 7) {
            let real_size = cmp::min(offset + size, data.len()) - offset;
            assert_eq!(
                file.read(offset, &mut buffer[offset .. offset + real_size]),
                Ok(real_size),
            );
            offset += real_size;
        }
    }

    debug!(actual = ?buffer[..10], expected = ?data[..10]);
    for (actual, expected) in buffer.iter().zip(data) {
        assert_eq!(*actual, expected);
    }
}

#[test_case]
fn big_file() {
    let (block_bitmap, inode_bitmap, inodes) = fs_helpers::simple_fs(Kind::Directory);
    let mut root = make_file(&inodes, &block_bitmap, &inode_bitmap);
    let mut file = root.insert("big-file", Kind::File).unwrap();
    let start_free_block_count = block_bitmap.free_count();

    let file_block_count = start_free_block_count * 99 / 100;
    file.set_size(file_block_count * BLOCK_SIZE).unwrap();
    debug!(file_block_count, file_size = %Size::bytes(file.size()));

    for file_block in 0 .. file_block_count {
        let data = format!("file block {file_block}");
        let data_len = data.len();
        assert_eq!(
            file.write(file_block * BLOCK_SIZE, data.as_bytes()),
            Ok(data_len),
        );
    }

    for file_block in 0 .. file_block_count {
        let data = format!("file block {file_block}");
        let data_len = data.len();
        let mut buffer = vec![0; data_len];
        assert_eq!(
            file.read(file_block * BLOCK_SIZE, &mut buffer),
            Ok(data_len),
        );
        assert_eq!(buffer, data.as_bytes());
    }

    let free_block_count = block_bitmap.free_count();
    let used_block_count = start_free_block_count - free_block_count;
    debug!(free_block_count, used_block_count);

    file.remove().unwrap();

    let end_free_block_count = block_bitmap.free_count();
    let leaked_block_count = start_free_block_count - end_free_block_count;
    assert_eq!(leaked_block_count, 0);
}

#[test_case]
fn set_size() {
    let (block_bitmap, inode_bitmap, inodes) = fs_helpers::simple_fs(Kind::File);
    let mut file = make_file(&inodes, &block_bitmap, &inode_bitmap);
    let start_free_block_count = block_bitmap.free_count();

    let filter = |x: &usize| (x + 2) % (BLOCK_SIZE / 3) <= 4;

    for offset in (0 ..= 2 * BLOCK_SIZE).filter(filter) {
        for size in (0 ..= 2 * BLOCK_SIZE).filter(filter) {
            if offset == size {
                debug!(offset, size);
            }
            check_expansion_is_zero_filled(&mut file, offset, size);
            file.set_size(offset).unwrap();
            check_expansion_is_zero_filled(&mut file, offset, size);
            file.set_size(offset).unwrap();
        }
    }

    file.set_size(0).unwrap();
    let end_free_block_count = block_bitmap.free_count();
    assert_eq!(start_free_block_count, end_free_block_count);

    fn check_expansion_is_zero_filled(
        file: &mut File,
        offset: usize,
        size: usize,
    ) {
        let mut buffer = vec![1; size];

        file.set_size(offset + size).unwrap();
        file.read(offset, &mut buffer).unwrap();
        assert!(buffer.iter().all(|&x| x == 0));

        buffer.fill(b'*');
        assert_eq!(file.write(offset, &buffer), Ok(size));
        file.read(offset, &mut buffer).unwrap();
        assert!(buffer.iter().all(|&x| x == b'*'));
    }
}
