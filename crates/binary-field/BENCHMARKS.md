# Benchmarks

`binary-field` at commit `a0a67b8` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 16.46 before, 25.74 after; its SMT sibling, CPU 29, 57% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 26.75 before, 9.67 after; its SMT sibling, CPU 29, 12% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Every field in the polynomial basis of its modulus: GF(2⁸) of AES, GF(2¹²⁸) of GCM, GF(2²³³) and GF(2⁵⁷¹) of NIST B-233 and B-571.
- Every result matches a bit-serial reference before it is measured.
- The build without carry-less multiplication drops PCLMULQDQ and VPCLMULQDQ.

## Functions

| Operation | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| add | `+` | `+` | `+` | `+` | `add` | `+` | `+` | `addf` | `addf` | `addf` | `+` | `+` |
| mul | `*` | `*` | `*` | `*` | `mul` | `*` | `*` | `mulf` | `mulf` | `mulf` | `*` | `*` |
| square | `square` | `a * a` | `a * a` | `a * a` | `a * a` | `square` | `square` | `a * a` | `a * a` | `a * a` | `a * a` | `a * a` |
| invert | `invert` | `checked_recip` | `inverse` | `a^(2^n − 2) by *` | `a^(2^n − 2) by *` | `invert` | `try_inverse` | `invert` | `invert` | `invert` | `a^(2^n − 2) by *` | `inverse` |
| sqrt | `sqrt` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `try_sqrt` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` |

- `gf256`: over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴).
- `galois_field_2pm`: over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴).
- `g2p`: over GF(2⁸) and GF(2¹⁶).
- `reed-solomon-erasure`: over GF(2⁸) and GF(2¹⁶).
- `hekate-math`: over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶).
- `plonky3`: over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸).
- `runtime-algebra`: over GF(2¹²⁸).
- `compile-algebra`: over GF(2¹²⁸).
- `core-algebra`: over GF(2¹²⁸).
- `polyval`: over GF(2¹²⁸).
- `galois_field_2pm lut`: over GF(2¹⁶).

## Functions, constant time

| Operation | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| add | `ConstantTime +` | `+` | `+` | `+` | `add` | `+` | `+` | `addf` | `addf` | `addf` | `+` | `+` |
| mul | `ConstantTime *` | `*` | – | – | – | `*` | `*` | `mulf` | `mulf` | – | `*` | – |
| square | `ConstantTime::square` | `a * a` | – | – | – | `square` | `square` | `a * a` | `a * a` | – | `a * a` | – |
| invert | `ConstantTime::invert` | `checked_recip` | – | – | – | `invert` | `try_inverse` | – | `invert` | – | `a^(2^n − 2) by *` | – |
| sqrt | `ConstantTime::sqrt` | `a^(2^(n−1)) by *` | – | – | – | `a^(2^(n−1)) by *` | `try_sqrt` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | – | `a^(2^(n−1)) by *` | – |

- `gf256`: add over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴); mul, square, invert and sqrt over GF(2¹⁶) and GF(2⁶⁴).
- `galois_field_2pm`: over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴).
- `g2p`: over GF(2⁸) and GF(2¹⁶).
- `reed-solomon-erasure`: over GF(2⁸) and GF(2¹⁶).
- `hekate-math`: over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶).
- `plonky3`: add over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸); mul, square and sqrt over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸); invert over GF(2⁸) and GF(2⁶⁴).
- `runtime-algebra`: over GF(2¹²⁸).
- `compile-algebra`: over GF(2¹²⁸).
- `core-algebra`: over GF(2¹²⁸).
- `polyval`: over GF(2¹²⁸).
- `galois_field_2pm lut`: over GF(2¹⁶).

## Results with carry-less multiplication

### add: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.4** / 1.1 | 1.5 / 1.0 | 1.5 / 1.0 | – | – | 1.5 / 1.1 | 1.5 / **0.9** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.2** | – | – | – | – | 2.1 / 1.5 | 1.7 / **1.2** | 1.4 / **1.2** | 1.4 / 1.4 | 1.5 / 1.3 | 1.9 / 1.7 | – |
| GF(2²³³) | **2.4** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.0** | – | – | – | – | 3.7 / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.3 / 78.8 | **2.0** / 11.2 | **2.0** / 11.3 | 13.4 / 23.0 | 14.6 / 29.1 | – | – | – | – | – |
| GF(2¹⁶) | **5.0** / **19.8** | 8.0 / 27.0 | 141 / 143 | 9.1 / 41.5 | 7.5 / 22.5 | 53.0 / 52.0 | 14.2 / 30.5 | – | – | – | – | 12.5 / 63.2 |
| GF(2⁶⁴) | **4.9** / **11.1** | 6.0 / 20.0 | 365 / 367 | – | – | 548 / 548 | 5.9 / 19.0 | – | – | – | – | – |
| GF(2¹²⁸) | 12.0 / **12.0** | – | – | – | – | 1,759 / 1,761 | **11.9** / 16.1 | **11.9** / 18.3 | 20.0 / 27.7 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **39.7** / **43.2** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | – | – | – | 5,724 / 5,734 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **167** / **163** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.4 / 68.8 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.3 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 7.0 / 27.8 | 138 / 136 | 6.5 / 11.9 | 6.5 / 17.1 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.4 / 24.0 |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.7 | 364 / 369 | – | – | 125 / 139 | 5.0 / 15.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.3** | – | – | – | – | 288 / 286 | 7.9 / 16.2 | 9.9 / 19.0 | 16.4 / 27.3 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **21.1** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.1** | – | – | – | – | 1,022 / 1,083 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.3** / **40.9** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 307 / – | 29.1 / – | 29.4 / – | 254 / – | 312 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 419 / – | 993 / – | 917 / – | 520 / – | 324 / – | 20.3 / – | – | – | – | – | 10.2 / – |
| GF(2⁶⁴) | **211** / – | 1,223 / – | 8,492 / – | – | – | 1,209 / – | 1,064 / – | – | – | – | – | – |
| GF(2¹²⁸) | **398** / – | – | – | – | – | 3,226 / – | 581 / – | 5,965 / – | 5,074 / – | 147,709 / – | 39,272 / – | – |
| GF(2²³³) | **7,907** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,086** / – | – | – | – | – | 9,711 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,041** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.3 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 239 / – | 2,076 / – | 329 / – | 221 / – | 792 / – | 2.0 / – | – | – | – | – | 474 / – |
| GF(2⁶⁴) | **11.2** / – | 933 / – | 24,089 / – | – | – | 34,520 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | – | – | – | 224,010 / – | 17.7 / – | 2,446 / – | 3,591 / – | 69,343 / – | 19,876 / – | – |
| GF(2²³³) | **47.8** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.5** / – | – | – | – | – | 1,461,519 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **133** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / 1.0 | 1.5 / 1.0 | – | – | 1.5 / 1.1 | 1.5 / **0.9** | – | – | – | – | – |
| GF(2¹²⁸) | **1.4** / **1.1** | – | – | – | – | 2.1 / 1.5 | 1.7 / 1.2 | **1.4** / 1.2 | **1.4** / 1.4 | 1.5 / 1.3 | 1.9 / 1.7 | – |
| GF(2²³³) | **2.9** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.4** / **2.1** | – | – | – | – | 3.7 / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.6** / **5.3** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / **17.3** | – | 13.4 / 23.0 | 14.6 / 29.1 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **5.0** / **11.1** | 6.0 / 20.0 | 548 / 548 | 5.9 / 19.0 | – | – | – |
| GF(2¹²⁸) | **11.9** / **12.1** | – | 1,759 / 1,761 | **11.9** / 16.1 | **11.9** / 18.3 | 20.0 / 27.7 | 146 / 155 |
| GF(2²³³) | **39.5** / **43.4** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **33.8** / **33.9** | – | 5,724 / 5,734 | – | – | – | – |
| GF(2⁵⁷¹) | **170** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.7** | – | 8.0 / 18.0 | 8.3 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.7 | 125 / 139 | 5.0 / 15.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.3** | – | 288 / 286 | 7.9 / 16.2 | 9.9 / 19.0 | 16.4 / 27.3 | 146 / 156 |
| GF(2²³³) | **20.8** / **32.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.0** / **18.1** | – | 1,022 / 1,083 | – | – | – | – |
| GF(2⁵⁷¹) | **41.2** / **40.9** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.1** / – | – | 254 / – | 312 / – | – | – |
| GF(2¹⁶) | **170** / – | 419 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,223 / – | 1,209 / – | 1,064 / – | – | – |
| GF(2¹²⁸) | **1,935** / – | – | 3,226 / – | – | 5,074 / – | 39,272 / – |
| GF(2²³³) | **7,913** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,081** / – | – | 9,711 / – | – | – | – |
| GF(2⁵⁷¹) | **27,095** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 239 / – | 792 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 933 / – | 34,520 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | 224,010 / – | 17.7 / – | 2,446 / – | 3,591 / – | 19,876 / – |
| GF(2²³³) | **47.8** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.5** / – | – | 1,461,519 / – | – | – | – | – |
| GF(2⁵⁷¹) | **134** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.8 / 1.4 | 1.8 / 1.3 | **1.3** / 1.3 | 1.4 / 1.3 | **1.3** / 1.4 | 1.5 / 1.4 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.0** | – | – | – | – | 2.9 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.3 / 78.7 | **2.0** / 11.3 | **2.0** / 11.3 | 13.4 / 23.0 | 15.1 / 29.3 | – | – | – | – | – |
| GF(2¹⁶) | **5.3** / 32.2 | 31.9 / 54.5 | 141 / 143 | 8.9 / 41.5 | 7.4 / **22.6** | 53.0 / 52.0 | 14.1 / 30.5 | – | – | – | – | 12.4 / 61.9 |
| GF(2⁶⁴) | **31.6** / **41.7** | 101 / 117 | 365 / 366 | – | – | 548 / 548 | 46.9 / 68.7 | – | – | – | – | – |
| GF(2¹²⁸) | **54.2** / **64.9** | – | – | – | – | 1,758 / 1,760 | 163 / 186 | 169 / 180 | 550 / 546 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **202** / **201** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **190** | – | – | – | – | 5,729 / 5,728 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,290** / **1,281** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.5 / 68.7 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 31.6 / 56.5 | 138 / 124 | 6.4 / 11.7 | 6.5 / 17.1 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.2 / 24.0 |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.8 / 126 | 364 / 371 | – | – | 126 / 138 | 24.5 / 44.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | – | – | – | 288 / 286 | 36.9 / 39.9 | 174 / 180 | 547 / 546 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **21.1** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **27.0** | – | – | – | – | 1,022 / 1,084 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.0** / **57.0** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 307 / – | 29.3 / – | 29.5 / – | 254 / – | 310 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 1,151 / – | 997 / – | 905 / – | 520 / – | 324 / – | 20.4 / – | – | – | – | – | 9.9 / – |
| GF(2⁶⁴) | **532** / – | 13,908 / – | 8,591 / – | – | – | 1,209 / – | 3,237 / – | – | – | – | – | – |
| GF(2¹²⁸) | **832** / – | – | – | – | – | 3,215 / – | 975 / – | 11,303 / – | 139,406 / – | 147,713 / – | 39,268 / – | – |
| GF(2²³³) | **9,362** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,693** / – | – | – | – | – | 9,713 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **48,582** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 814 / – | 2,072 / – | 329 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 575 / – |
| GF(2⁶⁴) | **35.6** / – | 7,932 / – | 24,108 / – | – | – | 34,520 / – | 40.2 / – | – | – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | – | – | – | 224,029 / – | 123 / – | 22,812 / – | 69,340 / – | 69,339 / – | 19,876 / – | – |
| GF(2²³³) | **110** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.1** / – | – | – | – | – | 1,461,629 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **674** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.8 / 1.4 | 1.8 / 1.3 | **1.3** / 1.3 | 1.4 / 1.3 | **1.3** / 1.4 | 1.5 / 1.4 | – |
| GF(2²³³) | **2.6** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.0** | – | – | – | – | 2.9 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.2** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **12.1** / **21.0** | – | 13.4 / 23.0 | 15.1 / 29.3 | – | – | – |
| GF(2¹⁶) | **15.1** / **22.5** | 31.9 / 54.5 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **31.6** / **41.6** | 101 / 117 | 548 / 548 | 46.9 / 68.7 | – | – | – |
| GF(2¹²⁸) | **54.3** / **64.8** | – | 1,758 / 1,760 | 163 / 186 | 169 / 180 | 550 / 546 | 146 / 155 |
| GF(2²³³) | **199** / **207** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **193** | – | 5,729 / 5,728 | – | – | – | – |
| GF(2⁵⁷¹) | **1,290** / **1,292** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.7** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.6** / **22.2** | 31.6 / 56.5 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.8 / 126 | 126 / 138 | 24.5 / 44.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.3** | – | 288 / 286 | 36.9 / 39.9 | 174 / 180 | 547 / 546 | 146 / 156 |
| GF(2²³³) | **21.0** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.3** / **27.1** | – | 1,022 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **54.9** / **57.1** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **89.8** / – | – | 254 / – | 310 / – | – | – |
| GF(2¹⁶) | **227** / – | 1,151 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **569** / – | 13,908 / – | 1,209 / – | 3,237 / – | – | – |
| GF(2¹²⁸) | **1,864** / – | – | 3,215 / – | – | 139,406 / – | 39,268 / – |
| GF(2²³³) | **9,332** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,643** / – | – | 9,713 / – | – | – | – |
| GF(2⁵⁷¹) | **48,617** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 814 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **35.6** / – | 7,932 / – | 34,520 / – | 40.2 / – | – | – | – |
| GF(2¹²⁸) | **65.7** / – | – | 224,029 / – | 123 / – | 22,812 / – | 69,340 / – | 19,876 / – |
| GF(2²³³) | **110** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.1** / – | – | 1,461,629 / – | – | – | – | – |
| GF(2⁵⁷¹) | **674** / – | – | – | – | – | – | – |

## WebAssembly

`binary-field` at commit `c7970be` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-06.

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
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, in WebAssembly, Chrome | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Chrome | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| RUSTFLAGS, in WebAssembly, Firefox | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Firefox | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| Browser, in WebAssembly, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff` |
| Target features, in WebAssembly, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Chrome | 9,837 to 18,002 cycles and 3,081 instructions; its noise in a figure, 556 to 7,403 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 7,013 to 9,376 cycles and 3,081 instructions; its noise in a figure, 243 to 1,891 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 16,809 to 25,311 cycles and 1,885 instructions; its noise in a figure, 7,277 to 18,838 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 12,934 to 22,033 cycles and 1,885 instructions; its noise in a figure, 1,392 to 7,242 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 18.11 before, 35.91 after; its SMT sibling, CPU 28, 38% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 34.95 before, 34.88 after; its SMT sibling, CPU 28, 64% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 34.88 before, 31.21 after; its SMT sibling, CPU 28, 52% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 31.99 before, 18.86 after; its SMT sibling, CPU 28, 44% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.1 | +0.1 (1.05×) | 2.2 vs plonky3 2.1 | +0.1 (1.05×) |
| add | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs galois_field_2pm 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.2 | −0.1 (0.95×) | 2.1 vs gf256 2.2 | −0.1 (0.95×) |
| add | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.4 vs plonky3 2.2 | +0.2 (1.09×) |
| add | GF(2¹²⁸) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.3 vs plonky3 3.3 | +0.0 (1.00×) | 3.2 vs hekate-math 3.0 | +0.2 (1.07×) |
| add | GF(2²³³) | 7.4 | – | 7.5 | – | 6.7 | – | 6.5 | – |
| add | GF(2²⁵⁶) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 7.4 vs hekate-math 8.3 | −0.9 (0.89×) | 6.7 vs hekate-math 6.5 | +0.2 (1.03×) | 6.7 vs hekate-math 6.8 | −0.1 (0.99×) |
| add | GF(2⁵⁷¹) | 23.0 | – | 23.2 | – | 18.1 | – | 15.7 | – |
| mul | GF(2⁸) | 2.5 vs g2p 2.7 | −0.2 (0.93×) | 2.5 vs g2p 2.3 | +0.2 (1.09×) | 2.2 vs reed-solomon-erasure 2.7 | −0.5 (0.81×) | 2.5 vs reed-solomon-erasure 2.7 | −0.2 (0.93×) |
| mul | GF(2¹⁶) | 5.3 vs reed-solomon-erasure 9.3 | −4.0 (0.57×) | 6.3 vs reed-solomon-erasure 9.1 | −2.8 (0.69×) | 6.8 vs reed-solomon-erasure 8.3 | −1.5 (0.82×) | 6.3 vs reed-solomon-erasure 8.8 | −2.5 (0.72×) |
| mul | GF(2⁶⁴) | 63.9 vs plonky3 92.3 | −28.4 (0.69×) | 64.3 vs plonky3 95.0 | −30.7 (0.68×) | 71.9 vs plonky3 95.0 | −23.1 (0.76×) | 72.0 vs plonky3 95.3 | −23.3 (0.76×) |
| mul | GF(2¹²⁸) | 184 vs polyval 232 | −48.0 (0.79×) | 184 vs polyval 231 | −47.0 (0.80×) | 200 vs polyval 258 | −58.0 (0.78×) | 198 vs polyval 258 | −60.0 (0.77×) |
| mul | GF(2²³³) | 787 | – | 790 | – | 1,114 | – | 1,093 | – |
| mul | GF(2²⁵⁶) | 706 vs hekate-math 8,405 | −7,699 (0.08×) | 701 vs hekate-math 8,435 | −7,734 (0.08×) | 1,104 vs hekate-math 9,555 | −8,451 (0.12×) | 1,090 vs hekate-math 9,554 | −8,464 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,008 | – | 4,015 | – | 4,884 | – | 4,870 | – |
| square | GF(2⁸) | 2.2 vs reed-solomon-erasure 2.3 | −0.1 (0.96×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.2 vs reed-solomon-erasure 2.2 | +0.0 (1.00×) | 2.1 vs reed-solomon-erasure 2.4 | −0.3 (0.88×) |
| square | GF(2¹⁶) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.3 vs plonky3 2.9 | −0.6 (0.79×) | 2.1 vs plonky3 3.7 | −1.6 (0.57×) | 2.2 vs plonky3 4.2 | −2.0 (0.52×) |
| square | GF(2⁶⁴) | 16.3 vs plonky3 46.2 | −29.9 (0.35×) | 16.1 vs plonky3 46.2 | −30.1 (0.35×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square | GF(2¹²⁸) | 28.0 vs plonky3 34.9 | −6.9 (0.80×) | 28.0 vs plonky3 30.8 | −2.8 (0.91×) | 31.0 vs plonky3 43.3 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square | GF(2²³³) | 50.7 | – | 50.7 | – | 61.1 | – | 61.1 | – |
| square | GF(2²⁵⁶) | 57.9 vs hekate-math 1,384 | −1,326 (0.04×) | 58.0 vs hekate-math 1,380 | −1,322 (0.04×) | 62.0 vs hekate-math 1,603 | −1,541 (0.04×) | 62.1 vs hekate-math 1,669 | −1,607 (0.04×) |
| square | GF(2⁵⁷¹) | 145 | – | 145 | – | 153 | – | 150 | – |
| invert | GF(2⁸) | 2.2 vs gf256 3.6 | −1.4 (0.61×) | 2.2 vs gf256 4.1 | −1.9 (0.54×) | 2.3 vs gf256 5.7 | −3.4 (0.40×) | 2.2 vs gf256 4.8 | −2.6 (0.46×) |
| invert | GF(2¹⁶) | 2.3 vs galois_field_2pm lut 8.4 | −6.1 (0.27×) | 3.0 vs galois_field_2pm lut 9.1 | −6.1 (0.33×) | 2.7 vs galois_field_2pm lut 9.9 | −7.2 (0.27×) | 3.5 vs galois_field_2pm lut 10.1 | −6.6 (0.35×) |
| invert | GF(2⁶⁴) | 373 vs hekate-math 1,582 | −1,209 (0.24×) | 356 vs hekate-math 1,576 | −1,220 (0.23×) | 379 vs hekate-math 2,085 | −1,706 (0.18×) | 391 vs hekate-math 2,093 | −1,702 (0.19×) |
| invert | GF(2¹²⁸) | 941 vs plonky3 1,225 | −284 (0.77×) | 886 vs plonky3 1,257 | −371 (0.70×) | 939 vs plonky3 1,296 | −357 (0.72×) | 1,026 vs plonky3 1,309 | −283 (0.78×) |
| invert | GF(2²³³) | 14,664 | – | 14,857 | – | 20,153 | – | 20,201 | – |
| invert | GF(2²⁵⁶) | 3,645 vs hekate-math 13,755 | −10,110 (0.26×) | 3,650 vs hekate-math 13,687 | −10,037 (0.27×) | 4,602 vs hekate-math 15,946 | −11,344 (0.29×) | 4,674 vs hekate-math 15,944 | −11,270 (0.29×) |
| invert | GF(2⁵⁷¹) | 132,468 | – | 130,122 | – | 162,394 | – | 159,988 | – |
| sqrt | GF(2⁸) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.1 vs plonky3 11.5 | −9.4 (0.18×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) |
| sqrt | GF(2¹⁶) | 2.7 vs plonky3 3.4 | −0.7 (0.79×) | 2.4 vs plonky3 4.1 | −1.7 (0.59×) | 2.3 vs plonky3 3.8 | −1.5 (0.61×) | 1.7 vs plonky3 4.6 | −2.9 (0.37×) |
| sqrt | GF(2⁶⁴) | 37.1 vs plonky3 80.1 | −43.0 (0.46×) | 37.3 vs plonky3 80.0 | −42.7 (0.47×) | 41.4 vs plonky3 84.3 | −42.9 (0.49×) | 41.0 vs plonky3 84.3 | −43.3 (0.49×) |
| sqrt | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 155 vs plonky3 153 | +2.0 (1.01×) | 157 vs plonky3 164 | −7.0 (0.96×) | 156 vs plonky3 164 | −8.0 (0.95×) |
| sqrt | GF(2²³³) | 124 | – | 128 | – | 130 | – | 133 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,169,505 | −2,169,383 (0.00×) | 123 vs hekate-math 2,154,350 | −2,154,227 (0.00×) | 169 vs hekate-math 2,448,001 | −2,447,832 (0.00×) | 180 vs hekate-math 2,457,246 | −2,457,066 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,226 | – | 2,247 | – | 2,533 | – | 2,523 | – |
| add, constant time | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs plonky3 2.1 | +0.1 (1.05×) |
| add, constant time | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs galois_field_2pm 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.1 vs gf256 2.2 | −0.1 (0.95×) | 2.4 vs plonky3 2.2 | +0.2 (1.09×) |
| add, constant time | GF(2¹²⁸) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.3 vs plonky3 3.3 | +0.0 (1.00×) | 3.4 vs hekate-math 3.0 | +0.4 (1.13×) |
| add, constant time | GF(2²³³) | 7.5 | – | 7.7 | – | 6.5 | – | 6.8 | – |
| add, constant time | GF(2²⁵⁶) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 7.6 vs hekate-math 8.3 | −0.7 (0.92×) | 6.2 vs hekate-math 6.5 | −0.3 (0.95×) | 6.2 vs hekate-math 6.8 | −0.6 (0.91×) |
| add, constant time | GF(2⁵⁷¹) | 23.2 | – | 23.1 | – | 15.8 | – | 15.6 | – |
| mul, constant time | GF(2⁸) | 11.7 vs plonky3 19.2 | −7.5 (0.61×) | 11.7 vs plonky3 19.3 | −7.6 (0.61×) | 13.0 vs plonky3 19.5 | −6.5 (0.67×) | 13.0 vs plonky3 19.6 | −6.6 (0.66×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 76.1 | −59.6 (0.22×) | 16.6 vs hekate-math 76.1 | −59.5 (0.22×) | 16.6 vs hekate-math 85.0 | −68.4 (0.20×) | 17.7 vs hekate-math 82.9 | −65.2 (0.21×) |
| mul, constant time | GF(2⁶⁴) | 64.4 vs plonky3 92.3 | −27.9 (0.70×) | 65.7 vs plonky3 95.0 | −29.3 (0.69×) | 72.0 vs plonky3 95.0 | −23.0 (0.76×) | 72.0 vs plonky3 95.3 | −23.3 (0.76×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 232 | −48.0 (0.79×) | 184 vs polyval 231 | −47.0 (0.80×) | 200 vs polyval 258 | −58.0 (0.78×) | 200 vs polyval 258 | −58.0 (0.78×) |
| mul, constant time | GF(2²³³) | 787 | – | 792 | – | 1,098 | – | 1,104 | – |
| mul, constant time | GF(2²⁵⁶) | 704 vs hekate-math 8,405 | −7,701 (0.08×) | 713 vs hekate-math 8,435 | −7,722 (0.08×) | 1,086 vs hekate-math 9,555 | −8,469 (0.11×) | 1,099 vs hekate-math 9,554 | −8,455 (0.12×) |
| mul, constant time | GF(2⁵⁷¹) | 4,010 | – | 4,045 | – | 4,892 | – | 4,958 | – |
| square, constant time | GF(2⁸) | 5.1 vs hekate-math 8.8 | −3.7 (0.58×) | 5.1 vs hekate-math 9.0 | −3.9 (0.57×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) | 5.7 vs hekate-math 9.7 | −4.0 (0.59×) |
| square, constant time | GF(2¹⁶) | 11.1 vs hekate-math 25.4 | −14.3 (0.44×) | 10.7 vs hekate-math 25.4 | −14.7 (0.42×) | 10.4 vs hekate-math 26.9 | −16.5 (0.39×) | 10.4 vs hekate-math 27.1 | −16.7 (0.38×) |
| square, constant time | GF(2⁶⁴) | 16.6 vs plonky3 46.2 | −29.6 (0.36×) | 16.7 vs plonky3 46.2 | −29.5 (0.36×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square, constant time | GF(2¹²⁸) | 28.0 vs plonky3 34.9 | −6.9 (0.80×) | 28.1 vs plonky3 30.8 | −2.7 (0.91×) | 31.0 vs plonky3 43.3 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square, constant time | GF(2²³³) | 53.1 | – | 53.0 | – | 60.5 | – | 60.3 | – |
| square, constant time | GF(2²⁵⁶) | 62.0 vs hekate-math 1,384 | −1,322 (0.04×) | 62.0 vs hekate-math 1,380 | −1,318 (0.04×) | 64.7 vs hekate-math 1,603 | −1,538 (0.04×) | 66.4 vs hekate-math 1,669 | −1,603 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 153 | – | 153 | – | 167 | – | 167 | – |
| invert, constant time | GF(2⁸) | 123 vs plonky3 272 | −149 (0.45×) | 123 vs plonky3 271 | −148 (0.45×) | 132 vs plonky3 277 | −145 (0.48×) | 132 vs plonky3 277 | −145 (0.48×) |
| invert, constant time | GF(2¹⁶) | 239 vs hekate-math 393 | −154 (0.61×) | 239 vs hekate-math 393 | −154 (0.61×) | 233 vs hekate-math 717 | −484 (0.32×) | 233 vs hekate-math 712 | −479 (0.33×) |
| invert, constant time | GF(2⁶⁴) | 1,322 vs hekate-math 1,582 | −260 (0.84×) | 1,325 vs hekate-math 1,576 | −251 (0.84×) | 1,314 vs hekate-math 2,085 | −771 (0.63×) | 1,314 vs hekate-math 2,093 | −779 (0.63×) |
| invert, constant time | GF(2¹²⁸) | 2,910 vs hekate-math 4,487 | −1,577 (0.65×) | 2,896 vs hekate-math 4,553 | −1,657 (0.64×) | 2,655 vs hekate-math 5,434 | −2,779 (0.49×) | 2,610 vs hekate-math 5,428 | −2,818 (0.48×) |
| invert, constant time | GF(2²³³) | 19,267 | – | 19,367 | – | 19,921 | – | 20,068 | – |
| invert, constant time | GF(2²⁵⁶) | 8,312 vs hekate-math 13,755 | −5,443 (0.60×) | 8,348 vs hekate-math 13,687 | −5,339 (0.61×) | 7,824 vs hekate-math 15,946 | −8,122 (0.49×) | 7,838 vs hekate-math 15,944 | −8,106 (0.49×) |
| invert, constant time | GF(2⁵⁷¹) | 134,729 | – | 130,672 | – | 161,816 | – | 160,102 | – |
| sqrt, constant time | GF(2⁸) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 6.4 vs plonky3 11.5 | −5.1 (0.56×) | 6.3 vs plonky3 11.5 | −5.2 (0.55×) |
| sqrt, constant time | GF(2¹⁶) | 16.8 vs hekate-math 1,243 | −1,226 (0.01×) | 16.8 vs hekate-math 1,243 | −1,226 (0.01×) | 20.2 vs hekate-math 1,357 | −1,337 (0.01×) | 20.0 vs hekate-math 1,312 | −1,292 (0.02×) |
| sqrt, constant time | GF(2⁶⁴) | 37.1 vs plonky3 80.1 | −43.0 (0.46×) | 37.6 vs plonky3 80.0 | −42.4 (0.47×) | 41.1 vs plonky3 84.3 | −43.2 (0.49×) | 40.9 vs plonky3 84.3 | −43.4 (0.49×) |
| sqrt, constant time | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 155 vs plonky3 153 | +2.0 (1.01×) | 160 vs plonky3 164 | −4.0 (0.98×) | 159 vs plonky3 164 | −5.0 (0.97×) |
| sqrt, constant time | GF(2²³³) | 112 | – | 114 | – | 134 | – | 133 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,169,505 | −2,169,383 (0.00×) | 123 vs hekate-math 2,154,350 | −2,154,227 (0.00×) | 156 vs hekate-math 2,448,001 | −2,447,845 (0.00×) | 171 vs hekate-math 2,457,246 | −2,457,075 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,236 | – | 2,290 | – | 2,524 | – | 2,537 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.3 vs plonky3 1.0 | +0.3 (1.30×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs gf256 1.2 | +0.0 (1.00×) | 1.1 vs galois_field_2pm lut 1.0 | +0.1 (1.10×) |
| add | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs galois_field_2pm 1.0 | +0.0 (1.00×) | 1.2 vs gf256 1.1 | +0.1 (1.09×) |
| add | GF(2¹²⁸) | 2.3 vs compile-algebra 2.4 | −0.1 (0.96×) | 2.3 vs plonky3 2.4 | −0.1 (0.96×) | 2.2 vs plonky3 2.3 | −0.1 (0.96×) | 2.3 vs plonky3 2.3 | +0.0 (1.00×) |
| add | GF(2²³³) | 4.8 | – | 4.8 | – | 4.9 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 4.8 vs hekate-math 5.1 | −0.3 (0.94×) | 4.9 vs hekate-math 5.7 | −0.8 (0.86×) | 4.6 vs hekate-math 5.3 | −0.7 (0.87×) | 4.6 vs hekate-math 4.9 | −0.3 (0.94×) |
| add | GF(2⁵⁷¹) | 17.5 | – | 17.5 | – | 13.3 | – | 13.4 | – |
| mul | GF(2⁸) | 10.5 vs reed-solomon-erasure 11.7 | −1.2 (0.90×) | 10.6 vs g2p 11.7 | −1.1 (0.91×) | 11.0 vs g2p 11.8 | −0.8 (0.93×) | 11.0 vs g2p 11.8 | −0.8 (0.93×) |
| mul | GF(2¹⁶) | 30.7 vs reed-solomon-erasure 23.8 | +6.9 (1.29×) | 33.7 vs reed-solomon-erasure 23.7 | +10.0 (1.42×) | 30.9 vs reed-solomon-erasure 23.9 | +7.0 (1.29×) | 31.5 vs reed-solomon-erasure 24.6 | +6.9 (1.28×) |
| mul | GF(2⁶⁴) | 71.8 vs plonky3 75.4 | −3.6 (0.95×) | 72.0 vs plonky3 86.7 | −14.7 (0.83×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.2 vs plonky3 100 | −26.8 (0.73×) |
| mul | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 230 | −46.0 (0.80×) | 198 vs polyval 263 | −65.0 (0.75×) | 197 vs polyval 262 | −65.0 (0.75×) |
| mul | GF(2²³³) | 752 | – | 755 | – | 1,078 | – | 1,078 | – |
| mul | GF(2²⁵⁶) | 709 vs hekate-math 8,416 | −7,707 (0.08×) | 699 vs hekate-math 8,437 | −7,738 (0.08×) | 1,050 vs hekate-math 9,558 | −8,508 (0.11×) | 1,034 vs hekate-math 9,586 | −8,552 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,018 | – | 4,011 | – | 4,804 | – | 4,715 | – |
| square | GF(2⁸) | 5.1 vs reed-solomon-erasure 7.8 | −2.7 (0.65×) | 5.1 vs reed-solomon-erasure 7.0 | −1.9 (0.73×) | 5.3 vs g2p 7.8 | −2.5 (0.68×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs plonky3 8.3 | −2.0 (0.76×) | 6.3 vs plonky3 8.4 | −2.1 (0.75×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.1 | −40.1 (0.32×) | 19.3 vs plonky3 61.8 | −42.5 (0.31×) | 19.3 vs plonky3 62.1 | −42.8 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 36.0 | −9.0 (0.75×) | 27.0 vs plonky3 32.3 | −5.3 (0.84×) | 30.6 vs plonky3 43.3 | −12.7 (0.71×) | 30.1 vs plonky3 35.0 | −4.9 (0.86×) |
| square | GF(2²³³) | 54.0 | – | 53.7 | – | 59.1 | – | 59.0 | – |
| square | GF(2²⁵⁶) | 59.0 vs hekate-math 1,382 | −1,323 (0.04×) | 59.0 vs hekate-math 1,389 | −1,330 (0.04×) | 63.0 vs hekate-math 1,605 | −1,542 (0.04×) | 63.1 vs hekate-math 1,450 | −1,387 (0.04×) |
| square | GF(2⁵⁷¹) | 164 | – | 164 | – | 183 | – | 184 | – |
| add, constant time | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs plonky3 1.0 | +0.1 (1.10×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.2 | −0.1 (0.92×) | 1.1 vs galois_field_2pm lut 1.0 | +0.1 (1.10×) |
| add, constant time | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs galois_field_2pm 1.0 | +0.3 (1.30×) | 1.2 vs gf256 1.1 | +0.1 (1.09×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs compile-algebra 2.4 | −0.1 (0.96×) | 2.3 vs plonky3 2.4 | −0.1 (0.96×) | 2.3 vs plonky3 2.3 | +0.0 (1.00×) | 2.2 vs plonky3 2.3 | −0.1 (0.96×) |
| add, constant time | GF(2²³³) | 5.2 | – | 4.8 | – | 5.3 | – | 5.0 | – |
| add, constant time | GF(2²⁵⁶) | 4.8 vs hekate-math 5.1 | −0.3 (0.94×) | 4.8 vs hekate-math 5.7 | −0.9 (0.84×) | 5.6 vs hekate-math 5.3 | +0.3 (1.06×) | 4.7 vs hekate-math 4.9 | −0.2 (0.96×) |
| add, constant time | GF(2⁵⁷¹) | 17.1 | – | 17.5 | – | 14.0 | – | 13.8 | – |
| mul, constant time | GF(2⁸) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.1 | −2.1 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.1 vs plonky3 23.0 | −1.9 (0.92×) |
| mul, constant time | GF(2¹⁶) | 22.0 vs hekate-math 81.9 | −59.9 (0.27×) | 22.0 vs hekate-math 83.8 | −61.8 (0.26×) | 22.7 vs hekate-math 101 | −78.3 (0.22×) | 23.0 vs hekate-math 101 | −78.0 (0.23×) |
| mul, constant time | GF(2⁶⁴) | 72.0 vs plonky3 75.4 | −3.4 (0.95×) | 72.1 vs plonky3 86.7 | −14.6 (0.83×) | 73.3 vs plonky3 100 | −26.7 (0.73×) | 73.1 vs plonky3 100 | −26.9 (0.73×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 230 | −46.0 (0.80×) | 197 vs polyval 263 | −66.0 (0.75×) | 197 vs polyval 262 | −65.0 (0.75×) |
| mul, constant time | GF(2²³³) | 755 | – | 757 | – | 1,079 | – | 1,089 | – |
| mul, constant time | GF(2²⁵⁶) | 697 vs hekate-math 8,416 | −7,719 (0.08×) | 704 vs hekate-math 8,437 | −7,733 (0.08×) | 1,060 vs hekate-math 9,558 | −8,498 (0.11×) | 1,046 vs hekate-math 9,586 | −8,540 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,060 | – | 4,004 | – | 4,774 | – | 4,771 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.2 | −4.2 (0.78×) | 15.0 vs hekate-math 19.9 | −4.9 (0.75×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.1 vs hekate-math 34.0 | −10.9 (0.68×) | 23.0 vs hekate-math 36.2 | −13.2 (0.64×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.1 | −40.1 (0.32×) | 19.3 vs plonky3 61.8 | −42.5 (0.31×) | 19.3 vs plonky3 62.1 | −42.8 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.3 vs plonky3 36.0 | −8.7 (0.76×) | 27.3 vs plonky3 32.3 | −5.0 (0.85×) | 31.0 vs plonky3 43.3 | −12.3 (0.72×) | 31.0 vs plonky3 35.0 | −4.0 (0.89×) |
| square, constant time | GF(2²³³) | 54.2 | – | 54.3 | – | 59.3 | – | 59.4 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,382 | −1,323 (0.04×) | 59.5 vs hekate-math 1,389 | −1,330 (0.04×) | 63.0 vs hekate-math 1,605 | −1,542 (0.04×) | 63.0 vs hekate-math 1,450 | −1,387 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 162 | – | 163 | – | 184 | – | 183 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.4 (1.8×) |
| add | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.3 (2.5×) | 3.2 (2.5×) |
| add | GF(2²³³) | 2.3 | 7.4 (3.2×) | 7.5 (3.3×) | 6.7 (2.9×) | 6.5 (2.8×) |
| add | GF(2²⁵⁶) | 2.6 | 7.5 (2.9×) | 7.4 (2.8×) | 6.7 (2.6×) | 6.7 (2.6×) |
| add | GF(2⁵⁷¹) | 6.7 | 23.0 (3.4×) | 23.2 (3.5×) | 18.1 (2.7×) | 15.7 (2.3×) |
| mul | GF(2⁸) | 2.0 | 2.5 (1.2×) | 2.5 (1.2×) | 2.2 (1.1×) | 2.5 (1.2×) |
| mul | GF(2¹⁶) | 5.3 | 5.3 (1.0×) | 6.3 (1.2×) | 6.8 (1.3×) | 6.3 (1.2×) |
| mul | GF(2⁶⁴) | 31.6 | 63.9 (2.0×) | 64.3 (2.0×) | 71.9 (2.3×) | 72.0 (2.3×) |
| mul | GF(2¹²⁸) | 54.2 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 198 (3.7×) |
| mul | GF(2²³³) | 202 | 787 (3.9×) | 790 (3.9×) | 1,114 (5.5×) | 1,093 (5.4×) |
| mul | GF(2²⁵⁶) | 189 | 706 (3.7×) | 701 (3.7×) | 1,104 (5.8×) | 1,090 (5.8×) |
| mul | GF(2⁵⁷¹) | 1,290 | 4,008 (3.1×) | 4,015 (3.1×) | 4,884 (3.8×) | 4,870 (3.8×) |
| square | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) | 2.1 (1.8×) |
| square | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.3 (1.4×) | 2.1 (1.2×) | 2.2 (1.3×) |
| square | GF(2⁶⁴) | 5.4 | 16.3 (3.0×) | 16.1 (3.0×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.0 (4.2×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square | GF(2²³³) | 21.1 | 50.7 (2.4×) | 50.7 (2.4×) | 61.1 (2.9×) | 61.1 (2.9×) |
| square | GF(2²⁵⁶) | 20.4 | 57.9 (2.8×) | 58.0 (2.8×) | 62.0 (3.0×) | 62.1 (3.0×) |
| square | GF(2⁵⁷¹) | 55.0 | 145 (2.6×) | 145 (2.6×) | 153 (2.8×) | 150 (2.7×) |
| invert | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.2 (1.8×) | 2.3 (1.9×) | 2.2 (1.8×) |
| invert | GF(2¹⁶) | 1.7 | 2.3 (1.4×) | 3.0 (1.8×) | 2.7 (1.6×) | 3.5 (2.1×) |
| invert | GF(2⁶⁴) | 532 | 373 (0.7×) | 356 (0.7×) | 379 (0.7×) | 391 (0.7×) |
| invert | GF(2¹²⁸) | 832 | 941 (1.1×) | 886 (1.1×) | 939 (1.1×) | 1,026 (1.2×) |
| invert | GF(2²³³) | 9,362 | 14,664 (1.6×) | 14,857 (1.6×) | 20,153 (2.2×) | 20,201 (2.2×) |
| invert | GF(2²⁵⁶) | 8,693 | 3,645 (0.4×) | 3,650 (0.4×) | 4,602 (0.5×) | 4,674 (0.5×) |
| invert | GF(2⁵⁷¹) | 48,582 | 132,468 (2.7×) | 130,122 (2.7×) | 162,394 (3.3×) | 159,988 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) |
| sqrt | GF(2¹⁶) | 1.7 | 2.7 (1.6×) | 2.4 (1.4×) | 2.3 (1.4×) | 1.7 (1.0×) |
| sqrt | GF(2⁶⁴) | 35.6 | 37.1 (1.0×) | 37.3 (1.0×) | 41.4 (1.2×) | 41.0 (1.2×) |
| sqrt | GF(2¹²⁸) | 65.6 | 154 (2.3×) | 155 (2.4×) | 157 (2.4×) | 156 (2.4×) |
| sqrt | GF(2²³³) | 110 | 124 (1.1×) | 128 (1.2×) | 130 (1.2×) | 133 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 123 (1.3×) | 169 (1.8×) | 180 (1.9×) |
| sqrt | GF(2⁵⁷¹) | 674 | 2,226 (3.3×) | 2,247 (3.3×) | 2,533 (3.8×) | 2,523 (3.7×) |
| add, constant time | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) |
| add, constant time | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.1 (1.6×) | 2.4 (1.8×) |
| add, constant time | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.3 (2.5×) | 3.4 (2.6×) |
| add, constant time | GF(2²³³) | 2.6 | 7.5 (2.9×) | 7.7 (3.0×) | 6.5 (2.5×) | 6.8 (2.6×) |
| add, constant time | GF(2²⁵⁶) | 2.5 | 7.5 (3.0×) | 7.6 (3.0×) | 6.2 (2.5×) | 6.2 (2.5×) |
| add, constant time | GF(2⁵⁷¹) | 7.2 | 23.2 (3.2×) | 23.1 (3.2×) | 15.8 (2.2×) | 15.6 (2.2×) |
| mul, constant time | GF(2⁸) | 12.1 | 11.7 (1.0×) | 11.7 (1.0×) | 13.0 (1.1×) | 13.0 (1.1×) |
| mul, constant time | GF(2¹⁶) | 15.1 | 16.5 (1.1×) | 16.6 (1.1×) | 16.6 (1.1×) | 17.7 (1.2×) |
| mul, constant time | GF(2⁶⁴) | 31.6 | 64.4 (2.0×) | 65.7 (2.1×) | 72.0 (2.3×) | 72.0 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.3 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 200 (3.7×) |
| mul, constant time | GF(2²³³) | 199 | 787 (4.0×) | 792 (4.0×) | 1,098 (5.5×) | 1,104 (5.5×) |
| mul, constant time | GF(2²⁵⁶) | 189 | 704 (3.7×) | 713 (3.8×) | 1,086 (5.7×) | 1,099 (5.8×) |
| mul, constant time | GF(2⁵⁷¹) | 1,290 | 4,010 (3.1×) | 4,045 (3.1×) | 4,892 (3.8×) | 4,958 (3.8×) |
| square, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 5.7 (1.6×) |
| square, constant time | GF(2¹⁶) | 8.6 | 11.1 (1.3×) | 10.7 (1.2×) | 10.4 (1.2×) | 10.4 (1.2×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 16.6 (3.1×) | 16.7 (3.1×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.1 (4.3×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 21.0 | 53.1 (2.5×) | 53.0 (2.5×) | 60.5 (2.9×) | 60.3 (2.9×) |
| square, constant time | GF(2²⁵⁶) | 20.3 | 62.0 (3.1×) | 62.0 (3.1×) | 64.7 (3.2×) | 66.4 (3.3×) |
| square, constant time | GF(2⁵⁷¹) | 54.9 | 153 (2.8×) | 153 (2.8×) | 167 (3.0×) | 167 (3.0×) |
| invert, constant time | GF(2⁸) | 89.8 | 123 (1.4×) | 123 (1.4×) | 132 (1.5×) | 132 (1.5×) |
| invert, constant time | GF(2¹⁶) | 227 | 239 (1.1×) | 239 (1.1×) | 233 (1.0×) | 233 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 569 | 1,322 (2.3×) | 1,325 (2.3×) | 1,314 (2.3×) | 1,314 (2.3×) |
| invert, constant time | GF(2¹²⁸) | 1,864 | 2,910 (1.6×) | 2,896 (1.6×) | 2,655 (1.4×) | 2,610 (1.4×) |
| invert, constant time | GF(2²³³) | 9,332 | 19,267 (2.1×) | 19,367 (2.1×) | 19,921 (2.1×) | 20,068 (2.2×) |
| invert, constant time | GF(2²⁵⁶) | 8,643 | 8,312 (1.0×) | 8,348 (1.0×) | 7,824 (0.9×) | 7,838 (0.9×) |
| invert, constant time | GF(2⁵⁷¹) | 48,617 | 134,729 (2.8×) | 130,672 (2.7×) | 161,816 (3.3×) | 160,102 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.4 (1.8×) | 6.3 (1.8×) |
| sqrt, constant time | GF(2¹⁶) | 13.5 | 16.8 (1.2×) | 16.8 (1.2×) | 20.2 (1.5×) | 20.0 (1.5×) |
| sqrt, constant time | GF(2⁶⁴) | 35.6 | 37.1 (1.0×) | 37.6 (1.1×) | 41.1 (1.2×) | 40.9 (1.1×) |
| sqrt, constant time | GF(2¹²⁸) | 65.7 | 154 (2.3×) | 155 (2.4×) | 160 (2.4×) | 159 (2.4×) |
| sqrt, constant time | GF(2²³³) | 110 | 112 (1.0×) | 114 (1.0×) | 134 (1.2×) | 133 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 123 (1.3×) | 156 (1.7×) | 171 (1.8×) |
| sqrt, constant time | GF(2⁵⁷¹) | 674 | 2,236 (3.3×) | 2,290 (3.4×) | 2,524 (3.7×) | 2,537 (3.8×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.3 (1.3×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) | 1.1 (1.1×) |
| add | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.2 (1.1×) |
| add | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.2 (2.0×) | 2.3 (2.1×) |
| add | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.9 (2.5×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.0 | 4.8 (2.4×) | 4.9 (2.5×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2⁵⁷¹) | 5.0 | 17.5 (3.5×) | 17.5 (3.5×) | 13.3 (2.7×) | 13.4 (2.7×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.6 (1.1×) | 11.0 (1.2×) | 11.0 (1.2×) |
| mul | GF(2¹⁶) | 32.2 | 30.7 (1.0×) | 33.7 (1.0×) | 30.9 (1.0×) | 31.5 (1.0×) |
| mul | GF(2⁶⁴) | 41.7 | 71.8 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.2 (1.8×) |
| mul | GF(2¹²⁸) | 64.9 | 184 (2.8×) | 184 (2.8×) | 198 (3.1×) | 197 (3.0×) |
| mul | GF(2²³³) | 201 | 752 (3.7×) | 755 (3.8×) | 1,078 (5.4×) | 1,078 (5.4×) |
| mul | GF(2²⁵⁶) | 190 | 709 (3.7×) | 699 (3.7×) | 1,050 (5.5×) | 1,034 (5.4×) |
| mul | GF(2⁵⁷¹) | 1,281 | 4,018 (3.1×) | 4,011 (3.1×) | 4,804 (3.8×) | 4,715 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.3 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 27.0 (1.9×) | 30.6 (2.2×) | 30.1 (2.1×) |
| square | GF(2²³³) | 32.0 | 54.0 (1.7×) | 53.7 (1.7×) | 59.1 (1.8×) | 59.0 (1.8×) |
| square | GF(2²⁵⁶) | 27.0 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.1 (2.3×) |
| square | GF(2⁵⁷¹) | 57.0 | 164 (2.9×) | 164 (2.9×) | 183 (3.2×) | 184 (3.2×) |
| add, constant time | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.3 (1.2×) | 1.2 (1.1×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.3 (2.1×) | 2.2 (2.0×) |
| add, constant time | GF(2²³³) | 2.0 | 5.2 (2.6×) | 4.8 (2.4×) | 5.3 (2.6×) | 5.0 (2.5×) |
| add, constant time | GF(2²⁵⁶) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 5.6 (2.8×) | 4.7 (2.4×) |
| add, constant time | GF(2⁵⁷¹) | 5.0 | 17.1 (3.4×) | 17.5 (3.5×) | 14.0 (2.8×) | 13.8 (2.8×) |
| mul, constant time | GF(2⁸) | 21.0 | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) | 21.1 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 22.0 (1.0×) | 22.0 (1.0×) | 22.7 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.6 | 72.0 (1.7×) | 72.1 (1.7×) | 73.3 (1.8×) | 73.1 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 207 | 755 (3.6×) | 757 (3.7×) | 1,079 (5.2×) | 1,089 (5.3×) |
| mul, constant time | GF(2²⁵⁶) | 193 | 697 (3.6×) | 704 (3.6×) | 1,060 (5.5×) | 1,046 (5.4×) |
| mul, constant time | GF(2⁵⁷¹) | 1,292 | 4,060 (3.1×) | 4,004 (3.1×) | 4,774 (3.7×) | 4,771 (3.7×) |
| square, constant time | GF(2⁸) | 10.7 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.1 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.3 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.3 | 27.3 (1.9×) | 27.3 (1.9×) | 31.0 (2.2×) | 31.0 (2.2×) |
| square, constant time | GF(2²³³) | 32.0 | 54.2 (1.7×) | 54.3 (1.7×) | 59.3 (1.9×) | 59.4 (1.9×) |
| square, constant time | GF(2²⁵⁶) | 27.1 | 59.0 (2.2×) | 59.5 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square, constant time | GF(2⁵⁷¹) | 57.1 | 162 (2.8×) | 163 (2.9×) | 184 (3.2×) | 183 (3.2×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
