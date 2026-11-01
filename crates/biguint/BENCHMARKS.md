# Benchmarks

`biguint` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod and mulmod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits.

The numbers measure `biguint` at commit `4d45ea7`, the last commit that changed `crates/biguint/src`. The bench ran at commit `b374b26` on 2026-11-01. These commands regenerate the results:

```sh
cargo run -p biguint-bench-report -- bench    # criterion, for both builds
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```

## Summary

Each cell: `biguint`'s time per operation over the fastest other crate's. Below 1, `biguint` is faster.

| Operation | Build | 64 bits | 128 bits | 256 bits | 384 bits | 512 bits | 1024 bits | 2048 bits | 4096 bits |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| add | default | 1.01 | 1.06 | 1.05 | 1.00 | 0.91 | 1.06 | 0.81 | 0.93 |
| add | native | 1.11 | 1.01 | 1.02 | 1.01 | 1.32 | 0.85 | 0.80 | 1.09 |
| mul | default | 1.13 | 1.26 | 1.23 | 1.24 | 1.15 | 1.30 | 1.31 | 1.08 |
| mul | native | 1.01 | 1.19 | 1.06 | 1.10 | 1.21 | 1.34 | 1.29 | 1.13 |
| div | default | 1.09 | 0.98 | 1.21 | 1.06 | 0.89 | 0.93 | 0.91 | 1.00 |
| div | native | 1.02 | 1.03 | 1.66 | 1.00 | 1.00 | 0.97 | 0.95 | 1.05 |
| mod | default | 1.06 | 1.03 | 1.16 | 0.93 | 1.03 | 0.95 | 0.95 | 0.89 |
| mod | native | 1.04 | 1.04 | 1.52 | 0.97 | 0.99 | 1.03 | 1.04 | 1.04 |
| mulmod | default | 1.11 | 1.36 | 0.98 | 0.93 | 0.98 | 1.17 | 0.79 | 0.79 |
| mulmod | native | 0.99 | 1.35 | 1.17 | 0.93 | 1.13 | 1.18 | 0.88 | 0.83 |

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor; `znver3` (the CPU that `-C target-cpu=native` selects); 1 logical CPU available to the run |
| Host | `x86_64-unknown-linux-gnu` |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Builds | `default`: no `RUSTFLAGS`, the baseline of the host target. `native`: `RUSTFLAGS="-C target-cpu=native"`. |
| Timing | criterion 0.8.2 with `--quick`: no warm-up; the iterations of a case double until two consecutive runs agree within 5%, from 100 ms to 3 s |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

**The times are provisional.** load average 17 to 28 (median 24) on 32 hardware threads during the run; each build pinned to CPU 15 Each time is an upper bound, and the times compare only within one table.

## Method

- Each case runs one crate's function over a batch of 256 operand sets. Each operand set and each result passes through `core::hint::black_box`, so the branch predictors see varied data and the compiler computes every result.
- Time per operation: criterion's median estimate of one iteration, divided by 256.
- Before a case is measured, its result on every operand set must equal the exact result from `num-bigint`, reduced modulo 2^N for the crates that wrap.

## Operands

At width N:

| Operation | Operands |
|---|---|
| add, mul | `a` and `b` uniform below 2^N |
| div, mod | a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held in an N-bit integer |
| mulmod | a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m` |

Every crate in a row gets the same 256 operand sets.

## Functions

Each crate runs its own fastest public function for an operation. A crate without one runs the obvious composition of its public functions, marked composed.

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---|---|---|---|---|---|---|
| add | `+` | `wrapping_add` at 64 and 128 bits | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` | `&a + &b` |
| mul | `*` | `wrapping_mul` at 64 and 128 bits | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `overflowing_mul` | `&a * &b` |
| div | `div_rem` | `/ NonZero` at 64 and 128 bits | `wrapping_div` | `wrapping_div_vartime` | `wrapping_div` | `/` | `&a / &b` |
| mod | `div_rem` | `% NonZero` at 64 and 128 bits | `wrapping_rem` | `rem_vartime` | `wrapping_rem` | `%` | `&a % &b` |
| mulmod | `mul_mod` | `u128` product, then `% NonZero<u128>` (composed) at 64 bits | `mul_mod` | `mul_mod_vartime` | `widening_mul`, the halves joined at twice the width, then `wrapping_rem` (composed) | `uint_full_mul_reg!` (`full_mul` in primitive-types), then `%` (composed) at 64, 128, 256, 384, 512, 1024 and 2048 bits | `&a * &b`, then `%` (composed) |

