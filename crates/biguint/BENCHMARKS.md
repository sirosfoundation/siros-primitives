# Benchmarks

`biguint` at commit `830bbfc` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 54.74 before, 57.34 after; its SMT sibling, CPU 29, 98% busy |
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
| 64 | 1.3 / **1.0** | **1.1** / 2.3 | 1.7 / 1.7 | 1.6 / 1.9 | 1.8 / 2.7 | 1.5 / 1.7 |
| 128 | 2.4 / **0.8** | 2.4 / 1.3 | **2.3** / 1.3 | 2.5 / 1.7 | **2.3** / 1.3 | – |
| 256 | 4.9 / 2.5 | – | **4.4** / **2.2** | 5.3 / 5.3 | 4.5 / 3.0 | – |
| 384 | 7.2 / **3.5** | – | **7.1** / 3.9 | 9.0 / 10.6 | 7.9 / 5.1 | – |
| 512 | **8.6** / **4.0** | – | 9.4 / 4.5 | 12.3 / 16.5 | 11.4 / 9.5 | – |
| 1024 | **24.2** / **23.7** | – | 26.6 / 23.8 | 28.4 / 28.8 | 28.1 / 30.0 | – |
| 2048 | **55.0** / **54.3** | – | 67.8 / 62.7 | 79.9 / 98.2 | 87.8 / 108 | – |
| 4096 | **156** / **176** | – | 187 / 189 | 211 / 270 | 213 / 260 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | 1.4 / **2.9** | 1.4 / – | **1.0** / – | 1.4 / – | 1.5 / – | – |
| 128 | 3.0 / – | **2.1** / **4.9** | 3.0 / – | 2.9 / – | 3.0 / – | 3.0 / – | – |
| 256 | **12.0** / – | – | 12.6 / – | 12.8 / – | 13.0 / – | 16.0 / – | – |
| 384 | **27.5** / – | – | 29.9 / – | 51.5 / – | 30.0 / – | 36.8 / – | – |
| 512 | **52.2** / – | – | 59.9 / – | 80.1 / – | 90.3 / – | – | – |
| 1024 | **239** / – | – | – | 434 / – | 374 / – | 437 / – | – |
| 2048 | **1,117** / – | – | 1,606 / – | 1,980 / – | 1,513 / – | 4,650 / – | 3,568 / – |
| 4096 | **3,853** / – | – | 6,475 / – | 7,718 / – | 6,116 / – | 24,876 / – | 11,309 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | **6.9** / – | **6.9** / – | 219 / – | **6.9** / – | **6.9** / – | 37.0 / – |
| 128 | 16.5 / – | **16.2** / – | 42.1 / – | 266 / – | 29.7 / – | 29.9 / – | 98.3 / – |
| 256 | **57.5** / – | – | 81.3 / – | 550 / – | 246 / – | 202 / – | 269 / – |
| 384 | **119** / – | – | 197 / – | 710 / – | 356 / – | 313 / – | 339 / – |
| 512 | **161** / – | – | 262 / – | 823 / – | 448 / – | 404 / – | 392 / – |
| 1024 | **585** / – | – | 662 / – | 1,836 / – | 1,293 / – | 1,254 / – | 984 / – |
| 2048 | **1,417** / – | – | 1,618 / – | 4,297 / – | 3,918 / – | 3,782 / – | 2,025 / – |
| 4096 | **4,752** / – | – | 5,985 / – | 12,473 / – | 11,185 / – | 17,072 / – | 7,612 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | **6.9** / – | **6.9** / – | 224 / – | **6.9** / – | 7.0 / – | 7.5 / – |
| 128 | 18.6 / – | **17.2** / – | 30.8 / – | 271 / – | 30.8 / – | 30.8 / – | 104 / – |
| 256 | **53.3** / – | – | 77.9 / – | 535 / – | 252 / – | 199 / – | 268 / – |
| 384 | **121** / – | – | 194 / – | 712 / – | 369 / – | 325 / – | 350 / – |
| 512 | **166** / – | – | 242 / – | 846 / – | 469 / – | 396 / – | 370 / – |
| 1024 | **463** / – | – | 581 / – | 1,659 / – | 1,117 / – | 1,377 / – | 1,012 / – |
| 2048 | **1,328** / – | – | 1,530 / – | 4,079 / – | 3,548 / – | 3,378 / – | 2,441 / – |
| 4096 | **4,774** / – | – | 5,075 / – | 15,323 / – | 9,767 / – | 17,020 / – | 6,588 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **11.9** / **22.9** | **11.9** / 23.0 | 19.7 / 44.5 | 227 / 210 | 216 / – |
| 128 | **61.9** / **60.2** | – | 94.2 / 96.3 | 528 / 508 | 256 / – |
| 256 | 178 / **165** | – | **170** / 230 | 1,013 / 986 | 575 / – |
| 384 | **361** / **373** | – | 433 / 408 | 1,609 / 1,537 | 1,072 / – |
| 512 | **567** / **568** | – | 609 / 576 | 2,044 / 1,903 | 1,600 / – |
| 1024 | **1,387** / **1,373** | – | 1,626 / 1,595 | 5,955 / 5,558 | 3,988 / – |
| 2048 | **5,893** / **4,807** | – | 7,428 / 7,376 | 16,528 / 18,787 | 10,827 / – |
| 4096 | **16,763** / **17,710** | – | 34,254 / 32,812 | 68,964 / 72,141 | 36,735 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 3.7 / **8.5** | **2.9** / – | 11.9 / 13.3 | 13.1 / – | 17.0 / – | **2.9** / – | 98.3 / – |
| 128 | **4.8** / **11.7** | 5.8 / – | 34.8 / 22.5 | 27.6 / – | 16.7 / – | 25.3 / – | 305 / – |
| 256 | **11.2** / **20.7** | – | 49.7 / 65.4 | 72.1 / – | 24.3 / – | 57.1 / – | 314 / – |
| 384 | **22.4** / **32.3** | – | 62.1 / 89.9 | 100 / – | 32.7 / – | 93.3 / – | 330 / – |
| 512 | 43.1 / **31.8** | – | 73.1 / 93.9 | 136 / – | **38.4** / – | 131 / – | 248 / – |
| 1024 | **102** / **87.6** | – | 173 / 189 | 189 / – | 103 / – | 278 / – | 580 / – |
| 2048 | **196** / **190** | – | 277 / 306 | 489 / – | 213 / – | 602 / – | 781 / – |
| 4096 | **350** / **374** | – | 845 / 879 | 782 / – | 470 / – | 1,458 / – | 892 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **4.1** / **7.4** | 13.0 / – | 16.3 / 16.8 | 5.5 / – | 14.1 / – | 12.7 / – | 224 / – |
| 128 | 5.8 / **11.3** | **5.1** / – | 36.0 / 24.3 | 11.1 / – | 14.9 / – | 18.1 / – | 483 / – |
| 256 | **10.9** / **16.1** | – | 52.7 / 69.6 | 52.5 / – | 25.1 / – | 50.8 / – | 464 / – |
| 384 | **19.8** / **39.0** | – | 93.7 / 99.6 | 85.6 / – | 33.1 / – | 85.0 / – | 495 / – |
| 512 | **34.3** / **43.3** | – | 106 / 135 | 130 / – | 52.1 / – | 146 / – | 638 / – |
| 1024 | 89.2 / **79.6** | – | 224 / 236 | 149 / – | **83.6** / – | 305 / – | 608 / – |
| 2048 | **183** / **190** | – | 523 / 583 | 332 / – | 196 / – | 717 / – | 973 / – |
| 4096 | **300** / **435** | – | 1,183 / 1,164 | 688 / – | 466 / – | 1,672 / – | 3,125 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **0.9** | **1.4** / **0.9** | **1.4** / 1.0 | **1.4** / 1.0 | **1.4** / 1.0 | **1.4** / **0.9** |
| 128 | 2.4 / 1.2 | 2.4 / 1.2 | 2.6 / 1.4 | 2.5 / **1.1** | **2.3** / **1.1** | – |
| 256 | **4.5** / **2.2** | – | 4.6 / **2.2** | 5.4 / 5.4 | 5.0 / 2.9 | – |
| 384 | 7.1 / **3.5** | – | **6.8** / 4.0 | 9.1 / 10.2 | 7.9 / 4.9 | – |
| 512 | **9.1** / **3.2** | – | 9.7 / 4.5 | 12.2 / 16.8 | 11.1 / 7.6 | – |
| 1024 | **23.6** / **22.1** | – | 25.1 / 23.9 | 36.2 / 37.4 | 36.3 / 35.0 | – |
| 2048 | **63.8** / **54.3** | – | 94.9 / 77.5 | 88.0 / 137 | 123 / 136 | – |
| 4096 | **216** / **222** | – | 219 / 228 | 252 / 320 | 271 / 319 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.5 / – | **1.0** / **2.9** | 1.4 / – | 1.4 / – | 1.3 / – | 1.5 / – |
| 128 | **2.9** / – | **2.9** / **4.9** | **2.9** / – | **2.9** / – | 3.1 / – | 3.0 / – |
| 256 | 12.4 / – | – | **12.1** / – | 12.8 / – | 13.0 / – | 16.0 / – |
| 384 | **29.6** / – | – | 30.5 / – | 30.0 / – | 33.7 / – | 36.7 / – |
| 512 | **51.4** / – | – | 59.6 / – | 80.2 / – | 91.2 / – | – |
| 1024 | **227** / – | – | – | 436 / – | 401 / – | 441 / – |
| 2048 | **1,161** / – | – | – | 1,980 / – | 1,513 / – | 4,650 / – |
| 4096 | **4,193** / – | – | – | 7,718 / – | 6,116 / – | 24,876 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.5** / – | 197 / – |
| 128 | **116** / – | 415 / – |
| 256 | **270** / – | 771 / – |
| 384 | **400** / – | 1,337 / – |
| 512 | **363** / – | 1,450 / – |
| 1024 | **1,144** / – | 3,358 / – |
| 2048 | **3,438** / – | 10,299 / – |
| 4096 | **12,251** / – | 30,814 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.8** / – | 193 / – |
| 128 | **112** / – | 385 / – |
| 256 | **297** / – | 724 / – |
| 384 | **305** / – | 1,033 / – |
| 512 | **492** / – | 1,465 / – |
| 1024 | **1,194** / – | 3,810 / – |
| 2048 | **4,101** / – | 10,625 / – |
| 4096 | **13,281** / – | 30,964 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **71.3** / **95.0** | 280 / – |
| 128 | **263** / **270** | 582 / – |
| 256 | **439** / **376** | 1,308 / – |
| 384 | **895** / **954** | 2,153 / – |
| 512 | **1,510** / **1,226** | 3,272 / – |
| 1024 | **4,334** / **4,192** | 8,471 / – |
| 2048 | **14,964** / **14,626** | 29,801 / – |
| 4096 | **51,856** / **51,232** | 103,181 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **5.8** / **18.9** | 13.1 / – |
| 128 | **21.7** / **22.7** | 27.6 / – |
| 256 | **30.4** / **26.2** | 72.1 / – |
| 384 | **58.8** / **46.2** | 100 / – |
| 512 | **72.9** / **73.1** | 136 / – |
| 1024 | **112** / **155** | 189 / – |
| 2048 | 587 / **396** | **489** / – |
| 4096 | 1,020 / **849** | **782** / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 9.8 / **9.3** | **5.5** / – |
| 128 | 17.8 / **19.7** | **11.1** / – |
| 256 | **29.7** / **25.6** | 52.5 / – |
| 384 | **47.6** / **43.3** | 85.6 / – |
| 512 | **60.8** / **57.0** | 130 / – |
| 1024 | **139** / **124** | 149 / – |
| 2048 | **286** / **300** | 332 / – |
| 4096 | **557** / **614** | 688 / – |

