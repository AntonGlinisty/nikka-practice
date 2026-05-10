    Finished `bench` profile [optimized] target(s) in 0.15s
     Running benches/8-queues-2-spsc.rs (/tmp/target/release/deps/8_queues_2_spsc-8bb1d5b4d717a6ea)
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0019 s (1.6M iterations)
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000: Analyzing
sequential/1. sequential spsc<u32, 16384>/1000
                        time:   [3.0804 µs 3.0865 µs 3.0926 µs]
                        thrpt:  [323.35 Melem/s 324.00 Melem/s 324.64 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000: Collecting 100 samples in estimated 5.0006 s (2.0M iterations)
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000: Analyzing
sequential/2. infinite_indices<u32, 16384>/1000
                        time:   [2.4859 µs 2.4904 µs 2.4951 µs]
                        thrpt:  [400.79 Melem/s 401.54 Melem/s 402.26 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking sequential/3. sentinel<u32, 16384>/1000
Benchmarking sequential/3. sentinel<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking sequential/3. sentinel<u32, 16384>/1000: Collecting 100 samples in estimated 5.0081 s (1.6M iterations)
Benchmarking sequential/3. sentinel<u32, 16384>/1000: Analyzing
sequential/3. sentinel<u32, 16384>/1000
                        time:   [3.0417 µs 3.0472 µs 3.0526 µs]
                        thrpt:  [327.59 Melem/s 328.17 Melem/s 328.76 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking sequential/1. sequential spsc<u32, 16384>/10000
Benchmarking sequential/1. sequential spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking sequential/1. sequential spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0069 s (172k iterations)
Benchmarking sequential/1. sequential spsc<u32, 16384>/10000: Analyzing
sequential/1. sequential spsc<u32, 16384>/10000
                        time:   [29.016 µs 29.094 µs 29.172 µs]
                        thrpt:  [342.80 Melem/s 343.72 Melem/s 344.64 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking sequential/2. infinite_indices<u32, 16384>/10000
Benchmarking sequential/2. infinite_indices<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking sequential/2. infinite_indices<u32, 16384>/10000: Collecting 100 samples in estimated 5.0590 s (227k iterations)
Benchmarking sequential/2. infinite_indices<u32, 16384>/10000: Analyzing
sequential/2. infinite_indices<u32, 16384>/10000
                        time:   [22.171 µs 22.211 µs 22.254 µs]
                        thrpt:  [449.36 Melem/s 450.23 Melem/s 451.03 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
Benchmarking sequential/3. sentinel<u32, 16384>/10000
Benchmarking sequential/3. sentinel<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking sequential/3. sentinel<u32, 16384>/10000: Collecting 100 samples in estimated 5.1304 s (177k iterations)
Benchmarking sequential/3. sentinel<u32, 16384>/10000: Analyzing
sequential/3. sentinel<u32, 16384>/10000
                        time:   [28.876 µs 28.933 µs 28.991 µs]
                        thrpt:  [344.94 Melem/s 345.63 Melem/s 346.30 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking sequential/1. sequential spsc<u32, 16384>/100000
Benchmarking sequential/1. sequential spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking sequential/1. sequential spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.8035 s (20k iterations)
Benchmarking sequential/1. sequential spsc<u32, 16384>/100000: Analyzing
sequential/1. sequential spsc<u32, 16384>/100000
                        time:   [288.22 µs 288.84 µs 289.45 µs]
                        thrpt:  [345.48 Melem/s 346.21 Melem/s 346.95 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking sequential/2. infinite_indices<u32, 16384>/100000
Benchmarking sequential/2. infinite_indices<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking sequential/2. infinite_indices<u32, 16384>/100000: Collecting 100 samples in estimated 5.5668 s (25k iterations)
Benchmarking sequential/2. infinite_indices<u32, 16384>/100000: Analyzing
sequential/2. infinite_indices<u32, 16384>/100000
                        time:   [219.32 µs 219.64 µs 219.98 µs]
                        thrpt:  [454.59 Melem/s 455.29 Melem/s 455.96 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) low mild
  1 (1.00%) high severe
Benchmarking sequential/3. sentinel<u32, 16384>/100000
Benchmarking sequential/3. sentinel<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking sequential/3. sentinel<u32, 16384>/100000: Collecting 100 samples in estimated 6.0074 s (20k iterations)
Benchmarking sequential/3. sentinel<u32, 16384>/100000: Analyzing
sequential/3. sentinel<u32, 16384>/100000
                        time:   [296.54 µs 297.58 µs 298.67 µs]
                        thrpt:  [334.82 Melem/s 336.05 Melem/s 337.22 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000000
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.0720 s (1800 iterations)
Benchmarking sequential/1. sequential spsc<u32, 16384>/1000000: Analyzing
sequential/1. sequential spsc<u32, 16384>/1000000
                        time:   [2.8301 ms 2.8418 ms 2.8553 ms]
                        thrpt:  [350.23 Melem/s 351.89 Melem/s 353.34 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000000
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000000: Collecting 100 samples in estimated 5.0071 s (2200 iterations)
Benchmarking sequential/2. infinite_indices<u32, 16384>/1000000: Analyzing
sequential/2. infinite_indices<u32, 16384>/1000000
                        time:   [2.2602 ms 2.2656 ms 2.2709 ms]
                        thrpt:  [440.36 Melem/s 441.39 Melem/s 442.44 Melem/s]
Benchmarking sequential/3. sentinel<u32, 16384>/1000000
Benchmarking sequential/3. sentinel<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking sequential/3. sentinel<u32, 16384>/1000000: Collecting 100 samples in estimated 5.0253 s (1700 iterations)
Benchmarking sequential/3. sentinel<u32, 16384>/1000000: Analyzing
sequential/3. sentinel<u32, 16384>/1000000
                        time:   [2.9448 ms 2.9550 ms 2.9651 ms]
                        thrpt:  [337.25 Melem/s 338.41 Melem/s 339.58 Melem/s]

