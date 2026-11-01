# Benchmarks

`biguint` at commit `1d394e1` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod and mulmod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits, measured on 2026-11-01.

## Summary

Each cell: `biguint`'s time per operation over the fastest other crate's. Below 1, `biguint` is faster.

| Operation | Build | 64 bits | 128 bits | 256 bits | 384 bits | 512 bits | 1024 bits | 2048 bits | 4096 bits |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| add | default | 1.04 | 1.03 | 1.00 | 1.00 | 1.00 | 1.14 | 0.84 | 0.90 |
| add | native | 1.02 | 1.06 | 1.00 | 0.98 | 0.98 | 0.88 | 0.98 | 0.87 |
| mul | default | 1.03 | 1.24 | 1.23 | 1.28 | 1.09 | 1.31 | 1.12 | 1.06 |
| mul | native | 1.11 | 1.16 | 1.16 | 1.24 | 1.34 | 1.13 | 1.14 | 1.20 |
| div | default | 1.01 | 1.01 | 1.38 | 1.04 | 1.07 | 0.92 | 0.90 | 0.88 |
| div | native | 1.02 | 1.00 | 2.08 | 0.94 | 1.08 | 0.99 | 1.03 | 1.09 |
| mod | default | 1.01 | 1.01 | 1.35 | 1.13 | 1.01 | 0.94 | 0.89 | 1.00 |
| mod | native | 1.03 | 0.99 | 1.22 | 1.38 | 0.95 | 0.99 | 1.00 | 1.04 |
| mulmod | default | 0.62 | 1.33 | 1.03 | 0.82 | 1.06 | 1.08 | 0.82 | 0.77 |
| mulmod | native | 0.61 | 1.35 | 1.29 | 0.93 | 1.06 | 1.25 | 1.18 | 0.88 |

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Builds | `default`: the baseline of the host target. `native`: `-C target-cpu=native`. |
| Timing | criterion 0.8.2 `--quick` |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

**The times are provisional.** Load average 19 to 27 (median 25) on 32 hardware threads during the run; each build pinned to one CPU. Each time is an upper bound, and the times compare only within one table.

## Method

Each case times one crate's function over 256 operand sets, the same for every crate in a row, with operands and results behind `core::hint::black_box`. The time per operation is criterion's median for the batch divided by 256. Every result equals `num-bigint`'s, reduced modulo 2^N for the crates that wrap.

| Operation | Operands at width N |
|---|---|
| add, mul | `a` and `b` uniform below 2^N |
| div, mod | a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held in an N-bit integer |
| mulmod | a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m` |

## Functions

Each cell times the crate's own function:

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---|---|---|---|---|---|---|
| add | `+` | `wrapping_add` at 64 and 128 bits | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` | `&a + &b` |
| mul | `*` | `wrapping_mul` at 64 and 128 bits | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `overflowing_mul` | `&a * &b` |
| div | `div_rem` | `/ NonZero` at 64 and 128 bits | `wrapping_div` | `wrapping_div_vartime` | `wrapping_div` | `/` | `&a / &b` |
| mod | `div_rem` | `% NonZero` at 64 and 128 bits | `wrapping_rem` | `rem_vartime` | `wrapping_rem` | `%` | `&a % &b` |
| mulmod | `mul_mod` | – | `mul_mod` | `mul_mod_vartime` | – | – | – |

- `native` is `u64` at 64 bits and `u128` at 128 bits.
- `crypto-bigint` is constant-time; its div, mod and mulmod cases run its `*_vartime` functions.
- `num-bigint` allocates each result.
- `uint` is `construct_uint!`, as in `primitive-types`.
- No primitive of this width: `native` at 256, 384, 512, 1024, 2048 and 4096 bits.
- No modular product: `native` mulmod at 64 and 128 bits, `bnum` mulmod, `uint` mulmod and `num-bigint` mulmod.

## Results