## WebAssembly

`biguint` at commit `d7a869e` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-04.

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
| Cost of a count, in WebAssembly, Chrome | 4,143 to 4,387 cycles and 3,081 instructions; its noise in a figure, 39.0 to 218 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,115 to 4,625 cycles and 3,081 instructions; its noise in a figure, 95.0 to 196 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 3,839 to 4,899 cycles and 1,885 instructions; its noise in a figure, 75.0 to 808 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 3,895 to 4,458 cycles and 1,885 instructions; its noise in a figure, 39.0 to 641 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 4.51 before, 4.92 after; its SMT sibling, CPU 28, 1% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 5.08 before, 5.23 after; its SMT sibling, CPU 28, 1% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 5.23 before, 4.61 after; its SMT sibling, CPU 28, 0% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 4.61 before, 5.79 after; its SMT sibling, CPU 28, 0% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** |
| 128 | **4.0** / 3.0 | **4.0** / **2.8** | **4.0** / 2.9 | **4.0** / **2.8** | **4.0** / **2.8** | – |
| 256 | **10.2** / 9.8 | – | **10.2** / 9.4 | **10.2** / 9.8 | 10.3 / **8.1** | – |
| 384 | **16.3** / 15.4 | – | **16.3** / **15.3** | 18.1 / 17.1 | 17.1 / 16.6 | – |
| 512 | 27.1 / 25.7 | – | 27.0 / 26.1 | 33.1 / 33.2 | **24.8** / **22.3** | – |
| 1024 | 64.3 / 64.2 | – | 63.9 / 63.8 | 85.5 / 85.1 | **61.7** / **62.1** | – |
| 2048 | **158** / **160** | – | 202 / 208 | 191 / 232 | 206 / 244 | – |
| 4096 | 431 / 425 | – | 407 / **405** | **395** / 454 | 411 / 469 | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | **11.0** / – | 11.5 / **11.8** | **11.0** / – | 24.5 / – | 11.1 / – | **11.0** / – | – |
| 256 | **66.4** / – | – | **66.4** / – | 95.9 / – | 120 / – | 68.5 / – | – |
| 384 | 177 / – | – | 166 / – | 206 / – | 242 / – | **162** / – | – |
| 512 | 342 / – | – | **321** / – | 377 / – | 425 / – | – | – |
| 1024 | 2,257 / – | – | – | 2,029 / – | **1,545** / – | 2,038 / – | – |
| 2048 | 8,435 / – | – | 6,361 / – | 8,015 / – | **6,270** / – | 9,857 / – | 11,785 / – |
| 4096 | 28,490 / – | – | 38,968 / – | 28,997 / – | **23,591** / – | 42,059 / – | 39,751 / – |

