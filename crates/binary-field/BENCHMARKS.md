# Benchmarks

`binary-field` at commit `0fbed1b` against `gf256`, `galois_field_2pm`, `g2p`, `reed-solomon-erasure`, `hekate-math`, `plonky3`, `runtime-algebra`, `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut`: add, mul, square, invert and sqrt over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), in instructions per operation, counted on 2026-11-02.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile, `-C target-cpu=native -C target-feature=-sse4a,-vpclmulqdq -C codegen-units=1` |
| Counter | Valgrind 3.22.0, Callgrind |
| Crates | gf256 0.3.1, galois_field_2pm 0.1.0, g2p 1.2.2, reed-solomon-erasure 6.0.0, hekate-math 0.12.1, p3-binary-field 0.9.0-rc.2, runtime-algebra 0.1.0, compile-algebra 0.1.0, core-algebra 0.1.0, polyval 0.7.3 |

## Method

Each case runs one crate's function over 256 operand sets, the same elements for every crate in a row, with operands and results behind `core::hint::black_box`. Callgrind counts the instructions of the loop over the batch, including the functions it calls; each count is that total divided by 256. The loop's own loads and stores are part of every count. Every instruction counts as one, whatever its latency: a table lookup weighs as much as an exclusive or, whatever the cache does with it. The build leaves out VPCLMULQDQ, whose 256-bit `vpclmulqdq` Valgrind cannot run. A native build may fuse two carry-less products into one such instruction. The counts are exact at any machine load. Before a case is counted, every result is checked against a bit-serial reference in the bench: the shift-and-add product reduced bit by bit, the extended Euclidean inverse confirmed by that product, and the root `r` of `a = r²`. Elements enter and leave each crate in its own representation; the conversions lie outside the count.

| Operation | Operands |
|---|---|
| add, mul | `a` and `b` uniform |
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

Each cell counts the crate's own function:

