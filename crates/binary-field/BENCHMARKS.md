# Benchmarks

`binary-field` at commit `d7a869e` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 39.52 before, 17.75 after; its SMT sibling, CPU 29, 12% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 16.17 before, 18.83 after; its SMT sibling, CPU 29, 36% busy |
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
| GF(2¹²⁸) | 1.4 / **1.1** | – | – | – | – | 1.6 / 1.5 | 1.4 / 1.3 | **1.3** / **1.1** | 1.4 / 1.3 | 1.4 / 1.3 | 1.6 / 1.3 | – |
| GF(2²³³) | **2.4** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.4** / **2.0** | – | – | – | – | 3.1 / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.4 / 78.7 | **2.0** / 11.3 | **2.0** / 11.3 | 13.4 / 23.0 | 14.9 / 27.7 | – | – | – | – | – |
| GF(2¹⁶) | 8.0 / **19.0** | 8.0 / 27.0 | 141 / 143 | 8.6 / 35.7 | **7.4** / 22.4 | 53.0 / 52.0 | 14.1 / 30.1 | – | – | – | – | 12.4 / 61.0 |
| GF(2⁶⁴) | **5.0** / **11.2** | 6.0 / 20.0 | 365 / 365 | – | – | 548 / 548 | 6.0 / 19.0 | – | – | – | – | – |
| GF(2¹²⁸) | **12.0** / **12.0** | – | – | – | – | 1,759 / 1,761 | **12.0** / 16.0 | **12.0** / 18.7 | 20.0 / 27.7 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **39.9** / **43.2** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | – | – | – | 5,730 / 5,721 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **167** / **163** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 58.7 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 7.0 / 27.8 | 139 / 126 | 6.5 / 11.1 | 6.4 / 17.1 | 23.0 / 30.0 | 2.0 / 8.0 | – | – | – | – | 10.2 / 24.0 |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.8 | 364 / 373 | – | – | 125 / 138 | 5.0 / 15.7 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.8** | – | – | – | – | 288 / 286 | 8.0 / 16.3 | 10.0 / 18.9 | 16.4 / 27.6 | 547 / 546 | 146 / 154 | – |
| GF(2²³³) | **20.7** / **32.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **17.5** | – | – | – | – | 1,022 / 1,083 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.4** / **41.9** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.2 / – | 309 / – | 28.9 / – | 29.4 / – | 254 / – | 307 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 420 / – | 998 / – | 796 / – | 522 / – | 324 / – | 20.4 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **210** / – | 1,224 / – | 8,510 / – | – | – | 1,221 / – | 1,065 / – | – | – | – | – | – |
| GF(2¹²⁸) | **503** / – | – | – | – | – | 3,201 / – | 594 / – | 5,956 / – | 5,046 / – | 147,714 / – | 39,276 / – | – |
| GF(2²³³) | **7,906** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,081** / – | – | – | – | – | 9,698 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,342** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.3 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 241 / – | 2,070 / – | 330 / – | 222 / – | 793 / – | 2.0 / – | – | – | – | – | 557 / – |
| GF(2⁶⁴) | **11.2** / – | 932 / – | 24,107 / – | – | – | 34,363 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | – | – | – | 223,758 / – | 17.7 / – | 2,453 / – | 3,569 / – | 69,369 / – | 19,877 / – | – |
| GF(2²³³) | **48.5** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | – | – | – | 1,462,400 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **134** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.2** / **1.0** | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.4 / **1.0** | – | – | – | – | 1.6 / 1.5 | 1.4 / 1.3 | **1.3** / 1.1 | 1.4 / 1.3 | 1.4 / 1.3 | 1.6 / 1.3 | – |
| GF(2²³³) | **2.2** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.0** | – | – | – | – | 3.1 / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.0** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / **17.3** | – | 13.4 / 23.0 | 14.9 / 27.7 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **4.9** / **11.2** | 6.0 / 20.0 | 548 / 548 | 6.0 / 19.0 | – | – | – |
| GF(2¹²⁸) | **12.0** / **12.0** | – | 1,759 / 1,761 | **12.0** / 16.0 | **12.0** / 18.7 | 20.0 / 27.7 | 146 / 155 |
| GF(2²³³) | **39.7** / **43.5** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | 5,730 / 5,721 | – | – | – | – |
| GF(2⁵⁷¹) | **170** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.7** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.1** | 6.0 / 20.8 | 125 / 138 | 5.0 / 15.7 | – | – | – |
| GF(2¹²⁸) | **6.7** / **13.3** | – | 288 / 286 | 8.0 / 16.3 | 10.0 / 18.9 | 16.4 / 27.6 | 146 / 154 |
| GF(2²³³) | **20.9** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **16.7** | – | 1,022 / 1,083 | – | – | – | – |
| GF(2⁵⁷¹) | **41.4** / **41.8** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.0** / – | – | 254 / – | 307 / – | – | – |
| GF(2¹⁶) | **168** / – | 420 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,224 / – | 1,221 / – | 1,065 / – | – | – |
| GF(2¹²⁸) | **1,938** / – | – | 3,201 / – | – | 5,046 / – | 39,276 / – |
| GF(2²³³) | **7,903** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,389** / – | – | 9,698 / – | – | – | – |
| GF(2⁵⁷¹) | **27,294** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 241 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 932 / – | 34,363 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | 223,758 / – | 17.7 / – | 2,453 / – | 3,569 / – | 19,877 / – |
| GF(2²³³) | **48.4** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | 1,462,400 / – | – | – | – | – |
| GF(2⁵⁷¹) | **133** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.4** / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | **1.4** / 1.3 | **1.4** / 1.3 | **1.4** / 1.3 | 1.7 / 1.4 | – |
| GF(2²³³) | **2.2** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.1** | – | – | – | – | 3.2 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.3 / 78.6 | **2.0** / 11.3 | **2.0** / 11.3 | 13.3 / 23.0 | 14.8 / 28.7 | – | – | – | – | – |
| GF(2¹⁶) | **5.4** / 32.1 | 31.4 / 54.3 | 141 / 143 | 8.5 / 34.8 | 7.4 / **22.5** | 53.0 / 52.0 | 14.2 / 30.4 | – | – | – | – | 12.5 / 61.8 |
| GF(2⁶⁴) | **31.9** / **41.5** | 101 / 117 | 365 / 368 | – | – | 548 / 548 | 46.8 / 68.7 | – | – | – | – | – |
| GF(2¹²⁸) | **55.4** / **64.8** | – | – | – | – | 1,757 / 1,761 | 157 / 186 | 172 / 176 | 550 / 546 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **202** / **206** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **191** / **185** | – | – | – | – | 5,723 / 5,723 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,292** / **1,285** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.5 / 58.6 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 32.0 / 56.4 | 138 / 127 | 6.5 / 11.6 | 6.5 / 17.1 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.1 / 24.0 |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.9 / 126 | 364 / 371 | – | – | 125 / 138 | 24.5 / 44.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.2** | – | – | – | – | 288 / 284 | 37.0 / 39.9 | 171 / 178 | 547 / 546 | 547 / 546 | 146 / 157 | – |
| GF(2²³³) | **20.8** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **27.0** | – | – | – | – | 1,021 / 1,082 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.1** / **57.1** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 309 / – | 29.2 / – | 29.5 / – | 254 / – | 307 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 1,152 / – | 994 / – | 773 / – | 518 / – | 324 / – | 20.4 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **530** / – | 13,918 / – | 8,534 / – | – | – | 1,221 / – | 3,249 / – | – | – | – | – | – |
| GF(2¹²⁸) | 1,067 / – | – | – | – | – | 3,211 / – | **973** / – | 11,302 / – | 139,433 / – | 147,718 / – | 39,275 / – | – |
| GF(2²³³) | **9,445** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,687** / – | – | – | – | – | 9,699 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **48,617** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 510 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 814 / – | 2,069 / – | 327 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 548 / – |
| GF(2⁶⁴) | **35.5** / – | 7,931 / – | 24,105 / – | – | – | 34,344 / – | 40.2 / – | – | – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | – | – | – | 223,163 / – | 123 / – | 23,053 / – | 69,370 / – | 69,369 / – | 19,893 / – | – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.8** / – | – | – | – | – | 1,455,916 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **685** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.2** / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | 1.4 / 1.3 | 1.4 / 1.3 | 1.4 / 1.3 | 1.7 / 1.4 | – |
| GF(2²³³) | **2.7** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.1** | – | – | – | – | 3.2 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.5** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 14.0 / **21.7** | – | **13.3** / 23.0 | 14.8 / 28.7 | – | – | – |
| GF(2¹⁶) | **15.0** / **22.5** | 31.4 / 54.3 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **31.7** / **41.3** | 101 / 117 | 548 / 548 | 46.8 / 68.7 | – | – | – |
| GF(2¹²⁸) | **54.9** / **64.8** | – | 1,757 / 1,761 | 157 / 186 | 172 / 176 | 550 / 546 | 146 / 155 |
| GF(2²³³) | **203** / **206** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **191** / **193** | – | 5,723 / 5,723 | – | – | – | – |
| GF(2⁵⁷¹) | **1,290** / **1,281** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.4** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.7** / **22.2** | 32.0 / 56.4 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.9 / 126 | 125 / 138 | 24.5 / 44.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.0** | – | 288 / 284 | 37.0 / 39.9 | 171 / 178 | 547 / 546 | 146 / 157 |
| GF(2²³³) | **20.6** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **26.5** | – | 1,021 / 1,082 | – | – | – | – |
| GF(2⁵⁷¹) | **55.1** / **57.0** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **104** / – | – | 254 / – | 307 / – | – | – |
| GF(2¹⁶) | **227** / – | 1,152 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **1,053** / – | 13,918 / – | 1,221 / – | 3,249 / – | – | – |
| GF(2¹²⁸) | **2,523** / – | – | 3,211 / – | – | 139,433 / – | 39,275 / – |
| GF(2²³³) | **9,362** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,670** / – | – | 9,699 / – | – | – | – |
| GF(2⁵⁷¹) | **48,672** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 814 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **35.6** / – | 7,931 / – | 34,344 / – | 40.2 / – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | 223,163 / – | 123 / – | 23,053 / – | 69,370 / – | 19,893 / – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.6** / – | – | 1,455,916 / – | – | – | – | – |
| GF(2⁵⁷¹) | **681** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file
```