### div: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 500 / – | **7.0** / – | **7.0** / – | 125 / – |
| 128 | 182 / – | 171 / – | **87.0** / – | 566 / – | 209 / – | 212 / – | 326 / – |
| 256 | **230** / – | – | 322 / – | 1,292 / – | 812 / – | 848 / – | 453 / – |
| 384 | **467** / – | – | 604 / – | 1,700 / – | 1,297 / – | 1,279 / – | 616 / – |
| 512 | **752** / – | – | 833 / – | 2,083 / – | 1,840 / – | 1,803 / – | 806 / – |
| 1024 | **1,461** / – | – | 2,122 / – | 4,148 / – | 4,558 / – | 4,522 / – | 1,935 / – |
| 2048 | **3,938** / – | – | 6,501 / – | 11,139 / – | 15,407 / – | 14,206 / – | 5,753 / – |
| 4096 | **13,186** / – | – | 23,190 / – | 36,575 / – | 48,778 / – | 51,993 / – | 19,972 / – |

### mod: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 518 / – | **7.0** / – | **7.0** / – | 33.6 / – |
| 128 | 181 / – | 170 / – | **88.3** / – | 569 / – | 215 / – | 212 / – | 335 / – |
| 256 | **235** / – | – | 323 / – | 1,294 / – | 813 / – | 845 / – | 463 / – |
| 384 | **467** / – | – | 582 / – | 1,699 / – | 1,303 / – | 1,285 / – | 616 / – |
| 512 | **755** / – | – | 791 / – | 2,084 / – | 1,841 / – | 1,804 / – | 816 / – |
| 1024 | **1,427** / – | – | 2,089 / – | 4,120 / – | 4,600 / – | 4,562 / – | 1,942 / – |
| 2048 | **3,939** / – | – | 6,487 / – | 11,183 / – | 15,340 / – | 14,233 / – | 5,782 / – |
| 4096 | **14,892** / – | – | 23,160 / – | 36,571 / – | 49,012 / – | 51,871 / – | 20,021 / – |

### mulmod: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 133 / 148 | 139 / 132 | **83.7** / **85.1** | 542 / 541 | 429 / – |
| 128 | **311** / **288** | – | 326 / 336 | 1,252 / 1,258 | 601 / – |
| 256 | **804** / 871 | – | 816 / **813** | 2,262 / 2,283 | 1,114 / – |
| 384 | **1,609** / 1,750 | – | **1,609** / **1,605** | 3,584 / 3,513 | 1,847 / – |
| 512 | **2,775** / 3,037 | – | 2,778 / **2,781** | 5,022 / 5,095 | 2,802 / – |
| 1024 | **8,604** / **8,643** | – | 11,103 / 11,043 | 15,218 / 15,176 | 8,898 / – |
| 2048 | **28,203** / **28,492** | – | 34,984 / 35,121 | 49,504 / 49,777 | 32,008 / – |
| 4096 | **93,938** / **94,412** | – | 160,834 / 160,411 | 175,578 / 176,467 | 112,543 / – |

### add_mod: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.0 / **7.2** | **3.3** / – | 43.5 / 41.3 | 12.9 / – | 7.9 / – | **3.3** / – | 297 / – |
| 128 | 15.3 / **18.4** | **9.6** / – | 23.4 / 40.8 | 23.7 / – | 11.1 / – | 9.7 / – | 305 / – |
| 256 | 30.5 / **37.6** | – | 43.1 / 77.5 | 47.5 / – | 22.7 / – | **22.2** / – | 339 / – |
| 384 | 61.9 / **100.0** | – | 62.1 / 101 | 96.8 / – | 45.7 / – | **34.4** / – | 369 / – |
| 512 | 75.1 / **90.7** | – | 90.6 / 131 | 109 / – | 63.9 / – | **46.2** / – | 399 / – |
| 1024 | 149 / **178** | – | 199 / 259 | 377 / – | 118 / – | **104** / – | 544 / – |
| 2048 | 295 / **343** | – | 788 / 892 | 575 / – | 355 / – | **252** / – | 919 / – |
| 4096 | 634 / **724** | – | 1,526 / 1,701 | 1,167 / – | 681 / – | **569** / – | 2,081 / – |

### sub_mod: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.0 / **7.8** | **3.0** / – | 52.5 / 47.0 | 5.0 / – | 4.9 / – | 4.3 / – | 407 / – |
| 128 | 14.1 / **19.0** | **6.8** / – | 31.1 / 47.4 | 10.3 / – | 8.7 / – | 9.2 / – | 415 / – |
| 256 | 36.0 / **52.0** | – | 60.3 / 90.9 | 28.1 / – | 18.8 / – | **18.7** / – | 466 / – |
| 384 | 52.6 / **75.5** | – | 82.5 / 129 | 48.9 / – | 40.2 / – | **31.1** / – | 537 / – |
| 512 | 89.7 / **121** | – | 120 / 164 | 71.4 / – | **46.1** / – | 62.6 / – | 590 / – |
| 1024 | 153 / **185** | – | 358 / 429 | 177 / – | 138 / – | **134** / – | 809 / – |
| 2048 | 371 / **411** | – | 1,115 / 1,231 | 518 / – | 360 / – | **293** / – | 1,355 / – |
| 4096 | 662 / **783** | – | 2,105 / 2,284 | 935 / – | **660** / – | 723 / – | 2,870 / – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** |
| 128 | **4.0** / 3.0 | **4.0** / **2.8** | **4.0** / 2.9 | **4.0** / **2.8** | **4.0** / **2.8** | – |
| 256 | **10.2** / 9.8 | – | 10.3 / 9.4 | **10.2** / 9.8 | 10.3 / **8.1** | – |
| 384 | **16.3** / **15.4** | – | 16.4 / **15.4** | 18.0 / 17.2 | 17.1 / 16.6 | – |
| 512 | 27.3 / 25.7 | – | 26.9 / 26.0 | 33.1 / 33.3 | **24.8** / **22.3** | – |
| 1024 | 64.3 / 64.3 | – | 63.7 / 63.7 | 85.8 / 86.0 | **61.7** / **62.1** | – |
| 2048 | **158** / **158** | – | 202 / 208 | 190 / 232 | 206 / 244 | – |
| 4096 | 432 / 424 | – | 407 / **405** | **391** / 455 | 411 / 469 | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | **11.0** / – | 11.6 / **11.8** | **11.0** / – | 24.5 / – | 11.1 / – | **11.0** / – |
| 256 | 69.5 / – | – | **66.7** / – | 96.0 / – | 120 / – | 69.1 / – |
| 384 | 180 / – | – | 165 / – | 206 / – | 241 / – | **164** / – |
| 512 | 350 / – | – | **320** / – | 376 / – | 426 / – | – |
| 1024 | 2,215 / – | – | – | 2,033 / – | **1,544** / – | 2,009 / – |
| 2048 | 8,259 / – | – | – | 8,015 / – | **6,270** / – | 9,857 / – |
| 4096 | 29,328 / – | – | – | 28,997 / – | **23,591** / – | 42,059 / – |

