# Benchmarks

`binary-field` at commit `54b9917` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`: add, add_vartime, mul, mul_vartime, square, square_vartime, invert, invert_vartime, sqrt and sqrt_vartime over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), in cycles per operation, measured on 2026-11-02.

The CPU is an AMD Ryzen 9 5950X 16-Core Processor, measured on its CPU 15.
Its counters give each figure per operation as the median, over 31 repetitions, of a run of 2n operations on fresh operands less a run of n, over n.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Features | `binary-field` built with `precomputed` |
| RUSTFLAGS, with carry-less multiplication | `-C target-cpu=native -C codegen-units=1` |
| Extensions, with carry-less multiplication | adx, aes, avx, avx2, bmi1, bmi2, f16c, fma, lzcnt, movbe, pclmulqdq, popcnt, sha, sse, sse2, sse3, sse4.1, sse4.2, sse4a, ssse3, vaes, vpclmulqdq |
| RUSTFLAGS, without carry-less multiplication | `-C target-cpu=native -C target-feature=-pclmulqdq,-vpclmulqdq -C codegen-units=1 --cfg polyval_backend="soft"` |
| Extensions, without carry-less multiplication | adx, aes, avx, avx2, bmi1, bmi2, f16c, fma, lzcnt, movbe, popcnt, sha, sse, sse2, sse3, sse4.1, sse4.2, sse4a, ssse3, vaes |
| Counters | one Linux perf_event group: user-mode cycles and branch misses |
| Load, with carry-less multiplication | 1-minute load average 23.44 before the sweep and 16.61 after; CPU 31, the SMT sibling of CPU 15, busy 39% of the sweep |
| Load, without carry-less multiplication | 1-minute load average 12.23 before the sweep and 17.17 after; CPU 31, the SMT sibling of CPU 15, busy 10% of the sweep |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

A sweep measures every case in 31 repetitions; repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine reaches all cases alike. Each run draws fresh operand sets from the case's distribution, outside the counted region: no operand set runs twice, so the branch predictor never learns a sequence. A repetition runs n operations uncounted, then counts a run of n operations and a run of 2n. Before the repetitions, n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as many, takes 1,048,576 cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.

One perf_event group on the measuring thread, pinned to one CPU, counts user-mode cycles and branch misses; a run during which the kernel shares the counters with other groups stops the sweep. A figure per operation is the median, over the 31 repetitions, of the long run less the short run, divided by n: the fixed cost of a run cancels, as in nanoBench's differencing, and each difference pairs two runs taken back to back. A case whose long runs count no more cycles than its short runs stops the sweep, and so does a chain step under one cycle.

Throughput runs independent operations, with operands and results behind `core::hint::black_box`. Latency runs the chain `x = op(x, y)`: from the second operand set on, each result is the first operand of the next operation, so each operation waits for the one before. Each result passes through an empty inline-assembly block, which keeps the compiler from regrouping the chain. The block holds a result of up to 64 bytes in general-purpose registers in one chain and in vector registers in another, each chain a case of its own in the sweep; it can only add moves between the two, so the latency is the chain with fewer cycles. A larger result passes through memory, in a single chain. The chain applies where the result has the type of the first operand and the other operands are plain values: a step copies them, and the copy of an allocated value would allocate.

The sweeps ran at a 1-minute load of 2 or more, so their cycles compare within each table only.

Before a case is measured, its results on 256 operand sets are checked against a bit-serial reference in the bench: the shift-and-add product reduced bit by bit, the extended Euclidean inverse confirmed by that product, and the root `r` of `a = r²`. Elements enter and leave each crate in its own representation; the conversions lie outside the counted region. The chains are those of add, mul and square. The build without carry-less multiplication drops PCLMULQDQ and VPCLMULQDQ and selects the software backend of `polyval`, so every crate takes its portable product.

| Operation | Operands |
|---|---|
| add | `a` and `b` uniform |
| mul | `a` and `b` uniform and nonzero |
| square, invert | `a` uniform and nonzero |
| sqrt | `a = r²` for `r` uniform and nonzero |

Each row is a field in the polynomial basis of its modulus: bit `i` of an element is the coefficient of `xⁱ`.

| Field | Modulus |
|---|---|
| GF(2⁸) | x⁸ + x⁴ + x³ + x + 1, AES |
| GF(2¹⁶) | x¹⁶ + x⁵ + x³ + x² + 1 |
| GF(2⁶⁴) | x⁶⁴ + x⁴ + x³ + x + 1 |
| GF(2¹²⁸) | x¹²⁸ + x⁷ + x² + x + 1, GCM |
| GF(2²³³) | x²³³ + x⁷⁴ + 1, NIST B-233 |
| GF(2²⁵⁶) | x²⁵⁶ + x¹⁰ + x⁵ + x² + 1 |
| GF(2⁵⁷¹) | x⁵⁷¹ + x¹⁰ + x⁵ + x² + 1, NIST B-571 |

## Functions

Each cell measures the crate's own function:

| Operation | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| add | `Secret +` | `+` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `+` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `+` over GF(2⁸) and GF(2¹⁶) | `add` over GF(2⁸) and GF(2¹⁶) | `+` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `+` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `+` over GF(2¹²⁸) | `+` over GF(2¹⁶) |
| add_vartime | `+` | – | – | – | – | – | – | – | – | – | – | – |
| mul | `Secret *` | `*` over GF(2¹⁶) and GF(2⁶⁴) | – | – | – | `*` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `*` over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸) | `mulf` over GF(2¹²⁸) | `mulf` over GF(2¹²⁸) | – | `*` over GF(2¹²⁸) | – |
| mul_vartime | `*` | `*` over GF(2⁸) | `*` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `*` over GF(2⁸) and GF(2¹⁶) | `mul` over GF(2⁸) and GF(2¹⁶) | – | `*` over GF(2¹⁶) | – | – | `mulf` over GF(2¹²⁸) | – | `*` over GF(2¹⁶) |
| square | `Secret::square` | – | – | – | – | `square` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `square` over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸) | – | – | – | – | – |
| square_vartime | `square` | – | – | – | – | – | `square` over GF(2¹⁶) | – | – | – | – | – |
| invert | `Secret::invert` | `checked_recip` over GF(2¹⁶) and GF(2⁶⁴) | – | – | – | `invert` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `try_inverse` over GF(2⁸) and GF(2⁶⁴) | – | `invert` over GF(2¹²⁸) | – | – | – |
| invert_vartime | `invert` | `checked_recip` over GF(2⁸) | `inverse` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | – | – | – | `try_inverse` over GF(2¹⁶) and GF(2¹²⁸) | `invert` over GF(2¹²⁸) | – | `invert` over GF(2¹²⁸) | – | `inverse` over GF(2¹⁶) |
| sqrt | `Secret::sqrt` | – | – | – | – | – | `try_sqrt` over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸) | – | – | – | – | – |
| sqrt_vartime | `sqrt` | – | – | – | – | – | `try_sqrt` over GF(2¹⁶) | – | – | – | – | – |

- `gf256` is its `gf2p16` and `gf2p64`, and at 8 bits the AES field that its `gf` macro declares; the macro's default modes are log and antilog tables up to 8 bits and Barrett reduction above.
- `galois_field_2pm` is its `gf2` module, which multiplies by carry-less products; `galois_field_2pm lut` is its `gf2_lut` module, with log and antilog tables for a primitive polynomial.
- `g2p` multiplies by lookups in 256 × 256 tables of byte products.
- `reed-solomon-erasure` is its `galois_8` module, GF(2⁸) with x⁸ + x⁴ + x³ + x² + 1, and its `galois_16` module, the extension of that field by y² + 2y + 128.
- `hekate-math` is its tower fields `Block8` to `Block256`: `Block8` is the AES field, and each level above extends the one below by X² + X + τ.
- `plonky3` is `p3-binary-field`: `Rijndael8b`, `Poly64` and `Ghash128` in the polynomial bases of the rows, and `BinaryField16` in the Wiedemann tower.
- `runtime-algebra`, `compile-algebra` and `core-algebra` are the `Gf2_128Field` of longfellow-zk's prover, compiler and reference.
- `polyval` is its `hazmat::FieldElement`, whose product is the crate's portable constant-time code. An element `a` enters POLYVAL's representation (RFC 8452) as the block of the bit reversal of `a·x⁻¹`, the map that carries POLYVAL's product to the GCM field's.
- Where a crate's basis differs from the row's (`reed-solomon-erasure`, `hekate-math` from 16 bits, `plonky3` at 16 bits), operands and results pass through the field isomorphism that the bench derives from the crate's tower: a root of the base polynomial, then a root of each quadratic.
- Fields of at most 64 bits: `gf256` over GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- Constant time, in the constant-time row: `gf256` add_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), mul_vartime over GF(2¹⁶) and GF(2⁶⁴) and invert_vartime over GF(2¹⁶) and GF(2⁶⁴), `galois_field_2pm` add_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` add_vartime over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` add_vartime over GF(2⁸) and GF(2¹⁶), `hekate-math` add_vartime, mul_vartime, square_vartime and invert_vartime over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶), `plonky3` add_vartime over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸), mul_vartime over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸), square_vartime over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸), invert_vartime over GF(2⁸) and GF(2⁶⁴) and sqrt_vartime over GF(2⁸), GF(2⁶⁴) and GF(2¹²⁸), `runtime-algebra` add_vartime and mul_vartime over GF(2¹²⁸), `compile-algebra` add_vartime, mul_vartime and invert_vartime over GF(2¹²⁸), `core-algebra` add_vartime over GF(2¹²⁸), `polyval` add_vartime and mul_vartime over GF(2¹²⁸) and `galois_field_2pm lut` add_vartime over GF(2¹⁶).
- Variable time, in the variable-time row: `gf256` mul and invert over GF(2⁸), `galois_field_2pm` mul and invert over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` mul over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` mul over GF(2⁸) and GF(2¹⁶), `plonky3` mul over GF(2¹⁶), square over GF(2¹⁶), invert over GF(2¹⁶) and GF(2¹²⁸) and sqrt over GF(2¹⁶), `runtime-algebra` invert over GF(2¹²⁸), `core-algebra` mul and invert over GF(2¹²⁸) and `galois_field_2pm lut` mul and invert over GF(2¹⁶).
- No squaring function: `gf256` square and square_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `galois_field_2pm` square and square_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` square and square_vartime over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` square and square_vartime over GF(2⁸) and GF(2¹⁶), `runtime-algebra` square and square_vartime over GF(2¹²⁸), `compile-algebra` square and square_vartime over GF(2¹²⁸), `core-algebra` square and square_vartime over GF(2¹²⁸), `polyval` square and square_vartime over GF(2¹²⁸) and `galois_field_2pm lut` square and square_vartime over GF(2¹⁶).
- No square root: `gf256` sqrt and sqrt_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `galois_field_2pm` sqrt and sqrt_vartime over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` sqrt and sqrt_vartime over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` sqrt and sqrt_vartime over GF(2⁸) and GF(2¹⁶), `hekate-math` sqrt and sqrt_vartime over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶), `runtime-algebra` sqrt and sqrt_vartime over GF(2¹²⁸), `compile-algebra` sqrt and sqrt_vartime over GF(2¹²⁸), `core-algebra` sqrt and sqrt_vartime over GF(2¹²⁸), `polyval` sqrt and sqrt_vartime over GF(2¹²⁸) and `galois_field_2pm lut` sqrt and sqrt_vartime over GF(2¹⁶).
- Fields of at most 127 bits: `galois_field_2pm` over GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- Fields of at most 32 bits: `g2p` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- No inverse function: `g2p` invert and invert_vartime over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` invert and invert_vartime over GF(2⁸) and GF(2¹⁶) and `polyval` invert and invert_vartime over GF(2¹²⁸).
- GF(2⁸) and GF(2¹⁶) only: `reed-solomon-erasure` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- No tower field of that degree: `hekate-math` over GF(2²³³) and GF(2⁵⁷¹).
- Plonky3 declares no such field: `plonky3` over GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- GF(2¹²⁸) only: `runtime-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), `compile-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), `core-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹) and `polyval` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- The AES polynomial is not primitive: `galois_field_2pm lut` over GF(2⁸).
- Tables of at most 16 bits: `galois_field_2pm lut` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).

## Results with carry-less multiplication

The charts show the first 8 crates; `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut` appear in the tables only.

### add: a + b, constant time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.0 / 3.1 | 5.0 / 3.1 | 5.1 / 3.1 | 5.0 / **3.0** | **2.2** / 3.5 | 5.1 / 3.0 | 5.0 / 3.0 | – | – | – | – | – |
| GF(2¹⁶) | 5.1 / 3.0 | 2.2 / 3.0 | 2.2 / **3.0** | **2.1** / 3.0 | 2.6 / 3.3 | 2.5 / 3.7 | 2.9 / 3.3 | – | – | – | – | 2.7 / 3.1 |
| GF(2⁶⁴) | 2.2 / 2.1 | 2.2 / **1.3** | 2.2 / 1.3 | – | – | 6.3 / 7.4 | **2.2** / 1.3 | – | – | – | – | – |
| GF(2¹²⁸) | 2.2 / 2.1 | – | – | – | – | 8.9 / 9.0 | 2.3 / 2.1 | 2.2 / **1.5** | **2.2** / 2.0 | 2.3 / 2.0 | 13.5 / 9.9 | – |
| GF(2²³³) | **4.3** / **2.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.5** / **2.6** | – | – | – | – | 16.6 / 19.2 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **9.5** / **19.9** | – | – | – | – | – | – | – | – | – | – | – |

### add_vartime: a + b, variable time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.0** / **3.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | **5.0** / **3.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁶⁴) | **2.2** / **2.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **2.3** / **2.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **3.4** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **3.6** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **7.6** / **19.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, constant time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **8.0** / **21.0** | – | – | – | – | 14.0 / 26.0 | 16.9 / 33.2 | – | – | – | – | – |
| GF(2¹⁶) | 8.2 / **21.0** | **8.1** / 29.1 | – | – | – | 53.1 / 54.4 | – | – | – | – | – | – |
| GF(2⁶⁴) | 6.1 / **14.8** | **6.0** / 20.1 | – | – | – | 548 / 554 | 6.0 / 19.1 | – | – | – | – | – |
| GF(2¹²⁸) | **11.0** / 17.1 | – | – | – | – | 1,785 / 1,822 | 12.0 / **16.1** | 12.0 / 18.7 | 30.0 / 30.3 | – | 118 / 134 | – |
| GF(2²³³) | **53.6** / **55.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **35.3** | – | – | – | – | 5,737 / 6,014 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **275** / **283** | – | – | – | – | – | – | – | – | – | – | – |

### mul_vartime: a × b, variable time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.3 / **11.7** | 3.8 / 13.5 | 84.6 / 92.8 | 5.5 / 13.4 | **2.7** / 13.5 | – | – | – | – | – | – | – |
| GF(2¹⁶) | **8.0** / **21.0** | – | 144 / 151 | 11.3 / 46.1 | 9.1 / 27.8 | – | 14.8 / 30.5 | – | – | – | – | 13.5 / 65.2 |
| GF(2⁶⁴) | **6.2** / **14.8** | – | 546 / 568 | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **11.0** / **17.1** | – | – | – | – | – | – | – | – | 590 / 561 | – | – |
| GF(2²³³) | **53.5** / **55.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** / **35.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **274** / **283** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², constant time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.7** / **17.0** | – | – | – | – | 8.4 / 21.1 | 8.6 / 25.1 | – | – | – | – | – |
| GF(2¹⁶) | **7.0** / **21.0** | – | – | – | – | 22.4 / 35.2 | – | – | – | – | – | – |
| GF(2⁶⁴) | 6.1 / **15.5** | – | – | – | – | 133 / 146 | **5.1** / 16.5 | – | – | – | – | – |
| GF(2¹²⁸) | **6.7** / **14.5** | – | – | – | – | 306 / 297 | 8.0 / 16.6 | – | – | – | – | – |
| GF(2²³³) | **34.8** / **44.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.1** / **19.7** | – | – | – | – | 1,082 / 1,127 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.3** / **56.2** | – | – | – | – | – | – | – | – | – | – | – |

### square_vartime: a², variable time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / **7.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | 5.4 / **7.1** | – | – | – | – | – | **2.6** / 10.1 | – | – | – | – | – |
| GF(2⁶⁴) | **6.2** / **15.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **6.8** / **14.5** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **34.8** / **44.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **13.2** / **19.8** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **41.3** / **56.3** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, constant time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **88.5** / – | – | – | – | – | 260 / – | 317 / – | – | – | – | – | – |
| GF(2¹⁶) | **204** / – | 421 / – | – | – | – | 339 / – | – | – | – | – | – | – |
| GF(2⁶⁴) | 1,099 / – | 1,228 / – | – | – | – | 1,257 / – | **1,066** / – | – | – | – | – | – |
| GF(2¹²⁸) | **2,039** / – | – | – | – | – | 3,420 / – | – | – | 5,211 / – | – | – | – |
| GF(2²³³) | **11,071** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,496** / – | – | – | – | – | 9,766 / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28,828** / – | – | – | – | – | – | – | – | – | – | – | – |

### invert_vartime: a⁻¹, variable time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.0 / – | **3.5** / – | 385 / – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | **3.7** / – | – | 1,284 / – | – | – | – | 20.7 / – | – | – | – | – | 14.4 / – |
| GF(2⁶⁴) | **363** / – | – | 10,299 / – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | 623 / – | – | – | – | – | – | **557** / – | 6,035 / – | – | 144,796 / – | – | – |
| GF(2²³³) | **11,081** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **5,296** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **28,802** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, constant time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.7** / – | – | – | – | – | – | 9.0 / – | – | – | – | – | – |
| GF(2¹⁶) | **13.8** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁶⁴) | 19.3 / – | – | – | – | – | – | **16.0** / – | – | – | – | – | – |
| GF(2¹²⁸) | **16.1** / – | – | – | – | – | – | 19.9 / – | – | – | – | – | – |
| GF(2²³³) | **79.1** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **48.3** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **318** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt_vartime: √a, variable time, with carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.1** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | 5.1 / – | – | – | – | – | – | **3.2** / – | – | – | – | – | – |
| GF(2⁶⁴) | **19.8** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **16.5** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **77.4** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **48.7** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **343** / – | – | – | – | – | – | – | – | – | – | – | – |

## Results without carry-less multiplication

The charts show the first 8 crates; `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut` appear in the tables only.

### add: a + b, constant time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.0 / 3.0 | 5.0 / 3.0 | 5.0 / 3.0 | 5.0 / **3.0** | **2.1** / 3.0 | 5.0 / 3.0 | 5.0 / 3.0 | – | – | – | – | – |
| GF(2¹⁶) | 5.0 / 3.0 | **2.1** / **3.0** | 2.1 / 3.0 | 2.1 / 3.0 | 2.3 / 3.1 | 2.4 / 3.2 | 2.3 / 3.0 | – | – | – | – | 2.1 / 3.0 |
| GF(2⁶⁴) | **2.2** / 1.3 | 2.2 / **1.3** | 2.2 / **1.3** | – | – | 2.9 / 2.2 | 2.2 / 1.3 | – | – | – | – | – |
| GF(2¹²⁸) | **2.2** / **1.3** | – | – | – | – | 3.9 / 3.5 | 2.6 / 2.0 | 2.2 / 2.1 | 2.2 / 2.1 | 2.2 / 2.1 | 5.1 / 3.2 | – |
| GF(2²³³) | **2.8** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.6** | – | – | – | – | 8.3 / 8.5 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.4** / **23.9** | – | – | – | – | – | – | – | – | – | – | – |

### add_vartime: a + b, variable time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.0** / **3.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | **5.0** / **3.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁶⁴) | **2.2** / **1.3** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **2.2** / **1.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **2.7** / **2.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **2.7** / **2.4** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **6.2** / **23.9** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b, constant time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 28.0 / 40.2 | – | – | – | – | **14.2** / **25.0** | 16.8 / 33.4 | – | – | – | – | – |
| GF(2¹⁶) | 45.0 / **52.0** | **30.6** / 58.4 | – | – | – | 54.1 / 53.7 | – | – | – | – | – | – |
| GF(2⁶⁴) | 66.0 / **78.4** | 104 / 121 | – | – | – | 550 / 548 | **47.3** / 84.7 | – | – | – | – | – |
| GF(2¹²⁸) | 235 / 232 | – | – | – | – | 1,763 / 1,764 | 162 / 187 | 174 / 182 | 578 / 577 | – | **122** / **133** | – |
| GF(2²³³) | **819** / **854** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **887** / **950** | – | – | – | – | 5,734 / 5,738 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **4,972** / **4,924** | – | – | – | – | – | – | – | – | – | – | – |

### mul_vartime: a × b, variable time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.3 / **10.5** | 3.7 / 13.4 | 81.9 / 83.9 | 5.3 / 12.5 | **2.9** / 12.7 | – | – | – | – | – | – | – |
| GF(2¹⁶) | 45.0 / 52.0 | – | 143 / 148 | **8.4** / 38.1 | 8.8 / **25.9** | – | 14.6 / 30.1 | – | – | – | – | 11.4 / 56.0 |
| GF(2⁶⁴) | **66.0** / **78.6** | – | 542 / 561 | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **235** / **232** | – | – | – | – | – | – | – | – | 547 / 546 | – | – |
| GF(2²³³) | **819** / **852** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **886** / **940** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **4,982** / **4,926** | – | – | – | – | – | – | – | – | – | – | – |

### square: a², constant time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.6** / **17.0** | – | – | – | – | 8.1 / 21.0 | 8.6 / 25.0 | – | – | – | – | – |
| GF(2¹⁶) | 25.8 / **31.0** | – | – | – | – | **22.0** / 35.0 | – | – | – | – | – | – |
| GF(2⁶⁴) | 49.3 / **23.0** | – | – | – | – | 129 / 143 | **27.0** / 45.0 | – | – | – | – | – |
| GF(2¹²⁸) | **30.7** / 60.7 | – | – | – | – | 304 / 293 | 39.0 / **44.0** | – | – | – | – | – |
| GF(2²³³) | **48.0** / **56.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.4** / **102** | – | – | – | – | 1,015 / 1,104 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **220** / **235** | – | – | – | – | – | – | – | – | – | – | – |

### square_vartime: a², variable time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.3** / **7.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | 5.3 / **7.0** | – | – | – | – | – | **2.6** / 10.0 | – | – | – | – | – |
| GF(2⁶⁴) | **49.3** / **23.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **30.7** / **63.6** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **48.0** / **56.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **93.5** / **102** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **220** / **235** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹, constant time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **239** / – | – | – | – | – | 255 / – | 311 / – | – | – | – | – | – |
| GF(2¹⁶) | 481 / – | 1,116 / – | – | – | – | **337** / – | – | – | – | – | – | – |
| GF(2⁶⁴) | 2,202 / – | 14,020 / – | – | – | – | **1,249** / – | 3,247 / – | – | – | – | – | – |
| GF(2¹²⁸) | 12,320 / – | – | – | – | – | **3,262** / – | – | – | 144,137 / – | – | – | – |
| GF(2²³³) | **21,352** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | 40,729 / – | – | – | – | – | **9,625** / – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **188,274** / – | – | – | – | – | – | – | – | – | – | – | – |

### invert_vartime: a⁻¹, variable time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 5.3 / – | **3.0** / – | 380 / – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | **3.4** / – | – | 1,197 / – | – | – | – | 20.3 / – | – | – | – | – | 9.8 / – |
| GF(2⁶⁴) | **1,136** / – | – | 9,740 / – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | 3,608 / – | – | – | – | – | – | **976** / – | 11,574 / – | – | 147,346 / – | – | – |
| GF(2²³³) | **21,314** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **40,768** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **189,244** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a, constant time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **4.6** / – | – | – | – | – | – | 8.6 / – | – | – | – | – | – |
| GF(2¹⁶) | **13.6** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁶⁴) | 83.6 / – | – | – | – | – | – | **40.4** / – | – | – | – | – | – |
| GF(2¹²⁸) | 180 / – | – | – | – | – | – | **124** / – | – | – | – | – | – |
| GF(2²³³) | **108** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **140** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **5,073** / – | – | – | – | – | – | – | – | – | – | – | – |

### sqrt_vartime: √a, variable time, without carry-less multiplication

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **5.0** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹⁶) | 5.3 / – | – | – | – | – | – | **3.2** / – | – | – | – | – | – |
| GF(2⁶⁴) | **85.8** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2¹²⁸) | **180** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²³³) | **108** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **141** / – | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **5,072** / – | – | – | – | – | – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p binary-field-bench-report -- report   # this file and benchmarks/*.svg
```
