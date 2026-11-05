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

`binary-field` at commit `d7a869e` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-04.

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
| Cost of a count, in WebAssembly, Chrome | 5,367 to 10,538 cycles and 3,081 instructions; its noise in a figure, 162 to 326 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,213 to 4,254 cycles and 3,081 instructions; its noise in a figure, 63.0 to 135 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,495 to 7,370 cycles and 1,885 instructions; its noise in a figure, 420 to 603 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,087 to 4,128 cycles and 1,885 instructions; its noise in a figure, 164 to 400 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 15.86 before, 14.82 after; its SMT sibling, CPU 28, 20% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 15.68 before, 12.74 after; its SMT sibling, CPU 28, 3% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 12.74 before, 5.77 after; its SMT sibling, CPU 28, 7% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 5.77 before, 2.84 after; its SMT sibling, CPU 28, 0% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | 3.8 / 2.3 | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.4** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27.3** / **20.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.6** | 4.8 / 18.0 | 77.9 / 77.0 | 2.7 / 11.8 | 2.7 / 11.8 | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **5.8** / 30.1 | 94.0 / 93.0 | 172 / 166 | 9.8 / 33.4 | 9.0 / **24.0** | 76.1 / 82.1 | 19.1 / 33.2 | – | – | – | – | 9.3 / 42.4 |
| GF(2⁶⁴) | 203 / 217 | 585 / 584 | 662 / 669 | – | – | 806 / 806 | **95.2** / **75.6** | – | – | – | – | – |
| GF(2¹²⁸) | 629 / 637 | – | – | – | – | 2,615 / 2,621 | 243 / 250 | 250 / 249 | 581 / 580 | 582 / 580 | **229** / **228** | – |
| GF(2²³³) | **3,422** / **3,316** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,511** / **3,576** | – | – | – | – | 8,415 / 8,433 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15,825** / **18,419** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **5.1** | 3.2 / 14.4 | 70.8 / 57.7 | 2.2 / 7.1 | 2.2 / 7.7 | 8.9 / 19.5 | 10.1 / 25.4 | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **6.1** | 92.6 / 96.3 | 164 / 119 | 7.0 / 12.1 | 7.7 / 17.7 | 25.4 / 34.0 | 3.0 / 8.3 | – | – | – | – | 8.1 / 27.0 |
| GF(2⁶⁴) | **15.5** / **25.7** | 585 / 583 | 661 / 665 | – | – | 180 / 195 | 46.3 / 59.0 | – | – | – | – | – |
| GF(2¹²⁸) | **27.9** / **31.0** | – | – | – | – | 506 / 493 | 34.6 / 36.2 | 251 / 250 | 581 / 579 | 581 / 579 | 227 / 228 | – |
| GF(2²³³) | **55.0** / **57.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **58.7** / **62.9** | – | – | – | – | 1,391 / 1,383 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **156** / **188** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 3.6 / – | 318 / – | 36.1 / – | 36.6 / – | 307 / – | 272 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 2,743 / – | 1,332 / – | 720 / – | 638 / – | 393 / – | 32.1 / – | – | – | – | – | 8.2 / – |
| GF(2⁶⁴) | 2,261 / – | 73,500 / – | 17,856 / – | – | – | **1,584** / – | 4,451 / – | – | – | – | – | – |
| GF(2¹²⁸) | 6,917 / – | – | – | – | – | 4,492 / – | **1,224** / – | 16,150 / – | 182,800 / – | 180,807 / – | 57,736 / – | – |
| GF(2²³³) | **48,438** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,666 / – | – | – | – | – | **13,729** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **305,677** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 30.8 / – | 488 / – | 15.0 / – | 15.0 / – | 174 / – | 10.9 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 1,447 / – | 2,309 / – | 308 / – | 292 / – | 1,242 / – | 3.4 / – | – | – | – | – | 446 / – |
| GF(2⁶⁴) | 123 / – | 36,601 / – | 42,467 / – | – | – | 51,006 / – | **80.0** / – | – | – | – | – | – |
| GF(2¹²⁸) | 381 / – | – | – | – | – | 332,771 / – | **149** / – | 31,539 / – | 107,944 / – | 107,807 / – | 28,980 / – | – |
| GF(2²³³) | **166** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **212** / – | – | – | – | – | 2,154,988 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **9,335** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | 3.8 / 2.3 | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.5** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27.1** / **20.6** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **13.0** / **21.4** | – | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.5** / **22.0** | 94.0 / 93.0 | 76.1 / 82.1 | – | – | – | – |
| GF(2⁶⁴) | 203 / 217 | 585 / 584 | 806 / 806 | **95.2** / **75.6** | – | – | – |
| GF(2¹²⁸) | 628 / 627 | – | 2,615 / 2,621 | 243 / 250 | 250 / 249 | 581 / 580 | **229** / **228** |
| GF(2²³³) | **3,461** / **3,424** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,457** / **3,553** | – | 8,415 / 8,433 | – | – | – | – |
| GF(2⁵⁷¹) | **15,809** / **18,454** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / **15.0** | – | 8.9 / 19.5 | 10.1 / 25.4 | – | – | – |
| GF(2¹⁶) | **10.2** / **23.0** | 92.6 / 96.3 | 25.4 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **15.4** / **25.8** | 585 / 583 | 180 / 195 | 46.3 / 59.0 | – | – | – |
| GF(2¹²⁸) | **27.8** / **30.9** | – | 506 / 493 | 34.6 / 36.2 | 251 / 250 | 581 / 579 | 227 / 228 |
| GF(2²³³) | **57.1** / **57.5** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **60.6** / **62.9** | – | 1,391 / 1,383 | – | – | – | – |
| GF(2⁵⁷¹) | **179** / **186** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **123** / – | – | 307 / – | 272 / – | – | – |
| GF(2¹⁶) | **239** / – | 2,743 / – | 393 / – | – | – | – |
| GF(2⁶⁴) | 3,612 / – | 73,500 / – | **1,584** / – | 4,451 / – | – | – |
| GF(2¹²⁸) | 10,476 / – | – | **4,492** / – | – | 182,800 / – | 57,736 / – |
| GF(2²³³) | **48,562** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,820 / – | – | **13,729** / – | – | – | – |
| GF(2⁵⁷¹) | **306,057** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | 174 / – | 10.9 / – | – | – | – |
| GF(2¹⁶) | **16.8** / – | 1,447 / – | 1,242 / – | – | – | – | – |
| GF(2⁶⁴) | 118 / – | 36,601 / – | 51,006 / – | **80.0** / – | – | – | – |
| GF(2¹²⁸) | 385 / – | – | 332,771 / – | **149** / – | 31,539 / – | 107,944 / – | 28,980 / – |
| GF(2²³³) | **166** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **208** / – | – | 2,154,988 / – | – | – | – | – |
| GF(2⁵⁷¹) | **9,287** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | 3.8 / 2.3 | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.4** / **4.8** | – | – | – | – | 7.9 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28.2** / **20.3** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.5** | 4.8 / 18.0 | 77.9 / 77.1 | 2.6 / 11.6 | 2.7 / 11.7 | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **5.8** / 30.1 | 90.1 / 92.8 | 173 / 166 | 9.9 / 32.9 | 8.6 / **23.8** | 76.0 / 82.0 | 19.0 / 33.3 | – | – | – | – | 9.4 / 42.5 |
| GF(2⁶⁴) | **62.6** / **51.4** | 585 / 585 | 661 / 670 | – | – | 806 / 806 | 92.0 / 65.1 | – | – | – | – | – |
| GF(2¹²⁸) | **162** / **154** | – | – | – | – | 2,619 / 2,616 | 243 / 250 | 250 / 250 | 581 / 580 | 581 / 580 | 228 / 228 | – |
| GF(2²³³) | **700** / **653** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **698** / **681** | – | – | – | – | 8,424 / 8,431 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **3,336** / **3,382** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **5.1** | 3.1 / 14.6 | 70.6 / 57.4 | 2.2 / 7.7 | 2.2 / 7.6 | 8.8 / 19.3 | 10.1 / 25.5 | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **6.1** | 91.5 / 98.0 | 164 / 152 | 6.9 / 12.0 | 7.8 / 17.2 | 25.4 / 34.0 | 3.0 / 8.3 | – | – | – | – | 8.2 / 27.0 |
| GF(2⁶⁴) | **15.6** / **26.0** | 585 / 583 | 661 / 670 | – | – | 179 / 197 | 46.2 / 59.0 | – | – | – | – | – |
| GF(2¹²⁸) | **27.8** / **30.8** | – | – | – | – | 500 / 488 | 30.6 / 32.4 | 251 / 241 | 581 / 579 | 581 / 579 | 227 / 228 | – |
| GF(2²³³) | **54.7** / **58.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **59.2** / **62.5** | – | – | – | – | 1,383 / 1,385 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **156** / **186** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 3.6 / – | 321 / – | 36.0 / – | 36.5 / – | 307 / – | 272 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 2,738 / – | 1,365 / – | 711 / – | 638 / – | 393 / – | 32.0 / – | – | – | – | – | 8.2 / – |
| GF(2⁶⁴) | **631** / – | 73,495 / – | 17,744 / – | – | – | 1,589 / – | 4,451 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,360 / – | – | – | – | – | 4,489 / – | **1,252** / – | 16,094 / – | 182,984 / – | 180,849 / – | 57,823 / – | – |
| GF(2²³³) | **19,745** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,806 / – | – | – | – | – | **13,703** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **129,161** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 30.8 / – | 490 / – | 15.0 / – | 14.9 / – | 174 / – | 10.9 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,445 / – | 2,307 / – | 306 / – | 291 / – | 1,242 / – | 3.3 / – | – | – | – | – | 445 / – |
| GF(2⁶⁴) | **60.0** / – | 36,593 / – | 42,336 / – | – | – | 51,586 / – | 80.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **111** / – | – | – | – | – | 332,034 / – | 149 / – | 31,523 / – | 108,941 / – | 107,898 / – | 28,970 / – | – |
| GF(2²³³) | **128** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **134** / – | – | – | – | – | 2,156,189 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,957** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | 3.8 / 2.3 | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.5** / **4.8** | – | – | – | – | 7.9 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28.2** / **20.4** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **13.0** / **21.3** | – | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.5** / **22.0** | 90.1 / 92.8 | 76.0 / 82.0 | – | – | – | – |
| GF(2⁶⁴) | **61.7** / **53.3** | 585 / 585 | 806 / 806 | 92.0 / 65.1 | – | – | – |
| GF(2¹²⁸) | **162** / **153** | – | 2,619 / 2,616 | 243 / 250 | 250 / 250 | 581 / 580 | 228 / 228 |
| GF(2²³³) | **653** / **691** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **690** / **678** | – | 8,424 / 8,431 | – | – | – | – |
| GF(2⁵⁷¹) | **3,358** / **3,374** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / **15.0** | – | 8.8 / 19.3 | 10.1 / 25.5 | – | – | – |
| GF(2¹⁶) | **10.2** / **23.0** | 91.5 / 98.0 | 25.4 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **15.4** / **25.9** | 585 / 583 | 179 / 197 | 46.2 / 59.0 | – | – | – |
| GF(2¹²⁸) | **28.0** / **30.7** | – | 500 / 488 | 30.6 / 32.4 | 251 / 241 | 581 / 579 | 227 / 228 |
| GF(2²³³) | **57.4** / **57.5** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **60.6** / **62.5** | – | 1,383 / 1,385 | – | – | – | – |
| GF(2⁵⁷¹) | **179** / **186** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **123** / – | – | 307 / – | 272 / – | – | – |
| GF(2¹⁶) | **239** / – | 2,738 / – | 393 / – | – | – | – |
| GF(2⁶⁴) | 1,946 / – | 73,495 / – | **1,589** / – | 4,451 / – | – | – |
| GF(2¹²⁸) | 5,742 / – | – | **4,489** / – | – | 182,984 / – | 57,823 / – |
| GF(2²³³) | **19,682** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,755 / – | – | **13,703** / – | – | – | – |
| GF(2⁵⁷¹) | **128,932** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | 174 / – | 10.9 / – | – | – | – |
| GF(2¹⁶) | **16.8** / – | 1,445 / – | 1,242 / – | – | – | – | – |
| GF(2⁶⁴) | **59.9** / – | 36,593 / – | 51,586 / – | 80.0 / – | – | – | – |
| GF(2¹²⁸) | **108** / – | – | 332,034 / – | 149 / – | 31,523 / – | 108,941 / – | 28,970 / – |
| GF(2²³³) | **126** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **136** / – | – | 2,156,189 / – | – | – | – | – |
| GF(2⁵⁷¹) | **1,970** / – | – | – | – | – | – | – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | – | – | – | – | 2.2 / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.1 | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.4 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.1** / 4.8 | – | – | – | – | 6.3 / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **17.1** / **16.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.8** | 5.5 / 18.0 | 81.3 / 79.0 | 2.6 / 11.8 | 2.6 / 11.8 | 22.0 / 29.6 | 19.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.1** / 31.0 | 127 / 127 | 223 / 220 | 9.9 / 32.8 | 9.0 / **23.9** | 83.4 / 98.9 | 18.8 / 33.0 | – | – | – | – | 11.1 / 43.0 |
| GF(2⁶⁴) | 233 / 232 | 442 / 416 | 613 / 604 | – | – | 886 / 881 | **95.1** / **100** | – | – | – | – | – |
| GF(2¹²⁸) | 740 / 738 | – | – | – | – | 2,902 / 2,904 | 298 / 291 | 284 / 285 | 590 / 581 | 613 / 580 | **257** / **263** | – |
| GF(2²³³) | **3,271** / **3,232** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,238** / **3,279** | – | – | – | – | 9,457 / 9,456 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15,838** / **15,674** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 3.6 / 15.1 | 72.1 / 61.3 | **2.2** / 7.9 | **2.2** / 7.9 | 9.6 / 19.9 | 11.5 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / **6.3** | 124 / 124 | 214 / 202 | 7.0 / 12.3 | 7.9 / 19.8 | 25.1 / 36.0 | 3.1 / 8.5 | – | – | – | – | 10.2 / 28.0 |
| GF(2⁶⁴) | **17.1** / **23.0** | 434 / 416 | 608 / 612 | – | – | 194 / 206 | 49.5 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **31.7** / **32.4** | – | – | – | – | 551 / 544 | 43.2 / 45.7 | 284 / 293 | 618 / 643 | 607 / 612 | 258 / 266 | – |
| GF(2²³³) | **62.1** / **64.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **65.0** / **67.7** | – | – | – | – | 1,526 / 1,452 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **174** / **250** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 4.0 / – | 400 / – | 33.2 / – | 33.8 / – | 571 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 3,897 / – | 2,294 / – | 681 / – | 590 / – | 717 / – | 32.1 / – | – | – | – | – | 9.3 / – |
| GF(2⁶⁴) | 2,374 / – | 54,012 / – | 24,867 / – | – | – | **2,095** / – | 4,678 / – | – | – | – | – | – |
| GF(2¹²⁸) | 7,997 / – | – | – | – | – | 5,395 / – | **1,307** / – | 16,710 / – | 155,766 / – | 155,775 / – | 65,778 / – | – |
| GF(2²³³) | **47,461** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,113 / – | – | – | – | – | **15,817** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **299,515** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 33.4 / – | 515 / – | 14.9 / – | 14.2 / – | 171 / – | 11.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 1,928 / – | 2,352 / – | 285 / – | 283 / – | 1,313 / – | 3.4 / – | – | – | – | – | 488 / – |
| GF(2⁶⁴) | 199 / – | 26,880 / – | 38,493 / – | – | – | 56,004 / – | **84.1** / – | – | – | – | – | – |
| GF(2¹²⁸) | 481 / – | – | – | – | – | 365,089 / – | **163** / – | 36,278 / – | 77,670 / – | 77,669 / – | 33,614 / – | – |
| GF(2²³³) | **233** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **386** / – | – | – | – | – | 2,416,617 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8,701** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | 2.2 / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | **2.1** / **1.1** | 2.2 / **1.1** | 2.2 / **1.1** | – | – | – | – | 2.2 / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.2 | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.2** | – | – | – | – | **3.3** / 2.4 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.7** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 6.6 / 4.7 | – | – | – | – | **6.3** / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **17.2** / **15.8** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.3** | – | 22.0 / 29.6 | 19.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **17.0** / **23.0** | 127 / 127 | 83.4 / 98.9 | – | – | – | – |
| GF(2⁶⁴) | 232 / 230 | 442 / 416 | 886 / 881 | **95.1** / **100** | – | – | – |
| GF(2¹²⁸) | 735 / 727 | – | 2,902 / 2,904 | 298 / 291 | 284 / 285 | 590 / 581 | **257** / **263** |
| GF(2²³³) | **3,255** / **3,229** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3,205** / **3,193** | – | 9,457 / 9,456 | – | – | – | – |
| GF(2⁵⁷¹) | **15,601** / **15,568** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / **15.0** | – | 9.6 / 19.9 | 11.5 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 124 / 124 | 25.1 / 36.0 | – | – | – | – |
| GF(2⁶⁴) | **17.0** / **23.0** | 434 / 416 | 194 / 206 | 49.5 / 62.0 | – | – | – |
| GF(2¹²⁸) | **31.2** / **32.3** | – | 551 / 544 | 43.2 / 45.7 | 284 / 293 | 618 / 643 | 258 / 266 |
| GF(2²³³) | **65.1** / **64.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **68.6** / **69.1** | – | 1,526 / 1,452 | – | – | – | – |
| GF(2⁵⁷¹) | **228** / **261** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **136** / – | – | 571 / – | 277 / – | – | – |
| GF(2¹⁶) | **233** / – | 3,897 / – | 717 / – | – | – | – |
| GF(2⁶⁴) | 3,631 / – | 54,012 / – | **2,095** / – | 4,678 / – | – | – |
| GF(2¹²⁸) | 12,006 / – | – | **5,395** / – | – | 155,766 / – | 65,778 / – |
| GF(2²³³) | **47,628** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 52,179 / – | – | **15,817** / – | – | – | – |
| GF(2⁵⁷¹) | **298,913** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / – | – | 171 / – | 11.5 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,928 / – | 1,313 / – | – | – | – | – |
| GF(2⁶⁴) | 200 / – | 26,880 / – | 56,004 / – | **84.1** / – | – | – | – |
| GF(2¹²⁸) | 490 / – | – | 365,089 / – | **163** / – | 36,278 / – | 77,670 / – | 33,614 / – |
| GF(2²³³) | **235** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **387** / – | – | 2,416,617 / – | – | – | – | – |
| GF(2⁵⁷¹) | **8,690** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.1 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | 6.4 / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **17.0** / **16.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.9** | 5.2 / 18.0 | 81.1 / 84.0 | 2.6 / 11.8 | 2.7 / 11.8 | 21.0 / 27.0 | 19.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.1** / 31.0 | 124 / 127 | 223 / 219 | 9.9 / 33.1 | 9.0 / **24.1** | 83.4 / 101 | 18.8 / 33.0 | – | – | – | – | 10.7 / 43.2 |
| GF(2⁶⁴) | **89.1** / **86.8** | 439 / 421 | 612 / 606 | – | – | 910 / 907 | 95.1 / 100 | – | – | – | – | – |
| GF(2¹²⁸) | 394 / 345 | – | – | – | – | 2,931 / 2,921 | 297 / 292 | 287 / 283 | 587 / 581 | 583 / 581 | **257** / **263** | – |
| GF(2²³³) | **1,753** / **1,759** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,153** / **1,187** | – | – | – | – | 9,477 / 9,477 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **5,384** / **5,283** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 4.0 / 16.0 | 73.0 / 61.4 | **2.2** / 7.9 | **2.2** / 7.9 | 9.6 / 19.9 | 11.5 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **6.3** | 122 / 124 | 214 / 202 | 7.1 / 12.3 | 7.9 / 15.7 | 27.0 / 36.0 | 3.1 / 8.5 | – | – | – | – | 10.1 / 28.0 |
| GF(2⁶⁴) | **17.1** / **23.0** | 435 / 430 | 609 / 607 | – | – | 193 / 199 | 49.6 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **31.8** / **32.0** | – | – | – | – | 551 / 520 | 33.6 / 34.0 | 282 / 292 | 617 / 612 | 618 / 612 | 256 / 264 | – |
| GF(2²³³) | **62.0** / **64.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **65.1** / **67.7** | – | – | – | – | 1,584 / 1,506 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **174** / **251** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 3.8 / – | 395 / – | 33.3 / – | 33.7 / – | 572 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 3,901 / – | 2,288 / – | 684 / – | 590 / – | 713 / – | 32.2 / – | – | – | – | – | 9.3 / – |
| GF(2⁶⁴) | **1,627** / – | 52,586 / – | 23,918 / – | – | – | 2,068 / – | 4,679 / – | – | – | – | – | – |
| GF(2¹²⁸) | 5,281 / – | – | – | – | – | 5,408 / – | **1,303** / – | 16,814 / – | 155,764 / – | 155,769 / – | 65,855 / – | – |
| GF(2²³³) | **32,321** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 33,627 / – | – | – | – | – | **15,835** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **191,154** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 32.5 / – | 514 / – | 14.5 / – | 14.5 / – | 174 / – | 11.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,923 / – | 2,346 / – | 284 / – | 283 / – | 1,356 / – | 3.4 / – | – | – | – | – | 483 / – |
| GF(2⁶⁴) | **56.6** / – | 26,801 / – | 38,734 / – | – | – | 56,799 / – | 84.1 / – | – | – | – | – | – |
| GF(2¹²⁸) | **149** / – | – | – | – | – | 369,632 / – | 163 / – | 36,971 / – | 77,705 / – | 77,685 / – | 33,503 / – | – |
| GF(2²³³) | **153** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **221** / – | – | – | – | – | 2,418,938 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **3,458** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | 3.3 / 2.1 | **2.2** / **1.1** | **2.2** / **1.1** | – | – | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | 6.4 / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **17.0** / **15.4** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.0** | – | 21.0 / 27.0 | 19.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.8** / **23.0** | 124 / 127 | 83.4 / 101 | – | – | – | – |
| GF(2⁶⁴) | **83.4** / **86.6** | 439 / 421 | 910 / 907 | 95.1 / 100 | – | – | – |
| GF(2¹²⁸) | 377 / 337 | – | 2,931 / 2,921 | 297 / 292 | 287 / 283 | 587 / 581 | **257** / **263** |
| GF(2²³³) | **1,812** / **1,781** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,138** / **1,186** | – | 9,477 / 9,477 | – | – | – | – |
| GF(2⁵⁷¹) | **5,302** / **5,462** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / **15.0** | – | 9.6 / 19.9 | 11.5 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 122 / 124 | 27.0 / 36.0 | – | – | – | – |
| GF(2⁶⁴) | **16.5** / **23.0** | 435 / 430 | 193 / 199 | 49.6 / 62.0 | – | – | – |
| GF(2¹²⁸) | **31.8** / **32.3** | – | 551 / 520 | 33.6 / 34.0 | 282 / 292 | 617 / 612 | 256 / 264 |
| GF(2²³³) | **65.1** / **64.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **68.3** / **67.7** | – | 1,584 / 1,506 | – | – | – | – |
| GF(2⁵⁷¹) | **229** / **260** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **136** / – | – | 572 / – | 277 / – | – | – |
| GF(2¹⁶) | **233** / – | 3,901 / – | 713 / – | – | – | – |
| GF(2⁶⁴) | 2,682 / – | 52,586 / – | **2,068** / – | 4,679 / – | – | – |
| GF(2¹²⁸) | 8,264 / – | – | **5,408** / – | – | 155,764 / – | 65,855 / – |
| GF(2²³³) | **32,534** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 33,230 / – | – | **15,835** / – | – | – | – |
| GF(2⁵⁷¹) | **191,247** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.7** / – | – | 174 / – | 11.5 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,923 / – | 1,356 / – | – | – | – | – |
| GF(2⁶⁴) | **64.8** / – | 26,801 / – | 56,799 / – | 84.1 / – | – | – | – |
| GF(2¹²⁸) | 172 / – | – | 369,632 / – | **163** / – | 36,971 / – | 77,705 / – | 33,503 / – |
| GF(2²³³) | **153** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **220** / – | – | 2,418,938 / – | – | – | – | – |
| GF(2⁵⁷¹) | **3,473** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
