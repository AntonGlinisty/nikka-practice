    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running tests/8-queues-4-sharded-mpmc.rs (/tmp/target/debug/deps/8_queues_4_sharded_mpmc-61861095a79e1733)

running 5 tests
test basic ... ok
test producer_and_consumer_are_multiple ... ok
test singleton_queue ... ok
2026-05-11T03:33:57.907173Z : sequential producer_sum=17041964526909660262 consumer_sum=17041964526909660262
test sequential ... ok
2026-05-11T03:34:11.532329Z : producer #0 finished producer_sum=5748067909771789215
2026-05-11T03:34:11.666874Z : consumer #0 finished consumer_sum=14571933187654422014
2026-05-11T03:34:11.793954Z : producer #1 finished producer_sum=14644585532409591817
2026-05-11T03:34:11.824855Z : consumer #1 finished consumer_sum=5820720254526959018
2026-05-11T03:34:11.825115Z :  concurrent producer_sum=1945909368471829416 consumer_sum=1945909368471829416
test concurrent ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.07s

