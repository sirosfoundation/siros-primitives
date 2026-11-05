# Benchmarks

`prime-field` at commit `f8b7bef` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 53.39 before, 33.14 after; its SMT sibling, CPU 29, 62% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands: canonical residues, the same for every crate.
- Integer operands `k`: uniform of exactly 64 bits, or of exactly the bit length of `p`, which may reach `p`. A crate without an operation on an integer runs its own conversion of `k` and then its operation on two elements.
- Divisors `k`: nonzero, uniform of exactly 64 bits over the fields of more than 64 bits, or of exactly one bit less than `p`. A crate without a division by an integer runs its own conversion of `k`, its inverse and a product; `prime-field × invert` is that composition in `prime-field`.
- Every result matches `num-bigint` before it is measured.
- `prime-field` runs `Fp` in the tables without a label and `ConstantTime` in the constant-time tables.

## Functions

| Operation | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---|---|---|---|---|---|---|---|---|---|
| add | `+` | – | `+` | `+` | `+` | `+` | `+` | `+` | `addf` | `addf` |
| sub | `-` | – | `-` | `-` | `-` | `-` | `-` | `-` | `subf` | `subf` |
| mul | `*` | – | `*` | `*` | `*` | `*` | `*` | `*` | `mulf` | `mulf` |
| square | `square` | – | `square` | `square` | `square` | `square` | `square` | `square` | `mulf(a, a)` | `mulf(a, a)` |
| invert | `invert` | – | `inverse` | `invert_vartime` | `invert`, `invert_vartime` | `invert` | `try_inverse` | `inv` | `invert` | `invert` |
| sqrt | `sqrt` | – | `sqrt` | `sqrt` | `sqrt` | `sqrt` | `try_sqrt` | – | `mulf to (p + 1) / 4` | `mulf to (p + 1) / 4` |
| mul_u64 | `* Uint<U64>` | – | `* From<u64>` | `* new` | `* from_u64`, `* From<u64>` | `* From<u64>` | `* from_int` | `* new` | `mulf of words64_to_element` | – |
| mul_uint | `* Uint` | – | – | `* new` | `* Reduce<U256>` | `* from_raw` | `* from_int` | `* new` | – | – |
| add_u64 | `+ Uint<U64>` | – | `+ From<u64>` | `+ new` | `+ from_u64`, `+ From<u64>` | `+ From<u64>` | `+ from_int` | `+ new` | `addf of words64_to_element` | – |
| add_uint | `+ Uint` | – | – | `+ new` | `+ Reduce<U256>` | `+ from_raw` | `+ from_int` | `+ new` | – | – |
| sub_u64 | `- Uint<U64>` | – | `- From<u64>` | `- new` | `- from_u64`, `- From<u64>` | `- From<u64>` | `- from_int` | `- new` | `subf of words64_to_element` | – |
| sub_uint | `- Uint` | – | – | `- new` | `- Reduce<U256>` | `- from_raw` | `- from_int` | `- new` | – | – |
| from_u64 | `From<u64>` | – | `From<u64>` | `new` | `from_u64`, `From<u64>` | `From<u64>` | `from_int` | `new` | `words64_to_element` | – |
| div_u64 | `/ NonZeroUint<U64>` | `* NonZero(From<u64>).invert()` | `* inverse of From<u64>` | `* invert_vartime of new` | `* invert of from_u64`, `* invert of From<u64>`, `* invert_vartime of From<u64>` | `* invert of From<u64>` | – | – | `mulf by invert of words64_to_element` | – |
| div_uint | `/ NonZeroUint` | `* NonZero(from_uint).invert()` | `* inverse of from_bigint` | `* invert_vartime of new` | `* invert of from_bytes`, `* invert_vartime of Reduce<U256>` | `* invert of from_raw` | `* try_inverse of from_int` | `* inv of new` | `mulf by invert of words64_to_element` | – |

- `prime-field`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `prime-field × invert`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `ark-ff`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base.
- `crypto-bigint`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `k256/p256`: add, sub, mul, square, invert, sqrt, mul_u64, add_u64, sub_u64, from_u64, div_u64 and div_uint over secp256k1 base, P-256 base and P-256 scalar; mul_uint, add_uint and sub_uint over P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31, BabyBear and Goldilocks.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base; mul_u64, add_u64, sub_u64, from_u64 and div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `compile-algebra`: add, sub, mul, square and invert over P-256 base and P-256 scalar; sqrt over P-256 base.

## Functions, constant time

| Operation | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---|---|---|---|---|---|---|---|
| add | `ConstantTime +` | – | `+` | `+` | `+` | – | `+` | `addf` |
| sub | `ConstantTime -` | – | `-` | `-` | `-` | `-` | `-` | `subf` |
| mul | `ConstantTime *` | – | `*` | `*` | `*` | – | `*` | `mulf` |
| square | `ConstantTime::square` | – | `square` | `square` | `square` | – | `square` | `mulf(a, a)` |
| invert | `ConstantTime::invert` | – | `invert` | `invert` | `pow(p − 2)` | – | `inv` | `mulf to p − 2` |
| sqrt | `ConstantTime::sqrt` | – | `sqrt` | `sqrt` | `sqrt` | – | – | `mulf to (p + 1) / 4` |
| mul_u64 | `ConstantTime * ConstantTime<Uint<U64>>` | – | `* new` | `* from_u64`, `* From<u64>` | `* From<u64>` | – | `* new` | – |
| mul_uint | `ConstantTime * ConstantTime<Uint>` | – | `* new` | `* Reduce<U256>` | `* from_raw` | – | `* new` | – |
| add_u64 | `ConstantTime + ConstantTime<Uint<U64>>` | – | `+ new` | `+ from_u64`, `+ From<u64>` | `+ From<u64>` | – | `+ new` | – |
| add_uint | `ConstantTime + ConstantTime<Uint>` | – | `+ new` | `+ Reduce<U256>` | `+ from_raw` | – | `+ new` | – |
| sub_u64 | `ConstantTime - ConstantTime<Uint<U64>>` | – | `- new` | `- from_u64`, `- From<u64>` | `- From<u64>` | – | `- new` | – |
| sub_uint | `ConstantTime - ConstantTime<Uint>` | – | `- new` | `- Reduce<U256>` | `- from_raw` | – | `- new` | – |
| from_u64 | `ConstantTime::new(from_u64)` | – | `new` | `from_u64`, `From<u64>` | `From<u64>` | – | `new` | – |
| div_u64 | `ConstantTime / ConstantTime<NonZeroUint<U64>>` | `ConstantTime * (ConstantTime + ConstantTime<Uint<U64>>).nonzero().invert()` | `* invert of new` | `* invert of from_u64`, `* invert of From<u64>` | `* pow(p − 2) of From<u64>` | – | – | `mulf by mulf to p − 2 of words64_to_element` |
| div_uint | `ConstantTime / ConstantTime<NonZeroUint>` | `ConstantTime * (ConstantTime + ConstantTime<Uint>).nonzero().invert()` | `* invert of new` | `* invert of from_bytes`, `* invert of Reduce<U256>` | `* pow(p − 2) of from_raw` | – | `* inv of new` | `mulf by mulf to p − 2 of words64_to_element` |

