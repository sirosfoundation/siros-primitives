# Benchmarks

`biguint` at commit `0126260` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 23.62 before, 22.79 after; its SMT sibling, CPU 29, 25% busy |
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
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / **1.2** | **2.4** / 1.3 | 2.5 / 1.4 | 2.5 / 1.4 | 2.6 / 1.4 | – |
| 256 | 4.8 / **2.3** | – | **4.6** / 2.5 | 5.6 / 5.5 | 5.0 / 3.1 | – |
| 384 | **7.0** / 4.0 | – | 7.2 / **3.8** | 8.9 / 10.8 | 7.9 / 5.0 | – |
| 512 | **9.5** / **4.4** | – | **9.5** / 4.7 | 12.2 / 16.7 | 11.4 / 7.5 | – |
| 1024 | **24.0** / **22.1** | – | 27.7 / 22.2 | 28.2 / 28.6 | 27.2 / 28.8 | – |
| 2048 | **56.0** / **51.9** | – | 83.3 / 67.3 | 80.7 / 100 | 96.1 / 105 | – |
| 4096 | **162** / **159** | – | 169 / 222 | 201 / 278 | 210 / 287 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | 1.5 / **3.0** | **1.4** / – | **1.4** / – | **1.4** / – | 1.5 / – | – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | **12.1** / – | – | 12.4 / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | **27.4** / – | – | 30.0 / – | 29.8 / – | 33.5 / – | 36.5 / – | – |
| 512 | **52.1** / – | – | 60.0 / – | 81.9 / – | 88.6 / – | – | – |
| 1024 | **221** / – | – | – | 432 / – | 388 / – | 438 / – | – |
| 2048 | **1,132** / – | – | 1,601 / – | 1,960 / – | 1,545 / – | 4,593 / – | 3,458 / – |
| 4096 | **3,849** / – | – | 6,469 / – | 7,818 / – | 6,122 / – | 25,864 / – | 11,205 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | **6.9** / – | **6.9** / – | 222 / – | **6.9** / – | 7.0 / – | 37.2 / – |
| 128 | **16.2** / – | **16.2** / – | 31.1 / – | 266 / – | 29.9 / – | 31.0 / – | 99.3 / – |
| 256 | **46.1** / – | – | 79.6 / – | 545 / – | 242 / – | 202 / – | 270 / – |
| 384 | **132** / – | – | 196 / – | 712 / – | 356 / – | 303 / – | 338 / – |
| 512 | **170** / – | – | 266 / – | 849 / – | 458 / – | 400 / – | 397 / – |
| 1024 | **484** / – | – | 588 / – | 1,633 / – | 1,104 / – | 984 / – | 800 / – |
| 2048 | **1,390** / – | – | 1,540 / – | 4,087 / – | 3,464 / – | 3,311 / – | 2,094 / – |
| 4096 | **4,620** / – | – | 4,815 / – | 12,780 / – | 12,111 / – | 12,503 / – | 6,417 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | 7.0 / – | **6.9** / – | 220 / – | **6.9** / – | 7.0 / – | 7.9 / – |
| 128 | **16.9** / – | **16.9** / – | 30.5 / – | 267 / – | 30.9 / – | 31.0 / – | 102 / – |
| 256 | **41.4** / – | – | 79.9 / – | 536 / – | 254 / – | 202 / – | 271 / – |
| 384 | **145** / – | – | 191 / – | 715 / – | 363 / – | 323 / – | 336 / – |
| 512 | **206** / – | – | 244 / – | 847 / – | 481 / – | 395 / – | 397 / – |
| 1024 | **474** / – | – | 565 / – | 1,609 / – | 1,130 / – | 985 / – | 799 / – |
| 2048 | **1,405** / – | – | 1,541 / – | 4,080 / – | 3,475 / – | 3,350 / – | 2,082 / – |
| 4096 | **4,571** / – | – | 4,885 / – | 12,772 / – | 12,365 / – | 12,596 / – | 6,396 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **11.9** / **23.0** | 12.0 / **23.0** | 20.2 / 49.2 | 227 / 225 | 145 / – |
| 128 | **47.3** / **50.8** | – | 81.6 / 90.3 | 495 / 506 | 365 / – |
| 256 | 178 / **184** | – | **168** / 202 | 983 / 1,000 | 557 / – |
| 384 | **317** / **348** | – | 375 / 390 | 1,456 / 1,491 | 854 / – |
| 512 | **490** / **504** | – | 539 / 557 | 1,959 / 1,976 | 1,257 / – |
| 1024 | **1,382** / **1,363** | – | 1,578 / 1,584 | 5,205 / 5,218 | 3,183 / – |
| 2048 | **5,209** / **5,200** | – | 7,391 / 7,394 | 15,703 / 15,999 | 9,981 / – |
| 4096 | **14,895** / **14,959** | – | 29,516 / 29,471 | 57,422 / 57,500 | 33,988 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.9 / **8.3** | **2.8** / – | 11.9 / 13.4 | 11.1 / – | 17.1 / – | 2.9 / – | 98.3 / – |
| 128 | **4.7** / **12.7** | 5.9 / – | 33.1 / 22.0 | 27.3 / – | 16.4 / – | 24.9 / – | 223 / – |
| 256 | **9.6** / **14.2** | – | 41.0 / 61.3 | 69.7 / – | 22.9 / – | 55.4 / – | 233 / – |
| 384 | **17.7** / **27.0** | – | 56.0 / 77.3 | 90.8 / – | 31.3 / – | 88.8 / – | 246 / – |
| 512 | **27.2** / **34.7** | – | 72.3 / 93.7 | 122 / – | 39.5 / – | 122 / – | 301 / – |
| 1024 | 72.3 / **80.1** | – | 142 / 173 | 143 / – | **70.6** / – | 268 / – | 278 / – |
| 2048 | 163 / **168** | – | 392 / 310 | 429 / – | **156** / – | 567 / – | 400 / – |
| 4096 | **268** / **311** | – | 696 / 759 | 900 / – | 379 / – | 1,303 / – | 1,068 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.4** / **7.6** | 11.6 / – | 11.9 / 13.4 | 4.6 / – | 13.1 / – | 11.8 / – | 162 / – |
| 128 | 4.3 / **11.5** | **3.8** / – | 32.8 / 22.5 | 11.5 / – | 13.5 / – | 18.3 / – | 274 / – |
| 256 | **8.6** / **14.0** | – | 46.5 / 59.3 | 49.3 / – | 22.7 / – | 47.2 / – | 314 / – |
| 384 | **16.4** / **26.6** | – | 68.0 / 85.9 | 81.7 / – | 28.9 / – | 80.2 / – | 322 / – |
| 512 | **25.0** / **37.7** | – | 88.6 / 106 | 111 / – | 40.1 / – | 128 / – | 405 / – |
| 1024 | **66.2** / **72.9** | – | 170 / 195 | 130 / – | 68.7 / – | 313 / – | 392 / – |
| 2048 | **153** / **136** | – | 441 / 466 | 290 / – | 159 / – | 641 / – | 564 / – |
| 4096 | **266** / **434** | – | 944 / 1,040 | 591 / – | 396 / – | 1,531 / – | 1,501 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | **2.4** / **1.3** | **2.4** / **1.3** | 2.5 / 1.4 | 2.5 / **1.3** | 2.6 / 1.4 | – |
| 256 | 4.6 / **2.3** | – | **4.5** / 2.4 | 5.6 / 5.5 | 5.1 / 3.1 | – |
| 384 | **6.9** / **3.5** | – | 7.2 / 3.8 | 8.9 / 10.5 | 8.0 / 5.0 | – |
| 512 | **9.1** / **4.3** | – | 9.6 / 5.2 | 12.3 / 16.7 | 11.3 / 7.7 | – |
| 1024 | **24.0** / **22.1** | – | 27.8 / 22.2 | 28.2 / 28.5 | 27.8 / 28.9 | – |
| 2048 | **55.4** / **51.9** | – | 83.3 / 68.1 | 80.5 / 99.8 | 96.3 / 105 | – |
| 4096 | **164** / **160** | – | 167 / 222 | 201 / 278 | 209 / 286 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | **1.4** / – | **1.4** / – | **1.4** / – | 1.5 / – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – |
| 256 | 12.3 / – | – | **12.2** / – | 13.0 / – | 13.0 / – | 15.9 / – |
| 384 | 30.0 / – | – | 30.1 / – | **29.8** / – | 33.6 / – | 36.6 / – |
| 512 | **50.9** / – | – | 60.1 / – | 81.6 / – | 88.5 / – | – |
| 1024 | **225** / – | – | – | 431 / – | 389 / – | 437 / – |
| 2048 | **1,188** / – | – | – | 1,960 / – | 1,545 / – | 4,593 / – |
| 4096 | **4,211** / – | – | – | 7,818 / – | 6,122 / – | 25,864 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.1** / – | 192 / – |
| 128 | **98.8** / – | 406 / – |
| 256 | **198** / – | 750 / – |
| 384 | **291** / – | 1,110 / – |
| 512 | **466** / – | 1,430 / – |
| 1024 | **1,139** / – | 3,323 / – |
| 2048 | **3,425** / – | 10,126 / – |
| 4096 | **11,829** / – | 30,812 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.6** / – | 189 / – |
| 128 | **105** / – | 391 / – |
| 256 | **216** / – | 719 / – |
| 384 | **311** / – | 1,058 / – |
| 512 | **477** / – | 1,440 / – |
| 1024 | **1,150** / – | 3,356 / – |
| 2048 | **3,453** / – | 10,063 / – |
| 4096 | **11,876** / – | 30,762 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **51.7** / **93.9** | 228 / – |
| 128 | **200** / **206** | 607 / – |
| 256 | **453** / **460** | 1,216 / – |
| 384 | **817** / **824** | 1,957 / – |
| 512 | **1,188** / **1,192** | 2,970 / – |
| 1024 | **3,865** / **3,864** | 7,638 / – |
| 2048 | **13,618** / **13,499** | 25,087 / – |
| 4096 | **48,875** / **48,757** | 86,080 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **7.5** / **18.7** | 11.1 / – |
| 128 | **12.1** / **19.7** | 27.3 / – |
| 256 | **22.0** / **27.6** | 69.7 / – |
| 384 | **42.5** / **38.2** | 90.8 / – |
| 512 | **59.8** / **52.3** | 122 / – |
| 1024 | **143** / **149** | **143** / – |
| 2048 | 642 / **347** | **429** / – |
| 4096 | **624** / **668** | 900 / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 6.3 / **8.9** | **4.6** / – |
| 128 | **7.4** / **11.8** | 11.5 / – |
| 256 | **13.7** / **16.8** | 49.3 / – |
| 384 | **26.8** / **27.2** | 81.7 / – |
| 512 | **33.8** / **33.6** | 111 / – |
| 1024 | **95.2** / **95.3** | 130 / – |
| 2048 | 386 / **304** | **290** / – |
| 4096 | **455** / **461** | 591 / – |

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

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** |
| 128 | **4.0** / 3.0 | **4.0** / **2.8** | **4.0** / 2.9 | **4.0** / **2.8** | **4.0** / **2.8** | – |
| 256 | **10.2** / 9.8 | – | 10.3 / 9.4 | **10.2** / 9.8 | 10.3 / **8.1** | – |
| 384 | **16.3** / **15.4** | – | 16.4 / **15.4** | 18.0 / 17.1 | 17.1 / 16.6 | – |
| 512 | 27.4 / 25.7 | – | 27.1 / 26.0 | 33.1 / 33.2 | **24.7** / **22.3** | – |
| 1024 | 64.1 / 64.0 | – | 63.8 / 63.7 | 85.5 / 84.5 | **61.6** / **62.1** | – |
| 2048 | **157** / **160** | – | 202 / 208 | 191 / 232 | 206 / 244 | – |
| 4096 | 430 / 424 | – | 406 / **404** | **389** / 457 | 411 / 468 | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.1** / – | 2.2 / **3.0** | **2.1** / – | **2.1** / – | **2.1** / – | **2.1** / – | – |
| 128 | **10.0** / – | 11.6 / **11.8** | 11.0 / – | 24.4 / – | 11.1 / – | 11.0 / – | – |
| 256 | **47.7** / – | – | 66.6 / – | 95.9 / – | 119 / – | 68.8 / – | – |
| 384 | 181 / – | – | 168 / – | 206 / – | 242 / – | **154** / – | – |
| 512 | **295** / – | – | 320 / – | 376 / – | 425 / – | – | – |
| 1024 | **1,340** / – | – | – | 2,083 / – | 1,544 / – | 2,055 / – | – |
| 2048 | **5,613** / – | – | 6,360 / – | 8,050 / – | 6,288 / – | 9,871 / – | 11,774 / – |
| 4096 | **17,687** / – | – | 38,989 / – | 29,248 / – | 23,616 / – | 43,185 / – | 39,585 / – |

