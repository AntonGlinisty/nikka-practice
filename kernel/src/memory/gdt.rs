use core::{
    fmt,
    mem,
};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    PrivilegeLevel,
    instructions::{
        segmentation::{
            CS,
            DS,
            ES,
            SS,
            Segment,
        },
        tables,
    },
    structures::{
        DescriptorTablePointer,
        gdt::{
            DescriptorFlags,
            SegmentSelector,
        },
        tss::TaskStateSegment,
    },
};

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::info,
    smp::CpuId,
};

use super::{
    Phys,
    Virt,
    size,
    tss::TSS,
};

pub(crate) fn init() {
    GDT.lock().load();

    unsafe {
        tables::load_tss(Gdt::tss(0));
    }

    info!("GDT init");
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub(crate) struct RealModePseudoDescriptor {
    limit: u16,
    base: u32,
}

impl RealModePseudoDescriptor {
    fn new<T>(address: Phys) -> Result<Self> {
        // 80286 limits physical addresses at 16 MiB.
        const MAX_REAL_MODE_BASE: u32 = 16 << 20;

        let base = address.try_into_u32()?;

        if base < MAX_REAL_MODE_BASE {
            Ok(Self {
                limit: (mem::size_of::<T>() - 1).try_into()?,
                base,
            })
        } else {
            Err(InvalidArgument)
        }
    }
}

impl fmt::Debug for RealModePseudoDescriptor {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            formatter,
            "{{ base: 0p{:06X}, limit: 0x{:04X} }}",
            { self.base },
            { self.limit }
        )
    }
}

#[repr(C, packed)]
pub(crate) struct SmpGdt<const TSS_COUNT: usize> {
    basic: [DescriptorFlags; BASIC_COUNT],
    tss: [TssEntry; TSS_COUNT],
}

impl<const TSS_COUNT: usize> SmpGdt<TSS_COUNT> {
    pub(crate) fn new() -> Self {
        let mut gdt = Self {
            basic: [DescriptorFlags::empty(); BASIC_COUNT],
            tss: [TssEntry::default(); TSS_COUNT],
        };

        for descriptor in [
            DescriptorFlags::KERNEL_CODE64,
            DescriptorFlags::KERNEL_DATA,
            DescriptorFlags::USER_CODE64,
            DescriptorFlags::USER_DATA,
        ] {
            gdt.basic[size::from(Self::basic_selector(descriptor).index())] = descriptor;
        }

        gdt
    }

    pub(crate) fn real_mode_pseudo_descriptor(address: Phys) -> Result<RealModePseudoDescriptor> {
        RealModePseudoDescriptor::new::<Self>(address)
    }

    pub(crate) fn set_tss(
        &mut self,
        cpu: CpuId,
        tss: &TaskStateSegment,
    ) {
        let cpu = size::from(cpu);
        assert!(cpu < TSS_COUNT);

        self.tss[cpu] = TssEntry::new(tss);
    }

    pub(crate) fn tss(cpu: CpuId) -> SegmentSelector {
        const TSS_DESCRIPTOR_WIDTH: usize =
            mem::size_of::<TssEntry>() / mem::size_of::<DescriptorFlags>();

        let cpu = size::from(cpu);
        assert!(cpu < TSS_COUNT);

        SegmentSelector::new(
            (BASIC_COUNT + cpu * TSS_DESCRIPTOR_WIDTH).try_into().unwrap(),
            PrivilegeLevel::Ring0,
        )
    }

    pub(crate) fn kernel_code() -> SegmentSelector {
        Self::basic_selector(DescriptorFlags::KERNEL_CODE64)
    }

    pub(crate) fn kernel_data() -> SegmentSelector {
        Self::basic_selector(DescriptorFlags::KERNEL_DATA)
    }

    pub(crate) fn user_code() -> SegmentSelector {
        Self::basic_selector(DescriptorFlags::USER_CODE64)
    }

    pub(crate) fn user_data() -> SegmentSelector {
        Self::basic_selector(DescriptorFlags::USER_DATA)
    }

    pub(crate) fn load(&self) {
        let pseudo_descriptor = DescriptorTablePointer {
            limit: (mem::size_of::<Self>() - 1).try_into().expect("too large GDT"),
            base: Virt::from_ref(self).into(),
        };

        unsafe {
            tables::lgdt(&pseudo_descriptor);

            CS::set_reg(Self::kernel_code());

            DS::set_reg(Self::kernel_data());
            ES::set_reg(Self::kernel_data());
            SS::set_reg(Self::kernel_data());
        }
    }

    fn basic_selector(flags: DescriptorFlags) -> SegmentSelector {
        let is_executable_equal_user = flags.contains(DescriptorFlags::EXECUTABLE) ==
            flags.contains(DescriptorFlags::DPL_RING_3);
        let index: u16 =
            // For the mandatory null descriptor.
            1 +
            if flags.contains(DescriptorFlags::DPL_RING_3) { 2 } else { 0 } +
            // This strange order is for the Star register.
            if is_executable_equal_user { 1 } else { 0 };

        assert!(size::from(index) < BASIC_COUNT);

        SegmentSelector::new(
            index,
            if flags.contains(DescriptorFlags::DPL_RING_3) {
                PrivilegeLevel::Ring3
            } else {
                PrivilegeLevel::Ring0
            },
        )
    }
}

pub(crate) type Gdt = SmpGdt<MAX_CPUS>;
pub(crate) type SmallGdt = SmpGdt<0>;

const BASIC_COUNT: usize = 5;
const MAX_CPUS: usize = CpuId::MAX as usize + 1;

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(C, packed)]
struct TssEntry(DescriptorFlags, DescriptorFlags);

impl TssEntry {
    fn new(tss: &TaskStateSegment) -> Self {
        const AVAILABLE_TSS: u64 = 0b1001 << 40;

        let base = Virt::from_ref(tss).into_u64();
        let limit = size::into_u64(mem::size_of::<TaskStateSegment>() - 1);

        Self(
            DescriptorFlags::from_bits(
                limit |
                    ((base & ((1 << 24) - 1)) << 16) |
                    AVAILABLE_TSS |
                    DescriptorFlags::PRESENT.bits() |
                    ((base >> 24) << 56),
            )
            .unwrap(),
            DescriptorFlags::from_bits(base >> 32).unwrap(),
        )
    }
}

impl Default for TssEntry {
    fn default() -> Self {
        Self(DescriptorFlags::empty(), DescriptorFlags::empty())
    }
}

lazy_static! {
    pub(crate) static ref GDT: Mutex<Gdt> = Mutex::new({
        let mut gdt = Gdt::new();

        gdt.set_tss(0, &TSS);

        gdt
    });
}
