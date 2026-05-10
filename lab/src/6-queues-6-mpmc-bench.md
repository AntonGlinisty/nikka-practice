    Finished `bench` profile [optimized] target(s) in 0.10s
     Running benches/8-queues-6-mpmc.rs (/tmp/target/release/deps/8_queues_6_mpmc-4361d7556f274c20)
Benchmarking mpmc/1. spsc<u32, 16384>/1000
Benchmarking mpmc/1. spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking mpmc/1. spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0076 s (1.6M iterations)
Benchmarking mpmc/1. spsc<u32, 16384>/1000: Analyzing
mpmc/1. spsc<u32, 16384>/1000
                        time:   [2.8044 µs 2.8389 µs 2.8784 µs]
                        thrpt:  [347.41 Melem/s 352.25 Melem/s 356.59 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking mpmc/2. mpmc<u32, 16384>/1000
Benchmarking mpmc/2. mpmc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking mpmc/2. mpmc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0016 s (202k iterations)
Benchmarking mpmc/2. mpmc<u32, 16384>/1000: Analyzing
mpmc/2. mpmc<u32, 16384>/1000
                        time:   [22.690 µs 23.017 µs 23.300 µs]
                        thrpt:  [42.918 Melem/s 43.446 Melem/s 44.073 Melem/s]
Benchmarking mpmc/1. spsc<u32, 16384>/10000
Benchmarking mpmc/1. spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking mpmc/1. spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0878 s (167k iterations)
Benchmarking mpmc/1. spsc<u32, 16384>/10000: Analyzing
mpmc/1. spsc<u32, 16384>/10000
                        time:   [28.626 µs 28.906 µs 29.186 µs]
                        thrpt:  [342.63 Melem/s 345.95 Melem/s 349.34 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking mpmc/2. mpmc<u32, 16384>/10000
Benchmarking mpmc/2. mpmc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking mpmc/2. mpmc<u32, 16384>/10000: Collecting 100 samples in estimated 5.4307 s (25k iterations)
Benchmarking mpmc/2. mpmc<u32, 16384>/10000: Analyzing
mpmc/2. mpmc<u32, 16384>/10000
                        time:   [212.01 µs 212.75 µs 213.46 µs]
                        thrpt:  [46.846 Melem/s 47.004 Melem/s 47.168 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
Benchmarking mpmc/1. spsc<u32, 16384>/100000
Benchmarking mpmc/1. spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking mpmc/1. spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.8320 s (20k iterations)
Benchmarking mpmc/1. spsc<u32, 16384>/100000: Analyzing
mpmc/1. spsc<u32, 16384>/100000
                        time:   [277.85 µs 279.30 µs 281.06 µs]
                        thrpt:  [355.80 Melem/s 358.04 Melem/s 359.91 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  6 (6.00%) high mild
  9 (9.00%) high severe
Benchmarking mpmc/2. mpmc<u32, 16384>/100000
Benchmarking mpmc/2. mpmc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking mpmc/2. mpmc<u32, 16384>/100000: Collecting 100 samples in estimated 5.0214 s (2400 iterations)
Benchmarking mpmc/2. mpmc<u32, 16384>/100000: Analyzing
mpmc/2. mpmc<u32, 16384>/100000
                        time:   [2.0571 ms 2.0681 ms 2.0777 ms]
                        thrpt:  [48.129 Melem/s 48.355 Melem/s 48.611 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking mpmc/1. spsc<u32, 16384>/1000000
Benchmarking mpmc/1. spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking mpmc/1. spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.1100 s (1700 iterations)
Benchmarking mpmc/1. spsc<u32, 16384>/1000000: Analyzing
mpmc/1. spsc<u32, 16384>/1000000
                        time:   [2.8316 ms 2.8606 ms 2.8931 ms]
                        thrpt:  [345.65 Melem/s 349.58 Melem/s 353.15 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking mpmc/2. mpmc<u32, 16384>/1000000
Benchmarking mpmc/2. mpmc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking mpmc/2. mpmc<u32, 16384>/1000000: Collecting 100 samples in estimated 6.2712 s (300 iterations)
Benchmarking mpmc/2. mpmc<u32, 16384>/1000000: Analyzing
mpmc/2. mpmc<u32, 16384>/1000000
                        time:   [20.621 ms 20.715 ms 20.801 ms]
                        thrpt:  [48.074 Melem/s 48.274 Melem/s 48.495 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low severe
  1 (1.00%) high severe

Benchmarking roundtrip/mpmc roundtrip<u32, 16384>/1
Benchmarking roundtrip/mpmc roundtrip<u32, 16384>/1: Warming up for 3.0000 s
Benchmarking roundtrip/mpmc roundtrip<u32, 16384>/1: Collecting 100 samples in estimated 5.0007 s (35M iterations)
Benchmarking roundtrip/mpmc roundtrip<u32, 16384>/1: Analyzing
roundtrip/mpmc roundtrip<u32, 16384>/1
                        time:   [143.43 ns 144.07 ns 144.63 ns]
                        thrpt:  [6.9142 Melem/s 6.9412 Melem/s 6.9719 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

