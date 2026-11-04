# Benchmarks

`biguint` at commit `35cb27c` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-03.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 15; 1-minute load 21.21 before, 18.38 after; its SMT sibling, CPU 31, 36% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: the cycles of 2n operations less those of n, over n, of rank 4 from the fewest among 31 repetitions that count both runs back to back.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands at width N: uniform below 2^N; a divisor of N/2 bits; a modulus of N bits with operands below it.
- Every result matches `num-bigint` before it is measured.
- A row compares the functions of `biguint`'s timing class: fixed for add and for mul up to 1024 bits, variable for the rest.
- A constant-time table compares `ConstantTime<Uint>` with every function whose time the width fixes; divisors and moduli are public.

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

- `native`: add, mul, div, mod, add_mod and sub_mod at 64 and 128 bits; mulmod at 64 bits.
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

- `native`: at 64 and 128 bits.
- `ruint`: mul at 64, 128, 256, 384 and 512 bits.
- `uint`: add at 64 bits; mul at 64, 128, 256, 384, 1024, 2048 and 4096 bits.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.1** | 1.5 / **1.1** | 1.5 / **1.1** | 1.5 / **1.1** | 1.5 / **1.1** | 1.5 / **1.1** |
| 128 | 2.7 / 1.7 | **2.6** / **1.5** | **2.6** / **1.5** | **2.6** / **1.5** | 2.7 / **1.5** | – |
| 256 | 5.2 / **2.7** | – | **5.0** / **2.7** | 5.8 / 5.5 | 5.2 / 3.2 | – |
| 384 | 7.8 / **3.9** | – | **7.4** / 4.0 | 9.2 / 10.7 | 8.0 / 5.2 | – |
| 512 | 10.4 / **5.4** | – | **10.1** / **5.4** | 12.5 / 16.7 | 11.5 / 8.5 | – |
| 1024 | **25.4** / 23.8 | – | 29.5 / **22.9** | 29.2 / 28.9 | 28.1 / 29.5 | – |
| 2048 | **57.0** / **53.2** | – | 124 / 71.2 | 79.5 / 97.1 | 98.6 / 107 | – |
| 4096 | 188 / **178** | – | **171** / 196 | 205 / 281 | 221 / 267 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **1.5** / – | **1.5** / **3.0** | **1.5** / – | **1.5** / – | **1.5** / – | **1.5** / – | – |
| 128 | **3.0** / – | 3.1 / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | 16.0 / – | – | **12.5** / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | 30.0 / – | – | 30.5 / – | **29.8** / – | 33.6 / – | 36.6 / – | – |
| 512 | **52.6** / – | – | 60.1 / – | 79.8 / – | 88.2 / – | – | – |
| 1024 | **220** / – | – | – | 373 / – | 395 / – | 436 / – | – |
| 2048 | **1,361** / – | – | 1,605 / – | 1,801 / – | 1,541 / – | 4,723 / – | 3,606 / – |
| 4096 | **4,170** / – | – | 6,475 / – | 7,394 / – | 6,147 / – | 25,032 / – | 11,324 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 223 / – | **7.0** / – | **7.0** / – | 37.3 / – |
| 128 | 17.5 / – | **16.4** / – | 31.9 / – | 268 / – | 30.0 / – | 30.0 / – | 103 / – |
| 256 | **60.6** / – | – | 81.6 / – | 546 / – | 244 / – | 204 / – | 273 / – |
| 384 | **131** / – | – | 200 / – | 713 / – | 361 / – | 313 / – | 339 / – |
| 512 | **186** / – | – | 268 / – | 854 / – | 455 / – | 418 / – | 400 / – |
| 1024 | **506** / – | – | 594 / – | 1,670 / – | 1,056 / – | 1,000 / – | 805 / – |
| 2048 | **1,397** / – | – | 1,546 / – | 4,138 / – | 3,424 / – | 3,313 / – | 2,098 / – |
| 4096 | **4,666** / – | – | 4,870 / – | 12,834 / – | 12,465 / – | 12,541 / – | 6,455 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 221 / – | **7.0** / – | **7.0** / – | 8.0 / – |
| 128 | 18.4 / – | **17.4** / – | 31.0 / – | 271 / – | 31.0 / – | 31.0 / – | 103 / – |
| 256 | **59.3** / – | – | 82.9 / – | 545 / – | 254 / – | 200 / – | 273 / – |
| 384 | **131** / – | – | 194 / – | 714 / – | 364 / – | 330 / – | 340 / – |
| 512 | **183** / – | – | 249 / – | 852 / – | 474 / – | 428 / – | 401 / – |
| 1024 | **512** / – | – | 573 / – | 1,656 / – | 1,063 / – | 995 / – | 806 / – |
| 2048 | **1,444** / – | – | 1,545 / – | 4,109 / – | 3,444 / – | 3,308 / – | 2,095 / – |
| 4096 | **4,680** / – | – | 4,899 / – | 12,835 / – | 12,583 / – | 12,587 / – | 6,435 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **12.0** / **23.1** | **12.0** / **23.1** | 20.5 / 44.9 | 237 / 233 | 146 / – |
| 128 | **51.4** / **59.0** | – | 83.3 / 91.2 | 493 / 502 | 371 / – |
| 256 | 194 / 222 | – | **170** / **206** | 980 / 984 | 564 / – |
| 384 | **348** / **359** | – | 374 / 392 | 1,465 / 1,498 | 863 / – |
| 512 | **528** / **539** | – | 543 / 561 | 1,999 / 2,037 | 1,173 / – |
| 1024 | **1,498** / **1,476** | – | 1,594 / 1,585 | 5,254 / 5,337 | 3,217 / – |
| 2048 | **5,643** / **5,619** | – | 7,410 / 7,395 | 16,347 / 16,430 | 10,092 / – |
| 4096 | **20,580** / **20,759** | – | 29,598 / 29,550 | 58,327 / 58,395 | 34,254 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 4.0 / **9.0** | **3.0** / – | 12.0 / 13.5 | 11.2 / – | 17.4 / – | **3.0** / – | 102 / – |
| 128 | 8.0 / **17.3** | **6.0** / – | 34.7 / 22.4 | 27.4 / – | 17.2 / – | 25.9 / – | 248 / – |
| 256 | **23.5** / **24.3** | – | 44.6 / 61.6 | 71.0 / – | 24.0 / – | 58.4 / – | 249 / – |
| 384 | 34.5 / **40.6** | – | 58.2 / 81.3 | 91.1 / – | **32.9** / – | 91.8 / – | 252 / – |
| 512 | 41.9 / **49.3** | – | 75.3 / 97.7 | 123 / – | **40.9** / – | 133 / – | 258 / – |
| 1024 | 93.0 / **93.7** | – | 151 / 175 | 143 / – | **76.0** / – | 281 / – | 298 / – |
| 2048 | 173 / **165** | – | 382 / 401 | 436 / – | **168** / – | 591 / – | 492 / – |
| 4096 | **337** / **410** | – | 759 / 830 | 911 / – | 415 / – | 1,431 / – | 1,074 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **4.0** / **7.7** | 12.2 / – | 12.0 / 13.5 | 4.7 / – | 13.5 / – | 12.4 / – | 163 / – |
| 128 | 6.0 / **12.3** | **3.9** / – | 33.4 / 23.3 | 11.5 / – | 14.2 / – | 18.7 / – | 341 / – |
| 256 | **13.1** / **16.7** | – | 48.2 / 64.3 | 49.2 / – | 22.9 / – | 49.3 / – | 344 / – |
| 384 | **21.0** / **28.0** | – | 70.0 / 88.6 | 82.3 / – | 30.0 / – | 81.5 / – | 364 / – |
| 512 | **29.4** / **37.6** | – | 91.7 / 111 | 111 / – | 40.6 / – | 133 / – | 311 / – |
| 1024 | **63.0** / **82.0** | – | 179 / 201 | 132 / – | 69.2 / – | 292 / – | 379 / – |
| 2048 | **142** / **157** | – | 484 / 472 | 292 / – | 166 / – | 667 / – | 590 / – |
| 4096 | **288** / **422** | – | 1,018 / 1,089 | 599 / – | 416 / – | 1,596 / – | 868 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.5** / 1.1 | **1.5** / 1.1 | **1.5** / 1.1 | **1.5** / 1.1 | **1.5** / **1.0** | **1.5** / **1.0** |
| 128 | 2.7 / **1.5** | **2.6** / 1.6 | 2.7 / **1.5** | **2.6** / **1.5** | **2.6** / **1.5** | – |
| 256 | **5.1** / 2.8 | – | **5.1** / **2.7** | 5.8 / 5.5 | 5.3 / 3.2 | – |
| 384 | 7.7 / **4.0** | – | **7.4** / **4.0** | 9.2 / 10.7 | 8.0 / 5.2 | – |
| 512 | 10.4 / 5.5 | – | **10.1** / **5.4** | 12.4 / 16.8 | 11.7 / 8.5 | – |
| 1024 | **25.1** / 23.6 | – | 29.6 / **22.7** | 29.2 / 29.0 | 28.1 / 29.9 | – |
| 2048 | **56.5** / **53.4** | – | 124 / 71.5 | 79.6 / 97.4 | 98.8 / 106 | – |
| 4096 | 191 / **177** | – | **171** / 196 | 203 / 280 | 225 / 266 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.5** / – | **1.5** / **3.0** | **1.5** / – | **1.5** / – | **1.5** / – | **1.5** / – |
| 128 | **3.0** / – | 3.1 / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – |
| 256 | 16.0 / – | – | **12.6** / – | 13.0 / – | 13.0 / – | 16.0 / – |
| 384 | 30.0 / – | – | 30.5 / – | **29.8** / – | 33.6 / – | 36.6 / – |
| 512 | **51.0** / – | – | 59.8 / – | 79.8 / – | 88.6 / – | – |
| 1024 | **226** / – | – | – | 372 / – | 391 / – | 435 / – |
| 2048 | **1,180** / – | – | – | 1,801 / – | 1,541 / – | 4,723 / – |
| 4096 | **4,736** / – | – | – | 7,394 / – | 6,147 / – | 25,032 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.5** / – | 198 / – |
| 128 | **113** / – | 412 / – |
| 256 | **277** / – | 760 / – |
| 384 | **290** / – | 1,119 / – |
| 512 | **478** / – | 1,455 / – |
| 1024 | **1,141** / – | 3,381 / – |
| 2048 | **3,437** / – | 10,237 / – |
| 4096 | **11,904** / – | 30,980 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.9** / – | 192 / – |
| 128 | **112** / – | 390 / – |
| 256 | **225** / – | 734 / – |
| 384 | **312** / – | 1,087 / – |
| 512 | **492** / – | 1,449 / – |
| 1024 | **1,159** / – | 3,354 / – |
| 2048 | **3,460** / – | 10,138 / – |
| 4096 | **11,884** / – | 30,931 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **52.0** / **93.8** | 232 / – |
| 128 | **216** / **219** | 599 / – |
| 256 | **451** / **459** | 1,226 / – |
| 384 | **824** / **826** | 1,969 / – |
| 512 | **1,193** / **1,187** | 3,015 / – |
| 1024 | **3,861** / **3,852** | 7,753 / – |
| 2048 | **13,694** / **13,564** | 25,520 / – |
| 4096 | **50,607** / **50,728** | 87,027 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 52.4 / **60.6** | **11.2** / – |
| 128 | 165 / **161** | **27.4** / – |
| 256 | 232 / **222** | **71.0** / – |
| 384 | 309 / **304** | **91.1** / – |
| 512 | 348 / **346** | **123** / – |
| 1024 | 556 / **544** | **143** / – |
| 2048 | 1,003 / **1,183** | **436** / – |
| 4096 | 2,169 / **2,147** | **911** / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 50.0 / **64.6** | **4.7** / – |
| 128 | 168 / **166** | **11.5** / – |
| 256 | 231 / **224** | **49.2** / – |
| 384 | 278 / **280** | **82.3** / – |
| 512 | 345 / **345** | **111** / – |
| 1024 | 551 / **531** | **132** / – |
| 2048 | 1,136 / **980** | **292** / – |
| 4096 | 2,099 / **2,088** | **599** / – |

