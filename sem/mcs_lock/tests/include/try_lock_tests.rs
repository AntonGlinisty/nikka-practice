#[rstest]
#[timeout(Duration::from_millis(500))]
fn try_lock_basic() {
    let lock = Lock::new(0);

    let mut node_primary = LockNode::new();
    let mut guard_primary = lock.lock(&mut node_primary);

    let mut node_secondary = LockNode::new();
    assert!(lock.try_lock(&mut node_secondary).is_err());

    *guard_primary += 1;
    drop(guard_primary);

    let mut guard_secondary = lock.try_lock(&mut node_secondary).unwrap();
    *guard_secondary += 1;
    drop(guard_secondary);

    assert_eq!(*lock.try_lock(&mut node_primary).unwrap(), 2);
}

#[rstest]
#[timeout(Duration::from_secs(4))]
fn try_lock_busy_polling() {
    const TARGET: usize = if cfg!(miri) {
        300
    } else {
        30_000
    };

    let lock = Lock::new(0usize);

    thread::scope(|s| {
        let lock = &lock;

        let handle_remainder_try_lock = |rem| {
            s.spawn(move || {
                let mut node = LockNode::new();
                loop {
                    if let Ok(mut guard) = lock.try_lock(&mut node) {
                        let value = *guard;
                        if value >= TARGET {
                            break;
                        }
                        if value % 3 == rem {
                            *guard += 1;
                        }
                    }
                    core::hint::spin_loop();
                }
            })
        };

        handle_remainder_try_lock(0);
        handle_remainder_try_lock(1);

        s.spawn(move || {
            let mut node = LockNode::new();
            loop {
                let mut guard = lock.lock(&mut node);
                let value = *guard;
                if value >= TARGET {
                    break;
                }
                if value % 3 == 2 {
                    *guard += 1;
                }
                drop(guard);
                core::hint::spin_loop();
            }
        });
    });

    assert_eq!(lock.section(|v| *v), TARGET);
}
