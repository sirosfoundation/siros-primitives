# Benchmarks

`biguint` at commit `adbe112` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `biguint` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 40.1 before, 33.92 after; its SMT sibling, CPU 29, 33% busy |
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
| 128 | **2.6** / **1.4** | **2.6** / **1.4** | **2.6** / **1.4** | **2.6** / 1.5 | 2.7 / **1.4** | – |
| 256 | 5.0 / **2.6** | – | **4.9** / **2.6** | 5.8 / 5.5 | 5.1 / 3.2 | – |
| 384 | 7.8 / **3.9** | – | **7.4** / 4.0 | 9.1 / 10.8 | 7.9 / 5.3 | – |
| 512 | 10.2 / **5.2** | – | **9.8** / **5.2** | 12.3 / 16.7 | 11.2 / 7.7 | – |
| 1024 | **25.2** / 23.6 | – | 28.7 / **22.4** | 29.2 / 28.5 | 26.8 / 28.9 | – |
| 2048 | **55.8** / **52.1** | – | 71.1 / 67.8 | 79.5 / 99.7 | 91.4 / 105 | – |
| 4096 | **154** / **174** | – | 183 / 192 | 202 / 276 | 221 / 263 | – |

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / – | **1.4** / **3.0** | 1.5 / – | 1.5 / – | **1.4** / – | 1.5 / – | – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – | – |
| 256 | 14.3 / – | – | **12.4** / – | 13.0 / – | 13.0 / – | 16.0 / – | – |
| 384 | **27.5** / – | – | 30.1 / – | 30.0 / – | 33.8 / – | 36.8 / – | – |
| 512 | **52.4** / – | – | 59.8 / – | 79.6 / – | 88.3 / – | – | – |
| 1024 | **221** / – | – | – | 433 / – | 392 / – | 439 / – | – |
| 2048 | **1,116** / – | – | 1,608 / – | 1,980 / – | 1,548 / – | 4,287 / – | 3,424 / – |
| 4096 | **3,835** / – | – | 6,449 / – | 7,970 / – | 6,129 / – | 23,154 / – | 11,297 / – |

### div: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **6.9** / – | 7.0 / – | 7.0 / – | 221 / – | **6.9** / – | 7.0 / – | 38.3 / – |
| 128 | 17.1 / – | **16.1** / – | 31.3 / – | 267 / – | 30.0 / – | 30.0 / – | 99.1 / – |
| 256 | **60.5** / – | – | 78.6 / – | 546 / – | 243 / – | 202 / – | 274 / – |
| 384 | **131** / – | – | 194 / – | 704 / – | 359 / – | 310 / – | 343 / – |
| 512 | **183** / – | – | 263 / – | 828 / – | 450 / – | 395 / – | 401 / – |
| 1024 | **500** / – | – | 586 / – | 1,619 / – | 1,088 / – | 983 / – | 797 / – |
| 2048 | **1,390** / – | – | 1,530 / – | 4,079 / – | 3,416 / – | 3,273 / – | 2,088 / – |
| 4096 | **4,631** / – | – | 4,834 / – | 12,669 / – | 11,619 / – | 12,334 / – | 6,403 / – |

### mod: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **7.0** / – | **7.0** / – | **7.0** / – | 221 / – | **7.0** / – | **7.0** / – | 7.9 / – |
| 128 | 18.0 / – | **16.9** / – | 30.5 / – | 269 / – | 31.0 / – | 31.0 / – | 103 / – |
| 256 | **58.6** / – | – | 80.5 / – | 527 / – | 254 / – | 199 / – | 271 / – |
| 384 | **130** / – | – | 191 / – | 707 / – | 361 / – | 327 / – | 337 / – |
| 512 | **180** / – | – | 244 / – | 847 / – | 471 / – | 395 / – | 395 / – |
| 1024 | **503** / – | – | 562 / – | 1,655 / – | 1,109 / – | 964 / – | 794 / – |
| 2048 | **1,434** / – | – | 1,525 / – | 4,071 / – | 3,402 / – | 3,288 / – | 2,079 / – |
| 4096 | **4,667** / – | – | 4,857 / – | 12,793 / – | 11,632 / – | 12,333 / – | 6,475 / – |

