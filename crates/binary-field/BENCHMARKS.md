# Benchmarks

`binary-field` at commit `d19565d` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 29.9 before, 18.38 after; its SMT sibling, CPU 29, 47% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 29.23 before, 19.84 after; its SMT sibling, CPU 29, 22% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n.
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
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.4 / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | **1.3** / **1.1** | 1.4 / 1.3 | 1.4 / 1.3 | 1.9 / 1.3 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.0** | – | – | – | – | 3.2 / 2.6 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.3 / 78.7 | **2.0** / 11.3 | 2.1 / 11.4 | 13.3 / 23.0 | 15.0 / 28.9 | – | – | – | – | – |
| GF(2¹⁶) | 8.0 / **19.0** | 8.0 / 27.0 | 141 / 143 | 8.3 / 30.8 | **7.4** / 22.6 | 53.0 / 52.0 | 14.1 / 30.3 | – | – | – | – | 11.3 / 53.1 |
| GF(2⁶⁴) | **5.0** / **11.2** | 6.0 / 20.0 | 365 / 366 | – | – | 548 / 548 | 6.0 / 19.0 | – | – | – | – | – |
| GF(2¹²⁸) | **12.0** / **12.0** | – | – | – | – | 1,759 / 1,760 | **12.0** / 16.0 | **12.0** / 18.6 | 20.0 / 27.7 | 550 / 546 | 146 / 156 | – |
| GF(2²³³) | **39.8** / **43.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | – | – | – | 5,722 / 5,713 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **167** / **163** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 58.6 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 7.0 / 27.8 | 138 / 139 | 6.4 / 11.6 | 6.4 / 17.1 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 9.3 / 24.0 |
| GF(2⁶⁴) | **4.5** / **11.3** | 6.0 / 20.7 | 365 / 372 | – | – | 126 / 138 | 5.0 / 15.6 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.2** | – | – | – | – | 290 / 285 | 7.9 / 16.2 | 10.0 / 18.9 | 16.5 / 27.7 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **21.2** / **32.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **17.1** | – | – | – | – | 1,021 / 1,084 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.4** / **41.8** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 307 / – | 29.0 / – | 29.3 / – | 254 / – | 309 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 420 / – | 1,009 / – | 658 / – | 521 / – | 324 / – | 20.9 / – | – | – | – | – | 9.0 / – |
| GF(2⁶⁴) | **209** / – | 1,223 / – | 8,439 / – | – | – | 1,220 / – | 1,065 / – | – | – | – | – | – |
| GF(2¹²⁸) | **499** / – | – | – | – | – | 3,193 / – | 545 / – | 5,975 / – | 5,041 / – | 147,715 / – | 39,276 / – | – |
| GF(2²³³) | **7,906** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,152** / – | – | – | – | – | 9,698 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,297** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 239 / – | 2,070 / – | 271 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 331 / – |
| GF(2⁶⁴) | **11.2** / – | 932 / – | 24,098 / – | – | – | 34,519 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | – | – | – | 221,740 / – | 17.7 / – | 2,447 / – | 3,620 / – | 69,341 / – | 19,875 / – | – |
| GF(2²³³) | **48.5** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | – | – | – | 1,462,398 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **131** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | **1.3** / **1.1** | 1.4 / 1.3 | 1.4 / 1.3 | 1.9 / 1.3 | – |
| GF(2²³³) | **2.7** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.0** | – | – | – | – | 3.2 / 2.6 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.3** / **4.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / **17.4** | – | 13.3 / 23.0 | 15.0 / 28.9 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **5.0** / **11.2** | 6.0 / 20.0 | 548 / 548 | 6.0 / 19.0 | – | – | – |
| GF(2¹²⁸) | **11.9** / **12.0** | – | 1,759 / 1,760 | 12.0 / 16.0 | 12.0 / 18.6 | 20.0 / 27.7 | 146 / 156 |
| GF(2²³³) | **39.6** / **43.4** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | 5,722 / 5,713 | – | – | – | – |
| GF(2⁵⁷¹) | **169** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.7** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.7 | 126 / 138 | 5.0 / 15.6 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | 290 / 285 | 7.9 / 16.2 | 10.0 / 18.9 | 16.5 / 27.7 | 146 / 156 |
| GF(2²³³) | **21.2** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **17.3** | – | 1,021 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **41.3** / **41.8** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.0** / – | – | 254 / – | 309 / – | – | – |
| GF(2¹⁶) | **167** / – | 420 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,223 / – | 1,220 / – | 1,065 / – | – | – |
| GF(2¹²⁸) | **1,937** / – | – | 3,193 / – | – | 5,041 / – | 39,276 / – |
| GF(2²³³) | **7,907** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,292** / – | – | 9,698 / – | – | – | – |
| GF(2⁵⁷¹) | **27,292** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.4** / – | 239 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 932 / – | 34,519 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | 221,740 / – | 17.7 / – | 2,447 / – | 3,620 / – | 19,875 / – |
| GF(2²³³) | **48.4** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | 1,462,398 / – | – | – | – | – |
| GF(2⁵⁷¹) | **132** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.4 / **1.1** | – | – | – | – | **1.3** / 1.3 | **1.3** / 1.3 | **1.3** / 1.3 | 1.4 / 1.4 | **1.3** / 1.4 | 1.4 / 1.2 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.1** | – | – | – | – | 3.0 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.4 / 78.6 | **2.0** / 11.3 | **2.0** / 11.4 | 13.3 / 23.0 | 14.8 / 29.8 | – | – | – | – | – |
| GF(2¹⁶) | **4.8** / 32.2 | 31.5 / 56.3 | 141 / 143 | 8.5 / 35.0 | 7.4 / **22.5** | 53.0 / 52.0 | 14.1 / 30.5 | – | – | – | – | 12.2 / 62.2 |
| GF(2⁶⁴) | **31.9** / **41.3** | 101 / 116 | 365 / 366 | – | – | 548 / 548 | 47.1 / 68.7 | – | – | – | – | – |
| GF(2¹²⁸) | **54.3** / **64.4** | – | – | – | – | 1,758 / 1,760 | 163 / 186 | 172 / 187 | 550 / 546 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **202** / **206** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **193** | – | – | – | – | 5,724 / 5,723 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,290** / **1,273** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 59.2 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 31.6 / 56.5 | 139 / 128 | 6.5 / 11.5 | 6.3 / 17.2 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.2 / 24.0 |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.9 / 126 | 364 / 370 | – | – | 126 / 138 | 24.5 / 44.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.3** | – | – | – | – | 288 / 285 | 36.9 / 39.9 | 175 / 178 | 547 / 546 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **21.3** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.5** / **27.1** | – | – | – | – | 1,022 / 1,082 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.2** / **57.2** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 308 / – | 29.3 / – | 29.5 / – | 254 / – | 307 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 1,150 / – | 1,001 / – | 773 / – | 518 / – | 324 / – | 20.4 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **529** / – | 13,918 / – | 8,402 / – | – | – | 1,221 / – | 3,243 / – | – | – | – | – | – |
| GF(2¹²⁸) | 1,069 / – | – | – | – | – | 3,210 / – | **974** / – | 11,333 / – | 139,428 / – | 147,716 / – | 39,263 / – | – |
| GF(2²³³) | **9,369** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,673** / – | – | – | – | – | 9,703 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **48,552** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.3 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 814 / – | 2,072 / – | 327 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 540 / – |
| GF(2⁶⁴) | **35.6** / – | 7,931 / – | 24,119 / – | – | – | 34,349 / – | 40.2 / – | – | – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | – | – | – | 222,942 / – | 123 / – | 22,953 / – | 69,370 / – | 69,368 / – | 19,879 / – | – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.0** / – | – | – | – | – | 1,469,489 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **674** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.4** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | **1.3** / 1.3 | **1.3** / 1.3 | **1.3** / 1.3 | 1.4 / 1.4 | **1.3** / 1.4 | 1.4 / 1.2 | – |
| GF(2²³³) | **2.4** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.1** | – | – | – | – | 3.0 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.8** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 14.0 / **21.7** | – | **13.3** / 23.0 | 14.8 / 29.8 | – | – | – |
| GF(2¹⁶) | **15.1** / **22.5** | 31.5 / 56.3 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **31.7** / **41.5** | 101 / 116 | 548 / 548 | 47.1 / 68.7 | – | – | – |
| GF(2¹²⁸) | **53.8** / **61.7** | – | 1,758 / 1,760 | 163 / 186 | 172 / 187 | 550 / 546 | 146 / 155 |
| GF(2²³³) | **203** / **207** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **192** | – | 5,724 / 5,723 | – | – | – | – |
| GF(2⁵⁷¹) | **1,289** / **1,284** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.9** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.6** / **22.2** | 31.6 / 56.5 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.9 / 126 | 126 / 138 | 24.5 / 44.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | 288 / 285 | 36.9 / 39.9 | 175 / 178 | 547 / 546 | 146 / 156 |
| GF(2²³³) | **20.7** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **26.6** | – | 1,022 / 1,082 | – | – | – | – |
| GF(2⁵⁷¹) | **55.1** / **57.1** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **104** / – | – | 254 / – | 307 / – | – | – |
| GF(2¹⁶) | **227** / – | 1,150 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **1,059** / – | 13,918 / – | 1,221 / – | 3,243 / – | – | – |
| GF(2¹²⁸) | **2,522** / – | – | 3,210 / – | – | 139,428 / – | 39,263 / – |
| GF(2²³³) | **9,221** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,696** / – | – | 9,703 / – | – | – | – |
| GF(2⁵⁷¹) | **48,540** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 814 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **35.7** / – | 7,931 / – | 34,349 / – | 40.2 / – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | 222,942 / – | 123 / – | 22,953 / – | 69,370 / – | 19,879 / – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.7** / – | – | 1,469,489 / – | – | – | – | – |
| GF(2⁵⁷¹) | **675** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file
```
