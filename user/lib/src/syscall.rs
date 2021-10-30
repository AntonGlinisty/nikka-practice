use ku::process::Syscall;

pub fn exit(code: usize) -> ! {
    syscall(Syscall::EXIT.bits(), code, 0, 0);

    unreachable!();
}

pub fn sched_yield() {
    syscall(Syscall::SCHED_YIELD.bits(), 0, 0, 0);
}

fn syscall(
    number: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
) {
    // TODO: your code here.
    unimplemented!();
}