## WebAssembly

`biguint` at commit `adbe112` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-04.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions.
- Each figure: the counts of rank 4 from the fewest among 31 of each run length, 2n operations less n, over n. A case whose difference counts under 50 times the noise of the fixed cost of a count stops the sweep.
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
| Cost of a count, in WebAssembly, Chrome | 4,900 to 14,343 cycles and 3,081 instructions; its noise in a figure, 212 to 4,996 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 5,240 to 16,430 cycles and 3,081 instructions; its noise in a figure, 328 to 6,056 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 6,087 to 18,588 cycles and 1,885 instructions; its noise in a figure, 207 to 10,127 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 7,820 to 52,817 cycles and 1,885 instructions; its noise in a figure, 1,687 to 15,591 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 26.17 before, 29.72 after; its SMT sibling, CPU 28, 54% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 35.06 before, 14.64 after; its SMT sibling, CPU 28, 31% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 14.64 before, 29.04 after; its SMT sibling, CPU 28, 61% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 28.56 before, 75.1 after; its SMT sibling, CPU 28, 65% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / **1.2** | 2.4 / **1.2** | 2.6 / **1.2** | 2.6 / **1.2** | **2.3** / **1.2** | 2.5 / **1.2** |
| 128 | 4.3 / **2.9** | **4.1** / 3.0 | 4.2 / 3.0 | 4.2 / 3.0 | **4.1** / 3.0 | – |
| 256 | 10.4 / 9.9 | – | 10.4 / 9.3 | 10.6 / 9.8 | **10.3** / **8.2** | – |
| 384 | **16.6** / **15.9** | – | **16.6** / **15.9** | 18.2 / 17.3 | 17.8 / 16.5 | – |
| 512 | 27.8 / 27.1 | – | 27.4 / 26.4 | 33.2 / 33.1 | **24.9** / **23.5** | – |
| 1024 | 64.8 / 64.7 | – | 64.5 / 64.3 | 86.4 / 85.3 | **62.4** / **62.6** | – |
| 2048 | **161** / **162** | – | 208 / 210 | 191 / 230 | 211 / 252 | – |
| 4096 | 419 / 430 | – | 418 / **419** | **402** / 468 | 421 / 460 | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | 12.0 / – | **11.6** / **11.9** | 12.0 / – | 25.7 / – | **11.6** / – | 12.0 / – | – |
| 256 | 65.0 / – | – | **64.6** / – | 91.7 / – | 104 / – | 66.6 / – | – |
| 384 | 177 / – | – | **167** / – | 208 / – | 243 / – | 174 / – | – |
| 512 | 343 / – | – | **326** / – | 376 / – | 430 / – | – | – |
| 1024 | 2,206 / – | – | – | 2,077 / – | **1,546** / – | 2,052 / – | – |
| 2048 | 8,733 / – | – | **6,371** / – | 8,289 / – | 6,380 / – | 9,877 / – | 11,857 / – |
| 4096 | 29,198 / – | – | 44,260 / – | 29,223 / – | **23,667** / – | 43,136 / – | 39,843 / – |

