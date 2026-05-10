    Finished `bench` profile [optimized] target(s) in 0.10s
     Running benches/8-queues-5-reserving-mpmc.rs (/tmp/target/release/deps/8_queues_5_reserving_mpmc-e87ff8660d19fa3c)
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0107 s (1.9M iterations)
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000: Analyzing
reserving_mpmc/1. spsc<u32, 16384>/1000
                        time:   [2.8250 µs 2.8483 µs 2.8709 µs]
                        thrpt:  [348.32 Melem/s 351.08 Melem/s 353.98 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0224 s (404k iterations)
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000: Analyzing
reserving_mpmc/2. reserving mpmc<u32, 16384>/1000
                        time:   [12.433 µs 12.644 µs 12.891 µs]
                        thrpt:  [77.573 Melem/s 79.092 Melem/s 80.428 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/10000
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0565 s (182k iterations)
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/10000: Analyzing
reserving_mpmc/1. spsc<u32, 16384>/10000
                        time:   [28.370 µs 28.739 µs 29.184 µs]
                        thrpt:  [342.66 Melem/s 347.96 Melem/s 352.49 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  4 (4.00%) high mild
  10 (10.00%) high severe
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/10000
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/10000: Collecting 100 samples in estimated 5.1316 s (40k iterations)
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/10000: Analyzing
reserving_mpmc/2. reserving mpmc<u32, 16384>/10000
                        time:   [131.92 µs 136.14 µs 140.73 µs]
                        thrpt:  [71.059 Melem/s 73.454 Melem/s 75.803 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/100000
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.3769 s (20k iterations)
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/100000: Analyzing
reserving_mpmc/1. spsc<u32, 16384>/100000
                        time:   [277.16 µs 278.80 µs 280.68 µs]
                        thrpt:  [356.27 Melem/s 358.68 Melem/s 360.80 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/100000
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/100000: Warming up for 3.0000 s

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/100000: Collecting 100 samples in estimated 6.4258 s (5050 iterations)
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/100000: Analyzing
reserving_mpmc/2. reserving mpmc<u32, 16384>/100000
                        time:   [1.3600 ms 1.4003 ms 1.4405 ms]
                        thrpt:  [69.418 Melem/s 71.414 Melem/s 73.528 Melem/s]
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000000
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.1381 s (1900 iterations)
Benchmarking reserving_mpmc/1. spsc<u32, 16384>/1000000: Analyzing
reserving_mpmc/1. spsc<u32, 16384>/1000000
                        time:   [2.7784 ms 2.8041 ms 2.8311 ms]
                        thrpt:  [353.21 Melem/s 356.62 Melem/s 359.92 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000000
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000000: Collecting 100 samples in estimated 6.0787 s (500 iterations)
Benchmarking reserving_mpmc/2. reserving mpmc<u32, 16384>/1000000: Analyzing
reserving_mpmc/2. reserving mpmc<u32, 16384>/1000000
                        time:   [12.292 ms 12.450 ms 12.645 ms]
                        thrpt:  [79.083 Melem/s 80.324 Melem/s 81.352 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking roundtrip/reserving mpmc roundtrip<u32, 16384>/1
Benchmarking roundtrip/reserving mpmc roundtrip<u32, 16384>/1: Warming up for 3.0000 s
Benchmarking roundtrip/reserving mpmc roundtrip<u32, 16384>/1: Collecting 100 samples in estimated 5.0006 s (36M iterations)
Benchmarking roundtrip/reserving mpmc roundtrip<u32, 16384>/1: Analyzing
roundtrip/reserving mpmc roundtrip<u32, 16384>/1
                        time:   [139.79 ns 140.47 ns 141.25 ns]
                        thrpt:  [7.0799 Melem/s 7.1189 Melem/s 7.1536 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