### div, constant time: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **97.1** / – | 386 / – |
| 128 | **228** / – | 899 / – |
| 256 | **614** / – | 1,689 / – |
| 384 | **863** / – | 2,517 / – |
| 512 | **1,097** / – | 3,669 / – |
| 1024 | **2,574** / – | 8,515 / – |
| 2048 | **7,546** / – | 22,506 / – |
| 4096 | **25,582** / – | 66,992 / – |

### mod, constant time: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **99.2** / – | 385 / – |
| 128 | **222** / – | 902 / – |
| 256 | **559** / – | 1,652 / – |
| 384 | **777** / – | 2,538 / – |
| 512 | **1,140** / – | 3,522 / – |
| 1024 | **2,564** / – | 8,487 / – |
| 2048 | **7,532** / – | 22,429 / – |
| 4096 | **25,232** / – | 67,737 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **123** / **64.3** | 476 / – |
| 128 | **615** / **643** | 1,295 / – |
| 256 | **1,368** / **1,307** | 2,861 / – |
| 384 | **2,297** / **2,517** | 4,953 / – |
| 512 | **3,764** / **4,110** | 7,635 / – |
| 1024 | **10,183** / **10,208** | 21,447 / – |
| 2048 | **35,671** / **35,747** | 65,208 / – |
| 4096 | **128,004** / **128,338** | 259,950 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 259 / **229** | **12.9** / – |
| 128 | 455 / **453** | **23.7** / – |
| 256 | 651 / **610** | **47.5** / – |
| 384 | 769 / **759** | **96.8** / – |
| 512 | 902 / **886** | **109** / – |
| 1024 | 1,483 / **1,461** | **377** / – |
| 2048 | 2,792 / **2,803** | **575** / – |
| 4096 | 5,243 / **5,204** | **1,167** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 261 / **77.3** | **5.0** / – |
| 128 | 454 / **447** | **10.3** / – |
| 256 | 588 / **570** | **28.1** / – |
| 384 | 724 / **727** | **48.9** / – |
| 512 | 867 / **866** | **71.4** / – |
| 1024 | 1,425 / **1,411** | **177** / – |
| 2048 | 2,717 / **2,768** | **518** / – |
| 4096 | 4,977 / **4,925** | **935** / – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** |
| 128 | 3.9 / 2.7 | **3.7** / **2.1** | 3.8 / 2.5 | 3.9 / 2.7 | 4.0 / **2.1** | – |
| 256 | **9.0** / 8.2 | – | **9.0** / 8.0 | 9.1 / 7.1 | **9.0** / **6.3** | – |
| 384 | 13.0 / 13.9 | – | 13.0 / 13.6 | **12.2** / **12.3** | 16.0 / 14.3 | – |
| 512 | 23.1 / 19.5 | – | 23.1 / 18.9 | **20.8** / **18.3** | 22.1 / 23.0 | – |
| 1024 | 49.0 / 44.4 | – | 49.1 / 44.6 | **44.9** / **44.0** | 60.1 / 59.6 | – |
| 2048 | **108** / **99.5** | – | 158 / 161 | 139 / 174 | 182 / 220 | – |
| 4096 | 342 / 334 | – | 292 / **290** | **275** / 314 | 361 / 411 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | 6.5 / – | **4.1** / **5.2** | 6.5 / – | 5.5 / – | 4.3 / – | 6.9 / – | – |
| 256 | **16.8** / – | – | **16.8** / – | 18.4 / – | 22.3 / – | 22.3 / – | – |
| 384 | 74.5 / – | – | 52.2 / – | **39.6** / – | 75.6 / – | 64.2 / – | – |
| 512 | 120 / – | – | **70.4** / – | 84.0 / – | 209 / – | – | – |
| 1024 | 727 / – | – | – | 593 / – | 700 / – | **503** / – | – |
| 2048 | 3,101 / – | – | **2,113** / – | 2,764 / – | 2,600 / – | 3,308 / – | 11,754 / – |
| 4096 | 10,490 / – | – | **8,005** / – | 11,432 / – | 10,110 / – | 14,661 / – | 39,648 / – |

