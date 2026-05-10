    Finished `bench` profile [optimized] target(s) in 0.10s
     Running benches/8-queues-3-spsc-slice.rs (/tmp/target/release/deps/8_queues_3_spsc_slice-350051effe34e753)
Benchmarking slice/5. slice spsc<u32, 16384>/1000
Benchmarking slice/5. slice spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking slice/5. slice spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0006 s (20M iterations)
Benchmarking slice/5. slice spsc<u32, 16384>/1000: Analyzing
slice/5. slice spsc<u32, 16384>/1000
                        time:   [247.05 ns 248.65 ns 250.99 ns]
                        thrpt:  [3.9842 Gelem/s 4.0218 Gelem/s 4.0477 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking slice/7. double memory copy<u32, 16384>/1000
Benchmarking slice/7. double memory copy<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking slice/7. double memory copy<u32, 16384>/1000: Collecting 100 samples in estimated 5.0004 s (29M iterations)
Benchmarking slice/7. double memory copy<u32, 16384>/1000: Analyzing
slice/7. double memory copy<u32, 16384>/1000
                        time:   [170.74 ns 171.08 ns 171.44 ns]
                        thrpt:  [5.8329 Gelem/s 5.8451 Gelem/s 5.8570 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking slice/5. slice spsc<u32, 16384>/10000
Benchmarking slice/5. slice spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking slice/5. slice spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0054 s (2.0M iterations)
Benchmarking slice/5. slice spsc<u32, 16384>/10000: Analyzing
slice/5. slice spsc<u32, 16384>/10000
                        time:   [2.4906 µs 2.4975 µs 2.5055 µs]
                        thrpt:  [3.9912 Gelem/s 4.0040 Gelem/s 4.0151 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking slice/7. double memory copy<u32, 16384>/10000
Benchmarking slice/7. double memory copy<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking slice/7. double memory copy<u32, 16384>/10000: Collecting 100 samples in estimated 5.0103 s (1.6M iterations)
Benchmarking slice/7. double memory copy<u32, 16384>/10000: Analyzing
slice/7. double memory copy<u32, 16384>/10000
                        time:   [2.9424 µs 2.9595 µs 2.9804 µs]
                        thrpt:  [3.3553 Gelem/s 3.3789 Gelem/s 3.3986 Gelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking slice/5. slice spsc<u32, 16384>/100000
Benchmarking slice/5. slice spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking slice/5. slice spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.0935 s (207k iterations)
Benchmarking slice/5. slice spsc<u32, 16384>/100000: Analyzing
slice/5. slice spsc<u32, 16384>/100000
                        time:   [24.556 µs 24.647 µs 24.745 µs]
                        thrpt:  [4.0412 Gelem/s 4.0572 Gelem/s 4.0724 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking slice/7. double memory copy<u32, 16384>/100000
Benchmarking slice/7. double memory copy<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking slice/7. double memory copy<u32, 16384>/100000: Collecting 100 samples in estimated 5.0607 s (167k iterations)
Benchmarking slice/7. double memory copy<u32, 16384>/100000: Analyzing
slice/7. double memory copy<u32, 16384>/100000
                        time:   [30.327 µs 30.397 µs 30.474 µs]
                        thrpt:  [3.2815 Gelem/s 3.2898 Gelem/s 3.2974 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking slice/5. slice spsc<u32, 16384>/1000000
Benchmarking slice/5. slice spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking slice/5. slice spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 6.2081 s (20k iterations)
Benchmarking slice/5. slice spsc<u32, 16384>/1000000: Analyzing
slice/5. slice spsc<u32, 16384>/1000000
                        time:   [316.17 µs 321.73 µs 327.37 µs]
                        thrpt:  [3.0546 Gelem/s 3.1082 Gelem/s 3.1629 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking slice/7. double memory copy<u32, 16384>/1000000
Benchmarking slice/7. double memory copy<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking slice/7. double memory copy<u32, 16384>/1000000: Collecting 100 samples in estimated 6.1331 s (20k iterations)
Benchmarking slice/7. double memory copy<u32, 16384>/1000000: Analyzing
slice/7. double memory copy<u32, 16384>/1000000
                        time:   [304.61 µs 305.10 µs 305.59 µs]
                        thrpt:  [3.2724 Gelem/s 3.2777 Gelem/s 3.2829 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

