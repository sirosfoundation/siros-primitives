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

`biguint` at commit `921b96e` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,026 to 4,542 cycles and 3,081 instructions; its noise in a figure, 60.0 to 479 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,024 to 4,664 cycles and 3,081 instructions; its noise in a figure, 23.0 to 196 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,140 to 7,902 cycles and 1,885 instructions; its noise in a figure, 217 to 1,313 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,026 to 7,214 cycles and 1,885 instructions; its noise in a figure, 317 to 4,621 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 9.9 before, 6.76 after; its SMT sibling, CPU 28, 2% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 6.76 before, 6.49 after; its SMT sibling, CPU 28, 1% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 6.49 before, 11.55 after; its SMT sibling, CPU 28, 4% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 11.55 before, 11.29 after; its SMT sibling, CPU 28, 4% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.2 vs native 2.1 | +0.1 (1.05×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) | 2.2 vs bnum 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.4 vs native 3.4 | +1.0 (1.29×) |
| add | 256 | 10.2 vs crypto-bigint 10.2 | +0.0 (1.00×) | 8.7 vs ruint 9.0 | −0.3 (0.97×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add | 384 | 16.3 vs ruint 16.4 | −0.1 (0.99×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 15.9 vs ruint 16.0 | −0.1 (0.99×) | 18.0 vs crypto-bigint 18.1 | −0.1 (0.99×) |
| add | 512 | 27.4 vs bnum 24.7 | +2.7 (1.11×) | 20.7 vs crypto-bigint 20.8 | −0.1 (1.00×) | 25.9 vs bnum 23.4 | +2.5 (1.11×) | 32.1 vs bnum 34.4 | −2.3 (0.93×) |
| add | 1024 | 64.1 vs bnum 61.6 | +2.5 (1.04×) | 45.3 vs crypto-bigint 45.1 | +0.2 (1.00×) | 69.2 vs ruint 73.7 | −4.5 (0.94×) | 74.0 vs crypto-bigint 68.7 | +5.3 (1.08×) |
| add | 2048 | 157 vs crypto-bigint 191 | −34.0 (0.82×) | 109 vs crypto-bigint 148 | −39.0 (0.74×) | 185 vs crypto-bigint 207 | −22.0 (0.89×) | 147 vs crypto-bigint 179 | −32.0 (0.82×) |
| add | 4096 | 430 vs crypto-bigint 389 | +41.0 (1.11×) | 382 vs crypto-bigint 283 | +99.0 (1.35×) | 429 vs bnum 407 | +22.0 (1.05×) | 405 vs crypto-bigint 337 | +68.0 (1.20×) |
| mul | 64 | 2.1 vs ruint 2.1 | +0.0 (1.00×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) |
| mul | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.1 | +2.4 (1.59×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.6 vs crypto-bigint 5.0 | +0.6 (1.12×) |
| mul | 256 | 47.7 vs ruint 66.6 | −18.9 (0.72×) | 16.8 vs ruint 16.8 | +0.0 (1.00×) | 66.6 vs uint 81.7 | −15.1 (0.82×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul | 384 | 181 vs uint 154 | +27.0 (1.18×) | 52.3 vs crypto-bigint 39.6 | +12.7 (1.32×) | 228 vs ruint 219 | +9.0 (1.04×) | 70.3 vs crypto-bigint 65.9 | +4.4 (1.07×) |
| mul | 512 | 295 vs ruint 320 | −25.0 (0.92×) | 70.7 vs ruint 69.2 | +1.5 (1.02×) | 363 vs ruint 443 | −80.0 (0.82×) | 131 vs ruint 131 | +0.0 (1.00×) |
| mul | 1024 | 1,340 vs bnum 1,544 | −204 (0.87×) | 691 vs uint 505 | +186 (1.37×) | 1,594 vs bnum 2,403 | −809 (0.66×) | 846 vs crypto-bigint 918 | −72.0 (0.92×) |
| mul | 2048 | 5,613 vs bnum 6,288 | −675 (0.89×) | 2,500 vs ruint 2,115 | +385 (1.18×) | 6,696 vs bnum 8,648 | −1,952 (0.77×) | 3,697 vs ruint 3,194 | +503 (1.16×) |
| mul | 4096 | 17,687 vs bnum 23,616 | −5,929 (0.75×) | 8,430 vs ruint 8,001 | +429 (1.05×) | 21,172 vs bnum 33,252 | −12,080 (0.64×) | 12,443 vs ruint 11,657 | +786 (1.07×) |
| div | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 60.1 vs ruint 93.8 | −33.7 (0.64×) | 28.3 vs ruint 43.3 | −15.0 (0.65×) | 103 vs ruint 104 | −1.0 (0.99×) | 36.4 vs ruint 55.7 | −19.3 (0.65×) |
| div | 256 | 216 vs ruint 320 | −104 (0.68×) | 87.0 vs ruint 156 | −69.0 (0.56×) | 221 vs ruint 411 | −190 (0.54×) | 157 vs ruint 177 | −20.0 (0.89×) |
| div | 384 | 351 vs ruint 604 | −253 (0.58×) | 203 vs ruint 349 | −146 (0.58×) | 402 vs ruint 772 | −370 (0.52×) | 295 vs ruint 411 | −116 (0.72×) |
| div | 512 | 509 vs num-bigint 807 | −298 (0.63×) | 295 vs ruint 422 | −127 (0.70×) | 575 vs num-bigint 1,026 | −451 (0.56×) | 395 vs ruint 567 | −172 (0.70×) |
| div | 1024 | 1,196 vs num-bigint 1,940 | −744 (0.62×) | 696 vs ruint 899 | −203 (0.77×) | 1,493 vs num-bigint 2,204 | −711 (0.68×) | 877 vs ruint 1,120 | −243 (0.78×) |
| div | 2048 | 3,242 vs num-bigint 5,774 | −2,532 (0.56×) | 2,005 vs ruint 2,358 | −353 (0.85×) | 4,377 vs num-bigint 6,105 | −1,728 (0.72×) | 2,459 vs ruint 2,691 | −232 (0.91×) |
| div | 4096 | 12,603 vs num-bigint 20,046 | −7,443 (0.63×) | 6,829 vs ruint 7,695 | −866 (0.89×) | 15,027 vs num-bigint 20,074 | −5,047 (0.75×) | 7,306 vs ruint 8,985 | −1,679 (0.81×) |
| mod | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 52.8 vs ruint 87.3 | −34.5 (0.60×) | 30.0 vs ruint 40.5 | −10.5 (0.74×) | 96.3 vs ruint 99.4 | −3.1 (0.97×) | 33.1 vs ruint 51.4 | −18.3 (0.64×) |
| mod | 256 | 217 vs ruint 321 | −104 (0.68×) | 82.0 vs ruint 157 | −75.0 (0.52×) | 245 vs ruint 406 | −161 (0.60×) | 138 vs ruint 175 | −37.0 (0.79×) |
| mod | 384 | 351 vs ruint 581 | −230 (0.60×) | 195 vs ruint 320 | −125 (0.61×) | 412 vs ruint 716 | −304 (0.58×) | 273 vs ruint 342 | −69.0 (0.80×) |
| mod | 512 | 486 vs ruint 798 | −312 (0.61×) | 279 vs ruint 403 | −124 (0.69×) | 569 vs ruint 998 | −429 (0.57×) | 353 vs ruint 468 | −115 (0.75×) |
| mod | 1024 | 1,217 vs num-bigint 1,940 | −723 (0.63×) | 720 vs ruint 961 | −241 (0.75×) | 1,498 vs num-bigint 2,217 | −719 (0.68×) | 948 vs ruint 1,079 | −131 (0.88×) |
| mod | 2048 | 3,347 vs num-bigint 5,774 | −2,427 (0.58×) | 1,955 vs ruint 2,277 | −322 (0.86×) | 4,274 vs num-bigint 6,169 | −1,895 (0.69×) | 2,223 vs ruint 2,649 | −426 (0.84×) |
| mod | 4096 | 12,491 vs num-bigint 20,253 | −7,762 (0.62×) | 6,494 vs ruint 7,790 | −1,296 (0.83×) | 14,884 vs num-bigint 20,215 | −5,331 (0.74×) | 6,911 vs ruint 8,929 | −2,018 (0.77×) |
| mulmod | 64 | 61.0 vs ruint 84.7 | −23.7 (0.72×) | 31.6 vs ruint 39.9 | −8.3 (0.79×) | 104 vs ruint 194 | −90.0 (0.54×) | 35.8 vs ruint 66.6 | −30.8 (0.54×) |
| mulmod | 128 | 197 vs ruint 325 | −128 (0.61×) | 81.7 vs ruint 137 | −55.3 (0.60×) | 247 vs ruint 428 | −181 (0.58×) | 158 vs ruint 209 | −51.0 (0.76×) |
| mulmod | 256 | 579 vs ruint 818 | −239 (0.71×) | 281 vs ruint 307 | −26.0 (0.92×) | 701 vs ruint 1,022 | −321 (0.69×) | 352 vs ruint 383 | −31.0 (0.92×) |
| mulmod | 384 | 1,041 vs ruint 1,614 | −573 (0.64×) | 507 vs ruint 538 | −31.0 (0.94×) | 1,414 vs ruint 1,716 | −302 (0.82×) | 669 vs ruint 696 | −27.0 (0.96×) |
| mulmod | 512 | 1,652 vs num-bigint 2,815 | −1,163 (0.59×) | 849 vs ruint 855 | −6.0 (0.99×) | 2,158 vs num-bigint 3,095 | −937 (0.70×) | 1,176 vs ruint 1,116 | +60.0 (1.05×) |
| mulmod | 1024 | 5,332 vs num-bigint 8,902 | −3,570 (0.60×) | 2,764 vs ruint 2,632 | +132 (1.05×) | 7,741 vs num-bigint 9,313 | −1,572 (0.83×) | 4,219 vs ruint 3,632 | +587 (1.16×) |
| mulmod | 2048 | 17,381 vs num-bigint 32,084 | −14,703 (0.54×) | 9,432 vs ruint 10,666 | −1,234 (0.88×) | 22,251 vs num-bigint 32,100 | −9,849 (0.69×) | 12,708 vs ruint 13,701 | −993 (0.93×) |
| mulmod | 4096 | 55,642 vs num-bigint 112,933 | −57,291 (0.49×) | 29,321 vs ruint 41,180 | −11,859 (0.71×) | 68,366 vs num-bigint 113,388 | −45,022 (0.60×) | 42,732 vs ruint 53,236 | −10,504 (0.80×) |
| add_mod | 64 | 5.0 vs native 3.3 | +1.7 (1.52×) | 6.0 vs native 4.0 | +2.0 (1.50×) | 6.2 vs native 3.7 | +2.5 (1.68×) | 9.0 vs native 5.0 | +4.0 (1.80×) |
| add_mod | 128 | 15.0 vs native 9.5 | +5.5 (1.58×) | 13.0 vs uint 8.9 | +4.1 (1.46×) | 19.9 vs native 9.8 | +10.1 (2.03×) | 20.2 vs native 9.2 | +11.0 (2.20×) |
| add_mod | 256 | 27.4 vs uint 22.1 | +5.3 (1.24×) | 26.6 vs uint 21.4 | +5.2 (1.24×) | 53.7 vs uint 21.4 | +32.3 (2.51×) | 47.3 vs uint 23.9 | +23.4 (1.98×) |
| add_mod | 384 | 62.2 vs uint 34.5 | +27.7 (1.80×) | 66.2 vs uint 38.5 | +27.7 (1.72×) | 104 vs uint 31.2 | +72.8 (3.33×) | 107 vs uint 40.4 | +66.6 (2.65×) |
| add_mod | 512 | 73.8 vs uint 46.9 | +26.9 (1.57×) | 44.5 vs uint 44.6 | −0.1 (1.00×) | 120 vs uint 47.3 | +72.7 (2.54×) | 88.8 vs uint 61.3 | +27.5 (1.45×) |
| add_mod | 1024 | 148 vs uint 106 | +42.0 (1.40×) | 128 vs uint 104 | +24.0 (1.23×) | 190 vs uint 139 | +51.0 (1.37×) | 191 vs uint 139 | +52.0 (1.37×) |
| add_mod | 2048 | 291 vs uint 250 | +41.0 (1.16×) | 218 vs uint 242 | −24.0 (0.90×) | 328 vs uint 288 | +40.0 (1.14×) | 319 vs uint 270 | +49.0 (1.18×) |
| add_mod | 4096 | 634 vs uint 618 | +16.0 (1.03×) | 489 vs uint 585 | −96.0 (0.84×) | 687 vs uint 714 | −27.0 (0.96×) | 729 vs uint 681 | +48.0 (1.07×) |
| sub_mod | 64 | 5.0 vs native 3.0 | +2.0 (1.67×) | 6.0 vs native 3.0 | +3.0 (2.00×) | 8.0 vs native 3.0 | +5.0 (2.67×) | 10.0 vs native 3.0 | +7.0 (3.33×) |
| sub_mod | 128 | 14.0 vs native 6.8 | +7.2 (2.06×) | 12.0 vs native 5.4 | +6.6 (2.22×) | 21.0 vs native 7.4 | +13.6 (2.84×) | 20.1 vs native 7.3 | +12.8 (2.75×) |
| sub_mod | 256 | 32.8 vs bnum 18.6 | +14.2 (1.76×) | 27.3 vs bnum 16.5 | +10.8 (1.65×) | 53.6 vs uint 20.1 | +33.5 (2.67×) | 55.4 vs uint 21.7 | +33.7 (2.55×) |
| sub_mod | 384 | 53.1 vs uint 31.3 | +21.8 (1.70×) | 33.1 vs uint 32.1 | +1.0 (1.03×) | 87.6 vs bnum 32.5 | +55.1 (2.70×) | 66.4 vs uint 44.3 | +22.1 (1.50×) |
| sub_mod | 512 | 80.4 vs bnum 46.9 | +33.5 (1.71×) | 50.1 vs bnum 44.4 | +5.7 (1.13×) | 107 vs bnum 54.3 | +52.7 (1.97×) | 89.8 vs bnum 63.0 | +26.8 (1.43×) |
| sub_mod | 1024 | 150 vs uint 134 | +16.0 (1.12×) | 104 vs uint 132 | −28.0 (0.79×) | 192 vs uint 152 | +40.0 (1.26×) | 144 vs uint 156 | −12.0 (0.92×) |
| sub_mod | 2048 | 382 vs uint 301 | +81.0 (1.27×) | 231 vs uint 299 | −68.0 (0.77×) | 578 vs uint 373 | +205 (1.55×) | 295 vs uint 390 | −95.0 (0.76×) |
| sub_mod | 4096 | 656 vs bnum 672 | −16.0 (0.98×) | 492 vs uint 679 | −187 (0.72×) | 666 vs bnum 673 | −7.0 (0.99×) | 643 vs bnum 740 | −97.0 (0.87×) |
| add, constant time | 64 | 2.2 vs crypto-bigint 2.1 | +0.1 (1.05×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) | 2.2 vs crypto-bigint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add, constant time | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.4 vs native 3.4 | +1.0 (1.29×) |
| add, constant time | 256 | 10.2 vs ruint 10.3 | −0.1 (0.99×) | 8.7 vs ruint 9.0 | −0.3 (0.97×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add, constant time | 384 | 16.3 vs ruint 16.3 | +0.0 (1.00×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 15.9 vs ruint 16.0 | −0.1 (0.99×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 27.5 vs bnum 24.7 | +2.8 (1.11×) | 20.8 vs crypto-bigint 20.8 | +0.0 (1.00×) | 26.0 vs bnum 23.3 | +2.7 (1.12×) | 32.1 vs bnum 34.4 | −2.3 (0.93×) |
| add, constant time | 1024 | 64.0 vs bnum 61.6 | +2.4 (1.04×) | 45.4 vs crypto-bigint 45.2 | +0.2 (1.00×) | 69.3 vs ruint 74.5 | −5.2 (0.93×) | 74.0 vs crypto-bigint 68.7 | +5.3 (1.08×) |
| add, constant time | 2048 | 157 vs crypto-bigint 191 | −34.0 (0.82×) | 108 vs crypto-bigint 148 | −40.0 (0.73×) | 185 vs crypto-bigint 207 | −22.0 (0.89×) | 146 vs crypto-bigint 179 | −33.0 (0.82×) |
| add, constant time | 4096 | 429 vs crypto-bigint 395 | +34.0 (1.09×) | 379 vs crypto-bigint 284 | +95.0 (1.33×) | 428 vs bnum 407 | +21.0 (1.05×) | 405 vs crypto-bigint 338 | +67.0 (1.20×) |
| mul, constant time | 64 | 2.1 vs native 2.1 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.5 vs native 4.2 | +2.3 (1.55×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.6 vs crypto-bigint 5.0 | +0.6 (1.12×) |
| mul, constant time | 256 | 47.8 vs ruint 66.7 | −18.9 (0.72×) | 16.8 vs ruint 16.8 | +0.0 (1.00×) | 67.3 vs uint 81.7 | −14.4 (0.82×) | 26.0 vs ruint 26.0 | +0.0 (1.00×) |
| mul, constant time | 384 | 164 vs uint 160 | +4.0 (1.02×) | 52.3 vs crypto-bigint 39.5 | +12.8 (1.32×) | 217 vs ruint 220 | −3.0 (0.99×) | 70.2 vs crypto-bigint 64.5 | +5.7 (1.09×) |
| mul, constant time | 512 | 298 vs ruint 320 | −22.0 (0.93×) | 72.3 vs ruint 69.2 | +3.1 (1.04×) | 357 vs ruint 443 | −86.0 (0.81×) | 131 vs ruint 129 | +2.0 (1.02×) |
| mul, constant time | 1024 | 1,344 vs bnum 1,544 | −200 (0.87×) | 693 vs uint 503 | +190 (1.38×) | 1,580 vs bnum 2,406 | −826 (0.66×) | 862 vs crypto-bigint 919 | −57.0 (0.94×) |
| mul, constant time | 2048 | 5,787 vs bnum 6,288 | −501 (0.92×) | 2,558 vs bnum 2,603 | −45.0 (0.98×) | 6,752 vs bnum 8,648 | −1,896 (0.78×) | 3,888 vs bnum 3,404 | +484 (1.14×) |
| mul, constant time | 4096 | 18,263 vs bnum 23,616 | −5,353 (0.77×) | 8,862 vs bnum 10,109 | −1,247 (0.88×) | 21,877 vs bnum 33,252 | −11,375 (0.66×) | 12,853 vs bnum 12,865 | −12.0 (1.00×) |
| div, constant time | 64 | 83.6 vs crypto-bigint 389 | −305 (0.21×) | 47.0 vs crypto-bigint 322 | −275 (0.15×) | 144 vs crypto-bigint 429 | −285 (0.34×) | 59.5 vs crypto-bigint 338 | −279 (0.18×) |
| div, constant time | 128 | 220 vs crypto-bigint 901 | −681 (0.24×) | 183 vs crypto-bigint 751 | −568 (0.24×) | 362 vs crypto-bigint 1,080 | −718 (0.34×) | 229 vs crypto-bigint 903 | −674 (0.25×) |
| div, constant time | 256 | 509 vs crypto-bigint 1,696 | −1,187 (0.30×) | 246 vs crypto-bigint 1,362 | −1,116 (0.18×) | 623 vs crypto-bigint 1,972 | −1,349 (0.32×) | 478 vs crypto-bigint 1,518 | −1,040 (0.31×) |
| div, constant time | 384 | 632 vs crypto-bigint 2,516 | −1,884 (0.25×) | 524 vs crypto-bigint 2,016 | −1,492 (0.26×) | 842 vs crypto-bigint 3,049 | −2,207 (0.28×) | 555 vs crypto-bigint 2,257 | −1,702 (0.25×) |
| div, constant time | 512 | 990 vs crypto-bigint 3,541 | −2,551 (0.28×) | 714 vs crypto-bigint 2,768 | −2,054 (0.26×) | 1,148 vs crypto-bigint 4,284 | −3,136 (0.27×) | 841 vs crypto-bigint 3,098 | −2,257 (0.27×) |
| div, constant time | 1024 | 2,424 vs crypto-bigint 8,493 | −6,069 (0.29×) | 1,796 vs crypto-bigint 7,043 | −5,247 (0.26×) | 2,897 vs crypto-bigint 10,435 | −7,538 (0.28×) | 2,255 vs crypto-bigint 7,764 | −5,509 (0.29×) |
| div, constant time | 2048 | 7,350 vs crypto-bigint 22,487 | −15,137 (0.33×) | 5,079 vs crypto-bigint 16,909 | −11,830 (0.30×) | 8,549 vs crypto-bigint 27,542 | −18,993 (0.31×) | 6,940 vs crypto-bigint 18,805 | −11,865 (0.37×) |
| div, constant time | 4096 | 25,331 vs crypto-bigint 67,208 | −41,877 (0.38×) | 16,113 vs crypto-bigint 43,448 | −27,335 (0.37×) | 28,855 vs crypto-bigint 85,172 | −56,317 (0.34×) | 21,125 vs crypto-bigint 50,685 | −29,560 (0.42×) |
| mod, constant time | 64 | 85.1 vs crypto-bigint 380 | −295 (0.22×) | 47.2 vs crypto-bigint 327 | −280 (0.14×) | 149 vs crypto-bigint 422 | −273 (0.35×) | 66.8 vs crypto-bigint 339 | −272 (0.20×) |
| mod, constant time | 128 | 216 vs crypto-bigint 885 | −669 (0.24×) | 177 vs crypto-bigint 741 | −564 (0.24×) | 335 vs crypto-bigint 1,068 | −733 (0.31×) | 223 vs crypto-bigint 870 | −647 (0.26×) |
| mod, constant time | 256 | 513 vs crypto-bigint 1,629 | −1,116 (0.31×) | 339 vs crypto-bigint 1,341 | −1,002 (0.25×) | 587 vs crypto-bigint 1,964 | −1,377 (0.30×) | 399 vs crypto-bigint 1,498 | −1,099 (0.27×) |
| mod, constant time | 384 | 734 vs crypto-bigint 2,531 | −1,797 (0.29×) | 529 vs crypto-bigint 1,971 | −1,442 (0.27×) | 797 vs crypto-bigint 2,989 | −2,192 (0.27×) | 564 vs crypto-bigint 2,233 | −1,669 (0.25×) |
| mod, constant time | 512 | 1,060 vs crypto-bigint 3,515 | −2,455 (0.30×) | 698 vs crypto-bigint 2,751 | −2,053 (0.25×) | 1,092 vs crypto-bigint 4,288 | −3,196 (0.25×) | 785 vs crypto-bigint 3,101 | −2,316 (0.25×) |
| mod, constant time | 1024 | 2,507 vs crypto-bigint 8,475 | −5,968 (0.30×) | 1,822 vs crypto-bigint 7,021 | −5,199 (0.26×) | 2,771 vs crypto-bigint 10,387 | −7,616 (0.27×) | 1,991 vs crypto-bigint 7,488 | −5,497 (0.27×) |
| mod, constant time | 2048 | 7,625 vs crypto-bigint 22,423 | −14,798 (0.34×) | 5,045 vs crypto-bigint 16,892 | −11,847 (0.30×) | 8,592 vs crypto-bigint 28,028 | −19,436 (0.31×) | 6,107 vs crypto-bigint 18,784 | −12,677 (0.33×) |
| mod, constant time | 4096 | 25,923 vs crypto-bigint 67,693 | −41,770 (0.38×) | 16,265 vs crypto-bigint 46,956 | −30,691 (0.35×) | 28,828 vs crypto-bigint 85,218 | −56,390 (0.34×) | 20,263 vs crypto-bigint 50,546 | −30,283 (0.40×) |
| mulmod, constant time | 64 | 174 vs crypto-bigint 475 | −301 (0.37×) | 97.4 vs crypto-bigint 351 | −254 (0.28×) | 263 vs crypto-bigint 536 | −273 (0.49×) | 125 vs crypto-bigint 381 | −256 (0.33×) |
| mulmod, constant time | 128 | 561 vs crypto-bigint 1,285 | −724 (0.44×) | 358 vs crypto-bigint 998 | −640 (0.36×) | 636 vs crypto-bigint 1,581 | −945 (0.40×) | 375 vs crypto-bigint 1,145 | −770 (0.33×) |
| mulmod, constant time | 256 | 1,212 vs crypto-bigint 2,845 | −1,633 (0.43×) | 738 vs crypto-bigint 1,975 | −1,237 (0.37×) | 1,429 vs crypto-bigint 3,292 | −1,863 (0.43×) | 857 vs crypto-bigint 2,273 | −1,416 (0.38×) |
| mulmod, constant time | 384 | 1,994 vs crypto-bigint 4,905 | −2,911 (0.41×) | 1,202 vs crypto-bigint 3,226 | −2,024 (0.37×) | 2,676 vs crypto-bigint 5,797 | −3,121 (0.46×) | 1,478 vs crypto-bigint 3,654 | −2,176 (0.40×) |
| mulmod, constant time | 512 | 3,064 vs crypto-bigint 7,624 | −4,560 (0.40×) | 1,829 vs crypto-bigint 4,663 | −2,834 (0.39×) | 3,648 vs crypto-bigint 8,785 | −5,137 (0.42×) | 2,354 vs crypto-bigint 5,506 | −3,152 (0.43×) |
| mulmod, constant time | 1024 | 9,559 vs crypto-bigint 21,365 | −11,806 (0.45×) | 5,951 vs crypto-bigint 13,268 | −7,317 (0.45×) | 10,446 vs crypto-bigint 25,045 | −14,599 (0.42×) | 7,670 vs crypto-bigint 14,056 | −6,386 (0.55×) |
| mulmod, constant time | 2048 | 32,879 vs crypto-bigint 65,260 | −32,381 (0.50×) | 19,574 vs crypto-bigint 35,265 | −15,691 (0.56×) | 36,270 vs crypto-bigint 76,479 | −40,209 (0.47×) | 25,158 vs crypto-bigint 45,423 | −20,265 (0.55×) |
| mulmod, constant time | 4096 | 119,268 vs crypto-bigint 259,181 | −139,913 (0.46×) | 69,939 vs crypto-bigint 110,710 | −40,771 (0.63×) | 132,124 vs crypto-bigint 256,735 | −124,611 (0.51×) | 88,748 vs crypto-bigint 144,747 | −55,999 (0.61×) |
| add_mod, constant time | 64 | 178 vs crypto-bigint 12.9 | +165 (13.80×) | 110 vs crypto-bigint 12.2 | +97.8 (9.02×) | 222 vs crypto-bigint 11.8 | +210 (18.81×) | 137 vs crypto-bigint 13.1 | +124 (10.46×) |
| add_mod, constant time | 128 | 397 vs crypto-bigint 23.7 | +373 (16.75×) | 283 vs crypto-bigint 21.2 | +262 (13.35×) | 425 vs crypto-bigint 23.0 | +402 (18.48×) | 292 vs crypto-bigint 22.9 | +269 (12.75×) |
| add_mod, constant time | 256 | 551 vs crypto-bigint 47.4 | +504 (11.62×) | 404 vs crypto-bigint 42.3 | +362 (9.55×) | 680 vs crypto-bigint 57.1 | +623 (11.91×) | 473 vs crypto-bigint 54.8 | +418 (8.63×) |
| add_mod, constant time | 384 | 679 vs crypto-bigint 96.7 | +582 (7.02×) | 507 vs crypto-bigint 105 | +402 (4.83×) | 787 vs crypto-bigint 86.1 | +701 (9.14×) | 625 vs crypto-bigint 82.7 | +542 (7.56×) |
| add_mod, constant time | 512 | 807 vs crypto-bigint 108 | +699 (7.47×) | 594 vs crypto-bigint 93.1 | +501 (6.38×) | 923 vs crypto-bigint 142 | +781 (6.50×) | 715 vs crypto-bigint 127 | +588 (5.63×) |
| add_mod, constant time | 1024 | 1,441 vs crypto-bigint 377 | +1,064 (3.82×) | 1,066 vs crypto-bigint 345 | +721 (3.09×) | 1,594 vs crypto-bigint 461 | +1,133 (3.46×) | 1,252 vs crypto-bigint 383 | +869 (3.27×) |
| add_mod, constant time | 2048 | 2,677 vs crypto-bigint 573 | +2,104 (4.67×) | 2,050 vs crypto-bigint 505 | +1,545 (4.06×) | 2,976 vs crypto-bigint 882 | +2,094 (3.37×) | 2,528 vs crypto-bigint 769 | +1,759 (3.29×) |
| add_mod, constant time | 4096 | 5,234 vs crypto-bigint 1,159 | +4,075 (4.52×) | 3,868 vs crypto-bigint 993 | +2,875 (3.90×) | 5,589 vs crypto-bigint 1,600 | +3,989 (3.49×) | 4,640 vs crypto-bigint 1,404 | +3,236 (3.30×) |
| sub_mod, constant time | 64 | 183 vs crypto-bigint 5.0 | +178 (36.60×) | 106 vs crypto-bigint 5.0 | +101 (21.20×) | 221 vs crypto-bigint 5.0 | +216 (44.20×) | 108 vs crypto-bigint 5.0 | +103 (21.60×) |
| sub_mod, constant time | 128 | 396 vs crypto-bigint 10.3 | +386 (38.45×) | 277 vs crypto-bigint 11.0 | +266 (25.18×) | 432 vs crypto-bigint 11.0 | +421 (39.27×) | 297 vs crypto-bigint 11.0 | +286 (27.00×) |
| sub_mod, constant time | 256 | 523 vs crypto-bigint 28.1 | +495 (18.61×) | 438 vs crypto-bigint 23.5 | +415 (18.64×) | 611 vs crypto-bigint 34.4 | +577 (17.76×) | 465 vs crypto-bigint 33.5 | +432 (13.88×) |
| sub_mod, constant time | 384 | 652 vs crypto-bigint 48.9 | +603 (13.33×) | 461 vs crypto-bigint 39.1 | +422 (11.79×) | 745 vs crypto-bigint 55.2 | +690 (13.50×) | 584 vs crypto-bigint 55.1 | +529 (10.60×) |
| sub_mod, constant time | 512 | 774 vs crypto-bigint 71.2 | +703 (10.87×) | 543 vs crypto-bigint 53.7 | +489 (10.11×) | 906 vs crypto-bigint 91.6 | +814 (9.89×) | 617 vs crypto-bigint 105 | +512 (5.88×) |
| sub_mod, constant time | 1024 | 1,389 vs crypto-bigint 177 | +1,212 (7.85×) | 973 vs crypto-bigint 142 | +831 (6.85×) | 1,611 vs crypto-bigint 348 | +1,263 (4.63×) | 1,256 vs crypto-bigint 275 | +981 (4.57×) |
| sub_mod, constant time | 2048 | 2,660 vs crypto-bigint 519 | +2,141 (5.13×) | 1,896 vs crypto-bigint 459 | +1,437 (4.13×) | 3,041 vs crypto-bigint 563 | +2,478 (5.40×) | 2,356 vs crypto-bigint 512 | +1,844 (4.60×) |
| sub_mod, constant time | 4096 | 4,951 vs crypto-bigint 941 | +4,010 (5.26×) | 3,652 vs crypto-bigint 773 | +2,879 (4.72×) | 5,321 vs crypto-bigint 1,474 | +3,847 (3.61×) | 4,230 vs crypto-bigint 1,351 | +2,879 (3.13×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.1 vs native 1.0 | +0.1 (1.10×) |
| add | 128 | 3.0 vs native 2.8 | +0.2 (1.07×) | 2.5 vs bnum 2.0 | +0.5 (1.25×) | 2.8 vs crypto-bigint 2.7 | +0.1 (1.04×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add | 256 | 9.8 vs bnum 8.1 | +1.7 (1.21×) | 6.8 vs bnum 6.3 | +0.5 (1.08×) | 8.9 vs ruint 9.0 | −0.1 (0.99×) | 11.4 vs bnum 7.7 | +3.7 (1.48×) |
| add | 384 | 15.4 vs ruint 15.4 | +0.0 (1.00×) | 12.2 vs crypto-bigint 12.4 | −0.2 (0.98×) | 15.4 vs ruint 15.2 | +0.2 (1.01×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add | 512 | 25.7 vs bnum 22.3 | +3.4 (1.15×) | 17.3 vs crypto-bigint 18.2 | −0.9 (0.95×) | 27.8 vs bnum 29.5 | −1.7 (0.94×) | 31.1 vs crypto-bigint 31.8 | −0.7 (0.98×) |
| add | 1024 | 64.0 vs bnum 62.1 | +1.9 (1.03×) | 43.6 vs crypto-bigint 44.2 | −0.6 (0.99×) | 71.1 vs ruint 68.9 | +2.2 (1.03×) | 71.4 vs crypto-bigint 63.8 | +7.6 (1.12×) |
| add | 2048 | 160 vs ruint 208 | −48.0 (0.77×) | 106 vs ruint 178 | −72.0 (0.60×) | 177 vs ruint 214 | −37.0 (0.83×) | 141 vs ruint 206 | −65.0 (0.68×) |
| add | 4096 | 424 vs ruint 404 | +20.0 (1.05×) | 364 vs ruint 292 | +72.0 (1.25×) | 426 vs ruint 429 | −3.0 (0.99×) | 400 vs ruint 391 | +9.0 (1.02×) |
| mulmod | 64 | 68.8 vs ruint 85.0 | −16.2 (0.81×) | 34.6 vs ruint 42.5 | −7.9 (0.81×) | 122 vs ruint 203 | −81.0 (0.60×) | 42.0 vs ruint 54.8 | −12.8 (0.77×) |
| mulmod | 128 | 206 vs ruint 339 | −133 (0.61×) | 86.1 vs ruint 138 | −51.9 (0.62×) | 255 vs ruint 432 | −177 (0.59×) | 167 vs ruint 208 | −41.0 (0.80×) |
| mulmod | 256 | 586 vs ruint 809 | −223 (0.72×) | 287 vs ruint 309 | −22.0 (0.93×) | 687 vs ruint 899 | −212 (0.76×) | 345 vs ruint 382 | −37.0 (0.90×) |
| mulmod | 384 | 1,038 vs ruint 1,600 | −562 (0.65×) | 503 vs ruint 536 | −33.0 (0.94×) | 1,409 vs ruint 1,697 | −288 (0.83×) | 661 vs ruint 690 | −29.0 (0.96×) |
| mulmod | 512 | 1,647 vs ruint 2,873 | −1,226 (0.57×) | 845 vs ruint 857 | −12.0 (0.99×) | 2,152 vs ruint 3,301 | −1,149 (0.65×) | 1,105 vs ruint 1,118 | −13.0 (0.99×) |
| mulmod | 1024 | 5,319 vs ruint 11,113 | −5,794 (0.48×) | 2,778 vs ruint 2,631 | +147 (1.06×) | 6,879 vs ruint 12,229 | −5,350 (0.56×) | 4,174 vs ruint 3,629 | +545 (1.15×) |
| mulmod | 2048 | 17,414 vs ruint 34,951 | −17,537 (0.50×) | 9,459 vs ruint 10,685 | −1,226 (0.89×) | 22,195 vs ruint 44,565 | −22,370 (0.50×) | 12,727 vs ruint 13,702 | −975 (0.93×) |
| mulmod | 4096 | 55,558 vs ruint 160,647 | −105,089 (0.35×) | 29,343 vs ruint 41,173 | −11,830 (0.71×) | 68,359 vs ruint 166,433 | −98,074 (0.41×) | 42,867 vs ruint 53,324 | −10,457 (0.80×) |
| add_mod | 64 | 7.3 vs ruint 39.5 | −32.2 (0.18×) | 11.1 vs ruint 40.4 | −29.3 (0.27×) | 8.0 vs ruint 78.1 | −70.1 (0.10×) | 12.3 vs ruint 76.9 | −64.6 (0.16×) |
| add_mod | 128 | 33.7 vs ruint 40.1 | −6.4 (0.84×) | 15.5 vs ruint 30.8 | −15.3 (0.50×) | 46.4 vs ruint 51.4 | −5.0 (0.90×) | 26.2 vs ruint 38.8 | −12.6 (0.68×) |
| add_mod | 256 | 36.3 vs ruint 74.0 | −37.7 (0.49×) | 31.0 vs ruint 60.1 | −29.1 (0.52×) | 50.6 vs ruint 91.1 | −40.5 (0.56×) | 51.4 vs ruint 77.8 | −26.4 (0.66×) |
| add_mod | 384 | 79.1 vs ruint 100 | −20.9 (0.79×) | 70.2 vs ruint 83.3 | −13.1 (0.84×) | 119 vs ruint 140 | −21.0 (0.85×) | 117 vs ruint 106 | +11.0 (1.10×) |
| add_mod | 512 | 87.9 vs ruint 129 | −41.1 (0.68×) | 60.1 vs ruint 101 | −40.9 (0.60×) | 130 vs ruint 175 | −45.0 (0.74×) | 102 vs ruint 140 | −38.0 (0.73×) |
| add_mod | 1024 | 170 vs ruint 272 | −102 (0.62×) | 134 vs ruint 203 | −69.0 (0.66×) | 220 vs ruint 365 | −145 (0.60×) | 214 vs ruint 294 | −80.0 (0.73×) |
| add_mod | 2048 | 329 vs ruint 856 | −527 (0.38×) | 251 vs ruint 778 | −527 (0.32×) | 368 vs ruint 1,121 | −753 (0.33×) | 356 vs ruint 952 | −596 (0.37×) |
| add_mod | 4096 | 720 vs ruint 1,701 | −981 (0.42×) | 586 vs ruint 1,454 | −868 (0.40×) | 759 vs ruint 1,933 | −1,174 (0.39×) | 770 vs ruint 1,686 | −916 (0.46×) |
| sub_mod | 64 | 7.9 vs ruint 46.1 | −38.2 (0.17×) | 9.3 vs ruint 45.3 | −36.0 (0.21×) | 10.0 vs ruint 76.3 | −66.3 (0.13×) | 11.7 vs ruint 78.3 | −66.6 (0.15×) |
| sub_mod | 128 | 34.5 vs ruint 45.3 | −10.8 (0.76×) | 13.9 vs ruint 40.7 | −26.8 (0.34×) | 47.6 vs ruint 65.5 | −17.9 (0.73×) | 25.5 vs ruint 52.1 | −26.6 (0.49×) |
| sub_mod | 256 | 46.5 vs ruint 95.8 | −49.3 (0.49×) | 33.4 vs ruint 79.3 | −45.9 (0.42×) | 63.7 vs ruint 131 | −67.3 (0.49×) | 53.7 vs ruint 119 | −65.3 (0.45×) |
| sub_mod | 384 | 70.2 vs ruint 129 | −58.8 (0.54×) | 47.9 vs ruint 107 | −59.1 (0.45×) | 102 vs ruint 193 | −91.0 (0.53×) | 84.2 vs ruint 174 | −89.8 (0.48×) |
| sub_mod | 512 | 116 vs ruint 168 | −52.0 (0.69×) | 65.6 vs ruint 138 | −72.4 (0.48×) | 136 vs ruint 250 | −114 (0.54×) | 110 vs ruint 219 | −109 (0.50×) |
| sub_mod | 1024 | 192 vs ruint 430 | −238 (0.45×) | 122 vs ruint 351 | −229 (0.35×) | 263 vs ruint 667 | −404 (0.39×) | 164 vs ruint 635 | −471 (0.26×) |
| sub_mod | 2048 | 393 vs ruint 1,214 | −821 (0.32×) | 236 vs ruint 1,103 | −867 (0.21×) | 677 vs ruint 1,469 | −792 (0.46×) | 340 vs ruint 1,341 | −1,001 (0.25×) |
| sub_mod | 4096 | 744 vs ruint 2,312 | −1,568 (0.32×) | 590 vs ruint 2,052 | −1,462 (0.29×) | 794 vs ruint 2,534 | −1,740 (0.31×) | 763 vs ruint 2,334 | −1,571 (0.33×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.1 vs native 1.0 | +0.1 (1.10×) |
| add, constant time | 128 | 3.0 vs native 2.8 | +0.2 (1.07×) | 2.5 vs bnum 2.0 | +0.5 (1.25×) | 2.8 vs crypto-bigint 2.7 | +0.1 (1.04×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add, constant time | 256 | 9.8 vs bnum 8.1 | +1.7 (1.21×) | 6.8 vs bnum 6.3 | +0.5 (1.08×) | 8.9 vs ruint 9.0 | −0.1 (0.99×) | 11.4 vs bnum 7.7 | +3.7 (1.48×) |
| add, constant time | 384 | 15.4 vs ruint 15.4 | +0.0 (1.00×) | 12.2 vs crypto-bigint 12.0 | +0.2 (1.02×) | 15.4 vs ruint 15.2 | +0.2 (1.01×) | 18.1 vs crypto-bigint 18.1 | +0.0 (1.00×) |
| add, constant time | 512 | 25.7 vs bnum 22.3 | +3.4 (1.15×) | 17.3 vs crypto-bigint 18.2 | −0.9 (0.95×) | 27.5 vs bnum 25.3 | +2.2 (1.09×) | 31.1 vs crypto-bigint 31.8 | −0.7 (0.98×) |
| add, constant time | 1024 | 64.0 vs bnum 62.1 | +1.9 (1.03×) | 43.6 vs crypto-bigint 44.2 | −0.6 (0.99×) | 71.1 vs ruint 68.9 | +2.2 (1.03×) | 71.3 vs crypto-bigint 63.7 | +7.6 (1.12×) |
| add, constant time | 2048 | 160 vs ruint 208 | −48.0 (0.77×) | 106 vs ruint 178 | −72.0 (0.60×) | 178 vs ruint 214 | −36.0 (0.83×) | 141 vs ruint 206 | −65.0 (0.68×) |
| add, constant time | 4096 | 423 vs ruint 405 | +18.0 (1.04×) | 364 vs ruint 292 | +72.0 (1.25×) | 427 vs ruint 429 | −2.0 (1.00×) | 386 vs ruint 391 | −5.0 (0.99×) |
| mulmod, constant time | 64 | 178 | – | 94.0 | – | 252 | – | 115 | – |
| mulmod, constant time | 128 | 568 | – | 328 | – | 618 | – | 378 | – |
| mulmod, constant time | 256 | 1,160 | – | 734 | – | 1,382 | – | 879 | – |
| mulmod, constant time | 384 | 1,989 | – | 1,199 | – | 2,433 | – | 1,524 | – |
| mulmod, constant time | 512 | 3,039 | – | 1,813 | – | 3,664 | – | 2,291 | – |
| mulmod, constant time | 1024 | 9,494 | – | 6,015 | – | 10,972 | – | 7,625 | – |
| mulmod, constant time | 2048 | 32,786 | – | 19,863 | – | 35,622 | – | 25,208 | – |
| mulmod, constant time | 4096 | 120,120 | – | 69,404 | – | 130,197 | – | 89,535 | – |
| add_mod, constant time | 64 | 139 | – | 86.6 | – | 198 | – | 130 | – |
| add_mod, constant time | 128 | 392 | – | 285 | – | 427 | – | 296 | – |
| add_mod, constant time | 256 | 542 | – | 395 | – | 636 | – | 462 | – |
| add_mod, constant time | 384 | 665 | – | 491 | – | 798 | – | 605 | – |
| add_mod, constant time | 512 | 792 | – | 580 | – | 915 | – | 705 | – |
| add_mod, constant time | 1024 | 1,401 | – | 1,074 | – | 1,574 | – | 1,297 | – |
| add_mod, constant time | 2048 | 2,723 | – | 1,984 | – | 2,902 | – | 2,478 | – |
| add_mod, constant time | 4096 | 5,282 | – | 3,835 | – | 5,603 | – | 4,699 | – |
| sub_mod, constant time | 64 | 91.1 | – | 78.8 | – | 198 | – | 118 | – |
| sub_mod, constant time | 128 | 390 | – | 288 | – | 445 | – | 300 | – |
| sub_mod, constant time | 256 | 518 | – | 366 | – | 608 | – | 430 | – |
| sub_mod, constant time | 384 | 641 | – | 473 | – | 735 | – | 573 | – |
| sub_mod, constant time | 512 | 786 | – | 542 | – | 905 | – | 718 | – |
| sub_mod, constant time | 1024 | 1,374 | – | 979 | – | 1,601 | – | 1,141 | – |
| sub_mod, constant time | 2048 | 2,714 | – | 1,861 | – | 2,981 | – | 2,279 | – |
| sub_mod, constant time | 4096 | 5,006 | – | 3,615 | – | 5,260 | – | 4,216 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.3 | 2.2 (1.7×) | 2.2 (1.7×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.4 (1.8×) |
| add | 256 | 4.9 | 10.2 (2.1×) | 8.7 (1.8×) | 10.0 (2.0×) | 11.5 (2.3×) |
| add | 384 | 7.2 | 16.3 (2.3×) | 12.3 (1.7×) | 15.9 (2.2×) | 18.0 (2.5×) |
| add | 512 | 8.6 | 27.4 (3.2×) | 20.7 (2.4×) | 25.9 (3.0×) | 32.1 (3.7×) |
| add | 1024 | 24.2 | 64.1 (2.6×) | 45.3 (1.9×) | 69.2 (2.9×) | 74.0 (3.1×) |
| add | 2048 | 55.0 | 157 (2.9×) | 109 (2.0×) | 185 (3.4×) | 147 (2.7×) |
| add | 4096 | 156 | 430 (2.8×) | 382 (2.4×) | 429 (2.8×) | 405 (2.6×) |
| mul | 64 | 1.5 | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul | 128 | 3.0 | 10.0 (3.3×) | 6.5 (2.2×) | 12.3 (4.1×) | 5.6 (1.9×) |
| mul | 256 | 12.0 | 47.7 (4.0×) | 16.8 (1.4×) | 66.6 (5.5×) | 26.0 (2.2×) |
| mul | 384 | 27.5 | 181 (6.6×) | 52.3 (1.9×) | 228 (8.3×) | 70.3 (2.6×) |
| mul | 512 | 52.2 | 295 (5.7×) | 70.7 (1.4×) | 363 (7.0×) | 131 (2.5×) |
| mul | 1024 | 239 | 1,340 (5.6×) | 691 (2.9×) | 1,594 (6.7×) | 846 (3.5×) |
| mul | 2048 | 1,117 | 5,613 (5.0×) | 2,500 (2.2×) | 6,696 (6.0×) | 3,697 (3.3×) |
| mul | 4096 | 3,853 | 17,687 (4.6×) | 8,430 (2.2×) | 21,172 (5.5×) | 12,443 (3.2×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.5 | 60.1 (3.6×) | 28.3 (1.7×) | 103 (6.2×) | 36.4 (2.2×) |
| div | 256 | 57.5 | 216 (3.8×) | 87.0 (1.5×) | 221 (3.8×) | 157 (2.7×) |
| div | 384 | 119 | 351 (2.9×) | 203 (1.7×) | 402 (3.4×) | 295 (2.5×) |
| div | 512 | 161 | 509 (3.2×) | 295 (1.8×) | 575 (3.6×) | 395 (2.5×) |
| div | 1024 | 585 | 1,196 (2.0×) | 696 (1.2×) | 1,493 (2.6×) | 877 (1.5×) |
| div | 2048 | 1,417 | 3,242 (2.3×) | 2,005 (1.4×) | 4,377 (3.1×) | 2,459 (1.7×) |
| div | 4096 | 4,752 | 12,603 (2.7×) | 6,829 (1.4×) | 15,027 (3.2×) | 7,306 (1.5×) |
| mod | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 18.6 | 52.8 (2.8×) | 30.0 (1.6×) | 96.3 (5.2×) | 33.1 (1.8×) |
| mod | 256 | 53.3 | 217 (4.1×) | 82.0 (1.5×) | 245 (4.6×) | 138 (2.6×) |
| mod | 384 | 121 | 351 (2.9×) | 195 (1.6×) | 412 (3.4×) | 273 (2.3×) |
| mod | 512 | 166 | 486 (2.9×) | 279 (1.7×) | 569 (3.4×) | 353 (2.1×) |
| mod | 1024 | 463 | 1,217 (2.6×) | 720 (1.6×) | 1,498 (3.2×) | 948 (2.0×) |
| mod | 2048 | 1,328 | 3,347 (2.5×) | 1,955 (1.5×) | 4,274 (3.2×) | 2,223 (1.7×) |
| mod | 4096 | 4,774 | 12,491 (2.6×) | 6,494 (1.4×) | 14,884 (3.1×) | 6,911 (1.4×) |
| mulmod | 64 | 11.9 | 61.0 (5.1×) | 31.6 (2.7×) | 104 (8.7×) | 35.8 (3.0×) |
| mulmod | 128 | 61.9 | 197 (3.2×) | 81.7 (1.3×) | 247 (4.0×) | 158 (2.6×) |
| mulmod | 256 | 178 | 579 (3.3×) | 281 (1.6×) | 701 (3.9×) | 352 (2.0×) |
| mulmod | 384 | 361 | 1,041 (2.9×) | 507 (1.4×) | 1,414 (3.9×) | 669 (1.9×) |
| mulmod | 512 | 567 | 1,652 (2.9×) | 849 (1.5×) | 2,158 (3.8×) | 1,176 (2.1×) |
| mulmod | 1024 | 1,387 | 5,332 (3.8×) | 2,764 (2.0×) | 7,741 (5.6×) | 4,219 (3.0×) |
| mulmod | 2048 | 5,893 | 17,381 (2.9×) | 9,432 (1.6×) | 22,251 (3.8×) | 12,708 (2.2×) |
| mulmod | 4096 | 16,763 | 55,642 (3.3×) | 29,321 (1.7×) | 68,366 (4.1×) | 42,732 (2.5×) |
| add_mod | 64 | 3.7 | 5.0 (1.4×) | 6.0 (1.6×) | 6.2 (1.7×) | 9.0 (2.4×) |
| add_mod | 128 | 4.8 | 15.0 (3.1×) | 13.0 (2.7×) | 19.9 (4.1×) | 20.2 (4.2×) |
| add_mod | 256 | 11.2 | 27.4 (2.4×) | 26.6 (2.4×) | 53.7 (4.8×) | 47.3 (4.2×) |
| add_mod | 384 | 22.4 | 62.2 (2.8×) | 66.2 (3.0×) | 104 (4.6×) | 107 (4.8×) |
| add_mod | 512 | 43.1 | 73.8 (1.7×) | 44.5 (1.0×) | 120 (2.8×) | 88.8 (2.1×) |
| add_mod | 1024 | 102 | 148 (1.5×) | 128 (1.3×) | 190 (1.9×) | 191 (1.9×) |
| add_mod | 2048 | 196 | 291 (1.5×) | 218 (1.1×) | 328 (1.7×) | 319 (1.6×) |
| add_mod | 4096 | 350 | 634 (1.8×) | 489 (1.4×) | 687 (2.0×) | 729 (2.1×) |
| sub_mod | 64 | 4.1 | 5.0 (1.2×) | 6.0 (1.5×) | 8.0 (2.0×) | 10.0 (2.4×) |
| sub_mod | 128 | 5.8 | 14.0 (2.4×) | 12.0 (2.1×) | 21.0 (3.6×) | 20.1 (3.5×) |
| sub_mod | 256 | 10.9 | 32.8 (3.0×) | 27.3 (2.5×) | 53.6 (4.9×) | 55.4 (5.1×) |
| sub_mod | 384 | 19.8 | 53.1 (2.7×) | 33.1 (1.7×) | 87.6 (4.4×) | 66.4 (3.4×) |
| sub_mod | 512 | 34.3 | 80.4 (2.3×) | 50.1 (1.5×) | 107 (3.1×) | 89.8 (2.6×) |
| sub_mod | 1024 | 89.2 | 150 (1.7×) | 104 (1.2×) | 192 (2.2×) | 144 (1.6×) |
| sub_mod | 2048 | 183 | 382 (2.1×) | 231 (1.3×) | 578 (3.2×) | 295 (1.6×) |
| sub_mod | 4096 | 300 | 656 (2.2×) | 492 (1.6×) | 666 (2.2×) | 643 (2.1×) |
| add, constant time | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.4 (1.8×) |
| add, constant time | 256 | 4.5 | 10.2 (2.3×) | 8.7 (1.9×) | 10.0 (2.2×) | 11.5 (2.6×) |
| add, constant time | 384 | 7.1 | 16.3 (2.3×) | 12.3 (1.7×) | 15.9 (2.2×) | 18.0 (2.5×) |
| add, constant time | 512 | 9.1 | 27.5 (3.0×) | 20.8 (2.3×) | 26.0 (2.9×) | 32.1 (3.5×) |
| add, constant time | 1024 | 23.6 | 64.0 (2.7×) | 45.4 (1.9×) | 69.3 (2.9×) | 74.0 (3.1×) |
| add, constant time | 2048 | 63.8 | 157 (2.5×) | 108 (1.7×) | 185 (2.9×) | 146 (2.3×) |
| add, constant time | 4096 | 216 | 429 (2.0×) | 379 (1.8×) | 428 (2.0×) | 405 (1.9×) |
| mul, constant time | 64 | 2.5 | 2.1 (0.8×) | 2.2 (0.9×) | 2.2 (0.9×) | 2.2 (0.9×) |
| mul, constant time | 128 | 2.9 | 10.0 (3.4×) | 6.5 (2.2×) | 12.3 (4.2×) | 5.6 (1.9×) |
| mul, constant time | 256 | 12.4 | 47.8 (3.9×) | 16.8 (1.4×) | 67.3 (5.4×) | 26.0 (2.1×) |
| mul, constant time | 384 | 29.6 | 164 (5.5×) | 52.3 (1.8×) | 217 (7.3×) | 70.2 (2.4×) |
| mul, constant time | 512 | 51.4 | 298 (5.8×) | 72.3 (1.4×) | 357 (6.9×) | 131 (2.5×) |
| mul, constant time | 1024 | 227 | 1,344 (5.9×) | 693 (3.1×) | 1,580 (7.0×) | 862 (3.8×) |
| mul, constant time | 2048 | 1,161 | 5,787 (5.0×) | 2,558 (2.2×) | 6,752 (5.8×) | 3,888 (3.3×) |
| mul, constant time | 4096 | 4,193 | 18,263 (4.4×) | 8,862 (2.1×) | 21,877 (5.2×) | 12,853 (3.1×) |
| div, constant time | 64 | 25.5 | 83.6 (3.3×) | 47.0 (1.8×) | 144 (5.6×) | 59.5 (2.3×) |
| div, constant time | 128 | 116 | 220 (1.9×) | 183 (1.6×) | 362 (3.1×) | 229 (2.0×) |
| div, constant time | 256 | 270 | 509 (1.9×) | 246 (0.9×) | 623 (2.3×) | 478 (1.8×) |
| div, constant time | 384 | 400 | 632 (1.6×) | 524 (1.3×) | 842 (2.1×) | 555 (1.4×) |
| div, constant time | 512 | 363 | 990 (2.7×) | 714 (2.0×) | 1,148 (3.2×) | 841 (2.3×) |
| div, constant time | 1024 | 1,144 | 2,424 (2.1×) | 1,796 (1.6×) | 2,897 (2.5×) | 2,255 (2.0×) |
| div, constant time | 2048 | 3,438 | 7,350 (2.1×) | 5,079 (1.5×) | 8,549 (2.5×) | 6,940 (2.0×) |
| div, constant time | 4096 | 12,251 | 25,331 (2.1×) | 16,113 (1.3×) | 28,855 (2.4×) | 21,125 (1.7×) |
| mod, constant time | 64 | 25.8 | 85.1 (3.3×) | 47.2 (1.8×) | 149 (5.8×) | 66.8 (2.6×) |
| mod, constant time | 128 | 112 | 216 (1.9×) | 177 (1.6×) | 335 (3.0×) | 223 (2.0×) |
| mod, constant time | 256 | 297 | 513 (1.7×) | 339 (1.1×) | 587 (2.0×) | 399 (1.3×) |
| mod, constant time | 384 | 305 | 734 (2.4×) | 529 (1.7×) | 797 (2.6×) | 564 (1.8×) |
| mod, constant time | 512 | 492 | 1,060 (2.2×) | 698 (1.4×) | 1,092 (2.2×) | 785 (1.6×) |
| mod, constant time | 1024 | 1,194 | 2,507 (2.1×) | 1,822 (1.5×) | 2,771 (2.3×) | 1,991 (1.7×) |
| mod, constant time | 2048 | 4,101 | 7,625 (1.9×) | 5,045 (1.2×) | 8,592 (2.1×) | 6,107 (1.5×) |
| mod, constant time | 4096 | 13,281 | 25,923 (2.0×) | 16,265 (1.2×) | 28,828 (2.2×) | 20,263 (1.5×) |
| mulmod, constant time | 64 | 71.3 | 174 (2.4×) | 97.4 (1.4×) | 263 (3.7×) | 125 (1.8×) |
| mulmod, constant time | 128 | 263 | 561 (2.1×) | 358 (1.4×) | 636 (2.4×) | 375 (1.4×) |
| mulmod, constant time | 256 | 439 | 1,212 (2.8×) | 738 (1.7×) | 1,429 (3.3×) | 857 (2.0×) |
| mulmod, constant time | 384 | 895 | 1,994 (2.2×) | 1,202 (1.3×) | 2,676 (3.0×) | 1,478 (1.7×) |
| mulmod, constant time | 512 | 1,510 | 3,064 (2.0×) | 1,829 (1.2×) | 3,648 (2.4×) | 2,354 (1.6×) |
| mulmod, constant time | 1024 | 4,334 | 9,559 (2.2×) | 5,951 (1.4×) | 10,446 (2.4×) | 7,670 (1.8×) |
| mulmod, constant time | 2048 | 14,964 | 32,879 (2.2×) | 19,574 (1.3×) | 36,270 (2.4×) | 25,158 (1.7×) |
| mulmod, constant time | 4096 | 51,856 | 119,268 (2.3×) | 69,939 (1.3×) | 132,124 (2.5×) | 88,748 (1.7×) |
| add_mod, constant time | 64 | 5.8 | 178 (30.7×) | 110 (19.0×) | 222 (38.3×) | 137 (23.6×) |
| add_mod, constant time | 128 | 21.7 | 397 (18.3×) | 283 (13.0×) | 425 (19.6×) | 292 (13.5×) |
| add_mod, constant time | 256 | 30.4 | 551 (18.1×) | 404 (13.3×) | 680 (22.4×) | 473 (15.6×) |
| add_mod, constant time | 384 | 58.8 | 679 (11.5×) | 507 (8.6×) | 787 (13.4×) | 625 (10.6×) |
| add_mod, constant time | 512 | 72.9 | 807 (11.1×) | 594 (8.1×) | 923 (12.7×) | 715 (9.8×) |
| add_mod, constant time | 1024 | 112 | 1,441 (12.9×) | 1,066 (9.5×) | 1,594 (14.2×) | 1,252 (11.2×) |
| add_mod, constant time | 2048 | 587 | 2,677 (4.6×) | 2,050 (3.5×) | 2,976 (5.1×) | 2,528 (4.3×) |
| add_mod, constant time | 4096 | 1,020 | 5,234 (5.1×) | 3,868 (3.8×) | 5,589 (5.5×) | 4,640 (4.5×) |
| sub_mod, constant time | 64 | 9.8 | 183 (18.7×) | 106 (10.8×) | 221 (22.6×) | 108 (11.0×) |
| sub_mod, constant time | 128 | 17.8 | 396 (22.2×) | 277 (15.6×) | 432 (24.3×) | 297 (16.7×) |
| sub_mod, constant time | 256 | 29.7 | 523 (17.6×) | 438 (14.7×) | 611 (20.6×) | 465 (15.7×) |
| sub_mod, constant time | 384 | 47.6 | 652 (13.7×) | 461 (9.7×) | 745 (15.7×) | 584 (12.3×) |
| sub_mod, constant time | 512 | 60.8 | 774 (12.7×) | 543 (8.9×) | 906 (14.9×) | 617 (10.1×) |
| sub_mod, constant time | 1024 | 139 | 1,389 (10.0×) | 973 (7.0×) | 1,611 (11.6×) | 1,256 (9.0×) |
| sub_mod, constant time | 2048 | 286 | 2,660 (9.3×) | 1,896 (6.6×) | 3,041 (10.6×) | 2,356 (8.2×) |
| sub_mod, constant time | 4096 | 557 | 4,951 (8.9×) | 3,652 (6.6×) | 5,321 (9.6×) | 4,230 (7.6×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.1 (1.1×) |
| add | 128 | 0.8 | 3.0 (3.8×) | 2.5 (3.1×) | 2.8 (3.5×) | 3.2 (4.0×) |
| add | 256 | 2.5 | 9.8 (3.9×) | 6.8 (2.7×) | 8.9 (3.6×) | 11.4 (4.6×) |
| add | 384 | 3.5 | 15.4 (4.4×) | 12.2 (3.5×) | 15.4 (4.4×) | 18.1 (5.2×) |
| add | 512 | 4.0 | 25.7 (6.4×) | 17.3 (4.3×) | 27.8 (7.0×) | 31.1 (7.8×) |
| add | 1024 | 23.7 | 64.0 (2.7×) | 43.6 (1.8×) | 71.1 (3.0×) | 71.4 (3.0×) |
| add | 2048 | 54.3 | 160 (2.9×) | 106 (2.0×) | 177 (3.3×) | 141 (2.6×) |
| add | 4096 | 176 | 424 (2.4×) | 364 (2.1×) | 426 (2.4×) | 400 (2.3×) |
| mulmod | 64 | 22.9 | 68.8 (3.0×) | 34.6 (1.5×) | 122 (5.3×) | 42.0 (1.8×) |
| mulmod | 128 | 60.2 | 206 (3.4×) | 86.1 (1.4×) | 255 (4.2×) | 167 (2.8×) |
| mulmod | 256 | 165 | 586 (3.6×) | 287 (1.7×) | 687 (4.2×) | 345 (2.1×) |
| mulmod | 384 | 373 | 1,038 (2.8×) | 503 (1.3×) | 1,409 (3.8×) | 661 (1.8×) |
| mulmod | 512 | 568 | 1,647 (2.9×) | 845 (1.5×) | 2,152 (3.8×) | 1,105 (1.9×) |
| mulmod | 1024 | 1,373 | 5,319 (3.9×) | 2,778 (2.0×) | 6,879 (5.0×) | 4,174 (3.0×) |
| mulmod | 2048 | 4,807 | 17,414 (3.6×) | 9,459 (2.0×) | 22,195 (4.6×) | 12,727 (2.6×) |
| mulmod | 4096 | 17,710 | 55,558 (3.1×) | 29,343 (1.7×) | 68,359 (3.9×) | 42,867 (2.4×) |
| add_mod | 64 | 8.5 | 7.3 (0.9×) | 11.1 (1.3×) | 8.0 (0.9×) | 12.3 (1.4×) |
| add_mod | 128 | 11.7 | 33.7 (2.9×) | 15.5 (1.3×) | 46.4 (4.0×) | 26.2 (2.2×) |
| add_mod | 256 | 20.7 | 36.3 (1.8×) | 31.0 (1.5×) | 50.6 (2.4×) | 51.4 (2.5×) |
| add_mod | 384 | 32.3 | 79.1 (2.4×) | 70.2 (2.2×) | 119 (3.7×) | 117 (3.6×) |
| add_mod | 512 | 31.8 | 87.9 (2.8×) | 60.1 (1.9×) | 130 (4.1×) | 102 (3.2×) |
| add_mod | 1024 | 87.6 | 170 (1.9×) | 134 (1.5×) | 220 (2.5×) | 214 (2.4×) |
| add_mod | 2048 | 190 | 329 (1.7×) | 251 (1.3×) | 368 (1.9×) | 356 (1.9×) |
| add_mod | 4096 | 374 | 720 (1.9×) | 586 (1.6×) | 759 (2.0×) | 770 (2.1×) |
| sub_mod | 64 | 7.4 | 7.9 (1.1×) | 9.3 (1.3×) | 10.0 (1.4×) | 11.7 (1.6×) |
| sub_mod | 128 | 11.3 | 34.5 (3.1×) | 13.9 (1.2×) | 47.6 (4.2×) | 25.5 (2.3×) |
| sub_mod | 256 | 16.1 | 46.5 (2.9×) | 33.4 (2.1×) | 63.7 (4.0×) | 53.7 (3.3×) |
| sub_mod | 384 | 39.0 | 70.2 (1.8×) | 47.9 (1.2×) | 102 (2.6×) | 84.2 (2.2×) |
| sub_mod | 512 | 43.3 | 116 (2.7×) | 65.6 (1.5×) | 136 (3.1×) | 110 (2.5×) |
| sub_mod | 1024 | 79.6 | 192 (2.4×) | 122 (1.5×) | 263 (3.3×) | 164 (2.1×) |
| sub_mod | 2048 | 190 | 393 (2.1×) | 236 (1.2×) | 677 (3.6×) | 340 (1.8×) |
| sub_mod | 4096 | 435 | 744 (1.7×) | 590 (1.4×) | 794 (1.8×) | 763 (1.8×) |
| add, constant time | 64 | 0.9 | 1.1 (1.2×) | 1.1 (1.2×) | 1.0 (1.1×) | 1.1 (1.2×) |
| add, constant time | 128 | 1.2 | 3.0 (2.5×) | 2.5 (2.1×) | 2.8 (2.3×) | 3.2 (2.7×) |
| add, constant time | 256 | 2.2 | 9.8 (4.5×) | 6.8 (3.1×) | 8.9 (4.0×) | 11.4 (5.2×) |
| add, constant time | 384 | 3.5 | 15.4 (4.4×) | 12.2 (3.5×) | 15.4 (4.4×) | 18.1 (5.2×) |
| add, constant time | 512 | 3.2 | 25.7 (8.0×) | 17.3 (5.4×) | 27.5 (8.6×) | 31.1 (9.7×) |
| add, constant time | 1024 | 22.1 | 64.0 (2.9×) | 43.6 (2.0×) | 71.1 (3.2×) | 71.3 (3.2×) |
| add, constant time | 2048 | 54.3 | 160 (2.9×) | 106 (2.0×) | 178 (3.3×) | 141 (2.6×) |
| add, constant time | 4096 | 222 | 423 (1.9×) | 364 (1.6×) | 427 (1.9×) | 386 (1.7×) |
| mulmod, constant time | 64 | 95.0 | 178 (1.9×) | 94.0 (1.0×) | 252 (2.7×) | 115 (1.2×) |
| mulmod, constant time | 128 | 270 | 568 (2.1×) | 328 (1.2×) | 618 (2.3×) | 378 (1.4×) |
| mulmod, constant time | 256 | 376 | 1,160 (3.1×) | 734 (2.0×) | 1,382 (3.7×) | 879 (2.3×) |
| mulmod, constant time | 384 | 954 | 1,989 (2.1×) | 1,199 (1.3×) | 2,433 (2.6×) | 1,524 (1.6×) |
| mulmod, constant time | 512 | 1,226 | 3,039 (2.5×) | 1,813 (1.5×) | 3,664 (3.0×) | 2,291 (1.9×) |
| mulmod, constant time | 1024 | 4,192 | 9,494 (2.3×) | 6,015 (1.4×) | 10,972 (2.6×) | 7,625 (1.8×) |
| mulmod, constant time | 2048 | 14,626 | 32,786 (2.2×) | 19,863 (1.4×) | 35,622 (2.4×) | 25,208 (1.7×) |
| mulmod, constant time | 4096 | 51,232 | 120,120 (2.3×) | 69,404 (1.4×) | 130,197 (2.5×) | 89,535 (1.7×) |
| add_mod, constant time | 64 | 18.9 | 139 (7.4×) | 86.6 (4.6×) | 198 (10.5×) | 130 (6.9×) |
| add_mod, constant time | 128 | 22.7 | 392 (17.3×) | 285 (12.6×) | 427 (18.8×) | 296 (13.0×) |
| add_mod, constant time | 256 | 26.2 | 542 (20.7×) | 395 (15.1×) | 636 (24.3×) | 462 (17.6×) |
| add_mod, constant time | 384 | 46.2 | 665 (14.4×) | 491 (10.6×) | 798 (17.3×) | 605 (13.1×) |
| add_mod, constant time | 512 | 73.1 | 792 (10.8×) | 580 (7.9×) | 915 (12.5×) | 705 (9.6×) |
| add_mod, constant time | 1024 | 155 | 1,401 (9.0×) | 1,074 (6.9×) | 1,574 (10.2×) | 1,297 (8.4×) |
| add_mod, constant time | 2048 | 396 | 2,723 (6.9×) | 1,984 (5.0×) | 2,902 (7.3×) | 2,478 (6.3×) |
| add_mod, constant time | 4096 | 849 | 5,282 (6.2×) | 3,835 (4.5×) | 5,603 (6.6×) | 4,699 (5.5×) |
| sub_mod, constant time | 64 | 9.3 | 91.1 (9.8×) | 78.8 (8.5×) | 198 (21.3×) | 118 (12.7×) |
| sub_mod, constant time | 128 | 19.7 | 390 (19.8×) | 288 (14.6×) | 445 (22.6×) | 300 (15.2×) |
| sub_mod, constant time | 256 | 25.6 | 518 (20.2×) | 366 (14.3×) | 608 (23.8×) | 430 (16.8×) |
| sub_mod, constant time | 384 | 43.3 | 641 (14.8×) | 473 (10.9×) | 735 (17.0×) | 573 (13.2×) |
| sub_mod, constant time | 512 | 57.0 | 786 (13.8×) | 542 (9.5×) | 905 (15.9×) | 718 (12.6×) |
| sub_mod, constant time | 1024 | 124 | 1,374 (11.1×) | 979 (7.9×) | 1,601 (12.9×) | 1,141 (9.2×) |
| sub_mod, constant time | 2048 | 300 | 2,714 (9.0×) | 1,861 (6.2×) | 2,981 (9.9×) | 2,279 (7.6×) |
| sub_mod, constant time | 4096 | 614 | 5,006 (8.2×) | 3,615 (5.9×) | 5,260 (8.6×) | 4,216 (6.9×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