### div: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 448 / – | **7.0** / – | **7.0** / – | 125 / – |
| 128 | 177 / – | 169 / – | **43.3** / – | 494 / – | 185 / – | 197 / – | 325 / – |
| 256 | **85.1** / – | – | 156 / – | 1,041 / – | 590 / – | 578 / – | 454 / – |
| 384 | **205** / – | – | 351 / – | 1,279 / – | 911 / – | 854 / – | 607 / – |
| 512 | **297** / – | – | 426 / – | 1,542 / – | 1,258 / – | 1,146 / – | 807 / – |
| 1024 | **718** / – | – | 903 / – | 3,066 / – | 2,748 / – | 2,405 / – | 1,937 / – |
| 2048 | **2,014** / – | – | 2,356 / – | 7,818 / – | 7,428 / – | 5,997 / – | 5,787 / – |
| 4096 | **6,826** / – | – | 7,681 / – | 20,064 / – | 22,312 / – | 19,879 / – | 20,048 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 448 / – | **7.0** / – | **7.0** / – | 33.4 / – |
| 128 | 178 / – | 170 / – | **40.2** / – | 493 / – | 187 / – | 195 / – | 353 / – |
| 256 | **75.3** / – | – | 158 / – | 1,042 / – | 591 / – | 577 / – | 471 / – |
| 384 | **196** / – | – | 321 / – | 1,284 / – | 909 / – | 856 / – | 631 / – |
| 512 | **278** / – | – | 403 / – | 1,542 / – | 1,255 / – | 1,160 / – | 827 / – |
| 1024 | **742** / – | – | 961 / – | 3,055 / – | 2,744 / – | 2,438 / – | 1,939 / – |
| 2048 | **1,960** / – | – | 2,292 / – | 7,833 / – | 7,391 / – | 5,972 / – | 5,775 / – |
| 4096 | **6,493** / – | – | 7,789 / – | 19,989 / – | 22,213 / – | 19,953 / – | 19,964 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 118 / 114 | 116 / 112 | **39.9** / **43.0** | 440 / 436 | 437 / – |
| 128 | **81.7** / **85.8** | – | 135 / 136 | 971 / 973 | 609 / – |
| 256 | **277** / **287** | – | 308 / 309 | 1,565 / 1,578 | 1,121 / – |
| 384 | 627 / 621 | – | **539** / **538** | 2,319 / 2,339 | 1,852 / – |
| 512 | 978 / 987 | – | **857** / **861** | 3,132 / 3,154 | 2,820 / – |
| 1024 | 3,393 / 3,333 | – | **3,197** / **3,177** | 8,426 / 8,865 | 8,910 / – |
| 2048 | **10,431** / **10,401** | – | 10,689 / 10,687 | 24,184 / 24,070 | 32,002 / – |
| 4096 | **34,701** / **34,807** | – | 41,159 / 41,139 | 74,437 / 74,689 | 112,828 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.0 / **7.3** | **4.0** / – | 44.2 / 43.5 | 12.2 / – | 7.9 / – | **4.0** / – | 291 / – |
| 128 | 14.0 / **17.0** | 9.0 / – | 21.7 / 30.8 | 21.2 / – | 11.2 / – | **8.9** / – | 305 / – |
| 256 | 35.0 / **34.8** | – | 44.1 / 61.3 | 41.7 / – | 21.8 / – | **21.3** / – | 333 / – |
| 384 | 62.1 / **72.3** | – | 60.4 / 75.0 | 105 / – | 40.4 / – | **39.2** / – | 372 / – |
| 512 | 61.5 / **68.8** | – | 85.6 / 97.1 | 93.5 / – | 50.5 / – | **44.6** / – | 404 / – |
| 1024 | 127 / **141** | – | 176 / 202 | 344 / – | 118 / – | **107** / – | 544 / – |
| 2048 | **239** / **270** | – | 745 / 780 | 504 / – | 366 / – | **239** / – | 893 / – |
| 4096 | **562** / **642** | – | 1,403 / 1,438 | 989 / – | 637 / – | 574 / – | 2,106 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.0 / **7.9** | **3.0** / – | 50.5 / 44.9 | 5.0 / – | 4.8 / – | 4.3 / – | 396 / – |
| 128 | 14.0 / **16.2** | **5.4** / – | 30.1 / 41.3 | 11.0 / – | 7.3 / – | 10.3 / – | 440 / – |
| 256 | 31.0 / **36.3** | – | 61.7 / 79.6 | 23.5 / – | **16.4** / – | 19.4 / – | 474 / – |
| 384 | 54.4 / **57.7** | – | 82.6 / 108 | 39.1 / – | 39.3 / – | **32.1** / – | 562 / – |
| 512 | 63.8 / **70.8** | – | 114 / 139 | 53.7 / – | **44.3** / – | 60.9 / – | 576 / – |
| 1024 | **107** / **125** | – | 330 / 373 | 141 / – | 133 / – | 131 / – | 795 / – |
| 2048 | **229** / **234** | – | 1,079 / 1,131 | 472 / – | 427 / – | 301 / – | 1,340 / – |
| 4096 | **532** / **612** | – | 1,993 / 2,046 | 789 / – | 737 / – | 705 / – | 2,767 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** |
| 128 | 3.9 / 2.7 | **3.7** / **2.1** | 3.8 / 2.5 | 3.9 / 2.7 | 4.0 / **2.1** | – |
| 256 | **9.0** / 8.2 | – | **9.0** / 8.0 | 9.1 / 7.1 | **9.0** / **6.3** | – |
| 384 | 13.0 / 13.9 | – | 13.0 / 13.6 | **12.2** / **12.3** | 16.0 / 14.2 | – |
| 512 | 23.1 / 19.6 | – | 23.0 / 18.9 | **20.8** / **17.8** | 22.0 / 23.0 | – |
| 1024 | 49.0 / 44.4 | – | 49.2 / 44.6 | **44.9** / **44.0** | 60.1 / 59.6 | – |
| 2048 | **108** / **99.6** | – | 158 / 162 | 139 / 173 | 182 / 220 | – |
| 4096 | 345 / 333 | – | 292 / **289** | **275** / 313 | 359 / 411 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 6.5 / – | **4.1** / **5.3** | 6.5 / – | 5.5 / – | 4.3 / – | 6.9 / – |
| 256 | **16.8** / – | – | **16.8** / – | 18.4 / – | 22.3 / – | 22.3 / – |
| 384 | 74.3 / – | – | 52.2 / – | **39.6** / – | 75.5 / – | 64.2 / – |
| 512 | 122 / – | – | **69.5** / – | 84.0 / – | 209 / – | – |
| 1024 | 726 / – | – | – | 591 / – | 700 / – | **503** / – |
| 2048 | 3,118 / – | – | – | 2,764 / – | **2,600** / – | 3,308 / – |
| 4096 | 10,601 / – | – | – | 11,432 / – | **10,110** / – | 14,661 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.2** / – | 321 / – |
| 128 | **199** / – | 752 / – |
| 256 | **354** / – | 1,367 / – |
| 384 | **523** / – | 2,025 / – |
| 512 | **775** / – | 2,767 / – |
| 1024 | **1,874** / – | 7,060 / – |
| 2048 | **5,350** / – | 16,818 / – |
| 4096 | **16,839** / – | 43,385 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.3** / – | 322 / – |
| 128 | **196** / – | 745 / – |
| 256 | **352** / – | 1,339 / – |
| 384 | **591** / – | 1,971 / – |
| 512 | **751** / – | 2,750 / – |
| 1024 | **1,878** / – | 7,034 / – |
| 2048 | **5,170** / – | 16,860 / – |
| 4096 | **16,759** / – | 46,898 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **97.1** / **94.0** | 351 / – |
| 128 | **298** / **355** | 998 / – |
| 256 | **767** / **767** | 1,979 / – |
| 384 | **1,305** / **1,347** | 3,216 / – |
| 512 | **1,995** / **1,981** | 4,660 / – |
| 1024 | **6,324** / **6,483** | 13,238 / – |
| 2048 | **21,662** / **22,153** | 35,473 / – |
| 4096 | **76,404** / **74,844** | 110,456 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 91.0 / **89.3** | **12.2** / – |
| 128 | 300 / **310** | **21.2** / – |
| 256 | 431 / **433** | **41.7** / – |
| 384 | 527 / **543** | **105** / – |
| 512 | 645 / **618** | **93.5** / – |
| 1024 | 1,123 / **1,129** | **344** / – |
| 2048 | 2,153 / **2,168** | **504** / – |
| 4096 | 3,967 / **4,017** | **989** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 105 / **78.8** | **5.0** / – |
| 128 | 298 / **308** | **11.0** / – |
| 256 | 390 / **386** | **23.5** / – |
| 384 | 491 / **499** | **39.1** / – |
| 512 | 574 / **566** | **53.7** / – |
| 1024 | 1,030 / **1,022** | **141** / – |
| 2048 | 2,077 / **2,053** | **472** / – |
| 4096 | 3,768 / **3,714** | **789** / – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / 1.1 | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / 1.1 | **2.2** / 1.1 | **2.2** / **1.0** |
| 128 | 4.0 / 2.8 | **3.7** / 3.0 | 4.0 / 3.1 | 4.3 / **2.7** | **3.7** / 3.0 | – |
| 256 | **10.0** / 8.9 | – | 10.1 / 9.1 | 10.5 / 9.2 | 10.1 / **8.7** | – |
| 384 | **15.9** / 15.6 | – | 16.0 / **15.2** | 17.4 / 17.5 | 17.5 / 16.8 | – |
| 512 | 26.0 / 28.5 | – | 27.7 / 29.8 | 37.9 / 37.2 | **23.4** / **26.3** | – |
| 1024 | **69.4** / 71.5 | – | 72.4 / **69.0** | 131 / 80.7 | 75.1 / 70.0 | – |
| 2048 | **188** / **178** | – | 213 / 214 | 207 / 251 | 204 / 255 | – |
| 4096 | 428 / **427** | – | 417 / 430 | **412** / 475 | 417 / 469 | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | 17.0 / – | **13.5** / **12.5** | 17.0 / – | 37.1 / – | **13.5** / – | 17.0 / – | – |
| 256 | 82.1 / – | – | 82.1 / – | 122 / – | 128 / – | **80.6** / – | – |
| 384 | 230 / – | – | **219** / – | 298 / – | 363 / – | 233 / – | – |
| 512 | 471 / – | – | **438** / – | 500 / – | 680 / – | – | – |
| 1024 | 2,397 / – | – | – | 2,595 / – | **2,389** / – | 2,747 / – | – |
| 2048 | 9,337 / – | – | 11,027 / – | 9,921 / – | **8,692** / – | 13,935 / – | 11,869 / – |
| 4096 | **29,418** / – | – | 39,885 / – | 35,650 / – | 33,233 / – | 65,927 / – | 39,439 / – |