### div: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 506 / – | **7.0** / – | **7.0** / – | 125 / – |
| 128 | 183 / – | 173 / – | **86.8** / – | 572 / – | 209 / – | 213 / – | 328 / – |
| 256 | **205** / – | – | 322 / – | 1,286 / – | 848 / – | 865 / – | 458 / – |
| 384 | **479** / – | – | 607 / – | 1,705 / – | 1,317 / – | 1,300 / – | 615 / – |
| 512 | **724** / – | – | 837 / – | 2,096 / – | 1,863 / – | 1,805 / – | 828 / – |
| 1024 | **1,467** / – | – | 2,125 / – | 4,163 / – | 4,625 / – | 4,610 / – | 1,935 / – |
| 2048 | **3,982** / – | – | 6,504 / – | 11,209 / – | 15,296 / – | 14,279 / – | 5,800 / – |
| 4096 | **13,685** / – | – | 23,205 / – | 36,930 / – | 49,576 / – | 52,482 / – | 20,292 / – |

### mod: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 514 / – | **7.0** / – | **7.0** / – | 33.4 / – |
| 128 | 182 / – | 174 / – | **85.8** / – | 573 / – | 216 / – | 214 / – | 335 / – |
| 256 | **205** / – | – | 322 / – | 1,293 / – | 855 / – | 859 / – | 468 / – |
| 384 | **477** / – | – | 581 / – | 1,709 / – | 1,331 / – | 1,298 / – | 616 / – |
| 512 | **703** / – | – | 804 / – | 2,090 / – | 1,864 / – | 1,818 / – | 812 / – |
| 1024 | **1,457** / – | – | 2,092 / – | 4,164 / – | 4,640 / – | 4,664 / – | 1,938 / – |
| 2048 | **4,067** / – | – | 6,507 / – | 11,211 / – | 15,392 / – | 14,244 / – | 5,761 / – |
| 4096 | **13,946** / – | – | 23,204 / – | 36,618 / – | 49,808 / – | 52,716 / – | 20,326 / – |

### mulmod: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 192 / 171 | 188 / 189 | **121** / **126** | 677 / 633 | 724 / – |
| 128 | **348** / **339** | – | 435 / 398 | 1,412 / 1,382 | 1,077 / – |
| 256 | 1,252 / 1,128 | – | **1,105** / **977** | 2,922 / 2,700 | 1,541 / – |
| 384 | 2,387 / 2,496 | – | **2,154** / **2,305** | 4,137 / 4,377 | 2,640 / – |
| 512 | **3,635** / 3,983 | – | 4,002 / **3,500** | 6,070 / 5,850 | 4,302 / – |
| 1024 | **10,282** / **11,318** | – | 12,485 / 12,718 | 17,159 / 17,830 | 10,866 / – |
| 2048 | **31,598** / **30,144** | – | 43,701 / 37,603 | 58,656 / 52,077 | 34,583 / – |
| 4096 | **89,590** / **106,615** | – | 170,805 / 154,296 | 186,889 / 185,848 | 131,876 / – |

### add_mod: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.7 / **7.5** | **3.4** / – | 45.2 / 44.7 | 11.8 / – | 8.3 / – | 3.5 / – | 296 / – |
| 128 | 17.1 / **18.0** | 9.7 / – | 31.5 / 47.2 | 25.0 / – | 11.5 / – | **9.5** / – | 371 / – |
| 256 | 30.5 / **41.8** | – | 43.0 / 80.4 | 53.6 / – | 28.0 / – | **26.8** / – | 367 / – |
| 384 | 64.7 / 108 | – | 62.6 / **106** | 82.7 / – | 53.2 / – | **35.9** / – | 403 / – |
| 512 | 75.0 / **92.7** | – | 91.5 / 141 | 114 / – | 74.7 / – | **55.9** / – | 410 / – |
| 1024 | 167 / **206** | – | 203 / 296 | 400 / – | **124** / – | 136 / – | 661 / – |
| 2048 | 300 / **397** | – | 838 / 951 | 607 / – | 353 / – | **260** / – | 1,010 / – |
| 4096 | 707 / **817** | – | 1,618 / 1,784 | 1,190 / – | 745 / – | **673** / – | 2,353 / – |

### sub_mod: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.3 / **7.6** | **3.5** / – | 60.6 / 52.8 | 5.2 / – | 5.2 / – | 4.5 / – | 386 / – |
| 128 | 16.5 / **18.8** | **8.6** / – | 40.3 / 52.3 | 10.7 / – | 10.0 / – | 10.4 / – | 453 / – |
| 256 | 32.7 / **58.6** | – | 67.8 / 100 | 35.6 / – | **19.7** / – | 20.7 / – | 743 / – |
| 384 | 40.3 / **79.4** | – | 86.6 / 138 | 58.5 / – | 40.1 / – | **38.7** / – | 751 / – |
| 512 | 126 / **119** | – | 121 / 175 | 76.2 / – | **54.4** / – | 74.5 / – | 561 / – |
| 1024 | 156 / **237** | – | 345 / 444 | 182 / – | **154** / – | 162 / – | 826 / – |
| 2048 | 413 / **459** | – | 1,128 / 1,248 | 630 / – | 384 / – | **348** / – | 1,896 / – |
| 4096 | **689** / **845** | – | 2,268 / 2,546 | 987 / – | 775 / – | 697 / – | 2,971 / – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / 1.3 | **2.3** / **1.2** | 2.5 / **1.2** | 2.4 / 1.3 | 2.5 / **1.2** | 2.4 / **1.2** |
| 128 | 4.4 / **2.9** | 4.2 / 3.0 | 4.2 / 3.1 | 4.5 / **2.9** | **4.1** / **2.9** | – |
| 256 | 10.4 / 10.0 | – | 10.5 / 9.3 | 10.5 / 9.7 | **10.3** / **8.2** | – |
| 384 | 16.6 / **15.8** | – | **16.4** / **15.8** | 18.1 / 17.3 | 17.7 / 16.5 | – |
| 512 | 27.5 / 27.0 | – | 27.4 / 26.5 | 32.8 / 33.3 | **25.1** / **22.8** | – |
| 1024 | 63.9 / 64.7 | – | 64.5 / 63.3 | 86.8 / 84.9 | **63.3** / **62.4** | – |
| 2048 | **159** / **162** | – | 208 / 211 | 192 / 232 | 212 / 251 | – |
| 4096 | 418 / 424 | – | 415 / **417** | **399** / 465 | 426 / 470 | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 12.0 / – | **11.6** / **11.9** | 12.0 / – | 25.7 / – | **11.6** / – | 12.0 / – |
| 256 | 69.2 / – | – | **64.9** / – | 91.7 / – | 103 / – | 67.3 / – |
| 384 | 178 / – | – | **167** / – | 209 / – | 244 / – | 173 / – |
| 512 | 348 / – | – | **327** / – | 376 / – | 424 / – | – |
| 1024 | 2,223 / – | – | – | 2,079 / – | **1,546** / – | 2,053 / – |
| 2048 | 8,455 / – | – | – | 8,289 / – | **6,380** / – | 9,877 / – |
| 4096 | 30,097 / – | – | – | 29,223 / – | **23,667** / – | 43,136 / – |

