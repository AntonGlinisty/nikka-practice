use core::{
    convert::TryInto,
    fmt,
    mem,
};

use lazy_static::lazy_static;
use x86_64::{
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
    structures::gdt::{
        Descriptor,
        GlobalDescriptorTable,
        SegmentSelector,
    },
};

use crate::{
    error::{
        Error::InvalidArgument,
        Result,
    },
    log::info,
};

use super::{
    Phys,
    tss::TSS,
};

pub(crate) fn init() {
    GDT.gdt.load();

    unsafe {
        CS::set_reg(GDT.kernel_code_selector);

        DS::set_reg(GDT.kernel_data_selector);
        ES::set_reg(GDT.kernel_data_selector);
        SS::set_reg(GDT.kernel_data_selector);

        tables::load_tss(GDT.tss_selector);
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

pub(crate) struct Gdt {
    gdt: GlobalDescriptorTable,
    tss_selector: SegmentSelector,
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

impl Gdt {
    pub(crate) fn new() -> Self {
        let mut gdt = GlobalDescriptorTable::new();

        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

        Self {
            gdt,
            tss_selector,
            kernel_code_selector,
            kernel_data_selector,
        }
    }

    pub(crate) fn real_mode_pseudo_descriptor(address: Phys) -> Result<RealModePseudoDescriptor> {
        RealModePseudoDescriptor::new::<Self>(address)
    }

    pub(crate) fn kernel_code(&self) -> SegmentSelector {
        self.kernel_code_selector
    }

    pub(crate) fn kernel_data(&self) -> SegmentSelector {
        self.kernel_data_selector
    }
}

lazy_static! {
    pub(crate) static ref GDT: Gdt = Gdt::new();
}
