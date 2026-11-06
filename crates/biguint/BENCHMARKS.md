# Benchmarks

`biguint` at commit `a0a67b8` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 15.83 before, 18.13 after; its SMT sibling, CPU 29, 35% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands at width N: uniform below 2^N; a divisor of N/2 bits; a modulus of N bits with operands below it.
- Every result matches `num-bigint` before it is measured.
- A row compares the functions of `biguint`'s timing class: fixed for add and for mul up to 1024 bits, variable for the rest.
- A constant-time table compares `ConstantTime<Uint>` with every function whose time the width fixes; divisors and moduli are public.
- add_mod and sub_mod take operands of any value. A `*` there marks a function with an unchecked precondition, operands below the modulus, which gives a wrong result for other inputs: those cells compare against an unchecked precondition.
- The rows of reduced operands take the one modulus of the width. `biguint` runs `Residue` and `ConstantTimeResidue` of a ring that `Modulus::scope` prepares outside the measured loop: the type holds each operand below the modulus, with no check.

## Functions

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---|---|---|---|---|---|---|
| add | `+` | `wrapping_add` | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` | – |
| mul | `*` | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `overflowing_mul` | `&a * &b` |
| div | `div_rem` | `/ NonZero` | `wrapping_div` | `wrapping_div_vartime` | `wrapping_div` | `/` | `&a / &b` |
| mod | `div_rem` | `% NonZero` | `wrapping_rem` | `rem_vartime` | `wrapping_rem` | `%` | `&a % &b` |
| mulmod | `mul_mod` | `u128 product % m` | `mul_mod` | `mul_mod_vartime` | – | – | `(&a * &b) % &m` |
| add_mod | `add_mod` | `overflowing_add, then - m` | `add_mod` | `add_mod` | `overflowing_add, then - m` | `overflowing_add, then - m` | `(&a + &b) % &m` |
| sub_mod | `sub_mod` | `-, else wrapping_sub then + m` | `a.add_mod(m - b.reduce_mod(m), m)` | `sub_mod` | `-, else overflowing_sub then + m` | `-, else overflowing_sub then + m` | `(&a + &m - &b) % &m` |
| add_mod, reduced operands | `Residue +` | `overflowing_add, then - m` | `add_mod` | `add_mod` | `overflowing_add, then - m` | `overflowing_add, then - m` | `(&a + &b) % &m` |
| sub_mod, reduced operands | `Residue -` | `-, else wrapping_sub then + m` | `a.add_mod(m - b, m)` | `sub_mod` | `-, else overflowing_sub then + m` | `-, else overflowing_sub then + m` | `(&a + &m - &b) % &m` |

- `native`: add, mul, div, mod, add_mod, sub_mod, add_mod, reduced operands and sub_mod, reduced operands at 64 and 128 bits; mulmod at 64 bits.
- `ruint`: mul at 64, 128, 256, 384, 512, 2048 and 4096 bits.
- `uint`: add at 64 bits; mul at 64, 128, 256, 384, 1024, 2048 and 4096 bits.
- `num-bigint`: mul at 2048 and 4096 bits.

## Functions, constant time

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---|---|---|---|---|---|
| add | `ConstantTime +` | `wrapping_add` | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` |
| mul | `ConstantTime *` | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `overflowing_mul` |
| div | `ConstantTime::div_rem` | – | – | `div_rem` | – | – |
| mod | `ConstantTime::div_rem` | – | – | `rem` | – | – |
| mulmod | `ConstantTime::mul_mod` | – | – | `mul_mod` | – | – |
| add_mod | `ConstantTime::add_mod` | – | – | `add_mod` | – | – |
| sub_mod | `ConstantTime::sub_mod` | – | – | `sub_mod` | – | – |
| add_mod, reduced operands | `ConstantTimeResidue +` | – | – | `add_mod` | – | – |
| sub_mod, reduced operands | `ConstantTimeResidue -` | – | – | `sub_mod` | – | – |

- `native`: at 64 and 128 bits.
- `ruint`: mul at 64, 128, 256, 384 and 512 bits.
- `uint`: add at 64 bits; mul at 64, 128, 256, 384, 1024, 2048 and 4096 bits.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / 1.4 | **2.4** / 1.4 | 2.5 / **1.3** | 2.5 / **1.3** | 2.7 / 1.4 | – |
| 256 | **4.7** / **2.5** | – | 4.9 / **2.5** | 5.6 / 5.5 | 5.0 / 3.1 | – |
| 384 | **7.1** / **3.5** | – | 7.3 / 3.9 | 8.9 / 10.9 | 8.0 / 5.1 | – |
| 512 | **9.2** / **4.0** | – | 9.5 / 4.5 | 12.3 / 16.7 | 11.5 / 7.8 | – |
| 1024 | **23.3** / **22.1** | – | 28.1 / 22.6 | 29.0 / 28.3 | 27.8 / 28.8 | – |
| 2048 | **53.5** / **51.4** | – | 83.9 / 68.2 | 80.8 / 100 | 97.7 / 106 | – |
| 4096 | **158** / **163** | – | 166 / 192 | 200 / 278 | 208 / 267 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | **1.4** / – | **1.4** / – | **1.4** / – | 1.5 / – | – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | **12.2** / – | – | 12.4 / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | **27.7** / – | – | 30.4 / – | 29.7 / – | 33.6 / – | 36.5 / – | – |
| 512 | **52.1** / – | – | 59.6 / – | 81.7 / – | 86.5 / – | – | – |
| 1024 | **221** / – | – | – | 433 / – | 387 / – | 436 / – | – |
| 2048 | **1,113** / – | – | 1,604 / – | 1,958 / – | 1,536 / – | 4,633 / – | 3,577 / – |
| 4096 | **3,859** / – | – | 6,444 / – | 7,742 / – | 6,116 / – | 24,948 / – | 11,164 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | 7.0 / – | **6.9** / – | 220 / – | **6.9** / – | 7.0 / – | 37.1 / – |
| 128 | 16.3 / – | **16.1** / – | 31.5 / – | 263 / – | 29.9 / – | 30.0 / – | 101 / – |
| 256 | **46.0** / – | – | 79.0 / – | 543 / – | 242 / – | 202 / – | 269 / – |
| 384 | **111** / – | – | 194 / – | 714 / – | 359 / – | 309 / – | 333 / – |
| 512 | **155** / – | – | 265 / – | 836 / – | 452 / – | 395 / – | 398 / – |
| 1024 | **462** / – | – | 587 / – | 1,624 / – | 1,093 / – | 980 / – | 795 / – |
| 2048 | **1,388** / – | – | 1,529 / – | 4,092 / – | 3,398 / – | 3,284 / – | 2,076 / – |
| 4096 | **4,594** / – | – | 4,835 / – | 12,821 / – | 11,731 / – | 12,352 / – | 6,369 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | 7.0 / – | 7.0 / – | 220 / – | 7.0 / – | 7.0 / – | 7.9 / – |
| 128 | **17.0** / – | 17.1 / – | 30.3 / – | 267 / – | 31.0 / – | 31.0 / – | 102 / – |
| 256 | **41.5** / – | – | 79.9 / – | 541 / – | 252 / – | 197 / – | 272 / – |
| 384 | **119** / – | – | 191 / – | 712 / – | 361 / – | 326 / – | 334 / – |
| 512 | **158** / – | – | 244 / – | 830 / – | 472 / – | 390 / – | 398 / – |
| 1024 | **485** / – | – | 559 / – | 1,593 / – | 1,111 / – | 970 / – | 798 / – |
| 2048 | **1,407** / – | – | 1,529 / – | 4,078 / – | 3,408 / – | 3,286 / – | 2,074 / – |
| 4096 | **4,562** / – | – | 4,879 / – | 12,830 / – | 11,654 / – | 12,422 / – | 6,363 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **11.9** / **23.0** | **11.9** / **23.0** | 20.2 / 44.1 | 227 / 222 | 147 / – |
| 128 | **47.4** / **50.7** | – | 81.4 / 90.6 | 496 / 500 | 368 / – |
| 256 | **162** / **183** | – | 166 / 204 | 986 / 1,006 | 564 / – |
| 384 | **297** / **316** | – | 376 / 390 | 1,461 / 1,496 | 857 / – |
| 512 | **467** / **483** | – | 540 / 557 | 1,954 / 1,978 | 1,171 / – |
| 1024 | **1,365** / **1,337** | – | 1,580 / 1,591 | 5,135 / 5,254 | 3,162 / – |
| 2048 | **5,191** / **5,198** | – | 7,381 / 7,431 | 16,031 / 16,079 | 9,919 / – |
| 4096 | **14,910** / **14,977** | – | 29,537 / 29,526 | 57,317 / 57,661 | 33,844 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.9** / **8.0** | 2.9\* / – | 11.9 / 13.4 | 11.1 / – | 17.3 / – | 2.9 / – | 99.4 / – |
| 128 | **4.8** / **11.9** | 6.0 / – | 34.2 / 21.2 | 27.2 / – | 16.4 / – | 24.9 / – | 234 / – |
| 256 | **9.7** / **18.5** | – | 43.4 / 60.7 | 70.3 / – | 23.1 / – | 55.0 / – | 228 / – |
| 384 | **15.6** / **22.9** | – | 56.6 / 79.2 | 90.9 / – | 31.8 / – | 88.2 / – | 236 / – |
| 512 | **27.7** / **33.5** | – | 89.8 / 92.7 | 122 / – | 40.2 / – | 128 / – | 287 / – |
| 1024 | **71.7** / **79.2** | – | 153 / 167 | 143 / – | 68.7\* / – | 272 / – | 339 / – |
| 2048 | **149** / **155** | – | 330 / 369 | 429 / – | 156 / – | 575 / – | 387 / – |
| 4096 | **268** / **333** | – | 672 / 798 | 891 / – | 372 / – | 1,469 / – | 741 / – |

\* `native` at 64 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

\* `bnum` at 1024 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.4** / **7.3** | 12.0 / – | 11.9 / 13.4 | 4.6 / – | 13.0 / – | 12.1 / – | 160 / – |
| 128 | **4.3** / **11.3** | 3.7\* / – | 31.4 / 22.5 | 11.5 / – | 13.2 / – | 18.1 / – | 319 / – |
| 256 | **8.7** / **14.4** | – | 48.3 / 61.7 | 49.3 / – | 21.9 / – | 48.0 / – | 317 / – |
| 384 | **15.6** / **23.1** | – | 68.8 / 87.2 | 82.0 / – | 28.8 / – | 80.6 / – | 331 / – |
| 512 | **25.2** / **37.4** | – | 89.5 / 109 | 111 / – | 39.6 / – | 127 / – | 414 / – |
| 1024 | **66.3** / **79.9** | – | 175 / 200 | 129 / – | 67.7 / – | 286 / – | 459 / – |
| 2048 | **146** / **144** | – | 452 / 454 | 323 / – | 157 / – | 656 / – | 494 / – |
| 4096 | **241** / **320** | – | 957 / 1,088 | 592 / – | 400 / – | 1,466 / – | 1,031 / – |

\* `native` at 128 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

### add_mod, reduced operands: a + b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.8** / **4.9** | 2.9 / – | 11.9 / 13.6 | 11.2 / – | 17.1 / – | 2.9 / – | 107 / – |
| 128 | **4.3** / **6.0** | 6.0 / – | 33.8 / 12.9 | 27.3 / – | 15.8 / – | 24.9 / – | 217 / – |
| 256 | **10.4** / **12.9** | – | 44.2 / 43.9 | 70.3 / – | 23.0 / – | 50.9 / – | 213 / – |
| 384 | **13.8** / **14.6** | – | 60.1 / 62.1 | 90.8 / – | 31.9 / – | 79.6 / – | 226 / – |
| 512 | **19.6** / **20.2** | – | 88.3 / 72.4 | 122 / – | 39.1 / – | 115 / – | 333 / – |
| 1024 | **48.3** / **44.9** | – | 154 / 150 | 144 / – | 72.3 / – | 251 / – | 347 / – |
| 2048 | **89.7** / **83.0** | – | 329 / 353 | 429 / – | 161 / – | 526 / – | 455 / – |
| 4096 | **227** / **240** | – | 671 / 777 | 893 / – | 357 / – | 1,233 / – | 535 / – |

