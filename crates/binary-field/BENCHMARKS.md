# Benchmarks

`binary-field` at commit `2378845` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 -C llvm-args=-align-loops=64 --cfg polyval_backend="soft"` |
| Load, with carry-less multiplication | CPU 13; 1-minute load 17.79 before, 12.99 after; its SMT sibling, CPU 29, 20% busy |
| Load, without carry-less multiplication | CPU 13; 1-minute load 20.73 before, 12.68 after; its SMT sibling, CPU 29, 21% busy |
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
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.8 / 1.4 | 1.7 / 1.3 | 1.4 / **1.1** | **1.3** / 1.3 | 1.4 / 1.3 | 1.7 / 1.3 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.0** | – | – | – | – | **2.7** / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.8** / **5.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.2 / 78.7 | **2.0** / 11.3 | **2.0** / 11.4 | 13.4 / 23.0 | 14.8 / 29.2 | – | – | – | – | – |
| GF(2¹⁶) | **5.0** / **20.0** | 8.0 / 27.0 | 141 / 143 | 8.9 / 43.8 | 7.5 / 22.6 | 53.0 / 52.0 | 14.1 / 30.4 | – | – | – | – | 12.3 / 61.9 |
| GF(2⁶⁴) | **4.9** / **11.1** | 5.9 / 20.0 | 365 / 367 | – | – | 548 / 548 | 5.9 / 19.1 | – | – | – | – | – |
| GF(2¹²⁸) | 12.0 / **12.0** | – | – | – | – | 1,758 / 1,760 | **11.9** / 16.1 | **11.9** / 18.5 | 20.0 / 27.7 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **39.7** / **43.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **33.9** / **34.0** | – | – | – | – | 5,728 / 5,733 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **167** / **163** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 53.1 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 17.9 | 8.2 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 7.0 / 27.8 | 138 / 127 | 6.5 / 11.5 | 6.5 / 17.2 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 10.0 / 24.0 |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.7 | 364 / 372 | – | – | 126 / 138 | 5.0 / 15.5 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | – | – | – | 288 / 285 | 7.9 / 16.3 | 9.9 / 18.9 | 16.3 / 27.0 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **20.6** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **19.2** | – | – | – | – | 1,021 / 1,083 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.2** / **40.9** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 309 / – | 29.1 / – | 29.4 / – | 254 / – | 309 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 419 / – | 1,001 / – | 963 / – | 520 / – | 324 / – | 20.4 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **211** / – | 1,224 / – | 8,370 / – | – | – | 1,209 / – | 1,064 / – | – | – | – | – | – |
| GF(2¹²⁸) | **410** / – | – | – | – | – | 3,226 / – | 579 / – | 5,950 / – | 5,051 / – | 147,687 / – | 39,269 / – | – |
| GF(2²³³) | **7,901** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,093** / – | – | – | – | – | 9,713 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **27,034** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.4 / – | 511 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 238 / – | 2,072 / – | 336 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 554 / – |
| GF(2⁶⁴) | **11.2** / – | 932 / – | 24,099 / – | – | – | 34,520 / – | 14.0 / – | – | – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | – | – | – | 224,028 / – | 17.7 / – | 2,443 / – | 3,574 / – | 69,340 / – | 19,876 / – | – |
| GF(2²³³) | **47.9** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.6** / – | – | – | – | – | 1,461,656 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **132** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / **1.0** | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.4 / **1.1** | – | – | – | – | 1.8 / 1.4 | 1.7 / 1.3 | 1.4 / **1.1** | **1.3** / 1.3 | 1.4 / 1.3 | 1.7 / 1.3 | – |
| GF(2²³³) | **2.2** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.0** | – | – | – | – | 2.7 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.6** / **4.6** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.5** / **17.2** | – | 13.4 / 23.0 | 14.8 / 29.2 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.0** | **8.0** / 27.0 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **4.9** / **11.1** | 5.9 / 20.0 | 548 / 548 | 5.9 / 19.1 | – | – | – |
| GF(2¹²⁸) | **11.9** / **12.0** | – | 1,758 / 1,760 | **11.9** / 16.1 | **11.9** / 18.5 | 20.0 / 27.7 | 146 / 155 |
| GF(2²³³) | **39.5** / **43.6** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **33.8** / **34.0** | – | 5,728 / 5,733 | – | – | – | – |
| GF(2⁵⁷¹) | **170** / **163** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.6** | – | 8.0 / 17.9 | 8.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | **7.0** / 27.8 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **4.5** / **11.2** | 6.0 / 20.7 | 126 / 138 | 5.0 / 15.5 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.1** | – | 288 / 285 | 7.9 / 16.3 | 9.9 / 18.9 | 16.3 / 27.0 | 146 / 156 |
| GF(2²³³) | **20.9** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.0** / **18.3** | – | 1,021 / 1,083 | – | – | – | – |
| GF(2⁵⁷¹) | **41.1** / **40.9** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **55.1** / – | – | 254 / – | 309 / – | – | – |
| GF(2¹⁶) | **170** / – | 419 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,224 / – | 1,209 / – | 1,064 / – | – | – |
| GF(2¹²⁸) | **1,943** / – | – | 3,226 / – | – | 5,051 / – | 39,269 / – |
| GF(2²³³) | **7,906** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,140** / – | – | 9,713 / – | – | – | – |
| GF(2⁵⁷¹) | **27,033** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.5** / – | 238 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **11.3** / – | 932 / – | 34,520 / – | 14.0 / – | – | – | – |
| GF(2¹²⁸) | **13.3** / – | – | 224,028 / – | 17.7 / – | 2,443 / – | 3,574 / – | 19,876 / – |
| GF(2²³³) | **47.8** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **35.4** / – | – | 1,461,656 / – | – | – | – | – |
| GF(2⁵⁷¹) | **134** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.4** / 1.2 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / 1.1 | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | 1.4 / **1.2** | – | – | – | – | 1.9 / 1.4 | 1.7 / 1.4 | 1.4 / 1.3 | 1.4 / 1.3 | 1.4 / 1.4 | **1.3** / 1.4 | – |
| GF(2²³³) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.1** | – | – | – | – | 2.9 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.9** / **5.1** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.0** / **9.3** | 3.1 / 12.0 | 76.3 / 78.8 | **2.0** / 11.3 | **2.0** / 11.3 | 13.3 / 23.0 | 15.2 / 29.2 | – | – | – | – | – |
| GF(2¹⁶) | **4.9** / 29.0 | 31.5 / 54.8 | 141 / 144 | 8.4 / 33.0 | 7.4 / **22.6** | 53.0 / 52.0 | 14.1 / 30.4 | – | – | – | – | 11.6 / 55.2 |
| GF(2⁶⁴) | **32.0** / **41.8** | 101 / 117 | 365 / 367 | – | – | 548 / 548 | 46.9 / 68.7 | – | – | – | – | – |
| GF(2¹²⁸) | **54.6** / **64.8** | – | – | – | – | 1,759 / 1,760 | 162 / 186 | 172 / 172 | 550 / 546 | 550 / 546 | 146 / 155 | – |
| GF(2²³³) | **199** / **200** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **194** | – | – | – | – | 5,727 / 5,730 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,290** / **1,277** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / **5.0** | 2.0 / 11.0 | 69.6 / 58.6 | 2.0 / 7.0 | 2.0 / 7.0 | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / **6.0** | 31.6 / 56.5 | 138 / 124 | 6.4 / 11.9 | 6.6 / 6.3 | 23.0 / 30.0 | 2.1 / 8.0 | – | – | – | – | 9.4 / 24.0 |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.8 / 126 | 364 / 372 | – | – | 126 / 134 | 24.5 / 44.0 | – | – | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | – | – | – | 288 / 285 | 36.9 / 39.9 | 173 / 184 | 547 / 546 | 547 / 546 | 146 / 156 | – |
| GF(2²³³) | **21.1** / **32.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.4** / **27.1** | – | – | – | – | 1,023 / 1,084 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.0** / **57.0** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 2.3 / – | 310 / – | 29.4 / – | 29.5 / – | 254 / – | 309 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 1,150 / – | 1,003 / – | 695 / – | 518 / – | 324 / – | 20.4 / – | – | – | – | – | 9.3 / – |
| GF(2⁶⁴) | **531** / – | 13,915 / – | 8,381 / – | – | – | 1,209 / – | 3,239 / – | – | – | – | – | – |
| GF(2¹²⁸) | **831** / – | – | – | – | – | 3,217 / – | 980 / – | 11,313 / – | 139,428 / – | 147,682 / – | 39,270 / – | – |
| GF(2²³³) | **9,397** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,718** / – | – | – | – | – | 9,705 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **48,591** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.2** / – | 23.3 / – | 512 / – | 14.4 / – | 14.4 / – | 150 / – | 8.5 / – | – | – | – | – | – |
| GF(2¹⁶) | **1.7** / – | 814 / – | 2,071 / – | 271 / – | 221 / – | 793 / – | 2.0 / – | – | – | – | – | 496 / – |
| GF(2⁶⁴) | **35.6** / – | 7,929 / – | 24,133 / – | – | – | 34,520 / – | 40.2 / – | – | – | – | – | – |
| GF(2¹²⁸) | **65.7** / – | – | – | – | – | 223,985 / – | 123 / – | 22,816 / – | 69,341 / – | 69,337 / – | 19,875 / – | – |
| GF(2²³³) | **110** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.3** / – | – | – | – | – | 1,461,265 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **673** / – | – | – | – | – | – | – | – | – | – | – | – |

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | – |
| GF(2¹⁶) | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | **1.5** / **1.0** | – | – | – | – | **1.5** / **1.0** |
| GF(2⁶⁴) | **1.3** / 1.1 | 1.5 / **1.0** | 1.5 / **1.0** | – | – | 1.5 / 1.1 | 1.5 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.3** / **1.1** | – | – | – | – | 1.9 / 1.4 | 1.7 / 1.4 | 1.4 / 1.3 | 1.4 / 1.3 | 1.4 / 1.4 | **1.3** / 1.4 | – |
| GF(2²³³) | **2.2** / **2.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.6** / **2.1** | – | – | – | – | 2.9 / 2.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.3** / **5.3** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 14.0 / **21.0** | – | **13.3** / 23.0 | 15.2 / 29.2 | – | – | – |
| GF(2¹⁶) | **15.0** / **22.5** | 31.5 / 54.8 | 53.0 / 52.0 | – | – | – | – |
| GF(2⁶⁴) | **31.8** / **41.7** | 101 / 117 | 548 / 548 | 46.9 / 68.7 | – | – | – |
| GF(2¹²⁸) | **54.5** / **64.9** | – | 1,759 / 1,760 | 162 / 186 | 172 / 172 | 550 / 546 | 146 / 155 |
| GF(2²³³) | **202** / **206** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **189** / **194** | – | 5,727 / 5,730 | – | – | – | – |
| GF(2⁵⁷¹) | **1,289** / **1,289** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / **10.7** | – | 8.0 / 18.0 | 8.4 / 23.0 | – | – | – |
| GF(2¹⁶) | **8.7** / **22.2** | 31.6 / 56.5 | 23.0 / 30.0 | – | – | – | – |
| GF(2⁶⁴) | **5.4** / **11.8** | 99.8 / 126 | 126 / 134 | 24.5 / 44.0 | – | – | – |
| GF(2¹²⁸) | **6.6** / **14.2** | – | 288 / 285 | 36.9 / 39.9 | 173 / 184 | 547 / 546 | 146 / 156 |
| GF(2²³³) | **21.1** / **32.1** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.3** / **27.2** | – | 1,023 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **54.9** / **57.0** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **92.0** / – | – | 254 / – | 309 / – | – | – |
| GF(2¹⁶) | **227** / – | 1,150 / – | 324 / – | – | – | – |
| GF(2⁶⁴) | **569** / – | 13,915 / – | 1,209 / – | 3,239 / – | – | – |
| GF(2¹²⁸) | **1,872** / – | – | 3,217 / – | – | 139,428 / – | 39,270 / – |
| GF(2²³³) | **9,401** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **8,694** / – | – | 9,705 / – | – | – | – |
| GF(2⁵⁷¹) | **48,649** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 150 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **13.4** / – | 814 / – | 793 / – | – | – | – | – |
| GF(2⁶⁴) | **35.7** / – | 7,929 / – | 34,520 / – | 40.2 / – | – | – | – |
| GF(2¹²⁸) | **65.7** / – | – | 223,985 / – | 123 / – | 22,816 / – | 69,341 / – | 19,875 / – |
| GF(2²³³) | **110** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.1** / – | – | 1,461,265 / – | – | – | – | – |
| GF(2⁵⁷¹) | **674** / – | – | – | – | – | – | – |

## WebAssembly

`binary-field` at commit `326f8ef` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,362 to 5,768 cycles and 3,081 instructions; its noise in a figure, 120 to 731 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,254 to 5,745 cycles and 3,081 instructions; its noise in a figure, 65.0 to 255 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,367 to 7,984 cycles and 1,885 instructions; its noise in a figure, 597 to 622 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 19,624 to 23,524 cycles and 1,885 instructions; its noise in a figure, 1,202 to 8,909 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 18.66 before, 16.8 after; its SMT sibling, CPU 28, 18% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 13.84 before, 12.82 after; its SMT sibling, CPU 28, 6% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 14.2 before, 22.71 after; its SMT sibling, CPU 28, 15% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 22.71 before, 19.43 after; its SMT sibling, CPU 28, 29% busy |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |


### Against the best competitor, throughput

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs hekate-math 2.1 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add | GF(2²³³) | 7.5 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add | GF(2²⁵⁶) | 7.5 vs hekate-math 8.0 | −0.5 (0.94×) | 7.5 vs hekate-math 7.9 | −0.4 (0.95×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) |
| add | GF(2⁵⁷¹) | 23.1 | – | 23.1 | – | 15.5 | – | 15.6 | – |
| mul | GF(2⁸) | 2.2 vs g2p 2.7 | −0.5 (0.81×) | 2.2 vs g2p 2.7 | −0.5 (0.81×) | 2.2 vs g2p 2.6 | −0.4 (0.85×) | 2.2 vs g2p 2.7 | −0.5 (0.81×) |
| mul | GF(2¹⁶) | 5.9 vs reed-solomon-erasure 8.8 | −2.9 (0.67×) | 5.9 vs reed-solomon-erasure 8.7 | −2.8 (0.68×) | 6.2 vs reed-solomon-erasure 9.0 | −2.8 (0.69×) | 6.1 vs reed-solomon-erasure 9.0 | −2.9 (0.68×) |
| mul | GF(2⁶⁴) | 64.0 vs plonky3 92.7 | −28.7 (0.69×) | 64.0 vs plonky3 91.9 | −27.9 (0.70×) | 72.1 vs plonky3 95.1 | −23.0 (0.76×) | 72.1 vs plonky3 95.1 | −23.0 (0.76×) |
| mul | GF(2¹²⁸) | 184 vs polyval 230 | −46.0 (0.80×) | 184 vs polyval 231 | −47.0 (0.80×) | 200 vs polyval 257 | −57.0 (0.78×) | 199 vs polyval 256 | −57.0 (0.78×) |
| mul | GF(2²³³) | 782 | – | 796 | – | 1,092 | – | 1,078 | – |
| mul | GF(2²⁵⁶) | 706 vs hekate-math 8,415 | −7,709 (0.08×) | 706 vs hekate-math 8,421 | −7,715 (0.08×) | 1,077 vs hekate-math 9,579 | −8,502 (0.11×) | 1,082 vs hekate-math 9,509 | −8,427 (0.11×) |
| mul | GF(2⁵⁷¹) | 4,009 | – | 4,003 | – | 4,854 | – | 4,885 | – |
| square | GF(2⁸) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.1 vs g2p 2.2 | −0.1 (0.95×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) | 2.2 vs g2p 2.2 | +0.0 (1.00×) |
| square | GF(2¹⁶) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.2 vs plonky3 3.0 | −0.8 (0.73×) | 2.2 vs plonky3 3.1 | −0.9 (0.71×) | 2.3 vs plonky3 3.1 | −0.8 (0.74×) |
| square | GF(2⁶⁴) | 16.0 vs plonky3 46.2 | −30.2 (0.35×) | 15.5 vs plonky3 46.2 | −30.7 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square | GF(2¹²⁸) | 28.1 vs plonky3 34.6 | −6.5 (0.81×) | 28.0 vs plonky3 30.8 | −2.8 (0.91×) | 31.0 vs plonky3 43.3 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square | GF(2²³³) | 50.7 | – | 50.6 | – | 61.0 | – | 61.0 | – |
| square | GF(2²⁵⁶) | 58.1 vs hekate-math 1,384 | −1,326 (0.04×) | 58.0 vs hekate-math 1,383 | −1,325 (0.04×) | 62.0 vs hekate-math 1,550 | −1,488 (0.04×) | 62.1 vs hekate-math 1,589 | −1,527 (0.04×) |
| square | GF(2⁵⁷¹) | 145 | – | 145 | – | 153 | – | 150 | – |
| invert | GF(2⁸) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.1 vs gf256 3.6 | −1.5 (0.58×) | 2.2 vs gf256 4.0 | −1.8 (0.55×) | 2.2 vs gf256 5.0 | −2.8 (0.44×) |
| invert | GF(2¹⁶) | 2.2 vs galois_field_2pm lut 8.4 | −6.2 (0.26×) | 2.2 vs galois_field_2pm lut 8.2 | −6.0 (0.27×) | 2.3 vs galois_field_2pm lut 9.4 | −7.1 (0.24×) | 2.3 vs galois_field_2pm lut 9.4 | −7.1 (0.24×) |
| invert | GF(2⁶⁴) | 789 vs hekate-math 1,582 | −793 (0.50×) | 789 vs hekate-math 1,586 | −797 (0.50×) | 896 vs hekate-math 2,115 | −1,219 (0.42×) | 899 vs hekate-math 2,084 | −1,185 (0.43×) |
| invert | GF(2¹²⁸) | 1,302 vs plonky3 1,228 | +74.0 (1.06×) | 1,352 vs plonky3 1,251 | +101 (1.08×) | 1,455 vs plonky3 1,320 | +135 (1.10×) | 1,467 vs plonky3 1,304 | +163 (1.12×) |
| invert | GF(2²³³) | 14,713 | – | 14,718 | – | 20,231 | – | 19,443 | – |
| invert | GF(2²⁵⁶) | 13,609 vs hekate-math 13,727 | −118 (0.99×) | 13,668 vs hekate-math 13,701 | −33.0 (1.00×) | 19,916 vs hekate-math 16,016 | +3,900 (1.24×) | 19,916 vs hekate-math 15,908 | +4,008 (1.25×) |
| invert | GF(2⁵⁷¹) | 130,201 | – | 130,251 | – | 161,484 | – | 161,043 | – |
| sqrt | GF(2⁸) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.1 vs plonky3 10.9 | −8.8 (0.19×) | 2.2 vs plonky3 10.9 | −8.7 (0.20×) | 2.2 vs plonky3 11.5 | −9.3 (0.19×) |
| sqrt | GF(2¹⁶) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) | 2.3 vs plonky3 3.4 | −1.1 (0.68×) |
| sqrt | GF(2⁶⁴) | 37.2 vs plonky3 80.1 | −42.9 (0.46×) | 37.2 vs plonky3 80.0 | −42.8 (0.47×) | 40.9 vs plonky3 84.2 | −43.3 (0.49×) | 41.0 vs plonky3 84.2 | −43.2 (0.49×) |
| sqrt | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 154 vs plonky3 149 | +5.0 (1.03×) | 159 vs plonky3 165 | −6.0 (0.96×) | 156 vs plonky3 165 | −9.0 (0.95×) |
| sqrt | GF(2²³³) | 96.0 | – | 112 | – | 132 | – | 131 | – |
| sqrt | GF(2²⁵⁶) | 122 vs hekate-math 2,155,109 | −2,154,987 (0.00×) | 122 vs hekate-math 2,156,589 | −2,156,467 (0.00×) | 160 vs hekate-math 2,446,763 | −2,446,603 (0.00×) | 172 vs hekate-math 2,438,312 | −2,438,140 (0.00×) |
| sqrt | GF(2⁵⁷¹) | 2,223 | – | 2,223 | – | 2,527 | – | 2,523 | – |
| add, constant time | GF(2⁸) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.1 vs gf256 2.1 | +0.0 (1.00×) | 2.2 vs hekate-math 2.1 | +0.1 (1.05×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.5 vs gf256 2.5 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) | 2.2 vs gf256 2.2 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.8 vs hekate-math 3.7 | +0.1 (1.03×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) | 3.3 vs hekate-math 3.3 | +0.0 (1.00×) |
| add, constant time | GF(2²³³) | 7.5 | – | 7.5 | – | 6.3 | – | 6.3 | – |
| add, constant time | GF(2²⁵⁶) | 7.4 vs hekate-math 8.0 | −0.6 (0.93×) | 7.5 vs hekate-math 7.9 | −0.4 (0.95×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) | 6.3 vs hekate-math 6.3 | +0.0 (1.00×) |
| add, constant time | GF(2⁵⁷¹) | 23.3 | – | 23.1 | – | 15.5 | – | 15.6 | – |
| mul, constant time | GF(2⁸) | 11.7 vs plonky3 19.2 | −7.5 (0.61×) | 11.7 vs plonky3 19.2 | −7.5 (0.61×) | 13.0 vs plonky3 19.5 | −6.5 (0.67×) | 13.0 vs plonky3 19.6 | −6.6 (0.66×) |
| mul, constant time | GF(2¹⁶) | 16.5 vs hekate-math 76.0 | −59.5 (0.22×) | 16.5 vs hekate-math 76.0 | −59.5 (0.22×) | 16.8 vs hekate-math 83.6 | −66.8 (0.20×) | 16.7 vs hekate-math 80.0 | −63.3 (0.21×) |
| mul, constant time | GF(2⁶⁴) | 64.1 vs plonky3 92.7 | −28.6 (0.69×) | 64.0 vs plonky3 91.9 | −27.9 (0.70×) | 72.0 vs plonky3 95.1 | −23.1 (0.76×) | 72.0 vs plonky3 95.1 | −23.1 (0.76×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 230 | −46.0 (0.80×) | 184 vs polyval 231 | −47.0 (0.80×) | 200 vs polyval 257 | −57.0 (0.78×) | 198 vs polyval 256 | −58.0 (0.77×) |
| mul, constant time | GF(2²³³) | 790 | – | 790 | – | 1,101 | – | 1,083 | – |
| mul, constant time | GF(2²⁵⁶) | 706 vs hekate-math 8,415 | −7,709 (0.08×) | 705 vs hekate-math 8,421 | −7,716 (0.08×) | 1,073 vs hekate-math 9,579 | −8,506 (0.11×) | 1,074 vs hekate-math 9,509 | −8,435 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,094 | – | 4,015 | – | 4,831 | – | 4,830 | – |
| square, constant time | GF(2⁸) | 5.1 vs hekate-math 9.1 | −4.0 (0.56×) | 5.1 vs hekate-math 8.9 | −3.8 (0.57×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) | 6.3 vs hekate-math 9.6 | −3.3 (0.66×) |
| square, constant time | GF(2¹⁶) | 10.3 vs hekate-math 25.4 | −15.1 (0.41×) | 10.2 vs hekate-math 25.4 | −15.2 (0.40×) | 10.4 vs hekate-math 27.0 | −16.6 (0.39×) | 10.4 vs hekate-math 27.0 | −16.6 (0.39×) |
| square, constant time | GF(2⁶⁴) | 16.0 vs plonky3 46.2 | −30.2 (0.35×) | 15.6 vs plonky3 46.2 | −30.6 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) | 17.0 vs plonky3 49.5 | −32.5 (0.34×) |
| square, constant time | GF(2¹²⁸) | 28.0 vs plonky3 34.6 | −6.6 (0.81×) | 28.1 vs plonky3 30.8 | −2.7 (0.91×) | 31.0 vs plonky3 43.3 | −12.3 (0.72×) | 31.0 vs plonky3 33.8 | −2.8 (0.92×) |
| square, constant time | GF(2²³³) | 53.1 | – | 53.0 | – | 58.9 | – | 60.3 | – |
| square, constant time | GF(2²⁵⁶) | 62.1 vs hekate-math 1,384 | −1,322 (0.04×) | 62.1 vs hekate-math 1,383 | −1,321 (0.04×) | 63.2 vs hekate-math 1,550 | −1,487 (0.04×) | 65.0 vs hekate-math 1,589 | −1,524 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 153 | – | 153 | – | 167 | – | 166 | – |
| invert, constant time | GF(2⁸) | 123 vs plonky3 272 | −149 (0.45×) | 123 vs plonky3 272 | −149 (0.45×) | 132 vs plonky3 277 | −145 (0.48×) | 127 vs plonky3 277 | −150 (0.46×) |
| invert, constant time | GF(2¹⁶) | 239 vs hekate-math 393 | −154 (0.61×) | 239 vs hekate-math 393 | −154 (0.61×) | 233 vs hekate-math 705 | −472 (0.33×) | 233 vs hekate-math 717 | −484 (0.32×) |
| invert, constant time | GF(2⁶⁴) | 1,329 vs hekate-math 1,582 | −253 (0.84×) | 1,308 vs hekate-math 1,586 | −278 (0.82×) | 1,312 vs hekate-math 2,115 | −803 (0.62×) | 1,316 vs hekate-math 2,084 | −768 (0.63×) |
| invert, constant time | GF(2¹²⁸) | 4,562 vs hekate-math 4,500 | +62.0 (1.01×) | 4,527 vs hekate-math 4,498 | +29.0 (1.01×) | 4,385 vs hekate-math 5,432 | −1,047 (0.81×) | 4,408 vs hekate-math 5,396 | −988 (0.82×) |
| invert, constant time | GF(2²³³) | 18,995 | – | 19,020 | – | 19,970 | – | 19,706 | – |
| invert, constant time | GF(2²⁵⁶) | 17,952 vs hekate-math 13,727 | +4,225 (1.31×) | 17,913 vs hekate-math 13,701 | +4,212 (1.31×) | 20,786 vs hekate-math 16,016 | +4,770 (1.30×) | 20,333 vs hekate-math 15,908 | +4,425 (1.28×) |
| invert, constant time | GF(2⁵⁷¹) | 130,173 | – | 130,068 | – | 161,693 | – | 161,379 | – |
| sqrt, constant time | GF(2⁸) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 5.1 vs plonky3 10.9 | −5.8 (0.47×) | 6.3 vs plonky3 10.9 | −4.6 (0.58×) | 5.7 vs plonky3 11.5 | −5.8 (0.50×) |
| sqrt, constant time | GF(2¹⁶) | 16.4 vs hekate-math 1,242 | −1,226 (0.01×) | 16.8 vs hekate-math 1,243 | −1,226 (0.01×) | 20.0 vs hekate-math 1,242 | −1,222 (0.02×) | 20.0 vs hekate-math 1,313 | −1,293 (0.02×) |
| sqrt, constant time | GF(2⁶⁴) | 37.3 vs plonky3 80.1 | −42.8 (0.47×) | 37.2 vs plonky3 80.0 | −42.8 (0.47×) | 40.8 vs plonky3 84.2 | −43.4 (0.48×) | 40.8 vs plonky3 84.2 | −43.4 (0.48×) |
| sqrt, constant time | GF(2¹²⁸) | 154 vs plonky3 149 | +5.0 (1.03×) | 154 vs plonky3 149 | +5.0 (1.03×) | 156 vs plonky3 165 | −9.0 (0.95×) | 156 vs plonky3 165 | −9.0 (0.95×) |
| sqrt, constant time | GF(2²³³) | 110 | – | 94.9 | – | 132 | – | 131 | – |
| sqrt, constant time | GF(2²⁵⁶) | 122 vs hekate-math 2,155,109 | −2,154,987 (0.00×) | 122 vs hekate-math 2,156,589 | −2,156,467 (0.00×) | 163 vs hekate-math 2,446,763 | −2,446,600 (0.00×) | 172 vs hekate-math 2,438,312 | −2,438,140 (0.00×) |
| sqrt, constant time | GF(2⁵⁷¹) | 2,224 | – | 2,225 | – | 2,523 | – | 2,535 | – |

### Against the best competitor, latency

Cycles per operation of `binary-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add | GF(2¹²⁸) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.1 vs compile-algebra 2.2 | −0.1 (0.95×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) |
| add | GF(2²³³) | 4.7 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add | GF(2²⁵⁶) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add | GF(2⁵⁷¹) | 17.4 | – | 17.4 | – | 12.9 | – | 12.9 | – |
| mul | GF(2⁸) | 10.5 vs g2p 11.6 | −1.1 (0.91×) | 10.5 vs g2p 11.7 | −1.2 (0.90×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) | 10.9 vs g2p 11.8 | −0.9 (0.92×) |
| mul | GF(2¹⁶) | 30.1 vs reed-solomon-erasure 24.0 | +6.1 (1.25×) | 30.2 vs reed-solomon-erasure 23.9 | +6.3 (1.26×) | 31.2 vs reed-solomon-erasure 23.8 | +7.4 (1.31×) | 31.0 vs reed-solomon-erasure 24.6 | +6.4 (1.26×) |
| mul | GF(2⁶⁴) | 70.7 vs plonky3 76.0 | −5.3 (0.93×) | 72.1 vs plonky3 75.4 | −3.3 (0.96×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.0 vs plonky3 100 | −27.0 (0.73×) |
| mul | GF(2¹²⁸) | 184 vs polyval 229 | −45.0 (0.80×) | 184 vs polyval 228 | −44.0 (0.81×) | 197 vs polyval 264 | −67.0 (0.75×) | 197 vs polyval 260 | −63.0 (0.76×) |
| mul | GF(2²³³) | 753 | – | 755 | – | 1,096 | – | 1,071 | – |
| mul | GF(2²⁵⁶) | 712 vs hekate-math 8,425 | −7,713 (0.08×) | 699 vs hekate-math 8,429 | −7,730 (0.08×) | 1,079 vs hekate-math 9,587 | −8,508 (0.11×) | 1,055 vs hekate-math 9,534 | −8,479 (0.11×) |
| mul | GF(2⁵⁷¹) | 3,992 | – | 3,998 | – | 4,723 | – | 4,745 | – |
| square | GF(2⁸) | 5.1 vs g2p 7.0 | −1.9 (0.73×) | 5.1 vs reed-solomon-erasure 7.7 | −2.6 (0.66×) | 5.3 vs reed-solomon-erasure 7.9 | −2.6 (0.67×) | 5.3 vs reed-solomon-erasure 7.9 | −2.6 (0.67×) |
| square | GF(2¹⁶) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.1 vs plonky3 8.3 | −2.2 (0.73×) | 6.3 vs reed-solomon-erasure 6.4 | −0.1 (0.98×) | 6.3 vs reed-solomon-erasure 7.1 | −0.8 (0.89×) |
| square | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) | 19.0 vs plonky3 61.6 | −42.6 (0.31×) |
| square | GF(2¹²⁸) | 27.0 vs plonky3 35.6 | −8.6 (0.76×) | 27.0 vs plonky3 32.5 | −5.5 (0.83×) | 30.1 vs plonky3 43.6 | −13.5 (0.69×) | 31.0 vs plonky3 35.0 | −4.0 (0.89×) |
| square | GF(2²³³) | 54.0 | – | 54.0 | – | 59.7 | – | 59.0 | – |
| square | GF(2²⁵⁶) | 59.0 vs hekate-math 1,381 | −1,322 (0.04×) | 59.0 vs hekate-math 1,379 | −1,320 (0.04×) | 63.0 vs hekate-math 1,451 | −1,388 (0.04×) | 63.0 vs hekate-math 1,487 | −1,424 (0.04×) |
| square | GF(2⁵⁷¹) | 161 | – | 160 | – | 183 | – | 183 | – |
| add, constant time | GF(2⁸) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2¹⁶) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) | 1.2 vs reed-solomon-erasure 1.1 | +0.1 (1.09×) | 1.1 vs gf256 1.1 | +0.0 (1.00×) |
| add, constant time | GF(2⁶⁴) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.3 vs gf256 1.3 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) | 1.0 vs gf256 1.0 | +0.0 (1.00×) |
| add, constant time | GF(2¹²⁸) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.3 vs hekate-math 2.4 | −0.1 (0.96×) | 2.1 vs compile-algebra 2.2 | −0.1 (0.95×) | 2.1 vs hekate-math 2.3 | −0.2 (0.91×) |
| add, constant time | GF(2²³³) | 4.8 | – | 4.8 | – | 4.6 | – | 4.6 | – |
| add, constant time | GF(2²⁵⁶) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.8 vs hekate-math 5.0 | −0.2 (0.96×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) | 4.6 vs hekate-math 4.6 | +0.0 (1.00×) |
| add, constant time | GF(2⁵⁷¹) | 17.2 | – | 17.3 | – | 12.9 | – | 12.9 | – |
| mul, constant time | GF(2⁸) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) | 21.0 vs plonky3 23.0 | −2.0 (0.91×) |
| mul, constant time | GF(2¹⁶) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 22.0 vs hekate-math 82.0 | −60.0 (0.27×) | 23.0 vs hekate-math 103 | −80.0 (0.22×) | 23.0 vs hekate-math 102 | −79.0 (0.23×) |
| mul, constant time | GF(2⁶⁴) | 72.1 vs plonky3 76.0 | −3.9 (0.95×) | 72.0 vs plonky3 75.4 | −3.4 (0.95×) | 73.1 vs plonky3 100 | −26.9 (0.73×) | 73.0 vs plonky3 100 | −27.0 (0.73×) |
| mul, constant time | GF(2¹²⁸) | 184 vs polyval 229 | −45.0 (0.80×) | 184 vs polyval 228 | −44.0 (0.81×) | 197 vs polyval 264 | −67.0 (0.75×) | 197 vs polyval 260 | −63.0 (0.76×) |
| mul, constant time | GF(2²³³) | 755 | – | 757 | – | 1,087 | – | 1,064 | – |
| mul, constant time | GF(2²⁵⁶) | 698 vs hekate-math 8,425 | −7,727 (0.08×) | 704 vs hekate-math 8,429 | −7,725 (0.08×) | 1,070 vs hekate-math 9,587 | −8,517 (0.11×) | 1,055 vs hekate-math 9,534 | −8,479 (0.11×) |
| mul, constant time | GF(2⁵⁷¹) | 4,011 | – | 3,995 | – | 4,694 | – | 4,749 | – |
| square, constant time | GF(2⁸) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) | 15.0 vs hekate-math 19.3 | −4.3 (0.78×) | 15.0 vs hekate-math 19.5 | −4.5 (0.77×) | 15.0 vs hekate-math 19.9 | −4.9 (0.75×) |
| square, constant time | GF(2¹⁶) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 34.0 | −11.0 (0.68×) | 23.0 vs hekate-math 36.0 | −13.0 (0.64×) |
| square, constant time | GF(2⁶⁴) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 59.0 | −40.0 (0.32×) | 19.0 vs plonky3 62.0 | −43.0 (0.31×) | 19.0 vs plonky3 61.6 | −42.6 (0.31×) |
| square, constant time | GF(2¹²⁸) | 27.3 vs plonky3 35.6 | −8.3 (0.77×) | 27.3 vs plonky3 32.5 | −5.2 (0.84×) | 31.0 vs plonky3 43.6 | −12.6 (0.71×) | 31.0 vs plonky3 35.0 | −4.0 (0.89×) |
| square, constant time | GF(2²³³) | 54.0 | – | 54.0 | – | 59.0 | – | 58.9 | – |
| square, constant time | GF(2²⁵⁶) | 59.0 vs hekate-math 1,381 | −1,322 (0.04×) | 59.0 vs hekate-math 1,379 | −1,320 (0.04×) | 63.0 vs hekate-math 1,451 | −1,388 (0.04×) | 63.0 vs hekate-math 1,487 | −1,424 (0.04×) |
| square, constant time | GF(2⁵⁷¹) | 164 | – | 164 | – | 183 | – | 183 | – |