### div, constant time: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **93.2** / – | 385 / – |
| 128 | **227** / – | 903 / – |
| 256 | **623** / – | 1,680 / – |
| 384 | **907** / – | 2,542 / – |
| 512 | **1,132** / – | 3,667 / – |
| 1024 | **2,596** / – | 8,505 / – |
| 2048 | **7,566** / – | 22,529 / – |
| 4096 | **25,719** / – | 67,396 / – |

### mod, constant time: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **95.7** / – | 384 / – |
| 128 | **299** / – | 893 / – |
| 256 | **583** / – | 1,661 / – |
| 384 | **834** / – | 2,552 / – |
| 512 | **1,141** / – | 3,590 / – |
| 1024 | **2,569** / – | 8,488 / – |
| 2048 | **7,551** / – | 22,457 / – |
| 4096 | **25,284** / – | 68,173 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **319** / **308** | 518 / – |
| 128 | **697** / **718** | 1,470 / – |
| 256 | **1,630** / **1,781** | 3,190 / – |
| 384 | **3,245** / **3,563** | 5,822 / – |
| 512 | **4,877** / **4,942** | 8,795 / – |
| 1024 | **10,456** / **11,752** | 25,146 / – |
| 2048 | **42,519** / **40,593** | 78,063 / – |
| 4096 | **136,533** / **130,867** | 274,679 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 273 / **234** | **11.8** / – |
| 128 | 477 / **483** | **25.0** / – |
| 256 | 665 / **611** | **53.6** / – |
| 384 | 772 / **766** | **82.7** / – |
| 512 | 938 / **985** | **114** / – |
| 1024 | 1,511 / **1,488** | **400** / – |
| 2048 | 2,869 / **2,827** | **607** / – |
| 4096 | 5,299 / **5,560** | **1,190** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 246 / **237** | **5.2** / – |
| 128 | 467 / **442** | **10.7** / – |
| 256 | 622 / **613** | **35.6** / – |
| 384 | 839 / **792** | **58.5** / – |
| 512 | 887 / **856** | **76.2** / – |
| 1024 | 1,467 / **1,407** | **182** / – |
| 2048 | 2,902 / **3,027** | **630** / – |
| 4096 | 5,481 / **5,073** | **987** / – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / **1.2** | 2.6 / 1.3 | **2.4** / **1.2** | 2.5 / 1.3 | **2.4** / **1.2** | **2.4** / **1.2** |
| 128 | 4.7 / 2.6 | **4.0** / 2.7 | 4.9 / 2.7 | 4.8 / 2.9 | 4.2 / **2.5** | – |
| 256 | 10.0 / 8.2 | – | **9.1** / 9.0 | 9.6 / 7.5 | 9.7 / **6.2** | – |
| 384 | 14.9 / 13.7 | – | **14.2** / 13.7 | 15.6 / **12.9** | 16.1 / 15.2 | – |
| 512 | 23.5 / 19.7 | – | 24.0 / 19.9 | **21.8** / **19.0** | 23.6 / 23.7 | – |
| 1024 | 51.8 / **44.0** | – | 51.2 / 44.8 | **47.1** / 44.5 | 63.1 / 60.3 | – |
| 2048 | **114** / **97.7** | – | 171 / 169 | 136 / 195 | 182 / 231 | – |
| 4096 | 378 / 377 | – | 321 / **310** | **275** / 355 | 361 / 415 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.6 / – | 2.5 / **3.0** | 2.6 / – | **2.4** / – | 2.5 / – | 2.6 / – | – |
| 128 | 4.7 / – | **4.4** / **5.3** | 4.7 / – | 5.4 / – | 4.8 / – | 4.8 / – | – |
| 256 | 23.7 / – | – | **17.5** / – | 18.4 / – | 21.4 / – | 21.4 / – | – |
| 384 | 60.0 / – | – | 41.1 / – | **39.7** / – | 75.4 / – | 51.7 / – | – |
| 512 | 122 / – | – | **74.2** / – | 84.5 / – | 198 / – | – | – |
| 1024 | 739 / – | – | – | 594 / – | 633 / – | **508** / – | – |
| 2048 | 3,144 / – | – | **2,171** / – | 2,752 / – | 2,598 / – | 3,322 / – | 11,900 / – |
| 4096 | 10,526 / – | – | **8,128** / – | 11,383 / – | 10,034 / – | 14,675 / – | 40,259 / – |

