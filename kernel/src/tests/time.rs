use core::{
    cmp,
    sync::atomic::{
        AtomicI64,
        Ordering,
    },
};

use chrono::{
    DateTime,
    Duration,
    Utc,
};
use x86_64::registers::rflags;

use ku::time::{
    self,
    test_scaffolding::{
        AtomicCorrelationPoint,
        forge_tsc,
    },
};

use kernel::{
    log::debug,
    time::test_scaffolding::TICKS_PER_SECOND,
    trap::{
        self,
        PIT,
        RTC,
        TRAP_STATS,
        TrapContext,
    },
};

mod init;

init!();

#[test_case]
fn rtc_read_inconsistent() {
    let start = time::now();
    let rtc_ticks = TRAP_STATS[RTC].count();
    const TICKS: i64 = 10;

    debug!(%start);
    assert!(start > DateTime::parse_from_rfc3339("2022-09-10T11:50:17+00:00").unwrap());

    for ticks in 1 .. TICKS {
        let mut rtc_count = TRAP_STATS[RTC].count();

        while rtc_count < rtc_ticks + (ticks as usize) {
            x86_64::instructions::hlt();

            let new_rtc_count = TRAP_STATS[RTC].count();
            if new_rtc_count != rtc_count {
                debug!(rtc_count, new_rtc_count);
                assert_eq!(new_rtc_count, rtc_count + 1, "unexpected RTC tick");
            }
            rtc_count = new_rtc_count;
        }

        let now = time::now();
        let max_now = start + Duration::seconds(ticks + 1);
        let min_now = start + Duration::seconds(ticks - 1);

        debug!(%now);

        assert!(
            min_now <= now && now <= max_now,
            "the RTC date does not follow its ticks",
        );
    }
}

#[test_case]
fn points_beyond_interval() {
    wait_for_two_correlation_points();

    let a_few_seconds_in_tsc = 10_000_000_000;

    let before_boot = time::datetime(forge_tsc(-a_few_seconds_in_tsc));
    let boot = time::datetime(forge_tsc(0));
    let now = time::now();
    let future = time::datetime(forge_tsc(time::tsc() + a_few_seconds_in_tsc));

    debug!(%before_boot);
    debug!(%boot);
    debug!(%now);
    debug!(%future);

    check_difference(before_boot, boot);
    check_difference(boot, now);
    check_difference(now, future);

    fn check_difference(
        left: DateTime<Utc>,
        right: DateTime<Utc>,
    ) {
        let difference = right - left;

        debug!(%left, %right, %difference);

        assert!(left < right);
        assert!(difference > Duration::milliseconds(100));
        assert!(difference < Duration::seconds(30));
    }
}

#[test_case]
fn high_resolution_date() {
    wait_for_two_correlation_points();

    const PIT_TICKS: i64 = 102;
    const NANOSECONDS_PER_PIT_TICK: i64 = 1_000_000_000 / TICKS_PER_SECOND as i64;
    let mut prev = DateTime::default();
    let timer_ticks = TRAP_STATS[PIT].count();
    let tick_duration = Duration::nanoseconds(NANOSECONDS_PER_PIT_TICK);
    let max_delta = Duration::nanoseconds(NANOSECONDS_PER_PIT_TICK * 13 / 10);
    let min_delta = Duration::nanoseconds(NANOSECONDS_PER_PIT_TICK * 7 / 10);

    let mut samples = 0;
    let mut max = Duration::min_value();
    let mut min = Duration::max_value();
    let mut sum = Duration::zero();
    let mut sum2 = 0;

    debug!("measuring");

    for ticks in 1 .. PIT_TICKS {
        while TRAP_STATS[PIT].count() < timer_ticks + ticks as usize {
            x86_64::instructions::hlt();
        }

        let now = time::now();

        if ticks > 1 {
            let delta = now - prev;

            if delta < min_delta || max_delta < delta {
                debug!(%ticks, %delta, %prev, %now);
            }

            samples += 1;
            max = cmp::max(max, delta);
            min = cmp::min(min, delta);
            sum = sum + delta;
            let error = (delta - tick_duration).num_nanoseconds().unwrap();
            sum2 += error * error;
        }

        prev = now;
    }

    let max_mean = Duration::nanoseconds(NANOSECONDS_PER_PIT_TICK * 101 / 100);
    let min_mean = Duration::nanoseconds(NANOSECONDS_PER_PIT_TICK * 99 / 100);
    let max_deviation = Duration::nanoseconds(5_000_000);
    let min_deviation = Duration::nanoseconds(1_000);

    let mean = sum / samples;
    let deviation = Duration::nanoseconds(num_integer::sqrt(sum2 / i64::from(samples)));

    debug!(samples, %min, %max, %mean, %deviation, "measured");
    debug!(%min_delta, %max_delta, %min_mean, %max_mean, %max_deviation, "restrictions");

    assert!(min_delta <= min && max <= max_delta);
    assert!(min_mean <= mean && mean <= max_mean);
    assert!(min_deviation <= deviation && deviation <= max_deviation);
}

#[test_case]
fn correlation_point() {
    trap::test_scaffolding::set_debug_handler(debug);

    const TRAP_FLAG: u64 = 1 << 8;
    let flags = rflags::read_raw();
    unsafe { rflags::write_raw(flags | TRAP_FLAG) };

    for iteration in 0 .. {
        let point = POINT.load();
        debug!(iteration, ?point);
        assert_eq!(point.count(), point.tsc());
        if point.count() > 1_000_000 && iteration > 20 {
            break;
        }
    }

    unsafe { rflags::write_raw(flags | TRAP_FLAG) };

    static POINT: AtomicCorrelationPoint = AtomicCorrelationPoint::new();

    extern "x86-interrupt" fn debug(_: TrapContext) {
        static VALUE: AtomicI64 = AtomicI64::new(0);

        let value = VALUE.fetch_add(1, Ordering::Relaxed);

        match (value / 1_000) % 4 {
            0 => POINT.store(time::test_scaffolding::equal_point(value + 1)),
            2 => POINT.inc(POINT.load().count() + 1),
            _ => {},
        }
    }
}

fn wait_for_two_correlation_points() {
    debug!("waiting for the RTC to tick twice");

    let rtc_ticks = TRAP_STATS[RTC].count();
    while TRAP_STATS[RTC].count() < rtc_ticks + 2 {
        x86_64::instructions::hlt();
    }
}