### Against native, throughput

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) |
| add | GF(2⁶⁴) | 1.4 | 2.5 (1.8×) | 2.5 (1.8×) | 2.2 (1.6×) | 2.2 (1.6×) |
| add | GF(2¹²⁸) | 1.4 | 3.8 (2.7×) | 3.8 (2.7×) | 3.3 (2.4×) | 3.3 (2.4×) |
| add | GF(2²³³) | 2.3 | 7.5 (3.3×) | 7.5 (3.3×) | 6.3 (2.7×) | 6.3 (2.7×) |
| add | GF(2²⁵⁶) | 2.7 | 7.5 (2.8×) | 7.5 (2.8×) | 6.3 (2.3×) | 6.3 (2.3×) |
| add | GF(2⁵⁷¹) | 6.9 | 23.1 (3.3×) | 23.1 (3.3×) | 15.5 (2.2×) | 15.6 (2.3×) |
| mul | GF(2⁸) | 2.0 | 2.2 (1.1×) | 2.2 (1.1×) | 2.2 (1.1×) | 2.2 (1.1×) |
| mul | GF(2¹⁶) | 4.9 | 5.9 (1.2×) | 5.9 (1.2×) | 6.2 (1.3×) | 6.1 (1.2×) |
| mul | GF(2⁶⁴) | 32.0 | 64.0 (2.0×) | 64.0 (2.0×) | 72.1 (2.3×) | 72.1 (2.3×) |
| mul | GF(2¹²⁸) | 54.6 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 199 (3.6×) |
| mul | GF(2²³³) | 199 | 782 (3.9×) | 796 (4.0×) | 1,092 (5.5×) | 1,078 (5.4×) |
| mul | GF(2²⁵⁶) | 189 | 706 (3.7×) | 706 (3.7×) | 1,077 (5.7×) | 1,082 (5.7×) |
| mul | GF(2⁵⁷¹) | 1,290 | 4,009 (3.1×) | 4,003 (3.1×) | 4,854 (3.8×) | 4,885 (3.8×) |
| square | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| square | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) |
| square | GF(2⁶⁴) | 5.4 | 16.0 (3.0×) | 15.5 (2.9×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square | GF(2¹²⁸) | 6.6 | 28.1 (4.3×) | 28.0 (4.2×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square | GF(2²³³) | 21.1 | 50.7 (2.4×) | 50.6 (2.4×) | 61.0 (2.9×) | 61.0 (2.9×) |
| square | GF(2²⁵⁶) | 20.4 | 58.1 (2.8×) | 58.0 (2.8×) | 62.0 (3.0×) | 62.1 (3.0×) |
| square | GF(2⁵⁷¹) | 55.0 | 145 (2.6×) | 145 (2.6×) | 153 (2.8×) | 150 (2.7×) |
| invert | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| invert | GF(2¹⁶) | 1.7 | 2.2 (1.3×) | 2.2 (1.3×) | 2.3 (1.4×) | 2.3 (1.4×) |
| invert | GF(2⁶⁴) | 531 | 789 (1.5×) | 789 (1.5×) | 896 (1.7×) | 899 (1.7×) |
| invert | GF(2¹²⁸) | 831 | 1,302 (1.6×) | 1,352 (1.6×) | 1,455 (1.8×) | 1,467 (1.8×) |
| invert | GF(2²³³) | 9,397 | 14,713 (1.6×) | 14,718 (1.6×) | 20,231 (2.2×) | 19,443 (2.1×) |
| invert | GF(2²⁵⁶) | 8,718 | 13,609 (1.6×) | 13,668 (1.6×) | 19,916 (2.3×) | 19,916 (2.3×) |
| invert | GF(2⁵⁷¹) | 48,591 | 130,201 (2.7×) | 130,251 (2.7×) | 161,484 (3.3×) | 161,043 (3.3×) |
| sqrt | GF(2⁸) | 1.2 | 2.1 (1.8×) | 2.1 (1.8×) | 2.2 (1.8×) | 2.2 (1.8×) |
| sqrt | GF(2¹⁶) | 1.7 | 2.3 (1.4×) | 2.3 (1.4×) | 2.3 (1.4×) | 2.3 (1.4×) |
| sqrt | GF(2⁶⁴) | 35.6 | 37.2 (1.0×) | 37.2 (1.0×) | 40.9 (1.1×) | 41.0 (1.2×) |
| sqrt | GF(2¹²⁸) | 65.7 | 154 (2.3×) | 154 (2.3×) | 159 (2.4×) | 156 (2.4×) |
| sqrt | GF(2²³³) | 110 | 96.0 (0.9×) | 112 (1.0×) | 132 (1.2×) | 131 (1.2×) |
| sqrt | GF(2²⁵⁶) | 93.3 | 122 (1.3×) | 122 (1.3×) | 160 (1.7×) | 172 (1.8×) |
| sqrt | GF(2⁵⁷¹) | 673 | 2,223 (3.3×) | 2,223 (3.3×) | 2,527 (3.8×) | 2,523 (3.7×) |
| add, constant time | GF(2⁸) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) | 2.1 (1.4×) |
| add, constant time | GF(2¹⁶) | 1.5 | 2.1 (1.4×) | 2.1 (1.4×) | 2.2 (1.5×) | 2.2 (1.5×) |
| add, constant time | GF(2⁶⁴) | 1.3 | 2.5 (1.9×) | 2.5 (1.9×) | 2.2 (1.7×) | 2.2 (1.7×) |
| add, constant time | GF(2¹²⁸) | 1.3 | 3.8 (2.9×) | 3.8 (2.9×) | 3.3 (2.5×) | 3.3 (2.5×) |
| add, constant time | GF(2²³³) | 2.2 | 7.5 (3.4×) | 7.5 (3.4×) | 6.3 (2.9×) | 6.3 (2.9×) |
| add, constant time | GF(2²⁵⁶) | 2.6 | 7.4 (2.8×) | 7.5 (2.9×) | 6.3 (2.4×) | 6.3 (2.4×) |
| add, constant time | GF(2⁵⁷¹) | 6.3 | 23.3 (3.7×) | 23.1 (3.7×) | 15.5 (2.5×) | 15.6 (2.5×) |
| mul, constant time | GF(2⁸) | 14.0 | 11.7 (0.8×) | 11.7 (0.8×) | 13.0 (0.9×) | 13.0 (0.9×) |
| mul, constant time | GF(2¹⁶) | 15.0 | 16.5 (1.1×) | 16.5 (1.1×) | 16.8 (1.1×) | 16.7 (1.1×) |
| mul, constant time | GF(2⁶⁴) | 31.8 | 64.1 (2.0×) | 64.0 (2.0×) | 72.0 (2.3×) | 72.0 (2.3×) |
| mul, constant time | GF(2¹²⁸) | 54.5 | 184 (3.4×) | 184 (3.4×) | 200 (3.7×) | 198 (3.6×) |
| mul, constant time | GF(2²³³) | 202 | 790 (3.9×) | 790 (3.9×) | 1,101 (5.5×) | 1,083 (5.4×) |
| mul, constant time | GF(2²⁵⁶) | 189 | 706 (3.7×) | 705 (3.7×) | 1,073 (5.7×) | 1,074 (5.7×) |
| mul, constant time | GF(2⁵⁷¹) | 1,289 | 4,094 (3.2×) | 4,015 (3.1×) | 4,831 (3.7×) | 4,830 (3.7×) |
| square, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 6.3 (1.8×) |
| square, constant time | GF(2¹⁶) | 8.7 | 10.3 (1.2×) | 10.2 (1.2×) | 10.4 (1.2×) | 10.4 (1.2×) |
| square, constant time | GF(2⁶⁴) | 5.4 | 16.0 (3.0×) | 15.6 (2.9×) | 17.0 (3.1×) | 17.0 (3.1×) |
| square, constant time | GF(2¹²⁸) | 6.6 | 28.0 (4.2×) | 28.1 (4.3×) | 31.0 (4.7×) | 31.0 (4.7×) |
| square, constant time | GF(2²³³) | 21.1 | 53.1 (2.5×) | 53.0 (2.5×) | 58.9 (2.8×) | 60.3 (2.9×) |
| square, constant time | GF(2²⁵⁶) | 20.3 | 62.1 (3.1×) | 62.1 (3.1×) | 63.2 (3.1×) | 65.0 (3.2×) |
| square, constant time | GF(2⁵⁷¹) | 54.9 | 153 (2.8×) | 153 (2.8×) | 167 (3.0×) | 166 (3.0×) |
| invert, constant time | GF(2⁸) | 92.0 | 123 (1.3×) | 123 (1.3×) | 132 (1.4×) | 127 (1.4×) |
| invert, constant time | GF(2¹⁶) | 227 | 239 (1.1×) | 239 (1.1×) | 233 (1.0×) | 233 (1.0×) |
| invert, constant time | GF(2⁶⁴) | 569 | 1,329 (2.3×) | 1,308 (2.3×) | 1,312 (2.3×) | 1,316 (2.3×) |
| invert, constant time | GF(2¹²⁸) | 1,872 | 4,562 (2.4×) | 4,527 (2.4×) | 4,385 (2.3×) | 4,408 (2.4×) |
| invert, constant time | GF(2²³³) | 9,401 | 18,995 (2.0×) | 19,020 (2.0×) | 19,970 (2.1×) | 19,706 (2.1×) |
| invert, constant time | GF(2²⁵⁶) | 8,694 | 17,952 (2.1×) | 17,913 (2.1×) | 20,786 (2.4×) | 20,333 (2.3×) |
| invert, constant time | GF(2⁵⁷¹) | 48,649 | 130,173 (2.7×) | 130,068 (2.7×) | 161,693 (3.3×) | 161,379 (3.3×) |
| sqrt, constant time | GF(2⁸) | 3.5 | 5.1 (1.5×) | 5.1 (1.5×) | 6.3 (1.8×) | 5.7 (1.6×) |
| sqrt, constant time | GF(2¹⁶) | 13.4 | 16.4 (1.2×) | 16.8 (1.3×) | 20.0 (1.5×) | 20.0 (1.5×) |
| sqrt, constant time | GF(2⁶⁴) | 35.7 | 37.3 (1.0×) | 37.2 (1.0×) | 40.8 (1.1×) | 40.8 (1.1×) |
| sqrt, constant time | GF(2¹²⁸) | 65.7 | 154 (2.3×) | 154 (2.3×) | 156 (2.4×) | 156 (2.4×) |
| sqrt, constant time | GF(2²³³) | 110 | 110 (1.0×) | 94.9 (0.9×) | 132 (1.2×) | 131 (1.2×) |
| sqrt, constant time | GF(2²⁵⁶) | 93.1 | 122 (1.3×) | 122 (1.3×) | 163 (1.8×) | 172 (1.8×) |
| sqrt, constant time | GF(2⁵⁷¹) | 674 | 2,224 (3.3×) | 2,225 (3.3×) | 2,523 (3.7×) | 2,535 (3.8×) |

### Against native, latency

Cycles per operation of `binary-field` in each build, then its ratio to the native figure.

| Operation | Field | Native, without carry-less multiplication | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) | 1.1 (1.1×) |
| add | GF(2⁶⁴) | 1.2 | 1.3 (1.1×) | 1.3 (1.1×) | 1.0 (0.8×) | 1.0 (0.8×) |
| add | GF(2¹²⁸) | 1.2 | 2.3 (1.9×) | 2.3 (1.9×) | 2.1 (1.8×) | 2.1 (1.8×) |
| add | GF(2²³³) | 2.0 | 4.7 (2.4×) | 4.8 (2.4×) | 4.6 (2.3×) | 4.6 (2.3×) |
| add | GF(2²⁵⁶) | 2.1 | 4.8 (2.3×) | 4.8 (2.3×) | 4.6 (2.2×) | 4.6 (2.2×) |
| add | GF(2⁵⁷¹) | 5.1 | 17.4 (3.4×) | 17.4 (3.4×) | 12.9 (2.5×) | 12.9 (2.5×) |
| mul | GF(2⁸) | 9.3 | 10.5 (1.1×) | 10.5 (1.1×) | 10.9 (1.2×) | 10.9 (1.2×) |
| mul | GF(2¹⁶) | 29.0 | 30.1 (1.0×) | 30.2 (1.0×) | 31.2 (1.1×) | 31.0 (1.1×) |
| mul | GF(2⁶⁴) | 41.8 | 70.7 (1.7×) | 72.1 (1.7×) | 73.1 (1.7×) | 73.0 (1.7×) |
| mul | GF(2¹²⁸) | 64.8 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul | GF(2²³³) | 200 | 753 (3.8×) | 755 (3.8×) | 1,096 (5.5×) | 1,071 (5.4×) |
| mul | GF(2²⁵⁶) | 194 | 712 (3.7×) | 699 (3.6×) | 1,079 (5.6×) | 1,055 (5.4×) |
| mul | GF(2⁵⁷¹) | 1,277 | 3,992 (3.1×) | 3,998 (3.1×) | 4,723 (3.7×) | 4,745 (3.7×) |
| square | GF(2⁸) | 5.0 | 5.1 (1.0×) | 5.1 (1.0×) | 5.3 (1.1×) | 5.3 (1.1×) |
| square | GF(2¹⁶) | 6.0 | 6.1 (1.0×) | 6.1 (1.0×) | 6.3 (1.1×) | 6.3 (1.1×) |
| square | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) |
| square | GF(2¹²⁸) | 14.2 | 27.0 (1.9×) | 27.0 (1.9×) | 30.1 (2.1×) | 31.0 (2.2×) |
| square | GF(2²³³) | 32.0 | 54.0 (1.7×) | 54.0 (1.7×) | 59.7 (1.9×) | 59.0 (1.8×) |
| square | GF(2²⁵⁶) | 27.1 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square | GF(2⁵⁷¹) | 57.0 | 161 (2.8×) | 160 (2.8×) | 183 (3.2×) | 183 (3.2×) |
| add, constant time | GF(2⁸) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) | 1.1 (1.1×) |
| add, constant time | GF(2¹⁶) | 1.0 | 1.1 (1.1×) | 1.1 (1.1×) | 1.2 (1.2×) | 1.1 (1.1×) |
| add, constant time | GF(2⁶⁴) | 1.1 | 1.3 (1.2×) | 1.3 (1.2×) | 1.0 (0.9×) | 1.0 (0.9×) |
| add, constant time | GF(2¹²⁸) | 1.1 | 2.3 (2.1×) | 2.3 (2.1×) | 2.1 (1.9×) | 2.1 (1.9×) |
| add, constant time | GF(2²³³) | 2.1 | 4.8 (2.3×) | 4.8 (2.3×) | 4.6 (2.2×) | 4.6 (2.2×) |
| add, constant time | GF(2²⁵⁶) | 2.1 | 4.8 (2.3×) | 4.8 (2.3×) | 4.6 (2.2×) | 4.6 (2.2×) |
| add, constant time | GF(2⁵⁷¹) | 5.3 | 17.2 (3.2×) | 17.3 (3.3×) | 12.9 (2.4×) | 12.9 (2.4×) |
| mul, constant time | GF(2⁸) | 21.0 | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) | 21.0 (1.0×) |
| mul, constant time | GF(2¹⁶) | 22.5 | 22.0 (1.0×) | 22.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| mul, constant time | GF(2⁶⁴) | 41.7 | 72.1 (1.7×) | 72.0 (1.7×) | 73.1 (1.8×) | 73.0 (1.8×) |
| mul, constant time | GF(2¹²⁸) | 64.9 | 184 (2.8×) | 184 (2.8×) | 197 (3.0×) | 197 (3.0×) |
| mul, constant time | GF(2²³³) | 206 | 755 (3.7×) | 757 (3.7×) | 1,087 (5.3×) | 1,064 (5.2×) |
| mul, constant time | GF(2²⁵⁶) | 194 | 698 (3.6×) | 704 (3.6×) | 1,070 (5.5×) | 1,055 (5.4×) |
| mul, constant time | GF(2⁵⁷¹) | 1,289 | 4,011 (3.1×) | 3,995 (3.1×) | 4,694 (3.6×) | 4,749 (3.7×) |
| square, constant time | GF(2⁸) | 10.7 | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) | 15.0 (1.4×) |
| square, constant time | GF(2¹⁶) | 22.2 | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) | 23.0 (1.0×) |
| square, constant time | GF(2⁶⁴) | 11.8 | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) | 19.0 (1.6×) |
| square, constant time | GF(2¹²⁸) | 14.2 | 27.3 (1.9×) | 27.3 (1.9×) | 31.0 (2.2×) | 31.0 (2.2×) |
| square, constant time | GF(2²³³) | 32.1 | 54.0 (1.7×) | 54.0 (1.7×) | 59.0 (1.8×) | 58.9 (1.8×) |
| square, constant time | GF(2²⁵⁶) | 27.2 | 59.0 (2.2×) | 59.0 (2.2×) | 63.0 (2.3×) | 63.0 (2.3×) |
| square, constant time | GF(2⁵⁷¹) | 57.0 | 164 (2.9×) | 164 (2.9×) | 183 (3.2×) | 183 (3.2×) |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p binary-field-bench-report -- report   # this file
```