### div: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 447 / – | **7.0** / – | **7.0** / – | 134 / – |
| 128 | 180 / – | 172 / – | **43.9** / – | 495 / – | 186 / – | 196 / – | 333 / – |
| 256 | **98.0** / – | – | 157 / – | 1,041 / – | 599 / – | 608 / – | 459 / – |
| 384 | **219** / – | – | 356 / – | 1,284 / – | 918 / – | 840 / – | 623 / – |
| 512 | **299** / – | – | 427 / – | 1,545 / – | 1,261 / – | 1,161 / – | 821 / – |
| 1024 | **754** / – | – | 917 / – | 3,059 / – | 2,809 / – | 2,447 / – | 1,924 / – |
| 2048 | **2,088** / – | – | 2,355 / – | 7,862 / – | 7,410 / – | 6,047 / – | 5,769 / – |
| 4096 | **7,128** / – | – | 7,694 / – | 20,156 / – | 22,297 / – | 20,118 / – | 20,289 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 448 / – | **7.0** / – | **7.0** / – | 33.4 / – |
| 128 | 180 / – | 174 / – | **39.9** / – | 495 / – | 188 / – | 196 / – | 355 / – |
| 256 | **83.6** / – | – | 159 / – | 1,043 / – | 603 / – | 591 / – | 469 / – |
| 384 | **215** / – | – | 327 / – | 1,286 / – | 924 / – | 836 / – | 620 / – |
| 512 | **292** / – | – | 407 / – | 1,544 / – | 1,258 / – | 1,191 / – | 831 / – |
| 1024 | **756** / – | – | 970 / – | 3,054 / – | 2,793 / – | 2,486 / – | 1,931 / – |
| 2048 | **2,108** / – | – | 2,298 / – | 7,824 / – | 7,404 / – | 6,075 / – | 5,792 / – |
| 4096 | **7,177** / – | – | 7,850 / – | 20,332 / – | 22,140 / – | 20,423 / – | 20,305 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 117 / 114 | 117 / 113 | **40.0** / **43.1** | 443 / 438 | 454 / – |
| 128 | **96.4** / **97.6** | – | 138 / 139 | 966 / 973 | 615 / – |
| 256 | 348 / 353 | – | **308** / **311** | 1,564 / 1,570 | 1,131 / – |
| 384 | 666 / 653 | – | **538** / **539** | 2,312 / 2,320 | 1,873 / – |
| 512 | 1,050 / 1,072 | – | **856** / **856** | 3,136 / 3,156 | 2,844 / – |
| 1024 | 3,415 / 3,391 | – | **2,638** / **2,626** | 8,346 / 8,467 | 8,976 / – |
| 2048 | **10,523** / **10,530** | – | 10,677 / 10,678 | 26,089 / 23,817 | 31,934 / – |
| 4096 | **34,836** / **34,768** | – | 41,300 / 41,169 | 74,855 / 74,928 | 113,564 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.3 / **7.3** | **4.0** / – | 44.3 / 39.8 | 12.2 / – | 8.0 / – | **4.0** / – | 302 / – |
| 128 | 15.1 / **16.2** | **9.0** / – | 21.6 / 30.9 | 21.2 / – | 11.0 / – | 9.4 / – | 312 / – |
| 256 | 28.0 / **31.0** | – | 42.9 / 62.2 | 48.3 / – | 22.2 / – | **21.9** / – | 341 / – |
| 384 | 62.9 / **73.4** | – | 60.6 / 80.6 | 72.1 / – | 44.1 / – | **40.6** / – | 379 / – |
| 512 | 61.9 / **73.1** | – | 86.0 / 100 | 94.7 / – | 61.7 / – | **45.6** / – | 413 / – |
| 1024 | 126 / **141** | – | 176 / 213 | 347 / – | 119 / – | **110** / – | 550 / – |
| 2048 | **241** / **276** | – | 747 / 784 | 513 / – | 334 / – | 248 / – | 938 / – |
| 4096 | **547** / **669** | – | 1,439 / 1,494 | 1,018 / – | 623 / – | 600 / – | 2,191 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 7.0 / **7.8** | **3.0** / – | 52.6 / 47.4 | 5.0 / – | 5.0 / – | 4.2 / – | 402 / – |
| 128 | 14.1 / **15.1** | **5.4** / – | 30.1 / 41.9 | 10.1 / – | 7.3 / – | 10.5 / – | 457 / – |
| 256 | 29.3 / **31.5** | – | 63.0 / 80.5 | 32.2 / – | **16.1** / – | 19.7 / – | 477 / – |
| 384 | 55.5 / **61.2** | – | 88.3 / 107 | 45.0 / – | 43.4 / – | **32.4** / – | 542 / – |
| 512 | 62.6 / **76.1** | – | 114 / 140 | 60.2 / – | **54.7** / – | 66.1 / – | 596 / – |
| 1024 | **106** / **123** | – | 340 / 373 | 141 / – | 142 / – | 132 / – | 823 / – |
| 2048 | **233** / **250** | – | 1,071 / 1,132 | 484 / – | 422 / – | 304 / – | 1,385 / – |
| 4096 | **537** / **668** | – | 2,002 / 2,113 | 762 / – | 746 / – | 703 / – | 2,992 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / 1.3 | 2.5 / **1.2** | **2.4** / 1.3 | 2.5 / **1.2** | 2.5 / **1.2** | 2.5 / 1.3 |
| 128 | 4.8 / 2.6 | **4.0** / **2.5** | 4.6 / 2.7 | 4.7 / 2.9 | 4.2 / **2.5** | – |
| 256 | 9.6 / 8.6 | – | **9.2** / 8.3 | 10.6 / 7.7 | 9.5 / **6.2** | – |
| 384 | 14.4 / 13.6 | – | **14.2** / 13.8 | 14.7 / **13.0** | 16.2 / 16.1 | – |
| 512 | 23.9 / 19.7 | – | 23.3 / 19.8 | **21.3** / **19.1** | 23.9 / 24.1 | – |
| 1024 | 51.6 / **44.0** | – | 50.6 / 44.7 | **46.3** / 45.0 | 61.9 / 59.6 | – |
| 2048 | **113** / **98.0** | – | 166 / 166 | 140 / 194 | 185 / 238 | – |
| 4096 | 356 / 349 | – | 305 / **308** | **275** / 357 | 360 / 415 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / – | 2.5 / **3.0** | 2.5 / – | 2.5 / – | 2.6 / – | **2.3** / – |
| 128 | 4.8 / – | 4.8 / **5.3** | **4.7** / – | 5.3 / – | **4.7** / – | **4.7** / – |
| 256 | 23.6 / – | – | **17.4** / – | 18.3 / – | 21.4 / – | 21.2 / – |
| 384 | 55.4 / – | – | 40.2 / – | **39.7** / – | 75.5 / – | 51.3 / – |
| 512 | 124 / – | – | **72.1** / – | 84.0 / – | 198 / – | – |
| 1024 | 736 / – | – | – | 593 / – | 633 / – | **508** / – |
| 2048 | 3,135 / – | – | – | 2,752 / – | **2,598** / – | 3,322 / – |
| 4096 | 10,732 / – | – | – | 11,383 / – | **10,034** / – | 14,675 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.6** / – | 320 / – |
| 128 | **178** / – | 754 / – |
| 256 | **355** / – | 1,363 / – |
| 384 | **564** / – | 2,039 / – |
| 512 | **806** / – | 2,757 / – |
| 1024 | **1,872** / – | 7,039 / – |
| 2048 | **5,367** / – | 16,837 / – |
| 4096 | **17,058** / – | 43,688 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.5** / – | 328 / – |
| 128 | **173** / – | 742 / – |
| 256 | **356** / – | 1,338 / – |
| 384 | **588** / – | 1,988 / – |
| 512 | **753** / – | 2,745 / – |
| 1024 | **1,903** / – | 7,031 / – |
| 2048 | **5,199** / – | 16,825 / – |
| 4096 | **17,148** / – | 43,715 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **88.0** / **95.0** | 352 / – |
| 128 | **372** / **404** | 998 / – |
| 256 | **864** / **791** | 1,985 / – |
| 384 | **1,305** / **1,349** | 3,245 / – |
| 512 | **2,027** / **1,995** | 4,665 / – |
| 1024 | **6,291** / **6,474** | 13,398 / – |
| 2048 | **21,296** / **21,266** | 36,611 / – |
| 4096 | **76,441** / **75,124** | 110,680 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 91.6 / **89.9** | **12.2** / – |
| 128 | 302 / **310** | **21.2** / – |
| 256 | 433 / **433** | **48.3** / – |
| 384 | 523 / **561** | **72.1** / – |
| 512 | 649 / **642** | **94.7** / – |
| 1024 | 1,126 / **1,128** | **347** / – |
| 2048 | 2,198 / **2,175** | **513** / – |
| 4096 | 3,989 / **4,039** | **1,018** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 90.8 / **91.3** | **5.0** / – |
| 128 | 301 / **311** | **10.1** / – |
| 256 | 392 / **387** | **32.2** / – |
| 384 | 537 / **521** | **45.0** / – |
| 512 | 574 / **568** | **60.2** / – |
| 1024 | 1,042 / **1,025** | **141** / – |
| 2048 | 2,046 / **2,025** | **484** / – |
| 4096 | 3,757 / **3,712** | **762** / – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **1.0** | 2.3 / 1.1 | **2.2** / 1.1 | 2.3 / 1.1 | **2.2** / **1.0** | **2.2** / **1.0** |
| 128 | 4.1 / 3.1 | 3.8 / 3.1 | 4.1 / 3.2 | 4.4 / **2.8** | **3.7** / 3.1 | – |
| 256 | **10.3** / 9.7 | – | **10.3** / 9.2 | 10.7 / 9.3 | **10.3** / **8.7** | – |
| 384 | **16.3** / **16.1** | – | 16.5 / **16.1** | 18.1 / 17.6 | 17.7 / 16.4 | – |
| 512 | 33.6 / 27.3 | – | 26.2 / 25.9 | 35.5 / 32.6 | **24.5** / **22.8** | – |
| 1024 | 75.1 / 78.2 | – | 71.9 / **75.2** | 133 / 86.3 | **71.4** / 81.9 | – |
| 2048 | **191** / **175** | – | 228 / 221 | 211 / 257 | 213 / 259 | – |
| 4096 | 438 / **437** | – | 449 / 441 | 419 / 494 | **411** / 474 | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.5** / – | **2.5** / **3.0** | 2.6 / – | 2.6 / – | 2.6 / – | 2.6 / – | – |
| 128 | 18.0 / – | 14.1 / **12.6** | 17.3 / – | 37.2 / – | **13.9** / – | 17.4 / – | – |
| 256 | 83.4 / – | – | **82.4** / – | 124 / – | 135 / – | 84.1 / – | – |
| 384 | 230 / – | – | **225** / – | 299 / – | 365 / – | 240 / – | – |
| 512 | 470 / – | – | **447** / – | 498 / – | 673 / – | – | – |
| 1024 | 2,563 / – | – | – | 2,601 / – | **2,340** / – | 3,004 / – | – |
| 2048 | 9,378 / – | – | 11,078 / – | 9,990 / – | **8,733** / – | 14,101 / – | 11,831 / – |
| 4096 | **29,763** / – | – | 40,002 / – | 35,825 / – | 33,449 / – | 67,998 / – | 39,955 / – |

