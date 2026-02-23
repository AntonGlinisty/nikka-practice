#[test]
fn fairness() {
    cfg_select! {
        miri => {
            const THREADS: usize = 4;
            const BASIC_BLOCK: Duration = Duration::from_micros(5);
            const THREAD_DISADVANTAGE: Duration = BASIC_BLOCK * 1500;
            const DURATION: Duration = THREAD_DISADVANTAGE * (THREADS - 1) as u32 * 600;
        }
        _ => {
            const THREADS: usize = 5;
            const THREAD_DISADVANTAGE: Duration = Duration::from_millis(2);
            const DURATION: Duration = THREAD_DISADVANTAGE * (THREADS - 1) as u32 * 600;
        }
    }
    let lock = Lock::new(vec![]);
    let finish = AtomicBool::new(false);
    let barrier_start = Barrier::new(THREADS + 1);
    let barrier_end = Barrier::new(THREADS + 1);

    let (cnt_mismatch, iterations) = thread::scope(|s| {
        for t in 0 .. THREADS {
            let lock = &lock;
            let finish = &finish;
            let barrier_start = &barrier_start;
            let barrier_end = &barrier_end;

            s.spawn(move || {
                loop {
                    barrier_start.wait();
                    if finish.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(THREAD_DISADVANTAGE * t as u32);
                    let mut node = LockNode::new();
                    let mut guard = lock.lock(&mut node);
                    guard.push(t);
                    if t == 0 {
                        thread::sleep(THREAD_DISADVANTAGE * (THREADS - 1) as u32);
                    }
                    drop(guard);
                    barrier_end.wait();
                }
            });
        }

        let mut cnt_mismatch = 0;
        let mut iterations = 0;
        let start = Instant::now();
        while start.elapsed() < DURATION {
            barrier_start.wait();
            barrier_end.wait();

            let mut node = LockNode::new();
            let history = core::mem::take(&mut *lock.try_lock(&mut node).unwrap());
            let expected = (0 .. THREADS).collect::<Vec<_>>();
            if history != expected {
                println!("Unexpected order: {history:?}");
                cnt_mismatch += 1;
            }
            iterations += 1;
        }
        finish.store(true, Ordering::Relaxed);
        barrier_start.wait();

        (cnt_mismatch, iterations)
    });

    let ratio = cnt_mismatch as f64 / iterations as f64;
    println!(
        "cnt_mismatch: {}, iterations: {}, ratio: {:.3}",
        cnt_mismatch, iterations, ratio,
    );
    assert!(ratio < 0.02, "ratio: {ratio}");
}