### div: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 507 / – | **7.0** / – | **7.0** / – | 125 / – |
| 128 | **60.1** / – | 171 / – | 93.8 / – | 568 / – | 208 / – | 211 / – | 325 / – |
| 256 | **216** / – | – | 320 / – | 1,288 / – | 804 / – | 855 / – | 454 / – |
| 384 | **351** / – | – | 604 / – | 1,701 / – | 1,305 / – | 1,282 / – | 612 / – |
| 512 | **509** / – | – | 828 / – | 2,088 / – | 1,846 / – | 1,811 / – | 807 / – |
| 1024 | **1,196** / – | – | 2,121 / – | 4,131 / – | 4,575 / – | 4,565 / – | 1,940 / – |
| 2048 | **3,242** / – | – | 6,508 / – | 11,143 / – | 15,391 / – | 14,283 / – | 5,774 / – |
| 4096 | **12,603** / – | – | 23,239 / – | 36,609 / – | 48,955 / – | 52,015 / – | 20,046 / – |

### mod: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 505 / – | **7.0** / – | **7.0** / – | 33.5 / – |
| 128 | **52.8** / – | 174 / – | 87.3 / – | 568 / – | 213 / – | 214 / – | 331 / – |
| 256 | **217** / – | – | 321 / – | 1,286 / – | 801 / – | 852 / – | 459 / – |
| 384 | **351** / – | – | 581 / – | 1,704 / – | 1,298 / – | 1,284 / – | 619 / – |
| 512 | **486** / – | – | 798 / – | 2,087 / – | 1,836 / – | 1,810 / – | 812 / – |
| 1024 | **1,217** / – | – | 2,102 / – | 4,129 / – | 4,580 / – | 4,541 / – | 1,940 / – |
| 2048 | **3,347** / – | – | 6,496 / – | 11,152 / – | 15,406 / – | 14,271 / – | 5,774 / – |
| 4096 | **12,491** / – | – | 23,093 / – | 36,555 / – | 49,848 / – | 51,799 / – | 20,253 / – |

