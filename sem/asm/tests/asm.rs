#![deny(warnings)]

use core::arch::asm;

use rand::{
    Rng,
    SeedableRng,
    distributions::Uniform,
    rngs::SmallRng,
};

use asm::{
    Struct,
    sum,
    sum_array,
    sum_struct,
    sum_va_arg,
};

#[test]
fn test_sum() {
    check_signature(sum);

    assert_eq!(dressed_sum(1, 2), 3);
    assert_eq!(dressed_sum(10, 20), 30);
    assert_eq!(dressed_sum(i64::MIN, i64::MAX), -1);

    for x in -100 .. 100 {
        for y in -100 .. 100 {
            assert_eq!(dressed_sum(x, y), x + y);
        }
    }

    fn dressed_sum(
        x: i64,
        y: i64,
    ) -> i64 {
        let result;

        unsafe {
            asm!(
                "
                call sum
                ",
                in("rdi") x,
                in("rsi") y,
                lateout("rax") result,
            );
        }

        result
    }
}

#[test]
fn test_sum_struct() {
    check_signature(sum_struct);

    assert_eq!(dressed_sum(1, 2), 3);
    assert_eq!(dressed_sum(10, 20), 30);
    assert_eq!(dressed_sum(i64::MIN, i64::MAX), -1);

    for x in -100 .. 100 {
        for y in -100 .. 100 {
            assert_eq!(dressed_sum(x, y), x + y);
        }
    }

    fn dressed_sum(
        x: i64,
        y: i64,
    ) -> i64 {
        let mut s = Struct { x, y, result: 0 };

        unsafe {
            asm!(
                "
                call sum_struct
                ",
                in("rdi") &mut s,
            );
        }

        s.result
    }
}

#[test]
fn test_sum_array() {
    check_signature(sum_array);

    assert_eq!(dressed_sum(&[1, 2]), 3);
    assert_eq!(dressed_sum(&[10, 20]), 30);
    assert_eq!(dressed_sum(&[i64::MIN, i64::MAX]), -1);

    for x in -100 .. 100 {
        for y in -100 .. 100 {
            for z in -100 .. 100 {
                assert_eq!(dressed_sum(&[x, y, z]), x + y + z);
            }
        }
    }

    let mut rng = SmallRng::seed_from_u64(SEED);
    let range = Uniform::new(-100, 100);

    for _iteration in 0 .. 1000 {
        for size in 0 .. 100 {
            let x: Vec<_> = (0 .. size).map(|_| rng.sample(range)).collect();
            assert_eq!(dressed_sum(&x), x.iter().sum());
        }
    }

    fn dressed_sum(x: &[i64]) -> i64 {
        let result;

        unsafe {
            asm!(
                "
                call sum_array
                ",
                in("rdi") x.as_ptr(),
                in("rsi") x.len(),
                lateout("rax") result,
            );
        }

        result
    }
}

#[test]
fn test_sum_va_arg() {
    check_signature(sum_va_arg);

    assert_eq!(dressed_sum(&[1, 2]), 3);
    assert_eq!(dressed_sum(&[10, 20]), 30);
    assert_eq!(dressed_sum(&[i64::MIN, i64::MAX]), -1);

    for x in -100 .. 100 {
        for y in -100 .. 100 {
            for z in -100 .. 100 {
                assert_eq!(dressed_sum(&[x, y, z]), x + y + z);
            }
        }
    }

    let mut rng = SmallRng::seed_from_u64(SEED);
    let range = Uniform::new(-100, 100);

    for _iteration in 0 .. 1000 {
        for size in 0 .. 100 {
            let x: Vec<_> = (0 .. size).map(|_| rng.sample(range)).collect();
            assert_eq!(dressed_sum(&x), x.iter().sum());
        }
    }

    /// Переключает стек внутрь `x`.
    /// Регистр `R15` зарезервирован под сохранённый `RSP`.
    #[allow(named_asm_labels)]
    fn dressed_sum(x: &[i64]) -> i64 {
        let result;

        unsafe {
            asm!(
                "
                mov r15, rsp

            push_loop_begin:
                or rcx, rcx
                jz push_loop_end
                dec rcx
                push [rsi + 8 * rcx]
                jmp push_loop_begin
            push_loop_end:

                xor rsi, rsi

                call sum_va_arg

                mov rsp, r15
                ",
                in("rcx") x.len(),
                in("rsi") x.as_ptr(),
                in("rdi") x.len(),
                lateout("rax") result,
            );
        }

        result
    }
}

fn check_signature(_function: extern "C" fn() -> !) {
}

const SEED: u64 = 314159265;
