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

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