- `native` is `u64` at 64 bits and `u128` at 128 bits. Its divisors are `NonZero`.
- `biguint` subtracts with the carry chain of its addition, so the bench measures add only.
- `crypto-bigint` is constant-time by design. Its div, mod and mulmod cases run its `*_vartime` functions. Its add and mul have no variable-time variant.
- `num-bigint` allocates every result and returns it exactly: add returns up to N + 1 bits, and mul returns the 2N-bit product. Its times include the allocation.
- `uint` is the parity crate. Its types come from `construct_uint!`, as in `primitive-types`; its add and mul wrap through `overflowing_*`.
- `native` has no case for add, mul, div, mod and mulmod at 256, 384, 512, 1024, 2048 and 4096 bits: no primitive of this width.
- `native` has no case for mulmod at 128 bits: no 256-bit primitive.
- `uint` has no case for mulmod at 4096 bits: `construct_uint!` stops at 64 words, below the 128 of the 8192-bit product.

## Results

Each cell: nanoseconds per operation. Bold marks the fastest time in each row. A dash marks a crate without a case.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 0.572 ns | 0.897 ns | 0.615 ns | 0.582 ns | 0.590 ns | **0.564 ns** | 16.6 ns |
| 64 | native | 0.569 ns | 0.655 ns | 0.561 ns | **0.513 ns** | 0.645 ns | 0.540 ns | 13.5 ns |
| 128 | default | 0.856 ns | 0.932 ns | **0.812 ns** | 0.875 ns | 0.813 ns | 0.859 ns | 61.4 ns |
| 128 | native | 0.828 ns | 0.879 ns | 0.830 ns | 0.848 ns | 0.819 ns | **0.818 ns** | 36.2 ns |
| 256 | default | 1.34 ns | – | **1.28 ns** | 1.30 ns | 1.32 ns | 1.59 ns | 38.3 ns |
| 256 | native | 1.28 ns | – | **1.25 ns** | 1.30 ns | 1.29 ns | 1.66 ns | 37.8 ns |
| 384 | default | 1.91 ns | – | **1.90 ns** | 1.98 ns | 2.11 ns | 2.87 ns | 38.8 ns |
| 384 | native | 2.01 ns | – | **1.99 ns** | 2.21 ns | 2.01 ns | 2.89 ns | 53.7 ns |
| 512 | default | **2.46 ns** | – | 2.74 ns | 3.11 ns | 2.70 ns | 6.41 ns | 60.4 ns |
| 512 | native | 3.57 ns | – | **2.71 ns** | 3.25 ns | 2.82 ns | 5.73 ns | 54.2 ns |
| 1024 | default | 7.45 ns | – | 7.66 ns | 10.9 ns | **7.04 ns** | 12.9 ns | 76.5 ns |
| 1024 | native | **6.31 ns** | – | 7.47 ns | 10.6 ns | 7.84 ns | 13.7 ns | 41.4 ns |
| 2048 | default | **14.9 ns** | – | 36.3 ns | 24.7 ns | 18.3 ns | 29.3 ns | 69.7 ns |
| 2048 | native | **15.3 ns** | – | 19.2 ns | 21.7 ns | 19.0 ns | 30.2 ns | 57.4 ns |
| 4096 | default | **51.2 ns** | – | 72.5 ns | 55.2 ns | 57.9 ns | 60.6 ns | 79.6 ns |
| 4096 | native | 58.6 ns | – | 57.9 ns | **53.9 ns** | 55.5 ns | 65.4 ns | 66.8 ns |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 0.648 ns | 0.730 ns | 0.626 ns | 0.601 ns | **0.574 ns** | 0.585 ns | 22.3 ns |
| 64 | native | 0.570 ns | 0.887 ns | 0.765 ns | 0.764 ns | 0.679 ns | **0.562 ns** | 20.4 ns |
| 128 | default | 1.07 ns | 0.996 ns | **0.854 ns** | 0.918 ns | 1.01 ns | 0.881 ns | 39.9 ns |
| 128 | native | 0.986 ns | 0.956 ns | 1.10 ns | 0.912 ns | **0.827 ns** | 0.943 ns | 39.8 ns |
| 256 | default | 4.84 ns | – | **3.95 ns** | 4.05 ns | 4.11 ns | 4.51 ns | 58.4 ns |
| 256 | native | 3.73 ns | – | 3.78 ns | 3.69 ns | **3.52 ns** | 3.94 ns | 53.7 ns |
| 384 | default | 11.6 ns | – | **9.33 ns** | 10.2 ns | 9.66 ns | 13.7 ns | 87.5 ns |
| 384 | native | 9.19 ns | – | **8.37 ns** | 9.20 ns | 9.07 ns | 11.1 ns | 79.6 ns |
| 512 | default | 18.3 ns | – | 20.7 ns | **16.0 ns** | 17.8 ns | 25.1 ns | 124 ns |
| 512 | native | 19.0 ns | – | 18.0 ns | **15.7 ns** | 17.6 ns | 25.4 ns | 144 ns |
| 1024 | default | 127 ns | – | 139 ns | 107 ns | **98.2 ns** | 115 ns | 311 ns |
| 1024 | native | 132 ns | – | 122 ns | 108 ns | **98.3 ns** | 103 ns | 294 ns |
| 2048 | default | 446 ns | – | 474 ns | 476 ns | **341 ns** | 512 ns | 922 ns |
| 2048 | native | 478 ns | – | 420 ns | 484 ns | **371 ns** | 528 ns | 975 ns |
| 4096 | default | 1,500 ns | – | 1,689 ns | 1,906 ns | **1,392 ns** | 2,779 ns | 3,149 ns |
| 4096 | native | 1,608 ns | – | 1,668 ns | 1,829 ns | **1,424 ns** | 2,759 ns | 3,024 ns |

