# Benchmarks

`biguint` at commit `1069a8e` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod, mulmod, add_mod and sub_mod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits, in cycles and instructions per operation, measured on 2026-11-02.

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
| Counters | one Linux perf_event group: user-mode cycles, instructions and branch misses |
| Load | 1-minute load average 19.91 before the sweep and 17.7 after; CPU 31, the SMT sibling of CPU 15, busy 6% of the sweep |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

A sweep measures every case in 31 repetitions; repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine reaches all cases alike. Each run draws fresh operand sets from the case's distribution, outside the counted region: no operand set runs twice, so the branch predictor never learns a sequence. A repetition runs n operations uncounted, then counts a run of n operations and a run of 2n. Before the repetitions, n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as many, takes 1,048,576 cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.

One perf_event group on the measuring thread, pinned to one CPU, counts user-mode cycles, instructions and branch misses; a run during which the kernel shares the counters with other groups stops the sweep. A figure per operation is the median, over the 31 repetitions, of the long run less the short run, divided by n: the fixed cost of a run cancels, as in nanoBench's differencing, and each difference pairs two runs taken back to back. A case whose long runs count no more cycles than its short runs stops the sweep, and so does a chain step under one cycle, the latency of a single instruction.

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

Per operation, each row gives throughput cycles, latency cycles for the operations that have chains, and instructions. Bold marks the fewest in each row.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | 2.2 | 2.2 | 2.2 | **2.2** | 2.2 | 2.2 | – |
|  | latency cycles | **1.3** | 1.3 | 1.3 | 2.0 | 1.3 | 1.3 | – |
|  | instructions | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | – |
| 128 | throughput cycles | 3.3 | 3.2 | 3.3 | **3.2** | 3.3 | – | – |
|  | latency cycles | **2.0** | 2.0 | 2.0 | 2.0 | 2.1 | – | – |
|  | instructions | **11.0** | **11.0** | **11.0** | **11.0** | **11.0** | – | – |
| 256 | throughput cycles | 5.7 | – | **5.6** | 6.0 | 5.7 | – | – |
|  | latency cycles | 3.3 | – | **3.3** | 5.3 | 3.3 | – | – |
|  | instructions | **17.0** | – | **17.0** | 22.0 | 19.0 | – | – |
| 384 | throughput cycles | 8.3 | – | **8.0** | 9.6 | 8.3 | – | – |
|  | latency cycles | 4.1 | – | **4.1** | 10.5 | 5.4 | – | – |
|  | instructions | **23.0** | – | **23.0** | 32.0 | 31.0 | – | – |
| 512 | throughput cycles | 10.9 | – | **10.2** | 12.4 | 11.7 | – | – |
|  | latency cycles | 5.7 | – | **5.7** | 16.5 | 7.9 | – | – |
|  | instructions | **29.0** | – | **29.0** | 42.0 | 43.0 | – | – |
| 1024 | throughput cycles | 25.9 | – | **23.1** | 29.1 | 29.3 | – | – |
|  | latency cycles | 40.2 | – | 40.1 | **39.7** | 39.7 | – | – |
|  | instructions | 70.0 | – | **63.0** | 92.0 | 103 | – | – |
| 2048 | throughput cycles | **59.4** | – | 83.2 | 79.6 | 96.6 | – | – |
|  | latency cycles | **75.8** | – | 103 | 115 | 104 | – | – |
|  | instructions | **143** | – | 164 | 208 | 268 | – | – |
| 4096 | throughput cycles | **163** | – | 173 | 204 | 211 | – | – |
|  | latency cycles | **220** | – | 238 | 323 | 315 | – | – |
|  | instructions | 484 | – | 468 | **459** | 596 | – | – |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | 2.3 | 2.2 | 2.3 | **2.2** | 2.2 | 2.3 | – |
|  | latency cycles | – | **3.0** | – | – | – | – | – |
|  | instructions | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | – |
| 128 | throughput cycles | 3.8 | 3.7 | **3.3** | 3.3 | 3.6 | 3.3 | – |
|  | latency cycles | – | **5.0** | – | – | – | – | – |
|  | instructions | **14.0** | 16.0 | **14.0** | **14.0** | 16.0 | **14.0** | – |
| 256 | throughput cycles | 13.0 | – | **12.7** | 13.0 | 16.0 | 16.0 | – |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | 46.0 | – | **44.0** | 46.0 | 51.0 | 57.0 | – |
| 384 | throughput cycles | **29.3** | – | 30.9 | 30.5 | 33.0 | 37.9 | – |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | 103 | – | **94.0** | 114 | 128 | 145 | – |
| 512 | throughput cycles | **54.9** | – | 59.6 | 96.5 | 77.8 | – | – |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | 192 | – | **172** | 346 | 273 | – | – |
| 1024 | throughput cycles | **228** | – | – | 390 | 393 | 438 | – |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | **795** | – | – | 1,370 | 1,259 | 1,449 | – |
| 2048 | throughput cycles | **1,166** | – | 1,611 | – | – | – | 3,495 |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | **3,565** | – | 4,390 | – | – | – | 9,748 |
| 4096 | throughput cycles | **4,740** | – | 6,655 | – | – | – | 11,339 |
|  | latency cycles | – | – | – | – | – | – | – |
|  | instructions | **11,678** | – | 14,900 | – | – | – | 31,564 |