### div: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 8.0 / – | 7.2 / – | **7.1** / – | 670 / – | 7.3 / – | 8.6 / – | 211 / – |
| 128 | 273 / – | 264 / – | **188** / – | 749 / – | 309 / – | 333 / – | 644 / – |
| 256 | **364** / – | – | 434 / – | 1,764 / – | 1,170 / – | 1,085 / – | 751 / – |
| 384 | **756** / – | – | 987 / – | 2,467 / – | 1,678 / – | 1,683 / – | 947 / – |
| 512 | **1,013** / – | – | 1,259 / – | 2,953 / – | 2,105 / – | 2,650 / – | 1,250 / – |
| 1024 | 2,560 / – | – | 3,553 / – | 4,618 / – | 4,971 / – | 7,084 / – | **2,413** / – |
| 2048 | 7,806 / – | – | 10,354 / – | 17,994 / – | 17,099 / – | 15,312 / – | **7,666** / – |
| 4096 | **21,918** / – | – | 27,849 / – | 66,681 / – | 47,424 / – | 45,952 / – | 24,727 / – |

### mod: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 7.9 / – | **7.1** / – | **7.1** / – | 649 / – | 8.1 / – | 7.9 / – | 51.4 / – |
| 128 | 275 / – | 275 / – | **173** / – | 752 / – | 289 / – | 316 / – | 508 / – |
| 256 | **358** / – | – | 474 / – | 1,613 / – | 1,216 / – | 1,182 / – | 786 / – |
| 384 | **608** / – | – | 846 / – | 2,235 / – | 1,635 / – | 1,640 / – | 928 / – |
| 512 | **989** / – | – | 1,125 / – | 2,836 / – | 2,349 / – | 2,902 / – | 1,364 / – |
| 1024 | 3,004 / – | – | 2,887 / – | 5,227 / – | 5,211 / – | 6,545 / – | **2,356** / – |
| 2048 | **7,362** / – | – | 8,958 / – | 15,154 / – | 17,696 / – | 14,837 / – | 7,500 / – |
| 4096 | **22,645** / – | – | 26,988 / – | 42,787 / – | 55,332 / – | 52,414 / – | 25,061 / – |

### mulmod: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 196 / 197 | 195 / 194 | **185** / **177** | 632 / 629 | 638 / – |
| 128 | **364** / **374** | – | 393 / 392 | 1,436 / 1,466 | 838 / – |
| 256 | 995 / 990 | – | **909** / **898** | 2,583 / 2,510 | 1,383 / – |
| 384 | 1,896 / 1,854 | – | **1,798** / **1,683** | 4,095 / 3,956 | 2,313 / – |
| 512 | **3,158** / **3,137** | – | 3,325 / 3,312 | 6,043 / 5,939 | 3,180 / – |
| 1024 | 9,917 / **9,615** | – | 12,430 / 12,357 | 17,579 / 16,863 | **9,620** / – |
| 2048 | 34,151 / **34,781** | – | 43,697 / 43,729 | 55,911 / 55,937 | **33,871** / – |
| 4096 | **106,096** / **104,591** | – | 174,817 / 172,716 | 190,460 / 185,984 | 122,721 / – |

### add_mod: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.8 / **8.3** | **3.7** / – | 82.0 / 78.2 | 11.9 / – | 7.9 / – | **3.7** / – | 365 / – |
| 128 | 21.5 / **39.3** | **9.7** / – | 22.5 / 52.0 | 23.0 / – | 10.0 / – | **9.7** / – | 385 / – |
| 256 | 52.9 / **60.1** | – | 59.8 / 101 | 57.3 / – | 25.7 / – | **21.8** / – | 412 / – |
| 384 | 99.2 / **133** | – | 95.3 / 137 | 86.1 / – | 48.9 / – | **33.0** / – | 436 / – |
| 512 | 123 / **140** | – | 122 / 176 | 139 / – | 62.9 / – | **46.6** / – | 468 / – |
| 1024 | 190 / **234** | – | 262 / 386 | 465 / – | 155 / – | **148** / – | 618 / – |
| 2048 | 349 / **399** | – | 1,024 / 1,144 | 895 / – | 391 / – | **288** / – | 1,029 / – |
| 4096 | **695** / **849** | – | 1,868 / 2,047 | 1,622 / – | 702 / – | 712 / – | 2,535 / – |

### sub_mod: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 8.0 / **7.9** | **3.0** / – | 87.1 / 80.3 | 5.0 / – | 5.8 / – | 5.3 / – | 453 / – |
| 128 | 20.0 / **39.5** | **7.4** / – | 37.0 / 64.3 | 11.0 / – | 8.4 / – | 9.2 / – | 475 / – |
| 256 | 47.1 / **69.3** | – | 88.0 / 138 | 31.7 / – | **19.2** / – | 20.6 / – | 531 / – |
| 384 | 84.2 / **112** | – | 137 / 199 | 57.8 / – | **32.5** / – | 38.2 / – | 586 / – |
| 512 | 99.5 / **146** | – | 176 / 247 | 94.3 / – | **63.4** / – | 71.7 / – | 635 / – |
| 1024 | 211 / **294** | – | 571 / 679 | 347 / – | 187 / – | **153** / – | 829 / – |
| 2048 | 599 / **674** | – | 1,415 / 1,509 | 567 / – | 423 / – | **383** / – | 1,364 / – |
| 4096 | **679** / **861** | – | 2,411 / 2,618 | 1,477 / – | 755 / – | 744 / – | 3,016 / – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / 1.1 | **2.2** / 1.1 | 2.3 / 1.1 | **2.2** / 1.1 | 2.3 / 1.1 | 2.3 / **1.0** |
| 128 | 4.0 / 3.1 | 3.8 / 3.1 | 3.9 / 3.2 | 4.4 / **2.8** | **3.7** / 3.0 | – |
| 256 | **10.1** / 9.3 | – | 10.5 / 9.2 | 10.9 / 9.2 | 10.3 / **9.0** | – |
| 384 | **16.0** / 16.9 | – | 16.1 / **15.7** | 17.5 / 17.5 | 18.0 / 16.5 | – |
| 512 | 33.3 / 27.0 | – | 27.0 / 26.6 | 35.9 / 32.1 | **24.2** / **23.1** | – |
| 1024 | 75.1 / 77.2 | – | **71.3** / **75.8** | 132 / 84.3 | 75.1 / 81.2 | – |
| 2048 | **188** / **176** | – | 231 / 218 | 215 / 259 | 212 / 274 | – |
| 4096 | 443 / 456 | – | 435 / **437** | 427 / 501 | **404** / 467 | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.7 / – | 2.7 / **3.0** | 2.6 / – | **2.4** / – | 2.7 / – | 2.5 / – |
| 128 | 17.4 / – | 14.1 / **12.6** | 17.3 / – | 37.2 / – | **13.7** / – | 17.7 / – |
| 256 | 83.7 / – | – | **82.4** / – | 124 / – | 135 / – | 84.3 / – |
| 384 | 233 / – | – | **225** / – | 300 / – | 361 / – | 243 / – |
| 512 | 495 / – | – | **448** / – | 501 / – | 668 / – | – |
| 1024 | 2,481 / – | – | – | 2,599 / – | **2,351** / – | 3,008 / – |
| 2048 | 9,211 / – | – | – | 9,990 / – | **8,733** / – | 14,101 / – |
| 4096 | **30,060** / – | – | – | 35,825 / – | 33,449 / – | 67,998 / – |

