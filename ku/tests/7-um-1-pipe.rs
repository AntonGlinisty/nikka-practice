#![feature(custom_test_frameworks)]
#![feature(slice_group_by)]
#![no_main]
#![no_std]
#![reexport_test_harness_main = "test_main"]
#![test_runner(kernel::test_runner)]

use core::fmt;

use rand::{
    Rng,
    SeedableRng,
    distributions::Alphanumeric,
    rngs::SmallRng,
};

use ku::{
    RingBuffer,
    RingBufferWriteTx,
    error::Result,
};

use kernel::{
    Subsystems,
    log::{
        debug,
        trace,
    },
    memory::BASE_ADDRESS_SPACE,
    process::test_scaffolding,
};

mod init;

init!(Subsystems::MEMORY | Subsystems::PROCESS);

const QUALITY: Quality = Quality::Paranoid;

#[test_case]
fn stress() {
    const SEED: u64 = 314159265;

    let mut ring_buffer = test_scaffolding::map_log(&mut BASE_ADDRESS_SPACE.lock()).unwrap();

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut read_ops = Generator::new(QUALITY, SEED);
    let mut write_ops = read_ops.clone();
    let mut write_errors = 0;

    for iteration in 0 .. 10_000 {
        if iteration % 1_000 == 0 {
            debug!(
                iteration,
                read_stats = ?ring_buffer.read_stats(),
                write_stats = ?ring_buffer.write_stats(),
            );
        }

        if rng.gen_ratio(3, 4) {
            write_errors +=
                write_with_retries(&mut ring_buffer, &mut write_ops, &mut read_ops, &mut rng);
        } else {
            read(&mut ring_buffer, &mut read_ops, &mut rng);
        }
    }

    assert!(
        write_errors > 0,
        "the test does not check for RingBuffer overflow",
    );
}

fn read(
    ring_buffer: &mut RingBuffer,
    read_ops: &mut Generator,
    rng: &mut SmallRng,
) {
    let mut tx = ring_buffer.read_tx();
    let data = tx.read();

    match read_ops.quality {
        Quality::Debuggable => trace!(
            data = %RunLengthEncoding(data),
            len = data.len(),
            "read_tx"
        ),
        Quality::Paranoid => trace!(?data, len = data.len(), "read_tx"),
    }

    if rng.gen_ratio(1, 2) {
        check_read_tx(data, read_ops);
        tx.commit();
    }
}

fn check_read_tx(
    mut data: &[u8],
    read_ops: &mut Generator,
) {
    let mut count = 0;

    while !data.is_empty() {
        let expected = read_ops.generate_operation();
        trace!(operation = %expected, "read");

        if expected.commit {
            let (got, more_data) = data.split_at(expected.data.len());

            // The main point of the stress test - check that read yields the same data that was written.
            assert_eq!(*got, *expected.data);

            data = more_data;
            count += 1;
        }
    }

    if count > 1 {
        trace!(count, "multiple write transactions in one read transaction");
    }
}

fn write_with_retries(
    ring_buffer: &mut RingBuffer,
    write_ops: &mut Generator,
    read_ops: &mut Generator,
    rng: &mut SmallRng,
) -> usize {
    let operation = write_ops.generate_operation();
    let mut write_errors = 0;

    while let Err(write_error) = write(ring_buffer, &operation, rng) {
        trace!(?write_error);
        read(ring_buffer, read_ops, rng);
        write_errors += 1;
    }

    write_errors
}

fn write(
    ring_buffer: &mut RingBuffer,
    operation: &Operation,
    rng: &mut SmallRng,
) -> Result<()> {
    let mut tx = ring_buffer.write_tx();

    write_chunked(&mut tx, operation, rng)?;

    if operation.commit {
        tx.commit();
    }

    Ok(())
}

fn write_chunked(
    tx: &mut RingBufferWriteTx,
    operation: &Operation,
    rng: &mut SmallRng,
) -> Result<()> {
    let mut chunk_count = 0;
    let mut data = operation.data;

    loop {
        let old_capacity = tx.capacity();
        let (chunk, more_data) = data.split_at(rng.gen_range(0 ..= data.len()));

        tx.write(chunk)?;

        assert_eq!(tx.capacity() + chunk.len(), old_capacity);

        data = more_data;
        chunk_count += 1;
        if data.is_empty() {
            break;
        }
    }

    trace!(%operation, chunk_count, "write");

    Ok(())
}

#[derive(Clone)]
struct Generator {
    buffer: [u8; RingBuffer::REAL_SIZE],
    quality: Quality,
    rng: SmallRng,
}

impl Generator {
    fn new(
        quality: Quality,
        seed: u64,
    ) -> Self {
        Self {
            buffer: [0; RingBuffer::REAL_SIZE],
            quality,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    fn generate_operation(&mut self) -> Operation {
        Operation::new(&mut self.buffer, self.quality, &mut self.rng)
    }
}

struct Operation<'a> {
    commit: bool,
    data: &'a [u8],
    quality: Quality,
}

impl<'a> Operation<'a> {
    fn new(
        data: &'a mut [u8],
        quality: Quality,
        rng: &mut SmallRng,
    ) -> Self {
        Self {
            commit: rng.gen_ratio(3, 4),
            data: Self::fill(data, quality, rng),
            quality,
        }
    }

    fn fill<'b>(
        data: &'b mut [u8],
        quality: Quality,
        rng: &mut SmallRng,
    ) -> &'b [u8] {
        let len = Self::generate_len(rng);

        match quality {
            Quality::Debuggable => data[.. len].fill(rng.sample(Alphanumeric)),
            Quality::Paranoid => rng.fill(&mut data[.. len]),
        }

        &data[.. len]
    }

    fn generate_len(rng: &mut SmallRng) -> usize {
        const BOUNDARY_SIZE: usize = 16;

        if rng.gen_ratio(3, 4) {
            rng.gen_range(BOUNDARY_SIZE .. RingBuffer::REAL_SIZE - BOUNDARY_SIZE + 1)
        } else if rng.gen_ratio(1, 2) {
            rng.gen_range(0 .. BOUNDARY_SIZE)
        } else {
            rng.gen_range(RingBuffer::REAL_SIZE - BOUNDARY_SIZE + 1 .. RingBuffer::REAL_SIZE + 1)
        }
    }
}

impl fmt::Display for Operation<'_> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self.quality {
            Quality::Debuggable => write!(
                formatter,
                "{:?} x {}, {}",
                if self.data.is_empty() {
                    ' '
                } else {
                    self.data[0] as char
                },
                self.data.len(),
                if self.commit {
                    "commit"
                } else {
                    "rollback"
                },
            ),
            Quality::Paranoid => write!(
                formatter,
                "{:?}, {}",
                self.data,
                if self.commit {
                    "commit"
                } else {
                    "rollback"
                },
            ),
        }
    }
}

struct RunLengthEncoding<'a>(&'a [u8]);

impl fmt::Display for RunLengthEncoding<'_> {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        let mut block_count = 0;
        let mut total_len = 0;

        for block in self.0.group_by(|a, b| a == b) {
            write!(formatter, "{:?} x {}, ", block[0] as char, block.len())?;
            block_count += 1;
            total_len += block.len();
        }

        write!(
            formatter,
            "block_count = {block_count}, total_len = {total_len}",
        )
    }
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
enum Quality {
    Debuggable,
    Paranoid,
}
