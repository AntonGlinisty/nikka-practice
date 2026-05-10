    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/8-queues-5-reserving-mpmc.rs (/tmp/target/debug/deps/8_queues_5_reserving_mpmc-b6360148f4e0a11e)

running 4 tests
test basic ... ok
test producer_and_consumer_are_multiple ... ok
2026-05-11T03:22:00.479222Z : consumer #1 finished consumer_sum=5158960966085240071
2026-05-11T03:22:01.009252Z : producer #1 finished producer_sum=14644585532409591817
2026-05-11T03:22:01.019317Z : producer #0 finished producer_sum=5748067909771789215
2026-05-11T03:22:01.020263Z : consumer #0 finished consumer_sum=15233692476096140961
2026-05-11T03:22:01.020465Z :  concurrent producer_sum=1945909368471829416 consumer_sum=1945909368471829416
test concurrent ... ok
2026-05-11T03:22:01.859127Z :  sequential producer_sum=5748067909771789215 consumer_sum=5748067909771789215
test sequential ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.09s

