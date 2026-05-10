    Finished `bench` profile [optimized] target(s) in 0.10s
     Running benches/8-queues-2-spsc.rs (/tmp/target/release/deps/8_queues_2_spsc-8bb1d5b4d717a6ea)
Benchmarking tx/1. single spsc<u32, 16384>/1000
Benchmarking tx/1. single spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking tx/1. single spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0382 s (556k iterations)
Benchmarking tx/1. single spsc<u32, 16384>/1000: Analyzing
tx/1. single spsc<u32, 16384>/1000
                        time:   [8.0144 µs 9.1555 µs 10.326 µs]
                        thrpt:  [96.847 Melem/s 109.22 Melem/s 124.78 Melem/s]
                 change:
                        time:   [+109.67% +139.71% +173.86%] (p = 0.00 < 0.05)
                        thrpt:  [−63.485% −58.283% −52.307%]
                        Performance has regressed.
Benchmarking tx/2. tx spsc<u32, 16384>/1000
Benchmarking tx/2. tx spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking tx/2. tx spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0023 s (4.2M iterations)
Benchmarking tx/2. tx spsc<u32, 16384>/1000: Analyzing
tx/2. tx spsc<u32, 16384>/1000
                        time:   [1.1802 µs 1.1856 µs 1.1943 µs]
                        thrpt:  [837.28 Melem/s 843.44 Melem/s 847.33 Melem/s]
                 change:
                        time:   [−1.5643% −0.4138% +0.5032%] (p = 0.50 > 0.05)
                        thrpt:  [−0.5007% +0.4155% +1.5892%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking tx/3. infinite_indices<u32, 16384>/1000
Benchmarking tx/3. infinite_indices<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking tx/3. infinite_indices<u32, 16384>/1000: Collecting 100 samples in estimated 5.0097 s (2.0M iterations)
Benchmarking tx/3. infinite_indices<u32, 16384>/1000: Analyzing
tx/3. infinite_indices<u32, 16384>/1000
                        time:   [2.4905 µs 2.4958 µs 2.5012 µs]
                        thrpt:  [399.80 Melem/s 400.67 Melem/s 401.53 Melem/s]
                 change:
                        time:   [−3.2963% −2.7600% −2.2350%] (p = 0.00 < 0.05)
                        thrpt:  [+2.2861% +2.8383% +3.4087%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking tx/4. sentinel<u32, 16384>/1000
Benchmarking tx/4. sentinel<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking tx/4. sentinel<u32, 16384>/1000: Collecting 100 samples in estimated 5.0076 s (1.6M iterations)
Benchmarking tx/4. sentinel<u32, 16384>/1000: Analyzing
tx/4. sentinel<u32, 16384>/1000
                        time:   [3.0858 µs 3.0951 µs 3.1058 µs]
                        thrpt:  [321.98 Melem/s 323.09 Melem/s 324.07 Melem/s]
                 change:
                        time:   [+2.0430% +2.5199% +2.9467%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8624% −2.4580% −2.0021%]
                        Performance has regressed.
Benchmarking tx/1. single spsc<u32, 16384>/10000
Benchmarking tx/1. single spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking tx/1. single spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.1734 s (76k iterations)
Benchmarking tx/1. single spsc<u32, 16384>/10000: Analyzing
tx/1. single spsc<u32, 16384>/10000
                        time:   [73.687 µs 83.967 µs 93.925 µs]
                        thrpt:  [106.47 Melem/s 119.09 Melem/s 135.71 Melem/s]
                 change:
                        time:   [−27.255% −13.298% +3.2598%] (p = 0.13 > 0.05)
                        thrpt:  [−3.1569% +15.338% +37.466%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking tx/2. tx spsc<u32, 16384>/10000
Benchmarking tx/2. tx spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking tx/2. tx spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0120 s (429k iterations)
Benchmarking tx/2. tx spsc<u32, 16384>/10000: Analyzing
tx/2. tx spsc<u32, 16384>/10000
                        time:   [11.584 µs 11.618 µs 11.653 µs]
                        thrpt:  [858.12 Melem/s 860.76 Melem/s 863.24 Melem/s]
                 change:
                        time:   [−2.7213% −1.0369% +0.2101%] (p = 0.20 > 0.05)
                        thrpt:  [−0.2096% +1.0478% +2.7974%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking tx/3. infinite_indices<u32, 16384>/10000
Benchmarking tx/3. infinite_indices<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking tx/3. infinite_indices<u32, 16384>/10000: Collecting 100 samples in estimated 5.1106 s (207k iterations)
Benchmarking tx/3. infinite_indices<u32, 16384>/10000: Analyzing
tx/3. infinite_indices<u32, 16384>/10000
                        time:   [30.188 µs 32.315 µs 34.631 µs]
                        thrpt:  [288.76 Melem/s 309.46 Melem/s 331.26 Melem/s]
                 change:
                        time:   [+26.310% +33.409% +40.927%] (p = 0.00 < 0.05)
                        thrpt:  [−29.041% −25.042% −20.829%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking tx/4. sentinel<u32, 16384>/10000
Benchmarking tx/4. sentinel<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking tx/4. sentinel<u32, 16384>/10000: Collecting 100 samples in estimated 5.1421 s (131k iterations)
Benchmarking tx/4. sentinel<u32, 16384>/10000: Analyzing
tx/4. sentinel<u32, 16384>/10000
                        time:   [38.145 µs 41.385 µs 44.646 µs]
                        thrpt:  [223.99 Melem/s 241.63 Melem/s 262.16 Melem/s]
                 change:
                        time:   [−0.2350% +9.9050% +20.293%] (p = 0.06 > 0.05)
                        thrpt:  [−16.869% −9.0123% +0.2355%]
                        No change in performance detected.
Benchmarking tx/1. single spsc<u32, 16384>/100000
Benchmarking tx/1. single spsc<u32, 16384>/100000: Warming up for 3.0000 s

Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.3s, enable flat sampling, or reduce sample count to 60.
Benchmarking tx/1. single spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.3027 s (5050 iterations)
Benchmarking tx/1. single spsc<u32, 16384>/100000: Analyzing
tx/1. single spsc<u32, 16384>/100000
                        time:   [882.65 µs 990.91 µs 1.0973 ms]
                        thrpt:  [91.133 Melem/s 100.92 Melem/s 113.29 Melem/s]
                 change:
                        time:   [−35.953% −25.202% −12.839%] (p = 0.00 < 0.05)
                        thrpt:  [+14.731% +33.694% +56.135%]
                        Performance has improved.
Benchmarking tx/2. tx spsc<u32, 16384>/100000
Benchmarking tx/2. tx spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking tx/2. tx spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.7743 s (20k iterations)
Benchmarking tx/2. tx spsc<u32, 16384>/100000: Analyzing
tx/2. tx spsc<u32, 16384>/100000
                        time:   [134.25 µs 139.64 µs 145.66 µs]
                        thrpt:  [686.53 Melem/s 716.12 Melem/s 744.88 Melem/s]
                 change:
                        time:   [−67.884% −61.887% −55.281%] (p = 0.00 < 0.05)
                        thrpt:  [+123.62% +162.38% +211.37%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking tx/3. infinite_indices<u32, 16384>/100000
Benchmarking tx/3. infinite_indices<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking tx/3. infinite_indices<u32, 16384>/100000: Collecting 100 samples in estimated 5.6771 s (25k iterations)
Benchmarking tx/3. infinite_indices<u32, 16384>/100000: Analyzing
tx/3. infinite_indices<u32, 16384>/100000
                        time:   [220.39 µs 220.72 µs 221.06 µs]
                        thrpt:  [452.36 Melem/s 453.07 Melem/s 453.75 Melem/s]
                 change:
                        time:   [−54.270% −50.326% −45.873%] (p = 0.00 < 0.05)
                        thrpt:  [+84.751% +101.31% +118.67%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking tx/4. sentinel<u32, 16384>/100000
Benchmarking tx/4. sentinel<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking tx/4. sentinel<u32, 16384>/100000: Collecting 100 samples in estimated 5.8518 s (20k iterations)
Benchmarking tx/4. sentinel<u32, 16384>/100000: Analyzing
tx/4. sentinel<u32, 16384>/100000
                        time:   [289.58 µs 291.12 µs 293.31 µs]
                        thrpt:  [340.93 Melem/s 343.50 Melem/s 345.33 Melem/s]
                 change:
                        time:   [−39.244% −35.214% −30.768%] (p = 0.00 < 0.05)
                        thrpt:  [+44.442% +54.354% +64.592%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  2 (2.00%) high severe
Benchmarking tx/1. single spsc<u32, 16384>/1000000
Benchmarking tx/1. single spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking tx/1. single spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.0125 s (1300 iterations)
Benchmarking tx/1. single spsc<u32, 16384>/1000000: Analyzing
tx/1. single spsc<u32, 16384>/1000000
                        time:   [3.9415 ms 4.2057 ms 4.4832 ms]
                        thrpt:  [223.06 Melem/s 237.77 Melem/s 253.71 Melem/s]
                 change:
                        time:   [−59.738% −54.518% −48.335%] (p = 0.00 < 0.05)
                        thrpt:  [+93.556% +119.87% +148.37%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking tx/2. tx spsc<u32, 16384>/1000000
Benchmarking tx/2. tx spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking tx/2. tx spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.1829 s (2600 iterations)
Benchmarking tx/2. tx spsc<u32, 16384>/1000000: Analyzing
tx/2. tx spsc<u32, 16384>/1000000
                        time:   [2.0319 ms 2.0379 ms 2.0441 ms]
                        thrpt:  [489.21 Melem/s 490.71 Melem/s 492.14 Melem/s]
                 change:
                        time:   [−0.8511% +0.0824% +1.0216%] (p = 0.87 > 0.05)
                        thrpt:  [−1.0112% −0.0823% +0.8584%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking tx/3. infinite_indices<u32, 16384>/1000000
Benchmarking tx/3. infinite_indices<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking tx/3. infinite_indices<u32, 16384>/1000000: Collecting 100 samples in estimated 5.0412 s (2300 iterations)
Benchmarking tx/3. infinite_indices<u32, 16384>/1000000: Analyzing
tx/3. infinite_indices<u32, 16384>/1000000
                        time:   [2.1925 ms 2.1958 ms 2.1992 ms]
                        thrpt:  [454.72 Melem/s 455.42 Melem/s 456.11 Melem/s]
                 change:
                        time:   [−3.6704% −3.4234% −3.1569%] (p = 0.00 < 0.05)
                        thrpt:  [+3.2598% +3.5448% +3.8102%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking tx/4. sentinel<u32, 16384>/1000000
Benchmarking tx/4. sentinel<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking tx/4. sentinel<u32, 16384>/1000000: Collecting 100 samples in estimated 5.1914 s (1800 iterations)
Benchmarking tx/4. sentinel<u32, 16384>/1000000: Analyzing
tx/4. sentinel<u32, 16384>/1000000
                        time:   [2.9077 ms 2.9180 ms 2.9286 ms]
                        thrpt:  [341.46 Melem/s 342.69 Melem/s 343.91 Melem/s]
                 change:
                        time:   [+0.6154% +1.0279% +1.4324%] (p = 0.00 < 0.05)
                        thrpt:  [−1.4122% −1.0174% −0.6116%]
                        Change within noise threshold.

