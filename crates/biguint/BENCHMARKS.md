# Benchmarks

`biguint` at commit `550ba94` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 22.09 before, 18.41 after; its SMT sibling, CPU 29, 24% busy |
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
| 64 | **1.4** / **1.0** | **1.4** / **1.0** | 1.5 / **1.0** | **1.4** / **1.0** | **1.4** / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / **1.3** | **2.4** / **1.3** | 2.5 / **1.3** | **2.4** / **1.3** | 2.6 / 1.4 | – |
| 256 | **4.5** / **2.3** | – | 4.8 / 2.4 | 5.6 / 5.5 | 5.0 / 3.1 | – |
| 384 | **7.1** / **3.5** | – | 7.2 / 3.9 | 9.0 / 10.5 | 7.9 / 5.2 | – |
| 512 | **9.2** / **5.2** | – | 9.5 / 5.3 | 12.3 / 16.5 | 11.5 / 7.9 | – |
| 1024 | **23.7** / **22.2** | – | 27.8 / 22.4 | 28.3 / 28.6 | 27.2 / 28.4 | – |
| 2048 | **54.3** / **51.5** | – | 71.6 / 64.3 | 79.2 / 91.6 | 90.8 / 106 | – |
| 4096 | **147** / **175** | – | 190 / 191 | 202 / 274 | 216 / 265 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | 1.5 / – | **1.4** / – | **1.4** / – | 1.5 / – | – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | **12.4** / – | – | 12.5 / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | **28.0** / – | – | 29.5 / – | 29.9 / – | 33.7 / – | 36.8 / – | – |
| 512 | **52.4** / – | – | 59.9 / – | 81.5 / – | 86.4 / – | – | – |
| 1024 | **221** / – | – | – | 435 / – | 388 / – | 439 / – | – |
| 2048 | **1,116** / – | – | 1,604 / – | 2,001 / – | 1,529 / – | 4,066 / – | 3,566 / – |
| 4096 | **3,842** / – | – | 6,449 / – | 7,769 / – | 6,108 / – | 24,934 / – | 11,270 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | **6.9** / – | **6.9** / – | 222 / – | 7.0 / – | 7.0 / – | 37.2 / – |
| 128 | **16.0** / – | 16.2 / – | 31.8 / – | 267 / – | 29.9 / – | 30.0 / – | 99.1 / – |
| 256 | **46.0** / – | – | 78.9 / – | 544 / – | 243 / – | 203 / – | 270 / – |
| 384 | **137** / – | – | 195 / – | 712 / – | 358 / – | 309 / – | 335 / – |
| 512 | **162** / – | – | 264 / – | 838 / – | 450 / – | 395 / – | 398 / – |
| 1024 | **479** / – | – | 587 / – | 1,643 / – | 1,096 / – | 969 / – | 792 / – |
| 2048 | **1,382** / – | – | 1,530 / – | 4,079 / – | 3,393 / – | 3,269 / – | 2,063 / – |
| 4096 | **4,612** / – | – | 4,856 / – | 12,766 / – | 12,329 / – | 12,296 / – | 6,360 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | 7.0 / – | 7.0 / – | 221 / – | 7.0 / – | 7.0 / – | 8.0 / – |
| 128 | **17.0** / – | 17.3 / – | 30.5 / – | 268 / – | 32.0 / – | 31.0 / – | 101 / – |
| 256 | **41.5** / – | – | 79.3 / – | 531 / – | 255 / – | 197 / – | 271 / – |
| 384 | **145** / – | – | 190 / – | 711 / – | 362 / – | 329 / – | 337 / – |
| 512 | **203** / – | – | 244 / – | 838 / – | 473 / – | 391 / – | 397 / – |
| 1024 | **476** / – | – | 564 / – | 1,618 / – | 1,114 / – | 968 / – | 795 / – |
| 2048 | **1,446** / – | – | 1,528 / – | 4,068 / – | 3,403 / – | 3,283 / – | 2,074 / – |
| 4096 | **4,563** / – | – | 4,895 / – | 12,796 / – | 12,438 / – | 12,628 / – | 6,381 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **11.9** / **23.0** | 12.0 / **23.0** | 20.2 / 42.3 | 228 / 226 | 145 / – |
| 128 | **47.3** / **50.7** | – | 80.2 / 90.3 | 495 / 501 | 366 / – |
| 256 | 177 / **185** | – | **165** / 204 | 982 / 1,004 | 558 / – |
| 384 | **315** / **346** | – | 376 / 390 | 1,460 / 1,498 | 858 / – |
| 512 | **497** / **501** | – | 538 / 559 | 1,958 / 1,981 | 1,164 / – |
| 1024 | **1,382** / **1,360** | – | 1,580 / 1,584 | 5,162 / 5,215 | 3,174 / – |
| 2048 | **5,205** / **5,198** | – | 7,382 / 7,401 | 15,934 / 16,071 | 9,900 / – |
| 4096 | **14,886** / **14,951** | – | 29,522 / 29,532 | 57,387 / 57,639 | 34,038 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.9** / **7.8** | 2.9\* / – | 11.9 / 13.4 | 11.1 / – | 17.2 / – | 3.0 / – | 97.3 / – |
| 128 | **3.8** / **11.6** | 6.0 / – | 33.6 / 21.4 | 27.3 / – | 16.4 / – | 24.7 / – | 236 / – |
| 256 | **9.6** / **14.7** | – | 43.4 / 62.3 | 69.9 / – | 22.9 / – | 55.6 / – | 230 / – |
| 384 | **17.9** / **28.4** | – | 57.0 / 78.7 | 90.6 / – | 31.9 / – | 88.9 / – | 244 / – |
| 512 | **27.3** / **32.0** | – | 73.5 / 92.4 | 122 / – | 40.5 / – | 127 / – | 284 / – |
| 1024 | **69.2** / **79.1** | – | 146 / 164 | 144 / – | 69.7 / – | 272 / – | 351 / – |
| 2048 | **161** / **163** | – | 300 / 342 | 430 / – | 155\* / – | 586 / – | 393 / – |
| 4096 | **299** / **324** | – | 751 / 769 | 894 / – | 372 / – | 1,448 / – | 647 / – |

\* `native` at 64 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

\* `bnum` at 2048 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.4** / **7.7** | 11.9 / – | 11.9 / 13.4 | 4.6 / – | 13.0 / – | 12.0 / – | 157 / – |
| 128 | **4.3** / **11.1** | 3.8\* / – | 32.5 / 22.0 | 11.5 / – | 13.7 / – | 18.5 / – | 335 / – |
| 256 | **9.0** / **14.3** | – | 46.0 / 61.7 | 49.2 / – | 22.3 / – | 47.9 / – | 318 / – |
| 384 | **16.5** / **29.3** | – | 67.4 / 84.4 | 81.6 / – | 29.2 / – | 78.0 / – | 336 / – |
| 512 | **25.6** / **37.8** | – | 89.2 / 106 | 111 / – | 39.7 / – | 127 / – | 418 / – |
| 1024 | **66.5** / **71.1** | – | 172 / 197 | 130 / – | 68.4 / – | 303 / – | 461 / – |
| 2048 | **148** / **147** | – | 437 / 463 | 290 / – | 178 / – | 655 / – | 538 / – |
| 4096 | **237** / **388** | – | 948 / 1,047 | 613 / – | 391 / – | 1,469 / – | 897 / – |

\* `native` at 128 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (one subtraction or addition of m).

### add_mod, reduced operands: a + b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.7** / **5.0** | 2.9 / – | 11.9 / 13.6 | 11.2 / – | 17.1 / – | 3.0 / – | 107 / – |
| 128 | **4.3** / **5.7** | 6.0 / – | 33.8 / 12.9 | 27.4 / – | 16.3 / – | 25.3 / – | 219 / – |
| 256 | **19.6** / **13.4** | – | 45.0 / 44.7 | 70.5 / – | 23.3 / – | 51.8 / – | 211 / – |
| 384 | **26.3** / **9.6** | – | 58.9 / 62.3 | 90.7 / – | 32.3 / – | 82.0 / – | 228 / – |
| 512 | **31.9** / **25.7** | – | 69.8 / 72.4 | 122 / – | 39.7 / – | 112 / – | 307 / – |
| 1024 | **47.4** / **44.1** | – | 147 / 149 | 144 / – | 70.9 / – | 253 / – | 295 / – |
| 2048 | **89.3** / **83.4** | – | 301 / 325 | 429 / – | 154 / – | 525 / – | 450 / – |
| 4096 | **235** / **253** | – | 749 / 751 | 900 / – | 368 / – | 1,201 / – | 544 / – |

### sub_mod, reduced operands: a − b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.3** / **3.9** | 11.7 / – | 11.9 / 13.6 | 4.6 / – | 13.4 / – | 12.0 / – | 170 / – |
| 128 | 4.8 / **4.8** | **3.9** / – | 31.5 / 13.0 | 11.5 / – | 13.5 / – | 18.0 / – | 313 / – |
| 256 | **7.4** / **7.4** | – | 45.6 / 43.2 | 48.9 / – | 22.0 / – | 44.6 / – | 269 / – |
| 384 | **11.0** / **10.3** | – | 69.2 / 68.2 | 81.8 / – | 29.4 / – | 74.0 / – | 294 / – |
| 512 | **18.6** / **16.3** | – | 77.7 / 79.7 | 111 / – | 40.2 / – | 114 / – | 453 / – |
| 1024 | **44.3** / **38.1** | – | 166 / 170 | 130 / – | 68.4 / – | 280 / – | 456 / – |
| 2048 | **83.5** / **75.8** | – | 406 / 406 | 291 / – | 179 / – | 616 / – | 595 / – |
| 4096 | **240** / **241** | – | 863 / 862 | 613 / – | 413 / – | 1,416 / – | 746 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | **1.4** / **1.0** | 1.5 / **1.0** | **1.4** / **1.0** | **1.4** / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / **1.3** | **2.4** / 1.4 | 2.5 / 1.4 | 2.5 / 1.5 | 2.6 / 1.4 | – |
| 256 | **4.6** / **2.4** | – | **4.6** / **2.4** | 5.7 / 5.5 | 5.0 / 2.9 | – |
| 384 | **7.0** / **3.7** | – | 7.3 / 3.9 | 9.1 / 10.3 | 7.9 / 5.1 | – |
| 512 | **9.2** / **4.7** | – | 9.8 / 5.1 | 12.5 / 16.5 | 11.5 / 7.8 | – |
| 1024 | **23.6** / **22.4** | – | 27.9 / 22.5 | 27.3 / 28.7 | 28.2 / 29.0 | – |
| 2048 | **52.6** / **50.3** | – | 72.0 / 62.8 | 79.0 / 94.4 | 91.1 / 106 | – |
| 4096 | **155** / **174** | – | 198 / 193 | 202 / 274 | 216 / 265 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | 1.5 / **3.0** | 1.5 / – | **1.4** / – | **1.4** / – | 1.5 / – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – |
| 256 | **12.4** / – | – | **12.4** / – | 13.0 / – | 13.0 / – | 16.0 / – |
| 384 | 30.0 / – | – | 30.6 / – | **29.8** / – | 33.6 / – | 36.8 / – |
| 512 | **51.1** / – | – | 59.9 / – | 82.1 / – | 86.5 / – | – |
| 1024 | **226** / – | – | – | 434 / – | 389 / – | 441 / – |
| 2048 | **1,160** / – | – | – | 2,001 / – | 1,529 / – | 4,066 / – |
| 4096 | **4,217** / – | – | – | 7,769 / – | 6,108 / – | 24,934 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.1** / – | 198 / – |
| 128 | **98.7** / – | 407 / – |
| 256 | **200** / – | 751 / – |
| 384 | **292** / – | 1,097 / – |
| 512 | **460** / – | 1,451 / – |
| 1024 | **1,142** / – | 3,345 / – |
| 2048 | **3,457** / – | 10,162 / – |
| 4096 | **11,885** / – | 30,828 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.6** / – | 188 / – |
| 128 | **105** / – | 387 / – |
| 256 | **328** / – | 723 / – |
| 384 | **314** / – | 1,053 / – |
| 512 | **470** / – | 1,438 / – |
| 1024 | **1,142** / – | 3,342 / – |
| 2048 | **3,459** / – | 10,090 / – |
| 4096 | **11,898** / – | 30,824 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **51.7** / **93.9** | 224 / – |
| 128 | **201** / **203** | 607 / – |
| 256 | **444** / **462** | 1,224 / – |
| 384 | **819** / **826** | 1,955 / – |
| 512 | **1,186** / **1,189** | 2,960 / – |
| 1024 | **3,872** / **3,864** | 7,644 / – |
| 2048 | **13,645** / **13,598** | 25,212 / – |
| 4096 | **48,900** / **48,809** | 86,171 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **6.6** / **8.9** | 11.1 / – |
| 128 | **11.0** / **12.5** | 27.3 / – |
| 256 | **20.4** / **18.2** | 69.9 / – |
| 384 | **39.0** / **36.0** | 90.6 / – |
| 512 | **49.8** / **47.8** | 122 / – |
| 1024 | **129** / **130** | 144 / – |
| 2048 | **303** / **307** | 430 / – |
| 4096 | **560** / **501** | 894 / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `biguint`'s; the note under the table says how the function deviates.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **6.3** / **8.9** | 4.6\* / – |
| 128 | **7.4** / **11.8** | 11.5 / – |
| 256 | **13.7** / **17.1** | 49.2 / – |
| 384 | **26.9** / **27.6** | 81.6 / – |
| 512 | **33.5** / **32.4** | 111 / – |
| 1024 | **96.5** / **93.5** | 130 / – |
| 2048 | **278** / **285** | 290 / – |
| 4096 | **503** / **487** | 613 / – |