### sub_mod, reduced operands: a − b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **3.9** | 12.0 / – | 11.9 / 13.6 | 4.6 / – | 13.2 / – | 12.3 / – | 171 / – |
| 128 | 4.7 / **4.8** | **3.7** / – | 31.6 / 12.9 | 11.4 / – | 13.4 / – | 17.7 / – | 298 / – |
| 256 | **7.3** / **7.4** | – | 46.0 / 44.9 | 49.2 / – | 21.7 / – | 44.2 / – | 267 / – |
| 384 | **11.0** / **10.1** | – | 70.1 / 67.6 | 82.0 / – | 29.0 / – | 75.5 / – | 298 / – |
| 512 | **18.3** / **16.1** | – | 78.6 / 79.7 | 111 / – | 40.4 / – | 117 / – | 439 / – |
| 1024 | **44.7** / **37.8** | – | 167 / 170 | 129 / – | 68.3 / – | 266 / – | 448 / – |
| 2048 | **83.2** / **75.6** | – | 391 / 381 | 325 / – | 159 / – | 577 / – | 611 / – |
| 4096 | **217** / **238** | – | 822 / 877 | 590 / – | 407 / – | 1,341 / – | 1,195 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / **1.3** | **2.4** / 1.4 | 2.6 / 1.4 | 2.5 / **1.3** | 2.6 / 1.4 | – |
| 256 | **4.5** / **2.3** | – | 4.6 / 2.4 | 5.6 / 5.4 | 5.1 / 3.1 | – |
| 384 | **7.0** / **3.7** | – | 7.2 / 3.9 | 9.0 / 10.5 | 7.9 / 5.0 | – |
| 512 | **9.5** / **4.1** | – | **9.5** / 4.4 | 12.3 / 16.7 | 11.5 / 7.9 | – |
| 1024 | **23.4** / **22.0** | – | 27.9 / 22.8 | 28.4 / 28.6 | 27.2 / 28.4 | – |
| 2048 | **53.3** / **51.0** | – | 83.7 / 68.5 | 80.6 / 102 | 96.7 / 106 | – |
| 4096 | **162** / **161** | – | 166 / 191 | 199 / 277 | 209 / 267 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | **1.4** / – | **1.4** / – | **1.4** / – | 1.5 / – |
| 128 | 3.0 / – | **2.9** / **5.0** | 3.0 / – | 3.0 / – | 3.0 / – | 3.0 / – |
| 256 | 12.4 / – | – | **12.3** / – | 13.0 / – | 13.0 / – | 16.0 / – |
| 384 | 30.0 / – | – | 30.0 / – | **29.8** / – | 33.6 / – | 36.6 / – |
| 512 | **50.9** / – | – | 59.9 / – | 81.6 / – | 85.8 / – | – |
| 1024 | **226** / – | – | – | 432 / – | 386 / – | 438 / – |
| 2048 | **1,154** / – | – | – | 1,958 / – | 1,536 / – | 4,633 / – |
| 4096 | **4,213** / – | – | – | 7,742 / – | 6,116 / – | 24,948 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.1** / – | 196 / – |
| 128 | **98.8** / – | 407 / – |
| 256 | **199** / – | 747 / – |
| 384 | **291** / – | 1,104 / – |
| 512 | **461** / – | 1,436 / – |
| 1024 | **1,139** / – | 3,355 / – |
| 2048 | **3,426** / – | 10,137 / – |
| 4096 | **11,846** / – | 30,753 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.5** / – | 186 / – |
| 128 | **105** / – | 386 / – |
| 256 | **216** / – | 720 / – |
| 384 | **311** / – | 1,053 / – |
| 512 | **466** / – | 1,446 / – |
| 1024 | **1,147** / – | 3,348 / – |
| 2048 | **3,445** / – | 10,193 / – |
| 4096 | **11,904** / – | 30,762 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **51.9** / **93.9** | 223 / – |
| 128 | **201** / **206** | 608 / – |
| 256 | **453** / **471** | 1,209 / – |
| 384 | **823** / **830** | 1,964 / – |
| 512 | **1,189** / **1,189** | 2,984 / – |
| 1024 | **3,856** / **3,878** | 7,663 / – |
| 2048 | **13,721** / **13,505** | 25,204 / – |
| 4096 | **48,937** / **48,916** | 86,058 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **6.6** / **8.9** | 11.1 / – |
| 128 | **11.0** / **12.5** | 27.2 / – |
| 256 | **20.6** / **18.1** | 70.3 / – |
| 384 | **38.7** / **36.0** | 90.9 / – |
| 512 | **56.5** / **47.8** | 122 / – |
| 1024 | **130** / **131** | 143 / – |
| 2048 | **303** / **308** | 429 / – |
| 4096 | **511** / **551** | 891 / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **6.2** / **8.9** | 4.6\* / – |
| 128 | **7.4** / **11.8** | 11.5 / – |
| 256 | **13.7** / **17.1** | 49.3 / – |
| 384 | **27.0** / **27.5** | 82.0 / – |
| 512 | **33.6** / **32.2** | 111 / – |
| 1024 | **96.9** / **94.2** | 129 / – |
| 2048 | **414** / **284** | 323\* / – |
| 4096 | **487** / **506** | 592 / – |

\* `crypto-bigint` at 64 and 2048 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (`Uint::sub_mod`: "Assumes `self - rhs` as unbounded signed integer is in `[-p, p)`").

### add_mod, reduced operands, constant time: a + b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **3.8** / **6.9** | 11.2 / – |
| 128 | **6.9** / **8.8** | 27.3 / – |
| 256 | **9.9** / **12.9** | 70.3 / – |
| 384 | **13.2** / **14.6** | 90.8 / – |
| 512 | **19.8** / **20.3** | 122 / – |
| 1024 | **57.8** / **51.6** | 144 / – |
| 2048 | **132** / **140** | 429 / – |
| 4096 | **269** / **301** | 893 / – |

### sub_mod, reduced operands, constant time: a − b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **3.2** / **3.9** | 4.6 / – |
| 128 | **5.1** / **4.8** | 11.4 / – |
| 256 | **8.9** / **6.9** | 49.2 / – |
| 384 | **11.0** / **9.5** | 82.0 / – |
| 512 | **18.5** / **18.0** | 111 / – |
| 1024 | **48.5** / **44.8** | 129 / – |
| 2048 | **116** / **121** | 325 / – |
| 4096 | **270** / **304** | 590 / – |

## WebAssembly

`biguint` at commit `c7970be` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-06.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions.
- A case whose difference counts under 50 times the noise of the fixed cost of a count stops the sweep.
- Latency: `x` in WebAssembly locals, with a copy stored after each step, or in memory, whichever is faster.

