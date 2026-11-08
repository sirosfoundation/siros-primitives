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
| Run 1, with carry-less multiplication | `a0a67b8`, 2026-11-05: 190 cells; CPU 13; 1-minute load 16.46 before, 25.74 after; its SMT sibling, CPU 29, 57% busy |
| Run 1, without carry-less multiplication | `a0a67b8`, 2026-11-05: 190 cells; CPU 13; 1-minute load 26.75 before, 9.67 after; its SMT sibling, CPU 29, 12% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each native figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Every field in the polynomial basis of its modulus: GF(2⁸) of AES, GF(2¹²⁸) of GCM, GF(2²³³) and GF(2⁵⁷¹) of NIST B-233 and B-571.
- Every result matches a bit-serial reference before it is measured.
- The build without carry-less multiplication drops PCLMULQDQ and VPCLMULQDQ.
- Over GF(2¹⁶) without carry-less multiplication, `precomputed` multiplies through tables of logarithms and antilogarithms. That favours throughput: each product chained on the last runs through two loads from those large tables, so `reed-solomon-erasure` leads latency there and `binary-field` leads throughput.

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

`binary-field` at commit `47a91e3` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-08.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A counted loop runs behind a function pointer, which keeps V8 from inlining it into the code around the count.
- A batch loop reads its step and its end by volatile loads, so LLVM unrolls it for no operation, whatever the operation's size. A chain through locals over operand sets of more than 32 bytes reads its step the same way.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions. They start at a 4 KiB boundary in every case.
- A figure of cycles is the lower rate of the fewest run of n operations and the fewest of 2n, each less the fewest empty count: under load, a run of each length seldom meets an idle sibling hardware thread.
- A case whose 2n runs counted no more than its n runs, whose difference counted under 50 times the noise of the fixed cost of a count, or whose chain step counted under 0.75 cycles ran 31 more repetitions, up to 3 times. A case that stays so stops the sweep.
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
| Cost of a count, in WebAssembly, Chrome | 6,059 to 7,609 cycles and 3,081 instructions; its noise in a figure, 154 to 8,428 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 14,311 to 15,041 cycles and 3,081 instructions; its noise in a figure, 3,812 to 5,678 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 12,330 to 12,624 cycles and 1,885 instructions; its noise in a figure, 93.0 to 2,774 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 5,562 to 9,654 cycles and 1,885 instructions; its noise in a figure, 148 to 1,482 cycles |
| Run 1, in WebAssembly, Chrome | `5d2b0e1`, 2026-11-08: 190 cells; CPU 12; 1-minute load 28.34 before, 34.56 after; its SMT sibling, CPU 28, 30% busy |
| Run 1, in WebAssembly with wide arithmetic, Chrome | `5d2b0e1`, 2026-11-08: 190 cells; CPU 12; 1-minute load 34.56 before, 33.92 after; its SMT sibling, CPU 28, 59% busy |
| Run 1, in WebAssembly, Firefox | `5d2b0e1`, 2026-11-08: 190 cells; CPU 12; 1-minute load 32.8 before, 17.5 after; its SMT sibling, CPU 28, 44% busy |
| Run 1, in WebAssembly with wide arithmetic, Firefox | `5d2b0e1`, 2026-11-08: 190 cells; CPU 12; 1-minute load 17.5 before, 10.87 after; its SMT sibling, CPU 28, 6% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 2.6 vs gf256 2.6 | +0.0 (1.00×) | 2.7 vs reed-solomon-erasure 2.6 | +0.1 (1.04×) |
| add | GF(2¹⁶) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 2.6 vs gf256 2.7 | −0.1 (0.96×) | 2.6 vs gf256 2.6 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 2.8 vs gf256 2.8 | +0.0 (1.00×) | 2.8 vs gf256 2.8 | +0.0 (1.00×) | 2.2 vs gf256 2.1 | +0.1 (1.05×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 4.1 vs plonky3 4.0 | +0.1 (1.02×) | 4.1 vs hekate-math 4.1 | +0.0 (1.00×) | 3.5 vs polyval 3.4 | +0.1 (1.03×) | 3.4 vs hekate-math 3.5 | −0.1 (0.97×) |
| add | GF(2²³³) | 8.1 | – | 8.2 | – | 7.0 | – | 7.0 | – |
| add | GF(2²⁵⁶) | 8.1 vs hekate-math 8.3 | −0.2 (0.98×) | 8.1 vs hekate-math 8.4 | −0.3 (0.96×) | 7.2 vs hekate-math 7.2 | +0.0 (1.00×) | 7.2 vs hekate-math 7.2 | +0.0 (1.00×) |
| add | GF(2⁵⁷¹) | 24.9 | – | 25.0 | – | 15.9 | – | 15.8 | – |
| mul | GF(2⁸) | 3.2 vs g2p 3.8 | −0.6 (0.84×) | 3.2 vs g2p 3.8 | −0.6 (0.84×) | 2.7 vs g2p 3.4 | −0.7 (0.79×) | 2.7 vs g2p 3.4 | −0.7 (0.79×) |
| mul | GF(2¹⁶) | 6.8 vs galois_field_2pm lut 10.4 | −3.6 (0.65×) | 6.8 vs reed-solomon-erasure 10.5 | −3.7 (0.65×) | 6.8 vs reed-solomon-erasure 10.2 | −3.4 (0.67×) | 6.7 vs reed-solomon-erasure 10.1 | −3.4 (0.66×) |
| mul | GF(2⁶⁴) | 66.8 vs plonky3 88.4 | −21.6 (0.76×) | 66.9 vs plonky3 87.8 | −20.9 (0.76×) | 70.1 vs plonky3 100 | −29.9 (0.70×) | 72.5 vs plonky3 100 | −27.5 (0.72×) |
| mul | GF(2¹²⁸) | 181 vs polyval 230 | −49.0 (0.79×) | 181 vs polyval 231 | −50.0 (0.78×) | 201 vs polyval 257 | −56.0 (0.78×) | 201 vs polyval 257 | −56.0 (0.78×) |
| mul | GF(2²³³) | 718 | – | 755 | – | 1,072 | – | 1,065 | – |
| mul | GF(2²⁵⁶) | 705 vs hekate-math 8,424 | −7,719 (0.08×) | 708 vs hekate-math 8,409 | −7,701 (0.08×) | 1,089 vs hekate-math 9,474 | −8,385 (0.11×) | 1,091 vs hekate-math 9,466 | −8,375 (0.12×) |
| mul | GF(2⁵⁷¹) | 4,012 | – | 4,009 | – | 4,865 | – | 4,840 | – |
| square | GF(2⁸) | 3.0 vs g2p 3.1 | −0.1 (0.97×) | 3.0 vs g2p 3.1 | −0.1 (0.97×) | 2.6 vs g2p 2.8 | −0.2 (0.93×) | 2.6 vs g2p 2.8 | −0.2 (0.93×) |
| square | GF(2¹⁶) | 3.3 vs plonky3 4.0 | −0.7 (0.82×) | 3.3 vs plonky3 3.9 | −0.6 (0.85×) | 2.8 vs plonky3 3.7 | −0.9 (0.76×) | 2.9 vs plonky3 3.7 | −0.8 (0.78×) |
| square | GF(2⁶⁴) | 15.0 vs plonky3 48.1 | −33.1 (0.31×) | 15.0 vs plonky3 47.9 | −32.9 (0.31×) | 16.0 vs plonky3 50.1 | −34.1 (0.32×) | 16.0 vs plonky3 50.0 | −34.0 (0.32×) |
| square | GF(2¹²⁸) | 27.7 vs plonky3 35.9 | −8.2 (0.77×) | 27.7 vs plonky3 30.8 | −3.1 (0.90×) | 30.0 vs plonky3 42.2 | −12.2 (0.71×) | 30.0 vs plonky3 33.7 | −3.7 (0.89×) |
| square | GF(2²³³) | 54.9 | – | 54.6 | – | 58.7 | – | 58.9 | – |
| square | GF(2²⁵⁶) | 60.1 vs hekate-math 1,398 | −1,338 (0.04×) | 59.8 vs hekate-math 1,387 | −1,327 (0.04×) | 64.1 vs hekate-math 1,571 | −1,507 (0.04×) | 64.1 vs hekate-math 1,571 | −1,507 (0.04×) |
| square | GF(2⁵⁷¹) | 144 | – | 144 | – | 149 | – | 149 | – |
| invert | GF(2⁸) | 3.1 vs gf256 4.6 | −1.5 (0.67×) | 3.1 vs gf256 4.6 | −1.5 (0.67×) | 2.6 vs gf256 4.2 | −1.6 (0.62×) | 2.6 vs gf256 4.2 | −1.6 (0.62×) |
| invert | GF(2¹⁶) | 3.3 vs galois_field_2pm lut 9.3 | −6.0 (0.35×) | 3.3 vs galois_field_2pm lut 9.2 | −5.9 (0.36×) | 2.8 vs galois_field_2pm lut 9.7 | −6.9 (0.29×) | 2.9 vs galois_field_2pm lut 9.7 | −6.8 (0.30×) |
| invert | GF(2⁶⁴) | 396 vs hekate-math 1,579 | −1,183 (0.25×) | 398 vs hekate-math 1,573 | −1,175 (0.25×) | 409 vs hekate-math 2,057 | −1,648 (0.20×) | 406 vs hekate-math 2,099 | −1,693 (0.19×) |
| invert | GF(2¹²⁸) | 996 vs plonky3 1,226 | −230 (0.81×) | 925 vs plonky3 1,252 | −327 (0.74×) | 963 vs plonky3 1,317 | −354 (0.73×) | 963 vs plonky3 1,320 | −357 (0.73×) |
| invert | GF(2²³³) | 14,635 | – | 14,720 | – | 19,576 | – | 20,288 | – |
| invert | GF(2²⁵⁶) | 3,620 vs hekate-math 13,712 | −10,092 (0.26×) | 3,742 vs hekate-math 13,703 | −9,961 (0.27×) | 4,529 vs hekate-math 15,807 | −11,278 (0.29×) | 4,520 vs hekate-math 15,810 | −11,290 (0.29×) |
| invert | GF(2⁵⁷¹) | 130,441 | – | 130,301 | – | 160,902 | – | 160,967 | – |
| sqrt | GF(2⁸) | 3.0 vs plonky3 11.3 | −8.3 (0.27×) | 3.1 vs plonky3 11.3 | −8.2 (0.27×) | 2.6 vs plonky3 11.0 | −8.4 (0.24×) | 2.6 vs plonky3 11.0 | −8.4 (0.24×) |
| sqrt | GF(2¹⁶) | 3.3 vs plonky3 3.7 | −0.4 (0.89×) | 3.4 vs plonky3 4.0 | −0.6 (0.85×) | 3.0 vs plonky3 3.7 | −0.7 (0.81×) | 2.8 vs plonky3 3.7 | −0.9 (0.76×) |
| sqrt | GF(2⁶⁴) | 40.1 vs plonky3 84.2 | −44.1 (0.48×) | 40.2 vs plonky3 84.0 | −43.8 (0.48×) | 40.1 vs plonky3 87.1 | −47.0 (0.46×) | 40.1 vs plonky3 91.1 | −51.0 (0.44×) |
| sqrt | GF(2¹²⁸) | 43.0 vs plonky3 156 | −113 (0.28×) | 42.8 vs plonky3 156 | −113 (0.27×) | 42.2 vs plonky3 163 | −121 (0.26×) | 42.1 vs plonky3 164 | −122 (0.26×) |
| sqrt | GF(2²³³) | 112 | – | 112 | – | 132 | – | 132 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,151,886 | −2,151,764 (0.00×) | 122 vs hekate-math 2,153,817 | −2,153,695 (0.00×) | 156 vs hekate-math 2,419,747 | −2,419,591 (0.00×) | 156 vs hekate-math 2,413,419 | −2,413,263 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,187 | – | 2,182 | – | 2,508 | – | 2,510 | – |
| add, constant time | GF(2⁸) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 2.6 vs gf256 2.6 | +0.0 (1.00×) | 2.7 vs reed-solomon-erasure 2.6 | +0.1 (1.04×) |
| add, constant time | GF(2¹⁶) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 3.1 vs gf256 3.1 | +0.0 (1.00×) | 2.7 vs gf256 2.7 | +0.0 (1.00×) | 2.6 vs gf256 2.6 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 2.8 vs gf256 2.8 | +0.0 (1.00×) | 2.8 vs gf256 2.8 | +0.0 (1.00×) | 2.2 vs gf256 2.1 | +0.1 (1.05×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 4.1 vs plonky3 4.0 | +0.1 (1.02×) | 4.1 vs hekate-math 4.1 | +0.0 (1.00×) | 3.4 vs polyval 3.4 | +0.0 (1.00×) | 3.4 vs hekate-math 3.5 | −0.1 (0.97×) |
| add, constant time | GF(2²³³) | 8.1 | – | 8.1 | – | 7.2 | – | 7.2 | – |
| add, constant time | GF(2²⁵⁶) | 8.1 vs hekate-math 8.3 | −0.2 (0.98×) | 8.1 vs hekate-math 8.4 | −0.3 (0.96×) | 7.2 vs hekate-math 7.2 | +0.0 (1.00×) | 7.2 vs hekate-math 7.2 | +0.0 (1.00×) |
| add, constant time | GF(2⁵⁷¹) | 24.9 | – | 25.0 | – | 16.1 | – | 15.9 | – |
| mul, constant time | GF(2⁸) | 12.8 vs plonky3 19.1 | −6.3 (0.67×) | 12.9 vs plonky3 19.3 | −6.4 (0.67×) | 13.2 vs plonky3 20.1 | −6.9 (0.66×) | 13.5 vs plonky3 20.0 | −6.5 (0.68×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 80.1 | −63.6 (0.21×) | 16.3 vs hekate-math 79.9 | −63.6 (0.20×) | 16.6 vs hekate-math 82.8 | −66.2 (0.20×) | 16.1 vs hekate-math 82.5 | −66.4 (0.20×) |
| mul, constant time | GF(2⁶⁴) | 67.0 vs plonky3 88.4 | −21.4 (0.76×) | 66.8 vs plonky3 87.8 | −21.0 (0.76×) | 70.1 vs plonky3 100 | −29.9 (0.70×) | 72.1 vs plonky3 100 | −27.9 (0.72×) |
| mul, constant time | GF(2¹²⁸) | 181 vs polyval 230 | −49.0 (0.79×) | 180 vs polyval 231 | −51.0 (0.78×) | 203 vs polyval 257 | −54.0 (0.79×) | 201 vs polyval 257 | −56.0 (0.78×) |
| mul, constant time | GF(2²³³) | 758 | – | 717 | – | 1,067 | – | 1,058 | – |
| mul, constant time | GF(2²⁵⁶) | 706 vs hekate-math 8,424 | −7,718 (0.08×) | 705 vs hekate-math 8,409 | −7,704 (0.08×) | 1,106 vs hekate-math 9,474 | −8,368 (0.12×) | 1,093 vs hekate-math 9,466 | −8,373 (0.12×) |
| mul, constant time | GF(2⁵⁷¹) | 4,066 | – | 4,025 | – | 4,900 | – | 4,903 | – |
| square, constant time | GF(2⁸) | 5.6 vs hekate-math 10.3 | −4.7 (0.54×) | 5.6 vs hekate-math 10.3 | −4.7 (0.54×) | 6.2 vs hekate-math 10.0 | −3.8 (0.62×) | 5.8 vs hekate-math 9.8 | −4.0 (0.59×) |
| square, constant time | GF(2¹⁶) | 11.5 vs hekate-math 25.9 | −14.4 (0.44×) | 11.5 vs hekate-math 25.8 | −14.3 (0.45×) | 11.0 vs hekate-math 27.0 | −16.0 (0.41×) | 11.0 vs hekate-math 27.0 | −16.0 (0.41×) |
| square, constant time | GF(2⁶⁴) | 15.0 vs plonky3 48.1 | −33.1 (0.31×) | 15.0 vs plonky3 47.9 | −32.9 (0.31×) | 16.0 vs plonky3 50.1 | −34.1 (0.32×) | 16.0 vs plonky3 50.0 | −34.0 (0.32×) |
| square, constant time | GF(2¹²⁸) | 27.7 vs plonky3 35.9 | −8.2 (0.77×) | 27.7 vs plonky3 30.8 | −3.1 (0.90×) | 30.0 vs plonky3 42.2 | −12.2 (0.71×) | 31.0 vs plonky3 33.7 | −2.7 (0.92×) |
| square, constant time | GF(2²³³) | 54.8 | – | 54.5 | – | 58.8 | – | 58.9 | – |
| square, constant time | GF(2²⁵⁶) | 60.2 vs hekate-math 1,398 | −1,338 (0.04×) | 59.8 vs hekate-math 1,387 | −1,327 (0.04×) | 64.1 vs hekate-math 1,571 | −1,507 (0.04×) | 64.0 vs hekate-math 1,571 | −1,507 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 144 | – | 144 | – | 147 | – | 147 | – |
| invert, constant time | GF(2⁸) | 129 vs plonky3 275 | −146 (0.47×) | 129 vs plonky3 274 | −145 (0.47×) | 131 vs plonky3 281 | −150 (0.47×) | 131 vs plonky3 280 | −149 (0.47×) |
| invert, constant time | GF(2¹⁶) | 238 vs hekate-math 394 | −156 (0.60×) | 237 vs hekate-math 393 | −156 (0.60×) | 236 vs hekate-math 717 | −481 (0.33×) | 236 vs hekate-math 717 | −481 (0.33×) |
| invert, constant time | GF(2⁶⁴) | 1,306 vs hekate-math 1,579 | −273 (0.83×) | 1,305 vs hekate-math 1,573 | −268 (0.83×) | 1,324 vs hekate-math 2,057 | −733 (0.64×) | 1,310 vs hekate-math 2,099 | −789 (0.62×) |
| invert, constant time | GF(2¹²⁸) | 2,913 vs hekate-math 4,508 | −1,595 (0.65×) | 2,886 vs hekate-math 4,509 | −1,623 (0.64×) | 2,626 vs hekate-math 5,395 | −2,769 (0.49×) | 2,618 vs hekate-math 5,393 | −2,775 (0.49×) |
| invert, constant time | GF(2²³³) | 19,074 | – | 19,068 | – | 19,953 | – | 19,652 | – |
| invert, constant time | GF(2²⁵⁶) | 8,255 vs hekate-math 13,712 | −5,457 (0.60×) | 8,245 vs hekate-math 13,703 | −5,458 (0.60×) | 7,855 vs hekate-math 15,807 | −7,952 (0.50×) | 7,909 vs hekate-math 15,810 | −7,901 (0.50×) |
| invert, constant time | GF(2⁵⁷¹) | 130,436 | – | 130,330 | – | 161,020 | – | 161,102 | – |
| sqrt, constant time | GF(2⁸) | 5.7 vs plonky3 11.3 | −5.6 (0.50×) | 5.6 vs plonky3 11.3 | −5.7 (0.50×) | 6.2 vs plonky3 11.0 | −4.8 (0.56×) | 6.2 vs plonky3 11.0 | −4.8 (0.56×) |
| sqrt, constant time | GF(2¹⁶) | 17.0 vs hekate-math 1,298 | −1,281 (0.01×) | 17.0 vs hekate-math 1,293 | −1,276 (0.01×) | 19.0 vs hekate-math 1,294 | −1,275 (0.01×) | 19.0 vs hekate-math 1,292 | −1,273 (0.01×) |
| sqrt, constant time | GF(2⁶⁴) | 40.0 vs plonky3 84.2 | −44.2 (0.48×) | 38.9 vs plonky3 84.0 | −45.1 (0.46×) | 40.1 vs plonky3 87.1 | −47.0 (0.46×) | 40.0 vs plonky3 91.1 | −51.1 (0.44×) |
| sqrt, constant time | GF(2¹²⁸) | 42.9 vs plonky3 156 | −113 (0.27×) | 43.0 vs plonky3 156 | −113 (0.28×) | 42.1 vs plonky3 163 | −121 (0.26×) | 42.1 vs plonky3 164 | −122 (0.26×) |
| sqrt, constant time | GF(2²³³) | 120 | – | 112 | – | 131 | – | 131 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,151,886 | −2,151,764 (0.00×) | 121 vs hekate-math 2,153,817 | −2,153,696 (0.00×) | 156 vs hekate-math 2,419,747 | −2,419,591 (0.00×) | 156 vs hekate-math 2,413,419 | −2,413,263 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,183 | – | 2,184 | – | 2,509 | – | 2,506 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs gf256 1.2 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 1.4 vs gf256 1.4 | +0.0 (1.00×) | 1.4 vs gf256 1.4 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 2.3 vs polyval 2.3 | +0.0 (1.00×) | 2.3 vs runtime-algebra 2.4 | −0.1 (0.96×) | 2.2 vs compile-algebra 2.1 | +0.1 (1.05×) | 2.2 vs hekate-math 2.2 | +0.0 (1.00×) |
| add | GF(2²³³) | 4.8 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 4.8 vs hekate-math 6.3 | −1.5 (0.76×) | 4.8 vs hekate-math 6.3 | −1.5 (0.76×) | 4.6 vs hekate-math 4.7 | −0.1 (0.98×) | 4.6 vs hekate-math 4.7 | −0.1 (0.98×) |
| add | GF(2⁵⁷¹) | 18.6 | – | 18.7 | – | 12.6 | – | 12.5 | – |
| mul | GF(2⁸) | 10.5 vs g2p 11.8 | −1.3 (0.89×) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) |
| mul | GF(2¹⁶) | 30.5 vs reed-solomon-erasure 24.7 | +5.8 (1.23×) | 30.4 vs reed-solomon-erasure 25.0 | +5.4 (1.22×) | 31.1 vs reed-solomon-erasure 24.0 | +7.1 (1.30×) | 31.0 vs reed-solomon-erasure 24.7 | +6.3 (1.26×) |
| mul | GF(2⁶⁴) | 72.1 vs plonky3 96.4 | −24.3 (0.75×) | 71.8 vs plonky3 86.6 | −14.8 (0.83×) | 76.4 vs plonky3 100 | −23.6 (0.76×) | 75.2 vs plonky3 100 | −24.8 (0.75×) |
| mul | GF(2¹²⁸) | 184 vs polyval 230 | −46.0 (0.80×) | 184 vs polyval 228 | −44.0 (0.81×) | 197 vs polyval 264 | −67.0 (0.75×) | 200 vs polyval 263 | −63.0 (0.76×) |
| mul | GF(2²³³) | 722 | – | 761 | – | 1,069 | – | 1,075 | – |
| mul | GF(2²⁵⁶) | 704 vs hekate-math 8,436 | −7,732 (0.08×) | 701 vs hekate-math 8,429 | −7,728 (0.08×) | 1,070 vs hekate-math 9,482 | −8,412 (0.11×) | 1,069 vs hekate-math 9,460 | −8,391 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,027 | – | 4,025 | – | 4,763 | – | 4,783 | – |
| square | GF(2⁸) | 5.1 vs g2p 7.4 | −2.3 (0.69×) | 5.1 vs g2p 7.8 | −2.7 (0.65×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs plonky3 8.5 | −2.2 (0.74×) | 6.3 vs plonky3 8.3 | −2.0 (0.76×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.1 | −40.1 (0.32×) | 18.9 vs plonky3 58.9 | −40.0 (0.32×) | 19.0 vs plonky3 62.1 | −43.1 (0.31×) | 19.0 vs plonky3 62.1 | −43.1 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 36.5 | −9.5 (0.74×) | 26.9 vs plonky3 31.9 | −5.0 (0.84×) | 30.0 vs plonky3 45.4 | −15.4 (0.66×) | 30.0 vs plonky3 35.0 | −5.0 (0.86×) |
| square | GF(2²³³) | 53.5 | – | 53.1 | – | 59.8 | – | 59.7 | – |
| square | GF(2²⁵⁶) | 59.1 vs hekate-math 1,381 | −1,322 (0.04×) | 58.7 vs hekate-math 1,378 | −1,319 (0.04×) | 63.1 vs hekate-math 1,529 | −1,466 (0.04×) | 63.1 vs hekate-math 1,527 | −1,464 (0.04×) |
| square | GF(2⁵⁷¹) | 162 | – | 161 | – | 181 | – | 181 | – |
| add, constant time | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs gf256 1.2 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 1.4 vs gf256 1.4 | +0.0 (1.00×) | 1.4 vs gf256 1.4 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs polyval 2.3 | +0.0 (1.00×) | 2.3 vs runtime-algebra 2.4 | −0.1 (0.96×) | 2.2 vs compile-algebra 2.1 | +0.1 (1.05×) | 2.2 vs hekate-math 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2²³³) | 4.8 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add, constant time | GF(2²⁵⁶) | 4.8 vs hekate-math 6.3 | −1.5 (0.76×) | 4.8 vs hekate-math 6.3 | −1.5 (0.76×) | 4.6 vs hekate-math 4.7 | −0.1 (0.98×) | 4.6 vs hekate-math 4.7 | −0.1 (0.98×) |
| add, constant time | GF(2⁵⁷¹) | 18.6 | – | 18.7 | – | 12.6 | – | 12.3 | – |
| mul, constant time | GF(2⁸) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) |
| mul, constant time | GF(2¹⁶) | 21.7 vs hekate-math 82.1 | −60.4 (0.26×) | 21.6 vs hekate-math 81.8 | −60.2 (0.26×) | 23.0 vs hekate-math 101 | −78.0 (0.23×) | 23.0 vs hekate-math 99.7 | −76.7 (0.23×) |
| mul, constant time | GF(2⁶⁴) | 72.1 vs plonky3 96.4 | −24.3 (0.75×) | 71.8 vs plonky3 86.6 | −14.8 (0.83×) | 76.1 vs plonky3 100 | −23.9 (0.76×) | 75.2 vs plonky3 100 | −24.8 (0.75×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 230 | −46.0 (0.80×) | 184 vs polyval 228 | −44.0 (0.81×) | 198 vs polyval 264 | −66.0 (0.75×) | 197 vs polyval 263 | −66.0 (0.75×) |
| mul, constant time | GF(2²³³) | 754 | – | 756 | – | 1,070 | – | 1,072 | – |
| mul, constant time | GF(2²⁵⁶) | 703 vs hekate-math 8,436 | −7,733 (0.08×) | 702 vs hekate-math 8,429 | −7,727 (0.08×) | 1,079 vs hekate-math 9,482 | −8,403 (0.11×) | 1,080 vs hekate-math 9,460 | −8,380 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,041 | – | 4,021 | – | 4,766 | – | 4,767 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.4 | −4.4 (0.77×) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.9 | −4.9 (0.75×) | 15.0 vs hekate-math 19.9 | −4.9 (0.75×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.1 | −11.1 (0.67×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 33.5 | −10.5 (0.69×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.1 | −40.1 (0.32×) | 18.9 vs plonky3 58.9 | −40.0 (0.32×) | 19.0 vs plonky3 62.1 | −43.1 (0.31×) | 19.0 vs plonky3 62.1 | −43.1 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.4 vs plonky3 36.5 | −9.1 (0.75×) | 27.3 vs plonky3 31.9 | −4.6 (0.86×) | 30.0 vs plonky3 45.4 | −15.4 (0.66×) | 30.0 vs plonky3 35.0 | −5.0 (0.86×) |
| square, constant time | GF(2²³³) | 53.4 | – | 53.1 | – | 59.8 | – | 59.8 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,381 | −1,322 (0.04×) | 58.7 vs hekate-math 1,378 | −1,319 (0.04×) | 63.0 vs hekate-math 1,529 | −1,466 (0.04×) | 63.0 vs hekate-math 1,527 | −1,464 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 162 | – | 161 | – | 181 | – | 181 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 3.1 (2.1×) | 3.1 (2.1×) | 2.6 (1.7×) | 2.7 (1.8×) |
| add | GF(2¹⁶) | 1.5 | 3.1 (2.1×) | 3.1 (2.1×) | 2.6 (1.7×) | 2.6 (1.7×) |
| add | GF(2⁶⁴) | 1.3 | 2.8 (2.2×) | 2.8 (2.2×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add | GF(2¹²⁸) | 1.3 | 4.1 (3.2×) | 4.1 (3.2×) | 3.5 (2.7×) | 3.4 (2.6×) |
| add | GF(2²³³) | 2.3 | 8.1 (3.5×) | 8.2 (3.6×) | 7.0 (3.0×) | 7.0 (3.0×) |
| add | GF(2²⁵⁶) | 2.6 | 8.1 (3.1×) | 8.1 (3.1×) | 7.2 (2.8×) | 7.2 (2.8×) |
| add | GF(2⁵⁷¹) | 6.7 | 24.9 (3.7×) | 25.0 (3.7×) | 15.9 (2.4×) | 15.8 (2.4×) |
| mul | GF(2⁸) | 2.0 | 3.2 (1.6×) | 3.2 (1.6×) | 2.7 (1.4×) | 2.7 (1.4×) |
| mul | GF(2¹⁶) | 5.3 | 6.8 (1.3×) | 6.8 (1.3×) | 6.8 (1.3×) | 6.7 (1.3×) |
| mul | GF(2⁶⁴) | 31.6 | 66.8 (2.1×) | 66.9 (2.1×) | 70.1 (2.2×) | 72.5 (2.3×) |
| mul | GF(2¹²⁸) | 54.2 | 181 (3.3×) | 181 (3.3×) | 201 (3.7×) | 201 (3.7×) |
| mul | GF(2²³³) | 202 | 718 (3.6×) | 755 (3.7×) | 1,072 (5.3×) | 1,065 (5.3×) |
| mul | GF(2²⁵⁶) | 189 | 705 (3.7×) | 708 (3.7×) | 1,089 (5.8×) | 1,091 (5.8×) |
| mul | GF(2⁵⁷¹) | 1,290 | 4,012 (3.1×) | 4,009 (3.1×) | 4,865 (3.8×) | 4,840 (3.8×) |
| square | GF(2⁸) | 1.2 | 3.0 (2.5×) | 3.0 (2.5×) | 2.6 (2.2×) | 2.6 (2.2×) |
| square | GF(2¹⁶) | 1.7 | 3.3 (1.9×) | 3.3 (1.9×) | 2.8 (1.6×) | 2.9 (1.7×) |
| square | GF(2⁶⁴) | 5.4 | 15.0 (2.8×) | 15.0 (2.8×) | 16.0 (3.0×) | 16.0 (3.0×) |
| square | GF(2¹²⁸) | 6.6 | 27.7 (4.2×) | 27.7 (4.2×) | 30.0 (4.5×) | 30.0 (4.5×) |
| square | GF(2²³³) | 21.1 | 54.9 (2.6×) | 54.6 (2.6×) | 58.7 (2.8×) | 58.9 (2.8×) |
| square | GF(2²⁵⁶) | 20.4 | 60.1 (2.9×) | 59.8 (2.9×) | 64.1 (3.1×) | 64.1 (3.1×) |
| square | GF(2⁵⁷¹) | 55.0 | 144 (2.6×) | 144 (2.6×) | 149 (2.7×) | 149 (2.7×) |
| invert | GF(2⁸) | 1.2 | 3.1 (2.6×) | 3.1 (2.6×) | 2.6 (2.2×) | 2.6 (2.2×) |
| invert | GF(2¹⁶) | 1.7 | 3.3 (1.9×) | 3.3 (1.9×) | 2.8 (1.6×) | 2.9 (1.7×) |
| invert | GF(2⁶⁴) | 532 | 396 (0.7×) | 398 (0.7×) | 409 (0.8×) | 406 (0.8×) |
| invert | GF(2¹²⁸) | 832 | 996 (1.2×) | 925 (1.1×) | 963 (1.2×) | 963 (1.2×) |
| invert | GF(2²³³) | 9,362 | 14,635 (1.6×) | 14,720 (1.6×) | 19,576 (2.1×) | 20,288 (2.2×) |
| invert | GF(2²⁵⁶) | 8,693 | 3,620 (0.4×) | 3,742 (0.4×) | 4,529 (0.5×) | 4,520 (0.5×) |
| invert | GF(2⁵⁷¹) | 48,582 | 130,441 (2.7×) | 130,301 (2.7×) | 160,902 (3.3×) | 160,967 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 3.0 (2.5×) | 3.1 (2.6×) | 2.6 (2.2×) | 2.6 (2.2×) |
| sqrt | GF(2¹⁶) | 1.7 | 3.3 (1.9×) | 3.4 (2.0×) | 3.0 (1.8×) | 2.8 (1.6×) |
| sqrt | GF(2⁶⁴) | 35.6 | 40.1 (1.1×) | 40.2 (1.1×) | 40.1 (1.1×) | 40.1 (1.1×) |
| sqrt | GF(2¹²⁸) | 65.6 | 43.0 (0.7×) | 42.8 (0.7×) | 42.2 (0.6×) | 42.1 (0.6×) |
| sqrt | GF(2²³³) | 110 | 112 (1.0×) | 112 (1.0×) | 132 (1.2×) | 132 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 122 (1.3×) | 156 (1.7×) | 156 (1.7×) |
| sqrt | GF(2⁵⁷¹) | 674 | 2,187 (3.2×) | 2,182 (3.2×) | 2,508 (3.7×) | 2,510 (3.7×) |
| add, constant time | GF(2⁸) | 1.5 | 3.1 (2.1×) | 3.1 (2.1×) | 2.6 (1.7×) | 2.7 (1.8×) |
| add, constant time | GF(2¹⁶) | 1.5 | 3.1 (2.1×) | 3.1 (2.1×) | 2.7 (1.8×) | 2.6 (1.7×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.8 (2.2×) | 2.8 (2.2×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add, constant time | GF(2¹²⁸) | 1.3 | 4.1 (3.2×) | 4.1 (3.2×) | 3.4 (2.6×) | 3.4 (2.6×) |
| add, constant time | GF(2²³³) | 2.6 | 8.1 (3.1×) | 8.1 (3.1×) | 7.2 (2.8×) | 7.2 (2.8×) |
| add, constant time | GF(2²⁵⁶) | 2.5 | 8.1 (3.2×) | 8.1 (3.2×) | 7.2 (2.9×) | 7.2 (2.9×) |
| add, constant time | GF(2⁵⁷¹) | 7.2 | 24.9 (3.5×) | 25.0 (3.5×) | 16.1 (2.2×) | 15.9 (2.2×) |
| mul, constant time | GF(2⁸) | 12.1 | 12.8 (1.1×) | 12.9 (1.1×) | 13.2 (1.1×) | 13.5 (1.1×) |
| mul, constant time | GF(2¹⁶) | 15.1 | 16.5 (1.1×) | 16.3 (1.1×) | 16.6 (1.1×) | 16.1 (1.1×) |
| mul, constant time | GF(2⁶⁴) | 31.6 | 67.0 (2.1×) | 66.8 (2.1×) | 70.1 (2.2×) | 72.1 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.3 | 181 (3.3×) | 180 (3.3×) | 203 (3.7×) | 201 (3.7×) |
| mul, constant time | GF(2²³³) | 199 | 758 (3.8×) | 717 (3.6×) | 1,067 (5.4×) | 1,058 (5.3×) |
| mul, constant time | GF(2²⁵⁶) | 189 | 706 (3.7×) | 705 (3.7×) | 1,106 (5.9×) | 1,093 (5.8×) |
| mul, constant time | GF(2⁵⁷¹) | 1,290 | 4,066 (3.2×) | 4,025 (3.1×) | 4,900 (3.8×) | 4,903 (3.8×) |
| square, constant time | GF(2⁸) | 3.5 | 5.6 (1.6×) | 5.6 (1.6×) | 6.2 (1.8×) | 5.8 (1.7×) |
| square, constant time | GF(2¹⁶) | 8.6 | 11.5 (1.3×) | 11.5 (1.3×) | 11.0 (1.3×) | 11.0 (1.3×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 15.0 (2.8×) | 15.0 (2.8×) | 16.0 (3.0×) | 16.0 (3.0×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 27.7 (4.2×) | 27.7 (4.2×) | 30.0 (4.5×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 21.0 | 54.8 (2.6×) | 54.5 (2.6×) | 58.8 (2.8×) | 58.9 (2.8×) |
| square, constant time | GF(2²⁵⁶) | 20.3 | 60.2 (3.0×) | 59.8 (2.9×) | 64.1 (3.2×) | 64.0 (3.2×) |
| square, constant time | GF(2⁵⁷¹) | 54.9 | 144 (2.6×) | 144 (2.6×) | 147 (2.7×) | 147 (2.7×) |
| invert, constant time | GF(2⁸) | 89.8 | 129 (1.4×) | 129 (1.4×) | 131 (1.5×) | 131 (1.5×) |
| invert, constant time | GF(2¹⁶) | 227 | 238 (1.0×) | 237 (1.0×) | 236 (1.0×) | 236 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 569 | 1,306 (2.3×) | 1,305 (2.3×) | 1,324 (2.3×) | 1,310 (2.3×) |
| invert, constant time | GF(2¹²⁸) | 1,864 | 2,913 (1.6×) | 2,886 (1.5×) | 2,626 (1.4×) | 2,618 (1.4×) |
| invert, constant time | GF(2²³³) | 9,332 | 19,074 (2.0×) | 19,068 (2.0×) | 19,953 (2.1×) | 19,652 (2.1×) |
| invert, constant time | GF(2²⁵⁶) | 8,643 | 8,255 (1.0×) | 8,245 (1.0×) | 7,855 (0.9×) | 7,909 (0.9×) |
| invert, constant time | GF(2⁵⁷¹) | 48,617 | 130,436 (2.7×) | 130,330 (2.7×) | 161,020 (3.3×) | 161,102 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.7 (1.6×) | 5.6 (1.6×) | 6.2 (1.8×) | 6.2 (1.8×) |
| sqrt, constant time | GF(2¹⁶) | 13.5 | 17.0 (1.3×) | 17.0 (1.3×) | 19.0 (1.4×) | 19.0 (1.4×) |
| sqrt, constant time | GF(2⁶⁴) | 35.6 | 40.0 (1.1×) | 38.9 (1.1×) | 40.1 (1.1×) | 40.0 (1.1×) |
| sqrt, constant time | GF(2¹²⁸) | 65.7 | 42.9 (0.7×) | 43.0 (0.7×) | 42.1 (0.6×) | 42.1 (0.6×) |
| sqrt, constant time | GF(2²³³) | 110 | 120 (1.1×) | 112 (1.0×) | 131 (1.2×) | 131 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 121 (1.3×) | 156 (1.7×) | 156 (1.7×) |
| sqrt, constant time | GF(2⁵⁷¹) | 674 | 2,183 (3.2×) | 2,184 (3.2×) | 2,509 (3.7×) | 2,506 (3.7×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) |
| add | GF(2⁶⁴) | 1.1 | 1.4 (1.3×) | 1.4 (1.3×) | 1.0 (0.9×) | 1.1 (1.0×) |
| add | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.2 (2.0×) | 2.2 (2.0×) |
| add | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2⁵⁷¹) | 5.0 | 18.6 (3.7×) | 18.7 (3.7×) | 12.6 (2.5×) | 12.5 (2.5×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.5 (1.1×) | 10.9 (1.2×) | 10.9 (1.2×) |
| mul | GF(2¹⁶) | 32.2 | 30.5 (0.9×) | 30.4 (0.9×) | 31.1 (1.0×) | 31.0 (1.0×) |
| mul | GF(2⁶⁴) | 41.7 | 72.1 (1.7×) | 71.8 (1.7×) | 76.4 (1.8×) | 75.2 (1.8×) |
| mul | GF(2¹²⁸) | 64.9 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 200 (3.1×) |
| mul | GF(2²³³) | 201 | 722 (3.6×) | 761 (3.8×) | 1,069 (5.3×) | 1,075 (5.3×) |
| mul | GF(2²⁵⁶) | 190 | 704 (3.7×) | 701 (3.7×) | 1,070 (5.6×) | 1,069 (5.6×) |
| mul | GF(2⁵⁷¹) | 1,281 | 4,027 (3.1×) | 4,025 (3.1×) | 4,763 (3.7×) | 4,783 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 18.9 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 26.9 (1.9×) | 30.0 (2.1×) | 30.0 (2.1×) |
| square | GF(2²³³) | 32.0 | 53.5 (1.7×) | 53.1 (1.7×) | 59.8 (1.9×) | 59.7 (1.9×) |
| square | GF(2²⁵⁶) | 27.0 | 59.1 (2.2×) | 58.7 (2.2×) | 63.1 (2.3×) | 63.1 (2.3×) |
| square | GF(2⁵⁷¹) | 57.0 | 162 (2.8×) | 161 (2.8×) | 181 (3.2×) | 181 (3.2×) |
| add, constant time | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.4 (1.3×) | 1.4 (1.3×) | 1.0 (0.9×) | 1.1 (1.0×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.2 (2.0×) | 2.2 (2.0×) |
| add, constant time | GF(2²³³) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add, constant time | GF(2²⁵⁶) | 2.0 | 4.8 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add, constant time | GF(2⁵⁷¹) | 5.0 | 18.6 (3.7×) | 18.7 (3.7×) | 12.6 (2.5×) | 12.3 (2.5×) |
| mul, constant time | GF(2⁸) | 21.0 | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 21.7 (1.0×) | 21.6 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.6 | 72.1 (1.7×) | 71.8 (1.7×) | 76.1 (1.8×) | 75.2 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 184 (2.8×) | 198 (3.1×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 207 | 754 (3.6×) | 756 (3.7×) | 1,070 (5.2×) | 1,072 (5.2×) |
| mul, constant time | GF(2²⁵⁶) | 193 | 703 (3.6×) | 702 (3.6×) | 1,079 (5.6×) | 1,080 (5.6×) |
| mul, constant time | GF(2⁵⁷¹) | 1,292 | 4,041 (3.1×) | 4,021 (3.1×) | 4,766 (3.7×) | 4,767 (3.7×) |
| square, constant time | GF(2⁸) | 10.7 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 18.9 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.3 | 27.4 (1.9×) | 27.3 (1.9×) | 30.0 (2.1×) | 30.0 (2.1×) |
| square, constant time | GF(2²³³) | 32.0 | 53.4 (1.7×) | 53.1 (1.7×) | 59.8 (1.9×) | 59.8 (1.9×) |
| square, constant time | GF(2²⁵⁶) | 27.1 | 59.0 (2.2×) | 58.7 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square, constant time | GF(2⁵⁷¹) | 57.1 | 162 (2.8×) | 161 (2.8×) | 181 (3.2×) | 181 (3.2×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- bench-wasm --changed --chrome <chrome> --firefox <firefox>   # the cells whose code changed, with the figures they are compared with
cargo run -p binary-field-bench-report -- report   # this file
```