- `prime-field`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `prime-field × invert`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `crypto-bigint`: div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `k256/p256`: add, sub, mul, square, invert, sqrt, mul_u64, add_u64, sub_u64, from_u64, div_u64 and div_uint over secp256k1 base, P-256 base and P-256 scalar; mul_uint, add_uint and sub_uint over P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base; div_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 9.6 / 17.2 | – | – | 1.9 / 3.0 | – | 2.4 / 5.0 | – |
| BabyBear | **1.9** / **2.9** | – | 9.6 / 17.0 | – | – | 2.0 / 3.0 | – | 2.6 / 4.9 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 19.0 | – | – | 1.9\* / 3.0 | 2.4 / **2.9** | 2.5 / 4.9 | – |
| BN254 scalar | **5.0** / **6.7** | 17.3 / 11.4 | 66.5 / 85.7 | – | 12.7 / 22.0 | – | – | 6.8 / 10.0 | – |
| secp256k1 base | **5.8** / **7.0** | 17.1 / 12.9 | 66.7 / 93.1 | 4.5\* / 3.7\* | 14.3 / 21.1 | – | – | 6.9 / 10.4 | – |
| P-256 base | **5.8** / **6.8** | 18.9 / 13.3 | 66.5 / 84.5 | 10.6 / 18.5 | 10.7 / 18.3 | – | – | 6.8 / 9.6 | 309 / – |
| P-256 scalar | **5.8** / **6.9** | 17.7 / 13.5 | 66.9 / 79.0 | 17.4 / 21.1 | 14.3 / 21.0 | – | – | 7.0 / 10.4 | 317 / – |
| Tom-256 base | **5.9** / **7.0** | – | 66.4 / 91.7 | – | 11.6 / 20.5 | – | – | 6.9 / 9.8 | – |
| BLS12-381 base | **9.9** / **9.0** | 19.9 / 13.4 | 87.3 / 112 | – | 20.2 / 27.3 | – | – | 16.4 / 12.5 | – |
| P-521 base | **12.2** / **14.9** | – | 117 / 155 | – | – | – | – | 25.0 / 51.4 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 3.9 | – |
| BabyBear | **1.9** / **2.9** | – | 4.2 / 11.9 | – | – | 2.0 / 3.0 | – | **1.9** / 3.9 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.3 / 11.7 | – | – | 1.9\* / 2.9\* | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.2** / **6.8** | 16.2 / 13.1 | 50.0 / 50.8 | – | 8.3 / 11.7 | – | – | 6.8 / 7.8 | – |
| secp256k1 base | **5.2** / **7.8** | 17.3 / 13.1 | 49.7 / 47.1 | 4.0\* / 3.4\* | 7.5 / 12.8 | – | – | 6.8 / **7.8** | – |
| P-256 base | **5.4** / **6.8** | 16.6 / 11.5 | 49.6 / 43.3 | 7.8 / 12.8 | 7.5 / 11.7 | – | – | 6.7 / 7.8 | 149 / – |
| P-256 scalar | **5.0** / **6.8** | 15.3 / 13.5 | 50.0 / 47.3 | 7.6 / 12.8 | 7.9 / 13.0 | – | – | 6.8 / 7.7 | 151 / – |
| Tom-256 base | **5.4** / **7.8** | – | 49.6 / 53.3 | – | 8.3 / 11.7 | – | – | 6.8 / **7.8** | – |
| BLS12-381 base | **10.0** / **10.0** | 21.4 / 15.7 | 82.4 / 87.7 | – | 15.2 / 19.7 | – | – | 16.7 / **10.0** | – |
| P-521 base | **11.7** / **12.4** | – | 121 / 138 | – | – | – | – | 27.9 / 50.9 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **6.9** | – | 16.9 / 34.7 | – | – | 2.8 / 7.0 | – | 7.0 / 17.0 | – |
| BabyBear | **3.6** / **10.9** | – | 16.9 / 34.4 | – | – | 4.8 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | **3.7** / **9.0** | – | 17.6 / 34.3 | – | – | 3.2\* / 9.9 | **3.7** / 11.0 | 6.9 / 16.9 | – |
| BN254 scalar | **68.8** / **72.4** | 74.9 / 102 | 119 / 150 | – | 92.1 / 93.8 | – | – | 94.7 / 127 | – |
| secp256k1 base | **44.3** / **54.8** | 89.2 / 114 | 118 / 153 | 84.4 / 87.5 | 91.8 / 95.8 | – | – | 94.7 / 128 | – |
| P-256 base | **49.3** / **60.0** | 66.9 / 87.3 | 117 / 157 | 67.5 / 72.2 | 66.8 / 73.0 | – | – | 50.8 / 63.7 | 564 / – |
| P-256 scalar | **72.7** / **83.9** | 92.7 / 113 | 119 / 152 | 140 / 141 | 93.9 / 99.8 | – | – | 94.5 / 128 | 631 / – |
| Tom-256 base | **71.1** / **75.8** | – | 117 / 151 | – | 92.1 / 94.8 | – | – | 94.6 / 129 | – |
| BLS12-381 base | **138** / **141** | 165 / 195 | 270 / 290 | – | 192 / 194 | – | – | 249 / 274 | – |
| P-521 base | **171** / **190** | – | 529 / 548 | – | – | – | – | 464 / 476 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 33.9 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.7 / 33.9 | – | – | 4.6 / 13.0 | – | 7.5 / 17.0 | – |
| Goldilocks | **3.7** / **9.0** | – | 17.5 / 35.0 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.5 / 17.0 | – |
| BN254 scalar | **63.2** / **66.9** | 74.4 / 99.7 | 120 / 151 | – | 75.7 / 89.4 | – | – | 93.5 / 129 | – |
| secp256k1 base | **32.7** / **42.9** | 86.1 / 101 | 111 / 151 | 57.9 / 78.5 | 74.5 / 84.9 | – | – | 93.7 / 130 | – |
| P-256 base | **40.1** / **43.9** | 64.7 / 80.7 | 121 / 153 | 60.9 / 71.9 | 54.9 / 57.9 | – | – | 57.2 / 57.9 | 563 / – |
| P-256 scalar | **64.4** / **72.9** | 88.1 / 111 | 116 / 153 | 129 / 147 | 76.9 / 93.5 | – | – | 94.0 / 129 | 637 / – |
| Tom-256 base | **56.9** / **64.4** | – | 115 / 151 | – | 69.5 / 82.9 | – | – | 93.9 / 128 | – |
| BLS12-381 base | **134** / **137** | 173 / 167 | 267 / 290 | – | 151 / 166 | – | – | 253 / 263 | – |
| P-521 base | **123** / **133** | – | 538 / 563 | – | – | – | – | 466 / 474 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **210** / – | – | 1,425 / – | – | – | 638 / – | – | 876 / – | – |
| BabyBear | **212** / – | – | 1,407 / – | – | – | 636 / – | – | 1,046 / – | – |
| Goldilocks | **431** / – | – | 2,853 / – | – | – | 1,312 / – | 691 / – | 1,775 / – | – |
| BN254 scalar | **5,122** / – | 17,384 / – | 14,192 / – | – | 7,690 / – | – | – | 13,503 / – | – |
| secp256k1 base | **4,842** / – | 15,878 / – | 14,340 / – | 24,819 / – | 7,224 / – | – | – | 14,231 / – | – |
| P-256 base | **4,855** / – | 15,221 / – | 14,351 / – | 28,402 / – | 7,180 / – | – | – | 14,842 / – | 89,163 / – |
| P-256 scalar | **5,046** / – | 16,547 / – | 14,381 / – | 30,091 / – | 7,244 / – | – | – | 14,235 / – | 89,561 / – |
| Tom-256 base | **5,049** / – | – | 14,657 / – | – | 7,226 / – | – | – | 14,210 / – | – |
| BLS12-381 base | **8,833** / – | 36,770 / – | 25,296 / – | – | 12,141 / – | – | – | 34,496 / – | – |
| P-521 base | **14,210** / – | – | 43,329 / – | – | – | – | – | 70,617 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **136** / – | – | 1,648 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,987** / – | – | 18,352 / – | – | – | 2,811 / – | – | – |
| Goldilocks | **2,845** / – | – | 26,591 / – | – | – | 3,154 / – | – | – |
| BN254 scalar | **34,616** / – | 46,318 / – | 134,017 / – | – | 85,097 / – | – | – | – |
| secp256k1 base | **12,235** / – | 44,777 / – | 54,411 / – | 26,445 / – | 47,952 / – | – | 69,560 / – | – |
| P-256 base | **12,111** / – | 18,741 / – | 43,129 / – | 28,638 / – | 33,607 / – | – | 29,405 / – | 163,311 / – |
| P-256 scalar | **24,192** / – | 40,074 / – | 51,869 / – | 81,228 / – | 41,859 / – | – | – | – |
| Tom-256 base | **21,349** / – | – | 48,728 / – | – | 44,141 / – | – | 52,426 / – | – |
| BLS12-381 base | **65,946** / – | 90,551 / – | 135,135 / – | – | 141,300 / – | – | 179,228 / – | – |
| P-521 base | **68,286** / – | – | 263,561 / – | – | – | – | 258,476 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.8** / **8.9** | – | 34.6 / 36.4 | – | – | 5.5\* / 6.9\* | – | – |
| BabyBear | **8.9** / **10.9** | – | 34.3 / 36.4 | – | – | 11.9 / 14.3 | – | – |
| Goldilocks | **6.9** / **9.1** | – | 38.0 / 38.0 | – | – | 3.2\* / 9.9 | 7.3 / 11.0 | – |
| BN254 scalar | **20.9** / **25.9** | 236 / 236 | 258 / 266 | – | 236 / 233 | – | – | 254 / 249 |
| secp256k1 base | **9.7** / **14.9** | 231 / 230 | 282 / 269 | 82.7 / 90.8 | 207 / 218 | – | – | 252 / 250 |
| P-256 base | **18.9** / **25.0** | 206 / 207 | 272 / 271 | 147 / 137 | 184 / 177 | – | – | 140 / 133 |
| P-256 scalar | **22.4** / **27.9** | 259 / 261 | 274 / 265 | 139 / 146 | 248 / 235 | – | – | 253 / 251 |
| Tom-256 base | **20.4** / **27.9** | – | 273 / 268 | – | 225 / 227 | – | – | 255 / 250 |
| BLS12-381 base | **28.9** / **34.5** | 424 / 425 | 545 / 545 | – | 424 / 429 | – | – | 575 / 578 |
| P-521 base | **20.0** / **32.8** | – | 911 / 913 | – | – | – | – | 960 / 968 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.4 / 36.5 | – | – | 5.5\* / 6.9\* | – |
| BabyBear | **8.9** / **10.9** | 34.4 / 36.4 | – | – | 11.9 / 14.0 | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 38.1 | – | – | 3.2\* / 9.9 | 7.3 / 10.9 |
| BN254 scalar | **103** / **114** | 281 / 277 | – | 186 / 185 | – | – |
| secp256k1 base | **56.8** / **67.9** | 286 / 271 | – | 148 / 155 | – | – |
| P-256 base | **101** / **110** | 285 / 275 | – | 128 / 134 | – | – |
| P-256 scalar | **110** / **116** | 281 / 278 | 174 / 180 | 191 / 188 | – | – |
| Tom-256 base | **108** / **114** | 280 / 269 | – | 170 / 172 | – | – |
| BLS12-381 base | **215** / **222** | 541 / 545 | – | 372 / 378 | – | – |
| P-521 base | **361** / **392** | 912 / 919 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | – | 34.2 / 26.5 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **5.9** / **6.5** | – | 31.6 / 29.3 | – | – | 8.9 / 8.0 | – | – |
| Goldilocks | **4.7** / **4.7** | – | 37.8 / 35.0 | – | – | 1.9\* / 2.9\* | 6.1 / 5.9 | – |
| BN254 scalar | **24.9** / **24.9** | 127 / 127 | 231 / 227 | – | 142 / 150 | – | – | 115 / 113 |
| secp256k1 base | **5.9** / **7.3** | 109 / 103 | 231 / 224 | 2.7\* / 2.1\* | 129 / 137 | – | – | 114 / 113 |
| P-256 base | **25.9** / **22.0** | 107 / 103 | 231 / 226 | 76.4 / 85.1 | 118 / 128 | – | – | 58.8 / 62.7 |
| P-256 scalar | 25.4 / 22.7 | 134 / 133 | 231 / 224 | **14.3** / **20.8** | 152 / 162 | – | – | 114 / 113 |
| Tom-256 base | **26.4** / **26.0** | – | 231 / 225 | – | 141 / 149 | – | – | 114 / 113 |
| BLS12-381 base | **38.2** / **37.1** | 224 / 221 | 376 / 382 | – | 261 / 260 | – | – | 318 / 311 |
| P-521 base | **10.8** / **12.7** | – | 596 / 598 | – | – | – | – | 522 / 537 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 32.9 / 26.6 | – | – | 4.1\* / 4.0\* | – |
| BabyBear | **5.9** / **7.0** | 31.6 / 30.4 | – | – | 8.9 / 8.0 | – |
| Goldilocks | **4.8** / **4.5** | 38.9 / 35.1 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 |
| BN254 scalar | **87.8** / **84.0** | 233 / 227 | – | 101 / 108 | – | – |
| secp256k1 base | **6.7** / **8.2** | 233 / 233 | – | 84.3 / 94.5 | – | – |
| P-256 base | **61.7** / **62.4** | 231 / 231 | – | 75.7 / 81.5 | – | – |
| P-256 scalar | 88.6 / 89.8 | 233 / 229 | **48.4** / **44.0** | 108 / 116 | – | – |
| Tom-256 base | **87.3** / **83.5** | 234 / 227 | – | 92.8 / 107 | – | – |
| BLS12-381 base | **170** / **166** | 376 / 381 | – | 206 / 213 | – | – |
| P-521 base | **23.1** / **23.7** | 596 / 605 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | – | 16.5 / 18.0 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **6.0** / **6.9** | – | 17.6 / 18.3 | – | – | 8.9 / 8.9 | – | – |
| Goldilocks | **4.3** / **4.2** | – | 26.8 / 22.3 | – | – | 1.9\* / 2.9\* | 4.9 / 5.3 | – |
| BN254 scalar | **24.8** / **24.9** | 130 / 120 | 180 / 177 | – | 145 / 143 | – | – | 113 / 111 |
| secp256k1 base | **4.2** / **6.9** | 105 / 101 | 191 / 179 | 27.6 / 2.9\* | 123 / 125 | – | – | 112 / 111 |
| P-256 base | **20.0** / **21.3** | 106 / 96.0 | 179 / 177 | 81.1 / 78.1 | 119 / 119 | – | – | 61.3 / 61.0 |
| P-256 scalar | 24.9 / 23.6 | 132 / 127 | 184 / 178 | **5.5** / **12.4** | 149 / 150 | – | – | 113 / 111 |
| Tom-256 base | **24.4** / **25.0** | – | 182 / 181 | – | 142 / 145 | – | – | 113 / 111 |
| BLS12-381 base | **37.9** / **38.6** | 222 / 223 | 363 / 362 | – | 253 / 255 | – | – | 315 / 311 |
| P-521 base | **9.6** / **11.1** | – | 587 / 603 | – | – | – | – | 522 / 535 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 18.0 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **5.9** / **7.0** | 17.6 / 18.4 | – | – | 8.9 / 8.9 | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.3 | – | – | 1.9\* / 2.9\* | 4.9 / 5.3 |
| BN254 scalar | **84.2** / **83.9** | 181 / 186 | – | 111 / 105 | – | – |
| secp256k1 base | **5.2** / **6.8** | 185 / 180 | – | 80.8 / 82.8 | – | – |
| P-256 base | **62.0** / **59.7** | 182 / 182 | – | 78.9 / 78.0 | – | – |
| P-256 scalar | 81.8 / 87.0 | 188 / 177 | **43.4** / **36.6** | 106 / 101 | – | – |
| Tom-256 base | **83.9** / **78.0** | 172 / 185 | – | 103 / 96.0 | – | – |
| BLS12-381 base | **168** / **169** | 363 / 366 | – | 206 / 203 | – | – |
| P-521 base | **20.4** / **15.8** | 590 / 605 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / – | – | 12.3 / – | – | – | **3.2** / – | – | – |
| BabyBear | **6.3** / – | – | 13.1 / – | – | – | 8.0 / – | – | – |
| Goldilocks | 3.9 / – | – | 17.5 / – | – | – | 1.0\* / – | **3.4** / – | – |
| BN254 scalar | **22.0** / – | 129 / – | 119 / – | – | 127 / – | – | – | 94.7 / – |
| secp256k1 base | **2.1** / – | 120 / – | 118 / – | 2.4 / – | 110 / – | – | – | 94.8 / – |
| P-256 base | **17.8** / – | 112 / – | 117 / – | 59.0 / – | 104 / – | – | – | 56.0 / – |
| P-256 scalar | 18.0 / – | 136 / – | 118 / – | **2.0** / – | 133 / – | – | – | 94.9 / – |
| Tom-256 base | **21.0** / – | – | 118 / – | – | 130 / – | – | – | 94.9 / – |
| BLS12-381 base | **29.2** / – | 226 / – | 261 / – | – | 226 / – | – | – | 326 / – |
| P-521 base | **5.0** / – | – | 444 / – | – | – | – | – | 486 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **611** / **608** | 4,176 / 4,148 | 16,426 / 16,338 | 13,321 / 13,305 | – | 6,223 / 6,204 | 11,382 / 10,567 |
| secp256k1 base | **568** / **567** | 4,044 / 4,038 | 13,258 / 13,293 | 13,376 / 13,304 | 23,725 / 24,796 | 6,139 / 6,154 | 11,437 / 10,583 |
| P-256 base | **601** / **609** | 4,179 / 4,156 | 14,791 / 14,822 | 13,307 / 13,311 | 29,506 / 29,457 | 6,119 / 6,120 | 10,461 / 10,451 |
| P-256 scalar | **618** / **620** | 4,103 / 4,086 | 16,035 / 16,010 | 13,399 / 13,247 | 22,045 / 21,906 | 6,141 / 6,185 | 11,412 / 10,583 |
| Tom-256 base | **606** / **610** | 4,193 / 4,189 | – | 13,395 / 13,274 | – | 6,146 / 6,156 | 11,423 / 10,539 |
| BLS12-381 base | **752** / **752** | 7,062 / 7,076 | 34,781 / 34,651 | 23,817 / 22,986 | – | 9,796 / 9,808 | 26,111 / 25,613 |
| P-521 base | **926** / **927** | 12,024 / 12,001 | – | 39,964 / 39,735 | – | – | 55,220 / 55,050 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **216** / **216** | 219 / **216** | – | 1,454 / 1,441 | – | – | 632 / 634 | – | 873 / 887 |
| BabyBear | **219** / **216** | 231 / 232 | – | 1,461 / 1,430 | – | – | 623 / 626 | – | 1,061 / 1,067 |
| Goldilocks | **435** / **428** | 452 / 454 | – | 2,917 / 2,916 | – | – | 1,288 / 1,289 | 704 / 706 | 1,743 / 1,725 |
| BN254 scalar | **5,010** / **4,993** | 5,025 / 5,021 | 16,697 / 16,781 | 14,549 / 14,605 | – | 7,355 / 7,353 | – | – | 15,184 / 14,275 |
| secp256k1 base | 4,894 / 4,878 | **4,863** / **4,869** | 15,731 / 15,655 | 14,736 / 14,623 | 27,394 / 25,338 | 7,383 / 7,377 | – | – | 15,385 / 14,468 |
| P-256 base | 5,052 / **5,024** | **5,037** / 5,028 | 15,543 / 15,448 | 14,719 / 14,639 | 29,345 / 29,012 | 7,374 / 7,355 | – | – | 14,234 / 14,743 |
| P-256 scalar | **4,963** / **4,949** | 4,975 / 4,973 | 16,739 / 16,621 | 14,615 / 14,649 | 30,054 / 29,747 | 7,407 / 7,432 | – | – | 15,193 / 14,393 |
| Tom-256 base | **5,056** / **5,039** | 5,104 / 5,050 | – | 14,665 / 14,649 | – | 7,410 / 7,383 | – | – | 15,286 / 14,415 |
| BLS12-381 base | **9,073** / **9,013** | 9,140 / 9,135 | 36,855 / 36,900 | 26,415 / 25,790 | – | 12,407 / 12,358 | – | – | 35,555 / 34,554 |
| P-521 base | 14,964 / **14,943** | **14,934** / 14,967 | – | 44,587 / 44,554 | – | – | – | – | 72,528 / 72,080 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 9.6 / 17.2 | – | – | – | 2.4 / 5.0 |
| BabyBear | **1.9** / **2.9** | 9.6 / 17.0 | – | – | – | 2.6 / 4.9 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 19.0 | – | – | 2.4 / **2.9** | 2.5 / 4.9 |
| BN254 scalar | **4.9** / **6.7** | 66.5 / 85.7 | – | 12.7 / 22.0 | – | 6.8 / 10.0 |
| secp256k1 base | **5.8** / **6.7** | 66.7 / 93.1 | 4.5\* / 3.7\* | 14.3 / 21.1 | – | 6.9 / 10.4 |
| P-256 base | **5.8** / **6.9** | 66.5 / 84.5 | 10.6 / 18.5 | 10.7 / 18.3 | – | 6.8 / 9.6 |
| P-256 scalar | **6.1** / **6.7** | 66.9 / 79.0 | 17.4 / 21.1 | 14.3 / 21.0 | – | 7.0 / 10.4 |
| Tom-256 base | **5.8** / **6.6** | 66.4 / 91.7 | – | 11.6 / 20.5 | – | 6.9 / 9.8 |
| BLS12-381 base | **9.6** / **8.7** | 87.3 / 112 | – | 20.2 / 27.3 | – | 16.4 / 12.5 |
| P-521 base | **12.2** / **14.6** | 117 / 155 | – | – | – | 25.0 / 51.4 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.3 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 3.9 |
| BabyBear | **1.9** / **2.9** | 4.2 / 11.9 | – | – | – | – | **1.9** / 3.9 |
| Goldilocks | **1.9** / **2.9** | 4.3 / 11.7 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.0** / **6.7** | 50.0 / 50.8 | – | 8.3 / 11.7 | – | – | 6.8 / 7.8 |
| secp256k1 base | **5.2** / **7.8** | 49.7 / 47.1 | 4.0\* / 3.4\* | 7.5 / 12.8 | – | – | 6.8 / **7.8** |
| P-256 base | **5.3** / **6.7** | 49.6 / 43.3 | 7.8 / 12.8 | 7.5 / 11.7 | – | – | 6.7 / 7.8 |
| P-256 scalar | **5.0** / **6.8** | 50.0 / 47.3 | 7.6 / 12.8 | 7.9 / 13.0 | – | – | 6.8 / 7.7 |
| Tom-256 base | **5.4** / 8.1 | 49.6 / 53.3 | – | 8.3 / 11.7 | – | – | 6.8 / **7.8** |
| BLS12-381 base | **9.6** / **9.7** | 82.4 / 87.7 | – | 15.2 / 19.7 | – | – | 16.7 / 10.0 |
| P-521 base | **12.0** / **12.3** | 121 / 138 | – | – | – | – | 27.9 / 50.9 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **6.9** | 16.9 / 34.7 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.4** / **10.9** | 16.9 / 34.4 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.6 / 34.3 | – | – | **3.7** / 11.0 | 6.9 / 16.9 |
| BN254 scalar | **67.7** / **78.8** | 119 / 150 | – | 92.1 / 93.8 | – | 94.7 / 127 |
| secp256k1 base | **44.4** / **54.8** | 118 / 153 | 84.4 / 87.5 | 91.8 / 95.8 | – | 94.7 / 128 |
| P-256 base | **49.0** / **59.8** | 117 / 157 | 67.5 / 72.2 | 66.8 / 73.0 | – | 50.8 / 63.7 |
| P-256 scalar | **73.1** / **83.8** | 119 / 152 | 140 / 141 | 93.9 / 99.8 | – | 94.5 / 128 |
| Tom-256 base | **69.9** / **75.6** | 117 / 151 | – | 92.1 / 94.8 | – | 94.6 / 129 |
| BLS12-381 base | **139** / **154** | 270 / 290 | – | 192 / 194 | – | 249 / 274 |
| P-521 base | **176** / **193** | 529 / 548 | – | – | – | 464 / 476 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 33.9 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.6** / **11.0** | 16.7 / 33.9 | – | – | – | 7.5 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 35.0 | – | – | **3.7** / 11.0 | 7.5 / 17.0 |
| BN254 scalar | **64.9** / **66.9** | 120 / 151 | – | 75.7 / 89.4 | – | 93.5 / 129 |
| secp256k1 base | **33.0** / **42.9** | 111 / 151 | 57.9 / 78.5 | 74.5 / 84.9 | – | 93.7 / 130 |
| P-256 base | **39.1** / **43.9** | 121 / 153 | 60.9 / 71.9 | 54.9 / 57.9 | – | 57.2 / 57.9 |
| P-256 scalar | **63.9** / **72.9** | 116 / 153 | 129 / 147 | 76.9 / 93.5 | – | 94.0 / 129 |
| Tom-256 base | **56.9** / **64.4** | 115 / 151 | – | 69.5 / 82.9 | – | 93.9 / 128 |
| BLS12-381 base | **137** / **146** | 267 / 290 | – | 151 / 166 | – | 253 / 263 |
| P-521 base | **121** / **133** | 538 / 563 | – | – | – | 466 / 474 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **251** / – | 3,853 / – | – | – | – | 959 / – |
| BabyBear | **381** / – | 3,848 / – | – | – | – | 944 / – |
| Goldilocks | **647** / – | 3,838 / – | – | – | 691 / – | 2,067 / – |
| BN254 scalar | **6,759** / – | 20,032 / – | – | 46,924 / – | – | 55,699 / – |
| secp256k1 base | **6,864** / – | 19,895 / – | 24,819 / – | 46,639 / – | – | 69,236 / – |
| P-256 base | **6,510** / – | 19,343 / – | 28,402 / – | 33,510 / – | – | 34,282 / – |
| P-256 scalar | **6,903** / – | 19,343 / – | 78,590 / – | 48,903 / – | – | 60,426 / – |
| Tom-256 base | **7,006** / – | 19,331 / – | – | 44,812 / – | – | 52,235 / – |
| BLS12-381 base | **12,601** / – | 34,733 / – | – | 142,108 / – | – | 182,513 / – |
| P-521 base | **21,309** / – | 64,000 / – | – | – | – | 523,003 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,648 / – | – | – | 446 / – |
| BabyBear | **5,597** / – | 18,352 / – | – | – | – |
| Goldilocks | **7,148** / – | 26,591 / – | – | – | – |
| BN254 scalar | **58,595** / – | 134,017 / – | – | 85,097 / – | – |
| secp256k1 base | **12,287** / – | 54,411 / – | 26,445 / – | 47,952 / – | 69,560 / – |
| P-256 base | **11,993** / – | 43,129 / – | 28,638 / – | 33,607 / – | 29,405 / – |
| P-256 scalar | **25,618** / – | 51,869 / – | 81,228 / – | 41,859 / – | – |
| Tom-256 base | **20,056** / – | 48,728 / – | – | 44,141 / – | 52,426 / – |
| BLS12-381 base | **68,093** / – | 135,135 / – | – | 141,300 / – | 179,228 / – |
| P-521 base | **68,425** / – | 263,561 / – | – | – | 258,476 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.6 / 36.4 | – | – | – |
| BabyBear | **9.0** / **10.9** | 34.3 / 36.4 | – | – | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 38.0 | – | – | 7.3 / 11.0 |
| BN254 scalar | **21.9** / **29.6** | 258 / 266 | – | 236 / 233 | – |
| secp256k1 base | **11.0** / **19.9** | 282 / 269 | 82.7 / 90.8 | 207 / 218 | – |
| P-256 base | **24.5** / **31.8** | 272 / 271 | 147 / 137 | 184 / 177 | – |
| P-256 scalar | **22.9** / **33.0** | 274 / 265 | 139 / 146 | 248 / 235 | – |
| Tom-256 base | **22.4** / **33.0** | 273 / 268 | – | 225 / 227 | – |
| BLS12-381 base | **34.6** / **39.6** | 545 / 545 | – | 424 / 429 | – |
| P-521 base | **26.4** / **44.1** | 911 / 913 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.4 / 36.5 | – | – | – |
| BabyBear | **9.0** / **11.0** | 34.4 / 36.4 | – | – | – |
| Goldilocks | **7.0** / **9.1** | 38.0 / 38.1 | – | – | 7.3 / 10.9 |
| BN254 scalar | **119** / **141** | 281 / 277 | – | 186 / 185 | – |
| secp256k1 base | **78.3** / **83.8** | 286 / 271 | – | 148 / 155 | – |
| P-256 base | **119** / **134** | 285 / 275 | – | 128 / **134** | – |
| P-256 scalar | **132** / **146** | 281 / 278 | 174 / 180 | 191 / 188 | – |
| Tom-256 base | **122** / **135** | 280 / 269 | – | 170 / 172 | – |
| BLS12-381 base | **256** / **266** | 541 / 545 | – | 372 / 378 | – |
| P-521 base | **434** / **471** | 912 / 919 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 34.2 / 26.5 | – | – | – |
| BabyBear | **5.9** / **7.0** | 31.6 / 29.3 | – | – | – |
| Goldilocks | **4.7** / **4.8** | 37.8 / 35.0 | – | – | 6.1 / 5.9 |
| BN254 scalar | **27.8** / **28.9** | 231 / 227 | – | 142 / 150 | – |
| secp256k1 base | **5.9** / **7.4** | 231 / 224 | 2.7\* / 2.1\* | 129 / 137 | – |
| P-256 base | **25.7** / **24.0** | 231 / 226 | 76.4 / 85.1 | 118 / 128 | – |
| P-256 scalar | 27.1 / 27.6 | 231 / 224 | **14.3** / **20.8** | 152 / 162 | – |
| Tom-256 base | **27.4** / **27.1** | 231 / 225 | – | 141 / 149 | – |
| BLS12-381 base | **43.8** / **44.8** | 376 / 382 | – | 261 / 260 | – |
| P-521 base | **10.8** / **13.0** | 596 / 598 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.5** / **4.6** | 32.9 / 26.6 | – | – | – |
| BabyBear | **5.9** / **7.0** | 31.6 / 30.4 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 38.9 / 35.1 | – | – | 6.0 / 5.9 |
| BN254 scalar | **92.8** / **91.8** | 233 / 227 | – | 101 / 108 | – |
| secp256k1 base | **9.2** / **8.8** | 233 / 233 | – | 84.3 / 94.5 | – |
| P-256 base | **70.3** / **69.0** | 231 / 231 | – | 75.7 / 81.5 | – |
| P-256 scalar | 95.8 / 89.8 | 233 / 229 | **48.4** / **44.0** | 108 / 116 | – |
| Tom-256 base | **84.3** / **86.7** | 234 / 227 | – | 92.8 / 107 | – |
| BLS12-381 base | **176** / **179** | 376 / 381 | – | 206 / 213 | – |
| P-521 base | **22.3** / **22.7** | 596 / 605 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 18.0 | – | – | – |
| BabyBear | **6.0** / **7.0** | 17.6 / 18.3 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.3 | – | – | 4.9 / 5.3 |
| BN254 scalar | **27.9** / **29.8** | 180 / 177 | – | 145 / 143 | – |
| secp256k1 base | **4.2** / **6.9** | 191 / 179 | 27.6 / 2.9\* | 123 / 125 | – |
| P-256 base | **23.3** / **22.9** | 179 / 177 | 81.1 / 78.1 | 119 / 119 | – |
| P-256 scalar | 26.4 / 25.3 | 184 / 178 | **5.5** / **12.4** | 149 / 150 | – |
| Tom-256 base | **27.6** / **28.1** | 182 / 181 | – | 142 / 145 | – |
| BLS12-381 base | **42.7** / **44.4** | 363 / 362 | – | 253 / 255 | – |
| P-521 base | **9.7** / **11.0** | 587 / 603 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 18.0 | – | – | – |
| BabyBear | **6.0** / **6.9** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.3 | – | – | 4.9 / 5.3 |
| BN254 scalar | **91.9** / **93.1** | 181 / 186 | – | 111 / 105 | – |
| secp256k1 base | **7.3** / **7.8** | 185 / 180 | – | 80.8 / 82.8 | – |
| P-256 base | **64.8** / **68.0** | 182 / 182 | – | 78.9 / 78.0 | – |
| P-256 scalar | 91.0 / 93.0 | 188 / 177 | **43.4** / **36.6** | 106 / 101 | – |
| Tom-256 base | **91.4** / **86.3** | 172 / 185 | – | 103 / 96.0 | – |
| BLS12-381 base | **175** / **172** | 363 / 366 | – | 206 / 203 | – |
| P-521 base | **19.1** / **19.5** | 590 / 605 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / – | 12.3 / – | – | – | – |
| BabyBear | **10.3** / – | 13.1 / – | – | – | – |
| Goldilocks | 7.9 / – | 17.5 / – | – | – | **3.4** / – |
| BN254 scalar | **60.6** / – | 119 / – | – | 127 / – | – |
| secp256k1 base | **2.2** / – | 118 / – | 2.4 / – | 110 / – | – |
| P-256 base | **30.6** / – | 117 / – | 59.0 / – | 104 / – | – |
| P-256 scalar | 62.9 / – | 118 / – | **2.0** / – | 133 / – | – |
| Tom-256 base | **55.0** / – | 118 / – | – | 130 / – | – |
| BLS12-381 base | **148** / – | 261 / – | – | 226 / – | – |
| P-521 base | **6.0** / – | 444 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **6,843** / **6,850** | 6,898 / 6,896 | 19,631 / 19,626 | – | 46,056 / 46,057 | 55,140 / 54,847 |
| secp256k1 base | 6,837 / **6,773** | **6,828** / 6,830 | 19,612 / 19,626 | 23,725 / 24,796 | 46,792 / 46,805 | 69,439 / 68,823 |
| P-256 base | **6,692** / **6,687** | 6,790 / 6,704 | 19,578 / 19,483 | 29,506 / 29,457 | 33,189 / 33,201 | 34,657 / 34,587 |
| P-256 scalar | **6,908** / **6,894** | **6,908** / 6,930 | 19,638 / 19,746 | 78,915 / 78,870 | 48,750 / 49,143 | 60,399 / 59,855 |
| Tom-256 base | **7,300** / **7,290** | 7,345 / 7,334 | 19,663 / 19,700 | – | 44,230 / 43,787 | 51,756 / 51,533 |
| BLS12-381 base | **12,435** / **12,396** | 12,468 / 12,482 | 35,727 / 34,903 | – | 139,399 / 140,024 | 178,484 / 178,578 |
| P-521 base | **20,370** / **20,371** | 20,414 / 20,413 | 64,767 / 64,571 | – | – | 496,489 / 496,099 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **260** / **264** | 275 / 268 | 3,902 / 3,905 | – | – | – | 993 / 994 |
| BabyBear | **405** / **405** | 421 / 416 | 3,903 / 3,898 | – | – | – | 992 / 993 |
| Goldilocks | **669** / **655** | 672 / 664 | 3,890 / 3,894 | – | – | 704 / 706 | 2,115 / 2,114 |
| BN254 scalar | **6,894** / **6,898** | 6,963 / 6,955 | 19,564 / 19,543 | – | 46,001 / 46,012 | – | 55,090 / 54,797 |
| secp256k1 base | **6,776** / **6,769** | 6,837 / 6,816 | 19,605 / 19,679 | 27,394 / 25,338 | 46,715 / 46,726 | – | 69,512 / 69,101 |
| P-256 base | **6,715** / **6,748** | 6,768 / 6,779 | 19,577 / 19,511 | 29,345 / 29,012 | 33,172 / 33,170 | – | 34,592 / 34,313 |
| P-256 scalar | **6,954** / **6,916** | 7,003 / 7,004 | 19,715 / 19,634 | 78,938 / 78,881 | 49,087 / 49,077 | – | 60,233 / 59,919 |
| Tom-256 base | **7,337** / **7,328** | 7,398 / 7,399 | 19,679 / 19,655 | – | 44,087 / 43,516 | – | 51,874 / 51,994 |
| BLS12-381 base | **12,585** / **12,511** | 12,645 / 12,558 | 35,435 / 35,022 | – | 141,621 / 140,295 | – | 178,644 / 176,916 |
| P-521 base | **20,388** / **20,379** | 20,432 / 20,414 | 64,795 / 64,841 | – | – | – | 494,075 / 495,902 |

## WebAssembly

