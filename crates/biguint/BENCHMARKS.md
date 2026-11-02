# Benchmarks

`biguint` at commit `1069a8e` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod, mulmod, add_mod and sub_mod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits, in cycles per operation, measured on 2026-11-02.

The CPU is an AMD Ryzen 9 5950X 16-Core Processor, measured on its CPU 15.
Its counters give each figure per operation as the median, over 31 repetitions, of a run of 2n operations on fresh operands less a run of n, over n.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1` |
| Extensions | adx, aes, avx, avx2, bmi1, bmi2, f16c, fma, lzcnt, movbe, pclmulqdq, popcnt, sha, sse, sse2, sse3, sse4.1, sse4.2, sse4a, ssse3, vaes, vpclmulqdq |
| Counters | one Linux perf_event group: user-mode cycles and branch misses |
| Load | 1-minute load average 19.91 before the sweep and 17.7 after; CPU 31, the SMT sibling of CPU 15, busy 6% of the sweep |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

A sweep measures every case in 31 repetitions; repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine reaches all cases alike. Each run draws fresh operand sets from the case's distribution, outside the counted region: no operand set runs twice, so the branch predictor never learns a sequence. A repetition runs n operations uncounted, then counts a run of n operations and a run of 2n. Before the repetitions, n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as many, takes 1,048,576 cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.

One perf_event group on the measuring thread, pinned to one CPU, counts user-mode cycles and branch misses; a run during which the kernel shares the counters with other groups stops the sweep. A figure per operation is the median, over the 31 repetitions, of the long run less the short run, divided by n: the fixed cost of a run cancels, as in nanoBench's differencing, and each difference pairs two runs taken back to back. A case whose long runs count no more cycles than its short runs stops the sweep, and so does a chain step under one cycle.

Throughput runs independent operations, with operands and results behind `core::hint::black_box`. Latency runs the chain `x = op(x, y)`: from the second operand set on, each result is the first operand of the next operation, so each operation waits for the one before. Each result passes through an empty inline-assembly block, which keeps the compiler from regrouping the chain. The block holds a result of up to 64 bytes in general-purpose registers in one chain and in vector registers in another, each chain a case of its own in the sweep; it can only add moves between the two, so the latency is the chain with fewer cycles. A larger result passes through memory, in a single chain. The chain applies where the result has the type of the first operand and the other operands are plain values: a step copies them, and the copy of an allocated value would allocate.

The sweep ran at a 1-minute load of 2 or more, so its cycles compare within each table only.

Before a case is measured, its results on 256 operand sets equal `num-bigint`'s, reduced modulo 2^N for the crates that wrap. The functions that a case calls, such as the allocator of `num-bigint`, are part of its counts.

The chains are those of add, of the native mul, whose hardware multiply takes the same time on all operands, and of the modular functions that return the type of their operands and reduce any operand: each step of such a chain has its own modulus, below which the result of the step before may not lie. A chain of products modulo 2^N gathers trailing zeros and a chain of quotients falls to zero, so the other cases of mul, div and mod have no chain; `crypto-bigint`'s add_mod and sub_mod need operands below the modulus, `biguint`'s mulmod returns an `Option`, and a step of `num-bigint` would copy allocated operands.

| Operation | Operands at width N |
|---|---|
| add, mul | `a` and `b` uniform below 2^N |
| div, mod | a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held in an N-bit integer |
| mulmod, add_mod, sub_mod | a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m` |

## Functions