### mulmod: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **61.0** / **68.8** | 138 / 132 | 84.7 / 85.0 | 546 / 544 | 439 / – |
| 128 | **197** / **206** | – | 325 / 339 | 1,271 / 1,261 | 600 / – |
| 256 | **579** / **586** | – | 818 / 809 | 2,267 / 2,287 | 1,115 / – |
| 384 | **1,041** / **1,038** | – | 1,614 / 1,600 | 3,582 / 3,487 | 1,853 / – |
| 512 | **1,652** / **1,647** | – | 2,867 / 2,873 | 5,029 / 5,074 | 2,815 / – |
| 1024 | **5,332** / **5,319** | – | 11,122 / 11,113 | 15,264 / 15,295 | 8,902 / – |
| 2048 | **17,381** / **17,414** | – | 34,947 / 34,951 | 49,752 / 49,808 | 32,084 / – |
| 4096 | **55,642** / **55,558** | – | 162,484 / 160,647 | 175,006 / 174,907 | 112,933 / – |

### add_mod: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.0 / **7.3** | **3.3** / – | 43.5 / 39.5 | 12.9 / – | 7.8 / – | **3.3** / – | 298 / – |
| 128 | 15.0 / **33.7** | **9.5** / – | 23.3 / 40.1 | 23.7 / – | 11.0 / – | 9.6 / – | 304 / – |
| 256 | 27.4 / **36.3** | – | 43.0 / 74.0 | 47.4 / – | 22.6 / – | **22.1** / – | 337 / – |
| 384 | 62.2 / **79.1** | – | 62.0 / 100 | 96.7 / – | 45.5 / – | **34.5** / – | 372 / – |
| 512 | 73.8 / **87.9** | – | 90.7 / 129 | 108 / – | 64.4 / – | **46.9** / – | 401 / – |
| 1024 | 148 / **170** | – | 198 / 272 | 377 / – | 118 / – | **106** / – | 554 / – |
| 2048 | 291 / **329** | – | 790 / 856 | 573 / – | 362 / – | **250** / – | 888 / – |
| 4096 | 634 / **720** | – | 1,553 / 1,701 | 1,159 / – | 666 / – | **618** / – | 2,134 / – |

### sub_mod: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 5.0 / **7.9** | **3.0** / – | 53.1 / 46.1 | 5.0 / – | 4.8 / – | 4.3 / – | 395 / – |
| 128 | 14.0 / **34.5** | **6.8** / – | 31.1 / 45.3 | 10.3 / – | 8.8 / – | 9.2 / – | 414 / – |
| 256 | 32.8 / **46.5** | – | 59.8 / 95.8 | 28.1 / – | **18.6** / – | 18.7 / – | 472 / – |
| 384 | 53.1 / **70.2** | – | 84.6 / 129 | 48.9 / – | 40.1 / – | **31.3** / – | 535 / – |
| 512 | 80.4 / **116** | – | 120 / 168 | 71.2 / – | **46.9** / – | 61.6 / – | 592 / – |
| 1024 | 150 / **192** | – | 344 / 430 | 177 / – | 141 / – | **134** / – | 814 / – |
| 2048 | 382 / **393** | – | 1,109 / 1,214 | 519 / – | 372 / – | **301** / – | 1,347 / – |
| 4096 | **656** / **744** | – | 2,110 / 2,312 | 941 / – | 672 / – | 707 / – | 2,839 / – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** |
| 128 | **4.0** / 3.0 | **4.0** / **2.8** | **4.0** / 2.9 | **4.0** / **2.8** | **4.0** / **2.8** | – |
| 256 | **10.2** / 9.8 | – | 10.3 / 9.4 | 10.3 / 9.8 | 10.3 / **8.1** | – |
| 384 | **16.3** / **15.4** | – | **16.3** / **15.4** | 18.0 / 17.1 | 17.1 / 16.6 | – |
| 512 | 27.5 / 25.7 | – | 27.1 / 26.1 | 33.2 / 33.2 | **24.7** / **22.3** | – |
| 1024 | 64.0 / 64.0 | – | 63.7 / 63.6 | 85.7 / 84.1 | **61.6** / **62.1** | – |
| 2048 | **157** / **160** | – | 202 / 208 | 191 / 231 | 206 / 244 | – |
| 4096 | 429 / 423 | – | 405 / **405** | **395** / 456 | 410 / 467 | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.1** / – | **2.1** / **3.0** | **2.1** / – | 2.2 / – | 2.2 / – | **2.1** / – |
| 128 | **10.0** / – | 11.6 / **11.8** | 11.0 / – | 24.4 / – | 11.1 / – | 11.0 / – |
| 256 | **47.8** / – | – | 66.7 / – | 95.8 / – | 120 / – | 69.2 / – |
| 384 | 164 / – | – | 165 / – | 206 / – | 241 / – | **160** / – |
| 512 | **298** / – | – | 320 / – | 377 / – | 425 / – | – |
| 1024 | **1,344** / – | – | – | 2,091 / – | 1,544 / – | 2,052 / – |
| 2048 | **5,787** / – | – | – | 8,050 / – | 6,288 / – | 9,871 / – |
| 4096 | **18,263** / – | – | – | 29,248 / – | 23,616 / – | 43,185 / – |

### div, constant time: a / b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **83.6** / – | 389 / – |
| 128 | **220** / – | 901 / – |
| 256 | **509** / – | 1,696 / – |
| 384 | **632** / – | 2,516 / – |
| 512 | **990** / – | 3,541 / – |
| 1024 | **2,424** / – | 8,493 / – |
| 2048 | **7,350** / – | 22,487 / – |
| 4096 | **25,331** / – | 67,208 / – |

