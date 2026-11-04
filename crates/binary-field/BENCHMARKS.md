# Benchmarks

`binary-field` at commit `7fb049f` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 17.34 before, 16.98 after; its SMT sibling, CPU 29, 12% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 28.98 before, 22.81 after; its SMT sibling, CPU 29, 70% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: the median over 31 repetitions of the cycles of 2n operations less those of n, over n.
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
| GF(2⁸) | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / 1.2 | **1.5** / **1.0** | **1.5** / 1.2 | **1.5** / 1.2 | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / 1.1 | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.4** / **1.1** | – | – | – | – | 2.0 / 1.6 | 1.8 / 1.4 | 1.5 / 1.2 | 1.5 / 1.4 | **1.4** / 1.4 | 1.8 / 1.7 | – |
| GF(2²³³) | **2.8** / **2.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.1** | – | – | – | – | 3.1 / 2.8 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.9** / **5.3** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **9.8** | 3.1 / 12.0 | 76.7 / 79.0 | 2.3 / 11.7 | **2.1** / 12.9 | 13.3 / 23.0 | 15.1 / 31.2 | – | – | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 141 / 143 | 9.1 / 37.4 | **8.0** / 25.0 | 53.2 / 52.7 | 14.2 / 30.6 | – | – | – | – | 13.4 / 65.4 |
| GF(2⁶⁴) | **5.0** / **11.2** | 5.9 / 20.0 | 365 / 368 | – | – | 549 / 548 | 5.9 / 19.1 | – | – | – | – | – |
| GF(2¹²⁸) | 12.0 / **12.0** | – | – | – | – | 1,770 / 1,761 | **11.9** / 16.1 | 12.0 / 18.7 | 20.0 / 27.7 | 550 / 547 | 147 / 156 | – |
| GF(2²³³) | **39.8** / **43.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | – | – | – | 5,731 / 5,229 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **164** / **142** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.8 / 70.8 | 2.0 / 7.0 | 2.0 / 7.0 | 8.1 / 18.2 | 8.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.9** / **6.0** | 7.0 / 27.8 | 138 / 132 | 7.0 / 12.5 | 6.9 / 17.7 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 13.1 / 24.2 |
| GF(2⁶⁴) | **4.5** / **11.3** | 6.0 / 20.8 | 364 / 378 | – | – | 126 / 138 | 5.0 / 16.3 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.5** | – | – | – | – | 288 / 287 | 7.9 / 16.6 | 9.9 / 18.9 | 16.5 / 29.7 | 548 / 546 | 146 / 157 | – |
| GF(2²³³) | **21.3** / **32.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.4** | – | – | – | – | 1,023 / 1,084 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.4** / **41.9** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 308 / – | 29.6 / – | 30.7 / – | 254 / – | 313 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.9** / – | 420 / – | 1,017 / – | 823 / – | 525 / – | 324 / – | 20.4 / – | – | – | – | – | 11.9 / – |
| GF(2⁶⁴) | **211** / – | 1,225 / – | 8,526 / – | – | – | 1,215 / – | 1,065 / – | – | – | – | – | – |
| GF(2¹²⁸) | **505** / – | – | – | – | – | 3,198 / – | 603 / – | 6,006 / – | 5,078 / – | 149,010 / – | 39,587 / – | – |
| GF(2²³³) | **7,909** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,083** / – | – | – | – | – | 9,704 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,422** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.5 / – | 518 / – | 14.5 / – | 14.5 / – | 151 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 241 / – | 2,074 / – | 351 / – | 232 / – | 810 / – | 2.0 / – | – | – | – | – | 585 / – |
| GF(2⁶⁴) | **11.3** / – | 933 / – | 24,142 / – | – | – | 34,371 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.4** / – | – | – | – | – | 223,906 / – | 17.8 / – | 2,459 / – | 3,603 / – | 69,397 / – | 19,864 / – | – |
| GF(2²³³) | **48.7** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.7** / – | – | – | – | – | 1,467,459 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **134** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | **1.5** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / 1.2 | **1.5** / **1.0** | **1.5** / 1.2 | **1.5** / 1.2 | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.4** / 1.2 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / 1.1 | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.2** | – | – | – | – | 2.0 / 1.6 | 1.8 / 1.4 | 1.5 / **1.2** | 1.5 / 1.4 | 1.4 / 1.4 | 1.8 / 1.7 | – |
| GF(2²³³) | **2.9** / **2.2** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.1** | – | – | – | – | 3.1 / 2.8 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.1** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.6** / **17.7** | – | 13.3 / 23.0 | 15.1 / 31.2 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.2 / 52.7 | – | – | – | – |
| GF(2⁶⁴) | **4.9** / **11.2** | 5.9 / 20.0 | 549 / 548 | 5.9 / 19.1 | – | – | – |
| GF(2¹²⁸) | **11.9** / **12.0** | – | 1,770 / 1,761 | **11.9** / 16.1 | 12.0 / 18.7 | 20.0 / 27.7 | 147 / 156 |
| GF(2²³³) | **39.6** / **43.7** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | 5,731 / 5,229 | – | – | – | – |
| GF(2⁵⁷¹) | **137** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **11.3** | – | 8.1 / 18.2 | 8.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.3** | 6.0 / 20.8 | 126 / 138 | 5.0 / 16.3 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.5** | – | 288 / 287 | 7.9 / 16.6 | 9.9 / 18.9 | 16.5 / 29.7 | 146 / 157 |
| GF(2²³³) | **21.2** / **32.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.4** | – | 1,023 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **41.1** / **41.6** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.1** / – | – | 254 / – | 313 / – | – | – |
| GF(2¹⁶) | **170** / – | 420 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **673** / – | 1,225 / – | 1,215 / – | 1,065 / – | – | – |
| GF(2¹²⁸) | **1,967** / – | – | 3,198 / – | – | 5,078 / – | 39,587 / – |
| GF(2²³³) | **7,910** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,083** / – | – | 9,704 / – | – | – | – |
| GF(2⁵⁷¹) | **27,462** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 151 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 241 / – | 810 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 933 / – | 34,371 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.4** / – | – | 223,906 / – | 17.8 / – | 2,459 / – | 3,603 / – | 19,864 / – |
| GF(2²³³) | **48.7** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | 1,467,459 / – | – | – | – | – |
| GF(2⁵⁷¹) | **135** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.9** / 2.0 | **1.9** / **1.2** | **1.9** / 1.9 | 2.0 / 1.7 | **1.9** / 2.0 | **1.9** / 2.3 | **1.9** / 1.9 | – | – | – | – | – |
| GF(2¹⁶) | **1.8** / 1.9 | **1.8** / **1.5** | 2.0 / 1.9 | **1.8** / 2.0 | 1.9 / 1.6 | 2.2 / 2.2 | 2.4 / 1.8 | – | – | – | – | 2.3 / 1.9 |
| GF(2⁶⁴) | **1.7** / 1.8 | 1.9 / **1.7** | 2.0 / 1.9 | – | – | 2.1 / 2.2 | 1.9 / 1.9 | – | – | – | – | – |
| GF(2¹²⁸) | 2.0 / 1.8 | – | – | – | – | 2.5 / 2.5 | 2.0 / **1.5** | **1.8** / 1.8 | 1.9 / 1.8 | **1.8** / 1.9 | 3.0 / 2.9 | – |
| GF(2²³³) | **3.6** / **2.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.6** / **2.6** | – | – | – | – | 6.1 / 6.2 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8.7** / **5.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 3.0 / 12.5 | 5.1 / **12.2** | 87.6 / 86.0 | 3.0 / 13.7 | **2.6** / 14.8 | 18.8 / 24.1 | 21.7 / 29.5 | – | – | – | – | – |
| GF(2¹⁶) | 10.4 / 43.8 | 42.0 / 55.9 | 163 / 163 | 20.2 / 65.4 | **9.4** / **24.0** | 56.8 / 60.6 | 17.8 / 30.6 | – | – | – | – | 49.4 / 95.4 |
| GF(2⁶⁴) | **33.4** / **41.9** | 138 / 119 | 414 / 402 | – | – | 621 / 609 | 56.8 / 78.8 | – | – | – | – | – |
| GF(2¹²⁸) | **55.8** / **65.4** | – | – | – | – | 2,097 / 2,130 | 194 / 192 | 186 / 195 | 639 / 606 | 614 / 597 | 167 / 167 | – |
| GF(2²³³) | **232** / **240** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **203** / **213** | – | – | – | – | 6,849 / 6,554 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,302** / **1,378** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.8** / **5.0** | 3.1 / 11.0 | 75.8 / 75.3 | 2.1 / 7.1 | 2.3 / 7.1 | 10.2 / 18.5 | 10.9 / 23.2 | – | – | – | – | – |
| GF(2¹⁶) | **2.4** / **6.1** | 42.3 / 56.7 | 143 / 138 | 11.2 / 12.0 | 7.8 / 17.7 | 27.8 / 30.6 | 2.5 / 8.1 | – | – | – | – | 41.2 / 24.4 |
| GF(2⁶⁴) | **6.2** / **12.0** | 103 / 127 | 436 / 430 | – | – | 149 / 143 | 28.7 / 44.1 | – | – | – | – | – |
| GF(2¹²⁸) | **7.4** / **14.4** | – | – | – | – | 305 / 296 | 41.2 / 44.8 | 199 / 198 | 608 / 589 | 641 / 644 | 153 / 160 | – |
| GF(2²³³) | **21.2** / **32.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **21.5** / **27.1** | – | – | – | – | 1,252 / 1,298 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **59.3** / **60.0** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.9** / – | 3.8 / – | 343 / – | 51.3 / – | 50.1 / – | 277 / – | 343 / – | – | – | – | – | – |
| GF(2¹⁶) | **3.1** / – | 1,389 / – | 1,136 / – | 970 / – | 573 / – | 361 / – | 28.3 / – | – | – | – | – | 37.4 / – |
| GF(2⁶⁴) | **550** / – | 14,270 / – | 9,662 / – | – | – | 1,446 / – | 3,332 / – | – | – | – | – | – |
| GF(2¹²⁸) | **1,144** / – | – | – | – | – | 3,958 / – | 1,330 / – | 12,234 / – | 161,607 / – | 170,881 / – | 42,885 / – | – |
| GF(2²³³) | **10,249** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,707** / – | – | – | – | – | 11,147 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **54,204** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.8** / – | 37.5 / – | 527 / – | 24.6 / – | 23.4 / – | 168 / – | 10.6 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.5** / – | 835 / – | 2,401 / – | 388 / – | 270 / – | 975 / – | 2.6 / – | – | – | – | – | 657 / – |
| GF(2⁶⁴) | **35.8** / – | 7,954 / – | 27,283 / – | – | – | 43,952 / – | 61.7 / – | – | – | – | – | – |
| GF(2¹²⁸) | **69.4** / – | – | – | – | – | 278,628 / – | 147 / – | 26,740 / – | 81,958 / – | 83,596 / – | 22,385 / – | – |
| GF(2²³³) | **120** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **105** / – | – | – | – | – | 1,850,543 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **791** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.0 / 1.9 | **1.9** / **1.2** | **1.9** / 1.9 | 2.0 / 1.7 | **1.9** / 2.0 | **1.9** / 2.3 | **1.9** / 1.9 | – | – | – | – | – |
| GF(2¹⁶) | **1.8** / 1.6 | **1.8** / **1.5** | 2.0 / 1.9 | **1.8** / 2.0 | 1.9 / 1.6 | 2.2 / 2.2 | 2.4 / 1.8 | – | – | – | – | 2.3 / 1.9 |
| GF(2⁶⁴) | **1.7** / **1.7** | 1.9 / **1.7** | 2.0 / 1.9 | – | – | 2.1 / 2.2 | 1.9 / 1.9 | – | – | – | – | – |
| GF(2¹²⁸) | **1.8** / 1.8 | – | – | – | – | 2.5 / 2.5 | 2.0 / **1.5** | **1.8** / 1.8 | 1.9 / 1.8 | **1.8** / 1.9 | 3.0 / 2.9 | – |
| GF(2²³³) | **3.2** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.3** / **2.3** | – | – | – | – | 6.1 / 6.2 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8.1** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **16.8** / **22.2** | – | 18.8 / 24.1 | 21.7 / 29.5 | – | – | – |
| GF(2¹⁶) | **18.8** / **23.2** | 42.0 / 55.9 | 56.8 / 60.6 | – | – | – | – |
| GF(2⁶⁴) | **34.0** / **42.1** | 138 / 119 | 621 / 609 | 56.8 / 78.8 | – | – | – |
| GF(2¹²⁸) | **58.0** / **64.9** | – | 2,097 / 2,130 | 194 / 192 | 186 / 195 | 639 / 606 | 167 / 167 |
| GF(2²³³) | **249** / **261** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **230** / **227** | – | 6,849 / 6,554 | – | – | – | – |
| GF(2⁵⁷¹) | **1,529** / **1,477** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.2** / **11.1** | – | 10.2 / 18.5 | 10.9 / 23.2 | – | – | – |
| GF(2¹⁶) | **12.7** / **22.4** | 42.3 / 56.7 | 27.8 / 30.6 | – | – | – | – |
| GF(2⁶⁴) | **5.8** / **11.9** | 103 / 127 | 149 / 143 | 28.7 / 44.1 | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.5** | – | 305 / 296 | 41.2 / 44.8 | 199 / 198 | 608 / 589 | 153 / 160 |
| GF(2²³³) | **21.1** / **32.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.5** / **27.2** | – | 1,252 / 1,298 | – | – | – | – |
| GF(2⁵⁷¹) | **56.8** / **59.6** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **112** / – | – | 277 / – | 343 / – | – | – |
| GF(2¹⁶) | **246** / – | 1,389 / – | 361 / – | – | – | – |
| GF(2⁶⁴) | **1,067** / – | 14,270 / – | 1,446 / – | 3,332 / – | – | – |
| GF(2¹²⁸) | **2,559** / – | – | 3,958 / – | – | 161,607 / – | 42,885 / – |
| GF(2²³³) | **10,062** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,746** / – | – | 11,147 / – | – | – | – |
| GF(2⁵⁷¹) | **53,211** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.1** / – | – | 168 / – | 10.6 / – | – | – | – |
| GF(2¹⁶) | **17.7** / – | 835 / – | 975 / – | – | – | – | – |
| GF(2⁶⁴) | **35.9** / – | 7,954 / – | 43,952 / – | 61.7 / – | – | – | – |
| GF(2¹²⁸) | **70.9** / – | – | 278,628 / – | 147 / – | 26,740 / – | 81,958 / – | 22,385 / – |
| GF(2²³³) | **119** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **106** / – | – | 1,850,543 / – | – | – | – | – |
| GF(2⁵⁷¹) | **795** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file
```