\* `crypto-bigint` at 64 bits: unchecked precondition: operands must be below m; other inputs give a wrong result (`Uint::sub_mod`: "Assumes `self - rhs` as unbounded signed integer is in `[-p, p)`").

### add_mod, reduced operands, constant time: a + b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **3.8** / **6.9** | 11.2 / – |
| 128 | **6.9** / **8.8** | 27.4 / – |
| 256 | **9.8** / **13.2** | 70.5 / – |
| 384 | **13.2** / **14.7** | 90.7 / – |
| 512 | **19.8** / **20.6** | 122 / – |
| 1024 | **56.0** / **51.9** | 144 / – |
| 2048 | **132** / **141** | 429 / – |
| 4096 | **288** / **290** | 900 / – |

### sub_mod, reduced operands, constant time: a − b mod m, a and b below m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **3.2** / **3.9** | 4.6 / – |
| 128 | **5.1** / **4.8** | 11.5 / – |
| 256 | **8.9** / **6.9** | 48.9 / – |
| 384 | **11.0** / **9.3** | 81.8 / – |
| 512 | **18.5** / **17.8** | 111 / – |
| 1024 | **47.4** / **43.9** | 130 / – |
| 2048 | **115** / **120** | 291 / – |
| 4096 | **273** / **356** | 613 / – |

## WebAssembly

