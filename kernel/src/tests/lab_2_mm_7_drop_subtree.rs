use kernel::{
    Subsystems,
    memory::{
        Virt,
        test_scaffolding::forbid_frame_leaks,
    },
};

mod init;
mod mm_helpers;

init!(Subsystems::PHYS_MEMORY | Subsystems::VIRT_MEMORY);

#[test_case]
fn drop_subtree() {
    {
        let _guard = forbid_frame_leaks();
        mm_helpers::map_intermediate(Virt::new(0xFFFF_F300_0000_0000).unwrap());
    }

    {
        let _guard = forbid_frame_leaks();
        mm_helpers::test_no_excessive_intermediate_flags(Virt::new(0xFFFF_F400_0000_0000).unwrap());
    }
}