### div: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 562 / – | **7.0** / – | **7.0** / – | 158 / – |
| 128 | 244 / – | 239 / – | **142** / – | 643 / – | 252 / – | 265 / – | 417 / – |
| 256 | **306** / – | – | 407 / – | 1,479 / – | 963 / – | 925 / – | 572 / – |
| 384 | **610** / – | – | 757 / – | 1,968 / – | 1,420 / – | 1,394 / – | 770 / – |
| 512 | **836** / – | – | 1,028 / – | 2,443 / – | 1,976 / – | 2,005 / – | 1,015 / – |
| 1024 | **2,221** / – | – | 2,680 / – | 4,965 / – | 4,823 / – | 4,650 / – | 2,245 / – |
| 2048 | **6,235** / – | – | 7,266 / – | 12,440 / – | 15,734 / – | 13,869 / – | 6,241 / – |
| 4096 | 20,685 / – | – | 25,122 / – | 38,559 / – | 50,834 / – | 48,504 / – | **20,454** / – |

### mod: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 561 / – | **7.0** / – | **7.0** / – | 31.5 / – |
| 128 | 245 / – | 240 / – | **96.2** / – | 643 / – | 253 / – | 263 / – | 427 / – |
| 256 | **308** / – | – | 396 / – | 1,479 / – | 962 / – | 931 / – | 581 / – |
| 384 | **609** / – | – | 730 / – | 1,963 / – | 1,414 / – | 1,388 / – | 774 / – |
| 512 | **821** / – | – | 999 / – | 2,447 / – | 1,974 / – | 2,007 / – | 1,028 / – |
| 1024 | **2,160** / – | – | 2,608 / – | 4,977 / – | 4,833 / – | 4,690 / – | 2,251 / – |
| 2048 | 6,280 / – | – | 7,486 / – | 12,459 / – | 15,741 / – | 13,943 / – | **6,261** / – |
| 4096 | 20,634 / – | – | 25,829 / – | 38,575 / – | 50,637 / – | 48,447 / – | **20,558** / – |

### mulmod: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 210 / 195 | 216 / 192 | **185** / **189** | 646 / 623 | 536 / – |
| 128 | **343** / **364** | – | 398 / 409 | 1,493 / 1,445 | 746 / – |
| 256 | **912** / **910** | – | 919 / **910** | 2,654 / 2,518 | 1,312 / – |
| 384 | 1,763 / 1,739 | – | **1,710** / **1,693** | 3,989 / 3,979 | 2,124 / – |
| 512 | **3,011** / **3,000** | – | 3,345 / 3,272 | 5,976 / 5,916 | 3,101 / – |
| 1024 | 9,577 / **9,666** | – | 12,704 / 12,661 | 16,759 / 16,689 | **9,168** / – |
| 2048 | 33,756 / **33,759** | – | 43,691 / 43,714 | 54,000 / 54,097 | **31,923** / – |
| 4096 | **105,597** / **105,966** | – | 165,892 / 165,725 | 179,472 / 180,188 | 113,926 / – |

### add_mod: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.2 / **8.0** | **3.7** / – | 85.4 / 80.0 | 11.8 / – | 7.6 / – | **3.7** / – | 359 / – |
| 128 | 19.6 / **38.2** | **9.8** / – | 23.1 / 52.1 | 23.1 / – | 10.0 / – | 10.0 / – | 376 / – |
| 256 | 52.1 / **52.4** | – | 58.9 / 96.8 | 57.1 / – | 25.8 / – | **20.5** / – | 408 / – |
| 384 | 106 / **123** | – | 97.6 / 149 | 86.1 / – | 39.8 / – | **32.2** / – | 435 / – |
| 512 | 110 / **138** | – | 124 / 174 | 141 / – | 71.8 / – | **46.8** / – | 466 / – |
| 1024 | 186 / **247** | – | 264 / 360 | 459 / – | 164 / – | **139** / – | 615 / – |
| 2048 | 324 / **393** | – | 1,024 / 1,072 | 888 / – | 400 / – | **280** / – | 965 / – |
| 4096 | 692 / **763** | – | 1,763 / 1,959 | 1,620 / – | 686 / – | **675** / – | 2,103 / – |

### sub_mod: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 8.0 / **9.1** | **3.0** / – | 79.3 / 78.1 | 5.0 / – | 5.8 / – | 7.0 / – | 452 / – |
| 128 | 19.7 / **39.3** | **7.4** / – | 37.1 / 62.1 | 11.0 / – | 8.2 / – | 10.3 / – | 483 / – |
| 256 | 50.7 / **63.6** | – | 91.7 / 132 | 34.4 / – | 22.2 / – | **20.9** / – | 543 / – |
| 384 | 86.5 / **108** | – | 139 / 190 | 57.6 / – | **33.2** / – | 35.6 / – | 592 / – |
| 512 | 102 / **142** | – | 179 / 249 | 93.7 / – | **56.2** / – | 74.5 / – | 642 / – |
| 1024 | 192 / **270** | – | 566 / 657 | 347 / – | 186 / – | **153** / – | 842 / – |
| 2048 | 585 / **685** | – | 1,357 / 1,491 | 564 / – | 427 / – | **376** / – | 1,324 / – |
| 4096 | **634** / **816** | – | 2,405 / 2,570 | 1,426 / – | 719 / – | 769 / – | 2,754 / – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / 1.1 | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / 1.1 | **2.2** / 1.1 | **2.2** / **1.0** |
| 128 | 4.0 / 2.8 | **3.7** / 3.0 | 4.0 / 3.1 | 4.3 / **2.7** | **3.7** / 3.0 | – |
| 256 | **10.0** / 8.9 | – | 10.1 / 9.0 | 10.5 / 9.2 | 10.1 / **8.7** | – |
| 384 | **15.9** / 15.5 | – | 16.1 / **15.2** | 17.4 / 17.5 | 17.5 / 16.8 | – |
| 512 | 26.0 / 27.4 | – | 27.6 / 30.0 | 37.9 / 37.3 | **23.5** / **26.9** | – |
| 1024 | **69.4** / 71.5 | – | 72.4 / **69.1** | 130 / 81.7 | 75.5 / 70.0 | – |
| 2048 | **188** / **177** | – | 213 / 214 | 207 / 252 | 204 / 257 | – |
| 4096 | 429 / **424** | – | 415 / 430 | **411** / 474 | 416 / 468 | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 17.0 / – | **13.5** / **12.5** | 17.0 / – | 37.1 / – | **13.5** / – | 17.0 / – |
| 256 | 82.1 / – | – | 82.2 / – | 122 / – | 128 / – | **80.3** / – |
| 384 | 227 / – | – | **217** / – | 297 / – | 363 / – | 235 / – |
| 512 | 467 / – | – | **441** / – | 502 / – | 681 / – | – |
| 1024 | 2,480 / – | – | – | 2,594 / – | **2,391** / – | 2,821 / – |
| 2048 | 9,321 / – | – | – | 9,921 / – | **8,692** / – | 13,935 / – |
| 4096 | **29,835** / – | – | – | 35,650 / – | 33,233 / – | 65,927 / – |

