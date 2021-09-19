use bitflags::bitflags;

bitflags! {
    pub struct Syscall: usize {
        const EXIT = 0;
    }
}
