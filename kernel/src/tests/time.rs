use core::{
    cmp,
    sync::atomic::{
        AtomicI64,
        AtomicUsize,
        Ordering,
    },
};

use chrono::{
    DateTime,
    Duration,
    Utc,
};
use x86_64::{
    instructions,
    registers::rflags::{
        self,
        RFlags,
    },
};

use ku::{
    memory::{
        Block,
        Virt,
    },
    time::{
        self,
        test_scaffolding::{
            AtomicCorrelationPoint,
            forge_tsc,
        },
    },
};

use kernel::{
    Subsystems,
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

init!(Subsystems::empty());

#[test_case]
fn rtc_read_inconsistent() {
    debug!("waiting for the RTC to tick at least once");
    while TRAP_STATS[RTC].count() == 0 {
        instructions::hlt();
    }

    let start = time::now();
    let rtc_ticks = TRAP_STATS[RTC].count();
    const TICKS: i64 = 10;

    debug!(%start);
    assert!(
        start > DateTime::parse_from_rfc3339("2022-09-10T11:50:17+00:00").unwrap(),
        "the RTC date does not pass the sanity check",
    );

    for ticks in 1 .. TICKS {
        let mut rtc_count = TRAP_STATS[RTC].count();

        while rtc_count < rtc_ticks + (ticks as usize) {
            instructions::hlt();

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

    debug!(timer_ticks = PIT_TICKS, "measuring");

    for ticks in 1 .. PIT_TICKS {
        while TRAP_STATS[PIT].count() < timer_ticks + ticks as usize {
            instructions::hlt();
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
    let max_deviation = Duration::milliseconds(5);
    let min_deviation = Duration::nanoseconds(100);

    let mean = sum / samples;
    let deviation = Duration::nanoseconds(num_integer::sqrt(sum2 / i64::from(samples)));

    debug!(samples, %min, %max, %mean, %deviation, "measured");
    debug!(
        %min_delta,
        %max_delta,
        %min_mean,
        %max_mean,
        %min_deviation,
        %max_deviation,
        "restrictions",
    );

    assert!(
        min_delta <= min && max <= max_delta,
        "one of the measured durations between PIT ticks is not accurate",
    );
    assert!(
        min_mean <= mean && mean <= max_mean,
        "measured mean PIT frequency is not accurate",
    );
    assert!(
        min_deviation <= deviation && deviation <= max_deviation,
        "measured deviation of the PIT frequency is too big",
    );
}

#[test_case]
fn correlation_point_reader() {
    trap::test_scaffolding::set_debug_handler(writer);

    reader();

    static POINT: AtomicCorrelationPoint = AtomicCorrelationPoint::new();

    fn reader() {
        let mut prev = POINT.load();
        let mut different = 0;
        let mut same = 0;
        let rtc_ticks = AtomicUsize::new(0);

        const ITERATIONS: usize = 10_000;

        while same < ITERATIONS || different < ITERATIONS {
            switch_trap_flag();
            let point = POINT.load();
            switch_trap_flag();

            if prev == point {
                same += 1;
            } else {
                different += 1;
            }

            if sample(&rtc_ticks) {
                debug!(same, different, ?point);
            }

            assert_eq!(2 * point.count(), point.tsc(), "{point:?} is inconsistent");

            prev = point;
        }
    }

    extern "x86-interrupt" fn writer(_: TrapContext) {
        static VALUE: AtomicI64 = AtomicI64::new(0);

        let mut value = VALUE.fetch_add(1, Ordering::Relaxed);
        if value % 37 == 17 {
            value -= 1;
        }

        match (value / 1_000) % 4 {
            0 => POINT.store(time::test_scaffolding::new_point(value + 1, 2 * value + 2)),
            2 => POINT.inc(2 * POINT.load().count() + 2),
            _ => {},
        }
    }
}

#[test_case]
fn correlation_point_writer() {
    trap::test_scaffolding::set_debug_handler(reader);

    writer();

    const LAST_ITERATION: i64 = 10_000;

    static ITERATION: AtomicI64 = AtomicI64::new(0);
    static POINT: AtomicCorrelationPoint = AtomicCorrelationPoint::new();

    fn writer() {
        switch_trap_flag();

        for iteration in 0 .. LAST_ITERATION / 2 {
            let value = iteration -
                if iteration % 37 == 17 {
                    1
                } else {
                    0
                };
            POINT.store(time::test_scaffolding::new_point(value, 2 * value));
            ITERATION.store(iteration, Ordering::Relaxed);
        }

        for iteration in LAST_ITERATION / 2 ..= LAST_ITERATION {
            POINT.inc(2 * POINT.load().count() + 2);
            ITERATION.store(iteration, Ordering::Relaxed);
        }

        switch_trap_flag();
    }

    extern "x86-interrupt" fn reader(_: TrapContext) {
        static FAILURE_COUNT: AtomicI64 = AtomicI64::new(0);
        static SUCCESS_COUNT: AtomicI64 = AtomicI64::new(0);
        static RTC_TICKS: AtomicUsize = AtomicUsize::new(0);

        let iteration = ITERATION.load(Ordering::Relaxed);
        let failure_count = FAILURE_COUNT.load(Ordering::Relaxed);
        let success_count = SUCCESS_COUNT.load(Ordering::Relaxed);

        if let Some(point) = time::test_scaffolding::try_load(&POINT) {
            if sample(&RTC_TICKS) {
                debug!(iteration, failure_count, success_count, ?point);
                ITERATION.fetch_add(1, Ordering::Relaxed);
            }
            assert_eq!(2 * point.count(), point.tsc(), "{point:?} is inconsistent");
            SUCCESS_COUNT.fetch_add(1, Ordering::Relaxed);
        } else {
            FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
        }

        if iteration == LAST_ITERATION {
            debug!(iteration, failure_count, success_count);
            assert!(
                success_count >= LAST_ITERATION && failure_count >= LAST_ITERATION,
                "the test is too weak",
            );
            ITERATION.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[allow(dead_code)]
// #[test_case] // Uncomment it if you wish.
fn down_the_rabbit_hole() {
    trap::test_scaffolding::set_debug_handler(collect_statistics);

    wait_for_two_correlation_points();

    let mut timer = ku::timer();
    debug!(rflags = ?rflags::read(), "how many instructions does it take to log something?");
    let time_to_log_a_message = timer.lap();

    switch_trap_flag();
    debug!(rflags = ?rflags::read(), "how many instructions does it take to log something?");
    switch_trap_flag();
    let time_to_log_a_message_in_the_stepping_mode = timer.elapsed();

    let max_rsp = MAX_RSP.load(Ordering::Relaxed);
    let min_rsp = cmp::min(max_rsp, MIN_RSP.load(Ordering::Relaxed));
    let rip = RIP.load(Ordering::Relaxed);

    debug!(
        instruction_count = INSTRUCTION_COUNT.load(Ordering::Relaxed),
        used_stack_space = ?Block::<Virt>::from_index(min_rsp, max_rsp).unwrap(),
        last_traced_instruction_address = %Virt::new(rip).unwrap(),
    );

    let stepping_slowdown_ratio =
        time_to_log_a_message_in_the_stepping_mode.into_f64() / time_to_log_a_message.into_f64();
    debug!(
        %time_to_log_a_message,
        %time_to_log_a_message_in_the_stepping_mode,
        stepping_slowdown_ratio,
    );

    static INSTRUCTION_COUNT: AtomicUsize = AtomicUsize::new(0);
    static MAX_RSP: AtomicUsize = AtomicUsize::new(usize::MIN);
    static MIN_RSP: AtomicUsize = AtomicUsize::new(usize::MAX);
    static RIP: AtomicUsize = AtomicUsize::new(0);

    extern "x86-interrupt" fn collect_statistics(context: TrapContext) {
        let context = context.get().mini_context();
        let rip = context.rip().into_usize();
        let rsp = context.rsp().into_usize();

        let max_rsp = cmp::max(rsp, MAX_RSP.load(Ordering::Relaxed));
        let min_rsp = cmp::min(rsp, MIN_RSP.load(Ordering::Relaxed));

        INSTRUCTION_COUNT.fetch_add(1, Ordering::Relaxed);
        MAX_RSP.store(max_rsp, Ordering::Relaxed);
        MIN_RSP.store(min_rsp, Ordering::Relaxed);
        RIP.store(rip, Ordering::Relaxed);
    }
}

fn sample(ticks: &AtomicUsize) -> bool {
    let prev_rtc_ticks = ticks.load(Ordering::Relaxed);
    let rtc_ticks = TRAP_STATS[RTC].count();
    if prev_rtc_ticks != rtc_ticks {
        ticks.store(rtc_ticks, Ordering::Relaxed);
        true
    } else {
        false
    }
}

fn switch_trap_flag() {
    unsafe {
        rflags::write(rflags::read() ^ RFlags::TRAP_FLAG);
    }
}

fn wait_for_two_correlation_points() {
    debug!("waiting for the RTC to tick twice");

    let rtc_ticks = TRAP_STATS[RTC].count();
    while TRAP_STATS[RTC].count() < rtc_ticks + 2 {
        instructions::hlt();
    }
}