Each cell: nanoseconds per operation. Bold marks the fastest time in each row.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 0.480 ns | **0.461 ns** | 0.484 ns | 0.591 ns | 0.480 ns | 0.476 ns | 10.6 ns |
| 64 | native | 0.496 ns | 0.490 ns | **0.485 ns** | 0.501 ns | 0.515 ns | 0.490 ns | 12.9 ns |
| 128 | default | 0.710 ns | 0.734 ns | 0.750 ns | 0.696 ns | **0.689 ns** | 0.694 ns | 46.7 ns |
| 128 | native | 0.760 ns | **0.716 ns** | 1.01 ns | 1.01 ns | 0.742 ns | 0.732 ns | 30.7 ns |
| 256 | default | **1.14 ns** | – | 1.14 ns | 1.22 ns | 1.24 ns | 1.33 ns | 30.9 ns |
| 256 | native | **1.18 ns** | – | 1.19 ns | 1.20 ns | 1.18 ns | 1.38 ns | 32.4 ns |
| 384 | default | **1.68 ns** | – | 1.68 ns | 1.91 ns | 1.82 ns | 2.57 ns | 29.8 ns |
| 384 | native | **1.75 ns** | – | 1.81 ns | 1.79 ns | 1.90 ns | 2.34 ns | 31.9 ns |
| 512 | default | **2.21 ns** | – | 2.22 ns | 2.79 ns | 2.48 ns | 4.63 ns | 45.9 ns |
| 512 | native | **2.29 ns** | – | 2.34 ns | 2.83 ns | 2.39 ns | 5.03 ns | 51.8 ns |
| 1024 | default | 6.92 ns | – | 6.99 ns | 9.45 ns | **6.08 ns** | 11.2 ns | 47.7 ns |
| 1024 | native | **7.09 ns** | – | 8.08 ns | 11.6 ns | 8.92 ns | 16.6 ns | 51.4 ns |
| 2048 | default | **13.5 ns** | – | 28.1 ns | 19.8 ns | 16.1 ns | 24.2 ns | 53.3 ns |
| 2048 | native | **16.4 ns** | – | 28.5 ns | 28.2 ns | 16.6 ns | 23.3 ns | 47.5 ns |
| 4096 | default | **43.9 ns** | – | 67.5 ns | 53.8 ns | 48.8 ns | 51.7 ns | 63.7 ns |
| 4096 | native | **35.7 ns** | – | 40.9 ns | 47.7 ns | 48.0 ns | 51.9 ns | 81.8 ns |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 0.550 ns | 0.564 ns | 0.538 ns | **0.536 ns** | 0.550 ns | 0.546 ns | 17.3 ns |
| 64 | native | 0.530 ns | 0.554 ns | **0.475 ns** | 0.482 ns | 0.490 ns | 0.513 ns | 18.1 ns |
| 128 | default | 0.914 ns | 0.786 ns | 0.780 ns | **0.737 ns** | 0.770 ns | 0.759 ns | 30.4 ns |
| 128 | native | 0.888 ns | **0.763 ns** | 0.800 ns | 0.769 ns | 1.12 ns | 0.922 ns | 35.1 ns |
| 256 | default | 4.06 ns | – | 3.43 ns | **3.31 ns** | 3.44 ns | 4.10 ns | 49.4 ns |
| 256 | native | 3.68 ns | – | **3.16 ns** | 4.10 ns | 3.97 ns | 4.94 ns | 47.4 ns |
| 384 | default | 9.53 ns | – | 7.71 ns | **7.46 ns** | 8.19 ns | 14.2 ns | 59.7 ns |
| 384 | native | 11.1 ns | – | 9.90 ns | 9.15 ns | **8.93 ns** | 12.1 ns | 92.7 ns |
| 512 | default | 15.2 ns | – | 17.4 ns | **14.0 ns** | 17.2 ns | 25.1 ns | 144 ns |
| 512 | native | 21.8 ns | – | 21.4 ns | **16.3 ns** | 17.2 ns | 24.0 ns | 136 ns |
| 1024 | default | 114 ns | – | 123 ns | 99.2 ns | **87.6 ns** | 113 ns | 260 ns |
| 1024 | native | 118 ns | – | 153 ns | 114 ns | 114 ns | **104 ns** | 269 ns |
| 2048 | default | 419 ns | – | 403 ns | 472 ns | **373 ns** | 569 ns | 918 ns |
| 2048 | native | 417 ns | – | 386 ns | 432 ns | **365 ns** | 492 ns | 1,115 ns |
| 4096 | default | 1,370 ns | – | 1,618 ns | 1,748 ns | **1,296 ns** | 2,674 ns | 2,756 ns |
| 4096 | native | 1,774 ns | – | 1,579 ns | 1,866 ns | **1,475 ns** | 2,647 ns | 2,715 ns |

### div: a / b

