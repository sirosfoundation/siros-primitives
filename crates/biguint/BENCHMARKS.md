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

`biguint` at commit `e517319` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,416 to 18,395 cycles and 3,081 instructions; its noise in a figure, 39.0 to 9,048 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 6,497 to 16,606 cycles and 3,081 instructions; its noise in a figure, 202 to 6,439 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 8,176 to 12,928 cycles and 1,885 instructions; its noise in a figure, 471 to 5,016 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 7,919 to 14,790 cycles and 1,885 instructions; its noise in a figure, 485 to 9,840 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 24.1 before, 40.73 after; its SMT sibling, CPU 28, 65% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 25.73 before, 20.82 after; its SMT sibling, CPU 28, 48% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 20.82 before, 29.8 after; its SMT sibling, CPU 28, 38% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 29.8 before, 19.08 after; its SMT sibling, CPU 28, 50% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |


### Against the best competitor, throughput

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 2.2 vs native 2.1 | +0.1 (1.05×) | 2.2 vs bnum 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| add | 128 | 4.0 vs native 4.0 | +0.0 (1.00×) | 3.8 vs native 3.7 | +0.1 (1.03×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.3 vs native 3.4 | +0.9 (1.26×) |
| add | 256 | 10.2 vs ruint 10.2 | +0.0 (1.00×) | 8.7 vs ruint 9.0 | −0.3 (0.97×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add | 384 | 16.5 vs ruint 16.4 | +0.1 (1.01×) | 12.3 vs crypto-bigint 12.2 | +0.1 (1.01×) | 15.9 vs ruint 15.9 | +0.0 (1.00×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add | 512 | 27.0 vs bnum 24.8 | +2.2 (1.09×) | 20.6 vs crypto-bigint 20.9 | −0.3 (0.99×) | 32.2 vs bnum 23.3 | +8.9 (1.38×) | 32.6 vs bnum 27.0 | +5.6 (1.21×) |
| add | 1024 | 63.8 vs bnum 62.0 | +1.8 (1.03×) | 45.6 vs crypto-bigint 46.1 | −0.5 (0.99×) | 72.2 vs ruint 67.5 | +4.7 (1.07×) | 69.2 vs crypto-bigint 68.2 | +1.0 (1.01×) |
| add | 2048 | 157 vs crypto-bigint 193 | −36.0 (0.81×) | 106 vs crypto-bigint 149 | −43.0 (0.71×) | 189 vs bnum 208 | −19.0 (0.91×) | 148 vs crypto-bigint 178 | −30.0 (0.83×) |
| add | 4096 | 424 vs crypto-bigint 390 | +34.0 (1.09×) | 339 vs crypto-bigint 264 | +75.0 (1.28×) | 428 vs bnum 395 | +33.0 (1.08×) | 418 vs crypto-bigint 327 | +91.0 (1.28×) |
| mul | 64 | 2.2 vs bnum 2.1 | +0.1 (1.05×) | 2.0 vs native 2.2 | −0.2 (0.91×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.6 vs bnum 4.1 | +2.5 (1.61×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.7 vs crypto-bigint 5.0 | +0.7 (1.14×) |
| mul | 256 | 47.9 vs ruint 66.6 | −18.7 (0.72×) | 17.4 vs ruint 16.6 | +0.8 (1.05×) | 67.0 vs uint 85.6 | −18.6 (0.78×) | 26.0 vs ruint 26.4 | −0.4 (0.98×) |
| mul | 384 | 174 vs uint 162 | +12.0 (1.07×) | 51.5 vs crypto-bigint 41.2 | +10.3 (1.25×) | 211 vs ruint 297 | −86.0 (0.71×) | 73.1 vs ruint 73.7 | −0.6 (0.99×) |
| mul | 512 | 296 vs ruint 323 | −27.0 (0.92×) | 83.9 vs ruint 79.4 | +4.5 (1.06×) | 376 vs ruint 566 | −190 (0.66×) | 133 vs ruint 134 | −1.0 (0.99×) |
| mul | 1024 | 1,339 vs bnum 1,541 | −202 (0.87×) | 694 vs uint 509 | +185 (1.36×) | 1,759 vs bnum 2,393 | −634 (0.74×) | 853 vs crypto-bigint 910 | −57.0 (0.94×) |
| mul | 2048 | 5,614 vs bnum 6,332 | −718 (0.89×) | 2,556 vs ruint 2,155 | +401 (1.19×) | 6,676 vs bnum 8,732 | −2,056 (0.76×) | 3,722 vs ruint 3,218 | +504 (1.16×) |
| mul | 4096 | 17,715 vs bnum 23,632 | −5,917 (0.75×) | 8,467 vs ruint 8,142 | +325 (1.04×) | 21,613 vs bnum 33,519 | −11,906 (0.64×) | 12,413 vs ruint 11,652 | +761 (1.07×) |
| div | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| div | 128 | 59.8 vs ruint 85.4 | −25.6 (0.70×) | 28.4 vs ruint 41.4 | −13.0 (0.69×) | 98.5 vs ruint 142 | −43.5 (0.69×) | 37.0 vs ruint 55.9 | −18.9 (0.66×) |
| div | 256 | 219 vs ruint 321 | −102 (0.68×) | 86.7 vs ruint 156 | −69.3 (0.56×) | 234 vs ruint 402 | −168 (0.58×) | 157 vs ruint 176 | −19.0 (0.89×) |
| div | 384 | 348 vs ruint 608 | −260 (0.57×) | 204 vs ruint 355 | −151 (0.57×) | 408 vs ruint 772 | −364 (0.53×) | 292 vs ruint 411 | −119 (0.71×) |
| div | 512 | 505 vs num-bigint 806 | −301 (0.63×) | 288 vs ruint 427 | −139 (0.67×) | 581 vs num-bigint 1,029 | −448 (0.56×) | 395 vs ruint 548 | −153 (0.72×) |
| div | 1024 | 1,205 vs num-bigint 1,942 | −737 (0.62×) | 714 vs ruint 890 | −176 (0.80×) | 1,523 vs num-bigint 2,234 | −711 (0.68×) | 933 vs ruint 1,106 | −173 (0.84×) |
| div | 2048 | 3,241 vs num-bigint 5,798 | −2,557 (0.56×) | 2,039 vs ruint 2,359 | −320 (0.86×) | 4,312 vs num-bigint 6,277 | −1,965 (0.69×) | 2,447 vs ruint 2,684 | −237 (0.91×) |
| div | 4096 | 12,683 vs num-bigint 20,120 | −7,437 (0.63×) | 6,826 vs ruint 7,612 | −786 (0.90×) | 14,829 vs num-bigint 20,446 | −5,617 (0.73×) | 7,315 vs ruint 9,017 | −1,702 (0.81×) |
| mod | 64 | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) | 7.0 vs native 7.0 | +0.0 (1.00×) |
| mod | 128 | 52.7 vs ruint 87.8 | −35.1 (0.60×) | 29.7 vs ruint 40.2 | −10.5 (0.74×) | 95.7 vs ruint 99.5 | −3.8 (0.96×) | 34.7 vs ruint 51.7 | −17.0 (0.67×) |
| mod | 256 | 215 vs ruint 321 | −106 (0.67×) | 81.9 vs ruint 158 | −76.1 (0.52×) | 221 vs ruint 400 | −179 (0.55×) | 141 vs ruint 175 | −34.0 (0.81×) |
| mod | 384 | 360 vs ruint 584 | −224 (0.62×) | 194 vs ruint 322 | −128 (0.60×) | 413 vs ruint 723 | −310 (0.57×) | 276 vs ruint 342 | −66.0 (0.81×) |
| mod | 512 | 491 vs ruint 797 | −306 (0.62×) | 278 vs ruint 407 | −129 (0.68×) | 565 vs ruint 991 | −426 (0.57×) | 362 vs ruint 472 | −110 (0.77×) |
| mod | 1024 | 1,222 vs num-bigint 1,940 | −718 (0.63×) | 737 vs ruint 951 | −214 (0.77×) | 1,535 vs num-bigint 2,265 | −730 (0.68×) | 994 vs ruint 1,053 | −59.0 (0.94×) |
| mod | 2048 | 3,341 vs num-bigint 5,832 | −2,491 (0.57×) | 1,983 vs ruint 2,287 | −304 (0.87×) | 4,288 vs num-bigint 6,312 | −2,024 (0.68×) | 2,144 vs ruint 2,681 | −537 (0.80×) |
| mod | 4096 | 12,556 vs num-bigint 20,221 | −7,665 (0.62×) | 6,476 vs ruint 7,850 | −1,374 (0.82×) | 14,819 vs num-bigint 20,507 | −5,688 (0.72×) | 7,411 vs ruint 8,939 | −1,528 (0.83×) |
| mulmod | 64 | 61.6 vs ruint 84.8 | −23.2 (0.73×) | 31.4 vs ruint 39.9 | −8.5 (0.79×) | 106 vs ruint 189 | −83.0 (0.56×) | 35.9 vs ruint 57.6 | −21.7 (0.62×) |
| mulmod | 128 | 202 vs ruint 324 | −122 (0.62×) | 81.8 vs ruint 135 | −53.2 (0.61×) | 248 vs ruint 397 | −149 (0.62×) | 161 vs ruint 203 | −42.0 (0.79×) |
| mulmod | 256 | 592 vs ruint 811 | −219 (0.73×) | 282 vs ruint 309 | −27.0 (0.91×) | 707 vs ruint 907 | −200 (0.78×) | 374 vs ruint 386 | −12.0 (0.97×) |
| mulmod | 384 | 1,042 vs ruint 1,604 | −562 (0.65×) | 509 vs ruint 540 | −31.0 (0.94×) | 1,393 vs ruint 1,706 | −313 (0.82×) | 685 vs ruint 685 | +0.0 (1.00×) |
| mulmod | 512 | 1,682 vs ruint 2,801 | −1,119 (0.60×) | 837 vs ruint 855 | −18.0 (0.98×) | 2,122 vs num-bigint 3,059 | −937 (0.69×) | 1,193 vs ruint 1,141 | +52.0 (1.05×) |
| mulmod | 1024 | 5,315 vs num-bigint 8,917 | −3,602 (0.60×) | 2,783 vs ruint 2,616 | +167 (1.06×) | 6,678 vs num-bigint 9,317 | −2,639 (0.72×) | 4,253 vs ruint 3,602 | +651 (1.18×) |
| mulmod | 2048 | 17,419 vs num-bigint 32,073 | −14,654 (0.54×) | 9,518 vs ruint 10,703 | −1,185 (0.89×) | 22,455 vs num-bigint 31,938 | −9,483 (0.70×) | 12,712 vs ruint 13,838 | −1,126 (0.92×) |
| mulmod | 4096 | 55,790 vs num-bigint 112,539 | −56,749 (0.50×) | 29,563 vs ruint 41,146 | −11,583 (0.72×) | 68,481 vs num-bigint 113,517 | −45,036 (0.60×) | 42,723 vs ruint 53,581 | −10,858 (0.80×) |
| add_mod | 64 | 7.2 vs uint 3.3 | +3.9 (2.18×) | 5.0 vs native 4.0 | +1.0 (1.25×) | 11.6 vs native 3.7 | +7.9 (3.14×) | 11.1 vs native 5.0 | +6.1 (2.22×) |
| add_mod | 128 | 11.5 vs native 10.1 | +1.4 (1.14×) | 13.9 vs uint 8.9 | +5.0 (1.56×) | 16.6 vs native 9.6 | +7.0 (1.73×) | 17.8 vs native 9.1 | +8.7 (1.96×) |
| add_mod | 256 | 23.0 vs uint 22.1 | +0.9 (1.04×) | 22.2 vs uint 21.5 | +0.7 (1.03×) | 34.4 vs uint 21.1 | +13.3 (1.63×) | 41.6 vs uint 24.3 | +17.3 (1.71×) |
| add_mod | 384 | 32.2 vs uint 37.4 | −5.2 (0.86×) | 26.5 vs uint 39.6 | −13.1 (0.67×) | 46.9 vs uint 33.3 | +13.6 (1.41×) | 54.6 vs uint 40.5 | +14.1 (1.35×) |
| add_mod | 512 | 48.8 vs uint 54.2 | −5.4 (0.90×) | 42.3 vs uint 44.4 | −2.1 (0.95×) | 67.7 vs uint 47.6 | +20.1 (1.42×) | 70.9 vs uint 67.8 | +3.1 (1.05×) |
| add_mod | 1024 | 159 vs uint 108 | +51.0 (1.47×) | 104 vs uint 108 | −4.0 (0.96×) | 174 vs uint 141 | +33.0 (1.23×) | 135 vs uint 137 | −2.0 (0.99×) |
| add_mod | 2048 | 344 vs uint 276 | +68.0 (1.25×) | 236 vs uint 242 | −6.0 (0.98×) | 503 vs uint 286 | +217 (1.76×) | 313 vs uint 288 | +25.0 (1.09×) |
| add_mod | 4096 | 658 vs uint 629 | +29.0 (1.05×) | 505 vs uint 616 | −111 (0.82×) | 668 vs bnum 673 | −5.0 (0.99×) | 712 vs uint 658 | +54.0 (1.08×) |
| sub_mod | 64 | 6.5 vs native 3.0 | +3.5 (2.17×) | 5.9 vs native 3.0 | +2.9 (1.97×) | 11.6 vs native 3.0 | +8.6 (3.87×) | 11.5 vs native 3.0 | +8.5 (3.83×) |
| sub_mod | 128 | 14.5 vs native 6.7 | +7.8 (2.16×) | 14.7 vs native 5.4 | +9.3 (2.72×) | 17.0 vs native 7.4 | +9.6 (2.30×) | 22.5 vs native 6.8 | +15.7 (3.31×) |
| sub_mod | 256 | 25.4 vs uint 17.7 | +7.7 (1.44×) | 24.3 vs bnum 16.4 | +7.9 (1.48×) | 43.6 vs bnum 19.0 | +24.6 (2.29×) | 32.5 vs bnum 20.2 | +12.3 (1.61×) |
| sub_mod | 384 | 31.0 vs uint 32.5 | −1.5 (0.95×) | 26.2 vs uint 32.5 | −6.3 (0.81×) | 42.3 vs bnum 33.7 | +8.6 (1.26×) | 50.6 vs bnum 44.0 | +6.6 (1.15×) |
| sub_mod | 512 | 48.2 vs bnum 51.5 | −3.3 (0.94×) | 35.4 vs bnum 45.9 | −10.5 (0.77×) | 50.0 vs bnum 49.0 | +1.0 (1.02×) | 64.8 vs bnum 59.6 | +5.2 (1.09×) |
| sub_mod | 1024 | 133 vs uint 136 | −3.0 (0.98×) | 106 vs uint 130 | −24.0 (0.82×) | 146 vs uint 151 | −5.0 (0.97×) | 134 vs uint 160 | −26.0 (0.84×) |
| sub_mod | 2048 | 286 vs uint 294 | −8.0 (0.97×) | 253 vs uint 304 | −51.0 (0.83×) | 346 vs uint 375 | −29.0 (0.92×) | 294 vs uint 393 | −99.0 (0.75×) |
| sub_mod | 4096 | 617 vs bnum 729 | −112 (0.85×) | 555 vs uint 717 | −162 (0.77×) | 677 vs uint 773 | −96.0 (0.88×) | 616 vs uint 761 | −145 (0.81×) |
| add, constant time | 64 | 2.2 vs crypto-bigint 2.1 | +0.1 (1.05×) | 2.2 vs ruint 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs bnum 2.1 | +0.1 (1.05×) |
| add, constant time | 128 | 3.9 vs native 4.0 | −0.1 (0.97×) | 3.9 vs native 3.7 | +0.2 (1.05×) | 4.0 vs native 3.7 | +0.3 (1.08×) | 4.3 vs native 3.4 | +0.9 (1.26×) |
| add, constant time | 256 | 10.7 vs ruint 10.2 | +0.5 (1.05×) | 9.0 vs ruint 9.0 | +0.0 (1.00×) | 10.0 vs ruint 10.1 | −0.1 (0.99×) | 11.5 vs bnum 11.0 | +0.5 (1.05×) |
| add, constant time | 384 | 16.4 vs ruint 16.5 | −0.1 (0.99×) | 12.4 vs crypto-bigint 12.3 | +0.1 (1.01×) | 16.0 vs ruint 16.2 | −0.2 (0.99×) | 18.0 vs bnum 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 27.2 vs bnum 24.8 | +2.4 (1.10×) | 20.6 vs crypto-bigint 20.9 | −0.3 (0.99×) | 29.9 vs bnum 23.3 | +6.6 (1.28×) | 32.6 vs bnum 27.1 | +5.5 (1.20×) |
| add, constant time | 1024 | 64.5 vs bnum 61.6 | +2.9 (1.05×) | 45.4 vs crypto-bigint 45.9 | −0.5 (0.99×) | 71.6 vs ruint 68.2 | +3.4 (1.05×) | 68.9 vs crypto-bigint 68.2 | +0.7 (1.01×) |
| add, constant time | 2048 | 162 vs crypto-bigint 195 | −33.0 (0.83×) | 109 vs crypto-bigint 149 | −40.0 (0.73×) | 188 vs bnum 209 | −21.0 (0.90×) | 147 vs crypto-bigint 178 | −31.0 (0.83×) |
| add, constant time | 4096 | 412 vs crypto-bigint 394 | +18.0 (1.05×) | 337 vs crypto-bigint 265 | +72.0 (1.27×) | 426 vs bnum 393 | +33.0 (1.08×) | 441 vs crypto-bigint 327 | +114 (1.35×) |
| mul, constant time | 64 | 2.2 vs native 2.1 | +0.1 (1.05×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) | 2.2 vs native 2.2 | +0.0 (1.00×) |
| mul, constant time | 128 | 10.0 vs ruint 11.0 | −1.0 (0.91×) | 6.6 vs native 4.2 | +2.4 (1.57×) | 12.3 vs native 13.5 | −1.2 (0.91×) | 5.6 vs crypto-bigint 5.0 | +0.6 (1.12×) |
| mul, constant time | 256 | 47.7 vs ruint 66.6 | −18.9 (0.72×) | 17.8 vs crypto-bigint 18.4 | −0.6 (0.97×) | 67.4 vs uint 85.6 | −18.2 (0.79×) | 26.0 vs ruint 26.5 | −0.5 (0.98×) |
| mul, constant time | 384 | 166 vs uint 162 | +4.0 (1.02×) | 52.3 vs crypto-bigint 40.1 | +12.2 (1.30×) | 216 vs ruint 297 | −81.0 (0.73×) | 72.6 vs ruint 74.1 | −1.5 (0.98×) |
| mul, constant time | 512 | 293 vs ruint 325 | −32.0 (0.90×) | 85.3 vs ruint 76.1 | +9.2 (1.12×) | 372 vs ruint 567 | −195 (0.66×) | 131 vs ruint 134 | −3.0 (0.98×) |
| mul, constant time | 1024 | 1,344 vs bnum 1,545 | −201 (0.87×) | 677 vs uint 503 | +174 (1.35×) | 1,733 vs bnum 2,402 | −669 (0.72×) | 870 vs crypto-bigint 907 | −37.0 (0.96×) |
| mul, constant time | 2048 | 5,778 vs bnum 6,332 | −554 (0.91×) | 2,594 vs crypto-bigint 2,751 | −157 (0.94×) | 6,648 vs bnum 8,732 | −2,084 (0.76×) | 3,912 vs bnum 3,422 | +490 (1.14×) |
| mul, constant time | 4096 | 18,507 vs bnum 23,632 | −5,125 (0.78×) | 9,097 vs bnum 10,209 | −1,112 (0.89×) | 21,802 vs bnum 33,519 | −11,717 (0.65×) | 12,893 vs bnum 12,882 | +11.0 (1.00×) |
| div, constant time | 64 | 83.3 vs crypto-bigint 381 | −298 (0.22×) | 46.7 vs crypto-bigint 320 | −273 (0.15×) | 145 vs crypto-bigint 427 | −282 (0.34×) | 59.9 vs crypto-bigint 338 | −278 (0.18×) |
| div, constant time | 128 | 228 vs crypto-bigint 904 | −676 (0.25×) | 183 vs crypto-bigint 753 | −570 (0.24×) | 356 vs crypto-bigint 1,074 | −718 (0.33×) | 230 vs crypto-bigint 905 | −675 (0.25×) |
| div, constant time | 256 | 510 vs crypto-bigint 1,663 | −1,153 (0.31×) | 336 vs crypto-bigint 1,364 | −1,028 (0.25×) | 623 vs crypto-bigint 1,972 | −1,349 (0.32×) | 474 vs crypto-bigint 1,518 | −1,044 (0.31×) |
| div, constant time | 384 | 695 vs crypto-bigint 2,516 | −1,821 (0.28×) | 526 vs crypto-bigint 2,017 | −1,491 (0.26×) | 832 vs crypto-bigint 3,072 | −2,240 (0.27×) | 573 vs crypto-bigint 2,267 | −1,694 (0.25×) |
| div, constant time | 512 | 1,008 vs crypto-bigint 3,561 | −2,553 (0.28×) | 717 vs crypto-bigint 2,771 | −2,054 (0.26×) | 1,167 vs crypto-bigint 4,288 | −3,121 (0.27×) | 859 vs crypto-bigint 3,099 | −2,240 (0.28×) |
| div, constant time | 1024 | 2,433 vs crypto-bigint 8,495 | −6,062 (0.29×) | 1,811 vs crypto-bigint 7,065 | −5,254 (0.26×) | 2,907 vs crypto-bigint 10,377 | −7,470 (0.28×) | 2,278 vs crypto-bigint 7,453 | −5,175 (0.31×) |
| div, constant time | 2048 | 7,333 vs crypto-bigint 22,546 | −15,213 (0.33×) | 5,042 vs crypto-bigint 16,893 | −11,851 (0.30×) | 8,691 vs crypto-bigint 27,671 | −18,980 (0.31×) | 6,675 vs crypto-bigint 18,601 | −11,926 (0.36×) |
| div, constant time | 4096 | 25,442 vs crypto-bigint 67,046 | −41,604 (0.38×) | 16,268 vs crypto-bigint 43,528 | −27,260 (0.37×) | 28,690 vs crypto-bigint 84,927 | −56,237 (0.34×) | 21,047 vs crypto-bigint 50,617 | −29,570 (0.42×) |
| mod, constant time | 64 | 85.2 vs crypto-bigint 380 | −295 (0.22×) | 47.4 vs crypto-bigint 326 | −279 (0.15×) | 151 vs crypto-bigint 424 | −273 (0.36×) | 67.4 vs crypto-bigint 338 | −271 (0.20×) |
| mod, constant time | 128 | 224 vs crypto-bigint 888 | −664 (0.25×) | 177 vs crypto-bigint 741 | −564 (0.24×) | 336 vs crypto-bigint 1,052 | −716 (0.32×) | 223 vs crypto-bigint 887 | −664 (0.25×) |
| mod, constant time | 256 | 520 vs crypto-bigint 1,649 | −1,129 (0.32×) | 339 vs crypto-bigint 1,340 | −1,001 (0.25×) | 587 vs crypto-bigint 1,969 | −1,382 (0.30×) | 401 vs crypto-bigint 1,509 | −1,108 (0.27×) |
| mod, constant time | 384 | 737 vs crypto-bigint 2,447 | −1,710 (0.30×) | 528 vs crypto-bigint 1,971 | −1,443 (0.27×) | 805 vs crypto-bigint 3,002 | −2,197 (0.27×) | 557 vs crypto-bigint 2,220 | −1,663 (0.25×) |
| mod, constant time | 512 | 1,056 vs crypto-bigint 3,680 | −2,624 (0.29×) | 698 vs crypto-bigint 2,748 | −2,050 (0.25×) | 1,085 vs crypto-bigint 4,280 | −3,195 (0.25×) | 785 vs crypto-bigint 3,108 | −2,323 (0.25×) |
| mod, constant time | 1024 | 2,514 vs crypto-bigint 8,513 | −5,999 (0.30×) | 1,817 vs crypto-bigint 7,083 | −5,266 (0.26×) | 2,810 vs crypto-bigint 10,373 | −7,563 (0.27×) | 2,050 vs crypto-bigint 7,507 | −5,457 (0.27×) |
| mod, constant time | 2048 | 7,594 vs crypto-bigint 22,442 | −14,848 (0.34×) | 5,048 vs crypto-bigint 16,898 | −11,850 (0.30×) | 8,298 vs crypto-bigint 27,598 | −19,300 (0.30×) | 5,987 vs crypto-bigint 18,575 | −12,588 (0.32×) |
| mod, constant time | 4096 | 25,936 vs crypto-bigint 67,484 | −41,548 (0.38×) | 16,303 vs crypto-bigint 43,502 | −27,199 (0.37×) | 28,740 vs crypto-bigint 85,488 | −56,748 (0.34×) | 20,184 vs crypto-bigint 50,545 | −30,361 (0.40×) |
| mulmod, constant time | 64 | 141 vs crypto-bigint 475 | −334 (0.30×) | 96.4 vs crypto-bigint 352 | −256 (0.27×) | 245 vs crypto-bigint 517 | −272 (0.47×) | 143 vs crypto-bigint 417 | −274 (0.34×) |
| mulmod, constant time | 128 | 574 vs crypto-bigint 1,280 | −706 (0.45×) | 365 vs crypto-bigint 1,007 | −642 (0.36×) | 593 vs crypto-bigint 1,531 | −938 (0.39×) | 381 vs crypto-bigint 1,146 | −765 (0.33×) |
| mulmod, constant time | 256 | 1,210 vs crypto-bigint 2,765 | −1,555 (0.44×) | 738 vs crypto-bigint 1,980 | −1,242 (0.37×) | 1,375 vs crypto-bigint 3,289 | −1,914 (0.42×) | 893 vs crypto-bigint 2,297 | −1,404 (0.39×) |
| mulmod, constant time | 384 | 1,989 vs crypto-bigint 4,815 | −2,826 (0.41×) | 1,208 vs crypto-bigint 3,222 | −2,014 (0.37×) | 2,329 vs crypto-bigint 5,686 | −3,357 (0.41×) | 1,638 vs crypto-bigint 3,773 | −2,135 (0.43×) |
| mulmod, constant time | 512 | 3,057 vs crypto-bigint 7,661 | −4,604 (0.40×) | 1,853 vs crypto-bigint 4,687 | −2,834 (0.40×) | 3,489 vs crypto-bigint 8,681 | −5,192 (0.40×) | 2,362 vs crypto-bigint 5,547 | −3,185 (0.43×) |
| mulmod, constant time | 1024 | 9,568 vs crypto-bigint 21,974 | −12,406 (0.44×) | 5,953 vs crypto-bigint 13,144 | −7,191 (0.45×) | 10,852 vs crypto-bigint 25,772 | −14,920 (0.42×) | 7,732 vs crypto-bigint 14,335 | −6,603 (0.54×) |
| mulmod, constant time | 2048 | 32,934 vs crypto-bigint 64,779 | −31,845 (0.51×) | 19,656 vs crypto-bigint 36,242 | −16,586 (0.54×) | 36,937 vs crypto-bigint 76,365 | −39,428 (0.48×) | 25,813 vs crypto-bigint 45,519 | −19,706 (0.57×) |
| mulmod, constant time | 4096 | 119,344 vs crypto-bigint 259,519 | −140,175 (0.46×) | 69,875 vs crypto-bigint 110,352 | −40,477 (0.63×) | 132,450 vs crypto-bigint 259,641 | −127,191 (0.51×) | 92,470 vs crypto-bigint 145,908 | −53,438 (0.63×) |
| add_mod, constant time | 64 | 10.5 vs crypto-bigint 14.3 | −3.8 (0.73×) | 9.0 vs crypto-bigint 12.2 | −3.2 (0.74×) | 18.4 vs crypto-bigint 11.9 | +6.5 (1.55×) | 15.4 vs crypto-bigint 13.2 | +2.2 (1.17×) |
| add_mod, constant time | 128 | 27.3 vs crypto-bigint 22.7 | +4.6 (1.20×) | 22.1 vs crypto-bigint 21.1 | +1.0 (1.05×) | 34.8 vs crypto-bigint 23.0 | +11.8 (1.51×) | 35.6 vs crypto-bigint 22.8 | +12.8 (1.56×) |
| add_mod, constant time | 256 | 102 vs crypto-bigint 47.1 | +54.9 (2.17×) | 79.7 vs crypto-bigint 41.0 | +38.7 (1.94×) | 117 vs crypto-bigint 57.3 | +59.7 (2.04×) | 100 vs crypto-bigint 54.8 | +45.2 (1.82×) |
| add_mod, constant time | 384 | 149 vs crypto-bigint 101 | +48.0 (1.48×) | 118 vs crypto-bigint 105 | +13.0 (1.12×) | 205 vs crypto-bigint 86.3 | +119 (2.38×) | 156 vs crypto-bigint 82.4 | +73.6 (1.89×) |
| add_mod, constant time | 512 | 212 vs crypto-bigint 102 | +110 (2.08×) | 147 vs crypto-bigint 93.6 | +53.4 (1.57×) | 289 vs crypto-bigint 140 | +149 (2.06×) | 206 vs crypto-bigint 131 | +75.0 (1.57×) |
| add_mod, constant time | 1024 | 524 vs crypto-bigint 383 | +141 (1.37×) | 314 vs crypto-bigint 345 | −31.0 (0.91×) | 841 vs crypto-bigint 466 | +375 (1.80×) | 512 vs crypto-bigint 395 | +117 (1.30×) |
| add_mod, constant time | 2048 | 1,248 vs crypto-bigint 578 | +670 (2.16×) | 688 vs crypto-bigint 503 | +185 (1.37×) | 2,394 vs crypto-bigint 880 | +1,514 (2.72×) | 1,150 vs crypto-bigint 768 | +382 (1.50×) |
| add_mod, constant time | 4096 | 1,954 vs crypto-bigint 1,204 | +750 (1.62×) | 1,726 vs crypto-bigint 992 | +734 (1.74×) | 2,020 vs crypto-bigint 1,618 | +402 (1.25×) | 2,034 vs crypto-bigint 1,400 | +634 (1.45×) |
| sub_mod, constant time | 64 | 10.6 vs crypto-bigint 5.0 | +5.6 (2.12×) | 9.3 vs crypto-bigint 5.0 | +4.3 (1.86×) | 18.4 vs crypto-bigint 5.0 | +13.4 (3.68×) | 14.8 vs crypto-bigint 5.0 | +9.8 (2.96×) |
| sub_mod, constant time | 128 | 24.5 vs crypto-bigint 10.2 | +14.3 (2.40×) | 19.3 vs crypto-bigint 11.0 | +8.3 (1.75×) | 35.2 vs crypto-bigint 10.6 | +24.6 (3.32×) | 30.2 vs crypto-bigint 11.0 | +19.2 (2.75×) |
| sub_mod, constant time | 256 | 89.8 vs crypto-bigint 28.1 | +61.7 (3.20×) | 71.8 vs crypto-bigint 23.6 | +48.2 (3.04×) | 118 vs crypto-bigint 31.8 | +86.2 (3.71×) | 79.3 vs crypto-bigint 30.4 | +48.9 (2.61×) |
| sub_mod, constant time | 384 | 142 vs crypto-bigint 49.0 | +93.0 (2.90×) | 103 vs crypto-bigint 39.2 | +63.8 (2.63×) | 187 vs crypto-bigint 62.3 | +125 (3.00×) | 99.5 vs crypto-bigint 52.3 | +47.2 (1.90×) |
| sub_mod, constant time | 512 | 191 vs crypto-bigint 71.3 | +120 (2.68×) | 140 vs crypto-bigint 53.8 | +86.2 (2.60×) | 267 vs crypto-bigint 103 | +164 (2.59×) | 177 vs crypto-bigint 105 | +72.0 (1.69×) |
| sub_mod, constant time | 1024 | 411 vs crypto-bigint 178 | +233 (2.31×) | 267 vs crypto-bigint 139 | +128 (1.92×) | 658 vs crypto-bigint 348 | +310 (1.89×) | 351 vs crypto-bigint 277 | +74.0 (1.27×) |
| sub_mod, constant time | 2048 | 887 vs crypto-bigint 525 | +362 (1.69×) | 502 vs crypto-bigint 460 | +42.0 (1.09×) | 1,692 vs crypto-bigint 568 | +1,124 (2.98×) | 612 vs crypto-bigint 519 | +93.0 (1.18×) |
| sub_mod, constant time | 4096 | 1,412 vs crypto-bigint 902 | +510 (1.57×) | 1,279 vs crypto-bigint 754 | +525 (1.70×) | 1,585 vs crypto-bigint 1,471 | +114 (1.08×) | 1,479 vs crypto-bigint 1,336 | +143 (1.11×) |

### Against the best competitor, latency

Cycles per operation of `biguint` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Bits | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add | 128 | 3.0 vs crypto-bigint 2.8 | +0.2 (1.07×) | 2.5 vs native 2.1 | +0.4 (1.19×) | 2.7 vs crypto-bigint 2.8 | −0.1 (0.96×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add | 256 | 9.7 vs bnum 8.0 | +1.7 (1.21×) | 6.8 vs bnum 6.2 | +0.6 (1.10×) | 9.0 vs bnum 9.0 | +0.0 (1.00×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add | 384 | 15.4 vs ruint 15.4 | +0.0 (1.00×) | 12.2 vs crypto-bigint 12.3 | −0.1 (0.99×) | 15.3 vs ruint 15.6 | −0.3 (0.98×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add | 512 | 25.7 vs bnum 22.4 | +3.3 (1.15×) | 17.3 vs crypto-bigint 18.7 | −1.4 (0.93×) | 26.5 vs bnum 22.5 | +4.0 (1.18×) | 32.3 vs bnum 28.1 | +4.2 (1.15×) |
| add | 1024 | 63.9 vs bnum 62.0 | +1.9 (1.03×) | 44.0 vs crypto-bigint 44.2 | −0.2 (1.00×) | 69.0 vs ruint 71.1 | −2.1 (0.97×) | 66.9 vs crypto-bigint 61.7 | +5.2 (1.08×) |
| add | 2048 | 163 vs ruint 209 | −46.0 (0.78×) | 106 vs crypto-bigint 183 | −77.0 (0.58×) | 175 vs ruint 219 | −44.0 (0.80×) | 141 vs ruint 207 | −66.0 (0.68×) |
| add | 4096 | 427 vs ruint 416 | +11.0 (1.03×) | 311 vs ruint 303 | +8.0 (1.03×) | 427 vs ruint 426 | +1.0 (1.00×) | 393 vs ruint 382 | +11.0 (1.03×) |
| mulmod | 64 | 68.6 vs ruint 85.2 | −16.6 (0.81×) | 34.7 vs ruint 43.0 | −8.3 (0.81×) | 121 vs ruint 176 | −55.0 (0.69×) | 40.6 vs ruint 56.4 | −15.8 (0.72×) |
| mulmod | 128 | 209 vs ruint 339 | −130 (0.62×) | 86.1 vs ruint 138 | −51.9 (0.62×) | 254 vs ruint 397 | −143 (0.64×) | 163 vs ruint 208 | −45.0 (0.78×) |
| mulmod | 256 | 582 vs ruint 810 | −228 (0.72×) | 291 vs ruint 310 | −19.0 (0.94×) | 697 vs ruint 904 | −207 (0.77×) | 344 vs ruint 382 | −38.0 (0.90×) |
| mulmod | 384 | 1,051 vs ruint 1,607 | −556 (0.65×) | 504 vs ruint 537 | −33.0 (0.94×) | 1,366 vs ruint 1,704 | −338 (0.80×) | 680 vs ruint 684 | −4.0 (0.99×) |
| mulmod | 512 | 1,653 vs ruint 2,772 | −1,119 (0.60×) | 845 vs ruint 853 | −8.0 (0.99×) | 2,085 vs ruint 3,253 | −1,168 (0.64×) | 1,166 vs ruint 1,132 | +34.0 (1.03×) |
| mulmod | 1024 | 5,319 vs ruint 10,880 | −5,561 (0.49×) | 2,775 vs ruint 2,630 | +145 (1.06×) | 6,744 vs ruint 12,716 | −5,972 (0.53×) | 4,264 vs ruint 3,564 | +700 (1.20×) |
| mulmod | 2048 | 17,419 vs ruint 34,885 | −17,466 (0.50×) | 9,438 vs ruint 10,701 | −1,263 (0.88×) | 22,556 vs ruint 44,632 | −22,076 (0.51×) | 12,844 vs ruint 13,840 | −996 (0.93×) |
| mulmod | 4096 | 55,724 vs ruint 161,517 | −105,793 (0.35×) | 29,414 vs ruint 41,188 | −11,774 (0.71×) | 68,728 vs ruint 164,986 | −96,258 (0.42×) | 42,954 vs ruint 53,535 | −10,581 (0.80×) |
| add_mod | 64 | 7.9 vs ruint 43.6 | −35.7 (0.18×) | 7.6 vs ruint 42.9 | −35.3 (0.18×) | 13.3 vs ruint 79.6 | −66.3 (0.17×) | 13.1 vs ruint 78.2 | −65.1 (0.17×) |
| add_mod | 128 | 23.4 vs ruint 43.2 | −19.8 (0.54×) | 13.4 vs ruint 31.3 | −17.9 (0.43×) | 38.1 vs ruint 53.3 | −15.2 (0.71×) | 17.9 vs ruint 39.3 | −21.4 (0.46×) |
| add_mod | 256 | 33.5 vs ruint 75.0 | −41.5 (0.45×) | 28.3 vs ruint 61.6 | −33.3 (0.46×) | 45.0 vs ruint 101 | −56.0 (0.45×) | 42.5 vs ruint 85.4 | −42.9 (0.50×) |
| add_mod | 384 | 59.2 vs ruint 96.5 | −37.3 (0.61×) | 42.3 vs ruint 79.3 | −37.0 (0.53×) | 75.3 vs ruint 134 | −58.7 (0.56×) | 68.7 vs ruint 118 | −49.3 (0.58×) |
| add_mod | 512 | 69.2 vs ruint 145 | −75.8 (0.48×) | 52.8 vs ruint 99.6 | −46.8 (0.53×) | 99.6 vs ruint 177 | −77.4 (0.56×) | 88.9 vs ruint 148 | −59.1 (0.60×) |
| add_mod | 1024 | 184 vs ruint 257 | −73.0 (0.72×) | 117 vs ruint 210 | −93.0 (0.56×) | 210 vs ruint 376 | −166 (0.56×) | 160 vs ruint 321 | −161 (0.50×) |
| add_mod | 2048 | 383 vs ruint 937 | −554 (0.41×) | 229 vs ruint 782 | −553 (0.29×) | 515 vs ruint 1,097 | −582 (0.47×) | 330 vs ruint 968 | −638 (0.34×) |
| add_mod | 4096 | 738 vs ruint 1,681 | −943 (0.44×) | 557 vs ruint 1,452 | −895 (0.38×) | 721 vs ruint 1,868 | −1,147 (0.39×) | 789 vs ruint 1,662 | −873 (0.47×) |
| sub_mod | 64 | 7.0 vs ruint 49.2 | −42.2 (0.14×) | 7.7 vs ruint 45.7 | −38.0 (0.17×) | 13.6 vs ruint 84.2 | −70.6 (0.16×) | 11.9 vs ruint 76.6 | −64.7 (0.16×) |
| sub_mod | 128 | 25.8 vs ruint 46.3 | −20.5 (0.56×) | 13.8 vs ruint 41.5 | −27.7 (0.33×) | 36.7 vs ruint 65.1 | −28.4 (0.56×) | 16.7 vs ruint 54.3 | −37.6 (0.31×) |
| sub_mod | 256 | 37.0 vs ruint 96.8 | −59.8 (0.38×) | 29.4 vs ruint 81.9 | −52.5 (0.36×) | 48.8 vs ruint 135 | −86.2 (0.36×) | 37.1 vs ruint 116 | −78.9 (0.32×) |
| sub_mod | 384 | 48.8 vs ruint 130 | −81.2 (0.38×) | 45.3 vs ruint 106 | −60.7 (0.43×) | 78.5 vs ruint 182 | −104 (0.43×) | 66.2 vs ruint 154 | −87.8 (0.43×) |
| sub_mod | 512 | 74.9 vs ruint 178 | −103 (0.42×) | 52.5 vs ruint 139 | −86.5 (0.38×) | 95.6 vs ruint 233 | −137 (0.41×) | 89.9 vs ruint 208 | −118 (0.43×) |
| sub_mod | 1024 | 165 vs ruint 457 | −292 (0.36×) | 115 vs ruint 358 | −243 (0.32×) | 200 vs ruint 650 | −450 (0.31×) | 173 vs ruint 620 | −447 (0.28×) |
| sub_mod | 2048 | 377 vs ruint 1,249 | −872 (0.30×) | 248 vs ruint 1,131 | −883 (0.22×) | 432 vs ruint 1,495 | −1,063 (0.29×) | 370 vs ruint 1,362 | −992 (0.27×) |
| sub_mod | 4096 | 756 vs ruint 2,478 | −1,722 (0.31×) | 640 vs ruint 2,091 | −1,451 (0.31×) | 750 vs ruint 2,729 | −1,979 (0.27×) | 777 vs ruint 2,365 | −1,588 (0.33×) |
| add, constant time | 64 | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.1 vs native 1.1 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) | 1.0 vs native 1.0 | +0.0 (1.00×) |
| add, constant time | 128 | 3.0 vs bnum 2.8 | +0.2 (1.07×) | 2.5 vs native 2.1 | +0.4 (1.19×) | 2.8 vs crypto-bigint 2.8 | +0.0 (1.00×) | 3.2 vs bnum 2.5 | +0.7 (1.28×) |
| add, constant time | 256 | 9.7 vs bnum 8.0 | +1.7 (1.21×) | 6.7 vs bnum 6.2 | +0.5 (1.08×) | 9.0 vs ruint 9.0 | +0.0 (1.00×) | 11.3 vs bnum 7.7 | +3.6 (1.47×) |
| add, constant time | 384 | 15.4 vs ruint 15.5 | −0.1 (0.99×) | 12.1 vs crypto-bigint 12.5 | −0.4 (0.97×) | 15.3 vs ruint 15.6 | −0.3 (0.98×) | 18.0 vs crypto-bigint 18.0 | +0.0 (1.00×) |
| add, constant time | 512 | 26.3 vs bnum 22.6 | +3.7 (1.16×) | 17.4 vs crypto-bigint 18.5 | −1.1 (0.94×) | 26.6 vs bnum 22.5 | +4.1 (1.18×) | 32.3 vs bnum 28.0 | +4.3 (1.15×) |
| add, constant time | 1024 | 72.3 vs bnum 62.0 | +10.3 (1.17×) | 43.7 vs ruint 44.4 | −0.7 (0.98×) | 69.4 vs ruint 71.2 | −1.8 (0.97×) | 66.9 vs crypto-bigint 61.8 | +5.1 (1.08×) |
| add, constant time | 2048 | 160 vs ruint 205 | −45.0 (0.78×) | 104 vs crypto-bigint 180 | −76.0 (0.58×) | 176 vs ruint 217 | −41.0 (0.81×) | 141 vs ruint 207 | −66.0 (0.68×) |
| add, constant time | 4096 | 419 vs ruint 414 | +5.0 (1.01×) | 311 vs ruint 302 | +9.0 (1.03×) | 425 vs ruint 425 | +0.0 (1.00×) | 393 vs ruint 381 | +12.0 (1.03×) |
| mulmod, constant time | 64 | 137 | – | 93.9 | – | 245 | – | 145 | – |
| mulmod, constant time | 128 | 576 | – | 349 | – | 619 | – | 368 | – |
| mulmod, constant time | 256 | 1,186 | – | 733 | – | 1,283 | – | 856 | – |
| mulmod, constant time | 384 | 2,002 | – | 1,211 | – | 2,294 | – | 1,511 | – |
| mulmod, constant time | 512 | 3,045 | – | 1,878 | – | 3,487 | – | 2,272 | – |
| mulmod, constant time | 1024 | 9,479 | – | 6,047 | – | 10,806 | – | 7,717 | – |
| mulmod, constant time | 2048 | 32,913 | – | 19,961 | – | 36,644 | – | 25,703 | – |
| mulmod, constant time | 4096 | 120,143 | – | 69,469 | – | 129,756 | – | 89,738 | – |
| add_mod, constant time | 64 | 12.3 | – | 13.0 | – | 19.5 | – | 14.5 | – |
| add_mod, constant time | 128 | 27.3 | – | 23.4 | – | 37.9 | – | 34.1 | – |
| add_mod, constant time | 256 | 111 | – | 76.0 | – | 112 | – | 90.1 | – |
| add_mod, constant time | 384 | 155 | – | 106 | – | 179 | – | 149 | – |
| add_mod, constant time | 512 | 211 | – | 137 | – | 285 | – | 201 | – |
| add_mod, constant time | 1024 | 530 | – | 313 | – | 829 | – | 445 | – |
| add_mod, constant time | 2048 | 1,279 | – | 689 | – | 2,341 | – | 1,111 | – |
| add_mod, constant time | 4096 | 2,004 | – | 1,727 | – | 1,990 | – | 2,059 | – |
| sub_mod, constant time | 64 | 12.2 | – | 11.0 | – | 19.5 | – | 14.3 | – |
| sub_mod, constant time | 128 | 24.2 | – | 18.7 | – | 33.7 | – | 28.9 | – |
| sub_mod, constant time | 256 | 88.5 | – | 66.8 | – | 89.7 | – | 65.7 | – |
| sub_mod, constant time | 384 | 133 | – | 97.0 | – | 161 | – | 113 | – |
| sub_mod, constant time | 512 | 189 | – | 130 | – | 260 | – | 174 | – |
| sub_mod, constant time | 1024 | 395 | – | 259 | – | 603 | – | 316 | – |
| sub_mod, constant time | 2048 | 892 | – | 494 | – | 1,538 | – | 685 | – |
| sub_mod, constant time | 4096 | 1,451 | – | 1,257 | – | 1,593 | – | 1,502 | – |

### Against native, throughput

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.3 | 2.2 (1.7×) | 2.2 (1.7×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add | 128 | 2.4 | 4.0 (1.7×) | 3.8 (1.6×) | 4.0 (1.7×) | 4.3 (1.8×) |
| add | 256 | 4.9 | 10.2 (2.1×) | 8.7 (1.8×) | 10.0 (2.0×) | 11.5 (2.3×) |
| add | 384 | 7.2 | 16.5 (2.3×) | 12.3 (1.7×) | 15.9 (2.2×) | 18.0 (2.5×) |
| add | 512 | 8.6 | 27.0 (3.1×) | 20.6 (2.4×) | 32.2 (3.7×) | 32.6 (3.8×) |
| add | 1024 | 24.2 | 63.8 (2.6×) | 45.6 (1.9×) | 72.2 (3.0×) | 69.2 (2.9×) |
| add | 2048 | 55.0 | 157 (2.9×) | 106 (1.9×) | 189 (3.4×) | 148 (2.7×) |
| add | 4096 | 156 | 424 (2.7×) | 339 (2.2×) | 428 (2.7×) | 418 (2.7×) |
| mul | 64 | 1.5 | 2.2 (1.5×) | 2.0 (1.3×) | 2.2 (1.5×) | 2.2 (1.5×) |
| mul | 128 | 3.0 | 10.0 (3.3×) | 6.6 (2.2×) | 12.3 (4.1×) | 5.7 (1.9×) |
| mul | 256 | 12.0 | 47.9 (4.0×) | 17.4 (1.4×) | 67.0 (5.6×) | 26.0 (2.2×) |
| mul | 384 | 27.5 | 174 (6.3×) | 51.5 (1.9×) | 211 (7.7×) | 73.1 (2.7×) |
| mul | 512 | 52.2 | 296 (5.7×) | 83.9 (1.6×) | 376 (7.2×) | 133 (2.5×) |
| mul | 1024 | 239 | 1,339 (5.6×) | 694 (2.9×) | 1,759 (7.4×) | 853 (3.6×) |
| mul | 2048 | 1,117 | 5,614 (5.0×) | 2,556 (2.3×) | 6,676 (6.0×) | 3,722 (3.3×) |
| mul | 4096 | 3,853 | 17,715 (4.6×) | 8,467 (2.2×) | 21,613 (5.6×) | 12,413 (3.2×) |
| div | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| div | 128 | 16.5 | 59.8 (3.6×) | 28.4 (1.7×) | 98.5 (6.0×) | 37.0 (2.2×) |
| div | 256 | 57.5 | 219 (3.8×) | 86.7 (1.5×) | 234 (4.1×) | 157 (2.7×) |
| div | 384 | 119 | 348 (2.9×) | 204 (1.7×) | 408 (3.4×) | 292 (2.5×) |
| div | 512 | 161 | 505 (3.1×) | 288 (1.8×) | 581 (3.6×) | 395 (2.5×) |
| div | 1024 | 585 | 1,205 (2.1×) | 714 (1.2×) | 1,523 (2.6×) | 933 (1.6×) |
| div | 2048 | 1,417 | 3,241 (2.3×) | 2,039 (1.4×) | 4,312 (3.0×) | 2,447 (1.7×) |
| div | 4096 | 4,752 | 12,683 (2.7×) | 6,826 (1.4×) | 14,829 (3.1×) | 7,315 (1.5×) |
| mod | 64 | 6.9 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mod | 128 | 18.6 | 52.7 (2.8×) | 29.7 (1.6×) | 95.7 (5.1×) | 34.7 (1.9×) |
| mod | 256 | 53.3 | 215 (4.0×) | 81.9 (1.5×) | 221 (4.1×) | 141 (2.6×) |
| mod | 384 | 121 | 360 (3.0×) | 194 (1.6×) | 413 (3.4×) | 276 (2.3×) |
| mod | 512 | 166 | 491 (3.0×) | 278 (1.7×) | 565 (3.4×) | 362 (2.2×) |
| mod | 1024 | 463 | 1,222 (2.6×) | 737 (1.6×) | 1,535 (3.3×) | 994 (2.1×) |
| mod | 2048 | 1,328 | 3,341 (2.5×) | 1,983 (1.5×) | 4,288 (3.2×) | 2,144 (1.6×) |
| mod | 4096 | 4,774 | 12,556 (2.6×) | 6,476 (1.4×) | 14,819 (3.1×) | 7,411 (1.6×) |
| mulmod | 64 | 11.9 | 61.6 (5.2×) | 31.4 (2.6×) | 106 (8.9×) | 35.9 (3.0×) |
| mulmod | 128 | 61.9 | 202 (3.3×) | 81.8 (1.3×) | 248 (4.0×) | 161 (2.6×) |
| mulmod | 256 | 178 | 592 (3.3×) | 282 (1.6×) | 707 (4.0×) | 374 (2.1×) |
| mulmod | 384 | 361 | 1,042 (2.9×) | 509 (1.4×) | 1,393 (3.9×) | 685 (1.9×) |
| mulmod | 512 | 567 | 1,682 (3.0×) | 837 (1.5×) | 2,122 (3.7×) | 1,193 (2.1×) |
| mulmod | 1024 | 1,387 | 5,315 (3.8×) | 2,783 (2.0×) | 6,678 (4.8×) | 4,253 (3.1×) |
| mulmod | 2048 | 5,893 | 17,419 (3.0×) | 9,518 (1.6×) | 22,455 (3.8×) | 12,712 (2.2×) |
| mulmod | 4096 | 16,763 | 55,790 (3.3×) | 29,563 (1.8×) | 68,481 (4.1×) | 42,723 (2.5×) |
| add_mod | 64 | 3.7 | 7.2 (1.9×) | 5.0 (1.4×) | 11.6 (3.1×) | 11.1 (3.0×) |
| add_mod | 128 | 4.8 | 11.5 (2.4×) | 13.9 (2.9×) | 16.6 (3.5×) | 17.8 (3.7×) |
| add_mod | 256 | 11.2 | 23.0 (2.1×) | 22.2 (2.0×) | 34.4 (3.1×) | 41.6 (3.7×) |
| add_mod | 384 | 22.4 | 32.2 (1.4×) | 26.5 (1.2×) | 46.9 (2.1×) | 54.6 (2.4×) |
| add_mod | 512 | 43.1 | 48.8 (1.1×) | 42.3 (1.0×) | 67.7 (1.6×) | 70.9 (1.6×) |
| add_mod | 1024 | 102 | 159 (1.6×) | 104 (1.0×) | 174 (1.7×) | 135 (1.3×) |
| add_mod | 2048 | 196 | 344 (1.8×) | 236 (1.2×) | 503 (2.6×) | 313 (1.6×) |
| add_mod | 4096 | 350 | 658 (1.9×) | 505 (1.4×) | 668 (1.9×) | 712 (2.0×) |
| sub_mod | 64 | 4.1 | 6.5 (1.6×) | 5.9 (1.4×) | 11.6 (2.8×) | 11.5 (2.8×) |
| sub_mod | 128 | 5.8 | 14.5 (2.5×) | 14.7 (2.5×) | 17.0 (2.9×) | 22.5 (3.9×) |
| sub_mod | 256 | 10.9 | 25.4 (2.3×) | 24.3 (2.2×) | 43.6 (4.0×) | 32.5 (3.0×) |
| sub_mod | 384 | 19.8 | 31.0 (1.6×) | 26.2 (1.3×) | 42.3 (2.1×) | 50.6 (2.6×) |
| sub_mod | 512 | 34.3 | 48.2 (1.4×) | 35.4 (1.0×) | 50.0 (1.5×) | 64.8 (1.9×) |
| sub_mod | 1024 | 89.2 | 133 (1.5×) | 106 (1.2×) | 146 (1.6×) | 134 (1.5×) |
| sub_mod | 2048 | 183 | 286 (1.6×) | 253 (1.4×) | 346 (1.9×) | 294 (1.6×) |
| sub_mod | 4096 | 300 | 617 (2.1×) | 555 (1.9×) | 677 (2.3×) | 616 (2.1×) |
| add, constant time | 64 | 1.4 | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add, constant time | 128 | 2.4 | 3.9 (1.6×) | 3.9 (1.6×) | 4.0 (1.7×) | 4.3 (1.8×) |
| add, constant time | 256 | 4.5 | 10.7 (2.4×) | 9.0 (2.0×) | 10.0 (2.2×) | 11.5 (2.6×) |
| add, constant time | 384 | 7.1 | 16.4 (2.3×) | 12.4 (1.7×) | 16.0 (2.3×) | 18.0 (2.5×) |
| add, constant time | 512 | 9.1 | 27.2 (3.0×) | 20.6 (2.3×) | 29.9 (3.3×) | 32.6 (3.6×) |
| add, constant time | 1024 | 23.6 | 64.5 (2.7×) | 45.4 (1.9×) | 71.6 (3.0×) | 68.9 (2.9×) |
| add, constant time | 2048 | 63.8 | 162 (2.5×) | 109 (1.7×) | 188 (2.9×) | 147 (2.3×) |
| add, constant time | 4096 | 216 | 412 (1.9×) | 337 (1.6×) | 426 (2.0×) | 441 (2.0×) |
| mul, constant time | 64 | 2.5 | 2.2 (0.9×) | 2.2 (0.9×) | 2.2 (0.9×) | 2.2 (0.9×) |
| mul, constant time | 128 | 2.9 | 10.0 (3.4×) | 6.6 (2.3×) | 12.3 (4.2×) | 5.6 (1.9×) |
| mul, constant time | 256 | 12.4 | 47.7 (3.8×) | 17.8 (1.4×) | 67.4 (5.4×) | 26.0 (2.1×) |
| mul, constant time | 384 | 29.6 | 166 (5.6×) | 52.3 (1.8×) | 216 (7.3×) | 72.6 (2.5×) |
| mul, constant time | 512 | 51.4 | 293 (5.7×) | 85.3 (1.7×) | 372 (7.2×) | 131 (2.5×) |
| mul, constant time | 1024 | 227 | 1,344 (5.9×) | 677 (3.0×) | 1,733 (7.6×) | 870 (3.8×) |
| mul, constant time | 2048 | 1,161 | 5,778 (5.0×) | 2,594 (2.2×) | 6,648 (5.7×) | 3,912 (3.4×) |
| mul, constant time | 4096 | 4,193 | 18,507 (4.4×) | 9,097 (2.2×) | 21,802 (5.2×) | 12,893 (3.1×) |
| div, constant time | 64 | 25.5 | 83.3 (3.3×) | 46.7 (1.8×) | 145 (5.7×) | 59.9 (2.3×) |
| div, constant time | 128 | 116 | 228 (2.0×) | 183 (1.6×) | 356 (3.1×) | 230 (2.0×) |
| div, constant time | 256 | 270 | 510 (1.9×) | 336 (1.2×) | 623 (2.3×) | 474 (1.8×) |
| div, constant time | 384 | 400 | 695 (1.7×) | 526 (1.3×) | 832 (2.1×) | 573 (1.4×) |
| div, constant time | 512 | 363 | 1,008 (2.8×) | 717 (2.0×) | 1,167 (3.2×) | 859 (2.4×) |
| div, constant time | 1024 | 1,144 | 2,433 (2.1×) | 1,811 (1.6×) | 2,907 (2.5×) | 2,278 (2.0×) |
| div, constant time | 2048 | 3,438 | 7,333 (2.1×) | 5,042 (1.5×) | 8,691 (2.5×) | 6,675 (1.9×) |
| div, constant time | 4096 | 12,251 | 25,442 (2.1×) | 16,268 (1.3×) | 28,690 (2.3×) | 21,047 (1.7×) |
| mod, constant time | 64 | 25.8 | 85.2 (3.3×) | 47.4 (1.8×) | 151 (5.9×) | 67.4 (2.6×) |
| mod, constant time | 128 | 112 | 224 (2.0×) | 177 (1.6×) | 336 (3.0×) | 223 (2.0×) |
| mod, constant time | 256 | 297 | 520 (1.8×) | 339 (1.1×) | 587 (2.0×) | 401 (1.4×) |
| mod, constant time | 384 | 305 | 737 (2.4×) | 528 (1.7×) | 805 (2.6×) | 557 (1.8×) |
| mod, constant time | 512 | 492 | 1,056 (2.1×) | 698 (1.4×) | 1,085 (2.2×) | 785 (1.6×) |
| mod, constant time | 1024 | 1,194 | 2,514 (2.1×) | 1,817 (1.5×) | 2,810 (2.4×) | 2,050 (1.7×) |
| mod, constant time | 2048 | 4,101 | 7,594 (1.9×) | 5,048 (1.2×) | 8,298 (2.0×) | 5,987 (1.5×) |
| mod, constant time | 4096 | 13,281 | 25,936 (2.0×) | 16,303 (1.2×) | 28,740 (2.2×) | 20,184 (1.5×) |
| mulmod, constant time | 64 | 71.3 | 141 (2.0×) | 96.4 (1.4×) | 245 (3.4×) | 143 (2.0×) |
| mulmod, constant time | 128 | 263 | 574 (2.2×) | 365 (1.4×) | 593 (2.3×) | 381 (1.4×) |
| mulmod, constant time | 256 | 439 | 1,210 (2.8×) | 738 (1.7×) | 1,375 (3.1×) | 893 (2.0×) |
| mulmod, constant time | 384 | 895 | 1,989 (2.2×) | 1,208 (1.3×) | 2,329 (2.6×) | 1,638 (1.8×) |
| mulmod, constant time | 512 | 1,510 | 3,057 (2.0×) | 1,853 (1.2×) | 3,489 (2.3×) | 2,362 (1.6×) |
| mulmod, constant time | 1024 | 4,334 | 9,568 (2.2×) | 5,953 (1.4×) | 10,852 (2.5×) | 7,732 (1.8×) |
| mulmod, constant time | 2048 | 14,964 | 32,934 (2.2×) | 19,656 (1.3×) | 36,937 (2.5×) | 25,813 (1.7×) |
| mulmod, constant time | 4096 | 51,856 | 119,344 (2.3×) | 69,875 (1.3×) | 132,450 (2.6×) | 92,470 (1.8×) |
| add_mod, constant time | 64 | 5.8 | 10.5 (1.8×) | 9.0 (1.6×) | 18.4 (3.2×) | 15.4 (2.7×) |
| add_mod, constant time | 128 | 21.7 | 27.3 (1.3×) | 22.1 (1.0×) | 34.8 (1.6×) | 35.6 (1.6×) |
| add_mod, constant time | 256 | 30.4 | 102 (3.4×) | 79.7 (2.6×) | 117 (3.8×) | 100 (3.3×) |
| add_mod, constant time | 384 | 58.8 | 149 (2.5×) | 118 (2.0×) | 205 (3.5×) | 156 (2.7×) |
| add_mod, constant time | 512 | 72.9 | 212 (2.9×) | 147 (2.0×) | 289 (4.0×) | 206 (2.8×) |
| add_mod, constant time | 1024 | 112 | 524 (4.7×) | 314 (2.8×) | 841 (7.5×) | 512 (4.6×) |
| add_mod, constant time | 2048 | 587 | 1,248 (2.1×) | 688 (1.2×) | 2,394 (4.1×) | 1,150 (2.0×) |
| add_mod, constant time | 4096 | 1,020 | 1,954 (1.9×) | 1,726 (1.7×) | 2,020 (2.0×) | 2,034 (2.0×) |
| sub_mod, constant time | 64 | 9.8 | 10.6 (1.1×) | 9.3 (0.9×) | 18.4 (1.9×) | 14.8 (1.5×) |
| sub_mod, constant time | 128 | 17.8 | 24.5 (1.4×) | 19.3 (1.1×) | 35.2 (2.0×) | 30.2 (1.7×) |
| sub_mod, constant time | 256 | 29.7 | 89.8 (3.0×) | 71.8 (2.4×) | 118 (4.0×) | 79.3 (2.7×) |
| sub_mod, constant time | 384 | 47.6 | 142 (3.0×) | 103 (2.2×) | 187 (3.9×) | 99.5 (2.1×) |
| sub_mod, constant time | 512 | 60.8 | 191 (3.1×) | 140 (2.3×) | 267 (4.4×) | 177 (2.9×) |
| sub_mod, constant time | 1024 | 139 | 411 (3.0×) | 267 (1.9×) | 658 (4.7×) | 351 (2.5×) |
| sub_mod, constant time | 2048 | 286 | 887 (3.1×) | 502 (1.8×) | 1,692 (5.9×) | 612 (2.1×) |
| sub_mod, constant time | 4096 | 557 | 1,412 (2.5×) | 1,279 (2.3×) | 1,585 (2.8×) | 1,479 (2.7×) |

### Against native, latency

Cycles per operation of `biguint` in each build, then its ratio to the native figure.

| Operation | Bits | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | 64 | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.0 (1.0×) |
| add | 128 | 0.8 | 3.0 (3.8×) | 2.5 (3.1×) | 2.7 (3.4×) | 3.2 (4.0×) |
| add | 256 | 2.5 | 9.7 (3.9×) | 6.8 (2.7×) | 9.0 (3.6×) | 11.3 (4.5×) |
| add | 384 | 3.5 | 15.4 (4.4×) | 12.2 (3.5×) | 15.3 (4.4×) | 18.0 (5.1×) |
| add | 512 | 4.0 | 25.7 (6.4×) | 17.3 (4.3×) | 26.5 (6.6×) | 32.3 (8.1×) |
| add | 1024 | 23.7 | 63.9 (2.7×) | 44.0 (1.9×) | 69.0 (2.9×) | 66.9 (2.8×) |
| add | 2048 | 54.3 | 163 (3.0×) | 106 (2.0×) | 175 (3.2×) | 141 (2.6×) |
| add | 4096 | 176 | 427 (2.4×) | 311 (1.8×) | 427 (2.4×) | 393 (2.2×) |
| mulmod | 64 | 22.9 | 68.6 (3.0×) | 34.7 (1.5×) | 121 (5.3×) | 40.6 (1.8×) |
| mulmod | 128 | 60.2 | 209 (3.5×) | 86.1 (1.4×) | 254 (4.2×) | 163 (2.7×) |
| mulmod | 256 | 165 | 582 (3.5×) | 291 (1.8×) | 697 (4.2×) | 344 (2.1×) |
| mulmod | 384 | 373 | 1,051 (2.8×) | 504 (1.4×) | 1,366 (3.7×) | 680 (1.8×) |
| mulmod | 512 | 568 | 1,653 (2.9×) | 845 (1.5×) | 2,085 (3.7×) | 1,166 (2.1×) |
| mulmod | 1024 | 1,373 | 5,319 (3.9×) | 2,775 (2.0×) | 6,744 (4.9×) | 4,264 (3.1×) |
| mulmod | 2048 | 4,807 | 17,419 (3.6×) | 9,438 (2.0×) | 22,556 (4.7×) | 12,844 (2.7×) |
| mulmod | 4096 | 17,710 | 55,724 (3.1×) | 29,414 (1.7×) | 68,728 (3.9×) | 42,954 (2.4×) |
| add_mod | 64 | 8.5 | 7.9 (0.9×) | 7.6 (0.9×) | 13.3 (1.6×) | 13.1 (1.5×) |
| add_mod | 128 | 11.7 | 23.4 (2.0×) | 13.4 (1.1×) | 38.1 (3.3×) | 17.9 (1.5×) |
| add_mod | 256 | 20.7 | 33.5 (1.6×) | 28.3 (1.4×) | 45.0 (2.2×) | 42.5 (2.1×) |
| add_mod | 384 | 32.3 | 59.2 (1.8×) | 42.3 (1.3×) | 75.3 (2.3×) | 68.7 (2.1×) |
| add_mod | 512 | 31.8 | 69.2 (2.2×) | 52.8 (1.7×) | 99.6 (3.1×) | 88.9 (2.8×) |
| add_mod | 1024 | 87.6 | 184 (2.1×) | 117 (1.3×) | 210 (2.4×) | 160 (1.8×) |
| add_mod | 2048 | 190 | 383 (2.0×) | 229 (1.2×) | 515 (2.7×) | 330 (1.7×) |
| add_mod | 4096 | 374 | 738 (2.0×) | 557 (1.5×) | 721 (1.9×) | 789 (2.1×) |
| sub_mod | 64 | 7.4 | 7.0 (0.9×) | 7.7 (1.0×) | 13.6 (1.8×) | 11.9 (1.6×) |
| sub_mod | 128 | 11.3 | 25.8 (2.3×) | 13.8 (1.2×) | 36.7 (3.2×) | 16.7 (1.5×) |
| sub_mod | 256 | 16.1 | 37.0 (2.3×) | 29.4 (1.8×) | 48.8 (3.0×) | 37.1 (2.3×) |
| sub_mod | 384 | 39.0 | 48.8 (1.3×) | 45.3 (1.2×) | 78.5 (2.0×) | 66.2 (1.7×) |
| sub_mod | 512 | 43.3 | 74.9 (1.7×) | 52.5 (1.2×) | 95.6 (2.2×) | 89.9 (2.1×) |
| sub_mod | 1024 | 79.6 | 165 (2.1×) | 115 (1.4×) | 200 (2.5×) | 173 (2.2×) |
| sub_mod | 2048 | 190 | 377 (2.0×) | 248 (1.3×) | 432 (2.3×) | 370 (1.9×) |
| sub_mod | 4096 | 435 | 756 (1.7×) | 640 (1.5×) | 750 (1.7×) | 777 (1.8×) |
| add, constant time | 64 | 0.9 | 1.1 (1.2×) | 1.1 (1.2×) | 1.0 (1.1×) | 1.0 (1.1×) |
| add, constant time | 128 | 1.2 | 3.0 (2.5×) | 2.5 (2.1×) | 2.8 (2.3×) | 3.2 (2.7×) |
| add, constant time | 256 | 2.2 | 9.7 (4.4×) | 6.7 (3.0×) | 9.0 (4.1×) | 11.3 (5.1×) |
| add, constant time | 384 | 3.5 | 15.4 (4.4×) | 12.1 (3.5×) | 15.3 (4.4×) | 18.0 (5.1×) |
| add, constant time | 512 | 3.2 | 26.3 (8.2×) | 17.4 (5.4×) | 26.6 (8.3×) | 32.3 (10.1×) |
| add, constant time | 1024 | 22.1 | 72.3 (3.3×) | 43.7 (2.0×) | 69.4 (3.1×) | 66.9 (3.0×) |
| add, constant time | 2048 | 54.3 | 160 (2.9×) | 104 (1.9×) | 176 (3.2×) | 141 (2.6×) |
| add, constant time | 4096 | 222 | 419 (1.9×) | 311 (1.4×) | 425 (1.9×) | 393 (1.8×) |
| mulmod, constant time | 64 | 95.0 | 137 (1.4×) | 93.9 (1.0×) | 245 (2.6×) | 145 (1.5×) |
| mulmod, constant time | 128 | 270 | 576 (2.1×) | 349 (1.3×) | 619 (2.3×) | 368 (1.4×) |
| mulmod, constant time | 256 | 376 | 1,186 (3.2×) | 733 (1.9×) | 1,283 (3.4×) | 856 (2.3×) |
| mulmod, constant time | 384 | 954 | 2,002 (2.1×) | 1,211 (1.3×) | 2,294 (2.4×) | 1,511 (1.6×) |
| mulmod, constant time | 512 | 1,226 | 3,045 (2.5×) | 1,878 (1.5×) | 3,487 (2.8×) | 2,272 (1.9×) |
| mulmod, constant time | 1024 | 4,192 | 9,479 (2.3×) | 6,047 (1.4×) | 10,806 (2.6×) | 7,717 (1.8×) |
| mulmod, constant time | 2048 | 14,626 | 32,913 (2.3×) | 19,961 (1.4×) | 36,644 (2.5×) | 25,703 (1.8×) |
| mulmod, constant time | 4096 | 51,232 | 120,143 (2.3×) | 69,469 (1.4×) | 129,756 (2.5×) | 89,738 (1.8×) |
| add_mod, constant time | 64 | 18.9 | 12.3 (0.7×) | 13.0 (0.7×) | 19.5 (1.0×) | 14.5 (0.8×) |
| add_mod, constant time | 128 | 22.7 | 27.3 (1.2×) | 23.4 (1.0×) | 37.9 (1.7×) | 34.1 (1.5×) |
| add_mod, constant time | 256 | 26.2 | 111 (4.2×) | 76.0 (2.9×) | 112 (4.3×) | 90.1 (3.4×) |
| add_mod, constant time | 384 | 46.2 | 155 (3.4×) | 106 (2.3×) | 179 (3.9×) | 149 (3.2×) |
| add_mod, constant time | 512 | 73.1 | 211 (2.9×) | 137 (1.9×) | 285 (3.9×) | 201 (2.7×) |
| add_mod, constant time | 1024 | 155 | 530 (3.4×) | 313 (2.0×) | 829 (5.3×) | 445 (2.9×) |
| add_mod, constant time | 2048 | 396 | 1,279 (3.2×) | 689 (1.7×) | 2,341 (5.9×) | 1,111 (2.8×) |
| add_mod, constant time | 4096 | 849 | 2,004 (2.4×) | 1,727 (2.0×) | 1,990 (2.3×) | 2,059 (2.4×) |
| sub_mod, constant time | 64 | 9.3 | 12.2 (1.3×) | 11.0 (1.2×) | 19.5 (2.1×) | 14.3 (1.5×) |
| sub_mod, constant time | 128 | 19.7 | 24.2 (1.2×) | 18.7 (0.9×) | 33.7 (1.7×) | 28.9 (1.5×) |
| sub_mod, constant time | 256 | 25.6 | 88.5 (3.5×) | 66.8 (2.6×) | 89.7 (3.5×) | 65.7 (2.6×) |
| sub_mod, constant time | 384 | 43.3 | 133 (3.1×) | 97.0 (2.2×) | 161 (3.7×) | 113 (2.6×) |
| sub_mod, constant time | 512 | 57.0 | 189 (3.3×) | 130 (2.3×) | 260 (4.6×) | 174 (3.1×) |
| sub_mod, constant time | 1024 | 124 | 395 (3.2×) | 259 (2.1×) | 603 (4.9×) | 316 (2.5×) |
| sub_mod, constant time | 2048 | 300 | 892 (3.0×) | 494 (1.6×) | 1,538 (5.1×) | 685 (2.3×) |
| sub_mod, constant time | 4096 | 614 | 1,451 (2.4×) | 1,257 (2.0×) | 1,593 (2.6×) | 1,502 (2.4×) |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
