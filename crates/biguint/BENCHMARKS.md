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
| Run 1 | `a0a67b8`, 2026-11-05: 557 cells; CPU 13; 1-minute load 15.83 before, 18.13 after; its SMT sibling, CPU 29, 35% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each native figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands at width N: uniform below 2^N; a divisor of N/2 bits; a modulus of N bits with operands below it.
- Every result matches `num-bigint` before it is measured.
- A row compares the functions of `biguint`'s timing class: fixed for add and for mul up to 1024 bits, variable for the rest.
- A constant-time table compares `ConstantTime<Uint>` with every function whose time the width fixes; divisors and moduli are public.
- add_mod and sub_mod take operands of any value. A `*` there marks a function with an unchecked precondition, operands below the modulus, which gives a wrong result for other inputs: those cells compare against an unchecked precondition.
- The rows of reduced operands take the one modulus of the width. `biguint` runs `Residue` and `ConstantTimeResidue` of a ring that `Modulus::scope` prepares outside the measured loop: the type holds each operand below the modulus, with no check.
- `Residue` keeps a reference to its ring and reads the modulus through it, which makes modulus mixing a compile error. `native` holds the modulus of its 64-bit and 128-bit rows in a register, which leads `Residue -` at 128 bits natively, and the rows of reduced operands at 64 and 128 bits in WebAssembly.
- `biguint` takes the operands of `add` and `mul` by value, and in WebAssembly LLVM loads them into locals ahead of the operation. `crypto-bigint`'s product at 384 bits reads its operands by reference as it goes, and `bnum`'s sum at 1024 bits reads 128-bit words, whose 16 loads LLVM keeps in place.
- With wide arithmetic, `biguint` compiles to the same WebAssembly as `ruint` for `mul` at 256, 384 and 512 bits and as `crypto-bigint` for `add` at 384, 512 and 1024 bits: those figures differ by where the browser places the code.
- Where the browsers favour different forms, `biguint` takes the form with the fewest cycles summed over Chrome and Firefox: without wide arithmetic for reduced `add_mod` at 256 to 512 bits, `add` at 512 bits and reduced constant-time `sub_mod` at 384 bits, and with it for `add` at 384 bits and `mulmod` at 512 bits.
- Without wide arithmetic, `add` at 512 bits runs `ruint`'s code with the operands in the order of the faster native sum, which costs latency in WebAssembly.

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

