# Benchmarks

`binary-field` at commit `bac0aed` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-04.

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
- Each figure: the cycles of 2n operations less those of n, over n, of rank 4 from the fewest among 31 repetitions that count both runs back to back.
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

## WebAssembly

`binary-field` at commit `d19565d` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-04.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions.
- Each figure: the counts of rank 4 from the fewest among 31 of each run length, 2n operations less n, over n. A case whose difference counts under 50 times the noise of the fixed cost of a count stops the sweep.
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
| Target features, in WebAssembly, Chrome | `bulk-memory`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Chrome | 5,423 to 5,481 cycles and 3,081 instructions; its noise in a figure, 408 to 494 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 7,610 to 20,231 cycles and 3,081 instructions; its noise in a figure, 2,052 to 2,609 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 21,917 to 27,236 cycles and 1,885 instructions; its noise in a figure, 5,609 to 8,048 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 6,967 to 11,801 cycles and 1,885 instructions; its noise in a figure, 800 to 1,266 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 21.31 before, 17.75 after; its SMT sibling, CPU 28, 21% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 21.08 before, 23.63 after; its SMT sibling, CPU 28, 71% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 24.7 before, 16.59 after; its SMT sibling, CPU 28, 43% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 16.59 before, 18.28 after; its SMT sibling, CPU 28, 35% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | 2.2 / 1.1 | **2.1** / 1.1 | 2.2 / 1.1 | 2.2 / 1.1 | 2.2 / **1.0** | **2.1** / 1.1 | **2.1** / 1.1 | – | – | – | – | 2.2 / 1.1 |
| GF(2⁶⁴) | 3.8 / 2.4 | **2.5** / 1.4 | 2.6 / **1.3** | – | – | 2.6 / 1.4 | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.4** | – | – | – | – | **3.7** / 2.6 | **3.7** / 2.6 | **3.7** / 2.7 | 3.9 / 2.6 | 3.9 / 2.5 | 3.8 / 2.5 | – |
| GF(2²³³) | **7.6** / **5.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8.1** / **5.0** | – | – | – | – | 8.2 / 5.6 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28.6** / **20.2** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.4** | 4.8 / 18.0 | 81.3 / 77.5 | 2.6 / 11.7 | 2.5 / 11.3 | 23.0 / 28.0 | 19.1 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **5.3** / 29.8 | 94.5 / 93.7 | 176 / 169 | 9.5 / 33.8 | 8.2 / **23.6** | 76.0 / 80.9 | 18.9 / 33.0 | – | – | – | – | 8.8 / 42.4 |
| GF(2⁶⁴) | 206 / 224 | 592 / 572 | 667 / 661 | – | – | 774 / 805 | **90.9** / **84.2** | – | – | – | – | – |
| GF(2¹²⁸) | 632 / 625 | – | – | – | – | 2,621 / 2,613 | 243 / 250 | 250 / 250 | 581 / 580 | 580 / 579 | **231** / **228** | – |
| GF(2²³³) | **3,411** / **3,410** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,454** / **3,661** | – | – | – | – | 8,413 / 8,426 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15,883** / **18,442** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **5.1** | 3.2 / 15.5 | 70.4 / 62.5 | 2.2 / 7.6 | 2.2 / 7.9 | 8.8 / 19.3 | 10.5 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / **6.1** | 92.7 / 99.0 | 164 / 160 | 6.4 / 12.3 | 7.4 / 18.4 | 25.3 / 34.0 | 3.0 / 8.3 | – | – | – | – | 7.5 / 27.1 |
| GF(2⁶⁴) | **15.8** / **26.0** | 579 / 572 | 653 / 675 | – | – | 182 / 196 | 47.9 / 57.0 | – | – | – | – | – |
| GF(2¹²⁸) | **28.2** / **31.5** | – | – | – | – | 504 / 498 | 34.3 / 35.7 | 251 / 250 | 580 / 579 | 580 / 579 | 227 / 227 | – |
| GF(2²³³) | **59.1** / **57.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **63.7** / **64.4** | – | – | – | – | 1,394 / 1,382 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **158** / **187** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 3.8 / – | 323 / – | 32.1 / – | 31.0 / – | 307 / – | 272 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 2,783 / – | 1,278 / – | 741 / – | 640 / – | 392 / – | 34.0 / – | – | – | – | – | 7.0 / – |
| GF(2⁶⁴) | 2,295 / – | 72,632 / – | 17,837 / – | – | – | **1,583** / – | 4,451 / – | – | – | – | – | – |
| GF(2¹²⁸) | 6,932 / – | – | – | – | – | 4,523 / – | **1,216** / – | 16,155 / – | 182,097 / – | 182,291 / – | 58,072 / – | – |
| GF(2²³³) | **46,526** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 51,974 / – | – | – | – | – | **13,690** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **307,221** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 30.3 / – | 484 / – | 12.8 / – | 13.0 / – | 174 / – | 10.0 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / – | 1,417 / – | 2,302 / – | 295 / – | 287 / – | 1,246 / – | 3.3 / – | – | – | – | – | 428 / – |
| GF(2⁶⁴) | 126 / – | 36,491 / – | 42,234 / – | – | – | 49,779 / – | **87.9** / – | – | – | – | – | – |
| GF(2¹²⁸) | 384 / – | – | – | – | – | 335,180 / – | **149** / – | 31,506 / – | 108,043 / – | 107,866 / – | 28,957 / – | – |
| GF(2²³³) | **166** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **204** / – | – | – | – | – | 2,152,900 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **9,327** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | 2.2 / 1.1 | **2.1** / 1.1 | 2.2 / 1.1 | 2.2 / 1.1 | 2.2 / **1.0** | **2.1** / 1.1 | **2.1** / 1.1 | – | – | – | – | 2.2 / 1.1 |
| GF(2⁶⁴) | 3.9 / 2.8 | **2.5** / 1.4 | 2.6 / **1.3** | – | – | 2.6 / 1.4 | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.9 / 2.7 | – | – | – | – | **3.7** / 2.6 | **3.7** / 2.6 | **3.7** / 2.7 | 3.9 / 2.6 | 3.9 / **2.5** | 3.8 / **2.5** | – |
| GF(2²³³) | **8.0** / **5.9** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.3** / **5.6** | – | – | – | – | 8.2 / **5.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28.2** / **19.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **13.0** / **21.3** | – | 23.0 / 28.0 | 19.1 / 23.0 | – | – | – |
| GF(2¹⁶) | **17.8** / **22.0** | 94.5 / 93.7 | 76.0 / 80.9 | – | – | – | – |
| GF(2⁶⁴) | 204 / 223 | 592 / 572 | 774 / 805 | **90.9** / **84.2** | – | – | – |
| GF(2¹²⁸) | 636 / 634 | – | 2,621 / 2,613 | 243 / 250 | 250 / 250 | 581 / 580 | **231** / **228** |
| GF(2²³³) | **3,418** / **3,419** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,501** / **3,639** | – | 8,413 / 8,426 | – | – | – | – |
| GF(2⁵⁷¹) | **15,822** / **18,528** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / **15.0** | – | 8.8 / 19.3 | 10.5 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.2** / **23.0** | 92.7 / 99.0 | 25.3 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **16.1** / **26.1** | 579 / 572 | 182 / 196 | 47.9 / 57.0 | – | – | – |
| GF(2¹²⁸) | **28.2** / **31.5** | – | 504 / 498 | 34.3 / 35.7 | 251 / 250 | 580 / 579 | 227 / 227 |
| GF(2²³³) | **61.7** / **57.8** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **66.7** / **63.5** | – | 1,394 / 1,382 | – | – | – | – |
| GF(2⁵⁷¹) | **190** / **187** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **138** / – | – | 307 / – | 272 / – | – | – |
| GF(2¹⁶) | **241** / – | 2,783 / – | 392 / – | – | – | – |
| GF(2⁶⁴) | 3,616 / – | 72,632 / – | **1,583** / – | 4,451 / – | – | – |
| GF(2¹²⁸) | 10,488 / – | – | **4,523** / – | – | 182,097 / – | 58,072 / – |
| GF(2²³³) | **46,514** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,030 / – | – | **13,690** / – | – | – | – |
| GF(2⁵⁷¹) | **307,556** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | 174 / – | 10.0 / – | – | – | – |
| GF(2¹⁶) | **15.2** / – | 1,417 / – | 1,246 / – | – | – | – | – |
| GF(2⁶⁴) | 126 / – | 36,491 / – | 49,779 / – | **87.9** / – | – | – | – |
| GF(2¹²⁸) | 386 / – | – | 335,180 / – | **149** / – | 31,506 / – | 108,043 / – | 28,957 / – |
| GF(2²³³) | **166** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **206** / – | – | 2,152,900 / – | – | – | – | – |
| GF(2⁵⁷¹) | **9,283** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.6 / 1.6 | 3.0 / 1.1 | 1.9 / **1.0** | **1.7** / 1.5 | 2.8 / 1.1 | 2.1 / 1.3 | 2.3 / 1.2 | – | – | – | – | – |
| GF(2¹⁶) | 2.6 / 1.4 | 2.5 / **1.1** | 2.5 / 1.3 | 3.4 / 1.2 | **2.3** / 1.7 | 3.1 / 1.3 | 3.0 / 1.4 | – | – | – | – | 2.9 / 1.4 |
| GF(2⁶⁴) | 4.9 / 2.9 | **2.7** / **1.6** | 3.0 / **1.6** | – | – | 2.8 / **1.6** | 3.0 / 1.7 | – | – | – | – | – |
| GF(2¹²⁸) | 4.4 / 3.1 | – | – | – | – | 4.4 / 2.9 | 4.6 / **2.8** | 4.6 / 3.1 | 4.1 / 2.9 | 4.1 / 3.1 | **4.0** / 3.0 | – |
| GF(2²³³) | **8.3** / **6.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 9.4 / **5.7** | – | – | – | – | **8.8** / 6.4 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **30.8** / **22.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.5** / **10.2** | 4.5 / 18.2 | 85.8 / 80.3 | 2.8 / 11.3 | 3.8 / 11.6 | 22.9 / 33.2 | 21.5 / 24.1 | – | – | – | – | – |
| GF(2¹⁶) | **5.9** / 27.4 | 94.9 / 96.5 | 185 / 177 | 9.3 / 25.2 | 8.8 / **23.5** | 72.0 / 85.2 | 15.9 / 34.6 | – | – | – | – | 6.9 / 33.4 |
| GF(2⁶⁴) | **64.8** / **63.8** | 598 / 581 | 696 / 666 | – | – | 800 / 805 | 91.7 / 107 | – | – | – | – | – |
| GF(2¹²⁸) | **201** / **180** | – | – | – | – | 2,660 / 2,615 | 249 / 252 | 260 / 256 | 602 / 592 | 617 / 579 | 238 / 225 | – |
| GF(2²³³) | **746** / **708** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **771** / **781** | – | – | – | – | 8,482 / 8,359 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **3,892** / **3,595** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.1** | 3.1 / 14.9 | 70.6 / 66.8 | 2.9 / 7.9 | 3.3 / 8.0 | 10.1 / 19.2 | 15.3 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **3.0** / **6.1** | 104 / 101 | 175 / 162 | 6.3 / 12.5 | 9.0 / 18.4 | 26.3 / 33.9 | 4.0 / 8.3 | – | – | – | – | 6.4 / 27.0 |
| GF(2⁶⁴) | **17.5** / **26.3** | 655 / 570 | 652 / 672 | – | – | 194 / 197 | 47.8 / 57.1 | – | – | – | – | – |
| GF(2¹²⁸) | **35.3** / **31.8** | – | – | – | – | 485 / 496 | 40.8 / 32.8 | 248 / 250 | 685 / 580 | 600 / 578 | 233 / 226 | – |
| GF(2²³³) | **60.2** / **57.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **62.8** / **64.4** | – | – | – | – | 1,427 / 1,383 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **162** / **191** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.3** / – | 4.5 / – | 337 / – | 31.7 / – | 30.4 / – | 311 / – | 271 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.8** / – | 2,846 / – | 1,445 / – | 840 / – | 641 / – | 408 / – | 39.5 / – | – | – | – | – | 6.7 / – |
| GF(2⁶⁴) | **626** / – | 72,530 / – | 17,556 / – | – | – | 1,581 / – | 4,449 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,542 / – | – | – | – | – | 4,507 / – | **1,207** / – | 16,183 / – | 183,932 / – | 188,377 / – | 58,626 / – | – |
| GF(2²³³) | **19,939** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 23,523 / – | – | – | – | – | **13,064** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **132,351** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 37.7 / – | 491 / – | 16.3 / – | 18.9 / – | 179 / – | 11.2 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.5** / – | 1,430 / – | 2,303 / – | 265 / – | 295 / – | 1,250 / – | 4.1 / – | – | – | – | – | 341 / – |
| GF(2⁶⁴) | **57.1** / – | 36,474 / – | 42,270 / – | – | – | 49,984 / – | 102 / – | – | – | – | – | – |
| GF(2¹²⁸) | **138** / – | – | – | – | – | 340,289 / – | 147 / – | 31,650 / – | 108,517 / – | 108,598 / – | 29,003 / – | – |
| GF(2²³³) | **134** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **181** / – | – | – | – | – | 2,238,453 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,947** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.3 / 1.3 | 3.0 / 1.1 | 1.9 / **1.0** | **1.7** / 1.5 | 2.8 / 1.1 | 2.1 / 1.3 | 2.3 / 1.2 | – | – | – | – | – |
| GF(2¹⁶) | 2.4 / **1.1** | 2.5 / **1.1** | 2.5 / 1.3 | 3.4 / 1.2 | **2.3** / 1.7 | 3.1 / 1.3 | 3.0 / 1.4 | – | – | – | – | 2.9 / 1.4 |
| GF(2⁶⁴) | 4.2 / 2.9 | **2.7** / **1.6** | 3.0 / **1.6** | – | – | 2.8 / **1.6** | 3.0 / 1.7 | – | – | – | – | – |
| GF(2¹²⁸) | 4.6 / 3.0 | – | – | – | – | 4.4 / 2.9 | 4.6 / **2.8** | 4.6 / 3.1 | 4.1 / 2.9 | 4.1 / 3.1 | **4.0** / 3.0 | – |
| GF(2²³³) | **8.8** / **6.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 9.1 / **6.1** | – | – | – | – | **8.8** / 6.4 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **30.9** / **23.5** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **18.4** / **23.3** | – | 22.9 / 33.2 | 21.5 / 24.1 | – | – | – |
| GF(2¹⁶) | **18.5** / **23.7** | 94.9 / 96.5 | 72.0 / 85.2 | – | – | – | – |
| GF(2⁶⁴) | **59.8** / **58.0** | 598 / 581 | 800 / 805 | 91.7 / 107 | – | – | – |
| GF(2¹²⁸) | **170** / **155** | – | 2,660 / 2,615 | 249 / 252 | 260 / 256 | 602 / 592 | 238 / 225 |
| GF(2²³³) | **696** / **696** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **707** / **718** | – | 8,482 / 8,359 | – | – | – | – |
| GF(2⁵⁷¹) | **3,652** / **3,638** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.9** / **15.0** | – | 10.1 / 19.2 | 15.3 / 25.0 | – | – | – |
| GF(2¹⁶) | **14.1** / **23.0** | 104 / 101 | 26.3 / 33.9 | – | – | – | – |
| GF(2⁶⁴) | **15.9** / **25.9** | 655 / 570 | 194 / 197 | 47.8 / 57.1 | – | – | – |
| GF(2¹²⁸) | **36.9** / **32.3** | – | 485 / 496 | 40.8 / 32.8 | 248 / 250 | 685 / 580 | 233 / 226 |
| GF(2²³³) | **77.9** / **58.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **76.3** / **64.8** | – | 1,427 / 1,383 | – | – | – | – |
| GF(2⁵⁷¹) | **200** / **190** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **138** / – | – | 311 / – | 271 / – | – | – |
| GF(2¹⁶) | **245** / – | 2,846 / – | 408 / – | – | – | – |
| GF(2⁶⁴) | 1,965 / – | 72,530 / – | **1,581** / – | 4,449 / – | – | – |
| GF(2¹²⁸) | 5,881 / – | – | **4,507** / – | – | 183,932 / – | 58,626 / – |
| GF(2²³³) | **19,822** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,813 / – | – | **13,064** / – | – | – | – |
| GF(2⁵⁷¹) | **131,306** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.9** / – | – | 179 / – | 11.2 / – | – | – | – |
| GF(2¹⁶) | **18.9** / – | 1,430 / – | 1,250 / – | – | – | – | – |
| GF(2⁶⁴) | **62.4** / – | 36,474 / – | 49,984 / – | 102 / – | – | – | – |
| GF(2¹²⁸) | **105** / – | – | 340,289 / – | 147 / – | 31,650 / – | 108,517 / – | 29,003 / – |
| GF(2²³³) | **143** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **163** / – | – | 2,238,453 / – | – | – | – | – |
| GF(2⁵⁷¹) | **2,158** / – | – | – | – | – | – | – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.1 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | 3.4 / **2.1** | – | – | – | – | 3.4 / 2.3 | 3.4 / 2.4 | 3.4 / 2.3 | **3.3** / 2.3 | 3.4 / 2.3 | 3.4 / 2.4 | – |
| GF(2²³³) | **6.6** / **4.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 6.6 / **4.8** | – | – | – | – | **6.2** / 4.9 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **17.7** / **15.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.9** | 6.3 / 18.0 | 81.4 / 79.2 | 2.9 / 11.8 | 2.7 / 11.3 | 21.0 / 29.7 | 19.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.2** / 30.8 | 126 / 127 | 222 / 218 | 10.0 / 34.9 | 8.8 / **23.4** | 82.7 / 94.9 | 19.0 / 32.9 | – | – | – | – | 10.3 / 43.5 |
| GF(2⁶⁴) | 232 / 236 | 443 / 426 | 607 / 610 | – | – | 896 / 898 | **95.5** / **100.0** | – | – | – | – | – |
| GF(2¹²⁸) | 739 / 727 | – | – | – | – | 2,959 / 2,949 | 297 / 292 | 286 / 282 | 644 / 581 | 644 / 581 | **258** / **264** | – |
| GF(2²³³) | **3,278** / **3,279** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,273** / **3,257** | – | – | – | – | 9,572 / 9,592 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15,980** / **15,655** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 4.8 / 16.0 | 73.1 / 68.1 | **2.2** / 7.9 | 2.4 / 8.0 | 9.6 / 19.9 | 11.1 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.7** / **6.3** | 125 / 125 | 210 / 207 | 6.2 / 12.2 | 8.1 / 20.6 | 25.0 / 33.9 | 3.1 / 8.3 | – | – | – | – | 9.3 / 28.0 |
| GF(2⁶⁴) | **16.3** / **23.0** | 438 / 419 | 608 / 608 | – | – | 194 / 205 | 49.6 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **31.6** / **32.5** | – | – | – | – | 568 / 568 | 43.5 / 44.2 | 284 / 304 | 611 / 612 | 620 / 612 | 254 / 263 | – |
| GF(2²³³) | **62.6** / **63.9** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **65.6** / **67.7** | – | – | – | – | 1,676 / 1,610 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **174** / **213** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 4.6 / – | 398 / – | 32.8 / – | 32.2 / – | 570 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.6** / – | 3,883 / – | 2,300 / – | 722 / – | 588 / – | 716 / – | 32.5 / – | – | – | – | – | 8.9 / – |
| GF(2⁶⁴) | 2,295 / – | 54,058 / – | 23,859 / – | – | – | **2,087** / – | 4,679 / – | – | – | – | – | – |
| GF(2¹²⁸) | 7,747 / – | – | – | – | – | 5,420 / – | **1,319** / – | 16,642 / – | 151,921 / – | 155,766 / – | 66,292 / – | – |
| GF(2²³³) | **47,422** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,181 / – | – | – | – | – | **16,001** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **300,939** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 32.6 / – | 514 / – | 14.2 / – | 14.2 / – | 173 / – | 11.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.4** / – | 1,893 / – | 2,355 / – | 271 / – | 276 / – | 1,295 / – | 3.7 / – | – | – | – | – | 487 / – |
| GF(2⁶⁴) | 202 / – | 26,890 / – | 38,545 / – | – | – | 56,517 / – | **83.9** / – | – | – | – | – | – |
| GF(2¹²⁸) | 487 / – | – | – | – | – | 370,193 / – | **164** / – | 37,906 / – | 77,795 / – | 81,630 / – | 33,394 / – | – |
| GF(2²³³) | **237** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **381** / – | – | – | – | – | 2,447,550 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8,711** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | 3.5 / 2.2 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | 3.4 / 2.3 | 3.4 / 2.4 | 3.4 / 2.3 | **3.3** / 2.3 | 3.4 / 2.3 | 3.4 / 2.4 | – |
| GF(2²³³) | **6.6** / **5.2** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 6.6 / 5.0 | – | – | – | – | **6.2** / **4.9** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **16.9** / **16.2** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.0** | – | 21.0 / 29.7 | 19.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **17.2** / **23.0** | 126 / 127 | 82.7 / 94.9 | – | – | – | – |
| GF(2⁶⁴) | 233 / 233 | 443 / 426 | 896 / 898 | **95.5** / **100.0** | – | – | – |
| GF(2¹²⁸) | 744 / 727 | – | 2,959 / 2,949 | 297 / 292 | 286 / 282 | 644 / 581 | **258** / **264** |
| GF(2²³³) | **3,294** / **3,281** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,223** / **3,245** | – | 9,572 / 9,592 | – | – | – | – |
| GF(2⁵⁷¹) | **15,770** / **15,585** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / **15.0** | – | 9.6 / 19.9 | 11.1 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 125 / 125 | 25.0 / 33.9 | – | – | – | – |
| GF(2⁶⁴) | **17.1** / **23.0** | 438 / 419 | 194 / 205 | 49.6 / 62.0 | – | – | – |
| GF(2¹²⁸) | **32.0** / **32.5** | – | 568 / 568 | 43.5 / 44.2 | 284 / 304 | 611 / 612 | 254 / 263 |
| GF(2²³³) | **65.6** / **63.9** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **69.2** / **67.8** | – | 1,676 / 1,610 | – | – | – | – |
| GF(2⁵⁷¹) | **268** / **214** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **132** / – | – | 570 / – | 277 / – | – | – |
| GF(2¹⁶) | **236** / – | 3,883 / – | 716 / – | – | – | – |
| GF(2⁶⁴) | 3,928 / – | 54,058 / – | **2,087** / – | 4,679 / – | – | – |
| GF(2¹²⁸) | 11,974 / – | – | **5,420** / – | – | 151,921 / – | 66,292 / – |
| GF(2²³³) | **47,391** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,127 / – | – | **16,001** / – | – | – | – |
| GF(2⁵⁷¹) | **300,920** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / – | – | 173 / – | 11.5 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,893 / – | 1,295 / – | – | – | – | – |
| GF(2⁶⁴) | 200 / – | 26,890 / – | 56,517 / – | **83.9** / – | – | – | – |
| GF(2¹²⁸) | 494 / – | – | 370,193 / – | **164** / – | 37,906 / – | 77,795 / – | 33,394 / – |
| GF(2²³³) | **236** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **381** / – | – | 2,447,550 / – | – | – | – | – |
| GF(2⁵⁷¹) | **8,669** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / 1.2 | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.2 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.2** | – | – | – | – | 3.4 / 2.4 | **3.3** / 2.4 | 3.4 / 2.4 | **3.3** / 2.4 | **3.3** / 2.3 | 3.4 / 2.4 | – |
| GF(2²³³) | **6.3** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.7** | – | – | – | – | **6.3** / **4.7** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **18.2** / **16.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.9** | 6.8 / 18.0 | 81.3 / 79.2 | 3.1 / 11.7 | 2.9 / 11.3 | 21.0 / 30.1 | 19.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.3** / 30.8 | 126 / 127 | 223 / 221 | 9.0 / 32.1 | 8.5 / **23.3** | 82.9 / 100 | 18.9 / 33.0 | – | – | – | – | 9.6 / 42.1 |
| GF(2⁶⁴) | 97.3 / **91.8** | 439 / 425 | 612 / 604 | – | – | 898 / 907 | **95.0** / 100 | – | – | – | – | – |
| GF(2¹²⁸) | 379 / 388 | – | – | – | – | 2,933 / 2,936 | 298 / 300 | 286 / 285 | 614 / 581 | 596 / 582 | **257** / **263** | – |
| GF(2²³³) | **1,848** / **1,842** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,179** / **1,205** | – | – | – | – | 9,499 / 9,506 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **5,827** / **5,370** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 5.6 / 14.9 | 72.8 / 64.0 | **2.2** / 7.9 | 2.4 / 7.9 | 9.6 / 19.4 | 11.1 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.7** / **6.3** | 125 / 127 | 216 / 208 | 6.7 / 12.2 | 7.7 / 19.9 | 25.0 / 35.9 | 3.2 / 8.4 | – | – | – | – | 9.9 / 28.0 |
| GF(2⁶⁴) | **16.5** / **23.0** | 438 / 419 | 605 / 608 | – | – | 194 / 205 | 49.5 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **32.0** / **32.3** | – | – | – | – | 567 / 570 | 33.9 / 34.0 | 282 / 292 | 620 / 612 | 622 / 613 | 256 / 265 | – |
| GF(2²³³) | **62.3** / **64.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **65.2** / **67.7** | – | – | – | – | 1,638 / 1,603 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **174** / **211** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 4.8 / – | 398 / – | 32.3 / – | 32.3 / – | 574 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.8** / – | 3,962 / – | 2,301 / – | 689 / – | 584 / – | 712 / – | 32.6 / – | – | – | – | – | 8.3 / – |
| GF(2⁶⁴) | **1,686** / – | 52,677 / – | 24,491 / – | – | – | 2,065 / – | 4,675 / – | – | – | – | – | – |
| GF(2¹²⁸) | 4,887 / – | – | – | – | – | 5,400 / – | **1,329** / – | 16,532 / – | 155,961 / – | 155,760 / – | 66,198 / – | – |
| GF(2²³³) | **32,708** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 33,613 / – | – | – | – | – | **15,825** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **192,209** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 32.5 / – | 525 / – | 14.2 / – | 14.4 / – | 173 / – | 10.8 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,911 / – | 2,363 / – | 268 / – | 276 / – | 1,346 / – | 3.4 / – | – | – | – | – | 454 / – |
| GF(2⁶⁴) | 88.4 / – | 26,896 / – | 38,728 / – | – | – | 56,864 / – | **83.9** / – | – | – | – | – | – |
| GF(2¹²⁸) | 194 / – | – | – | – | – | 370,601 / – | **164** / – | 37,004 / – | 81,530 / – | 81,534 / – | 33,453 / – | – |
| GF(2²³³) | **153** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **222** / – | – | – | – | – | 2,425,529 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **3,551** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / 1.2 | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | – | – | – | – | 2.2 / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.2 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.2** | – | – | – | – | 3.4 / 2.4 | **3.3** / 2.4 | 3.4 / 2.4 | **3.3** / 2.4 | **3.3** / 2.3 | 3.4 / 2.4 | – |
| GF(2²³³) | **6.3** / **4.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / 4.8 | – | – | – | – | **6.3** / **4.7** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **18.1** / **15.7** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.0** | – | 21.0 / 30.1 | 19.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **17.2** / **23.0** | 126 / 127 | 82.9 / 100 | – | – | – | – |
| GF(2⁶⁴) | **94.3** / **97.9** | 439 / 425 | 898 / 907 | 95.0 / 100 | – | – | – |
| GF(2¹²⁸) | 383 / 396 | – | 2,933 / 2,936 | 298 / 300 | 286 / 285 | 614 / 581 | **257** / **263** |
| GF(2²³³) | **1,834** / **1,829** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,167** / **1,194** | – | 9,499 / 9,506 | – | – | – | – |
| GF(2⁵⁷¹) | **5,440** / **5,237** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.7** / **15.0** | – | 9.6 / 19.4 | 11.1 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 125 / 127 | 25.0 / 35.9 | – | – | – | – |
| GF(2⁶⁴) | **16.6** / **22.9** | 438 / 419 | 194 / 205 | 49.5 / 62.0 | – | – | – |
| GF(2¹²⁸) | **31.7** / **32.5** | – | 567 / 570 | 33.9 / 34.0 | 282 / 292 | 620 / 612 | 256 / 265 |
| GF(2²³³) | **65.9** / **63.9** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **68.3** / **67.7** | – | 1,638 / 1,603 | – | – | – | – |
| GF(2⁵⁷¹) | **266** / **210** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **136** / – | – | 574 / – | 277 / – | – | – |
| GF(2¹⁶) | **233** / – | 3,962 / – | 712 / – | – | – | – |
| GF(2⁶⁴) | 2,745 / – | 52,677 / – | **2,065** / – | 4,675 / – | – | – |
| GF(2¹²⁸) | 8,187 / – | – | **5,400** / – | – | 155,961 / – | 66,198 / – |
| GF(2²³³) | **32,667** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 34,012 / – | – | **15,825** / – | – | – | – |
| GF(2⁵⁷¹) | **192,572** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / – | – | 173 / – | 10.8 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,911 / – | 1,346 / – | – | – | – | – |
| GF(2⁶⁴) | 88.9 / – | 26,896 / – | 56,864 / – | **83.9** / – | – | – | – |
| GF(2¹²⁸) | 193 / – | – | 370,601 / – | **164** / – | 37,004 / – | 81,530 / – | 33,453 / – |
| GF(2²³³) | **153** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **221** / – | – | 2,425,529 / – | – | – | – | – |
| GF(2⁵⁷¹) | **3,458** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