`biguint` at commit `58bddf1` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,729 to 13,713 cycles and 3,081 instructions; its noise in a figure, 203 to 2,576 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,618 to 11,918 cycles and 3,081 instructions; its noise in a figure, 176 to 1,585 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 5,293 to 19,107 cycles and 1,885 instructions; its noise in a figure, 131 to 7,745 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 5,659 to 9,708 cycles and 1,885 instructions; its noise in a figure, 176 to 1,822 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 20.67 before, 17.8 after; its SMT sibling, CPU 28, 25% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 25.51 before, 14.81 after; its SMT sibling, CPU 28, 34% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 14.81 before, 16.79 after; its SMT sibling, CPU 28, 17% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 16.79 before, 19.37 after; its SMT sibling, CPU 28, 22% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.2 vs crypto-bigint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.3 vs native 3.4 | +0.9 (1.26×) |
| add | 256 | 10.3 vs ruint 10.3 | +0.0 (1.00×) | 8.6 vs ruint 9.0 | −0.4 (0.96×) | 10.1 vs ruint 10.1 | +0.0 (1.00×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add | 384 | 16.3 vs ruint 16.4 | −0.1 (0.99×) | 12.3 vs crypto-bigint 12.3 | +0.0 (1.00×) | 15.9 vs ruint 16.1 | −0.2 (0.99×) | 18.0 vs bnum 18.0 | +0.0 (1.00×) |
| add | 512 | 27.3 vs bnum 24.7 | +2.6 (1.11×) | 20.7 vs crypto-bigint 20.7 | +0.0 (1.00×) | 26.7 vs bnum 23.6 | +3.1 (1.13×) | 32.1 vs bnum 27.0 | +5.1 (1.19×) |
| add | 1024 | 64.1 vs bnum 61.9 | +2.2 (1.04×) | 46.8 vs crypto-bigint 45.4 | +1.4 (1.03×) | 82.1 vs bnum 67.2 | +14.9 (1.22×) | 68.8 vs crypto-bigint 68.2 | +0.6 (1.01×) |
| add | 2048 | 161 vs crypto-bigint 193 | −32.0 (0.83×) | 112 vs crypto-bigint 137 | −25.0 (0.82×) | 188 vs crypto-bigint 206 | −18.0 (0.91×) | 147 vs crypto-bigint 177 | −30.0 (0.83×) |
| add | 4096 | 427 vs crypto-bigint 388 | +39.0 (1.10×) | 323 vs crypto-bigint 264 | +59.0 (1.22×) | 467 vs bnum 387 | +80.0 (1.21×) | 405 vs crypto-bigint 326 | +79.0 (1.24×) |
| mul | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.1 vs native 2.1 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.2 | +2.3 (1.55×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.7 vs crypto-bigint 5.0 | +0.7 (1.14×) |
| mul | 256 | 47.7 vs ruint 66.6 | −18.9 (0.72×) | 16.7 vs ruint 16.8 | −0.1 (0.99×) | 66.9 vs uint 81.2 | −14.3 (0.82×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul | 384 | 186 vs ruint 166 | +20.0 (1.12×) | 52.6 vs crypto-bigint 39.6 | +13.0 (1.33×) | 207 vs ruint 220 | −13.0 (0.94×) | 72.6 vs crypto-bigint 71.7 | +0.9 (1.01×) |
| mul | 512 | 290 vs ruint 322 | −32.0 (0.90×) | 75.8 vs ruint 78.0 | −2.2 (0.97×) | 374 vs ruint 445 | −71.0 (0.84×) | 131 vs ruint 131 | +0.0 (1.00×) |
| mul | 1024 | 1,342 vs bnum 1,546 | −204 (0.87×) | 692 vs uint 505 | +187 (1.37×) | 1,578 vs bnum 2,363 | −785 (0.67×) | 859 vs crypto-bigint 930 | −71.0 (0.92×) |
| mul | 2048 | 5,585 vs bnum 6,283 | −698 (0.89×) | 2,479 vs ruint 2,136 | +343 (1.16×) | 6,616 vs bnum 8,683 | −2,067 (0.76×) | 3,670 vs ruint 3,207 | +463 (1.14×) |
| mul | 4096 | 17,711 vs bnum 23,558 | −5,847 (0.75×) | 8,458 vs ruint 8,002 | +456 (1.06×) | 21,190 vs bnum 34,809 | −13,619 (0.61×) | 12,251 vs ruint 11,629 | +622 (1.05×) |
| div | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 60.3 vs ruint 88.5 | −28.2 (0.68×) | 28.4 vs ruint 42.9 | −14.5 (0.66×) | 87.0 vs ruint 145 | −58.0 (0.60×) | 34.7 vs ruint 55.7 | −21.0 (0.62×) |
| div | 256 | 203 vs ruint 321 | −118 (0.63×) | 84.3 vs ruint 156 | −71.7 (0.54×) | 226 vs ruint 402 | −176 (0.56×) | 110 vs ruint 174 | −64.0 (0.63×) |
| div | 384 | 347 vs ruint 606 | −259 (0.57×) | 206 vs ruint 352 | −146 (0.59×) | 405 vs ruint 765 | −360 (0.53×) | 279 vs ruint 414 | −135 (0.67×) |
| div | 512 | 474 vs num-bigint 819 | −345 (0.58×) | 308 vs ruint 429 | −121 (0.72×) | 567 vs num-bigint 1,002 | −435 (0.57×) | 377 vs ruint 547 | −170 (0.69×) |
| div | 1024 | 1,132 vs num-bigint 1,931 | −799 (0.59×) | 686 vs ruint 922 | −236 (0.74×) | 1,468 vs num-bigint 2,207 | −739 (0.67×) | 857 vs ruint 1,121 | −264 (0.76×) |
| div | 2048 | 3,223 vs num-bigint 5,775 | −2,552 (0.56×) | 2,005 vs ruint 2,352 | −347 (0.85×) | 4,288 vs num-bigint 6,165 | −1,877 (0.70×) | 2,379 vs ruint 2,681 | −302 (0.89×) |
| div | 4096 | 12,835 vs num-bigint 20,237 | −7,402 (0.63×) | 6,991 vs ruint 7,620 | −629 (0.92×) | 15,188 vs num-bigint 20,315 | −5,127 (0.75×) | 7,465 vs ruint 9,047 | −1,582 (0.83×) |
| mod | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 52.8 vs ruint 94.1 | −41.3 (0.56×) | 30.2 vs ruint 40.0 | −9.8 (0.76×) | 90.2 vs ruint 100 | −9.8 (0.90×) | 34.0 vs ruint 50.2 | −16.2 (0.68×) |
| mod | 256 | 207 vs ruint 321 | −114 (0.64×) | 74.3 vs ruint 158 | −83.7 (0.47×) | 223 vs ruint 410 | −187 (0.54×) | 108 vs ruint 174 | −66.0 (0.62×) |
| mod | 384 | 347 vs ruint 582 | −235 (0.60×) | 197 vs ruint 324 | −127 (0.61×) | 404 vs ruint 724 | −320 (0.56×) | 266 vs ruint 344 | −78.0 (0.77×) |
| mod | 512 | 472 vs ruint 800 | −328 (0.59×) | 280 vs ruint 408 | −128 (0.69×) | 558 vs num-bigint 992 | −434 (0.56×) | 349 vs ruint 467 | −118 (0.75×) |
| mod | 1024 | 1,195 vs num-bigint 1,933 | −738 (0.62×) | 706 vs ruint 970 | −264 (0.73×) | 1,469 vs num-bigint 2,191 | −722 (0.67×) | 939 vs ruint 1,050 | −111 (0.89×) |
| mod | 2048 | 3,315 vs num-bigint 5,792 | −2,477 (0.57×) | 1,927 vs ruint 2,288 | −361 (0.84×) | 4,312 vs num-bigint 6,181 | −1,869 (0.70×) | 2,085 vs ruint 2,619 | −534 (0.80×) |
| mod | 4096 | 12,879 vs num-bigint 20,165 | −7,286 (0.64×) | 6,628 vs ruint 7,779 | −1,151 (0.85×) | 15,104 vs num-bigint 20,315 | −5,211 (0.74×) | 7,686 vs ruint 8,918 | −1,232 (0.86×) |
| mulmod | 64 | 60.8 vs ruint 84.8 | −24.0 (0.72×) | 31.4 vs ruint 39.9 | −8.5 (0.79×) | 108 vs ruint 188 | −80.0 (0.57×) | 36.1 vs ruint 55.8 | −19.7 (0.65×) |
| mulmod | 128 | 207 vs ruint 322 | −115 (0.64×) | 81.7 vs ruint 135 | −53.3 (0.61×) | 239 vs ruint 388 | −149 (0.62×) | 125 vs ruint 205 | −80.0 (0.61×) |
| mulmod | 256 | 581 vs ruint 808 | −227 (0.72×) | 286 vs ruint 309 | −23.0 (0.93×) | 687 vs ruint 896 | −209 (0.77×) | 357 vs ruint 383 | −26.0 (0.93×) |
| mulmod | 384 | 1,024 vs ruint 1,581 | −557 (0.65×) | 511 vs ruint 541 | −30.0 (0.94×) | 1,377 vs ruint 1,692 | −315 (0.81×) | 653 vs ruint 687 | −34.0 (0.95×) |
| mulmod | 512 | 1,704 vs ruint 2,780 | −1,076 (0.61×) | 845 vs ruint 862 | −17.0 (0.98×) | 2,098 vs num-bigint 3,122 | −1,024 (0.67×) | 1,165 vs ruint 1,114 | +51.0 (1.05×) |
| mulmod | 1024 | 5,323 vs num-bigint 8,904 | −3,581 (0.60×) | 2,758 vs ruint 2,608 | +150 (1.06×) | 6,673 vs num-bigint 9,445 | −2,772 (0.71×) | 4,235 vs ruint 3,562 | +673 (1.19×) |
| mulmod | 2048 | 17,558 vs num-bigint 32,006 | −14,448 (0.55×) | 9,462 vs ruint 10,689 | −1,227 (0.89×) | 22,162 vs num-bigint 32,421 | −10,259 (0.68×) | 12,782 vs ruint 13,733 | −951 (0.93×) |
| mulmod | 4096 | 55,999 vs num-bigint 113,268 | −57,269 (0.49×) | 29,638 vs ruint 41,295 | −11,657 (0.72×) | 68,024 vs num-bigint 115,812 | −47,788 (0.59×) | 43,084 vs ruint 53,124 | −10,040 (0.81×) |
| add_mod | 64 | 4.0 vs ruint 44.3 | −40.3 (0.09×) | 4.0 vs ruint 43.5 | −39.5 (0.09×) | 7.1 vs ruint 82.6 | −75.5 (0.09×) | 7.0 vs ruint 80.5 | −73.5 (0.09×) |
| add_mod | 128 | 9.1 vs ruint 22.9 | −13.8 (0.40×) | 9.1 vs ruint 21.7 | −12.6 (0.42×) | 11.2 vs ruint 22.7 | −11.5 (0.49×) | 14.0 vs ruint 25.0 | −11.0 (0.56×) |
| add_mod | 256 | 21.9 vs ruint 43.1 | −21.2 (0.51×) | 21.9 vs ruint 44.1 | −22.2 (0.50×) | 24.5 vs ruint 59.0 | −34.5 (0.42×) | 30.5 vs ruint 59.0 | −28.5 (0.52×) |
| add_mod | 384 | 30.6 vs ruint 62.3 | −31.7 (0.49×) | 28.0 vs ruint 60.4 | −32.4 (0.46×) | 47.0 vs ruint 95.7 | −48.7 (0.49×) | 46.3 vs ruint 83.7 | −37.4 (0.55×) |
| add_mod | 512 | 46.6 vs ruint 90.7 | −44.1 (0.51×) | 40.5 vs ruint 85.8 | −45.3 (0.47×) | 73.3 vs ruint 121 | −47.7 (0.61×) | 59.8 vs ruint 121 | −61.2 (0.49×) |
| add_mod | 1024 | 150 vs ruint 200 | −50.0 (0.75×) | 104 vs ruint 184 | −80.0 (0.57×) | 187 vs ruint 263 | −76.0 (0.71×) | 130 vs ruint 247 | −117 (0.53×) |
| add_mod | 2048 | 337 vs ruint 787 | −450 (0.43×) | 243 vs ruint 745 | −502 (0.33×) | 466 vs num-bigint 1,005 | −539 (0.46×) | 312 vs ruint 873 | −561 (0.36×) |
| add_mod | 4096 | 830 vs ruint 1,529 | −699 (0.54×) | 671 vs ruint 1,398 | −727 (0.48×) | 846 vs ruint 1,869 | −1,023 (0.45×) | 864 vs ruint 1,656 | −792 (0.52×) |
| sub_mod | 64 | 4.0 vs ruint 53.4 | −49.4 (0.07×) | 4.0 vs ruint 50.4 | −46.4 (0.08×) | 7.0 vs ruint 79.4 | −72.4 (0.09×) | 7.6 vs ruint 78.8 | −71.2 (0.10×) |
| sub_mod | 128 | 10.5 vs ruint 31.1 | −20.6 (0.34×) | 10.2 vs ruint 30.1 | −19.9 (0.34×) | 10.7 vs ruint 37.7 | −27.0 (0.28×) | 15.7 vs ruint 38.0 | −22.3 (0.41×) |
| sub_mod | 256 | 23.1 vs ruint 60.1 | −37.0 (0.38×) | 19.8 vs ruint 61.5 | −41.7 (0.32×) | 25.1 vs ruint 85.7 | −60.6 (0.29×) | 28.7 vs ruint 85.5 | −56.8 (0.34×) |
| sub_mod | 384 | 31.1 vs ruint 86.1 | −55.0 (0.36×) | 26.4 vs ruint 84.2 | −57.8 (0.31×) | 48.0 vs ruint 131 | −83.0 (0.37×) | 47.1 vs ruint 122 | −74.9 (0.39×) |
| sub_mod | 512 | 49.2 vs ruint 120 | −70.8 (0.41×) | 37.8 vs ruint 114 | −76.2 (0.33×) | 55.1 vs ruint 176 | −121 (0.31×) | 66.6 vs ruint 176 | −109 (0.38×) |
| sub_mod | 1024 | 135 vs ruint 353 | −218 (0.38×) | 109 vs ruint 329 | −220 (0.33×) | 141 vs ruint 566 | −425 (0.25×) | 133 vs ruint 550 | −417 (0.24×) |
| sub_mod | 2048 | 293 vs ruint 1,109 | −816 (0.26×) | 249 vs ruint 1,069 | −820 (0.23×) | 358 vs num-bigint 1,355 | −997 (0.26×) | 280 vs ruint 1,289 | −1,009 (0.22×) |
| sub_mod | 4096 | 796 vs ruint 2,118 | −1,322 (0.38×) | 709 vs ruint 1,991 | −1,282 (0.36×) | 811 vs ruint 2,422 | −1,611 (0.33×) | 809 vs ruint 2,269 | −1,460 (0.36×) |
| add_mod, reduced operands | 64 | 4.2 vs native 3.4 | +0.8 (1.24×) | 4.1 vs native 4.0 | +0.1 (1.02×) | 4.4 vs native 3.7 | +0.7 (1.19×) | 4.4 vs native 5.0 | −0.6 (0.88×) |
| add_mod, reduced operands | 128 | 7.9 vs native 9.6 | −1.7 (0.82×) | 9.2 vs native 9.0 | +0.2 (1.02×) | 7.7 vs native 9.5 | −1.8 (0.81×) | 11.6 vs native 9.1 | +2.5 (1.27×) |
| add_mod, reduced operands | 256 | 17.3 vs uint 22.0 | −4.7 (0.79×) | 15.6 vs uint 21.3 | −5.7 (0.73×) | 17.2 vs uint 19.6 | −2.4 (0.88×) | 22.0 vs uint 24.7 | −2.7 (0.89×) |
| add_mod, reduced operands | 384 | 28.2 vs uint 33.5 | −5.3 (0.84×) | 22.1 vs uint 37.0 | −14.9 (0.60×) | 28.2 vs uint 31.1 | −2.9 (0.91×) | 32.2 vs uint 39.2 | −7.0 (0.82×) |
| add_mod, reduced operands | 512 | 41.5 vs uint 42.5 | −1.0 (0.98×) | 28.7 vs uint 41.6 | −12.9 (0.69×) | 49.1 vs uint 44.9 | +4.2 (1.09×) | 43.3 vs uint 60.0 | −16.7 (0.72×) |
| add_mod, reduced operands | 1024 | 134 vs uint 101 | +33.0 (1.33×) | 67.1 vs uint 103 | −35.9 (0.65×) | 146 vs uint 138 | +8.0 (1.06×) | 93.5 vs uint 129 | −35.5 (0.72×) |
| add_mod, reduced operands | 2048 | 255 vs uint 233 | +22.0 (1.09×) | 147 vs uint 227 | −80.0 (0.65×) | 275 vs uint 267 | +8.0 (1.03×) | 221 vs uint 266 | −45.0 (0.83×) |
| add_mod, reduced operands | 4096 | 624 vs uint 575 | +49.0 (1.09×) | 479 vs uint 557 | −78.0 (0.86×) | 653 vs uint 601 | +52.0 (1.09×) | 655 vs uint 667 | −12.0 (0.98×) |
| sub_mod, reduced operands | 64 | 3.4 vs native 3.0 | +0.4 (1.13×) | 4.0 vs native 3.0 | +1.0 (1.33×) | 4.7 vs native 3.0 | +1.7 (1.57×) | 5.3 vs native 3.0 | +2.3 (1.77×) |
| sub_mod, reduced operands | 128 | 6.8 vs native 6.7 | +0.1 (1.01×) | 7.3 vs native 5.3 | +2.0 (1.38×) | 7.3 vs native 7.4 | −0.1 (0.99×) | 9.5 vs native 7.6 | +1.9 (1.25×) |
| sub_mod, reduced operands | 256 | 16.9 vs bnum 18.7 | −1.8 (0.90×) | 13.3 vs bnum 16.5 | −3.2 (0.81×) | 14.9 vs bnum 19.0 | −4.1 (0.78×) | 20.6 vs bnum 21.9 | −1.3 (0.94×) |
| sub_mod, reduced operands | 384 | 26.2 vs uint 30.6 | −4.4 (0.86×) | 20.7 vs uint 31.4 | −10.7 (0.66×) | 25.4 vs uint 41.5 | −16.1 (0.61×) | 31.1 vs uint 38.7 | −7.6 (0.80×) |
| sub_mod, reduced operands | 512 | 37.0 vs bnum 46.6 | −9.6 (0.79×) | 29.6 vs bnum 44.8 | −15.2 (0.66×) | 36.9 vs bnum 61.3 | −24.4 (0.60×) | 42.6 vs bnum 65.4 | −22.8 (0.65×) |
| sub_mod, reduced operands | 1024 | 96.3 vs uint 132 | −35.7 (0.73×) | 69.9 vs uint 126 | −56.1 (0.55×) | 95.7 vs uint 145 | −49.3 (0.66×) | 92.2 vs uint 159 | −66.8 (0.58×) |
| sub_mod, reduced operands | 2048 | 240 vs uint 274 | −34.0 (0.88×) | 163 vs uint 281 | −118 (0.58×) | 323 vs uint 366 | −43.0 (0.88×) | 206 vs uint 368 | −162 (0.56×) |
| sub_mod, reduced operands | 4096 | 585 vs uint 644 | −59.0 (0.91×) | 512 vs uint 678 | −166 (0.76×) | 593 vs uint 707 | −114 (0.84×) | 649 vs uint 719 | −70.0 (0.90×) |
| add, constant time | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add, constant time | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.3 vs native 3.4 | +0.9 (1.26×) |
| add, constant time | 256 | 10.3 vs crypto-bigint 10.2 | +0.1 (1.01×) | 8.6 vs ruint 9.0 | −0.4 (0.96×) | 10.1 vs ruint 10.1 | +0.0 (1.00×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add, constant time | 384 | 16.4 vs ruint 16.4 | +0.0 (1.00×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 16.0 vs ruint 16.0 | +0.0 (1.00×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 27.4 vs bnum 24.7 | +2.7 (1.11×) | 20.8 vs crypto-bigint 20.7 | +0.1 (1.00×) | 26.9 vs bnum 23.6 | +3.3 (1.14×) | 32.1 vs bnum 27.0 | +5.1 (1.19×) |
| add, constant time | 1024 | 64.0 vs bnum 61.6 | +2.4 (1.04×) | 46.2 vs crypto-bigint 45.9 | +0.3 (1.01×) | 82.4 vs bnum 67.8 | +14.6 (1.22×) | 68.9 vs crypto-bigint 68.3 | +0.6 (1.01×) |
| add, constant time | 2048 | 160 vs crypto-bigint 192 | −32.0 (0.83×) | 113 vs crypto-bigint 138 | −25.0 (0.82×) | 188 vs crypto-bigint 206 | −18.0 (0.91×) | 147 vs crypto-bigint 177 | −30.0 (0.83×) |
| add, constant time | 4096 | 426 vs crypto-bigint 390 | +36.0 (1.09×) | 321 vs crypto-bigint 263 | +58.0 (1.22×) | 464 vs bnum 386 | +78.0 (1.20×) | 403 vs crypto-bigint 325 | +78.0 (1.24×) |
| mul, constant time | 64 | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.2 | +2.3 (1.55×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.7 vs crypto-bigint 5.0 | +0.7 (1.14×) |
| mul, constant time | 256 | 48.0 vs ruint 66.5 | −18.5 (0.72×) | 16.7 vs ruint 16.8 | −0.1 (0.99×) | 67.6 vs uint 81.3 | −13.7 (0.83×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul, constant time | 384 | 186 vs ruint 166 | +20.0 (1.12×) | 52.6 vs crypto-bigint 39.6 | +13.0 (1.33×) | 227 vs ruint 221 | +6.0 (1.03×) | 72.8 vs crypto-bigint 71.9 | +0.9 (1.01×) |
| mul, constant time | 512 | 304 vs ruint 321 | −17.0 (0.95×) | 74.6 vs ruint 80.7 | −6.1 (0.92×) | 352 vs ruint 443 | −91.0 (0.79×) | 131 vs ruint 131 | +0.0 (1.00×) |
| mul, constant time | 1024 | 1,346 vs bnum 1,544 | −198 (0.87×) | 695 vs uint 503 | +192 (1.38×) | 1,574 vs bnum 2,354 | −780 (0.67×) | 862 vs crypto-bigint 932 | −70.0 (0.92×) |
| mul, constant time | 2048 | 5,796 vs bnum 6,283 | −487 (0.92×) | 2,594 vs bnum 2,601 | −7.0 (1.00×) | 6,791 vs bnum 8,683 | −1,892 (0.78×) | 3,884 vs bnum 3,418 | +466 (1.14×) |
| mul, constant time | 4096 | 18,389 vs bnum 23,558 | −5,169 (0.78×) | 8,883 vs bnum 10,097 | −1,214 (0.88×) | 21,912 vs bnum 34,809 | −12,897 (0.63×) | 12,827 vs bnum 12,899 | −72.0 (0.99×) |
| div, constant time | 64 | 84.3 vs crypto-bigint 397 | −313 (0.21×) | 47.3 vs crypto-bigint 320 | −273 (0.15×) | 145 vs crypto-bigint 427 | −282 (0.34×) | 61.8 vs crypto-bigint 340 | −278 (0.18×) |
| div, constant time | 128 | 254 vs crypto-bigint 898 | −644 (0.28×) | 145 vs crypto-bigint 749 | −604 (0.19×) | 360 vs crypto-bigint 1,079 | −719 (0.33×) | 231 vs crypto-bigint 896 | −665 (0.26×) |
| div, constant time | 256 | 505 vs crypto-bigint 1,685 | −1,180 (0.30×) | 245 vs crypto-bigint 1,361 | −1,116 (0.18×) | 612 vs crypto-bigint 1,975 | −1,363 (0.31×) | 473 vs crypto-bigint 1,525 | −1,052 (0.31×) |
| div, constant time | 384 | 503 vs crypto-bigint 2,499 | −1,996 (0.20×) | 474 vs crypto-bigint 2,015 | −1,541 (0.24×) | 808 vs crypto-bigint 3,039 | −2,231 (0.27×) | 399 vs crypto-bigint 2,279 | −1,880 (0.18×) |
| div, constant time | 512 | 1,011 vs crypto-bigint 3,512 | −2,501 (0.29×) | 712 vs crypto-bigint 2,754 | −2,042 (0.26×) | 1,165 vs crypto-bigint 4,272 | −3,107 (0.27×) | 858 vs crypto-bigint 3,099 | −2,241 (0.28×) |
| div, constant time | 1024 | 2,466 vs crypto-bigint 8,470 | −6,004 (0.29×) | 1,782 vs crypto-bigint 7,021 | −5,239 (0.25×) | 2,830 vs crypto-bigint 10,371 | −7,541 (0.27×) | 2,189 vs crypto-bigint 7,460 | −5,271 (0.29×) |
| div, constant time | 2048 | 7,321 vs crypto-bigint 22,490 | −15,169 (0.33×) | 5,079 vs crypto-bigint 16,920 | −11,841 (0.30×) | 8,613 vs crypto-bigint 27,481 | −18,868 (0.31×) | 6,353 vs crypto-bigint 18,615 | −12,262 (0.34×) |
| div, constant time | 4096 | 25,665 vs crypto-bigint 67,185 | −41,520 (0.38×) | 16,331 vs crypto-bigint 43,617 | −27,286 (0.37×) | 27,610 vs crypto-bigint 85,427 | −57,817 (0.32×) | 21,492 vs crypto-bigint 50,596 | −29,104 (0.42×) |
| mod, constant time | 64 | 83.1 vs crypto-bigint 384 | −301 (0.22×) | 48.8 vs crypto-bigint 325 | −276 (0.15×) | 152 vs crypto-bigint 420 | −268 (0.36×) | 67.6 vs crypto-bigint 334 | −266 (0.20×) |
| mod, constant time | 128 | 206 vs crypto-bigint 910 | −704 (0.23×) | 152 vs crypto-bigint 744 | −592 (0.20×) | 334 vs crypto-bigint 1,053 | −719 (0.32×) | 226 vs crypto-bigint 867 | −641 (0.26×) |
| mod, constant time | 256 | 530 vs crypto-bigint 1,658 | −1,128 (0.32×) | 331 vs crypto-bigint 1,340 | −1,009 (0.25×) | 568 vs crypto-bigint 1,958 | −1,390 (0.29×) | 414 vs crypto-bigint 1,519 | −1,105 (0.27×) |
| mod, constant time | 384 | 742 vs crypto-bigint 2,457 | −1,715 (0.30×) | 527 vs crypto-bigint 1,974 | −1,447 (0.27×) | 781 vs crypto-bigint 2,995 | −2,214 (0.26×) | 563 vs crypto-bigint 2,213 | −1,650 (0.25×) |
| mod, constant time | 512 | 1,041 vs crypto-bigint 3,607 | −2,566 (0.29×) | 756 vs crypto-bigint 2,755 | −1,999 (0.27×) | 1,097 vs crypto-bigint 4,285 | −3,188 (0.26×) | 808 vs crypto-bigint 3,108 | −2,300 (0.26×) |
| mod, constant time | 1024 | 2,510 vs crypto-bigint 8,459 | −5,949 (0.30×) | 1,783 vs crypto-bigint 7,010 | −5,227 (0.25×) | 2,730 vs crypto-bigint 10,311 | −7,581 (0.26×) | 1,992 vs crypto-bigint 7,418 | −5,426 (0.27×) |
| mod, constant time | 2048 | 7,618 vs crypto-bigint 22,444 | −14,826 (0.34×) | 5,084 vs crypto-bigint 16,861 | −11,777 (0.30×) | 8,188 vs crypto-bigint 27,334 | −19,146 (0.30×) | 5,939 vs crypto-bigint 18,466 | −12,527 (0.32×) |
| mod, constant time | 4096 | 26,068 vs crypto-bigint 67,384 | −41,316 (0.39×) | 16,355 vs crypto-bigint 43,375 | −27,020 (0.38×) | 28,822 vs crypto-bigint 83,576 | −54,754 (0.34×) | 20,303 vs crypto-bigint 50,555 | −30,252 (0.40×) |
| mulmod, constant time | 64 | 202 vs crypto-bigint 467 | −265 (0.43×) | 105 vs crypto-bigint 351 | −246 (0.30×) | 236 vs crypto-bigint 508 | −272 (0.46×) | 105 vs crypto-bigint 380 | −275 (0.28×) |
| mulmod, constant time | 128 | 562 vs crypto-bigint 1,295 | −733 (0.43×) | 361 vs crypto-bigint 996 | −635 (0.36×) | 607 vs crypto-bigint 1,536 | −929 (0.40×) | 376 vs crypto-bigint 1,147 | −771 (0.33×) |
| mulmod, constant time | 256 | 1,215 vs crypto-bigint 2,870 | −1,655 (0.42×) | 735 vs crypto-bigint 1,978 | −1,243 (0.37×) | 1,326 vs crypto-bigint 3,282 | −1,956 (0.40×) | 816 vs crypto-bigint 2,280 | −1,464 (0.36×) |
| mulmod, constant time | 384 | 1,983 vs crypto-bigint 4,858 | −2,875 (0.41×) | 1,203 vs crypto-bigint 3,245 | −2,042 (0.37×) | 2,309 vs crypto-bigint 5,694 | −3,385 (0.41×) | 1,511 vs crypto-bigint 3,714 | −2,203 (0.41×) |
| mulmod, constant time | 512 | 3,048 vs crypto-bigint 7,630 | −4,582 (0.40×) | 1,830 vs crypto-bigint 4,669 | −2,839 (0.39×) | 3,521 vs crypto-bigint 8,707 | −5,186 (0.40×) | 2,385 vs crypto-bigint 5,529 | −3,144 (0.43×) |
| mulmod, constant time | 1024 | 9,561 vs crypto-bigint 21,427 | −11,866 (0.45×) | 5,958 vs crypto-bigint 13,281 | −7,323 (0.45×) | 10,777 vs crypto-bigint 25,214 | −14,437 (0.43×) | 7,712 vs crypto-bigint 14,115 | −6,403 (0.55×) |
| mulmod, constant time | 2048 | 32,883 vs crypto-bigint 65,327 | −32,444 (0.50×) | 19,593 vs crypto-bigint 36,125 | −16,532 (0.54×) | 36,440 vs crypto-bigint 76,569 | −40,129 (0.48×) | 25,871 vs crypto-bigint 45,277 | −19,406 (0.57×) |
| mulmod, constant time | 4096 | 119,413 vs crypto-bigint 260,000 | −140,587 (0.46×) | 70,114 vs crypto-bigint 110,652 | −40,538 (0.63×) | 134,990 vs crypto-bigint 257,537 | −122,547 (0.52×) | 88,804 vs crypto-bigint 143,814 | −55,010 (0.62×) |
| add_mod, constant time | 64 | 9.1 | – | 9.0 | – | 18.4 | – | 13.8 | – |
| add_mod, constant time | 128 | 26.5 | – | 21.6 | – | 34.4 | – | 34.1 | – |
| add_mod, constant time | 256 | 82.0 | – | 65.0 | – | 108 | – | 83.5 | – |
| add_mod, constant time | 384 | 120 | – | 110 | – | 166 | – | 140 | – |
| add_mod, constant time | 512 | 153 | – | 145 | – | 247 | – | 179 | – |
| add_mod, constant time | 1024 | 356 | – | 270 | – | 627 | – | 343 | – |
| add_mod, constant time | 2048 | 865 | – | 530 | – | 1,675 | – | 736 | – |
| add_mod, constant time | 4096 | 1,870 | – | 1,719 | – | 1,957 | – | 1,973 | – |
| sub_mod, constant time | 64 | 10.2 | – | 9.3 | – | 18.3 | – | 14.2 | – |
| sub_mod, constant time | 128 | 15.5 | – | 13.0 | – | 29.7 | – | 23.0 | – |
| sub_mod, constant time | 256 | 45.8 | – | 32.2 | – | 73.8 | – | 66.7 | – |
| sub_mod, constant time | 384 | 102 | – | 51.2 | – | 135 | – | 98.5 | – |
| sub_mod, constant time | 512 | 146 | – | 88.1 | – | 219 | – | 141 | – |
| sub_mod, constant time | 1024 | 321 | – | 210 | – | 518 | – | 273 | – |
| sub_mod, constant time | 2048 | 727 | – | 510 | – | 1,331 | – | 635 | – |
| sub_mod, constant time | 4096 | 1,529 | – | 1,429 | – | 1,694 | – | 1,679 | – |
| add_mod, reduced operands, constant time | 64 | 5.8 vs crypto-bigint 13.0 | −7.2 (0.45×) | 6.1 vs crypto-bigint 12.2 | −6.1 (0.50×) | 6.0 vs crypto-bigint 11.9 | −5.9 (0.50×) | 7.4 vs crypto-bigint 13.2 | −5.8 (0.56×) |
| add_mod, reduced operands, constant time | 128 | 13.6 vs crypto-bigint 23.5 | −9.9 (0.58×) | 11.8 vs crypto-bigint 21.1 | −9.3 (0.56×) | 13.6 vs crypto-bigint 23.1 | −9.5 (0.59×) | 18.0 vs crypto-bigint 22.8 | −4.8 (0.79×) |
| add_mod, reduced operands, constant time | 256 | 29.1 vs crypto-bigint 47.5 | −18.4 (0.61×) | 23.6 vs crypto-bigint 40.9 | −17.3 (0.58×) | 31.4 vs crypto-bigint 57.5 | −26.1 (0.55×) | 32.1 vs crypto-bigint 54.8 | −22.7 (0.59×) |
| add_mod, reduced operands, constant time | 384 | 47.5 vs crypto-bigint 96.8 | −49.3 (0.49×) | 37.0 vs crypto-bigint 105 | −68.0 (0.35×) | 50.5 vs crypto-bigint 87.0 | −36.5 (0.58×) | 47.1 vs crypto-bigint 82.7 | −35.6 (0.57×) |
| add_mod, reduced operands, constant time | 512 | 81.3 vs crypto-bigint 108 | −26.7 (0.75×) | 49.7 vs crypto-bigint 93.9 | −44.2 (0.53×) | 85.9 vs crypto-bigint 141 | −55.1 (0.61×) | 62.7 vs crypto-bigint 128 | −65.3 (0.49×) |
| add_mod, reduced operands, constant time | 1024 | 169 vs crypto-bigint 377 | −208 (0.45×) | 95.6 vs crypto-bigint 343 | −247 (0.28×) | 335 vs crypto-bigint 463 | −128 (0.72×) | 127 vs crypto-bigint 384 | −257 (0.33×) |
| add_mod, reduced operands, constant time | 2048 | 381 vs crypto-bigint 580 | −199 (0.66×) | 182 vs crypto-bigint 503 | −321 (0.36×) | 738 vs crypto-bigint 881 | −143 (0.84×) | 290 vs crypto-bigint 772 | −482 (0.38×) |
| add_mod, reduced operands, constant time | 4096 | 876 vs crypto-bigint 1,178 | −302 (0.74×) | 742 vs crypto-bigint 1,026 | −284 (0.72×) | 898 vs crypto-bigint 1,599 | −701 (0.56×) | 931 vs crypto-bigint 1,442 | −511 (0.65×) |
| sub_mod, reduced operands, constant time | 64 | 4.8 vs crypto-bigint 5.0 | −0.2 (0.96×) | 4.9 vs crypto-bigint 5.0 | −0.1 (0.98×) | 4.7 vs crypto-bigint 5.0 | −0.3 (0.94×) | 5.0 vs crypto-bigint 5.0 | +0.0 (1.00×) |
| sub_mod, reduced operands, constant time | 128 | 9.6 vs crypto-bigint 10.2 | −0.6 (0.94×) | 9.0 vs crypto-bigint 11.0 | −2.0 (0.82×) | 9.0 vs crypto-bigint 10.6 | −1.6 (0.85×) | 12.0 vs crypto-bigint 11.0 | +1.0 (1.09×) |
| sub_mod, reduced operands, constant time | 256 | 24.3 vs crypto-bigint 28.1 | −3.8 (0.86×) | 20.2 vs crypto-bigint 23.6 | −3.4 (0.86×) | 31.0 vs crypto-bigint 31.6 | −0.6 (0.98×) | 28.1 vs crypto-bigint 30.1 | −2.0 (0.93×) |
| sub_mod, reduced operands, constant time | 384 | 40.5 vs crypto-bigint 48.9 | −8.4 (0.83×) | 33.3 vs crypto-bigint 39.0 | −5.7 (0.85×) | 66.3 vs crypto-bigint 61.7 | +4.6 (1.07×) | 44.5 vs crypto-bigint 52.3 | −7.8 (0.85×) |
| sub_mod, reduced operands, constant time | 512 | 66.1 vs crypto-bigint 71.2 | −5.1 (0.93×) | 46.5 vs crypto-bigint 53.6 | −7.1 (0.87×) | 97.1 vs crypto-bigint 107 | −9.9 (0.91×) | 66.6 vs crypto-bigint 105 | −38.4 (0.63×) |
| sub_mod, reduced operands, constant time | 1024 | 148 vs crypto-bigint 178 | −30.0 (0.83×) | 114 vs crypto-bigint 139 | −25.0 (0.82×) | 218 vs crypto-bigint 345 | −127 (0.63×) | 137 vs crypto-bigint 276 | −139 (0.50×) |
| sub_mod, reduced operands, constant time | 2048 | 354 vs crypto-bigint 519 | −165 (0.68×) | 254 vs crypto-bigint 456 | −202 (0.56×) | 605 vs crypto-bigint 564 | +41.0 (1.07×) | 293 vs crypto-bigint 506 | −213 (0.58×) |
| sub_mod, reduced operands, constant time | 4096 | 864 vs crypto-bigint 903 | −39.0 (0.96×) | 741 vs crypto-bigint 747 | −6.0 (0.99×) | 992 vs crypto-bigint 1,465 | −473 (0.68×) | 926 vs crypto-bigint 1,315 | −389 (0.70×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs crypto-bigint 1.0 | +0.1 (1.10×) | 1.1 vs native 1.0 | +0.1 (1.10×) |
| add | 128 | 2.9 vs native 2.8 | +0.1 (1.04×) | 2.5 vs bnum 2.0 | +0.5 (1.25×) | 2.7 vs crypto-bigint 2.7 | +0.0 (1.00×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add | 256 | 9.8 vs bnum 8.1 | +1.7 (1.21×) | 6.8 vs bnum 6.3 | +0.5 (1.08×) | 8.9 vs bnum 8.7 | +0.2 (1.02×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add | 384 | 15.3 vs ruint 15.4 | −0.1 (0.99×) | 12.2 vs crypto-bigint 12.3 | −0.1 (0.99×) | 15.3 vs ruint 15.5 | −0.2 (0.99×) | 18.1 vs crypto-bigint 18.0 | +0.1 (1.01×) |
| add | 512 | 26.0 vs bnum 22.6 | +3.4 (1.15×) | 17.6 vs crypto-bigint 18.2 | −0.6 (0.97×) | 25.8 vs bnum 22.6 | +3.2 (1.14×) | 31.2 vs bnum 28.1 | +3.1 (1.11×) |
| add | 1024 | 65.7 vs bnum 62.0 | +3.7 (1.06×) | 45.1 vs crypto-bigint 44.2 | +0.9 (1.02×) | 69.0 vs ruint 72.6 | −3.6 (0.95×) | 67.2 vs crypto-bigint 61.3 | +5.9 (1.10×) |
| add | 2048 | 162 vs ruint 202 | −40.0 (0.80×) | 108 vs ruint 158 | −50.0 (0.68×) | 179 vs ruint 216 | −37.0 (0.83×) | 142 vs ruint 206 | −64.0 (0.69×) |
| add | 4096 | 419 vs ruint 407 | +12.0 (1.03×) | 308 vs ruint 292 | +16.0 (1.05×) | 438 vs ruint 424 | +14.0 (1.03×) | 400 vs ruint 380 | +20.0 (1.05×) |
| mulmod | 64 | 68.5 vs ruint 85.2 | −16.7 (0.80×) | 35.1 vs ruint 42.9 | −7.8 (0.82×) | 110 vs ruint 179 | −69.0 (0.61×) | 42.0 vs ruint 55.0 | −13.0 (0.76×) |
| mulmod | 128 | 210 vs ruint 338 | −128 (0.62×) | 85.9 vs ruint 137 | −51.1 (0.63×) | 261 vs ruint 387 | −126 (0.67×) | 141 vs ruint 207 | −66.0 (0.68×) |
| mulmod | 256 | 578 vs ruint 810 | −232 (0.71×) | 283 vs ruint 309 | −26.0 (0.92×) | 688 vs ruint 890 | −202 (0.77×) | 358 vs ruint 382 | −24.0 (0.94×) |
| mulmod | 384 | 1,050 vs ruint 1,609 | −559 (0.65×) | 489 vs ruint 538 | −49.0 (0.91×) | 1,350 vs ruint 1,695 | −345 (0.80×) | 651 vs ruint 683 | −32.0 (0.95×) |
| mulmod | 512 | 1,665 vs ruint 2,769 | −1,104 (0.60×) | 858 vs ruint 855 | +3.0 (1.00×) | 2,050 vs ruint 3,258 | −1,208 (0.63×) | 1,111 vs ruint 1,117 | −6.0 (0.99×) |
| mulmod | 1024 | 5,298 vs ruint 10,785 | −5,487 (0.49×) | 2,759 vs ruint 2,603 | +156 (1.06×) | 6,821 vs ruint 12,630 | −5,809 (0.54×) | 4,170 vs ruint 3,570 | +600 (1.17×) |
| mulmod | 2048 | 17,555 vs ruint 36,366 | −18,811 (0.48×) | 9,487 vs ruint 10,683 | −1,196 (0.89×) | 22,223 vs ruint 43,728 | −21,505 (0.51×) | 12,807 vs ruint 13,735 | −928 (0.93×) |
| mulmod | 4096 | 55,918 vs ruint 161,365 | −105,447 (0.35×) | 29,652 vs ruint 41,187 | −11,535 (0.72×) | 68,405 vs ruint 170,058 | −101,653 (0.40×) | 43,035 vs ruint 53,172 | −10,137 (0.81×) |
| add_mod | 64 | 6.3 vs ruint 41.0 | −34.7 (0.15×) | 6.2 vs ruint 43.2 | −37.0 (0.14×) | 8.6 vs ruint 77.0 | −68.4 (0.11×) | 9.0 vs ruint 77.9 | −68.9 (0.12×) |
| add_mod | 128 | 31.3 vs ruint 41.1 | −9.8 (0.76×) | 10.4 vs ruint 31.0 | −20.6 (0.34×) | 41.7 vs ruint 48.2 | −6.5 (0.87×) | 16.4 vs ruint 40.2 | −23.8 (0.41×) |
| add_mod | 256 | 43.2 vs ruint 77.5 | −34.3 (0.56×) | 30.9 vs ruint 63.4 | −32.5 (0.49×) | 54.5 vs ruint 91.5 | −37.0 (0.60×) | 43.8 vs ruint 78.6 | −34.8 (0.56×) |
| add_mod | 384 | 56.7 vs ruint 103 | −46.3 (0.55×) | 42.3 vs ruint 74.7 | −32.4 (0.57×) | 79.8 vs ruint 141 | −61.2 (0.57×) | 66.1 vs ruint 103 | −36.9 (0.64×) |
| add_mod | 512 | 69.8 vs ruint 136 | −66.2 (0.51×) | 53.4 vs ruint 103 | −49.6 (0.52×) | 97.6 vs ruint 170 | −72.4 (0.57×) | 81.5 vs ruint 137 | −55.5 (0.59×) |
| add_mod | 1024 | 161 vs ruint 266 | −105 (0.61×) | 119 vs ruint 206 | −87.0 (0.58×) | 213 vs ruint 336 | −123 (0.63×) | 158 vs ruint 301 | −143 (0.52×) |
| add_mod | 2048 | 339 vs ruint 919 | −580 (0.37×) | 237 vs ruint 773 | −536 (0.31×) | 581 vs ruint 1,137 | −556 (0.51×) | 324 vs ruint 947 | −623 (0.34×) |
| add_mod | 4096 | 895 vs ruint 1,673 | −778 (0.53×) | 797 vs ruint 1,430 | −633 (0.56×) | 997 vs ruint 1,994 | −997 (0.50×) | 948 vs ruint 1,686 | −738 (0.56×) |
| sub_mod | 64 | 4.7 vs ruint 47.7 | −43.0 (0.10×) | 7.1 vs ruint 46.7 | −39.6 (0.15×) | 8.3 vs ruint 76.2 | −67.9 (0.11×) | 9.9 vs ruint 76.4 | −66.5 (0.13×) |
| sub_mod | 128 | 35.0 vs ruint 45.6 | −10.6 (0.77×) | 11.0 vs ruint 40.2 | −29.2 (0.27×) | 47.3 vs ruint 64.5 | −17.2 (0.73×) | 15.6 vs ruint 52.3 | −36.7 (0.30×) |
| sub_mod | 256 | 43.4 vs ruint 96.5 | −53.1 (0.45×) | 32.3 vs ruint 79.7 | −47.4 (0.41×) | 61.3 vs ruint 131 | −69.7 (0.47×) | 46.2 vs ruint 115 | −68.8 (0.40×) |
| sub_mod | 384 | 58.5 vs ruint 130 | −71.5 (0.45×) | 41.8 vs ruint 109 | −67.2 (0.38×) | 79.1 vs ruint 187 | −108 (0.42×) | 66.7 vs ruint 152 | −85.3 (0.44×) |
| sub_mod | 512 | 64.7 vs ruint 162 | −97.3 (0.40×) | 53.8 vs ruint 140 | −86.2 (0.38×) | 96.2 vs ruint 234 | −138 (0.41×) | 89.1 vs ruint 205 | −116 (0.43×) |
| sub_mod | 1024 | 164 vs ruint 421 | −257 (0.39×) | 126 vs ruint 350 | −224 (0.36×) | 197 vs ruint 641 | −444 (0.31×) | 170 vs ruint 610 | −440 (0.28×) |
| sub_mod | 2048 | 364 vs ruint 1,216 | −852 (0.30×) | 263 vs ruint 1,112 | −849 (0.24×) | 417 vs ruint 1,467 | −1,050 (0.28×) | 364 vs ruint 1,384 | −1,020 (0.26×) |
| sub_mod | 4096 | 924 vs ruint 2,333 | −1,409 (0.40×) | 819 vs ruint 2,064 | −1,245 (0.40×) | 969 vs ruint 2,573 | −1,604 (0.38×) | 997 vs ruint 2,395 | −1,398 (0.42×) |
| add_mod, reduced operands | 64 | 5.0 vs ruint 44.7 | −39.7 (0.11×) | 5.0 vs ruint 45.5 | −40.5 (0.11×) | 5.0 vs ruint 84.0 | −79.0 (0.06×) | 5.0 vs ruint 83.4 | −78.4 (0.06×) |
| add_mod, reduced operands | 128 | 6.7 vs ruint 23.4 | −16.7 (0.29×) | 7.5 vs ruint 23.7 | −16.2 (0.32×) | 7.5 vs ruint 24.4 | −16.9 (0.31×) | 9.8 vs ruint 25.0 | −15.2 (0.39×) |
| add_mod, reduced operands | 256 | 16.7 vs ruint 42.8 | −26.1 (0.39×) | 13.0 vs ruint 43.3 | −30.3 (0.30×) | 16.9 vs ruint 60.1 | −43.2 (0.28×) | 22.3 vs ruint 59.1 | −36.8 (0.38×) |
| add_mod, reduced operands | 384 | 29.5 vs ruint 61.6 | −32.1 (0.48×) | 19.7 vs ruint 59.9 | −40.2 (0.33×) | 29.0 vs ruint 92.8 | −63.8 (0.31×) | 32.0 vs ruint 80.7 | −48.7 (0.40×) |
| add_mod, reduced operands | 512 | 42.5 vs ruint 86.0 | −43.5 (0.49×) | 28.1 vs ruint 79.2 | −51.1 (0.35×) | 50.8 vs ruint 112 | −61.2 (0.45×) | 43.4 vs ruint 111 | −67.6 (0.39×) |
| add_mod, reduced operands | 1024 | 115 vs ruint 195 | −80.0 (0.59×) | 68.4 vs ruint 183 | −115 (0.37×) | 148 vs ruint 258 | −110 (0.57×) | 92.0 vs ruint 246 | −154 (0.37×) |
| add_mod, reduced operands | 2048 | 247 vs ruint 794 | −547 (0.31×) | 158 vs ruint 756 | −598 (0.21×) | 276 vs ruint 1,020 | −744 (0.27×) | 208 vs ruint 898 | −690 (0.23×) |
| add_mod, reduced operands | 4096 | 620 vs ruint 1,517 | −897 (0.41×) | 484 vs ruint 1,406 | −922 (0.34×) | 619 vs ruint 1,807 | −1,188 (0.34×) | 651 vs ruint 1,636 | −985 (0.40×) |
| sub_mod, reduced operands | 64 | 2.5 vs ruint 45.1 | −42.6 (0.06×) | 2.7 vs ruint 44.3 | −41.6 (0.06×) | 3.2 vs ruint 83.4 | −80.2 (0.04×) | 3.7 vs ruint 82.3 | −78.6 (0.04×) |
| sub_mod, reduced operands | 128 | 5.4 vs ruint 25.5 | −20.1 (0.21×) | 6.1 vs ruint 24.2 | −18.1 (0.25×) | 6.1 vs ruint 26.1 | −20.0 (0.23×) | 9.2 vs ruint 27.1 | −17.9 (0.34×) |
| sub_mod, reduced operands | 256 | 15.0 vs ruint 49.7 | −34.7 (0.30×) | 11.9 vs ruint 50.5 | −38.6 (0.24×) | 13.7 vs ruint 65.5 | −51.8 (0.21×) | 19.0 vs ruint 66.4 | −47.4 (0.29×) |
| sub_mod, reduced operands | 384 | 25.0 vs ruint 77.2 | −52.2 (0.32×) | 18.9 vs ruint 75.3 | −56.4 (0.25×) | 25.4 vs ruint 103 | −77.6 (0.25×) | 30.0 vs ruint 95.1 | −65.1 (0.32×) |
| sub_mod, reduced operands | 512 | 37.0 vs ruint 109 | −72.0 (0.34×) | 28.9 vs ruint 102 | −73.1 (0.28×) | 41.3 vs ruint 132 | −90.7 (0.31×) | 41.5 vs ruint 132 | −90.5 (0.31×) |
| sub_mod, reduced operands | 1024 | 96.2 vs ruint 263 | −167 (0.37×) | 68.0 vs ruint 238 | −170 (0.29×) | 93.6 vs ruint 320 | −226 (0.29×) | 91.1 vs ruint 302 | −211 (0.30×) |
| sub_mod, reduced operands | 2048 | 242 vs ruint 982 | −740 (0.25×) | 169 vs ruint 932 | −763 (0.18×) | 292 vs ruint 1,216 | −924 (0.24×) | 204 vs ruint 1,114 | −910 (0.18×) |
| sub_mod, reduced operands | 4096 | 602 vs ruint 1,883 | −1,281 (0.32×) | 509 vs ruint 1,732 | −1,223 (0.29×) | 614 vs ruint 2,138 | −1,524 (0.29×) | 635 vs ruint 1,972 | −1,337 (0.32×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs crypto-bigint 1.0 | +0.1 (1.10×) | 1.1 vs crypto-bigint 1.0 | +0.1 (1.10×) | 1.1 vs native 1.0 | +0.1 (1.10×) |
| add, constant time | 128 | 2.9 vs native 2.8 | +0.1 (1.04×) | 2.5 vs native 2.1 | +0.4 (1.19×) | 2.7 vs crypto-bigint 2.7 | +0.0 (1.00×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add, constant time | 256 | 9.7 vs bnum 8.1 | +1.6 (1.20×) | 6.8 vs bnum 6.3 | +0.5 (1.08×) | 8.9 vs bnum 8.7 | +0.2 (1.02×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add, constant time | 384 | 15.4 vs ruint 15.4 | +0.0 (1.00×) | 12.2 vs crypto-bigint 12.4 | −0.2 (0.98×) | 15.3 vs ruint 15.5 | −0.2 (0.99×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 26.0 vs bnum 22.6 | +3.4 (1.15×) | 17.6 vs crypto-bigint 18.8 | −1.2 (0.94×) | 25.8 vs bnum 22.6 | +3.2 (1.14×) | 31.1 vs bnum 28.1 | +3.0 (1.11×) |
| add, constant time | 1024 | 65.3 vs bnum 62.1 | +3.2 (1.05×) | 44.8 vs crypto-bigint 44.6 | +0.2 (1.00×) | 68.9 vs ruint 74.2 | −5.3 (0.93×) | 67.3 vs crypto-bigint 61.4 | +5.9 (1.10×) |
| add, constant time | 2048 | 161 vs ruint 202 | −41.0 (0.80×) | 107 vs ruint 159 | −52.0 (0.67×) | 180 vs ruint 216 | −36.0 (0.83×) | 142 vs ruint 205 | −63.0 (0.69×) |
| add, constant time | 4096 | 418 vs ruint 407 | +11.0 (1.03×) | 304 vs ruint 291 | +13.0 (1.04×) | 437 vs ruint 425 | +12.0 (1.03×) | 409 vs ruint 382 | +27.0 (1.07×) |
| mulmod, constant time | 64 | 144 | – | 94.5 | – | 243 | – | 110 | – |
| mulmod, constant time | 128 | 564 | – | 350 | – | 607 | – | 368 | – |
| mulmod, constant time | 256 | 1,202 | – | 742 | – | 1,291 | – | 844 | – |
| mulmod, constant time | 384 | 2,001 | – | 1,198 | – | 2,300 | – | 1,494 | – |
| mulmod, constant time | 512 | 3,041 | – | 1,884 | – | 3,465 | – | 2,292 | – |
| mulmod, constant time | 1024 | 9,519 | – | 5,983 | – | 10,970 | – | 7,683 | – |
| mulmod, constant time | 2048 | 32,886 | – | 19,838 | – | 36,578 | – | 25,463 | – |
| mulmod, constant time | 4096 | 120,203 | – | 69,756 | – | 131,614 | – | 89,355 | – |
| add_mod, constant time | 64 | 12.3 | – | 13.0 | – | 19.7 | – | 14.3 | – |
| add_mod, constant time | 128 | 28.1 | – | 22.6 | – | 39.0 | – | 34.1 | – |
| add_mod, constant time | 256 | 83.6 | – | 66.6 | – | 98.8 | – | 89.7 | – |
| add_mod, constant time | 384 | 112 | – | 108 | – | 175 | – | 127 | – |
| add_mod, constant time | 512 | 152 | – | 135 | – | 249 | – | 167 | – |
| add_mod, constant time | 1024 | 354 | – | 261 | – | 639 | – | 349 | – |
| add_mod, constant time | 2048 | 849 | – | 515 | – | 1,599 | – | 767 | – |
| add_mod, constant time | 4096 | 1,876 | – | 1,763 | – | 1,941 | – | 1,969 | – |
| sub_mod, constant time | 64 | 12.1 | – | 11.0 | – | 19.0 | – | 14.1 | – |
| sub_mod, constant time | 128 | 24.0 | – | 19.0 | – | 31.6 | – | 23.0 | – |
| sub_mod, constant time | 256 | 54.7 | – | 36.9 | – | 75.0 | – | 57.7 | – |
| sub_mod, constant time | 384 | 102 | – | 57.8 | – | 123 | – | 85.5 | – |
| sub_mod, constant time | 512 | 143 | – | 84.3 | – | 191 | – | 117 | – |
| sub_mod, constant time | 1024 | 314 | – | 200 | – | 531 | – | 255 | – |
| sub_mod, constant time | 2048 | 730 | – | 510 | – | 1,266 | – | 582 | – |
| sub_mod, constant time | 4096 | 1,548 | – | 1,423 | – | 1,667 | – | 1,687 | – |
| add_mod, reduced operands, constant time | 64 | 8.1 | – | 8.0 | – | 8.0 | – | 8.5 | – |
| add_mod, reduced operands, constant time | 128 | 15.0 | – | 13.1 | – | 14.6 | – | 16.2 | – |
| add_mod, reduced operands, constant time | 256 | 30.8 | – | 23.0 | – | 28.5 | – | 31.1 | – |
| add_mod, reduced operands, constant time | 384 | 51.7 | – | 34.9 | – | 51.8 | – | 49.6 | – |
| add_mod, reduced operands, constant time | 512 | 69.5 | – | 49.2 | – | 94.1 | – | 66.1 | – |
| add_mod, reduced operands, constant time | 1024 | 171 | – | 91.4 | – | 331 | – | 144 | – |
| add_mod, reduced operands, constant time | 2048 | 378 | – | 183 | – | 743 | – | 299 | – |
| add_mod, reduced operands, constant time | 4096 | 883 | – | 749 | – | 893 | – | 970 | – |
| sub_mod, reduced operands, constant time | 64 | 4.0 | – | 5.0 | – | 4.0 | – | 5.1 | – |
| sub_mod, reduced operands, constant time | 128 | 10.6 | – | 10.0 | – | 10.0 | – | 10.1 | – |
| sub_mod, reduced operands, constant time | 256 | 23.1 | – | 19.1 | – | 31.0 | – | 28.0 | – |
| sub_mod, reduced operands, constant time | 384 | 41.3 | – | 32.0 | – | 55.8 | – | 43.4 | – |
| sub_mod, reduced operands, constant time | 512 | 65.2 | – | 44.6 | – | 78.7 | – | 62.6 | – |
| sub_mod, reduced operands, constant time | 1024 | 150 | – | 112 | – | 217 | – | 141 | – |
| sub_mod, reduced operands, constant time | 2048 | 346 | – | 257 | – | 637 | – | 303 | – |
| sub_mod, reduced operands, constant time | 4096 | 877 | – | 733 | – | 997 | – | 945 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.3 (1.8×) |
| add | 256 | 4.5 | 10.3 (2.3×) | 8.6 (1.9×) | 10.1 (2.2×) | 11.5 (2.6×) |
| add | 384 | 7.1 | 16.3 (2.3×) | 12.3 (1.7×) | 15.9 (2.2×) | 18.0 (2.5×) |
| add | 512 | 9.2 | 27.3 (3.0×) | 20.7 (2.2×) | 26.7 (2.9×) | 32.1 (3.5×) |
| add | 1024 | 23.7 | 64.1 (2.7×) | 46.8 (2.0×) | 82.1 (3.5×) | 68.8 (2.9×) |
| add | 2048 | 54.3 | 161 (3.0×) | 112 (2.1×) | 188 (3.5×) | 147 (2.7×) |
| add | 4096 | 147 | 427 (2.9×) | 323 (2.2×) | 467 (3.2×) | 405 (2.8×) |
| mul | 64 | 1.5 | 2.2 (1.5×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul | 128 | 3.0 | 10.0 (3.3×) | 6.5 (2.2×) | 12.3 (4.1×) | 5.7 (1.9×) |
| mul | 256 | 12.4 | 47.7 (3.8×) | 16.7 (1.3×) | 66.9 (5.4×) | 26.0 (2.1×) |
| mul | 384 | 28.0 | 186 (6.6×) | 52.6 (1.9×) | 207 (7.4×) | 72.6 (2.6×) |
| mul | 512 | 52.4 | 290 (5.5×) | 75.8 (1.4×) | 374 (7.1×) | 131 (2.5×) |
| mul | 1024 | 221 | 1,342 (6.1×) | 692 (3.1×) | 1,578 (7.1×) | 859 (3.9×) |
| mul | 2048 | 1,116 | 5,585 (5.0×) | 2,479 (2.2×) | 6,616 (5.9×) | 3,670 (3.3×) |
| mul | 4096 | 3,842 | 17,711 (4.6×) | 8,458 (2.2×) | 21,190 (5.5×) | 12,251 (3.2×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.0 | 60.3 (3.8×) | 28.4 (1.8×) | 87.0 (5.4×) | 34.7 (2.2×) |
| div | 256 | 46.0 | 203 (4.4×) | 84.3 (1.8×) | 226 (4.9×) | 110 (2.4×) |
| div | 384 | 137 | 347 (2.5×) | 206 (1.5×) | 405 (3.0×) | 279 (2.0×) |
| div | 512 | 162 | 474 (2.9×) | 308 (1.9×) | 567 (3.5×) | 377 (2.3×) |
| div | 1024 | 479 | 1,132 (2.4×) | 686 (1.4×) | 1,468 (3.1×) | 857 (1.8×) |
| div | 2048 | 1,382 | 3,223 (2.3×) | 2,005 (1.5×) | 4,288 (3.1×) | 2,379 (1.7×) |
| div | 4096 | 4,612 | 12,835 (2.8×) | 6,991 (1.5×) | 15,188 (3.3×) | 7,465 (1.6×) |
| mod | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 17.0 | 52.8 (3.1×) | 30.2 (1.8×) | 90.2 (5.3×) | 34.0 (2.0×) |
| mod | 256 | 41.5 | 207 (5.0×) | 74.3 (1.8×) | 223 (5.4×) | 108 (2.6×) |
| mod | 384 | 145 | 347 (2.4×) | 197 (1.4×) | 404 (2.8×) | 266 (1.8×) |
| mod | 512 | 203 | 472 (2.3×) | 280 (1.4×) | 558 (2.7×) | 349 (1.7×) |
| mod | 1024 | 476 | 1,195 (2.5×) | 706 (1.5×) | 1,469 (3.1×) | 939 (2.0×) |
| mod | 2048 | 1,446 | 3,315 (2.3×) | 1,927 (1.3×) | 4,312 (3.0×) | 2,085 (1.4×) |
| mod | 4096 | 4,563 | 12,879 (2.8×) | 6,628 (1.5×) | 15,104 (3.3×) | 7,686 (1.7×) |
| mulmod | 64 | 11.9 | 60.8 (5.1×) | 31.4 (2.6×) | 108 (9.1×) | 36.1 (3.0×) |
| mulmod | 128 | 47.3 | 207 (4.4×) | 81.7 (1.7×) | 239 (5.1×) | 125 (2.6×) |
| mulmod | 256 | 177 | 581 (3.3×) | 286 (1.6×) | 687 (3.9×) | 357 (2.0×) |
| mulmod | 384 | 315 | 1,024 (3.3×) | 511 (1.6×) | 1,377 (4.4×) | 653 (2.1×) |
| mulmod | 512 | 497 | 1,704 (3.4×) | 845 (1.7×) | 2,098 (4.2×) | 1,165 (2.3×) |
| mulmod | 1024 | 1,382 | 5,323 (3.9×) | 2,758 (2.0×) | 6,673 (4.8×) | 4,235 (3.1×) |
| mulmod | 2048 | 5,205 | 17,558 (3.4×) | 9,462 (1.8×) | 22,162 (4.3×) | 12,782 (2.5×) |
| mulmod | 4096 | 14,886 | 55,999 (3.8×) | 29,638 (2.0×) | 68,024 (4.6×) | 43,084 (2.9×) |
| add_mod | 64 | 2.9 | 4.0 (1.4×) | 4.0 (1.4×) | 7.1 (2.4×) | 7.0 (2.4×) |
| add_mod | 128 | 3.8 | 9.1 (2.4×) | 9.1 (2.4×) | 11.2 (2.9×) | 14.0 (3.7×) |
| add_mod | 256 | 9.6 | 21.9 (2.3×) | 21.9 (2.3×) | 24.5 (2.6×) | 30.5 (3.2×) |
| add_mod | 384 | 17.9 | 30.6 (1.7×) | 28.0 (1.6×) | 47.0 (2.6×) | 46.3 (2.6×) |
| add_mod | 512 | 27.3 | 46.6 (1.7×) | 40.5 (1.5×) | 73.3 (2.7×) | 59.8 (2.2×) |
| add_mod | 1024 | 69.2 | 150 (2.2×) | 104 (1.5×) | 187 (2.7×) | 130 (1.9×) |
| add_mod | 2048 | 161 | 337 (2.1×) | 243 (1.5×) | 466 (2.9×) | 312 (1.9×) |
| add_mod | 4096 | 299 | 830 (2.8×) | 671 (2.2×) | 846 (2.8×) | 864 (2.9×) |
| sub_mod | 64 | 2.4 | 4.0 (1.7×) | 4.0 (1.7×) | 7.0 (2.9×) | 7.6 (3.2×) |
| sub_mod | 128 | 4.3 | 10.5 (2.4×) | 10.2 (2.4×) | 10.7 (2.5×) | 15.7 (3.7×) |
| sub_mod | 256 | 9.0 | 23.1 (2.6×) | 19.8 (2.2×) | 25.1 (2.8×) | 28.7 (3.2×) |
| sub_mod | 384 | 16.5 | 31.1 (1.9×) | 26.4 (1.6×) | 48.0 (2.9×) | 47.1 (2.9×) |
| sub_mod | 512 | 25.6 | 49.2 (1.9×) | 37.8 (1.5×) | 55.1 (2.2×) | 66.6 (2.6×) |
| sub_mod | 1024 | 66.5 | 135 (2.0×) | 109 (1.6×) | 141 (2.1×) | 133 (2.0×) |
| sub_mod | 2048 | 148 | 293 (2.0×) | 249 (1.7×) | 358 (2.4×) | 280 (1.9×) |
| sub_mod | 4096 | 237 | 796 (3.4×) | 709 (3.0×) | 811 (3.4×) | 809 (3.4×) |
| add_mod, reduced operands | 64 | 2.7 | 4.2 (1.6×) | 4.1 (1.5×) | 4.4 (1.6×) | 4.4 (1.6×) |
| add_mod, reduced operands | 128 | 4.3 | 7.9 (1.8×) | 9.2 (2.1×) | 7.7 (1.8×) | 11.6 (2.7×) |
| add_mod, reduced operands | 256 | 19.6 | 17.3 (0.9×) | 15.6 (0.8×) | 17.2 (0.9×) | 22.0 (1.1×) |
| add_mod, reduced operands | 384 | 26.3 | 28.2 (1.1×) | 22.1 (0.8×) | 28.2 (1.1×) | 32.2 (1.2×) |
| add_mod, reduced operands | 512 | 31.9 | 41.5 (1.3×) | 28.7 (0.9×) | 49.1 (1.5×) | 43.3 (1.4×) |
| add_mod, reduced operands | 1024 | 47.4 | 134 (2.8×) | 67.1 (1.4×) | 146 (3.1×) | 93.5 (2.0×) |
| add_mod, reduced operands | 2048 | 89.3 | 255 (2.9×) | 147 (1.6×) | 275 (3.1×) | 221 (2.5×) |
| add_mod, reduced operands | 4096 | 235 | 624 (2.7×) | 479 (2.0×) | 653 (2.8×) | 655 (2.8×) |
| sub_mod, reduced operands | 64 | 2.3 | 3.4 (1.5×) | 4.0 (1.7×) | 4.7 (2.0×) | 5.3 (2.3×) |
| sub_mod, reduced operands | 128 | 4.8 | 6.8 (1.4×) | 7.3 (1.5×) | 7.3 (1.5×) | 9.5 (2.0×) |
| sub_mod, reduced operands | 256 | 7.4 | 16.9 (2.3×) | 13.3 (1.8×) | 14.9 (2.0×) | 20.6 (2.8×) |
| sub_mod, reduced operands | 384 | 11.0 | 26.2 (2.4×) | 20.7 (1.9×) | 25.4 (2.3×) | 31.1 (2.8×) |
| sub_mod, reduced operands | 512 | 18.6 | 37.0 (2.0×) | 29.6 (1.6×) | 36.9 (2.0×) | 42.6 (2.3×) |
| sub_mod, reduced operands | 1024 | 44.3 | 96.3 (2.2×) | 69.9 (1.6×) | 95.7 (2.2×) | 92.2 (2.1×) |
| sub_mod, reduced operands | 2048 | 83.5 | 240 (2.9×) | 163 (2.0×) | 323 (3.9×) | 206 (2.5×) |
| sub_mod, reduced operands | 4096 | 240 | 585 (2.4×) | 512 (2.1×) | 593 (2.5×) | 649 (2.7×) |
| add, constant time | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.3 (1.8×) |
| add, constant time | 256 | 4.6 | 10.3 (2.2×) | 8.6 (1.9×) | 10.1 (2.2×) | 11.5 (2.5×) |
| add, constant time | 384 | 7.0 | 16.4 (2.3×) | 12.3 (1.8×) | 16.0 (2.3×) | 18.0 (2.6×) |
| add, constant time | 512 | 9.2 | 27.4 (3.0×) | 20.8 (2.3×) | 26.9 (2.9×) | 32.1 (3.5×) |
| add, constant time | 1024 | 23.6 | 64.0 (2.7×) | 46.2 (2.0×) | 82.4 (3.5×) | 68.9 (2.9×) |
| add, constant time | 2048 | 52.6 | 160 (3.0×) | 113 (2.1×) | 188 (3.6×) | 147 (2.8×) |
| add, constant time | 4096 | 155 | 426 (2.7×) | 321 (2.1×) | 464 (3.0×) | 403 (2.6×) |
| mul, constant time | 64 | 1.5 | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul, constant time | 128 | 3.0 | 10.0 (3.3×) | 6.5 (2.2×) | 12.3 (4.1×) | 5.7 (1.9×) |
| mul, constant time | 256 | 12.4 | 48.0 (3.9×) | 16.7 (1.3×) | 67.6 (5.5×) | 26.0 (2.1×) |
| mul, constant time | 384 | 30.0 | 186 (6.2×) | 52.6 (1.8×) | 227 (7.6×) | 72.8 (2.4×) |
| mul, constant time | 512 | 51.1 | 304 (5.9×) | 74.6 (1.5×) | 352 (6.9×) | 131 (2.6×) |
| mul, constant time | 1024 | 226 | 1,346 (6.0×) | 695 (3.1×) | 1,574 (7.0×) | 862 (3.8×) |
| mul, constant time | 2048 | 1,160 | 5,796 (5.0×) | 2,594 (2.2×) | 6,791 (5.9×) | 3,884 (3.3×) |
| mul, constant time | 4096 | 4,217 | 18,389 (4.4×) | 8,883 (2.1×) | 21,912 (5.2×) | 12,827 (3.0×) |
| div, constant time | 64 | 25.1 | 84.3 (3.4×) | 47.3 (1.9×) | 145 (5.8×) | 61.8 (2.5×) |
| div, constant time | 128 | 98.7 | 254 (2.6×) | 145 (1.5×) | 360 (3.6×) | 231 (2.3×) |
| div, constant time | 256 | 200 | 505 (2.5×) | 245 (1.2×) | 612 (3.1×) | 473 (2.4×) |
| div, constant time | 384 | 292 | 503 (1.7×) | 474 (1.6×) | 808 (2.8×) | 399 (1.4×) |
| div, constant time | 512 | 460 | 1,011 (2.2×) | 712 (1.5×) | 1,165 (2.5×) | 858 (1.9×) |
| div, constant time | 1024 | 1,142 | 2,466 (2.2×) | 1,782 (1.6×) | 2,830 (2.5×) | 2,189 (1.9×) |
| div, constant time | 2048 | 3,457 | 7,321 (2.1×) | 5,079 (1.5×) | 8,613 (2.5×) | 6,353 (1.8×) |
| div, constant time | 4096 | 11,885 | 25,665 (2.2×) | 16,331 (1.4×) | 27,610 (2.3×) | 21,492 (1.8×) |
| mod, constant time | 64 | 25.6 | 83.1 (3.2×) | 48.8 (1.9×) | 152 (5.9×) | 67.6 (2.6×) |
| mod, constant time | 128 | 105 | 206 (2.0×) | 152 (1.4×) | 334 (3.2×) | 226 (2.2×) |
| mod, constant time | 256 | 328 | 530 (1.6×) | 331 (1.0×) | 568 (1.7×) | 414 (1.3×) |
| mod, constant time | 384 | 314 | 742 (2.4×) | 527 (1.7×) | 781 (2.5×) | 563 (1.8×) |
| mod, constant time | 512 | 470 | 1,041 (2.2×) | 756 (1.6×) | 1,097 (2.3×) | 808 (1.7×) |
| mod, constant time | 1024 | 1,142 | 2,510 (2.2×) | 1,783 (1.6×) | 2,730 (2.4×) | 1,992 (1.7×) |
| mod, constant time | 2048 | 3,459 | 7,618 (2.2×) | 5,084 (1.5×) | 8,188 (2.4×) | 5,939 (1.7×) |
| mod, constant time | 4096 | 11,898 | 26,068 (2.2×) | 16,355 (1.4×) | 28,822 (2.4×) | 20,303 (1.7×) |
| mulmod, constant time | 64 | 51.7 | 202 (3.9×) | 105 (2.0×) | 236 (4.6×) | 105 (2.0×) |
| mulmod, constant time | 128 | 201 | 562 (2.8×) | 361 (1.8×) | 607 (3.0×) | 376 (1.9×) |
| mulmod, constant time | 256 | 444 | 1,215 (2.7×) | 735 (1.7×) | 1,326 (3.0×) | 816 (1.8×) |
| mulmod, constant time | 384 | 819 | 1,983 (2.4×) | 1,203 (1.5×) | 2,309 (2.8×) | 1,511 (1.8×) |
| mulmod, constant time | 512 | 1,186 | 3,048 (2.6×) | 1,830 (1.5×) | 3,521 (3.0×) | 2,385 (2.0×) |
| mulmod, constant time | 1024 | 3,872 | 9,561 (2.5×) | 5,958 (1.5×) | 10,777 (2.8×) | 7,712 (2.0×) |
| mulmod, constant time | 2048 | 13,645 | 32,883 (2.4×) | 19,593 (1.4×) | 36,440 (2.7×) | 25,871 (1.9×) |
| mulmod, constant time | 4096 | 48,900 | 119,413 (2.4×) | 70,114 (1.4×) | 134,990 (2.8×) | 88,804 (1.8×) |
| add_mod, constant time | 64 | 6.6 | 9.1 (1.4×) | 9.0 (1.4×) | 18.4 (2.8×) | 13.8 (2.1×) |
| add_mod, constant time | 128 | 11.0 | 26.5 (2.4×) | 21.6 (2.0×) | 34.4 (3.1×) | 34.1 (3.1×) |
| add_mod, constant time | 256 | 20.4 | 82.0 (4.0×) | 65.0 (3.2×) | 108 (5.3×) | 83.5 (4.1×) |
| add_mod, constant time | 384 | 39.0 | 120 (3.1×) | 110 (2.8×) | 166 (4.3×) | 140 (3.6×) |
| add_mod, constant time | 512 | 49.8 | 153 (3.1×) | 145 (2.9×) | 247 (5.0×) | 179 (3.6×) |
| add_mod, constant time | 1024 | 129 | 356 (2.8×) | 270 (2.1×) | 627 (4.9×) | 343 (2.7×) |
| add_mod, constant time | 2048 | 303 | 865 (2.9×) | 530 (1.7×) | 1,675 (5.5×) | 736 (2.4×) |
| add_mod, constant time | 4096 | 560 | 1,870 (3.3×) | 1,719 (3.1×) | 1,957 (3.5×) | 1,973 (3.5×) |
| sub_mod, constant time | 64 | 6.3 | 10.2 (1.6×) | 9.3 (1.5×) | 18.3 (2.9×) | 14.2 (2.3×) |
| sub_mod, constant time | 128 | 7.4 | 15.5 (2.1×) | 13.0 (1.8×) | 29.7 (4.0×) | 23.0 (3.1×) |
| sub_mod, constant time | 256 | 13.7 | 45.8 (3.3×) | 32.2 (2.4×) | 73.8 (5.4×) | 66.7 (4.9×) |
| sub_mod, constant time | 384 | 26.9 | 102 (3.8×) | 51.2 (1.9×) | 135 (5.0×) | 98.5 (3.7×) |
| sub_mod, constant time | 512 | 33.5 | 146 (4.4×) | 88.1 (2.6×) | 219 (6.5×) | 141 (4.2×) |
| sub_mod, constant time | 1024 | 96.5 | 321 (3.3×) | 210 (2.2×) | 518 (5.4×) | 273 (2.8×) |
| sub_mod, constant time | 2048 | 278 | 727 (2.6×) | 510 (1.8×) | 1,331 (4.8×) | 635 (2.3×) |
| sub_mod, constant time | 4096 | 503 | 1,529 (3.0×) | 1,429 (2.8×) | 1,694 (3.4×) | 1,679 (3.3×) |
| add_mod, reduced operands, constant time | 64 | 3.8 | 5.8 (1.5×) | 6.1 (1.6×) | 6.0 (1.6×) | 7.4 (1.9×) |
| add_mod, reduced operands, constant time | 128 | 6.9 | 13.6 (2.0×) | 11.8 (1.7×) | 13.6 (2.0×) | 18.0 (2.6×) |
| add_mod, reduced operands, constant time | 256 | 9.8 | 29.1 (3.0×) | 23.6 (2.4×) | 31.4 (3.2×) | 32.1 (3.3×) |
| add_mod, reduced operands, constant time | 384 | 13.2 | 47.5 (3.6×) | 37.0 (2.8×) | 50.5 (3.8×) | 47.1 (3.6×) |
| add_mod, reduced operands, constant time | 512 | 19.8 | 81.3 (4.1×) | 49.7 (2.5×) | 85.9 (4.3×) | 62.7 (3.2×) |
| add_mod, reduced operands, constant time | 1024 | 56.0 | 169 (3.0×) | 95.6 (1.7×) | 335 (6.0×) | 127 (2.3×) |
| add_mod, reduced operands, constant time | 2048 | 132 | 381 (2.9×) | 182 (1.4×) | 738 (5.6×) | 290 (2.2×) |
| add_mod, reduced operands, constant time | 4096 | 288 | 876 (3.0×) | 742 (2.6×) | 898 (3.1×) | 931 (3.2×) |
| sub_mod, reduced operands, constant time | 64 | 3.2 | 4.8 (1.5×) | 4.9 (1.5×) | 4.7 (1.5×) | 5.0 (1.6×) |
| sub_mod, reduced operands, constant time | 128 | 5.1 | 9.6 (1.9×) | 9.0 (1.8×) | 9.0 (1.8×) | 12.0 (2.4×) |
| sub_mod, reduced operands, constant time | 256 | 8.9 | 24.3 (2.7×) | 20.2 (2.3×) | 31.0 (3.5×) | 28.1 (3.2×) |
| sub_mod, reduced operands, constant time | 384 | 11.0 | 40.5 (3.7×) | 33.3 (3.0×) | 66.3 (6.0×) | 44.5 (4.0×) |
| sub_mod, reduced operands, constant time | 512 | 18.5 | 66.1 (3.6×) | 46.5 (2.5×) | 97.1 (5.2×) | 66.6 (3.6×) |
| sub_mod, reduced operands, constant time | 1024 | 47.4 | 148 (3.1×) | 114 (2.4×) | 218 (4.6×) | 137 (2.9×) |
| sub_mod, reduced operands, constant time | 2048 | 115 | 354 (3.1×) | 254 (2.2×) | 605 (5.3×) | 293 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 273 | 864 (3.2×) | 741 (2.7×) | 992 (3.6×) | 926 (3.4×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | 128 | 1.3 | 2.9 (2.2×) | 2.5 (1.9×) | 2.7 (2.1×) | 3.2 (2.5×) |
| add | 256 | 2.3 | 9.8 (4.3×) | 6.8 (3.0×) | 8.9 (3.9×) | 11.3 (4.9×) |
| add | 384 | 3.5 | 15.3 (4.4×) | 12.2 (3.5×) | 15.3 (4.4×) | 18.1 (5.2×) |
| add | 512 | 5.2 | 26.0 (5.0×) | 17.6 (3.4×) | 25.8 (5.0×) | 31.2 (6.0×) |
| add | 1024 | 22.2 | 65.7 (3.0×) | 45.1 (2.0×) | 69.0 (3.1×) | 67.2 (3.0×) |
| add | 2048 | 51.5 | 162 (3.1×) | 108 (2.1×) | 179 (3.5×) | 142 (2.8×) |
| add | 4096 | 175 | 419 (2.4×) | 308 (1.8×) | 438 (2.5×) | 400 (2.3×) |
| mulmod | 64 | 23.0 | 68.5 (3.0×) | 35.1 (1.5×) | 110 (4.8×) | 42.0 (1.8×) |
| mulmod | 128 | 50.7 | 210 (4.1×) | 85.9 (1.7×) | 261 (5.1×) | 141 (2.8×) |
| mulmod | 256 | 185 | 578 (3.1×) | 283 (1.5×) | 688 (3.7×) | 358 (1.9×) |
| mulmod | 384 | 346 | 1,050 (3.0×) | 489 (1.4×) | 1,350 (3.9×) | 651 (1.9×) |
| mulmod | 512 | 501 | 1,665 (3.3×) | 858 (1.7×) | 2,050 (4.1×) | 1,111 (2.2×) |
| mulmod | 1024 | 1,360 | 5,298 (3.9×) | 2,759 (2.0×) | 6,821 (5.0×) | 4,170 (3.1×) |
| mulmod | 2048 | 5,198 | 17,555 (3.4×) | 9,487 (1.8×) | 22,223 (4.3×) | 12,807 (2.5×) |
| mulmod | 4096 | 14,951 | 55,918 (3.7×) | 29,652 (2.0×) | 68,405 (4.6×) | 43,035 (2.9×) |
| add_mod | 64 | 7.8 | 6.3 (0.8×) | 6.2 (0.8×) | 8.6 (1.1×) | 9.0 (1.2×) |
| add_mod | 128 | 11.6 | 31.3 (2.7×) | 10.4 (0.9×) | 41.7 (3.6×) | 16.4 (1.4×) |
| add_mod | 256 | 14.7 | 43.2 (2.9×) | 30.9 (2.1×) | 54.5 (3.7×) | 43.8 (3.0×) |
| add_mod | 384 | 28.4 | 56.7 (2.0×) | 42.3 (1.5×) | 79.8 (2.8×) | 66.1 (2.3×) |
| add_mod | 512 | 32.0 | 69.8 (2.2×) | 53.4 (1.7×) | 97.6 (3.0×) | 81.5 (2.5×) |
| add_mod | 1024 | 79.1 | 161 (2.0×) | 119 (1.5×) | 213 (2.7×) | 158 (2.0×) |
| add_mod | 2048 | 163 | 339 (2.1×) | 237 (1.5×) | 581 (3.6×) | 324 (2.0×) |
| add_mod | 4096 | 324 | 895 (2.8×) | 797 (2.5×) | 997 (3.1×) | 948 (2.9×) |
| sub_mod | 64 | 7.7 | 4.7 (0.6×) | 7.1 (0.9×) | 8.3 (1.1×) | 9.9 (1.3×) |
| sub_mod | 128 | 11.1 | 35.0 (3.2×) | 11.0 (1.0×) | 47.3 (4.3×) | 15.6 (1.4×) |
| sub_mod | 256 | 14.3 | 43.4 (3.0×) | 32.3 (2.3×) | 61.3 (4.3×) | 46.2 (3.2×) |
| sub_mod | 384 | 29.3 | 58.5 (2.0×) | 41.8 (1.4×) | 79.1 (2.7×) | 66.7 (2.3×) |
| sub_mod | 512 | 37.8 | 64.7 (1.7×) | 53.8 (1.4×) | 96.2 (2.5×) | 89.1 (2.4×) |
| sub_mod | 1024 | 71.1 | 164 (2.3×) | 126 (1.8×) | 197 (2.8×) | 170 (2.4×) |
| sub_mod | 2048 | 147 | 364 (2.5×) | 263 (1.8×) | 417 (2.8×) | 364 (2.5×) |
| sub_mod | 4096 | 388 | 924 (2.4×) | 819 (2.1×) | 969 (2.5×) | 997 (2.6×) |
| add_mod, reduced operands | 64 | 5.0 | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) | 5.0 (1.0×) |
| add_mod, reduced operands | 128 | 5.7 | 6.7 (1.2×) | 7.5 (1.3×) | 7.5 (1.3×) | 9.8 (1.7×) |
| add_mod, reduced operands | 256 | 13.4 | 16.7 (1.2×) | 13.0 (1.0×) | 16.9 (1.3×) | 22.3 (1.7×) |
| add_mod, reduced operands | 384 | 9.6 | 29.5 (3.1×) | 19.7 (2.1×) | 29.0 (3.0×) | 32.0 (3.3×) |
| add_mod, reduced operands | 512 | 25.7 | 42.5 (1.7×) | 28.1 (1.1×) | 50.8 (2.0×) | 43.4 (1.7×) |
| add_mod, reduced operands | 1024 | 44.1 | 115 (2.6×) | 68.4 (1.6×) | 148 (3.4×) | 92.0 (2.1×) |
| add_mod, reduced operands | 2048 | 83.4 | 247 (3.0×) | 158 (1.9×) | 276 (3.3×) | 208 (2.5×) |
| add_mod, reduced operands | 4096 | 253 | 620 (2.5×) | 484 (1.9×) | 619 (2.4×) | 651 (2.6×) |
| sub_mod, reduced operands | 64 | 3.9 | 2.5 (0.6×) | 2.7 (0.7×) | 3.2 (0.8×) | 3.7 (0.9×) |
| sub_mod, reduced operands | 128 | 4.8 | 5.4 (1.1×) | 6.1 (1.3×) | 6.1 (1.3×) | 9.2 (1.9×) |
| sub_mod, reduced operands | 256 | 7.4 | 15.0 (2.0×) | 11.9 (1.6×) | 13.7 (1.9×) | 19.0 (2.6×) |
| sub_mod, reduced operands | 384 | 10.3 | 25.0 (2.4×) | 18.9 (1.8×) | 25.4 (2.5×) | 30.0 (2.9×) |
| sub_mod, reduced operands | 512 | 16.3 | 37.0 (2.3×) | 28.9 (1.8×) | 41.3 (2.5×) | 41.5 (2.5×) |
| sub_mod, reduced operands | 1024 | 38.1 | 96.2 (2.5×) | 68.0 (1.8×) | 93.6 (2.5×) | 91.1 (2.4×) |
| sub_mod, reduced operands | 2048 | 75.8 | 242 (3.2×) | 169 (2.2×) | 292 (3.9×) | 204 (2.7×) |
| sub_mod, reduced operands | 4096 | 241 | 602 (2.5×) | 509 (2.1×) | 614 (2.5×) | 635 (2.6×) |
| add, constant time | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | 128 | 1.3 | 2.9 (2.2×) | 2.5 (1.9×) | 2.7 (2.1×) | 3.2 (2.5×) |
| add, constant time | 256 | 2.4 | 9.7 (4.0×) | 6.8 (2.8×) | 8.9 (3.7×) | 11.3 (4.7×) |
| add, constant time | 384 | 3.7 | 15.4 (4.2×) | 12.2 (3.3×) | 15.3 (4.1×) | 18.0 (4.9×) |
| add, constant time | 512 | 4.7 | 26.0 (5.5×) | 17.6 (3.7×) | 25.8 (5.5×) | 31.1 (6.6×) |
| add, constant time | 1024 | 22.4 | 65.3 (2.9×) | 44.8 (2.0×) | 68.9 (3.1×) | 67.3 (3.0×) |
| add, constant time | 2048 | 50.3 | 161 (3.2×) | 107 (2.1×) | 180 (3.6×) | 142 (2.8×) |
| add, constant time | 4096 | 174 | 418 (2.4×) | 304 (1.7×) | 437 (2.5×) | 409 (2.4×) |
| mulmod, constant time | 64 | 93.9 | 144 (1.5×) | 94.5 (1.0×) | 243 (2.6×) | 110 (1.2×) |
| mulmod, constant time | 128 | 203 | 564 (2.8×) | 350 (1.7×) | 607 (3.0×) | 368 (1.8×) |
| mulmod, constant time | 256 | 462 | 1,202 (2.6×) | 742 (1.6×) | 1,291 (2.8×) | 844 (1.8×) |
| mulmod, constant time | 384 | 826 | 2,001 (2.4×) | 1,198 (1.5×) | 2,300 (2.8×) | 1,494 (1.8×) |
| mulmod, constant time | 512 | 1,189 | 3,041 (2.6×) | 1,884 (1.6×) | 3,465 (2.9×) | 2,292 (1.9×) |
| mulmod, constant time | 1024 | 3,864 | 9,519 (2.5×) | 5,983 (1.5×) | 10,970 (2.8×) | 7,683 (2.0×) |
| mulmod, constant time | 2048 | 13,598 | 32,886 (2.4×) | 19,838 (1.5×) | 36,578 (2.7×) | 25,463 (1.9×) |
| mulmod, constant time | 4096 | 48,809 | 120,203 (2.5×) | 69,756 (1.4×) | 131,614 (2.7×) | 89,355 (1.8×) |
| add_mod, constant time | 64 | 8.9 | 12.3 (1.4×) | 13.0 (1.5×) | 19.7 (2.2×) | 14.3 (1.6×) |
| add_mod, constant time | 128 | 12.5 | 28.1 (2.2×) | 22.6 (1.8×) | 39.0 (3.1×) | 34.1 (2.7×) |
| add_mod, constant time | 256 | 18.2 | 83.6 (4.6×) | 66.6 (3.7×) | 98.8 (5.4×) | 89.7 (4.9×) |
| add_mod, constant time | 384 | 36.0 | 112 (3.1×) | 108 (3.0×) | 175 (4.9×) | 127 (3.5×) |
| add_mod, constant time | 512 | 47.8 | 152 (3.2×) | 135 (2.8×) | 249 (5.2×) | 167 (3.5×) |
| add_mod, constant time | 1024 | 130 | 354 (2.7×) | 261 (2.0×) | 639 (4.9×) | 349 (2.7×) |
| add_mod, constant time | 2048 | 307 | 849 (2.8×) | 515 (1.7×) | 1,599 (5.2×) | 767 (2.5×) |
| add_mod, constant time | 4096 | 501 | 1,876 (3.7×) | 1,763 (3.5×) | 1,941 (3.9×) | 1,969 (3.9×) |
| sub_mod, constant time | 64 | 8.9 | 12.1 (1.4×) | 11.0 (1.2×) | 19.0 (2.1×) | 14.1 (1.6×) |
| sub_mod, constant time | 128 | 11.8 | 24.0 (2.0×) | 19.0 (1.6×) | 31.6 (2.7×) | 23.0 (1.9×) |
| sub_mod, constant time | 256 | 17.1 | 54.7 (3.2×) | 36.9 (2.2×) | 75.0 (4.4×) | 57.7 (3.4×) |
| sub_mod, constant time | 384 | 27.6 | 102 (3.7×) | 57.8 (2.1×) | 123 (4.5×) | 85.5 (3.1×) |
| sub_mod, constant time | 512 | 32.4 | 143 (4.4×) | 84.3 (2.6×) | 191 (5.9×) | 117 (3.6×) |
| sub_mod, constant time | 1024 | 93.5 | 314 (3.4×) | 200 (2.1×) | 531 (5.7×) | 255 (2.7×) |
| sub_mod, constant time | 2048 | 285 | 730 (2.6×) | 510 (1.8×) | 1,266 (4.4×) | 582 (2.0×) |
| sub_mod, constant time | 4096 | 487 | 1,548 (3.2×) | 1,423 (2.9×) | 1,667 (3.4×) | 1,687 (3.5×) |
| add_mod, reduced operands, constant time | 64 | 6.9 | 8.1 (1.2×) | 8.0 (1.2×) | 8.0 (1.2×) | 8.5 (1.2×) |
| add_mod, reduced operands, constant time | 128 | 8.8 | 15.0 (1.7×) | 13.1 (1.5×) | 14.6 (1.7×) | 16.2 (1.8×) |
| add_mod, reduced operands, constant time | 256 | 13.2 | 30.8 (2.3×) | 23.0 (1.7×) | 28.5 (2.2×) | 31.1 (2.4×) |
| add_mod, reduced operands, constant time | 384 | 14.7 | 51.7 (3.5×) | 34.9 (2.4×) | 51.8 (3.5×) | 49.6 (3.4×) |
| add_mod, reduced operands, constant time | 512 | 20.6 | 69.5 (3.4×) | 49.2 (2.4×) | 94.1 (4.6×) | 66.1 (3.2×) |
| add_mod, reduced operands, constant time | 1024 | 51.9 | 171 (3.3×) | 91.4 (1.8×) | 331 (6.4×) | 144 (2.8×) |
| add_mod, reduced operands, constant time | 2048 | 141 | 378 (2.7×) | 183 (1.3×) | 743 (5.3×) | 299 (2.1×) |
| add_mod, reduced operands, constant time | 4096 | 290 | 883 (3.0×) | 749 (2.6×) | 893 (3.1×) | 970 (3.3×) |
| sub_mod, reduced operands, constant time | 64 | 3.9 | 4.0 (1.0×) | 5.0 (1.3×) | 4.0 (1.0×) | 5.1 (1.3×) |
| sub_mod, reduced operands, constant time | 128 | 4.8 | 10.6 (2.2×) | 10.0 (2.1×) | 10.0 (2.1×) | 10.1 (2.1×) |
| sub_mod, reduced operands, constant time | 256 | 6.9 | 23.1 (3.3×) | 19.1 (2.8×) | 31.0 (4.5×) | 28.0 (4.1×) |
| sub_mod, reduced operands, constant time | 384 | 9.3 | 41.3 (4.4×) | 32.0 (3.4×) | 55.8 (6.0×) | 43.4 (4.7×) |
| sub_mod, reduced operands, constant time | 512 | 17.8 | 65.2 (3.7×) | 44.6 (2.5×) | 78.7 (4.4×) | 62.6 (3.5×) |
| sub_mod, reduced operands, constant time | 1024 | 43.9 | 150 (3.4×) | 112 (2.6×) | 217 (4.9×) | 141 (3.2×) |
| sub_mod, reduced operands, constant time | 2048 | 120 | 346 (2.9×) | 257 (2.1×) | 637 (5.3×) | 303 (2.5×) |
| sub_mod, reduced operands, constant time | 4096 | 356 | 877 (2.5×) | 733 (2.1×) | 997 (2.8×) | 945 (2.7×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
