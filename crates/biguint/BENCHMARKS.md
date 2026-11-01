# Benchmarks

`biguint` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod and mulmod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits.

The numbers measure `biguint` at commit `e98092a`, the last commit that changed `crates/biguint/src`. The bench ran at commit `d39aa74` on 2026-11-01. These commands regenerate the results:

```sh
cargo run -p biguint-bench-report -- bench    # criterion, for both builds
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```

## Summary

Each cell: `biguint`'s time per operation over the fastest other crate's. Below 1, `biguint` is faster.

| Operation | Build | 64 bits | 128 bits | 256 bits | 384 bits | 512 bits | 1024 bits | 2048 bits | 4096 bits |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| add | default | 1.02 | 1.01 | 1.00 | 0.99 | 1.00 | 3.13 | 4.25 | 1.77 |
| add | native | 1.00 | 1.07 | 0.97 | 1.01 | 1.03 | 0.88 | 2.55 | 2.04 |
| mul | default | 1.00 | 1.01 | 1.14 | 1.09 | 0.93 | 1.63 | 1.60 | 1.76 |
| mul | native | 1.01 | 1.18 | 1.07 | 0.96 | 0.89 | 1.31 | 1.26 | 1.27 |
| div | default | 4.80 | 3.10 | 1.38 | 1.27 | 1.12 | 1.19 | 1.09 | 1.03 |
| div | native | 4.15 | 2.92 | 1.66 | 1.18 | 1.04 | 1.05 | 1.10 | 1.08 |
| mod | default | 4.81 | 3.04 | 1.35 | 1.34 | 1.26 | 1.20 | 1.11 | 1.04 |
| mod | native | 4.09 | 2.82 | 1.42 | 1.31 | 1.13 | 1.06 | 1.13 | 1.14 |
| mulmod | default | 3.43 | 1.77 | 1.20 | 0.99 | 1.39 | 1.68 | 1.48 | 1.48 |
| mulmod | native | 3.45 | 1.31 | 1.21 | 0.95 | 1.00 | 1.62 | 1.40 | 1.32 |

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor; `znver3` (the CPU that `-C target-cpu=native` selects); 1 logical CPU available to the run |
| Host | `x86_64-unknown-linux-gnu` |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Builds | `default`: no `RUSTFLAGS`, the baseline of the host target. `native`: `RUSTFLAGS="-C target-cpu=native"`. |
| Timing | criterion 0.8.2 with `--quick`: no warm-up; the iterations of a case double until two consecutive runs agree within 5%, from 100 ms to 3 s |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