### mod, constant time: a mod b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **85.1** / – | 380 / – |
| 128 | **216** / – | 885 / – |
| 256 | **513** / – | 1,629 / – |
| 384 | **734** / – | 2,531 / – |
| 512 | **1,060** / – | 3,515 / – |
| 1024 | **2,507** / – | 8,475 / – |
| 2048 | **7,625** / – | 22,423 / – |
| 4096 | **25,923** / – | 67,693 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **174** / **178** | 475 / – |
| 128 | **561** / **568** | 1,285 / – |
| 256 | **1,212** / **1,160** | 2,845 / – |
| 384 | **1,994** / **1,989** | 4,905 / – |
| 512 | **3,064** / **3,039** | 7,624 / – |
| 1024 | **9,559** / **9,494** | 21,365 / – |
| 2048 | **32,879** / **32,786** | 65,260 / – |
| 4096 | **119,268** / **120,120** | 259,181 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 178 / **139** | **12.9** / – |
| 128 | 397 / **392** | **23.7** / – |
| 256 | 551 / **542** | **47.4** / – |
| 384 | 679 / **665** | **96.7** / – |
| 512 | 807 / **792** | **108** / – |
| 1024 | 1,441 / **1,401** | **377** / – |
| 2048 | 2,677 / **2,723** | **573** / – |
| 4096 | 5,234 / **5,282** | **1,159** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 183 / **91.1** | **5.0** / – |
| 128 | 396 / **390** | **10.3** / – |
| 256 | 523 / **518** | **28.1** / – |
| 384 | 652 / **641** | **48.9** / – |
| 512 | 774 / **786** | **71.2** / – |
| 1024 | 1,389 / **1,374** | **177** / – |
| 2048 | 2,660 / **2,714** | **519** / – |
| 4096 | 4,951 / **5,006** | **941** / – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** |
| 128 | 3.8 / 2.5 | **3.7** / 2.1 | 3.8 / 2.5 | 3.9 / 2.7 | 4.0 / **2.0** | – |
| 256 | **8.7** / 6.8 | – | 9.0 / 8.0 | 9.1 / 7.1 | 9.0 / **6.3** | – |
| 384 | 12.3 / **12.2** | – | 13.0 / 13.6 | **12.2** / 12.4 | 16.0 / 14.3 | – |
| 512 | **20.7** / **17.3** | – | 23.1 / 18.9 | 20.8 / 18.2 | 22.1 / 23.0 | – |
| 1024 | 45.3 / **43.6** | – | 49.1 / 44.7 | **45.1** / 44.2 | 60.2 / 59.7 | – |
| 2048 | **109** / **106** | – | 177 / 178 | 148 / 185 | 210 / 228 | – |
| 4096 | 382 / 364 | – | 304 / **292** | **283** / 332 | 404 / 422 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / – | 2.2 / **3.0** | **2.1** / – | 2.2 / – | 2.2 / – | 2.2 / – | – |
| 128 | 6.5 / – | **4.1** / **5.2** | 6.5 / – | 5.5 / – | 4.3 / – | 6.9 / – | – |
| 256 | **16.8** / – | – | **16.8** / – | 18.4 / – | 22.3 / – | 22.3 / – | – |
| 384 | 52.3 / – | – | 52.2 / – | **39.6** / – | 75.4 / – | 64.1 / – | – |
| 512 | 70.7 / – | – | **69.2** / – | 84.0 / – | 209 / – | – | – |
| 1024 | 691 / – | – | – | 590 / – | 700 / – | **505** / – | – |
| 2048 | 2,500 / – | – | **2,115** / – | 2,753 / – | 2,603 / – | 3,306 / – | 11,735 / – |
| 4096 | 8,430 / – | – | **8,001** / – | 11,621 / – | 10,109 / – | 14,667 / – | 39,769 / – |