### mulmod: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | num-bigint |
|---|---:|---:|---:|---:|---:|
| 64 | **11.9** / **23.0** | 12.0 / **23.0** | 20.3 / 34.6 | 227 / 226 | 145 / – |
| 128 | **51.2** / **58.8** | – | 81.7 / 90.4 | 495 / 503 | 367 / – |
| 256 | 192 / 220 | – | **169** / **203** | 989 / 973 | 561 / – |
| 384 | **333** / **352** | – | 374 / 394 | 1,468 / 1,488 | 862 / – |
| 512 | **525** / **531** | – | 550 / 556 | 1,969 / 1,962 | 1,172 / – |
| 1024 | **1,464** / **1,454** | – | 1,593 / 1,583 | 5,225 / 5,217 | 3,229 / – |
| 2048 | **5,216** / **5,192** | – | 7,429 / 7,375 | 16,053 / 16,150 | 10,010 / – |
| 4096 | **15,148** / **15,092** | – | 29,498 / 29,498 | 57,031 / 57,474 | 34,063 / – |

### add_mod: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | 4.0 / **8.9** | **2.9** / – | 12.0 / 13.5 | 11.1 / – | 17.1 / – | 3.0 / – | 99.4 / – |
| 128 | 8.0 / **17.2** | **6.0** / – | 32.7 / 21.6 | 27.4 / – | 16.3 / – | 24.3 / – | 231 / – |
| 256 | **22.6** / **24.0** | – | 43.8 / 60.9 | 70.1 / – | 23.7 / – | 55.6 / – | 232 / – |
| 384 | 35.1 / **35.3** | – | 55.7 / 75.4 | 91.0 / – | **31.8** / – | 88.6 / – | 251 / – |
| 512 | 42.3 / **46.6** | – | 74.4 / 90.7 | 122 / – | **39.9** / – | 124 / – | 274 / – |
| 1024 | 92.7 / **89.0** | – | 153 / 166 | 145 / – | **74.3** / – | 271 / – | 383 / – |
| 2048 | 178 / **164** | – | 348 / 337 | 442 / – | **161** / – | 568 / – | 410 / – |
| 4096 | **334** / **421** | – | 675 / 744 | 903 / – | 372 / – | 1,363 / – | 959 / – |

### sub_mod: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | **3.9** / **7.4** | 12.1 / – | 12.0 / 13.4 | 4.6 / – | 13.1 / – | 11.8 / – | 160 / – |
| 128 | 6.0 / **11.3** | **3.9** / – | 31.7 / 22.3 | 11.4 / – | 13.6 / – | 17.9 / – | 277 / – |
| 256 | **12.9** / **14.7** | – | 48.0 / 64.8 | 48.6 / – | 22.5 / – | 46.2 / – | 316 / – |
| 384 | **19.9** / **26.3** | – | 68.1 / 84.4 | 82.0 / – | 29.2 / – | 80.6 / – | 336 / – |
| 512 | **28.8** / **34.5** | – | 89.5 / 108 | 111 / – | 38.1 / – | 135 / – | 386 / – |
| 1024 | **61.8** / **73.3** | – | 173 / 194 | 130 / – | 67.3 / – | 298 / – | 419 / – |
| 2048 | **142** / **144** | – | 425 / 451 | 290 / – | 158 / – | 634 / – | 547 / – |
| 4096 | **239** / **338** | – | 957 / 1,015 | 594 / – | 355 / – | 1,531 / – | 1,226 / – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** |
| 128 | **2.6** / **1.4** | **2.6** / **1.4** | **2.6** / **1.4** | **2.6** / 1.5 | **2.6** / **1.4** | – |
| 256 | 5.0 / **2.6** | – | **4.8** / **2.6** | 5.8 / 5.5 | 5.1 / 3.2 | – |
| 384 | 7.7 / **3.9** | – | **7.3** / 4.0 | 9.0 / 10.8 | 8.0 / 5.2 | – |
| 512 | 10.2 / 5.4 | – | **9.7** / **5.3** | 12.3 / 16.7 | 11.3 / 8.2 | – |
| 1024 | **25.3** / 23.5 | – | 28.5 / **22.5** | 29.2 / 28.5 | 27.7 / 29.1 | – |
| 2048 | **56.0** / **52.3** | – | 71.2 / 67.7 | 79.5 / 100 | 91.4 / 105 | – |
| 4096 | **155** / **175** | – | 180 / 192 | 200 / 277 | 222 / 265 | – |

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint |
|---|---:|---:|---:|---:|---:|---:|
| 64 | **1.4** / – | **1.4** / **3.0** | 1.5 / – | **1.4** / – | 1.5 / – | 1.5 / – |
| 128 | **3.0** / – | **3.0** / **5.0** | **3.0** / – | **3.0** / – | **3.0** / – | **3.0** / – |
| 256 | 16.0 / – | – | **12.4** / – | 13.0 / – | 13.0 / – | 16.0 / – |
| 384 | 30.1 / – | – | **30.0** / – | 30.1 / – | 33.7 / – | 36.8 / – |
| 512 | **51.3** / – | – | 60.0 / – | 79.6 / – | 88.0 / – | – |
| 1024 | **227** / – | – | – | 433 / – | 389 / – | 443 / – |
| 2048 | **1,150** / – | – | – | 1,980 / – | 1,548 / – | 4,287 / – |
| 4096 | **4,228** / – | – | – | 7,970 / – | 6,129 / – | 23,154 / – |