### div, constant time: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **166** / – | 424 / – |
| 128 | **417** / – | 1,085 / – |
| 256 | **710** / – | 1,972 / – |
| 384 | **974** / – | 3,057 / – |
| 512 | **1,408** / – | 4,275 / – |
| 1024 | **3,542** / – | 10,421 / – |
| 2048 | **10,440** / – | 27,634 / – |
| 4096 | **34,520** / – | 85,135 / – |

### mod, constant time: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **168** / – | 420 / – |
| 128 | **434** / – | 1,053 / – |
| 256 | **749** / – | 1,950 / – |
| 384 | **1,036** / – | 2,985 / – |
| 512 | **1,474** / – | 4,276 / – |
| 1024 | **3,672** / – | 10,379 / – |
| 2048 | **10,249** / – | 27,997 / – |
| 4096 | **34,964** / – | 85,307 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **312** / **316** | 517 / – |
| 128 | **781** / **751** | 1,545 / – |
| 256 | **1,752** / **1,705** | 3,311 / – |
| 384 | **3,066** / **3,071** | 5,741 / – |
| 512 | **4,843** / **4,832** | 8,738 / – |
| 1024 | **14,403** / **14,345** | 24,968 / – |
| 2048 | **47,025** / **45,961** | 76,441 / – |
| 4096 | **160,050** / **159,789** | 256,284 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 271 / **274** | **11.8** / – |
| 128 | 537 / **542** | **23.1** / – |
| 256 | 789 / **783** | **57.1** / – |
| 384 | 959 / **948** | **86.1** / – |
| 512 | 1,113 / **1,098** | **141** / – |
| 1024 | 1,838 / **1,813** | **459** / – |
| 2048 | 3,358 / **3,302** | **888** / – |
| 4096 | 6,193 / **6,098** | **1,620** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 269 / **271** | **5.0** / – |
| 128 | 567 / **544** | **11.0** / – |
| 256 | 738 / **748** | **34.4** / – |
| 384 | 897 / **904** | **57.6** / – |
| 512 | 1,092 / **1,082** | **93.7** / – |
| 1024 | 1,824 / **1,829** | **347** / – |
| 2048 | 3,454 / **3,414** | **564** / – |
| 4096 | 5,824 / **5,794** | **1,426** / – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** |
| 128 | 4.3 / 3.3 | **3.4** / 2.6 | 4.3 / 3.2 | 4.3 / 3.3 | 3.5 / **2.5** | – |
| 256 | 13.0 / 11.7 | – | 13.0 / 11.7 | 12.0 / 11.0 | **11.0** / **7.7** | – |
| 384 | 20.0 / 20.0 | – | 20.0 / 20.0 | **18.0** / **18.0** | 18.1 / 19.1 | – |
| 512 | 33.0 / 30.6 | – | 34.0 / 30.6 | 32.5 / **29.2** | **27.1** / 29.8 | – |
| 1024 | 72.1 / 64.5 | – | 72.1 / 64.3 | **68.4** / **63.3** | 70.0 / 69.2 | – |
| 2048 | **150** / **131** | – | 204 / 205 | 177 / 222 | 206 / 256 | – |
| 4096 | 430 / 418 | – | 381 / **390** | **335** / 392 | 415 / 472 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | 5.6 / – | 5.4 / **5.5** | 5.6 / – | **5.0** / – | 5.4 / – | 5.8 / – | – |
| 256 | **26.1** / – | – | **26.1** / – | 27.8 / – | 35.8 / – | 39.4 / – | – |
| 384 | 95.0 / – | – | 68.9 / – | **67.3** / – | 112 / – | 105 / – | – |
| 512 | 198 / – | – | **128** / – | 132 / – | 246 / – | – | – |
| 1024 | 1,259 / – | – | – | **904** / – | 1,019 / – | 1,082 / – | – |
| 2048 | 5,117 / – | – | **3,214** / – | 4,030 / – | 3,409 / – | 5,335 / – | 11,700 / – |
| 4096 | 17,175 / – | – | **11,582** / – | 16,189 / – | 12,868 / – | 22,728 / – | 39,075 / – |

