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

`binary-field` at commit `c7970be` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-06.

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
| Cost of a count, in WebAssembly, Chrome | 4,157 to 12,590 cycles and 3,081 instructions; its noise in a figure, 91.0 to 6,362 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,922 to 7,155 cycles and 3,081 instructions; its noise in a figure, 115 to 316 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 10,351 to 13,640 cycles and 1,885 instructions; its noise in a figure, 984 to 2,108 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 6,646 to 14,611 cycles and 1,885 instructions; its noise in a figure, 354 to 2,712 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 15.62 before, 14.99 after; its SMT sibling, CPU 28, 36% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 14.99 before, 15.22 after; its SMT sibling, CPU 28, 14% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 15.22 before, 18.75 after; its SMT sibling, CPU 28, 15% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 18.75 before, 17.78 after; its SMT sibling, CPU 28, 51% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 2.2 vs gf256 2.1 | +0.1 (1.05×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.1 | +0.1 (1.05×) |
| add | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add | GF(2²³³) | 7.9 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add | GF(2²⁵⁶) | 7.7 vs hekate-math 8.1 | −0.4 (0.95×) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 6.3 vs hekate-math 6.6 | −0.3 (0.95×) | 6.3 vs hekate-math 6.5 | −0.2 (0.97×) |
| add | GF(2⁵⁷¹) | 23.1 | – | 23.1 | – | 15.4 | – | 15.7 | – |
| mul | GF(2⁸) | 2.2 vs g2p 2.6 | −0.4 (0.85×) | 2.2 vs g2p 2.6 | −0.4 (0.85×) | 2.2 vs reed-solomon-erasure 2.6 | −0.4 (0.85×) | 2.3 vs g2p 2.7 | −0.4 (0.85×) |
| mul | GF(2¹⁶) | 5.6 vs reed-solomon-erasure 8.7 | −3.1 (0.64×) | 6.2 vs reed-solomon-erasure 8.8 | −2.6 (0.70×) | 6.4 vs reed-solomon-erasure 9.0 | −2.6 (0.71×) | 6.9 vs reed-solomon-erasure 9.1 | −2.2 (0.76×) |
| mul | GF(2⁶⁴) | 63.0 vs plonky3 92.7 | −29.7 (0.68×) | 64.0 vs plonky3 92.4 | −28.4 (0.69×) | 72.1 vs plonky3 95.1 | −23.0 (0.76×) | 72.1 vs plonky3 95.1 | −23.0 (0.76×) |
| mul | GF(2¹²⁸) | 184 vs polyval 233 | −49.0 (0.79×) | 184 vs polyval 227 | −43.0 (0.81×) | 200 vs polyval 259 | −59.0 (0.77×) | 199 vs polyval 257 | −58.0 (0.77×) |
| mul | GF(2²³³) | 777 | – | 795 | – | 1,080 | – | 1,089 | – |
| mul | GF(2²⁵⁶) | 709 vs hekate-math 8,421 | −7,712 (0.08×) | 710 vs hekate-math 8,423 | −7,713 (0.08×) | 1,079 vs hekate-math 9,489 | −8,410 (0.11×) | 1,080 vs hekate-math 9,487 | −8,407 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,028 | – | 4,004 | – | 4,956 | – | 4,837 | – |
| square | GF(2⁸) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) |
| square | GF(2¹⁶) | 2.3 vs plonky3 3.0 | −0.7 (0.77×) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.3 vs plonky3 3.1 | −0.8 (0.74×) | 2.2 vs plonky3 3.1 | −0.9 (0.71×) |
| square | GF(2⁶⁴) | 16.0 vs plonky3 46.3 | −30.3 (0.35×) | 15.7 vs plonky3 46.1 | −30.4 (0.34×) | 16.0 vs plonky3 49.6 | −33.6 (0.32×) | 16.0 vs plonky3 49.5 | −33.5 (0.32×) |
| square | GF(2¹²⁸) | 28.0 vs plonky3 34.6 | −6.6 (0.81×) | 28.0 vs plonky3 30.7 | −2.7 (0.91×) | 31.0 vs plonky3 47.2 | −16.2 (0.66×) | 31.1 vs plonky3 33.8 | −2.7 (0.92×) |
| square | GF(2²³³) | 50.6 | – | 50.7 | – | 61.1 | – | 61.0 | – |
| square | GF(2²⁵⁶) | 58.1 vs hekate-math 1,381 | −1,323 (0.04×) | 58.1 vs hekate-math 1,381 | −1,323 (0.04×) | 62.0 vs hekate-math 1,597 | −1,535 (0.04×) | 62.1 vs hekate-math 1,672 | −1,610 (0.04×) |
| square | GF(2⁵⁷¹) | 145 | – | 145 | – | 150 | – | 150 | – |
| invert | GF(2⁸) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.2 vs gf256 3.6 | −1.4 (0.61×) | 2.2 vs gf256 4.0 | −1.8 (0.55×) | 2.2 vs gf256 4.2 | −2.0 (0.52×) |
| invert | GF(2¹⁶) | 2.2 vs galois_field_2pm lut 9.2 | −7.0 (0.24×) | 2.2 vs galois_field_2pm lut 9.6 | −7.4 (0.23×) | 2.3 vs galois_field_2pm lut 9.6 | −7.3 (0.24×) | 2.3 vs galois_field_2pm lut 9.4 | −7.1 (0.24×) |
| invert | GF(2⁶⁴) | 374 vs hekate-math 1,581 | −1,207 (0.24×) | 381 vs hekate-math 1,587 | −1,206 (0.24×) | 384 vs hekate-math 2,061 | −1,677 (0.19×) | 389 vs hekate-math 2,063 | −1,674 (0.19×) |
| invert | GF(2¹²⁸) | 908 vs plonky3 1,211 | −303 (0.75×) | 910 vs plonky3 1,252 | −342 (0.73×) | 933 vs plonky3 1,316 | −383 (0.71×) | 954 vs plonky3 1,314 | −360 (0.73×) |
| invert | GF(2²³³) | 14,577 | – | 14,541 | – | 19,938 | – | 20,237 | – |
| invert | GF(2²⁵⁶) | 3,648 vs hekate-math 13,698 | −10,050 (0.27×) | 3,729 vs hekate-math 13,684 | −9,955 (0.27×) | 4,463 vs hekate-math 15,803 | −11,340 (0.28×) | 4,547 vs hekate-math 15,829 | −11,282 (0.29×) |
| invert | GF(2⁵⁷¹) | 130,196 | – | 129,918 | – | 160,371 | – | 160,889 | – |
| sqrt | GF(2⁸) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) |
| sqrt | GF(2¹⁶) | 2.3 vs plonky3 3.3 | −1.0 (0.70×) | 2.3 vs plonky3 3.3 | −1.0 (0.70×) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) | 2.4 vs plonky3 3.4 | −1.0 (0.71×) |
| sqrt | GF(2⁶⁴) | 37.2 vs plonky3 80.2 | −43.0 (0.46×) | 37.1 vs plonky3 80.2 | −43.1 (0.46×) | 41.1 vs plonky3 84.1 | −43.0 (0.49×) | 41.0 vs plonky3 84.2 | −43.2 (0.49×) |
| sqrt | GF(2¹²⁸) | 154 vs plonky3 153 | +1.0 (1.01×) | 154 vs plonky3 149 | +5.0 (1.03×) | 156 vs plonky3 163 | −7.0 (0.96×) | 157 vs plonky3 163 | −6.0 (0.96×) |
| sqrt | GF(2²³³) | 98.7 | – | 127 | – | 132 | – | 134 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,175,458 | −2,175,336 (0.00×) | 122 vs hekate-math 2,160,505 | −2,160,383 (0.00×) | 153 vs hekate-math 2,425,157 | −2,425,004 (0.00×) | 189 vs hekate-math 2,422,552 | −2,422,363 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,224 | – | 2,215 | – | 2,525 | – | 2,522 | – |
| add, constant time | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.1 | +0.1 (1.05×) |
| add, constant time | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 3.8 vs plonky3 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add, constant time | GF(2²³³) | 7.4 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add, constant time | GF(2²⁵⁶) | 7.5 vs hekate-math 8.1 | −0.6 (0.93×) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 6.3 vs hekate-math 6.6 | −0.3 (0.95×) | 6.3 vs hekate-math 6.5 | −0.2 (0.97×) |
| add, constant time | GF(2⁵⁷¹) | 23.0 | – | 23.1 | – | 15.4 | – | 15.7 | – |
| mul, constant time | GF(2⁸) | 11.7 vs plonky3 19.2 | −7.5 (0.61×) | 11.7 vs plonky3 19.2 | −7.5 (0.61×) | 13.0 vs plonky3 19.5 | −6.5 (0.67×) | 13.0 vs plonky3 19.6 | −6.6 (0.66×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 76.1 | −59.6 (0.22×) | 16.5 vs hekate-math 76.1 | −59.6 (0.22×) | 16.8 vs hekate-math 83.5 | −66.7 (0.20×) | 16.8 vs hekate-math 83.4 | −66.6 (0.20×) |
| mul, constant time | GF(2⁶⁴) | 63.9 vs plonky3 92.7 | −28.8 (0.69×) | 64.0 vs plonky3 92.4 | −28.4 (0.69×) | 72.0 vs plonky3 95.1 | −23.1 (0.76×) | 72.2 vs plonky3 95.1 | −22.9 (0.76×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 233 | −49.0 (0.79×) | 184 vs polyval 227 | −43.0 (0.81×) | 200 vs polyval 259 | −59.0 (0.77×) | 200 vs polyval 257 | −57.0 (0.78×) |
| mul, constant time | GF(2²³³) | 788 | – | 784 | – | 1,057 | – | 1,085 | – |
| mul, constant time | GF(2²⁵⁶) | 706 vs hekate-math 8,421 | −7,715 (0.08×) | 708 vs hekate-math 8,423 | −7,715 (0.08×) | 1,081 vs hekate-math 9,489 | −8,408 (0.11×) | 1,085 vs hekate-math 9,487 | −8,402 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,066 | – | 4,052 | – | 4,952 | – | 4,960 | – |
| square, constant time | GF(2⁸) | 5.1 vs hekate-math 8.8 | −3.7 (0.58×) | 5.1 vs hekate-math 9.0 | −3.9 (0.57×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) |
| square, constant time | GF(2¹⁶) | 10.6 vs hekate-math 25.4 | −14.8 (0.42×) | 10.2 vs hekate-math 25.4 | −15.2 (0.40×) | 10.0 vs hekate-math 25.1 | −15.1 (0.40×) | 10.0 vs hekate-math 27.0 | −17.0 (0.37×) |
| square, constant time | GF(2⁶⁴) | 16.2 vs plonky3 46.3 | −30.1 (0.35×) | 16.0 vs plonky3 46.1 | −30.1 (0.35×) | 16.0 vs plonky3 49.6 | −33.6 (0.32×) | 16.0 vs plonky3 49.5 | −33.5 (0.32×) |
| square, constant time | GF(2¹²⁸) | 28.0 vs plonky3 34.6 | −6.6 (0.81×) | 28.0 vs plonky3 30.7 | −2.7 (0.91×) | 31.0 vs plonky3 47.2 | −16.2 (0.66×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square, constant time | GF(2²³³) | 53.1 | – | 53.0 | – | 60.2 | – | 60.2 | – |
| square, constant time | GF(2²⁵⁶) | 62.1 vs hekate-math 1,381 | −1,319 (0.04×) | 62.0 vs hekate-math 1,381 | −1,319 (0.04×) | 66.2 vs hekate-math 1,597 | −1,531 (0.04×) | 63.5 vs hekate-math 1,672 | −1,609 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 153 | – | 153 | – | 167 | – | 167 | – |
| invert, constant time | GF(2⁸) | 123 vs plonky3 272 | −149 (0.45×) | 123 vs plonky3 272 | −149 (0.45×) | 132 vs plonky3 277 | −145 (0.48×) | 132 vs plonky3 277 | −145 (0.48×) |
| invert, constant time | GF(2¹⁶) | 239 vs hekate-math 393 | −154 (0.61×) | 239 vs hekate-math 393 | −154 (0.61×) | 233 vs hekate-math 716 | −483 (0.33×) | 233 vs hekate-math 715 | −482 (0.33×) |
| invert, constant time | GF(2⁶⁴) | 1,311 vs hekate-math 1,581 | −270 (0.83×) | 1,322 vs hekate-math 1,587 | −265 (0.83×) | 1,313 vs hekate-math 2,061 | −748 (0.64×) | 1,314 vs hekate-math 2,063 | −749 (0.64×) |
| invert, constant time | GF(2¹²⁸) | 2,895 vs hekate-math 4,496 | −1,601 (0.64×) | 2,868 vs hekate-math 4,490 | −1,622 (0.64×) | 2,619 vs hekate-math 5,394 | −2,775 (0.49×) | 2,610 vs hekate-math 5,347 | −2,737 (0.49×) |
| invert, constant time | GF(2²³³) | 19,019 | – | 19,112 | – | 19,671 | – | 19,842 | – |
| invert, constant time | GF(2²⁵⁶) | 8,289 vs hekate-math 13,698 | −5,409 (0.61×) | 8,322 vs hekate-math 13,684 | −5,362 (0.61×) | 7,779 vs hekate-math 15,803 | −8,024 (0.49×) | 7,758 vs hekate-math 15,829 | −8,071 (0.49×) |
| invert, constant time | GF(2⁵⁷¹) | 130,136 | – | 130,286 | – | 159,920 | – | 160,805 | – |
| sqrt, constant time | GF(2⁸) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.7 vs plonky3 11.5 | −5.8 (0.50×) | 5.7 vs plonky3 11.5 | −5.8 (0.50×) |
| sqrt, constant time | GF(2¹⁶) | 16.8 vs hekate-math 1,243 | −1,226 (0.01×) | 16.8 vs hekate-math 1,242 | −1,225 (0.01×) | 20.0 vs hekate-math 1,300 | −1,280 (0.02×) | 20.0 vs hekate-math 1,357 | −1,337 (0.01×) |
| sqrt, constant time | GF(2⁶⁴) | 37.3 vs plonky3 80.2 | −42.9 (0.47×) | 37.1 vs plonky3 80.2 | −43.1 (0.46×) | 40.8 vs plonky3 84.1 | −43.3 (0.49×) | 41.1 vs plonky3 84.2 | −43.1 (0.49×) |
| sqrt, constant time | GF(2¹²⁸) | 154 vs plonky3 153 | +1.0 (1.01×) | 154 vs plonky3 149 | +5.0 (1.03×) | 156 vs plonky3 163 | −7.0 (0.96×) | 157 vs plonky3 163 | −6.0 (0.96×) |
| sqrt, constant time | GF(2²³³) | 125 | – | 104 | – | 133 | – | 132 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,175,458 | −2,175,336 (0.00×) | 122 vs hekate-math 2,160,505 | −2,160,383 (0.00×) | 161 vs hekate-math 2,425,157 | −2,424,996 (0.00×) | 171 vs hekate-math 2,422,552 | −2,422,381 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,214 | – | 2,217 | – | 2,534 | – | 2,538 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) |
| add | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) | 2.1 vs runtime-algebra 2.3 | −0.2 (0.91×) |
| add | GF(2²³³) | 5.1 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 4.9 vs hekate-math 5.1 | −0.2 (0.96×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.5 | +0.1 (1.02×) |
| add | GF(2⁵⁷¹) | 17.4 | – | 17.1 | – | 12.9 | – | 13.0 | – |
| mul | GF(2⁸) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.7 vs g2p 11.8 | −1.1 (0.91×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) |
| mul | GF(2¹⁶) | 31.5 vs reed-solomon-erasure 24.2 | +7.3 (1.30×) | 33.2 vs reed-solomon-erasure 23.8 | +9.4 (1.39×) | 31.5 vs reed-solomon-erasure 24.0 | +7.5 (1.31×) | 35.0 vs reed-solomon-erasure 24.0 | +11.0 (1.46×) |
| mul | GF(2⁶⁴) | 72.0 vs plonky3 99.9 | −27.9 (0.72×) | 72.0 vs plonky3 80.9 | −8.9 (0.89×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.1 vs plonky3 100 | −26.9 (0.73×) |
| mul | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 228 | −44.0 (0.81×) | 197 vs polyval 262 | −65.0 (0.75×) | 197 vs polyval 262 | −65.0 (0.75×) |
| mul | GF(2²³³) | 751 | – | 757 | – | 1,073 | – | 1,077 | – |
| mul | GF(2²⁵⁶) | 705 vs hekate-math 8,428 | −7,723 (0.08×) | 707 vs hekate-math 8,431 | −7,724 (0.08×) | 1,054 vs hekate-math 9,455 | −8,401 (0.11×) | 1,038 vs hekate-math 9,484 | −8,446 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,016 | – | 4,006 | – | 4,745 | – | 4,753 | – |
| square | GF(2⁸) | 5.1 vs reed-solomon-erasure 7.7 | −2.6 (0.66×) | 5.1 vs reed-solomon-erasure 7.6 | −2.5 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) | 5.3 vs g2p 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs plonky3 8.3 | −2.0 (0.76×) | 6.3 vs plonky3 8.3 | −2.0 (0.76×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 34.7 | −7.7 (0.78×) | 27.0 vs plonky3 32.6 | −5.6 (0.83×) | 30.3 vs plonky3 43.9 | −13.6 (0.69×) | 31.0 vs plonky3 35.0 | −4.0 (0.89×) |
| square | GF(2²³³) | 54.0 | – | 54.0 | – | 59.9 | – | 59.8 | – |
| square | GF(2²⁵⁶) | 59.0 vs hekate-math 1,383 | −1,324 (0.04×) | 59.0 vs hekate-math 1,377 | −1,318 (0.04×) | 63.0 vs hekate-math 1,453 | −1,390 (0.04×) | 63.1 vs hekate-math 1,489 | −1,426 (0.04×) |
| square | GF(2⁵⁷¹) | 165 | – | 164 | – | 183 | – | 183 | – |
| add, constant time | GF(2⁸) | 1.2 vs gf256 1.1 | +0.1 (1.09×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) |
| add, constant time | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) | 2.1 vs runtime-algebra 2.3 | −0.2 (0.91×) |
| add, constant time | GF(2²³³) | 5.1 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add, constant time | GF(2²⁵⁶) | 5.0 vs hekate-math 5.1 | −0.1 (0.98×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.5 | +0.1 (1.02×) |
| add, constant time | GF(2⁵⁷¹) | 17.6 | – | 17.4 | – | 12.9 | – | 12.9 | – |
| mul, constant time | GF(2⁸) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) |
| mul, constant time | GF(2¹⁶) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 22.0 vs hekate-math 81.9 | −59.9 (0.27×) | 23.0 vs hekate-math 95.1 | −72.1 (0.24×) | 23.0 vs hekate-math 101 | −78.0 (0.23×) |
| mul, constant time | GF(2⁶⁴) | 71.9 vs plonky3 99.9 | −28.0 (0.72×) | 72.0 vs plonky3 80.9 | −8.9 (0.89×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.1 vs plonky3 100 | −26.9 (0.73×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 228 | −44.0 (0.81×) | 184 vs polyval 228 | −44.0 (0.81×) | 197 vs polyval 262 | −65.0 (0.75×) | 197 vs polyval 262 | −65.0 (0.75×) |
| mul, constant time | GF(2²³³) | 757 | – | 754 | – | 1,065 | – | 1,075 | – |
| mul, constant time | GF(2²⁵⁶) | 697 vs hekate-math 8,428 | −7,731 (0.08×) | 710 vs hekate-math 8,431 | −7,721 (0.08×) | 1,045 vs hekate-math 9,455 | −8,410 (0.11×) | 1,037 vs hekate-math 9,484 | −8,447 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,008 | – | 4,015 | – | 4,687 | – | 4,706 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.4 | −4.4 (0.77×) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.3 vs plonky3 62.0 | −42.7 (0.31×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.3 vs plonky3 34.7 | −7.4 (0.79×) | 27.3 vs plonky3 32.6 | −5.3 (0.84×) | 31.0 vs plonky3 43.9 | −12.9 (0.71×) | 31.0 vs plonky3 35.0 | −4.0 (0.89×) |
| square, constant time | GF(2²³³) | 54.4 | – | 54.0 | – | 59.0 | – | 59.0 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,383 | −1,324 (0.04×) | 59.0 vs hekate-math 1,377 | −1,318 (0.04×) | 63.0 vs hekate-math 1,453 | −1,390 (0.04×) | 63.0 vs hekate-math 1,489 | −1,426 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 165 | – | 164 | – | 184 | – | 183 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 2.2 (1.5×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.3 (2.5×) | 3.3 (2.5×) |
| add | GF(2²³³) | 2.3 | 7.9 (3.4×) | 7.5 (3.3×) | 6.3 (2.7×) | 6.3 (2.7×) |
| add | GF(2²⁵⁶) | 2.6 | 7.7 (3.0×) | 7.5 (2.9×) | 6.3 (2.4×) | 6.3 (2.4×) |
| add | GF(2⁵⁷¹) | 6.7 | 23.1 (3.4×) | 23.1 (3.4×) | 15.4 (2.3×) | 15.7 (2.3×) |
| mul | GF(2⁸) | 2.0 | 2.2 (1.1×) | 2.2 (1.1×) | 2.2 (1.1×) | 2.3 (1.1×) |
| mul | GF(2¹⁶) | 5.3 | 5.6 (1.1×) | 6.2 (1.2×) | 6.4 (1.2×) | 6.9 (1.3×) |
| mul | GF(2⁶⁴) | 31.6 | 63.0 (2.0×) | 64.0 (2.0×) | 72.1 (2.3×) | 72.1 (2.3×) |
| mul | GF(2¹²⁸) | 54.2 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 199 (3.7×) |
| mul | GF(2²³³) | 202 | 777 (3.8×) | 795 (3.9×) | 1,080 (5.3×) | 1,089 (5.4×) |
| mul | GF(2²⁵⁶) | 189 | 709 (3.8×) | 710 (3.8×) | 1,079 (5.7×) | 1,080 (5.7×) |
| mul | GF(2⁵⁷¹) | 1,290 | 4,028 (3.1×) | 4,004 (3.1×) | 4,956 (3.8×) | 4,837 (3.7×) |
| square | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| square | GF(2¹⁶) | 1.7 | 2.3 (1.4×) | 2.2 (1.3×) | 2.3 (1.4×) | 2.2 (1.3×) |
| square | GF(2⁶⁴) | 5.4 | 16.0 (3.0×) | 15.7 (2.9×) | 16.0 (3.0×) | 16.0 (3.0×) |
| square | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.0 (4.2×) | 31.0 (4.7×) | 31.1 (4.7×) |
| square | GF(2²³³) | 21.1 | 50.6 (2.4×) | 50.7 (2.4×) | 61.1 (2.9×) | 61.0 (2.9×) |
| square | GF(2²⁵⁶) | 20.4 | 58.1 (2.8×) | 58.1 (2.8×) | 62.0 (3.0×) | 62.1 (3.0×) |
| square | GF(2⁵⁷¹) | 55.0 | 145 (2.6×) | 145 (2.6×) | 150 (2.7×) | 150 (2.7×) |
| invert | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| invert | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) | 2.3 (1.4×) |
| invert | GF(2⁶⁴) | 532 | 374 (0.7×) | 381 (0.7×) | 384 (0.7×) | 389 (0.7×) |
| invert | GF(2¹²⁸) | 832 | 908 (1.1×) | 910 (1.1×) | 933 (1.1×) | 954 (1.1×) |
| invert | GF(2²³³) | 9,362 | 14,577 (1.6×) | 14,541 (1.6×) | 19,938 (2.1×) | 20,237 (2.2×) |
| invert | GF(2²⁵⁶) | 8,693 | 3,648 (0.4×) | 3,729 (0.4×) | 4,463 (0.5×) | 4,547 (0.5×) |
| invert | GF(2⁵⁷¹) | 48,582 | 130,196 (2.7×) | 129,918 (2.7×) | 160,371 (3.3×) | 160,889 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| sqrt | GF(2¹⁶) | 1.7 | 2.3 (1.4×) | 2.3 (1.4×) | 2.3 (1.4×) | 2.4 (1.4×) |
| sqrt | GF(2⁶⁴) | 35.6 | 37.2 (1.0×) | 37.1 (1.0×) | 41.1 (1.2×) | 41.0 (1.2×) |
| sqrt | GF(2¹²⁸) | 65.6 | 154 (2.3×) | 154 (2.3×) | 156 (2.4×) | 157 (2.4×) |
| sqrt | GF(2²³³) | 110 | 98.7 (0.9×) | 127 (1.2×) | 132 (1.2×) | 134 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 122 (1.3×) | 153 (1.6×) | 189 (2.0×) |
| sqrt | GF(2⁵⁷¹) | 674 | 2,224 (3.3×) | 2,215 (3.3×) | 2,525 (3.7×) | 2,522 (3.7×) |
| add, constant time | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add, constant time | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add, constant time | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.3 (2.5×) | 3.3 (2.5×) |
| add, constant time | GF(2²³³) | 2.6 | 7.4 (2.8×) | 7.5 (2.9×) | 6.3 (2.4×) | 6.3 (2.4×) |
| add, constant time | GF(2²⁵⁶) | 2.5 | 7.5 (3.0×) | 7.5 (3.0×) | 6.3 (2.5×) | 6.3 (2.5×) |
| add, constant time | GF(2⁵⁷¹) | 7.2 | 23.0 (3.2×) | 23.1 (3.2×) | 15.4 (2.1×) | 15.7 (2.2×) |
| mul, constant time | GF(2⁸) | 12.1 | 11.7 (1.0×) | 11.7 (1.0×) | 13.0 (1.1×) | 13.0 (1.1×) |
| mul, constant time | GF(2¹⁶) | 15.1 | 16.5 (1.1×) | 16.5 (1.1×) | 16.8 (1.1×) | 16.8 (1.1×) |
| mul, constant time | GF(2⁶⁴) | 31.6 | 63.9 (2.0×) | 64.0 (2.0×) | 72.0 (2.3×) | 72.2 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.3 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 200 (3.7×) |
| mul, constant time | GF(2²³³) | 199 | 788 (4.0×) | 784 (3.9×) | 1,057 (5.3×) | 1,085 (5.5×) |
| mul, constant time | GF(2²⁵⁶) | 189 | 706 (3.7×) | 708 (3.7×) | 1,081 (5.7×) | 1,085 (5.7×) |
| mul, constant time | GF(2⁵⁷¹) | 1,290 | 4,066 (3.2×) | 4,052 (3.1×) | 4,952 (3.8×) | 4,960 (3.8×) |
| square, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 6.3 (1.8×) |
| square, constant time | GF(2¹⁶) | 8.6 | 10.6 (1.2×) | 10.2 (1.2×) | 10.0 (1.2×) | 10.0 (1.2×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 16.2 (3.0×) | 16.0 (3.0×) | 16.0 (3.0×) | 16.0 (3.0×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.0 (4.2×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 21.0 | 53.1 (2.5×) | 53.0 (2.5×) | 60.2 (2.9×) | 60.2 (2.9×) |
| square, constant time | GF(2²⁵⁶) | 20.3 | 62.1 (3.1×) | 62.0 (3.1×) | 66.2 (3.3×) | 63.5 (3.1×) |
| square, constant time | GF(2⁵⁷¹) | 54.9 | 153 (2.8×) | 153 (2.8×) | 167 (3.0×) | 167 (3.0×) |
| invert, constant time | GF(2⁸) | 89.8 | 123 (1.4×) | 123 (1.4×) | 132 (1.5×) | 132 (1.5×) |
| invert, constant time | GF(2¹⁶) | 227 | 239 (1.1×) | 239 (1.1×) | 233 (1.0×) | 233 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 569 | 1,311 (2.3×) | 1,322 (2.3×) | 1,313 (2.3×) | 1,314 (2.3×) |
| invert, constant time | GF(2¹²⁸) | 1,864 | 2,895 (1.6×) | 2,868 (1.5×) | 2,619 (1.4×) | 2,610 (1.4×) |
| invert, constant time | GF(2²³³) | 9,332 | 19,019 (2.0×) | 19,112 (2.0×) | 19,671 (2.1×) | 19,842 (2.1×) |
| invert, constant time | GF(2²⁵⁶) | 8,643 | 8,289 (1.0×) | 8,322 (1.0×) | 7,779 (0.9×) | 7,758 (0.9×) |
| invert, constant time | GF(2⁵⁷¹) | 48,617 | 130,136 (2.7×) | 130,286 (2.7×) | 159,920 (3.3×) | 160,805 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 5.7 (1.6×) | 5.7 (1.6×) |
| sqrt, constant time | GF(2¹⁶) | 13.5 | 16.8 (1.2×) | 16.8 (1.2×) | 20.0 (1.5×) | 20.0 (1.5×) |
| sqrt, constant time | GF(2⁶⁴) | 35.6 | 37.3 (1.0×) | 37.1 (1.0×) | 40.8 (1.1×) | 41.1 (1.2×) |
| sqrt, constant time | GF(2¹²⁸) | 65.7 | 154 (2.3×) | 154 (2.3×) | 156 (2.4×) | 157 (2.4×) |
| sqrt, constant time | GF(2²³³) | 110 | 125 (1.1×) | 104 (0.9×) | 133 (1.2×) | 132 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 122 (1.3×) | 161 (1.7×) | 171 (1.8×) |
| sqrt, constant time | GF(2⁵⁷¹) | 674 | 2,214 (3.3×) | 2,217 (3.3×) | 2,534 (3.8×) | 2,538 (3.8×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) |
| add | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.1 (1.9×) | 2.1 (1.9×) |
| add | GF(2²³³) | 2.0 | 5.1 (2.5×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.0 | 4.9 (2.5×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2⁵⁷¹) | 5.0 | 17.4 (3.5×) | 17.1 (3.4×) | 12.9 (2.6×) | 13.0 (2.6×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.5 (1.1×) | 10.7 (1.2×) | 10.9 (1.2×) |
| mul | GF(2¹⁶) | 32.2 | 31.5 (1.0×) | 33.2 (1.0×) | 31.5 (1.0×) | 35.0 (1.1×) |
| mul | GF(2⁶⁴) | 41.7 | 72.0 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.1 (1.8×) |
| mul | GF(2¹²⁸) | 64.9 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul | GF(2²³³) | 201 | 751 (3.7×) | 757 (3.8×) | 1,073 (5.3×) | 1,077 (5.4×) |
| mul | GF(2²⁵⁶) | 190 | 705 (3.7×) | 707 (3.7×) | 1,054 (5.5×) | 1,038 (5.5×) |
| mul | GF(2⁵⁷¹) | 1,281 | 4,016 (3.1×) | 4,006 (3.1×) | 4,745 (3.7×) | 4,753 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.0 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 27.0 (1.9×) | 30.3 (2.1×) | 31.0 (2.2×) |
| square | GF(2²³³) | 32.0 | 54.0 (1.7×) | 54.0 (1.7×) | 59.9 (1.9×) | 59.8 (1.9×) |
| square | GF(2²⁵⁶) | 27.0 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.1 (2.3×) |
| square | GF(2⁵⁷¹) | 57.0 | 165 (2.9×) | 164 (2.9×) | 183 (3.2×) | 183 (3.2×) |
| add, constant time | GF(2⁸) | 1.0 | 1.2 (1.2×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.1 (1.9×) | 2.1 (1.9×) |
| add, constant time | GF(2²³³) | 2.0 | 5.1 (2.5×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add, constant time | GF(2²⁵⁶) | 2.0 | 5.0 (2.5×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add, constant time | GF(2⁵⁷¹) | 5.0 | 17.6 (3.5×) | 17.4 (3.5×) | 12.9 (2.6×) | 12.9 (2.6×) |
| mul, constant time | GF(2⁸) | 21.0 | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 22.0 (1.0×) | 22.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.6 | 71.9 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.1 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 207 | 757 (3.7×) | 754 (3.6×) | 1,065 (5.1×) | 1,075 (5.2×) |
| mul, constant time | GF(2²⁵⁶) | 193 | 697 (3.6×) | 710 (3.7×) | 1,045 (5.4×) | 1,037 (5.4×) |
| mul, constant time | GF(2⁵⁷¹) | 1,292 | 4,008 (3.1×) | 4,015 (3.1×) | 4,687 (3.6×) | 4,706 (3.6×) |
| square, constant time | GF(2⁸) | 10.7 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.3 (1.6×) | 19.0 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.3 | 27.3 (1.9×) | 27.3 (1.9×) | 31.0 (2.2×) | 31.0 (2.2×) |
| square, constant time | GF(2²³³) | 32.0 | 54.4 (1.7×) | 54.0 (1.7×) | 59.0 (1.8×) | 59.0 (1.8×) |
| square, constant time | GF(2²⁵⁶) | 27.1 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square, constant time | GF(2⁵⁷¹) | 57.1 | 165 (2.9×) | 164 (2.9×) | 184 (3.2×) | 183 (3.2×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