![div: a / b](benchmarks/div.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 1.59 ns | 1.60 ns | **1.57 ns** | 52.7 ns | 1.57 ns | 2.52 ns | 12.9 ns |
| 64 | native | 1.62 ns | **1.59 ns** | 1.61 ns | 54.8 ns | 1.66 ns | 1.69 ns | 13.4 ns |
| 128 | default | 3.61 ns | **3.57 ns** | 6.30 ns | 63.3 ns | 19.6 ns | 7.35 ns | 29.6 ns |
| 128 | native | **3.65 ns** | 3.66 ns | 6.57 ns | 74.9 ns | 15.3 ns | 8.60 ns | 35.0 ns |
| 256 | default | 23.2 ns | – | **16.8 ns** | 123 ns | 52.5 ns | 44.0 ns | 63.2 ns |
| 256 | native | 27.5 ns | – | **13.2 ns** | 131 ns | 55.9 ns | 54.3 ns | 88.9 ns |
| 384 | default | 43.6 ns | – | **41.9 ns** | 156 ns | 72.2 ns | 68.1 ns | 113 ns |
| 384 | native | **53.7 ns** | – | 57.4 ns | 185 ns | 90.9 ns | 100 ns | 114 ns |
| 512 | default | 68.0 ns | – | **63.7 ns** | 189 ns | 101 ns | 93.2 ns | 89.9 ns |
| 512 | native | 66.4 ns | – | **61.3 ns** | 192 ns | 95.9 ns | 93.2 ns | 81.2 ns |
| 1024 | default | **132 ns** | – | 144 ns | 359 ns | 259 ns | 255 ns | 186 ns |
| 1024 | native | **138 ns** | – | 140 ns | 380 ns | 260 ns | 234 ns | 257 ns |
| 2048 | default | **371 ns** | – | 414 ns | 988 ns | 988 ns | 890 ns | 551 ns |
| 2048 | native | 514 ns | – | **497 ns** | 1,376 ns | 1,158 ns | 1,107 ns | 527 ns |
| 4096 | default | **1,212 ns** | – | 1,372 ns | 2,854 ns | 3,283 ns | 3,142 ns | 1,546 ns |
| 4096 | native | 1,280 ns | – | **1,177 ns** | 3,158 ns | 3,733 ns | 3,437 ns | 1,644 ns |

### mod: a mod b

![mod: a mod b](benchmarks/mod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | 1.59 ns | 1.59 ns | **1.58 ns** | 52.2 ns | 1.58 ns | 2.62 ns | 2.30 ns |
| 64 | native | 1.66 ns | 1.64 ns | **1.61 ns** | 53.1 ns | 1.62 ns | 1.62 ns | 2.51 ns |
| 128 | default | 3.75 ns | **3.70 ns** | 6.38 ns | 63.6 ns | 22.0 ns | 7.57 ns | 26.7 ns |
| 128 | native | **3.77 ns** | 3.82 ns | 6.65 ns | 70.9 ns | 9.38 ns | 7.36 ns | 25.9 ns |
| 256 | default | 23.0 ns | – | **17.1 ns** | 120 ns | 56.3 ns | 45.8 ns | 65.7 ns |
| 256 | native | 22.7 ns | – | **18.6 ns** | 137 ns | 52.4 ns | 54.8 ns | 109 ns |
| 384 | default | 43.6 ns | – | **38.5 ns** | 159 ns | 77.5 ns | 72.1 ns | 77.3 ns |
| 384 | native | 58.3 ns | – | **42.3 ns** | 166 ns | 89.5 ns | 96.7 ns | 128 ns |
| 512 | default | 57.0 ns | – | **56.7 ns** | 186 ns | 106 ns | 93.9 ns | 88.1 ns |
| 512 | native | **76.1 ns** | – | 79.8 ns | 192 ns | 92.5 ns | 88.3 ns | 81.9 ns |
| 1024 | default | **131 ns** | – | 140 ns | 368 ns | 273 ns | 258 ns | 185 ns |
| 1024 | native | **138 ns** | – | 139 ns | 401 ns | 244 ns | 208 ns | 193 ns |
| 2048 | default | **377 ns** | – | 424 ns | 942 ns | 930 ns | 897 ns | 521 ns |
| 2048 | native | 385 ns | – | **384 ns** | 1,009 ns | 789 ns | 725 ns | 480 ns |
| 4096 | default | **1,374 ns** | – | 1,380 ns | 3,022 ns | 3,491 ns | 3,508 ns | 1,562 ns |
| 4096 | native | 1,232 ns | – | **1,186 ns** | 3,175 ns | 3,180 ns | 2,857 ns | 1,482 ns |

### mulmod: a × b mod m

![mulmod: a × b mod m](benchmarks/mulmod.svg)

| Bits | Build | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | default | **2.74 ns** | – | 4.40 ns | 58.8 ns | – | – | – |
| 64 | native | **2.76 ns** | – | 4.50 ns | 57.4 ns | – | – | – |
| 128 | default | 20.3 ns | – | **15.2 ns** | 115 ns | – | – | – |
| 128 | native | 19.0 ns | – | **14.1 ns** | 117 ns | – | – | – |
| 256 | default | 45.9 ns | – | **44.5 ns** | 210 ns | – | – | – |
| 256 | native | 52.0 ns | – | **40.2 ns** | 219 ns | – | – | – |
| 384 | default | **87.1 ns** | – | 106 ns | 367 ns | – | – | – |
| 384 | native | **99.1 ns** | – | 107 ns | 361 ns | – | – | – |
| 512 | default | 159 ns | – | **150 ns** | 497 ns | – | – | – |
| 512 | native | 140 ns | – | **132 ns** | 439 ns | – | – | – |
| 1024 | default | 534 ns | – | **495 ns** | 1,182 ns | – | – | – |
| 1024 | native | 476 ns | – | **380 ns** | 1,715 ns | – | – | – |
| 2048 | default | **1,660 ns** | – | 2,022 ns | 3,912 ns | – | – | – |
| 2048 | native | 2,159 ns | – | **1,830 ns** | 4,015 ns | – | – | – |
| 4096 | default | **6,145 ns** | – | 7,947 ns | 14,005 ns | – | – | – |
| 4096 | native | **5,916 ns** | – | 6,757 ns | 13,138 ns | – | – | – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # criterion, default and native builds
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```