### div, constant time: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **198** / – | 495 / – |
| 128 | **469** / – | 1,202 / – |
| 256 | **883** / – | 2,055 / – |
| 384 | **1,227** / – | 3,404 / – |
| 512 | **1,522** / – | 4,599 / – |
| 1024 | **3,895** / – | 11,464 / – |
| 2048 | **10,672** / – | 29,178 / – |
| 4096 | **31,486** / – | 87,827 / – |

### mod, constant time: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **208** / – | 498 / – |
| 128 | **518** / – | 1,049 / – |
| 256 | **917** / – | 1,951 / – |
| 384 | **1,199** / – | 3,570 / – |
| 512 | **1,863** / – | 4,637 / – |
| 1024 | **4,675** / – | 11,419 / – |
| 2048 | **10,939** / – | 33,834 / – |
| 4096 | **34,356** / – | 83,197 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **321** / **319** | 529 / – |
| 128 | **767** / **754** | 1,556 / – |
| 256 | **1,762** / **1,701** | 3,324 / – |
| 384 | **3,197** / **3,018** | 5,843 / – |
| 512 | **4,915** / **4,870** | 8,728 / – |
| 1024 | **14,587** / **14,359** | 25,259 / – |
| 2048 | **47,374** / **47,427** | 79,425 / – |
| 4096 | **165,836** / **160,927** | 263,755 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 271 / **269** | **11.9** / – |
| 128 | 532 / **539** | **23.0** / – |
| 256 | 804 / **787** | **57.3** / – |
| 384 | 962 / **972** | **86.1** / – |
| 512 | 1,109 / **1,113** | **139** / – |
| 1024 | 1,886 / **1,856** | **465** / – |
| 2048 | 3,350 / **3,304** | **895** / – |
| 4096 | 6,167 / **6,163** | **1,622** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 271 / **254** | **5.0** / – |
| 128 | 524 / **552** | **11.0** / – |
| 256 | 735 / **744** | **31.7** / – |
| 384 | 913 / **930** | **57.8** / – |
| 512 | 1,095 / **1,072** | **94.3** / – |
| 1024 | 1,849 / **1,857** | **347** / – |
| 2048 | 3,434 / **3,393** | **567** / – |
| 4096 | 5,834 / **5,848** | **1,477** / – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.6** / 1.5 | 3.3 / 1.4 | 3.1 / 1.3 | 3.0 / 1.3 | 2.7 / 1.5 | 2.7 / **1.1** |
| 128 | 5.3 / 3.7 | 4.9 / **3.1** | 5.2 / 5.1 | 6.0 / 4.1 | **4.2** / 3.7 | – |
| 256 | 16.2 / 15.2 | – | 16.0 / 17.5 | 15.3 / 11.3 | **13.1** / **11.0** | – |
| 384 | 28.0 / 22.7 | – | 22.9 / 27.2 | 30.5 / 24.8 | **18.5** / **20.6** | – |
| 512 | 40.7 / 41.3 | – | 38.7 / 36.6 | **38.6** / **36.3** | 55.1 / 41.1 | – |
| 1024 | 117 / **67.2** | – | 78.9 / 75.7 | **73.9** / 82.4 | 84.5 / 89.7 | – |
| 2048 | **155** / **162** | – | 236 / 208 | 185 / 266 | 237 / 267 | – |
| 4096 | 479 / 477 | – | 424 / **400** | **397** / 459 | 601 / 610 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 3.0 / – | 2.6 / **3.0** | 2.6 / – | 2.9 / – | **2.4** / – | 2.7 / – | – |
| 128 | 6.1 / – | 5.9 / **6.4** | 5.8 / – | **5.5** / – | 5.7 / – | 6.9 / – | – |
| 256 | **45.4** / – | – | 88.2 / – | 95.6 / – | 127 / – | 133 / – | – |
| 384 | **169** / – | – | 206 / – | 232 / – | 358 / – | 289 / – | – |
| 512 | **234** / – | – | 304 / – | 347 / – | 690 / – | – | – |
| 1024 | 1,295 / – | – | – | **902** / – | 1,517 / – | 1,331 / – | – |
| 2048 | 5,367 / – | – | 4,970 / – | 6,371 / – | **4,492** / – | 6,112 / – | 14,272 / – |
| 4096 | 21,797 / – | – | 13,313 / – | 28,894 / – | **10,571** / – | 28,080 / – | 51,904 / – |