`prime-field` at commit `f8b7bef` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-04.

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
| Features | `prime-field` with its default features |
| RUSTFLAGS, in WebAssembly, Chrome | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Chrome | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| RUSTFLAGS, in WebAssembly, Firefox | `-C codegen-units=1` |
| RUSTFLAGS, in WebAssembly with wide arithmetic, Firefox | `-C codegen-units=1 -C target-feature=+wide-arithmetic` |
| Browser, in WebAssembly, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff` |
| Target features, in WebAssembly, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Chrome | 3,988 to 5,417 cycles and 3,081 instructions; its noise in a figure, 75.0 to 252 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,159 to 15,355 cycles and 3,081 instructions; its noise in a figure, 71.0 to 2,610 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,744 to 13,499 cycles and 1,885 instructions; its noise in a figure, 450 to 3,705 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 5,909 to 72,134 cycles and 1,885 instructions; its noise in a figure, 250 to 12,334 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 5.17 before, 7.49 after; its SMT sibling, CPU 28, 3% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 10.06 before, 39.55 after; its SMT sibling, CPU 28, 32% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 39.55 before, 28.24 after; its SMT sibling, CPU 28, 25% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 27.76 before, 38.9 after; its SMT sibling, CPU 28, 64% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | – | 11.0 / 21.3 | – | – | 4.4 / 7.5\* | – | 10.8 / 18.0 | – |
| BabyBear | **2.4** / **3.0** | – | 11.0 / 21.3 | – | – | **2.4** / **3.0** | – | 10.8 / 18.0 | – |
| Goldilocks | **3.3** / **3.0** | – | 10.9 / 20.1 | – | – | 3.5 / 6.0 | **3.3** / 4.0 | 10.7 / 18.0 | – |
| BN254 scalar | 27.3 / 35.5 | **13.5** / **13.3** | 53.0 / 83.1 | – | 36.1 / 41.9 | – | – | 62.1 / 104 | – |
| secp256k1 base | 29.7 / 39.0 | **15.2** / **15.5** | 53.0 / 82.9 | 22.4\* / 14.5\* | 35.0 / 39.8 | – | – | 62.2 / 104 | – |
| P-256 base | 26.3 / 31.5 | **15.7** / **15.1** | 53.1 / 82.9 | 30.5 / 39.6 | 30.6 / 38.9 | – | – | 62.0 / 104 | 433 / – |
| P-256 scalar | 29.7 / 39.0 | **15.3** / **15.3** | 53.0 / 82.9 | 41.1 / 57.9 | 35.4 / 42.1 | – | – | 62.2 / 103 | 439 / – |
| Tom-256 base | **27.6** / **36.2** | – | 53.1 / 83.0 | – | 36.2 / 42.9 | – | – | 62.1 / 104 | – |
| BLS12-381 base | 49.0 / 56.4 | **35.4** / **43.8** | 107 / 112 | – | 63.5 / 63.1 | – | – | 115 / 158 | – |
| P-521 base | **85.8** / **84.7** | – | 145 / 183 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.6 / 7.9 | – |
| BabyBear | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 2.8 / **3.0** | – | 8.5 / 8.2 | – |
| Goldilocks | **2.6** / **3.0** | – | 4.5 / 8.0 | – | – | 3.5 / 5.0 | 2.8 / **3.0** | 11.4 / 8.1 | – |
| BN254 scalar | 25.0 / 30.6 | **18.5** / **16.4** | 30.9 / 32.0 | – | 23.9 / 26.0 | – | – | 37.3 / 77.3 | – |
| secp256k1 base | 22.4 / 27.7 | **18.4** / **16.7** | 30.9 / 29.0 | 25.5 / 19.8\* | 22.3 / 25.2 | – | – | 37.6 / 76.9 | – |
| P-256 base | 21.0 / 26.7 | **17.0** / **15.5** | 30.9 / 27.2 | 20.8 / 26.5 | 20.5 / 25.1 | – | – | 37.4 / 77.0 | 242 / – |
| P-256 scalar | 24.8 / 27.6 | **18.0** / **16.5** | 30.9 / 31.0 | 23.2 / 25.0 | 23.6 / 26.8 | – | – | 37.8 / 77.3 | 241 / – |
| Tom-256 base | **23.3** / 27.0 | – | 30.9 / 32.0 | – | 23.8 / **26.0** | – | – | 37.4 / 77.2 | – |
| BLS12-381 base | 43.0 / 45.1 | **33.7** / **30.6** | 54.1 / 101 | – | 43.3 / 45.2 | – | – | 72.5 / 103 | – |
| P-521 base | **68.6** / **68.5** | – | 88.6 / 159 | – | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | – | 29.3 / 46.6 | – | – | 5.2 / 12.0 | – | 51.7 / 59.0 | – |
| BabyBear | 9.0 / 17.0 | – | 29.3 / 46.7 | – | – | **4.1** / **12.2** | – | 51.5 / 59.0 | – |
| Goldilocks | 14.1 / 22.7 | – | 32.4 / 47.1 | – | – | 10.0\* / 16.6\* | **12.3** / **18.0** | 51.6 / 59.0 | – |
| BN254 scalar | 336 / 350 | **319** / **316** | 432 / 443 | – | 366 / 378 | – | – | 899 / 934 | – |
| secp256k1 base | **242** / **257** | 279 / 276 | 430 / 436 | 157\* / 160\* | 303 / 306 | – | – | 911 / 943 | – |
| P-256 base | 244 / **199** | 261 / 258 | 435 / 436 | **242** / 242 | 243 / 243 | – | – | 554 / 597 | 1,074 / – |
| P-256 scalar | 336 / 377 | **317** / **312** | 427 / 437 | 523 / 510 | 343 / 350 | – | – | 909 / 939 | 1,067 / – |
| Tom-256 base | **299** / **337** | – | 430 / 440 | – | 345 / 340 | – | – | 912 / 942 | – |
| BLS12-381 base | 999 / 1,303 | **803** / **684** | 915 / 989 | – | 936 / 927 | – | – | 2,008 / 2,034 | – |
| P-521 base | **1,356** / **1,644** | – | 2,129 / 2,135 | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | – | 28.6 / 45.3 | – | – | 5.1\* / 12.0\* | – | 46.5 / 56.8 | – |
| BabyBear | 8.0 / 17.0 | – | 28.7 / 45.4 | – | – | **5.3** / **12.1** | – | 46.5 / 55.6 | – |
| Goldilocks | 13.2 / 21.3 | – | 32.6 / 46.8 | – | – | 8.7\* / 16.3\* | **12.4** / **20.0** | 46.6 / 56.1 | – |
| BN254 scalar | 317 / 302 | **270** / **265** | 323 / 350 | – | 310 / 314 | – | – | 898 / 924 | – |
| secp256k1 base | **200** / **176** | 231 / 226 | 324 / 348 | 98.5\* / 116\* | 215 / 222 | – | – | 913 / 930 | – |
| P-256 base | **167** / 189 | 214 / 210 | 320 / 347 | 240 / 243 | 181 / **177** | – | – | 555 / 581 | 1,070 / – |
| P-256 scalar | 300 / 303 | **264** / **259** | 324 / 349 | 503 / 515 | 282 / 266 | – | – | 916 / 937 | 1,064 / – |
| Tom-256 base | **314** / **251** | – | 324 / 349 | – | 317 / 312 | – | – | 914 / 939 | – |
| BLS12-381 base | 766 / 981 | **637** / **622** | 909 / 976 | – | 670 / 913 | – | – | 2,008 / 2,029 | – |
| P-521 base | **748** / **869** | – | 2,147 / 2,176 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **266** / – | – | 1,730 / – | – | – | 676 / – | – | 2,217 / – | – |
| BabyBear | **270** / – | – | 1,699 / – | – | – | 695 / – | – | 2,354 / – | – |
| Goldilocks | **537** / – | – | 3,442 / – | – | – | 1,380 / – | 1,771 / – | 4,434 / – | – |
| BN254 scalar | **9,704** / – | 23,936 / – | 18,366 / – | – | 14,235 / – | – | – | 41,552 / – | – |
| secp256k1 base | **9,484** / – | 22,197 / – | 18,578 / – | 33,273 / – | 14,297 / – | – | – | 42,428 / – | – |
| P-256 base | **9,307** / – | 21,899 / – | 18,449 / – | 66,001 / – | 14,320 / – | – | – | 41,275 / – | 249,004 / – |
| P-256 scalar | **9,576** / – | 24,525 / – | 18,481 / – | 46,933 / – | 14,314 / – | – | – | 42,491 / – | 250,098 / – |
| Tom-256 base | **9,649** / – | – | 18,541 / – | – | 14,345 / – | – | – | 42,528 / – | – |
| BLS12-381 base | **19,147** / – | 46,662 / – | 32,528 / – | – | 25,599 / – | – | – | 97,689 / – | – |
| P-521 base | **35,448** / – | – | 65,126 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 329 / – | – | 3,283 / – | – | – | **324** / – | 1,741 / – | – |
| BabyBear | 3,394 / – | – | 34,115 / – | – | – | **3,095** / – | – | – |
| Goldilocks | **7,590** / – | – | 48,791 / – | – | – | 5,614\* / – | – | – |
| BN254 scalar | 220,291 / – | **152,639** / – | 329,599 / – | – | 378,232 / – | – | – | – |
| secp256k1 base | **67,719** / – | 119,217 / – | 159,054 / – | 33,066\* / – | 157,595 / – | – | 476,423 / – | – |
| P-256 base | 70,768 / – | **59,537** / – | 134,736 / – | 64,788 / – | 130,260 / – | – | 170,450 / – | 310,168 / – |
| P-256 scalar | 145,096 / – | **119,817** / – | 156,723 / – | 239,155 / – | 152,858 / – | – | – | – |
| Tom-256 base | **120,135** / – | – | 149,245 / – | – | 183,037 / – | – | 337,613 / – | – |
| BLS12-381 base | 544,659 / – | **421,805** / – | 542,512 / – | – | 808,557 / – | – | 1,234,024 / – | – |
| P-521 base | **448,540** / – | – | 1,520,930 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | – | 60.3 / 59.7 | – | – | 9.1\* / 12.6\* | – | – |
| BabyBear | **27.0** / **26.3** | – | 64.0 / 65.5 | – | – | 35.1 / 34.1 | – | – |
| Goldilocks | 28.0 / 28.2 | – | 72.0 / 74.5 | – | – | 10.0\* / 16.6\* | **24.9** / **23.6** | – |
| BN254 scalar | **119** / **110** | 694 / 652 | 853 / 871 | – | 1,028 / 1,036 | – | – | 1,835 / 1,851 |
| secp256k1 base | **50.8** / **57.3** | 451 / 479 | 862 / 857 | 157 / 163 | 706 / 709 | – | – | 1,825 / 1,827 |
| P-256 base | **94.5** / **90.1** | 604 / 530 | 863 / 870 | 485 / 487 | 720 / 702 | – | – | 1,205 / 1,201 |
| P-256 scalar | **109** / **109** | 634 / 618 | 857 / 865 | 519 / 504 | 1,012 / 991 | – | – | 1,846 / 1,859 |
| Tom-256 base | **104** / **104** | – | 862 / 856 | – | 920 / 933 | – | – | 1,860 / 1,853 |
| BLS12-381 base | **179** / **169** | 1,635 / 1,633 | 1,943 / 1,943 | – | 2,405 / 2,181 | – | – | 4,099 / 4,084 |
| P-521 base | **125** / **119** | – | 3,015 / 3,055 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 60.7 / 59.7 | – | – | 9.1\* / 12.6\* | – |
| BabyBear | 27.0 / 26.3 | 63.9 / 65.4 | – | – | **23.1** / **23.6** | – |
| Goldilocks | 28.0 / 28.2 | 71.6 / 73.9 | – | – | 10.0\* / 16.6\* | **24.9** / **23.6** |
| BN254 scalar | **622** / **610** | 850 / 864 | – | 762 / 764 | – | – |
| secp256k1 base | **356** / **360** | 859 / 860 | – | 519 / 524 | – | – |
| P-256 base | 522 / 527 | 874 / 853 | – | **450** / **453** | – | – |
| P-256 scalar | 632 / 631 | 861 / 863 | **553** / **554** | 730 / 735 | – | – |
| Tom-256 base | **591** / **583** | 866 / 870 | – | 700 / 699 | – | – |
| BLS12-381 base | **1,331** / **1,326** | 1,951 / 1,935 | – | 1,927 / 1,948 | – | – |
| P-521 base | **1,828** / **1,808** | 3,016 / 3,050 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | – | 36.0 / 35.4 | – | – | 7.6\* / 8.1\* | – | – |
| BabyBear | **17.8** / **18.3** | – | 39.9 / 47.5 | – | – | 32.1 / 30.7 | – | – |
| Goldilocks | 14.2 / 13.5 | – | 51.0 / 58.4 | – | – | 3.5\* / 6.0\* | **13.3** / **13.1** | – |
| BN254 scalar | **139** / **133** | 347 / 339 | 488 / 515 | – | 694 / 679 | – | – | 1,051 / 1,048 |
| secp256k1 base | **25.4** / **34.0** | 163 / 158 | 505 / 508 | 19.6\* / 5.3\* | 434 / 407 | – | – | 1,014 / 1,027 |
| P-256 base | **93.3** / **95.1** | 267 / 260 | 507 / 504 | 277 / 278 | 480 / 481 | – | – | 699 / 680 |
| P-256 scalar | 115 / 114 | 342 / 334 | 510 / 504 | **40.6** / **57.3** | 685 / 649 | – | – | 1,048 / 1,053 |
| Tom-256 base | **123** / **119** | – | 504 / 504 | – | 632 / 607 | – | – | 1,053 / 1,052 |
| BLS12-381 base | **202** / **214** | 847 / 839 | 1,070 / 1,090 | – | 1,352 / 1,385 | – | – | 2,216 / 2,215 |
| P-521 base | **85.6** / **85.9** | – | 1,683 / 1,679 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 36.1 / 35.4 | – | – | 7.6\* / 8.1\* | – |
| BabyBear | **17.8** / **18.3** | 39.8 / 47.6 | – | – | 22.0 / 21.5 | – |
| Goldilocks | 14.2 / 13.7 | 49.3 / 58.3 | – | – | 3.5\* / 6.0\* | **13.3** / **13.1** |
| BN254 scalar | 457 / 533 | 506 / 509 | – | **412** / **412** | – | – |
| secp256k1 base | **28.2** / **36.8** | 508 / 508 | – | 224 / 225 | – | – |
| P-256 base | 267 / 264 | 505 / 509 | – | **248** / **245** | – | – |
| P-256 scalar | 426 / 523 | 503 / 513 | **94.8** / **81.7** | 380 / 389 | – | – |
| Tom-256 base | 432 / 448 | 510 / 506 | – | **379** / **376** | – | – |
| BLS12-381 base | 1,249 / 1,585 | 1,083 / 1,083 | – | **989** / **969** | – | – |
| P-521 base | **101** / **103** | 1,691 / 1,685 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | – | 23.5 / 22.1 | – | – | 6.1\* / 6.1 | – | – |
| BabyBear | **18.5** / **17.8** | – | 27.9 / 28.6 | – | – | 32.2 / 31.8 | – | – |
| Goldilocks | 13.6 / 13.2 | – | 36.7 / 36.4 | – | – | 3.5\* / 5.0\* | **11.5** / **11.1** | – |
| BN254 scalar | **129** / **138** | 353 / 347 | 503 / 491 | – | 674 / 652 | – | – | 1,001 / 992 |
| secp256k1 base | **18.8** / **23.6** | 161 / 157 | 498 / 485 | 14.0\* / 11.0\* | 412 / 409 | – | – | 974 / 971 |
| P-256 base | **91.6** / **90.4** | 273 / 264 | 499 / 480 | 264 / 265 | 482 / 473 | – | – | 644 / 645 |
| P-256 scalar | 109 / 107 | 344 / 335 | 494 / 495 | **21.2** / **21.1** | 676 / 634 | – | – | 1,004 / 1,000 |
| Tom-256 base | **122** / **121** | – | 501 / 487 | – | 625 / 598 | – | – | 1,001 / 998 |
| BLS12-381 base | **220** / **223** | 819 / 821 | 1,083 / 1,086 | – | 1,301 / 1,318 | – | – | 2,159 / 2,159 |
| P-521 base | **59.9** / **62.6** | – | 1,667 / 1,664 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 23.4 / 22.1 | – | – | 6.1\* / 6.1 | – |
| BabyBear | **18.4** / **17.8** | 27.9 / 28.4 | – | – | 23.0 / 22.0 | – |
| Goldilocks | 13.6 / 13.2 | 36.8 / 36.3 | – | – | 3.5\* / 5.0\* | **11.5** / **11.1** |
| BN254 scalar | 456 / 536 | 495 / 500 | – | **401** / **400** | – | – |
| secp256k1 base | **22.3** / **27.8** | 502 / 489 | – | 215 / 212 | – | – |
| P-256 base | 252 / 259 | 509 / 485 | – | **233** / **235** | – | – |
| P-256 scalar | 377 / 501 | 498 / 488 | **69.2** / **71.7** | 372 / 373 | – | – |
| Tom-256 base | 381 / 451 | 499 / 491 | – | **364** / **362** | – | – |
| BLS12-381 base | 1,279 / 1,576 | 1,074 / 1,076 | – | **976** / **938** | – | – |
| P-521 base | **81.8** / **86.7** | 1,654 / 1,687 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 18.0 / – | – | – | **4.1** / – | – | – |
| BabyBear | **17.0** / – | – | 21.6 / – | – | – | 30.7 / – | – | – |
| Goldilocks | 12.1 / – | – | 29.3 / – | – | – | 2.1\* / – | **10.6** / – | – |
| BN254 scalar | **124** / – | 317 / – | 435 / – | – | 613 / – | – | – | 924 / – |
| secp256k1 base | **3.8** / – | 150 / – | 433 / – | 6.1 / – | 354 / – | – | – | 920 / – |
| P-256 base | **72.1** / – | 240 / – | 433 / – | 236 / – | 443 / – | – | – | 554 / – |
| P-256 scalar | 88.0 / – | 310 / – | 428 / – | **4.0** / – | 600 / – | – | – | 930 / – |
| Tom-256 base | **100** / – | – | 428 / – | – | 564 / – | – | – | 932 / – |
| BLS12-381 base | **165** / – | 775 / – | 901 / – | – | 1,259 / – | – | – | 2,055 / – |
| P-521 base | **6.0** / – | – | 1,427 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **1,447** / **1,447** | 10,930 / 9,632 | 25,166 / 25,122 | 18,006 / 18,012 | – | 13,518 / 13,436 | 33,469 / 33,417 |
| secp256k1 base | **1,147** / **1,149** | 8,876 / 8,975 | 18,861 / 18,759 | 17,803 / 17,841 | 33,699 / 33,473 | 13,112 / 13,093 | 33,940 / 34,102 |
| P-256 base | **1,321** / **1,319** | 8,844 / 9,023 | 22,424 / 22,604 | 18,035 / 17,891 | 67,196 / 66,647 | 13,131 / 13,133 | 33,042 / 32,569 |
| P-256 scalar | **1,423** / **1,426** | 10,924 / 9,547 | 24,102 / 24,216 | 18,076 / 17,949 | 35,892 / 35,615 | 13,343 / 13,395 | 34,334 / 33,805 |
| Tom-256 base | **1,385** / **1,383** | 10,684 / 9,426 | – | 17,866 / 17,894 | – | 13,326 / 13,320 | 34,212 / 33,312 |
| BLS12-381 base | **2,240** / **2,245** | 19,412 / 18,552 | 48,184 / 48,148 | 31,221 / 31,123 | – | 24,077 / 23,998 | 73,490 / 73,742 |
| P-521 base | **2,852** / **2,761** | 35,079 / 32,361 | – | 62,634 / 62,606 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 276 / 277 | **273** / **273** | – | 1,825 / 1,836 | – | – | 670 / 672 | – | 2,390 / 2,392 |
| BabyBear | **284** / **286** | 307 / 306 | – | 1,811 / 1,824 | – | – | 682 / 676 | – | 2,541 / 2,519 |
| Goldilocks | **561** / **558** | 596 / 589 | – | 3,577 / 3,597 | – | – | 1,374 / 1,375 | 1,814 / 1,804 | 4,745 / 4,736 |
| BN254 scalar | 12,007 / 10,832 | **10,791** / **10,703** | 25,075 / 25,029 | 19,431 / 19,413 | – | 15,032 / 15,052 | – | – | 43,879 / 43,855 |
| secp256k1 base | 9,922 / **9,918** | **9,891** / 9,983 | 22,600 / 22,585 | 19,320 / 19,303 | 33,114 / 33,810 | 14,866 / 14,877 | – | – | 44,310 / 43,998 |
| P-256 base | **9,999** / 10,228 | 10,155 / **10,127** | 22,674 / 22,810 | 19,480 / 19,438 | 67,368 / 66,712 | 14,925 / 14,886 | – | – | 42,655 / 42,857 |
| P-256 scalar | 12,055 / 10,877 | **10,758** / **10,608** | 24,878 / 24,961 | 19,560 / 19,529 | 47,715 / 47,221 | 15,056 / 15,065 | – | – | 44,382 / 44,279 |
| Tom-256 base | 11,953 / 10,722 | **10,589** / **10,479** | – | 19,483 / 19,508 | – | 15,044 / 15,026 | – | – | 44,206 / 44,325 |
| BLS12-381 base | **21,764** / 21,891 | 23,740 / **21,373** | 48,453 / 48,916 | 34,565 / 34,562 | – | 27,622 / 27,552 | – | – | 101,454 / 101,521 |
| P-521 base | 38,885 / 37,200 | **38,831** / **37,185** | – | 69,041 / 68,242 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | 11.0 / 21.3 | – | – | – | 10.8 / 18.0 |
| BabyBear | **2.4** / **3.0** | 11.0 / 21.3 | – | – | – | 10.8 / 18.0 |
| Goldilocks | **3.3** / **3.0** | 10.9 / 20.1 | – | – | **3.3** / 4.0 | 10.7 / 18.0 |
| BN254 scalar | **27.4** / **35.6** | 53.0 / 83.1 | – | 36.1 / 41.9 | – | 62.1 / 104 |
| secp256k1 base | **29.8** / **39.0** | 53.0 / 82.9 | 22.4\* / 14.5\* | 35.0 / 39.8 | – | 62.2 / 104 |
| P-256 base | **26.3** / **31.7** | 53.1 / 82.9 | 30.5 / 39.6 | 30.6 / 38.9 | – | 62.0 / 104 |
| P-256 scalar | **29.8** / **39.0** | 53.0 / 82.9 | 41.1 / 57.9 | 35.4 / 42.1 | – | 62.2 / 103 |
| Tom-256 base | **27.7** / **36.2** | 53.1 / 83.0 | – | 36.2 / 42.9 | – | 62.1 / 104 |
| BLS12-381 base | **49.1** / **56.4** | 107 / 112 | – | 63.5 / 63.1 | – | 115 / 158 |
| P-521 base | **85.1** / **84.7** | 145 / 183 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.6 / 7.9 |
| BabyBear | **2.5** / **3.0** | 4.5 / 8.0 | – | – | – | – | 8.5 / 8.2 |
| Goldilocks | **2.6** / **3.0** | 4.5 / 8.0 | – | – | – | 2.8 / **3.0** | 11.4 / 8.1 |
| BN254 scalar | 25.0 / 30.7 | 30.9 / 32.0 | – | **23.9** / **26.0** | – | – | 37.3 / 77.3 |
| secp256k1 base | 22.5 / 27.7 | 30.9 / 29.0 | 25.5 / 19.8\* | **22.3** / **25.2** | – | – | 37.6 / 76.9 |
| P-256 base | 21.0 / 26.6 | 30.9 / 27.2 | 20.8 / 26.5 | **20.5** / **25.1** | – | – | 37.4 / 77.0 |
| P-256 scalar | 24.8 / 27.6 | 30.9 / 31.0 | **23.2** / **25.0** | 23.6 / 26.8 | – | – | 37.8 / 77.3 |
| Tom-256 base | **23.2** / 27.0 | 30.9 / 32.0 | – | 23.8 / **26.0** | – | – | 37.4 / 77.2 |
| BLS12-381 base | **43.0** / **45.1** | 54.1 / 101 | – | 43.3 / 45.2 | – | – | 72.5 / 103 |
| P-521 base | **68.6** / **68.5** | 88.6 / 159 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | 29.3 / 46.6 | – | – | – | 51.7 / 59.0 |
| BabyBear | **9.0** / **17.0** | 29.3 / 46.7 | – | – | – | 51.5 / 59.0 |
| Goldilocks | 13.7 / 22.7 | 32.4 / 47.1 | – | – | **12.3** / **18.0** | 51.6 / 59.0 |
| BN254 scalar | **347** / **377** | 432 / 443 | – | 366 / 378 | – | 899 / 934 |
| secp256k1 base | **236** / **258** | 430 / 436 | 157\* / 160\* | 303 / 306 | – | 911 / 943 |
| P-256 base | 245 / **199** | 435 / 436 | **242** / 242 | 243 / 243 | – | 554 / 597 |
| P-256 scalar | **335** / 377 | 427 / 437 | 523 / 510 | 343 / **350** | – | 909 / 939 |
| Tom-256 base | **300** / **334** | 430 / 440 | – | 345 / 340 | – | 912 / 942 |
| BLS12-381 base | 1,049 / 1,311 | **915** / 989 | – | 936 / **927** | – | 2,008 / 2,034 |
| P-521 base | **1,354** / **1,635** | 2,129 / 2,135 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square, constant time: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | 28.6 / 45.3 | – | – | – | 46.5 / 56.8 |
| BabyBear | **8.0** / **17.0** | 28.7 / 45.4 | – | – | – | 46.5 / 55.6 |
| Goldilocks | 13.3 / 21.1 | 32.6 / 46.8 | – | – | **12.4** / **20.0** | 46.6 / 56.1 |
| BN254 scalar | **300** / **299** | 323 / 350 | – | 310 / 314 | – | 898 / 924 |
| secp256k1 base | **180** / **176** | 324 / 348 | 98.5\* / 116\* | 215 / 222 | – | 913 / 930 |
| P-256 base | **165** / 189 | 320 / 347 | 240 / 243 | 181 / **177** | – | 555 / 581 |
| P-256 scalar | 300 / 285 | 324 / 349 | 503 / 515 | **282** / **266** | – | 916 / 937 |
| Tom-256 base | **311** / **249** | 324 / 349 | – | 317 / 312 | – | 914 / 939 |
| BLS12-381 base | 846 / 1,082 | 909 / 976 | – | **670** / **913** | – | 2,008 / 2,029 |
| P-521 base | **750** / **865** | 2,147 / 2,176 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert, constant time: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **438** / – | 5,739 / – | – | – | – | 3,342 / – |
| BabyBear | **678** / – | 5,729 / – | – | – | – | 3,299 / – |
| Goldilocks | **1,624** / – | 5,741 / – | – | – | 1,771 / – | 7,048 / – |
| BN254 scalar | **13,031** / – | 28,613 / – | – | 233,345 / – | – | 361,463 / – |
| secp256k1 base | **12,629** / – | 28,705 / – | 33,273 / – | 199,571 / – | – | 478,668 / – |
| P-256 base | **12,572** / – | 29,388 / – | 66,001 / – | 167,188 / – | – | 224,611 / – |
| P-256 scalar | **12,964** / – | 28,768 / – | 219,056 / – | 229,840 / – | – | 406,111 / – |
| Tom-256 base | **12,784** / – | 28,800 / – | – | 217,155 / – | – | 339,831 / – |
| BLS12-381 base | **25,414** / – | 49,278 / – | – | 870,432 / – | – | 1,240,060 / – |
| P-521 base | **45,094** / – | 103,430 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **334** / – | 3,283 / – | – | – | 1,741 / – |
| BabyBear | **8,379** / – | 34,115 / – | – | – | – |
| Goldilocks | **17,142** / – | 48,791 / – | – | – | – |
| BN254 scalar | 348,169 / – | **329,599** / – | – | 378,232 / – | – |
| secp256k1 base | **68,226** / – | 159,054 / – | 33,066\* / – | 157,595 / – | 476,423 / – |
| P-256 base | 71,607 / – | 134,736 / – | **64,788** / – | 130,260 / – | 170,450 / – |
| P-256 scalar | **150,643** / – | 156,723 / – | 239,155 / – | 152,858 / – | – |
| Tom-256 base | **119,231** / – | 149,245 / – | – | 183,037 / – | 337,613 / – |
| BLS12-381 base | 563,100 / – | **542,512** / – | – | 808,557 / – | 1,234,024 / – |
| P-521 base | **448,211** / – | 1,520,930 / – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 60.3 / 59.7 | – | – | – |
| BabyBear | **27.0** / **26.3** | 64.0 / 65.5 | – | – | – |
| Goldilocks | 28.0 / 28.2 | 72.0 / 74.5 | – | – | **24.9** / **23.6** |
| BN254 scalar | **146** / **150** | 853 / 871 | – | 1,028 / 1,036 | – |
| secp256k1 base | **73.9** / **85.3** | 862 / 857 | 157 / 163 | 706 / 709 | – |
| P-256 base | **184** / **120** | 863 / 870 | 485 / 487 | 720 / 702 | – |
| P-256 scalar | **134** / **130** | 857 / 865 | 519 / 504 | 1,012 / 991 | – |
| Tom-256 base | **120** / **128** | 862 / 856 | – | 920 / 933 | – |
| BLS12-381 base | **203** / **219** | 1,943 / 1,943 | – | 2,405 / 2,181 | – |
| P-521 base | **206** / **208** | 3,015 / 3,055 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 60.7 / 59.7 | – | – | – |
| BabyBear | **27.0** / **26.3** | 63.9 / 65.4 | – | – | – |
| Goldilocks | 28.0 / 28.2 | 71.6 / 73.9 | – | – | **24.9** / **23.6** |
| BN254 scalar | **696** / **706** | 850 / 864 | – | 762 / 764 | – |
| secp256k1 base | **422** / **439** | 859 / 860 | – | 519 / 524 | – |
| P-256 base | 585 / 593 | 874 / 853 | – | **450** / **453** | – |
| P-256 scalar | 689 / 690 | 861 / 863 | **553** / **554** | 730 / 735 | – |
| Tom-256 base | **648** / **640** | 866 / 870 | – | 700 / 699 | – |
| BLS12-381 base | **1,514** / **1,519** | 1,951 / 1,935 | – | 1,927 / 1,948 | – |
| P-521 base | **2,631** / **2,658** | 3,016 / 3,050 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 36.0 / 35.4 | – | – | – |
| BabyBear | **17.7** / **18.3** | 39.9 / 47.5 | – | – | – |
| Goldilocks | 14.2 / 13.5 | 51.0 / 58.4 | – | – | **13.3** / **13.1** |
| BN254 scalar | **182** / **171** | 488 / 515 | – | 694 / 679 | – |
| secp256k1 base | **25.4** / **34.0** | 505 / 508 | 19.6\* / 5.3\* | 434 / 407 | – |
| P-256 base | **117** / **131** | 507 / 504 | 277 / 278 | 480 / 481 | – |
| P-256 scalar | 137 / 165 | 510 / 504 | **40.6** / **57.3** | 685 / 649 | – |
| Tom-256 base | **153** / **151** | 504 / 504 | – | 632 / 607 | – |
| BLS12-381 base | **268** / **264** | 1,070 / 1,090 | – | 1,352 / 1,385 | – |
| P-521 base | **85.7** / **85.6** | 1,683 / 1,679 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 36.1 / 35.4 | – | – | – |
| BabyBear | **17.9** / **18.3** | 39.8 / 47.6 | – | – | – |
| Goldilocks | 14.2 / 13.6 | 49.3 / 58.3 | – | – | **13.3** / **13.1** |
| BN254 scalar | 504 / 584 | 506 / 509 | – | **412** / **412** | – |
| secp256k1 base | **52.0** / **52.0** | 508 / 508 | – | 224 / 225 | – |
| P-256 base | 301 / 297 | 505 / 509 | – | **248** / **245** | – |
| P-256 scalar | 444 / 528 | 503 / 513 | **94.8** / **81.7** | 380 / 389 | – |
| Tom-256 base | 454 / 477 | 510 / 506 | – | **379** / **376** | – |
| BLS12-381 base | 1,317 / 1,643 | 1,083 / 1,083 | – | **989** / **969** | – |
| P-521 base | **160** / **163** | 1,691 / 1,685 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 23.5 / 22.1 | – | – | – |
| BabyBear | **18.4** / **17.9** | 27.9 / 28.6 | – | – | – |
| Goldilocks | 13.6 / 13.2 | 36.7 / 36.4 | – | – | **11.5** / **11.1** |
| BN254 scalar | **151** / **162** | 503 / 491 | – | 674 / 652 | – |
| secp256k1 base | **18.8** / **23.3** | 498 / 485 | 14.0\* / 11.0\* | 412 / 409 | – |
| P-256 base | **123** / **112** | 499 / 480 | 264 / 265 | 482 / 473 | – |
| P-256 scalar | 136 / 137 | 494 / 495 | **21.2** / **21.1** | 676 / 634 | – |
| Tom-256 base | **157** / **148** | 501 / 487 | – | 625 / 598 | – |
| BLS12-381 base | **241** / **243** | 1,083 / 1,086 | – | 1,301 / 1,318 | – |
| P-521 base | **60.3** / **62.7** | 1,667 / 1,664 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.4** | 23.4 / 22.1 | – | – | – |
| BabyBear | **18.4** / **17.9** | 27.9 / 28.4 | – | – | – |
| Goldilocks | 13.6 / 13.2 | 36.8 / 36.3 | – | – | **11.5** / **11.1** |
| BN254 scalar | 502 / 581 | 495 / 500 | – | **401** / **400** | – |
| secp256k1 base | **46.9** / **43.4** | 502 / 489 | – | 215 / 212 | – |
| P-256 base | 238 / 281 | 509 / 485 | – | **233** / **235** | – |
| P-256 scalar | 409 / 509 | 498 / 488 | **69.2** / **71.7** | 372 / 373 | – |
| Tom-256 base | 379 / 468 | 499 / 491 | – | **364** / **362** | – |
| BLS12-381 base | 1,311 / 1,572 | 1,074 / 1,076 | – | **976** / **938** | – |
| P-521 base | **144** / **147** | 1,654 / 1,687 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / – | 18.0 / – | – | – | – |
| BabyBear | **16.3** / – | 21.6 / – | – | – | – |
| Goldilocks | 22.9 / – | 29.3 / – | – | – | **10.6** / – |
| BN254 scalar | **257** / – | 435 / – | – | 613 / – | – |
| secp256k1 base | **3.4** / – | 433 / – | 6.1 / – | 354 / – | – |
| P-256 base | **125** / – | 433 / – | 236 / – | 443 / – | – |
| P-256 scalar | 232 / – | 428 / – | **4.0** / – | 600 / – | – |
| Tom-256 base | **218** / – | 428 / – | – | 564 / – | – |
| BLS12-381 base | **529** / – | 901 / – | – | 1,259 / – | – |
| P-521 base | **6.0** / – | 1,427 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | 15,491 / **13,716** | **15,284** / 13,835 | 28,515 / 29,485 | – | 232,380 / 232,899 | 352,454 / 347,968 |
| secp256k1 base | **12,934** / **13,054** | 12,967 / 13,083 | 28,459 / 29,611 | 33,699 / 33,473 | 200,430 / 202,638 | 466,863 / 464,461 |
| P-256 base | 12,988 / **13,100** | **12,901** / 13,110 | 28,694 / 29,426 | 67,196 / 66,647 | 165,083 / 166,615 | 225,472 / 226,016 |
| P-256 scalar | 15,443 / **13,626** | **15,160** / 13,652 | 28,440 / 29,465 | 219,098 / 224,317 | 235,487 / 235,349 | 395,766 / 394,207 |
| Tom-256 base | **13,341** / **13,583** | 15,067 / 13,605 | 28,604 / 29,518 | – | 215,721 / 216,165 | 331,495 / 329,120 |
| BLS12-381 base | 27,958 / 26,808 | **27,944** / **26,739** | 51,600 / 50,966 | – | 848,123 / 847,349 | 1,243,258 / 1,245,986 |
| P-521 base | 48,750 / 46,853 | **48,522** / **46,730** | 107,490 / 106,095 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **455** / **451** | 457 / 454 | 5,968 / 5,494 | – | – | – | 3,506 / 3,526 |
| BabyBear | **717** / **741** | 721 / **741** | 6,010 / 5,710 | – | – | – | 3,558 / 3,532 |
| Goldilocks | **1,763** / **1,746** | 1,842 / 1,939 | 5,976 / 5,711 | – | – | 1,814 / 1,804 | 7,319 / 7,372 |
| BN254 scalar | 15,826 / **14,033** | **15,517** / 14,254 | 28,598 / 29,647 | – | 232,061 / 231,497 | – | 352,660 / 352,067 |
| secp256k1 base | **12,930** / **13,057** | 12,960 / 13,075 | 28,466 / 29,909 | 33,114 / 33,810 | 200,649 / 200,938 | – | 471,397 / 466,239 |
| P-256 base | **12,974** / **13,283** | 15,062 / 13,324 | 28,571 / 30,279 | 67,368 / 66,712 | 166,040 / 164,631 | – | 226,004 / 225,272 |
| P-256 scalar | 15,863 / **14,019** | **15,552** / 14,081 | 28,451 / 29,925 | 219,474 / 222,140 | 235,429 / 236,044 | – | 396,631 / 396,202 |
| Tom-256 base | 15,417 / 13,950 | **15,169** / **13,909** | 28,214 / 29,541 | – | 215,703 / 215,318 | – | 331,071 / 330,432 |
| BLS12-381 base | **27,407** / **27,794** | 27,863 / 28,013 | 51,080 / 51,273 | – | 846,688 / 846,162 | – | 1,240,892 / 1,242,193 |
| P-521 base | 49,534 / **46,579** | **47,255** / 46,959 | 106,104 / 106,071 | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | – | 11.7 / 19.3 | – | – | 4.4 / 7.5\* | – | 10.8 / 18.0 | – |
| BabyBear | **2.4** / **3.0** | – | 11.7 / 18.7 | – | – | **2.4** / **3.0** | – | 10.8 / 18.0 | – |
| Goldilocks | 3.3 / **3.0** | – | 11.4 / 19.1 | – | – | 4.5 / 8.0 | **3.2** / 4.0 | 10.8 / 18.0 | – |
| BN254 scalar | 16.4 / 19.6 | **13.2** / **11.4** | 42.5 / 87.3 | – | 20.7 / 23.1 | – | – | 62.2 / 104 | – |
| secp256k1 base | 18.2 / 22.6 | **12.8** / **11.2** | 42.7 / 87.2 | 22.6 / 14.5\* | 19.3 / 23.1 | – | – | 62.3 / 104 | – |
| P-256 base | 18.1 / 22.8 | **12.3** / **11.2** | 42.7 / 87.4 | 17.9 / 21.7 | 18.0 / 21.0 | – | – | 62.2 / 104 | 443 / – |
| P-256 scalar | 17.9 / 23.8 | **13.3** / **11.3** | 42.5 / 87.0 | 42.3 / 58.1 | 20.6 / 24.8 | – | – | 62.1 / 103 | 435 / – |
| Tom-256 base | **18.1** / **23.9** | – | 42.4 / 87.3 | – | 20.2 / 24.0 | – | – | 62.0 / 103 | – |
| BLS12-381 base | **25.6** / **28.2** | 30.2 / 38.1 | 107 / 115 | – | 32.4 / 34.1 | – | – | 115 / 158 | – |
| P-521 base | **46.6** / **49.2** | – | 134 / 179 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.5 / 8.2 | – |
| BabyBear | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 2.8 / **3.0** | – | 8.7 / 7.9 | – |
| Goldilocks | **2.6** / **3.0** | – | 4.5 / 8.0 | – | – | 3.5 / 5.0 | 2.8 / **3.0** | 10.1 / 8.0 | – |
| BN254 scalar | 16.6 / 16.9 | 16.9 / **13.9** | 26.7 / 27.0 | – | **15.0** / 15.0 | – | – | 37.7 / 77.0 | – |
| secp256k1 base | 14.4 / 15.5 | 16.6 / **14.0** | 26.7 / 27.2 | 25.6 / 19.7 | **14.2** / 14.5 | – | – | 37.0 / 77.4 | – |
| P-256 base | 13.4 / 15.6 | 16.0 / **13.0** | 26.9 / 25.8 | 13.5 / 14.7 | **12.5** / 14.0 | – | – | 37.8 / 77.3 | 237 / – |
| P-256 scalar | 14.7 / 15.9 | 16.9 / **14.0** | 26.7 / 29.0 | 23.3 / 25.1 | **14.4** / 15.8 | – | – | 37.3 / 77.1 | 232 / – |
| Tom-256 base | 15.7 / 17.0 | – | 26.7 / 27.0 | – | **14.9** / **14.9** | – | – | 37.7 / 77.3 | – |
| BLS12-381 base | 24.5 / 24.8 | 27.1 / **24.1** | 43.8 / 97.2 | – | **23.5** / 24.9 | – | – | 72.7 / 104 | – |
| P-521 base | **37.1** / **39.9** | – | 64.8 / 145 | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | – | 15.8 / 26.8 | – | – | 5.2 / 12.0 | – | 51.2 / 59.0 | – |
| BabyBear | 5.2 / **11.0** | – | 15.8 / 26.8 | – | – | **4.1** / 12.2 | – | 51.6 / 59.0 | – |
| Goldilocks | 7.0 / **14.4** | – | 16.2 / 27.0 | – | – | 11.7 / 17.8 | **6.5** / 14.5 | 51.4 / 59.0 | – |
| BN254 scalar | 95.6 / **90.9** | 249 / 249 | 133 / 135 | – | **91.5** / 93.6 | – | – | 902 / 928 | – |
| secp256k1 base | **64.0** / **77.7** | 202 / 202 | 133 / 135 | 158 / 160 | 90.9 / 90.4 | – | – | 907 / 943 | – |
| P-256 base | **71.5** / **72.9** | 180 / 181 | 133 / 134 | 85.1 / 78.1 | 86.2 / 75.2 | – | – | 556 / 584 | 1,077 / – |
| P-256 scalar | **86.3** / **93.9** | 251 / 242 | 133 / 134 | 257 / 254 | 94.9 / 94.0 | – | – | 908 / 943 | 1,074 / – |
| Tom-256 base | **80.1** / **89.3** | – | 133 / 134 | – | 93.7 / 94.2 | – | – | 908 / 945 | – |
| BLS12-381 base | 279 / 286 | 553 / 550 | 265 / 311 | – | **195** / **194** | – | – | 2,012 / 2,042 | – |
| P-521 base | **399** / **420** | – | 599 / 618 | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | – | 15.6 / 27.0 | – | – | 5.1 / 12.0 | – | 46.6 / 53.7 | – |
| BabyBear | **4.8** / **11.0** | – | 15.6 / 27.1 | – | – | 5.1 / 12.1 | – | 46.6 / 55.0 | – |
| Goldilocks | **6.0** / **12.0** | – | 15.9 / 27.1 | – | – | 9.1 / 18.0 | **6.0** / 12.8 | 46.6 / 55.4 | – |
| BN254 scalar | 94.6 / **90.0** | 201 / 202 | 109 / 108 | – | **85.2** / 96.5 | – | – | 899 / 923 | – |
| secp256k1 base | **61.2** / **70.0** | 163 / 161 | 110 / 109 | 98.6 / 118 | 77.9 / 85.0 | – | – | 915 / 944 | – |
| P-256 base | 78.4 / 62.0 | 136 / 135 | 109 / 108 | **73.8** / 76.6 | 75.8 / **61.8** | – | – | 561 / 598 | 1,088 / – |
| P-256 scalar | **80.8** / **88.7** | 186 / 185 | 110 / 109 | 248 / 254 | 83.0 / 92.3 | – | – | 919 / 931 | 1,086 / – |
| Tom-256 base | 83.9 / **83.0** | – | 110 / 108 | – | **78.3** / 85.5 | – | – | 916 / 937 | – |
| BLS12-381 base | 277 / 275 | 439 / 441 | 296 / 313 | – | **164** / **165** | – | – | 2,005 / 2,014 | – |
| P-521 base | **262** / **264** | – | 615 / 638 | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **267** / – | – | 1,572 / – | – | – | 679 / – | – | 2,212 / – | – |
| BabyBear | **267** / – | – | 1,546 / – | – | – | 706 / – | – | 2,357 / – | – |
| Goldilocks | **537** / – | – | 3,118 / – | – | – | 1,389 / – | 912 / – | 4,423 / – | – |
| BN254 scalar | **6,515** / – | 22,531 / – | 15,193 / – | – | 7,997 / – | – | – | 41,710 / – | – |
| secp256k1 base | **6,051** / – | 20,503 / – | 15,376 / – | 33,464 / – | 8,025 / – | – | – | 42,582 / – | – |
| P-256 base | **5,965** / – | 20,738 / – | 15,375 / – | 22,140 / – | 8,044 / – | – | – | 40,955 / – | 250,560 / – |
| P-256 scalar | **6,388** / – | 23,095 / – | 15,298 / – | 46,894 / – | 7,969 / – | – | – | 42,508 / – | 248,703 / – |
| Tom-256 base | **6,401** / – | – | 15,336 / – | – | 8,068 / – | – | – | 42,273 / – | – |
| BLS12-381 base | **11,469** / – | 42,115 / – | 26,257 / – | – | 13,126 / – | – | – | 98,042 / – | – |
| P-521 base | **18,190** / – | – | 50,398 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 329 / – | – | 1,643 / – | – | – | **324** / – | 1,741 / – | – |
| BabyBear | **2,040** / – | – | 20,207 / – | – | – | 3,086 / – | – | – |
| Goldilocks | **3,728** / – | – | 28,881 / – | – | – | 5,700 / – | – | – |
| BN254 scalar | **47,450** / – | 122,757 / – | 133,951 / – | – | 102,000 / – | – | – | – |
| secp256k1 base | **18,998** / – | 90,636 / – | 55,495 / – | 33,104 / – | 43,120 / – | – | 479,075 / – | – |
| P-256 base | **18,939** / – | 39,891 / – | 47,110 / – | 21,668 / – | 38,056 / – | – | 170,169 / – | 318,559 / – |
| P-256 scalar | **35,758** / – | 88,934 / – | 54,740 / – | 115,762 / – | 49,913 / – | – | – | – |
| Tom-256 base | **19,552** / – | – | 51,955 / – | – | 45,705 / – | – | 339,062 / – | – |
| BLS12-381 base | **134,009** / – | 283,750 / – | 145,228 / – | – | 155,087 / – | – | 1,235,027 / – | – |
| P-521 base | **142,346** / – | – | 296,313 / – | – | – | – | – | – |

### mul_u64: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | – | 30.5 / 31.8 | – | – | 9.1\* / 12.6\* | – | – |
| BabyBear | **11.2** / **11.0** | – | 33.0 / 32.3 | – | – | 12.0 / 13.3 | – | – |
| Goldilocks | 11.0 / **12.0** | – | 34.5 / 35.6 | – | – | 11.7 / 17.8 | **10.8** / 13.7 | – |
| BN254 scalar | **28.8** / **34.5** | 515 / 498 | 263 / 259 | – | 286 / 291 | – | – | 1,844 / 1,839 |
| secp256k1 base | **12.6** / **16.0** | 337 / 336 | 263 / 259 | 157 / 163 | 269 / 281 | – | – | 1,853 / 1,844 |
| P-256 base | **27.3** / **33.0** | 351 / 352 | 262 / 259 | 150 / 152 | 235 / 252 | – | – | 1,215 / 1,210 |
| P-256 scalar | **30.3** / **34.6** | 493 / 489 | 263 / 259 | 252 / 254 | 288 / 298 | – | – | 1,842 / 1,868 |
| Tom-256 base | **30.0** / **33.0** | – | 263 / 258 | – | 292 / 301 | – | – | 1,861 / 1,859 |
| BLS12-381 base | **44.2** / **47.9** | 1,123 / 1,123 | 509 / 593 | – | 482 / 487 | – | – | 4,086 / 4,088 |
| P-521 base | **32.1** / **39.4** | – | 989 / 996 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.4 / 31.0 | – | – | 8.9\* / 12.5\* | – |
| BabyBear | **11.2** / **11.0** | 32.9 / 32.1 | – | – | 12.1 / 13.3 | – |
| Goldilocks | 11.0 / **12.0** | 34.7 / 35.6 | – | – | 11.7 / 17.8 | **10.8** / 13.7 |
| BN254 scalar | **179** / 187 | 263 / 261 | – | 180 / **181** | – | – |
| secp256k1 base | **113** / **115** | 262 / 259 | – | 152 / 155 | – | – |
| P-256 base | 174 / 181 | 263 / 260 | – | **143** / **145** | – | – |
| P-256 scalar | 187 / 190 | 262 / 260 | 295 / 293 | **184** / **187** | – | – |
| Tom-256 base | **178** / 179 | 262 / 259 | – | 180 / **178** | – | – |
| BLS12-381 base | **381** / **377** | 519 / 594 | – | 382 / 378 | – | – |
| P-521 base | **617** / **645** | 1,007 / 1,009 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **8.6** | – | 30.3 / 30.1 | – | – | 7.6\* / 8.1\* | – | – |
| BabyBear | **8.6** / **8.2** | – | 31.6 / 30.1 | – | – | 10.0 / 9.2 | – | – |
| Goldilocks | 7.7 / **7.3** | – | 32.5 / 30.9 | – | – | 4.5\* / 8.0 | **7.4** / 7.5 | – |
| BN254 scalar | **45.1** / **41.9** | 271 / 265 | 221 / 228 | – | 239 / 227 | – | – | 1,055 / 1,052 |
| secp256k1 base | **16.1** / **23.0** | 122 / 120 | 221 / 229 | 13.8\* / 5.3\* | 236 / 201 | – | – | 1,021 / 1,027 |
| P-256 base | **42.6** / **40.8** | 171 / 163 | 225 / 228 | 96.8 / 92.5 | 214 / 193 | – | – | 701 / 682 |
| P-256 scalar | 41.4 / **41.4** | 248 / 242 | 221 / 229 | **41.1** / 56.8 | 239 / 216 | – | – | 1,052 / 1,055 |
| Tom-256 base | **45.3** / **45.8** | – | 220 / 229 | – | 245 / 223 | – | – | 1,056 / 1,054 |
| BLS12-381 base | **66.4** / **69.5** | 615 / 615 | 405 / 412 | – | 323 / 324 | – | – | 2,210 / 2,224 |
| P-521 base | **44.2** / **45.2** | – | 649 / 657 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 30.4 / 30.1 | – | – | 7.6\* / 8.1\* | – |
| BabyBear | **8.6** / **8.2** | 31.6 / 30.0 | – | – | 10.0 / 9.2 | – |
| Goldilocks | 7.7 / **7.3** | 32.6 / 31.4 | – | – | 4.5\* / 8.0 | **7.4** / 7.4 |
| BN254 scalar | 136 / 129 | 238 / 231 | – | **112** / **112** | – | – |
| secp256k1 base | **18.6** / **23.5** | 237 / 231 | – | 86.1 / 85.3 | – | – |
| P-256 base | 95.7 / 93.6 | 237 / 231 | – | **91.3** / **88.7** | – | – |
| P-256 scalar | 114 / 120 | 237 / 232 | **94.7** / **82.1** | 113 / 113 | – | – |
| Tom-256 base | 115 / 115 | 237 / 231 | – | **110** / **110** | – | – |
| BLS12-381 base | 362 / 355 | 408 / 400 | – | **224** / **223** | – | – |
| P-521 base | **71.3** / **63.7** | 666 / 664 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | – | 17.9 / 18.3 | – | – | 6.1\* / 6.1 | – | – |
| BabyBear | **8.5** / **8.2** | – | 19.5 / 19.3 | – | – | 10.7 / 9.0 | – | – |
| Goldilocks | **7.0** / **6.4** | – | 23.3 / 23.2 | – | – | 3.5\* / 5.0\* | **7.0** / 6.8 | – |
| BN254 scalar | **45.1** / **42.2** | 274 / 267 | 151 / 153 | – | 253 / 214 | – | – | 1,005 / 1,004 |
| secp256k1 base | **12.3** / **15.1** | 123 / 121 | 150 / 152 | 14.0 / 11.0\* | 245 / 216 | – | – | 976 / 979 |
| P-256 base | **36.3** / **36.0** | 173 / 165 | 151 / 150 | 89.8 / 88.0 | 230 / 200 | – | – | 647 / 645 |
| P-256 scalar | 38.7 / 37.8 | 255 / 243 | 152 / 151 | **21.2** / **21.2** | 236 / 216 | – | – | 1,004 / 994 |
| Tom-256 base | **42.3** / **41.6** | – | 151 / 151 | – | 240 / 219 | – | – | 1,005 / 988 |
| BLS12-381 base | **69.0** / **70.0** | 611 / 602 | 386 / 376 | – | 314 / 314 | – | – | 2,150 / 2,171 |
| P-521 base | **34.6** / **35.5** | – | 620 / 640 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.4** | 17.9 / 18.3 | – | – | 6.1\* / 6.1 | – |
| BabyBear | **8.5** / **8.2** | 19.5 / 19.4 | – | – | 10.7 / 9.0 | – |
| Goldilocks | **7.0** / **6.5** | 23.3 / 23.2 | – | – | 3.5\* / 5.0\* | **7.0** / 7.1 |
| BN254 scalar | 137 / 128 | 153 / 150 | – | **103** / **105** | – | – |
| secp256k1 base | **15.0** / **15.7** | 152 / 150 | – | 108 / 80.8 | – | – |
| P-256 base | 87.7 / 88.2 | 153 / 149 | – | **87.0** / **85.7** | – | – |
| P-256 scalar | 114 / 113 | 153 / 149 | **70.0** / **71.9** | 106 / 106 | – | – |
| Tom-256 base | 109 / 109 | 153 / 150 | – | **104** / **103** | – | – |
| BLS12-381 base | 361 / 354 | 387 / 381 | – | **215** / **215** | – | – |
| P-521 base | **55.7** / **49.2** | 634 / 639 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 11.6 / – | – | – | **4.1** / – | – | – |
| BabyBear | **7.3** / – | – | 12.6 / – | – | – | 7.8 / – | – | – |
| Goldilocks | **5.5** / – | – | 17.0 / – | – | – | 2.2\* / – | 5.9 / – | – |
| BN254 scalar | **28.1** / – | 248 / – | 133 / – | – | 188 / – | – | – | 930 / – |
| secp256k1 base | **3.7** / – | 114 / – | 133 / – | 6.1 / – | 169 / – | – | – | 932 / – |
| P-256 base | **25.3** / – | 147 / – | 133 / – | 76.0 / – | 159 / – | – | – | 563 / – |
| P-256 scalar | 25.9 / – | 225 / – | 133 / – | **4.0** / – | 189 / – | – | – | 931 / – |
| Tom-256 base | **28.9** / – | – | 133 / – | – | 192 / – | – | – | 937 / – |
| BLS12-381 base | **44.6** / – | 568 / – | 255 / – | – | 290 / – | – | – | 2,057 / – |
| P-521 base | **6.0** / – | – | 500 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **835** / **844** | 5,645 / 5,605 | 23,054 / 23,240 | 14,171 / 14,122 | – | 7,019 / 6,961 | 33,315 / 33,636 |
| secp256k1 base | **776** / **774** | 5,065 / 5,025 | 17,548 / 17,280 | 14,215 / 14,056 | 33,608 / 33,624 | 6,949 / 6,961 | 34,405 / 34,544 |
| P-256 base | **877** / **888** | 5,227 / 5,105 | 20,606 / 20,578 | 14,196 / 14,231 | 22,284 / 23,668 | 7,015 / 6,946 | 33,765 / 32,229 |
| P-256 scalar | **899** / **901** | 5,515 / 5,434 | 22,453 / 22,677 | 14,238 / 14,159 | 35,415 / 35,361 | 7,072 / 6,959 | 35,513 / 33,570 |
| Tom-256 base | **826** / **855** | 5,370 / 5,414 | – | 14,187 / 14,106 | – | 6,901 / 6,954 | 34,999 / 33,818 |
| BLS12-381 base | **1,130** / **1,137** | 9,855 / 9,959 | 42,303 / 42,269 | 23,742 / 23,845 | – | 11,054 / 11,060 | 74,120 / 73,293 |
| P-521 base | **1,366** / **1,367** | 14,892 / 14,757 | – | 45,441 / 45,424 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 277 / 278 | **274** / **271** | – | 1,600 / 1,609 | – | – | 675 / 673 | – | 2,426 / 2,389 |
| BabyBear | **277** / **279** | 288 / 288 | – | 1,582 / 1,596 | – | – | 670 / 656 | – | 2,569 / 2,505 |
| Goldilocks | **545** / **546** | 562 / 564 | – | 3,179 / 3,194 | – | – | 1,386 / 1,377 | 949 / 927 | 4,764 / 4,754 |
| BN254 scalar | **6,730** / **6,601** | 6,763 / 6,622 | 22,978 / 23,257 | 15,628 / 15,504 | – | 8,201 / 8,192 | – | – | 44,307 / 44,119 |
| secp256k1 base | 6,060 / **6,047** | **6,054** / 6,062 | 20,789 / 20,787 | 15,716 / 15,617 | 33,240 / 33,892 | 8,202 / 8,183 | – | – | 44,212 / 44,509 |
| P-256 base | **6,134** / **6,145** | 6,249 / 6,187 | 21,333 / 21,345 | 15,731 / 15,644 | 22,464 / 22,441 | 8,195 / 8,248 | – | – | 42,782 / 42,756 |
| P-256 scalar | **6,643** / **6,569** | 6,693 / 6,647 | 23,793 / 23,644 | 15,654 / 15,580 | 47,138 / 47,245 | 8,339 / 8,217 | – | – | 45,562 / 44,322 |
| Tom-256 base | 6,711 / **6,458** | **6,643** / 6,654 | – | 15,613 / 15,664 | – | 8,258 / 8,236 | – | – | 43,852 / 44,483 |
| BLS12-381 base | **12,324** / **12,240** | 12,454 / 12,355 | 43,487 / 43,515 | 27,070 / 26,962 | – | 13,696 / 13,706 | – | – | 101,813 / 102,601 |
| P-521 base | **18,449** / 18,251 | 18,699 / **18,222** | – | 50,997 / 51,273 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | 11.7 / 19.3 | – | – | – | 10.8 / 18.0 |
| BabyBear | **2.4** / **3.0** | 11.7 / 18.7 | – | – | – | 10.8 / 18.0 |
| Goldilocks | 3.3 / **3.0** | 11.4 / 19.1 | – | – | **3.2** / 4.0 | 10.8 / 18.0 |
| BN254 scalar | **16.4** / **19.5** | 42.5 / 87.3 | – | 20.7 / 23.1 | – | 62.2 / 104 |
| secp256k1 base | **18.5** / **22.6** | 42.7 / 87.2 | 22.6 / 14.5\* | 19.3 / 23.1 | – | 62.3 / 104 |
| P-256 base | 18.1 / 22.8 | 42.7 / 87.4 | **17.9** / 21.7 | 18.0 / **21.0** | – | 62.2 / 104 |
| P-256 scalar | **17.9** / **23.7** | 42.5 / 87.0 | 42.3 / 58.1 | 20.6 / 24.8 | – | 62.1 / 103 |
| Tom-256 base | **18.0** / **23.9** | 42.4 / 87.3 | – | 20.2 / 24.0 | – | 62.0 / 103 |
| BLS12-381 base | **25.6** / **28.4** | 107 / 115 | – | 32.4 / 34.1 | – | 115 / 158 |
| P-521 base | **46.6** / **48.8** | 134 / 179 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.6** / **3.0** | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.5 / 8.2 |
| BabyBear | **2.5** / **3.0** | 4.5 / 8.0 | – | – | – | – | 8.7 / 7.9 |
| Goldilocks | **2.6** / **3.0** | 4.5 / 8.0 | – | – | – | 2.8 / **3.0** | 10.1 / 8.0 |
| BN254 scalar | 16.7 / 16.9 | 26.7 / 27.0 | – | **15.0** / **15.0** | – | – | 37.7 / 77.0 |
| secp256k1 base | 14.3 / 15.5 | 26.7 / 27.2 | 25.6 / 19.7 | **14.2** / **14.5** | – | – | 37.0 / 77.4 |
| P-256 base | 13.4 / 15.5 | 26.9 / 25.8 | 13.5 / 14.7 | **12.5** / **14.0** | – | – | 37.8 / 77.3 |
| P-256 scalar | 14.8 / **15.8** | 26.7 / 29.0 | 23.3 / 25.1 | **14.4** / **15.8** | – | – | 37.3 / 77.1 |
| Tom-256 base | 15.7 / 17.1 | 26.7 / 27.0 | – | **14.9** / **14.9** | – | – | 37.7 / 77.3 |
| BLS12-381 base | 24.6 / **24.9** | 43.8 / 97.2 | – | **23.5** / **24.9** | – | – | 72.7 / 104 |
| P-521 base | **37.2** / **39.7** | 64.8 / 145 | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | 15.8 / 26.8 | – | – | – | 51.2 / 59.0 |
| BabyBear | **5.2** / **11.0** | 15.8 / 26.8 | – | – | – | 51.6 / 59.0 |
| Goldilocks | 7.0 / **14.4** | 16.2 / 27.0 | – | – | **6.5** / 14.5 | 51.4 / 59.0 |
| BN254 scalar | **80.8** / 94.5 | 133 / 135 | – | 91.5 / **93.6** | – | 902 / 928 |
| secp256k1 base | **64.0** / **77.5** | 133 / 135 | 158 / 160 | 90.9 / 90.4 | – | 907 / 943 |
| P-256 base | **71.9** / **73.2** | 133 / 134 | 85.1 / 78.1 | 86.2 / 75.2 | – | 556 / 584 |
| P-256 scalar | **85.3** / **93.6** | 133 / 134 | 257 / 254 | 94.9 / 94.0 | – | 908 / 943 |
| Tom-256 base | **81.9** / **88.5** | 133 / 134 | – | 93.7 / 94.2 | – | 908 / 945 |
| BLS12-381 base | 256 / 261 | 265 / 311 | – | **195** / **194** | – | 2,012 / 2,042 |
| P-521 base | **400** / **419** | 599 / 618 | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **11.9** | 15.6 / 27.0 | – | – | – | 46.6 / 53.7 |
| BabyBear | **4.8** / **11.0** | 15.6 / 27.1 | – | – | – | 46.6 / 55.0 |
| Goldilocks | **6.0** / **12.0** | 15.9 / 27.1 | – | – | **6.0** / 12.8 | 46.6 / 55.4 |
| BN254 scalar | **82.9** / **84.1** | 109 / 108 | – | 85.2 / 96.5 | – | 899 / 923 |
| secp256k1 base | **61.4** / **69.9** | 110 / 109 | 98.6 / 118 | 77.9 / 85.0 | – | 915 / 944 |
| P-256 base | 78.5 / 62.1 | 109 / 108 | **73.8** / 76.6 | 75.8 / **61.8** | – | 561 / 598 |
| P-256 scalar | **80.3** / **89.0** | 110 / 109 | 248 / 254 | 83.0 / 92.3 | – | 919 / 931 |
| Tom-256 base | **75.6** / **83.0** | 110 / 108 | – | 78.3 / 85.5 | – | 916 / 937 |
| BLS12-381 base | 233 / 228 | 296 / 313 | – | **164** / **165** | – | 2,005 / 2,014 |
| P-521 base | **263** / **263** | 615 / 638 | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **438** / – | 5,336 / – | – | – | – | 3,341 / – |
| BabyBear | **459** / – | 5,344 / – | – | – | – | 3,319 / – |
| Goldilocks | **857** / – | 5,346 / – | – | – | 912 / – | 7,054 / – |
| BN254 scalar | **9,422** / – | 25,541 / – | – | 60,043 / – | – | 363,047 / – |
| secp256k1 base | **9,105** / – | 25,625 / – | 33,464 / – | 52,650 / – | – | 476,369 / – |
| P-256 base | **8,929** / – | 25,286 / – | 22,140 / – | 43,686 / – | – | 225,690 / – |
| P-256 scalar | **9,346** / – | 25,606 / – | 109,882 / – | 60,503 / – | – | 406,499 / – |
| Tom-256 base | **9,256** / – | 25,489 / – | – | 58,355 / – | – | 339,906 / – |
| BLS12-381 base | **16,498** / – | 42,681 / – | – | 164,826 / – | – | 1,238,817 / – |
| P-521 base | **26,056** / – | 84,111 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **334** / – | 1,643 / – | – | – | 1,741 / – |
| BabyBear | **6,463** / – | 20,207 / – | – | – | – |
| Goldilocks | **8,886** / – | 28,881 / – | – | – | – |
| BN254 scalar | **83,973** / – | 133,951 / – | – | 102,000 / – | – |
| secp256k1 base | **19,799** / – | 55,495 / – | 33,104 / – | 43,120 / – | 479,075 / – |
| P-256 base | **19,805** / – | 47,110 / – | 21,668 / – | 38,056 / – | 170,169 / – |
| P-256 scalar | **37,051** / – | 54,740 / – | 115,762 / – | 49,913 / – | – |
| Tom-256 base | **27,576** / – | 51,955 / – | – | 45,705 / – | 339,062 / – |
| BLS12-381 base | **142,850** / – | 145,228 / – | – | 155,087 / – | 1,235,027 / – |
| P-521 base | **136,021** / – | 296,313 / – | – | – | – |

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.5 / 31.8 | – | – | – |
| BabyBear | **11.2** / **11.0** | 33.0 / 32.3 | – | – | – |
| Goldilocks | 11.0 / **12.0** | 34.5 / 35.6 | – | – | **10.8** / 13.7 |
| BN254 scalar | **38.8** / **50.9** | 263 / 259 | – | 286 / 291 | – |
| secp256k1 base | **24.9** / **39.0** | 263 / 259 | 157 / 163 | 269 / 281 | – |
| P-256 base | **40.4** / **53.2** | 262 / 259 | 150 / 152 | 235 / 252 | – |
| P-256 scalar | **43.5** / **55.0** | 263 / 259 | 252 / 254 | 288 / 298 | – |
| Tom-256 base | **41.8** / **52.0** | 263 / 258 | – | 292 / 301 | – |
| BLS12-381 base | **65.9** / **72.0** | 509 / 593 | – | 482 / 487 | – |
| P-521 base | **69.3** / **77.7** | 989 / 996 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.4 / 31.0 | – | – | – |
| BabyBear | **11.2** / **11.0** | 32.9 / 32.1 | – | – | – |
| Goldilocks | 11.0 / **12.0** | 34.7 / 35.6 | – | – | **10.8** / 13.7 |
| BN254 scalar | 248 / 250 | 263 / 261 | – | **180** / **181** | – |
| secp256k1 base | 192 / 192 | 262 / 259 | – | **152** / **155** | – |
| P-256 base | 271 / 247 | 263 / 260 | – | **143** / **145** | – |
| P-256 scalar | 264 / 264 | 262 / 260 | 295 / 293 | **184** / **187** | – |
| Tom-256 base | 259 / 258 | 262 / 259 | – | **180** / **178** | – |
| BLS12-381 base | 516 / 514 | 519 / 594 | – | **382** / **378** | – |
| P-521 base | **969** / **998** | 1,007 / 1,009 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.6** | 30.3 / 30.1 | – | – | – |
| BabyBear | **8.6** / **8.2** | 31.6 / 30.1 | – | – | – |
| Goldilocks | 7.7 / **7.3** | 32.5 / 30.9 | – | – | **7.4** / 7.5 |
| BN254 scalar | **58.0** / **56.3** | 221 / 228 | – | 239 / 227 | – |
| secp256k1 base | **16.1** / **23.0** | 221 / 229 | 13.8\* / 5.3\* | 236 / 201 | – |
| P-256 base | **58.7** / **56.4** | 225 / 228 | 96.8 / 92.5 | 214 / 193 | – |
| P-256 scalar | 57.0 / 58.5 | 221 / 229 | **41.1** / **56.8** | 239 / 216 | – |
| Tom-256 base | **62.6** / **61.8** | 220 / 229 | – | 245 / 223 | – |
| BLS12-381 base | **93.2** / **96.0** | 405 / 412 | – | 323 / 324 | – |
| P-521 base | **44.1** / **45.1** | 649 / 657 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 30.4 / 30.1 | – | – | – |
| BabyBear | **8.6** / **8.2** | 31.6 / 30.0 | – | – | – |
| Goldilocks | 7.7 / **7.3** | 32.6 / 31.4 | – | – | **7.4** / 7.4 |
| BN254 scalar | 134 / 131 | 238 / 231 | – | **112** / **112** | – |
| secp256k1 base | **29.6** / **29.3** | 237 / 231 | – | 86.1 / 85.3 | – |
| P-256 base | 116 / 108 | 237 / 231 | – | **91.3** / **88.7** | – |
| P-256 scalar | 136 / 136 | 237 / 232 | **94.7** / **82.1** | 113 / 113 | – |
| Tom-256 base | 113 / 131 | 237 / 231 | – | **110** / **110** | – |
| BLS12-381 base | 358 / 364 | 408 / 400 | – | **224** / **223** | – |
| P-521 base | **105** / **93.2** | 666 / 664 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 17.9 / 18.3 | – | – | – |
| BabyBear | **8.5** / **8.3** | 19.5 / 19.3 | – | – | – |
| Goldilocks | **7.0** / **6.6** | 23.3 / 23.2 | – | – | **7.0** / 6.8 |
| BN254 scalar | **60.1** / **56.8** | 151 / 153 | – | 253 / 214 | – |
| secp256k1 base | **12.3** / **15.1** | 150 / 152 | 14.0 / 11.0\* | 245 / 216 | – |
| P-256 base | **52.2** / **49.0** | 151 / 150 | 89.8 / 88.0 | 230 / 200 | – |
| P-256 scalar | 52.3 / 50.5 | 152 / 151 | **21.2** / **21.2** | 236 / 216 | – |
| Tom-256 base | **56.4** / **56.4** | 151 / 151 | – | 240 / 219 | – |
| BLS12-381 base | **86.1** / **90.0** | 386 / 376 | – | 314 / 314 | – |
| P-521 base | **34.6** / **35.2** | 620 / 640 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.4** | 17.9 / 18.3 | – | – | – |
| BabyBear | **8.5** / **8.3** | 19.5 / 19.4 | – | – | – |
| Goldilocks | **7.0** / **6.5** | 23.3 / 23.2 | – | – | **7.0** / 7.1 |
| BN254 scalar | 134 / 130 | 153 / 150 | – | **103** / **105** | – |
| secp256k1 base | **26.0** / **24.9** | 152 / 150 | – | 108 / 80.8 | – |
| P-256 base | 106 / 98.2 | 153 / 149 | – | **87.0** / **85.7** | – |
| P-256 scalar | 123 / 127 | 153 / 149 | **70.0** / **71.9** | 106 / 106 | – |
| Tom-256 base | 133 / 130 | 153 / 150 | – | **104** / **103** | – |
| BLS12-381 base | 344 / 346 | 387 / 381 | – | **215** / **215** | – |
| P-521 base | **86.8** / **85.3** | 634 / 639 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / – | 11.6 / – | – | – | – |
| BabyBear | **10.4** / – | 12.6 / – | – | – | – |
| Goldilocks | 9.3 / – | 17.0 / – | – | – | **5.9** / – |
| BN254 scalar | **81.1** / – | 133 / – | – | 188 / – | – |
| secp256k1 base | **3.4** / – | 133 / – | 6.1 / – | 169 / – | – |
| P-256 base | **48.4** / – | 133 / – | 76.0 / – | 159 / – | – |
| P-256 scalar | 81.0 / – | 133 / – | **4.0** / – | 189 / – | – |
| Tom-256 base | **79.0** / – | 133 / – | – | 192 / – | – |
| BLS12-381 base | **154** / – | 255 / – | – | 290 / – | – |
| P-521 base | **6.0** / – | 500 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **9,504** / **9,387** | 9,558 / 9,424 | 24,366 / 25,715 | – | 61,566 / 61,052 | 351,988 / 347,491 |
| secp256k1 base | **9,088** / **9,030** | 9,145 / 9,138 | 24,369 / 25,719 | 33,608 / 33,624 | 54,371 / 54,408 | 468,537 / 465,122 |
| P-256 base | **9,154** / **9,170** | 9,262 / 9,222 | 24,453 / 25,539 | 22,284 / 23,668 | 44,188 / 44,178 | 226,926 / 227,812 |
| P-256 scalar | **9,399** / **9,502** | 9,684 / 9,683 | 24,484 / 25,942 | 110,689 / 116,089 | 59,838 / 59,467 | 396,781 / 396,182 |
| Tom-256 base | **9,372** / 9,366 | 9,478 / **9,345** | 24,185 / 26,045 | – | 58,227 / 57,573 | 331,739 / 331,801 |
| BLS12-381 base | **17,018** / **17,051** | 17,117 / 17,278 | 43,346 / 43,475 | – | 160,491 / 161,442 | 1,251,660 / 1,239,358 |
| P-521 base | **26,388** / **26,326** | 26,597 / 26,807 | 84,388 / 84,701 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 457 / **451** | **456** / 454 | 5,572 / 5,202 | – | – | – | 3,505 / 3,535 |
| BabyBear | 466 / **436** | **439** / 440 | 5,533 / 5,282 | – | – | – | 3,559 / 3,469 |
| Goldilocks | 950 / 950 | 952 / 948 | 5,570 / 5,102 | – | – | **949** / **927** | 7,342 / 7,506 |
| BN254 scalar | **9,645** / **9,500** | 9,668 / 9,636 | 25,481 / 26,079 | – | 60,447 / 61,563 | – | 352,976 / 352,710 |
| secp256k1 base | **9,118** / **9,080** | 9,195 / 9,109 | 24,842 / 26,226 | 33,240 / 33,892 | 53,690 / 54,677 | – | 469,421 / 466,689 |
| P-256 base | **9,096** / **9,135** | 9,357 / 9,241 | 24,655 / 25,571 | 22,464 / 22,441 | 43,084 / 43,650 | – | 227,586 / 226,227 |
| P-256 scalar | **9,521** / **9,575** | 9,699 / 9,609 | 24,811 / 26,132 | 110,061 / 116,030 | 59,786 / 59,440 | – | 396,939 / 398,391 |
| Tom-256 base | **9,484** / 9,499 | 9,511 / **9,422** | 25,080 / 26,324 | – | 58,847 / 57,936 | – | 331,577 / 330,846 |
| BLS12-381 base | **17,400** / **17,269** | 17,815 / 17,625 | 44,234 / 43,521 | – | 162,979 / 161,832 | – | 1,247,505 / 1,238,077 |
| P-521 base | **26,165** / **26,442** | 27,053 / 26,807 | 87,617 / 83,177 | – | – | – | – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | – | 10.8 / 19.8 | – | – | 4.4\* / 7.8 | – | 10.8 / 18.3 | – |
| BabyBear | 3.0 / **3.0** | – | 10.8 / 19.4 | – | – | **2.7** / **3.0** | – | 10.8 / 18.3 | – |
| Goldilocks | **3.1** / **3.0** | – | 12.1 / 19.0 | – | – | 4.0 / 6.0 | 4.0 / 4.4 | 10.8 / 18.3 | – |
| BN254 scalar | 30.2 / 39.7 | **14.2** / **13.7** | 59.7 / 90.0 | – | 39.5 / 44.0 | – | – | 76.5 / 116 | – |
| secp256k1 base | 31.5 / 39.4 | **16.4** / **14.9** | 59.6 / 90.3 | 16.3\* / 15.7\* | 35.2 / 38.3 | – | – | 76.6 / 116 | – |
| P-256 base | 27.9 / 33.1 | **17.3** / **15.7** | 59.5 / 89.9 | 33.9 / 39.6 | 33.9 / 39.2 | – | – | 76.4 / 120 | 542 / – |
| P-256 scalar | 33.1 / 41.0 | **17.3** / **15.9** | 59.5 / 90.0 | 56.4 / 60.6 | 38.2 / 42.5 | – | – | 76.4 / 120 | 533 / – |
| Tom-256 base | **35.9** / **39.6** | – | 59.4 / 89.9 | – | 38.5 / 42.6 | – | – | 76.8 / 115 | – |
| BLS12-381 base | 54.0 / 61.5 | **40.0** / **45.6** | 91.0 / 138 | – | 69.9 / 67.3 | – | – | 125 / 166 | – |
| P-521 base | **88.2** / **89.4** | – | 171 / 245 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 17.2 / 14.4 | – |
| BabyBear | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / **3.0** | – | 16.6 / 14.5 | – |
| Goldilocks | **3.0** / **3.0** | – | 5.0 / 8.0 | – | – | 4.0 / 5.0 | **3.0** / **3.0** | 18.7 / 13.2 | – |
| BN254 scalar | 24.3 / 32.0 | **18.9** / **16.6** | 34.4 / 30.1 | – | 23.2 / 26.1 | – | – | 54.6 / 78.0 | – |
| secp256k1 base | 21.0 / 25.9 | **19.9** / **17.1** | 34.6 / 32.8 | 16.1\* / 18.5\* | 20.3 / 24.6 | – | – | 54.8 / 77.2 | – |
| P-256 base | 22.2 / 23.7 | **18.2** / **15.4** | 38.2 / 29.1 | 21.7 / 24.6 | 21.1 / 25.0 | – | – | 54.3 / 78.0 | 278 / – |
| P-256 scalar | 22.4 / 24.3 | **18.7** / **16.8** | 39.3 / 33.7 | 26.5 / 27.0 | 23.2 / 27.9 | – | – | 54.7 / 78.4 | 279 / – |
| Tom-256 base | 23.6 / **25.2** | – | 31.8 / 30.1 | – | **23.5** / 26.6 | – | – | 54.7 / 78.2 | – |
| BLS12-381 base | 42.0 / 43.4 | **31.2** / **30.4** | 55.6 / 110 | – | 44.5 / 44.1 | – | – | 67.9 / 103 | – |
| P-521 base | **63.6** / **62.5** | – | 124 / 214 | – | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | – | 38.9 / 50.0 | – | – | 5.5\* / 12.0 | – | 49.8 / 56.0 | – |
| BabyBear | 15.1 / 22.0 | – | 38.7 / 50.6 | – | – | **6.5** / **13.0** | – | 49.9 / 56.1 | – |
| Goldilocks | 18.0 / 22.8 | – | 37.8 / 51.3 | – | – | 13.1\* / 18.0\* | **17.1** / **19.0** | 49.8 / 56.0 | – |
| BN254 scalar | 593 / 590 | **398** / **381** | 888 / 899 | – | 702 / 693 | – | – | 1,058 / 1,076 | – |
| secp256k1 base | **407** / 408 | **407** / **400** | 882 / 905 | 171\* / 171\* | 600 / 588 | – | – | 1,072 / 1,085 | – |
| P-256 base | 457 / 459 | **444** / **444** | 886 / 902 | 486 / 472 | 490 / 482 | – | – | 865 / 873 | 1,316 / – |
| P-256 scalar | 634 / 632 | **415** / **405** | 889 / 906 | 825 / 822 | 684 / 674 | – | – | 1,072 / 1,079 | 1,303 / – |
| Tom-256 base | **569** / **572** | – | 880 / 900 | – | 603 / 590 | – | – | 1,071 / 1,084 | – |
| BLS12-381 base | 1,837 / 1,828 | **1,060** / **1,028** | 1,291 / 1,309 | – | 1,723 / 1,569 | – | – | 2,166 / 2,179 | – |
| P-521 base | **1,803** / **1,770** | – | 2,718 / 2,773 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | – | 38.5 / 52.9 | – | – | 5.4\* / 12.0 | – | 49.1 / 56.0 | – |
| BabyBear | 16.9 / 28.0 | – | 38.5 / 51.2 | – | – | **5.6** / **13.0** | – | 49.0 / 56.0 | – |
| Goldilocks | 18.1 / 22.9 | – | 38.6 / 52.0 | – | – | 11.3\* / 18.0\* | **17.1** / **20.0** | 49.1 / 56.0 | – |
| BN254 scalar | 500 / 501 | **358** / **348** | 635 / 650 | – | 533 / 551 | – | – | 1,059 / 1,066 | – |
| secp256k1 base | 330 / 322 | **309** / **299** | 631 / 642 | 115\* / 136\* | 464 / 442 | – | – | 1,068 / 1,075 | – |
| P-256 base | 366 / 363 | 339 / **330** | 636 / 616 | 473 / 454 | **336** / 346 | – | – | 877 / 736 | 1,318 / – |
| P-256 scalar | 527 / 537 | **352** / **344** | 643 / 640 | 815 / 810 | 552 / 570 | – | – | 1,071 / 1,080 | 1,319 / – |
| Tom-256 base | 488 / 472 | – | 639 / 632 | – | **475** / **453** | – | – | 1,072 / 1,078 | – |
| BLS12-381 base | 1,509 / 1,430 | **831** / **826** | 1,348 / 1,337 | – | 1,283 / 1,233 | – | – | 2,172 / 2,172 | – |
| P-521 base | **952** / **943** | – | 2,792 / 2,846 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **269** / – | – | 2,162 / – | – | – | 710 / – | – | 1,997 / – | – |
| BabyBear | **290** / – | – | 2,060 / – | – | – | 696 / – | – | 2,077 / – | – |
| Goldilocks | **571** / – | – | 4,211 / – | – | – | 1,516 / – | 1,831 / – | 4,042 / – | – |
| BN254 scalar | **11,911** / – | 25,090 / – | 23,742 / – | – | 21,377 / – | – | – | 51,951 / – | – |
| secp256k1 base | **12,000** / – | 24,058 / – | 23,751 / – | 37,261 / – | 21,518 / – | – | – | 52,310 / – | – |
| P-256 base | **11,124** / – | 24,865 / – | 23,766 / – | 127,319 / – | 21,458 / – | – | – | 51,443 / – | 275,909 / – |
| P-256 scalar | **11,823** / – | 25,660 / – | 23,778 / – | 62,744 / – | 21,636 / – | – | – | 52,542 / – | 271,979 / – |
| Tom-256 base | **11,734** / – | – | 23,845 / – | – | 21,775 / – | – | – | 52,247 / – | – |
| BLS12-381 base | **25,332** / – | 50,773 / – | 44,364 / – | – | 37,668 / – | – | – | 125,115 / – | – |
| P-521 base | **51,648** / – | – | 80,203 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **330** / – | – | 3,696 / – | – | – | 331 / – | 1,746 / – | – |
| BabyBear | 6,222 / – | – | 38,401 / – | – | – | **2,997** / – | – | – |
| Goldilocks | **7,909** / – | – | 54,516 / – | – | – | 6,033\* / – | – | – |
| BN254 scalar | 266,820 / – | **214,115** / – | 584,124 / – | – | 523,801 / – | – | – | – |
| secp256k1 base | **98,469** / – | 173,159 / – | 247,849 / – | 36,947\* / – | 280,311 / – | – | 537,243 / – | – |
| P-256 base | **96,945** / – | 98,462 / – | 197,378 / – | 121,856 / – | 210,535 / – | – | 238,587 / – | 370,256 / – |
| P-256 scalar | 189,655 / – | **159,348** / – | 246,406 / – | 363,290 / – | 274,599 / – | – | – | – |
| Tom-256 base | **152,559** / – | – | 228,901 / – | – | 287,509 / – | – | 376,664 / – | – |
| BLS12-381 base | 764,841 / – | **569,281** / – | 616,249 / – | – | 1,184,280 / – | – | 1,321,204 / – | – |
| P-521 base | **532,694** / – | – | 1,563,360 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | – | 72.0 / 73.2 | – | – | 9.6\* / 12.8\* | – | – |
| BabyBear | 47.1 / 47.6 | – | 83.4 / 86.5 | – | – | **46.3** / **45.2** | – | – |
| Goldilocks | 36.3 / 35.9 | – | 94.5 / 89.8 | – | – | 13.3\* / 18.0\* | **31.6** / **32.1** | – |
| BN254 scalar | **187** / **177** | 836 / 831 | 1,738 / 1,745 | – | 1,419 / 1,595 | – | – | 2,148 / 2,159 |
| secp256k1 base | **72.1** / **72.7** | 616 / 613 | 1,754 / 1,739 | 167 / 176 | 1,009 / 1,019 | – | – | 2,144 / 2,148 |
| P-256 base | **167** / **163** | 860 / 838 | 1,762 / 1,734 | 905 / 903 | 1,289 / 1,159 | – | – | 1,428 / 1,459 |
| P-256 scalar | **171** / **178** | 825 / 828 | 1,736 / 1,765 | 821 / 814 | 1,464 / 1,589 | – | – | 2,330 / 2,231 |
| Tom-256 base | **172** / **165** | – | 1,751 / 1,732 | – | 1,260 / 1,291 | – | – | 2,158 / 2,165 |
| BLS12-381 base | **262** / **245** | 2,122 / 2,132 | 2,455 / 2,468 | – | 3,714 / 3,725 | – | – | 4,554 / 4,485 |
| P-521 base | **164** / **152** | – | 4,002 / 4,097 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 72.6 / 71.0 | – | – | 9.5\* / 12.8\* | – |
| BabyBear | 47.1 / 47.4 | 85.9 / 83.3 | – | – | **35.1** / **34.0** | – |
| Goldilocks | 36.3 / 35.9 | 94.4 / 89.6 | – | – | 13.4\* / 18.0\* | **31.6** / **32.3** |
| BN254 scalar | **903** / **888** | 1,808 / 1,730 | – | 1,363 / 1,342 | – | – |
| secp256k1 base | **456** / **456** | 1,790 / 1,747 | – | 979 / 967 | – | – |
| P-256 base | **831** / **810** | 1,786 / 1,742 | – | 1,222 / 926 | – | – |
| P-256 scalar | **837** / **834** | 1,797 / 1,733 | 878 / 863 | 1,359 / 1,380 | – | – |
| Tom-256 base | **839** / **844** | 1,761 / 1,671 | – | 1,185 / 1,195 | – | – |
| BLS12-381 base | **1,784** / **1,774** | 2,505 / 2,484 | – | 3,713 / 3,661 | – | – |
| P-521 base | **2,551** / **2,547** | 4,007 / 4,105 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | – | 40.2 / 43.4 | – | – | 7.8\* / 7.1\* | – | – |
| BabyBear | **33.3** / **33.1** | – | 52.3 / 52.4 | – | – | 43.5 / 41.9 | – | – |
| Goldilocks | 21.0 / 20.5 | – | 57.1 / 58.5 | – | – | 4.0\* / 6.0\* | **19.1** / **19.1** | – |
| BN254 scalar | **223** / **211** | 408 / 402 | 967 / 950 | – | 784 / 794 | – | – | 1,213 / 1,211 |
| secp256k1 base | **27.7** / **38.8** | 228 / 221 | 965 / 949 | 15.8\* / 5.4\* | 508 / 487 | – | – | 1,188 / 1,190 |
| P-256 base | **178** / **175** | 409 / 401 | 968 / 948 | 498 / 502 | 676 / 701 | – | – | 998 / 1,010 |
| P-256 scalar | 199 / 192 | 408 / 404 | 943 / 945 | **54.7** / **62.9** | 837 / 821 | – | – | 1,292 / 1,315 |
| Tom-256 base | **191** / **189** | – | 962 / 949 | – | 732 / 708 | – | – | 1,214 / 1,214 |
| BLS12-381 base | **308** / **297** | 1,086 / 1,080 | 1,383 / 1,370 | – | 1,827 / 1,714 | – | – | 2,462 / 2,443 |
| P-521 base | **87.1** / **85.7** | – | 2,170 / 2,264 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 39.9 / 42.0 | – | – | 7.8\* / 7.1\* | – |
| BabyBear | **33.3** / 33.0 | 51.8 / 51.4 | – | – | 34.5 / **31.3** | – |
| Goldilocks | 21.1 / 20.5 | 57.0 / 58.3 | – | – | 4.0\* / 6.0\* | **19.1** / **19.1** |
| BN254 scalar | 732 / 742 | 988 / 950 | – | **702** / **685** | – | – |
| secp256k1 base | **29.9** / **38.0** | 990 / 958 | – | 425 / 432 | – | – |
| P-256 base | **564** / **551** | 991 / 953 | – | 674 / 657 | – | – |
| P-256 scalar | 756 / 740 | 968 / 949 | **116** / **107** | 717 / 722 | – | – |
| Tom-256 base | 677 / 654 | 983 / 954 | – | **623** / **627** | – | – |
| BLS12-381 base | 2,053 / 2,185 | **1,379** / **1,368** | – | 1,699 / 1,733 | – | – |
| P-521 base | **116** / **109** | 2,206 / 2,296 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | – | 30.9 / 30.5 | – | – | 6.3\* / 6.1\* | – | – |
| BabyBear | **32.6** / **32.7** | – | 41.5 / 41.7 | – | – | 43.9 / 41.6 | – | – |
| Goldilocks | 20.4 / 19.9 | – | 45.2 / 48.2 | – | – | 4.3\* / 5.0\* | **19.0** / **18.1** | – |
| BN254 scalar | **204** / **206** | 401 / 399 | 952 / 960 | – | 783 / 762 | – | – | 1,161 / 1,163 |
| secp256k1 base | **18.5** / **24.5** | 231 / 221 | 941 / 946 | 14.0\* / 11.0\* | 491 / 482 | – | – | 1,144 / 1,136 |
| P-256 base | **177** / **171** | 406 / 394 | 933 / 961 | 486 / 492 | 685 / 687 | – | – | 910 / 937 |
| P-256 scalar | 188 / 183 | 404 / 401 | 948 / 978 | **24.3** / **24.5** | 806 / 812 | – | – | 1,244 / 1,272 |
| Tom-256 base | **187** / **183** | – | 933 / 970 | – | 727 / 705 | – | – | 1,160 / 1,162 |
| BLS12-381 base | **293** / **277** | 1,077 / 1,052 | 1,365 / 1,363 | – | 1,789 / 1,680 | – | – | 2,398 / 2,353 |
| P-521 base | **58.4** / **60.4** | – | 2,152 / 2,235 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | 30.7 / 30.5 | – | – | 6.3\* / 6.0\* | – |
| BabyBear | **32.6** / 32.7 | 42.2 / 42.3 | – | – | 34.7 / **31.1** | – |
| Goldilocks | 20.4 / 19.9 | 46.0 / 48.1 | – | – | 4.3\* / 5.0\* | **19.0** / **18.1** |
| BN254 scalar | 739 / 748 | 938 / 981 | – | **675** / **677** | – | – |
| secp256k1 base | **22.5** / **25.5** | 938 / 953 | – | 420 / 413 | – | – |
| P-256 base | **563** / **559** | 940 / 954 | – | 679 / 611 | – | – |
| P-256 scalar | 795 / 776 | 951 / 971 | **88.6** / **94.7** | 705 / 703 | – | – |
| Tom-256 base | 689 / 706 | 937 / 948 | – | **618** / **609** | – | – |
| BLS12-381 base | 2,033 / 2,132 | **1,352** / **1,376** | – | 1,784 / 1,761 | – | – |
| P-521 base | **89.3** / **77.5** | 2,183 / 2,285 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 5.2 / – | – | 23.6 / – | – | – | **3.9** / – | – | – |
| BabyBear | **32.5** / – | – | 34.9 / – | – | – | 41.4 / – | – | – |
| Goldilocks | 19.0 / – | – | 38.6 / – | – | – | 2.2\* / – | **17.0** / – | – |
| BN254 scalar | **178** / – | 372 / – | 881 / – | – | 707 / – | – | – | 1,079 / – |
| secp256k1 base | **3.8** / – | 222 / – | 886 / – | 5.9 / – | 438 / – | – | – | 1,078 / – |
| P-256 base | **150** / – | 392 / – | 895 / – | 465 / – | 647 / – | – | – | 839 / – |
| P-256 scalar | 132 / – | 379 / – | 891 / – | **4.0** / – | 747 / – | – | – | 1,087 / – |
| Tom-256 base | **165** / – | – | 894 / – | – | 659 / – | – | – | 1,078 / – |
| BLS12-381 base | **245** / – | 1,013 / – | 1,254 / – | – | 1,747 / – | – | – | 2,340 / – |
| P-521 base | **6.0** / – | – | 1,966 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **1,785** / **1,769** | 11,611 / 11,564 | 26,161 / 26,311 | 24,219 / 24,139 | – | 19,344 / 19,415 | 43,908 / 43,122 |
| secp256k1 base | **1,369** / **1,361** | 11,220 / 11,167 | 20,704 / 20,777 | 24,215 / 24,171 | 37,510 / 37,493 | 18,968 / 18,910 | 44,178 / 44,330 |
| P-256 base | **1,723** / **1,717** | 10,810 / 10,838 | 24,983 / 24,925 | 24,196 / 24,100 | 128,167 / 126,429 | 18,857 / 18,832 | 43,873 / 43,791 |
| P-256 scalar | **1,761** / **1,728** | 11,928 / 11,794 | 25,914 / 26,015 | 24,176 / 24,019 | 49,075 / 48,834 | 19,288 / 19,473 | 44,101 / 44,853 |
| Tom-256 base | **1,750** / **1,750** | 11,547 / 11,588 | – | 24,175 / 23,958 | – | 19,109 / 19,130 | 43,934 / 43,873 |
| BLS12-381 base | **2,823** / **2,798** | 24,839 / 24,550 | 50,433 / 50,498 | 43,801 / 43,773 | – | 32,296 / 32,422 | 101,989 / 101,875 |
| P-521 base | **3,706** / **3,701** | 46,575 / 46,111 | – | 77,937 / 77,987 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **281** / 286 | 289 / **284** | – | 2,232 / 2,223 | – | – | 739 / 720 | – | 2,082 / 2,092 |
| BabyBear | **304** / **310** | 342 / 343 | – | 2,213 / 2,221 | – | – | 726 / 711 | – | 2,174 / 2,181 |
| Goldilocks | **601** / **598** | 637 / 630 | – | 4,316 / 4,304 | – | – | 1,532 / 1,515 | 1,896 / 1,893 | 4,139 / 4,129 |
| BN254 scalar | **13,086** / **13,014** | 13,247 / 13,098 | 26,196 / 26,137 | 25,814 / 25,868 | – | 22,942 / 23,018 | – | – | 54,206 / 53,758 |
| secp256k1 base | 12,401 / **12,217** | **12,263** / 12,271 | 24,677 / 24,796 | 25,813 / 25,780 | 37,604 / 37,551 | 22,560 / 22,731 | – | – | 54,280 / 54,558 |
| P-256 base | 12,282 / **12,143** | **12,191** / 12,295 | 25,436 / 25,634 | 25,822 / 25,793 | 128,506 / 128,952 | 22,693 / 22,780 | – | – | 54,077 / 53,931 |
| P-256 scalar | **13,435** / **13,354** | 13,489 / 13,503 | 26,872 / 26,965 | 25,911 / 25,810 | 64,290 / 64,224 | 23,164 / 23,068 | – | – | 54,334 / 54,379 |
| Tom-256 base | 13,057 / 13,077 | **13,036** / **12,964** | – | 25,897 / 25,908 | – | 22,913 / 22,894 | – | – | 54,112 / 54,124 |
| BLS12-381 base | 28,681 / 28,552 | **27,813** / **27,703** | 52,241 / 52,044 | 47,147 / 47,026 | – | 40,500 / 40,309 | – | – | 130,468 / 130,388 |
| P-521 base | 52,848 / 52,573 | **52,796** / **52,203** | – | 84,308 / 84,084 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | 10.8 / 19.8 | – | – | – | 10.8 / 18.3 |
| BabyBear | **3.0** / **3.0** | 10.8 / 19.4 | – | – | – | 10.8 / 18.3 |
| Goldilocks | **3.1** / **3.0** | 12.1 / 19.0 | – | – | 4.0 / 4.4 | 10.8 / 18.3 |
| BN254 scalar | **30.2** / **38.7** | 59.7 / 90.0 | – | 39.5 / 44.0 | – | 76.5 / 116 |
| secp256k1 base | **31.5** / 39.3 | 59.6 / 90.3 | 16.3\* / 15.7\* | 35.2 / **38.3** | – | 76.6 / 116 |
| P-256 base | **28.0** / **33.1** | 59.5 / 89.9 | 33.9 / 39.6 | 33.9 / 39.2 | – | 76.4 / 120 |
| P-256 scalar | **33.1** / **41.1** | 59.5 / 90.0 | 56.4 / 60.6 | 38.2 / 42.5 | – | 76.4 / 120 |
| Tom-256 base | **35.8** / **39.8** | 59.4 / 89.9 | – | 38.5 / 42.6 | – | 76.8 / 115 |
| BLS12-381 base | **53.4** / **65.2** | 91.0 / 138 | – | 69.9 / 67.3 | – | 125 / 166 |
| P-521 base | **88.0** / **88.5** | 171 / 245 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 17.2 / 14.4 |
| BabyBear | **2.7** / **3.0** | 5.0 / 8.0 | – | – | – | – | 16.6 / 14.5 |
| Goldilocks | **3.0** / **3.0** | 5.0 / 8.0 | – | – | – | **3.0** / **3.0** | 18.7 / 13.2 |
| BN254 scalar | 24.4 / 32.0 | 34.4 / 30.1 | – | **23.2** / **26.1** | – | – | 54.6 / 78.0 |
| secp256k1 base | 21.0 / 25.9 | 34.6 / 32.8 | 16.1\* / 18.5\* | **20.3** / **24.6** | – | – | 54.8 / 77.2 |
| P-256 base | 22.2 / **23.6** | 38.2 / 29.1 | 21.7 / 24.6 | **21.1** / 25.0 | – | – | 54.3 / 78.0 |
| P-256 scalar | **22.4** / **24.4** | 39.3 / 33.7 | 26.5 / 27.0 | 23.2 / 27.9 | – | – | 54.7 / 78.4 |
| Tom-256 base | 23.6 / **25.3** | 31.8 / 30.1 | – | **23.5** / 26.6 | – | – | 54.7 / 78.2 |
| BLS12-381 base | **42.0** / **43.3** | 55.6 / 110 | – | 44.5 / 44.1 | – | – | 67.9 / 103 |
| P-521 base | **63.6** / **62.6** | 124 / 214 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | 38.9 / 50.0 | – | – | – | 49.8 / 56.0 |
| BabyBear | **15.1** / **22.0** | 38.7 / 50.6 | – | – | – | 49.9 / 56.1 |
| Goldilocks | 18.1 / 22.8 | 37.8 / 51.3 | – | – | **17.1** / **19.0** | 49.8 / 56.0 |
| BN254 scalar | **612** / **611** | 888 / 899 | – | 702 / 693 | – | 1,058 / 1,076 |
| secp256k1 base | **403** / **405** | 882 / 905 | 171\* / 171\* | 600 / 588 | – | 1,072 / 1,085 |
| P-256 base | **459** / **452** | 886 / 902 | 486 / 472 | 490 / 482 | – | 865 / 873 |
| P-256 scalar | **611** / **630** | 889 / 906 | 825 / 822 | 684 / 674 | – | 1,072 / 1,079 |
| Tom-256 base | **573** / **569** | 880 / 900 | – | 603 / 590 | – | 1,071 / 1,084 |
| BLS12-381 base | 1,884 / 1,924 | **1,291** / **1,309** | – | 1,723 / 1,569 | – | 2,166 / 2,179 |
| P-521 base | **1,791** / **1,776** | 2,718 / 2,773 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square, constant time: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | 38.5 / 52.9 | – | – | – | 49.1 / 56.0 |
| BabyBear | **16.9** / **28.0** | 38.5 / 51.2 | – | – | – | 49.0 / 56.0 |
| Goldilocks | 18.1 / 22.9 | 38.6 / 52.0 | – | – | **17.1** / **20.0** | 49.1 / 56.0 |
| BN254 scalar | **522** / **548** | 635 / 650 | – | 533 / 551 | – | 1,059 / 1,066 |
| secp256k1 base | **333** / **359** | 631 / 642 | 115\* / 136\* | 464 / 442 | – | 1,068 / 1,075 |
| P-256 base | 365 / 354 | 636 / 616 | 473 / 454 | **336** / **346** | – | 877 / 736 |
| P-256 scalar | **521** / **536** | 643 / 640 | 815 / 810 | 552 / 570 | – | 1,071 / 1,080 |
| Tom-256 base | 480 / 477 | 639 / 632 | – | **475** / **453** | – | 1,072 / 1,078 |
| BLS12-381 base | 2,004 / 1,811 | 1,348 / 1,337 | – | **1,283** / **1,233** | – | 2,172 / 2,172 |
| P-521 base | **956** / **947** | 2,792 / 2,846 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert, constant time: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **428** / – | 6,855 / – | – | – | – | 3,330 / – |
| BabyBear | **1,369** / – | 6,831 / – | – | – | – | 3,334 / – |
| Goldilocks | 2,040 / – | 6,872 / – | – | – | **1,831** / – | 7,235 / – |
| BN254 scalar | **16,879** / – | 36,107 / – | – | 320,030 / – | – | 404,711 / – |
| secp256k1 base | **16,887** / – | 36,116 / – | 37,261 / – | 272,492 / – | – | 541,144 / – |
| P-256 base | **16,377** / – | 36,309 / – | 127,319 / – | 213,901 / – | – | 329,923 / – |
| P-256 scalar | **16,977** / – | 36,055 / – | 351,653 / – | 326,147 / – | – | 454,955 / – |
| Tom-256 base | **17,013** / – | 36,099 / – | – | 287,446 / – | – | 379,687 / – |
| BLS12-381 base | **34,150** / – | 66,176 / – | – | 1,214,581 / – | – | 1,324,185 / – |
| P-521 base | **67,355** / – | 123,109 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **340** / – | 3,696 / – | – | – | 1,746 / – |
| BabyBear | **16,493** / – | 38,401 / – | – | – | – |
| Goldilocks | **25,990** / – | 54,516 / – | – | – | – |
| BN254 scalar | **451,765** / – | 584,124 / – | – | 523,801 / – | – |
| secp256k1 base | **99,693** / – | 247,849 / – | 36,947\* / – | 280,311 / – | 537,243 / – |
| P-256 base | **96,884** / – | 197,378 / – | 121,856 / – | 210,535 / – | 238,587 / – |
| P-256 scalar | **199,959** / – | 246,406 / – | 363,290 / – | 274,599 / – | – |
| Tom-256 base | **153,023** / – | 228,901 / – | – | 287,509 / – | 376,664 / – |
| BLS12-381 base | 938,429 / – | **616,249** / – | – | 1,184,280 / – | 1,321,204 / – |
| P-521 base | **541,153** / – | 1,563,360 / – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 72.0 / 73.2 | – | – | – |
| BabyBear | **47.1** / **47.9** | 83.4 / 86.5 | – | – | – |
| Goldilocks | 36.3 / 35.9 | 94.5 / 89.8 | – | – | **31.6** / **32.1** |
| BN254 scalar | **201** / **199** | 1,738 / 1,745 | – | 1,419 / 1,595 | – |
| secp256k1 base | **94.0** / **101** | 1,754 / 1,739 | 167 / 176 | 1,009 / 1,019 | – |
| P-256 base | **189** / **193** | 1,762 / 1,734 | 905 / 903 | 1,289 / 1,159 | – |
| P-256 scalar | **204** / **204** | 1,736 / 1,765 | 821 / 814 | 1,464 / 1,589 | – |
| Tom-256 base | **190** / **195** | 1,751 / 1,732 | – | 1,260 / 1,291 | – |
| BLS12-381 base | **284** / **287** | 2,455 / 2,468 | – | 3,714 / 3,725 | – |
| P-521 base | **237** / **243** | 4,002 / 4,097 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 72.6 / 71.0 | – | – | – |
| BabyBear | **47.2** / **47.6** | 85.9 / 83.3 | – | – | – |
| Goldilocks | 36.3 / 35.9 | 94.4 / 89.6 | – | – | **31.6** / **32.3** |
| BN254 scalar | **946** / **959** | 1,808 / 1,730 | – | 1,363 / 1,342 | – |
| secp256k1 base | **541** / **535** | 1,790 / 1,747 | – | 979 / 967 | – |
| P-256 base | **871** / **868** | 1,786 / 1,742 | – | 1,222 / 926 | – |
| P-256 scalar | 930 / 931 | 1,797 / 1,733 | **878** / **863** | 1,359 / 1,380 | – |
| Tom-256 base | **847** / **845** | 1,761 / 1,671 | – | 1,185 / 1,195 | – |
| BLS12-381 base | **2,045** / **2,022** | 2,505 / 2,484 | – | 3,713 / 3,661 | – |
| P-521 base | **3,388** / **3,384** | 4,007 / 4,105 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.2** | 40.2 / 43.4 | – | – | – |
| BabyBear | **33.4** / **33.1** | 52.3 / 52.4 | – | – | – |
| Goldilocks | 21.0 / 20.5 | 57.1 / 58.5 | – | – | **19.1** / **19.1** |
| BN254 scalar | **253** / **233** | 967 / 950 | – | 784 / 794 | – |
| secp256k1 base | **27.7** / **38.5** | 965 / 949 | 15.8\* / 5.4\* | 508 / 487 | – |
| P-256 base | **215** / **207** | 968 / 948 | 498 / 502 | 676 / 701 | – |
| P-256 scalar | 226 / 233 | 943 / 945 | **54.7** / **62.9** | 837 / 821 | – |
| Tom-256 base | **221** / **209** | 962 / 949 | – | 732 / 708 | – |
| BLS12-381 base | **350** / **350** | 1,383 / 1,370 | – | 1,827 / 1,714 | – |
| P-521 base | **87.1** / **87.9** | 2,170 / 2,264 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 39.9 / 42.0 | – | – | – |
| BabyBear | **33.4** / **33.1** | 51.8 / 51.4 | – | – | – |
| Goldilocks | 21.0 / 20.5 | 57.0 / 58.3 | – | – | **19.1** / **19.1** |
| BN254 scalar | 803 / 812 | 988 / 950 | – | **702** / **685** | – |
| secp256k1 base | **59.3** / **57.6** | 990 / 958 | – | 425 / 432 | – |
| P-256 base | **623** / **593** | 991 / 953 | – | 674 / 657 | – |
| P-256 scalar | 797 / 765 | 968 / 949 | **116** / **107** | 717 / 722 | – |
| Tom-256 base | 726 / 696 | 983 / 954 | – | **623** / **627** | – |
| BLS12-381 base | 2,171 / 2,258 | **1,379** / **1,368** | – | 1,699 / 1,733 | – |
| P-521 base | **165** / **163** | 2,206 / 2,296 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.5** | 30.9 / 30.5 | – | – | – |
| BabyBear | **33.3** / **32.7** | 41.5 / 41.7 | – | – | – |
| Goldilocks | 20.4 / 19.9 | 45.2 / 48.2 | – | – | **19.0** / **18.1** |
| BN254 scalar | **232** / **237** | 952 / 960 | – | 783 / 762 | – |
| secp256k1 base | **18.8** / **24.7** | 941 / 946 | 14.0\* / 11.0\* | 491 / 482 | – |
| P-256 base | **198** / **199** | 933 / 961 | 486 / 492 | 685 / 687 | – |
| P-256 scalar | 235 / 227 | 948 / 978 | **24.3** / **24.5** | 806 / 812 | – |
| Tom-256 base | **220** / **212** | 933 / 970 | – | 727 / 705 | – |
| BLS12-381 base | **349** / **348** | 1,365 / 1,363 | – | 1,789 / 1,680 | – |
| P-521 base | **58.4** / **61.3** | 2,152 / 2,235 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.5** | 30.7 / 30.5 | – | – | – |
| BabyBear | **33.3** / **32.7** | 42.2 / 42.3 | – | – | – |
| Goldilocks | 20.4 / 19.9 | 46.0 / 48.1 | – | – | **19.0** / **18.1** |
| BN254 scalar | 862 / 861 | 938 / 981 | – | **675** / **677** | – |
| secp256k1 base | **53.0** / **51.2** | 938 / 953 | – | 420 / 413 | – |
| P-256 base | **589** / **593** | 940 / 954 | – | 679 / 611 | – |
| P-256 scalar | 797 / 797 | 951 / 971 | **88.6** / **94.7** | 705 / 703 | – |
| Tom-256 base | 727 / 717 | 937 / 948 | – | **618** / **609** | – |
| BLS12-381 base | 2,471 / 2,479 | **1,352** / **1,376** | – | 1,784 / 1,761 | – |
| P-521 base | **158** / **158** | 2,183 / 2,285 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / – | 23.6 / – | – | – | – |
| BabyBear | **23.4** / – | 34.9 / – | – | – | – |
| Goldilocks | 30.8 / – | 38.6 / – | – | – | **17.0** / – |
| BN254 scalar | **510** / – | 881 / – | – | 707 / – | – |
| secp256k1 base | **3.4** / – | 886 / – | 5.9 / – | 438 / – | – |
| P-256 base | **290** / – | 895 / – | 465 / – | 647 / – | – |
| P-256 scalar | 465 / – | 891 / – | **4.0** / – | 747 / – | – |
| Tom-256 base | **406** / – | 894 / – | – | 659 / – | – |
| BLS12-381 base | **1,035** / – | 1,254 / – | – | 1,747 / – | – |
| P-521 base | **6.0** / – | 1,966 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **17,702** / **17,520** | 17,805 / 17,808 | 38,425 / 38,369 | – | 327,150 / 326,491 | 405,880 / 406,606 |
| secp256k1 base | **17,195** / **17,272** | 17,349 / 17,336 | 38,428 / 38,344 | 37,510 / 37,493 | 276,377 / 270,784 | 537,379 / 539,241 |
| P-256 base | **17,007** / **16,822** | 17,247 / 17,122 | 38,401 / 38,421 | 128,167 / 126,429 | 218,675 / 218,241 | 321,377 / 317,478 |
| P-256 scalar | 17,941 / **17,880** | **17,901** / 18,058 | 38,462 / 38,240 | 349,895 / 351,801 | 332,451 / 330,074 | 457,527 / 457,683 |
| Tom-256 base | 17,880 / **17,742** | **17,800** / 17,850 | 38,468 / 38,115 | – | 283,513 / 282,119 | 379,422 / 380,666 |
| BLS12-381 base | **35,954** / 36,232 | 36,822 / **36,055** | 68,669 / 68,482 | – | 1,229,687 / 1,222,671 | 1,329,792 / 1,328,306 |
| P-521 base | **66,648** / 67,598 | 67,766 / **67,031** | 127,557 / 127,290 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **462** / **459** | 464 / 461 | 7,007 / 6,991 | – | – | – | 3,419 / 3,421 |
| BabyBear | **1,484** / **1,463** | 1,493 / 1,489 | 7,018 / 6,980 | – | – | – | 3,417 / 3,420 |
| Goldilocks | 2,061 / 2,049 | 2,060 / 2,059 | 7,073 / 6,954 | – | – | **1,896** / **1,893** | 7,113 / 7,114 |
| BN254 scalar | 18,407 / **18,181** | **18,182** / 18,304 | 38,290 / 38,038 | – | 321,457 / 324,052 | – | 405,136 / 404,716 |
| secp256k1 base | 17,192 / **17,070** | **17,173** / 17,272 | 38,457 / 38,206 | 37,604 / 37,551 | 272,519 / 276,681 | – | 541,159 / 540,362 |
| P-256 base | **17,319** / 17,348 | 17,507 / **17,264** | 38,434 / 38,330 | 128,506 / 128,952 | 217,295 / 215,352 | – | 325,921 / 322,143 |
| P-256 scalar | **18,426** / **18,273** | 18,583 / 18,615 | 38,275 / 38,239 | 354,829 / 354,008 | 325,883 / 328,472 | – | 455,891 / 455,970 |
| Tom-256 base | **18,159** / **18,074** | 18,380 / 18,392 | 38,362 / 38,235 | – | 285,330 / 284,278 | – | 379,991 / 378,311 |
| BLS12-381 base | **38,027** / **38,207** | 38,840 / 38,513 | 68,724 / 68,534 | – | 1,223,296 / 1,224,313 | – | 1,330,201 / 1,327,505 |
| P-521 base | 67,769 / **67,182** | **67,593** / 67,494 | 127,388 / 127,101 | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | – | 11.4 / 19.4 | – | – | 4.4\* / 7.8 | – | 10.8 / 17.7 | – |
| BabyBear | 3.0 / **3.0** | – | 11.4 / 20.7 | – | – | **2.7** / **3.0** | – | 10.8 / 17.7 | – |
| Goldilocks | **3.1** / **3.0** | – | 10.9 / 18.7 | – | – | 6.7 / 8.1 | 4.0 / 4.7 | 10.8 / 17.8 | – |
| BN254 scalar | 27.0 / 25.6 | **17.9** / **16.5** | 60.1 / 93.0 | – | 35.0 / 34.0 | – | – | 76.4 / 115 | – |
| secp256k1 base | 29.0 / 29.8 | **19.5** / **17.9** | 60.2 / 92.5 | 16.1\* / 14.5\* | 33.0 / 31.0 | – | – | 76.8 / 115 | – |
| P-256 base | 28.0 / 28.1 | **19.8** / **18.9** | 60.3 / 93.1 | 32.0 / 29.4 | 32.0 / 30.0 | – | – | 76.9 / 119 | 538 / – |
| P-256 scalar | 29.5 / 30.1 | **20.0** / **18.5** | 60.1 / 92.6 | 56.7 / 60.9 | 35.0 / 33.0 | – | – | 76.4 / 120 | 554 / – |
| Tom-256 base | **29.0** / **29.1** | – | 59.5 / 89.7 | – | 34.0 / 33.0 | – | – | 76.1 / 115 | – |
| BLS12-381 base | **41.2** / **41.0** | 42.7 / 44.5 | 83.4 / 137 | – | 54.1 / 54.1 | – | – | 126 / 166 | – |
| P-521 base | **74.1** / **74.1** | – | 171 / 244 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 16.8 / 14.7 | – |
| BabyBear | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / **3.0** | – | 17.3 / 13.8 | – |
| Goldilocks | **3.0** / **3.0** | – | 5.0 / 8.0 | – | – | 4.3 / 5.0 | **3.0** / **3.0** | 18.7 / 12.8 | – |
| BN254 scalar | 26.0 / 25.0 | 24.1 / **21.9** | 37.0 / 30.7 | – | **24.0** / 22.0 | – | – | 54.5 / 75.5 | – |
| secp256k1 base | 22.5 / 22.0 | 24.4 / 21.9 | 37.2 / 28.6 | 16.4\* / 17.6\* | **22.0** / **21.0** | – | – | 54.5 / 75.4 | – |
| P-256 base | **22.0** / **20.0** | 22.9 / 21.3 | 37.0 / 27.5 | **22.0** / **20.0** | **22.0** / **20.0** | – | – | 54.6 / 76.3 | 289 / – |
| P-256 scalar | 23.5 / **21.5** | 23.5 / 21.9 | 37.2 / 30.5 | 26.6 / 26.8 | **23.0** / **21.5** | – | – | 55.1 / 76.9 | 286 / – |
| Tom-256 base | **24.0** / 22.5 | – | 37.2 / 30.6 | – | **24.0** / **22.0** | – | – | 54.9 / 75.8 | – |
| BLS12-381 base | **37.0** / **36.1** | 39.8 / 37.5 | 57.3 / 106 | – | 37.1 / 36.2 | – | – | 68.8 / 103 | – |
| P-521 base | **53.0** / **53.0** | – | 128 / 175 | – | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | – | 19.2 / 30.7 | – | – | 5.5\* / 12.0 | – | 49.9 / 57.1 | – |
| BabyBear | **5.0** / **11.0** | – | 19.2 / 30.5 | – | – | 6.5 / 13.0 | – | 50.2 / 57.0 | – |
| Goldilocks | 9.0 / **14.0** | – | 19.5 / 30.1 | – | – | 13.7 / 19.1 | **8.1** / **14.0** | 49.9 / 57.2 | – |
| BN254 scalar | **162** / **152** | 274 / 273 | 204 / 211 | – | 182 / 178 | – | – | 1,065 / 1,083 | – |
| secp256k1 base | **112** / **113** | 258 / 257 | 205 / 215 | 176 / 175 | 164 / 159 | – | – | 1,078 / 1,088 | – |
| P-256 base | **127** / **129** | 236 / 236 | 202 / 211 | 135 / 131 | 135 / 131 | – | – | 883 / 871 | 1,271 / – |
| P-256 scalar | 183 / **173** | 293 / 297 | 203 / 212 | 341 / 336 | **181** / 177 | – | – | 1,076 / 1,084 | 1,265 / – |
| Tom-256 base | **162** / **165** | – | 204 / 211 | – | 176 / 172 | – | – | 1,076 / 1,087 | – |
| BLS12-381 base | 641 / 624 | 780 / 778 | 529 / 533 | – | **383** / **378** | – | – | 2,170 / 2,186 | – |
| P-521 base | **886** / **849** | – | 1,054 / 1,101 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### square: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | – | 18.0 / 31.3 | – | – | 5.5 / 12.0 | – | 49.0 / 56.0 | – |
| BabyBear | 5.9 / **11.0** | – | 17.9 / 31.2 | – | – | **5.8** / 13.0 | – | 49.1 / 56.0 | – |
| Goldilocks | 10.0 / 14.4 | – | 18.3 / 28.9 | – | – | 11.0 / 19.0 | **7.0** / **14.3** | 49.2 / 56.0 | – |
| BN254 scalar | 161 / **142** | 236 / 235 | 175 / 176 | – | **145** / 147 | – | – | 1,066 / 1,064 | – |
| secp256k1 base | **108** / **82.8** | 218 / 214 | 174 / 177 | 115 / 136 | 128 / 119 | – | – | 1,078 / 1,074 | – |
| P-256 base | 129 / 113 | 191 / 189 | 175 / 176 | 128 / 130 | **122** / **109** | – | – | 881 / 772 | 1,283 / – |
| P-256 scalar | 165 / 142 | 259 / 260 | 176 / 176 | 336 / 340 | **149** / **138** | – | – | 1,078 / 1,067 | 1,285 / – |
| Tom-256 base | 159 / 135 | – | 175 / 177 | – | **133** / **130** | – | – | 1,079 / 1,074 | – |
| BLS12-381 base | 423 / 410 | 571 / 625 | 564 / 529 | – | **330** / **295** | – | – | 2,168 / 2,163 | – |
| P-521 base | **420** / **422** | – | 1,115 / 1,137 | – | – | – | – | – | – |

### invert: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **277** / – | – | 1,971 / – | – | – | 727 / – | – | 2,002 / – | – |
| BabyBear | **272** / – | – | 1,916 / – | – | – | 722 / – | – | 2,088 / – | – |
| Goldilocks | **571** / – | – | 3,839 / – | – | – | 1,557 / – | 998 / – | 4,035 / – | – |
| BN254 scalar | **7,897** / – | 26,691 / – | 18,912 / – | – | 14,105 / – | – | – | 52,240 / – | – |
| secp256k1 base | **7,964** / – | 25,459 / – | 18,998 / – | 37,393 / – | 14,112 / – | – | – | 52,264 / – | – |
| P-256 base | **7,797** / – | 26,385 / – | 19,024 / – | 34,390 / – | 13,958 / – | – | – | 51,493 / – | 274,169 / – |
| P-256 scalar | **8,181** / – | 27,698 / – | 19,056 / – | 63,381 / – | 14,105 / – | – | – | 52,161 / – | 273,615 / – |
| Tom-256 base | **8,070** / – | – | 19,080 / – | – | 14,045 / – | – | – | 52,533 / – | – |
| BLS12-381 base | **15,894** / – | 52,618 / – | 33,664 / – | – | 24,084 / – | – | – | 124,928 / – | – |
| P-521 base | **29,745** / – | – | 62,451 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 330 / – | – | 2,089 / – | – | – | **329** / – | 1,751 / – | – |
| BabyBear | **2,091** / – | – | 23,037 / – | – | – | 2,998 / – | – | – |
| Goldilocks | **9,387** / – | – | 33,003 / – | – | – | 6,334\* / – | – | – |
| BN254 scalar | **80,004** / – | 147,875 / – | 198,648 / – | – | 179,695 / – | – | – | – |
| secp256k1 base | **30,933** / – | 130,012 / – | 79,874 / – | 37,257 / – | 87,940 / – | – | 533,849 / – | – |
| P-256 base | **32,542** / – | 55,507 / – | 67,723 / – | 34,049 / – | 70,505 / – | – | 256,152 / – | 374,441 / – |
| P-256 scalar | **64,716** / – | 118,443 / – | 79,186 / – | 154,146 / – | 72,293 / – | – | – | – |
| Tom-256 base | **56,511** / – | – | 74,858 / – | – | 93,684 / – | – | 374,334 / – | – |
| BLS12-381 base | 293,456 / – | 437,060 / – | **269,109** / – | – | 375,555 / – | – | 1,323,049 / – | – |
| P-521 base | **266,997** / – | – | 594,739 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | – | 38.7 / 39.8 | – | – | 9.2\* / 12.8\* | – | – |
| BabyBear | **14.7** / **13.1** | – | 40.2 / 40.8 | – | – | 17.7 / 16.7 | – | – |
| Goldilocks | 17.1 / 17.1 | – | 49.9 / 49.6 | – | – | 13.9\* / 19.0 | **13.6** / **14.4** | – |
| BN254 scalar | **44.1** / **45.3** | 658 / 632 | 407 / 409 | – | 582 / 549 | – | – | 2,153 / 2,149 |
| secp256k1 base | **23.0** / **23.0** | 431 / 429 | 406 / 412 | 168 / 176 | 411 / 409 | – | – | 2,149 / 2,158 |
| P-256 base | **41.2** / **41.9** | 561 / 544 | 406 / 408 | 258 / 258 | 347 / 351 | – | – | 1,421 / 1,428 |
| P-256 scalar | **47.9** / **47.7** | 698 / 718 | 408 / 410 | 339 / 341 | 447 / 488 | – | – | 2,182 / 2,174 |
| Tom-256 base | **44.3** / **45.1** | – | 407 / 409 | – | 519 / 554 | – | – | 2,159 / 2,161 |
| BLS12-381 base | **61.8** / **62.4** | 1,671 / 1,653 | 957 / 1,017 | – | 1,176 / 1,198 | – | – | 4,407 / 4,394 |
| P-521 base | **49.3** / **52.4** | – | 1,841 / 1,858 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 38.9 / 38.8 | – | – | 9.2\* / 12.8\* | – |
| BabyBear | **14.7** / **13.1** | 40.3 / 40.7 | – | – | 17.7 / 16.6 | – |
| Goldilocks | 17.1 / 17.1 | 49.8 / 50.6 | – | – | 13.9\* / 19.0 | **13.5** / **15.1** |
| BN254 scalar | **260** / **258** | 410 / 407 | – | 349 / 354 | – | – |
| secp256k1 base | **165** / **165** | 409 / 407 | – | 267 / 268 | – | – |
| P-256 base | **236** / **235** | 408 / 409 | – | 253 / 252 | – | – |
| P-256 scalar | **265** / **264** | 408 / 414 | 486 / 453 | 385 / 347 | – | – |
| Tom-256 base | **254** / **256** | 408 / 408 | – | 336 / 336 | – | – |
| BLS12-381 base | **552** / **554** | 1,022 / 1,040 | – | 1,008 / 1,021 | – | – |
| P-521 base | **1,079** / **1,005** | 1,880 / 1,860 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | – | 30.8 / 30.4 | – | – | 7.8\* / 7.2\* | – | – |
| BabyBear | **12.5** / **11.0** | – | 32.4 / 31.9 | – | – | 15.0 / 13.1 | – | – |
| Goldilocks | 12.0 / 10.0 | – | 36.5 / 33.5 | – | – | 6.7\* / 8.0\* | **11.0** / **9.7** | – |
| BN254 scalar | **69.0** / **66.5** | 355 / 317 | 313 / 310 | – | 322 / 298 | – | – | 1,213 / 1,208 |
| secp256k1 base | **25.0** / **29.0** | 184 / 183 | 311 / 307 | 13.1\* / 5.3\* | 304 / 245 | – | – | 1,182 / 1,193 |
| P-256 base | **63.3** / **61.2** | 279 / 273 | 318 / 309 | 159 / 158 | 273 / 249 | – | – | 958 / 964 |
| P-256 scalar | 76.6 / 67.5 | 345 / 352 | 311 / 311 | **55.1** / **63.1** | 321 / 298 | – | – | 1,225 / 1,226 |
| Tom-256 base | **68.1** / **67.1** | – | 321 / 311 | – | 375 / 296 | – | – | 1,215 / 1,212 |
| BLS12-381 base | **104** / **98.0** | 844 / 839 | 650 / 648 | – | 682 / 620 | – | – | 2,408 / 2,389 |
| P-521 base | **64.2** / **64.4** | – | 1,143 / 1,160 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 30.4 / 30.2 | – | – | 7.8\* / 7.2\* | – |
| BabyBear | **12.5** / **11.0** | 31.8 / 31.9 | – | – | 15.0 / 13.1 | – |
| Goldilocks | 12.0 / 10.0 | 36.5 / 33.6 | – | – | 6.7\* / 8.1\* | **11.0** / **9.7** |
| BN254 scalar | 206 / **199** | 314 / 312 | – | **205** / 205 | – | – |
| secp256k1 base | **31.0** / **30.2** | 317 / 313 | – | 140 / 141 | – | – |
| P-256 base | 157 / 155 | 315 / 307 | – | **154** / **153** | – | – |
| P-256 scalar | 204 / 202 | 313 / 311 | **117** / **108** | 204 / 205 | – | – |
| Tom-256 base | **185** / **194** | 312 / 314 | – | 197 / 196 | – | – |
| BLS12-381 base | 804 / 790 | 651 / 648 | – | **534** / **427** | – | – |
| P-521 base | **91.3** / **86.1** | 1,213 / 1,159 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | – | 20.9 / 20.3 | – | – | 6.3\* / 6.1\* | – | – |
| BabyBear | **12.5** / **10.8** | – | 21.8 / 21.3 | – | – | 15.1 / 13.5 | – | – |
| Goldilocks | 11.0 / 9.4 | – | 28.4 / 24.0 | – | – | 4.0\* / 5.0\* | **9.5** / **8.7** | – |
| BN254 scalar | **67.5** / **66.1** | 360 / 342 | 255 / 286 | – | 358 / 290 | – | – | 1,158 / 1,167 |
| secp256k1 base | **18.5** / **17.4** | 193 / 184 | 256 / 287 | 14.0\* / 11.0\* | 263 / 240 | – | – | 1,135 / 1,134 |
| P-256 base | **58.2** / **55.5** | 279 / 276 | 228 / 287 | 149 / 149 | 265 / 243 | – | – | 898 / 883 |
| P-256 scalar | 63.9 / 62.1 | 396 / 339 | 259 / 290 | **24.3** / **24.4** | 315 / 289 | – | – | 1,171 / 1,180 |
| Tom-256 base | **63.6** / **62.6** | – | 283 / 287 | – | 312 / 288 | – | – | 1,156 / 1,168 |
| BLS12-381 base | **97.1** / **94.0** | 841 / 852 | 618 / 625 | – | 588 / 589 | – | – | 2,329 / 2,316 |
| P-521 base | **44.1** / **46.3** | – | 1,136 / 1,120 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | 21.0 / 20.2 | – | – | 6.3\* / 6.1\* | – |
| BabyBear | **12.5** / **10.9** | 21.8 / 21.3 | – | – | 15.1 / 13.4 | – |
| Goldilocks | 11.0 / 9.4 | 28.3 / 24.1 | – | – | 4.0\* / 5.0\* | **9.5** / **8.6** |
| BN254 scalar | 208 / 199 | 259 / 283 | – | **194** / **194** | – | – |
| secp256k1 base | **24.5** / **22.0** | 256 / 282 | – | 130 / 130 | – | – |
| P-256 base | 149 / 151 | 231 / 285 | – | **143** / **143** | – | – |
| P-256 scalar | 200 / 197 | 280 / 285 | **89.9** / **95.6** | 194 / 194 | – | – |
| Tom-256 base | 193 / 194 | 259 / 283 | – | **188** / **188** | – | – |
| BLS12-381 base | 799 / 792 | 654 / 635 | – | **408** / **408** | – | – |
| P-521 base | **70.8** / **65.3** | 1,166 / 1,127 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 5.2 / – | – | 13.2 / – | – | – | **3.9** / – | – | – |
| BabyBear | **10.3** / – | – | 15.0 / – | – | – | 11.8 / – | – | – |
| Goldilocks | 8.7 / – | – | 17.0 / – | – | – | 2.2\* / – | **7.1** / – | – |
| BN254 scalar | **46.0** / – | 301 / – | 205 / – | – | 265 / – | – | – | 1,101 / – |
| secp256k1 base | **3.8** / – | 181 / – | 205 / – | 6.0 / – | 233 / – | – | – | 1,099 / – |
| P-256 base | **39.1** / – | 257 / – | 204 / – | 130 / – | 218 / – | – | – | 818 / – |
| P-256 scalar | 43.0 / – | 317 / – | 204 / – | **4.0** / – | 266 / – | – | – | 1,085 / – |
| Tom-256 base | **43.4** / – | – | 204 / – | – | 265 / – | – | – | 1,115 / – |
| BLS12-381 base | **64.0** / – | 781 / – | 477 / – | – | 569 / – | – | – | 2,227 / – |
| P-521 base | **6.0** / – | – | 850 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **998** / **994** | 7,210 / 7,178 | 28,465 / 27,445 | 18,007 / 17,807 | – | 11,796 / 11,746 | 44,549 / 44,149 |
| secp256k1 base | **888** / **891** | 7,070 / 7,039 | 21,380 / 21,806 | 17,924 / 17,944 | 37,528 / 37,530 | 11,640 / 11,566 | 44,384 / 44,844 |
| P-256 base | **945** / **945** | 6,789 / 6,781 | 26,853 / 26,696 | 17,900 / 17,917 | 34,678 / 34,624 | 11,564 / 11,569 | 44,745 / 43,133 |
| P-256 scalar | **1,014** / **1,009** | 7,390 / 7,353 | 28,195 / 27,088 | 17,901 / 17,882 | 48,435 / 48,457 | 11,745 / 11,809 | 45,008 / 44,414 |
| Tom-256 base | **976** / **980** | 7,182 / 7,153 | – | 17,907 / 17,933 | – | 11,829 / 11,724 | 43,813 / 44,636 |
| BLS12-381 base | **1,305** / **1,306** | 14,333 / 14,257 | 52,946 / 52,893 | 31,989 / 31,966 | – | 18,657 / 18,671 | 103,655 / 100,407 |
| P-521 base | **1,862** / **1,872** | 25,985 / 26,031 | – | 58,292 / 57,662 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **288** / 289 | 290 / **287** | – | 1,969 / 1,972 | – | – | 738 / 711 | – | 2,096 / 2,096 |
| BabyBear | **282** / **284** | 306 / 301 | – | 1,983 / 1,967 | – | – | 736 / 712 | – | 2,198 / 2,190 |
| Goldilocks | **589** / **583** | 612 / 608 | – | 3,899 / 3,878 | – | – | 1,556 / 1,551 | 1,041 / 1,028 | 4,148 / 4,129 |
| BN254 scalar | **8,349** / **8,330** | 8,382 / 8,459 | 27,898 / 27,421 | 19,500 / 19,645 | – | 14,572 / 14,439 | – | – | 54,751 / 54,289 |
| secp256k1 base | **8,127** / **8,039** | 8,136 / 8,040 | 27,245 / 26,031 | 19,580 / 19,527 | 37,537 / 37,514 | 14,533 / 14,614 | – | – | 54,237 / 55,467 |
| P-256 base | **7,882** / **7,927** | 7,905 / 7,945 | 26,507 / 27,630 | 19,537 / 19,581 | 34,944 / 34,694 | 14,398 / 14,464 | – | – | 53,998 / 53,408 |
| P-256 scalar | 8,685 / 8,690 | **8,679** / **8,657** | 28,292 / 28,330 | 19,734 / 19,589 | 64,795 / 63,991 | 14,558 / 14,817 | – | – | 54,675 / 55,134 |
| Tom-256 base | **8,417** / 8,460 | 8,498 / **8,457** | – | 19,452 / 19,575 | – | 14,660 / 14,629 | – | – | 54,760 / 55,465 |
| BLS12-381 base | 17,267 / 17,139 | **16,966** / **16,933** | 54,303 / 54,062 | 35,253 / 35,626 | – | 24,636 / 24,895 | – | – | 130,577 / 128,386 |
| P-521 base | 30,663 / 30,520 | **30,309** / **30,273** | – | 63,379 / 64,703 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | 11.4 / 19.4 | – | – | – | 10.8 / 17.7 |
| BabyBear | **3.0** / **3.0** | 11.4 / 20.7 | – | – | – | 10.8 / 17.7 |
| Goldilocks | **3.1** / **3.0** | 10.9 / 18.7 | – | – | 4.0 / 4.7 | 10.8 / 17.8 |
| BN254 scalar | **27.0** / **25.9** | 60.1 / 93.0 | – | 35.0 / 34.0 | – | 76.4 / 115 |
| secp256k1 base | **29.0** / **29.8** | 60.2 / 92.5 | 16.1\* / 14.5\* | 33.0 / 31.0 | – | 76.8 / 115 |
| P-256 base | **28.0** / **27.7** | 60.3 / 93.1 | 32.0 / 29.4 | 32.0 / 30.0 | – | 76.9 / 119 |
| P-256 scalar | **29.5** / **30.1** | 60.1 / 92.6 | 56.7 / 60.9 | 35.0 / 33.0 | – | 76.4 / 120 |
| Tom-256 base | **29.1** / **29.1** | 59.5 / 89.7 | – | 34.0 / 33.0 | – | 76.1 / 115 |
| BLS12-381 base | **41.2** / **41.1** | 83.4 / 137 | – | 54.1 / 54.1 | – | 126 / 166 |
| P-521 base | **74.0** / **74.1** | 171 / 244 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 16.8 / 14.7 |
| BabyBear | **2.7** / **3.0** | 5.0 / 8.0 | – | – | – | – | 17.3 / 13.8 |
| Goldilocks | **3.0** / **3.0** | 5.0 / 8.0 | – | – | – | **3.0** / **3.0** | 18.7 / 12.8 |
| BN254 scalar | 26.0 / 25.0 | 37.0 / 30.7 | – | **24.0** / **22.0** | – | – | 54.5 / 75.5 |
| secp256k1 base | 22.5 / 22.0 | 37.2 / 28.6 | 16.4\* / 17.6\* | **22.0** / **21.0** | – | – | 54.5 / 75.4 |
| P-256 base | **22.0** / **20.0** | 37.0 / 27.5 | **22.0** / **20.0** | **22.0** / **20.0** | – | – | 54.6 / 76.3 |
| P-256 scalar | 23.5 / **21.5** | 37.2 / 30.5 | 26.6 / 26.8 | **23.0** / **21.5** | – | – | 55.1 / 76.9 |
| Tom-256 base | **24.0** / 22.5 | 37.2 / 30.6 | – | **24.0** / **22.0** | – | – | 54.9 / 75.8 |
| BLS12-381 base | **37.0** / **36.1** | 57.3 / 106 | – | 37.1 / 36.2 | – | – | 68.8 / 103 |
| P-521 base | **53.1** / **53.1** | 128 / 175 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | 19.2 / 30.7 | – | – | – | 49.9 / 57.1 |
| BabyBear | **5.0** / **11.0** | 19.2 / 30.5 | – | – | – | 50.2 / 57.0 |
| Goldilocks | 9.0 / **14.0** | 19.5 / 30.1 | – | – | **8.1** / **14.0** | 49.9 / 57.2 |
| BN254 scalar | **179** / **167** | 204 / 211 | – | 182 / 178 | – | 1,065 / 1,083 |
| secp256k1 base | **108** / **118** | 205 / 215 | 176 / 175 | 164 / 159 | – | 1,078 / 1,088 |
| P-256 base | **124** / **128** | 202 / 211 | 135 / 131 | 135 / 131 | – | 883 / 871 |
| P-256 scalar | 196 / **171** | 203 / 212 | 341 / 336 | **181** / 177 | – | 1,076 / 1,084 |
| Tom-256 base | **164** / **167** | 204 / 211 | – | 176 / 172 | – | 1,076 / 1,087 |
| BLS12-381 base | 653 / 628 | 529 / 533 | – | **383** / **378** | – | 2,170 / 2,186 |
| P-521 base | **886** / **845** | 1,054 / 1,101 | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | 18.0 / 31.3 | – | – | – | 49.0 / 56.0 |
| BabyBear | **5.8** / **11.0** | 17.9 / 31.2 | – | – | – | 49.1 / 56.0 |
| Goldilocks | 10.0 / 14.4 | 18.3 / 28.9 | – | – | **7.0** / **14.3** | 49.2 / 56.0 |
| BN254 scalar | 174 / **140** | 175 / 176 | – | **145** / 147 | – | 1,066 / 1,064 |
| secp256k1 base | **104** / **95.7** | 174 / 177 | 115 / 136 | 128 / 119 | – | 1,078 / 1,074 |
| P-256 base | **116** / **104** | 175 / 176 | 128 / 130 | 122 / 109 | – | 881 / 772 |
| P-256 scalar | 174 / 140 | 176 / 176 | 336 / 340 | **149** / **138** | – | 1,078 / 1,067 |
| Tom-256 base | 159 / 132 | 175 / 177 | – | **133** / **130** | – | 1,079 / 1,074 |
| BLS12-381 base | 566 / 423 | 564 / 529 | – | **330** / **295** | – | 2,168 / 2,163 |
| P-521 base | **420** / **422** | 1,115 / 1,137 | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **431** / – | 6,456 / – | – | – | – | 3,322 / – |
| BabyBear | **417** / – | 6,484 / – | – | – | – | 3,436 / – |
| Goldilocks | 1,182 / – | 6,488 / – | – | – | **998** / – | 7,096 / – |
| BN254 scalar | **12,585** / – | 30,270 / – | – | 99,633 / – | – | 405,122 / – |
| secp256k1 base | **12,680** / – | 30,524 / – | 37,393 / – | 91,955 / – | – | 539,516 / – |
| P-256 base | **12,369** / – | 30,707 / – | 34,390 / – | 76,828 / – | – | 331,345 / – |
| P-256 scalar | **12,750** / – | 30,324 / – | 146,687 / – | 98,741 / – | – | 455,240 / – |
| Tom-256 base | **12,727** / – | 30,423 / – | – | 97,660 / – | – | 378,501 / – |
| BLS12-381 base | **24,170** / – | 53,688 / – | – | 380,953 / – | – | 1,324,857 / – |
| P-521 base | **41,615** / – | 99,059 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **341** / – | 2,089 / – | – | – | 1,751 / – |
| BabyBear | **6,482** / – | 23,037 / – | – | – | – |
| Goldilocks | **21,637** / – | 33,003 / – | – | – | – |
| BN254 scalar | **152,744** / – | 198,648 / – | – | 179,695 / – | – |
| secp256k1 base | **31,094** / – | 79,874 / – | 37,257 / – | 87,940 / – | 533,849 / – |
| P-256 base | **32,566** / – | 67,723 / – | 34,049 / – | 70,505 / – | 256,152 / – |
| P-256 scalar | **68,202** / – | 79,186 / – | 154,146 / – | 72,293 / – | – |
| Tom-256 base | **51,039** / – | 74,858 / – | – | 93,684 / – | 374,334 / – |
| BLS12-381 base | 303,571 / – | **269,109** / – | – | 375,555 / – | 1,323,049 / – |
| P-521 base | **273,204** / – | 594,739 / – | – | – | – |

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 38.7 / 39.8 | – | – | – |
| BabyBear | **14.7** / **13.0** | 40.2 / 40.8 | – | – | – |
| Goldilocks | 17.1 / 17.1 | 49.9 / 49.6 | – | – | **13.6** / **14.4** |
| BN254 scalar | **57.5** / **58.1** | 407 / 409 | – | 582 / 549 | – |
| secp256k1 base | **37.2** / **44.5** | 406 / 412 | 168 / 176 | 411 / 409 | – |
| P-256 base | **55.7** / **60.1** | 406 / 408 | 258 / 258 | 347 / 351 | – |
| P-256 scalar | **62.1** / **62.6** | 408 / 410 | 339 / 341 | 447 / 488 | – |
| Tom-256 base | **63.5** / **61.1** | 407 / 409 | – | 519 / 554 | – |
| BLS12-381 base | **86.0** / **86.7** | 957 / 1,017 | – | 1,176 / 1,198 | – |
| P-521 base | **95.6** / **94.9** | 1,841 / 1,858 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **14.0** | 38.9 / 38.8 | – | – | – |
| BabyBear | **14.7** / **13.1** | 40.3 / 40.7 | – | – | – |
| Goldilocks | 17.1 / 17.1 | 49.8 / 50.6 | – | – | **13.5** / **15.1** |
| BN254 scalar | **310** / **313** | 410 / 407 | – | 349 / 354 | – |
| secp256k1 base | **251** / **252** | 409 / 407 | – | 267 / 268 | – |
| P-256 base | 305 / 305 | 408 / 409 | – | **253** / **252** | – |
| P-256 scalar | **335** / **334** | 408 / 414 | 486 / 453 | 385 / 347 | – |
| Tom-256 base | **326** / **326** | 408 / 408 | – | 336 / 336 | – |
| BLS12-381 base | **687** / **688** | 1,022 / 1,040 | – | 1,008 / 1,021 | – |
| P-521 base | **1,523** / **1,501** | 1,880 / 1,860 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.3** | 30.8 / 30.4 | – | – | – |
| BabyBear | **12.5** / **11.0** | 32.4 / 31.9 | – | – | – |
| Goldilocks | 12.0 / 10.0 | 36.5 / 33.5 | – | – | **11.0** / **9.7** |
| BN254 scalar | **81.8** / **76.1** | 313 / 310 | – | 322 / 298 | – |
| secp256k1 base | **25.1** / **29.0** | 311 / 307 | 13.1\* / 5.3\* | 304 / 245 | – |
| P-256 base | **72.8** / **73.1** | 318 / 309 | 159 / 158 | 273 / 249 | – |
| P-256 scalar | 81.6 / 85.2 | 311 / 311 | **55.1** / **63.1** | 321 / 298 | – |
| Tom-256 base | **86.0** / **84.8** | 321 / 311 | – | 375 / 296 | – |
| BLS12-381 base | **124** / **123** | 650 / 648 | – | 682 / 620 | – |
| P-521 base | **64.1** / **64.4** | 1,143 / 1,160 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.3** | 30.4 / 30.2 | – | – | – |
| BabyBear | **12.5** / **11.0** | 31.8 / 31.9 | – | – | – |
| Goldilocks | 12.0 / 10.0 | 36.5 / 33.6 | – | – | **11.0** / **9.7** |
| BN254 scalar | 220 / 218 | 314 / 312 | – | **205** / **205** | – |
| secp256k1 base | **44.4** / **44.6** | 317 / 313 | – | 140 / 141 | – |
| P-256 base | 168 / 164 | 315 / 307 | – | **154** / **153** | – |
| P-256 scalar | 223 / 214 | 313 / 311 | **117** / **108** | 204 / 205 | – |
| Tom-256 base | 218 / 225 | 312 / 314 | – | **197** / **196** | – |
| BLS12-381 base | 870 / 831 | 651 / 648 | – | **534** / **427** | – |
| P-521 base | **138** / **135** | 1,213 / 1,159 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **6.6** | 20.9 / 20.3 | – | – | – |
| BabyBear | **12.5** / **11.1** | 21.8 / 21.3 | – | – | – |
| Goldilocks | 11.0 / 9.4 | 28.4 / 24.0 | – | – | **9.5** / **8.7** |
| BN254 scalar | **80.3** / **79.7** | 255 / 286 | – | 358 / 290 | – |
| secp256k1 base | **18.5** / **17.3** | 256 / 287 | 14.0\* / 11.0\* | 263 / 240 | – |
| P-256 base | **68.3** / **69.7** | 228 / 287 | 149 / 149 | 265 / 243 | – |
| P-256 scalar | 77.6 / 78.7 | 259 / 290 | **24.3** / **24.4** | 315 / 289 | – |
| Tom-256 base | **80.7** / **80.0** | 283 / 287 | – | 312 / 288 | – |
| BLS12-381 base | **118** / **120** | 618 / 625 | – | 588 / 589 | – |
| P-521 base | **45.4** / **46.2** | 1,136 / 1,120 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **6.6** | 21.0 / 20.2 | – | – | – |
| BabyBear | **12.5** / **10.9** | 21.8 / 21.3 | – | – | – |
| Goldilocks | 11.0 / 9.4 | 28.3 / 24.1 | – | – | **9.5** / **8.6** |
| BN254 scalar | 225 / 224 | 259 / 283 | – | **194** / **194** | – |
| secp256k1 base | **37.5** / **37.8** | 256 / 282 | – | 130 / 130 | – |
| P-256 base | 160 / 159 | 231 / 285 | – | **143** / **143** | – |
| P-256 scalar | 221 / 209 | 280 / 285 | **89.9** / **95.6** | 194 / 194 | – |
| Tom-256 base | 203 / 204 | 259 / 283 | – | **188** / **188** | – |
| BLS12-381 base | 853 / 825 | 654 / 635 | – | **408** / **408** | – |
| P-521 base | **115** / **113** | 1,166 / 1,127 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / – | 13.2 / – | – | – | – |
| BabyBear | **12.7** / – | 15.0 / – | – | – | – |
| Goldilocks | 12.5 / – | 17.0 / – | – | – | **7.1** / – |
| BN254 scalar | **120** / – | 205 / – | – | 265 / – | – |
| secp256k1 base | **3.4** / – | 205 / – | 6.0 / – | 233 / – | – |
| P-256 base | **84.6** / – | 204 / – | 130 / – | 218 / – | – |
| P-256 scalar | 118 / – | 204 / – | **4.0** / – | 266 / – | – |
| Tom-256 base | **114** / – | 204 / – | – | 265 / – | – |
| BLS12-381 base | **273** / – | 477 / – | – | 569 / – | – |
| P-521 base | **6.0** / – | 850 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **12,900** / **12,905** | 13,177 / 13,005 | 30,862 / 30,732 | – | 101,320 / 100,874 | 405,307 / 405,570 |
| secp256k1 base | **12,871** / **12,796** | 12,995 / 12,797 | 30,661 / 30,967 | 37,528 / 37,530 | 93,535 / 93,618 | 539,737 / 539,134 |
| P-256 base | **12,482** / **12,530** | 12,495 / 12,547 | 30,570 / 31,173 | 34,678 / 34,624 | 76,027 / 75,757 | 334,817 / 327,858 |
| P-256 scalar | **13,283** / **13,221** | 13,399 / 13,309 | 30,656 / 30,688 | 143,110 / 142,691 | 113,880 / 113,577 | 457,011 / 456,976 |
| Tom-256 base | **12,936** / **12,941** | 12,998 / 13,003 | 30,838 / 30,769 | – | 109,906 / 109,766 | 379,465 / 379,077 |
| BLS12-381 base | **24,587** / 24,615 | 24,601 / **24,449** | 54,410 / 53,887 | – | 390,137 / 391,478 | 1,335,564 / 1,331,787 |
| P-521 base | 43,656 / **43,285** | **43,534** / 43,627 | 100,826 / 100,422 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **462** / **459** | 465 / 466 | 6,500 / 6,445 | – | – | – | 3,418 / 3,414 |
| BabyBear | 455 / **438** | **452** / 444 | 6,484 / 6,426 | – | – | – | 3,416 / 3,415 |
| Goldilocks | 1,209 / 1,216 | 1,214 / 1,222 | 6,496 / 6,459 | – | – | **1,041** / **1,028** | 7,110 / 7,125 |
| BN254 scalar | **13,249** / 13,294 | 13,399 / **13,271** | 30,351 / 30,990 | – | 100,792 / 100,792 | – | 403,935 / 402,773 |
| secp256k1 base | **12,796** / **12,796** | 12,913 / 12,844 | 30,742 / 30,914 | 37,537 / 37,514 | 93,218 / 93,343 | – | 546,091 / 539,090 |
| P-256 base | **12,630** / **12,560** | 12,759 / 12,699 | 30,857 / 30,681 | 34,944 / 34,694 | 75,981 / 75,711 | – | 335,891 / 334,494 |
| P-256 scalar | **13,520** / **13,404** | 13,636 / 13,433 | 30,649 / 30,734 | 143,565 / 142,854 | 114,042 / 113,761 | – | 454,881 / 454,587 |
| Tom-256 base | **13,270** / **13,132** | 13,300 / 13,217 | 30,558 / 31,094 | – | 109,982 / 109,807 | – | 383,227 / 379,435 |
| BLS12-381 base | **25,301** / **25,201** | 25,493 / 25,532 | 54,178 / 53,984 | – | 391,156 / 388,855 | – | 1,334,692 / 1,328,342 |
| P-521 base | 43,719 / **43,247** | **43,696** / 43,627 | 100,407 / 101,357 | – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