`biguint` at commit `47a91e3` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-07.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A counted loop runs behind a function pointer, which keeps V8 from inlining it into the code around the count.
- A batch loop reads its step and its end by volatile loads, so LLVM unrolls it for no operation, whatever the operation's size. A chain through locals over operand sets of more than 32 bytes reads its step the same way.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions. They start at a 4 KiB boundary in every case.
- A figure of cycles is the lower rate of the fewest run of n operations and the fewest of 2n, each less the fewest empty count: under load, a run of each length seldom meets an idle sibling hardware thread.
- A case whose 2n runs counted no more than its n runs, whose difference counted under 50 times the noise of the fixed cost of a count, or whose chain step counted under 0.75 cycles ran 31 more repetitions, up to 3 times. A case that stays so stops the sweep.
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
| Cost of a count, in WebAssembly, Chrome | 5,692 to 29,827 cycles and 3,081 instructions; its noise in a figure, 132 to 17,312 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 5,203 to 14,757 cycles and 3,081 instructions; its noise in a figure, 158 to 4,689 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 5,072 to 19,223 cycles and 1,885 instructions; its noise in a figure, 141 to 5,813 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 5,771 to 22,143 cycles and 1,885 instructions; its noise in a figure, 40.0 to 9,677 cycles |
| Run 1, in WebAssembly, Chrome | `5d2b0e1`, 2026-11-07: 557 cells; CPU 12; 1-minute load 37.45 before, 36.77 after; its SMT sibling, CPU 28, 64% busy |
| Run 1, in WebAssembly with wide arithmetic, Chrome | `5d2b0e1`, 2026-11-07: 557 cells; CPU 12; 1-minute load 36.77 before, 24.55 after; its SMT sibling, CPU 28, 28% busy |
| Run 1, in WebAssembly, Firefox | `5d2b0e1`, 2026-11-07: 557 cells; CPU 12; 1-minute load 24.55 before, 24.42 after; its SMT sibling, CPU 28, 41% busy |
| Run 1, in WebAssembly with wide arithmetic, Firefox | `5d2b0e1`, 2026-11-07: 557 cells; CPU 12; 1-minute load 24.42 before, 18.33 after; its SMT sibling, CPU 28, 18% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.5 vs crypto-bigint 2.4 | +0.1 (1.04×) | 2.4 vs native 2.4 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add | 128 | 4.1 vs native 4.0 | +0.1 (1.02×) | 4.1 vs native 4.1 | +0.0 (1.00×) | 3.6 vs native 3.7 | −0.1 (0.97×) | 3.7 vs native 3.7 | +0.0 (1.00×) |
| add | 256 | 9.7 vs bnum 9.7 | +0.0 (1.00×) | 8.6 vs bnum 8.7 | −0.1 (0.99×) | 9.5 vs bnum 9.5 | +0.0 (1.00×) | 9.1 vs bnum 9.1 | +0.0 (1.00×) |
| add | 384 | 16.5 vs ruint 17.9 | −1.4 (0.92×) | 14.0 vs crypto-bigint 14.5 | −0.5 (0.97×) | 16.5 vs bnum 16.1 | +0.4 (1.02×) | 17.6 vs bnum 17.1 | +0.5 (1.03×) |
| add | 512 | 24.1 vs ruint 24.0 | +0.1 (1.00×) | 18.3 vs crypto-bigint 18.1 | +0.2 (1.01×) | 23.3 vs bnum 22.4 | +0.9 (1.04×) | 24.1 vs crypto-bigint 24.2 | −0.1 (1.00×) |
| add | 1024 | 55.8 vs bnum 53.6 | +2.2 (1.04×) | 45.2 vs crypto-bigint 45.3 | −0.1 (1.00×) | 59.4 vs bnum 50.9 | +8.5 (1.17×) | 67.5 vs bnum 58.1 | +9.4 (1.16×) |
| add | 2048 | 120 vs crypto-bigint 192 | −72.0 (0.62×) | 108 vs crypto-bigint 136 | −28.0 (0.79×) | 131 vs bnum 207 | −76.0 (0.63×) | 146 vs crypto-bigint 177 | −31.0 (0.82×) |
| add | 4096 | 250 vs crypto-bigint 394 | −144 (0.63×) | 257 vs crypto-bigint 261 | −4.0 (0.98×) | 320 vs bnum 394 | −74.0 (0.81×) | 318 vs crypto-bigint 325 | −7.0 (0.98×) |
| mul | 64 | 2.6 vs native 2.5 | +0.1 (1.04×) | 2.5 vs native 2.5 | +0.0 (1.00×) | 2.3 vs native 2.3 | +0.0 (1.00×) | 2.3 vs native 2.3 | +0.0 (1.00×) |
| mul | 128 | 10.4 vs uint 11.3 | −0.9 (0.92×) | 4.1 vs bnum 4.1 | +0.0 (1.00×) | 11.5 vs native 14.0 | −2.5 (0.82×) | 5.1 vs uint 4.7 | +0.4 (1.09×) |
| mul | 256 | 46.9 vs ruint 63.2 | −16.3 (0.74×) | 17.1 vs ruint 17.2 | −0.1 (0.99×) | 63.3 vs ruint 79.5 | −16.2 (0.80×) | 24.1 vs ruint 24.1 | +0.0 (1.00×) |
| mul | 384 | 117 vs uint 164 | −47.0 (0.71×) | 42.8 vs crypto-bigint 41.1 | +1.7 (1.04×) | 160 vs ruint 222 | −62.0 (0.72×) | 66.4 vs crypto-bigint 64.1 | +2.3 (1.04×) |
| mul | 512 | 199 vs ruint 334 | −135 (0.60×) | 75.9 vs ruint 75.3 | +0.6 (1.01×) | 329 vs ruint 434 | −105 (0.76×) | 123 vs ruint 123 | +0.0 (1.00×) |
| mul | 1024 | 1,293 vs bnum 1,681 | −388 (0.77×) | 308 vs uint 509 | −201 (0.61×) | 1,510 vs bnum 2,320 | −810 (0.65×) | 680 vs crypto-bigint 907 | −227 (0.75×) |
| mul | 2048 | 4,887 vs bnum 6,289 | −1,402 (0.78×) | 1,706 vs ruint 2,109 | −403 (0.81×) | 5,807 vs bnum 8,656 | −2,849 (0.67×) | 3,065 vs ruint 3,279 | −214 (0.93×) |
| mul | 4096 | 16,901 vs bnum 23,820 | −6,919 (0.71×) | 6,864 vs ruint 8,151 | −1,287 (0.84×) | 20,427 vs bnum 33,312 | −12,885 (0.61×) | 10,783 vs ruint 11,697 | −914 (0.92×) |
| div | 64 | 7.0 vs ruint 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 67.4 vs ruint 92.3 | −24.9 (0.73×) | 31.7 vs ruint 42.7 | −11.0 (0.74×) | 85.6 vs ruint 97.3 | −11.7 (0.88×) | 37.0 vs ruint 54.3 | −17.3 (0.68×) |
| div | 256 | 214 vs ruint 324 | −110 (0.66×) | 86.5 vs ruint 149 | −62.5 (0.58×) | 222 vs ruint 397 | −175 (0.56×) | 104 vs ruint 167 | −63.0 (0.62×) |
| div | 384 | 354 vs ruint 614 | −260 (0.58×) | 203 vs ruint 342 | −139 (0.59×) | 400 vs num-bigint 771 | −371 (0.52×) | 303 vs ruint 428 | −125 (0.71×) |
| div | 512 | 492 vs ruint 847 | −355 (0.58×) | 313 vs ruint 416 | −103 (0.75×) | 577 vs num-bigint 1,007 | −430 (0.57×) | 392 vs ruint 555 | −163 (0.71×) |
| div | 1024 | 1,187 vs num-bigint 2,007 | −820 (0.59×) | 597 vs ruint 882 | −285 (0.68×) | 1,504 vs num-bigint 2,185 | −681 (0.69×) | 844 vs ruint 1,097 | −253 (0.77×) |
| div | 2048 | 3,234 vs num-bigint 6,239 | −3,005 (0.52×) | 1,557 vs ruint 2,378 | −821 (0.65×) | 4,239 vs num-bigint 6,106 | −1,867 (0.69×) | 2,306 vs ruint 2,712 | −406 (0.85×) |
| div | 4096 | 12,989 vs num-bigint 21,374 | −8,385 (0.61×) | 4,990 vs ruint 7,612 | −2,622 (0.66×) | 15,190 vs num-bigint 20,245 | −5,055 (0.75×) | 7,182 vs ruint 9,001 | −1,819 (0.80×) |
| mod | 64 | 7.1 vs native 7.0 | +0.1 (1.01×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 62.0 vs ruint 87.6 | −25.6 (0.71×) | 30.1 vs ruint 41.0 | −10.9 (0.73×) | 76.7 vs ruint 94.6 | −17.9 (0.81×) | 34.1 vs ruint 49.8 | −15.7 (0.68×) |
| mod | 256 | 217 vs ruint 326 | −109 (0.67×) | 73.7 vs ruint 147 | −73.3 (0.50×) | 220 vs ruint 399 | −179 (0.55×) | 99.7 vs ruint 165 | −65.3 (0.60×) |
| mod | 384 | 354 vs ruint 577 | −223 (0.61×) | 195 vs ruint 314 | −119 (0.62×) | 416 vs ruint 712 | −296 (0.58×) | 253 vs ruint 343 | −90.0 (0.74×) |
| mod | 512 | 479 vs ruint 828 | −349 (0.58×) | 293 vs ruint 398 | −105 (0.74×) | 551 vs ruint 1,003 | −452 (0.55×) | 375 vs ruint 457 | −82.0 (0.82×) |
| mod | 1024 | 1,217 vs num-bigint 2,046 | −829 (0.59×) | 585 vs ruint 941 | −356 (0.62×) | 1,504 vs num-bigint 2,184 | −680 (0.69×) | 805 vs ruint 1,036 | −231 (0.78×) |
| mod | 2048 | 3,468 vs num-bigint 6,157 | −2,689 (0.56×) | 1,570 vs ruint 2,290 | −720 (0.69×) | 4,279 vs num-bigint 6,147 | −1,868 (0.70×) | 2,101 vs ruint 2,626 | −525 (0.80×) |
| mod | 4096 | 13,079 vs num-bigint 20,564 | −7,485 (0.64×) | 5,125 vs ruint 7,743 | −2,618 (0.66×) | 15,237 vs num-bigint 20,174 | −4,937 (0.76×) | 6,900 vs ruint 8,911 | −2,011 (0.77×) |
| mulmod | 64 | 68.8 vs ruint 86.1 | −17.3 (0.80×) | 30.7 vs ruint 46.2 | −15.5 (0.66×) | 95.9 vs ruint 178 | −82.1 (0.54×) | 36.0 vs ruint 54.6 | −18.6 (0.66×) |
| mulmod | 128 | 188 vs ruint 319 | −131 (0.59×) | 81.8 vs ruint 123 | −41.2 (0.67×) | 232 vs ruint 381 | −149 (0.61×) | 137 vs ruint 197 | −60.0 (0.70×) |
| mulmod | 256 | 573 vs ruint 801 | −228 (0.72×) | 264 vs ruint 299 | −35.0 (0.88×) | 690 vs ruint 896 | −206 (0.77×) | 362 vs ruint 377 | −15.0 (0.96×) |
| mulmod | 384 | 963 vs ruint 1,596 | −633 (0.60×) | 461 vs ruint 528 | −67.0 (0.87×) | 1,284 vs ruint 1,686 | −402 (0.76×) | 627 vs ruint 684 | −57.0 (0.92×) |
| mulmod | 512 | 1,586 vs ruint 2,815 | −1,229 (0.56×) | 823 vs ruint 848 | −25.0 (0.97×) | 2,027 vs num-bigint 3,093 | −1,066 (0.66×) | 1,125 vs ruint 1,116 | +9.0 (1.01×) |
| mulmod | 1024 | 5,293 vs num-bigint 9,005 | −3,712 (0.59×) | 2,449 vs ruint 2,601 | −152 (0.94×) | 6,642 vs num-bigint 9,361 | −2,719 (0.71×) | 3,134 vs ruint 3,583 | −449 (0.87×) |
| mulmod | 2048 | 17,435 vs num-bigint 32,056 | −14,621 (0.54×) | 7,954 vs ruint 10,659 | −2,705 (0.75×) | 21,831 vs num-bigint 32,181 | −10,350 (0.68×) | 11,252 vs ruint 13,745 | −2,493 (0.82×) |
| mulmod | 4096 | 56,179 vs num-bigint 113,825 | −57,646 (0.49×) | 26,743 vs ruint 41,177 | −14,434 (0.65×) | 67,867 vs num-bigint 115,364 | −47,497 (0.59×) | 38,845 vs ruint 53,304 | −14,459 (0.73×) |
| add_mod | 64 | 4.0 vs ruint 35.2 | −31.2 (0.11×) | 4.0 vs ruint 34.8 | −30.8 (0.11×) | 7.9 vs ruint 70.6 | −62.7 (0.11×) | 7.7 vs ruint 72.4 | −64.7 (0.11×) |
| add_mod | 128 | 7.5 vs ruint 25.1 | −17.6 (0.30×) | 8.3 vs ruint 22.5 | −14.2 (0.37×) | 11.1 vs ruint 23.2 | −12.1 (0.48×) | 12.4 vs ruint 27.4 | −15.0 (0.45×) |
| add_mod | 256 | 22.6 vs ruint 34.9 | −12.3 (0.65×) | 23.1 vs ruint 36.0 | −12.9 (0.64×) | 24.8 vs ruint 44.7 | −19.9 (0.55×) | 29.5 vs ruint 46.7 | −17.2 (0.63×) |
| add_mod | 384 | 32.4 vs ruint 51.9 | −19.5 (0.62×) | 27.7 vs ruint 51.5 | −23.8 (0.54×) | 41.0 vs ruint 74.3 | −33.3 (0.55×) | 42.6 vs ruint 68.1 | −25.5 (0.63×) |
| add_mod | 512 | 44.3 vs ruint 75.5 | −31.2 (0.59×) | 38.1 vs ruint 70.1 | −32.0 (0.54×) | 54.7 vs ruint 104 | −49.3 (0.53×) | 58.6 vs ruint 106 | −47.4 (0.55×) |
| add_mod | 1024 | 113 vs ruint 157 | −44.0 (0.72×) | 103 vs ruint 138 | −35.0 (0.75×) | 133 vs ruint 210 | −77.0 (0.63×) | 128 vs ruint 196 | −68.0 (0.65×) |
| add_mod | 2048 | 245 vs ruint 751 | −506 (0.33×) | 234 vs ruint 709 | −475 (0.33×) | 323 vs ruint 977 | −654 (0.33×) | 282 vs ruint 872 | −590 (0.32×) |
| add_mod | 4096 | 634 vs ruint 1,473 | −839 (0.43×) | 621 vs ruint 1,363 | −742 (0.46×) | 654 vs ruint 1,735 | −1,081 (0.38×) | 766 vs ruint 1,568 | −802 (0.49×) |
| sub_mod | 64 | 3.8 vs ruint 42.5 | −38.7 (0.09×) | 4.0 vs ruint 37.5 | −33.5 (0.11×) | 7.6 vs ruint 74.9 | −67.3 (0.10×) | 7.9 vs ruint 73.3 | −65.4 (0.11×) |
| sub_mod | 128 | 9.5 vs ruint 30.3 | −20.8 (0.31×) | 8.0 vs ruint 29.2 | −21.2 (0.27×) | 10.5 vs ruint 36.5 | −26.0 (0.29×) | 12.0 vs ruint 37.4 | −25.4 (0.32×) |
| sub_mod | 256 | 24.3 vs ruint 51.4 | −27.1 (0.47×) | 20.9 vs ruint 50.6 | −29.7 (0.41×) | 25.3 vs ruint 75.9 | −50.6 (0.33×) | 28.5 vs ruint 76.7 | −48.2 (0.37×) |
| sub_mod | 384 | 31.3 vs ruint 82.9 | −51.6 (0.38×) | 27.0 vs ruint 79.0 | −52.0 (0.34×) | 35.8 vs ruint 122 | −86.2 (0.29×) | 40.9 vs ruint 118 | −77.1 (0.35×) |
| sub_mod | 512 | 43.6 vs ruint 109 | −65.4 (0.40×) | 36.4 vs ruint 103 | −66.6 (0.35×) | 52.9 vs ruint 169 | −116 (0.31×) | 57.6 vs ruint 167 | −109 (0.34×) |
| sub_mod | 1024 | 104 vs ruint 302 | −198 (0.34×) | 106 vs ruint 284 | −178 (0.37×) | 116 vs ruint 542 | −426 (0.21×) | 128 vs ruint 497 | −369 (0.26×) |
| sub_mod | 2048 | 230 vs ruint 1,086 | −856 (0.21×) | 250 vs ruint 1,077 | −827 (0.23×) | 285 vs num-bigint 1,308 | −1,023 (0.22×) | 271 vs ruint 1,281 | −1,010 (0.21×) |
| sub_mod | 4096 | 605 vs ruint 2,076 | −1,471 (0.29×) | 660 vs ruint 1,973 | −1,313 (0.33×) | 627 vs ruint 2,366 | −1,739 (0.27×) | 778 vs ruint 2,308 | −1,530 (0.34×) |
| add_mod, reduced operands | 64 | 4.2 vs native 3.4 | +0.8 (1.24×) | 4.2 vs native 4.1 | +0.1 (1.02×) | 4.1 vs native 3.2 | +0.9 (1.28×) | 4.1 vs native 4.4 | −0.3 (0.93×) |
| add_mod, reduced operands | 128 | 8.6 vs uint 9.1 | −0.5 (0.95×) | 8.8 vs native 9.4 | −0.6 (0.94×) | 7.7 vs uint 9.2 | −1.5 (0.84×) | 10.6 vs native 9.5 | +1.1 (1.12×) |
| add_mod, reduced operands | 256 | 20.3 vs uint 18.9 | +1.4 (1.07×) | 18.1 vs uint 21.1 | −3.0 (0.86×) | 17.0 vs uint 17.8 | −0.8 (0.96×) | 21.3 vs uint 22.0 | −0.7 (0.97×) |
| add_mod, reduced operands | 384 | 29.9 vs uint 32.2 | −2.3 (0.93×) | 25.8 vs uint 31.7 | −5.9 (0.81×) | 31.1 vs uint 30.1 | +1.0 (1.03×) | 31.6 vs uint 34.0 | −2.4 (0.93×) |
| add_mod, reduced operands | 512 | 35.9 vs uint 42.2 | −6.3 (0.85×) | 32.1 vs uint 40.5 | −8.4 (0.79×) | 40.6 vs uint 39.0 | +1.6 (1.04×) | 43.2 vs uint 45.4 | −2.2 (0.95×) |
| add_mod, reduced operands | 1024 | 76.7 vs uint 92.9 | −16.2 (0.83×) | 65.8 vs uint 94.0 | −28.2 (0.70×) | 78.9 vs uint 107 | −28.1 (0.74×) | 87.1 vs uint 116 | −28.9 (0.75×) |
| add_mod, reduced operands | 2048 | 203 vs uint 217 | −14.0 (0.94×) | 149 vs uint 217 | −68.0 (0.69×) | 221 vs uint 254 | −33.0 (0.87×) | 195 vs uint 241 | −46.0 (0.81×) |
| add_mod, reduced operands | 4096 | 427 vs uint 542 | −115 (0.79×) | 458 vs uint 559 | −101 (0.82×) | 506 vs uint 607 | −101 (0.83×) | 520 vs uint 608 | −88.0 (0.86×) |
| sub_mod, reduced operands | 64 | 3.7 vs native 3.1 | +0.6 (1.19×) | 4.2 vs native 3.1 | +1.1 (1.35×) | 4.3 vs native 2.8 | +1.5 (1.54×) | 5.2 vs native 2.8 | +2.4 (1.86×) |
| sub_mod, reduced operands | 128 | 6.5 vs native 7.2 | −0.7 (0.90×) | 7.5 vs native 5.9 | +1.6 (1.27×) | 6.7 vs native 7.3 | −0.6 (0.92×) | 9.5 vs native 6.8 | +2.7 (1.40×) |
| sub_mod, reduced operands | 256 | 18.8 vs uint 19.4 | −0.6 (0.97×) | 14.3 vs bnum 16.9 | −2.6 (0.85×) | 15.1 vs bnum 19.0 | −3.9 (0.79×) | 19.2 vs uint 19.7 | −0.5 (0.97×) |
| sub_mod, reduced operands | 384 | 27.6 vs uint 30.1 | −2.5 (0.92×) | 23.9 vs uint 30.2 | −6.3 (0.79×) | 26.0 vs uint 30.8 | −4.8 (0.84×) | 29.9 vs uint 33.6 | −3.7 (0.89×) |
| sub_mod, reduced operands | 512 | 36.5 vs bnum 47.3 | −10.8 (0.77×) | 31.9 vs bnum 45.1 | −13.2 (0.71×) | 38.7 vs bnum 52.3 | −13.6 (0.74×) | 41.7 vs bnum 53.4 | −11.7 (0.78×) |
| sub_mod, reduced operands | 1024 | 70.1 vs uint 132 | −61.9 (0.53×) | 70.1 vs uint 131 | −60.9 (0.54×) | 77.9 vs uint 167 | −89.1 (0.47×) | 90.2 vs uint 145 | −54.8 (0.62×) |
| sub_mod, reduced operands | 2048 | 202 vs uint 252 | −50.0 (0.80×) | 160 vs uint 259 | −99.0 (0.62×) | 258 vs uint 318 | −60.0 (0.81×) | 202 vs uint 310 | −108 (0.65×) |
| sub_mod, reduced operands | 4096 | 468 vs uint 629 | −161 (0.74×) | 476 vs uint 634 | −158 (0.75×) | 468 vs bnum 670 | −202 (0.70×) | 544 vs uint 691 | −147 (0.79×) |
| add, constant time | 64 | 2.4 vs native 2.4 | +0.0 (1.00×) | 2.4 vs native 2.4 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add, constant time | 128 | 4.1 vs native 4.1 | +0.0 (1.00×) | 4.1 vs native 4.1 | +0.0 (1.00×) | 3.6 vs ruint 3.6 | +0.0 (1.00×) | 3.7 vs native 3.7 | +0.0 (1.00×) |
| add, constant time | 256 | 9.7 vs bnum 9.6 | +0.1 (1.01×) | 8.6 vs bnum 8.7 | −0.1 (0.99×) | 9.4 vs bnum 9.5 | −0.1 (0.99×) | 9.1 vs bnum 9.1 | +0.0 (1.00×) |
| add, constant time | 384 | 16.5 vs ruint 17.9 | −1.4 (0.92×) | 14.1 vs crypto-bigint 14.5 | −0.4 (0.97×) | 16.5 vs bnum 16.1 | +0.4 (1.02×) | 17.6 vs bnum 17.2 | +0.4 (1.02×) |
| add, constant time | 512 | 24.0 vs ruint 23.9 | +0.1 (1.00×) | 18.3 vs crypto-bigint 18.1 | +0.2 (1.01×) | 23.3 vs bnum 22.2 | +1.1 (1.05×) | 24.1 vs crypto-bigint 24.1 | +0.0 (1.00×) |
| add, constant time | 1024 | 55.7 vs bnum 53.6 | +2.1 (1.04×) | 45.4 vs crypto-bigint 45.4 | +0.0 (1.00×) | 59.3 vs bnum 50.8 | +8.5 (1.17×) | 67.5 vs bnum 58.3 | +9.2 (1.16×) |
| add, constant time | 2048 | 118 vs crypto-bigint 192 | −74.0 (0.61×) | 108 vs crypto-bigint 136 | −28.0 (0.79×) | 131 vs bnum 207 | −76.0 (0.63×) | 146 vs crypto-bigint 177 | −31.0 (0.82×) |
| add, constant time | 4096 | 254 vs crypto-bigint 393 | −139 (0.65×) | 257 vs crypto-bigint 260 | −3.0 (0.99×) | 321 vs bnum 395 | −74.0 (0.81×) | 317 vs crypto-bigint 325 | −8.0 (0.98×) |
| mul, constant time | 64 | 2.6 vs native 2.6 | +0.0 (1.00×) | 2.5 vs native 2.5 | +0.0 (1.00×) | 2.3 vs native 2.3 | +0.0 (1.00×) | 2.3 vs native 2.3 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.3 vs ruint 11.6 | −1.3 (0.89×) | 4.1 vs ruint 4.1 | +0.0 (1.00×) | 12.1 vs native 14.0 | −1.9 (0.86×) | 5.0 vs uint 4.7 | +0.3 (1.06×) |
| mul, constant time | 256 | 49.9 vs uint 61.5 | −11.6 (0.81×) | 17.1 vs ruint 17.2 | −0.1 (0.99×) | 62.9 vs ruint 79.4 | −16.5 (0.79×) | 24.2 vs ruint 24.1 | +0.1 (1.00×) |
| mul, constant time | 384 | 122 vs uint 164 | −42.0 (0.74×) | 42.9 vs crypto-bigint 41.1 | +1.8 (1.04×) | 162 vs ruint 222 | −60.0 (0.73×) | 66.1 vs crypto-bigint 64.2 | +1.9 (1.03×) |
| mul, constant time | 512 | 203 vs ruint 336 | −133 (0.60×) | 75.0 vs ruint 75.1 | −0.1 (1.00×) | 328 vs ruint 434 | −106 (0.76×) | 123 vs ruint 124 | −1.0 (0.99×) |
| mul, constant time | 1024 | 1,222 vs bnum 1,603 | −381 (0.76×) | 308 vs uint 510 | −202 (0.60×) | 1,531 vs bnum 2,318 | −787 (0.66×) | 671 vs crypto-bigint 907 | −236 (0.74×) |
| mul, constant time | 2048 | 5,096 vs bnum 6,289 | −1,193 (0.81×) | 1,692 vs bnum 2,553 | −861 (0.66×) | 5,898 vs bnum 8,656 | −2,758 (0.68×) | 3,070 vs bnum 3,383 | −313 (0.91×) |
| mul, constant time | 4096 | 16,851 vs bnum 23,820 | −6,969 (0.71×) | 7,041 vs bnum 9,832 | −2,791 (0.72×) | 20,693 vs bnum 33,312 | −12,619 (0.62×) | 11,038 vs bnum 12,816 | −1,778 (0.86×) |
| div, constant time | 64 | 92.2 vs crypto-bigint 383 | −291 (0.24×) | 48.9 vs crypto-bigint 320 | −271 (0.15×) | 130 vs crypto-bigint 421 | −291 (0.31×) | 61.0 vs crypto-bigint 337 | −276 (0.18×) |
| div, constant time | 128 | 339 vs crypto-bigint 931 | −592 (0.36×) | 169 vs crypto-bigint 734 | −565 (0.23×) | 364 vs crypto-bigint 1,081 | −717 (0.34×) | 236 vs crypto-bigint 900 | −664 (0.26×) |
| div, constant time | 256 | 539 vs crypto-bigint 1,766 | −1,227 (0.31×) | 249 vs crypto-bigint 1,358 | −1,109 (0.18×) | 625 vs crypto-bigint 1,969 | −1,344 (0.32×) | 453 vs crypto-bigint 1,508 | −1,055 (0.30×) |
| div, constant time | 384 | 752 vs crypto-bigint 2,598 | −1,846 (0.29×) | 424 vs crypto-bigint 2,017 | −1,593 (0.21×) | 836 vs crypto-bigint 3,051 | −2,215 (0.27×) | 511 vs crypto-bigint 2,260 | −1,749 (0.23×) |
| div, constant time | 512 | 1,046 vs crypto-bigint 3,749 | −2,703 (0.28×) | 620 vs crypto-bigint 2,758 | −2,138 (0.22×) | 1,190 vs crypto-bigint 4,265 | −3,075 (0.28×) | 774 vs crypto-bigint 3,091 | −2,317 (0.25×) |
| div, constant time | 1024 | 2,484 vs crypto-bigint 8,546 | −6,062 (0.29×) | 1,641 vs crypto-bigint 7,026 | −5,385 (0.23×) | 2,795 vs crypto-bigint 10,618 | −7,823 (0.26×) | 2,093 vs crypto-bigint 7,505 | −5,412 (0.28×) |
| div, constant time | 2048 | 7,586 vs crypto-bigint 22,549 | −14,963 (0.34×) | 4,492 vs crypto-bigint 16,816 | −12,324 (0.27×) | 8,472 vs crypto-bigint 27,490 | −19,018 (0.31×) | 6,059 vs crypto-bigint 18,513 | −12,454 (0.33×) |
| div, constant time | 4096 | 25,655 vs crypto-bigint 67,233 | −41,578 (0.38×) | 13,925 vs crypto-bigint 43,431 | −29,506 (0.32×) | 27,724 vs crypto-bigint 83,998 | −56,274 (0.33×) | 20,131 vs crypto-bigint 50,680 | −30,549 (0.40×) |
| mod, constant time | 64 | 85.1 vs crypto-bigint 399 | −314 (0.21×) | 50.9 vs crypto-bigint 320 | −269 (0.16×) | 159 vs crypto-bigint 417 | −258 (0.38×) | 61.2 vs crypto-bigint 335 | −274 (0.18×) |
| mod, constant time | 128 | 310 vs crypto-bigint 932 | −622 (0.33×) | 161 vs crypto-bigint 737 | −576 (0.22×) | 342 vs crypto-bigint 1,043 | −701 (0.33×) | 221 vs crypto-bigint 864 | −643 (0.26×) |
| mod, constant time | 256 | 528 vs crypto-bigint 1,667 | −1,139 (0.32×) | 317 vs crypto-bigint 1,335 | −1,018 (0.24×) | 584 vs crypto-bigint 1,955 | −1,371 (0.30×) | 402 vs crypto-bigint 1,499 | −1,097 (0.27×) |
| mod, constant time | 384 | 755 vs crypto-bigint 2,569 | −1,814 (0.29×) | 502 vs crypto-bigint 1,965 | −1,463 (0.26×) | 783 vs crypto-bigint 2,995 | −2,212 (0.26×) | 549 vs crypto-bigint 2,225 | −1,676 (0.25×) |
| mod, constant time | 512 | 1,064 vs crypto-bigint 3,509 | −2,445 (0.30×) | 727 vs crypto-bigint 2,748 | −2,021 (0.26×) | 1,110 vs crypto-bigint 4,270 | −3,160 (0.26×) | 771 vs crypto-bigint 3,091 | −2,320 (0.25×) |
| mod, constant time | 1024 | 2,531 vs crypto-bigint 8,449 | −5,918 (0.30×) | 1,680 vs crypto-bigint 6,976 | −5,296 (0.24×) | 2,741 vs crypto-bigint 10,400 | −7,659 (0.26×) | 1,914 vs crypto-bigint 7,454 | −5,540 (0.26×) |
| mod, constant time | 2048 | 7,706 vs crypto-bigint 22,438 | −14,732 (0.34×) | 4,578 vs crypto-bigint 16,759 | −12,181 (0.27×) | 8,342 vs crypto-bigint 27,331 | −18,989 (0.31×) | 5,773 vs crypto-bigint 18,478 | −12,705 (0.31×) |
| mod, constant time | 4096 | 25,982 vs crypto-bigint 67,461 | −41,479 (0.39×) | 14,274 vs crypto-bigint 43,225 | −28,951 (0.33×) | 28,438 vs crypto-bigint 85,248 | −56,810 (0.33×) | 19,678 vs crypto-bigint 50,466 | −30,788 (0.39×) |
| mulmod, constant time | 64 | 178 vs crypto-bigint 470 | −292 (0.38×) | 86.1 vs crypto-bigint 349 | −263 (0.25×) | 225 vs crypto-bigint 524 | −299 (0.43×) | 106 vs crypto-bigint 367 | −261 (0.29×) |
| mulmod, constant time | 128 | 563 vs crypto-bigint 1,275 | −712 (0.44×) | 346 vs crypto-bigint 992 | −646 (0.35×) | 595 vs crypto-bigint 1,519 | −924 (0.39×) | 363 vs crypto-bigint 1,132 | −769 (0.32×) |
| mulmod, constant time | 256 | 1,217 vs crypto-bigint 2,860 | −1,643 (0.43×) | 680 vs crypto-bigint 1,984 | −1,304 (0.34×) | 1,338 vs crypto-bigint 3,266 | −1,928 (0.41×) | 796 vs crypto-bigint 2,274 | −1,478 (0.35×) |
| mulmod, constant time | 384 | 1,944 vs crypto-bigint 4,804 | −2,860 (0.40×) | 1,257 vs crypto-bigint 3,217 | −1,960 (0.39×) | 2,244 vs crypto-bigint 5,627 | −3,383 (0.40×) | 1,421 vs crypto-bigint 3,700 | −2,279 (0.38×) |
| mulmod, constant time | 512 | 3,025 vs crypto-bigint 7,711 | −4,686 (0.39×) | 1,855 vs crypto-bigint 4,908 | −3,053 (0.38×) | 3,553 vs crypto-bigint 9,023 | −5,470 (0.39×) | 2,240 vs crypto-bigint 5,845 | −3,605 (0.38×) |
| mulmod, constant time | 1024 | 9,603 vs crypto-bigint 21,412 | −11,809 (0.45×) | 5,221 vs crypto-bigint 13,143 | −7,922 (0.40×) | 10,782 vs crypto-bigint 25,121 | −14,339 (0.43×) | 7,063 vs crypto-bigint 14,106 | −7,043 (0.50×) |
| mulmod, constant time | 2048 | 32,976 vs crypto-bigint 65,661 | −32,685 (0.50×) | 17,090 vs crypto-bigint 36,341 | −19,251 (0.47×) | 36,417 vs crypto-bigint 76,095 | −39,678 (0.48×) | 24,018 vs crypto-bigint 45,806 | −21,788 (0.52×) |
| mulmod, constant time | 4096 | 119,622 vs crypto-bigint 259,597 | −139,975 (0.46×) | 60,969 vs crypto-bigint 110,384 | −49,415 (0.55×) | 132,064 vs crypto-bigint 255,839 | −123,775 (0.52×) | 86,501 vs crypto-bigint 144,248 | −57,747 (0.60×) |
| add_mod, constant time | 64 | 10.4 | – | 10.4 | – | 20.0 | – | 14.6 | – |
| add_mod, constant time | 128 | 26.7 | – | 22.2 | – | 34.3 | – | 34.1 | – |
| add_mod, constant time | 256 | 78.8 | – | 66.1 | – | 98.8 | – | 70.0 | – |
| add_mod, constant time | 384 | 122 | – | 110 | – | 167 | – | 132 | – |
| add_mod, constant time | 512 | 150 | – | 143 | – | 236 | – | 168 | – |
| add_mod, constant time | 1024 | 323 | – | 267 | – | 594 | – | 337 | – |
| add_mod, constant time | 2048 | 849 | – | 532 | – | 1,699 | – | 732 | – |
| add_mod, constant time | 4096 | 1,857 | – | 1,675 | – | 2,040 | – | 1,988 | – |
| sub_mod, constant time | 64 | 11.2 | – | 10.7 | – | 18.1 | – | 15.1 | – |
| sub_mod, constant time | 128 | 16.0 | – | 13.7 | – | 29.1 | – | 22.1 | – |
| sub_mod, constant time | 256 | 46.3 | – | 32.7 | – | 72.1 | – | 59.8 | – |
| sub_mod, constant time | 384 | 105 | – | 52.2 | – | 135 | – | 88.3 | – |
| sub_mod, constant time | 512 | 144 | – | 89.0 | – | 222 | – | 138 | – |
| sub_mod, constant time | 1024 | 319 | – | 210 | – | 520 | – | 271 | – |
| sub_mod, constant time | 2048 | 730 | – | 512 | – | 1,316 | – | 656 | – |
| sub_mod, constant time | 4096 | 1,547 | – | 1,389 | – | 1,680 | – | 1,634 | – |
| add_mod, reduced operands, constant time | 64 | 6.5 vs crypto-bigint 13.0 | −6.5 (0.50×) | 6.7 vs crypto-bigint 12.7 | −6.0 (0.53×) | 6.3 vs crypto-bigint 12.0 | −5.7 (0.53×) | 6.8 vs crypto-bigint 12.6 | −5.8 (0.54×) |
| add_mod, reduced operands, constant time | 128 | 15.4 vs crypto-bigint 21.8 | −6.4 (0.71×) | 12.9 vs crypto-bigint 22.2 | −9.3 (0.58×) | 13.3 vs crypto-bigint 22.3 | −9.0 (0.60×) | 16.0 vs crypto-bigint 21.7 | −5.7 (0.74×) |
| add_mod, reduced operands, constant time | 256 | 30.7 vs crypto-bigint 51.7 | −21.0 (0.59×) | 25.3 vs crypto-bigint 49.1 | −23.8 (0.52×) | 33.2 vs crypto-bigint 54.8 | −21.6 (0.61×) | 31.1 vs crypto-bigint 56.5 | −25.4 (0.55×) |
| add_mod, reduced operands, constant time | 384 | 47.8 vs crypto-bigint 82.1 | −34.3 (0.58×) | 36.8 vs crypto-bigint 69.6 | −32.8 (0.53×) | 50.0 vs crypto-bigint 87.0 | −37.0 (0.57×) | 45.2 vs crypto-bigint 82.2 | −37.0 (0.55×) |
| add_mod, reduced operands, constant time | 512 | 59.1 vs crypto-bigint 110 | −50.9 (0.54×) | 50.4 vs crypto-bigint 94.8 | −44.4 (0.53×) | 70.9 vs crypto-bigint 141 | −70.1 (0.50×) | 60.7 vs crypto-bigint 126 | −65.3 (0.48×) |
| add_mod, reduced operands, constant time | 1024 | 137 vs crypto-bigint 377 | −240 (0.36×) | 95.6 vs crypto-bigint 347 | −251 (0.28×) | 285 vs crypto-bigint 453 | −168 (0.63×) | 120 vs crypto-bigint 378 | −258 (0.32×) |
| add_mod, reduced operands, constant time | 2048 | 281 vs crypto-bigint 577 | −296 (0.49×) | 182 vs crypto-bigint 508 | −326 (0.36×) | 667 vs crypto-bigint 880 | −213 (0.76×) | 279 vs crypto-bigint 761 | −482 (0.37×) |
| add_mod, reduced operands, constant time | 4096 | 778 vs crypto-bigint 1,194 | −416 (0.65×) | 733 vs crypto-bigint 1,017 | −284 (0.72×) | 805 vs crypto-bigint 1,616 | −811 (0.50×) | 878 vs crypto-bigint 1,395 | −517 (0.63×) |
| sub_mod, reduced operands, constant time | 64 | 4.8 vs crypto-bigint 5.2 | −0.4 (0.92×) | 5.2 vs crypto-bigint 5.1 | +0.1 (1.02×) | 4.5 vs crypto-bigint 4.8 | −0.3 (0.94×) | 4.8 vs crypto-bigint 4.8 | +0.0 (1.00×) |
| sub_mod, reduced operands, constant time | 128 | 9.5 vs crypto-bigint 11.2 | −1.7 (0.85×) | 9.5 vs crypto-bigint 11.3 | −1.8 (0.84×) | 9.2 vs crypto-bigint 11.1 | −1.9 (0.83×) | 11.0 vs crypto-bigint 11.8 | −0.8 (0.93×) |
| sub_mod, reduced operands, constant time | 256 | 26.6 vs crypto-bigint 35.4 | −8.8 (0.75×) | 22.9 vs crypto-bigint 32.5 | −9.6 (0.70×) | 29.5 vs crypto-bigint 30.7 | −1.2 (0.96×) | 27.2 vs crypto-bigint 31.7 | −4.5 (0.86×) |
| sub_mod, reduced operands, constant time | 384 | 41.2 vs crypto-bigint 56.3 | −15.1 (0.73×) | 36.2 vs crypto-bigint 44.7 | −8.5 (0.81×) | 59.2 vs crypto-bigint 56.6 | +2.6 (1.05×) | 44.0 vs crypto-bigint 52.7 | −8.7 (0.83×) |
| sub_mod, reduced operands, constant time | 512 | 62.7 vs crypto-bigint 78.6 | −15.9 (0.80×) | 47.7 vs crypto-bigint 59.5 | −11.8 (0.80×) | 96.1 vs crypto-bigint 96.4 | −0.3 (1.00×) | 62.7 vs crypto-bigint 115 | −52.3 (0.55×) |
| sub_mod, reduced operands, constant time | 1024 | 135 vs crypto-bigint 186 | −51.0 (0.73×) | 115 vs crypto-bigint 144 | −29.0 (0.80×) | 236 vs crypto-bigint 348 | −112 (0.68×) | 136 vs crypto-bigint 195 | −59.0 (0.70×) |
| sub_mod, reduced operands, constant time | 2048 | 298 vs crypto-bigint 535 | −237 (0.56×) | 256 vs crypto-bigint 530 | −274 (0.48×) | 543 vs crypto-bigint 558 | −15.0 (0.97×) | 292 vs crypto-bigint 490 | −198 (0.60×) |
| sub_mod, reduced operands, constant time | 4096 | 761 vs crypto-bigint 926 | −165 (0.82×) | 731 vs crypto-bigint 763 | −32.0 (0.96×) | 840 vs crypto-bigint 1,501 | −661 (0.56×) | 897 vs crypto-bigint 1,390 | −493 (0.65×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.1 vs ruint 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs ruint 1.0 | +0.0 (1.00×) |
| add | 128 | 3.0 vs native 2.8 | +0.2 (1.07×) | 2.4 vs native 2.1 | +0.3 (1.14×) | 3.0 vs crypto-bigint 2.8 | +0.2 (1.07×) | 2.9 vs native 2.5 | +0.4 (1.16×) |
| add | 256 | 8.9 vs bnum 8.9 | +0.0 (1.00×) | 6.6 vs bnum 6.6 | +0.0 (1.00×) | 8.9 vs bnum 8.9 | +0.0 (1.00×) | 7.7 vs bnum 7.7 | +0.0 (1.00×) |
| add | 384 | 15.8 vs ruint 15.8 | +0.0 (1.00×) | 11.2 vs crypto-bigint 10.9 | +0.3 (1.03×) | 16.1 vs ruint 15.7 | +0.4 (1.03×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add | 512 | 22.2 vs ruint 21.9 | +0.3 (1.01×) | 16.2 vs crypto-bigint 16.6 | −0.4 (0.98×) | 21.5 vs ruint 21.2 | +0.3 (1.01×) | 24.6 vs crypto-bigint 25.1 | −0.5 (0.98×) |
| add | 1024 | 50.7 vs bnum 51.1 | −0.4 (0.99×) | 39.7 vs crypto-bigint 39.6 | +0.1 (1.00×) | 52.2 vs bnum 51.3 | +0.9 (1.02×) | 58.0 vs crypto-bigint 58.2 | −0.2 (1.00×) |
| add | 2048 | 115 vs ruint 204 | −89.0 (0.56×) | 94.6 vs ruint 160 | −65.4 (0.59×) | 118 vs ruint 216 | −98.0 (0.55×) | 125 vs ruint 206 | −81.0 (0.61×) |
| add | 4096 | 233 vs ruint 416 | −183 (0.56×) | 219 vs ruint 295 | −76.0 (0.74×) | 237 vs ruint 422 | −185 (0.56×) | 269 vs ruint 380 | −111 (0.71×) |
| mulmod | 64 | 68.5 vs ruint 85.2 | −16.7 (0.80×) | 34.8 vs ruint 42.9 | −8.1 (0.81×) | 122 vs ruint 178 | −56.0 (0.69×) | 42.8 vs ruint 55.0 | −12.2 (0.78×) |
| mulmod | 128 | 209 vs ruint 335 | −126 (0.62×) | 85.0 vs ruint 127 | −42.0 (0.67×) | 258 vs ruint 387 | −129 (0.67×) | 150 vs ruint 198 | −48.0 (0.76×) |
| mulmod | 256 | 588 vs ruint 809 | −221 (0.73×) | 268 vs ruint 303 | −35.0 (0.88×) | 700 vs ruint 890 | −190 (0.79×) | 344 vs ruint 378 | −34.0 (0.91×) |
| mulmod | 384 | 950 vs ruint 1,598 | −648 (0.59×) | 445 vs ruint 534 | −89.0 (0.83×) | 1,260 vs ruint 1,685 | −425 (0.75×) | 647 vs ruint 683 | −36.0 (0.95×) |
| mulmod | 512 | 1,567 vs ruint 2,809 | −1,242 (0.56×) | 776 vs ruint 855 | −79.0 (0.91×) | 2,114 vs ruint 3,257 | −1,143 (0.65×) | 1,087 vs ruint 1,113 | −26.0 (0.98×) |
| mulmod | 1024 | 5,307 vs ruint 10,760 | −5,453 (0.49×) | 2,413 vs ruint 2,600 | −187 (0.93×) | 6,731 vs ruint 12,516 | −5,785 (0.54×) | 3,168 vs ruint 3,576 | −408 (0.89×) |
| mulmod | 2048 | 17,524 vs ruint 34,518 | −16,994 (0.51×) | 8,004 vs ruint 10,667 | −2,663 (0.75×) | 21,812 vs ruint 43,637 | −21,825 (0.50×) | 11,326 vs ruint 13,737 | −2,411 (0.82×) |
| mulmod | 4096 | 56,469 vs ruint 161,980 | −105,511 (0.35×) | 26,734 vs ruint 41,179 | −14,445 (0.65×) | 67,924 vs ruint 166,717 | −98,793 (0.41×) | 38,730 vs ruint 53,293 | −14,563 (0.73×) |
| add_mod | 64 | 6.2 vs ruint 33.7 | −27.5 (0.18×) | 6.1 vs ruint 42.6 | −36.5 (0.14×) | 8.2 vs ruint 70.6 | −62.4 (0.12×) | 8.2 vs ruint 71.4 | −63.2 (0.11×) |
| add_mod | 128 | 31.3 vs ruint 41.5 | −10.2 (0.75×) | 10.6 vs ruint 30.3 | −19.7 (0.35×) | 41.1 vs ruint 49.6 | −8.5 (0.83×) | 16.0 vs ruint 37.7 | −21.7 (0.42×) |
| add_mod | 256 | 40.6 vs ruint 72.9 | −32.3 (0.56×) | 30.8 vs ruint 60.1 | −29.3 (0.51×) | 54.3 vs ruint 87.8 | −33.5 (0.62×) | 43.4 vs ruint 71.4 | −28.0 (0.61×) |
| add_mod | 384 | 54.7 vs ruint 98.9 | −44.2 (0.55×) | 41.3 vs ruint 76.6 | −35.3 (0.54×) | 78.2 vs ruint 124 | −45.8 (0.63×) | 63.6 vs ruint 99.3 | −35.7 (0.64×) |
| add_mod | 512 | 67.3 vs ruint 131 | −63.7 (0.51×) | 51.4 vs ruint 99.3 | −47.9 (0.52×) | 90.9 vs ruint 160 | −69.1 (0.57×) | 81.4 vs ruint 139 | −57.6 (0.59×) |
| add_mod | 1024 | 128 vs ruint 266 | −138 (0.48×) | 118 vs ruint 204 | −86.0 (0.58×) | 158 vs ruint 350 | −192 (0.45×) | 160 vs ruint 298 | −138 (0.54×) |
| add_mod | 2048 | 289 vs ruint 864 | −575 (0.33×) | 228 vs ruint 769 | −541 (0.30×) | 339 vs ruint 1,094 | −755 (0.31×) | 302 vs ruint 907 | −605 (0.33×) |
| add_mod | 4096 | 659 vs ruint 1,580 | −921 (0.42×) | 699 vs ruint 1,401 | −702 (0.50×) | 834 vs ruint 1,858 | −1,024 (0.45×) | 869 vs ruint 1,638 | −769 (0.53×) |
| sub_mod | 64 | 4.4 vs ruint 38.2 | −33.8 (0.12×) | 7.3 vs ruint 36.3 | −29.0 (0.20×) | 7.9 vs ruint 72.0 | −64.1 (0.11×) | 8.1 vs ruint 71.9 | −63.8 (0.11×) |
| sub_mod | 128 | 32.1 vs ruint 47.4 | −15.3 (0.68×) | 9.2 vs ruint 42.9 | −33.7 (0.21×) | 40.2 vs ruint 63.3 | −23.1 (0.64×) | 15.3 vs ruint 52.7 | −37.4 (0.29×) |
| sub_mod | 256 | 45.3 vs ruint 97.6 | −52.3 (0.46×) | 30.5 vs ruint 75.6 | −45.1 (0.40×) | 55.3 vs ruint 129 | −73.7 (0.43×) | 43.0 vs ruint 109 | −66.0 (0.39×) |
| sub_mod | 384 | 53.9 vs ruint 127 | −73.1 (0.42×) | 41.3 vs ruint 104 | −62.7 (0.40×) | 70.9 vs ruint 178 | −107 (0.40×) | 60.1 vs ruint 145 | −84.9 (0.41×) |
| sub_mod | 512 | 70.4 vs ruint 166 | −95.6 (0.42×) | 51.9 vs ruint 134 | −82.1 (0.39×) | 88.7 vs ruint 231 | −142 (0.38×) | 80.4 vs ruint 202 | −122 (0.40×) |
| sub_mod | 1024 | 135 vs ruint 425 | −290 (0.32×) | 115 vs ruint 354 | −239 (0.32×) | 160 vs ruint 758 | −598 (0.21×) | 166 vs ruint 633 | −467 (0.26×) |
| sub_mod | 2048 | 280 vs ruint 1,192 | −912 (0.23×) | 249 vs ruint 1,119 | −870 (0.22×) | 372 vs ruint 1,435 | −1,063 (0.26×) | 331 vs ruint 1,290 | −959 (0.26×) |
| sub_mod | 4096 | 778 vs ruint 2,188 | −1,410 (0.36×) | 731 vs ruint 2,033 | −1,302 (0.36×) | 881 vs ruint 2,469 | −1,588 (0.36×) | 903 vs ruint 2,338 | −1,435 (0.39×) |
| add_mod, reduced operands | 64 | 5.0 vs ruint 36.4 | −31.4 (0.14×) | 5.0 vs ruint 43.1 | −38.1 (0.12×) | 5.0 vs ruint 77.8 | −72.8 (0.06×) | 5.0 vs ruint 78.6 | −73.6 (0.06×) |
| add_mod, reduced operands | 128 | 6.7 vs ruint 24.7 | −18.0 (0.27×) | 6.8 vs ruint 23.1 | −16.3 (0.29×) | 8.1 vs ruint 24.2 | −16.1 (0.33×) | 10.3 vs ruint 25.1 | −14.8 (0.41×) |
| add_mod, reduced operands | 256 | 16.6 vs ruint 41.9 | −25.3 (0.40×) | 13.5 vs ruint 42.7 | −29.2 (0.32×) | 16.7 vs ruint 55.1 | −38.4 (0.30×) | 20.3 vs ruint 55.5 | −35.2 (0.37×) |
| add_mod, reduced operands | 384 | 27.3 vs ruint 61.6 | −34.3 (0.44×) | 19.7 vs ruint 59.9 | −40.2 (0.33×) | 27.6 vs ruint 87.9 | −60.3 (0.31×) | 32.1 vs ruint 79.0 | −46.9 (0.41×) |
| add_mod, reduced operands | 512 | 34.6 vs ruint 85.1 | −50.5 (0.41×) | 26.6 vs ruint 79.2 | −52.6 (0.34×) | 39.6 vs ruint 115 | −75.4 (0.34×) | 42.9 vs ruint 115 | −72.1 (0.37×) |
| add_mod, reduced operands | 1024 | 75.5 vs ruint 189 | −114 (0.40×) | 63.0 vs ruint 172 | −109 (0.37×) | 76.5 vs ruint 265 | −189 (0.29×) | 88.5 vs ruint 248 | −160 (0.36×) |
| add_mod, reduced operands | 2048 | 198 vs ruint 774 | −576 (0.26×) | 144 vs ruint 734 | −590 (0.20×) | 230 vs ruint 1,009 | −779 (0.23×) | 200 vs ruint 892 | −692 (0.22×) |
| add_mod, reduced operands | 4096 | 414 vs ruint 1,487 | −1,073 (0.28×) | 419 vs ruint 1,387 | −968 (0.30×) | 497 vs ruint 1,740 | −1,243 (0.29×) | 538 vs ruint 1,603 | −1,065 (0.34×) |
| sub_mod, reduced operands | 64 | 2.3 vs ruint 36.9 | −34.6 (0.06×) | 2.7 vs ruint 36.6 | −33.9 (0.07×) | 3.2 vs ruint 78.5 | −75.3 (0.04×) | 3.7 vs ruint 77.4 | −73.7 (0.05×) |
| sub_mod, reduced operands | 128 | 5.8 vs ruint 26.1 | −20.3 (0.22×) | 5.6 vs ruint 23.7 | −18.1 (0.24×) | 6.2 vs ruint 26.1 | −19.9 (0.24×) | 8.8 vs ruint 26.1 | −17.3 (0.34×) |
| sub_mod, reduced operands | 256 | 15.2 vs ruint 49.4 | −34.2 (0.31×) | 12.4 vs ruint 50.3 | −37.9 (0.25×) | 13.3 vs ruint 62.4 | −49.1 (0.21×) | 18.8 vs ruint 63.4 | −44.6 (0.30×) |
| sub_mod, reduced operands | 384 | 24.9 vs ruint 78.7 | −53.8 (0.32×) | 19.1 vs ruint 75.9 | −56.8 (0.25×) | 25.3 vs ruint 97.3 | −72.0 (0.26×) | 29.6 vs ruint 90.7 | −61.1 (0.33×) |
| sub_mod, reduced operands | 512 | 34.5 vs ruint 110 | −75.5 (0.31×) | 27.4 vs ruint 102 | −74.6 (0.27×) | 40.6 vs ruint 132 | −91.4 (0.31×) | 41.1 vs ruint 133 | −91.9 (0.31×) |
| sub_mod, reduced operands | 1024 | 70.0 vs ruint 254 | −184 (0.28×) | 68.7 vs ruint 240 | −171 (0.29×) | 78.8 vs ruint 369 | −290 (0.21×) | 90.8 vs ruint 304 | −213 (0.30×) |
| sub_mod, reduced operands | 2048 | 204 vs ruint 977 | −773 (0.21×) | 164 vs ruint 1,017 | −853 (0.16×) | 287 vs ruint 1,206 | −919 (0.24×) | 204 vs ruint 1,092 | −888 (0.19×) |
| sub_mod, reduced operands | 4096 | 432 vs ruint 1,857 | −1,425 (0.23×) | 441 vs ruint 1,735 | −1,294 (0.25×) | 471 vs ruint 2,106 | −1,635 (0.22×) | 555 vs ruint 2,026 | −1,471 (0.27×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs ruint 1.0 | +0.0 (1.00×) |
| add, constant time | 128 | 3.0 vs bnum 2.8 | +0.2 (1.07×) | 2.4 vs native 2.1 | +0.3 (1.14×) | 2.9 vs crypto-bigint 2.8 | +0.1 (1.04×) | 2.9 vs native 2.5 | +0.4 (1.16×) |
| add, constant time | 256 | 8.9 vs bnum 9.0 | −0.1 (0.99×) | 6.6 vs bnum 6.6 | +0.0 (1.00×) | 8.8 vs bnum 8.9 | −0.1 (0.99×) | 7.7 vs bnum 7.8 | −0.1 (0.99×) |
| add, constant time | 384 | 15.6 vs ruint 15.9 | −0.3 (0.98×) | 11.2 vs crypto-bigint 10.9 | +0.3 (1.03×) | 16.1 vs ruint 15.7 | +0.4 (1.03×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add, constant time | 512 | 22.4 vs ruint 21.7 | +0.7 (1.03×) | 16.7 vs crypto-bigint 16.8 | −0.1 (0.99×) | 21.6 vs ruint 21.0 | +0.6 (1.03×) | 24.6 vs crypto-bigint 25.1 | −0.5 (0.98×) |
| add, constant time | 1024 | 50.6 vs bnum 51.2 | −0.6 (0.99×) | 39.7 vs crypto-bigint 39.6 | +0.1 (1.00×) | 52.1 vs bnum 51.3 | +0.8 (1.02×) | 58.0 vs crypto-bigint 58.1 | −0.1 (1.00×) |
| add, constant time | 2048 | 114 vs ruint 204 | −90.0 (0.56×) | 94.6 vs ruint 159 | −64.4 (0.59×) | 118 vs ruint 216 | −98.0 (0.55×) | 125 vs ruint 206 | −81.0 (0.61×) |
| add, constant time | 4096 | 234 vs ruint 417 | −183 (0.56×) | 219 vs ruint 296 | −77.0 (0.74×) | 237 vs ruint 422 | −185 (0.56×) | 269 vs ruint 382 | −113 (0.70×) |
| mulmod, constant time | 64 | 159 | – | 93.1 | – | 242 | – | 115 | – |
| mulmod, constant time | 128 | 567 | – | 312 | – | 606 | – | 362 | – |
| mulmod, constant time | 256 | 1,183 | – | 696 | – | 1,303 | – | 819 | – |
| mulmod, constant time | 384 | 1,921 | – | 1,175 | – | 2,268 | – | 1,428 | – |
| mulmod, constant time | 512 | 3,013 | – | 1,736 | – | 3,451 | – | 2,222 | – |
| mulmod, constant time | 1024 | 9,583 | – | 5,196 | – | 10,929 | – | 7,083 | – |
| mulmod, constant time | 2048 | 33,040 | – | 17,147 | – | 36,976 | – | 24,135 | – |
| mulmod, constant time | 4096 | 120,322 | – | 59,507 | – | 131,241 | – | 85,773 | – |
| add_mod, constant time | 64 | 12.1 | – | 13.0 | – | 19.7 | – | 13.1 | – |
| add_mod, constant time | 128 | 27.2 | – | 23.3 | – | 35.0 | – | 34.1 | – |
| add_mod, constant time | 256 | 82.8 | – | 68.5 | – | 102 | – | 78.2 | – |
| add_mod, constant time | 384 | 112 | – | 108 | – | 171 | – | 127 | – |
| add_mod, constant time | 512 | 146 | – | 136 | – | 233 | – | 166 | – |
| add_mod, constant time | 1024 | 328 | – | 263 | – | 629 | – | 353 | – |
| add_mod, constant time | 2048 | 874 | – | 513 | – | 1,594 | – | 751 | – |
| add_mod, constant time | 4096 | 1,859 | – | 1,691 | – | 2,003 | – | 1,958 | – |
| sub_mod, constant time | 64 | 12.2 | – | 11.0 | – | 19.2 | – | 13.8 | – |
| sub_mod, constant time | 128 | 24.8 | – | 20.0 | – | 31.7 | – | 22.0 | – |
| sub_mod, constant time | 256 | 52.1 | – | 36.1 | – | 75.3 | – | 58.5 | – |
| sub_mod, constant time | 384 | 103 | – | 57.6 | – | 125 | – | 87.4 | – |
| sub_mod, constant time | 512 | 141 | – | 86.1 | – | 196 | – | 119 | – |
| sub_mod, constant time | 1024 | 313 | – | 201 | – | 533 | – | 256 | – |
| sub_mod, constant time | 2048 | 739 | – | 512 | – | 1,277 | – | 573 | – |
| sub_mod, constant time | 4096 | 1,537 | – | 1,391 | – | 1,680 | – | 1,614 | – |
| add_mod, reduced operands, constant time | 64 | 8.3 | – | 8.0 | – | 8.0 | – | 8.5 | – |
| add_mod, reduced operands, constant time | 128 | 15.9 | – | 14.0 | – | 14.7 | – | 15.5 | – |
| add_mod, reduced operands, constant time | 256 | 31.7 | – | 24.0 | – | 29.1 | – | 31.1 | – |
| add_mod, reduced operands, constant time | 384 | 46.1 | – | 32.8 | – | 48.4 | – | 46.7 | – |
| add_mod, reduced operands, constant time | 512 | 58.1 | – | 43.8 | – | 71.8 | – | 62.3 | – |
| add_mod, reduced operands, constant time | 1024 | 142 | – | 93.7 | – | 298 | – | 122 | – |
| add_mod, reduced operands, constant time | 2048 | 278 | – | 184 | – | 678 | – | 293 | – |
| add_mod, reduced operands, constant time | 4096 | 720 | – | 712 | – | 814 | – | 823 | – |
| sub_mod, reduced operands, constant time | 64 | 4.0 | – | 5.0 | – | 4.0 | – | 5.1 | – |
| sub_mod, reduced operands, constant time | 128 | 11.1 | – | 9.4 | – | 10.1 | – | 10.3 | – |
| sub_mod, reduced operands, constant time | 256 | 24.4 | – | 19.9 | – | 28.3 | – | 27.1 | – |
| sub_mod, reduced operands, constant time | 384 | 39.3 | – | 29.9 | – | 45.6 | – | 44.7 | – |
| sub_mod, reduced operands, constant time | 512 | 59.4 | – | 44.9 | – | 87.4 | – | 63.1 | – |
| sub_mod, reduced operands, constant time | 1024 | 132 | – | 110 | – | 224 | – | 140 | – |
| sub_mod, reduced operands, constant time | 2048 | 294 | – | 257 | – | 555 | – | 306 | – |
| sub_mod, reduced operands, constant time | 4096 | 774 | – | 698 | – | 871 | – | 875 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.4 | 2.5 (1.8×) | 2.4 (1.7×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add | 128 | 2.4 | 4.1 (1.7×) | 4.1 (1.7×) | 3.6 (1.5×) | 3.7 (1.5×) |
| add | 256 | 4.7 | 9.7 (2.1×) | 8.6 (1.8×) | 9.5 (2.0×) | 9.1 (1.9×) |
| add | 384 | 7.1 | 16.5 (2.3×) | 14.0 (2.0×) | 16.5 (2.3×) | 17.6 (2.5×) |
| add | 512 | 9.2 | 24.1 (2.6×) | 18.3 (2.0×) | 23.3 (2.5×) | 24.1 (2.6×) |
| add | 1024 | 23.3 | 55.8 (2.4×) | 45.2 (1.9×) | 59.4 (2.5×) | 67.5 (2.9×) |
| add | 2048 | 53.5 | 120 (2.2×) | 108 (2.0×) | 131 (2.4×) | 146 (2.7×) |
| add | 4096 | 158 | 250 (1.6×) | 257 (1.6×) | 320 (2.0×) | 318 (2.0×) |
| mul | 64 | 1.5 | 2.6 (1.7×) | 2.5 (1.7×) | 2.3 (1.5×) | 2.3 (1.5×) |
| mul | 128 | 3.0 | 10.4 (3.5×) | 4.1 (1.4×) | 11.5 (3.8×) | 5.1 (1.7×) |
| mul | 256 | 12.2 | 46.9 (3.8×) | 17.1 (1.4×) | 63.3 (5.2×) | 24.1 (2.0×) |
| mul | 384 | 27.7 | 117 (4.2×) | 42.8 (1.5×) | 160 (5.8×) | 66.4 (2.4×) |
| mul | 512 | 52.1 | 199 (3.8×) | 75.9 (1.5×) | 329 (6.3×) | 123 (2.4×) |
| mul | 1024 | 221 | 1,293 (5.9×) | 308 (1.4×) | 1,510 (6.8×) | 680 (3.1×) |
| mul | 2048 | 1,113 | 4,887 (4.4×) | 1,706 (1.5×) | 5,807 (5.2×) | 3,065 (2.8×) |
| mul | 4096 | 3,859 | 16,901 (4.4×) | 6,864 (1.8×) | 20,427 (5.3×) | 10,783 (2.8×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.3 | 67.4 (4.1×) | 31.7 (1.9×) | 85.6 (5.3×) | 37.0 (2.3×) |
| div | 256 | 46.0 | 214 (4.7×) | 86.5 (1.9×) | 222 (4.8×) | 104 (2.3×) |
| div | 384 | 111 | 354 (3.2×) | 203 (1.8×) | 400 (3.6×) | 303 (2.7×) |
| div | 512 | 155 | 492 (3.2×) | 313 (2.0×) | 577 (3.7×) | 392 (2.5×) |
| div | 1024 | 462 | 1,187 (2.6×) | 597 (1.3×) | 1,504 (3.3×) | 844 (1.8×) |
| div | 2048 | 1,388 | 3,234 (2.3×) | 1,557 (1.1×) | 4,239 (3.1×) | 2,306 (1.7×) |
| div | 4096 | 4,594 | 12,989 (2.8×) | 4,990 (1.1×) | 15,190 (3.3×) | 7,182 (1.6×) |
| mod | 64 | 6.9 | 7.1 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 17.0 | 62.0 (3.6×) | 30.1 (1.8×) | 76.7 (4.5×) | 34.1 (2.0×) |
| mod | 256 | 41.5 | 217 (5.2×) | 73.7 (1.8×) | 220 (5.3×) | 99.7 (2.4×) |
| mod | 384 | 119 | 354 (3.0×) | 195 (1.6×) | 416 (3.5×) | 253 (2.1×) |
| mod | 512 | 158 | 479 (3.0×) | 293 (1.9×) | 551 (3.5×) | 375 (2.4×) |
| mod | 1024 | 485 | 1,217 (2.5×) | 585 (1.2×) | 1,504 (3.1×) | 805 (1.7×) |
| mod | 2048 | 1,407 | 3,468 (2.5×) | 1,570 (1.1×) | 4,279 (3.0×) | 2,101 (1.5×) |
| mod | 4096 | 4,562 | 13,079 (2.9×) | 5,125 (1.1×) | 15,237 (3.3×) | 6,900 (1.5×) |
| mulmod | 64 | 11.9 | 68.8 (5.8×) | 30.7 (2.6×) | 95.9 (8.1×) | 36.0 (3.0×) |
| mulmod | 128 | 47.4 | 188 (4.0×) | 81.8 (1.7×) | 232 (4.9×) | 137 (2.9×) |
| mulmod | 256 | 162 | 573 (3.5×) | 264 (1.6×) | 690 (4.3×) | 362 (2.2×) |
| mulmod | 384 | 297 | 963 (3.2×) | 461 (1.6×) | 1,284 (4.3×) | 627 (2.1×) |
| mulmod | 512 | 467 | 1,586 (3.4×) | 823 (1.8×) | 2,027 (4.3×) | 1,125 (2.4×) |
| mulmod | 1024 | 1,365 | 5,293 (3.9×) | 2,449 (1.8×) | 6,642 (4.9×) | 3,134 (2.3×) |
| mulmod | 2048 | 5,191 | 17,435 (3.4×) | 7,954 (1.5×) | 21,831 (4.2×) | 11,252 (2.2×) |
| mulmod | 4096 | 14,910 | 56,179 (3.8×) | 26,743 (1.8×) | 67,867 (4.6×) | 38,845 (2.6×) |
| add_mod | 64 | 2.9 | 4.0 (1.4×) | 4.0 (1.4×) | 7.9 (2.7×) | 7.7 (2.7×) |
| add_mod | 128 | 4.8 | 7.5 (1.6×) | 8.3 (1.7×) | 11.1 (2.3×) | 12.4 (2.6×) |
| add_mod | 256 | 9.7 | 22.6 (2.3×) | 23.1 (2.4×) | 24.8 (2.6×) | 29.5 (3.0×) |
| add_mod | 384 | 15.6 | 32.4 (2.1×) | 27.7 (1.8×) | 41.0 (2.6×) | 42.6 (2.7×) |
| add_mod | 512 | 27.7 | 44.3 (1.6×) | 38.1 (1.4×) | 54.7 (2.0×) | 58.6 (2.1×) |
| add_mod | 1024 | 71.7 | 113 (1.6×) | 103 (1.4×) | 133 (1.9×) | 128 (1.8×) |
| add_mod | 2048 | 149 | 245 (1.6×) | 234 (1.6×) | 323 (2.2×) | 282 (1.9×) |
| add_mod | 4096 | 268 | 634 (2.4×) | 621 (2.3×) | 654 (2.4×) | 766 (2.9×) |
| sub_mod | 64 | 2.4 | 3.8 (1.6×) | 4.0 (1.7×) | 7.6 (3.2×) | 7.9 (3.3×) |
| sub_mod | 128 | 4.3 | 9.5 (2.2×) | 8.0 (1.9×) | 10.5 (2.4×) | 12.0 (2.8×) |
| sub_mod | 256 | 8.7 | 24.3 (2.8×) | 20.9 (2.4×) | 25.3 (2.9×) | 28.5 (3.3×) |
| sub_mod | 384 | 15.6 | 31.3 (2.0×) | 27.0 (1.7×) | 35.8 (2.3×) | 40.9 (2.6×) |
| sub_mod | 512 | 25.2 | 43.6 (1.7×) | 36.4 (1.4×) | 52.9 (2.1×) | 57.6 (2.3×) |
| sub_mod | 1024 | 66.3 | 104 (1.6×) | 106 (1.6×) | 116 (1.7×) | 128 (1.9×) |
| sub_mod | 2048 | 146 | 230 (1.6×) | 250 (1.7×) | 285 (2.0×) | 271 (1.9×) |
| sub_mod | 4096 | 241 | 605 (2.5×) | 660 (2.7×) | 627 (2.6×) | 778 (3.2×) |
| add_mod, reduced operands | 64 | 2.8 | 4.2 (1.5×) | 4.2 (1.5×) | 4.1 (1.5×) | 4.1 (1.5×) |
| add_mod, reduced operands | 128 | 4.3 | 8.6 (2.0×) | 8.8 (2.0×) | 7.7 (1.8×) | 10.6 (2.5×) |
| add_mod, reduced operands | 256 | 10.4 | 20.3 (2.0×) | 18.1 (1.7×) | 17.0 (1.6×) | 21.3 (2.0×) |
| add_mod, reduced operands | 384 | 13.8 | 29.9 (2.2×) | 25.8 (1.9×) | 31.1 (2.3×) | 31.6 (2.3×) |
| add_mod, reduced operands | 512 | 19.6 | 35.9 (1.8×) | 32.1 (1.6×) | 40.6 (2.1×) | 43.2 (2.2×) |
| add_mod, reduced operands | 1024 | 48.3 | 76.7 (1.6×) | 65.8 (1.4×) | 78.9 (1.6×) | 87.1 (1.8×) |
| add_mod, reduced operands | 2048 | 89.7 | 203 (2.3×) | 149 (1.7×) | 221 (2.5×) | 195 (2.2×) |
| add_mod, reduced operands | 4096 | 227 | 427 (1.9×) | 458 (2.0×) | 506 (2.2×) | 520 (2.3×) |
| sub_mod, reduced operands | 64 | 2.2 | 3.7 (1.7×) | 4.2 (1.9×) | 4.3 (2.0×) | 5.2 (2.4×) |
| sub_mod, reduced operands | 128 | 4.7 | 6.5 (1.4×) | 7.5 (1.6×) | 6.7 (1.4×) | 9.5 (2.0×) |
| sub_mod, reduced operands | 256 | 7.3 | 18.8 (2.6×) | 14.3 (2.0×) | 15.1 (2.1×) | 19.2 (2.6×) |
| sub_mod, reduced operands | 384 | 11.0 | 27.6 (2.5×) | 23.9 (2.2×) | 26.0 (2.4×) | 29.9 (2.7×) |
| sub_mod, reduced operands | 512 | 18.3 | 36.5 (2.0×) | 31.9 (1.7×) | 38.7 (2.1×) | 41.7 (2.3×) |
| sub_mod, reduced operands | 1024 | 44.7 | 70.1 (1.6×) | 70.1 (1.6×) | 77.9 (1.7×) | 90.2 (2.0×) |
| sub_mod, reduced operands | 2048 | 83.2 | 202 (2.4×) | 160 (1.9×) | 258 (3.1×) | 202 (2.4×) |
| sub_mod, reduced operands | 4096 | 217 | 468 (2.2×) | 476 (2.2×) | 468 (2.2×) | 544 (2.5×) |
| add, constant time | 64 | 1.4 | 2.4 (1.7×) | 2.4 (1.7×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 4.1 (1.7×) | 4.1 (1.7×) | 3.6 (1.5×) | 3.7 (1.5×) |
| add, constant time | 256 | 4.5 | 9.7 (2.2×) | 8.6 (1.9×) | 9.4 (2.1×) | 9.1 (2.0×) |
| add, constant time | 384 | 7.0 | 16.5 (2.4×) | 14.1 (2.0×) | 16.5 (2.4×) | 17.6 (2.5×) |
| add, constant time | 512 | 9.5 | 24.0 (2.5×) | 18.3 (1.9×) | 23.3 (2.5×) | 24.1 (2.5×) |
| add, constant time | 1024 | 23.4 | 55.7 (2.4×) | 45.4 (1.9×) | 59.3 (2.5×) | 67.5 (2.9×) |
| add, constant time | 2048 | 53.3 | 118 (2.2×) | 108 (2.0×) | 131 (2.5×) | 146 (2.7×) |
| add, constant time | 4096 | 162 | 254 (1.6×) | 257 (1.6×) | 321 (2.0×) | 317 (2.0×) |
| mul, constant time | 64 | 1.5 | 2.6 (1.7×) | 2.5 (1.7×) | 2.3 (1.5×) | 2.3 (1.5×) |
| mul, constant time | 128 | 3.0 | 10.3 (3.4×) | 4.1 (1.4×) | 12.1 (4.0×) | 5.0 (1.7×) |
| mul, constant time | 256 | 12.4 | 49.9 (4.0×) | 17.1 (1.4×) | 62.9 (5.1×) | 24.2 (2.0×) |
| mul, constant time | 384 | 30.0 | 122 (4.1×) | 42.9 (1.4×) | 162 (5.4×) | 66.1 (2.2×) |
| mul, constant time | 512 | 50.9 | 203 (4.0×) | 75.0 (1.5×) | 328 (6.4×) | 123 (2.4×) |
| mul, constant time | 1024 | 226 | 1,222 (5.4×) | 308 (1.4×) | 1,531 (6.8×) | 671 (3.0×) |
| mul, constant time | 2048 | 1,154 | 5,096 (4.4×) | 1,692 (1.5×) | 5,898 (5.1×) | 3,070 (2.7×) |
| mul, constant time | 4096 | 4,213 | 16,851 (4.0×) | 7,041 (1.7×) | 20,693 (4.9×) | 11,038 (2.6×) |
| div, constant time | 64 | 25.1 | 92.2 (3.7×) | 48.9 (1.9×) | 130 (5.2×) | 61.0 (2.4×) |
| div, constant time | 128 | 98.8 | 339 (3.4×) | 169 (1.7×) | 364 (3.7×) | 236 (2.4×) |
| div, constant time | 256 | 199 | 539 (2.7×) | 249 (1.3×) | 625 (3.1×) | 453 (2.3×) |
| div, constant time | 384 | 291 | 752 (2.6×) | 424 (1.5×) | 836 (2.9×) | 511 (1.8×) |
| div, constant time | 512 | 461 | 1,046 (2.3×) | 620 (1.3×) | 1,190 (2.6×) | 774 (1.7×) |
| div, constant time | 1024 | 1,139 | 2,484 (2.2×) | 1,641 (1.4×) | 2,795 (2.5×) | 2,093 (1.8×) |
| div, constant time | 2048 | 3,426 | 7,586 (2.2×) | 4,492 (1.3×) | 8,472 (2.5×) | 6,059 (1.8×) |
| div, constant time | 4096 | 11,846 | 25,655 (2.2×) | 13,925 (1.2×) | 27,724 (2.3×) | 20,131 (1.7×) |
| mod, constant time | 64 | 25.5 | 85.1 (3.3×) | 50.9 (2.0×) | 159 (6.2×) | 61.2 (2.4×) |
| mod, constant time | 128 | 105 | 310 (3.0×) | 161 (1.5×) | 342 (3.3×) | 221 (2.1×) |
| mod, constant time | 256 | 216 | 528 (2.4×) | 317 (1.5×) | 584 (2.7×) | 402 (1.9×) |
| mod, constant time | 384 | 311 | 755 (2.4×) | 502 (1.6×) | 783 (2.5×) | 549 (1.8×) |
| mod, constant time | 512 | 466 | 1,064 (2.3×) | 727 (1.6×) | 1,110 (2.4×) | 771 (1.7×) |
| mod, constant time | 1024 | 1,147 | 2,531 (2.2×) | 1,680 (1.5×) | 2,741 (2.4×) | 1,914 (1.7×) |
| mod, constant time | 2048 | 3,445 | 7,706 (2.2×) | 4,578 (1.3×) | 8,342 (2.4×) | 5,773 (1.7×) |
| mod, constant time | 4096 | 11,904 | 25,982 (2.2×) | 14,274 (1.2×) | 28,438 (2.4×) | 19,678 (1.7×) |
| mulmod, constant time | 64 | 51.9 | 178 (3.4×) | 86.1 (1.7×) | 225 (4.3×) | 106 (2.0×) |
| mulmod, constant time | 128 | 201 | 563 (2.8×) | 346 (1.7×) | 595 (3.0×) | 363 (1.8×) |
| mulmod, constant time | 256 | 453 | 1,217 (2.7×) | 680 (1.5×) | 1,338 (3.0×) | 796 (1.8×) |
| mulmod, constant time | 384 | 823 | 1,944 (2.4×) | 1,257 (1.5×) | 2,244 (2.7×) | 1,421 (1.7×) |
| mulmod, constant time | 512 | 1,189 | 3,025 (2.5×) | 1,855 (1.6×) | 3,553 (3.0×) | 2,240 (1.9×) |
| mulmod, constant time | 1024 | 3,856 | 9,603 (2.5×) | 5,221 (1.4×) | 10,782 (2.8×) | 7,063 (1.8×) |
| mulmod, constant time | 2048 | 13,721 | 32,976 (2.4×) | 17,090 (1.2×) | 36,417 (2.7×) | 24,018 (1.8×) |
| mulmod, constant time | 4096 | 48,937 | 119,622 (2.4×) | 60,969 (1.2×) | 132,064 (2.7×) | 86,501 (1.8×) |
| add_mod, constant time | 64 | 6.6 | 10.4 (1.6×) | 10.4 (1.6×) | 20.0 (3.0×) | 14.6 (2.2×) |
| add_mod, constant time | 128 | 11.0 | 26.7 (2.4×) | 22.2 (2.0×) | 34.3 (3.1×) | 34.1 (3.1×) |
| add_mod, constant time | 256 | 20.6 | 78.8 (3.8×) | 66.1 (3.2×) | 98.8 (4.8×) | 70.0 (3.4×) |
| add_mod, constant time | 384 | 38.7 | 122 (3.2×) | 110 (2.8×) | 167 (4.3×) | 132 (3.4×) |
| add_mod, constant time | 512 | 56.5 | 150 (2.7×) | 143 (2.5×) | 236 (4.2×) | 168 (3.0×) |
| add_mod, constant time | 1024 | 130 | 323 (2.5×) | 267 (2.1×) | 594 (4.6×) | 337 (2.6×) |
| add_mod, constant time | 2048 | 303 | 849 (2.8×) | 532 (1.8×) | 1,699 (5.6×) | 732 (2.4×) |
| add_mod, constant time | 4096 | 511 | 1,857 (3.6×) | 1,675 (3.3×) | 2,040 (4.0×) | 1,988 (3.9×) |
| sub_mod, constant time | 64 | 6.2 | 11.2 (1.8×) | 10.7 (1.7×) | 18.1 (2.9×) | 15.1 (2.4×) |
| sub_mod, constant time | 128 | 7.4 | 16.0 (2.2×) | 13.7 (1.9×) | 29.1 (3.9×) | 22.1 (3.0×) |
| sub_mod, constant time | 256 | 13.7 | 46.3 (3.4×) | 32.7 (2.4×) | 72.1 (5.3×) | 59.8 (4.4×) |
| sub_mod, constant time | 384 | 27.0 | 105 (3.9×) | 52.2 (1.9×) | 135 (5.0×) | 88.3 (3.3×) |
| sub_mod, constant time | 512 | 33.6 | 144 (4.3×) | 89.0 (2.6×) | 222 (6.6×) | 138 (4.1×) |
| sub_mod, constant time | 1024 | 96.9 | 319 (3.3×) | 210 (2.2×) | 520 (5.4×) | 271 (2.8×) |
| sub_mod, constant time | 2048 | 414 | 730 (1.8×) | 512 (1.2×) | 1,316 (3.2×) | 656 (1.6×) |
| sub_mod, constant time | 4096 | 487 | 1,547 (3.2×) | 1,389 (2.9×) | 1,680 (3.4×) | 1,634 (3.4×) |
| add_mod, reduced operands, constant time | 64 | 3.8 | 6.5 (1.7×) | 6.7 (1.8×) | 6.3 (1.7×) | 6.8 (1.8×) |
| add_mod, reduced operands, constant time | 128 | 6.9 | 15.4 (2.2×) | 12.9 (1.9×) | 13.3 (1.9×) | 16.0 (2.3×) |
| add_mod, reduced operands, constant time | 256 | 9.9 | 30.7 (3.1×) | 25.3 (2.6×) | 33.2 (3.4×) | 31.1 (3.1×) |
| add_mod, reduced operands, constant time | 384 | 13.2 | 47.8 (3.6×) | 36.8 (2.8×) | 50.0 (3.8×) | 45.2 (3.4×) |
| add_mod, reduced operands, constant time | 512 | 19.8 | 59.1 (3.0×) | 50.4 (2.5×) | 70.9 (3.6×) | 60.7 (3.1×) |
| add_mod, reduced operands, constant time | 1024 | 57.8 | 137 (2.4×) | 95.6 (1.7×) | 285 (4.9×) | 120 (2.1×) |
| add_mod, reduced operands, constant time | 2048 | 132 | 281 (2.1×) | 182 (1.4×) | 667 (5.1×) | 279 (2.1×) |
| add_mod, reduced operands, constant time | 4096 | 269 | 778 (2.9×) | 733 (2.7×) | 805 (3.0×) | 878 (3.3×) |
| sub_mod, reduced operands, constant time | 64 | 3.2 | 4.8 (1.5×) | 5.2 (1.6×) | 4.5 (1.4×) | 4.8 (1.5×) |
| sub_mod, reduced operands, constant time | 128 | 5.1 | 9.5 (1.9×) | 9.5 (1.9×) | 9.2 (1.8×) | 11.0 (2.2×) |
| sub_mod, reduced operands, constant time | 256 | 8.9 | 26.6 (3.0×) | 22.9 (2.6×) | 29.5 (3.3×) | 27.2 (3.1×) |
| sub_mod, reduced operands, constant time | 384 | 11.0 | 41.2 (3.7×) | 36.2 (3.3×) | 59.2 (5.4×) | 44.0 (4.0×) |
| sub_mod, reduced operands, constant time | 512 | 18.5 | 62.7 (3.4×) | 47.7 (2.6×) | 96.1 (5.2×) | 62.7 (3.4×) |
| sub_mod, reduced operands, constant time | 1024 | 48.5 | 135 (2.8×) | 115 (2.4×) | 236 (4.9×) | 136 (2.8×) |
| sub_mod, reduced operands, constant time | 2048 | 116 | 298 (2.6×) | 256 (2.2×) | 543 (4.7×) | 292 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 270 | 761 (2.8×) | 731 (2.7×) | 840 (3.1×) | 897 (3.3×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add | 128 | 1.4 | 3.0 (2.1×) | 2.4 (1.7×) | 3.0 (2.1×) | 2.9 (2.1×) |
| add | 256 | 2.5 | 8.9 (3.6×) | 6.6 (2.6×) | 8.9 (3.6×) | 7.7 (3.1×) |
| add | 384 | 3.5 | 15.8 (4.5×) | 11.2 (3.2×) | 16.1 (4.6×) | 18.1 (5.2×) |
| add | 512 | 4.0 | 22.2 (5.5×) | 16.2 (4.0×) | 21.5 (5.4×) | 24.6 (6.2×) |
| add | 1024 | 22.1 | 50.7 (2.3×) | 39.7 (1.8×) | 52.2 (2.4×) | 58.0 (2.6×) |
| add | 2048 | 51.4 | 115 (2.2×) | 94.6 (1.8×) | 118 (2.3×) | 125 (2.4×) |
| add | 4096 | 163 | 233 (1.4×) | 219 (1.3×) | 237 (1.5×) | 269 (1.7×) |
| mulmod | 64 | 23.0 | 68.5 (3.0×) | 34.8 (1.5×) | 122 (5.3×) | 42.8 (1.9×) |
| mulmod | 128 | 50.7 | 209 (4.1×) | 85.0 (1.7×) | 258 (5.1×) | 150 (3.0×) |
| mulmod | 256 | 183 | 588 (3.2×) | 268 (1.5×) | 700 (3.8×) | 344 (1.9×) |
| mulmod | 384 | 316 | 950 (3.0×) | 445 (1.4×) | 1,260 (4.0×) | 647 (2.0×) |
| mulmod | 512 | 483 | 1,567 (3.2×) | 776 (1.6×) | 2,114 (4.4×) | 1,087 (2.3×) |
| mulmod | 1024 | 1,337 | 5,307 (4.0×) | 2,413 (1.8×) | 6,731 (5.0×) | 3,168 (2.4×) |
| mulmod | 2048 | 5,198 | 17,524 (3.4×) | 8,004 (1.5×) | 21,812 (4.2×) | 11,326 (2.2×) |
| mulmod | 4096 | 14,977 | 56,469 (3.8×) | 26,734 (1.8×) | 67,924 (4.5×) | 38,730 (2.6×) |
| add_mod | 64 | 8.0 | 6.2 (0.8×) | 6.1 (0.8×) | 8.2 (1.0×) | 8.2 (1.0×) |
| add_mod | 128 | 11.9 | 31.3 (2.6×) | 10.6 (0.9×) | 41.1 (3.5×) | 16.0 (1.3×) |
| add_mod | 256 | 18.5 | 40.6 (2.2×) | 30.8 (1.7×) | 54.3 (2.9×) | 43.4 (2.3×) |
| add_mod | 384 | 22.9 | 54.7 (2.4×) | 41.3 (1.8×) | 78.2 (3.4×) | 63.6 (2.8×) |
| add_mod | 512 | 33.5 | 67.3 (2.0×) | 51.4 (1.5×) | 90.9 (2.7×) | 81.4 (2.4×) |
| add_mod | 1024 | 79.2 | 128 (1.6×) | 118 (1.5×) | 158 (2.0×) | 160 (2.0×) |
| add_mod | 2048 | 155 | 289 (1.9×) | 228 (1.5×) | 339 (2.2×) | 302 (1.9×) |
| add_mod | 4096 | 333 | 659 (2.0×) | 699 (2.1×) | 834 (2.5×) | 869 (2.6×) |
| sub_mod | 64 | 7.3 | 4.4 (0.6×) | 7.3 (1.0×) | 7.9 (1.1×) | 8.1 (1.1×) |
| sub_mod | 128 | 11.3 | 32.1 (2.8×) | 9.2 (0.8×) | 40.2 (3.6×) | 15.3 (1.4×) |
| sub_mod | 256 | 14.4 | 45.3 (3.1×) | 30.5 (2.1×) | 55.3 (3.8×) | 43.0 (3.0×) |
| sub_mod | 384 | 23.1 | 53.9 (2.3×) | 41.3 (1.8×) | 70.9 (3.1×) | 60.1 (2.6×) |
| sub_mod | 512 | 37.4 | 70.4 (1.9×) | 51.9 (1.4×) | 88.7 (2.4×) | 80.4 (2.1×) |
| sub_mod | 1024 | 79.9 | 135 (1.7×) | 115 (1.4×) | 160 (2.0×) | 166 (2.1×) |
| sub_mod | 2048 | 144 | 280 (1.9×) | 249 (1.7×) | 372 (2.6×) | 331 (2.3×) |
| sub_mod | 4096 | 320 | 778 (2.4×) | 731 (2.3×) | 881 (2.8×) | 903 (2.8×) |
| add_mod, reduced operands | 64 | 4.9 | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) |
| add_mod, reduced operands | 128 | 6.0 | 6.7 (1.1×) | 6.8 (1.1×) | 8.1 (1.3×) | 10.3 (1.7×) |
| add_mod, reduced operands | 256 | 12.9 | 16.6 (1.3×) | 13.5 (1.0×) | 16.7 (1.3×) | 20.3 (1.6×) |
| add_mod, reduced operands | 384 | 14.6 | 27.3 (1.9×) | 19.7 (1.3×) | 27.6 (1.9×) | 32.1 (2.2×) |
| add_mod, reduced operands | 512 | 20.2 | 34.6 (1.7×) | 26.6 (1.3×) | 39.6 (2.0×) | 42.9 (2.1×) |
| add_mod, reduced operands | 1024 | 44.9 | 75.5 (1.7×) | 63.0 (1.4×) | 76.5 (1.7×) | 88.5 (2.0×) |
| add_mod, reduced operands | 2048 | 83.0 | 198 (2.4×) | 144 (1.7×) | 230 (2.8×) | 200 (2.4×) |
| add_mod, reduced operands | 4096 | 240 | 414 (1.7×) | 419 (1.7×) | 497 (2.1×) | 538 (2.2×) |
| sub_mod, reduced operands | 64 | 3.9 | 2.3 (0.6×) | 2.7 (0.7×) | 3.2 (0.8×) | 3.7 (0.9×) |
| sub_mod, reduced operands | 128 | 4.8 | 5.8 (1.2×) | 5.6 (1.2×) | 6.2 (1.3×) | 8.8 (1.8×) |
| sub_mod, reduced operands | 256 | 7.4 | 15.2 (2.1×) | 12.4 (1.7×) | 13.3 (1.8×) | 18.8 (2.5×) |
| sub_mod, reduced operands | 384 | 10.1 | 24.9 (2.5×) | 19.1 (1.9×) | 25.3 (2.5×) | 29.6 (2.9×) |
| sub_mod, reduced operands | 512 | 16.1 | 34.5 (2.1×) | 27.4 (1.7×) | 40.6 (2.5×) | 41.1 (2.6×) |
| sub_mod, reduced operands | 1024 | 37.8 | 70.0 (1.9×) | 68.7 (1.8×) | 78.8 (2.1×) | 90.8 (2.4×) |
| sub_mod, reduced operands | 2048 | 75.6 | 204 (2.7×) | 164 (2.2×) | 287 (3.8×) | 204 (2.7×) |
| sub_mod, reduced operands | 4096 | 238 | 432 (1.8×) | 441 (1.9×) | 471 (2.0×) | 555 (2.3×) |
| add, constant time | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add, constant time | 128 | 1.3 | 3.0 (2.3×) | 2.4 (1.8×) | 2.9 (2.2×) | 2.9 (2.2×) |
| add, constant time | 256 | 2.3 | 8.9 (3.9×) | 6.6 (2.9×) | 8.8 (3.8×) | 7.7 (3.3×) |
| add, constant time | 384 | 3.7 | 15.6 (4.2×) | 11.2 (3.0×) | 16.1 (4.4×) | 18.1 (4.9×) |
| add, constant time | 512 | 4.1 | 22.4 (5.5×) | 16.7 (4.1×) | 21.6 (5.3×) | 24.6 (6.0×) |
| add, constant time | 1024 | 22.0 | 50.6 (2.3×) | 39.7 (1.8×) | 52.1 (2.4×) | 58.0 (2.6×) |
| add, constant time | 2048 | 51.0 | 114 (2.2×) | 94.6 (1.9×) | 118 (2.3×) | 125 (2.5×) |
| add, constant time | 4096 | 161 | 234 (1.5×) | 219 (1.4×) | 237 (1.5×) | 269 (1.7×) |
| mulmod, constant time | 64 | 93.9 | 159 (1.7×) | 93.1 (1.0×) | 242 (2.6×) | 115 (1.2×) |
| mulmod, constant time | 128 | 206 | 567 (2.8×) | 312 (1.5×) | 606 (2.9×) | 362 (1.8×) |
| mulmod, constant time | 256 | 471 | 1,183 (2.5×) | 696 (1.5×) | 1,303 (2.8×) | 819 (1.7×) |
| mulmod, constant time | 384 | 830 | 1,921 (2.3×) | 1,175 (1.4×) | 2,268 (2.7×) | 1,428 (1.7×) |
| mulmod, constant time | 512 | 1,189 | 3,013 (2.5×) | 1,736 (1.5×) | 3,451 (2.9×) | 2,222 (1.9×) |
| mulmod, constant time | 1024 | 3,878 | 9,583 (2.5×) | 5,196 (1.3×) | 10,929 (2.8×) | 7,083 (1.8×) |
| mulmod, constant time | 2048 | 13,505 | 33,040 (2.4×) | 17,147 (1.3×) | 36,976 (2.7×) | 24,135 (1.8×) |
| mulmod, constant time | 4096 | 48,916 | 120,322 (2.5×) | 59,507 (1.2×) | 131,241 (2.7×) | 85,773 (1.8×) |
| add_mod, constant time | 64 | 8.9 | 12.1 (1.4×) | 13.0 (1.5×) | 19.7 (2.2×) | 13.1 (1.5×) |
| add_mod, constant time | 128 | 12.5 | 27.2 (2.2×) | 23.3 (1.9×) | 35.0 (2.8×) | 34.1 (2.7×) |
| add_mod, constant time | 256 | 18.1 | 82.8 (4.6×) | 68.5 (3.8×) | 102 (5.6×) | 78.2 (4.3×) |
| add_mod, constant time | 384 | 36.0 | 112 (3.1×) | 108 (3.0×) | 171 (4.8×) | 127 (3.5×) |
| add_mod, constant time | 512 | 47.8 | 146 (3.1×) | 136 (2.8×) | 233 (4.9×) | 166 (3.5×) |
| add_mod, constant time | 1024 | 131 | 328 (2.5×) | 263 (2.0×) | 629 (4.8×) | 353 (2.7×) |
| add_mod, constant time | 2048 | 308 | 874 (2.8×) | 513 (1.7×) | 1,594 (5.2×) | 751 (2.4×) |
| add_mod, constant time | 4096 | 551 | 1,859 (3.4×) | 1,691 (3.1×) | 2,003 (3.6×) | 1,958 (3.6×) |
| sub_mod, constant time | 64 | 8.9 | 12.2 (1.4×) | 11.0 (1.2×) | 19.2 (2.2×) | 13.8 (1.6×) |
| sub_mod, constant time | 128 | 11.8 | 24.8 (2.1×) | 20.0 (1.7×) | 31.7 (2.7×) | 22.0 (1.9×) |
| sub_mod, constant time | 256 | 17.1 | 52.1 (3.0×) | 36.1 (2.1×) | 75.3 (4.4×) | 58.5 (3.4×) |
| sub_mod, constant time | 384 | 27.5 | 103 (3.7×) | 57.6 (2.1×) | 125 (4.5×) | 87.4 (3.2×) |
| sub_mod, constant time | 512 | 32.2 | 141 (4.4×) | 86.1 (2.7×) | 196 (6.1×) | 119 (3.7×) |
| sub_mod, constant time | 1024 | 94.2 | 313 (3.3×) | 201 (2.1×) | 533 (5.7×) | 256 (2.7×) |
| sub_mod, constant time | 2048 | 284 | 739 (2.6×) | 512 (1.8×) | 1,277 (4.5×) | 573 (2.0×) |
| sub_mod, constant time | 4096 | 506 | 1,537 (3.0×) | 1,391 (2.7×) | 1,680 (3.3×) | 1,614 (3.2×) |
| add_mod, reduced operands, constant time | 64 | 6.9 | 8.3 (1.2×) | 8.0 (1.2×) | 8.0 (1.2×) | 8.5 (1.2×) |
| add_mod, reduced operands, constant time | 128 | 8.8 | 15.9 (1.8×) | 14.0 (1.6×) | 14.7 (1.7×) | 15.5 (1.8×) |
| add_mod, reduced operands, constant time | 256 | 12.9 | 31.7 (2.5×) | 24.0 (1.9×) | 29.1 (2.3×) | 31.1 (2.4×) |
| add_mod, reduced operands, constant time | 384 | 14.6 | 46.1 (3.2×) | 32.8 (2.2×) | 48.4 (3.3×) | 46.7 (3.2×) |
| add_mod, reduced operands, constant time | 512 | 20.3 | 58.1 (2.9×) | 43.8 (2.2×) | 71.8 (3.5×) | 62.3 (3.1×) |
| add_mod, reduced operands, constant time | 1024 | 51.6 | 142 (2.8×) | 93.7 (1.8×) | 298 (5.8×) | 122 (2.4×) |
| add_mod, reduced operands, constant time | 2048 | 140 | 278 (2.0×) | 184 (1.3×) | 678 (4.8×) | 293 (2.1×) |
| add_mod, reduced operands, constant time | 4096 | 301 | 720 (2.4×) | 712 (2.4×) | 814 (2.7×) | 823 (2.7×) |
| sub_mod, reduced operands, constant time | 64 | 3.9 | 4.0 (1.0×) | 5.0 (1.3×) | 4.0 (1.0×) | 5.1 (1.3×) |
| sub_mod, reduced operands, constant time | 128 | 4.8 | 11.1 (2.3×) | 9.4 (2.0×) | 10.1 (2.1×) | 10.3 (2.1×) |
| sub_mod, reduced operands, constant time | 256 | 6.9 | 24.4 (3.5×) | 19.9 (2.9×) | 28.3 (4.1×) | 27.1 (3.9×) |
| sub_mod, reduced operands, constant time | 384 | 9.5 | 39.3 (4.1×) | 29.9 (3.1×) | 45.6 (4.8×) | 44.7 (4.7×) |
| sub_mod, reduced operands, constant time | 512 | 18.0 | 59.4 (3.3×) | 44.9 (2.5×) | 87.4 (4.9×) | 63.1 (3.5×) |
| sub_mod, reduced operands, constant time | 1024 | 44.8 | 132 (2.9×) | 110 (2.5×) | 224 (5.0×) | 140 (3.1×) |
| sub_mod, reduced operands, constant time | 2048 | 121 | 294 (2.4×) | 257 (2.1×) | 555 (4.6×) | 306 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 304 | 774 (2.5×) | 698 (2.3×) | 871 (2.9×) | 875 (2.9×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- bench-wasm --changed --chrome <chrome> --firefox <firefox>   # the cells whose code changed, with the figures they are compared with
cargo run -p biguint-bench-report -- report   # this file
```
