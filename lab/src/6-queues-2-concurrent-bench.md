    Finished `bench` profile [optimized] target(s) in 0.13s
     Running benches/8-queues-2-spsc.rs (/tmp/target/release/deps/8_queues_2_spsc-8bb1d5b4d717a6ea)
Benchmarking concurrent/1. spsc<u32, 16384>/1000
Benchmarking concurrent/1. spsc<u32, 16384>/1000: Warming up for 3.0000 s
Benchmarking concurrent/1. spsc<u32, 16384>/1000: Collecting 100 samples in estimated 5.0133 s (1.8M iterations)
Benchmarking concurrent/1. spsc<u32, 16384>/1000: Analyzing
concurrent/1. spsc<u32, 16384>/1000
                        time:   [3.1998 µs 3.3859 µs 3.5658 µs]
                        thrpt:  [280.44 Melem/s 295.34 Melem/s 312.51 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking concurrent/1. spsc<u32, 16384>/10000
Benchmarking concurrent/1. spsc<u32, 16384>/10000: Warming up for 3.0000 s
Benchmarking concurrent/1. spsc<u32, 16384>/10000: Collecting 100 samples in estimated 5.0334 s (111k iterations)
Benchmarking concurrent/1. spsc<u32, 16384>/10000: Analyzing
concurrent/1. spsc<u32, 16384>/10000
                        time:   [66.903 µs 75.946 µs 85.959 µs]
                        thrpt:  [116.33 Melem/s 131.67 Melem/s 149.47 Melem/s]
Benchmarking concurrent/1. spsc<u32, 16384>/100000
Benchmarking concurrent/1. spsc<u32, 16384>/100000: Warming up for 3.0000 s
Benchmarking concurrent/1. spsc<u32, 16384>/100000: Collecting 100 samples in estimated 5.3227 s (10k iterations)
Benchmarking concurrent/1. spsc<u32, 16384>/100000: Analyzing
concurrent/1. spsc<u32, 16384>/100000
                        time:   [462.00 µs 497.07 µs 535.30 µs]
                        thrpt:  [186.81 Melem/s 201.18 Melem/s 216.45 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking concurrent/1. spsc<u32, 16384>/1000000
Benchmarking concurrent/1. spsc<u32, 16384>/1000000: Warming up for 3.0000 s
Benchmarking concurrent/1. spsc<u32, 16384>/1000000: Collecting 100 samples in estimated 5.3841 s (1200 iterations)
Benchmarking concurrent/1. spsc<u32, 16384>/1000000: Analyzing
concurrent/1. spsc<u32, 16384>/1000000
                        time:   [4.1913 ms 4.5503 ms 4.9199 ms]
                        thrpt:  [203.26 Melem/s 219.77 Melem/s 238.59 Melem/s]