### Setup, WebAssembly

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS, in WebAssembly, Chrome | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Chrome | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| RUSTFLAGS, in WebAssembly, Firefox | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Firefox | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| Browser, in WebAssembly, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff` |
| Target features, in WebAssembly, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Chrome | 7,138 to 13,497 cycles and 3,081 instructions; its noise in a figure, 204 to 7,466 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 6,072 to 14,196 cycles and 3,081 instructions; its noise in a figure, 242 to 7,310 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 10,056 to 37,322 cycles and 1,885 instructions; its noise in a figure, 550 to 13,270 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 9,236 to 21,509 cycles and 1,885 instructions; its noise in a figure, 511 to 8,195 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 53.1 before, 36.99 after; its SMT sibling, CPU 28, 63% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 36.99 before, 55.66 after; its SMT sibling, CPU 28, 57% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 55.66 before, 51.77 after; its SMT sibling, CPU 28, 65% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 49.86 before, 18.11 after; its SMT sibling, CPU 28, 46% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.1 vs ruint 2.1 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.4 vs ruint 2.1 | +0.3 (1.14×) | 2.3 vs native 2.2 | +0.1 (1.05×) |
| add | 128 | 4.2 vs native 3.7 | +0.5 (1.14×) | 3.9 vs native 3.7 | +0.2 (1.05×) | 3.9 vs bnum 3.6 | +0.3 (1.08×) | 4.3 vs bnum 3.5 | +0.8 (1.23×) |
| add | 256 | 9.7 vs crypto-bigint 10.3 | −0.6 (0.94×) | 8.8 vs ruint 9.0 | −0.2 (0.98×) | 10.0 vs bnum 9.9 | +0.1 (1.01×) | 11.5 vs bnum 11.1 | +0.4 (1.04×) |
| add | 384 | 16.5 vs bnum 17.1 | −0.6 (0.96×) | 12.8 vs crypto-bigint 12.3 | +0.5 (1.04×) | 15.3 vs ruint 16.2 | −0.9 (0.94×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add | 512 | 26.3 vs bnum 24.7 | +1.6 (1.06×) | 20.8 vs crypto-bigint 20.9 | −0.1 (1.00×) | 26.0 vs bnum 23.6 | +2.4 (1.10×) | 32.3 vs bnum 27.6 | +4.7 (1.17×) |
| add | 1024 | 55.4 vs bnum 61.8 | −6.4 (0.90×) | 45.4 vs crypto-bigint 45.3 | +0.1 (1.00×) | 72.5 vs ruint 67.8 | +4.7 (1.07×) | 69.9 vs crypto-bigint 69.3 | +0.6 (1.01×) |
| add | 2048 | 115 vs crypto-bigint 199 | −84.0 (0.58×) | 108 vs crypto-bigint 137 | −29.0 (0.79×) | 143 vs bnum 204 | −61.0 (0.70×) | 149 vs crypto-bigint 177 | −28.0 (0.84×) |
| add | 4096 | 349 vs crypto-bigint 395 | −46.0 (0.88×) | 263 vs crypto-bigint 265 | −2.0 (0.99×) | 402 vs bnum 396 | +6.0 (1.02×) | 322 vs crypto-bigint 326 | −4.0 (0.99×) |
| mul | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs uint 2.1 | +0.1 (1.05×) |
| mul | 128 | 10.0 vs uint 11.0 | −1.0 (0.91×) | 7.1 vs native 4.3 | +2.8 (1.65×) | 12.3 vs bnum 13.6 | −1.3 (0.90×) | 5.7 vs crypto-bigint 5.1 | +0.6 (1.12×) |
| mul | 256 | 48.0 vs ruint 66.7 | −18.7 (0.72×) | 15.6 vs ruint 17.7 | −2.1 (0.88×) | 67.7 vs ruint 81.1 | −13.4 (0.83×) | 26.0 vs ruint 26.1 | −0.1 (1.00×) |
| mul | 384 | 117 vs uint 165 | −48.0 (0.71×) | 54.2 vs crypto-bigint 39.9 | +14.3 (1.36×) | 166 vs ruint 218 | −52.0 (0.76×) | 72.8 vs crypto-bigint 71.7 | +1.1 (1.02×) |
| mul | 512 | 207 vs ruint 329 | −122 (0.63×) | 78.1 vs ruint 76.7 | +1.4 (1.02×) | 307 vs ruint 439 | −132 (0.70×) | 132 vs ruint 131 | +1.0 (1.01×) |
| mul | 1024 | 1,261 vs bnum 1,515 | −254 (0.83×) | 317 vs uint 521 | −204 (0.61×) | 1,491 vs bnum 2,354 | −863 (0.63×) | 568 vs crypto-bigint 900 | −332 (0.63×) |
| mul | 2048 | 5,611 vs bnum 6,283 | −672 (0.89×) | 2,320 vs ruint 2,185 | +135 (1.06×) | 6,714 vs bnum 8,650 | −1,936 (0.78×) | 3,522 vs ruint 3,247 | +275 (1.08×) |
| mul | 4096 | 17,771 vs bnum 23,617 | −5,846 (0.75×) | 7,925 vs ruint 8,132 | −207 (0.97×) | 21,315 vs bnum 31,724 | −10,409 (0.67×) | 11,934 vs ruint 11,640 | +294 (1.03×) |
| div | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 59.7 vs ruint 88.6 | −28.9 (0.67×) | 28.5 vs ruint 41.7 | −13.2 (0.68×) | 83.7 vs ruint 152 | −68.3 (0.55×) | 36.1 vs ruint 54.9 | −18.8 (0.66×) |
| div | 256 | 210 vs ruint 325 | −115 (0.65×) | 84.1 vs ruint 161 | −76.9 (0.52×) | 221 vs ruint 402 | −181 (0.55×) | 110 vs ruint 174 | −64.0 (0.63×) |
| div | 384 | 354 vs num-bigint 618 | −264 (0.57×) | 207 vs ruint 349 | −142 (0.59×) | 405 vs num-bigint 753 | −348 (0.54×) | 285 vs ruint 437 | −152 (0.65×) |
| div | 512 | 474 vs num-bigint 819 | −345 (0.58×) | 306 vs ruint 444 | −138 (0.69×) | 573 vs num-bigint 1,006 | −433 (0.57×) | 414 vs ruint 549 | −135 (0.75×) |
| div | 1024 | 1,144 vs num-bigint 1,972 | −828 (0.58×) | 775 vs ruint 1,082 | −307 (0.72×) | 1,477 vs num-bigint 2,282 | −805 (0.65×) | 868 vs ruint 1,135 | −267 (0.76×) |
| div | 2048 | 3,234 vs num-bigint 5,752 | −2,518 (0.56×) | 2,037 vs ruint 2,349 | −312 (0.87×) | 4,342 vs num-bigint 6,245 | −1,903 (0.70×) | 2,257 vs ruint 2,726 | −469 (0.83×) |
| div | 4096 | 12,808 vs num-bigint 20,009 | −7,201 (0.64×) | 7,115 vs ruint 7,593 | −478 (0.94×) | 15,198 vs num-bigint 20,353 | −5,155 (0.75×) | 7,494 vs ruint 9,150 | −1,656 (0.82×) |
| mod | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.1 vs native 7.0 | +0.1 (1.01×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 52.8 vs ruint 96.0 | −43.2 (0.55×) | 30.0 vs ruint 40.2 | −10.2 (0.75×) | 92.6 vs ruint 97.9 | −5.3 (0.95×) | 33.7 vs ruint 51.0 | −17.3 (0.66×) |
| mod | 256 | 208 vs ruint 323 | −115 (0.64×) | 74.5 vs ruint 160 | −85.5 (0.47×) | 224 vs ruint 400 | −176 (0.56×) | 109 vs ruint 175 | −66.0 (0.62×) |
| mod | 384 | 347 vs ruint 577 | −230 (0.60×) | 198 vs ruint 310 | −112 (0.64×) | 410 vs ruint 720 | −310 (0.57×) | 266 vs ruint 344 | −78.0 (0.77×) |
| mod | 512 | 467 vs ruint 804 | −337 (0.58×) | 285 vs ruint 419 | −134 (0.68×) | 548 vs ruint 984 | −436 (0.56×) | 350 vs ruint 477 | −127 (0.73×) |
| mod | 1024 | 1,195 vs num-bigint 1,939 | −744 (0.62×) | 717 vs ruint 1,168 | −451 (0.61×) | 1,476 vs num-bigint 2,223 | −747 (0.66×) | 905 vs ruint 1,052 | −147 (0.86×) |
| mod | 2048 | 3,311 vs num-bigint 5,742 | −2,431 (0.58×) | 1,934 vs ruint 2,302 | −368 (0.84×) | 4,367 vs num-bigint 6,124 | −1,757 (0.71×) | 2,114 vs ruint 2,626 | −512 (0.81×) |
| mod | 4096 | 12,974 vs num-bigint 20,046 | −7,072 (0.65×) | 6,515 vs ruint 7,779 | −1,264 (0.84×) | 15,422 vs num-bigint 20,392 | −4,970 (0.76×) | 7,649 vs ruint 8,930 | −1,281 (0.86×) |
| mulmod | 64 | 61.4 vs ruint 85.0 | −23.6 (0.72×) | 31.2 vs ruint 40.0 | −8.8 (0.78×) | 107 vs ruint 188 | −81.0 (0.57×) | 36.1 vs ruint 53.7 | −17.6 (0.67×) |
| mulmod | 128 | 201 vs ruint 322 | −121 (0.62×) | 82.2 vs ruint 135 | −52.8 (0.61×) | 240 vs ruint 386 | −146 (0.62×) | 127 vs ruint 206 | −79.0 (0.62×) |
| mulmod | 256 | 581 vs ruint 813 | −232 (0.71×) | 284 vs ruint 308 | −24.0 (0.92×) | 701 vs ruint 898 | −197 (0.78×) | 360 vs ruint 380 | −20.0 (0.95×) |
| mulmod | 384 | 977 vs ruint 1,611 | −634 (0.61×) | 515 vs ruint 540 | −25.0 (0.95×) | 1,345 vs ruint 1,712 | −367 (0.79×) | 628 vs ruint 696 | −68.0 (0.90×) |
| mulmod | 512 | 1,583 vs ruint 2,794 | −1,211 (0.57×) | 884 vs ruint 857 | +27.0 (1.03×) | 2,154 vs num-bigint 3,105 | −951 (0.69×) | 1,074 vs ruint 1,115 | −41.0 (0.96×) |
| mulmod | 1024 | 5,346 vs num-bigint 9,009 | −3,663 (0.59×) | 2,737 vs ruint 2,649 | +88.0 (1.03×) | 6,727 vs num-bigint 9,249 | −2,522 (0.73×) | 3,605 vs ruint 3,639 | −34.0 (0.99×) |
| mulmod | 2048 | 17,492 vs num-bigint 32,028 | −14,536 (0.55×) | 8,747 vs ruint 10,720 | −1,973 (0.82×) | 22,288 vs num-bigint 32,375 | −10,087 (0.69×) | 13,109 vs ruint 13,800 | −691 (0.95×) |
| mulmod | 4096 | 56,007 vs num-bigint 112,495 | −56,488 (0.50×) | 26,793 vs ruint 41,119 | −14,326 (0.65×) | 69,330 vs num-bigint 115,974 | −46,644 (0.60×) | 46,093 vs ruint 53,398 | −7,305 (0.86×) |
| add_mod | 64 | 4.0 vs ruint 43.6 | −39.6 (0.09×) | 4.0 vs ruint 43.2 | −39.2 (0.09×) | 8.0 vs ruint 80.2 | −72.2 (0.10×) | 7.7 vs ruint 80.4 | −72.7 (0.10×) |
| add_mod | 128 | 9.1 vs ruint 23.0 | −13.9 (0.40×) | 9.2 vs ruint 21.9 | −12.7 (0.42×) | 12.5 vs ruint 22.1 | −9.6 (0.57×) | 13.7 vs ruint 25.0 | −11.3 (0.55×) |
| add_mod | 256 | 23.1 vs ruint 43.2 | −20.1 (0.53×) | 21.9 vs ruint 43.9 | −22.0 (0.50×) | 24.7 vs ruint 61.2 | −36.5 (0.40×) | 30.4 vs ruint 59.2 | −28.8 (0.51×) |
| add_mod | 384 | 30.9 vs ruint 62.5 | −31.6 (0.49×) | 28.5 vs ruint 60.0 | −31.5 (0.47×) | 46.8 vs ruint 96.7 | −49.9 (0.48×) | 45.2 vs ruint 86.1 | −40.9 (0.52×) |
| add_mod | 512 | 44.8 vs ruint 90.2 | −45.4 (0.50×) | 38.7 vs ruint 86.0 | −47.3 (0.45×) | 60.7 vs ruint 121 | −60.3 (0.50×) | 62.4 vs ruint 122 | −59.6 (0.51×) |
| add_mod | 1024 | 114 vs ruint 212 | −98.0 (0.54×) | 104 vs ruint 183 | −79.0 (0.57×) | 144 vs ruint 273 | −129 (0.53×) | 135 vs ruint 251 | −116 (0.54×) |
| add_mod | 2048 | 278 vs ruint 806 | −528 (0.34×) | 245 vs ruint 746 | −501 (0.33×) | 378 vs ruint 1,041 | −663 (0.36×) | 291 vs ruint 882 | −591 (0.33×) |
| add_mod | 4096 | 734 vs ruint 1,532 | −798 (0.48×) | 750 vs ruint 1,415 | −665 (0.53×) | 792 vs ruint 1,859 | −1,067 (0.43×) | 941 vs ruint 1,673 | −732 (0.56×) |
| sub_mod | 64 | 3.9 vs ruint 53.9 | −50.0 (0.07×) | 4.0 vs ruint 50.5 | −46.5 (0.08×) | 8.0 vs ruint 80.3 | −72.3 (0.10×) | 8.1 vs ruint 80.0 | −71.9 (0.10×) |
| sub_mod | 128 | 10.8 vs ruint 31.2 | −20.4 (0.35×) | 10.2 vs ruint 30.1 | −19.9 (0.34×) | 13.8 vs ruint 37.2 | −23.4 (0.37×) | 15.8 vs ruint 37.3 | −21.5 (0.42×) |
| sub_mod | 256 | 23.8 vs ruint 60.4 | −36.6 (0.39×) | 19.8 vs ruint 60.0 | −40.2 (0.33×) | 26.1 vs ruint 86.9 | −60.8 (0.30×) | 28.8 vs ruint 87.0 | −58.2 (0.33×) |
| sub_mod | 384 | 31.1 vs ruint 87.4 | −56.3 (0.36×) | 27.0 vs ruint 82.6 | −55.6 (0.33×) | 44.1 vs ruint 142 | −97.9 (0.31×) | 48.4 vs ruint 128 | −79.6 (0.38×) |
| sub_mod | 512 | 45.5 vs ruint 121 | −75.5 (0.38×) | 37.6 vs ruint 114 | −76.4 (0.33×) | 62.0 vs ruint 177 | −115 (0.35×) | 68.0 vs ruint 173 | −105 (0.39×) |
| sub_mod | 1024 | 105 vs ruint 355 | −250 (0.30×) | 110 vs ruint 331 | −221 (0.33×) | 128 vs ruint 575 | −447 (0.22×) | 134 vs ruint 564 | −430 (0.24×) |
| sub_mod | 2048 | 237 vs ruint 1,114 | −877 (0.21×) | 249 vs ruint 1,063 | −814 (0.23×) | 287 vs ruint 1,386 | −1,099 (0.21×) | 277 vs ruint 1,270 | −993 (0.22×) |
| sub_mod | 4096 | 725 vs ruint 2,122 | −1,397 (0.34×) | 775 vs ruint 2,021 | −1,246 (0.38×) | 846 vs ruint 2,424 | −1,578 (0.35×) | 965 vs ruint 2,332 | −1,367 (0.41×) |
| add_mod, reduced operands | 64 | 4.2 vs native 3.4 | +0.8 (1.24×) | 4.1 vs native 4.0 | +0.1 (1.02×) | 4.5 vs uint 3.7 | +0.8 (1.22×) | 4.3 vs native 5.0 | −0.7 (0.86×) |
| add_mod, reduced operands | 128 | 8.3 vs native 9.6 | −1.3 (0.86×) | 9.0 vs native 9.1 | −0.1 (0.99×) | 8.2 vs native 9.7 | −1.5 (0.85×) | 11.5 vs native 9.1 | +2.4 (1.26×) |
| add_mod, reduced operands | 256 | 16.9 vs uint 22.0 | −5.1 (0.77×) | 17.0 vs uint 21.6 | −4.6 (0.79×) | 18.7 vs uint 21.1 | −2.4 (0.89×) | 22.5 vs uint 24.7 | −2.2 (0.91×) |
| add_mod, reduced operands | 384 | 27.8 vs uint 32.9 | −5.1 (0.84×) | 22.2 vs uint 38.2 | −16.0 (0.58×) | 28.2 vs uint 34.7 | −6.5 (0.81×) | 32.4 vs uint 40.8 | −8.4 (0.79×) |
| add_mod, reduced operands | 512 | 34.8 vs uint 44.1 | −9.3 (0.79×) | 28.3 vs uint 43.5 | −15.2 (0.65×) | 41.4 vs uint 42.9 | −1.5 (0.97×) | 42.3 vs uint 62.4 | −20.1 (0.68×) |
| add_mod, reduced operands | 1024 | 77.2 vs uint 102 | −24.8 (0.76×) | 68.9 vs uint 106 | −37.1 (0.65×) | 88.7 vs uint 145 | −56.3 (0.61×) | 92.1 vs uint 143 | −50.9 (0.64×) |
| add_mod, reduced operands | 2048 | 198 vs uint 245 | −47.0 (0.81×) | 150 vs uint 225 | −75.0 (0.67×) | 233 vs uint 294 | −61.0 (0.79×) | 220 vs uint 260 | −40.0 (0.85×) |
| add_mod, reduced operands | 4096 | 535 vs uint 619 | −84.0 (0.86×) | 462 vs uint 560 | −98.0 (0.82×) | 585 vs uint 683 | −98.0 (0.86×) | 525 vs uint 672 | −147 (0.78×) |
| sub_mod, reduced operands | 64 | 3.6 vs native 3.0 | +0.6 (1.20×) | 4.0 vs native 3.0 | +1.0 (1.33×) | 4.7 vs native 3.1 | +1.6 (1.52×) | 6.2 vs native 3.0 | +3.2 (2.07×) |
| sub_mod, reduced operands | 128 | 7.1 vs native 6.8 | +0.3 (1.04×) | 7.3 vs native 5.4 | +1.9 (1.35×) | 7.7 vs native 9.2 | −1.5 (0.84×) | 9.6 vs native 7.3 | +2.3 (1.32×) |
| sub_mod, reduced operands | 256 | 17.0 vs uint 18.9 | −1.9 (0.90×) | 13.2 vs bnum 16.6 | −3.4 (0.80×) | 14.9 vs bnum 20.4 | −5.5 (0.73×) | 20.2 vs bnum 20.8 | −0.6 (0.97×) |
| sub_mod, reduced operands | 384 | 26.5 vs uint 31.0 | −4.5 (0.85×) | 20.8 vs uint 31.6 | −10.8 (0.66×) | 25.6 vs bnum 32.8 | −7.2 (0.78×) | 30.4 vs bnum 42.8 | −12.4 (0.71×) |
| sub_mod, reduced operands | 512 | 34.4 vs bnum 47.7 | −13.3 (0.72×) | 29.8 vs bnum 46.5 | −16.7 (0.64×) | 37.7 vs bnum 49.0 | −11.3 (0.77×) | 41.8 vs bnum 63.9 | −22.1 (0.65×) |
| sub_mod, reduced operands | 1024 | 66.6 vs uint 132 | −65.4 (0.50×) | 68.0 vs uint 129 | −61.0 (0.53×) | 83.3 vs uint 154 | −70.7 (0.54×) | 90.2 vs uint 149 | −58.8 (0.61×) |
| sub_mod, reduced operands | 2048 | 209 vs uint 291 | −82.0 (0.72×) | 157 vs uint 288 | −131 (0.55×) | 280 vs uint 369 | −89.0 (0.76×) | 199 vs uint 371 | −172 (0.54×) |
| sub_mod, reduced operands | 4096 | 537 vs bnum 681 | −144 (0.79×) | 469 vs uint 696 | −227 (0.67×) | 588 vs bnum 715 | −127 (0.82×) | 589 vs bnum 780 | −191 (0.76×) |
| add, constant time | 64 | 2.2 vs uint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs crypto-bigint 2.1 | +0.1 (1.05×) | 2.2 vs bnum 2.0 | +0.2 (1.10×) |
| add, constant time | 128 | 4.0 vs bnum 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs bnum 3.7 | +0.3 (1.08×) | 4.2 vs native 3.3 | +0.9 (1.27×) |
| add, constant time | 256 | 10.2 vs crypto-bigint 9.7 | +0.5 (1.05×) | 9.0 vs crypto-bigint 8.9 | +0.1 (1.01×) | 10.1 vs bnum 10.1 | +0.0 (1.00×) | 11.5 vs bnum 11.1 | +0.4 (1.04×) |
| add, constant time | 384 | 16.0 vs ruint 16.2 | −0.2 (0.99×) | 12.5 vs crypto-bigint 12.4 | +0.1 (1.01×) | 15.2 vs ruint 16.3 | −1.1 (0.93×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add, constant time | 512 | 27.9 vs bnum 24.6 | +3.3 (1.13×) | 20.9 vs crypto-bigint 20.8 | +0.1 (1.00×) | 26.3 vs bnum 23.7 | +2.6 (1.11×) | 32.3 vs bnum 27.7 | +4.6 (1.17×) |
| add, constant time | 1024 | 59.7 vs bnum 61.9 | −2.2 (0.96×) | 45.8 vs crypto-bigint 45.0 | +0.8 (1.02×) | 71.1 vs bnum 67.5 | +3.6 (1.05×) | 69.8 vs crypto-bigint 69.1 | +0.7 (1.01×) |
| add, constant time | 2048 | 117 vs crypto-bigint 196 | −79.0 (0.60×) | 109 vs crypto-bigint 138 | −29.0 (0.79×) | 144 vs crypto-bigint 207 | −63.0 (0.70×) | 150 vs crypto-bigint 177 | −27.0 (0.85×) |
| add, constant time | 4096 | 344 vs crypto-bigint 392 | −48.0 (0.88×) | 267 vs crypto-bigint 278 | −11.0 (0.96×) | 400 vs bnum 397 | +3.0 (1.01×) | 319 vs crypto-bigint 328 | −9.0 (0.97×) |
| mul, constant time | 64 | 2.2 vs ruint 2.0 | +0.2 (1.10×) | 2.2 vs crypto-bigint 2.3 | −0.1 (0.96×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs ruint 2.2 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.1 vs uint 10.9 | −0.8 (0.93×) | 6.7 vs bnum 4.4 | +2.3 (1.52×) | 12.3 vs bnum 13.6 | −1.3 (0.90×) | 5.7 vs crypto-bigint 5.0 | +0.7 (1.14×) |
| mul, constant time | 256 | 47.9 vs ruint 66.8 | −18.9 (0.72×) | 22.7 vs ruint 17.0 | +5.7 (1.34×) | 67.1 vs ruint 81.1 | −14.0 (0.83×) | 26.0 vs ruint 26.1 | −0.1 (1.00×) |
| mul, constant time | 384 | 116 vs uint 168 | −52.0 (0.69×) | 55.9 vs crypto-bigint 40.4 | +15.5 (1.38×) | 168 vs ruint 218 | −50.0 (0.77×) | 72.4 vs crypto-bigint 71.5 | +0.9 (1.01×) |
| mul, constant time | 512 | 210 vs ruint 321 | −111 (0.65×) | 74.9 vs crypto-bigint 85.0 | −10.1 (0.88×) | 307 vs ruint 442 | −135 (0.69×) | 132 vs ruint 131 | +1.0 (1.01×) |
| mul, constant time | 1024 | 1,193 vs bnum 1,551 | −358 (0.77×) | 349 vs uint 504 | −155 (0.69×) | 1,491 vs bnum 2,347 | −856 (0.64×) | 566 vs crypto-bigint 895 | −329 (0.63×) |
| mul, constant time | 2048 | 5,793 vs bnum 6,283 | −490 (0.92×) | 2,439 vs bnum 2,622 | −183 (0.93×) | 6,672 vs bnum 8,650 | −1,978 (0.77×) | 3,634 vs bnum 3,433 | +201 (1.06×) |
| mul, constant time | 4096 | 18,754 vs bnum 23,617 | −4,863 (0.79×) | 8,525 vs bnum 10,106 | −1,581 (0.84×) | 21,984 vs bnum 31,724 | −9,740 (0.69×) | 12,511 vs bnum 12,846 | −335 (0.97×) |
| div, constant time | 64 | 83.2 vs crypto-bigint 395 | −312 (0.21×) | 47.4 vs crypto-bigint 322 | −275 (0.15×) | 146 vs crypto-bigint 424 | −278 (0.34×) | 61.4 vs crypto-bigint 344 | −283 (0.18×) |
| div, constant time | 128 | 280 vs crypto-bigint 909 | −629 (0.31×) | 179 vs crypto-bigint 780 | −601 (0.23×) | 362 vs crypto-bigint 1,070 | −708 (0.34×) | 231 vs crypto-bigint 925 | −694 (0.25×) |
| div, constant time | 256 | 521 vs crypto-bigint 1,694 | −1,173 (0.31×) | 334 vs crypto-bigint 1,390 | −1,056 (0.24×) | 635 vs crypto-bigint 1,973 | −1,338 (0.32×) | 471 vs crypto-bigint 1,507 | −1,036 (0.31×) |
| div, constant time | 384 | 819 vs crypto-bigint 2,521 | −1,702 (0.32×) | 457 vs crypto-bigint 2,038 | −1,581 (0.22×) | 811 vs crypto-bigint 3,042 | −2,231 (0.27×) | 545 vs crypto-bigint 2,301 | −1,756 (0.24×) |
| div, constant time | 512 | 1,023 vs crypto-bigint 3,556 | −2,533 (0.29×) | 745 vs crypto-bigint 2,826 | −2,081 (0.26×) | 1,186 vs crypto-bigint 4,260 | −3,074 (0.28×) | 862 vs crypto-bigint 3,105 | −2,243 (0.28×) |
| div, constant time | 1024 | 2,475 vs crypto-bigint 8,536 | −6,061 (0.29×) | 1,796 vs crypto-bigint 7,066 | −5,270 (0.25×) | 2,881 vs crypto-bigint 10,340 | −7,459 (0.28×) | 2,187 vs crypto-bigint 7,485 | −5,298 (0.29×) |
| div, constant time | 2048 | 7,476 vs crypto-bigint 22,588 | −15,112 (0.33×) | 5,195 vs crypto-bigint 17,724 | −12,529 (0.29×) | 8,404 vs crypto-bigint 27,667 | −19,263 (0.30×) | 6,477 vs crypto-bigint 18,511 | −12,034 (0.35×) |
| div, constant time | 4096 | 25,658 vs crypto-bigint 67,391 | −41,733 (0.38×) | 16,426 vs crypto-bigint 43,621 | −27,195 (0.38×) | 27,938 vs crypto-bigint 84,183 | −56,245 (0.33×) | 21,618 vs crypto-bigint 50,736 | −29,118 (0.43×) |
| mod, constant time | 64 | 83.6 vs crypto-bigint 384 | −300 (0.22×) | 48.2 vs crypto-bigint 324 | −276 (0.15×) | 142 vs crypto-bigint 421 | −279 (0.34×) | 67.6 vs crypto-bigint 341 | −273 (0.20×) |
| mod, constant time | 128 | 347 vs crypto-bigint 900 | −553 (0.39×) | 169 vs crypto-bigint 742 | −573 (0.23×) | 332 vs crypto-bigint 1,060 | −728 (0.31×) | 226 vs crypto-bigint 900 | −674 (0.25×) |
| mod, constant time | 256 | 520 vs crypto-bigint 1,652 | −1,132 (0.31×) | 333 vs crypto-bigint 1,350 | −1,017 (0.25×) | 581 vs crypto-bigint 1,948 | −1,367 (0.30×) | 414 vs crypto-bigint 1,501 | −1,087 (0.28×) |
| mod, constant time | 384 | 748 vs crypto-bigint 2,498 | −1,750 (0.30×) | 537 vs crypto-bigint 1,991 | −1,454 (0.27×) | 805 vs crypto-bigint 2,995 | −2,190 (0.27×) | 560 vs crypto-bigint 2,238 | −1,678 (0.25×) |
| mod, constant time | 512 | 1,056 vs crypto-bigint 3,680 | −2,624 (0.29×) | 756 vs crypto-bigint 2,758 | −2,002 (0.27×) | 1,084 vs crypto-bigint 4,284 | −3,200 (0.25×) | 832 vs crypto-bigint 3,110 | −2,278 (0.27×) |
| mod, constant time | 1024 | 2,541 vs crypto-bigint 8,511 | −5,970 (0.30×) | 1,955 vs crypto-bigint 7,283 | −5,328 (0.27×) | 2,745 vs crypto-bigint 10,449 | −7,704 (0.26×) | 1,996 vs crypto-bigint 7,421 | −5,425 (0.27×) |
| mod, constant time | 2048 | 7,775 vs crypto-bigint 22,344 | −14,569 (0.35×) | 5,269 vs crypto-bigint 17,157 | −11,888 (0.31×) | 8,332 vs crypto-bigint 27,700 | −19,368 (0.30×) | 5,909 vs crypto-bigint 18,527 | −12,618 (0.32×) |
| mod, constant time | 4096 | 25,871 vs crypto-bigint 69,260 | −43,389 (0.37×) | 16,456 vs crypto-bigint 43,339 | −26,883 (0.38×) | 29,046 vs crypto-bigint 88,465 | −59,419 (0.33×) | 20,672 vs crypto-bigint 50,385 | −29,713 (0.41×) |
| mulmod, constant time | 64 | 203 vs crypto-bigint 482 | −279 (0.42×) | 99.7 vs crypto-bigint 357 | −257 (0.28×) | 237 vs crypto-bigint 528 | −291 (0.45×) | 113 vs crypto-bigint 380 | −267 (0.30×) |
| mulmod, constant time | 128 | 573 vs crypto-bigint 1,270 | −697 (0.45×) | 360 vs crypto-bigint 996 | −636 (0.36×) | 601 vs crypto-bigint 1,540 | −939 (0.39×) | 372 vs crypto-bigint 1,146 | −774 (0.32×) |
| mulmod, constant time | 256 | 1,218 vs crypto-bigint 2,772 | −1,554 (0.44×) | 744 vs crypto-bigint 1,984 | −1,240 (0.38×) | 1,348 vs crypto-bigint 3,330 | −1,982 (0.40×) | 818 vs crypto-bigint 2,279 | −1,461 (0.36×) |
| mulmod, constant time | 384 | 1,965 vs crypto-bigint 4,839 | −2,874 (0.41×) | 1,225 vs crypto-bigint 3,252 | −2,027 (0.38×) | 2,257 vs crypto-bigint 5,649 | −3,392 (0.40×) | 1,500 vs crypto-bigint 3,747 | −2,247 (0.40×) |
| mulmod, constant time | 512 | 3,009 vs crypto-bigint 7,718 | −4,709 (0.39×) | 1,824 vs crypto-bigint 4,665 | −2,841 (0.39×) | 3,495 vs crypto-bigint 8,680 | −5,185 (0.40×) | 2,307 vs crypto-bigint 5,528 | −3,221 (0.42×) |
| mulmod, constant time | 1024 | 9,611 vs crypto-bigint 21,454 | −11,843 (0.45×) | 5,616 vs crypto-bigint 13,464 | −7,848 (0.42×) | 10,989 vs crypto-bigint 25,200 | −14,211 (0.44×) | 7,558 vs crypto-bigint 14,239 | −6,681 (0.53×) |
| mulmod, constant time | 2048 | 32,861 vs crypto-bigint 64,781 | −31,920 (0.51×) | 18,903 vs crypto-bigint 36,271 | −17,368 (0.52×) | 36,878 vs crypto-bigint 77,299 | −40,421 (0.48×) | 25,971 vs crypto-bigint 46,201 | −20,230 (0.56×) |
| mulmod, constant time | 4096 | 119,911 vs crypto-bigint 261,147 | −141,236 (0.46×) | 68,830 vs crypto-bigint 111,657 | −42,827 (0.62×) | 132,639 vs crypto-bigint 253,339 | −120,700 (0.52×) | 91,464 vs crypto-bigint 145,680 | −54,216 (0.63×) |
| add_mod, constant time | 64 | 9.0 | – | 9.5 | – | 19.3 | – | 13.8 | – |
| add_mod, constant time | 128 | 26.5 | – | 21.5 | – | 34.5 | – | 35.0 | – |
| add_mod, constant time | 256 | 82.3 | – | 65.4 | – | 108 | – | 74.9 | – |
| add_mod, constant time | 384 | 120 | – | 111 | – | 167 | – | 141 | – |
| add_mod, constant time | 512 | 150 | – | 145 | – | 238 | – | 180 | – |
| add_mod, constant time | 1024 | 323 | – | 272 | – | 607 | – | 344 | – |
| add_mod, constant time | 2048 | 867 | – | 531 | – | 1,702 | – | 761 | – |
| add_mod, constant time | 4096 | 1,886 | – | 1,717 | – | 2,017 | – | 2,011 | – |
| sub_mod, constant time | 64 | 10.2 | – | 9.3 | – | 18.9 | – | 14.3 | – |
| sub_mod, constant time | 128 | 15.6 | – | 13.0 | – | 30.9 | – | 23.0 | – |
| sub_mod, constant time | 256 | 45.9 | – | 31.9 | – | 73.8 | – | 65.2 | – |
| sub_mod, constant time | 384 | 103 | – | 51.4 | – | 136 | – | 96.7 | – |
| sub_mod, constant time | 512 | 140 | – | 88.2 | – | 224 | – | 141 | – |
| sub_mod, constant time | 1024 | 321 | – | 210 | – | 533 | – | 274 | – |
| sub_mod, constant time | 2048 | 743 | – | 510 | – | 1,322 | – | 639 | – |
| sub_mod, constant time | 4096 | 1,549 | – | 1,421 | – | 1,751 | – | 1,682 | – |
| add_mod, reduced operands, constant time | 64 | 5.8 vs crypto-bigint 13.0 | −7.2 (0.45×) | 6.0 vs crypto-bigint 12.2 | −6.2 (0.49×) | 6.1 vs crypto-bigint 11.9 | −5.8 (0.51×) | 7.4 vs crypto-bigint 13.2 | −5.8 (0.56×) |
| add_mod, reduced operands, constant time | 128 | 13.6 vs crypto-bigint 23.4 | −9.8 (0.58×) | 12.0 vs crypto-bigint 21.1 | −9.1 (0.57×) | 13.4 vs crypto-bigint 23.4 | −10.0 (0.57×) | 18.0 vs crypto-bigint 22.8 | −4.8 (0.79×) |
| add_mod, reduced operands, constant time | 256 | 29.1 vs crypto-bigint 47.6 | −18.5 (0.61×) | 23.7 vs crypto-bigint 43.1 | −19.4 (0.55×) | 31.5 vs crypto-bigint 57.3 | −25.8 (0.55×) | 32.1 vs crypto-bigint 54.8 | −22.7 (0.59×) |
| add_mod, reduced operands, constant time | 384 | 48.1 vs crypto-bigint 97.2 | −49.1 (0.49×) | 36.7 vs crypto-bigint 105 | −68.3 (0.35×) | 50.5 vs crypto-bigint 87.4 | −36.9 (0.58×) | 47.3 vs crypto-bigint 82.4 | −35.1 (0.57×) |
| add_mod, reduced operands, constant time | 512 | 72.2 vs crypto-bigint 109 | −36.8 (0.66×) | 49.8 vs crypto-bigint 94.2 | −44.4 (0.53×) | 85.2 vs crypto-bigint 141 | −55.8 (0.60×) | 61.5 vs crypto-bigint 128 | −66.5 (0.48×) |
| add_mod, reduced operands, constant time | 1024 | 138 vs crypto-bigint 379 | −241 (0.36×) | 96.4 vs crypto-bigint 344 | −248 (0.28×) | 295 vs crypto-bigint 464 | −169 (0.64×) | 122 vs crypto-bigint 383 | −261 (0.32×) |
| add_mod, reduced operands, constant time | 2048 | 280 vs crypto-bigint 565 | −285 (0.50×) | 183 vs crypto-bigint 507 | −324 (0.36×) | 653 vs crypto-bigint 879 | −226 (0.74×) | 289 vs crypto-bigint 770 | −481 (0.38×) |
| add_mod, reduced operands, constant time | 4096 | 812 vs crypto-bigint 1,190 | −378 (0.68×) | 748 vs crypto-bigint 1,052 | −304 (0.71×) | 862 vs crypto-bigint 1,608 | −746 (0.54×) | 876 vs crypto-bigint 1,446 | −570 (0.61×) |
| sub_mod, reduced operands, constant time | 64 | 4.8 vs crypto-bigint 5.0 | −0.2 (0.96×) | 4.9 vs crypto-bigint 5.0 | −0.1 (0.98×) | 5.1 vs crypto-bigint 5.2 | −0.1 (0.98×) | 5.1 vs crypto-bigint 5.0 | +0.1 (1.02×) |
| sub_mod, reduced operands, constant time | 128 | 9.7 vs crypto-bigint 10.2 | −0.5 (0.95×) | 9.0 vs crypto-bigint 11.0 | −2.0 (0.82×) | 9.0 vs crypto-bigint 10.6 | −1.6 (0.85×) | 12.0 vs crypto-bigint 11.0 | +1.0 (1.09×) |
| sub_mod, reduced operands, constant time | 256 | 24.3 vs crypto-bigint 28.1 | −3.8 (0.86×) | 20.3 vs crypto-bigint 23.6 | −3.3 (0.86×) | 30.3 vs crypto-bigint 31.7 | −1.4 (0.96×) | 28.1 vs crypto-bigint 30.1 | −2.0 (0.93×) |
| sub_mod, reduced operands, constant time | 384 | 40.6 vs crypto-bigint 48.9 | −8.3 (0.83×) | 33.3 vs crypto-bigint 39.1 | −5.8 (0.85×) | 50.0 vs crypto-bigint 57.4 | −7.4 (0.87×) | 44.7 vs crypto-bigint 52.3 | −7.6 (0.85×) |
| sub_mod, reduced operands, constant time | 512 | 65.4 vs crypto-bigint 71.6 | −6.2 (0.91×) | 46.5 vs crypto-bigint 53.8 | −7.3 (0.86×) | 92.1 vs crypto-bigint 94.0 | −1.9 (0.98×) | 63.9 vs crypto-bigint 106 | −42.1 (0.60×) |
| sub_mod, reduced operands, constant time | 1024 | 133 vs crypto-bigint 177 | −44.0 (0.75×) | 114 vs crypto-bigint 139 | −25.0 (0.82×) | 215 vs crypto-bigint 347 | −132 (0.62×) | 137 vs crypto-bigint 276 | −139 (0.50×) |
| sub_mod, reduced operands, constant time | 2048 | 294 vs crypto-bigint 523 | −229 (0.56×) | 255 vs crypto-bigint 457 | −202 (0.56×) | 553 vs crypto-bigint 561 | −8.0 (0.99×) | 293 vs crypto-bigint 495 | −202 (0.59×) |
| sub_mod, reduced operands, constant time | 4096 | 849 vs crypto-bigint 912 | −63.0 (0.93×) | 711 vs crypto-bigint 771 | −60.0 (0.92×) | 903 vs crypto-bigint 1,465 | −562 (0.62×) | 914 vs crypto-bigint 1,344 | −430 (0.68×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.3 vs native 1.1 | +0.2 (1.18×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs ruint 1.0 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add | 128 | 3.1 vs native 2.8 | +0.3 (1.11×) | 2.5 vs bnum 2.1 | +0.4 (1.19×) | 2.7 vs crypto-bigint 2.7 | +0.0 (1.00×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add | 256 | 9.9 vs bnum 8.1 | +1.8 (1.22×) | 8.2 vs bnum 6.3 | +1.9 (1.30×) | 8.9 vs bnum 9.0 | −0.1 (0.99×) | 11.4 vs bnum 8.0 | +3.4 (1.43×) |
| add | 384 | 14.6 vs bnum 16.7 | −2.1 (0.87×) | 12.2 vs crypto-bigint 12.3 | −0.1 (0.99×) | 16.1 vs ruint 15.7 | +0.4 (1.03×) | 18.1 vs bnum 17.9 | +0.2 (1.01×) |
| add | 512 | 23.6 vs bnum 22.5 | +1.1 (1.05×) | 18.3 vs crypto-bigint 18.0 | +0.3 (1.02×) | 25.3 vs bnum 22.5 | +2.8 (1.12×) | 31.2 vs bnum 28.1 | +3.1 (1.11×) |
| add | 1024 | 55.4 vs bnum 62.3 | −6.9 (0.89×) | 47.3 vs crypto-bigint 45.6 | +1.7 (1.04×) | 60.0 vs ruint 72.6 | −12.6 (0.83×) | 67.5 vs crypto-bigint 61.6 | +5.9 (1.10×) |
| add | 2048 | 117 vs ruint 204 | −87.0 (0.57×) | 107 vs ruint 162 | −55.0 (0.66×) | 125 vs ruint 216 | −91.0 (0.58×) | 142 vs ruint 208 | −66.0 (0.68×) |
| add | 4096 | 352 vs ruint 412 | −60.0 (0.85×) | 248 vs ruint 297 | −49.0 (0.84×) | 385 vs ruint 426 | −41.0 (0.90×) | 301 vs ruint 383 | −82.0 (0.79×) |
| mulmod | 64 | 68.0 vs ruint 85.2 | −17.2 (0.80×) | 35.8 vs ruint 43.3 | −7.5 (0.83×) | 113 vs ruint 182 | −69.0 (0.62×) | 41.5 vs ruint 54.1 | −12.6 (0.77×) |
| mulmod | 128 | 209 vs ruint 339 | −130 (0.62×) | 87.1 vs ruint 137 | −49.9 (0.64×) | 262 vs ruint 388 | −126 (0.68×) | 140 vs ruint 207 | −67.0 (0.68×) |
| mulmod | 256 | 575 vs ruint 813 | −238 (0.71×) | 279 vs ruint 309 | −30.0 (0.90×) | 692 vs ruint 901 | −209 (0.77×) | 345 vs ruint 380 | −35.0 (0.91×) |
| mulmod | 384 | 962 vs ruint 1,603 | −641 (0.60×) | 513 vs ruint 533 | −20.0 (0.96×) | 1,316 vs ruint 1,706 | −390 (0.77×) | 606 vs ruint 691 | −85.0 (0.88×) |
| mulmod | 512 | 1,575 vs ruint 2,770 | −1,195 (0.57×) | 866 vs ruint 856 | +10.0 (1.01×) | 2,060 vs ruint 3,254 | −1,194 (0.63×) | 1,076 vs ruint 1,115 | −39.0 (0.97×) |
| mulmod | 1024 | 5,328 vs ruint 10,803 | −5,475 (0.49×) | 2,754 vs ruint 2,608 | +146 (1.06×) | 6,822 vs ruint 12,692 | −5,870 (0.54×) | 3,527 vs ruint 3,637 | −110 (0.97×) |
| mulmod | 2048 | 17,427 vs ruint 34,886 | −17,459 (0.50×) | 8,364 vs ruint 10,740 | −2,376 (0.78×) | 22,424 vs ruint 44,238 | −21,814 (0.51×) | 13,175 vs ruint 13,760 | −585 (0.96×) |
| mulmod | 4096 | 55,946 vs ruint 160,386 | −104,440 (0.35×) | 27,016 vs ruint 41,317 | −14,301 (0.65×) | 68,443 vs ruint 170,597 | −102,154 (0.40×) | 45,374 vs ruint 53,475 | −8,101 (0.85×) |
| add_mod | 64 | 6.4 vs ruint 40.4 | −34.0 (0.16×) | 6.3 vs ruint 41.0 | −34.7 (0.15×) | 9.5 vs ruint 77.7 | −68.2 (0.12×) | 9.2 vs ruint 77.0 | −67.8 (0.12×) |
| add_mod | 128 | 31.7 vs ruint 39.1 | −7.4 (0.81×) | 11.0 vs ruint 31.2 | −20.2 (0.35×) | 41.8 vs ruint 53.8 | −12.0 (0.78×) | 16.2 vs ruint 40.1 | −23.9 (0.40×) |
| add_mod | 256 | 41.0 vs ruint 81.9 | −40.9 (0.50×) | 31.7 vs ruint 62.2 | −30.5 (0.51×) | 56.2 vs ruint 99.9 | −43.7 (0.56×) | 43.5 vs ruint 76.3 | −32.8 (0.57×) |
| add_mod | 384 | 55.2 vs ruint 107 | −51.8 (0.52×) | 47.2 vs ruint 77.4 | −30.2 (0.61×) | 83.7 vs ruint 150 | −66.3 (0.56×) | 63.4 vs ruint 103 | −39.6 (0.62×) |
| add_mod | 512 | 66.6 vs ruint 134 | −67.4 (0.50×) | 55.9 vs ruint 105 | −49.1 (0.53×) | 95.3 vs ruint 158 | −62.7 (0.60×) | 83.5 vs ruint 140 | −56.5 (0.60×) |
| add_mod | 1024 | 145 vs ruint 281 | −136 (0.52×) | 122 vs ruint 208 | −86.0 (0.59×) | 157 vs ruint 396 | −239 (0.40×) | 157 vs ruint 338 | −181 (0.46×) |
| add_mod | 2048 | 307 vs ruint 910 | −603 (0.34×) | 237 vs ruint 799 | −562 (0.30×) | 375 vs ruint 1,194 | −819 (0.31×) | 315 vs ruint 962 | −647 (0.33×) |
| add_mod | 4096 | 865 vs ruint 1,765 | −900 (0.49×) | 764 vs ruint 1,457 | −693 (0.52×) | 912 vs ruint 2,003 | −1,091 (0.46×) | 921 vs ruint 1,678 | −757 (0.55×) |
| sub_mod | 64 | 4.8 vs ruint 48.2 | −43.4 (0.10×) | 7.2 vs ruint 46.2 | −39.0 (0.16×) | 8.7 vs ruint 75.6 | −66.9 (0.12×) | 9.1 vs ruint 75.9 | −66.8 (0.12×) |
| sub_mod | 128 | 33.8 vs ruint 46.6 | −12.8 (0.73×) | 11.3 vs ruint 41.2 | −29.9 (0.27×) | 45.6 vs ruint 65.7 | −20.1 (0.69×) | 15.8 vs ruint 52.3 | −36.5 (0.30×) |
| sub_mod | 256 | 46.4 vs ruint 96.9 | −50.5 (0.48×) | 33.8 vs ruint 81.3 | −47.5 (0.42×) | 60.4 vs ruint 130 | −69.6 (0.46×) | 44.1 vs ruint 116 | −71.9 (0.38×) |
| sub_mod | 384 | 52.7 vs ruint 126 | −73.3 (0.42×) | 44.5 vs ruint 109 | −64.5 (0.41×) | 85.4 vs ruint 194 | −109 (0.44×) | 64.5 vs ruint 151 | −86.5 (0.43×) |
| sub_mod | 512 | 71.6 vs ruint 182 | −110 (0.39×) | 53.3 vs ruint 140 | −86.7 (0.38×) | 103 vs ruint 252 | −149 (0.41×) | 87.4 vs ruint 211 | −124 (0.41×) |
| sub_mod | 1024 | 116 vs ruint 447 | −331 (0.26×) | 105 vs ruint 370 | −265 (0.28×) | 179 vs ruint 667 | −488 (0.27×) | 167 vs ruint 603 | −436 (0.28×) |
| sub_mod | 2048 | 281 vs ruint 1,229 | −948 (0.23×) | 260 vs ruint 1,104 | −844 (0.24×) | 379 vs ruint 1,450 | −1,071 (0.26×) | 358 vs ruint 1,399 | −1,041 (0.26×) |
| sub_mod | 4096 | 873 vs ruint 2,334 | −1,461 (0.37×) | 820 vs ruint 2,075 | −1,255 (0.40×) | 923 vs ruint 2,711 | −1,788 (0.34×) | 1,099 vs ruint 2,397 | −1,298 (0.46×) |
| add_mod, reduced operands | 64 | 5.0 vs ruint 44.0 | −39.0 (0.11×) | 5.0 vs ruint 44.4 | −39.4 (0.11×) | 5.0 vs ruint 87.2 | −82.2 (0.06×) | 5.0 vs ruint 83.4 | −78.4 (0.06×) |
| add_mod, reduced operands | 128 | 6.7 vs ruint 23.5 | −16.8 (0.29×) | 7.6 vs ruint 23.8 | −16.2 (0.32×) | 7.6 vs ruint 24.4 | −16.8 (0.31×) | 10.4 vs ruint 25.0 | −14.6 (0.42×) |
| add_mod, reduced operands | 256 | 17.0 vs ruint 43.0 | −26.0 (0.40×) | 13.1 vs ruint 43.0 | −29.9 (0.30×) | 16.8 vs ruint 60.5 | −43.7 (0.28×) | 21.3 vs ruint 59.3 | −38.0 (0.36×) |
| add_mod, reduced operands | 384 | 29.5 vs ruint 61.8 | −32.3 (0.48×) | 19.6 vs ruint 59.9 | −40.3 (0.33×) | 28.5 vs ruint 92.9 | −64.4 (0.31×) | 32.1 vs ruint 81.0 | −48.9 (0.40×) |
| add_mod, reduced operands | 512 | 35.6 vs ruint 86.1 | −50.5 (0.41×) | 28.4 vs ruint 82.0 | −53.6 (0.35×) | 43.0 vs ruint 116 | −73.0 (0.37×) | 43.3 vs ruint 111 | −67.7 (0.39×) |
| add_mod, reduced operands | 1024 | 76.9 vs ruint 195 | −118 (0.39×) | 69.1 vs ruint 182 | −113 (0.38×) | 88.9 vs ruint 264 | −175 (0.34×) | 93.3 vs ruint 251 | −158 (0.37×) |
| add_mod, reduced operands | 2048 | 204 vs ruint 817 | −613 (0.25×) | 157 vs ruint 755 | −598 (0.21×) | 231 vs ruint 1,036 | −805 (0.22×) | 205 vs ruint 918 | −713 (0.22×) |
| add_mod, reduced operands | 4096 | 548 vs ruint 1,546 | −998 (0.35×) | 440 vs ruint 1,434 | −994 (0.31×) | 629 vs ruint 1,871 | −1,242 (0.34×) | 520 vs ruint 1,658 | −1,138 (0.31×) |
| sub_mod, reduced operands | 64 | 2.7 vs ruint 46.2 | −43.5 (0.06×) | 2.7 vs ruint 44.9 | −42.2 (0.06×) | 2.6 vs ruint 83.4 | −80.8 (0.03×) | 3.5 vs ruint 83.4 | −79.9 (0.04×) |
| sub_mod, reduced operands | 128 | 5.4 vs ruint 25.5 | −20.1 (0.21×) | 6.1 vs ruint 24.2 | −18.1 (0.25×) | 5.4 vs ruint 26.4 | −21.0 (0.20×) | 8.8 vs ruint 27.0 | −18.2 (0.33×) |
| sub_mod, reduced operands | 256 | 15.0 vs ruint 49.6 | −34.6 (0.30×) | 12.0 vs ruint 50.7 | −38.7 (0.24×) | 13.3 vs ruint 65.9 | −52.6 (0.20×) | 19.0 vs ruint 66.7 | −47.7 (0.28×) |
| sub_mod, reduced operands | 384 | 25.3 vs ruint 77.1 | −51.8 (0.33×) | 19.1 vs ruint 74.7 | −55.6 (0.26×) | 25.3 vs ruint 104 | −78.7 (0.24×) | 30.0 vs ruint 94.4 | −64.4 (0.32×) |
| sub_mod, reduced operands | 512 | 34.9 vs ruint 110 | −75.1 (0.32×) | 29.2 vs ruint 103 | −73.8 (0.28×) | 40.6 vs ruint 131 | −90.4 (0.31×) | 41.9 vs ruint 134 | −92.1 (0.31×) |
| sub_mod, reduced operands | 1024 | 68.6 vs ruint 262 | −193 (0.26×) | 68.0 vs ruint 237 | −169 (0.29×) | 80.2 vs ruint 318 | −238 (0.25×) | 92.3 vs ruint 308 | −216 (0.30×) |
| sub_mod, reduced operands | 2048 | 213 vs ruint 979 | −766 (0.22×) | 172 vs ruint 915 | −743 (0.19×) | 299 vs ruint 1,225 | −926 (0.24×) | 205 vs ruint 1,075 | −870 (0.19×) |
| sub_mod, reduced operands | 4096 | 570 vs ruint 1,900 | −1,330 (0.30×) | 450 vs ruint 1,747 | −1,297 (0.26×) | 610 vs ruint 2,169 | −1,559 (0.28×) | 571 vs ruint 1,976 | −1,405 (0.29×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.0 | +0.1 (1.10×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add, constant time | 128 | 3.0 vs bnum 2.7 | +0.3 (1.11×) | 2.5 vs bnum 2.1 | +0.4 (1.19×) | 2.7 vs crypto-bigint 2.7 | +0.0 (1.00×) | 3.2 vs bnum 2.6 | +0.6 (1.23×) |
| add, constant time | 256 | 9.8 vs bnum 8.3 | +1.5 (1.18×) | 8.6 vs bnum 6.3 | +2.3 (1.37×) | 8.8 vs ruint 9.0 | −0.2 (0.98×) | 11.3 vs bnum 8.0 | +3.3 (1.41×) |
| add, constant time | 384 | 14.9 vs ruint 15.4 | −0.5 (0.97×) | 12.2 vs crypto-bigint 12.2 | +0.0 (1.00×) | 16.1 vs ruint 15.5 | +0.6 (1.04×) | 18.1 vs crypto-bigint 18.0 | +0.1 (1.01×) |
| add, constant time | 512 | 23.8 vs bnum 23.6 | +0.2 (1.01×) | 18.2 vs ruint 18.9 | −0.7 (0.96×) | 25.3 vs bnum 22.6 | +2.7 (1.12×) | 31.3 vs bnum 28.7 | +2.6 (1.09×) |
| add, constant time | 1024 | 55.3 vs bnum 62.3 | −7.0 (0.89×) | 47.2 vs crypto-bigint 45.5 | +1.7 (1.04×) | 59.5 vs ruint 72.9 | −13.4 (0.82×) | 67.7 vs crypto-bigint 61.6 | +6.1 (1.10×) |
| add, constant time | 2048 | 115 vs ruint 202 | −87.0 (0.57×) | 106 vs ruint 163 | −57.0 (0.65×) | 125 vs ruint 224 | −99.0 (0.56×) | 141 vs ruint 206 | −65.0 (0.68×) |
| add, constant time | 4096 | 354 vs ruint 408 | −54.0 (0.87×) | 248 vs ruint 297 | −49.0 (0.84×) | 401 vs ruint 425 | −24.0 (0.94×) | 305 vs ruint 393 | −88.0 (0.78×) |
| mulmod, constant time | 64 | 131 | – | 94.3 | – | 243 | – | 110 | – |
| mulmod, constant time | 128 | 572 | – | 367 | – | 603 | – | 371 | – |
| mulmod, constant time | 256 | 1,189 | – | 745 | – | 1,273 | – | 844 | – |
| mulmod, constant time | 384 | 1,929 | – | 1,208 | – | 2,273 | – | 1,490 | – |
| mulmod, constant time | 512 | 3,013 | – | 1,849 | – | 3,546 | – | 2,294 | – |
| mulmod, constant time | 1024 | 9,551 | – | 5,652 | – | 10,948 | – | 7,484 | – |
| mulmod, constant time | 2048 | 32,887 | – | 19,411 | – | 37,045 | – | 25,512 | – |
| mulmod, constant time | 4096 | 120,362 | – | 68,757 | – | 132,524 | – | 91,127 | – |
| add_mod, constant time | 64 | 12.3 | – | 13.0 | – | 18.8 | – | 14.0 | – |
| add_mod, constant time | 128 | 28.2 | – | 22.5 | – | 43.3 | – | 34.1 | – |
| add_mod, constant time | 256 | 83.7 | – | 67.1 | – | 98.7 | – | 92.7 | – |
| add_mod, constant time | 384 | 112 | – | 108 | – | 178 | – | 127 | – |
| add_mod, constant time | 512 | 151 | – | 135 | – | 246 | – | 168 | – |
| add_mod, constant time | 1024 | 327 | – | 260 | – | 621 | – | 350 | – |
| add_mod, constant time | 2048 | 868 | – | 519 | – | 1,605 | – | 758 | – |
| add_mod, constant time | 4096 | 1,876 | – | 1,684 | – | 2,043 | – | 2,049 | – |
| sub_mod, constant time | 64 | 12.1 | – | 11.0 | – | 18.3 | – | 14.1 | – |
| sub_mod, constant time | 128 | 24.0 | – | 19.0 | – | 31.9 | – | 23.0 | – |
| sub_mod, constant time | 256 | 54.7 | – | 37.0 | – | 76.1 | – | 57.5 | – |
| sub_mod, constant time | 384 | 103 | – | 57.5 | – | 122 | – | 85.4 | – |
| sub_mod, constant time | 512 | 142 | – | 85.9 | – | 199 | – | 117 | – |
| sub_mod, constant time | 1024 | 310 | – | 200 | – | 534 | – | 254 | – |
| sub_mod, constant time | 2048 | 742 | – | 512 | – | 1,278 | – | 584 | – |
| sub_mod, constant time | 4096 | 1,563 | – | 1,422 | – | 1,737 | – | 1,715 | – |
| add_mod, reduced operands, constant time | 64 | 8.1 | – | 8.0 | – | 8.0 | – | 8.5 | – |
| add_mod, reduced operands, constant time | 128 | 15.0 | – | 13.2 | – | 14.7 | – | 16.2 | – |
| add_mod, reduced operands, constant time | 256 | 30.9 | – | 23.1 | – | 28.1 | – | 31.6 | – |
| add_mod, reduced operands, constant time | 384 | 51.7 | – | 35.0 | – | 51.6 | – | 49.3 | – |
| add_mod, reduced operands, constant time | 512 | 68.6 | – | 50.4 | – | 87.4 | – | 64.2 | – |
| add_mod, reduced operands, constant time | 1024 | 142 | – | 91.1 | – | 305 | – | 144 | – |
| add_mod, reduced operands, constant time | 2048 | 277 | – | 184 | – | 669 | – | 298 | – |
| add_mod, reduced operands, constant time | 4096 | 821 | – | 716 | – | 868 | – | 825 | – |
| sub_mod, reduced operands, constant time | 64 | 4.0 | – | 5.0 | – | 4.0 | – | 5.1 | – |
| sub_mod, reduced operands, constant time | 128 | 10.6 | – | 10.0 | – | 10.0 | – | 10.1 | – |
| sub_mod, reduced operands, constant time | 256 | 23.1 | – | 19.1 | – | 31.0 | – | 28.0 | – |
| sub_mod, reduced operands, constant time | 384 | 42.1 | – | 32.7 | – | 50.1 | – | 43.5 | – |
| sub_mod, reduced operands, constant time | 512 | 59.8 | – | 44.2 | – | 91.2 | – | 62.4 | – |
| sub_mod, reduced operands, constant time | 1024 | 129 | – | 112 | – | 232 | – | 141 | – |
| sub_mod, reduced operands, constant time | 2048 | 300 | – | 257 | – | 570 | – | 304 | – |
| sub_mod, reduced operands, constant time | 4096 | 852 | – | 690 | – | 874 | – | 884 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.4 | 2.1 (1.5×) | 2.2 (1.6×) | 2.4 (1.7×) | 2.3 (1.6×) |
| add | 128 | 2.4 | 4.2 (1.8×) | 3.9 (1.6×) | 3.9 (1.6×) | 4.3 (1.8×) |
| add | 256 | 4.7 | 9.7 (2.1×) | 8.8 (1.9×) | 10.0 (2.1×) | 11.5 (2.4×) |
| add | 384 | 7.1 | 16.5 (2.3×) | 12.8 (1.8×) | 15.3 (2.2×) | 18.0 (2.5×) |
| add | 512 | 9.2 | 26.3 (2.9×) | 20.8 (2.3×) | 26.0 (2.8×) | 32.3 (3.5×) |
| add | 1024 | 23.3 | 55.4 (2.4×) | 45.4 (1.9×) | 72.5 (3.1×) | 69.9 (3.0×) |
| add | 2048 | 53.5 | 115 (2.1×) | 108 (2.0×) | 143 (2.7×) | 149 (2.8×) |
| add | 4096 | 158 | 349 (2.2×) | 263 (1.7×) | 402 (2.5×) | 322 (2.0×) |
| mul | 64 | 1.5 | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul | 128 | 3.0 | 10.0 (3.3×) | 7.1 (2.4×) | 12.3 (4.1×) | 5.7 (1.9×) |
| mul | 256 | 12.2 | 48.0 (3.9×) | 15.6 (1.3×) | 67.7 (5.5×) | 26.0 (2.1×) |
| mul | 384 | 27.7 | 117 (4.2×) | 54.2 (2.0×) | 166 (6.0×) | 72.8 (2.6×) |
| mul | 512 | 52.1 | 207 (4.0×) | 78.1 (1.5×) | 307 (5.9×) | 132 (2.5×) |
| mul | 1024 | 221 | 1,261 (5.7×) | 317 (1.4×) | 1,491 (6.7×) | 568 (2.6×) |
| mul | 2048 | 1,113 | 5,611 (5.0×) | 2,320 (2.1×) | 6,714 (6.0×) | 3,522 (3.2×) |
| mul | 4096 | 3,859 | 17,771 (4.6×) | 7,925 (2.1×) | 21,315 (5.5×) | 11,934 (3.1×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.3 | 59.7 (3.7×) | 28.5 (1.7×) | 83.7 (5.1×) | 36.1 (2.2×) |
| div | 256 | 46.0 | 210 (4.6×) | 84.1 (1.8×) | 221 (4.8×) | 110 (2.4×) |
| div | 384 | 111 | 354 (3.2×) | 207 (1.9×) | 405 (3.6×) | 285 (2.6×) |
| div | 512 | 155 | 474 (3.1×) | 306 (2.0×) | 573 (3.7×) | 414 (2.7×) |
| div | 1024 | 462 | 1,144 (2.5×) | 775 (1.7×) | 1,477 (3.2×) | 868 (1.9×) |
| div | 2048 | 1,388 | 3,234 (2.3×) | 2,037 (1.5×) | 4,342 (3.1×) | 2,257 (1.6×) |
| div | 4096 | 4,594 | 12,808 (2.8×) | 7,115 (1.5×) | 15,198 (3.3×) | 7,494 (1.6×) |
| mod | 64 | 6.9 | 7.0 (1.0×) | 7.1 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 17.0 | 52.8 (3.1×) | 30.0 (1.8×) | 92.6 (5.4×) | 33.7 (2.0×) |
| mod | 256 | 41.5 | 208 (5.0×) | 74.5 (1.8×) | 224 (5.4×) | 109 (2.6×) |
| mod | 384 | 119 | 347 (2.9×) | 198 (1.7×) | 410 (3.4×) | 266 (2.2×) |
| mod | 512 | 158 | 467 (3.0×) | 285 (1.8×) | 548 (3.5×) | 350 (2.2×) |
| mod | 1024 | 485 | 1,195 (2.5×) | 717 (1.5×) | 1,476 (3.0×) | 905 (1.9×) |
| mod | 2048 | 1,407 | 3,311 (2.4×) | 1,934 (1.4×) | 4,367 (3.1×) | 2,114 (1.5×) |
| mod | 4096 | 4,562 | 12,974 (2.8×) | 6,515 (1.4×) | 15,422 (3.4×) | 7,649 (1.7×) |
| mulmod | 64 | 11.9 | 61.4 (5.2×) | 31.2 (2.6×) | 107 (9.0×) | 36.1 (3.0×) |
| mulmod | 128 | 47.4 | 201 (4.2×) | 82.2 (1.7×) | 240 (5.1×) | 127 (2.7×) |
| mulmod | 256 | 162 | 581 (3.6×) | 284 (1.8×) | 701 (4.3×) | 360 (2.2×) |
| mulmod | 384 | 297 | 977 (3.3×) | 515 (1.7×) | 1,345 (4.5×) | 628 (2.1×) |
| mulmod | 512 | 467 | 1,583 (3.4×) | 884 (1.9×) | 2,154 (4.6×) | 1,074 (2.3×) |
| mulmod | 1024 | 1,365 | 5,346 (3.9×) | 2,737 (2.0×) | 6,727 (4.9×) | 3,605 (2.6×) |
| mulmod | 2048 | 5,191 | 17,492 (3.4×) | 8,747 (1.7×) | 22,288 (4.3×) | 13,109 (2.5×) |
| mulmod | 4096 | 14,910 | 56,007 (3.8×) | 26,793 (1.8×) | 69,330 (4.6×) | 46,093 (3.1×) |
| add_mod | 64 | 2.9 | 4.0 (1.4×) | 4.0 (1.4×) | 8.0 (2.8×) | 7.7 (2.7×) |
| add_mod | 128 | 4.8 | 9.1 (1.9×) | 9.2 (1.9×) | 12.5 (2.6×) | 13.7 (2.9×) |
| add_mod | 256 | 9.7 | 23.1 (2.4×) | 21.9 (2.3×) | 24.7 (2.5×) | 30.4 (3.1×) |
| add_mod | 384 | 15.6 | 30.9 (2.0×) | 28.5 (1.8×) | 46.8 (3.0×) | 45.2 (2.9×) |
| add_mod | 512 | 27.7 | 44.8 (1.6×) | 38.7 (1.4×) | 60.7 (2.2×) | 62.4 (2.3×) |
| add_mod | 1024 | 71.7 | 114 (1.6×) | 104 (1.5×) | 144 (2.0×) | 135 (1.9×) |
| add_mod | 2048 | 149 | 278 (1.9×) | 245 (1.6×) | 378 (2.5×) | 291 (2.0×) |
| add_mod | 4096 | 268 | 734 (2.7×) | 750 (2.8×) | 792 (3.0×) | 941 (3.5×) |
| sub_mod | 64 | 2.4 | 3.9 (1.6×) | 4.0 (1.7×) | 8.0 (3.3×) | 8.1 (3.4×) |
| sub_mod | 128 | 4.3 | 10.8 (2.5×) | 10.2 (2.4×) | 13.8 (3.2×) | 15.8 (3.7×) |
| sub_mod | 256 | 8.7 | 23.8 (2.7×) | 19.8 (2.3×) | 26.1 (3.0×) | 28.8 (3.3×) |
| sub_mod | 384 | 15.6 | 31.1 (2.0×) | 27.0 (1.7×) | 44.1 (2.8×) | 48.4 (3.1×) |
| sub_mod | 512 | 25.2 | 45.5 (1.8×) | 37.6 (1.5×) | 62.0 (2.5×) | 68.0 (2.7×) |
| sub_mod | 1024 | 66.3 | 105 (1.6×) | 110 (1.7×) | 128 (1.9×) | 134 (2.0×) |
| sub_mod | 2048 | 146 | 237 (1.6×) | 249 (1.7×) | 287 (2.0×) | 277 (1.9×) |
| sub_mod | 4096 | 241 | 725 (3.0×) | 775 (3.2×) | 846 (3.5×) | 965 (4.0×) |
| add_mod, reduced operands | 64 | 2.8 | 4.2 (1.5×) | 4.1 (1.5×) | 4.5 (1.6×) | 4.3 (1.5×) |
| add_mod, reduced operands | 128 | 4.3 | 8.3 (1.9×) | 9.0 (2.1×) | 8.2 (1.9×) | 11.5 (2.7×) |
| add_mod, reduced operands | 256 | 10.4 | 16.9 (1.6×) | 17.0 (1.6×) | 18.7 (1.8×) | 22.5 (2.2×) |
| add_mod, reduced operands | 384 | 13.8 | 27.8 (2.0×) | 22.2 (1.6×) | 28.2 (2.0×) | 32.4 (2.3×) |
| add_mod, reduced operands | 512 | 19.6 | 34.8 (1.8×) | 28.3 (1.4×) | 41.4 (2.1×) | 42.3 (2.2×) |
| add_mod, reduced operands | 1024 | 48.3 | 77.2 (1.6×) | 68.9 (1.4×) | 88.7 (1.8×) | 92.1 (1.9×) |
| add_mod, reduced operands | 2048 | 89.7 | 198 (2.2×) | 150 (1.7×) | 233 (2.6×) | 220 (2.5×) |
| add_mod, reduced operands | 4096 | 227 | 535 (2.4×) | 462 (2.0×) | 585 (2.6×) | 525 (2.3×) |
| sub_mod, reduced operands | 64 | 2.2 | 3.6 (1.6×) | 4.0 (1.8×) | 4.7 (2.1×) | 6.2 (2.8×) |
| sub_mod, reduced operands | 128 | 4.7 | 7.1 (1.5×) | 7.3 (1.6×) | 7.7 (1.6×) | 9.6 (2.0×) |
| sub_mod, reduced operands | 256 | 7.3 | 17.0 (2.3×) | 13.2 (1.8×) | 14.9 (2.0×) | 20.2 (2.8×) |
| sub_mod, reduced operands | 384 | 11.0 | 26.5 (2.4×) | 20.8 (1.9×) | 25.6 (2.3×) | 30.4 (2.8×) |
| sub_mod, reduced operands | 512 | 18.3 | 34.4 (1.9×) | 29.8 (1.6×) | 37.7 (2.1×) | 41.8 (2.3×) |
| sub_mod, reduced operands | 1024 | 44.7 | 66.6 (1.5×) | 68.0 (1.5×) | 83.3 (1.9×) | 90.2 (2.0×) |
| sub_mod, reduced operands | 2048 | 83.2 | 209 (2.5×) | 157 (1.9×) | 280 (3.4×) | 199 (2.4×) |
| sub_mod, reduced operands | 4096 | 217 | 537 (2.5×) | 469 (2.2×) | 588 (2.7×) | 589 (2.7×) |
| add, constant time | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.2 (1.8×) |
| add, constant time | 256 | 4.5 | 10.2 (2.3×) | 9.0 (2.0×) | 10.1 (2.2×) | 11.5 (2.6×) |
| add, constant time | 384 | 7.0 | 16.0 (2.3×) | 12.5 (1.8×) | 15.2 (2.2×) | 18.1 (2.6×) |
| add, constant time | 512 | 9.5 | 27.9 (2.9×) | 20.9 (2.2×) | 26.3 (2.8×) | 32.3 (3.4×) |
| add, constant time | 1024 | 23.4 | 59.7 (2.6×) | 45.8 (2.0×) | 71.1 (3.0×) | 69.8 (3.0×) |
| add, constant time | 2048 | 53.3 | 117 (2.2×) | 109 (2.0×) | 144 (2.7×) | 150 (2.8×) |
| add, constant time | 4096 | 162 | 344 (2.1×) | 267 (1.6×) | 400 (2.5×) | 319 (2.0×) |
| mul, constant time | 64 | 1.5 | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul, constant time | 128 | 3.0 | 10.1 (3.4×) | 6.7 (2.2×) | 12.3 (4.1×) | 5.7 (1.9×) |
| mul, constant time | 256 | 12.4 | 47.9 (3.9×) | 22.7 (1.8×) | 67.1 (5.4×) | 26.0 (2.1×) |
| mul, constant time | 384 | 30.0 | 116 (3.9×) | 55.9 (1.9×) | 168 (5.6×) | 72.4 (2.4×) |
| mul, constant time | 512 | 50.9 | 210 (4.1×) | 74.9 (1.5×) | 307 (6.0×) | 132 (2.6×) |
| mul, constant time | 1024 | 226 | 1,193 (5.3×) | 349 (1.5×) | 1,491 (6.6×) | 566 (2.5×) |
| mul, constant time | 2048 | 1,154 | 5,793 (5.0×) | 2,439 (2.1×) | 6,672 (5.8×) | 3,634 (3.1×) |
| mul, constant time | 4096 | 4,213 | 18,754 (4.5×) | 8,525 (2.0×) | 21,984 (5.2×) | 12,511 (3.0×) |
| div, constant time | 64 | 25.1 | 83.2 (3.3×) | 47.4 (1.9×) | 146 (5.8×) | 61.4 (2.4×) |
| div, constant time | 128 | 98.8 | 280 (2.8×) | 179 (1.8×) | 362 (3.7×) | 231 (2.3×) |
| div, constant time | 256 | 199 | 521 (2.6×) | 334 (1.7×) | 635 (3.2×) | 471 (2.4×) |
| div, constant time | 384 | 291 | 819 (2.8×) | 457 (1.6×) | 811 (2.8×) | 545 (1.9×) |
| div, constant time | 512 | 461 | 1,023 (2.2×) | 745 (1.6×) | 1,186 (2.6×) | 862 (1.9×) |
| div, constant time | 1024 | 1,139 | 2,475 (2.2×) | 1,796 (1.6×) | 2,881 (2.5×) | 2,187 (1.9×) |
| div, constant time | 2048 | 3,426 | 7,476 (2.2×) | 5,195 (1.5×) | 8,404 (2.5×) | 6,477 (1.9×) |
| div, constant time | 4096 | 11,846 | 25,658 (2.2×) | 16,426 (1.4×) | 27,938 (2.4×) | 21,618 (1.8×) |
| mod, constant time | 64 | 25.5 | 83.6 (3.3×) | 48.2 (1.9×) | 142 (5.6×) | 67.6 (2.7×) |
| mod, constant time | 128 | 105 | 347 (3.3×) | 169 (1.6×) | 332 (3.2×) | 226 (2.2×) |
| mod, constant time | 256 | 216 | 520 (2.4×) | 333 (1.5×) | 581 (2.7×) | 414 (1.9×) |
| mod, constant time | 384 | 311 | 748 (2.4×) | 537 (1.7×) | 805 (2.6×) | 560 (1.8×) |
| mod, constant time | 512 | 466 | 1,056 (2.3×) | 756 (1.6×) | 1,084 (2.3×) | 832 (1.8×) |
| mod, constant time | 1024 | 1,147 | 2,541 (2.2×) | 1,955 (1.7×) | 2,745 (2.4×) | 1,996 (1.7×) |
| mod, constant time | 2048 | 3,445 | 7,775 (2.3×) | 5,269 (1.5×) | 8,332 (2.4×) | 5,909 (1.7×) |
| mod, constant time | 4096 | 11,904 | 25,871 (2.2×) | 16,456 (1.4×) | 29,046 (2.4×) | 20,672 (1.7×) |
| mulmod, constant time | 64 | 51.9 | 203 (3.9×) | 99.7 (1.9×) | 237 (4.6×) | 113 (2.2×) |
| mulmod, constant time | 128 | 201 | 573 (2.9×) | 360 (1.8×) | 601 (3.0×) | 372 (1.9×) |
| mulmod, constant time | 256 | 453 | 1,218 (2.7×) | 744 (1.6×) | 1,348 (3.0×) | 818 (1.8×) |
| mulmod, constant time | 384 | 823 | 1,965 (2.4×) | 1,225 (1.5×) | 2,257 (2.7×) | 1,500 (1.8×) |
| mulmod, constant time | 512 | 1,189 | 3,009 (2.5×) | 1,824 (1.5×) | 3,495 (2.9×) | 2,307 (1.9×) |
| mulmod, constant time | 1024 | 3,856 | 9,611 (2.5×) | 5,616 (1.5×) | 10,989 (2.8×) | 7,558 (2.0×) |
| mulmod, constant time | 2048 | 13,721 | 32,861 (2.4×) | 18,903 (1.4×) | 36,878 (2.7×) | 25,971 (1.9×) |
| mulmod, constant time | 4096 | 48,937 | 119,911 (2.5×) | 68,830 (1.4×) | 132,639 (2.7×) | 91,464 (1.9×) |
| add_mod, constant time | 64 | 6.6 | 9.0 (1.4×) | 9.5 (1.4×) | 19.3 (2.9×) | 13.8 (2.1×) |
| add_mod, constant time | 128 | 11.0 | 26.5 (2.4×) | 21.5 (2.0×) | 34.5 (3.1×) | 35.0 (3.2×) |
| add_mod, constant time | 256 | 20.6 | 82.3 (4.0×) | 65.4 (3.2×) | 108 (5.2×) | 74.9 (3.6×) |
| add_mod, constant time | 384 | 38.7 | 120 (3.1×) | 111 (2.9×) | 167 (4.3×) | 141 (3.6×) |
| add_mod, constant time | 512 | 56.5 | 150 (2.7×) | 145 (2.6×) | 238 (4.2×) | 180 (3.2×) |
| add_mod, constant time | 1024 | 130 | 323 (2.5×) | 272 (2.1×) | 607 (4.7×) | 344 (2.6×) |
| add_mod, constant time | 2048 | 303 | 867 (2.9×) | 531 (1.8×) | 1,702 (5.6×) | 761 (2.5×) |
| add_mod, constant time | 4096 | 511 | 1,886 (3.7×) | 1,717 (3.4×) | 2,017 (3.9×) | 2,011 (3.9×) |
| sub_mod, constant time | 64 | 6.2 | 10.2 (1.6×) | 9.3 (1.5×) | 18.9 (3.0×) | 14.3 (2.3×) |
| sub_mod, constant time | 128 | 7.4 | 15.6 (2.1×) | 13.0 (1.8×) | 30.9 (4.2×) | 23.0 (3.1×) |
| sub_mod, constant time | 256 | 13.7 | 45.9 (3.4×) | 31.9 (2.3×) | 73.8 (5.4×) | 65.2 (4.8×) |
| sub_mod, constant time | 384 | 27.0 | 103 (3.8×) | 51.4 (1.9×) | 136 (5.0×) | 96.7 (3.6×) |
| sub_mod, constant time | 512 | 33.6 | 140 (4.2×) | 88.2 (2.6×) | 224 (6.7×) | 141 (4.2×) |
| sub_mod, constant time | 1024 | 96.9 | 321 (3.3×) | 210 (2.2×) | 533 (5.5×) | 274 (2.8×) |
| sub_mod, constant time | 2048 | 414 | 743 (1.8×) | 510 (1.2×) | 1,322 (3.2×) | 639 (1.5×) |
| sub_mod, constant time | 4096 | 487 | 1,549 (3.2×) | 1,421 (2.9×) | 1,751 (3.6×) | 1,682 (3.5×) |
| add_mod, reduced operands, constant time | 64 | 3.8 | 5.8 (1.5×) | 6.0 (1.6×) | 6.1 (1.6×) | 7.4 (1.9×) |
| add_mod, reduced operands, constant time | 128 | 6.9 | 13.6 (2.0×) | 12.0 (1.7×) | 13.4 (1.9×) | 18.0 (2.6×) |
| add_mod, reduced operands, constant time | 256 | 9.9 | 29.1 (2.9×) | 23.7 (2.4×) | 31.5 (3.2×) | 32.1 (3.2×) |
| add_mod, reduced operands, constant time | 384 | 13.2 | 48.1 (3.6×) | 36.7 (2.8×) | 50.5 (3.8×) | 47.3 (3.6×) |
| add_mod, reduced operands, constant time | 512 | 19.8 | 72.2 (3.6×) | 49.8 (2.5×) | 85.2 (4.3×) | 61.5 (3.1×) |
| add_mod, reduced operands, constant time | 1024 | 57.8 | 138 (2.4×) | 96.4 (1.7×) | 295 (5.1×) | 122 (2.1×) |
| add_mod, reduced operands, constant time | 2048 | 132 | 280 (2.1×) | 183 (1.4×) | 653 (4.9×) | 289 (2.2×) |
| add_mod, reduced operands, constant time | 4096 | 269 | 812 (3.0×) | 748 (2.8×) | 862 (3.2×) | 876 (3.3×) |
| sub_mod, reduced operands, constant time | 64 | 3.2 | 4.8 (1.5×) | 4.9 (1.5×) | 5.1 (1.6×) | 5.1 (1.6×) |
| sub_mod, reduced operands, constant time | 128 | 5.1 | 9.7 (1.9×) | 9.0 (1.8×) | 9.0 (1.8×) | 12.0 (2.4×) |
| sub_mod, reduced operands, constant time | 256 | 8.9 | 24.3 (2.7×) | 20.3 (2.3×) | 30.3 (3.4×) | 28.1 (3.2×) |
| sub_mod, reduced operands, constant time | 384 | 11.0 | 40.6 (3.7×) | 33.3 (3.0×) | 50.0 (4.5×) | 44.7 (4.1×) |
| sub_mod, reduced operands, constant time | 512 | 18.5 | 65.4 (3.5×) | 46.5 (2.5×) | 92.1 (5.0×) | 63.9 (3.5×) |
| sub_mod, reduced operands, constant time | 1024 | 48.5 | 133 (2.7×) | 114 (2.4×) | 215 (4.4×) | 137 (2.8×) |
| sub_mod, reduced operands, constant time | 2048 | 116 | 294 (2.5×) | 255 (2.2×) | 553 (4.8×) | 293 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 270 | 849 (3.1×) | 711 (2.6×) | 903 (3.3×) | 914 (3.4×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.3 (1.3×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add | 128 | 1.4 | 3.1 (2.2×) | 2.5 (1.8×) | 2.7 (1.9×) | 3.2 (2.3×) |
| add | 256 | 2.5 | 9.9 (4.0×) | 8.2 (3.3×) | 8.9 (3.6×) | 11.4 (4.6×) |
| add | 384 | 3.5 | 14.6 (4.2×) | 12.2 (3.5×) | 16.1 (4.6×) | 18.1 (5.2×) |
| add | 512 | 4.0 | 23.6 (5.9×) | 18.3 (4.6×) | 25.3 (6.3×) | 31.2 (7.8×) |
| add | 1024 | 22.1 | 55.4 (2.5×) | 47.3 (2.1×) | 60.0 (2.7×) | 67.5 (3.1×) |
| add | 2048 | 51.4 | 117 (2.3×) | 107 (2.1×) | 125 (2.4×) | 142 (2.8×) |
| add | 4096 | 163 | 352 (2.2×) | 248 (1.5×) | 385 (2.4×) | 301 (1.8×) |
| mulmod | 64 | 23.0 | 68.0 (3.0×) | 35.8 (1.6×) | 113 (4.9×) | 41.5 (1.8×) |
| mulmod | 128 | 50.7 | 209 (4.1×) | 87.1 (1.7×) | 262 (5.2×) | 140 (2.8×) |
| mulmod | 256 | 183 | 575 (3.1×) | 279 (1.5×) | 692 (3.8×) | 345 (1.9×) |
| mulmod | 384 | 316 | 962 (3.0×) | 513 (1.6×) | 1,316 (4.2×) | 606 (1.9×) |
| mulmod | 512 | 483 | 1,575 (3.3×) | 866 (1.8×) | 2,060 (4.3×) | 1,076 (2.2×) |
| mulmod | 1024 | 1,337 | 5,328 (4.0×) | 2,754 (2.1×) | 6,822 (5.1×) | 3,527 (2.6×) |
| mulmod | 2048 | 5,198 | 17,427 (3.4×) | 8,364 (1.6×) | 22,424 (4.3×) | 13,175 (2.5×) |
| mulmod | 4096 | 14,977 | 55,946 (3.7×) | 27,016 (1.8×) | 68,443 (4.6×) | 45,374 (3.0×) |
| add_mod | 64 | 8.0 | 6.4 (0.8×) | 6.3 (0.8×) | 9.5 (1.2×) | 9.2 (1.1×) |
| add_mod | 128 | 11.9 | 31.7 (2.7×) | 11.0 (0.9×) | 41.8 (3.5×) | 16.2 (1.4×) |
| add_mod | 256 | 18.5 | 41.0 (2.2×) | 31.7 (1.7×) | 56.2 (3.0×) | 43.5 (2.4×) |
| add_mod | 384 | 22.9 | 55.2 (2.4×) | 47.2 (2.1×) | 83.7 (3.7×) | 63.4 (2.8×) |
| add_mod | 512 | 33.5 | 66.6 (2.0×) | 55.9 (1.7×) | 95.3 (2.8×) | 83.5 (2.5×) |
| add_mod | 1024 | 79.2 | 145 (1.8×) | 122 (1.5×) | 157 (2.0×) | 157 (2.0×) |
| add_mod | 2048 | 155 | 307 (2.0×) | 237 (1.5×) | 375 (2.4×) | 315 (2.0×) |
| add_mod | 4096 | 333 | 865 (2.6×) | 764 (2.3×) | 912 (2.7×) | 921 (2.8×) |
| sub_mod | 64 | 7.3 | 4.8 (0.7×) | 7.2 (1.0×) | 8.7 (1.2×) | 9.1 (1.2×) |
| sub_mod | 128 | 11.3 | 33.8 (3.0×) | 11.3 (1.0×) | 45.6 (4.0×) | 15.8 (1.4×) |
| sub_mod | 256 | 14.4 | 46.4 (3.2×) | 33.8 (2.3×) | 60.4 (4.2×) | 44.1 (3.1×) |
| sub_mod | 384 | 23.1 | 52.7 (2.3×) | 44.5 (1.9×) | 85.4 (3.7×) | 64.5 (2.8×) |
| sub_mod | 512 | 37.4 | 71.6 (1.9×) | 53.3 (1.4×) | 103 (2.8×) | 87.4 (2.3×) |
| sub_mod | 1024 | 79.9 | 116 (1.5×) | 105 (1.3×) | 179 (2.2×) | 167 (2.1×) |
| sub_mod | 2048 | 144 | 281 (2.0×) | 260 (1.8×) | 379 (2.6×) | 358 (2.5×) |
| sub_mod | 4096 | 320 | 873 (2.7×) | 820 (2.6×) | 923 (2.9×) | 1,099 (3.4×) |
| add_mod, reduced operands | 64 | 4.9 | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) |
| add_mod, reduced operands | 128 | 6.0 | 6.7 (1.1×) | 7.6 (1.3×) | 7.6 (1.3×) | 10.4 (1.7×) |
| add_mod, reduced operands | 256 | 12.9 | 17.0 (1.3×) | 13.1 (1.0×) | 16.8 (1.3×) | 21.3 (1.7×) |
| add_mod, reduced operands | 384 | 14.6 | 29.5 (2.0×) | 19.6 (1.3×) | 28.5 (2.0×) | 32.1 (2.2×) |
| add_mod, reduced operands | 512 | 20.2 | 35.6 (1.8×) | 28.4 (1.4×) | 43.0 (2.1×) | 43.3 (2.1×) |
| add_mod, reduced operands | 1024 | 44.9 | 76.9 (1.7×) | 69.1 (1.5×) | 88.9 (2.0×) | 93.3 (2.1×) |
| add_mod, reduced operands | 2048 | 83.0 | 204 (2.5×) | 157 (1.9×) | 231 (2.8×) | 205 (2.5×) |
| add_mod, reduced operands | 4096 | 240 | 548 (2.3×) | 440 (1.8×) | 629 (2.6×) | 520 (2.2×) |
| sub_mod, reduced operands | 64 | 3.9 | 2.7 (0.7×) | 2.7 (0.7×) | 2.6 (0.7×) | 3.5 (0.9×) |
| sub_mod, reduced operands | 128 | 4.8 | 5.4 (1.1×) | 6.1 (1.3×) | 5.4 (1.1×) | 8.8 (1.8×) |
| sub_mod, reduced operands | 256 | 7.4 | 15.0 (2.0×) | 12.0 (1.6×) | 13.3 (1.8×) | 19.0 (2.6×) |
| sub_mod, reduced operands | 384 | 10.1 | 25.3 (2.5×) | 19.1 (1.9×) | 25.3 (2.5×) | 30.0 (3.0×) |
| sub_mod, reduced operands | 512 | 16.1 | 34.9 (2.2×) | 29.2 (1.8×) | 40.6 (2.5×) | 41.9 (2.6×) |
| sub_mod, reduced operands | 1024 | 37.8 | 68.6 (1.8×) | 68.0 (1.8×) | 80.2 (2.1×) | 92.3 (2.4×) |
| sub_mod, reduced operands | 2048 | 75.6 | 213 (2.8×) | 172 (2.3×) | 299 (4.0×) | 205 (2.7×) |
| sub_mod, reduced operands | 4096 | 238 | 570 (2.4×) | 450 (1.9×) | 610 (2.6×) | 571 (2.4×) |
| add, constant time | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) |
| add, constant time | 128 | 1.3 | 3.0 (2.3×) | 2.5 (1.9×) | 2.7 (2.1×) | 3.2 (2.5×) |
| add, constant time | 256 | 2.3 | 9.8 (4.3×) | 8.6 (3.7×) | 8.8 (3.8×) | 11.3 (4.9×) |
| add, constant time | 384 | 3.7 | 14.9 (4.0×) | 12.2 (3.3×) | 16.1 (4.4×) | 18.1 (4.9×) |
| add, constant time | 512 | 4.1 | 23.8 (5.8×) | 18.2 (4.4×) | 25.3 (6.2×) | 31.3 (7.6×) |
| add, constant time | 1024 | 22.0 | 55.3 (2.5×) | 47.2 (2.1×) | 59.5 (2.7×) | 67.7 (3.1×) |
| add, constant time | 2048 | 51.0 | 115 (2.3×) | 106 (2.1×) | 125 (2.5×) | 141 (2.8×) |
| add, constant time | 4096 | 161 | 354 (2.2×) | 248 (1.5×) | 401 (2.5×) | 305 (1.9×) |
| mulmod, constant time | 64 | 93.9 | 131 (1.4×) | 94.3 (1.0×) | 243 (2.6×) | 110 (1.2×) |
| mulmod, constant time | 128 | 206 | 572 (2.8×) | 367 (1.8×) | 603 (2.9×) | 371 (1.8×) |
| mulmod, constant time | 256 | 471 | 1,189 (2.5×) | 745 (1.6×) | 1,273 (2.7×) | 844 (1.8×) |
| mulmod, constant time | 384 | 830 | 1,929 (2.3×) | 1,208 (1.5×) | 2,273 (2.7×) | 1,490 (1.8×) |
| mulmod, constant time | 512 | 1,189 | 3,013 (2.5×) | 1,849 (1.6×) | 3,546 (3.0×) | 2,294 (1.9×) |
| mulmod, constant time | 1024 | 3,878 | 9,551 (2.5×) | 5,652 (1.5×) | 10,948 (2.8×) | 7,484 (1.9×) |
| mulmod, constant time | 2048 | 13,505 | 32,887 (2.4×) | 19,411 (1.4×) | 37,045 (2.7×) | 25,512 (1.9×) |
| mulmod, constant time | 4096 | 48,916 | 120,362 (2.5×) | 68,757 (1.4×) | 132,524 (2.7×) | 91,127 (1.9×) |
| add_mod, constant time | 64 | 8.9 | 12.3 (1.4×) | 13.0 (1.5×) | 18.8 (2.1×) | 14.0 (1.6×) |
| add_mod, constant time | 128 | 12.5 | 28.2 (2.3×) | 22.5 (1.8×) | 43.3 (3.5×) | 34.1 (2.7×) |
| add_mod, constant time | 256 | 18.1 | 83.7 (4.6×) | 67.1 (3.7×) | 98.7 (5.5×) | 92.7 (5.1×) |
| add_mod, constant time | 384 | 36.0 | 112 (3.1×) | 108 (3.0×) | 178 (4.9×) | 127 (3.5×) |
| add_mod, constant time | 512 | 47.8 | 151 (3.2×) | 135 (2.8×) | 246 (5.1×) | 168 (3.5×) |
| add_mod, constant time | 1024 | 131 | 327 (2.5×) | 260 (2.0×) | 621 (4.7×) | 350 (2.7×) |
| add_mod, constant time | 2048 | 308 | 868 (2.8×) | 519 (1.7×) | 1,605 (5.2×) | 758 (2.5×) |
| add_mod, constant time | 4096 | 551 | 1,876 (3.4×) | 1,684 (3.1×) | 2,043 (3.7×) | 2,049 (3.7×) |
| sub_mod, constant time | 64 | 8.9 | 12.1 (1.4×) | 11.0 (1.2×) | 18.3 (2.1×) | 14.1 (1.6×) |
| sub_mod, constant time | 128 | 11.8 | 24.0 (2.0×) | 19.0 (1.6×) | 31.9 (2.7×) | 23.0 (1.9×) |
| sub_mod, constant time | 256 | 17.1 | 54.7 (3.2×) | 37.0 (2.2×) | 76.1 (4.5×) | 57.5 (3.4×) |
| sub_mod, constant time | 384 | 27.5 | 103 (3.7×) | 57.5 (2.1×) | 122 (4.4×) | 85.4 (3.1×) |
| sub_mod, constant time | 512 | 32.2 | 142 (4.4×) | 85.9 (2.7×) | 199 (6.2×) | 117 (3.6×) |
| sub_mod, constant time | 1024 | 94.2 | 310 (3.3×) | 200 (2.1×) | 534 (5.7×) | 254 (2.7×) |
| sub_mod, constant time | 2048 | 284 | 742 (2.6×) | 512 (1.8×) | 1,278 (4.5×) | 584 (2.1×) |
| sub_mod, constant time | 4096 | 506 | 1,563 (3.1×) | 1,422 (2.8×) | 1,737 (3.4×) | 1,715 (3.4×) |
| add_mod, reduced operands, constant time | 64 | 6.9 | 8.1 (1.2×) | 8.0 (1.2×) | 8.0 (1.2×) | 8.5 (1.2×) |
| add_mod, reduced operands, constant time | 128 | 8.8 | 15.0 (1.7×) | 13.2 (1.5×) | 14.7 (1.7×) | 16.2 (1.8×) |
| add_mod, reduced operands, constant time | 256 | 12.9 | 30.9 (2.4×) | 23.1 (1.8×) | 28.1 (2.2×) | 31.6 (2.4×) |
| add_mod, reduced operands, constant time | 384 | 14.6 | 51.7 (3.5×) | 35.0 (2.4×) | 51.6 (3.5×) | 49.3 (3.4×) |
| add_mod, reduced operands, constant time | 512 | 20.3 | 68.6 (3.4×) | 50.4 (2.5×) | 87.4 (4.3×) | 64.2 (3.2×) |
| add_mod, reduced operands, constant time | 1024 | 51.6 | 142 (2.8×) | 91.1 (1.8×) | 305 (5.9×) | 144 (2.8×) |
| add_mod, reduced operands, constant time | 2048 | 140 | 277 (2.0×) | 184 (1.3×) | 669 (4.8×) | 298 (2.1×) |
| add_mod, reduced operands, constant time | 4096 | 301 | 821 (2.7×) | 716 (2.4×) | 868 (2.9×) | 825 (2.7×) |
| sub_mod, reduced operands, constant time | 64 | 3.9 | 4.0 (1.0×) | 5.0 (1.3×) | 4.0 (1.0×) | 5.1 (1.3×) |
| sub_mod, reduced operands, constant time | 128 | 4.8 | 10.6 (2.2×) | 10.0 (2.1×) | 10.0 (2.1×) | 10.1 (2.1×) |
| sub_mod, reduced operands, constant time | 256 | 6.9 | 23.1 (3.3×) | 19.1 (2.8×) | 31.0 (4.5×) | 28.0 (4.1×) |
| sub_mod, reduced operands, constant time | 384 | 9.5 | 42.1 (4.4×) | 32.7 (3.4×) | 50.1 (5.3×) | 43.5 (4.6×) |
| sub_mod, reduced operands, constant time | 512 | 18.0 | 59.8 (3.3×) | 44.2 (2.5×) | 91.2 (5.1×) | 62.4 (3.5×) |
| sub_mod, reduced operands, constant time | 1024 | 44.8 | 129 (2.9×) | 112 (2.5×) | 232 (5.2×) | 141 (3.1×) |
| sub_mod, reduced operands, constant time | 2048 | 121 | 300 (2.5×) | 257 (2.1×) | 570 (4.7×) | 304 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 304 | 852 (2.8×) | 690 (2.3×) | 874 (2.9×) | 884 (2.9×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