Each cell measures the crate's own function:

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---|---|---|---|---|---|---|
| add | `+` | `wrapping_add` at 64 and 128 bits | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` at 64 bits | – |
| mul | `*` | `wrapping_mul` at 64 and 128 bits | `wrapping_mul` at 64, 128, 256, 384, 512, 2048 and 4096 bits | `wrapping_mul` at 64, 128, 256, 384, 512 and 1024 bits | `wrapping_mul` at 64, 128, 256, 384, 512 and 1024 bits | `overflowing_mul` at 64, 128, 256, 384 and 1024 bits | `&a * &b` at 2048 and 4096 bits |
| div | `div_rem` | `/ NonZero` at 64 and 128 bits | `wrapping_div` | `wrapping_div_vartime` | `wrapping_div` | `/` | `&a / &b` |
| mod | `div_rem` | `% NonZero` at 64 and 128 bits | `wrapping_rem` | `rem_vartime` | `wrapping_rem` | `%` | `&a % &b` |
| mulmod | `mul_mod` | – | `mul_mod` | `mul_mod_vartime` | – | – | – |
| add_mod | `add_mod` | – | `add_mod` | – | – | – | `(&a + &b) % &m` |
| sub_mod | `sub_mod` | – | `a.add_mod(m - b.reduce_mod(m), m)` | – | – | – | `(&a + &m - &b) % &m` |

- Each row compares functions of one timing class, the class of `biguint`'s function: add, and mul up to 1024 bits, take a time that the width fixes; mul from 2048 bits, div, mod, mulmod, add_mod and sub_mod take a time that depends on the values.
- `native` is `u64` at 64 bits and `u128` at 128 bits.
- `crypto-bigint` is constant-time; its div, mod and mulmod cases run its `*_vartime` functions.
- `num-bigint` allocates each result.
- `uint` is `construct_uint!`, as in `primitive-types`.
- No primitive of this width: `native` at 256, 384, 512, 1024, 2048 and 4096 bits.
- No modular product: `native` mulmod at 64 and 128 bits, `bnum` mulmod, `uint` mulmod and `num-bigint` mulmod.
- No modular sum or difference: `native` add_mod and sub_mod at 64 and 128 bits, `bnum` add_mod and sub_mod and `uint` add_mod and sub_mod.
- Variable time in a fixed-time row: `ruint` mul at 1024 bits, `uint` add at 128, 256, 384, 512, 1024, 2048 and 4096 bits and mul at 512 bits and `num-bigint` add and mul at 64, 128, 256, 384, 512 and 1024 bits.
- Fixed time in a variable-time row: `crypto-bigint` mul at 2048 and 4096 bits, add_mod and sub_mod, `bnum` mul at 2048 and 4096 bits and `uint` mul at 2048 and 4096 bits.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.3** | 2.2 / 1.3 | 2.2 / 1.3 | **2.2** / 2.0 | 2.2 / 1.3 | 2.2 / 1.3 | – |
| 128 | 3.3 / **2.0** | 3.2 / 2.0 | 3.3 / 2.0 | **3.2** / 2.0 | 3.3 / 2.1 | – | – |
| 256 | 5.7 / 3.3 | – | **5.6** / **3.3** | 6.0 / 5.3 | 5.7 / 3.3 | – | – |
| 384 | 8.3 / 4.1 | – | **8.0** / **4.1** | 9.6 / 10.5 | 8.3 / 5.4 | – | – |
| 512 | 10.9 / 5.7 | – | **10.2** / **5.7** | 12.4 / 16.5 | 11.7 / 7.9 | – | – |
| 1024 | 25.9 / 40.2 | – | **23.1** / 40.1 | 29.1 / **39.7** | 29.3 / 39.7 | – | – |
| 2048 | **59.4** / **75.8** | – | 83.2 / 103 | 79.6 / 115 | 96.6 / 104 | – | – |
| 4096 | **163** / **220** | – | 173 / 238 | 204 / 323 | 211 / 315 | – | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.3 / – | 2.2 / **3.0** | 2.3 / – | **2.2** / – | 2.2 / – | 2.3 / – | – |
| 128 | 3.8 / – | 3.7 / **5.0** | **3.3** / – | 3.3 / – | 3.6 / – | 3.3 / – | – |
| 256 | 13.0 / – | – | **12.7** / – | 13.0 / – | 16.0 / – | 16.0 / – | – |
| 384 | **29.3** / – | – | 30.9 / – | 30.5 / – | 33.0 / – | 37.9 / – | – |
| 512 | **54.9** / – | – | 59.6 / – | 96.5 / – | 77.8 / – | – | – |
| 1024 | **228** / – | – | – | 390 / – | 393 / – | 438 / – | – |
| 2048 | **1,166** / – | – | 1,611 / – | – | – | – | 3,495 / – |
| 4096 | **4,740** / – | – | 6,655 / – | – | – | – | 11,339 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 7.0 / – | **7.0** / – | 7.0 / – | 229 / – | 7.0 / – | 7.0 / – | 39.5 / – |
| 128 | 17.3 / – | **16.8** / – | 32.5 / – | 272 / – | 31.2 / – | 30.6 / – | 113 / – |
| 256 | 87.0 / – | – | **82.3** / – | 544 / – | 246 / – | 202 / – | 290 / – |
| 384 | **164** / – | – | 199 / – | 707 / – | 363 / – | 314 / – | 354 / – |
| 512 | **221** / – | – | 257 / – | 844 / – | 492 / – | 397 / – | 420 / – |
| 1024 | **514** / – | – | 593 / – | 1,636 / – | 1,131 / – | 991 / – | 816 / – |
| 2048 | **1,447** / – | – | 1,564 / – | 4,119 / – | 3,403 / – | 3,307 / – | 2,091 / – |
| 4096 | **4,605** / – | – | 4,924 / – | 12,952 / – | 12,618 / – | 12,561 / – | 6,441 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | 7.0 / – | 7.0 / – | 229 / – | 7.0 / – | 7.0 / – | 8.0 / – |
| 128 | 18.8 / – | **18.1** / – | 31.2 / – | 273 / – | 33.1 / – | 31.1 / – | 115 / – |
| 256 | 83.9 / – | – | **83.8** / – | 544 / – | 258 / – | 197 / – | 303 / – |
| 384 | **175** / – | – | 191 / – | 704 / – | 370 / – | 331 / – | 359 / – |
| 512 | **226** / – | – | 230 / – | 845 / – | 523 / – | 393 / – | 423 / – |
| 1024 | **501** / – | – | 573 / – | 1,634 / – | 1,150 / – | 988 / – | 819 / – |
| 2048 | **1,424** / – | – | 1,538 / – | 4,110 / – | 3,404 / – | 3,287 / – | 2,094 / – |
| 4096 | **4,582** / – | – | 4,921 / – | 12,982 / – | 12,713 / – | 12,629 / – | 6,463 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **12.0** / – | – | 23.5 / **46.6** | 254 / 254 | – | – | – |
| 128 | **66.5** / – | – | 86.4 / **88.0** | 500 / 510 | – | – | – |
| 256 | 186 / – | – | **171** / **208** | 1,001 / 1,001 | – | – | – |
| 384 | **354** / – | – | 384 / **400** | 1,469 / 1,480 | – | – | – |
| 512 | 573 / – | – | **526** / **548** | 2,000 / 2,007 | – | – | – |
| 1024 | 1,795 / – | – | **1,552** / **1,564** | 5,312 / 5,351 | – | – | – |
| 2048 | **5,946** / – | – | 7,429 / **7,451** | 16,412 / 16,511 | – | – | – |
| 4096 | **19,903** / – | – | 29,612 / **29,630** | 57,739 / 58,315 | – | – | – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 15.5 / 15.5 | – | **13.0** / **14.0** | – | – | – | 114 / – |
| 128 | **18.3** / **21.4** | – | 38.9 / 24.5 | – | – | – | 274 / – |
| 256 | **23.5** / **29.5** | – | 46.6 / 59.5 | – | – | – | 266 / – |
| 384 | **34.9** / **44.8** | – | 62.8 / 74.6 | – | – | – | 279 / – |
| 512 | **40.1** / **46.8** | – | 81.2 / 93.1 | – | – | – | 284 / – |
| 1024 | **83.6** / **115** | – | 159 / 193 | – | – | – | 341 / – |
| 2048 | **215** / **226** | – | 381 / 461 | – | – | – | 462 / – |
| 4096 | **328** / **392** | – | 731 / 818 | – | – | – | 839 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **4.0** / **6.1** | – | 14.0 / 14.0 | – | – | – | 179 / – |
| 128 | **16.8** / **20.5** | – | 33.1 / 22.9 | – | – | – | 365 / – |
| 256 | **21.7** / **28.3** | – | 50.1 / 64.4 | – | – | – | 354 / – |
| 384 | **33.3** / **44.6** | – | 69.4 / 83.2 | – | – | – | 369 / – |
| 512 | **41.7** / **51.6** | – | 87.3 / 103 | – | – | – | 339 / – |
| 1024 | **64.0** / **89.2** | – | 193 / 222 | – | – | – | 500 / – |
| 2048 | **164** / **328** | – | 557 / 493 | – | – | – | 609 / – |
| 4096 | **310** / **376** | – | 1,035 / 1,110 | – | – | – | 1,090 / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```
