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

`binary-field` at commit `e517319` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 9,286 to 9,512 cycles and 3,081 instructions; its noise in a figure, 1,125 to 1,845 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 5,933 to 7,198 cycles and 3,081 instructions; its noise in a figure, 78.0 to 339 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 6,885 to 21,658 cycles and 1,885 instructions; its noise in a figure, 407 to 9,988 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 12,244 to 27,950 cycles and 1,885 instructions; its noise in a figure, 2,322 to 10,783 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 20.75 before, 26.52 after; its SMT sibling, CPU 28, 27% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 26.16 before, 20.31 after; its SMT sibling, CPU 28, 44% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 20.31 before, 21.34 after; its SMT sibling, CPU 28, 65% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 21.34 before, 15.72 after; its SMT sibling, CPU 28, 29% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.3 vs hekate-math 2.0 | +0.3 (1.15×) | 2.5 vs galois_field_2pm 2.0 | +0.5 (1.25×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs plonky3 2.1 | +0.0 (1.00×) | 2.4 vs g2p 2.1 | +0.3 (1.14×) | 2.2 vs galois_field_2pm 2.1 | +0.1 (1.05×) |
| add | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.3 vs galois_field_2pm 2.0 | +0.3 (1.15×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 3.8 vs runtime-algebra 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.7 vs hekate-math 3.3 | +0.4 (1.12×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add | GF(2²³³) | 7.5 | – | 8.0 | – | 7.0 | – | 6.3 | – |
| add | GF(2²⁵⁶) | 7.4 vs hekate-math 7.9 | −0.5 (0.94×) | 7.4 vs hekate-math 8.0 | −0.6 (0.93×) | 7.0 vs hekate-math 6.7 | +0.3 (1.04×) | 6.4 vs hekate-math 6.5 | −0.1 (0.98×) |
| add | GF(2⁵⁷¹) | 23.4 | – | 23.2 | – | 15.9 | – | 15.6 | – |
| mul | GF(2⁸) | 2.2 vs g2p 2.6 | −0.4 (0.85×) | 2.3 vs g2p 2.7 | −0.4 (0.85×) | 2.3 vs g2p 2.9 | −0.6 (0.79×) | 2.2 vs g2p 2.7 | −0.5 (0.81×) |
| mul | GF(2¹⁶) | 5.8 vs reed-solomon-erasure 8.7 | −2.9 (0.67×) | 6.6 vs reed-solomon-erasure 8.6 | −2.0 (0.77×) | 5.9 vs reed-solomon-erasure 9.6 | −3.7 (0.61×) | 6.0 vs reed-solomon-erasure 9.0 | −3.0 (0.67×) |
| mul | GF(2⁶⁴) | 63.9 vs plonky3 92.7 | −28.8 (0.69×) | 64.0 vs plonky3 92.9 | −28.9 (0.69×) | 72.0 vs plonky3 95.2 | −23.2 (0.76×) | 72.0 vs plonky3 95.4 | −23.4 (0.75×) |
| mul | GF(2¹²⁸) | 184 vs polyval 232 | −48.0 (0.79×) | 184 vs polyval 232 | −48.0 (0.79×) | 200 vs polyval 257 | −57.0 (0.78×) | 198 vs polyval 258 | −60.0 (0.77×) |
| mul | GF(2²³³) | 781 | – | 790 | – | 1,069 | – | 1,096 | – |
| mul | GF(2²⁵⁶) | 706 vs hekate-math 8,424 | −7,718 (0.08×) | 705 vs hekate-math 8,420 | −7,715 (0.08×) | 1,121 vs hekate-math 9,484 | −8,363 (0.12×) | 1,074 vs hekate-math 9,575 | −8,501 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,009 | – | 4,003 | – | 4,832 | – | 4,860 | – |
| square | GF(2⁸) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) |
| square | GF(2¹⁶) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.3 vs plonky3 3.2 | −0.9 (0.72×) | 2.1 vs plonky3 3.8 | −1.7 (0.55×) | 2.3 vs plonky3 3.2 | −0.9 (0.72×) |
| square | GF(2⁶⁴) | 15.0 vs plonky3 46.1 | −31.1 (0.33×) | 16.2 vs plonky3 46.2 | −30.0 (0.35×) | 17.1 vs plonky3 49.6 | −32.5 (0.34×) | 17.0 vs plonky3 49.6 | −32.6 (0.34×) |
| square | GF(2¹²⁸) | 28.0 vs plonky3 34.5 | −6.5 (0.81×) | 28.0 vs plonky3 30.7 | −2.7 (0.91×) | 31.1 vs plonky3 43.4 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square | GF(2²³³) | 50.6 | – | 50.8 | – | 61.2 | – | 61.0 | – |
| square | GF(2²⁵⁶) | 58.0 vs hekate-math 1,383 | −1,325 (0.04×) | 58.1 vs hekate-math 1,394 | −1,336 (0.04×) | 61.9 vs hekate-math 1,529 | −1,467 (0.04×) | 62.1 vs hekate-math 1,517 | −1,455 (0.04×) |
| square | GF(2⁵⁷¹) | 145 | – | 145 | – | 153 | – | 153 | – |
| invert | GF(2⁸) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.2 vs gf256 4.0 | −1.8 (0.55×) | 2.2 vs gf256 6.0 | −3.8 (0.37×) |
| invert | GF(2¹⁶) | 2.2 vs galois_field_2pm lut 8.2 | −6.0 (0.27×) | 2.3 vs galois_field_2pm lut 10.2 | −7.9 (0.23×) | 2.4 vs galois_field_2pm lut 9.7 | −7.3 (0.25×) | 2.3 vs galois_field_2pm lut 9.1 | −6.8 (0.25×) |
| invert | GF(2⁶⁴) | 798 vs hekate-math 1,583 | −785 (0.50×) | 799 vs hekate-math 1,582 | −783 (0.51×) | 907 vs hekate-math 2,065 | −1,158 (0.44×) | 910 vs hekate-math 2,093 | −1,183 (0.43×) |
| invert | GF(2¹²⁸) | 2,510 vs plonky3 1,217 | +1,293 (2.06×) | 2,527 vs plonky3 1,257 | +1,270 (2.01×) | 2,658 vs plonky3 1,316 | +1,342 (2.02×) | 2,623 vs plonky3 1,301 | +1,322 (2.02×) |
| invert | GF(2²³³) | 20,737 | – | 20,903 | – | 24,425 | – | 24,489 | – |
| invert | GF(2²⁵⁶) | 22,945 vs hekate-math 13,705 | +9,240 (1.67×) | 22,982 vs hekate-math 13,713 | +9,269 (1.68×) | 29,361 vs hekate-math 15,799 | +13,562 (1.86×) | 29,214 vs hekate-math 15,965 | +13,249 (1.83×) |
| invert | GF(2⁵⁷¹) | 130,158 | – | 130,306 | – | 160,149 | – | 160,983 | – |
| sqrt | GF(2⁸) | 2.2 vs plonky3 10.9 | −8.7 (0.20×) | 2.2 vs plonky3 10.9 | −8.7 (0.20×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) |
| sqrt | GF(2¹⁶) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) | 2.3 vs plonky3 3.6 | −1.3 (0.64×) | 2.8 vs plonky3 3.7 | −0.9 (0.76×) | 2.3 vs plonky3 3.5 | −1.2 (0.66×) |
| sqrt | GF(2⁶⁴) | 37.2 vs plonky3 80.0 | −42.8 (0.47×) | 37.7 vs plonky3 80.3 | −42.6 (0.47×) | 41.0 vs plonky3 84.0 | −43.0 (0.49×) | 40.6 vs plonky3 84.4 | −43.8 (0.48×) |
| sqrt | GF(2¹²⁸) | 154 vs plonky3 150 | +4.0 (1.03×) | 154 vs plonky3 155 | −1.0 (0.99×) | 159 vs plonky3 163 | −4.0 (0.98×) | 156 vs plonky3 163 | −7.0 (0.96×) |
| sqrt | GF(2²³³) | 102 | – | 115 | – | 134 | – | 134 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,156,012 | −2,155,890 (0.00×) | 123 vs hekate-math 2,161,373 | −2,161,250 (0.00×) | 159 vs hekate-math 2,432,484 | −2,432,325 (0.00×) | 172 vs hekate-math 2,441,090 | −2,440,918 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,218 | – | 2,224 | – | 2,538 | – | 2,548 | – |
| add, constant time | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.3 vs hekate-math 2.0 | +0.3 (1.15×) | 2.3 vs galois_field_2pm 2.0 | +0.3 (1.15×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs plonky3 2.1 | +0.1 (1.05×) | 2.9 vs g2p 2.1 | +0.8 (1.38×) | 2.2 vs galois_field_2pm 2.1 | +0.1 (1.05×) |
| add, constant time | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.1 vs galois_field_2pm 2.0 | +0.1 (1.05×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 3.8 vs runtime-algebra 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.6 vs hekate-math 3.3 | +0.3 (1.09×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add, constant time | GF(2²³³) | 7.4 | – | 7.2 | – | 6.5 | – | 6.5 | – |
| add, constant time | GF(2²⁵⁶) | 7.5 vs hekate-math 7.9 | −0.4 (0.95×) | 7.6 vs hekate-math 8.0 | −0.4 (0.95×) | 6.6 vs hekate-math 6.7 | −0.1 (0.99×) | 6.3 vs hekate-math 6.5 | −0.2 (0.97×) |
| add, constant time | GF(2⁵⁷¹) | 23.3 | – | 23.4 | – | 15.7 | – | 15.5 | – |
| mul, constant time | GF(2⁸) | 13.0 vs plonky3 19.2 | −6.2 (0.68×) | 13.0 vs plonky3 19.2 | −6.2 (0.68×) | 15.5 vs plonky3 19.5 | −4.0 (0.79×) | 15.5 vs plonky3 19.5 | −4.0 (0.79×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 76.1 | −59.6 (0.22×) | 16.5 vs hekate-math 76.0 | −59.5 (0.22×) | 17.2 vs hekate-math 84.2 | −67.0 (0.20×) | 16.8 vs hekate-math 82.9 | −66.1 (0.20×) |
| mul, constant time | GF(2⁶⁴) | 63.8 vs plonky3 92.7 | −28.9 (0.69×) | 64.0 vs plonky3 92.9 | −28.9 (0.69×) | 72.1 vs plonky3 95.2 | −23.1 (0.76×) | 72.1 vs plonky3 95.4 | −23.3 (0.76×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 232 | −48.0 (0.79×) | 184 vs polyval 232 | −48.0 (0.79×) | 201 vs polyval 257 | −56.0 (0.78×) | 197 vs polyval 258 | −61.0 (0.76×) |
| mul, constant time | GF(2²³³) | 792 | – | 795 | – | 1,062 | – | 1,087 | – |
| mul, constant time | GF(2²⁵⁶) | 705 vs hekate-math 8,424 | −7,719 (0.08×) | 706 vs hekate-math 8,420 | −7,714 (0.08×) | 1,093 vs hekate-math 9,484 | −8,391 (0.12×) | 1,108 vs hekate-math 9,575 | −8,467 (0.12×) |
| mul, constant time | GF(2⁵⁷¹) | 4,028 | – | 4,066 | – | 4,980 | – | 4,882 | – |
| square, constant time | GF(2⁸) | 5.1 vs hekate-math 9.1 | −4.0 (0.56×) | 5.1 vs hekate-math 9.0 | −3.9 (0.57×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) |
| square, constant time | GF(2¹⁶) | 10.2 vs hekate-math 25.4 | −15.2 (0.40×) | 10.3 vs hekate-math 25.4 | −15.1 (0.41×) | 10.4 vs hekate-math 27.0 | −16.6 (0.39×) | 10.4 vs hekate-math 25.1 | −14.7 (0.41×) |
| square, constant time | GF(2⁶⁴) | 16.9 vs plonky3 46.1 | −29.2 (0.37×) | 16.6 vs plonky3 46.2 | −29.6 (0.36×) | 17.0 vs plonky3 49.6 | −32.6 (0.34×) | 17.0 vs plonky3 49.6 | −32.6 (0.34×) |
| square, constant time | GF(2¹²⁸) | 28.0 vs plonky3 34.5 | −6.5 (0.81×) | 28.1 vs plonky3 30.7 | −2.6 (0.92×) | 31.1 vs plonky3 43.4 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square, constant time | GF(2²³³) | 53.0 | – | 53.1 | – | 57.2 | – | 58.9 | – |
| square, constant time | GF(2²⁵⁶) | 62.1 vs hekate-math 1,383 | −1,321 (0.04×) | 62.2 vs hekate-math 1,394 | −1,332 (0.04×) | 64.0 vs hekate-math 1,529 | −1,465 (0.04×) | 63.9 vs hekate-math 1,517 | −1,453 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 153 | – | 153 | – | 167 | – | 167 | – |
| invert, constant time | GF(2⁸) | 123 vs plonky3 272 | −149 (0.45×) | 123 vs plonky3 272 | −149 (0.45×) | 136 vs plonky3 277 | −141 (0.49×) | 136 vs plonky3 277 | −141 (0.49×) |
| invert, constant time | GF(2¹⁶) | 239 vs hekate-math 393 | −154 (0.61×) | 239 vs hekate-math 393 | −154 (0.61×) | 233 vs hekate-math 719 | −486 (0.32×) | 236 vs hekate-math 706 | −470 (0.33×) |
| invert, constant time | GF(2⁶⁴) | 1,312 vs hekate-math 1,583 | −271 (0.83×) | 1,305 vs hekate-math 1,582 | −277 (0.82×) | 1,303 vs hekate-math 2,065 | −762 (0.63×) | 1,319 vs hekate-math 2,093 | −774 (0.63×) |
| invert, constant time | GF(2¹²⁸) | 4,565 vs hekate-math 4,502 | +63.0 (1.01×) | 4,664 vs hekate-math 4,490 | +174 (1.04×) | 4,395 vs hekate-math 5,381 | −986 (0.82×) | 4,392 vs hekate-math 5,440 | −1,048 (0.81×) |
| invert, constant time | GF(2²³³) | 20,751 | – | 20,889 | – | 24,238 | – | 24,295 | – |
| invert, constant time | GF(2²⁵⁶) | 22,948 vs hekate-math 13,705 | +9,243 (1.67×) | 22,976 vs hekate-math 13,713 | +9,263 (1.68×) | 29,510 vs hekate-math 15,799 | +13,711 (1.87×) | 29,312 vs hekate-math 15,965 | +13,347 (1.84×) |
| invert, constant time | GF(2⁵⁷¹) | 130,404 | – | 129,893 | – | 160,824 | – | 160,708 | – |
| sqrt, constant time | GF(2⁸) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 6.3 vs plonky3 11.5 | −5.2 (0.55×) | 6.3 vs plonky3 11.5 | −5.2 (0.55×) |
| sqrt, constant time | GF(2¹⁶) | 16.8 vs hekate-math 1,242 | −1,225 (0.01×) | 16.9 vs hekate-math 1,243 | −1,226 (0.01×) | 20.0 vs hekate-math 1,353 | −1,333 (0.01×) | 20.0 vs hekate-math 1,356 | −1,336 (0.01×) |
| sqrt, constant time | GF(2⁶⁴) | 37.2 vs plonky3 80.0 | −42.8 (0.47×) | 37.3 vs plonky3 80.3 | −43.0 (0.46×) | 41.1 vs plonky3 84.0 | −42.9 (0.49×) | 40.9 vs plonky3 84.4 | −43.5 (0.48×) |
| sqrt, constant time | GF(2¹²⁸) | 154 vs plonky3 150 | +4.0 (1.03×) | 154 vs plonky3 155 | −1.0 (0.99×) | 156 vs plonky3 163 | −7.0 (0.96×) | 156 vs plonky3 163 | −7.0 (0.96×) |
| sqrt, constant time | GF(2²³³) | 106 | – | 120 | – | 134 | – | 132 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,156,012 | −2,155,890 (0.00×) | 122 vs hekate-math 2,161,373 | −2,161,251 (0.00×) | 191 vs hekate-math 2,432,484 | −2,432,293 (0.00×) | 186 vs hekate-math 2,441,090 | −2,440,904 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,220 | – | 2,224 | – | 2,522 | – | 2,529 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.0 vs gf256 1.1 | −0.1 (0.91×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs g2p 1.0 | +0.1 (1.10×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) |
| add | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.4 vs gf256 1.3 | +0.1 (1.08×) | 1.0 vs galois_field_2pm 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 2.3 vs plonky3 2.4 | −0.1 (0.96×) | 2.5 vs polyval 2.1 | +0.4 (1.19×) | 2.2 vs hekate-math 2.4 | −0.2 (0.92×) | 2.1 vs plonky3 2.3 | −0.2 (0.91×) |
| add | GF(2²³³) | 4.8 | – | 4.9 | – | 4.6 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 5.0 vs hekate-math 5.0 | +0.0 (1.00×) | 5.0 vs hekate-math 6.2 | −1.2 (0.81×) | 5.2 vs hekate-math 5.2 | +0.0 (1.00×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add | GF(2⁵⁷¹) | 17.3 | – | 17.7 | – | 15.2 | – | 13.0 | – |
| mul | GF(2⁸) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.4 vs g2p 11.7 | −1.3 (0.89×) | 11.0 vs g2p 11.9 | −0.9 (0.92×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) |
| mul | GF(2¹⁶) | 30.2 vs reed-solomon-erasure 23.9 | +6.3 (1.26×) | 35.3 vs reed-solomon-erasure 24.2 | +11.1 (1.46×) | 30.9 vs reed-solomon-erasure 24.9 | +6.0 (1.24×) | 30.8 vs reed-solomon-erasure 25.1 | +5.7 (1.23×) |
| mul | GF(2⁶⁴) | 71.9 vs plonky3 75.3 | −3.4 (0.95×) | 72.0 vs plonky3 81.2 | −9.2 (0.89×) | 73.1 vs plonky3 99.9 | −26.8 (0.73×) | 73.0 vs plonky3 100 | −27.0 (0.73×) |
| mul | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 229 | −45.0 (0.80×) | 197 vs polyval 263 | −66.0 (0.75×) | 197 vs polyval 265 | −68.0 (0.74×) |
| mul | GF(2²³³) | 749 | – | 760 | – | 1,074 | – | 1,065 | – |
| mul | GF(2²⁵⁶) | 703 vs hekate-math 8,432 | −7,729 (0.08×) | 712 vs hekate-math 8,429 | −7,717 (0.08×) | 1,040 vs hekate-math 9,477 | −8,437 (0.11×) | 1,057 vs hekate-math 9,572 | −8,515 (0.11×) |
| mul | GF(2⁵⁷¹) | 3,994 | – | 4,036 | – | 4,820 | – | 4,792 | – |
| square | GF(2⁸) | 5.1 vs g2p 7.7 | −2.6 (0.66×) | 5.1 vs reed-solomon-erasure 6.8 | −1.7 (0.75×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs plonky3 8.3 | −2.0 (0.76×) | 6.3 vs plonky3 8.4 | −2.1 (0.75×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.3 vs plonky3 61.6 | −42.3 (0.31×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 35.0 | −8.0 (0.77×) | 27.0 vs plonky3 32.4 | −5.4 (0.83×) | 30.3 vs plonky3 43.8 | −13.5 (0.69×) | 29.8 vs plonky3 35.1 | −5.3 (0.85×) |
| square | GF(2²³³) | 54.0 | – | 53.9 | – | 58.9 | – | 59.1 | – |
| square | GF(2²⁵⁶) | 59.0 vs hekate-math 1,382 | −1,323 (0.04×) | 59.0 vs hekate-math 1,381 | −1,322 (0.04×) | 63.0 vs hekate-math 1,555 | −1,492 (0.04×) | 63.0 vs hekate-math 1,537 | −1,474 (0.04×) |
| square | GF(2⁵⁷¹) | 164 | – | 161 | – | 183 | – | 184 | – |
| add, constant time | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs gf256 1.1 | +0.1 (1.09×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.3 vs g2p 1.0 | +0.3 (1.30×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) |
| add, constant time | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs galois_field_2pm 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs plonky3 2.4 | −0.1 (0.96×) | 2.4 vs polyval 2.1 | +0.3 (1.14×) | 3.2 vs hekate-math 2.4 | +0.8 (1.33×) | 2.1 vs plonky3 2.3 | −0.2 (0.91×) |
| add, constant time | GF(2²³³) | 4.8 | – | 4.9 | – | 5.2 | – | 4.5 | – |
| add, constant time | GF(2²⁵⁶) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 5.0 vs hekate-math 6.2 | −1.2 (0.81×) | 6.0 vs hekate-math 5.2 | +0.8 (1.15×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add, constant time | GF(2⁵⁷¹) | 17.7 | – | 17.5 | – | 14.2 | – | 12.8 | – |
| mul, constant time | GF(2⁸) | 21.3 vs plonky3 23.0 | −1.7 (0.93×) | 21.3 vs plonky3 23.0 | −1.7 (0.93×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) |
| mul, constant time | GF(2¹⁶) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 23.0 vs hekate-math 95.2 | −72.2 (0.24×) | 23.0 vs hekate-math 95.3 | −72.3 (0.24×) |
| mul, constant time | GF(2⁶⁴) | 71.7 vs plonky3 75.3 | −3.6 (0.95×) | 72.2 vs plonky3 81.2 | −9.0 (0.89×) | 73.1 vs plonky3 99.9 | −26.8 (0.73×) | 73.2 vs plonky3 100 | −26.8 (0.73×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 229 | −45.0 (0.80×) | 197 vs polyval 263 | −66.0 (0.75×) | 197 vs polyval 265 | −68.0 (0.74×) |
| mul, constant time | GF(2²³³) | 756 | – | 759 | – | 1,088 | – | 1,069 | – |
| mul, constant time | GF(2²⁵⁶) | 701 vs hekate-math 8,432 | −7,731 (0.08×) | 713 vs hekate-math 8,429 | −7,716 (0.08×) | 1,046 vs hekate-math 9,477 | −8,431 (0.11×) | 1,048 vs hekate-math 9,572 | −8,524 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,009 | – | 4,005 | – | 4,686 | – | 4,809 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.3 vs plonky3 61.6 | −42.3 (0.31×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.3 vs plonky3 35.0 | −7.7 (0.78×) | 27.3 vs plonky3 32.4 | −5.1 (0.84×) | 31.0 vs plonky3 43.8 | −12.8 (0.71×) | 31.0 vs plonky3 35.1 | −4.1 (0.88×) |
| square, constant time | GF(2²³³) | 55.1 | – | 53.9 | – | 59.6 | – | 59.0 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,382 | −1,323 (0.04×) | 59.0 vs hekate-math 1,381 | −1,322 (0.04×) | 63.0 vs hekate-math 1,555 | −1,492 (0.04×) | 63.1 vs hekate-math 1,537 | −1,474 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 165 | – | 165 | – | 183 | – | 183 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.3 (1.5×) | 2.5 (1.7×) | 2.1 (1.4×) |
| add | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.4 (1.6×) | 2.2 (1.5×) |
| add | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.3 (1.8×) | 2.2 (1.7×) |
| add | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.7 (2.8×) | 3.3 (2.5×) |
| add | GF(2²³³) | 2.3 | 7.5 (3.3×) | 8.0 (3.5×) | 7.0 (3.0×) | 6.3 (2.7×) |
| add | GF(2²⁵⁶) | 2.6 | 7.4 (2.8×) | 7.4 (2.8×) | 7.0 (2.7×) | 6.4 (2.5×) |
| add | GF(2⁵⁷¹) | 6.7 | 23.4 (3.5×) | 23.2 (3.5×) | 15.9 (2.4×) | 15.6 (2.3×) |
| mul | GF(2⁸) | 2.0 | 2.2 (1.1×) | 2.3 (1.1×) | 2.3 (1.1×) | 2.2 (1.1×) |
| mul | GF(2¹⁶) | 5.3 | 5.8 (1.1×) | 6.6 (1.2×) | 5.9 (1.1×) | 6.0 (1.1×) |
| mul | GF(2⁶⁴) | 31.6 | 63.9 (2.0×) | 64.0 (2.0×) | 72.0 (2.3×) | 72.0 (2.3×) |
| mul | GF(2¹²⁸) | 54.2 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 198 (3.7×) |
| mul | GF(2²³³) | 202 | 781 (3.9×) | 790 (3.9×) | 1,069 (5.3×) | 1,096 (5.4×) |
| mul | GF(2²⁵⁶) | 189 | 706 (3.7×) | 705 (3.7×) | 1,121 (5.9×) | 1,074 (5.7×) |
| mul | GF(2⁵⁷¹) | 1,290 | 4,009 (3.1×) | 4,003 (3.1×) | 4,832 (3.7×) | 4,860 (3.8×) |
| square | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| square | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.3 (1.4×) | 2.1 (1.2×) | 2.3 (1.4×) |
| square | GF(2⁶⁴) | 5.4 | 15.0 (2.8×) | 16.2 (3.0×) | 17.1 (3.2×) | 17.0 (3.1×) |
| square | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.0 (4.2×) | 31.1 (4.7×) | 31.0 (4.7×) |
| square | GF(2²³³) | 21.1 | 50.6 (2.4×) | 50.8 (2.4×) | 61.2 (2.9×) | 61.0 (2.9×) |
| square | GF(2²⁵⁶) | 20.4 | 58.0 (2.8×) | 58.1 (2.8×) | 61.9 (3.0×) | 62.1 (3.0×) |
| square | GF(2⁵⁷¹) | 55.0 | 145 (2.6×) | 145 (2.6×) | 153 (2.8×) | 153 (2.8×) |
| invert | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| invert | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.3 (1.4×) | 2.4 (1.4×) | 2.3 (1.4×) |
| invert | GF(2⁶⁴) | 532 | 798 (1.5×) | 799 (1.5×) | 907 (1.7×) | 910 (1.7×) |
| invert | GF(2¹²⁸) | 832 | 2,510 (3.0×) | 2,527 (3.0×) | 2,658 (3.2×) | 2,623 (3.2×) |
| invert | GF(2²³³) | 9,362 | 20,737 (2.2×) | 20,903 (2.2×) | 24,425 (2.6×) | 24,489 (2.6×) |
| invert | GF(2²⁵⁶) | 8,693 | 22,945 (2.6×) | 22,982 (2.6×) | 29,361 (3.4×) | 29,214 (3.4×) |
| invert | GF(2⁵⁷¹) | 48,582 | 130,158 (2.7×) | 130,306 (2.7×) | 160,149 (3.3×) | 160,983 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| sqrt | GF(2¹⁶) | 1.7 | 2.3 (1.4×) | 2.3 (1.4×) | 2.8 (1.6×) | 2.3 (1.4×) |
| sqrt | GF(2⁶⁴) | 35.6 | 37.2 (1.0×) | 37.7 (1.1×) | 41.0 (1.2×) | 40.6 (1.1×) |
| sqrt | GF(2¹²⁸) | 65.6 | 154 (2.3×) | 154 (2.3×) | 159 (2.4×) | 156 (2.4×) |
| sqrt | GF(2²³³) | 110 | 102 (0.9×) | 115 (1.0×) | 134 (1.2×) | 134 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 123 (1.3×) | 159 (1.7×) | 172 (1.8×) |
| sqrt | GF(2⁵⁷¹) | 674 | 2,218 (3.3×) | 2,224 (3.3×) | 2,538 (3.8×) | 2,548 (3.8×) |
| add, constant time | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.3 (1.5×) | 2.3 (1.5×) | 2.1 (1.4×) |
| add, constant time | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.2 (1.5×) | 2.9 (1.9×) | 2.2 (1.5×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.1 (1.6×) | 2.2 (1.7×) |
| add, constant time | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.6 (2.8×) | 3.3 (2.5×) |
| add, constant time | GF(2²³³) | 2.6 | 7.4 (2.8×) | 7.2 (2.8×) | 6.5 (2.5×) | 6.5 (2.5×) |
| add, constant time | GF(2²⁵⁶) | 2.5 | 7.5 (3.0×) | 7.6 (3.0×) | 6.6 (2.6×) | 6.3 (2.5×) |
| add, constant time | GF(2⁵⁷¹) | 7.2 | 23.3 (3.2×) | 23.4 (3.2×) | 15.7 (2.2×) | 15.5 (2.2×) |
| mul, constant time | GF(2⁸) | 12.1 | 13.0 (1.1×) | 13.0 (1.1×) | 15.5 (1.3×) | 15.5 (1.3×) |
| mul, constant time | GF(2¹⁶) | 15.1 | 16.5 (1.1×) | 16.5 (1.1×) | 17.2 (1.1×) | 16.8 (1.1×) |
| mul, constant time | GF(2⁶⁴) | 31.6 | 63.8 (2.0×) | 64.0 (2.0×) | 72.1 (2.3×) | 72.1 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.3 | 184 (3.4×) | 184 (3.4×) | 201 (3.7×) | 197 (3.6×) |
| mul, constant time | GF(2²³³) | 199 | 792 (4.0×) | 795 (4.0×) | 1,062 (5.3×) | 1,087 (5.5×) |
| mul, constant time | GF(2²⁵⁶) | 189 | 705 (3.7×) | 706 (3.7×) | 1,093 (5.8×) | 1,108 (5.9×) |
| mul, constant time | GF(2⁵⁷¹) | 1,290 | 4,028 (3.1×) | 4,066 (3.2×) | 4,980 (3.9×) | 4,882 (3.8×) |
| square, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 6.3 (1.8×) |
| square, constant time | GF(2¹⁶) | 8.6 | 10.2 (1.2×) | 10.3 (1.2×) | 10.4 (1.2×) | 10.4 (1.2×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 16.9 (3.1×) | 16.6 (3.1×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.1 (4.3×) | 31.1 (4.7×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 21.0 | 53.0 (2.5×) | 53.1 (2.5×) | 57.2 (2.7×) | 58.9 (2.8×) |
| square, constant time | GF(2²⁵⁶) | 20.3 | 62.1 (3.1×) | 62.2 (3.1×) | 64.0 (3.2×) | 63.9 (3.1×) |
| square, constant time | GF(2⁵⁷¹) | 54.9 | 153 (2.8×) | 153 (2.8×) | 167 (3.0×) | 167 (3.0×) |
| invert, constant time | GF(2⁸) | 89.8 | 123 (1.4×) | 123 (1.4×) | 136 (1.5×) | 136 (1.5×) |
| invert, constant time | GF(2¹⁶) | 227 | 239 (1.1×) | 239 (1.1×) | 233 (1.0×) | 236 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 569 | 1,312 (2.3×) | 1,305 (2.3×) | 1,303 (2.3×) | 1,319 (2.3×) |
| invert, constant time | GF(2¹²⁸) | 1,864 | 4,565 (2.4×) | 4,664 (2.5×) | 4,395 (2.4×) | 4,392 (2.4×) |
| invert, constant time | GF(2²³³) | 9,332 | 20,751 (2.2×) | 20,889 (2.2×) | 24,238 (2.6×) | 24,295 (2.6×) |
| invert, constant time | GF(2²⁵⁶) | 8,643 | 22,948 (2.7×) | 22,976 (2.7×) | 29,510 (3.4×) | 29,312 (3.4×) |
| invert, constant time | GF(2⁵⁷¹) | 48,617 | 130,404 (2.7×) | 129,893 (2.7×) | 160,824 (3.3×) | 160,708 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 6.3 (1.8×) |
| sqrt, constant time | GF(2¹⁶) | 13.5 | 16.8 (1.2×) | 16.9 (1.3×) | 20.0 (1.5×) | 20.0 (1.5×) |
| sqrt, constant time | GF(2⁶⁴) | 35.6 | 37.2 (1.0×) | 37.3 (1.0×) | 41.1 (1.2×) | 40.9 (1.1×) |
| sqrt, constant time | GF(2¹²⁸) | 65.7 | 154 (2.3×) | 154 (2.3×) | 156 (2.4×) | 156 (2.4×) |
| sqrt, constant time | GF(2²³³) | 110 | 106 (1.0×) | 120 (1.1×) | 134 (1.2×) | 132 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 122 (1.3×) | 191 (2.1×) | 186 (2.0×) |
| sqrt, constant time | GF(2⁵⁷¹) | 674 | 2,220 (3.3×) | 2,224 (3.3×) | 2,522 (3.7×) | 2,529 (3.8×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.0 (1.0×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) |
| add | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.4 (1.3×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.5 (2.3×) | 2.2 (2.0×) | 2.1 (1.9×) |
| add | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.9 (2.5×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.0 | 5.0 (2.5×) | 5.0 (2.5×) | 5.2 (2.6×) | 4.6 (2.3×) |
| add | GF(2⁵⁷¹) | 5.0 | 17.3 (3.5×) | 17.7 (3.5×) | 15.2 (3.0×) | 13.0 (2.6×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.4 (1.1×) | 11.0 (1.2×) | 10.9 (1.2×) |
| mul | GF(2¹⁶) | 32.2 | 30.2 (0.9×) | 35.3 (1.1×) | 30.9 (1.0×) | 30.8 (1.0×) |
| mul | GF(2⁶⁴) | 41.7 | 71.9 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.0 (1.8×) |
| mul | GF(2¹²⁸) | 64.9 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul | GF(2²³³) | 201 | 749 (3.7×) | 760 (3.8×) | 1,074 (5.3×) | 1,065 (5.3×) |
| mul | GF(2²⁵⁶) | 190 | 703 (3.7×) | 712 (3.7×) | 1,040 (5.5×) | 1,057 (5.6×) |
| mul | GF(2⁵⁷¹) | 1,281 | 3,994 (3.1×) | 4,036 (3.2×) | 4,820 (3.8×) | 4,792 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.3 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 27.0 (1.9×) | 30.3 (2.1×) | 29.8 (2.1×) |
| square | GF(2²³³) | 32.0 | 54.0 (1.7×) | 53.9 (1.7×) | 58.9 (1.8×) | 59.1 (1.8×) |
| square | GF(2²⁵⁶) | 27.0 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square | GF(2⁵⁷¹) | 57.0 | 164 (2.9×) | 161 (2.8×) | 183 (3.2×) | 184 (3.2×) |
| add, constant time | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.3 (1.3×) | 1.2 (1.2×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.4 (2.2×) | 3.2 (2.9×) | 2.1 (1.9×) |
| add, constant time | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.9 (2.5×) | 5.2 (2.6×) | 4.5 (2.2×) |
| add, constant time | GF(2²⁵⁶) | 2.0 | 4.8 (2.4×) | 5.0 (2.5×) | 6.0 (3.0×) | 4.6 (2.3×) |
| add, constant time | GF(2⁵⁷¹) | 5.0 | 17.7 (3.5×) | 17.5 (3.5×) | 14.2 (2.8×) | 12.8 (2.6×) |
| mul, constant time | GF(2⁸) | 21.0 | 21.3 (1.0×) | 21.3 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 22.0 (1.0×) | 22.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.6 | 71.7 (1.7×) | 72.2 (1.7×) | 73.1 (1.8×) | 73.2 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 207 | 756 (3.7×) | 759 (3.7×) | 1,088 (5.3×) | 1,069 (5.2×) |
| mul, constant time | GF(2²⁵⁶) | 193 | 701 (3.6×) | 713 (3.7×) | 1,046 (5.4×) | 1,048 (5.4×) |
| mul, constant time | GF(2⁵⁷¹) | 1,292 | 4,009 (3.1×) | 4,005 (3.1×) | 4,686 (3.6×) | 4,809 (3.7×) |
| square, constant time | GF(2⁸) | 10.7 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.3 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.3 | 27.3 (1.9×) | 27.3 (1.9×) | 31.0 (2.2×) | 31.0 (2.2×) |
| square, constant time | GF(2²³³) | 32.0 | 55.1 (1.7×) | 53.9 (1.7×) | 59.6 (1.9×) | 59.0 (1.8×) |
| square, constant time | GF(2²⁵⁶) | 27.1 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.1 (2.3×) |
| square, constant time | GF(2⁵⁷¹) | 57.1 | 165 (2.9×) | 165 (2.9×) | 183 (3.2×) | 183 (3.2×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
