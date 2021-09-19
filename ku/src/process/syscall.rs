use bitflags::bitflags;

bitflags! {
    pub struct ExitCode: usize {
        const OK = 0;
        const PANIC = 1;
        const UNIMPLEMENTED_SYSCALL = 2;
    }
}

bitflags! {
    pub struct Syscall: usize {
        const EXIT = 0;
    }
}