### div: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 448 / – | **7.0** / – | **7.0** / – | 125 / – |
| 128 | **28.3** / – | 170 / – | 43.3 / – | 492 / – | 184 / – | 197 / – | 326 / – |
| 256 | **87.0** / – | – | 156 / – | 1,038 / – | 594 / – | 569 / – | 450 / – |
| 384 | **203** / – | – | 349 / – | 1,287 / – | 906 / – | 864 / – | 608 / – |
| 512 | **295** / – | – | 422 / – | 1,542 / – | 1,252 / – | 1,150 / – | 804 / – |
| 1024 | **696** / – | – | 899 / – | 3,065 / – | 2,756 / – | 2,442 / – | 1,929 / – |
| 2048 | **2,005** / – | – | 2,358 / – | 7,827 / – | 7,414 / – | 6,054 / – | 5,768 / – |
| 4096 | **6,829** / – | – | 7,695 / – | 20,032 / – | 22,650 / – | 19,805 / – | 20,059 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 449 / – | **7.0** / – | **7.0** / – | 33.4 / – |
| 128 | **30.0** / – | 171 / – | 40.5 / – | 493 / – | 186 / – | 198 / – | 332 / – |
| 256 | **82.0** / – | – | 157 / – | 1,041 / – | 588 / – | 564 / – | 457 / – |
| 384 | **195** / – | – | 320 / – | 1,278 / – | 912 / – | 865 / – | 615 / – |
| 512 | **279** / – | – | 403 / – | 1,540 / – | 1,252 / – | 1,162 / – | 816 / – |
| 1024 | **720** / – | – | 961 / – | 3,067 / – | 2,765 / – | 2,413 / – | 1,943 / – |
| 2048 | **1,955** / – | – | 2,277 / – | 7,832 / – | 7,427 / – | 6,068 / – | 5,796 / – |
| 4096 | **6,494** / – | – | 7,790 / – | 20,028 / – | 22,485 / – | 19,932 / – | 19,947 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **31.6** / **34.6** | 116 / 113 | 39.9 / 42.5 | 439 / 436 | 416 / – |
| 128 | **81.7** / **86.1** | – | 137 / 138 | 952 / 976 | 606 / – |
| 256 | **281** / **287** | – | 307 / 309 | 1,566 / 1,569 | 1,111 / – |
| 384 | **507** / **503** | – | 538 / 536 | 2,305 / 2,318 | 1,843 / – |
| 512 | **849** / **845** | – | 855 / 857 | 3,150 / 3,160 | 2,822 / – |
| 1024 | 2,764 / 2,778 | – | **2,632** / **2,631** | 8,375 / 8,597 | 8,911 / – |
| 2048 | **9,432** / **9,459** | – | 10,666 / 10,685 | 23,594 / 24,109 | 32,017 / – |
| 4096 | **29,321** / **29,343** | – | 41,180 / 41,173 | 74,538 / 73,953 | 113,225 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.0 / **11.1** | **4.0** / – | 44.0 / 40.4 | 12.2 / – | 7.9 / – | **4.0** / – | 296 / – |
| 128 | 13.0 / **15.5** | 9.0 / – | 21.7 / 30.8 | 21.2 / – | 11.2 / – | **8.9** / – | 301 / – |
| 256 | 26.6 / **31.0** | – | 44.0 / 60.1 | 42.3 / – | 21.8 / – | **21.4** / – | 336 / – |
| 384 | 66.2 / **70.2** | – | 60.3 / 83.3 | 105 / – | 40.3 / – | **38.5** / – | 372 / – |
| 512 | **44.5** / **60.1** | – | 85.6 / 101 | 93.1 / – | 57.8 / – | 44.6 / – | 401 / – |
| 1024 | 128 / **134** | – | 184 / 203 | 345 / – | 118 / – | **104** / – | 548 / – |
| 2048 | **218** / **251** | – | 752 / 778 | 505 / – | 352 / – | 242 / – | 904 / – |
| 4096 | **489** / **586** | – | 1,405 / 1,454 | 993 / – | 647 / – | 585 / – | 1,950 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.0 / **9.3** | **3.0** / – | 50.5 / 45.3 | 5.0 / – | 4.8 / – | 4.3 / – | 409 / – |
| 128 | 12.0 / **13.9** | **5.4** / – | 30.1 / 40.7 | 11.0 / – | 7.2 / – | 10.2 / – | 420 / – |
| 256 | 27.3 / **33.4** | – | 61.4 / 79.3 | 23.5 / – | **16.5** / – | 19.4 / – | 468 / – |
| 384 | 33.1 / **47.9** | – | 82.6 / 107 | 39.1 / – | 39.3 / – | **32.1** / – | 513 / – |
| 512 | 50.1 / **65.6** | – | 114 / 138 | 53.7 / – | **44.4** / – | 62.6 / – | 567 / – |
| 1024 | **104** / **122** | – | 329 / 351 | 142 / – | 135 / – | 132 / – | 781 / – |
| 2048 | **231** / **236** | – | 1,061 / 1,103 | 459 / – | 414 / – | 299 / – | 1,345 / – |
| 4096 | **492** / **590** | – | 1,984 / 2,052 | 773 / – | 706 / – | 679 / – | 2,733 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** |
| 128 | 3.8 / 2.5 | **3.7** / 2.1 | 3.8 / 2.5 | 3.9 / 2.7 | 4.0 / **2.0** | – |
| 256 | **8.7** / 6.8 | – | 9.0 / 8.0 | 9.1 / 7.1 | 9.0 / **6.3** | – |
| 384 | 12.3 / 12.2 | – | 13.0 / 13.6 | **12.2** / **12.0** | 16.0 / 14.3 | – |
| 512 | **20.8** / **17.3** | – | 23.1 / 18.9 | **20.8** / 18.2 | 22.1 / 23.0 | – |
| 1024 | 45.4 / **43.6** | – | 49.1 / 44.6 | **45.2** / 44.2 | 60.2 / 59.5 | – |
| 2048 | **108** / **106** | – | 177 / 178 | 148 / 185 | 210 / 228 | – |
| 4096 | 379 / 364 | – | 304 / **292** | **284** / 332 | 402 / 422 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 6.5 / – | **4.2** / **5.3** | 6.5 / – | 5.5 / – | 4.3 / – | 6.9 / – |
| 256 | **16.8** / – | – | **16.8** / – | 18.4 / – | 22.3 / – | 22.3 / – |
| 384 | 52.3 / – | – | 52.2 / – | **39.5** / – | 75.5 / – | 64.2 / – |
| 512 | 72.3 / – | – | **69.2** / – | 84.0 / – | 209 / – | – |
| 1024 | 693 / – | – | – | 592 / – | 698 / – | **503** / – |
| 2048 | **2,558** / – | – | – | 2,753 / – | 2,603 / – | 3,306 / – |
| 4096 | **8,862** / – | – | – | 11,621 / – | 10,109 / – | 14,667 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.0** / – | 322 / – |
| 128 | **183** / – | 751 / – |
| 256 | **246** / – | 1,362 / – |
| 384 | **524** / – | 2,016 / – |
| 512 | **714** / – | 2,768 / – |
| 1024 | **1,796** / – | 7,043 / – |
| 2048 | **5,079** / – | 16,909 / – |
| 4096 | **16,113** / – | 43,448 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **47.2** / – | 327 / – |
| 128 | **177** / – | 741 / – |
| 256 | **339** / – | 1,341 / – |
| 384 | **529** / – | 1,971 / – |
| 512 | **698** / – | 2,751 / – |
| 1024 | **1,822** / – | 7,021 / – |
| 2048 | **5,045** / – | 16,892 / – |
| 4096 | **16,265** / – | 46,956 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **97.4** / **94.0** | 351 / – |
| 128 | **358** / **328** | 998 / – |
| 256 | **738** / **734** | 1,975 / – |
| 384 | **1,202** / **1,199** | 3,226 / – |
| 512 | **1,829** / **1,813** | 4,663 / – |
| 1024 | **5,951** / **6,015** | 13,268 / – |
| 2048 | **19,574** / **19,863** | 35,265 / – |
| 4096 | **69,939** / **69,404** | 110,710 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 110 / **86.6** | **12.2** / – |
| 128 | 283 / **285** | **21.2** / – |
| 256 | 404 / **395** | **42.3** / – |
| 384 | 507 / **491** | **105** / – |
| 512 | 594 / **580** | **93.1** / – |
| 1024 | 1,066 / **1,074** | **345** / – |
| 2048 | 2,050 / **1,984** | **505** / – |
| 4096 | 3,868 / **3,835** | **993** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 106 / **78.8** | **5.0** / – |
| 128 | 277 / **288** | **11.0** / – |
| 256 | 438 / **366** | **23.5** / – |
| 384 | 461 / **473** | **39.1** / – |
| 512 | 543 / **542** | **53.7** / – |
| 1024 | 973 / **979** | **142** / – |
| 2048 | 1,896 / **1,861** | **459** / – |
| 4096 | 3,652 / **3,615** | **773** / – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.0** | 2.2 / **1.0** | 2.2 / **1.0** | 2.2 / 1.1 | **2.1** / **1.0** | 2.2 / **1.0** |
| 128 | 4.0 / 2.8 | **3.7** / 3.0 | 4.0 / 3.1 | 4.3 / **2.7** | **3.7** / 2.9 | – |
| 256 | **10.0** / **8.9** | – | 10.1 / 9.0 | 10.5 / 9.2 | 10.1 / 9.0 | – |
| 384 | **15.9** / 15.4 | – | 16.0 / **15.2** | 17.4 / 17.3 | 17.5 / 16.8 | – |
| 512 | 25.9 / **27.8** | – | 27.6 / 30.0 | 37.9 / 37.1 | **23.4** / 29.5 | – |
| 1024 | **69.2** / 71.1 | – | 73.7 / **68.9** | 130 / 81.3 | 79.8 / 70.0 | – |
| 2048 | **185** / **177** | – | 213 / 214 | 207 / 251 | 217 / 251 | – |
| 4096 | 429 / **426** | – | 414 / 429 | 410 / 474 | **407** / 465 | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – | – |
| 128 | **12.3** / – | 13.5 / **12.3** | 17.0 / – | 37.1 / – | 14.0 / – | 17.0 / – | – |
| 256 | **66.6** / – | – | 82.3 / – | 122 / – | 128 / – | 81.7 / – | – |
| 384 | 228 / – | – | **219** / – | 299 / – | 365 / – | 237 / – | – |
| 512 | **363** / – | – | 443 / – | 502 / – | 678 / – | – | – |
| 1024 | **1,594** / – | – | – | 2,610 / – | 2,403 / – | 2,848 / – | – |
| 2048 | **6,696** / – | – | 10,819 / – | 9,965 / – | 8,648 / – | 14,178 / – | 11,927 / – |
| 4096 | **21,172** / – | – | 39,287 / – | 35,694 / – | 33,252 / – | 66,508 / – | 39,520 / – |