### div: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 488 / – | **7.0** / – | **7.0** / – | 157 / – |
| 128 | 243 / – | 244 / – | **55.8** / – | 546 / – | 243 / – | 253 / – | 421 / – |
| 256 | **150** / – | – | 177 / – | 1,084 / – | 736 / – | 747 / – | 584 / – |
| 384 | **290** / – | – | 409 / – | 1,354 / – | 1,111 / – | 1,036 / – | 773 / – |
| 512 | **382** / – | – | 555 / – | 1,651 / – | 1,522 / – | 1,463 / – | 1,021 / – |
| 1024 | **883** / – | – | 1,126 / – | 3,299 / – | 3,413 / – | 3,060 / – | 2,244 / – |
| 2048 | **2,362** / – | – | 2,682 / – | 7,820 / – | 10,562 / – | 7,529 / – | 6,289 / – |
| 4096 | **7,274** / – | – | 8,958 / – | 23,284 / – | 33,113 / – | 25,052 / – | 20,443 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 489 / – | **7.0** / – | **7.0** / – | 31.4 / – |
| 128 | 245 / – | 243 / – | **51.7** / – | 546 / – | 243 / – | 254 / – | 422 / – |
| 256 | **161** / – | – | 174 / – | 1,090 / – | 746 / – | 749 / – | 583 / – |
| 384 | **280** / – | – | 340 / – | 1,353 / – | 1,114 / – | 1,048 / – | 780 / – |
| 512 | **353** / – | – | 463 / – | 1,639 / – | 1,515 / – | 1,443 / – | 1,030 / – |
| 1024 | **958** / – | – | 1,073 / – | 3,298 / – | 3,393 / – | 3,105 / – | 2,258 / – |
| 2048 | **2,173** / – | – | 2,621 / – | 7,806 / – | 10,586 / – | 7,528 / – | 6,256 / – |
| 4096 | **7,338** / – | – | 8,812 / – | 23,327 / – | 33,226 / – | 25,197 / – | 20,448 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | 187 / 184 | 186 / 182 | **64.6** / **55.6** | 479 / 476 | 554 / – |
| 128 | **162** / **169** | – | 205 / 205 | 1,096 / 1,095 | 748 / – |
| 256 | **360** / **341** | – | 382 / 380 | 1,622 / 1,628 | 1,321 / – |
| 384 | 880 / 886 | – | **717** / **730** | 2,399 / 2,398 | 2,119 / – |
| 512 | 1,458 / 1,435 | – | **1,126** / **1,125** | 2,978 / 2,992 | 3,125 / – |
| 1024 | 4,850 / 4,771 | – | **3,578** / **3,596** | 8,548 / 8,563 | 9,263 / – |
| 2048 | 15,395 / 15,397 | – | **13,759** / **13,779** | 27,830 / 27,869 | 32,175 / – |
| 4096 | **52,225** / **52,445** | – | 53,798 / 53,915 | 98,430 / 98,969 | 113,551 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 9.1 / **9.0** | **5.0** / – | 83.4 / 79.0 | 13.2 / – | 8.4 / – | **5.0** / – | 354 / – |
| 128 | 20.6 / **35.8** | **9.1** / – | 25.1 / 38.6 | 22.9 / – | 9.9 / – | 11.8 / – | 373 / – |
| 256 | 52.8 / **54.7** | – | 59.7 / 77.9 | 54.8 / – | 32.4 / – | **23.7** / – | 402 / – |
| 384 | 102 / 115 | – | 88.4 / **109** | 82.9 / – | 57.1 / – | **39.4** / – | 440 / – |
| 512 | 107 / **119** | – | 122 / 145 | 127 / – | 80.6 / – | **61.5** / – | 474 / – |
| 1024 | 183 / **214** | – | 264 / 300 | 384 / – | 163 / – | **136** / – | 608 / – |
| 2048 | 285 / **342** | – | 945 / 948 | 770 / – | 404 / – | **278** / – | 1,003 / – |
| 4096 | **644** / **757** | – | 1,681 / 1,702 | 1,403 / – | 747 / – | 661 / – | 2,131 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 10.2 / **12.1** | **3.0** / – | 77.3 / 75.8 | 5.0 / – | 5.6 / – | 6.8 / – | 485 / – |
| 128 | 21.1 / **31.5** | **7.3** / – | 38.0 / 54.2 | 11.0 / – | 8.6 / – | 9.3 / – | 507 / – |
| 256 | 53.4 / **56.3** | – | 89.8 / 118 | 33.5 / – | 21.7 / – | **21.2** / – | 551 / – |
| 384 | 83.6 / **99.8** | – | 122 / 159 | 52.5 / – | 42.2 / – | **41.1** / – | 604 / – |
| 512 | 97.8 / **113** | – | 176 / 205 | 105 / – | **66.2** / – | 75.1 / – | 650 / – |
| 1024 | **145** / **175** | – | 551 / 609 | 275 / – | 179 / – | 156 / – | 849 / – |
| 2048 | **276** / **326** | – | 1,265 / 1,362 | 503 / – | 419 / – | 378 / – | 1,324 / – |
| 4096 | **650** / **822** | – | 2,259 / 2,346 | 1,376 / – | 712 / – | 777 / – | 2,893 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.0** | 2.2 / **1.0** | 2.2 / **1.0** | **2.1** / **1.0** | 2.2 / **1.0** | 2.2 / **1.0** |
| 128 | 4.3 / 3.2 | **3.4** / 2.6 | 4.3 / 3.2 | 4.3 / 3.3 | 3.5 / **2.5** | – |
| 256 | 13.0 / 11.7 | – | 13.0 / 11.7 | 12.0 / 11.0 | **11.0** / **7.7** | – |
| 384 | 20.0 / 20.0 | – | 20.0 / 20.0 | **18.0** / **18.0** | 18.1 / 19.1 | – |
| 512 | 33.0 / 30.6 | – | 34.0 / 30.6 | 32.5 / **29.2** | **27.0** / 29.8 | – |
| 1024 | 72.1 / 64.5 | – | 72.2 / 64.3 | **68.4** / **63.2** | 70.0 / 69.2 | – |
| 2048 | **150** / **131** | – | 204 / 205 | 177 / 222 | 206 / 255 | – |
| 4096 | 424 / 417 | – | 379 / **390** | **334** / 391 | 416 / 476 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 5.6 / – | 5.4 / **5.5** | 5.7 / – | **5.0** / – | 5.4 / – | 5.8 / – |
| 256 | **26.1** / – | – | **26.1** / – | 27.8 / – | 36.3 / – | 39.4 / – |
| 384 | 99.2 / – | – | 68.4 / – | **67.3** / – | 112 / – | 105 / – |
| 512 | 195 / – | – | **128** / – | 132 / – | 246 / – | – |
| 1024 | 1,268 / – | – | – | **904** / – | 1,024 / – | 1,083 / – |
| 2048 | 5,218 / – | – | – | 4,030 / – | **3,409** / – | 5,335 / – |
| 4096 | 17,892 / – | – | – | 16,189 / – | **12,868** / – | 22,728 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **59.4** / – | 338 / – |
| 128 | **264** / – | 910 / – |
| 256 | **472** / – | 1,515 / – |
| 384 | **615** / – | 2,274 / – |
| 512 | **875** / – | 3,100 / – |
| 1024 | **2,329** / – | 7,514 / – |
| 2048 | **6,800** / – | 18,568 / – |
| 4096 | **21,769** / – | 50,618 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **66.7** / – | 339 / – |
| 128 | **241** / – | 882 / – |
| 256 | **427** / – | 1,502 / – |
| 384 | **568** / – | 2,216 / – |
| 512 | **843** / – | 3,104 / – |
| 1024 | **2,101** / – | 7,513 / – |
| 2048 | **6,190** / – | 18,591 / – |
| 4096 | **20,763** / – | 50,380 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **125** / **118** | 382 / – |
| 128 | **405** / **387** | 1,146 / – |
| 256 | **847** / **900** | 2,291 / – |
| 384 | **1,685** / **1,716** | 3,701 / – |
| 512 | **2,683** / **2,711** | 5,528 / – |
| 1024 | **8,499** / **8,460** | 14,624 / – |
| 2048 | **28,815** / **27,569** | 45,185 / – |
| 4096 | **98,084** / **98,774** | 143,742 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 116 / **124** | **13.2** / – |
| 128 | 299 / **296** | **22.9** / – |
| 256 | 478 / **468** | **54.8** / – |
| 384 | 607 / **628** | **82.9** / – |
| 512 | 712 / **722** | **127** / – |
| 1024 | 1,331 / **1,325** | **384** / – |
| 2048 | 2,630 / **2,509** | **770** / – |
| 4096 | 4,780 / **4,707** | **1,403** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 113 / **117** | **5.0** / – |
| 128 | 304 / **314** | **11.0** / – |
| 256 | 436 / **429** | **33.5** / – |
| 384 | 570 / **549** | **52.5** / – |
| 512 | 632 / **633** | **105** / – |
| 1024 | 1,209 / **1,223** | **275** / – |
| 2048 | 2,396 / **2,423** | **503** / – |
| 4096 | 4,392 / **4,383** | **1,376** / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
