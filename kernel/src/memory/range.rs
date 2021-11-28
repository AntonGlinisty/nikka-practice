use core::cmp;

use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

use crate::log::info;

use super::{
    block::Block,
    frage::Frame,
    size::Size,
};

pub(super) fn physical(memory_map: &MemoryMap) -> Block<Frame> {
    let mut total_frame_count = 0;
    let mut usable_total_frame_count = 0;

    for region in memory_map.iter() {
        let usable = region.region_type == MemoryRegionType::Usable;
        let start = region.range.start_frame_number;
        let end = region.range.end_frame_number;

        if usable {
            usable_total_frame_count += end - start;
            total_frame_count = cmp::max(total_frame_count, end);
        }
    }

    info!(
        total = %Size::new_u64::<Frame>(total_frame_count),
        usable = %Size::new_u64::<Frame>(usable_total_frame_count),
        total_frame_count,
        usable_total_frame_count,
        "available memory",
    );

    Block::from_index_u64(0, total_frame_count).expect("invalid memory map from bootloader")
}