### div: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 565 / – | **7.0** / – | **7.0** / – | 160 / – |
| 128 | **103** / – | 239 / – | 104 / – | 652 / – | 255 / – | 270 / – | 443 / – |
| 256 | **221** / – | – | 411 / – | 1,502 / – | 931 / – | 918 / – | 617 / – |
| 384 | **402** / – | – | 772 / – | 1,987 / – | 1,449 / – | 1,395 / – | 810 / – |
| 512 | **575** / – | – | 1,029 / – | 2,421 / – | 2,032 / – | 1,942 / – | 1,026 / – |
| 1024 | **1,493** / – | – | 2,707 / – | 4,980 / – | 4,960 / – | 4,718 / – | 2,204 / – |
| 2048 | **4,377** / – | – | 7,254 / – | 12,534 / – | 15,995 / – | 13,763 / – | 6,105 / – |
| 4096 | **15,027** / – | – | 24,768 / – | 38,668 / – | 51,591 / – | 47,991 / – | 20,074 / – |

### mod: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 564 / – | **7.0** / – | **7.0** / – | 31.4 / – |
| 128 | **96.3** / – | 239 / – | 99.4 / – | 651 / – | 257 / – | 269 / – | 448 / – |
| 256 | **245** / – | – | 406 / – | 1,497 / – | 951 / – | 923 / – | 619 / – |
| 384 | **412** / – | – | 716 / – | 1,985 / – | 1,437 / – | 1,378 / – | 807 / – |
| 512 | **569** / – | – | 998 / – | 2,424 / – | 2,025 / – | 1,942 / – | 1,029 / – |
| 1024 | **1,498** / – | – | 2,594 / – | 4,966 / – | 4,880 / – | 4,749 / – | 2,217 / – |
| 2048 | **4,274** / – | – | 7,488 / – | 12,559 / – | 16,043 / – | 13,810 / – | 6,169 / – |
| 4096 | **14,884** / – | – | 25,736 / – | 38,704 / – | 51,401 / – | 48,139 / – | 20,215 / – |

### mulmod: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **104** / **122** | 226 / 206 | 194 / 203 | 660 / 620 | 527 / – |
| 128 | **247** / **255** | – | 428 / 432 | 1,595 / 1,466 | 766 / – |
| 256 | **701** / **687** | – | 1,022 / 899 | 2,946 / 2,530 | 1,330 / – |
| 384 | **1,414** / **1,409** | – | 1,716 / 1,697 | 4,125 / 3,926 | 2,119 / – |
| 512 | **2,158** / **2,152** | – | 3,326 / 3,301 | 6,146 / 5,933 | 3,095 / – |
| 1024 | **7,741** / **6,879** | – | 12,426 / 12,229 | 16,628 / 16,652 | 9,313 / – |
| 2048 | **22,251** / **22,195** | – | 44,588 / 44,565 | 54,289 / 54,298 | 32,100 / – |
| 4096 | **68,366** / **68,359** | – | 166,259 / 166,433 | 183,347 / 183,817 | 113,388 / – |

### add_mod: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 6.2 / **8.0** | **3.7** / – | 81.1 / 78.1 | 11.8 / – | 7.8 / – | **3.7** / – | 356 / – |
| 128 | 19.9 / **46.4** | **9.8** / – | 22.8 / 51.4 | 23.0 / – | 10.0 / – | 10.1 / – | 384 / – |
| 256 | 53.7 / **50.6** | – | 59.6 / 91.1 | 57.1 / – | 25.6 / – | **21.4** / – | 414 / – |
| 384 | 104 / **119** | – | 96.2 / 140 | 86.1 / – | 43.9 / – | **31.2** / – | 448 / – |
| 512 | 120 / **130** | – | 123 / 175 | 142 / – | 60.7 / – | **47.3** / – | 492 / – |
| 1024 | 190 / **220** | – | 261 / 365 | 461 / – | 164 / – | **139** / – | 617 / – |
| 2048 | 328 / **368** | – | 1,020 / 1,121 | 882 / – | 376 / – | **288** / – | 1,019 / – |
| 4096 | **687** / **759** | – | 1,784 / 1,933 | 1,600 / – | 754 / – | 714 / – | 2,387 / – |

### sub_mod: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 8.0 / **10.0** | **3.0** / – | 79.7 / 76.3 | 5.0 / – | 6.4 / – | 5.0 / – | 453 / – |
| 128 | 21.0 / **47.6** | **7.4** / – | 37.0 / 65.5 | 11.0 / – | 8.3 / – | 9.1 / – | 481 / – |
| 256 | 53.6 / **63.7** | – | 91.4 / 131 | 34.4 / – | 21.9 / – | **20.1** / – | 529 / – |
| 384 | 87.6 / **102** | – | 137 / 193 | 55.2 / – | **32.5** / – | 34.8 / – | 584 / – |
| 512 | 107 / **136** | – | 177 / 250 | 91.6 / – | **54.3** / – | 72.3 / – | 636 / – |
| 1024 | 192 / **263** | – | 583 / 667 | 348 / – | 185 / – | **152** / – | 849 / – |
| 2048 | 578 / **677** | – | 1,370 / 1,469 | 563 / – | 415 / – | **373** / – | 1,360 / – |
| 4096 | **666** / **794** | – | 2,397 / 2,534 | 1,474 / – | 673 / – | 738 / – | 2,776 / – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / **1.0** | 2.2 / **1.0** | 2.2 / **1.0** | **2.1** / 1.1 | **2.1** / **1.0** | **2.1** / **1.0** |
| 128 | 4.0 / 2.8 | **3.7** / 3.0 | 4.0 / 3.1 | 4.3 / **2.7** | **3.7** / 2.9 | – |
| 256 | **10.0** / **8.9** | – | 10.1 / 9.0 | 10.5 / 9.2 | 10.1 / 9.0 | – |
| 384 | **15.9** / 15.4 | – | 16.0 / **15.2** | 17.4 / 17.3 | 17.5 / 16.8 | – |
| 512 | 26.0 / 27.5 | – | 27.6 / 29.9 | 37.9 / 36.9 | **23.3** / **25.3** | – |
| 1024 | **69.3** / 71.1 | – | 74.5 / **68.9** | 131 / 81.0 | 79.7 / 70.1 | – |
| 2048 | **185** / **178** | – | 214 / 214 | 207 / 251 | 217 / 251 | – |
| 4096 | 428 / **427** | – | 414 / 429 | 410 / 475 | **407** / 466 | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | **12.3** / – | 13.5 / **12.3** | 17.0 / – | 37.1 / – | 14.0 / – | 17.0 / – |
| 256 | **67.3** / – | – | 82.3 / – | 122 / – | 128 / – | 81.7 / – |
| 384 | **217** / – | – | 220 / – | 301 / – | 367 / – | 237 / – |
| 512 | **357** / – | – | 443 / – | 503 / – | 676 / – | – |
| 1024 | **1,580** / – | – | – | 2,612 / – | 2,406 / – | 2,994 / – |
| 2048 | **6,752** / – | – | – | 9,965 / – | 8,648 / – | 14,178 / – |
| 4096 | **21,877** / – | – | – | 35,694 / – | 33,252 / – | 66,508 / – |