### div: a / b

![div: a / b](benchmarks/div.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 1.74 ns | 1.63 ns | **1.60 ns** | 62.4 ns | 1.78 ns | 3.05 ns | 14.0 ns |
| 64 | native | 1.61 ns | **1.58 ns** | 1.60 ns | 55.5 ns | 1.64 ns | 1.59 ns | 13.8 ns |
| 128 | default | **3.61 ns** | 3.67 ns | 6.92 ns | 67.5 ns | 19.9 ns | 8.49 ns | 29.8 ns |
| 128 | native | 3.72 ns | **3.60 ns** | 6.04 ns | 66.9 ns | 9.20 ns | 7.58 ns | 29.8 ns |
| 256 | default | 23.3 ns | – | **19.3 ns** | 163 ns | 76.9 ns | 47.2 ns | 72.6 ns |
| 256 | native | 23.3 ns | – | **14.1 ns** | 134 ns | 48.0 ns | 48.2 ns | 76.6 ns |
| 384 | default | 47.0 ns | – | **44.4 ns** | 163 ns | 81.9 ns | 76.5 ns | 81.1 ns |
| 384 | native | **50.5 ns** | – | 50.7 ns | 168 ns | 79.4 ns | 73.0 ns | 93.2 ns |
| 512 | default | **63.1 ns** | – | 70.5 ns | 200 ns | 113 ns | 106 ns | 106 ns |
| 512 | native | **62.8 ns** | – | 63.1 ns | 205 ns | 101 ns | 105 ns | 98.7 ns |
| 1024 | default | **145 ns** | – | 156 ns | 402 ns | 297 ns | 307 ns | 201 ns |
| 1024 | native | **146 ns** | – | 150 ns | 418 ns | 275 ns | 230 ns | 189 ns |
| 2048 | default | **406 ns** | – | 445 ns | 987 ns | 1,022 ns | 966 ns | 572 ns |
| 2048 | native | **395 ns** | – | 413 ns | 1,008 ns | 874 ns | 808 ns | 521 ns |
| 4096 | default | **1,578 ns** | – | 1,580 ns | 3,067 ns | 3,439 ns | 3,917 ns | 1,642 ns |
| 4096 | native | 1,264 ns | – | **1,199 ns** | 3,353 ns | 3,169 ns | 3,193 ns | 1,571 ns |

### mod: a mod b

![mod: a mod b](benchmarks/mod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 1.72 ns | **1.62 ns** | 1.62 ns | 60.3 ns | 1.65 ns | 2.81 ns | 2.77 ns |
| 64 | native | 1.66 ns | **1.59 ns** | 1.59 ns | 56.2 ns | 1.64 ns | 1.60 ns | 2.69 ns |
| 128 | default | 3.80 ns | **3.70 ns** | 6.82 ns | 67.1 ns | 22.5 ns | 9.22 ns | 33.3 ns |
| 128 | native | 3.88 ns | **3.74 ns** | 6.38 ns | 68.9 ns | 7.96 ns | 8.18 ns | 29.9 ns |
| 256 | default | 24.6 ns | – | **21.2 ns** | 127 ns | 61.2 ns | 47.6 ns | 66.6 ns |
| 256 | native | 23.8 ns | – | **15.6 ns** | 130 ns | 44.8 ns | 47.3 ns | 75.8 ns |
| 384 | default | **63.4 ns** | – | 67.8 ns | 210 ns | 106 ns | 78.6 ns | 81.4 ns |
| 384 | native | **45.9 ns** | – | 47.5 ns | 173 ns | 76.7 ns | 79.3 ns | 87.5 ns |
| 512 | default | 60.8 ns | – | **59.3 ns** | 204 ns | 117 ns | 96.5 ns | 104 ns |
| 512 | native | **62.3 ns** | – | 63.3 ns | 203 ns | 99.2 ns | 102 ns | 95.6 ns |
| 1024 | default | **138 ns** | – | 146 ns | 375 ns | 285 ns | 262 ns | 186 ns |
| 1024 | native | 147 ns | – | **143 ns** | 396 ns | 284 ns | 220 ns | 191 ns |
| 2048 | default | **397 ns** | – | 418 ns | 974 ns | 970 ns | 986 ns | 541 ns |
| 2048 | native | 403 ns | – | **388 ns** | 1,003 ns | 863 ns | 832 ns | 500 ns |
| 4096 | default | **1,284 ns** | – | 1,440 ns | 3,164 ns | 3,390 ns | 3,199 ns | 1,598 ns |
| 4096 | native | 1,250 ns | – | **1,203 ns** | 3,293 ns | 3,277 ns | 3,247 ns | 1,568 ns |

### mulmod: a × b mod m

![mulmod: a × b mod m](benchmarks/mulmod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 3.16 ns | **2.84 ns** | 4.58 ns | 61.5 ns | 4.71 ns | 10.5 ns | 37.4 ns |
| 64 | native | **2.96 ns** | 3.01 ns | 5.42 ns | 60.8 ns | 4.82 ns | 9.95 ns | 45.6 ns |
| 128 | default | 20.5 ns | – | **15.1 ns** | 114 ns | 55.9 ns | 44.9 ns | 98.1 ns |
| 128 | native | 20.3 ns | – | **15.0 ns** | 118 ns | 54.0 ns | 51.4 ns | 117 ns |
| 256 | default | **43.9 ns** | – | 45.0 ns | 209 ns | 99.7 ns | 106 ns | 145 ns |
| 256 | native | 49.6 ns | – | **42.5 ns** | 214 ns | 113 ns | 100 ns | 165 ns |
| 384 | default | **82.8 ns** | – | 88.9 ns | 311 ns | 194 ns | 154 ns | 199 ns |
| 384 | native | **89.3 ns** | – | 95.7 ns | 343 ns | 198 ns | 162 ns | 237 ns |
| 512 | default | **131 ns** | – | 133 ns | 417 ns | 280 ns | 285 ns | 329 ns |
| 512 | native | 151 ns | – | **134 ns** | 455 ns | 323 ns | 276 ns | 380 ns |
| 1024 | default | 482 ns | – | **410 ns** | 1,268 ns | 1,299 ns | 1,303 ns | 821 ns |
| 1024 | native | 492 ns | – | **416 ns** | 1,331 ns | 1,064 ns | 1,356 ns | 965 ns |
| 2048 | default | **1,602 ns** | – | 2,040 ns | 3,993 ns | 4,165 ns | 7,353 ns | 2,394 ns |
| 2048 | native | **1,934 ns** | – | 2,187 ns | 4,914 ns | 4,501 ns | 5,537 ns | 2,623 ns |
| 4096 | default | **6,499 ns** | – | 8,509 ns | 15,914 ns | 15,807 ns | – | 8,215 ns |
| 4096 | native | **6,244 ns** | – | 7,501 ns | 14,780 ns | 13,765 ns | – | 8,750 ns |
