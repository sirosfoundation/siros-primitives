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
- Each figure: the median over 31 repetitions of the cycles of 2n operations less those of n, over n.
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
| add_mod | `add_mod` | `overflowing_add, then - m` | `add_mod` | – | `overflowing_add, then - m` | `overflowing_add, then - m` | `(&a + &b) % &m` |
| sub_mod | `sub_mod` | `-, else wrapping_sub then + m` | `a.add_mod(m - b.reduce_mod(m), m)` | – | `-, else overflowing_sub then + m` | `-, else overflowing_sub then + m` | `(&a + &m - &b) % &m` |

- `native`: add, mul, div, mod, add_mod and sub_mod at 64 and 128 bits; mulmod at 64 bits.
- `ruint`: mul at 64, 128, 256, 384, 512, 2048 and 4096 bits.
- `crypto-bigint`: mul at 64, 128, 256, 384, 512 and 1024 bits.
- `bnum`: mul at 64, 128, 256, 384, 512 and 1024 bits.
- `uint`: add at 64 bits; mul at 64, 128, 256, 384 and 1024 bits.
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
| 2048 | **1,361** / – | – | 1,605 / – | – | – | – | 3,606 / – |
| 4096 | **4,170** / – | – | 6,475 / – | – | – | – | 11,324 / – |

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

| Bits | biguint | native | ruint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | 4.0 / **9.0** | **3.0** / – | 12.0 / 13.5 | 17.4 / – | **3.0** / – | 102 / – |
| 128 | 8.0 / **17.3** | **6.0** / – | 34.7 / 22.4 | 17.2 / – | 25.9 / – | 248 / – |
| 256 | **23.5** / **24.3** | – | 44.6 / 61.6 | 24.0 / – | 58.4 / – | 249 / – |
| 384 | 34.5 / **40.6** | – | 58.2 / 81.3 | **32.9** / – | 91.8 / – | 252 / – |
| 512 | 41.9 / **49.3** | – | 75.3 / 97.7 | **40.9** / – | 133 / – | 258 / – |
| 1024 | 93.0 / **93.7** | – | 151 / 175 | **76.0** / – | 281 / – | 298 / – |
| 2048 | 173 / **165** | – | 382 / 401 | **168** / – | 591 / – | 492 / – |
| 4096 | **337** / **410** | – | 759 / 830 | 415 / – | 1,431 / – | 1,074 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **4.0** / **7.7** | 12.2 / – | 12.0 / 13.5 | 13.5 / – | 12.4 / – | 163 / – |
| 128 | 6.0 / **12.3** | **3.9** / – | 33.4 / 23.3 | 14.2 / – | 18.7 / – | 341 / – |
| 256 | **13.1** / **16.7** | – | 48.2 / 64.3 | 22.9 / – | 49.3 / – | 344 / – |
| 384 | **21.0** / **28.0** | – | 70.0 / 88.6 | 30.0 / – | 81.5 / – | 364 / – |
| 512 | **29.4** / **37.6** | – | 91.7 / 111 | 40.6 / – | 133 / – | 311 / – |
| 1024 | **63.0** / **82.0** | – | 179 / 201 | 69.2 / – | 292 / – | 379 / – |
| 2048 | **142** / **157** | – | 484 / 472 | 166 / – | 667 / – | 590 / – |
| 4096 | **288** / **422** | – | 1,018 / 1,089 | 416 / – | 1,596 / – | 868 / – |

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

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # this file
```
