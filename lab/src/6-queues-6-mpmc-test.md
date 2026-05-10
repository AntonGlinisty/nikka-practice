    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/8-queues-6-mpmc.rs (/tmp/target/debug/deps/8_queues_6_mpmc-f0f28c1f45e0834d)

running 6 tests
test basic ... ok
test producer_and_consumer_are_multiple ... ok
test singleton_queue ... ok
test sequential ... ok
test overflow ... ok
2026-05-11T03:22:04.500454Z : consumer #0 finished consumer_sum=3603205820
2026-05-11T03:22:04.545561Z : producer #1 finished producer_sum=3404708035
2026-05-11T03:22:04.565205Z : producer #0 finished producer_sum=1333327745
2026-05-11T03:22:04.568821Z : consumer #1 finished consumer_sum=1134829960
2026-05-11T03:22:04.569013Z :  concurrent producer_sum=443068484 consumer_sum=443068484
test concurrent ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s

