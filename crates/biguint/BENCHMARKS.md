# Benchmarks

`biguint` at commit `d7a869e` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 5.93 before, 6.16 after; its SMT sibling, CPU 29, 2% busy |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n.
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
| 128 | **2.5** / **1.4** | **2.5** / **1.4** | **2.5** / 1.5 | 2.6 / **1.4** | 2.6 / **1.4** | – |
| 256 | **4.8** / **2.6** | – | **4.8** / **2.6** | 5.6 / 5.5 | 5.0 / 3.2 | – |
| 384 | 7.4 / **3.9** | – | **7.3** / **3.9** | 9.1 / 10.7 | 7.9 / 5.2 | – |
| 512 | 9.6 / 5.0 | – | **9.5** / **4.9** | 12.3 / 16.7 | 11.6 / 7.9 | – |
| 1024 | **24.1** / **22.6** | – | 28.0 / 22.8 | 29.2 / 28.8 | 27.6 / 28.6 | – |
| 2048 | **55.0** / **52.2** | – | 71.2 / 68.3 | 79.3 / 101 | 90.7 / 106 | – |
| 4096 | **154** / 199 | – | 167 / **191** | 202 / 298 | 206 / 283 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | **1.4** / – | 1.5 / – | **1.4** / – | **1.4** / – | – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | **12.4** / – | – | **12.4** / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | **27.9** / – | – | 30.3 / – | 29.8 / – | 33.6 / – | 36.6 / – | – |
| 512 | **52.1** / – | – | 59.4 / – | 81.3 / – | 93.2 / – | – | – |
| 1024 | **220** / – | – | – | 432 / – | 399 / – | 438 / – | – |
| 2048 | **1,112** / – | – | 1,601 / – | 1,961 / – | 1,542 / – | 4,570 / – | 3,463 / – |
| 4096 | **3,850** / – | – | 6,451 / – | 7,741 / – | 6,111 / – | 25,012 / – | 11,243 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 220 / – | **7.0** / – | **7.0** / – | 37.2 / – |
| 128 | 16.9 / – | **16.2** / – | 31.6 / – | 267 / – | 30.0 / – | 31.0 / – | 99.0 / – |
| 256 | **56.7** / – | – | 80.2 / – | 540 / – | 242 / – | 200 / – | 286 / – |
| 384 | **119** / – | – | 194 / – | 711 / – | 357 / – | 304 / – | 354 / – |
| 512 | **161** / – | – | 267 / – | 849 / – | 456 / – | 400 / – | 413 / – |
| 1024 | **464** / – | – | 591 / – | 1,609 / – | 1,113 / – | 993 / – | 824 / – |
| 2048 | **1,337** / – | – | 1,542 / – | 4,085 / – | 3,436 / – | 3,279 / – | 2,065 / – |
| 4096 | **4,647** / – | – | 4,824 / – | 12,773 / – | 12,354 / – | 12,346 / – | 6,382 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 220 / – | **7.0** / – | **7.0** / – | 8.0 / – |
| 128 | 18.5 / – | **17.1** / – | 30.5 / – | 266 / – | 31.0 / – | 31.0 / – | 101 / – |
| 256 | **53.8** / – | – | 80.1 / – | 536 / – | 254 / – | 201 / – | 269 / – |
| 384 | **121** / – | – | 192 / – | 712 / – | 362 / – | 322 / – | 336 / – |
| 512 | **164** / – | – | 245 / – | 843 / – | 479 / – | 397 / – | 395 / – |
| 1024 | **454** / – | – | 565 / – | 1,584 / – | 1,128 / – | 972 / – | 802 / – |
| 2048 | **1,332** / – | – | 1,539 / – | 4,054 / – | 3,459 / – | 3,335 / – | 2,079 / – |
| 4096 | **4,489** / – | – | 4,889 / – | 12,771 / – | 12,293 / – | 12,482 / – | 6,375 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **12.0** / 23.1 | **12.0** / **23.0** | 20.3 / 44.1 | 227 / 225 | 146 / – |
| 128 | **45.7** / **51.3** | – | 81.3 / 90.6 | 495 / 502 | 368 / – |
| 256 | 180 / **180** | – | **165** / 205 | 988 / 989 | 563 / – |
| 384 | **307** / **328** | – | 378 / 391 | 1,458 / 1,498 | 864 / – |
| 512 | **472** / **499** | – | 540 / 557 | 1,951 / 1,972 | 1,187 / – |
| 1024 | **1,387** / **1,373** | – | 1,576 / 1,580 | 5,160 / 5,232 | 3,170 / – |
| 2048 | **5,190** / **5,183** | – | 7,388 / 7,396 | 15,960 / 16,054 | 9,927 / – |
| 4096 | **14,854** / **14,891** | – | 29,568 / 29,525 | 57,480 / 57,457 | 34,087 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 4.0 / **8.6** | **2.9** / – | 12.0 / 13.5 | 11.1 / – | 17.1 / – | **2.9** / – | 99.9 / – |
| 128 | 8.0 / **17.2** | **6.0** / – | 33.7 / 21.8 | 27.4 / – | 16.6 / – | 24.8 / – | 238 / – |
| 256 | **23.0** / **24.6** | – | 44.1 / 61.7 | 70.0 / – | 23.3 / – | 55.8 / – | 232 / – |
| 384 | 33.0 / **40.4** | – | 56.2 / 76.9 | 90.7 / – | **31.6** / – | 88.0 / – | 250 / – |
| 512 | 42.2 / **49.4** | – | 75.4 / 93.2 | 122 / – | **40.3** / – | 125 / – | 253 / – |
| 1024 | 88.7 / **92.5** | – | 148 / 171 | 145 / – | **74.8** / – | 268 / – | 292 / – |
| 2048 | 182 / **160** | – | 307 / 399 | 430 / – | **158** / – | 600 / – | 373 / – |
| 4096 | **288** / **389** | – | 683 / 741 | 912 / – | 372 / – | 1,336 / – | 630 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **4.0** / **8.7** | 11.7 / – | 12.0 / 13.5 | 4.6 / – | 13.0 / – | 11.8 / – | 161 / – |
| 128 | 6.0 / **11.4** | **3.9** / – | 31.9 / 22.2 | 11.4 / – | 13.7 / – | 17.9 / – | 284 / – |
| 256 | **13.0** / **16.5** | – | 47.1 / 60.8 | 49.0 / – | 22.6 / – | 47.4 / – | 329 / – |
| 384 | **18.0** / **27.6** | – | 67.7 / 84.6 | 82.0 / – | 29.3 / – | 80.8 / – | 347 / – |
| 512 | **26.9** / **36.9** | – | 91.2 / 106 | 111 / – | 39.2 / – | 128 / – | 457 / – |
| 1024 | 72.8 / **77.3** | – | 175 / 202 | 131 / – | **68.0** / – | 288 / – | 380 / – |
| 2048 | **159** / **146** | – | 449 / 449 | 291 / – | 165 / – | 627 / – | 498 / – |
| 4096 | **238** / **376** | – | 1,068 / 1,057 | 595 / – | 377 / – | 1,506 / – | 854 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | 2.6 / **1.4** | **2.5** / 1.5 | **2.5** / **1.4** | 2.6 / **1.4** | 2.6 / 1.5 | – |
| 256 | **4.8** / **2.6** | – | **4.8** / 2.7 | 5.6 / 5.5 | 5.0 / 3.2 | – |
| 384 | **7.3** / **3.9** | – | **7.3** / 4.0 | 9.1 / 10.7 | 7.9 / 5.2 | – |
| 512 | 9.6 / **5.0** | – | **9.5** / 5.5 | 12.5 / 16.7 | 11.6 / 7.7 | – |
| 1024 | **24.3** / **22.6** | – | 28.1 / **22.6** | 29.1 / 28.5 | 28.1 / 28.8 | – |
| 2048 | **55.8** / **52.1** | – | 71.5 / 68.3 | 79.4 / 102 | 91.1 / 105 | – |
| 4096 | **154** / 199 | – | 167 / **192** | 202 / 300 | 205 / 286 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 1.5 / – | **1.4** / **3.0** | **1.4** / – | **1.4** / – | **1.4** / – | 1.5 / – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – |
| 256 | 12.4 / – | – | **12.3** / – | 13.0 / – | 13.0 / – | 16.0 / – |
| 384 | 30.0 / – | – | 30.1 / – | **29.8** / – | 33.6 / – | 36.5 / – |
| 512 | **50.9** / – | – | 59.6 / – | 81.0 / – | 92.8 / – | – |
| 1024 | **225** / – | – | – | 432 / – | 397 / – | 437 / – |
| 2048 | **1,156** / – | – | – | 1,961 / – | 1,542 / – | 4,570 / – |
| 4096 | **4,211** / – | – | – | 7,741 / – | 6,111 / – | 25,012 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.5** / – | 189 / – |
| 128 | **112** / – | 406 / – |
| 256 | **250** / – | 746 / – |
| 384 | **290** / – | 1,102 / – |
| 512 | **477** / – | 1,435 / – |
| 1024 | **1,146** / – | 3,320 / – |
| 2048 | **3,433** / – | 10,077 / – |
| 4096 | **11,815** / – | 30,815 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.9** / – | 183 / – |
| 128 | **111** / – | 388 / – |
| 256 | **365** / – | 720 / – |
| 384 | **313** / – | 1,059 / – |
| 512 | **495** / – | 1,427 / – |
| 1024 | **1,161** / – | 3,328 / – |
| 2048 | **3,453** / – | 10,089 / – |
| 4096 | **11,871** / – | 30,697 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **52.1** / **93.7** | 221 / – |
| 128 | **211** / **213** | 606 / – |
| 256 | **451** / **461** | 1,220 / – |
| 384 | **817** / **827** | 1,960 / – |
| 512 | **1,178** / **1,184** | 2,964 / – |
| 1024 | **3,846** / **3,847** | 7,666 / – |
| 2048 | **13,709** / **13,651** | 25,090 / – |
| 4096 | **48,737** / **48,933** | 85,836 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 52.2 / **59.3** | **11.1** / – |
| 128 | 164 / **161** | **27.4** / – |
| 256 | 224 / **222** | **70.0** / – |
| 384 | 311 / **304** | **90.7** / – |
| 512 | 340 / **346** | **122** / – |
| 1024 | 573 / **543** | **145** / – |
| 2048 | 994 / **1,216** | **430** / – |
| 4096 | 2,116 / **2,142** | **912** / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 49.9 / **64.5** | **4.6** / – |
| 128 | 166 / **164** | **11.4** / – |
| 256 | 225 / **223** | **49.0** / – |
| 384 | 278 / **279** | **82.0** / – |
| 512 | 337 / **343** | **111** / – |
| 1024 | 570 / **529** | **131** / – |
| 2048 | 1,170 / **975** | **291** / – |
| 4096 | 2,057 / **2,169** | **595** / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # this file
```
