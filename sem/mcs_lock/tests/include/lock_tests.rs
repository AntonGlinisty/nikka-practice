#[rstest]
#[timeout(Duration::from_millis(500))]
fn basic() {
    let lock = Lock::new(0);

    let mut node = LockNode::new();
    lock.section(|v| *v += 1);
    *lock.lock(&mut node) += 1;
    lock.section(|v| *v += 1);
    *lock.lock(&mut LockNode::new()) += 1;
    assert_eq!(lock.section(|v| *v), 4);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn simple_multithreaded() {
    let max_threads = thread::available_parallelism().unwrap().get();
    let num_threads = if cfg!(miri) {
        3
    } else {
        6
    }
    .min(max_threads);
    const NUM_ITERATIONS: usize = if cfg!(miri) {
        100
    } else {
        100_000
    };

    let lock = Lock::new(0);

    thread::scope(|s| {
        for _ in 0 .. num_threads {
            s.spawn(|| {
                let mut node = LockNode::new();
                for _ in 0 .. NUM_ITERATIONS {
                    *lock.lock(&mut node) += 1;
                }
            });
        }
    });

    assert_eq!(lock.section(|v| *v), num_threads * NUM_ITERATIONS);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn long_sections() {
    let max_threads = thread::available_parallelism().unwrap().get();
    let num_threads = 3.min(max_threads);
    const NUM_ITERATIONS: usize = if cfg!(miri) {
        100
    } else {
        1000
    };

    let lock = Lock::new(0);

    thread::scope(|s| {
        for _ in 0 .. num_threads {
            s.spawn(|| {
                let mut node = LockNode::new();
                for _ in 0 .. NUM_ITERATIONS {
                    let mut guard = lock.lock(&mut node);
                    std::thread::yield_now();
                    *guard += 1;
                }
            });
        }
    });

    assert_eq!(lock.section(|v| *v), num_threads * NUM_ITERATIONS);
}
