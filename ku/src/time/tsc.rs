use core::{
    arch::x86_64,
    convert::TryInto,
    fmt::{
        Debug,
        Display,
        Formatter,
        Result,
    },
    iter,
    mem,
};

use itertools::Itertools;
use num_traits::cast;
// FloatCore is used for a substitute of `f64::abs()`. `f64::abs()` is declared in `std` whilst this is a `#[no_std]` library.
// Moreover, `f64::abs()` from `std` uses FPU but this library is used in the kernel which is compiled with `+soft-float`.
// However, rustc fails to recognize that FloatCore is really used.
#[allow(unused_imports)]
use num_traits::float::FloatCore;
use number_prefix::NumberPrefix;
use serde::{
    Deserialize,
    Serialize,
};

use super::rtc::Rtc;

/// Modelled after [`std::time::Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html).
/// Note: `Tsc` is valid only on the current computer and between reboots.
/// Do not store it in a file or send it over a network.
/// `Serialize` and `Deserialize` traits are for sending `Tsc` between the kernel and the user space.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tsc(i64);
/// Modelled after [`std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html).
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct TscDuration(i64);

impl Tsc {
    #[inline(always)]
    pub fn now() -> Self {
        Self(tsc())
    }

    #[inline(always)]
    pub fn elapsed(&self) -> TscDuration {
        TscDuration(tsc() - self.0)
    }

    #[inline(always)]
    pub fn lap(&mut self) -> TscDuration {
        let lap_end = tsc();
        TscDuration(lap_end - mem::replace(&mut self.0, lap_end))
    }

    pub(super) fn get(&self) -> i64 {
        self.0
    }
}

impl TscDuration {
    fn into_f64(self) -> f64 {
        let tsc: u64 = self.0.try_into().expect("duration should not be negative");
        cast::cast(tsc).expect(U64_AS_F64_SHOULD_NOT_FAIL)
    }
}

impl Debug for TscDuration {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        match NumberPrefix::decimal(self.into_f64()) {
            NumberPrefix::Standalone(_) => {
                write!(formatter, "{} tsc", self.0)
            },
            NumberPrefix::Prefixed(prefix, value) => {
                write!(formatter, "{:.3} {}tsc", value, prefix.symbol())
            },
        }
    }
}

impl Display for TscDuration {
    fn fmt(
        &self,
        formatter: &mut Formatter,
    ) -> Result {
        if let Some(hz) = Rtc::tsc_per_second() {
            let hz: f64 = cast::cast(hz.get()).expect(U64_AS_F64_SHOULD_NOT_FAIL);
            let value = self.into_f64() / hz;

            match NumberPrefix::decimal(value) {
                NumberPrefix::Standalone(value) => {
                    let (value, prefix) = fractional_prefix(value);
                    write!(formatter, "{value:.3} {prefix}s")
                },
                NumberPrefix::Prefixed(prefix, value) => {
                    write!(formatter, "{:.3} {}s", value, prefix.symbol())
                },
            }
        } else {
            (self as &dyn Debug).fmt(formatter)
        }
    }
}

#[inline(always)]
pub fn tsc() -> i64 {
    // Do not use `x86::fence::lfence()` and `x86::time::rdtsc()` because they are not marked as `#[inline]`.
    unsafe {
        x86_64::_mm_lfence();
        x86_64::_rdtsc()
            .try_into()
            .expect("i64 overflow when storing TSC is expected only after tens of years of uptime")
    }
}

fn fractional_prefix(value: f64) -> (f64, &'static str) {
    // Alas, `number_prefix` does not support fractional prefixes.
    iter::successors(Some(value), |x| Some(x * 1000.0))
        .zip(["", "m", "u", "n"])
        .find_or_last(|x| x.0.abs() >= 1.0)
        .unwrap()
}

const U64_AS_F64_SHOULD_NOT_FAIL: &str =
    "a cast from u64 to f64 can loose precision but should not fail";

#[cfg(test)]
mod test {
    use super::fractional_prefix as fp;

    #[test]
    fn fractional_prefix() {
        assert!(check(1.0, 1.0, ""));
        assert!(check(100.0, 100.0, ""));
        assert!(check(1_000_000.0, 1_000_000.0, ""));

        assert!(check(0.1, 100.0, "m"));
        assert!(check(0.001, 1.0, "m"));

        assert!(check(0.000_1, 100.0, "u"));
        assert!(check(0.000_001, 1.0, "u"));

        assert!(check(0.000_000_1, 100.0, "n"));
        assert!(check(0.000_000_001, 1.0, "n"));

        assert!(check(0.000_000_000_1, 0.1, "n"));
        assert!(check(0.000_000_000_001, 0.001, "n"));

        fn check(
            argument: f64,
            result: f64,
            prefix: &str,
        ) -> bool {
            for sign in [1.0, -1.0] {
                if !equal(fp(sign * argument), (sign * result, prefix)) {
                    return false;
                }
            }
            true
        }

        fn equal(
            a: (f64, &str),
            b: (f64, &str),
        ) -> bool {
            (a.0 - b.0).abs() < 1e-6 && a.1 == b.1
        }
    }
}
