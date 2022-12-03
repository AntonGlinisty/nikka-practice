use core::{
    arch::asm,
    hint,
    mem,
    ops::Range,
};

use bitflags::bitflags;
use chrono::Duration;
use derive_more::Display;
use static_assertions::const_assert;
use x86::io;

use ku::{
    error::{
        Error::{
            Medium,
            Timeout,
        },
        Result,
    },
    log::{
        error,
        trace,
    },
    time,
};

#[derive(Clone, Copy, Debug, Display)]
#[display(
    fmt = "{{ id: {}, base_port: {:#04X}, disk: {} }}",
    id,
    base_port,
    disk
)]
pub(super) struct Disk {
    base_port: u16,
    disk: u8,
    id: u8,
}

impl Disk {
    fn new(
        base_port: u16,
        disk: u8,
        id: u8,
    ) -> Self {
        Self {
            base_port,
            disk,
            id,
        }
    }

    pub(super) fn list() -> [Self; 4] {
        const ATA0_BASE_PORT: u16 = 0x01F0;
        const ATA1_BASE_PORT: u16 = 0x0170;

        [
            Self::new(ATA0_BASE_PORT, 0, 0),
            Self::new(ATA0_BASE_PORT, 1, 1),
            Self::new(ATA1_BASE_PORT, 0, 2),
            Self::new(ATA1_BASE_PORT, 1, 3),
        ]
    }

    pub(super) fn pio_read(
        &self,
        sectors: Range<usize>,
        buffer: &mut [u32],
    ) -> Result<()> {
        assert_eq!(mem::size_of_val(buffer), sectors.len() * SECTOR_SIZE);

        unsafe {
            self.send_command(Command::READ, sectors.start, sectors.len())?;
        }

        for sector in buffer.chunks_mut(SECTOR_SIZE / mem::size_of::<u32>()) {
            self.wait_ready()?;

            unsafe {
                ins32(self.base_port, sector);
            }
        }

        Ok(())
    }

    pub(super) fn pio_write(
        &self,
        sectors: Range<usize>,
        buffer: &[u32],
    ) -> Result<()> {
        assert_eq!(mem::size_of_val(buffer), sectors.len() * SECTOR_SIZE);

        unsafe {
            self.send_command(Command::WRITE, sectors.start, sectors.len())?;
        }

        for sector in buffer.chunks(SECTOR_SIZE / mem::size_of::<u32>()) {
            self.wait_ready()?;

            unsafe {
                outs32(self.base_port, sector);
            }
        }

        Ok(())
    }

    unsafe fn send_command(
        &self,
        command: Command,
        start_sector: usize,
        sector_count: usize,
    ) -> Result<()> {
        const LBA: u8 = 1 << 6;
        const RESERVED_SHOULD_BE_ONES: u8 = (1 << 5) | (1 << 7);

        assert!(start_sector < (1 << 24));
        assert!(sector_count < (1 << 8));

        self.wait_ready()?;

        io::outb(self.base_port + 2, sector_count as u8);
        io::outb(self.base_port + 3, start_sector as u8);
        io::outb(self.base_port + 4, (start_sector >> 8) as u8);
        io::outb(self.base_port + 5, (start_sector >> 16) as u8);
        io::outb(
            self.base_port + 6,
            RESERVED_SHOULD_BE_ONES | LBA | (self.disk << 4) | ((start_sector >> 24) & 0xF) as u8,
        );
        io::outb(self.base_port + 7, command.bits());

        Ok(())
    }

    fn wait_ready(&self) -> Result<()> {
        bitflags! {
            struct Status: u8 {
                const ERROR = 1 << 0;
                const FAILURE = 1 << 5;
                const READY = 1 << 6;
                const BUSY = 1 << 7;
            }
        }

        let mut iterations = 1;
        let mut last_status = None;
        let start = time::timer();
        let timeout = Duration::seconds(TIMEOUT_IN_SECONDS);

        while last_status.is_none() || !start.has_passed(timeout) {
            let status = Status::from_bits_truncate(unsafe { io::inb(self.base_port + 7) });
            last_status = Some(status);

            if status.contains(Status::READY) && !status.contains(Status::BUSY) {
                trace!(elapsed = %start.elapsed(), iterations, ?status, "waited for IDE");

                return if status.intersects(Status::ERROR | Status::FAILURE) {
                    Err(Medium)
                } else {
                    Ok(())
                };
            }

            hint::spin_loop();
            iterations += 1;
        }

        error!(elapsed = %start.elapsed(), iterations, ?last_status, "timeout waiting for IDE");

        Err(Timeout)
    }
}

unsafe fn outs32(
    port: u16,
    buffer: &[u32],
) {
    asm!(
        "
        rep outs dx, LONG PTR [rsi]
        ",
        in("rdx") port,
        in("rcx") buffer.len(),
        in("rsi") buffer.as_ptr(),
        lateout("rcx") _,
        options(nostack),
    );
}

unsafe fn ins32(
    port: u16,
    buffer: &mut [u32],
) {
    asm!(
        "
        rep ins LONG PTR [rdi], dx
        ",
        in("rdx") port,
        in("rcx") buffer.len(),
        in("rdi") buffer.as_mut_ptr(),
        lateout("rcx") _,
        options(nostack),
    );
}

pub(super) const SECTOR_SIZE: usize = 1 << 9;

const TIMEOUT_IN_SECONDS: i64 = 10;

const_assert!(SECTOR_SIZE % mem::size_of::<u32>() == 0);

bitflags! {
    struct Command: u8 {
        const READ = 0x20;
        const WRITE = 0x30;
    }
}