### div: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 7.1 / – | **7.0** / – | **7.0** / – | 492 / – | **7.0** / – | **7.0** / – | 173 / – |
| 128 | 245 / – | 238 / – | **55.3** / – | 549 / – | 241 / – | 254 / – | 430 / – |
| 256 | 186 / – | – | **176** / – | 1,083 / – | 736 / – | 691 / – | 587 / – |
| 384 | **259** / – | – | 430 / – | 1,357 / – | 1,112 / – | 1,028 / – | 776 / – |
| 512 | **372** / – | – | 559 / – | 1,631 / – | 1,505 / – | 1,430 / – | 1,005 / – |
| 1024 | **935** / – | – | 1,128 / – | 3,323 / – | 3,376 / – | 3,101 / – | 2,198 / – |
| 2048 | **2,369** / – | – | 2,738 / – | 7,831 / – | 10,465 / – | 7,748 / – | 6,179 / – |
| 4096 | **7,607** / – | – | 9,040 / – | 23,334 / – | 31,458 / – | 25,568 / – | 20,309 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 489 / – | **7.0** / – | **7.0** / – | 31.2 / – |
| 128 | 245 / – | 242 / – | **51.3** / – | 548 / – | 253 / – | 254 / – | 437 / – |
| 256 | **182** / – | – | **182** / – | 1,082 / – | 731 / – | 701 / – | 599 / – |
| 384 | **278** / – | – | 345 / – | 1,360 / – | 1,113 / – | 1,032 / – | 786 / – |
| 512 | **380** / – | – | 463 / – | 1,637 / – | 1,506 / – | 1,403 / – | 993 / – |
| 1024 | **824** / – | – | 1,083 / – | 3,299 / – | 3,379 / – | 3,106 / – | 2,209 / – |
| 2048 | **2,268** / – | – | 2,658 / – | 7,859 / – | 10,399 / – | 7,803 / – | 6,213 / – |
| 4096 | **7,549** / – | – | 8,973 / – | 23,388 / – | 31,263 / – | 25,300 / – | 20,341 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 186 / 184 | 184 / 181 | **59.8** / **54.4** | 476 / 476 | 607 / – |
| 128 | **153** / **181** | – | 207 / 207 | 1,086 / 1,094 | 821 / – |
| 256 | 406 / 399 | – | **385** / **387** | 1,592 / 1,617 | 1,430 / – |
| 384 | 999 / 964 | – | **696** / **690** | 2,412 / 2,380 | 2,198 / – |
| 512 | 1,623 / 1,654 | – | **1,148** / **1,127** | 2,981 / 3,009 | 3,172 / – |
| 1024 | 5,029 / 5,058 | – | **3,591** / **3,585** | 9,017 / 8,734 | 9,447 / – |
| 2048 | 15,528 / 15,616 | – | **13,902** / **13,806** | 28,332 / 28,413 | 32,334 / – |
| 4096 | 54,898 / 55,014 | – | **54,112** / **53,525** | 99,587 / 99,522 | 117,436 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 11.3 / **10.3** | **5.0** / – | 84.8 / 80.4 | 13.2 / – | 9.0 / – | **5.0** / – | 356 / – |
| 128 | 22.6 / **36.0** | **9.2** / – | 25.4 / 40.1 | 22.8 / – | 10.6 / – | 12.5 / – | 408 / – |
| 256 | 51.0 / **47.9** | – | 61.6 / 90.2 | 54.7 / – | 31.5 / – | **24.4** / – | 431 / – |
| 384 | 98.8 / 119 | – | 87.8 / **106** | 82.3 / – | 55.3 / – | **48.0** / – | 440 / – |
| 512 | 107 / **131** | – | 127 / 147 | 128 / – | **59.3** / – | 65.6 / – | 470 / – |
| 1024 | 191 / **215** | – | 252 / 342 | 393 / – | **161** / – | 163 / – | 633 / – |
| 2048 | 304 / **404** | – | 931 / 982 | 769 / – | 416 / – | **291** / – | 1,020 / – |
| 4096 | **684** / **840** | – | 1,713 / 1,799 | 1,398 / – | 778 / – | 751 / – | 2,507 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 13.9 / **11.4** | **3.5** / – | 99.2 / 85.3 | 5.5 / – | 7.0 / – | 7.9 / – | 680 / – |
| 128 | 28.3 / **41.0** | 9.5 / – | 49.8 / 61.0 | 12.1 / – | **9.0** / – | 11.0 / – | 521 / – |
| 256 | 64.3 / **67.5** | – | 95.2 / 143 | 34.4 / – | **28.8** / – | 38.4 / – | 761 / – |
| 384 | 120 / **115** | – | 151 / 166 | **53.6** / – | 65.6 / – | 55.9 / – | 806 / – |
| 512 | 140 / **128** | – | 181 / 253 | 108 / – | **86.3** / – | 90.1 / – | 809 / – |
| 1024 | 191 / **232** | – | 625 / 655 | 276 / – | 286 / – | **166** / – | 1,300 / – |
| 2048 | **341** / **367** | – | 1,438 / 1,433 | 540 / – | 470 / – | 462 / – | 2,314 / – |
| 4096 | 857 / **928** | – | 2,563 / 2,515 | 1,318 / – | 862 / – | **820** / – | 4,383 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.9 / **1.2** | 2.5 / 1.5 | 2.5 / 1.5 | 3.1 / **1.2** | **2.4** / 1.3 | 2.6 / 1.4 |
| 128 | 5.8 / 3.7 | **3.9** / **3.5** | 5.4 / 4.4 | 5.9 / 4.9 | 4.0 / 4.0 | – |
| 256 | 15.1 / **11.8** | – | 15.4 / 16.5 | 14.5 / 13.3 | **13.4** / 11.9 | – |
| 384 | 33.4 / 30.9 | – | 25.6 / **17.8** | 23.1 / 21.2 | **21.2** / 18.7 | – |
| 512 | 38.5 / **30.0** | – | 35.7 / 37.5 | 36.8 / 35.4 | **27.1** / 43.5 | – |
| 1024 | 83.0 / 76.8 | – | 89.0 / **68.9** | **75.6** / 85.6 | 150 / 73.8 | – |
| 2048 | **220** / **194** | – | 336 / 288 | 240 / 245 | 249 / 303 | – |
| 4096 | 584 / 568 | – | 703 / **403** | **375** / 593 | 654 / 600 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 3.3 / – | 3.2 / **3.0** | **2.3** / – | 2.8 / – | 2.6 / – | 3.2 / – |
| 128 | 6.5 / – | 5.8 / **6.6** | 6.7 / – | 6.0 / – | **5.5** / – | 5.8 / – |
| 256 | **33.6** / – | – | 84.8 / – | 103 / – | 117 / – | 141 / – |
| 384 | **96.7** / – | – | 227 / – | 246 / – | 323 / – | 291 / – |
| 512 | **196** / – | – | 307 / – | 347 / – | 864 / – | – |
| 1024 | 1,433 / – | – | – | 1,418 / – | 1,390 / – | **1,220** / – |
| 2048 | 7,790 / – | – | – | 6,371 / – | **4,492** / – | 6,112 / – |
| 4096 | 19,802 / – | – | – | 28,894 / – | **10,571** / – | 28,080 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **59.9** / – | 344 / – |
| 128 | **266** / – | 894 / – |
| 256 | **474** / – | 1,539 / – |
| 384 | **609** / – | 2,312 / – |
| 512 | **912** / – | 3,105 / – |
| 1024 | **2,327** / – | 7,526 / – |
| 2048 | **6,794** / – | 18,507 / – |
| 4096 | **22,058** / – | 50,830 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **67.0** / – | 345 / – |
| 128 | **244** / – | 879 / – |
| 256 | **428** / – | 1,523 / – |
| 384 | **566** / – | 2,232 / – |
| 512 | **877** / – | 3,092 / – |
| 1024 | **2,135** / – | 7,477 / – |
| 2048 | **6,240** / – | 18,733 / – |
| 4096 | **21,262** / – | 51,016 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **147** / **135** | 383 / – |
| 128 | **401** / **387** | 1,157 / – |
| 256 | **922** / **930** | 2,275 / – |
| 384 | **1,721** / **1,731** | 3,727 / – |
| 512 | **2,762** / **2,719** | 5,552 / – |
| 1024 | **8,400** / **8,425** | 14,104 / – |
| 2048 | **28,644** / **27,887** | 46,505 / – |
| 4096 | **100,415** / **99,186** | 147,079 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 117 / **116** | **13.2** / – |
| 128 | 305 / **308** | **22.8** / – |
| 256 | 482 / **469** | **54.7** / – |
| 384 | 630 / **619** | **82.3** / – |
| 512 | 718 / **715** | **128** / – |
| 1024 | 1,336 / **1,394** | **393** / – |
| 2048 | 2,619 / **2,554** | **769** / – |
| 4096 | 4,745 / **4,778** | **1,398** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 152 / **149** | **5.5** / – |
| 128 | 325 / **326** | **12.1** / – |
| 256 | 441 / **437** | **34.4** / – |
| 384 | 578 / **557** | **53.6** / – |
| 512 | 692 / **658** | **108** / – |
| 1024 | 1,320 / **1,218** | **276** / – |
| 2048 | 2,542 / **2,400** | **540** / – |
| 4096 | 4,667 / **4,471** | **1,318** / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
