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
| Cost of a count, in WebAssembly, Chrome | 4,015 to 4,505 cycles and 3,081 instructions; its noise in a figure, 61.0 to 113 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,188 to 4,936 cycles and 3,081 instructions; its noise in a figure, 82.0 to 583 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 5,871 to 11,208 cycles and 1,885 instructions; its noise in a figure, 491 to 3,695 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 5,415 to 17,670 cycles and 1,885 instructions; its noise in a figure, 107 to 8,677 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 2.01 before, 5.33 after; its SMT sibling, CPU 28, 2% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 5.33 before, 11.86 after; its SMT sibling, CPU 28, 2% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 11.86 before, 26.67 after; its SMT sibling, CPU 28, 28% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 26.67 before, 35.97 after; its SMT sibling, CPU 28, 48% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs bnum 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.4 vs native 3.4 | +1.0 (1.29×) |
| add | 256 | 10.3 vs crypto-bigint 10.2 | +0.1 (1.01×) | 8.7 vs ruint 9.0 | −0.3 (0.97×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add | 384 | 16.2 vs ruint 16.3 | −0.1 (0.99×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 15.2 vs ruint 16.0 | −0.8 (0.95×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add | 512 | 26.3 vs bnum 24.7 | +1.6 (1.06×) | 20.8 vs crypto-bigint 20.7 | +0.1 (1.00×) | 26.4 vs bnum 23.8 | +2.6 (1.11×) | 32.1 vs bnum 27.0 | +5.1 (1.19×) |
| add | 1024 | 55.3 vs bnum 61.5 | −6.2 (0.90×) | 45.3 vs crypto-bigint 45.4 | −0.1 (1.00×) | 72.4 vs ruint 69.9 | +2.5 (1.04×) | 69.0 vs crypto-bigint 68.7 | +0.3 (1.00×) |
| add | 2048 | 117 vs crypto-bigint 190 | −73.0 (0.62×) | 106 vs crypto-bigint 138 | −32.0 (0.77×) | 143 vs bnum 205 | −62.0 (0.70×) | 147 vs crypto-bigint 176 | −29.0 (0.84×) |
| add | 4096 | 346 vs crypto-bigint 389 | −43.0 (0.89×) | 259 vs crypto-bigint 262 | −3.0 (0.99×) | 421 vs bnum 390 | +31.0 (1.08×) | 314 vs crypto-bigint 327 | −13.0 (0.96×) |
| mul | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.1 | +2.4 (1.59×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.6 vs crypto-bigint 5.0 | +0.6 (1.12×) |
| mul | 256 | 47.9 vs ruint 66.6 | −18.7 (0.72×) | 16.8 vs ruint 16.8 | +0.0 (1.00×) | 67.0 vs uint 81.7 | −14.7 (0.82×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul | 384 | 115 vs ruint 168 | −53.0 (0.68×) | 54.0 vs crypto-bigint 39.6 | +14.4 (1.36×) | 156 vs ruint 220 | −64.0 (0.71×) | 68.6 vs crypto-bigint 64.7 | +3.9 (1.06×) |
| mul | 512 | 205 vs ruint 321 | −116 (0.64×) | 71.3 vs ruint 71.1 | +0.2 (1.00×) | 306 vs ruint 439 | −133 (0.70×) | 129 vs ruint 130 | −1.0 (0.99×) |
| mul | 1024 | 1,235 vs bnum 1,541 | −306 (0.80×) | 315 vs uint 504 | −189 (0.62×) | 1,496 vs bnum 2,356 | −860 (0.63×) | 565 vs crypto-bigint 893 | −328 (0.63×) |
| mul | 2048 | 5,581 vs bnum 6,276 | −695 (0.89×) | 2,316 vs ruint 2,127 | +189 (1.09×) | 6,612 vs bnum 8,722 | −2,110 (0.76×) | 3,535 vs ruint 3,222 | +313 (1.10×) |
| mul | 4096 | 18,145 vs bnum 23,566 | −5,421 (0.77×) | 8,098 vs ruint 8,004 | +94.0 (1.01×) | 21,248 vs bnum 34,706 | −13,458 (0.61×) | 11,918 vs ruint 11,635 | +283 (1.02×) |
| div | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 59.9 vs ruint 87.8 | −27.9 (0.68×) | 28.4 vs ruint 43.2 | −14.8 (0.66×) | 102 vs ruint 145 | −43.0 (0.70×) | 34.4 vs ruint 54.9 | −20.5 (0.63×) |
| div | 256 | 206 vs ruint 322 | −116 (0.64×) | 84.3 vs ruint 158 | −73.7 (0.53×) | 220 vs ruint 403 | −183 (0.55×) | 108 vs ruint 175 | −67.0 (0.62×) |
| div | 384 | 347 vs ruint 607 | −260 (0.57×) | 206 vs ruint 350 | −144 (0.59×) | 412 vs num-bigint 772 | −360 (0.53×) | 287 vs ruint 414 | −127 (0.69×) |
| div | 512 | 474 vs num-bigint 815 | −341 (0.58×) | 306 vs ruint 430 | −124 (0.71×) | 557 vs num-bigint 1,014 | −457 (0.55×) | 400 vs ruint 569 | −169 (0.70×) |
| div | 1024 | 1,135 vs num-bigint 1,944 | −809 (0.58×) | 688 vs ruint 898 | −210 (0.77×) | 1,485 vs num-bigint 2,203 | −718 (0.67×) | 881 vs ruint 1,097 | −216 (0.80×) |
| div | 2048 | 3,214 vs num-bigint 5,782 | −2,568 (0.56×) | 2,011 vs ruint 2,358 | −347 (0.85×) | 4,257 vs num-bigint 6,170 | −1,913 (0.69×) | 2,242 vs ruint 2,670 | −428 (0.84×) |
| div | 4096 | 12,825 vs num-bigint 20,147 | −7,322 (0.64×) | 6,997 vs ruint 7,592 | −595 (0.92×) | 15,089 vs num-bigint 20,237 | −5,148 (0.75×) | 7,557 vs ruint 8,969 | −1,412 (0.84×) |
| mod | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 52.8 vs ruint 92.7 | −39.9 (0.57×) | 29.7 vs ruint 40.2 | −10.5 (0.74×) | 89.7 vs ruint 99.9 | −10.2 (0.90×) | 33.7 vs ruint 52.3 | −18.6 (0.64×) |
| mod | 256 | 207 vs ruint 321 | −114 (0.64×) | 74.1 vs ruint 158 | −83.9 (0.47×) | 218 vs ruint 411 | −193 (0.53×) | 108 vs ruint 175 | −67.0 (0.62×) |
| mod | 384 | 347 vs ruint 588 | −241 (0.59×) | 197 vs ruint 322 | −125 (0.61×) | 405 vs ruint 713 | −308 (0.57×) | 268 vs ruint 342 | −74.0 (0.78×) |
| mod | 512 | 471 vs ruint 794 | −323 (0.59×) | 283 vs ruint 405 | −122 (0.70×) | 566 vs ruint 999 | −433 (0.57×) | 352 vs ruint 460 | −108 (0.77×) |
| mod | 1024 | 1,194 vs num-bigint 1,943 | −749 (0.61×) | 705 vs ruint 963 | −258 (0.73×) | 1,479 vs num-bigint 2,193 | −714 (0.67×) | 904 vs ruint 1,081 | −177 (0.84×) |
| mod | 2048 | 3,307 vs num-bigint 5,808 | −2,501 (0.57×) | 1,933 vs ruint 2,282 | −349 (0.85×) | 4,289 vs num-bigint 6,127 | −1,838 (0.70×) | 2,179 vs ruint 2,624 | −445 (0.83×) |
| mod | 4096 | 12,799 vs num-bigint 20,141 | −7,342 (0.64×) | 6,607 vs ruint 7,751 | −1,144 (0.85×) | 15,019 vs num-bigint 20,179 | −5,160 (0.74×) | 7,193 vs ruint 8,933 | −1,740 (0.81×) |
| mulmod | 64 | 62.4 vs ruint 84.3 | −21.9 (0.74×) | 31.4 vs ruint 39.9 | −8.5 (0.79×) | 105 vs ruint 188 | −83.0 (0.56×) | 37.4 vs ruint 54.3 | −16.9 (0.69×) |
| mulmod | 128 | 205 vs ruint 324 | −119 (0.63×) | 81.8 vs ruint 135 | −53.2 (0.61×) | 237 vs ruint 389 | −152 (0.61×) | 128 vs ruint 206 | −78.0 (0.62×) |
| mulmod | 256 | 580 vs ruint 810 | −230 (0.72×) | 280 vs ruint 307 | −27.0 (0.91×) | 692 vs ruint 899 | −207 (0.77×) | 363 vs ruint 384 | −21.0 (0.95×) |
| mulmod | 384 | 968 vs ruint 1,600 | −632 (0.60×) | 511 vs ruint 540 | −29.0 (0.95×) | 1,338 vs ruint 1,698 | −360 (0.79×) | 630 vs ruint 696 | −66.0 (0.91×) |
| mulmod | 512 | 1,582 vs ruint 2,836 | −1,254 (0.56×) | 890 vs ruint 856 | +34.0 (1.04×) | 2,189 vs num-bigint 3,087 | −898 (0.71×) | 1,070 vs ruint 1,109 | −39.0 (0.96×) |
| mulmod | 1024 | 5,333 vs num-bigint 8,882 | −3,549 (0.60×) | 2,725 vs ruint 2,593 | +132 (1.05×) | 6,685 vs num-bigint 9,260 | −2,575 (0.72×) | 3,625 vs ruint 3,609 | +16.0 (1.00×) |
| mulmod | 2048 | 17,415 vs num-bigint 31,900 | −14,485 (0.55×) | 8,579 vs ruint 10,694 | −2,115 (0.80×) | 21,754 vs num-bigint 31,958 | −10,204 (0.68×) | 13,227 vs ruint 13,808 | −581 (0.96×) |
| mulmod | 4096 | 56,014 vs num-bigint 113,142 | −57,128 (0.50×) | 26,920 vs ruint 41,209 | −14,289 (0.65×) | 68,548 vs num-bigint 113,952 | −45,404 (0.60×) | 45,349 vs ruint 53,186 | −7,837 (0.85×) |
| add_mod | 64 | 4.0 vs ruint 44.3 | −40.3 (0.09×) | 4.0 vs ruint 43.9 | −39.9 (0.09×) | 6.7 vs ruint 84.2 | −77.5 (0.08×) | 7.0 vs ruint 92.2 | −85.2 (0.08×) |
| add_mod | 128 | 8.7 vs ruint 22.9 | −14.2 (0.38×) | 9.1 vs ruint 21.6 | −12.5 (0.42×) | 11.1 vs ruint 23.1 | −12.0 (0.48×) | 14.2 vs ruint 24.9 | −10.7 (0.57×) |
| add_mod | 256 | 21.8 vs ruint 43.0 | −21.2 (0.51×) | 21.8 vs ruint 43.9 | −22.1 (0.50×) | 24.5 vs ruint 59.9 | −35.4 (0.41×) | 31.2 vs ruint 59.8 | −28.6 (0.52×) |
| add_mod | 384 | 30.7 vs ruint 62.4 | −31.7 (0.49×) | 28.4 vs ruint 60.3 | −31.9 (0.47×) | 42.9 vs ruint 92.2 | −49.3 (0.47×) | 48.7 vs ruint 84.9 | −36.2 (0.57×) |
| add_mod | 512 | 44.4 vs ruint 90.8 | −46.4 (0.49×) | 37.8 vs ruint 85.9 | −48.1 (0.44×) | 67.2 vs ruint 122 | −54.8 (0.55×) | 63.4 vs ruint 124 | −60.6 (0.51×) |
| add_mod | 1024 | 112 vs ruint 199 | −87.0 (0.56×) | 105 vs ruint 181 | −76.0 (0.58×) | 137 vs ruint 261 | −124 (0.52×) | 134 vs ruint 243 | −109 (0.55×) |
| add_mod | 2048 | 248 vs ruint 790 | −542 (0.31×) | 239 vs ruint 746 | −507 (0.32×) | 376 vs num-bigint 974 | −598 (0.39×) | 290 vs ruint 907 | −617 (0.32×) |
| add_mod | 4096 | 701 vs ruint 1,542 | −841 (0.45×) | 732 vs ruint 1,403 | −671 (0.52×) | 797 vs ruint 1,840 | −1,043 (0.43×) | 904 vs ruint 1,664 | −760 (0.54×) |
| sub_mod | 64 | 4.0 vs ruint 53.8 | −49.8 (0.07×) | 4.0 vs ruint 50.1 | −46.1 (0.08×) | 7.2 vs ruint 81.0 | −73.8 (0.09×) | 9.1 vs ruint 78.8 | −69.7 (0.12×) |
| sub_mod | 128 | 10.5 vs ruint 31.1 | −20.6 (0.34×) | 10.2 vs ruint 30.1 | −19.9 (0.34×) | 10.8 vs ruint 37.1 | −26.3 (0.29×) | 16.8 vs ruint 37.8 | −21.0 (0.44×) |
| sub_mod | 256 | 23.1 vs ruint 61.2 | −38.1 (0.38×) | 19.8 vs ruint 61.1 | −41.3 (0.32×) | 25.4 vs ruint 87.3 | −61.9 (0.29×) | 28.7 vs ruint 86.0 | −57.3 (0.33×) |
| sub_mod | 384 | 30.8 vs ruint 84.4 | −53.6 (0.36×) | 27.1 vs ruint 85.1 | −58.0 (0.32×) | 44.2 vs ruint 134 | −89.8 (0.33×) | 46.1 vs ruint 127 | −80.9 (0.36×) |
| sub_mod | 512 | 44.2 vs ruint 120 | −75.8 (0.37×) | 37.3 vs ruint 114 | −76.7 (0.33×) | 62.5 vs ruint 178 | −116 (0.35×) | 64.7 vs ruint 172 | −107 (0.38×) |
| sub_mod | 1024 | 103 vs ruint 353 | −250 (0.29×) | 110 vs ruint 327 | −217 (0.34×) | 126 vs ruint 566 | −440 (0.22×) | 136 vs ruint 539 | −403 (0.25×) |
| sub_mod | 2048 | 232 vs ruint 1,113 | −881 (0.21×) | 247 vs ruint 1,083 | −836 (0.23×) | 294 vs ruint 1,383 | −1,089 (0.21×) | 273 vs ruint 1,269 | −996 (0.22×) |
| sub_mod | 4096 | 703 vs ruint 2,123 | −1,420 (0.33×) | 743 vs ruint 1,998 | −1,255 (0.37×) | 806 vs ruint 2,457 | −1,651 (0.33×) | 953 vs ruint 2,279 | −1,326 (0.42×) |
| add_mod, reduced operands | 64 | 4.2 vs native 3.4 | +0.8 (1.24×) | 4.1 vs native 4.0 | +0.1 (1.02×) | 4.4 vs native 3.7 | +0.7 (1.19×) | 4.4 vs native 5.0 | −0.6 (0.88×) |
| add_mod, reduced operands | 128 | 7.8 vs native 9.6 | −1.8 (0.81×) | 9.1 vs native 9.0 | +0.1 (1.01×) | 7.8 vs native 9.5 | −1.7 (0.82×) | 11.6 vs native 9.1 | +2.5 (1.27×) |
| add_mod, reduced operands | 256 | 17.1 vs uint 21.5 | −4.4 (0.80×) | 15.5 vs uint 21.2 | −5.7 (0.73×) | 17.2 vs uint 19.5 | −2.3 (0.88×) | 22.3 vs uint 25.1 | −2.8 (0.89×) |
| add_mod, reduced operands | 384 | 27.8 vs uint 33.2 | −5.4 (0.84×) | 22.1 vs uint 36.9 | −14.8 (0.60×) | 28.7 vs uint 31.4 | −2.7 (0.91×) | 32.0 vs uint 38.5 | −6.5 (0.83×) |
| add_mod, reduced operands | 512 | 34.6 vs uint 42.6 | −8.0 (0.81×) | 28.4 vs uint 41.4 | −13.0 (0.69×) | 40.2 vs uint 44.4 | −4.2 (0.91×) | 42.8 vs uint 63.0 | −20.2 (0.68×) |
| add_mod, reduced operands | 1024 | 77.0 vs uint 102 | −25.0 (0.75×) | 65.8 vs uint 101 | −35.2 (0.65×) | 86.8 vs uint 136 | −49.2 (0.64×) | 92.5 vs uint 127 | −34.5 (0.73×) |
| add_mod, reduced operands | 2048 | 203 vs uint 231 | −28.0 (0.88×) | 149 vs uint 225 | −76.0 (0.66×) | 233 vs uint 260 | −27.0 (0.90×) | 220 vs uint 256 | −36.0 (0.86×) |
| add_mod, reduced operands | 4096 | 533 vs uint 570 | −37.0 (0.94×) | 458 vs uint 574 | −116 (0.80×) | 565 vs uint 659 | −94.0 (0.86×) | 624 vs uint 669 | −45.0 (0.93×) |
| sub_mod, reduced operands | 64 | 3.5 vs native 3.0 | +0.5 (1.17×) | 4.0 vs native 3.0 | +1.0 (1.33×) | 4.4 vs native 3.0 | +1.4 (1.47×) | 5.5 vs native 3.0 | +2.5 (1.83×) |
| sub_mod, reduced operands | 128 | 6.8 vs native 6.7 | +0.1 (1.01×) | 7.3 vs native 5.4 | +1.9 (1.35×) | 7.2 vs native 7.4 | −0.2 (0.97×) | 9.5 vs native 7.2 | +2.3 (1.32×) |
| sub_mod, reduced operands | 256 | 16.9 vs bnum 18.7 | −1.8 (0.90×) | 13.3 vs bnum 16.4 | −3.1 (0.81×) | 14.9 vs bnum 19.3 | −4.4 (0.77×) | 20.6 vs bnum 20.5 | +0.1 (1.00×) |
| sub_mod, reduced operands | 384 | 26.3 vs uint 30.5 | −4.2 (0.86×) | 20.7 vs uint 31.3 | −10.6 (0.66×) | 25.4 vs bnum 32.6 | −7.2 (0.78×) | 31.1 vs uint 39.6 | −8.5 (0.79×) |
| sub_mod, reduced operands | 512 | 34.2 vs bnum 46.9 | −12.7 (0.73×) | 29.6 vs bnum 44.3 | −14.7 (0.67×) | 38.2 vs bnum 58.5 | −20.3 (0.65×) | 43.4 vs bnum 61.7 | −18.3 (0.70×) |
| sub_mod, reduced operands | 1024 | 67.2 vs uint 132 | −64.8 (0.51×) | 69.8 vs uint 127 | −57.2 (0.55×) | 77.2 vs uint 144 | −66.8 (0.54×) | 92.5 vs uint 151 | −58.5 (0.61×) |
| sub_mod, reduced operands | 2048 | 203 vs uint 277 | −74.0 (0.73×) | 157 vs uint 277 | −120 (0.57×) | 289 vs uint 363 | −74.0 (0.80×) | 207 vs uint 371 | −164 (0.56×) |
| sub_mod, reduced operands | 4096 | 528 vs bnum 632 | −104 (0.84×) | 459 vs uint 686 | −227 (0.67×) | 549 vs bnum 724 | −175 (0.76×) | 557 vs uint 692 | −135 (0.80×) |
| add, constant time | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.1 vs ruint 2.1 | +0.0 (1.00×) | 2.2 vs uint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add, constant time | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.3 vs native 3.4 | +0.9 (1.26×) |
| add, constant time | 256 | 10.3 vs ruint 10.2 | +0.1 (1.01×) | 8.7 vs ruint 9.0 | −0.3 (0.97×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add, constant time | 384 | 16.2 vs ruint 16.4 | −0.2 (0.99×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 15.2 vs ruint 16.1 | −0.9 (0.94×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 26.2 vs bnum 24.7 | +1.5 (1.06×) | 20.8 vs crypto-bigint 20.7 | +0.1 (1.00×) | 26.4 vs bnum 23.8 | +2.6 (1.11×) | 32.2 vs bnum 27.0 | +5.2 (1.19×) |
| add, constant time | 1024 | 55.3 vs bnum 61.5 | −6.2 (0.90×) | 45.9 vs crypto-bigint 45.3 | +0.6 (1.01×) | 72.7 vs ruint 69.2 | +3.5 (1.05×) | 69.2 vs crypto-bigint 68.6 | +0.6 (1.01×) |
| add, constant time | 2048 | 117 vs crypto-bigint 190 | −73.0 (0.62×) | 108 vs crypto-bigint 137 | −29.0 (0.79×) | 142 vs crypto-bigint 206 | −64.0 (0.69×) | 147 vs crypto-bigint 177 | −30.0 (0.83×) |
| add, constant time | 4096 | 346 vs crypto-bigint 390 | −44.0 (0.89×) | 259 vs crypto-bigint 262 | −3.0 (0.99×) | 403 vs bnum 391 | +12.0 (1.03×) | 317 vs crypto-bigint 327 | −10.0 (0.97×) |
| mul, constant time | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.1 | +2.4 (1.59×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.6 vs crypto-bigint 5.0 | +0.6 (1.12×) |
| mul, constant time | 256 | 47.9 vs ruint 66.5 | −18.6 (0.72×) | 16.8 vs ruint 16.8 | +0.0 (1.00×) | 66.4 vs uint 81.7 | −15.3 (0.81×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul, constant time | 384 | 115 vs ruint 167 | −52.0 (0.69×) | 54.0 vs crypto-bigint 39.6 | +14.4 (1.36×) | 165 vs ruint 219 | −54.0 (0.75×) | 69.4 vs crypto-bigint 63.6 | +5.8 (1.09×) |
| mul, constant time | 512 | 205 vs ruint 322 | −117 (0.64×) | 70.4 vs ruint 70.1 | +0.3 (1.00×) | 307 vs ruint 438 | −131 (0.70×) | 130 vs ruint 129 | +1.0 (1.01×) |
| mul, constant time | 1024 | 1,189 vs bnum 1,544 | −355 (0.77×) | 315 vs uint 504 | −189 (0.62×) | 1,506 vs bnum 2,352 | −846 (0.64×) | 565 vs crypto-bigint 892 | −327 (0.63×) |
| mul, constant time | 2048 | 5,782 vs bnum 6,276 | −494 (0.92×) | 2,403 vs bnum 2,598 | −195 (0.92×) | 6,698 vs bnum 8,722 | −2,024 (0.77×) | 3,640 vs bnum 3,406 | +234 (1.07×) |
| mul, constant time | 4096 | 18,939 vs bnum 23,566 | −4,627 (0.80×) | 8,417 vs bnum 10,097 | −1,680 (0.83×) | 21,596 vs bnum 34,706 | −13,110 (0.62×) | 12,501 vs bnum 12,857 | −356 (0.97×) |
| div, constant time | 64 | 83.5 vs crypto-bigint 392 | −309 (0.21×) | 47.3 vs crypto-bigint 322 | −275 (0.15×) | 145 vs crypto-bigint 428 | −283 (0.34×) | 61.5 vs crypto-bigint 339 | −278 (0.18×) |
| div, constant time | 128 | 273 vs crypto-bigint 910 | −637 (0.30×) | 150 vs crypto-bigint 751 | −601 (0.20×) | 351 vs crypto-bigint 1,089 | −738 (0.32×) | 230 vs crypto-bigint 909 | −679 (0.25×) |
| div, constant time | 256 | 512 vs crypto-bigint 1,691 | −1,179 (0.30×) | 241 vs crypto-bigint 1,364 | −1,123 (0.18×) | 622 vs crypto-bigint 1,978 | −1,356 (0.31×) | 472 vs crypto-bigint 1,529 | −1,057 (0.31×) |
| div, constant time | 384 | 659 vs crypto-bigint 2,513 | −1,854 (0.26×) | 369 vs crypto-bigint 2,018 | −1,649 (0.18×) | 815 vs crypto-bigint 3,046 | −2,231 (0.27×) | 538 vs crypto-bigint 2,273 | −1,735 (0.24×) |
| div, constant time | 512 | 841 vs crypto-bigint 3,514 | −2,673 (0.24×) | 665 vs crypto-bigint 2,767 | −2,102 (0.24×) | 1,175 vs crypto-bigint 4,293 | −3,118 (0.27×) | 856 vs crypto-bigint 3,098 | −2,242 (0.28×) |
| div, constant time | 1024 | 2,463 vs crypto-bigint 8,485 | −6,022 (0.29×) | 1,779 vs crypto-bigint 7,037 | −5,258 (0.25×) | 2,848 vs crypto-bigint 10,356 | −7,508 (0.28×) | 2,211 vs crypto-bigint 7,444 | −5,233 (0.30×) |
| div, constant time | 2048 | 7,293 vs crypto-bigint 22,453 | −15,160 (0.32×) | 5,082 vs crypto-bigint 16,840 | −11,758 (0.30×) | 8,383 vs crypto-bigint 27,481 | −19,098 (0.31×) | 6,350 vs crypto-bigint 18,614 | −12,264 (0.34×) |
| div, constant time | 4096 | 25,512 vs crypto-bigint 67,160 | −41,648 (0.38×) | 16,252 vs crypto-bigint 43,311 | −27,059 (0.38×) | 27,637 vs crypto-bigint 85,166 | −57,529 (0.32×) | 20,890 vs crypto-bigint 50,552 | −29,662 (0.41×) |
| mod, constant time | 64 | 83.3 vs crypto-bigint 385 | −302 (0.22×) | 48.0 vs crypto-bigint 323 | −275 (0.15×) | 141 vs crypto-bigint 422 | −281 (0.33×) | 67.4 vs crypto-bigint 339 | −272 (0.20×) |
| mod, constant time | 128 | 269 vs crypto-bigint 890 | −621 (0.30×) | 138 vs crypto-bigint 742 | −604 (0.19×) | 332 vs crypto-bigint 1,057 | −725 (0.31×) | 223 vs crypto-bigint 881 | −658 (0.25×) |
| mod, constant time | 256 | 526 vs crypto-bigint 1,637 | −1,111 (0.32×) | 331 vs crypto-bigint 1,341 | −1,010 (0.25×) | 572 vs crypto-bigint 1,959 | −1,387 (0.29×) | 411 vs crypto-bigint 1,523 | −1,112 (0.27×) |
| mod, constant time | 384 | 746 vs crypto-bigint 2,606 | −1,860 (0.29×) | 530 vs crypto-bigint 1,968 | −1,438 (0.27×) | 794 vs crypto-bigint 2,997 | −2,203 (0.26×) | 559 vs crypto-bigint 2,211 | −1,652 (0.25×) |
| mod, constant time | 512 | 1,038 vs crypto-bigint 3,694 | −2,656 (0.28×) | 752 vs crypto-bigint 2,753 | −2,001 (0.27×) | 1,074 vs crypto-bigint 4,294 | −3,220 (0.25×) | 825 vs crypto-bigint 3,109 | −2,284 (0.27×) |
| mod, constant time | 1024 | 2,524 vs crypto-bigint 8,465 | −5,941 (0.30×) | 1,781 vs crypto-bigint 7,004 | −5,223 (0.25×) | 2,739 vs crypto-bigint 10,348 | −7,609 (0.26×) | 1,986 vs crypto-bigint 7,446 | −5,460 (0.27×) |
| mod, constant time | 2048 | 7,608 vs crypto-bigint 22,407 | −14,799 (0.34×) | 5,040 vs crypto-bigint 16,941 | −11,901 (0.30×) | 8,307 vs crypto-bigint 27,335 | −19,028 (0.30×) | 6,024 vs crypto-bigint 18,507 | −12,483 (0.33×) |
| mod, constant time | 4096 | 25,837 vs crypto-bigint 67,358 | −41,521 (0.38×) | 16,274 vs crypto-bigint 43,319 | −27,045 (0.38×) | 28,499 vs crypto-bigint 85,083 | −56,584 (0.33×) | 20,574 vs crypto-bigint 50,661 | −30,087 (0.41×) |
| mulmod, constant time | 64 | 203 vs crypto-bigint 475 | −272 (0.43×) | 96.7 vs crypto-bigint 352 | −255 (0.27×) | 239 vs crypto-bigint 520 | −281 (0.46×) | 128 vs crypto-bigint 381 | −253 (0.34×) |
| mulmod, constant time | 128 | 484 vs crypto-bigint 1,285 | −801 (0.38×) | 286 vs crypto-bigint 1,031 | −745 (0.28×) | 591 vs crypto-bigint 1,537 | −946 (0.38×) | 380 vs crypto-bigint 1,142 | −762 (0.33×) |
| mulmod, constant time | 256 | 1,219 vs crypto-bigint 2,778 | −1,559 (0.44×) | 735 vs crypto-bigint 1,979 | −1,244 (0.37×) | 1,344 vs crypto-bigint 3,286 | −1,942 (0.41×) | 825 vs crypto-bigint 2,282 | −1,457 (0.36×) |
| mulmod, constant time | 384 | 1,925 vs crypto-bigint 4,834 | −2,909 (0.40×) | 1,209 vs crypto-bigint 3,239 | −2,030 (0.37×) | 2,264 vs crypto-bigint 5,638 | −3,374 (0.40×) | 1,482 vs crypto-bigint 3,714 | −2,232 (0.40×) |
| mulmod, constant time | 512 | 3,011 vs crypto-bigint 7,633 | −4,622 (0.39×) | 1,819 vs crypto-bigint 4,657 | −2,838 (0.39×) | 3,540 vs crypto-bigint 8,668 | −5,128 (0.41×) | 2,358 vs crypto-bigint 5,581 | −3,223 (0.42×) |
| mulmod, constant time | 1024 | 9,588 vs crypto-bigint 21,372 | −11,784 (0.45×) | 5,600 vs crypto-bigint 13,302 | −7,702 (0.42×) | 10,790 vs crypto-bigint 25,072 | −14,282 (0.43×) | 7,477 vs crypto-bigint 14,063 | −6,586 (0.53×) |
| mulmod, constant time | 2048 | 32,857 vs crypto-bigint 64,591 | −31,734 (0.51×) | 18,879 vs crypto-bigint 36,352 | −17,473 (0.52×) | 36,555 vs crypto-bigint 76,678 | −40,123 (0.48×) | 26,153 vs crypto-bigint 46,020 | −19,867 (0.57×) |
| mulmod, constant time | 4096 | 119,495 vs crypto-bigint 260,554 | −141,059 (0.46×) | 68,974 vs crypto-bigint 110,624 | −41,650 (0.62×) | 132,569 vs crypto-bigint 259,837 | −127,268 (0.51×) | 90,736 vs crypto-bigint 143,909 | −53,173 (0.63×) |
| add_mod, constant time | 64 | 9.0 | – | 9.0 | – | 18.3 | – | 14.2 | – |
| add_mod, constant time | 128 | 26.5 | – | 21.5 | – | 34.3 | – | 35.1 | – |
| add_mod, constant time | 256 | 82.0 | – | 65.1 | – | 107 | – | 73.9 | – |
| add_mod, constant time | 384 | 120 | – | 110 | – | 166 | – | 144 | – |
| add_mod, constant time | 512 | 149 | – | 145 | – | 236 | – | 179 | – |
| add_mod, constant time | 1024 | 324 | – | 270 | – | 601 | – | 343 | – |
| add_mod, constant time | 2048 | 861 | – | 529 | – | 1,710 | – | 755 | – |
| add_mod, constant time | 4096 | 1,882 | – | 1,710 | – | 1,977 | – | 2,002 | – |
| sub_mod, constant time | 64 | 10.2 | – | 9.3 | – | 18.7 | – | 14.4 | – |
| sub_mod, constant time | 128 | 15.6 | – | 13.0 | – | 29.7 | – | 22.1 | – |
| sub_mod, constant time | 256 | 45.8 | – | 32.2 | – | 73.9 | – | 65.7 | – |
| sub_mod, constant time | 384 | 102 | – | 51.1 | – | 135 | – | 98.3 | – |
| sub_mod, constant time | 512 | 140 | – | 88.2 | – | 223 | – | 141 | – |
| sub_mod, constant time | 1024 | 321 | – | 210 | – | 570 | – | 272 | – |
| sub_mod, constant time | 2048 | 741 | – | 510 | – | 1,321 | – | 635 | – |
| sub_mod, constant time | 4096 | 1,540 | – | 1,424 | – | 1,731 | – | 1,699 | – |
| add_mod, reduced operands, constant time | 64 | 5.8 vs crypto-bigint 13.0 | −7.2 (0.45×) | 6.1 vs crypto-bigint 12.2 | −6.1 (0.50×) | 6.0 vs crypto-bigint 11.9 | −5.9 (0.50×) | 7.4 vs crypto-bigint 13.2 | −5.8 (0.56×) |
| add_mod, reduced operands, constant time | 128 | 13.6 vs crypto-bigint 23.5 | −9.9 (0.58×) | 11.8 vs crypto-bigint 21.1 | −9.3 (0.56×) | 13.4 vs crypto-bigint 23.1 | −9.7 (0.58×) | 18.1 vs crypto-bigint 22.8 | −4.7 (0.79×) |
| add_mod, reduced operands, constant time | 256 | 29.1 vs crypto-bigint 47.5 | −18.4 (0.61×) | 23.6 vs crypto-bigint 43.0 | −19.4 (0.55×) | 31.0 vs crypto-bigint 57.2 | −26.2 (0.54×) | 32.0 vs crypto-bigint 55.7 | −23.7 (0.57×) |
| add_mod, reduced operands, constant time | 384 | 47.6 vs crypto-bigint 96.9 | −49.3 (0.49×) | 36.8 vs crypto-bigint 105 | −68.2 (0.35×) | 50.1 vs crypto-bigint 86.6 | −36.5 (0.58×) | 49.6 vs crypto-bigint 82.1 | −32.5 (0.60×) |
| add_mod, reduced operands, constant time | 512 | 72.0 vs crypto-bigint 108 | −36.0 (0.67×) | 49.7 vs crypto-bigint 93.7 | −44.0 (0.53×) | 88.4 vs crypto-bigint 140 | −51.6 (0.63×) | 64.7 vs crypto-bigint 128 | −63.3 (0.51×) |
| add_mod, reduced operands, constant time | 1024 | 137 vs crypto-bigint 377 | −240 (0.36×) | 95.6 vs crypto-bigint 343 | −247 (0.28×) | 292 vs crypto-bigint 459 | −167 (0.64×) | 124 vs crypto-bigint 383 | −259 (0.32×) |
| add_mod, reduced operands, constant time | 2048 | 278 vs crypto-bigint 564 | −286 (0.49×) | 181 vs crypto-bigint 504 | −323 (0.36×) | 648 vs crypto-bigint 878 | −230 (0.74×) | 291 vs crypto-bigint 771 | −480 (0.38×) |
| add_mod, reduced operands, constant time | 4096 | 804 vs crypto-bigint 1,175 | −371 (0.68×) | 726 vs crypto-bigint 1,029 | −303 (0.71×) | 859 vs crypto-bigint 1,632 | −773 (0.53×) | 869 vs crypto-bigint 1,448 | −579 (0.60×) |
| sub_mod, reduced operands, constant time | 64 | 4.8 vs crypto-bigint 5.0 | −0.2 (0.96×) | 4.9 vs crypto-bigint 5.0 | −0.1 (0.98×) | 4.7 vs crypto-bigint 5.0 | −0.3 (0.94×) | 5.0 vs crypto-bigint 5.0 | +0.0 (1.00×) |
| sub_mod, reduced operands, constant time | 128 | 9.7 vs crypto-bigint 10.2 | −0.5 (0.95×) | 9.0 vs crypto-bigint 11.0 | −2.0 (0.82×) | 9.0 vs crypto-bigint 10.8 | −1.8 (0.83×) | 12.1 vs crypto-bigint 11.0 | +1.1 (1.10×) |
| sub_mod, reduced operands, constant time | 256 | 24.4 vs crypto-bigint 28.1 | −3.7 (0.87×) | 20.2 vs crypto-bigint 23.6 | −3.4 (0.86×) | 31.0 vs crypto-bigint 31.6 | −0.6 (0.98×) | 28.1 vs crypto-bigint 30.1 | −2.0 (0.93×) |
| sub_mod, reduced operands, constant time | 384 | 40.4 vs crypto-bigint 48.9 | −8.5 (0.83×) | 33.3 vs crypto-bigint 39.1 | −5.8 (0.85×) | 46.3 vs crypto-bigint 57.5 | −11.2 (0.81×) | 44.7 vs crypto-bigint 52.3 | −7.6 (0.85×) |
| sub_mod, reduced operands, constant time | 512 | 64.8 vs crypto-bigint 71.3 | −6.5 (0.91×) | 46.4 vs crypto-bigint 53.6 | −7.2 (0.87×) | 89.6 vs crypto-bigint 94.8 | −5.2 (0.95×) | 66.9 vs crypto-bigint 105 | −38.1 (0.64×) |
| sub_mod, reduced operands, constant time | 1024 | 133 vs crypto-bigint 178 | −45.0 (0.75×) | 114 vs crypto-bigint 138 | −24.0 (0.83×) | 223 vs crypto-bigint 347 | −124 (0.64×) | 137 vs crypto-bigint 276 | −139 (0.50×) |
| sub_mod, reduced operands, constant time | 2048 | 294 vs crypto-bigint 520 | −226 (0.57×) | 254 vs crypto-bigint 455 | −201 (0.56×) | 549 vs crypto-bigint 564 | −15.0 (0.97×) | 293 vs crypto-bigint 510 | −217 (0.57×) |
| sub_mod, reduced operands, constant time | 4096 | 833 vs crypto-bigint 906 | −73.0 (0.92×) | 708 vs crypto-bigint 774 | −66.0 (0.91×) | 890 vs crypto-bigint 1,454 | −564 (0.61×) | 893 vs crypto-bigint 1,348 | −455 (0.66×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add | 128 | 2.9 vs native 2.8 | +0.1 (1.04×) | 2.5 vs bnum 2.0 | +0.5 (1.25×) | 2.7 vs crypto-bigint 2.8 | −0.1 (0.96×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add | 256 | 9.7 vs bnum 8.1 | +1.6 (1.20×) | 7.4 vs bnum 6.2 | +1.2 (1.19×) | 8.9 vs bnum 8.7 | +0.2 (1.02×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add | 384 | 14.8 vs ruint 15.4 | −0.6 (0.96×) | 12.3 vs crypto-bigint 12.3 | +0.0 (1.00×) | 16.0 vs ruint 15.2 | +0.8 (1.05×) | 18.1 vs crypto-bigint 18.0 | +0.1 (1.01×) |
| add | 512 | 23.5 vs bnum 22.5 | +1.0 (1.04×) | 18.3 vs ruint 18.8 | −0.5 (0.97×) | 25.3 vs bnum 22.8 | +2.5 (1.11×) | 31.1 vs bnum 28.3 | +2.8 (1.10×) |
| add | 1024 | 55.2 vs bnum 62.1 | −6.9 (0.89×) | 45.7 vs crypto-bigint 44.3 | +1.4 (1.03×) | 59.6 vs ruint 78.7 | −19.1 (0.76×) | 66.9 vs crypto-bigint 61.9 | +5.0 (1.08×) |
| add | 2048 | 115 vs ruint 202 | −87.0 (0.57×) | 105 vs ruint 159 | −54.0 (0.66×) | 126 vs ruint 214 | −88.0 (0.59×) | 141 vs ruint 204 | −63.0 (0.69×) |
| add | 4096 | 346 vs ruint 406 | −60.0 (0.85×) | 247 vs ruint 291 | −44.0 (0.85×) | 390 vs ruint 428 | −38.0 (0.91×) | 303 vs ruint 380 | −77.0 (0.80×) |
| mulmod | 64 | 68.6 vs ruint 85.1 | −16.5 (0.81×) | 34.9 vs ruint 42.7 | −7.8 (0.82×) | 123 vs ruint 179 | −56.0 (0.69×) | 42.3 vs ruint 52.4 | −10.1 (0.81×) |
| mulmod | 128 | 209 vs ruint 338 | −129 (0.62×) | 86.0 vs ruint 137 | −51.0 (0.63×) | 256 vs ruint 390 | −134 (0.66×) | 141 vs ruint 205 | −64.0 (0.69×) |
| mulmod | 256 | 570 vs ruint 812 | −242 (0.70×) | 278 vs ruint 310 | −32.0 (0.90×) | 681 vs ruint 893 | −212 (0.76×) | 346 vs ruint 378 | −32.0 (0.92×) |
| mulmod | 384 | 963 vs ruint 1,593 | −630 (0.60×) | 511 vs ruint 537 | −26.0 (0.95×) | 1,331 vs ruint 1,689 | −358 (0.79×) | 624 vs ruint 693 | −69.0 (0.90×) |
| mulmod | 512 | 1,575 vs ruint 2,836 | −1,261 (0.56×) | 892 vs ruint 854 | +38.0 (1.04×) | 2,084 vs ruint 3,285 | −1,201 (0.63×) | 1,084 vs ruint 1,118 | −34.0 (0.97×) |
| mulmod | 1024 | 5,313 vs ruint 10,688 | −5,375 (0.50×) | 2,754 vs ruint 2,602 | +152 (1.06×) | 6,776 vs ruint 12,538 | −5,762 (0.54×) | 3,531 vs ruint 3,603 | −72.0 (0.98×) |
| mulmod | 2048 | 17,395 vs ruint 35,052 | −17,657 (0.50×) | 8,602 vs ruint 10,671 | −2,069 (0.81×) | 21,892 vs ruint 44,732 | −22,840 (0.49×) | 13,239 vs ruint 13,819 | −580 (0.96×) |
| mulmod | 4096 | 55,781 vs ruint 161,376 | −105,595 (0.35×) | 26,810 vs ruint 41,186 | −14,376 (0.65×) | 68,561 vs ruint 168,797 | −100,236 (0.41×) | 45,522 vs ruint 53,265 | −7,743 (0.85×) |
| add_mod | 64 | 6.3 vs ruint 40.9 | −34.6 (0.15×) | 6.2 vs ruint 39.9 | −33.7 (0.16×) | 8.7 vs ruint 78.6 | −69.9 (0.11×) | 8.7 vs ruint 77.2 | −68.5 (0.11×) |
| add_mod | 128 | 31.9 vs ruint 40.0 | −8.1 (0.80×) | 10.5 vs ruint 30.6 | −20.1 (0.34×) | 46.5 vs ruint 52.7 | −6.2 (0.88×) | 16.1 vs ruint 37.7 | −21.6 (0.43×) |
| add_mod | 256 | 43.3 vs ruint 74.0 | −30.7 (0.59×) | 30.5 vs ruint 60.5 | −30.0 (0.50×) | 54.2 vs ruint 97.7 | −43.5 (0.55×) | 46.8 vs ruint 84.1 | −37.3 (0.56×) |
| add_mod | 384 | 54.7 vs ruint 99.4 | −44.7 (0.55×) | 44.4 vs ruint 81.4 | −37.0 (0.55×) | 81.9 vs ruint 126 | −44.1 (0.65×) | 67.1 vs ruint 105 | −37.9 (0.64×) |
| add_mod | 512 | 67.0 vs ruint 127 | −60.0 (0.53×) | 54.5 vs ruint 100 | −45.5 (0.55×) | 95.8 vs ruint 174 | −78.2 (0.55×) | 86.8 vs ruint 152 | −65.2 (0.57×) |
| add_mod | 1024 | 135 vs ruint 262 | −127 (0.52×) | 124 vs ruint 217 | −93.0 (0.57×) | 165 vs ruint 361 | −196 (0.46×) | 150 vs ruint 303 | −153 (0.50×) |
| add_mod | 2048 | 282 vs ruint 907 | −625 (0.31×) | 231 vs ruint 775 | −544 (0.30×) | 340 vs ruint 1,114 | −774 (0.31×) | 341 vs ruint 964 | −623 (0.35×) |
| add_mod | 4096 | 816 vs ruint 1,648 | −832 (0.50×) | 741 vs ruint 1,454 | −713 (0.51×) | 910 vs ruint 2,013 | −1,103 (0.45×) | 966 vs ruint 1,730 | −764 (0.56×) |
| sub_mod | 64 | 4.7 vs ruint 47.9 | −43.2 (0.10×) | 7.1 vs ruint 44.8 | −37.7 (0.16×) | 8.7 vs ruint 77.0 | −68.3 (0.11×) | 9.7 vs ruint 75.8 | −66.1 (0.13×) |
| sub_mod | 128 | 34.0 vs ruint 46.9 | −12.9 (0.72×) | 11.1 vs ruint 41.7 | −30.6 (0.27×) | 45.2 vs ruint 66.1 | −20.9 (0.68×) | 15.5 vs ruint 53.4 | −37.9 (0.29×) |
| sub_mod | 256 | 46.3 vs ruint 102 | −55.7 (0.45×) | 31.9 vs ruint 76.8 | −44.9 (0.42×) | 57.4 vs ruint 132 | −74.6 (0.43×) | 44.7 vs ruint 111 | −66.3 (0.40×) |
| sub_mod | 384 | 52.4 vs ruint 132 | −79.6 (0.40×) | 44.8 vs ruint 109 | −64.2 (0.41×) | 78.7 vs ruint 180 | −101 (0.44×) | 66.7 vs ruint 155 | −88.3 (0.43×) |
| sub_mod | 512 | 66.9 vs ruint 168 | −101 (0.40×) | 53.8 vs ruint 132 | −78.2 (0.41×) | 101 vs ruint 245 | −144 (0.41×) | 88.5 vs ruint 205 | −117 (0.43×) |
| sub_mod | 1024 | 122 vs ruint 410 | −288 (0.30×) | 104 vs ruint 353 | −249 (0.29×) | 171 vs ruint 638 | −467 (0.27×) | 179 vs ruint 608 | −429 (0.29×) |
| sub_mod | 2048 | 278 vs ruint 1,240 | −962 (0.22×) | 259 vs ruint 1,118 | −859 (0.23×) | 361 vs ruint 1,519 | −1,158 (0.24×) | 335 vs ruint 1,400 | −1,065 (0.24×) |
| sub_mod | 4096 | 833 vs ruint 2,249 | −1,416 (0.37×) | 815 vs ruint 2,076 | −1,261 (0.39×) | 901 vs ruint 2,757 | −1,856 (0.33×) | 1,091 vs ruint 2,387 | −1,296 (0.46×) |
| add_mod, reduced operands | 64 | 5.0 vs ruint 44.6 | −39.6 (0.11×) | 5.0 vs ruint 43.9 | −38.9 (0.11×) | 5.0 vs ruint 84.8 | −79.8 (0.06×) | 5.0 vs ruint 83.8 | −78.8 (0.06×) |
| add_mod, reduced operands | 128 | 6.7 vs ruint 23.3 | −16.6 (0.29×) | 7.5 vs ruint 23.6 | −16.1 (0.32×) | 7.1 vs ruint 24.2 | −17.1 (0.29×) | 9.9 vs ruint 25.1 | −15.2 (0.39×) |
| add_mod, reduced operands | 256 | 16.7 vs ruint 42.4 | −25.7 (0.39×) | 12.9 vs ruint 43.3 | −30.4 (0.30×) | 16.4 vs ruint 59.6 | −43.2 (0.28×) | 21.3 vs ruint 58.7 | −37.4 (0.36×) |
| add_mod, reduced operands | 384 | 29.1 vs ruint 61.5 | −32.4 (0.47×) | 19.5 vs ruint 59.7 | −40.2 (0.33×) | 28.4 vs ruint 90.3 | −61.9 (0.31×) | 31.7 vs ruint 82.3 | −50.6 (0.39×) |
| add_mod, reduced operands | 512 | 35.3 vs ruint 85.7 | −50.4 (0.41×) | 28.2 vs ruint 79.1 | −50.9 (0.36×) | 41.7 vs ruint 114 | −72.3 (0.37×) | 43.5 vs ruint 115 | −71.5 (0.38×) |
| add_mod, reduced operands | 1024 | 76.4 vs ruint 192 | −116 (0.40×) | 67.8 vs ruint 181 | −113 (0.37×) | 86.9 vs ruint 259 | −172 (0.34×) | 93.0 vs ruint 244 | −151 (0.38×) |
| add_mod, reduced operands | 2048 | 202 vs ruint 794 | −592 (0.25×) | 154 vs ruint 748 | −594 (0.21×) | 223 vs ruint 1,022 | −799 (0.22×) | 207 vs ruint 912 | −705 (0.23×) |
| add_mod, reduced operands | 4096 | 533 vs ruint 1,522 | −989 (0.35×) | 435 vs ruint 1,420 | −985 (0.31×) | 609 vs ruint 1,796 | −1,187 (0.34×) | 549 vs ruint 1,648 | −1,099 (0.33×) |
| sub_mod, reduced operands | 64 | 2.6 vs ruint 45.4 | −42.8 (0.06×) | 2.6 vs ruint 44.4 | −41.8 (0.06×) | 3.3 vs ruint 85.7 | −82.4 (0.04×) | 3.5 vs ruint 83.6 | −80.1 (0.04×) |
| sub_mod, reduced operands | 128 | 5.4 vs ruint 25.5 | −20.1 (0.21×) | 6.0 vs ruint 24.2 | −18.2 (0.25×) | 5.7 vs ruint 26.1 | −20.4 (0.22×) | 8.4 vs ruint 27.1 | −18.7 (0.31×) |
| sub_mod, reduced operands | 256 | 14.9 vs ruint 50.0 | −35.1 (0.30×) | 12.3 vs ruint 50.4 | −38.1 (0.24×) | 13.3 vs ruint 66.3 | −53.0 (0.20×) | 19.0 vs ruint 65.4 | −46.4 (0.29×) |
| sub_mod, reduced operands | 384 | 24.9 vs ruint 77.2 | −52.3 (0.32×) | 18.9 vs ruint 74.6 | −55.7 (0.25×) | 25.1 vs ruint 103 | −77.9 (0.24×) | 29.9 vs ruint 95.7 | −65.8 (0.31×) |
| sub_mod, reduced operands | 512 | 35.1 vs ruint 109 | −73.9 (0.32×) | 29.1 vs ruint 102 | −72.9 (0.29×) | 39.7 vs ruint 131 | −91.3 (0.30×) | 42.0 vs ruint 129 | −87.0 (0.33×) |
| sub_mod, reduced operands | 1024 | 67.7 vs ruint 261 | −193 (0.26×) | 69.3 vs ruint 239 | −170 (0.29×) | 80.2 vs ruint 316 | −236 (0.25×) | 90.8 vs ruint 316 | −225 (0.29×) |
| sub_mod, reduced operands | 2048 | 205 vs ruint 975 | −770 (0.21×) | 170 vs ruint 941 | −771 (0.18×) | 292 vs ruint 1,232 | −940 (0.24×) | 206 vs ruint 1,087 | −881 (0.19×) |
| sub_mod, reduced operands | 4096 | 526 vs ruint 1,883 | −1,357 (0.28×) | 454 vs ruint 1,741 | −1,287 (0.26×) | 608 vs ruint 2,249 | −1,641 (0.27×) | 556 vs ruint 2,005 | −1,449 (0.28×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add, constant time | 128 | 2.9 vs native 2.8 | +0.1 (1.04×) | 2.5 vs bnum 2.0 | +0.5 (1.25×) | 2.7 vs crypto-bigint 2.8 | −0.1 (0.96×) | 3.3 vs bnum 2.5 | +0.8 (1.32×) |
| add, constant time | 256 | 9.7 vs bnum 8.1 | +1.6 (1.20×) | 7.4 vs bnum 6.2 | +1.2 (1.19×) | 8.9 vs bnum 8.7 | +0.2 (1.02×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add, constant time | 384 | 14.8 vs ruint 15.4 | −0.6 (0.96×) | 12.3 vs crypto-bigint 12.3 | +0.0 (1.00×) | 16.0 vs ruint 15.2 | +0.8 (1.05×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add, constant time | 512 | 23.8 vs bnum 22.5 | +1.3 (1.06×) | 18.3 vs ruint 18.8 | −0.5 (0.97×) | 25.3 vs bnum 22.7 | +2.6 (1.11×) | 31.1 vs bnum 28.3 | +2.8 (1.10×) |
| add, constant time | 1024 | 55.1 vs bnum 62.0 | −6.9 (0.89×) | 45.7 vs crypto-bigint 43.8 | +1.9 (1.04×) | 59.6 vs ruint 78.4 | −18.8 (0.76×) | 66.9 vs crypto-bigint 61.9 | +5.0 (1.08×) |
| add, constant time | 2048 | 115 vs ruint 202 | −87.0 (0.57×) | 105 vs ruint 160 | −55.0 (0.66×) | 126 vs ruint 214 | −88.0 (0.59×) | 141 vs ruint 205 | −64.0 (0.69×) |
| add, constant time | 4096 | 347 vs ruint 407 | −60.0 (0.85×) | 247 vs ruint 290 | −43.0 (0.85×) | 388 vs ruint 428 | −40.0 (0.91×) | 303 vs ruint 381 | −78.0 (0.80×) |
| mulmod, constant time | 64 | 114 | – | 94.5 | – | 244 | – | 124 | – |
| mulmod, constant time | 128 | 567 | – | 241 | – | 596 | – | 370 | – |
| mulmod, constant time | 256 | 1,185 | – | 741 | – | 1,302 | – | 849 | – |
| mulmod, constant time | 384 | 1,923 | – | 1,202 | – | 2,252 | – | 1,485 | – |
| mulmod, constant time | 512 | 3,004 | – | 1,848 | – | 3,508 | – | 2,300 | – |
| mulmod, constant time | 1024 | 9,532 | – | 5,612 | – | 10,977 | – | 7,579 | – |
| mulmod, constant time | 2048 | 32,868 | – | 19,267 | – | 36,959 | – | 25,601 | – |
| mulmod, constant time | 4096 | 120,192 | – | 68,248 | – | 132,024 | – | 92,001 | – |
| add_mod, constant time | 64 | 12.3 | – | 13.0 | – | 19.6 | – | 14.4 | – |
| add_mod, constant time | 128 | 28.4 | – | 22.5 | – | 37.9 | – | 34.1 | – |
| add_mod, constant time | 256 | 83.7 | – | 66.7 | – | 98.5 | – | 91.9 | – |
| add_mod, constant time | 384 | 112 | – | 108 | – | 174 | – | 127 | – |
| add_mod, constant time | 512 | 150 | – | 135 | – | 243 | – | 173 | – |
| add_mod, constant time | 1024 | 326 | – | 261 | – | 620 | – | 352 | – |
| add_mod, constant time | 2048 | 861 | – | 517 | – | 1,575 | – | 757 | – |
| add_mod, constant time | 4096 | 1,873 | – | 1,691 | – | 1,967 | – | 2,017 | – |
| sub_mod, constant time | 64 | 12.1 | – | 11.0 | – | 18.2 | – | 13.8 | – |
| sub_mod, constant time | 128 | 24.0 | – | 19.0 | – | 32.7 | – | 22.5 | – |
| sub_mod, constant time | 256 | 54.6 | – | 36.9 | – | 76.3 | – | 59.0 | – |
| sub_mod, constant time | 384 | 102 | – | 57.5 | – | 121 | – | 85.4 | – |
| sub_mod, constant time | 512 | 142 | – | 85.8 | – | 200 | – | 117 | – |
| sub_mod, constant time | 1024 | 309 | – | 199 | – | 521 | – | 254 | – |
| sub_mod, constant time | 2048 | 742 | – | 510 | – | 1,287 | – | 612 | – |
| sub_mod, constant time | 4096 | 1,542 | – | 1,423 | – | 1,735 | – | 1,691 | – |
| add_mod, reduced operands, constant time | 64 | 8.1 | – | 8.0 | – | 8.0 | – | 8.4 | – |
| add_mod, reduced operands, constant time | 128 | 15.0 | – | 13.1 | – | 14.6 | – | 16.4 | – |
| add_mod, reduced operands, constant time | 256 | 30.8 | – | 23.0 | – | 28.8 | – | 31.2 | – |
| add_mod, reduced operands, constant time | 384 | 51.5 | – | 35.0 | – | 50.2 | – | 49.7 | – |
| add_mod, reduced operands, constant time | 512 | 58.4 | – | 50.4 | – | 68.6 | – | 63.8 | – |
| add_mod, reduced operands, constant time | 1024 | 142 | – | 91.4 | – | 304 | – | 144 | – |
| add_mod, reduced operands, constant time | 2048 | 276 | – | 183 | – | 689 | – | 299 | – |
| add_mod, reduced operands, constant time | 4096 | 802 | – | 705 | – | 860 | – | 893 | – |
| sub_mod, reduced operands, constant time | 64 | 4.0 | – | 5.0 | – | 4.0 | – | 5.1 | – |
| sub_mod, reduced operands, constant time | 128 | 10.6 | – | 10.0 | – | 10.0 | – | 10.1 | – |
| sub_mod, reduced operands, constant time | 256 | 23.1 | – | 19.1 | – | 31.0 | – | 28.1 | – |
| sub_mod, reduced operands, constant time | 384 | 41.6 | – | 32.7 | – | 50.8 | – | 43.5 | – |
| sub_mod, reduced operands, constant time | 512 | 59.4 | – | 44.4 | – | 91.5 | – | 62.9 | – |
| sub_mod, reduced operands, constant time | 1024 | 129 | – | 112 | – | 238 | – | 141 | – |
| sub_mod, reduced operands, constant time | 2048 | 300 | – | 257 | – | 572 | – | 303 | – |
| sub_mod, reduced operands, constant time | 4096 | 831 | – | 689 | – | 902 | – | 849 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.4 (1.8×) |
| add | 256 | 4.7 | 10.3 (2.2×) | 8.7 (1.9×) | 10.0 (2.1×) | 11.5 (2.4×) |
| add | 384 | 7.1 | 16.2 (2.3×) | 12.3 (1.7×) | 15.2 (2.1×) | 18.0 (2.5×) |
| add | 512 | 9.2 | 26.3 (2.9×) | 20.8 (2.3×) | 26.4 (2.9×) | 32.1 (3.5×) |
| add | 1024 | 23.3 | 55.3 (2.4×) | 45.3 (1.9×) | 72.4 (3.1×) | 69.0 (3.0×) |
| add | 2048 | 53.5 | 117 (2.2×) | 106 (2.0×) | 143 (2.7×) | 147 (2.7×) |
| add | 4096 | 158 | 346 (2.2×) | 259 (1.6×) | 421 (2.7×) | 314 (2.0×) |
| mul | 64 | 1.5 | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul | 128 | 3.0 | 10.0 (3.3×) | 6.5 (2.2×) | 12.3 (4.1×) | 5.6 (1.9×) |
| mul | 256 | 12.2 | 47.9 (3.9×) | 16.8 (1.4×) | 67.0 (5.5×) | 26.0 (2.1×) |
| mul | 384 | 27.7 | 115 (4.2×) | 54.0 (1.9×) | 156 (5.6×) | 68.6 (2.5×) |
| mul | 512 | 52.1 | 205 (3.9×) | 71.3 (1.4×) | 306 (5.9×) | 129 (2.5×) |
| mul | 1024 | 221 | 1,235 (5.6×) | 315 (1.4×) | 1,496 (6.8×) | 565 (2.6×) |
| mul | 2048 | 1,113 | 5,581 (5.0×) | 2,316 (2.1×) | 6,612 (5.9×) | 3,535 (3.2×) |
| mul | 4096 | 3,859 | 18,145 (4.7×) | 8,098 (2.1×) | 21,248 (5.5×) | 11,918 (3.1×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.3 | 59.9 (3.7×) | 28.4 (1.7×) | 102 (6.3×) | 34.4 (2.1×) |
| div | 256 | 46.0 | 206 (4.5×) | 84.3 (1.8×) | 220 (4.8×) | 108 (2.3×) |
| div | 384 | 111 | 347 (3.1×) | 206 (1.9×) | 412 (3.7×) | 287 (2.6×) |
| div | 512 | 155 | 474 (3.1×) | 306 (2.0×) | 557 (3.6×) | 400 (2.6×) |
| div | 1024 | 462 | 1,135 (2.5×) | 688 (1.5×) | 1,485 (3.2×) | 881 (1.9×) |
| div | 2048 | 1,388 | 3,214 (2.3×) | 2,011 (1.4×) | 4,257 (3.1×) | 2,242 (1.6×) |
| div | 4096 | 4,594 | 12,825 (2.8×) | 6,997 (1.5×) | 15,089 (3.3×) | 7,557 (1.6×) |
| mod | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 17.0 | 52.8 (3.1×) | 29.7 (1.7×) | 89.7 (5.3×) | 33.7 (2.0×) |
| mod | 256 | 41.5 | 207 (5.0×) | 74.1 (1.8×) | 218 (5.3×) | 108 (2.6×) |
| mod | 384 | 119 | 347 (2.9×) | 197 (1.7×) | 405 (3.4×) | 268 (2.3×) |
| mod | 512 | 158 | 471 (3.0×) | 283 (1.8×) | 566 (3.6×) | 352 (2.2×) |
| mod | 1024 | 485 | 1,194 (2.5×) | 705 (1.5×) | 1,479 (3.0×) | 904 (1.9×) |
| mod | 2048 | 1,407 | 3,307 (2.4×) | 1,933 (1.4×) | 4,289 (3.0×) | 2,179 (1.5×) |
| mod | 4096 | 4,562 | 12,799 (2.8×) | 6,607 (1.4×) | 15,019 (3.3×) | 7,193 (1.6×) |
| mulmod | 64 | 11.9 | 62.4 (5.2×) | 31.4 (2.6×) | 105 (8.8×) | 37.4 (3.1×) |
| mulmod | 128 | 47.4 | 205 (4.3×) | 81.8 (1.7×) | 237 (5.0×) | 128 (2.7×) |
| mulmod | 256 | 162 | 580 (3.6×) | 280 (1.7×) | 692 (4.3×) | 363 (2.2×) |
| mulmod | 384 | 297 | 968 (3.3×) | 511 (1.7×) | 1,338 (4.5×) | 630 (2.1×) |
| mulmod | 512 | 467 | 1,582 (3.4×) | 890 (1.9×) | 2,189 (4.7×) | 1,070 (2.3×) |
| mulmod | 1024 | 1,365 | 5,333 (3.9×) | 2,725 (2.0×) | 6,685 (4.9×) | 3,625 (2.7×) |
| mulmod | 2048 | 5,191 | 17,415 (3.4×) | 8,579 (1.7×) | 21,754 (4.2×) | 13,227 (2.5×) |
| mulmod | 4096 | 14,910 | 56,014 (3.8×) | 26,920 (1.8×) | 68,548 (4.6×) | 45,349 (3.0×) |
| add_mod | 64 | 2.9 | 4.0 (1.4×) | 4.0 (1.4×) | 6.7 (2.3×) | 7.0 (2.4×) |
| add_mod | 128 | 4.8 | 8.7 (1.8×) | 9.1 (1.9×) | 11.1 (2.3×) | 14.2 (3.0×) |
| add_mod | 256 | 9.7 | 21.8 (2.2×) | 21.8 (2.2×) | 24.5 (2.5×) | 31.2 (3.2×) |
| add_mod | 384 | 15.6 | 30.7 (2.0×) | 28.4 (1.8×) | 42.9 (2.8×) | 48.7 (3.1×) |
| add_mod | 512 | 27.7 | 44.4 (1.6×) | 37.8 (1.4×) | 67.2 (2.4×) | 63.4 (2.3×) |
| add_mod | 1024 | 71.7 | 112 (1.6×) | 105 (1.5×) | 137 (1.9×) | 134 (1.9×) |
| add_mod | 2048 | 149 | 248 (1.7×) | 239 (1.6×) | 376 (2.5×) | 290 (1.9×) |
| add_mod | 4096 | 268 | 701 (2.6×) | 732 (2.7×) | 797 (3.0×) | 904 (3.4×) |
| sub_mod | 64 | 2.4 | 4.0 (1.7×) | 4.0 (1.7×) | 7.2 (3.0×) | 9.1 (3.8×) |
| sub_mod | 128 | 4.3 | 10.5 (2.4×) | 10.2 (2.4×) | 10.8 (2.5×) | 16.8 (3.9×) |
| sub_mod | 256 | 8.7 | 23.1 (2.7×) | 19.8 (2.3×) | 25.4 (2.9×) | 28.7 (3.3×) |
| sub_mod | 384 | 15.6 | 30.8 (2.0×) | 27.1 (1.7×) | 44.2 (2.8×) | 46.1 (3.0×) |
| sub_mod | 512 | 25.2 | 44.2 (1.8×) | 37.3 (1.5×) | 62.5 (2.5×) | 64.7 (2.6×) |
| sub_mod | 1024 | 66.3 | 103 (1.6×) | 110 (1.7×) | 126 (1.9×) | 136 (2.1×) |
| sub_mod | 2048 | 146 | 232 (1.6×) | 247 (1.7×) | 294 (2.0×) | 273 (1.9×) |
| sub_mod | 4096 | 241 | 703 (2.9×) | 743 (3.1×) | 806 (3.3×) | 953 (4.0×) |
| add_mod, reduced operands | 64 | 2.8 | 4.2 (1.5×) | 4.1 (1.5×) | 4.4 (1.6×) | 4.4 (1.6×) |
| add_mod, reduced operands | 128 | 4.3 | 7.8 (1.8×) | 9.1 (2.1×) | 7.8 (1.8×) | 11.6 (2.7×) |
| add_mod, reduced operands | 256 | 10.4 | 17.1 (1.6×) | 15.5 (1.5×) | 17.2 (1.7×) | 22.3 (2.1×) |
| add_mod, reduced operands | 384 | 13.8 | 27.8 (2.0×) | 22.1 (1.6×) | 28.7 (2.1×) | 32.0 (2.3×) |
| add_mod, reduced operands | 512 | 19.6 | 34.6 (1.8×) | 28.4 (1.4×) | 40.2 (2.1×) | 42.8 (2.2×) |
| add_mod, reduced operands | 1024 | 48.3 | 77.0 (1.6×) | 65.8 (1.4×) | 86.8 (1.8×) | 92.5 (1.9×) |
| add_mod, reduced operands | 2048 | 89.7 | 203 (2.3×) | 149 (1.7×) | 233 (2.6×) | 220 (2.5×) |
| add_mod, reduced operands | 4096 | 227 | 533 (2.3×) | 458 (2.0×) | 565 (2.5×) | 624 (2.7×) |
| sub_mod, reduced operands | 64 | 2.2 | 3.5 (1.6×) | 4.0 (1.8×) | 4.4 (2.0×) | 5.5 (2.5×) |
| sub_mod, reduced operands | 128 | 4.7 | 6.8 (1.4×) | 7.3 (1.6×) | 7.2 (1.5×) | 9.5 (2.0×) |
| sub_mod, reduced operands | 256 | 7.3 | 16.9 (2.3×) | 13.3 (1.8×) | 14.9 (2.0×) | 20.6 (2.8×) |
| sub_mod, reduced operands | 384 | 11.0 | 26.3 (2.4×) | 20.7 (1.9×) | 25.4 (2.3×) | 31.1 (2.8×) |
| sub_mod, reduced operands | 512 | 18.3 | 34.2 (1.9×) | 29.6 (1.6×) | 38.2 (2.1×) | 43.4 (2.4×) |
| sub_mod, reduced operands | 1024 | 44.7 | 67.2 (1.5×) | 69.8 (1.6×) | 77.2 (1.7×) | 92.5 (2.1×) |
| sub_mod, reduced operands | 2048 | 83.2 | 203 (2.4×) | 157 (1.9×) | 289 (3.5×) | 207 (2.5×) |
| sub_mod, reduced operands | 4096 | 217 | 528 (2.4×) | 459 (2.1×) | 549 (2.5×) | 557 (2.6×) |
| add, constant time | 64 | 1.4 | 2.2 (1.6×) | 2.1 (1.5×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.3 (1.8×) |
| add, constant time | 256 | 4.5 | 10.3 (2.3×) | 8.7 (1.9×) | 10.0 (2.2×) | 11.5 (2.6×) |
| add, constant time | 384 | 7.0 | 16.2 (2.3×) | 12.3 (1.8×) | 15.2 (2.2×) | 18.0 (2.6×) |
| add, constant time | 512 | 9.5 | 26.2 (2.8×) | 20.8 (2.2×) | 26.4 (2.8×) | 32.2 (3.4×) |
| add, constant time | 1024 | 23.4 | 55.3 (2.4×) | 45.9 (2.0×) | 72.7 (3.1×) | 69.2 (3.0×) |
| add, constant time | 2048 | 53.3 | 117 (2.2×) | 108 (2.0×) | 142 (2.7×) | 147 (2.8×) |
| add, constant time | 4096 | 162 | 346 (2.1×) | 259 (1.6×) | 403 (2.5×) | 317 (2.0×) |
| mul, constant time | 64 | 1.5 | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul, constant time | 128 | 3.0 | 10.0 (3.3×) | 6.5 (2.2×) | 12.3 (4.1×) | 5.6 (1.9×) |
| mul, constant time | 256 | 12.4 | 47.9 (3.9×) | 16.8 (1.4×) | 66.4 (5.4×) | 26.0 (2.1×) |
| mul, constant time | 384 | 30.0 | 115 (3.8×) | 54.0 (1.8×) | 165 (5.5×) | 69.4 (2.3×) |
| mul, constant time | 512 | 50.9 | 205 (4.0×) | 70.4 (1.4×) | 307 (6.0×) | 130 (2.6×) |
| mul, constant time | 1024 | 226 | 1,189 (5.3×) | 315 (1.4×) | 1,506 (6.7×) | 565 (2.5×) |
| mul, constant time | 2048 | 1,154 | 5,782 (5.0×) | 2,403 (2.1×) | 6,698 (5.8×) | 3,640 (3.2×) |
| mul, constant time | 4096 | 4,213 | 18,939 (4.5×) | 8,417 (2.0×) | 21,596 (5.1×) | 12,501 (3.0×) |
| div, constant time | 64 | 25.1 | 83.5 (3.3×) | 47.3 (1.9×) | 145 (5.8×) | 61.5 (2.5×) |
| div, constant time | 128 | 98.8 | 273 (2.8×) | 150 (1.5×) | 351 (3.6×) | 230 (2.3×) |
| div, constant time | 256 | 199 | 512 (2.6×) | 241 (1.2×) | 622 (3.1×) | 472 (2.4×) |
| div, constant time | 384 | 291 | 659 (2.3×) | 369 (1.3×) | 815 (2.8×) | 538 (1.8×) |
| div, constant time | 512 | 461 | 841 (1.8×) | 665 (1.4×) | 1,175 (2.5×) | 856 (1.9×) |
| div, constant time | 1024 | 1,139 | 2,463 (2.2×) | 1,779 (1.6×) | 2,848 (2.5×) | 2,211 (1.9×) |
| div, constant time | 2048 | 3,426 | 7,293 (2.1×) | 5,082 (1.5×) | 8,383 (2.4×) | 6,350 (1.9×) |
| div, constant time | 4096 | 11,846 | 25,512 (2.2×) | 16,252 (1.4×) | 27,637 (2.3×) | 20,890 (1.8×) |
| mod, constant time | 64 | 25.5 | 83.3 (3.3×) | 48.0 (1.9×) | 141 (5.5×) | 67.4 (2.6×) |
| mod, constant time | 128 | 105 | 269 (2.6×) | 138 (1.3×) | 332 (3.2×) | 223 (2.1×) |
| mod, constant time | 256 | 216 | 526 (2.4×) | 331 (1.5×) | 572 (2.6×) | 411 (1.9×) |
| mod, constant time | 384 | 311 | 746 (2.4×) | 530 (1.7×) | 794 (2.6×) | 559 (1.8×) |
| mod, constant time | 512 | 466 | 1,038 (2.2×) | 752 (1.6×) | 1,074 (2.3×) | 825 (1.8×) |
| mod, constant time | 1024 | 1,147 | 2,524 (2.2×) | 1,781 (1.6×) | 2,739 (2.4×) | 1,986 (1.7×) |
| mod, constant time | 2048 | 3,445 | 7,608 (2.2×) | 5,040 (1.5×) | 8,307 (2.4×) | 6,024 (1.7×) |
| mod, constant time | 4096 | 11,904 | 25,837 (2.2×) | 16,274 (1.4×) | 28,499 (2.4×) | 20,574 (1.7×) |
| mulmod, constant time | 64 | 51.9 | 203 (3.9×) | 96.7 (1.9×) | 239 (4.6×) | 128 (2.5×) |
| mulmod, constant time | 128 | 201 | 484 (2.4×) | 286 (1.4×) | 591 (2.9×) | 380 (1.9×) |
| mulmod, constant time | 256 | 453 | 1,219 (2.7×) | 735 (1.6×) | 1,344 (3.0×) | 825 (1.8×) |
| mulmod, constant time | 384 | 823 | 1,925 (2.3×) | 1,209 (1.5×) | 2,264 (2.8×) | 1,482 (1.8×) |
| mulmod, constant time | 512 | 1,189 | 3,011 (2.5×) | 1,819 (1.5×) | 3,540 (3.0×) | 2,358 (2.0×) |
| mulmod, constant time | 1024 | 3,856 | 9,588 (2.5×) | 5,600 (1.5×) | 10,790 (2.8×) | 7,477 (1.9×) |
| mulmod, constant time | 2048 | 13,721 | 32,857 (2.4×) | 18,879 (1.4×) | 36,555 (2.7×) | 26,153 (1.9×) |
| mulmod, constant time | 4096 | 48,937 | 119,495 (2.4×) | 68,974 (1.4×) | 132,569 (2.7×) | 90,736 (1.9×) |
| add_mod, constant time | 64 | 6.6 | 9.0 (1.4×) | 9.0 (1.4×) | 18.3 (2.8×) | 14.2 (2.2×) |
| add_mod, constant time | 128 | 11.0 | 26.5 (2.4×) | 21.5 (2.0×) | 34.3 (3.1×) | 35.1 (3.2×) |
| add_mod, constant time | 256 | 20.6 | 82.0 (4.0×) | 65.1 (3.2×) | 107 (5.2×) | 73.9 (3.6×) |
| add_mod, constant time | 384 | 38.7 | 120 (3.1×) | 110 (2.8×) | 166 (4.3×) | 144 (3.7×) |
| add_mod, constant time | 512 | 56.5 | 149 (2.6×) | 145 (2.6×) | 236 (4.2×) | 179 (3.2×) |
| add_mod, constant time | 1024 | 130 | 324 (2.5×) | 270 (2.1×) | 601 (4.6×) | 343 (2.6×) |
| add_mod, constant time | 2048 | 303 | 861 (2.8×) | 529 (1.7×) | 1,710 (5.6×) | 755 (2.5×) |
| add_mod, constant time | 4096 | 511 | 1,882 (3.7×) | 1,710 (3.3×) | 1,977 (3.9×) | 2,002 (3.9×) |
| sub_mod, constant time | 64 | 6.2 | 10.2 (1.6×) | 9.3 (1.5×) | 18.7 (3.0×) | 14.4 (2.3×) |
| sub_mod, constant time | 128 | 7.4 | 15.6 (2.1×) | 13.0 (1.8×) | 29.7 (4.0×) | 22.1 (3.0×) |
| sub_mod, constant time | 256 | 13.7 | 45.8 (3.3×) | 32.2 (2.4×) | 73.9 (5.4×) | 65.7 (4.8×) |
| sub_mod, constant time | 384 | 27.0 | 102 (3.8×) | 51.1 (1.9×) | 135 (5.0×) | 98.3 (3.6×) |
| sub_mod, constant time | 512 | 33.6 | 140 (4.2×) | 88.2 (2.6×) | 223 (6.6×) | 141 (4.2×) |
| sub_mod, constant time | 1024 | 96.9 | 321 (3.3×) | 210 (2.2×) | 570 (5.9×) | 272 (2.8×) |
| sub_mod, constant time | 2048 | 414 | 741 (1.8×) | 510 (1.2×) | 1,321 (3.2×) | 635 (1.5×) |
| sub_mod, constant time | 4096 | 487 | 1,540 (3.2×) | 1,424 (2.9×) | 1,731 (3.6×) | 1,699 (3.5×) |
| add_mod, reduced operands, constant time | 64 | 3.8 | 5.8 (1.5×) | 6.1 (1.6×) | 6.0 (1.6×) | 7.4 (1.9×) |
| add_mod, reduced operands, constant time | 128 | 6.9 | 13.6 (2.0×) | 11.8 (1.7×) | 13.4 (1.9×) | 18.1 (2.6×) |
| add_mod, reduced operands, constant time | 256 | 9.9 | 29.1 (2.9×) | 23.6 (2.4×) | 31.0 (3.1×) | 32.0 (3.2×) |
| add_mod, reduced operands, constant time | 384 | 13.2 | 47.6 (3.6×) | 36.8 (2.8×) | 50.1 (3.8×) | 49.6 (3.8×) |
| add_mod, reduced operands, constant time | 512 | 19.8 | 72.0 (3.6×) | 49.7 (2.5×) | 88.4 (4.5×) | 64.7 (3.3×) |
| add_mod, reduced operands, constant time | 1024 | 57.8 | 137 (2.4×) | 95.6 (1.7×) | 292 (5.1×) | 124 (2.1×) |
| add_mod, reduced operands, constant time | 2048 | 132 | 278 (2.1×) | 181 (1.4×) | 648 (4.9×) | 291 (2.2×) |
| add_mod, reduced operands, constant time | 4096 | 269 | 804 (3.0×) | 726 (2.7×) | 859 (3.2×) | 869 (3.2×) |
| sub_mod, reduced operands, constant time | 64 | 3.2 | 4.8 (1.5×) | 4.9 (1.5×) | 4.7 (1.5×) | 5.0 (1.6×) |
| sub_mod, reduced operands, constant time | 128 | 5.1 | 9.7 (1.9×) | 9.0 (1.8×) | 9.0 (1.8×) | 12.1 (2.4×) |
| sub_mod, reduced operands, constant time | 256 | 8.9 | 24.4 (2.7×) | 20.2 (2.3×) | 31.0 (3.5×) | 28.1 (3.2×) |
| sub_mod, reduced operands, constant time | 384 | 11.0 | 40.4 (3.7×) | 33.3 (3.0×) | 46.3 (4.2×) | 44.7 (4.1×) |
| sub_mod, reduced operands, constant time | 512 | 18.5 | 64.8 (3.5×) | 46.4 (2.5×) | 89.6 (4.8×) | 66.9 (3.6×) |
| sub_mod, reduced operands, constant time | 1024 | 48.5 | 133 (2.7×) | 114 (2.4×) | 223 (4.6×) | 137 (2.8×) |
| sub_mod, reduced operands, constant time | 2048 | 116 | 294 (2.5×) | 254 (2.2×) | 549 (4.7×) | 293 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 270 | 833 (3.1×) | 708 (2.6×) | 890 (3.3×) | 893 (3.3×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add | 128 | 1.4 | 2.9 (2.1×) | 2.5 (1.8×) | 2.7 (1.9×) | 3.2 (2.3×) |
| add | 256 | 2.5 | 9.7 (3.9×) | 7.4 (3.0×) | 8.9 (3.6×) | 11.3 (4.5×) |
| add | 384 | 3.5 | 14.8 (4.2×) | 12.3 (3.5×) | 16.0 (4.6×) | 18.1 (5.2×) |
| add | 512 | 4.0 | 23.5 (5.9×) | 18.3 (4.6×) | 25.3 (6.3×) | 31.1 (7.8×) |
| add | 1024 | 22.1 | 55.2 (2.5×) | 45.7 (2.1×) | 59.6 (2.7×) | 66.9 (3.0×) |
| add | 2048 | 51.4 | 115 (2.2×) | 105 (2.0×) | 126 (2.5×) | 141 (2.7×) |
| add | 4096 | 163 | 346 (2.1×) | 247 (1.5×) | 390 (2.4×) | 303 (1.9×) |
| mulmod | 64 | 23.0 | 68.6 (3.0×) | 34.9 (1.5×) | 123 (5.3×) | 42.3 (1.8×) |
| mulmod | 128 | 50.7 | 209 (4.1×) | 86.0 (1.7×) | 256 (5.0×) | 141 (2.8×) |
| mulmod | 256 | 183 | 570 (3.1×) | 278 (1.5×) | 681 (3.7×) | 346 (1.9×) |
| mulmod | 384 | 316 | 963 (3.0×) | 511 (1.6×) | 1,331 (4.2×) | 624 (2.0×) |
| mulmod | 512 | 483 | 1,575 (3.3×) | 892 (1.8×) | 2,084 (4.3×) | 1,084 (2.2×) |
| mulmod | 1024 | 1,337 | 5,313 (4.0×) | 2,754 (2.1×) | 6,776 (5.1×) | 3,531 (2.6×) |
| mulmod | 2048 | 5,198 | 17,395 (3.3×) | 8,602 (1.7×) | 21,892 (4.2×) | 13,239 (2.5×) |
| mulmod | 4096 | 14,977 | 55,781 (3.7×) | 26,810 (1.8×) | 68,561 (4.6×) | 45,522 (3.0×) |
| add_mod | 64 | 8.0 | 6.3 (0.8×) | 6.2 (0.8×) | 8.7 (1.1×) | 8.7 (1.1×) |
| add_mod | 128 | 11.9 | 31.9 (2.7×) | 10.5 (0.9×) | 46.5 (3.9×) | 16.1 (1.4×) |
| add_mod | 256 | 18.5 | 43.3 (2.3×) | 30.5 (1.6×) | 54.2 (2.9×) | 46.8 (2.5×) |
| add_mod | 384 | 22.9 | 54.7 (2.4×) | 44.4 (1.9×) | 81.9 (3.6×) | 67.1 (2.9×) |
| add_mod | 512 | 33.5 | 67.0 (2.0×) | 54.5 (1.6×) | 95.8 (2.9×) | 86.8 (2.6×) |
| add_mod | 1024 | 79.2 | 135 (1.7×) | 124 (1.6×) | 165 (2.1×) | 150 (1.9×) |
| add_mod | 2048 | 155 | 282 (1.8×) | 231 (1.5×) | 340 (2.2×) | 341 (2.2×) |
| add_mod | 4096 | 333 | 816 (2.5×) | 741 (2.2×) | 910 (2.7×) | 966 (2.9×) |
| sub_mod | 64 | 7.3 | 4.7 (0.6×) | 7.1 (1.0×) | 8.7 (1.2×) | 9.7 (1.3×) |
| sub_mod | 128 | 11.3 | 34.0 (3.0×) | 11.1 (1.0×) | 45.2 (4.0×) | 15.5 (1.4×) |
| sub_mod | 256 | 14.4 | 46.3 (3.2×) | 31.9 (2.2×) | 57.4 (4.0×) | 44.7 (3.1×) |
| sub_mod | 384 | 23.1 | 52.4 (2.3×) | 44.8 (1.9×) | 78.7 (3.4×) | 66.7 (2.9×) |
| sub_mod | 512 | 37.4 | 66.9 (1.8×) | 53.8 (1.4×) | 101 (2.7×) | 88.5 (2.4×) |
| sub_mod | 1024 | 79.9 | 122 (1.5×) | 104 (1.3×) | 171 (2.1×) | 179 (2.2×) |
| sub_mod | 2048 | 144 | 278 (1.9×) | 259 (1.8×) | 361 (2.5×) | 335 (2.3×) |
| sub_mod | 4096 | 320 | 833 (2.6×) | 815 (2.5×) | 901 (2.8×) | 1,091 (3.4×) |
| add_mod, reduced operands | 64 | 4.9 | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) |
| add_mod, reduced operands | 128 | 6.0 | 6.7 (1.1×) | 7.5 (1.2×) | 7.1 (1.2×) | 9.9 (1.7×) |
| add_mod, reduced operands | 256 | 12.9 | 16.7 (1.3×) | 12.9 (1.0×) | 16.4 (1.3×) | 21.3 (1.7×) |
| add_mod, reduced operands | 384 | 14.6 | 29.1 (2.0×) | 19.5 (1.3×) | 28.4 (1.9×) | 31.7 (2.2×) |
| add_mod, reduced operands | 512 | 20.2 | 35.3 (1.7×) | 28.2 (1.4×) | 41.7 (2.1×) | 43.5 (2.2×) |
| add_mod, reduced operands | 1024 | 44.9 | 76.4 (1.7×) | 67.8 (1.5×) | 86.9 (1.9×) | 93.0 (2.1×) |
| add_mod, reduced operands | 2048 | 83.0 | 202 (2.4×) | 154 (1.9×) | 223 (2.7×) | 207 (2.5×) |
| add_mod, reduced operands | 4096 | 240 | 533 (2.2×) | 435 (1.8×) | 609 (2.5×) | 549 (2.3×) |
| sub_mod, reduced operands | 64 | 3.9 | 2.6 (0.7×) | 2.6 (0.7×) | 3.3 (0.8×) | 3.5 (0.9×) |
| sub_mod, reduced operands | 128 | 4.8 | 5.4 (1.1×) | 6.0 (1.2×) | 5.7 (1.2×) | 8.4 (1.8×) |
| sub_mod, reduced operands | 256 | 7.4 | 14.9 (2.0×) | 12.3 (1.7×) | 13.3 (1.8×) | 19.0 (2.6×) |
| sub_mod, reduced operands | 384 | 10.1 | 24.9 (2.5×) | 18.9 (1.9×) | 25.1 (2.5×) | 29.9 (3.0×) |
| sub_mod, reduced operands | 512 | 16.1 | 35.1 (2.2×) | 29.1 (1.8×) | 39.7 (2.5×) | 42.0 (2.6×) |
| sub_mod, reduced operands | 1024 | 37.8 | 67.7 (1.8×) | 69.3 (1.8×) | 80.2 (2.1×) | 90.8 (2.4×) |
| sub_mod, reduced operands | 2048 | 75.6 | 205 (2.7×) | 170 (2.2×) | 292 (3.9×) | 206 (2.7×) |
| sub_mod, reduced operands | 4096 | 238 | 526 (2.2×) | 454 (1.9×) | 608 (2.6×) | 556 (2.3×) |
| add, constant time | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add, constant time | 128 | 1.3 | 2.9 (2.2×) | 2.5 (1.9×) | 2.7 (2.1×) | 3.3 (2.5×) |
| add, constant time | 256 | 2.3 | 9.7 (4.2×) | 7.4 (3.2×) | 8.9 (3.9×) | 11.3 (4.9×) |
| add, constant time | 384 | 3.7 | 14.8 (4.0×) | 12.3 (3.3×) | 16.0 (4.3×) | 18.1 (4.9×) |
| add, constant time | 512 | 4.1 | 23.8 (5.8×) | 18.3 (4.5×) | 25.3 (6.2×) | 31.1 (7.6×) |
| add, constant time | 1024 | 22.0 | 55.1 (2.5×) | 45.7 (2.1×) | 59.6 (2.7×) | 66.9 (3.0×) |
| add, constant time | 2048 | 51.0 | 115 (2.3×) | 105 (2.1×) | 126 (2.5×) | 141 (2.8×) |
| add, constant time | 4096 | 161 | 347 (2.2×) | 247 (1.5×) | 388 (2.4×) | 303 (1.9×) |
| mulmod, constant time | 64 | 93.9 | 114 (1.2×) | 94.5 (1.0×) | 244 (2.6×) | 124 (1.3×) |
| mulmod, constant time | 128 | 206 | 567 (2.8×) | 241 (1.2×) | 596 (2.9×) | 370 (1.8×) |
| mulmod, constant time | 256 | 471 | 1,185 (2.5×) | 741 (1.6×) | 1,302 (2.8×) | 849 (1.8×) |
| mulmod, constant time | 384 | 830 | 1,923 (2.3×) | 1,202 (1.4×) | 2,252 (2.7×) | 1,485 (1.8×) |
| mulmod, constant time | 512 | 1,189 | 3,004 (2.5×) | 1,848 (1.6×) | 3,508 (3.0×) | 2,300 (1.9×) |
| mulmod, constant time | 1024 | 3,878 | 9,532 (2.5×) | 5,612 (1.4×) | 10,977 (2.8×) | 7,579 (2.0×) |
| mulmod, constant time | 2048 | 13,505 | 32,868 (2.4×) | 19,267 (1.4×) | 36,959 (2.7×) | 25,601 (1.9×) |
| mulmod, constant time | 4096 | 48,916 | 120,192 (2.5×) | 68,248 (1.4×) | 132,024 (2.7×) | 92,001 (1.9×) |
| add_mod, constant time | 64 | 8.9 | 12.3 (1.4×) | 13.0 (1.5×) | 19.6 (2.2×) | 14.4 (1.6×) |
| add_mod, constant time | 128 | 12.5 | 28.4 (2.3×) | 22.5 (1.8×) | 37.9 (3.0×) | 34.1 (2.7×) |
| add_mod, constant time | 256 | 18.1 | 83.7 (4.6×) | 66.7 (3.7×) | 98.5 (5.4×) | 91.9 (5.1×) |
| add_mod, constant time | 384 | 36.0 | 112 (3.1×) | 108 (3.0×) | 174 (4.8×) | 127 (3.5×) |
| add_mod, constant time | 512 | 47.8 | 150 (3.1×) | 135 (2.8×) | 243 (5.1×) | 173 (3.6×) |
| add_mod, constant time | 1024 | 131 | 326 (2.5×) | 261 (2.0×) | 620 (4.7×) | 352 (2.7×) |
| add_mod, constant time | 2048 | 308 | 861 (2.8×) | 517 (1.7×) | 1,575 (5.1×) | 757 (2.5×) |
| add_mod, constant time | 4096 | 551 | 1,873 (3.4×) | 1,691 (3.1×) | 1,967 (3.6×) | 2,017 (3.7×) |
| sub_mod, constant time | 64 | 8.9 | 12.1 (1.4×) | 11.0 (1.2×) | 18.2 (2.0×) | 13.8 (1.6×) |
| sub_mod, constant time | 128 | 11.8 | 24.0 (2.0×) | 19.0 (1.6×) | 32.7 (2.8×) | 22.5 (1.9×) |
| sub_mod, constant time | 256 | 17.1 | 54.6 (3.2×) | 36.9 (2.2×) | 76.3 (4.5×) | 59.0 (3.5×) |
| sub_mod, constant time | 384 | 27.5 | 102 (3.7×) | 57.5 (2.1×) | 121 (4.4×) | 85.4 (3.1×) |
| sub_mod, constant time | 512 | 32.2 | 142 (4.4×) | 85.8 (2.7×) | 200 (6.2×) | 117 (3.6×) |
| sub_mod, constant time | 1024 | 94.2 | 309 (3.3×) | 199 (2.1×) | 521 (5.5×) | 254 (2.7×) |
| sub_mod, constant time | 2048 | 284 | 742 (2.6×) | 510 (1.8×) | 1,287 (4.5×) | 612 (2.2×) |
| sub_mod, constant time | 4096 | 506 | 1,542 (3.0×) | 1,423 (2.8×) | 1,735 (3.4×) | 1,691 (3.3×) |
| add_mod, reduced operands, constant time | 64 | 6.9 | 8.1 (1.2×) | 8.0 (1.2×) | 8.0 (1.2×) | 8.4 (1.2×) |
| add_mod, reduced operands, constant time | 128 | 8.8 | 15.0 (1.7×) | 13.1 (1.5×) | 14.6 (1.7×) | 16.4 (1.9×) |
| add_mod, reduced operands, constant time | 256 | 12.9 | 30.8 (2.4×) | 23.0 (1.8×) | 28.8 (2.2×) | 31.2 (2.4×) |
| add_mod, reduced operands, constant time | 384 | 14.6 | 51.5 (3.5×) | 35.0 (2.4×) | 50.2 (3.4×) | 49.7 (3.4×) |
| add_mod, reduced operands, constant time | 512 | 20.3 | 58.4 (2.9×) | 50.4 (2.5×) | 68.6 (3.4×) | 63.8 (3.1×) |
| add_mod, reduced operands, constant time | 1024 | 51.6 | 142 (2.8×) | 91.4 (1.8×) | 304 (5.9×) | 144 (2.8×) |
| add_mod, reduced operands, constant time | 2048 | 140 | 276 (2.0×) | 183 (1.3×) | 689 (4.9×) | 299 (2.1×) |
| add_mod, reduced operands, constant time | 4096 | 301 | 802 (2.7×) | 705 (2.3×) | 860 (2.9×) | 893 (3.0×) |
| sub_mod, reduced operands, constant time | 64 | 3.9 | 4.0 (1.0×) | 5.0 (1.3×) | 4.0 (1.0×) | 5.1 (1.3×) |
| sub_mod, reduced operands, constant time | 128 | 4.8 | 10.6 (2.2×) | 10.0 (2.1×) | 10.0 (2.1×) | 10.1 (2.1×) |
| sub_mod, reduced operands, constant time | 256 | 6.9 | 23.1 (3.3×) | 19.1 (2.8×) | 31.0 (4.5×) | 28.1 (4.1×) |
| sub_mod, reduced operands, constant time | 384 | 9.5 | 41.6 (4.4×) | 32.7 (3.4×) | 50.8 (5.3×) | 43.5 (4.6×) |
| sub_mod, reduced operands, constant time | 512 | 18.0 | 59.4 (3.3×) | 44.4 (2.5×) | 91.5 (5.1×) | 62.9 (3.5×) |
| sub_mod, reduced operands, constant time | 1024 | 44.8 | 129 (2.9×) | 112 (2.5×) | 238 (5.3×) | 141 (3.1×) |
| sub_mod, reduced operands, constant time | 2048 | 121 | 300 (2.5×) | 257 (2.1×) | 572 (4.7×) | 303 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 304 | 831 (2.7×) | 689 (2.3×) | 902 (3.0×) | 849 (2.8×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
