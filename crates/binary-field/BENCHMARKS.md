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

## WebAssembly

`binary-field` at commit `e939470` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 3,927 to 4,477 cycles and 3,081 instructions; its noise in a figure, 79.0 to 134 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,166 to 4,220 cycles and 3,081 instructions; its noise in a figure, 74.0 to 108 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,097 to 5,221 cycles and 1,885 instructions; its noise in a figure, 72.0 to 429 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,244 to 4,440 cycles and 1,885 instructions; its noise in a figure, 198 to 601 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 7.48 before, 12.14 after; its SMT sibling, CPU 28, 6% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 18.28 before, 5.83 after; its SMT sibling, CPU 28, 2% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 5.83 before, 3.58 after; its SMT sibling, CPU 28, 3% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 3.58 before, 6.47 after; its SMT sibling, CPU 28, 1% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add | GF(2²³³) | 7.4 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add | GF(2²⁵⁶) | 7.4 vs hekate-math 8.0 | −0.6 (0.93×) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) | 6.3 vs hekate-math 6.4 | −0.1 (0.98×) |
| add | GF(2⁵⁷¹) | 23.1 | – | 23.2 | – | 15.5 | – | 15.4 | – |
| mul | GF(2⁸) | 2.2 vs g2p 2.7 | −0.5 (0.81×) | 2.2 vs g2p 2.6 | −0.4 (0.85×) | 2.2 vs reed-solomon-erasure 2.6 | −0.4 (0.85×) | 2.2 vs g2p 2.6 | −0.4 (0.85×) |
| mul | GF(2¹⁶) | 5.8 vs reed-solomon-erasure 8.7 | −2.9 (0.67×) | 5.7 vs reed-solomon-erasure 8.7 | −3.0 (0.66×) | 6.2 vs reed-solomon-erasure 8.9 | −2.7 (0.70×) | 6.2 vs reed-solomon-erasure 9.0 | −2.8 (0.69×) |
| mul | GF(2⁶⁴) | 63.9 vs plonky3 92.6 | −28.7 (0.69×) | 63.6 vs plonky3 92.0 | −28.4 (0.69×) | 73.1 vs plonky3 95.1 | −22.0 (0.77×) | 72.0 vs plonky3 95.6 | −23.6 (0.75×) |
| mul | GF(2¹²⁸) | 184 vs polyval 227 | −43.0 (0.81×) | 184 vs polyval 227 | −43.0 (0.81×) | 200 vs polyval 258 | −58.0 (0.78×) | 200 vs polyval 258 | −58.0 (0.78×) |
| mul | GF(2²³³) | 754 | – | 723 | – | 1,089 | – | 1,070 | – |
| mul | GF(2²⁵⁶) | 706 vs hekate-math 8,420 | −7,714 (0.08×) | 705 vs hekate-math 8,413 | −7,708 (0.08×) | 1,073 vs hekate-math 9,581 | −8,508 (0.11×) | 1,068 vs hekate-math 9,564 | −8,496 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,011 | – | 3,988 | – | 4,979 | – | 4,910 | – |
| square | GF(2⁸) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) |
| square | GF(2¹⁶) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.3 vs plonky3 3.1 | −0.8 (0.74×) | 2.3 vs plonky3 3.1 | −0.8 (0.74×) |
| square | GF(2⁶⁴) | 15.8 vs plonky3 46.2 | −30.4 (0.34×) | 15.2 vs plonky3 46.2 | −31.0 (0.33×) | 17.0 vs plonky3 49.0 | −32.0 (0.35×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square | GF(2¹²⁸) | 28.0 vs plonky3 34.6 | −6.6 (0.81×) | 28.0 vs plonky3 30.7 | −2.7 (0.91×) | 31.0 vs plonky3 43.2 | −12.2 (0.72×) | 31.0 vs plonky3 33.5 | −2.5 (0.93×) |
| square | GF(2²³³) | 50.7 | – | 50.6 | – | 61.0 | – | 61.0 | – |
| square | GF(2²⁵⁶) | 58.1 vs hekate-math 1,383 | −1,325 (0.04×) | 57.9 vs hekate-math 1,383 | −1,325 (0.04×) | 62.0 vs hekate-math 1,533 | −1,471 (0.04×) | 62.0 vs hekate-math 1,665 | −1,603 (0.04×) |
| square | GF(2⁵⁷¹) | 145 | – | 145 | – | 152 | – | 151 | – |
| invert | GF(2⁸) | 2.2 vs gf256 3.6 | −1.4 (0.61×) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.2 vs gf256 4.0 | −1.8 (0.55×) | 2.2 vs gf256 4.0 | −1.8 (0.55×) |
| invert | GF(2¹⁶) | 2.2 vs galois_field_2pm lut 8.2 | −6.0 (0.27×) | 2.2 vs galois_field_2pm lut 8.2 | −6.0 (0.27×) | 2.3 vs galois_field_2pm lut 9.4 | −7.1 (0.24×) | 2.3 vs galois_field_2pm lut 9.5 | −7.2 (0.24×) |
| invert | GF(2⁶⁴) | 826 vs hekate-math 1,577 | −751 (0.52×) | 793 vs hekate-math 1,582 | −789 (0.50×) | 896 vs hekate-math 2,090 | −1,194 (0.43×) | 895 vs hekate-math 2,088 | −1,193 (0.43×) |
| invert | GF(2¹²⁸) | 2,512 vs plonky3 1,221 | +1,291 (2.06×) | 2,506 vs plonky3 1,253 | +1,253 (2.00×) | 2,675 vs plonky3 1,303 | +1,372 (2.05×) | 2,680 vs plonky3 1,319 | +1,361 (2.03×) |
| invert | GF(2²³³) | 20,793 | – | 20,878 | – | 24,297 | – | 23,914 | – |
| invert | GF(2²⁵⁶) | 22,975 vs hekate-math 13,703 | +9,272 (1.68×) | 22,988 vs hekate-math 13,698 | +9,290 (1.68×) | 29,628 vs hekate-math 15,994 | +13,634 (1.85×) | 29,266 vs hekate-math 15,926 | +13,340 (1.84×) |
| invert | GF(2⁵⁷¹) | 130,347 | – | 130,186 | – | 161,832 | – | 160,604 | – |
| sqrt | GF(2⁸) | 2.2 vs plonky3 10.9 | −8.7 (0.20×) | 2.2 vs plonky3 10.9 | −8.7 (0.20×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) |
| sqrt | GF(2¹⁶) | 2.2 vs plonky3 3.3 | −1.1 (0.67×) | 2.2 vs plonky3 3.3 | −1.1 (0.67×) | 2.2 vs plonky3 3.4 | −1.2 (0.65×) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) |
| sqrt | GF(2⁶⁴) | 37.2 vs plonky3 80.1 | −42.9 (0.46×) | 37.2 vs plonky3 80.0 | −42.8 (0.47×) | 40.0 vs plonky3 84.1 | −44.1 (0.48×) | 40.7 vs plonky3 84.1 | −43.4 (0.48×) |
| sqrt | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 154 vs plonky3 149 | +5.0 (1.03×) | 156 vs plonky3 163 | −7.0 (0.96×) | 156 vs plonky3 163 | −7.0 (0.96×) |
| sqrt | GF(2²³³) | 96.5 | – | 107 | – | 132 | – | 132 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,155,728 | −2,155,606 (0.00×) | 122 vs hekate-math 2,155,481 | −2,155,359 (0.00×) | 161 vs hekate-math 2,447,405 | −2,447,244 (0.00×) | 186 vs hekate-math 2,443,528 | −2,443,342 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,207 | – | 2,210 | – | 2,528 | – | 2,533 | – |
| add, constant time | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add, constant time | GF(2²³³) | 7.5 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add, constant time | GF(2²⁵⁶) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 7.4 vs hekate-math 8.0 | −0.6 (0.93×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) | 6.3 vs hekate-math 6.4 | −0.1 (0.98×) |
| add, constant time | GF(2⁵⁷¹) | 23.1 | – | 23.2 | – | 15.5 | – | 15.4 | – |
| mul, constant time | GF(2⁸) | 13.0 vs plonky3 19.2 | −6.2 (0.68×) | 13.0 vs plonky3 19.2 | −6.2 (0.68×) | 15.5 vs plonky3 19.7 | −4.2 (0.79×) | 15.5 vs plonky3 19.5 | −4.0 (0.79×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 76.0 | −59.5 (0.22×) | 16.5 vs hekate-math 76.0 | −59.5 (0.22×) | 16.7 vs hekate-math 83.4 | −66.7 (0.20×) | 16.8 vs hekate-math 83.3 | −66.5 (0.20×) |
| mul, constant time | GF(2⁶⁴) | 63.9 vs plonky3 92.6 | −28.7 (0.69×) | 61.9 vs plonky3 92.0 | −30.1 (0.67×) | 73.1 vs plonky3 95.1 | −22.0 (0.77×) | 72.0 vs plonky3 95.6 | −23.6 (0.75×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 227 | −43.0 (0.81×) | 184 vs polyval 227 | −43.0 (0.81×) | 200 vs polyval 258 | −58.0 (0.78×) | 200 vs polyval 258 | −58.0 (0.78×) |
| mul, constant time | GF(2²³³) | 794 | – | 787 | – | 1,096 | – | 1,070 | – |
| mul, constant time | GF(2²⁵⁶) | 704 vs hekate-math 8,420 | −7,716 (0.08×) | 704 vs hekate-math 8,413 | −7,709 (0.08×) | 1,116 vs hekate-math 9,581 | −8,465 (0.12×) | 1,073 vs hekate-math 9,564 | −8,491 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,063 | – | 4,066 | – | 4,875 | – | 4,860 | – |
| square, constant time | GF(2⁸) | 5.1 vs hekate-math 8.9 | −3.8 (0.57×) | 5.0 vs hekate-math 8.9 | −3.9 (0.56×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) |
| square, constant time | GF(2¹⁶) | 10.2 vs hekate-math 25.4 | −15.2 (0.40×) | 10.2 vs hekate-math 25.4 | −15.2 (0.40×) | 10.4 vs hekate-math 27.0 | −16.6 (0.39×) | 10.4 vs hekate-math 27.0 | −16.6 (0.39×) |
| square, constant time | GF(2⁶⁴) | 15.6 vs plonky3 46.2 | −30.6 (0.34×) | 16.0 vs plonky3 46.2 | −30.2 (0.35×) | 17.0 vs plonky3 49.0 | −32.0 (0.35×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square, constant time | GF(2¹²⁸) | 28.0 vs plonky3 34.6 | −6.6 (0.81×) | 28.0 vs plonky3 30.7 | −2.7 (0.91×) | 31.0 vs plonky3 43.2 | −12.2 (0.72×) | 31.0 vs plonky3 33.5 | −2.5 (0.93×) |
| square, constant time | GF(2²³³) | 53.1 | – | 53.0 | – | 60.2 | – | 60.2 | – |
| square, constant time | GF(2²⁵⁶) | 62.1 vs hekate-math 1,383 | −1,321 (0.04×) | 62.1 vs hekate-math 1,383 | −1,321 (0.04×) | 64.1 vs hekate-math 1,533 | −1,469 (0.04×) | 65.3 vs hekate-math 1,665 | −1,600 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 162 | – | 162 | – | 208 | – | 208 | – |
| invert, constant time | GF(2⁸) | 123 vs plonky3 272 | −149 (0.45×) | 123 vs plonky3 272 | −149 (0.45×) | 136 vs plonky3 277 | −141 (0.49×) | 136 vs plonky3 277 | −141 (0.49×) |
| invert, constant time | GF(2¹⁶) | 239 vs hekate-math 393 | −154 (0.61×) | 239 vs hekate-math 393 | −154 (0.61×) | 233 vs hekate-math 707 | −474 (0.33×) | 233 vs hekate-math 707 | −474 (0.33×) |
| invert, constant time | GF(2⁶⁴) | 1,302 vs hekate-math 1,577 | −275 (0.83×) | 1,317 vs hekate-math 1,582 | −265 (0.83×) | 1,310 vs hekate-math 2,090 | −780 (0.63×) | 1,312 vs hekate-math 2,088 | −776 (0.63×) |
| invert, constant time | GF(2¹²⁸) | 4,539 vs hekate-math 4,494 | +45.0 (1.01×) | 4,532 vs hekate-math 4,498 | +34.0 (1.01×) | 4,408 vs hekate-math 5,439 | −1,031 (0.81×) | 4,380 vs hekate-math 5,437 | −1,057 (0.81×) |
| invert, constant time | GF(2²³³) | 20,876 | – | 20,848 | – | 24,485 | – | 24,133 | – |
| invert, constant time | GF(2²⁵⁶) | 22,979 vs hekate-math 13,703 | +9,276 (1.68×) | 22,971 vs hekate-math 13,698 | +9,273 (1.68×) | 29,410 vs hekate-math 15,994 | +13,416 (1.84×) | 29,289 vs hekate-math 15,926 | +13,363 (1.84×) |
| invert, constant time | GF(2⁵⁷¹) | 130,358 | – | 130,084 | – | 161,689 | – | 160,724 | – |
| sqrt, constant time | GF(2⁸) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 6.3 vs plonky3 11.5 | −5.2 (0.55×) | 5.7 vs plonky3 11.5 | −5.8 (0.50×) |
| sqrt, constant time | GF(2¹⁶) | 16.8 vs hekate-math 1,243 | −1,226 (0.01×) | 16.8 vs hekate-math 1,242 | −1,225 (0.01×) | 20.0 vs hekate-math 1,355 | −1,335 (0.01×) | 20.0 vs hekate-math 1,356 | −1,336 (0.01×) |
| sqrt, constant time | GF(2⁶⁴) | 37.2 vs plonky3 80.1 | −42.9 (0.46×) | 37.1 vs plonky3 80.0 | −42.9 (0.46×) | 40.9 vs plonky3 84.1 | −43.2 (0.49×) | 40.9 vs plonky3 84.1 | −43.2 (0.49×) |
| sqrt, constant time | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 154 vs plonky3 149 | +5.0 (1.03×) | 159 vs plonky3 163 | −4.0 (0.98×) | 156 vs plonky3 163 | −7.0 (0.96×) |
| sqrt, constant time | GF(2²³³) | 111 | – | 113 | – | 132 | – | 133 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,155,728 | −2,155,606 (0.00×) | 122 vs hekate-math 2,155,481 | −2,155,359 (0.00×) | 149 vs hekate-math 2,447,405 | −2,447,256 (0.00×) | 181 vs hekate-math 2,443,528 | −2,443,347 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,241 | – | 2,210 | – | 2,527 | – | 2,549 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 2.3 vs plonky3 2.3 | +0.0 (1.00×) | 2.3 vs compile-algebra 2.3 | +0.0 (1.00×) | 2.1 vs plonky3 2.2 | −0.1 (0.95×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) |
| add | GF(2²³³) | 4.8 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add | GF(2⁵⁷¹) | 17.2 | – | 17.5 | – | 12.9 | – | 12.9 | – |
| mul | GF(2⁸) | 10.5 vs reed-solomon-erasure 11.6 | −1.1 (0.91×) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) |
| mul | GF(2¹⁶) | 30.1 vs reed-solomon-erasure 24.1 | +6.0 (1.25×) | 30.2 vs reed-solomon-erasure 23.9 | +6.3 (1.26×) | 30.9 vs reed-solomon-erasure 24.6 | +6.3 (1.26×) | 31.0 vs reed-solomon-erasure 24.0 | +7.0 (1.29×) |
| mul | GF(2⁶⁴) | 72.0 vs plonky3 77.7 | −5.7 (0.93×) | 72.0 vs plonky3 76.1 | −4.1 (0.95×) | 73.0 vs plonky3 100 | −27.0 (0.73×) | 73.1 vs plonky3 100 | −26.9 (0.73×) |
| mul | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 182 vs polyval 229 | −47.0 (0.79×) | 197 vs polyval 263 | −66.0 (0.75×) | 197 vs polyval 263 | −66.0 (0.75×) |
| mul | GF(2²³³) | 759 | – | 721 | – | 1,067 | – | 1,074 | – |
| mul | GF(2²⁵⁶) | 705 vs hekate-math 8,431 | −7,726 (0.08×) | 707 vs hekate-math 8,427 | −7,720 (0.08×) | 1,049 vs hekate-math 9,582 | −8,533 (0.11×) | 1,058 vs hekate-math 9,570 | −8,512 (0.11×) |
| mul | GF(2⁵⁷¹) | 3,978 | – | 4,018 | – | 4,724 | – | 4,759 | – |
| square | GF(2⁸) | 5.1 vs reed-solomon-erasure 7.7 | −2.6 (0.66×) | 5.1 vs g2p 7.6 | −2.5 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs plonky3 8.4 | −2.1 (0.75×) | 6.3 vs plonky3 8.4 | −2.1 (0.75×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 34.3 | −7.3 (0.79×) | 27.0 vs plonky3 32.6 | −5.6 (0.83×) | 30.1 vs plonky3 43.7 | −13.6 (0.69×) | 30.0 vs plonky3 34.0 | −4.0 (0.88×) |
| square | GF(2²³³) | 54.0 | – | 53.9 | – | 59.0 | – | 59.0 | – |
| square | GF(2²⁵⁶) | 59.0 vs hekate-math 1,380 | −1,321 (0.04×) | 59.0 vs hekate-math 1,379 | −1,320 (0.04×) | 63.0 vs hekate-math 1,450 | −1,387 (0.04×) | 63.0 vs hekate-math 1,506 | −1,443 (0.04×) |
| square | GF(2⁵⁷¹) | 161 | – | 165 | – | 234 | – | 235 | – |
| add, constant time | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs plonky3 2.3 | +0.0 (1.00×) | 2.3 vs compile-algebra 2.3 | +0.0 (1.00×) | 2.1 vs plonky3 2.2 | −0.1 (0.95×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) |
| add, constant time | GF(2²³³) | 4.8 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add, constant time | GF(2²⁵⁶) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add, constant time | GF(2⁵⁷¹) | 17.7 | – | 17.5 | – | 12.9 | – | 12.9 | – |
| mul, constant time | GF(2⁸) | 21.3 vs plonky3 23.0 | −1.7 (0.93×) | 21.3 vs plonky3 23.0 | −1.7 (0.93×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) |
| mul, constant time | GF(2¹⁶) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 22.0 vs hekate-math 81.9 | −59.9 (0.27×) | 23.0 vs hekate-math 98.7 | −75.7 (0.23×) | 23.0 vs hekate-math 101 | −78.0 (0.23×) |
| mul, constant time | GF(2⁶⁴) | 72.0 vs plonky3 77.7 | −5.7 (0.93×) | 72.0 vs plonky3 76.1 | −4.1 (0.95×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.1 vs plonky3 100 | −26.9 (0.73×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 182 vs polyval 229 | −47.0 (0.79×) | 197 vs polyval 263 | −66.0 (0.75×) | 197 vs polyval 263 | −66.0 (0.75×) |
| mul, constant time | GF(2²³³) | 758 | – | 759 | – | 1,075 | – | 1,073 | – |
| mul, constant time | GF(2²⁵⁶) | 704 vs hekate-math 8,431 | −7,727 (0.08×) | 706 vs hekate-math 8,427 | −7,721 (0.08×) | 1,051 vs hekate-math 9,582 | −8,531 (0.11×) | 1,057 vs hekate-math 9,570 | −8,513 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 3,984 | – | 4,002 | – | 4,717 | – | 4,773 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.1 | −4.1 (0.79×) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.3 vs plonky3 34.3 | −7.0 (0.80×) | 27.3 vs plonky3 32.6 | −5.3 (0.84×) | 31.0 vs plonky3 43.7 | −12.7 (0.71×) | 31.0 vs plonky3 34.0 | −3.0 (0.91×) |
| square, constant time | GF(2²³³) | 54.1 | – | 54.0 | – | 59.0 | – | 59.8 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,380 | −1,321 (0.04×) | 59.0 vs hekate-math 1,379 | −1,320 (0.04×) | 63.0 vs hekate-math 1,450 | −1,387 (0.04×) | 63.0 vs hekate-math 1,506 | −1,443 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 164 | – | 165 | – | 246 | – | 246 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add | GF(2¹²⁸) | 1.4 | 3.8 (2.7×) | 3.8 (2.7×) | 3.3 (2.4×) | 3.3 (2.4×) |
| add | GF(2²³³) | 2.2 | 7.4 (3.4×) | 7.5 (3.4×) | 6.3 (2.9×) | 6.3 (2.9×) |
| add | GF(2²⁵⁶) | 2.6 | 7.4 (2.8×) | 7.5 (2.9×) | 6.3 (2.4×) | 6.3 (2.4×) |
| add | GF(2⁵⁷¹) | 6.7 | 23.1 (3.4×) | 23.2 (3.5×) | 15.5 (2.3×) | 15.4 (2.3×) |
| mul | GF(2⁸) | 2.0 | 2.2 (1.1×) | 2.2 (1.1×) | 2.2 (1.1×) | 2.2 (1.1×) |
| mul | GF(2¹⁶) | 5.4 | 5.8 (1.1×) | 5.7 (1.1×) | 6.2 (1.1×) | 6.2 (1.1×) |
| mul | GF(2⁶⁴) | 31.9 | 63.9 (2.0×) | 63.6 (2.0×) | 73.1 (2.3×) | 72.0 (2.3×) |
| mul | GF(2¹²⁸) | 55.4 | 184 (3.3×) | 184 (3.3×) | 200 (3.6×) | 200 (3.6×) |
| mul | GF(2²³³) | 202 | 754 (3.7×) | 723 (3.6×) | 1,089 (5.4×) | 1,070 (5.3×) |
| mul | GF(2²⁵⁶) | 191 | 706 (3.7×) | 705 (3.7×) | 1,073 (5.6×) | 1,068 (5.6×) |
| mul | GF(2⁵⁷¹) | 1,292 | 4,011 (3.1×) | 3,988 (3.1×) | 4,979 (3.9×) | 4,910 (3.8×) |
| square | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| square | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) | 2.3 (1.4×) |
| square | GF(2⁶⁴) | 5.4 | 15.8 (2.9×) | 15.2 (2.8×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square | GF(2¹²⁸) | 6.7 | 28.0 (4.2×) | 28.0 (4.2×) | 31.0 (4.6×) | 31.0 (4.6×) |
| square | GF(2²³³) | 20.8 | 50.7 (2.4×) | 50.6 (2.4×) | 61.0 (2.9×) | 61.0 (2.9×) |
| square | GF(2²⁵⁶) | 20.4 | 58.1 (2.8×) | 57.9 (2.8×) | 62.0 (3.0×) | 62.0 (3.0×) |
| square | GF(2⁵⁷¹) | 55.1 | 145 (2.6×) | 145 (2.6×) | 152 (2.8×) | 151 (2.7×) |
| invert | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| invert | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) | 2.3 (1.4×) |
| invert | GF(2⁶⁴) | 530 | 826 (1.6×) | 793 (1.5×) | 896 (1.7×) | 895 (1.7×) |
| invert | GF(2¹²⁸) | 1,067 | 2,512 (2.4×) | 2,506 (2.3×) | 2,675 (2.5×) | 2,680 (2.5×) |
| invert | GF(2²³³) | 9,445 | 20,793 (2.2×) | 20,878 (2.2×) | 24,297 (2.6×) | 23,914 (2.5×) |
| invert | GF(2²⁵⁶) | 8,687 | 22,975 (2.6×) | 22,988 (2.6×) | 29,628 (3.4×) | 29,266 (3.4×) |
| invert | GF(2⁵⁷¹) | 48,617 | 130,347 (2.7×) | 130,186 (2.7×) | 161,832 (3.3×) | 160,604 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| sqrt | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) |
| sqrt | GF(2⁶⁴) | 35.5 | 37.2 (1.0×) | 37.2 (1.0×) | 40.0 (1.1×) | 40.7 (1.1×) |
| sqrt | GF(2¹²⁸) | 65.6 | 154 (2.3×) | 154 (2.3×) | 156 (2.4×) | 156 (2.4×) |
| sqrt | GF(2²³³) | 111 | 96.5 (0.9×) | 107 (1.0×) | 132 (1.2×) | 132 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.8 | 122 (1.3×) | 122 (1.3×) | 161 (1.7×) | 186 (2.0×) |
| sqrt | GF(2⁵⁷¹) | 685 | 2,207 (3.2×) | 2,210 (3.2×) | 2,528 (3.7×) | 2,533 (3.7×) |
| add, constant time | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add, constant time | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add, constant time | GF(2¹²⁸) | 1.2 | 3.8 (3.2×) | 3.8 (3.2×) | 3.3 (2.8×) | 3.3 (2.8×) |
| add, constant time | GF(2²³³) | 2.7 | 7.5 (2.8×) | 7.5 (2.8×) | 6.3 (2.3×) | 6.3 (2.3×) |
| add, constant time | GF(2²⁵⁶) | 2.6 | 7.5 (2.9×) | 7.4 (2.8×) | 6.3 (2.4×) | 6.3 (2.4×) |
| add, constant time | GF(2⁵⁷¹) | 7.5 | 23.1 (3.1×) | 23.2 (3.1×) | 15.5 (2.1×) | 15.4 (2.1×) |
| mul, constant time | GF(2⁸) | 14.0 | 13.0 (0.9×) | 13.0 (0.9×) | 15.5 (1.1×) | 15.5 (1.1×) |
| mul, constant time | GF(2¹⁶) | 15.0 | 16.5 (1.1×) | 16.5 (1.1×) | 16.7 (1.1×) | 16.8 (1.1×) |
| mul, constant time | GF(2⁶⁴) | 31.7 | 63.9 (2.0×) | 61.9 (2.0×) | 73.1 (2.3×) | 72.0 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.9 | 184 (3.4×) | 184 (3.4×) | 200 (3.6×) | 200 (3.6×) |
| mul, constant time | GF(2²³³) | 203 | 794 (3.9×) | 787 (3.9×) | 1,096 (5.4×) | 1,070 (5.3×) |
| mul, constant time | GF(2²⁵⁶) | 191 | 704 (3.7×) | 704 (3.7×) | 1,116 (5.8×) | 1,073 (5.6×) |
| mul, constant time | GF(2⁵⁷¹) | 1,290 | 4,063 (3.1×) | 4,066 (3.2×) | 4,875 (3.8×) | 4,860 (3.8×) |
| square, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.0 (1.4×) | 6.3 (1.8×) | 6.3 (1.8×) |
| square, constant time | GF(2¹⁶) | 8.7 | 10.2 (1.2×) | 10.2 (1.2×) | 10.4 (1.2×) | 10.4 (1.2×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 15.6 (2.9×) | 16.0 (3.0×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.0 (4.2×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 20.6 | 53.1 (2.6×) | 53.0 (2.6×) | 60.2 (2.9×) | 60.2 (2.9×) |
| square, constant time | GF(2²⁵⁶) | 20.4 | 62.1 (3.0×) | 62.1 (3.0×) | 64.1 (3.1×) | 65.3 (3.2×) |
| square, constant time | GF(2⁵⁷¹) | 55.1 | 162 (2.9×) | 162 (2.9×) | 208 (3.8×) | 208 (3.8×) |
| invert, constant time | GF(2⁸) | 104 | 123 (1.2×) | 123 (1.2×) | 136 (1.3×) | 136 (1.3×) |
| invert, constant time | GF(2¹⁶) | 227 | 239 (1.1×) | 239 (1.1×) | 233 (1.0×) | 233 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 1,053 | 1,302 (1.2×) | 1,317 (1.3×) | 1,310 (1.2×) | 1,312 (1.2×) |
| invert, constant time | GF(2¹²⁸) | 2,523 | 4,539 (1.8×) | 4,532 (1.8×) | 4,408 (1.7×) | 4,380 (1.7×) |
| invert, constant time | GF(2²³³) | 9,362 | 20,876 (2.2×) | 20,848 (2.2×) | 24,485 (2.6×) | 24,133 (2.6×) |
| invert, constant time | GF(2²⁵⁶) | 8,670 | 22,979 (2.7×) | 22,971 (2.6×) | 29,410 (3.4×) | 29,289 (3.4×) |
| invert, constant time | GF(2⁵⁷¹) | 48,672 | 130,358 (2.7×) | 130,084 (2.7×) | 161,689 (3.3×) | 160,724 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 5.7 (1.6×) |
| sqrt, constant time | GF(2¹⁶) | 13.5 | 16.8 (1.2×) | 16.8 (1.2×) | 20.0 (1.5×) | 20.0 (1.5×) |
| sqrt, constant time | GF(2⁶⁴) | 35.6 | 37.2 (1.0×) | 37.1 (1.0×) | 40.9 (1.1×) | 40.9 (1.1×) |
| sqrt, constant time | GF(2¹²⁸) | 65.6 | 154 (2.3×) | 154 (2.3×) | 159 (2.4×) | 156 (2.4×) |
| sqrt, constant time | GF(2²³³) | 111 | 111 (1.0×) | 113 (1.0×) | 132 (1.2×) | 133 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.6 | 122 (1.3×) | 122 (1.3×) | 149 (1.6×) | 181 (1.9×) |
| sqrt, constant time | GF(2⁵⁷¹) | 681 | 2,241 (3.3×) | 2,210 (3.2×) | 2,527 (3.7×) | 2,549 (3.7×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.1 (1.9×) | 2.1 (1.9×) |
| add | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.1 | 4.8 (2.3×) | 4.8 (2.3×) | 4.6 (2.2×) | 4.6 (2.2×) |
| add | GF(2⁵⁷¹) | 5.1 | 17.2 (3.4×) | 17.5 (3.4×) | 12.9 (2.5×) | 12.9 (2.5×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.5 (1.1×) | 10.9 (1.2×) | 10.9 (1.2×) |
| mul | GF(2¹⁶) | 32.1 | 30.1 (0.9×) | 30.2 (0.9×) | 30.9 (1.0×) | 31.0 (1.0×) |
| mul | GF(2⁶⁴) | 41.5 | 72.0 (1.7×) | 72.0 (1.7×) | 73.0 (1.8×) | 73.1 (1.8×) |
| mul | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 182 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul | GF(2²³³) | 206 | 759 (3.7×) | 721 (3.5×) | 1,067 (5.2×) | 1,074 (5.2×) |
| mul | GF(2²⁵⁶) | 185 | 705 (3.8×) | 707 (3.8×) | 1,049 (5.7×) | 1,058 (5.7×) |
| mul | GF(2⁵⁷¹) | 1,285 | 3,978 (3.1×) | 4,018 (3.1×) | 4,724 (3.7×) | 4,759 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 27.0 (1.9×) | 30.1 (2.1×) | 30.0 (2.1×) |
| square | GF(2²³³) | 32.0 | 54.0 (1.7×) | 53.9 (1.7×) | 59.0 (1.8×) | 59.0 (1.8×) |
| square | GF(2²⁵⁶) | 27.0 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square | GF(2⁵⁷¹) | 57.1 | 161 (2.8×) | 165 (2.9×) | 234 (4.1×) | 235 (4.1×) |
| add, constant time | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.1 (1.9×) | 2.1 (1.9×) |
| add, constant time | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add, constant time | GF(2²⁵⁶) | 2.1 | 4.8 (2.3×) | 4.8 (2.3×) | 4.6 (2.2×) | 4.6 (2.2×) |
| add, constant time | GF(2⁵⁷¹) | 4.6 | 17.7 (3.8×) | 17.5 (3.8×) | 12.9 (2.8×) | 12.9 (2.8×) |
| mul, constant time | GF(2⁸) | 21.7 | 21.3 (1.0×) | 21.3 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 22.0 (1.0×) | 22.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.3 | 72.0 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.1 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 182 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 206 | 758 (3.7×) | 759 (3.7×) | 1,075 (5.2×) | 1,073 (5.2×) |
| mul, constant time | GF(2²⁵⁶) | 193 | 704 (3.6×) | 706 (3.7×) | 1,051 (5.4×) | 1,057 (5.5×) |
| mul, constant time | GF(2⁵⁷¹) | 1,281 | 3,984 (3.1×) | 4,002 (3.1×) | 4,717 (3.7×) | 4,773 (3.7×) |
| square, constant time | GF(2⁸) | 10.4 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.0 | 27.3 (1.9×) | 27.3 (1.9×) | 31.0 (2.2×) | 31.0 (2.2×) |
| square, constant time | GF(2²³³) | 32.0 | 54.1 (1.7×) | 54.0 (1.7×) | 59.0 (1.8×) | 59.8 (1.9×) |
| square, constant time | GF(2²⁵⁶) | 26.5 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.4×) | 63.0 (2.4×) |
| square, constant time | GF(2⁵⁷¹) | 57.0 | 164 (2.9×) | 165 (2.9×) | 246 (4.3×) | 246 (4.3×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