**The times are provisional.** The 1-minute load average stayed between 14.2 and 21.1 during the runs, which shared the machine with other builds and ran on one logical CPU each. Each time is an upper bound, and the times compare only within one table.

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
| 64 | default | 0.465 ns | **0.456 ns** | 0.473 ns | 0.476 ns | 0.467 ns | 0.467 ns | 10.8 ns |
| 64 | native | 0.785 ns | 0.801 ns | 0.797 ns | 0.788 ns | 0.793 ns | **0.781 ns** | 20.7 ns |
| 128 | default | 0.689 ns | **0.685 ns** | 0.689 ns | 0.686 ns | 0.704 ns | 0.688 ns | 45.4 ns |
| 128 | native | 1.05 ns | **0.981 ns** | 1.11 ns | 1.11 ns | 1.00 ns | 1.15 ns | 56.3 ns |
| 256 | default | 1.15 ns | – | **1.14 ns** | 1.15 ns | 1.18 ns | 1.41 ns | 29.8 ns |
| 256 | native | **1.63 ns** | – | 1.68 ns | 1.68 ns | 1.69 ns | 2.50 ns | 56.4 ns |
| 384 | default | **1.66 ns** | – | 1.67 ns | 1.73 ns | 1.79 ns | 2.53 ns | 29.6 ns |
| 384 | native | 2.52 ns | – | **2.49 ns** | 2.86 ns | 2.79 ns | 4.23 ns | 55.4 ns |
| 512 | default | **2.14 ns** | – | 2.15 ns | 2.69 ns | 2.30 ns | 4.56 ns | 46.4 ns |
| 512 | native | 3.34 ns | – | **3.24 ns** | 4.32 ns | 3.82 ns | 8.95 ns | 77.8 ns |
| 1024 | default | 18.9 ns | – | 6.73 ns | 9.41 ns | **6.06 ns** | 10.9 ns | 50.1 ns |
| 1024 | native | **7.87 ns** | – | 8.97 ns | 12.9 ns | 9.88 ns | 19.1 ns | 61.9 ns |
| 2048 | default | 68.8 ns | – | 37.3 ns | 19.9 ns | **16.2 ns** | 24.1 ns | 57.9 ns |
| 2048 | native | 59.8 ns | – | **23.5 ns** | 25.4 ns | 24.3 ns | 23.5 ns | 45.1 ns |
| 4096 | default | 81.7 ns | – | 67.0 ns | **46.2 ns** | 47.0 ns | 49.3 ns | 65.9 ns |
| 4096 | native | 78.0 ns | – | **38.2 ns** | 45.9 ns | 48.0 ns | 49.0 ns | 55.6 ns |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 0.532 ns | **0.529 ns** | 0.530 ns | 0.530 ns | 0.533 ns | 0.532 ns | 16.9 ns |
| 64 | native | 0.472 ns | 0.474 ns | 0.470 ns | 0.473 ns | 0.472 ns | **0.469 ns** | 16.5 ns |
| 128 | default | 0.739 ns | 0.761 ns | 0.735 ns | 0.741 ns | 0.787 ns | **0.732 ns** | 30.6 ns |
| 128 | native | 0.845 ns | 0.727 ns | 0.751 ns | 0.837 ns | **0.717 ns** | 0.757 ns | 29.7 ns |
| 256 | default | 3.40 ns | – | **2.98 ns** | 3.17 ns | 3.18 ns | 3.84 ns | 43.8 ns |
| 256 | native | 3.18 ns | – | **2.96 ns** | 3.39 ns | 3.39 ns | 3.61 ns | 42.2 ns |
| 384 | default | 7.91 ns | – | 7.48 ns | **7.25 ns** | 7.61 ns | 10.1 ns | 62.0 ns |
| 384 | native | **7.04 ns** | – | 7.36 ns | 8.51 ns | 7.83 ns | 9.08 ns | 61.6 ns |
| 512 | default | **13.0 ns** | – | 17.5 ns | 14.0 ns | 16.0 ns | 22.7 ns | 104 ns |
| 512 | native | **12.2 ns** | – | 16.3 ns | 13.6 ns | 16.0 ns | 22.0 ns | 111 ns |
| 1024 | default | 143 ns | – | 119 ns | 94.0 ns | **87.5 ns** | 101 ns | 260 ns |
| 1024 | native | 118 ns | – | 105 ns | 91.7 ns | **89.8 ns** | 95.5 ns | 267 ns |
| 2048 | default | 523 ns | – | 389 ns | 424 ns | **326 ns** | 499 ns | 854 ns |
| 2048 | native | 424 ns | – | 369 ns | 419 ns | **337 ns** | 486 ns | 829 ns |
| 4096 | default | 2,257 ns | – | 1,494 ns | 1,719 ns | **1,282 ns** | 2,641 ns | 2,689 ns |
| 4096 | native | 1,689 ns | – | 1,493 ns | 1,684 ns | **1,331 ns** | 2,585 ns | 2,603 ns |

### div: a / b