### div, constant time: a / b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **144** / – | 429 / – |
| 128 | **362** / – | 1,080 / – |
| 256 | **623** / – | 1,972 / – |
| 384 | **842** / – | 3,049 / – |
| 512 | **1,148** / – | 4,284 / – |
| 1024 | **2,897** / – | 10,435 / – |
| 2048 | **8,549** / – | 27,542 / – |
| 4096 | **28,855** / – | 85,172 / – |

### mod, constant time: a mod b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **149** / – | 422 / – |
| 128 | **335** / – | 1,068 / – |
| 256 | **587** / – | 1,964 / – |
| 384 | **797** / – | 2,989 / – |
| 512 | **1,092** / – | 4,288 / – |
| 1024 | **2,771** / – | 10,387 / – |
| 2048 | **8,592** / – | 28,028 / – |
| 4096 | **28,828** / – | 85,218 / – |

### mulmod, constant time: a × b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **263** / **252** | 536 / – |
| 128 | **636** / **618** | 1,581 / – |
| 256 | **1,429** / **1,382** | 3,292 / – |
| 384 | **2,676** / **2,433** | 5,797 / – |
| 512 | **3,648** / **3,664** | 8,785 / – |
| 1024 | **10,446** / **10,972** | 25,045 / – |
| 2048 | **36,270** / **35,622** | 76,479 / – |
| 4096 | **132,124** / **130,197** | 256,735 / – |

### add_mod, constant time: a + b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 222 / **198** | **11.8** / – |
| 128 | 425 / **427** | **23.0** / – |
| 256 | 680 / **636** | **57.1** / – |
| 384 | 787 / **798** | **86.1** / – |
| 512 | 923 / **915** | **142** / – |
| 1024 | 1,594 / **1,574** | **461** / – |
| 2048 | 2,976 / **2,902** | **882** / – |
| 4096 | 5,589 / **5,603** | **1,600** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 221 / **198** | **5.0** / – |
| 128 | 432 / **445** | **11.0** / – |
| 256 | 611 / **608** | **34.4** / – |
| 384 | 745 / **735** | **55.2** / – |
| 512 | 906 / **905** | **91.6** / – |
| 1024 | 1,611 / **1,601** | **348** / – |
| 2048 | 3,041 / **2,981** | **563** / – |
| 4096 | 5,321 / **5,260** | **1,474** / – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / 1.1 | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** |
| 128 | 4.4 / 3.2 | **3.4** / 2.6 | 4.3 / 3.3 | 4.3 / 3.2 | 3.5 / **2.5** | – |
| 256 | 11.5 / 11.4 | – | 13.0 / 11.7 | 11.5 / 11.0 | **11.0** / **7.7** | – |
| 384 | **18.0** / **18.1** | – | 20.1 / 19.6 | 18.1 / **18.1** | 20.8 / 19.1 | – |
| 512 | **32.1** / **31.1** | – | 40.5 / 33.3 | 38.1 / 31.8 | 34.4 / 39.9 | – |
| 1024 | 74.0 / 71.4 | – | 72.2 / 64.5 | **68.7** / **63.8** | 71.9 / 70.9 | – |
| 2048 | **147** / **141** | – | 205 / 206 | 179 / 221 | 206 / 259 | – |
| 4096 | 405 / 400 | – | 380 / **391** | **337** / 393 | 417 / 478 | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.2 / – | 2.2 / **3.0** | **2.1** / – | 2.2 / – | 2.2 / – | 2.2 / – | – |
| 128 | 5.6 / – | 5.4 / **5.5** | 5.6 / – | **5.0** / – | 5.5 / – | 5.8 / – | – |
| 256 | **26.0** / – | – | **26.0** / – | 27.0 / – | 34.5 / – | 36.2 / – | – |
| 384 | 70.3 / – | – | 69.4 / – | **65.9** / – | 112 / – | 105 / – | – |
| 512 | **131** / – | – | **131** / – | **131** / – | 242 / – | – | – |
| 1024 | **846** / – | – | – | 918 / – | 1,024 / – | 1,092 / – | – |
| 2048 | 3,697 / – | – | **3,194** / – | 4,033 / – | 3,404 / – | 5,334 / – | 11,868 / – |
| 4096 | 12,443 / – | – | **11,657** / – | 16,323 / – | 12,865 / – | 22,761 / – | 39,591 / – |

### div: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 492 / – | **7.0** / – | **7.0** / – | 158 / – |
| 128 | **36.4** / – | 239 / – | 55.7 / – | 549 / – | 240 / – | 250 / – | 415 / – |
| 256 | **157** / – | – | 177 / – | 1,084 / – | 737 / – | 712 / – | 593 / – |
| 384 | **295** / – | – | 411 / – | 1,359 / – | 1,115 / – | 1,053 / – | 780 / – |
| 512 | **395** / – | – | 567 / – | 1,637 / – | 1,498 / – | 1,536 / – | 1,009 / – |
| 1024 | **877** / – | – | 1,120 / – | 3,318 / – | 3,527 / – | 3,055 / – | 2,179 / – |
| 2048 | **2,459** / – | – | 2,691 / – | 8,499 / – | 10,578 / – | 7,590 / – | 6,107 / – |
| 4096 | **7,306** / – | – | 8,985 / – | 23,324 / – | 33,087 / – | 24,714 / – | 20,159 / – |

### mod: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 491 / – | **7.0** / – | **7.0** / – | 31.3 / – |
| 128 | **33.1** / – | 239 / – | 51.4 / – | 548 / – | 241 / – | 254 / – | 429 / – |
| 256 | **138** / – | – | 175 / – | 1,081 / – | 733 / – | 702 / – | 581 / – |
| 384 | **273** / – | – | 342 / – | 1,357 / – | 1,113 / – | 1,098 / – | 761 / – |
| 512 | **353** / – | – | 468 / – | 1,629 / – | 1,515 / – | 1,492 / – | 1,009 / – |
| 1024 | **948** / – | – | 1,079 / – | 3,312 / – | 3,526 / – | 3,268 / – | 2,180 / – |
| 2048 | **2,223** / – | – | 2,649 / – | 8,518 / – | 10,607 / – | 7,637 / – | 6,119 / – |
| 4096 | **6,911** / – | – | 8,929 / – | 23,332 / – | 32,931 / – | 24,959 / – | 20,140 / – |

### mulmod: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **35.8** / **42.0** | 187 / 183 | 66.6 / 54.8 | 477 / 477 | 521 / – |
| 128 | **158** / **167** | – | 209 / 208 | 1,098 / 1,094 | 752 / – |
| 256 | **352** / **345** | – | 383 / 382 | 1,640 / 1,642 | 1,346 / – |
| 384 | **669** / **661** | – | 696 / 690 | 2,375 / 2,402 | 2,171 / – |
| 512 | 1,176 / **1,105** | – | **1,116** / 1,118 | 2,977 / 2,998 | 3,123 / – |
| 1024 | 4,219 / 4,174 | – | **3,632** / **3,629** | 8,694 / 8,752 | 9,362 / – |
| 2048 | **12,708** / **12,727** | – | 13,701 / 13,702 | 28,243 / 28,176 | 32,277 / – |
| 4096 | **42,732** / **42,867** | – | 53,236 / 53,324 | 99,075 / 99,532 | 115,285 / – |

