# Benchmarks

`binary-field` at commit `04319df` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 21.05 before, 9.06 after; its SMT sibling, CPU 29, 4% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 13.61 before, 23.97 after; its SMT sibling, CPU 29, 20% busy |
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
| GF(2¹²⁸) | **1.4** / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | **1.4** / **1.1** | **1.4** / 1.4 | **1.4** / 1.4 | 1.6 / 1.4 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.1** | – | – | – | – | 2.7 / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.9** / **5.2** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.4 / 78.7 | **2.0** / 11.3 | **2.0** / 11.3 | 13.4 / 23.0 | 14.7 / 29.1 | – | – | – | – | – |
| GF(2¹⁶) | **5.0** / **20.2** | 8.0 / 27.0 | 141 / 144 | 8.6 / 36.4 | 7.4 / 22.5 | 53.0 / 52.0 | 14.1 / 30.5 | – | – | – | – | 12.6 / 63.1 |
| GF(2⁶⁴) | **4.9** / **11.2** | 6.0 / 20.0 | 365 / 366 | – | – | 548 / 548 | 5.9 / 19.0 | – | – | – | – | – |
| GF(2¹²⁸) | 12.0 / **12.0** | – | – | – | – | 1,758 / 1,760 | **11.9** / 16.0 | **11.9** / 18.6 | 20.0 / 27.7 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **39.8** / **43.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **33.9** / **34.0** | – | – | – | – | 5,727 / 5,733 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **167** / **163** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 58.6 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 7.0 / 27.8 | 138 / 125 | 6.5 / 10.7 | 6.5 / 17.1 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.3 / 24.0 |
| GF(2⁶⁴) | **4.5** / **11.1** | 6.0 / 20.8 | 364 / 373 | – | – | 125 / 138 | 5.0 / 15.7 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.5** | – | – | – | – | 288 / 285 | 7.9 / 16.6 | 10.0 / 18.9 | 16.4 / 28.6 | 547 / 546 | 146 / 157 | – |
| GF(2²³³) | **21.4** / **31.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.9** | – | – | – | – | 1,022 / 1,084 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.2** / **40.9** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 309 / – | 29.1 / – | 29.4 / – | 254 / – | 310 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 420 / – | 1,004 / – | 818 / – | 520 / – | 324 / – | 20.4 / – | – | – | – | – | 10.1 / – |
| GF(2⁶⁴) | **211** / – | 1,224 / – | 8,454 / – | – | – | 1,209 / – | 1,065 / – | – | – | – | – | – |
| GF(2¹²⁸) | **399** / – | – | – | – | – | 3,222 / – | 588 / – | 5,972 / – | 5,056 / – | 147,689 / – | 39,272 / – | – |
| GF(2²³³) | **7,902** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,265** / – | – | – | – | – | 9,715 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,039** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 510 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 238 / – | 2,075 / – | 331 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 574 / – |
| GF(2⁶⁴) | **11.2** / – | 932 / – | 24,121 / – | – | – | 34,519 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | – | – | – | 224,029 / – | 17.7 / – | 2,446 / – | 3,574 / – | 69,340 / – | 19,875 / – | – |
| GF(2²³³) | **47.8** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.5** / – | – | – | – | – | 1,461,721 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **134** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.7 / 1.5 | 1.6 / 1.3 | 1.4 / **1.1** | 1.4 / 1.4 | 1.4 / 1.4 | 1.6 / 1.4 | – |
| GF(2²³³) | **2.2** / **2.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.1** | – | – | – | – | **2.7** / 2.7 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.3** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / **17.4** | – | 13.4 / 23.0 | 14.7 / 29.1 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **4.8** / **11.2** | 6.0 / 20.0 | 548 / 548 | 5.9 / 19.0 | – | – | – |
| GF(2¹²⁸) | **11.9** / **12.0** | – | 1,758 / 1,760 | **11.9** / 16.0 | **11.9** / 18.6 | 20.0 / 27.7 | 146 / 155 |
| GF(2²³³) | **39.5** / **43.5** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **33.9** / **34.0** | – | 5,727 / 5,733 | – | – | – | – |
| GF(2⁵⁷¹) | **170** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.9** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.0** | 6.0 / 20.8 | 125 / 138 | 5.0 / 15.7 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.5** | – | 288 / 285 | 7.9 / 16.6 | 10.0 / 18.9 | 16.4 / 28.6 | 146 / 157 |
| GF(2²³³) | **21.2** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.0** / **18.8** | – | 1,022 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **41.1** / **41.0** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.0** / – | – | 254 / – | 310 / – | – | – |
| GF(2¹⁶) | **170** / – | 420 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,224 / – | 1,209 / – | 1,065 / – | – | – |
| GF(2¹²⁸) | **1,967** / – | – | 3,222 / – | – | 5,056 / – | 39,272 / – |
| GF(2²³³) | **7,894** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,363** / – | – | 9,715 / – | – | – | – |
| GF(2⁵⁷¹) | **27,031** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.4** / – | 238 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 932 / – | 34,519 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | 224,029 / – | 17.7 / – | 2,446 / – | 3,574 / – | 19,875 / – |
| GF(2²³³) | **47.7** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.5** / – | – | 1,461,721 / – | – | – | – | – |
| GF(2⁵⁷¹) | **133** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.2** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.5 / **1.1** | – | – | – | – | 1.5 / 1.3 | 1.4 / 1.3 | **1.3** / 1.3 | 1.4 / 1.3 | **1.3** / 1.3 | 1.9 / 1.3 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.0** | – | – | – | – | **2.5** / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.7** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.2 / 78.8 | **2.0** / 11.3 | **2.0** / 11.4 | 13.3 / 23.0 | 14.9 / 29.4 | – | – | – | – | – |
| GF(2¹⁶) | **5.2** / 32.7 | 31.4 / 54.9 | 141 / 143 | 8.9 / 42.1 | 7.4 / **22.3** | 53.0 / 52.0 | 14.1 / 30.4 | – | – | – | – | 12.4 / 61.9 |
| GF(2⁶⁴) | **31.9** / **41.4** | 100 / 116 | 365 / 367 | – | – | 548 / 548 | 47.1 / 68.7 | – | – | – | – | – |
| GF(2¹²⁸) | **53.6** / **64.8** | – | – | – | – | 1,759 / 1,760 | 162 / 186 | 171 / 180 | 549 / 546 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **202** / **222** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **191** / **191** | – | – | – | – | 5,748 / 5,754 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,304** / **1,273** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.7 / 59.0 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 17.9 | 8.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 31.5 / 56.5 | 138 / 115 | 6.5 / 11.5 | 6.4 / 17.0 | 23.0 / 30.0 | 2.0 / 8.0 | – | – | – | – | 10.1 / 24.0 |
| GF(2⁶⁴) | **5.4** / **11.8** | 98.7 / 126 | 364 / 371 | – | – | 126 / 140 | 24.5 / 44.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.3** | – | – | – | – | 288 / 286 | 36.9 / 39.9 | 175 / 180 | 547 / 546 | 547 / 546 | 146 / 157 | – |
| GF(2²³³) | **21.2** / **32.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **27.1** | – | – | – | – | 1,022 / 1,083 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.0** / **57.1** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 309 / – | 29.3 / – | 29.4 / – | 254 / – | 312 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 1,151 / – | 1,001 / – | 952 / – | 519 / – | 324 / – | 20.4 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **529** / – | 13,929 / – | 8,480 / – | – | – | 1,209 / – | 3,238 / – | – | – | – | – | – |
| GF(2¹²⁸) | **830** / – | – | – | – | – | 3,214 / – | 973 / – | 11,306 / – | 139,428 / – | 147,693 / – | 39,271 / – | – |
| GF(2²³³) | **9,369** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,795** / – | – | – | – | – | 9,712 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **48,596** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 510 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 814 / – | 2,074 / – | 329 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 566 / – |
| GF(2⁶⁴) | **35.6** / – | 7,932 / – | 24,129 / – | – | – | 34,519 / – | 40.2 / – | – | – | – | – | – |
| GF(2¹²⁸) | **65.7** / – | – | – | – | – | 224,007 / – | 123 / – | 22,607 / – | 69,341 / – | 69,339 / – | 19,876 / – | – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.5** / – | – | – | – | – | 1,468,202 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **675** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.2** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.2** / **1.1** | – | – | – | – | 1.5 / 1.3 | 1.4 / 1.3 | 1.3 / 1.3 | 1.4 / 1.3 | 1.3 / 1.3 | 1.9 / 1.3 | – |
| GF(2²³³) | **2.0** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.5** / **2.0** | – | – | – | – | **2.5** / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.1** / **4.5** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 13.9 / **21.0** | – | **13.3** / 23.0 | 14.9 / 29.4 | – | – | – |
| GF(2¹⁶) | **15.0** / **22.5** | 31.4 / 54.9 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **31.6** / **41.2** | 100 / 116 | 548 / 548 | 47.1 / 68.7 | – | – | – |
| GF(2¹²⁸) | **54.9** / **64.8** | – | 1,759 / 1,760 | 162 / 186 | 171 / 180 | 549 / 546 | 146 / 155 |
| GF(2²³³) | **200** / **221** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **190** / **191** | – | 5,748 / 5,754 | – | – | – | – |
| GF(2⁵⁷¹) | **1,310** / **1,289** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.8** | – | 8.0 / 17.9 | 8.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.6** / **22.2** | 31.5 / 56.5 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **5.4** / **11.8** | 98.7 / 126 | 126 / 140 | 24.5 / 44.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | 288 / 286 | 36.9 / 39.9 | 175 / 180 | 547 / 546 | 146 / 157 |
| GF(2²³³) | **20.6** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.3** / **27.3** | – | 1,022 / 1,083 | – | – | – | – |
| GF(2⁵⁷¹) | **55.0** / **57.1** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **92.0** / – | – | 254 / – | 312 / – | – | – |
| GF(2¹⁶) | **227** / – | 1,151 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **569** / – | 13,929 / – | 1,209 / – | 3,238 / – | – | – |
| GF(2¹²⁸) | **1,915** / – | – | 3,214 / – | – | 139,428 / – | 39,271 / – |
| GF(2²³³) | **9,374** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,592** / – | – | 9,712 / – | – | – | – |
| GF(2⁵⁷¹) | **48,628** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 814 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **35.6** / – | 7,932 / – | 34,519 / – | 40.2 / – | – | – | – |
| GF(2¹²⁸) | **65.6** / – | – | 224,007 / – | 123 / – | 22,607 / – | 69,341 / – | 19,876 / – |
| GF(2²³³) | **110** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.3** / – | – | 1,468,202 / – | – | – | – | – |
| GF(2⁵⁷¹) | **675** / – | – | – | – | – | – | – |

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

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | **2.5** / **1.3** | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | 3.8 / 2.4 | **3.7** / **2.3** | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | – |
| GF(2²³³) | **7.4** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.4** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **23.1** / **17.2** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.5** | 4.7 / 18.0 | 77.7 / 77.1 | 2.7 / 11.7 | 2.7 / 11.6 | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **5.8** / 30.1 | 93.2 / 93.0 | 172 / 166 | 9.8 / 33.1 | 8.7 / **24.1** | 76.0 / 82.0 | 19.0 / 33.2 | – | – | – | – | 9.3 / 42.3 |
| GF(2⁶⁴) | **63.9** / **72.0** | 585 / 585 | 662 / 669 | – | – | 806 / 806 | 92.6 / 77.7 | – | – | – | – | – |
| GF(2¹²⁸) | **184** / **184** | – | – | – | – | 2,613 / 2,618 | 243 / 250 | 250 / 250 | 581 / 580 | 581 / 580 | 227 / 228 | – |
| GF(2²³³) | **754** / **759** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **706** / **705** | – | – | – | – | 8,420 / 8,431 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **4,011** / **3,978** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.1** | 3.1 / 14.7 | 70.7 / 53.8 | **2.2** / 7.8 | **2.2** / 7.7 | 8.9 / 19.1 | 10.1 / 25.4 | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **6.1** | 92.4 / 98.7 | 164 / 154 | 6.9 / 12.0 | 7.8 / 17.4 | 25.4 / 34.0 | 3.0 / 8.3 | – | – | – | – | 8.1 / 27.0 |
| GF(2⁶⁴) | **15.8** / **19.0** | 585 / 583 | 661 / 669 | – | – | 180 / 197 | 46.2 / 59.0 | – | – | – | – | – |
| GF(2¹²⁸) | **28.0** / **27.0** | – | – | – | – | 503 / 493 | 34.6 / 34.3 | 251 / 250 | 581 / 579 | 581 / 579 | 227 / 229 | – |
| GF(2²³³) | **50.7** / **54.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **58.1** / **59.0** | – | – | – | – | 1,383 / 1,380 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **145** / **161** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 3.6 / – | 321 / – | 36.0 / – | 36.6 / – | 307 / – | 272 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 2,738 / – | 1,347 / – | 716 / – | 637 / – | 393 / – | 32.1 / – | – | – | – | – | 8.2 / – |
| GF(2⁶⁴) | **826** / – | 73,516 / – | 18,101 / – | – | – | 1,577 / – | 4,451 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,512 / – | – | – | – | – | 4,494 / – | **1,221** / – | 16,106 / – | 182,019 / – | 182,259 / – | 57,682 / – | – |
| GF(2²³³) | **20,793** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,975 / – | – | – | – | – | **13,703** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **130,347** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 30.8 / – | 490 / – | 14.9 / – | 15.0 / – | 174 / – | 10.9 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,452 / – | 2,305 / – | 307 / – | 290 / – | 1,243 / – | 3.3 / – | – | – | – | – | 445 / – |
| GF(2⁶⁴) | **37.2** / – | 36,610 / – | 42,442 / – | – | – | 51,581 / – | 80.1 / – | – | – | – | – | – |
| GF(2¹²⁸) | 154 / – | – | – | – | – | 332,020 / – | **149** / – | 31,525 / – | 107,940 / – | 107,926 / – | 28,969 / – | – |
| GF(2²³³) | **96.5** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **122** / – | – | – | – | – | 2,155,728 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **2,207** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | **2.5** / **1.3** | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | 3.8 / 2.4 | **3.7** / **2.3** | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.5** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **23.1** / **17.7** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **13.0** / **21.3** | – | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.5** / **22.0** | 93.2 / 93.0 | 76.0 / 82.0 | – | – | – | – |
| GF(2⁶⁴) | **63.9** / **72.0** | 585 / 585 | 806 / 806 | 92.6 / 77.7 | – | – | – |
| GF(2¹²⁸) | **184** / **184** | – | 2,613 / 2,618 | 243 / 250 | 250 / 250 | 581 / 580 | 227 / 228 |
| GF(2²³³) | **794** / **758** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **704** / **704** | – | 8,420 / 8,431 | – | – | – | – |
| GF(2⁵⁷¹) | **4,063** / **3,984** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / **15.0** | – | 8.9 / 19.1 | 10.1 / 25.4 | – | – | – |
| GF(2¹⁶) | **10.2** / **23.0** | 92.4 / 98.7 | 25.4 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **15.6** / **19.0** | 585 / 583 | 180 / 197 | 46.2 / 59.0 | – | – | – |
| GF(2¹²⁸) | **28.0** / **27.3** | – | 503 / 493 | 34.6 / 34.3 | 251 / 250 | 581 / 579 | 227 / 229 |
| GF(2²³³) | **53.1** / **54.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **62.1** / **59.0** | – | 1,383 / 1,380 | – | – | – | – |
| GF(2⁵⁷¹) | **162** / **164** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **123** / – | – | 307 / – | 272 / – | – | – |
| GF(2¹⁶) | **239** / – | 2,738 / – | 393 / – | – | – | – |
| GF(2⁶⁴) | **1,302** / – | 73,516 / – | 1,577 / – | 4,451 / – | – | – |
| GF(2¹²⁸) | 4,539 / – | – | **4,494** / – | – | 182,019 / – | 57,682 / – |
| GF(2²³³) | **20,876** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,979 / – | – | **13,703** / – | – | – | – |
| GF(2⁵⁷¹) | **130,358** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | 174 / – | 10.9 / – | – | – | – |
| GF(2¹⁶) | **16.8** / – | 1,452 / – | 1,243 / – | – | – | – | – |
| GF(2⁶⁴) | **37.2** / – | 36,610 / – | 51,581 / – | 80.1 / – | – | – | – |
| GF(2¹²⁸) | 154 / – | – | 332,020 / – | **149** / – | 31,525 / – | 107,940 / – | 28,969 / – |
| GF(2²³³) | **111** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **122** / – | – | 2,155,728 / – | – | – | – | – |
| GF(2⁵⁷¹) | **2,241** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | **2.5** / **1.3** | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | **3.7** / 2.4 | **3.7** / 2.4 | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.5** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **23.2** / **17.5** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.5** | 4.8 / 18.0 | 77.7 / 76.9 | 2.6 / 11.7 | 2.7 / 11.7 | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **5.7** / 30.2 | 90.3 / 93.2 | 174 / 167 | 10.1 / 34.2 | 8.7 / **23.9** | 76.0 / 81.9 | 19.0 / 33.2 | – | – | – | – | 9.5 / 42.3 |
| GF(2⁶⁴) | **63.6** / **72.0** | 585 / 585 | 662 / 669 | – | – | 805 / 801 | 92.0 / 76.1 | – | – | – | – | – |
| GF(2¹²⁸) | **184** / **182** | – | – | – | – | 2,616 / 2,620 | 243 / 250 | 248 / 250 | 581 / 580 | 581 / 580 | 227 / 229 | – |
| GF(2²³³) | **723** / **721** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **705** / **707** | – | – | – | – | 8,413 / 8,427 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **3,988** / **4,018** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **5.1** | 3.1 / 14.8 | 70.7 / 57.6 | 2.2 / 7.6 | 2.2 / 7.6 | 8.9 / 19.3 | 10.1 / 25.4 | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **6.1** | 92.4 / 97.0 | 164 / 154 | 6.9 / 12.0 | 7.8 / 17.1 | 25.4 / 34.0 | 3.0 / 8.3 | – | – | – | – | 8.2 / 27.0 |
| GF(2⁶⁴) | **15.2** / **19.0** | 585 / 583 | 661 / 668 | – | – | 182 / 196 | 46.2 / 59.0 | – | – | – | – | – |
| GF(2¹²⁸) | **28.0** / **27.0** | – | – | – | – | 505 / 493 | 30.7 / 32.6 | 251 / 250 | 581 / 579 | 581 / 579 | 225 / 226 | – |
| GF(2²³³) | **50.6** / **53.9** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **57.9** / **59.0** | – | – | – | – | 1,383 / 1,379 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **145** / **165** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / – | 3.6 / – | 323 / – | 36.1 / – | 36.7 / – | 307 / – | 272 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 2,741 / – | 1,369 / – | 745 / – | 640 / – | 393 / – | 32.1 / – | – | – | – | – | 8.2 / – |
| GF(2⁶⁴) | **793** / – | 73,526 / – | 18,114 / – | – | – | 1,582 / – | 4,451 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,506 / – | – | – | – | – | 4,498 / – | **1,253** / – | 16,029 / – | 181,218 / – | 181,277 / – | 57,688 / – | – |
| GF(2²³³) | **20,878** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,988 / – | – | – | – | – | **13,698** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **130,186** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 30.8 / – | 490 / – | 15.0 / – | 15.2 / – | 174 / – | 10.9 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,453 / – | 2,304 / – | 307 / – | 291 / – | 1,242 / – | 3.3 / – | – | – | – | – | 446 / – |
| GF(2⁶⁴) | **37.2** / – | 36,602 / – | 42,420 / – | – | – | 52,003 / – | 80.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | 154 / – | – | – | – | – | 332,830 / – | **149** / – | 31,538 / – | 108,082 / – | 107,797 / – | 28,962 / – | – |
| GF(2²³³) | **107** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **122** / – | – | – | – | – | 2,155,481 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **2,210** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | **2.1** / **1.1** |
| GF(2⁶⁴) | **2.5** / **1.3** | **2.5** / **1.3** | **2.5** / **1.3** | – | – | **2.5** / **1.3** | **2.5** / **1.3** | – | – | – | – | – |
| GF(2¹²⁸) | 3.8 / **2.3** | – | – | – | – | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / 2.4 | **3.7** / **2.3** | **3.7** / 2.4 | **3.7** / 2.4 | – |
| GF(2²³³) | **7.5** / **4.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7.4** / **4.8** | – | – | – | – | 8.0 / 5.0 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **23.2** / **17.5** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **13.0** / **21.3** | – | 22.0 / 28.0 | 19.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.5** / **22.0** | 90.3 / 93.2 | 76.0 / 81.9 | – | – | – | – |
| GF(2⁶⁴) | **61.9** / **72.0** | 585 / 585 | 805 / 801 | 92.0 / 76.1 | – | – | – |
| GF(2¹²⁸) | **184** / **182** | – | 2,616 / 2,620 | 243 / 250 | 248 / 250 | 581 / 580 | 227 / 229 |
| GF(2²³³) | **787** / **759** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **704** / **706** | – | 8,413 / 8,427 | – | – | – | – |
| GF(2⁵⁷¹) | **4,066** / **4,002** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.0** / **15.0** | – | 8.9 / 19.3 | 10.1 / 25.4 | – | – | – |
| GF(2¹⁶) | **10.2** / **23.0** | 92.4 / 97.0 | 25.4 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **16.0** / **19.0** | 585 / 583 | 182 / 196 | 46.2 / 59.0 | – | – | – |
| GF(2¹²⁸) | **28.0** / **27.3** | – | 505 / 493 | 30.7 / 32.6 | 251 / 250 | 581 / 579 | 225 / 226 |
| GF(2²³³) | **53.0** / **54.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **62.1** / **59.0** | – | 1,383 / 1,379 | – | – | – | – |
| GF(2⁵⁷¹) | **162** / **165** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **123** / – | – | 307 / – | 272 / – | – | – |
| GF(2¹⁶) | **239** / – | 2,741 / – | 393 / – | – | – | – |
| GF(2⁶⁴) | **1,317** / – | 73,526 / – | 1,582 / – | 4,451 / – | – | – |
| GF(2¹²⁸) | 4,532 / – | – | **4,498** / – | – | 181,218 / – | 57,688 / – |
| GF(2²³³) | **20,848** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 22,971 / – | – | **13,698** / – | – | – | – |
| GF(2⁵⁷¹) | **130,084** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | 174 / – | 10.9 / – | – | – | – |
| GF(2¹⁶) | **16.8** / – | 1,453 / – | 1,242 / – | – | – | – | – |
| GF(2⁶⁴) | **37.1** / – | 36,602 / – | 52,003 / – | 80.0 / – | – | – | – |
| GF(2¹²⁸) | 154 / – | – | 332,830 / – | **149** / – | 31,538 / – | 108,082 / – | 28,962 / – |
| GF(2²³³) | **113** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **122** / – | – | 2,155,481 / – | – | – | – | – |
| GF(2⁵⁷¹) | **2,210** / – | – | – | – | – | – | – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.2 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | **6.3** / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15.5** / **12.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.9** | 5.5 / 18.0 | 78.8 / 83.8 | 2.7 / 11.8 | 2.6 / 11.8 | 22.0 / 29.6 | 19.7 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.2** / 30.9 | 127 / 127 | 221 / 218 | 10.0 / 33.8 | 8.9 / **24.6** | 83.4 / 98.7 | 19.1 / 33.0 | – | – | – | – | 10.7 / 43.2 |
| GF(2⁶⁴) | **73.1** / **73.0** | 439 / 427 | 606 / 606 | – | – | 893 / 883 | 95.1 / 100 | – | – | – | – | – |
| GF(2¹²⁸) | **200** / **197** | – | – | – | – | 2,910 / 2,938 | 293 / 300 | 287 / 284 | 588 / 581 | 609 / 581 | 258 / 263 | – |
| GF(2²³³) | **1,089** / **1,067** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,073** / **1,049** | – | – | – | – | 9,581 / 9,582 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **4,979** / **4,724** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 4.0 / 16.0 | 72.5 / 59.5 | **2.2** / 7.9 | **2.2** / 7.9 | 9.6 / 19.5 | 11.5 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / **6.3** | 123 / 126 | 209 / 203 | 7.0 / 12.2 | 8.0 / 19.9 | 27.0 / 36.0 | 3.1 / 8.4 | – | – | – | – | 9.6 / 28.0 |
| GF(2⁶⁴) | **17.0** / **19.0** | 435 / 419 | 607 / 608 | – | – | 193 / 199 | 49.0 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **31.0** / **30.1** | – | – | – | – | 542 / 546 | 43.2 / 43.7 | 284 / 293 | 590 / 612 | 613 / 613 | 258 / 263 | – |
| GF(2²³³) | **61.0** / **59.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **62.0** / **63.0** | – | – | – | – | 1,533 / 1,450 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **152** / **234** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 4.0 / – | 394 / – | 33.3 / – | 33.9 / – | 572 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 3,991 / – | 2,286 / – | 707 / – | 589 / – | 707 / – | 32.6 / – | – | – | – | – | 9.4 / – |
| GF(2⁶⁴) | **896** / – | 52,586 / – | 23,863 / – | – | – | 2,090 / – | 4,678 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,675 / – | – | – | – | – | 5,439 / – | **1,303** / – | 16,542 / – | 155,747 / – | 155,780 / – | 66,466 / – | – |
| GF(2²³³) | **24,297** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 29,628 / – | – | – | – | – | **15,994** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **161,832** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 33.5 / – | 512 / – | 14.9 / – | 14.9 / – | 173 / – | 11.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / – | 1,903 / – | 2,353 / – | 287 / – | 281 / – | 1,355 / – | 3.4 / – | – | – | – | – | 485 / – |
| GF(2⁶⁴) | **40.0** / – | 26,807 / – | 38,646 / – | – | – | 55,821 / – | 84.1 / – | – | – | – | – | – |
| GF(2¹²⁸) | **156** / – | – | – | – | – | 366,179 / – | 163 / – | 36,956 / – | 77,688 / – | 81,515 / – | 33,121 / – | – |
| GF(2²³³) | **132** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **161** / – | – | – | – | – | 2,447,405 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **2,528** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.2 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | **6.3** / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15.5** / **12.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.0** | – | 22.0 / 29.6 | 19.7 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.7** / **23.0** | 127 / 127 | 83.4 / 98.7 | – | – | – | – |
| GF(2⁶⁴) | **73.1** / **73.1** | 439 / 427 | 893 / 883 | 95.1 / 100 | – | – | – |
| GF(2¹²⁸) | **200** / **197** | – | 2,910 / 2,938 | 293 / 300 | 287 / 284 | 588 / 581 | 258 / 263 |
| GF(2²³³) | **1,096** / **1,075** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,116** / **1,051** | – | 9,581 / 9,582 | – | – | – | – |
| GF(2⁵⁷¹) | **4,875** / **4,717** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / **15.0** | – | 9.6 / 19.5 | 11.5 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 123 / 126 | 27.0 / 36.0 | – | – | – | – |
| GF(2⁶⁴) | **17.0** / **19.0** | 435 / 419 | 193 / 199 | 49.0 / 62.0 | – | – | – |
| GF(2¹²⁸) | **31.0** / **31.0** | – | 542 / 546 | 43.2 / 43.7 | 284 / 293 | 590 / 612 | 258 / 263 |
| GF(2²³³) | **60.2** / **59.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **64.1** / **63.0** | – | 1,533 / 1,450 | – | – | – | – |
| GF(2⁵⁷¹) | **208** / **246** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **136** / – | – | 572 / – | 277 / – | – | – |
| GF(2¹⁶) | **233** / – | 3,991 / – | 707 / – | – | – | – |
| GF(2⁶⁴) | **1,310** / – | 52,586 / – | 2,090 / – | 4,678 / – | – | – |
| GF(2¹²⁸) | **4,408** / – | – | 5,439 / – | – | 155,747 / – | 66,466 / – |
| GF(2²³³) | **24,485** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 29,410 / – | – | **15,994** / – | – | – | – |
| GF(2⁵⁷¹) | **161,689** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / – | – | 173 / – | 11.5 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,903 / – | 1,355 / – | – | – | – | – |
| GF(2⁶⁴) | **40.9** / – | 26,807 / – | 55,821 / – | 84.1 / – | – | – | – |
| GF(2¹²⁸) | **159** / – | – | 366,179 / – | 163 / – | 36,956 / – | 77,688 / – | 33,121 / – |
| GF(2²³³) | **132** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **149** / – | – | 2,447,405 / – | – | – | – | – |
| GF(2⁵⁷¹) | **2,527** / – | – | – | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | 6.4 / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15.4** / **12.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **10.9** | 5.2 / 18.0 | 80.3 / 80.2 | 2.6 / 11.8 | 2.6 / 11.8 | 21.0 / 29.4 | 19.5 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **6.2** / 31.0 | 127 / 127 | 224 / 223 | 9.8 / 31.9 | 9.0 / **24.0** | 83.3 / 101 | 19.0 / 33.0 | – | – | – | – | 10.8 / 43.2 |
| GF(2⁶⁴) | **72.0** / **73.1** | 427 / 421 | 613 / 604 | – | – | 886 / 887 | 95.6 / 100 | – | – | – | – | – |
| GF(2¹²⁸) | **200** / **197** | – | – | – | – | 2,929 / 2,928 | 298 / 299 | 283 / 284 | 643 / 580 | 595 / 580 | 258 / 263 | – |
| GF(2²³³) | **1,070** / **1,074** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,068** / **1,058** | – | – | – | – | 9,564 / 9,570 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **4,910** / **4,759** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / **5.3** | 3.6 / 16.0 | 72.9 / 61.4 | **2.2** / 7.9 | **2.2** / 7.9 | 9.6 / 19.5 | 10.9 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / **6.3** | 124 / 123 | 214 / 203 | 7.0 / 12.2 | 8.0 / 19.7 | 27.0 / 34.0 | 3.1 / 8.4 | – | – | – | – | 9.9 / 28.0 |
| GF(2⁶⁴) | **17.0** / **19.3** | 426 / 430 | 607 / 604 | – | – | 194 / 199 | 49.5 / 62.0 | – | – | – | – | – |
| GF(2¹²⁸) | **31.0** / **30.0** | – | – | – | – | 551 / 544 | 33.5 / 34.0 | 280 / 291 | 614 / 612 | 611 / 612 | 258 / 265 | – |
| GF(2²³³) | **61.0** / **59.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **62.0** / **63.0** | – | – | – | – | 1,665 / 1,506 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **151** / **235** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 4.0 / – | 392 / – | 32.4 / – | 32.9 / – | 571 / – | 277 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 3,888 / – | 2,269 / – | 661 / – | 587 / – | 707 / – | 32.6 / – | – | – | – | – | 9.5 / – |
| GF(2⁶⁴) | **895** / – | 52,633 / – | 23,801 / – | – | – | 2,088 / – | 4,677 / – | – | – | – | – | – |
| GF(2¹²⁸) | 2,680 / – | – | – | – | – | 5,437 / – | **1,319** / – | 16,663 / – | 155,750 / – | 155,785 / – | 65,852 / – | – |
| GF(2²³³) | **23,914** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 29,266 / – | – | – | – | – | **15,926** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **160,604** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.2** / – | 33.4 / – | 500 / – | 14.5 / – | 14.5 / – | 175 / – | 11.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 1,890 / – | 2,356 / – | 285 / – | 281 / – | 1,356 / – | 3.4 / – | – | – | – | – | 490 / – |
| GF(2⁶⁴) | **40.7** / – | 26,882 / – | 38,289 / – | – | – | 56,068 / – | 84.1 / – | – | – | – | – | – |
| GF(2¹²⁸) | **156** / – | – | – | – | – | 364,924 / – | 163 / – | 36,959 / – | 81,531 / – | 81,537 / – | 33,560 / – | – |
| GF(2²³³) | **132** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **186** / – | – | – | – | – | 2,443,528 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **2,533** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | **2.1** / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | **2.2** / **1.1** | – | – | – | – | **2.2** / **1.1** |
| GF(2⁶⁴) | **2.2** / **1.0** | **2.2** / **1.0** | **2.2** / **1.0** | – | – | **2.2** / **1.0** | **2.2** / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **3.3** / **2.1** | – | – | – | – | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | **3.3** / 2.3 | – |
| GF(2²³³) | **6.3** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **6.3** / **4.6** | – | – | – | – | 6.4 / **4.6** | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **15.4** / **12.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **15.5** / **21.0** | – | 21.0 / 29.4 | 19.5 / 23.0 | – | – | – |
| GF(2¹⁶) | **16.8** / **23.0** | 127 / 127 | 83.3 / 101 | – | – | – | – |
| GF(2⁶⁴) | **72.0** / **73.1** | 427 / 421 | 886 / 887 | 95.6 / 100 | – | – | – |
| GF(2¹²⁸) | **200** / **197** | – | 2,929 / 2,928 | 298 / 299 | 283 / 284 | 643 / 580 | 258 / 263 |
| GF(2²³³) | **1,070** / **1,073** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **1,073** / **1,057** | – | 9,564 / 9,570 | – | – | – | – |
| GF(2⁵⁷¹) | **4,860** / **4,773** | – | – | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.3** / **15.0** | – | 9.6 / 19.5 | 10.9 / 25.0 | – | – | – |
| GF(2¹⁶) | **10.4** / **23.0** | 124 / 123 | 27.0 / 34.0 | – | – | – | – |
| GF(2⁶⁴) | **17.0** / **19.3** | 426 / 430 | 194 / 199 | 49.5 / 62.0 | – | – | – |
| GF(2¹²⁸) | **31.0** / **31.0** | – | 551 / 544 | 33.5 / 34.0 | 280 / 291 | 614 / 612 | 258 / 265 |
| GF(2²³³) | **60.2** / **59.8** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **65.3** / **63.0** | – | 1,665 / 1,506 | – | – | – | – |
| GF(2⁵⁷¹) | **208** / **246** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **136** / – | – | 571 / – | 277 / – | – | – |
| GF(2¹⁶) | **233** / – | 3,888 / – | 707 / – | – | – | – |
| GF(2⁶⁴) | **1,312** / – | 52,633 / – | 2,088 / – | 4,677 / – | – | – |
| GF(2¹²⁸) | **4,380** / – | – | 5,437 / – | – | 155,750 / – | 65,852 / – |
| GF(2²³³) | **24,133** / – | – | – | – | – | – |
| GF(2²⁵⁶) | 29,289 / – | – | **15,926** / – | – | – | – |
| GF(2⁵⁷¹) | **160,724** / – | – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.7** / – | – | 175 / – | 11.5 / – | – | – | – |
| GF(2¹⁶) | **20.0** / – | 1,890 / – | 1,356 / – | – | – | – | – |
| GF(2⁶⁴) | **40.9** / – | 26,882 / – | 56,068 / – | 84.1 / – | – | – | – |
| GF(2¹²⁸) | **156** / – | – | 364,924 / – | 163 / – | 36,959 / – | 81,531 / – | 33,560 / – |
| GF(2²³³) | **133** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **181** / – | – | 2,443,528 / – | – | – | – | – |
| GF(2⁵⁷¹) | **2,549** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