| Operation | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| add | `+` | `+` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `+` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `+` over GF(2⁸) and GF(2¹⁶) | `add` over GF(2⁸) and GF(2¹⁶) | `+` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `+` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `addf` over GF(2¹²⁸) | `+` over GF(2¹²⁸) | `+` over GF(2¹⁶) |
| mul | `*` | `*` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `*` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `*` over GF(2⁸) and GF(2¹⁶) | `mul` over GF(2⁸) and GF(2¹⁶) | `*` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `*` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | `mulf` over GF(2¹²⁸) | `mulf` over GF(2¹²⁸) | `mulf` over GF(2¹²⁸) | `*` over GF(2¹²⁸) | `*` over GF(2¹⁶) |
| square | `square` | – | – | – | – | `square` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `square` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | – | – | – | – | – |
| invert | `invert` | `checked_recip` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | `inverse` over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴) | – | – | `invert` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶) | `try_inverse` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | `invert` over GF(2¹²⁸) | `invert` over GF(2¹²⁸) | `invert` over GF(2¹²⁸) | – | `inverse` over GF(2¹⁶) |
| sqrt | `sqrt` | – | – | – | – | – | `try_sqrt` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴) and GF(2¹²⁸) | – | – | – | – | – |

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
- No squaring function: `gf256` square over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `galois_field_2pm` square over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` square over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` square over GF(2⁸) and GF(2¹⁶), `runtime-algebra` square over GF(2¹²⁸), `compile-algebra` square over GF(2¹²⁸), `core-algebra` square over GF(2¹²⁸), `polyval` square over GF(2¹²⁸) and `galois_field_2pm lut` square over GF(2¹⁶).
- No square root: `gf256` sqrt over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `galois_field_2pm` sqrt over GF(2⁸), GF(2¹⁶) and GF(2⁶⁴), `g2p` sqrt over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` sqrt over GF(2⁸) and GF(2¹⁶), `hekate-math` sqrt over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2¹²⁸) and GF(2²⁵⁶), `runtime-algebra` sqrt over GF(2¹²⁸), `compile-algebra` sqrt over GF(2¹²⁸), `core-algebra` sqrt over GF(2¹²⁸), `polyval` sqrt over GF(2¹²⁸) and `galois_field_2pm lut` sqrt over GF(2¹⁶).
- Fields of at most 127 bits: `galois_field_2pm` over GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- Fields of at most 32 bits: `g2p` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- No inverse function: `g2p` invert over GF(2⁸) and GF(2¹⁶), `reed-solomon-erasure` invert over GF(2⁸) and GF(2¹⁶) and `polyval` invert over GF(2¹²⁸).
- GF(2⁸) and GF(2¹⁶) only: `reed-solomon-erasure` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- No tower field of that degree: `hekate-math` over GF(2²³³) and GF(2⁵⁷¹).
- Plonky3 declares no such field: `plonky3` over GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- GF(2¹²⁸) only: `runtime-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), `compile-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹), `core-algebra` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹) and `polyval` over GF(2⁸), GF(2¹⁶), GF(2⁶⁴), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).
- The AES polynomial is not primitive: `galois_field_2pm lut` over GF(2⁸).
- Tables of at most 16 bits: `galois_field_2pm lut` over GF(2⁶⁴), GF(2¹²⁸), GF(2²³³), GF(2²⁵⁶) and GF(2⁵⁷¹).

## Results

Each cell: instructions per operation. Bold marks the fewest in each row.
The charts show the first 8 crates; `compile-algebra`, `core-algebra`, `polyval` and `galois_field_2pm lut` appear in the tables only.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | – | – | – | – | – |
| GF(2¹⁶) | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | – | – | – | – | **8.0** |
| GF(2⁶⁴) | **8.0** | **8.0** | **8.0** | – | – | **8.0** | **8.0** | – | – | – | – | – |
| GF(2¹²⁸) | **8.0** | – | – | – | – | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | – |
| GF(2²³³) | **8.0** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **8.0** | – | – | – | – | 11.1 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **14.0** | – | – | – | – | – | – | – | – | – | – | – |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 21.0 | 20.0 | 323 | **11.0** | **11.0** | 67.1 | 59.1 | – | – | – | – | – |
| GF(2¹⁶) | 21.0 | **17.0** | 627 | 22.1 | 31.1 | 239 | 69.1 | – | – | – | – | 31.0 |
| GF(2⁶⁴) | 17.0 | 13.0 | 1,353 | – | – | 2,516 | **13.0** | – | – | – | – | – |
| GF(2¹²⁸) | 26.0 | – | – | – | – | 8,043 | **21.0** | **21.0** | 78.1 | 2,068 | 526 | – |
| GF(2²³³) | **106** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **76.0** | – | – | – | – | 25,820 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **472** | – | – | – | – | – | – | – | – | – | – | – |

### square: a²

![square: a²](benchmarks/square.svg)

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | **20.0** | – | – | – | – | 33.0 | 34.1 | – | – | – | – | – |
| GF(2¹⁶) | 20.0 | – | – | – | – | 88.1 | **11.0** | – | – | – | – | – |
| GF(2⁶⁴) | **16.0** | – | – | – | – | 588 | 22.0 | – | – | – | – | – |
| GF(2¹²⁸) | 20.0 | – | – | – | – | 1,245 | **16.0** | – | – | – | – | – |
| GF(2²³³) | **52.1** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **34.0** | – | – | – | – | 3,950 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **97.1** | – | – | – | – | – | – | – | – | – | – | – |

### invert: a⁻¹

![invert: a⁻¹](benchmarks/invert.svg)

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 172 | **15.0** | 1,408 | – | – | 807 | 518 | – | – | – | – | – |
| GF(2¹⁶) | 317 | 386 | 3,482 | – | – | 1,078 | 95.1 | – | – | – | – | **25.1** |
| GF(2⁶⁴) | **994** | 1,025 | 28,265 | – | – | 4,870 | 1,081 | – | – | – | – | – |
| GF(2¹²⁸) | 2,053 | – | – | – | – | 13,718 | **493** | 6,368 | 13,665 | 533,598 | – | – |
| GF(2²³³) | **12,300** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **7,589** | – | – | – | – | 41,949 | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **62,025** | – | – | – | – | – | – | – | – | – | – | – |

### sqrt: √a

![sqrt: √a](benchmarks/sqrt.svg)

| Field | binary-field | gf256 | galois_field_2pm | g2p | reed-solomon-erasure | hekate-math | plonky3 | runtime-algebra | compile-algebra | core-algebra | polyval | galois_field_2pm lut |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| GF(2⁸) | 49.1 | – | – | – | – | – | **35.1** | – | – | – | – | – |
| GF(2¹⁶) | 57.1 | – | – | – | – | – | **12.0** | – | – | – | – | – |
| GF(2⁶⁴) | 61.1 | – | – | – | – | – | **58.1** | – | – | – | – | – |
| GF(2¹²⁸) | **55.1** | – | – | – | – | – | 81.1 | – | – | – | – | – |
| GF(2²³³) | **231** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2²⁵⁶) | **169** | – | – | – | – | – | – | – | – | – | – | – |
| GF(2⁵⁷¹) | **687** | – | – | – | – | – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p binary-field-bench-report -- bench    # Callgrind counts of the native build
cargo run -p binary-field-bench-report -- report   # this file and benchmarks/*.svg
```