![div: a / b](benchmarks/div.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 7.42 ns | **1.55 ns** | 1.55 ns | 51.1 ns | 1.55 ns | 2.52 ns | 12.9 ns |
| 64 | native | 6.51 ns | 1.57 ns | 1.57 ns | 52.3 ns | 1.57 ns | **1.57 ns** | 13.1 ns |
| 128 | default | 10.9 ns | **3.52 ns** | 6.19 ns | 61.1 ns | 19.0 ns | 7.34 ns | 25.6 ns |
| 128 | native | 10.4 ns | **3.56 ns** | 5.87 ns | 62.4 ns | 8.57 ns | 6.78 ns | 25.0 ns |
| 256 | default | 22.8 ns | – | **16.6 ns** | 121 ns | 52.9 ns | 43.7 ns | 62.6 ns |
| 256 | native | 20.5 ns | – | **12.4 ns** | 122 ns | 42.4 ns | 44.0 ns | 61.9 ns |
| 384 | default | 51.1 ns | – | **40.3 ns** | 151 ns | 73.2 ns | 66.5 ns | 73.2 ns |
| 384 | native | 51.0 ns | – | **43.1 ns** | 158 ns | 68.0 ns | 68.0 ns | 73.0 ns |
| 512 | default | 68.7 ns | – | **61.6 ns** | 187 ns | 95.6 ns | 91.8 ns | 86.6 ns |
| 512 | native | 60.2 ns | – | **58.0 ns** | 185 ns | 90.2 ns | 86.2 ns | 83.2 ns |
| 1024 | default | 164 ns | – | **138 ns** | 358 ns | 263 ns | 247 ns | 174 ns |
| 1024 | native | 142 ns | – | **135 ns** | 366 ns | 247 ns | 200 ns | 162 ns |
| 2048 | default | 455 ns | – | **418 ns** | 887 ns | 921 ns | 851 ns | 477 ns |
| 2048 | native | 393 ns | – | **358 ns** | 913 ns | 796 ns | 713 ns | 458 ns |
| 4096 | default | 1,375 ns | – | **1,333 ns** | 2,748 ns | 3,051 ns | 3,043 ns | 1,407 ns |
| 4096 | native | 1,206 ns | – | **1,118 ns** | 2,850 ns | 2,952 ns | 2,826 ns | 1,426 ns |

### mod: a mod b

![mod: a mod b](benchmarks/mod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 7.45 ns | **1.55 ns** | 1.55 ns | 51.5 ns | 1.55 ns | 2.46 ns | 2.26 ns |
| 64 | native | 6.41 ns | **1.57 ns** | 1.57 ns | 51.9 ns | 1.57 ns | 1.57 ns | 2.11 ns |
| 128 | default | 10.9 ns | **3.58 ns** | 6.16 ns | 60.9 ns | 21.4 ns | 7.22 ns | 24.6 ns |
| 128 | native | 10.2 ns | **3.63 ns** | 5.86 ns | 62.5 ns | 7.25 ns | 6.98 ns | 25.5 ns |
| 256 | default | 22.9 ns | – | **16.9 ns** | 120 ns | 56.4 ns | 44.8 ns | 61.8 ns |
| 256 | native | 20.5 ns | – | **14.5 ns** | 122 ns | 41.3 ns | 42.5 ns | 63.8 ns |
| 384 | default | 48.5 ns | – | **36.1 ns** | 151 ns | 76.0 ns | 68.3 ns | 70.3 ns |
| 384 | native | 51.9 ns | – | **39.7 ns** | 158 ns | 69.3 ns | 72.7 ns | 77.2 ns |
| 512 | default | 68.9 ns | – | **54.6 ns** | 181 ns | 99.5 ns | 90.6 ns | 85.1 ns |
| 512 | native | 60.4 ns | – | **53.4 ns** | 186 ns | 91.4 ns | 84.9 ns | 81.7 ns |
| 1024 | default | 162 ns | – | **135 ns** | 352 ns | 254 ns | 240 ns | 172 ns |
| 1024 | native | 141 ns | – | **134 ns** | 396 ns | 243 ns | 212 ns | 162 ns |
| 2048 | default | 448 ns | – | **403 ns** | 874 ns | 906 ns | 856 ns | 477 ns |
| 2048 | native | 396 ns | – | **351 ns** | 898 ns | 791 ns | 950 ns | 609 ns |
| 4096 | default | 1,378 ns | – | **1,323 ns** | 2,746 ns | 3,051 ns | 3,077 ns | 1,408 ns |
| 4096 | native | 1,432 ns | – | **1,258 ns** | 3,483 ns | 3,376 ns | 3,915 ns | 1,422 ns |

### mulmod: a × b mod m

![mulmod: a × b mod m](benchmarks/mulmod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 9.11 ns | **2.65 ns** | 4.28 ns | 57.4 ns | 4.37 ns | 8.75 ns | 36.1 ns |
| 64 | native | 9.32 ns | **2.70 ns** | 4.40 ns | 55.5 ns | 4.39 ns | 8.83 ns | 35.9 ns |
| 128 | default | 24.7 ns | – | **14.0 ns** | 113 ns | 53.7 ns | 45.7 ns | 96.2 ns |
| 128 | native | 17.1 ns | – | **13.1 ns** | 114 ns | 51.1 ns | 48.3 ns | 99.5 ns |
| 256 | default | 50.8 ns | – | **42.3 ns** | 205 ns | 99.3 ns | 104 ns | 139 ns |
| 256 | native | 46.7 ns | – | **38.5 ns** | 206 ns | 100 ns | 93.5 ns | 134 ns |
| 384 | default | **87.2 ns** | – | 88.1 ns | 308 ns | 187 ns | 156 ns | 201 ns |
| 384 | native | **82.4 ns** | – | 86.7 ns | 302 ns | 175 ns | 141 ns | 202 ns |
| 512 | default | 188 ns | – | **135 ns** | 421 ns | 280 ns | 288 ns | 302 ns |
| 512 | native | **124 ns** | – | 125 ns | 430 ns | 273 ns | 241 ns | 297 ns |
| 1024 | default | 685 ns | – | **407 ns** | 1,166 ns | 1,096 ns | 1,131 ns | 739 ns |
| 1024 | native | 606 ns | – | **375 ns** | 1,167 ns | 934 ns | 969 ns | 719 ns |
| 2048 | default | 2,907 ns | – | **1,965 ns** | 3,842 ns | 3,820 ns | 4,893 ns | 2,286 ns |
| 2048 | native | 2,319 ns | – | **1,661 ns** | 3,670 ns | 3,615 ns | 4,855 ns | 2,263 ns |
| 4096 | default | 11,301 ns | – | 7,839 ns | 13,563 ns | 13,557 ns | – | **7,627 ns** |
| 4096 | native | 8,823 ns | – | **6,662 ns** | 12,972 ns | 12,873 ns | – | 7,883 ns |
