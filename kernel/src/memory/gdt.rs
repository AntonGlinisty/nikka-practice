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

use crate::log::info;

use super::tss::TSS;

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

struct Gdt {
    gdt: GlobalDescriptorTable,
    tss_selector: SegmentSelector,
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

impl Gdt {
    fn new() -> Self {
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
}

lazy_static! {
    static ref GDT: Gdt = Gdt::new();
}
