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
| Load, with carry-less multiplication | CPU 15; 1-minute load 14.59 before, 20.94 after; its SMT sibling, CPU 31, 40% busy |
| Load, without carry-less multiplication | CPU 15; 1-minute load 20.94 before, 23.27 after; its SMT sibling, CPU 31, 52% busy |
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
| add | `+` | `+`° | `+`° | `+`° | `add`° | `+`° | `+`° | `addf`° | `addf`° | `addf`° | `+`° | `+`° |
| mul | `*` | `*`, `*`° | `*` | `*` | `mul` | `*`° | `*`°, `*` | `mulf`° | `mulf`° | `mulf` | `*`° | `*` |
| square | `square` | `a * a`, `a * a`° | `a * a` | `a * a` | `a * a` | `square`° | `square`°, `square` | `a * a`° | `a * a`° | `a * a` | `a * a`° | `a * a` |
| invert | `invert` | `checked_recip`, `checked_recip`° | `inverse` | `a^(2^n − 2) by *` | `a^(2^n − 2) by *` | `invert`° | `try_inverse`°, `try_inverse` | `invert` | `invert`° | `invert` | `a^(2^n − 2) by *`° | `inverse` |
| sqrt | `sqrt` | `a^(2^(n−1)) by *`, `a^(2^(n−1)) by *`° | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *`° | `try_sqrt`°, `try_sqrt` | `a^(2^(n−1)) by *`° | `a^(2^(n−1)) by *`° | `a^(2^(n−1)) by *` | `a^(2^(n−1)) by *`° | `a^(2^(n−1)) by *` |

° The crate's constant-time function, where it has no variable-time one.

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
| GF(2⁸) | 1.6 / 1.2 | **1.5°** / 1.2° | 1.6° / 1.2° | **1.5°** / **1.1°** | 2.1° / 2.1° | 1.6° / **1.1°** | 1.6° / **1.1°** | – | – | – | – | – |
| GF(2¹⁶) | 1.6 / 1.2 | **1.5°** / 1.2° | 1.6° / **1.1°** | **1.5°** / 1.2° | **1.5°** / 2.2° | 1.8° / 2.1° | 1.9° / 1.6° | – | – | – | – | 1.6° / **1.1°** |
| GF(2⁶⁴) | 1.7 / 1.2 | **1.5°** / 1.1° | **1.5°** / 1.1° | – | – | 4.5° / 3.3° | 1.6° / **1.0°** | – | – | – | – | – |
| GF(2¹²⁸) | **1.5** / **1.3** | – | – | – | – | 4.9° / 5.5° | 2.0° / 1.6° | 1.6° / **1.3°** | 1.6° / 1.7° | 1.6° / 1.7° | 3.7° / 4.6° | – |
| GF(2²³³) | **3.4** / **2.2** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.0** / **2.3** | – | – | – | – | 4.8° / 4.9° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.1** / **5.2** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### mul: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.4** / **10.6** | 3.2 / 12.1 | 81.1 / 81.0 | 2.5 / 13.1 | 2.7 / 12.7 | 13.6° / 23.1° | 16.4° / 30.9° | – | – | – | – | – |
| GF(2¹⁶) | **8.0** / **19.1** | **8.0°** / 27.2° | 142 / 144 | 10.5 / 60.7 | 8.8 / 24.9 | 55.1° / 55.4° | 14.3 / 30.7 | – | – | – | – | 11.5 / 74.9 |
| GF(2⁶⁴) | **5.1** / **11.2** | 6.0° / 20.1° | 366 / 382 | – | – | 566° / 559° | 6.0° / 19.1° | – | – | – | – | – |
| GF(2¹²⁸) | **12.0** / **12.0** | – | – | – | – | 1,818° / 1,849° | 12.1° / 16.1° | **12.0°** / 18.7° | 20.0° / 27.7° | 560 / 550 | 153° / 156° | – |
| GF(2²³³) | **39.9** / **43.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | – | – | – | 5,968° / 5,788° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **268** / **267** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### square: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.3** / **5.0** | 2.0 / 11.0 | 70.2 / 72.5 | 2.1 / 7.0 | 2.4 / 7.0 | 8.0° / 18.4° | 9.2° / 23.0° | – | – | – | – | – |
| GF(2¹⁶) | 2.8 / **6.1** | 7.3° / 27.8° | 143 / 142 | 7.9 / 12.3 | 6.8 / 17.9 | 23.8° / 30.1° | **2.3** / 8.0 | – | – | – | – | 30.1 / 24.4 |
| GF(2⁶⁴) | 5.4 / **11.3** | 6.0° / 20.8° | 385 / 382 | – | – | 130° / 138° | **5.0°** / 16.3° | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.5** | – | – | – | – | 289° / 299° | 8.0° / 16.6° | 10.0° / 19.0° | 16.5° / 29.7° | 569 / 565 | 147° / 161° | – |
| GF(2²³³) | **21.3** / **32.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.8** | – | – | – | – | 1,072° / 1,084° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.4** / **43.4** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### invert: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 2.8 / – | **2.4** / – | 313 / – | 30.8 / – | 30.7 / – | 261° / – | 318° / – | – | – | – | – | – |
| GF(2¹⁶) | **2.3** / – | 422° / – | 1,032 / – | 1,123 / – | 556 / – | 330° / – | 21.0 / – | – | – | – | – | 26.6 / – |
| GF(2⁶⁴) | **216** / – | 1,234° / – | 8,815 / – | – | – | 1,250° / – | 1,070° / – | – | – | – | – | – |
| GF(2¹²⁸) | **539** / – | – | – | – | – | 3,449° / – | 632 / – | 6,056 / – | 5,132° / – | 155,990 / – | 41,869° / – | – |
| GF(2²³³) | **7,960** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,188** / – | – | – | – | – | 9,727° / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **29,405** / – | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### sqrt: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.4** / – | 25.4 / – | 518 / – | 16.1 / – | 15.2 / – | 151° / – | 8.5° / – | – | – | – | – | – |
| GF(2¹⁶) | 3.3 / – | 239° / – | 2,109 / – | 352 / – | 238 / – | 815° / – | **2.5** / – | – | – | – | – | 506 / – |
| GF(2⁶⁴) | **11.5** / – | 935° / – | 24,752 / – | – | – | 35,872° / – | 14.1° / – | – | – | – | – | – |
| GF(2¹²⁸) | **16.6** / – | – | – | – | – | 233,242° / – | 18.5° / – | 2,460° / – | 3,602° / – | 69,765 / – | 20,209° / – | – |
| GF(2²³³) | **52.9** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **37.8** / – | – | – | – | – | 1,485,714° / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **311** / – | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### add, constant time: a + b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 1.7 / **1.1** | **1.5** / 1.2 | 1.6 / 1.2 | **1.5** / **1.1** | 2.1 / 2.1 | 1.6 / **1.1** | 1.6 / **1.1** | – | – | – | – | – |
| GF(2¹⁶) | 1.6 / **1.1** | **1.5** / 1.2 | 1.6 / **1.1** | **1.5** / 1.2 | **1.5** / 2.2 | 1.8 / 2.1 | 1.9 / 1.6 | – | – | – | – | 1.6 / **1.1** |
| GF(2⁶⁴) | 1.6 / 1.2 | **1.5** / 1.1 | **1.5** / 1.1 | – | – | 4.5 / 3.3 | 1.6 / **1.0** | – | – | – | – | – |
| GF(2¹²⁸) | **1.6** / **1.2** | – | – | – | – | 4.9 / 5.5 | 2.0 / 1.6 | **1.6** / 1.3 | **1.6** / 1.7 | **1.6** / 1.7 | 3.7 / 4.6 | – |
| GF(2²³³) | **3.4** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.1** / **2.3** | – | – | – | – | 4.8 / 4.9 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8.5** / **5.7** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **6.2** / **17.7** | – | 13.6 / 23.1 | 16.4 / 30.9 | – | – | – |
| GF(2¹⁶) | **8.0** / **19.1** | **8.0** / 27.2 | 55.1 / 55.4 | – | – | – | – |
| GF(2⁶⁴) | **5.0** / **11.2** | 6.0 / 20.1 | 566 / 559 | 6.0 / 19.1 | – | – | – |
| GF(2¹²⁸) | **12.0** / **12.6** | – | 1,818 / 1,849 | 12.1 / 16.1 | **12.0** / 18.7 | 20.0 / 27.7 | 153 / 156 |
| GF(2²³³) | **41.9** / **44.2** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **34.0** | – | 5,968 / 5,788 | – | – | – | – |
| GF(2⁵⁷¹) | **314** / **267** | – | – | – | – | – | – |

### square, constant time: a², with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.7** / **11.1** | – | 8.0 / 18.4 | 9.2 / 23.0 | – | – | – |
| GF(2¹⁶) | **7.0** / **19.0** | 7.3 / 27.8 | 23.8 / 30.1 | – | – | – | – |
| GF(2⁶⁴) | **4.7** / **11.3** | 6.0 / 20.8 | 130 / 138 | 5.0 / 16.3 | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.5** | – | 289 / 299 | 8.0 / 16.6 | 10.0 / 19.0 | 16.5 / 29.7 | 147 / 161 |
| GF(2²³³) | **21.3** / **32.0** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **18.8** | – | 1,072 / 1,084 | – | – | – | – |
| GF(2⁵⁷¹) | **41.2** / **43.3** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **62.1** / – | – | 261 / – | 318 / – | – | – |
| GF(2¹⁶) | **175** / – | 422 / – | 330 / – | – | – | – |
| GF(2⁶⁴) | **672** / – | 1,234 / – | 1,250 / – | 1,070 / – | – | – |
| GF(2¹²⁸) | **1,961** / – | – | 3,449 / – | – | 5,132 / – | 41,869 / – |
| GF(2²³³) | **7,946** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,156** / – | – | 9,727 / – | – | – | – |
| GF(2⁵⁷¹) | **28,791** / – | – | – | – | – | – |

### sqrt, constant time: √a, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.6** / – | – | 151 / – | 8.5 / – | – | – | – |
| GF(2¹⁶) | **14.0** / – | 239 / – | 815 / – | – | – | – | – |
| GF(2⁶⁴) | **11.4** / – | 935 / – | 35,872 / – | 14.1 / – | – | – | – |
| GF(2¹²⁸) | **13.8** / – | – | 233,242 / – | 18.5 / – | 2,460 / – | 3,602 / – | 20,209 / – |
| GF(2²³³) | **52.8** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **37.6** / – | – | 1,485,714 / – | – | – | – | – |
| GF(2⁵⁷¹) | **311** / – | – | – | – | – | – | – |

## Results without carry-less multiplication

### add: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 1.6 / 1.2 | 1.6° / **1.0°** | 1.8° / 1.2° | 1.6° / 1.1° | 1.9° / 1.5° | **1.5°** / **1.0°** | 1.6° / 1.2° | – | – | – | – | – |
| GF(2¹⁶) | 1.7 / **1.0** | **1.6°** / 1.2° | 1.7° / 1.1° | **1.6°** / 1.2° | 2.9° / 2.3° | 2.4° / 3.0° | 2.6° / 3.0° | – | – | – | – | 2.0° / 1.1° |
| GF(2⁶⁴) | 1.8 / 1.3 | **1.5°** / **1.1°** | **1.5°** / **1.1°** | – | – | 5.5° / 6.9° | 2.0° / 1.8° | – | – | – | – | – |
| GF(2¹²⁸) | 1.7 / **1.4** | – | – | – | – | 4.3° / 7.4° | 2.1° / 1.8° | 1.7° / 1.5° | **1.6°** / 1.5° | **1.6°** / **1.4°** | 8.0° / 5.7° | – |
| GF(2²³³) | **3.0** / **2.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.2** / **2.5** | – | – | – | – | 9.0° / 7.4° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.8** / **5.6** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### mul: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 3.8 / 12.0 | 4.2 / 12.3 | 78.9 / 80.8 | **3.4** / 13.8 | 4.0 / **11.4** | 14.5° / 23.2° | 16.3° / 31.0° | – | – | – | – | – |
| GF(2¹⁶) | **7.8** / 45.5 | 32.3° / 56.0° | 152 / 155 | 12.8 / 70.6 | 9.5 / **22.1** | 53.6° / 57.5° | 17.0 / 30.8 | – | – | – | – | 32.1 / 82.1 |
| GF(2⁶⁴) | **33.5** / **42.1** | 105° / 118° | 403 / 382 | – | – | 572° / 571° | 52.9° / 70.4° | – | – | – | – | – |
| GF(2¹²⁸) | **57.9** / **65.0** | – | – | – | – | 1,921° / 1,798° | 169° / 192° | 177° / 186° | 597° / 566° | 607 / 598 | 160° / 162° | – |
| GF(2²³³) | **222** / **225** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **195** / **195** | – | – | – | – | 6,078° / 5,997° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,517** / **1,559** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### square: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.8** / **5.1** | 2.3 / 11.1 | 75.9 / 73.4 | 2.2 / 7.1 | 2.3 / 7.1 | 9.1° / 18.5° | 9.2° / 23.1° | – | – | – | – | – |
| GF(2¹⁶) | **2.5** / **6.1** | 36.0° / 56.6° | 162 / 147 | 9.5 / 12.7 | 7.6 / 17.8 | 25.3° / 31.6° | **2.5** / 8.1 | – | – | – | – | 21.1 / 24.6 |
| GF(2⁶⁴) | **5.7** / **11.9** | 103° / 127° | 390 / 396 | – | – | 151° / 160° | 30.6° / 44.3° | – | – | – | – | – |
| GF(2¹²⁸) | **7.0** / **14.5** | – | – | – | – | 354° / 317° | 37.1° / 44.3° | 205° / 206° | 640° / 612° | 603 / 590 | 153° / 169° | – |
| GF(2²³³) | **21.3** / **32.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.5** / **27.1** | – | – | – | – | 1,205° / 1,179° | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **55.7** / **55.8** | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### invert: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **2.3** / – | 3.4 / – | 323 / – | 34.2 / – | 34.2 / – | 264° / – | 321° / – | – | – | – | – | – |
| GF(2¹⁶) | **3.3** / – | 1,187° / – | 1,043 / – | 1,080 / – | 533 / – | 335° / – | 23.1 / – | – | – | – | – | 28.9 / – |
| GF(2⁶⁴) | **544** / – | 13,979° / – | 9,362 / – | – | – | 1,308° / – | 3,289° / – | – | – | – | – | – |
| GF(2¹²⁸) | 1,162 / – | – | – | – | – | 3,637° / – | **1,089** / – | 11,830 / – | 146,773° / – | 158,072 / – | 40,464° / – | – |
| GF(2²³³) | **10,269** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **9,176** / – | – | – | – | – | 10,896° / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **53,297** / – | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### sqrt: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **1.9** / – | 29.1 / – | 539 / – | 16.1 / – | 17.6 / – | 157° / – | 9.1° / – | – | – | – | – | – |
| GF(2¹⁶) | 2.4 / – | 826° / – | 2,175 / – | 360 / – | 257 / – | 847° / – | **2.3** / – | – | – | – | – | 2,004 / – |
| GF(2⁶⁴) | **36.6** / – | 7,954° / – | 25,269 / – | – | – | 36,406° / – | 45.3° / – | – | – | – | – | – |
| GF(2¹²⁸) | **67.5** / – | – | – | – | – | 244,307° / – | 128° / – | 24,849° / – | 76,406° / – | 71,759 / – | 20,025° / – | – |
| GF(2²³³) | **105** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **98.0** / – | – | – | – | – | 1,610,938° / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **1,298** / – | – | – | – | – | – | – | – | – | – | – | – |

° The crate's constant-time function, where it has no variable-time one.

### add, constant time: a + b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 1.7 / 1.6 | 1.6 / **1.0** | 1.8 / 1.2 | 1.6 / 1.1 | 1.9 / 1.5 | **1.5** / **1.0** | 1.6 / 1.2 | – | – | – | – | – |
| GF(2¹⁶) | 1.7 / 1.6 | **1.6** / 1.2 | 1.7 / **1.1** | **1.6** / 1.2 | 2.9 / 2.3 | 2.4 / 3.0 | 2.6 / 3.0 | – | – | – | – | 2.0 / **1.1** |
| GF(2⁶⁴) | 1.7 / 1.3 | **1.5** / **1.1** | **1.5** / **1.1** | – | – | 5.5 / 6.9 | 2.0 / 1.8 | – | – | – | – | – |
| GF(2¹²⁸) | 1.7 / 1.6 | – | – | – | – | 4.3 / 7.4 | 2.1 / 1.8 | 1.7 / 1.5 | **1.6** / 1.5 | **1.6** / **1.4** | 8.0 / 5.7 | – |
| GF(2²³³) | **3.4** / **2.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.9** / **2.2** | – | – | – | – | 9.0 / 7.4 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **8.8** / **5.7** | – | – | – | – | – | – | – | – | – | – | – |

### mul, constant time: a × b, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **14.0** / **21.9** | – | 14.5 / 23.2 | 16.3 / 31.0 | – | – | – |
| GF(2¹⁶) | **15.2** / **22.8** | 32.3 / 56.0 | 53.6 / 57.5 | – | – | – | – |
| GF(2⁶⁴) | **32.8** / **42.2** | 105 / 118 | 572 / 571 | 52.9 / 70.4 | – | – | – |
| GF(2¹²⁸) | **57.0** / **65.0** | – | 1,921 / 1,798 | 169 / 192 | 177 / 186 | 597 / 566 | 160 / 162 |
| GF(2²³³) | **206** / **197** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **193** / **189** | – | 6,078 / 5,997 | – | – | – | – |
| GF(2⁵⁷¹) | **1,429** / **1,402** | – | – | – | – | – | – |

### square, constant time: a², without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.6** / **11.1** | – | 9.1 / 18.5 | 9.2 / 23.1 | – | – | – |
| GF(2¹⁶) | **9.2** / **22.4** | 36.0 / 56.6 | 25.3 / 31.6 | – | – | – | – |
| GF(2⁶⁴) | **6.0** / **11.8** | 103 / 127 | 151 / 160 | 30.6 / 44.3 | – | – | – |
| GF(2¹²⁸) | **7.0** / **14.5** | – | 354 / 317 | 37.1 / 44.3 | 205 / 206 | 640 / 612 | 153 / 169 |
| GF(2²³³) | **21.3** / **32.2** | – | – | – | – | – | – |
| GF(2²⁵⁶) | **20.5** / **27.5** | – | 1,205 / 1,179 | – | – | – | – |
| GF(2⁵⁷¹) | **58.4** / **56.3** | – | – | – | – | – | – |

### invert, constant time: a⁻¹, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **112** / – | – | 264 / – | 321 / – | – | – |
| GF(2¹⁶) | **232** / – | 1,187 / – | 335 / – | – | – | – |
| GF(2⁶⁴) | **1,074** / – | 13,979 / – | 1,308 / – | 3,289 / – | – | – |
| GF(2¹²⁸) | **2,526** / – | – | 3,637 / – | – | 146,773 / – | 40,464 / – |
| GF(2²³³) | **10,354** / – | – | – | – | – | – |
| GF(2²⁵⁶) | **9,474** / – | – | 10,896 / – | – | – | – |
| GF(2⁵⁷¹) | **53,592** / – | – | – | – | – | – |

### sqrt, constant time: √a, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | binary-field | gf256 | hekate-math | plonky3 | runtime-algebra | compile-algebra | polyval |
|---|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **3.5** / – | – | 157 / – | 9.1 / – | – | – | – |
| GF(2¹⁶) | **15.0** / – | 826 / – | 847 / – | – | – | – | – |
| GF(2⁶⁴) | **36.6** / – | 7,954 / – | 36,406 / – | 45.3 / – | – | – | – |
| GF(2¹²⁸) | **67.8** / – | – | 244,307 / – | 128 / – | 24,849 / – | 76,406 / – | 20,025 / – |
| GF(2²³³) | **109** / – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **96.1** / – | – | 1,610,938 / – | – | – | – | – |
| GF(2⁵⁷¹) | **1,381** / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file
```
