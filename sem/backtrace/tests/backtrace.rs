#![allow(named_asm_labels)]
#![deny(warnings)]

use std::arch::asm;

use sentinel_frame::with_sentinel_frame;

use backtrace::{
    Backtrace,
    backtrace,
};

macro_rules! call {
    ($function:ident, $return_address:ident, $arg1:expr) => {
        call!($function, $return_address, $arg1, 0);
    };
    ($function:ident, $return_address:ident, $arg1:expr, $arg2: expr) => {
        unsafe {
            asm!(
                concat!("call {", stringify!($function), "}\n", stringify!($return_address), ":"),
                in("rdi") $arg1,
                in("rsi") $arg2,
                $function = sym $function,
                clobber_abi("C"),
            );
        }
    }
}

macro_rules! check_return_address {
    ($ptr:expr, $return_address:ident) => {
        let ptr = $ptr as *const ();
        let return_address: usize;
        unsafe {
            asm!(
                concat!("mov {return_address}, OFFSET ", stringify!($return_address)),
                return_address = out(reg) return_address,
            );
        }
        eprintln!(
            "checking {} == {}: {:#X} == {:#X}",
            stringify!($ptr),
            stringify!($return_address),
            ptr.addr(),
            return_address,
        );
        assert_eq!(ptr.addr(), return_address);
    };
}

struct Bt(Backtrace, Backtrace);

#[with_sentinel_frame]
fn r(
    recursion: usize,
    bt: &mut Bt,
) {
    let bt = (bt as *mut Bt) as usize;
    call!(r0, r_return_after_r0, recursion, bt);
    call!(r1, r_return_after_r1, recursion, bt);
}

extern "C" fn r0(
    recursion: usize,
    bt: usize,
) {
    if recursion > 1 {
        call!(r0, r0_return, recursion - 1, bt);
    } else {
        let bt = unsafe { &mut *(bt as *mut Bt) };
        bt.0 = backtrace();
    }
}

extern "C" fn r1(
    recursion: usize,
    bt: usize,
) {
    if recursion > 1 {
        call!(r1, r1_return, recursion - 1, bt);
    } else {
        let bt = unsafe { &mut *(bt as *mut Bt) };
        bt.1 = backtrace();
    }
}

extern "C" fn s4(bt: usize) {
    let bt = unsafe { &mut *(bt as *mut Backtrace) };
    *bt = backtrace();
}

extern "C" fn s3(bt: usize) {
    call!(s4, s3_return, bt);
}

extern "C" fn s2(bt: usize) {
    call!(s3, s2_return, bt);
}

#[with_sentinel_frame]
fn s1(bt: &mut Backtrace) {
    let bt = (bt as *mut Backtrace) as usize;
    call!(s2, s1_return, bt);
}

#[test]
fn simple() {
    let mut bt = Backtrace::new();

    s1(&mut bt);

    check_return_address!(bt[1], s3_return);
    check_return_address!(bt[2], s2_return);
    check_return_address!(bt[3], s1_return);
}

#[test]
fn recursive() {
    let mut bt = Bt(Backtrace::new(), Backtrace::new());
    let mut min_len = 1;

    for recursion in (1 .. 20).step_by(5) {
        r(recursion, &mut bt);

        if recursion == 1 {
            min_len = bt.0.len();
        }

        assert_eq!(bt.0.len(), min_len + recursion - 1);
        assert_eq!(bt.1.len(), min_len + recursion - 1);

        for i in 1 .. recursion {
            check_return_address!(bt.0[i], r0_return);
        }
        check_return_address!(bt.0[recursion], r_return_after_r0);

        for i in 1 .. recursion {
            check_return_address!(bt.1[i], r1_return);
        }
        check_return_address!(bt.1[recursion], r_return_after_r1);
    }
}
