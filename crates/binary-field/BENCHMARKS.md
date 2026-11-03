# Benchmarks

`binary-field` at commit `35cb27c` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-03.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 15; 1-minute load 15.18 before, 23.41 after; its SMT sibling, CPU 31, 67% busy |
| Load, without carry-less multiplication | CPU 15; 1-minute load 23.94 before, 26.8 after; its SMT sibling, CPU 31, 71% busy |
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

| Operation | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | runtime-algebra | core-algebra | galois_field_2pm lut |
|---|---|---|---|---|---|---|---|---|---|
| add | `+` | – | – | – | – | – | – | – | – |
| mul | `*` | `*` | `*` | `*` | `mul` | `*` | – | `mulf` | `*` |
| square | `square` | `a * a` | `a * a` | `a * a` | `a * a` | `square` | – | `a * a` | `a * a` |
| invert | `invert` | `checked_recip` | `inverse` | `a^(2^n − 2) by *` | `a^(2^n − 2) by *` | `try_inverse` | `invert` | `invert` | `inverse` |
| sqrt | `sqrt` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `try_sqrt` | – | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` |

- `gf256`: over GF(2⁸).
- `galois_field_2pm`: over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴).
- `g2p`: over GF(2⁸) and GF(2¹⁶).
- `reed-solomon-erasure`: over GF(2⁸) and GF(2¹⁶).
- `plonky3`: mul, square and sqrt over GF(2¹⁶); invert over GF(2¹⁶) and GF(2¹²⁸).
- `runtime-algebra`: over GF(2¹²⁸).
- `core-algebra`: over GF(2¹²⁸).
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

| Field | binary-field |
|---|---:|
| GF(2⁸) | **2.3** / **2.1** |
| GF(2¹⁶) | **2.5** / **2.3** |
| GF(2⁶⁴) | **1.8** / **1.4** |
| GF(2¹²⁸) | **1.7** / **1.5** |
| GF(2²³³) | **3.7** / **3.3** |
| GF(2²⁵⁶) | **3.6** / **3.5** |
| GF(2⁵⁷¹) | **8.1** / **6.9** |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.4** / 12.4 | 3.9 / **12.3** | 78.1 / 81.2 | 4.6 / 14.7 | 7.8 / 12.8 | – | – | – |
| GF(2¹⁶) | **8.1** / **19.2** | – | 166 / 167 | 21.8 / 96.0 | 8.9 / 27.0 | 18.8 / 38.2 | – | 40.7 / 96.0 |
| GF(2⁶⁴) | **6.4** / **11.2** | – | 425 / 433 | – | – | – | – | – |
| GF(2¹²⁸) | **13.9** / **13.7** | – | – | – | – | – | 640 / 644 | – |
| GF(2²³³) | **43.4** / **45.2** | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **37.9** / **36.6** | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **327** / **321** | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **5.1** | 2.2 / 11.1 | 82.7 / 73.0 | 2.4 / 7.1 | 4.4 / 7.2 | – | – | – |
| GF(2¹⁶) | **3.6** / **6.1** | – | 159 / 153 | 11.6 / 12.7 | 9.4 / 18.3 | 4.1 / 8.1 | – | 37.4 / 24.4 |
| GF(2⁶⁴) | **6.1** / **11.3** | – | 439 / 433 | – | – | – | – | – |
| GF(2¹²⁸) | **7.6** / **14.4** | – | – | – | – | – | 652 / 602 | – |
| GF(2²³³) | **21.5** / **32.6** | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **15.2** / **18.8** | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **45.0** / **43.6** | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | runtime-algebra | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.9** / – | 3.9 / – | 348 / – | 48.8 / – | 46.6 / – | – | – | – | – |
| GF(2¹⁶) | **4.5** / – | – | 1,154 / – | 1,275 / – | 606 / – | 35.9 / – | – | – | 32.3 / – |
| GF(2⁶⁴) | **253** / – | – | 9,608 / – | – | – | – | – | – | – |
| GF(2¹²⁸) | **604** / – | – | – | – | – | 740 / – | 6,415 / – | 167,789 / – | – |
| GF(2²³³) | **8,066** / – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,172** / – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **31,891** / – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.4** / – | 35.9 / – | 555 / – | 25.1 / – | 26.1 / – | – | – | – |
| GF(2¹⁶) | **5.1** / – | – | 2,465 / – | 431 / – | 266 / – | 8.7 / – | – | 619 / – |
| GF(2⁶⁴) | **14.8** / – | – | 27,237 / – | – | – | – | – | – |
| GF(2¹²⁸) | **40.1** / – | – | – | – | – | – | 81,193 / – | – |
| GF(2²³³) | **58.5** / – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **124** / – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **525** / – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.2 / **1.8** | 2.2 / 2.2 | **2.0** / 2.2 | 2.3 / 2.0 | 7.1 / 4.3 | 2.1 / 1.9 | 2.2 / 2.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / 2.3 | 2.5 / 2.6 | 2.6 / 2.4 | 2.6 / **2.2** | 8.4 / 6.5 | 11.2 / 6.3 | 17.2 / 6.8 | – | – | – | – | 2.8 / 2.6 |
| GF(2⁶⁴) | 2.3 / 2.2 | 1.9 / **1.8** | **1.7** / 2.0 | – | – | 27.4 / 19.6 | 2.3 / **1.8** | – | – | – | – | – |
| GF(2¹²⁸) | **1.9** / **1.7** | – | – | – | – | 26.9 / 19.0 | 2.7 / 2.1 | 2.3 / 2.2 | 2.0 / 2.2 | 2.0 / 2.0 | 18.9 / 21.6 | – |
| GF(2²³³) | **4.6** / **2.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.5** / **3.3** | – | – | – | – | 28.4 / 18.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **11.9** / **7.8** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **7.5** / **17.5** | – | 17.2 / 23.3 | 21.6 / 29.6 | – | – | – |
| GF(2¹⁶) | **9.5** / **19.3** | 9.8 / 27.1 | 76.7 / 62.8 | – | – | – | – |
| GF(2⁶⁴) | **5.9** / **11.2** | 6.4 / 20.1 | 695 / 698 | 7.0 / 19.2 | – | – | – |
| GF(2¹²⁸) | 13.1 / **12.6** | – | 1,810 / 1,869 | 12.8 / 16.1 | **12.3** / 18.7 | 20.2 / 28.0 | 180 / 178 |
| GF(2²³³) | **44.4** / **45.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.4** / **37.3** | – | 5,844 / 6,224 | – | – | – | – |
| GF(2⁵⁷¹) | **309** / **270** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.2** / **11.2** | – | 10.3 / 18.5 | 11.3 / 23.1 | – | – | – |
| GF(2¹⁶) | **8.2** / **19.1** | 9.0 / 27.9 | 30.0 / 32.1 | – | – | – | – |
| GF(2⁶⁴) | **5.2** / **11.3** | 6.5 / 20.8 | 164 / 161 | 6.7 / 16.6 | – | – | – |
| GF(2¹²⁸) | **7.3** / **14.5** | – | 368 / 355 | 9.3 / 16.4 | 10.7 / 19.0 | 21.3 / 29.0 | 172 / 176 |
| GF(2²³³) | **21.7** / **32.5** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **14.4** / **18.8** | – | 1,216 / 1,314 | – | – | – | – |
| GF(2⁵⁷¹) | **46.9** / **44.9** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **73.3** / – | – | 277 / – | 346 / – | – | – |
| GF(2¹⁶) | **222** / – | 431 / – | 362 / – | – | – | – |
| GF(2⁶⁴) | **678** / – | 1,251 / – | 1,448 / – | 1,109 / – | – | – |
| GF(2¹²⁸) | **1,956** / – | – | 4,049 / – | – | 5,568 / – | 45,710 / – |
| GF(2²³³) | **8,014** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,130** / – | – | 11,947 / – | – | – | – |
| GF(2⁵⁷¹) | **31,223** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.3** / – | – | 169 / – | 15.3 / – | – | – | – |
| GF(2¹⁶) | **18.1** / – | 252 / – | 894 / – | – | – | – | – |
| GF(2⁶⁴) | **12.3** / – | 935 / – | 43,137 / – | 30.4 / – | – | – | – |
| GF(2¹²⁸) | 45.6 / – | – | 284,328 / – | **34.9** / – | 2,462 / – | 3,609 / – | 20,428 / – |
| GF(2²³³) | **79.9** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **120** / – | – | 1,778,167 / – | – | – | – | – |
| GF(2⁵⁷¹) | **389** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field |
|---|---:|
| GF(2⁸) | **2.4** / **2.2** |
| GF(2¹⁶) | **2.5** / **2.1** |
| GF(2⁶⁴) | **2.4** / **2.0** |
| GF(2¹²⁸) | **2.0** / **1.8** |
| GF(2²³³) | **3.3** / **2.9** |
| GF(2²⁵⁶) | **3.5** / **3.0** |
| GF(2⁵⁷¹) | **7.2** / **6.3** |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.3** / 12.8 | 6.2 / **12.4** | 84.9 / 85.3 | **5.3** / 14.4 | 5.7 / 16.1 | – | – | – |
| GF(2¹⁶) | **10.2** / 48.9 | – | 170 / 177 | 27.4 / 58.7 | 10.9 / **27.0** | 18.9 / 31.2 | – | 26.9 / 90.4 |
| GF(2⁶⁴) | **34.0** / **42.3** | – | 391 / 390 | – | – | – | – | – |
| GF(2¹²⁸) | **58.4** / **65.4** | – | – | – | – | – | 606 / 584 | – |
| GF(2²³³) | **209** / **194** | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **190** / **195** | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,441** / **1,408** | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.2 / **5.1** | **2.1** / 11.0 | 80.1 / 71.5 | **2.1** / 7.1 | 3.5 / 7.1 | – | – | – |
| GF(2¹⁶) | **3.2** / **6.1** | – | 159 / 153 | 14.3 / 12.1 | 9.6 / 17.7 | 7.6 / 8.1 | – | 29.6 / 24.5 |
| GF(2⁶⁴) | **6.0** / **11.8** | – | 384 / 388 | – | – | – | – | – |
| GF(2¹²⁸) | **7.8** / **14.5** | – | – | – | – | – | 578 / 631 | – |
| GF(2²³³) | **21.3** / **32.3** | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **21.2** / **27.1** | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **56.7** / **56.3** | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | runtime-algebra | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.6** / – | 3.8 / – | 352 / – | 43.1 / – | 49.0 / – | – | – | – | – |
| GF(2¹⁶) | **5.3** / – | – | 1,168 / – | 1,226 / – | 596 / – | 36.6 / – | – | – | 43.4 / – |
| GF(2⁶⁴) | **546** / – | – | 9,671 / – | – | – | – | – | – | – |
| GF(2¹²⁸) | **1,148** / – | – | – | – | – | 1,357 / – | 12,333 / – | 166,002 / – | – |
| GF(2²³³) | **10,232** / – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **9,594** / – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **54,111** / – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | plonky3 | core-algebra | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.9** / – | 39.9 / – | 564 / – | 24.8 / – | 26.8 / – | – | – | – |
| GF(2¹⁶) | 7.6 / – | – | 2,324 / – | 473 / – | 245 / – | **5.3** / – | – | 193 / – |
| GF(2⁶⁴) | **36.9** / – | – | 25,293 / – | – | – | – | – | – |
| GF(2¹²⁸) | **85.3** / – | – | – | – | – | – | 83,301 / – | – |
| GF(2²³³) | **129** / – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **105** / – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,552** / – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **2.2** | 2.5 / 2.4 | 2.2 / **2.2** | 2.5 / 2.4 | 10.4 / 7.5 | 2.3 / 2.5 | 2.6 / **2.2** | – | – | – | – | – |
| GF(2¹⁶) | 2.7 / 2.8 | **2.5** / 2.6 | 2.8 / 2.6 | 2.8 / 3.0 | 10.0 / 11.4 | 7.3 / 10.1 | 12.2 / 9.8 | – | – | – | – | 2.8 / **2.5** |
| GF(2⁶⁴) | 2.7 / 2.0 | 2.0 / 1.9 | **1.9** / **1.8** | – | – | 42.6 / 23.7 | 3.1 / 2.0 | – | – | – | – | – |
| GF(2¹²⁸) | 2.2 / 2.3 | – | – | – | – | 41.0 / 32.8 | 2.9 / 2.7 | 2.4 / 2.1 | **2.0** / **2.0** | 2.2 / 2.3 | 36.4 / 32.9 | – |
| GF(2²³³) | **5.6** / **4.7** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.8** / **3.4** | – | – | – | – | 49.1 / 47.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **11.5** / **7.4** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 18.4 / **22.5** | – | **17.5** / 23.7 | 21.2 / 30.6 | – | – | – |
| GF(2¹⁶) | **15.6** / **23.0** | 41.8 / 55.9 | 68.9 / 68.4 | – | – | – | – |
| GF(2⁶⁴) | **35.2** / **42.4** | 166 / 119 | 579 / 723 | 58.0 / 75.6 | – | – | – |
| GF(2¹²⁸) | **59.2** / **65.8** | – | 2,216 / 2,168 | 197 / 203 | 204 / 212 | 660 / 657 | 185 / 193 |
| GF(2²³³) | **247** / **226** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **226** / **231** | – | 7,270 / 6,807 | – | – | – | – |
| GF(2⁵⁷¹) | **1,723** / **1,475** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.2** / **11.0** | – | 8.7 / 18.4 | 11.0 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.9** / **22.4** | 31.7 / 56.6 | 25.6 / 30.4 | – | – | – | – |
| GF(2⁶⁴) | **6.5** / **11.9** | 104 / 127 | 158 / 167 | 29.6 / 44.3 | – | – | – |
| GF(2¹²⁸) | **7.3** / **14.5** | – | 354 / 317 | 39.9 / 42.2 | 183 / 198 | 589 / 599 | 180 / 173 |
| GF(2²³³) | **21.3** / **32.4** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **21.4** / **27.1** | – | 1,175 / 1,136 | – | – | – | – |
| GF(2⁵⁷¹) | **59.1** / **56.3** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **111** / – | – | 269 / – | 339 / – | – | – |
| GF(2¹⁶) | **237** / – | 1,382 / – | 366 / – | – | – | – |
| GF(2⁶⁴) | **1,078** / – | 14,288 / – | 1,531 / – | 3,365 / – | – | – |
| GF(2¹²⁸) | **2,548** / – | – | 4,057 / – | – | 179,162 / – | 45,191 / – |
| GF(2²³³) | **10,833** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **9,774** / – | – | 11,475 / – | – | – | – |
| GF(2⁵⁷¹) | **55,378** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / – | – | 168 / – | 15.1 / – | – | – | – |
| GF(2¹⁶) | **13.6** / – | 820 / – | 807 / – | – | – | – | – |
| GF(2⁶⁴) | **37.2** / – | 7,956 / – | 43,814 / – | 71.6 / – | – | – | – |
| GF(2¹²⁸) | **83.8** / – | – | 277,893 / – | 145 / – | 26,243 / – | 78,131 / – | 21,374 / – |
| GF(2²³³) | **144** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **131** / – | – | 1,630,689 / – | – | – | – | – |
| GF(2⁵⁷¹) | **1,577** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file
```