### div, constant time: a / b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.4** / – | 196 / – |
| 128 | **112** / – | 405 / – |
| 256 | **249** / – | 752 / – |
| 384 | **290** / – | 1,104 / – |
| 512 | **477** / – | 1,452 / – |
| 1024 | **1,136** / – | 3,347 / – |
| 2048 | **3,428** / – | 10,106 / – |
| 4096 | **11,818** / – | 30,880 / – |

### mod, constant time: a mod b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **25.9** / – | 189 / – |
| 128 | **111** / – | 387 / – |
| 256 | **271** / – | 719 / – |
| 384 | **307** / – | 1,048 / – |
| 512 | **491** / – | 1,425 / – |
| 1024 | **1,149** / – | 3,323 / – |
| 2048 | **3,448** / – | 10,109 / – |
| 4096 | **11,869** / – | 30,811 / – |

### mulmod, constant time: a × b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | **51.3** / **93.7** | 224 / – |
| 128 | **212** / **213** | 610 / – |
| 256 | **457** / **456** | 1,229 / – |
| 384 | **812** / **828** | 1,997 / – |
| 512 | **1,181** / **1,185** | 2,985 / – |
| 1024 | **3,857** / **3,841** | 7,698 / – |
| 2048 | **13,699** / **13,647** | 25,192 / – |
| 4096 | **48,745** / **48,971** | 85,939 / – |

### add_mod, constant time: a + b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 52.2 / **59.3** | **11.1** / – |
| 128 | 164 / **161** | **27.4** / – |
| 256 | 224 / **221** | **70.1** / – |
| 384 | 311 / **305** | **91.0** / – |
| 512 | 341 / **346** | **122** / – |
| 1024 | 593 / **551** | **145** / – |
| 2048 | 1,190 / **1,162** | **442** / – |
| 4096 | 2,190 / **2,140** | **903** / – |

### sub_mod, constant time: a − b mod m

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Bits | biguint | crypto-bigint |
|---|---:|---:|
| 64 | 49.9 / **64.6** | **4.6** / – |
| 128 | 166 / **164** | **11.4** / – |
| 256 | 225 / **222** | **48.6** / – |
| 384 | 278 / **277** | **82.0** / – |
| 512 | 339 / **344** | **111** / – |
| 1024 | 555 / **532** | **130** / – |
| 2048 | 1,131 / **946** | **290** / – |
| 4096 | 2,042 / **2,054** | **594** / – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # the CPU's counts of the native build
cargo run -p biguint-bench-report -- report   # this file
```