### div: a / b

![div: a / b](benchmarks/div.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | 7.0 | **7.0** | 7.0 | 229 | 7.0 | 7.0 | 39.5 |
|  | instructions | **14.0** | **14.0** | 16.0 | 451 | 23.0 | 21.0 | 143 |
| 128 | throughput cycles | 17.3 | **16.8** | 32.5 | 272 | 31.2 | 30.6 | 113 |
|  | instructions | **26.0** | 26.0 | 58.5 | 518 | 92.0 | 91.0 | 341 |
| 256 | throughput cycles | 87.0 | – | **82.3** | 544 | 246 | 202 | 290 |
|  | instructions | **171** | – | 200 | 1,312 | 545 | 564 | 1,073 |
| 384 | throughput cycles | **164** | – | 199 | 707 | 363 | 314 | 354 |
|  | instructions | **497** | – | 592 | 1,789 | 776 | 785 | 1,243 |
| 512 | throughput cycles | **221** | – | 257 | 844 | 492 | 397 | 420 |
|  | instructions | **637** | – | 770 | 2,262 | 1,202 | 1,060 | 1,372 |
| 1024 | throughput cycles | **514** | – | 593 | 1,636 | 1,131 | 991 | 816 |
|  | instructions | **1,347** | – | 1,653 | 4,908 | 3,149 | 2,377 | 2,277 |
| 2048 | throughput cycles | **1,447** | – | 1,564 | 4,119 | 3,403 | 3,307 | 2,091 |
|  | instructions | **3,431** | – | 4,154 | 13,654 | 7,981 | 6,598 | 5,220 |
| 4096 | throughput cycles | **4,605** | – | 4,924 | 12,952 | 12,618 | 12,561 | 6,441 |
|  | instructions | **10,661** | – | 12,286 | 45,588 | 29,295 | 27,377 | 15,030 |

### mod: a mod b

![mod: a mod b](benchmarks/mod.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | **7.0** | 7.0 | 7.0 | 229 | 7.0 | 7.0 | 8.0 |
|  | instructions | **14.0** | **14.0** | 16.0 | 451 | 23.0 | 21.0 | 44.0 |
| 128 | throughput cycles | 18.8 | **18.1** | 31.2 | 273 | 33.1 | 31.1 | 115 |
|  | instructions | 29.0 | **29.0** | 58.5 | 518 | 93.0 | 91.0 | 362 |
| 256 | throughput cycles | 83.9 | – | **83.8** | 544 | 258 | 197 | 303 |
|  | instructions | **155** | – | 202 | 1,312 | 546 | 564 | 1,094 |
| 384 | throughput cycles | **175** | – | 191 | 704 | 370 | 331 | 359 |
|  | instructions | **477** | – | 572 | 1,789 | 776 | 789 | 1,263 |
| 512 | throughput cycles | **226** | – | 230 | 845 | 523 | 393 | 423 |
|  | instructions | **603** | – | 747 | 2,262 | 1,202 | 1,060 | 1,393 |
| 1024 | throughput cycles | **501** | – | 573 | 1,634 | 1,150 | 988 | 819 |
|  | instructions | **1,268** | – | 1,596 | 4,908 | 3,150 | 2,376 | 2,298 |
| 2048 | throughput cycles | **1,424** | – | 1,538 | 4,110 | 3,404 | 3,287 | 2,094 |
|  | instructions | **3,101** | – | 4,102 | 13,654 | 7,980 | 6,603 | 5,242 |
| 4096 | throughput cycles | **4,582** | – | 4,921 | 12,982 | 12,713 | 12,629 | 6,463 |
|  | instructions | **9,452** | – | 12,254 | 45,588 | 29,292 | 27,452 | 15,050 |

### mulmod: a × b mod m

![mulmod: a × b mod m](benchmarks/mulmod.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | **12.0** | – | 23.5 | 254 | – | – | – |
|  | latency cycles | – | – | **46.6** | 254 | – | – | – |
|  | instructions | **30.0** | – | 84.0 | 496 | – | – | – |
| 128 | throughput cycles | **66.5** | – | 86.4 | 500 | – | – | – |
|  | latency cycles | – | – | **88.0** | 510 | – | – | – |
|  | instructions | **149** | – | 192 | 935 | – | – | – |
| 256 | throughput cycles | 186 | – | **171** | 1,001 | – | – | – |
|  | latency cycles | – | – | **208** | 1,001 | – | – | – |
|  | instructions | 507 | – | **439** | 2,297 | – | – | – |
| 384 | throughput cycles | **354** | – | 384 | 1,469 | – | – | – |
|  | latency cycles | – | – | **400** | 1,480 | – | – | – |
|  | instructions | 885 | – | **829** | 3,783 | – | – | – |
| 512 | throughput cycles | 573 | – | **526** | 2,000 | – | – | – |
|  | latency cycles | – | – | **548** | 2,007 | – | – | – |
|  | instructions | 1,359 | – | **1,250** | 5,741 | – | – | – |
| 1024 | throughput cycles | 1,795 | – | **1,552** | 5,312 | – | – | – |
|  | latency cycles | – | – | **1,564** | 5,351 | – | – | – |
|  | instructions | **3,847** | – | 3,930 | 18,246 | – | – | – |
| 2048 | throughput cycles | **5,946** | – | 7,429 | 16,412 | – | – | – |
|  | latency cycles | – | – | **7,451** | 16,511 | – | – | – |
|  | instructions | **12,717** | – | 16,343 | 63,447 | – | – | – |
| 4096 | throughput cycles | **19,903** | – | 29,612 | 57,739 | – | – | – |
|  | latency cycles | – | – | **29,630** | 58,315 | – | – | – |
|  | instructions | **39,367** | – | 63,913 | 242,293 | – | – | – |

### add_mod: a + b mod m

![add_mod: a + b mod m](benchmarks/add_mod.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | 15.5 | – | **13.0** | – | – | – | 114 |
|  | latency cycles | 15.5 | – | **14.0** | – | – | – | – |
|  | instructions | **19.4** | – | 42.0 | – | – | – | 360 |
| 128 | throughput cycles | **18.3** | – | 38.9 | – | – | – | 274 |
|  | latency cycles | **21.4** | – | 24.5 | – | – | – | – |
|  | instructions | **31.3** | – | 56.0 | – | – | – | 984 |
| 256 | throughput cycles | **23.5** | – | 46.6 | – | – | – | 266 |
|  | latency cycles | **29.5** | – | 59.5 | – | – | – | – |
|  | instructions | **43.5** | – | 120 | – | – | – | 1,037 |
| 384 | throughput cycles | **34.9** | – | 62.8 | – | – | – | 279 |
|  | latency cycles | **44.8** | – | 74.6 | – | – | – | – |
|  | instructions | **65.3** | – | 171 | – | – | – | 1,081 |
| 512 | throughput cycles | **40.1** | – | 81.2 | – | – | – | 284 |
|  | latency cycles | **46.8** | – | 93.1 | – | – | – | – |
|  | instructions | **96.1** | – | 217 | – | – | – | 1,053 |
| 1024 | throughput cycles | **83.6** | – | 159 | – | – | – | 341 |
|  | latency cycles | **115** | – | 193 | – | – | – | – |
|  | instructions | **203** | – | 395 | – | – | – | 1,165 |
| 2048 | throughput cycles | **215** | – | 381 | – | – | – | 462 |
|  | latency cycles | **226** | – | 461 | – | – | – | – |
|  | instructions | **424** | – | 697 | – | – | – | 1,415 |
| 4096 | throughput cycles | **328** | – | 731 | – | – | – | 839 |
|  | latency cycles | **392** | – | 818 | – | – | – | – |
|  | instructions | **814** | – | 2,021 | – | – | – | 1,887 |

### sub_mod: a − b mod m

![sub_mod: a − b mod m](benchmarks/sub_mod.svg)

| Bits | Per operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | throughput cycles | **4.0** | – | 14.0 | – | – | – | 179 |
|  | latency cycles | **6.1** | – | 14.0 | – | – | – | – |
|  | instructions | **19.0** | – | 47.0 | – | – | – | 523 |
| 128 | throughput cycles | **16.8** | – | 33.1 | – | – | – | 365 |
|  | latency cycles | **20.5** | – | 22.9 | – | – | – | – |
|  | instructions | **30.5** | – | 64.0 | – | – | – | 1,231 |
| 256 | throughput cycles | **21.7** | – | 50.1 | – | – | – | 354 |
|  | latency cycles | **28.3** | – | 64.4 | – | – | – | – |
|  | instructions | **48.5** | – | 145 | – | – | – | 1,315 |
| 384 | throughput cycles | **33.3** | – | 69.4 | – | – | – | 369 |
|  | latency cycles | **44.6** | – | 83.2 | – | – | – | – |
|  | instructions | **65.5** | – | 208 | – | – | – | 1,369 |
| 512 | throughput cycles | **41.7** | – | 87.3 | – | – | – | 339 |
|  | latency cycles | **51.6** | – | 103 | – | – | – | – |
|  | instructions | **88.5** | – | 265 | – | – | – | 1,226 |
| 1024 | throughput cycles | **64.0** | – | 193 | – | – | – | 500 |
|  | latency cycles | **89.2** | – | 222 | – | – | – | – |
|  | instructions | **189** | – | 545 | – | – | – | 1,621 |
| 2048 | throughput cycles | **164** | – | 557 | – | – | – | 609 |
|  | latency cycles | **328** | – | 493 | – | – | – | – |
|  | instructions | **451** | – | 976 | – | – | – | 1,840 |
| 4096 | throughput cycles | **310** | – | 1,035 | – | – | – | 1,090 |
|  | latency cycles | **376** | – | 1,110 | – | – | – | – |
|  | instructions | **892** | – | 2,680 | – | – | – | 2,407 |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```