### add_mod: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 9.0 / **12.3** | **5.0** / – | 82.3 / 76.9 | 13.1 / – | 8.4 / – | **5.0** / – | 356 / – |
| 128 | 20.2 / **26.2** | **9.2** / – | 25.1 / 38.8 | 22.9 / – | 10.4 / – | 12.3 / – | 367 / – |
| 256 | 47.3 / **51.4** | – | 59.0 / 77.8 | 54.8 / – | 33.9 / – | **23.9** / – | 410 / – |
| 384 | 107 / 117 | – | 87.7 / **106** | 82.7 / – | 53.4 / – | **40.4** / – | 440 / – |
| 512 | 88.8 / **102** | – | 122 / 140 | 127 / – | 80.7 / – | **61.3** / – | 473 / – |
| 1024 | 191 / **214** | – | 247 / 294 | 383 / – | 161 / – | **139** / – | 599 / – |
| 2048 | 319 / **356** | – | 911 / 952 | 769 / – | 404 / – | **270** / – | 999 / – |
| 4096 | 729 / **770** | – | 1,638 / 1,686 | 1,404 / – | 700 / – | **681** / – | 2,154 / – |

### sub_mod: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 10.0 / **11.7** | **3.0** / – | 78.9 / 78.3 | 5.0 / – | 5.8 / – | 7.0 / – | 463 / – |
| 128 | 20.1 / **25.5** | **7.3** / – | 38.0 / 52.1 | 11.0 / – | 8.8 / – | 9.4 / – | 493 / – |
| 256 | 55.4 / **53.7** | – | 89.6 / 119 | 33.5 / – | 24.5 / – | **21.7** / – | 534 / – |
| 384 | 66.4 / **84.2** | – | 141 / 174 | 55.1 / – | 47.9 / – | **44.3** / – | 587 / – |
| 512 | 89.8 / **110** | – | 182 / 219 | 105 / – | **63.0** / – | 71.5 / – | 638 / – |
| 1024 | **144** / **164** | – | 601 / 635 | 275 / – | 232 / – | 156 / – | 831 / – |
| 2048 | **295** / **340** | – | 1,254 / 1,341 | 512 / – | 437 / – | 390 / – | 1,338 / – |
| 4096 | **643** / **763** | – | 2,274 / 2,334 | 1,351 / – | 740 / – | 767 / – | 2,870 / – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / 1.1 | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** |
| 128 | 4.4 / 3.2 | **3.4** / 2.6 | 4.3 / 3.3 | 4.3 / 3.2 | 3.5 / **2.5** | – |
| 256 | 11.5 / 11.4 | – | 13.0 / 11.7 | 11.5 / 11.0 | **11.0** / **7.7** | – |
| 384 | **18.0** / **18.1** | – | 20.1 / 19.6 | **18.0** / **18.1** | 20.8 / 19.1 | – |
| 512 | **32.1** / **31.1** | – | 40.4 / 33.3 | 38.1 / 31.8 | 34.4 / 40.1 | – |
| 1024 | 74.0 / 71.3 | – | 72.2 / 64.5 | **68.7** / **63.7** | 71.4 / 71.4 | – |
| 2048 | **146** / **141** | – | 204 / 206 | 179 / 222 | 206 / 258 | – |
| 4096 | 405 / **386** | – | 381 / 391 | **338** / 391 | 419 / 471 | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **2.2** / – | **2.2** / **3.0** | **2.2** / – | **2.2** / – | **2.2** / – | **2.2** / – |
| 128 | 5.6 / – | 5.4 / **5.5** | 5.6 / – | **5.0** / – | 5.4 / – | 5.8 / – |
| 256 | **26.0** / – | – | **26.0** / – | 27.0 / – | 34.2 / – | 36.2 / – |
| 384 | 70.2 / – | – | 70.2 / – | **64.5** / – | 113 / – | 105 / – |
| 512 | 131 / – | – | **129** / – | 131 / – | 244 / – | – |
| 1024 | **862** / – | – | – | 919 / – | 1,019 / – | 1,090 / – |
| 2048 | 3,888 / – | – | – | 4,033 / – | **3,404** / – | 5,334 / – |
| 4096 | **12,853** / – | – | – | 16,323 / – | 12,865 / – | 22,761 / – |

### div, constant time: a / b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **59.5** / – | 338 / – |
| 128 | **229** / – | 903 / – |
| 256 | **478** / – | 1,518 / – |
| 384 | **555** / – | 2,257 / – |
| 512 | **841** / – | 3,098 / – |
| 1024 | **2,255** / – | 7,764 / – |
| 2048 | **6,940** / – | 18,805 / – |
| 4096 | **21,125** / – | 50,685 / – |

### mod, constant time: a mod b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **66.8** / – | 339 / – |
| 128 | **223** / – | 870 / – |
| 256 | **399** / – | 1,498 / – |
| 384 | **564** / – | 2,233 / – |
| 512 | **785** / – | 3,101 / – |
| 1024 | **1,991** / – | 7,488 / – |
| 2048 | **6,107** / – | 18,784 / – |
| 4096 | **20,263** / – | 50,546 / – |

### mulmod, constant time: a × b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **125** / **115** | 381 / – |
| 128 | **375** / **378** | 1,145 / – |
| 256 | **857** / **879** | 2,273 / – |
| 384 | **1,478** / **1,524** | 3,654 / – |
| 512 | **2,354** / **2,291** | 5,506 / – |
| 1024 | **7,670** / **7,625** | 14,056 / – |
| 2048 | **25,158** / **25,208** | 45,423 / – |
| 4096 | **88,748** / **89,535** | 144,747 / – |

### add_mod, constant time: a + b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 137 / **130** | **13.1** / – |
| 128 | 292 / **296** | **22.9** / – |
| 256 | 473 / **462** | **54.8** / – |
| 384 | 625 / **605** | **82.7** / – |
| 512 | 715 / **705** | **127** / – |
| 1024 | 1,252 / **1,297** | **383** / – |
| 2048 | 2,528 / **2,478** | **769** / – |
| 4096 | 4,640 / **4,699** | **1,404** / – |

### sub_mod, constant time: a − b mod m, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 108 / **118** | **5.0** / – |
| 128 | 297 / **300** | **11.0** / – |
| 256 | 465 / **430** | **33.5** / – |
| 384 | 584 / **573** | **55.1** / – |
| 512 | 617 / **718** | **105** / – |
| 1024 | 1,256 / **1,141** | **275** / – |
| 2048 | 2,356 / **2,279** | **512** / – |
| 4096 | 4,230 / **4,216** | **1,351** / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p biguint-bench-report -- report   # this file
```
