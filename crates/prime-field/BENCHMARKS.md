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
- Each figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times.
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

`prime-field` at commit `81f38e6` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,070 to 5,469 cycles and 3,081 instructions; its noise in a figure, 59.0 to 255 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,047 to 5,209 cycles and 3,081 instructions; its noise in a figure, 28.0 to 104 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 4,047 to 4,758 cycles and 1,885 instructions; its noise in a figure, 83.0 to 708 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,036 to 4,793 cycles and 1,885 instructions; its noise in a figure, 52.0 to 1,181 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 6.67 before, 13.42 after; its SMT sibling, CPU 28, 6% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 13.14 before, 6.42 after; its SMT sibling, CPU 28, 3% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 6.15 before, 10.63 after; its SMT sibling, CPU 28, 6% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 10.63 before, 10.89 after; its SMT sibling, CPU 28, 5% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.4 vs crypto-bigint 10.8 | −6.4 (0.41×) | 4.4 vs runtime-algebra 10.9 | −6.5 (0.40×) |
| add | BabyBear | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) |
| add | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add | BN254 scalar | 16.7 vs ark-ff 13.6 | +3.1 (1.23×) | 12.3 vs ark-ff 13.2 | −0.9 (0.93×) | 17.1 vs ark-ff 14.1 | +3.0 (1.21×) | 19.0 vs ark-ff 18.5 | +0.5 (1.03×) |
| add | secp256k1 base | 14.1 vs ark-ff 15.2 | −1.1 (0.93×) | 11.8 vs ark-ff 12.8 | −1.0 (0.92×) | 14.9 vs ark-ff 16.7 | −1.8 (0.89×) | 20.5 vs ark-ff 19.9 | +0.6 (1.03×) |
| add | P-256 base | 15.1 vs ark-ff 15.6 | −0.5 (0.97×) | 12.3 vs ark-ff 12.3 | +0.0 (1.00×) | 15.0 vs ark-ff 17.4 | −2.4 (0.86×) | 20.0 vs ark-ff 20.5 | −0.5 (0.98×) |
| add | P-256 scalar | 15.3 vs ark-ff 15.4 | −0.1 (0.99×) | 11.9 vs ark-ff 13.4 | −1.5 (0.89×) | 16.1 vs ark-ff 17.1 | −1.0 (0.94×) | 21.4 vs ark-ff 19.5 | +1.9 (1.10×) |
| add | Tom-256 base | 15.0 vs halo2curves 36.6 | −21.6 (0.41×) | 11.7 vs halo2curves 20.2 | −8.5 (0.58×) | 15.8 vs halo2curves 38.6 | −22.8 (0.41×) | 20.5 vs halo2curves 34.0 | −13.5 (0.60×) |
| add | BLS12-381 base | 25.8 vs ark-ff 35.4 | −9.6 (0.73×) | 17.8 vs ark-ff 30.2 | −12.4 (0.59×) | 24.9 vs ark-ff 39.8 | −14.9 (0.63×) | 29.4 vs ark-ff 41.8 | −12.4 (0.70×) |
| add | P-521 base | 42.3 vs crypto-bigint 142 | −99.7 (0.30×) | 26.3 vs crypto-bigint 134 | −108 (0.20×) | 42.4 vs crypto-bigint 178 | −136 (0.24×) | 44.6 vs crypto-bigint 175 | −130 (0.25×) |
| sub | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub | BabyBear | 2.5 vs plonky3 2.7 | −0.2 (0.93×) | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) |
| sub | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 16.6 vs ark-ff 18.4 | −1.8 (0.90×) | 11.9 vs halo2curves 15.0 | −3.1 (0.79×) | 15.1 vs ark-ff 19.0 | −3.9 (0.79×) | 19.6 vs ark-ff 22.8 | −3.2 (0.86×) |
| sub | secp256k1 base | 16.7 vs ark-ff 18.5 | −1.8 (0.90×) | 11.8 vs halo2curves 14.2 | −2.4 (0.83×) | 15.6 vs ark-ff 19.6 | −4.0 (0.80×) | 19.9 vs halo2curves 22.0 | −2.1 (0.90×) |
| sub | P-256 base | 15.2 vs ark-ff 16.9 | −1.7 (0.90×) | 11.9 vs halo2curves 12.4 | −0.5 (0.96×) | 14.9 vs ark-ff 18.5 | −3.6 (0.81×) | 18.8 vs halo2curves 21.5 | −2.7 (0.87×) |
| sub | P-256 scalar | 16.8 vs ark-ff 17.9 | −1.1 (0.94×) | 11.9 vs halo2curves 14.4 | −2.5 (0.83×) | 15.5 vs ark-ff 18.7 | −3.2 (0.83×) | 19.9 vs halo2curves 23.0 | −3.1 (0.87×) |
| sub | Tom-256 base | 16.5 vs halo2curves 23.8 | −7.3 (0.69×) | 11.9 vs halo2curves 15.0 | −3.1 (0.79×) | 15.5 vs halo2curves 23.4 | −7.9 (0.66×) | 19.6 vs halo2curves 24.0 | −4.4 (0.82×) |
| sub | BLS12-381 base | 25.4 vs ark-ff 33.5 | −8.1 (0.76×) | 17.9 vs halo2curves 23.5 | −5.6 (0.76×) | 24.8 vs ark-ff 31.4 | −6.6 (0.79×) | 29.2 vs halo2curves 37.0 | −7.8 (0.79×) |
| sub | P-521 base | 48.4 vs crypto-bigint 88.7 | −40.3 (0.55×) | 28.6 vs crypto-bigint 65.0 | −36.4 (0.44×) | 42.4 vs crypto-bigint 124 | −81.6 (0.34×) | 44.3 vs crypto-bigint 128 | −83.7 (0.35×) |
| mul | Mersenne-31 | 5.1 vs crypto-bigint 29.3 | −24.2 (0.17×) | 5.1 vs crypto-bigint 15.8 | −10.7 (0.32×) | 5.5 vs crypto-bigint 38.8 | −33.3 (0.14×) | 5.5 vs crypto-bigint 19.2 | −13.7 (0.29×) |
| mul | BabyBear | 7.8 vs plonky3 4.1 | +3.7 (1.90×) | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 7.8 vs plonky3 6.5 | +1.3 (1.20×) | 5.0 vs plonky3 6.5 | −1.5 (0.77×) |
| mul | Goldilocks | 12.2 vs winter-math 12.3 | −0.1 (0.99×) | 7.0 vs winter-math 6.5 | +0.5 (1.08×) | 12.4 vs winter-math 17.1 | −4.7 (0.73×) | 9.0 vs winter-math 7.7 | +1.3 (1.17×) |
| mul | BN254 scalar | 235 vs ark-ff 318 | −83.0 (0.74×) | 95.9 vs halo2curves 91.4 | +4.5 (1.05×) | 386 vs ark-ff 404 | −18.0 (0.96×) | 162 vs halo2curves 182 | −20.0 (0.89×) |
| mul | secp256k1 base | 162 vs ark-ff 279 | −117 (0.58×) | 64.1 vs halo2curves 91.0 | −26.9 (0.70×) | 218 vs ark-ff 407 | −189 (0.54×) | 108 vs halo2curves 163 | −55.0 (0.66×) |
| mul | P-256 base | 177 vs k256/p256 241 | −64.0 (0.73×) | 71.4 vs k256/p256 85.0 | −13.6 (0.84×) | 284 vs k256/p256 473 | −189 (0.60×) | 122 vs k256/p256 135 | −13.0 (0.90×) |
| mul | P-256 scalar | 241 vs ark-ff 314 | −73.0 (0.77×) | 86.6 vs halo2curves 93.5 | −6.9 (0.93×) | 319 vs ark-ff 422 | −103 (0.76×) | 172 vs halo2curves 181 | −9.0 (0.95×) |
| mul | Tom-256 base | 237 vs halo2curves 344 | −107 (0.69×) | 80.2 vs halo2curves 94.2 | −14.0 (0.85×) | 304 vs halo2curves 608 | −304 (0.50×) | 149 vs halo2curves 176 | −27.0 (0.85×) |
| mul | BLS12-381 base | 726 vs ark-ff 805 | −79.0 (0.90×) | 248 vs halo2curves 195 | +53.0 (1.27×) | 916 vs ark-ff 1,061 | −145 (0.86×) | 421 vs halo2curves 473 | −52.0 (0.89×) |
| mul | P-521 base | 1,020 vs crypto-bigint 2,114 | −1,094 (0.48×) | 399 vs crypto-bigint 600 | −201 (0.67×) | 1,637 vs crypto-bigint 2,716 | −1,079 (0.60×) | 846 vs crypto-bigint 1,054 | −208 (0.80×) |
| square | Mersenne-31 | 5.2 vs crypto-bigint 28.5 | −23.3 (0.18×) | 5.2 vs crypto-bigint 15.6 | −10.4 (0.33×) | 5.4 vs crypto-bigint 38.3 | −32.9 (0.14×) | 5.4 vs crypto-bigint 18.0 | −12.6 (0.30×) |
| square | BabyBear | 8.3 vs plonky3 5.1 | +3.2 (1.63×) | 4.8 vs plonky3 5.2 | −0.4 (0.92×) | 8.0 vs plonky3 5.7 | +2.3 (1.40×) | 5.8 vs plonky3 5.7 | +0.1 (1.02×) |
| square | Goldilocks | 10.1 vs winter-math 12.4 | −2.3 (0.81×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 12.0 vs winter-math 17.0 | −5.0 (0.71×) | 9.7 vs winter-math 7.0 | +2.7 (1.39×) |
| square | BN254 scalar | 215 vs ark-ff 271 | −56.0 (0.79×) | 95.0 vs halo2curves 84.3 | +10.7 (1.13×) | 374 vs ark-ff 359 | +15.0 (1.04×) | 160 vs halo2curves 144 | +16.0 (1.11×) |
| square | secp256k1 base | 143 vs halo2curves 215 | −72.0 (0.67×) | 60.8 vs halo2curves 77.9 | −17.1 (0.78×) | 192 vs ark-ff 306 | −114 (0.63×) | 99.5 vs halo2curves 121 | −21.5 (0.82×) |
| square | P-256 base | 159 vs halo2curves 200 | −41.0 (0.80×) | 78.3 vs k256/p256 73.6 | +4.7 (1.06×) | 220 vs ark-ff 341 | −121 (0.65×) | 116 vs halo2curves 107 | +9.0 (1.08×) |
| square | P-256 scalar | 219 vs ark-ff 266 | −47.0 (0.82×) | 83.6 vs halo2curves 83.0 | +0.6 (1.01×) | 281 vs ark-ff 350 | −69.0 (0.80×) | 144 vs halo2curves 140 | +4.0 (1.03×) |
| square | Tom-256 base | 205 vs halo2curves 317 | −112 (0.65×) | 75.9 vs halo2curves 79.8 | −3.9 (0.95×) | 226 vs halo2curves 494 | −268 (0.46×) | 158 vs halo2curves 134 | +24.0 (1.18×) |
| square | BLS12-381 base | 578 vs ark-ff 633 | −55.0 (0.91×) | 282 vs halo2curves 163 | +119 (1.73×) | 887 vs ark-ff 825 | +62.0 (1.08×) | 449 vs halo2curves 286 | +163 (1.57×) |
| square | P-521 base | 563 vs crypto-bigint 2,154 | −1,591 (0.26×) | 260 vs crypto-bigint 614 | −354 (0.42×) | 909 vs crypto-bigint 2,798 | −1,889 (0.32×) | 419 vs crypto-bigint 1,123 | −704 (0.37×) |
| invert | Mersenne-31 | 266 vs plonky3 676 | −410 (0.39×) | 266 vs plonky3 672 | −406 (0.40×) | 268 vs plonky3 718 | −450 (0.37×) | 270 vs plonky3 729 | −459 (0.37×) |
| invert | BabyBear | 269 vs plonky3 701 | −432 (0.38×) | 268 vs plonky3 700 | −432 (0.38×) | 277 vs plonky3 728 | −451 (0.38×) | 277 vs plonky3 718 | −441 (0.39×) |
| invert | Goldilocks | 537 vs winter-math 1,772 | −1,235 (0.30×) | 537 vs winter-math 912 | −375 (0.59×) | 563 vs winter-math 1,837 | −1,274 (0.31×) | 571 vs winter-math 998 | −427 (0.57×) |
| invert | BN254 scalar | 8,440 vs halo2curves 14,085 | −5,645 (0.60×) | 6,417 vs halo2curves 8,010 | −1,593 (0.80×) | 11,668 vs halo2curves 21,266 | −9,598 (0.55×) | 7,879 vs halo2curves 14,337 | −6,458 (0.55×) |
| invert | secp256k1 base | 8,502 vs halo2curves 14,123 | −5,621 (0.60×) | 6,031 vs halo2curves 8,001 | −1,970 (0.75×) | 11,702 vs halo2curves 21,577 | −9,875 (0.54×) | 7,960 vs halo2curves 14,385 | −6,425 (0.55×) |
| invert | P-256 base | 8,282 vs halo2curves 14,147 | −5,865 (0.59×) | 6,062 vs halo2curves 8,013 | −1,951 (0.76×) | 11,269 vs halo2curves 21,387 | −10,118 (0.53×) | 7,747 vs halo2curves 14,263 | −6,516 (0.54×) |
| invert | P-256 scalar | 8,482 vs halo2curves 14,142 | −5,660 (0.60×) | 6,399 vs halo2curves 8,029 | −1,630 (0.80×) | 11,980 vs halo2curves 21,472 | −9,492 (0.56×) | 8,223 vs halo2curves 14,303 | −6,080 (0.57×) |
| invert | Tom-256 base | 8,554 vs halo2curves 14,150 | −5,596 (0.60×) | 6,400 vs halo2curves 8,023 | −1,623 (0.80×) | 11,444 vs halo2curves 21,426 | −9,982 (0.53×) | 7,943 vs halo2curves 14,362 | −6,419 (0.55×) |
| invert | BLS12-381 base | 19,335 vs halo2curves 25,564 | −6,229 (0.76×) | 11,422 vs halo2curves 13,127 | −1,705 (0.87×) | 24,208 vs halo2curves 37,675 | −13,467 (0.64×) | 15,490 vs halo2curves 23,613 | −8,123 (0.66×) |
| invert | P-521 base | 33,221 vs crypto-bigint 65,268 | −32,047 (0.51×) | 18,196 vs crypto-bigint 50,293 | −32,097 (0.36×) | 47,873 vs crypto-bigint 79,649 | −31,776 (0.60×) | 29,337 vs crypto-bigint 62,401 | −33,064 (0.47×) |
| sqrt | Mersenne-31 | 329 vs plonky3 324 | +5.0 (1.02×) | 329 vs plonky3 324 | +5.0 (1.02×) | 330 vs plonky3 329 | +1.0 (1.00×) | 330 vs plonky3 329 | +1.0 (1.00×) |
| sqrt | BabyBear | 2,670 vs plonky3 3,090 | −420 (0.86×) | 2,040 vs plonky3 3,118 | −1,078 (0.65×) | 2,760 vs plonky3 2,989 | −229 (0.92×) | 2,167 vs plonky3 3,000 | −833 (0.72×) |
| sqrt | Goldilocks | 5,768 vs crypto-bigint 48,789 | −43,021 (0.12×) | 3,697 vs crypto-bigint 28,889 | −25,192 (0.13×) | 6,762 vs crypto-bigint 54,915 | −48,153 (0.12×) | 9,179 vs crypto-bigint 32,998 | −23,819 (0.28×) |
| sqrt | BN254 scalar | 108,085 vs ark-ff 147,910 | −39,825 (0.73×) | 47,313 vs halo2curves 102,211 | −54,898 (0.46×) | 196,081 vs ark-ff 201,732 | −5,651 (0.97×) | 81,894 vs ark-ff 140,915 | −59,021 (0.58×) |
| sqrt | secp256k1 base | 42,090 vs ark-ff 130,955 | −88,865 (0.32×) | 19,095 vs halo2curves 42,787 | −23,692 (0.45×) | 72,835 vs ark-ff 176,941 | −104,106 (0.41×) | 30,979 vs crypto-bigint 79,732 | −48,753 (0.39×) |
| sqrt | P-256 base | 44,757 vs ark-ff 59,370 | −14,613 (0.75×) | 16,334 vs k256/p256 21,580 | −5,246 (0.76×) | 78,047 vs ark-ff 98,628 | −20,581 (0.79×) | 27,589 vs k256/p256 34,036 | −6,447 (0.81×) |
| sqrt | P-256 scalar | 74,255 vs ark-ff 115,519 | −41,264 (0.64×) | 35,837 vs halo2curves 49,826 | −13,989 (0.72×) | 113,654 vs ark-ff 157,635 | −43,981 (0.72×) | 64,107 vs halo2curves 72,365 | −8,258 (0.89×) |
| sqrt | Tom-256 base | 65,526 vs crypto-bigint 149,734 | −84,208 (0.44×) | 21,187 vs halo2curves 45,702 | −24,515 (0.46×) | 94,927 vs crypto-bigint 223,204 | −128,277 (0.43×) | 55,442 vs crypto-bigint 74,698 | −19,256 (0.74×) |
| sqrt | BLS12-381 base | 311,855 vs ark-ff 419,409 | −107,554 (0.74×) | 133,690 vs crypto-bigint 145,054 | −11,364 (0.92×) | 423,946 vs ark-ff 567,436 | −143,490 (0.75×) | 212,164 vs crypto-bigint 272,471 | −60,307 (0.78×) |
| sqrt | P-521 base | 320,120 vs crypto-bigint 1,528,245 | −1,208,125 (0.21×) | 143,071 vs crypto-bigint 296,403 | −153,332 (0.48×) | 473,856 vs crypto-bigint 1,565,745 | −1,091,889 (0.30×) | 268,222 vs crypto-bigint 591,360 | −323,138 (0.45×) |
| mul_u64 | Mersenne-31 | 9.4 vs crypto-bigint 61.3 | −51.9 (0.15×) | 9.3 vs crypto-bigint 30.5 | −21.2 (0.30×) | 9.6 vs crypto-bigint 69.6 | −60.0 (0.14×) | 9.7 vs crypto-bigint 38.7 | −29.0 (0.25×) |
| mul_u64 | BabyBear | 17.0 vs plonky3 34.9 | −17.9 (0.49×) | 11.2 vs plonky3 12.0 | −0.8 (0.93×) | 20.1 vs plonky3 45.9 | −25.8 (0.44×) | 14.7 vs plonky3 17.7 | −3.0 (0.83×) |
| mul_u64 | Goldilocks | 19.1 vs winter-math 25.0 | −5.9 (0.76×) | 11.0 vs winter-math 10.8 | +0.2 (1.02×) | 21.1 vs winter-math 32.7 | −11.6 (0.65×) | 17.1 vs winter-math 13.5 | +3.6 (1.27×) |
| mul_u64 | BN254 scalar | 99.6 vs ark-ff 694 | −594 (0.14×) | 28.8 vs crypto-bigint 263 | −234 (0.11×) | 124 vs ark-ff 828 | −704 (0.15×) | 44.1 vs crypto-bigint 406 | −362 (0.11×) |
| mul_u64 | secp256k1 base | 42.2 vs ark-ff 451 | −409 (0.09×) | 12.6 vs crypto-bigint 262 | −249 (0.05×) | 65.1 vs ark-ff 633 | −568 (0.10×) | 23.0 vs crypto-bigint 408 | −385 (0.06×) |
| mul_u64 | P-256 base | 72.2 vs k256/p256 484 | −412 (0.15×) | 27.3 vs k256/p256 150 | −123 (0.18×) | 94.2 vs ark-ff 878 | −784 (0.11×) | 42.1 vs k256/p256 259 | −217 (0.16×) |
| mul_u64 | P-256 scalar | 84.1 vs k256/p256 517 | −433 (0.16×) | 30.3 vs k256/p256 249 | −219 (0.12×) | 113 vs ark-ff 808 | −695 (0.14×) | 48.0 vs k256/p256 395 | −347 (0.12×) |
| mul_u64 | Tom-256 base | 82.2 vs crypto-bigint 867 | −785 (0.09×) | 30.0 vs crypto-bigint 262 | −232 (0.11×) | 115 vs halo2curves 1,246 | −1,131 (0.09×) | 44.3 vs crypto-bigint 407 | −363 (0.11×) |
| mul_u64 | BLS12-381 base | 131 vs ark-ff 1,636 | −1,505 (0.08×) | 44.2 vs halo2curves 481 | −437 (0.09×) | 167 vs ark-ff 2,139 | −1,972 (0.08×) | 61.5 vs crypto-bigint 1,034 | −973 (0.06×) |
| mul_u64 | P-521 base | 92.1 vs crypto-bigint 3,011 | −2,919 (0.03×) | 32.1 vs crypto-bigint 984 | −952 (0.03×) | 232 vs crypto-bigint 3,977 | −3,745 (0.06×) | 49.4 vs crypto-bigint 1,825 | −1,776 (0.03×) |
| mul_uint | Mersenne-31 | 9.4 vs crypto-bigint 60.5 | −51.1 (0.16×) | 9.4 vs crypto-bigint 30.5 | −21.1 (0.31×) | 9.7 vs crypto-bigint 72.3 | −62.6 (0.13×) | 9.7 vs crypto-bigint 38.9 | −29.2 (0.25×) |
| mul_uint | BabyBear | 17.0 vs plonky3 23.0 | −6.0 (0.74×) | 11.2 vs plonky3 11.6 | −0.4 (0.97×) | 20.0 vs plonky3 35.1 | −15.1 (0.57×) | 14.7 vs plonky3 17.7 | −3.0 (0.83×) |
| mul_uint | Goldilocks | 19.1 vs winter-math 24.9 | −5.8 (0.77×) | 11.0 vs winter-math 10.8 | +0.2 (1.02×) | 21.1 vs winter-math 32.7 | −11.6 (0.65×) | 17.1 vs winter-math 13.6 | +3.5 (1.26×) |
| mul_uint | BN254 scalar | 433 vs halo2curves 756 | −323 (0.57×) | 177 vs halo2curves 181 | −4.0 (0.98×) | 542 vs halo2curves 1,347 | −805 (0.40×) | 253 vs halo2curves 349 | −96.0 (0.72×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 519 | −253 (0.51×) | 113 vs halo2curves 154 | −41.0 (0.73×) | 324 vs halo2curves 971 | −647 (0.33×) | 164 vs halo2curves 265 | −101 (0.62×) |
| mul_uint | P-256 base | 353 vs halo2curves 456 | −103 (0.77×) | 173 vs halo2curves 143 | +30.0 (1.21×) | 430 vs halo2curves 994 | −564 (0.43×) | 236 vs halo2curves 252 | −16.0 (0.94×) |
| mul_uint | P-256 scalar | 426 vs k256/p256 549 | −123 (0.78×) | 188 vs halo2curves 183 | +5.0 (1.03×) | 473 vs k256/p256 886 | −413 (0.53×) | 263 vs halo2curves 347 | −84.0 (0.76×) |
| mul_uint | Tom-256 base | 410 vs halo2curves 692 | −282 (0.59×) | 178 vs halo2curves 180 | −2.0 (0.99×) | 488 vs halo2curves 1,183 | −695 (0.41×) | 254 vs halo2curves 335 | −81.0 (0.76×) |
| mul_uint | BLS12-381 base | 990 vs crypto-bigint 1,903 | −913 (0.52×) | 380 vs halo2curves 382 | −2.0 (0.99×) | 1,155 vs crypto-bigint 2,501 | −1,346 (0.46×) | 552 vs crypto-bigint 940 | −388 (0.59×) |
| mul_uint | P-521 base | 1,472 vs crypto-bigint 3,010 | −1,538 (0.49×) | 618 vs crypto-bigint 1,008 | −390 (0.61×) | 2,383 vs crypto-bigint 3,978 | −1,595 (0.60×) | 986 vs crypto-bigint 1,899 | −913 (0.52×) |
| add_u64 | Mersenne-31 | 8.1 vs crypto-bigint 36.1 | −28.0 (0.22×) | 8.1 vs crypto-bigint 30.4 | −22.3 (0.27×) | 9.2 vs crypto-bigint 40.2 | −31.0 (0.23×) | 9.1 vs crypto-bigint 30.7 | −21.6 (0.30×) |
| add_u64 | BabyBear | 10.9 vs plonky3 32.1 | −21.2 (0.34×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 44.0 | −31.0 (0.30×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| add_u64 | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 10.2 | +1.8 (1.18×) |
| add_u64 | BN254 scalar | 122 vs ark-ff 349 | −227 (0.35×) | 53.3 vs crypto-bigint 221 | −168 (0.24×) | 178 vs ark-ff 429 | −251 (0.41×) | 86.7 vs crypto-bigint 311 | −224 (0.28×) |
| add_u64 | secp256k1 base | 8.0 vs ark-ff 162 | −154 (0.05×) | 8.0 vs ark-ff 121 | −113 (0.07×) | 8.0 vs ark-ff 228 | −220 (0.04×) | 11.5 vs ark-ff 184 | −173 (0.06×) |
| add_u64 | P-256 base | 72.3 vs ark-ff 268 | −196 (0.27×) | 36.8 vs k256/p256 96.6 | −59.8 (0.38×) | 85.7 vs ark-ff 409 | −323 (0.21×) | 67.5 vs k256/p256 159 | −91.5 (0.42×) |
| add_u64 | P-256 scalar | 70.2 vs k256/p256 41.7 | +28.5 (1.68×) | 40.4 vs k256/p256 40.9 | −0.5 (0.99×) | 72.5 vs k256/p256 54.6 | +17.9 (1.33×) | 83.3 vs k256/p256 54.7 | +28.6 (1.52×) |
| add_u64 | Tom-256 base | 89.6 vs crypto-bigint 505 | −415 (0.18×) | 41.6 vs crypto-bigint 221 | −179 (0.19×) | 109 vs halo2curves 737 | −628 (0.15×) | 71.1 vs crypto-bigint 313 | −242 (0.23×) |
| add_u64 | BLS12-381 base | 179 vs ark-ff 836 | −657 (0.21×) | 65.9 vs halo2curves 323 | −257 (0.20×) | 258 vs ark-ff 1,071 | −813 (0.24×) | 96.5 vs halo2curves 632 | −536 (0.15×) |
| add_u64 | P-521 base | 23.1 vs crypto-bigint 1,668 | −1,645 (0.01×) | 16.1 vs crypto-bigint 649 | −633 (0.02×) | 22.1 vs crypto-bigint 2,182 | −2,160 (0.01×) | 25.1 vs crypto-bigint 1,127 | −1,102 (0.02×) |
| add_uint | Mersenne-31 | 8.2 vs crypto-bigint 36.1 | −27.9 (0.23×) | 8.2 vs crypto-bigint 30.4 | −22.2 (0.27×) | 9.2 vs crypto-bigint 40.2 | −31.0 (0.23×) | 9.1 vs crypto-bigint 30.4 | −21.3 (0.30×) |
| add_uint | BabyBear | 10.9 vs plonky3 22.0 | −11.1 (0.50×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 34.8 | −21.8 (0.37×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| add_uint | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 10.2 | +1.8 (1.18×) |
| add_uint | BN254 scalar | 290 vs halo2curves 412 | −122 (0.70×) | 142 vs halo2curves 113 | +29.0 (1.26×) | 414 vs halo2curves 675 | −261 (0.61×) | 212 vs halo2curves 205 | +7.0 (1.03×) |
| add_uint | secp256k1 base | 20.9 vs halo2curves 223 | −202 (0.09×) | 14.6 vs halo2curves 85.4 | −70.8 (0.17×) | 18.5 vs halo2curves 425 | −407 (0.04×) | 24.4 vs halo2curves 140 | −116 (0.17×) |
| add_uint | P-256 base | 204 vs halo2curves 244 | −40.0 (0.84×) | 87.2 vs halo2curves 91.5 | −4.3 (0.95×) | 333 vs halo2curves 513 | −180 (0.65×) | 165 vs halo2curves 153 | +12.0 (1.08×) |
| add_uint | P-256 scalar | 295 vs k256/p256 94.6 | +200 (3.12×) | 135 vs k256/p256 94.8 | +40.2 (1.42×) | 382 vs k256/p256 120 | +262 (3.18×) | 226 vs k256/p256 117 | +109 (1.93×) |
| add_uint | Tom-256 base | 294 vs halo2curves 378 | −84.0 (0.78×) | 134 vs halo2curves 110 | +24.0 (1.22×) | 374 vs halo2curves 622 | −248 (0.60×) | 212 vs halo2curves 196 | +16.0 (1.08×) |
| add_uint | BLS12-381 base | 836 vs halo2curves 979 | −143 (0.85×) | 343 vs halo2curves 225 | +118 (1.52×) | 1,036 vs crypto-bigint 1,386 | −350 (0.75×) | 521 vs halo2curves 424 | +97.0 (1.23×) |
| add_uint | P-521 base | 64.8 vs crypto-bigint 1,693 | −1,628 (0.04×) | 42.0 vs crypto-bigint 666 | −624 (0.06×) | 86.7 vs crypto-bigint 2,189 | −2,102 (0.04×) | 72.7 vs crypto-bigint 1,193 | −1,120 (0.06×) |
| sub_u64 | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.1 vs crypto-bigint 30.7 | −23.6 (0.23×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_u64 | BabyBear | 11.0 vs crypto-bigint 27.9 | −16.9 (0.39×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs crypto-bigint 42.2 | −29.2 (0.31×) | 12.5 vs plonky3 15.1 | −2.6 (0.83×) |
| sub_u64 | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 352 | −239 (0.32×) | 51.3 vs crypto-bigint 151 | −99.7 (0.34×) | 139 vs ark-ff 402 | −263 (0.35×) | 73.6 vs crypto-bigint 257 | −183 (0.29×) |
| sub_u64 | secp256k1 base | 8.7 vs ark-ff 161 | −152 (0.05×) | 8.0 vs ark-ff 124 | −116 (0.06×) | 8.0 vs ark-ff 231 | −223 (0.03×) | 10.0 vs ark-ff 193 | −183 (0.05×) |
| sub_u64 | P-256 base | 71.4 vs k256/p256 264 | −193 (0.27×) | 36.8 vs k256/p256 89.4 | −52.6 (0.41×) | 80.9 vs ark-ff 404 | −323 (0.20×) | 80.9 vs k256/p256 149 | −68.1 (0.54×) |
| sub_u64 | P-256 scalar | 69.3 vs k256/p256 21.2 | +48.1 (3.27×) | 40.2 vs k256/p256 21.2 | +19.0 (1.90×) | 75.5 vs k256/p256 24.5 | +51.0 (3.08×) | 84.0 vs k256/p256 24.2 | +59.8 (3.47×) |
| sub_u64 | Tom-256 base | 88.1 vs crypto-bigint 493 | −405 (0.18×) | 50.1 vs crypto-bigint 151 | −101 (0.33×) | 106 vs halo2curves 716 | −610 (0.15×) | 72.2 vs crypto-bigint 256 | −184 (0.28×) |
| sub_u64 | BLS12-381 base | 162 vs ark-ff 837 | −675 (0.19×) | 67.6 vs halo2curves 308 | −240 (0.22×) | 250 vs ark-ff 1,106 | −856 (0.23×) | 96.3 vs halo2curves 612 | −516 (0.16×) |
| sub_u64 | P-521 base | 22.5 vs crypto-bigint 1,673 | −1,651 (0.01×) | 18.7 vs crypto-bigint 622 | −603 (0.03×) | 20.5 vs crypto-bigint 2,159 | −2,139 (0.01×) | 21.1 vs crypto-bigint 1,139 | −1,118 (0.02×) |
| sub_uint | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.1 vs crypto-bigint 30.7 | −23.6 (0.23×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_uint | BabyBear | 11.0 vs plonky3 23.0 | −12.0 (0.48×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs plonky3 34.9 | −21.9 (0.37×) | 12.5 vs plonky3 15.1 | −2.6 (0.83×) |
| sub_uint | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.3 vs winter-math 19.0 | −7.7 (0.59×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_uint | BN254 scalar | 287 vs halo2curves 401 | −114 (0.72×) | 146 vs halo2curves 104 | +42.0 (1.40×) | 417 vs halo2curves 654 | −237 (0.64×) | 216 vs halo2curves 194 | +22.0 (1.11×) |
| sub_uint | secp256k1 base | 21.2 vs halo2curves 215 | −194 (0.10×) | 14.6 vs halo2curves 108 | −93.4 (0.14×) | 19.3 vs halo2curves 411 | −392 (0.05×) | 23.5 vs halo2curves 130 | −107 (0.18×) |
| sub_uint | P-256 base | 206 vs halo2curves 233 | −27.0 (0.88×) | 87.8 vs halo2curves 85.7 | +2.1 (1.02×) | 376 vs halo2curves 499 | −123 (0.75×) | 165 vs halo2curves 144 | +21.0 (1.15×) |
| sub_uint | P-256 scalar | 290 vs k256/p256 67.8 | +222 (4.28×) | 133 vs k256/p256 70.4 | +62.6 (1.89×) | 391 vs k256/p256 87.6 | +303 (4.46×) | 227 vs k256/p256 89.5 | +138 (2.54×) |
| sub_uint | Tom-256 base | 294 vs halo2curves 364 | −70.0 (0.81×) | 131 vs halo2curves 104 | +27.0 (1.26×) | 369 vs halo2curves 607 | −238 (0.61×) | 217 vs halo2curves 187 | +30.0 (1.16×) |
| sub_uint | BLS12-381 base | 825 vs halo2curves 972 | −147 (0.85×) | 348 vs halo2curves 216 | +132 (1.61×) | 1,007 vs crypto-bigint 1,355 | −348 (0.74×) | 524 vs halo2curves 407 | +117 (1.29×) |
| sub_uint | P-521 base | 75.5 vs crypto-bigint 1,677 | −1,602 (0.05×) | 42.1 vs crypto-bigint 635 | −593 (0.07×) | 79.1 vs crypto-bigint 2,168 | −2,089 (0.04×) | 69.1 vs crypto-bigint 1,164 | −1,095 (0.06×) |
| from_u64 | Mersenne-31 | 4.7 vs plonky3 4.1 | +0.6 (1.15×) | 4.7 vs plonky3 4.1 | +0.6 (1.15×) | 5.2 vs plonky3 3.8 | +1.4 (1.37×) | 5.2 vs plonky3 3.9 | +1.3 (1.33×) |
| from_u64 | BabyBear | 11.1 vs crypto-bigint 21.6 | −10.5 (0.51×) | 7.3 vs plonky3 7.5 | −0.2 (0.97×) | 12.9 vs crypto-bigint 35.1 | −22.2 (0.37×) | 10.3 vs plonky3 11.8 | −1.5 (0.87×) |
| from_u64 | Goldilocks | 7.2 vs winter-math 10.6 | −3.4 (0.68×) | 5.5 vs winter-math 5.9 | −0.4 (0.93×) | 9.0 vs winter-math 17.0 | −8.0 (0.53×) | 8.7 vs winter-math 7.0 | +1.7 (1.24×) |
| from_u64 | BN254 scalar | 87.9 vs ark-ff 317 | −229 (0.28×) | 28.2 vs crypto-bigint 132 | −104 (0.21×) | 121 vs ark-ff 370 | −249 (0.33×) | 46.0 vs crypto-bigint 204 | −158 (0.23×) |
| from_u64 | secp256k1 base | 3.8 vs k256/p256 6.0 | −2.2 (0.63×) | 3.8 vs k256/p256 6.0 | −2.2 (0.63×) | 3.8 vs k256/p256 5.9 | −2.1 (0.64×) | 3.8 vs k256/p256 6.0 | −2.2 (0.63×) |
| from_u64 | P-256 base | 51.9 vs k256/p256 237 | −185 (0.22×) | 25.3 vs k256/p256 76.0 | −50.7 (0.33×) | 61.0 vs ark-ff 380 | −319 (0.16×) | 39.0 vs k256/p256 131 | −92.0 (0.30×) |
| from_u64 | P-256 scalar | 50.7 vs k256/p256 4.0 | +46.7 (12.68×) | 25.9 vs k256/p256 4.0 | +21.9 (6.47×) | 58.6 vs k256/p256 4.0 | +54.6 (14.65×) | 43.0 vs k256/p256 4.0 | +39.0 (10.75×) |
| from_u64 | Tom-256 base | 71.4 vs crypto-bigint 430 | −359 (0.17×) | 28.9 vs crypto-bigint 133 | −104 (0.22×) | 85.8 vs halo2curves 658 | −572 (0.13×) | 43.4 vs crypto-bigint 204 | −161 (0.21×) |
| from_u64 | BLS12-381 base | 137 vs ark-ff 780 | −643 (0.18×) | 44.6 vs crypto-bigint 254 | −209 (0.18×) | 217 vs ark-ff 998 | −781 (0.22×) | 64.0 vs crypto-bigint 516 | −452 (0.12×) |
| from_u64 | P-521 base | 6.0 vs crypto-bigint 1,460 | −1,454 (0.00×) | 6.0 vs crypto-bigint 506 | −500 (0.01×) | 6.0 vs crypto-bigint 1,964 | −1,958 (0.00×) | 6.0 vs crypto-bigint 889 | −883 (0.01×) |
| div_u64 | BN254 scalar | 1,215 vs halo2curves 13,498 | −12,283 (0.09×) | 834 vs halo2curves 7,017 | −6,183 (0.12×) | 1,432 vs halo2curves 19,335 | −17,903 (0.07×) | 982 vs halo2curves 11,753 | −10,771 (0.08×) |
| div_u64 | secp256k1 base | 1,000 vs halo2curves 13,076 | −12,076 (0.08×) | 766 vs halo2curves 6,923 | −6,157 (0.11×) | 1,233 vs halo2curves 18,951 | −17,718 (0.07×) | 914 vs halo2curves 11,588 | −10,674 (0.08×) |
| div_u64 | P-256 base | 1,089 vs halo2curves 13,089 | −12,000 (0.08×) | 867 vs halo2curves 6,881 | −6,014 (0.13×) | 1,350 vs halo2curves 18,888 | −17,538 (0.07×) | 948 vs halo2curves 11,565 | −10,617 (0.08×) |
| div_u64 | P-256 scalar | 1,183 vs halo2curves 13,320 | −12,137 (0.09×) | 891 vs halo2curves 6,955 | −6,064 (0.13×) | 1,422 vs halo2curves 19,404 | −17,982 (0.07×) | 1,004 vs halo2curves 11,679 | −10,675 (0.09×) |
| div_u64 | Tom-256 base | 1,192 vs halo2curves 13,332 | −12,140 (0.09×) | 823 vs halo2curves 6,918 | −6,095 (0.12×) | 1,493 vs halo2curves 19,200 | −17,707 (0.08×) | 972 vs halo2curves 11,645 | −10,673 (0.08×) |
| div_u64 | BLS12-381 base | 1,795 vs halo2curves 24,070 | −22,275 (0.07×) | 1,106 vs halo2curves 11,006 | −9,900 (0.10×) | 2,475 vs halo2curves 32,205 | −29,730 (0.08×) | 1,307 vs halo2curves 18,616 | −17,309 (0.07×) |
| div_u64 | P-521 base | 2,432 vs crypto-bigint 62,460 | −60,028 (0.04×) | 1,369 vs crypto-bigint 45,412 | −44,043 (0.03×) | 3,577 vs crypto-bigint 77,902 | −74,325 (0.05×) | 1,930 vs crypto-bigint 57,887 | −55,957 (0.03×) |
| div_uint | Mersenne-31 | 277 vs crypto-bigint 1,816 | −1,539 (0.15×) | 277 vs crypto-bigint 1,606 | −1,329 (0.17×) | 287 vs runtime-algebra 2,098 | −1,811 (0.14×) | 281 vs crypto-bigint 1,976 | −1,695 (0.14×) |
| div_uint | BabyBear | 278 vs plonky3 680 | −402 (0.41×) | 278 vs plonky3 662 | −384 (0.42×) | 284 vs plonky3 753 | −469 (0.38×) | 291 vs plonky3 716 | −425 (0.41×) |
| div_uint | Goldilocks | 550 vs winter-math 1,825 | −1,275 (0.30×) | 544 vs winter-math 946 | −402 (0.58×) | 594 vs winter-math 1,894 | −1,300 (0.31×) | 586 vs winter-math 1,047 | −461 (0.56×) |
| div_uint | BN254 scalar | 9,213 vs halo2curves 15,095 | −5,882 (0.61×) | 6,685 vs halo2curves 8,226 | −1,541 (0.81×) | 12,642 vs halo2curves 23,032 | −10,390 (0.55×) | 8,365 vs halo2curves 14,515 | −6,150 (0.58×) |
| div_uint | secp256k1 base | 8,585 vs halo2curves 14,909 | −6,324 (0.58×) | 6,068 vs halo2curves 8,204 | −2,136 (0.74×) | 11,908 vs halo2curves 22,760 | −10,852 (0.52×) | 8,118 vs halo2curves 14,549 | −6,431 (0.56×) |
| div_uint | P-256 base | 8,786 vs halo2curves 14,816 | −6,030 (0.59×) | 6,169 vs halo2curves 8,220 | −2,051 (0.75×) | 12,357 vs halo2curves 22,601 | −10,244 (0.55×) | 7,901 vs halo2curves 14,404 | −6,503 (0.55×) |
| div_uint | P-256 scalar | 9,231 vs halo2curves 15,092 | −5,861 (0.61×) | 6,641 vs halo2curves 8,240 | −1,599 (0.81×) | 12,629 vs halo2curves 23,112 | −10,483 (0.55×) | 8,667 vs halo2curves 14,686 | −6,019 (0.59×) |
| div_uint | Tom-256 base | 9,122 vs halo2curves 15,034 | −5,912 (0.61×) | 6,655 vs halo2curves 8,247 | −1,592 (0.81×) | 12,167 vs halo2curves 23,023 | −10,856 (0.53×) | 8,491 vs halo2curves 14,559 | −6,068 (0.58×) |
| div_uint | BLS12-381 base | 21,024 vs halo2curves 27,656 | −6,632 (0.76×) | 12,074 vs halo2curves 13,593 | −1,519 (0.89×) | 26,726 vs halo2curves 40,048 | −13,322 (0.67×) | 16,831 vs halo2curves 24,437 | −7,606 (0.69×) |
| div_uint | P-521 base | 34,627 vs crypto-bigint 68,848 | −34,221 (0.50×) | 18,428 vs crypto-bigint 52,233 | −33,805 (0.35×) | 49,325 vs crypto-bigint 84,182 | −34,857 (0.59×) | 30,683 vs crypto-bigint 63,807 | −33,124 (0.48×) |
| add, constant time | Mersenne-31 | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.4 vs crypto-bigint 10.8 | −6.4 (0.41×) | 4.4 vs runtime-algebra 10.9 | −6.5 (0.40×) |
| add, constant time | BabyBear | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 3.0 vs crypto-bigint 10.8 | −7.8 (0.28×) | 3.0 vs runtime-algebra 10.9 | −7.9 (0.28×) |
| add, constant time | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add, constant time | BN254 scalar | 23.6 vs halo2curves 36.2 | −12.6 (0.65×) | 16.3 vs halo2curves 20.7 | −4.4 (0.79×) | 20.6 vs halo2curves 39.4 | −18.8 (0.52×) | 26.5 vs halo2curves 35.0 | −8.5 (0.76×) |
| add, constant time | secp256k1 base | 26.3 vs halo2curves 35.1 | −8.8 (0.75×) | 18.2 vs halo2curves 19.4 | −1.2 (0.94×) | 24.5 vs halo2curves 35.3 | −10.8 (0.69×) | 29.0 vs halo2curves 33.0 | −4.0 (0.88×) |
| add, constant time | P-256 base | 23.7 vs k256/p256 30.5 | −6.8 (0.78×) | 18.1 vs k256/p256 17.9 | +0.2 (1.01×) | 22.8 vs halo2curves 33.6 | −10.8 (0.68×) | 28.0 vs k256/p256 31.5 | −3.5 (0.89×) |
| add, constant time | P-256 scalar | 26.9 vs halo2curves 35.5 | −8.6 (0.76×) | 17.9 vs halo2curves 20.6 | −2.7 (0.87×) | 24.1 vs halo2curves 38.1 | −14.0 (0.63×) | 29.5 vs halo2curves 35.0 | −5.5 (0.84×) |
| add, constant time | Tom-256 base | 27.6 vs halo2curves 36.6 | −9.0 (0.75×) | 18.1 vs halo2curves 20.2 | −2.1 (0.90×) | 24.1 vs halo2curves 38.6 | −14.5 (0.62×) | 28.5 vs halo2curves 34.0 | −5.5 (0.84×) |
| add, constant time | BLS12-381 base | 42.9 vs halo2curves 63.5 | −20.6 (0.68×) | 25.7 vs halo2curves 32.5 | −6.8 (0.79×) | 35.4 vs halo2curves 71.9 | −36.5 (0.49×) | 41.1 vs halo2curves 54.0 | −12.9 (0.76×) |
| add, constant time | P-521 base | 76.7 vs crypto-bigint 142 | −65.3 (0.54×) | 46.5 vs crypto-bigint 134 | −87.5 (0.35×) | 77.3 vs crypto-bigint 178 | −101 (0.43×) | 74.0 vs crypto-bigint 175 | −101 (0.42×) |
| sub, constant time | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.6 vs crypto-bigint 4.5 | −1.9 (0.58×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | BabyBear | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.6 vs crypto-bigint 4.5 | −1.9 (0.58×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 22.9 vs halo2curves 24.0 | −1.1 (0.95×) | 16.6 vs halo2curves 15.0 | +1.6 (1.11×) | 21.4 vs halo2curves 23.3 | −1.9 (0.92×) | 26.0 vs halo2curves 24.0 | +2.0 (1.08×) |
| sub, constant time | secp256k1 base | 19.8 vs halo2curves 22.3 | −2.5 (0.89×) | 14.4 vs halo2curves 14.2 | +0.2 (1.01×) | 18.6 vs halo2curves 20.2 | −1.6 (0.92×) | 22.5 vs halo2curves 22.0 | +0.5 (1.02×) |
| sub, constant time | P-256 base | 17.3 vs halo2curves 20.5 | −3.2 (0.84×) | 13.4 vs halo2curves 12.4 | +1.0 (1.08×) | 18.4 vs halo2curves 21.1 | −2.7 (0.87×) | 22.0 vs halo2curves 21.5 | +0.5 (1.02×) |
| sub, constant time | P-256 scalar | 21.2 vs k256/p256 23.2 | −2.0 (0.91×) | 15.0 vs halo2curves 14.4 | +0.6 (1.04×) | 22.2 vs halo2curves 23.1 | −0.9 (0.96×) | 24.0 vs halo2curves 23.0 | +1.0 (1.04×) |
| sub, constant time | Tom-256 base | 21.0 vs halo2curves 23.8 | −2.8 (0.88×) | 15.3 vs halo2curves 15.0 | +0.3 (1.02×) | 20.7 vs halo2curves 23.4 | −2.7 (0.88×) | 24.0 vs halo2curves 24.0 | +0.0 (1.00×) |
| sub, constant time | BLS12-381 base | 37.7 vs halo2curves 43.3 | −5.6 (0.87×) | 24.4 vs halo2curves 23.5 | +0.9 (1.04×) | 34.7 vs halo2curves 44.9 | −10.2 (0.77×) | 37.0 vs halo2curves 37.0 | +0.0 (1.00×) |
| sub, constant time | P-521 base | 56.2 vs crypto-bigint 88.7 | −32.5 (0.63×) | 37.2 vs crypto-bigint 65.0 | −27.8 (0.57×) | 58.0 vs crypto-bigint 124 | −66.0 (0.47×) | 53.0 vs crypto-bigint 128 | −75.0 (0.41×) |
| mul, constant time | Mersenne-31 | 5.1 vs crypto-bigint 29.3 | −24.2 (0.17×) | 5.2 vs crypto-bigint 15.8 | −10.6 (0.33×) | 5.5 vs crypto-bigint 38.8 | −33.3 (0.14×) | 5.5 vs crypto-bigint 19.2 | −13.7 (0.29×) |
| mul, constant time | BabyBear | 7.8 vs crypto-bigint 29.3 | −21.5 (0.27×) | 5.2 vs crypto-bigint 15.8 | −10.6 (0.33×) | 7.8 vs crypto-bigint 39.2 | −31.4 (0.20×) | 5.0 vs crypto-bigint 19.2 | −14.2 (0.26×) |
| mul, constant time | Goldilocks | 12.1 vs winter-math 12.3 | −0.2 (0.98×) | 7.0 vs winter-math 6.5 | +0.5 (1.08×) | 12.4 vs winter-math 17.1 | −4.7 (0.73×) | 9.0 vs winter-math 7.7 | +1.3 (1.17×) |
| mul, constant time | BN254 scalar | 236 vs halo2curves 375 | −139 (0.63×) | 81.1 vs halo2curves 91.4 | −10.3 (0.89×) | 415 vs halo2curves 698 | −283 (0.59×) | 145 vs halo2curves 182 | −37.0 (0.80×) |
| mul, constant time | secp256k1 base | 160 vs halo2curves 303 | −143 (0.53×) | 64.3 vs halo2curves 91.0 | −26.7 (0.71×) | 212 vs halo2curves 594 | −382 (0.36×) | 109 vs halo2curves 163 | −54.0 (0.67×) |
| mul, constant time | P-256 base | 177 vs k256/p256 241 | −64.0 (0.73×) | 72.0 vs k256/p256 85.0 | −13.0 (0.85×) | 281 vs k256/p256 473 | −192 (0.59×) | 120 vs k256/p256 135 | −15.0 (0.89×) |
| mul, constant time | P-256 scalar | 239 vs halo2curves 344 | −105 (0.69×) | 86.6 vs halo2curves 93.5 | −6.9 (0.93×) | 316 vs halo2curves 710 | −394 (0.45×) | 156 vs halo2curves 181 | −25.0 (0.86×) |
| mul, constant time | Tom-256 base | 236 vs halo2curves 344 | −108 (0.69×) | 80.1 vs halo2curves 94.2 | −14.1 (0.85×) | 304 vs halo2curves 608 | −304 (0.50×) | 156 vs halo2curves 176 | −20.0 (0.89×) |
| mul, constant time | BLS12-381 base | 727 vs halo2curves 884 | −157 (0.82×) | 248 vs halo2curves 195 | +53.0 (1.27×) | 916 vs crypto-bigint 1,290 | −374 (0.71×) | 421 vs halo2curves 473 | −52.0 (0.89×) |
| mul, constant time | P-521 base | 1,019 vs crypto-bigint 2,114 | −1,095 (0.48×) | 400 vs crypto-bigint 600 | −200 (0.67×) | 1,643 vs crypto-bigint 2,716 | −1,073 (0.60×) | 847 vs crypto-bigint 1,054 | −207 (0.80×) |
| square, constant time | Mersenne-31 | 5.2 vs crypto-bigint 28.5 | −23.3 (0.18×) | 5.2 vs crypto-bigint 15.6 | −10.4 (0.33×) | 5.4 vs crypto-bigint 38.3 | −32.9 (0.14×) | 5.4 vs crypto-bigint 18.0 | −12.6 (0.30×) |
| square, constant time | BabyBear | 8.3 vs crypto-bigint 28.5 | −20.2 (0.29×) | 4.9 vs crypto-bigint 15.6 | −10.7 (0.31×) | 8.0 vs crypto-bigint 38.2 | −30.2 (0.21×) | 5.8 vs crypto-bigint 18.2 | −12.4 (0.32×) |
| square, constant time | Goldilocks | 10.1 vs winter-math 12.4 | −2.3 (0.81×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 12.0 vs winter-math 17.0 | −5.0 (0.71×) | 9.7 vs winter-math 7.0 | +2.7 (1.39×) |
| square, constant time | BN254 scalar | 213 vs halo2curves 310 | −97.0 (0.69×) | 77.8 vs halo2curves 84.3 | −6.5 (0.92×) | 230 vs halo2curves 543 | −313 (0.42×) | 140 vs halo2curves 144 | −4.0 (0.97×) |
| square, constant time | secp256k1 base | 143 vs halo2curves 215 | −72.0 (0.67×) | 61.2 vs halo2curves 77.9 | −16.7 (0.79×) | 192 vs halo2curves 430 | −238 (0.45×) | 102 vs halo2curves 121 | −19.0 (0.84×) |
| square, constant time | P-256 base | 160 vs halo2curves 200 | −40.0 (0.80×) | 78.4 vs k256/p256 73.6 | +4.8 (1.07×) | 253 vs halo2curves 341 | −88.0 (0.74×) | 113 vs halo2curves 107 | +6.0 (1.06×) |
| square, constant time | P-256 scalar | 214 vs halo2curves 269 | −55.0 (0.80×) | 81.6 vs halo2curves 83.0 | −1.4 (0.98×) | 280 vs halo2curves 554 | −274 (0.51×) | 155 vs halo2curves 140 | +15.0 (1.11×) |
| square, constant time | Tom-256 base | 204 vs halo2curves 317 | −113 (0.64×) | 84.4 vs halo2curves 79.8 | +4.6 (1.06×) | 226 vs halo2curves 494 | −268 (0.46×) | 158 vs halo2curves 134 | +24.0 (1.18×) |
| square, constant time | BLS12-381 base | 577 vs halo2curves 670 | −93.0 (0.86×) | 282 vs halo2curves 163 | +119 (1.73×) | 889 vs halo2curves 1,296 | −407 (0.69×) | 448 vs halo2curves 286 | +162 (1.57×) |
| square, constant time | P-521 base | 559 vs crypto-bigint 2,154 | −1,595 (0.26×) | 262 vs crypto-bigint 614 | −352 (0.43×) | 915 vs crypto-bigint 2,798 | −1,883 (0.33×) | 420 vs crypto-bigint 1,123 | −703 (0.37×) |
| invert, constant time | Mersenne-31 | 438 vs runtime-algebra 3,341 | −2,903 (0.13×) | 438 vs runtime-algebra 3,341 | −2,903 (0.13×) | 430 vs runtime-algebra 3,315 | −2,885 (0.13×) | 428 vs runtime-algebra 3,332 | −2,904 (0.13×) |
| invert, constant time | BabyBear | 520 vs runtime-algebra 3,302 | −2,782 (0.16×) | 459 vs runtime-algebra 3,305 | −2,846 (0.14×) | 550 vs runtime-algebra 3,329 | −2,779 (0.17×) | 417 vs runtime-algebra 3,342 | −2,925 (0.12×) |
| invert, constant time | Goldilocks | 1,261 vs winter-math 1,772 | −511 (0.71×) | 856 vs winter-math 912 | −56.0 (0.94×) | 1,559 vs winter-math 1,837 | −278 (0.85×) | 1,194 vs winter-math 998 | +196 (1.20×) |
| invert, constant time | BN254 scalar | 12,021 vs crypto-bigint 28,421 | −16,400 (0.42×) | 9,515 vs crypto-bigint 25,649 | −16,134 (0.37×) | 16,834 vs crypto-bigint 35,997 | −19,163 (0.47×) | 12,527 vs crypto-bigint 30,794 | −18,267 (0.41×) |
| invert, constant time | secp256k1 base | 11,868 vs crypto-bigint 28,539 | −16,671 (0.42×) | 9,096 vs crypto-bigint 25,562 | −16,466 (0.36×) | 16,751 vs crypto-bigint 35,964 | −19,213 (0.47×) | 12,594 vs crypto-bigint 30,446 | −17,852 (0.41×) |
| invert, constant time | P-256 base | 11,532 vs crypto-bigint 29,241 | −17,709 (0.39×) | 8,923 vs k256/p256 22,180 | −13,257 (0.40×) | 16,376 vs crypto-bigint 36,083 | −19,707 (0.45×) | 12,309 vs crypto-bigint 30,452 | −18,143 (0.40×) |
| invert, constant time | P-256 scalar | 11,880 vs crypto-bigint 28,615 | −16,735 (0.42×) | 9,328 vs crypto-bigint 25,677 | −16,349 (0.36×) | 16,986 vs crypto-bigint 35,927 | −18,941 (0.47×) | 12,736 vs crypto-bigint 30,685 | −17,949 (0.42×) |
| invert, constant time | Tom-256 base | 11,796 vs crypto-bigint 28,767 | −16,971 (0.41×) | 9,293 vs crypto-bigint 25,568 | −16,275 (0.36×) | 16,889 vs crypto-bigint 35,871 | −18,982 (0.47×) | 12,702 vs crypto-bigint 30,692 | −17,990 (0.41×) |
| invert, constant time | BLS12-381 base | 24,008 vs crypto-bigint 49,507 | −25,499 (0.48×) | 16,592 vs crypto-bigint 42,655 | −26,063 (0.39×) | 32,855 vs crypto-bigint 65,893 | −33,038 (0.50×) | 24,079 vs crypto-bigint 53,633 | −29,554 (0.45×) |
| invert, constant time | P-521 base | 41,752 vs crypto-bigint 101,758 | −60,006 (0.41×) | 26,001 vs crypto-bigint 84,120 | −58,119 (0.31×) | 63,515 vs crypto-bigint 122,580 | −59,065 (0.52×) | 41,255 vs crypto-bigint 101,412 | −60,157 (0.41×) |
| sqrt, constant time | Mersenne-31 | 334 vs runtime-algebra 1,741 | −1,407 (0.19×) | 334 vs crypto-bigint 1,637 | −1,303 (0.20×) | 341 vs runtime-algebra 1,747 | −1,406 (0.20×) | 340 vs runtime-algebra 1,679 | −1,339 (0.20×) |
| sqrt, constant time | BabyBear | 7,466 vs crypto-bigint 34,149 | −26,683 (0.22×) | 6,456 vs crypto-bigint 20,193 | −13,737 (0.32×) | 7,849 vs crypto-bigint 38,424 | −30,575 (0.20×) | 6,489 vs crypto-bigint 23,175 | −16,686 (0.28×) |
| sqrt, constant time | Goldilocks | 12,876 vs crypto-bigint 48,789 | −35,913 (0.26×) | 8,887 vs crypto-bigint 28,889 | −20,002 (0.31×) | 15,824 vs crypto-bigint 54,915 | −39,091 (0.29×) | 21,641 vs crypto-bigint 32,998 | −11,357 (0.66×) |
| sqrt, constant time | BN254 scalar | 178,607 vs crypto-bigint 330,187 | −151,580 (0.54×) | 83,838 vs halo2curves 102,211 | −18,373 (0.82×) | 298,605 vs halo2curves 521,248 | −222,643 (0.57×) | 152,636 vs halo2curves 179,561 | −26,925 (0.85×) |
| sqrt, constant time | secp256k1 base | 42,199 vs halo2curves 158,742 | −116,543 (0.27×) | 19,399 vs halo2curves 42,787 | −23,388 (0.45×) | 68,083 vs crypto-bigint 244,994 | −176,911 (0.28×) | 31,139 vs crypto-bigint 79,732 | −48,593 (0.39×) |
| sqrt, constant time | P-256 base | 44,750 vs k256/p256 64,488 | −19,738 (0.69×) | 15,771 vs k256/p256 21,580 | −5,809 (0.73×) | 79,517 vs k256/p256 121,574 | −42,057 (0.65×) | 32,304 vs k256/p256 34,036 | −1,732 (0.95×) |
| sqrt, constant time | P-256 scalar | 78,923 vs halo2curves 153,029 | −74,106 (0.52×) | 37,110 vs halo2curves 49,826 | −12,716 (0.74×) | 120,380 vs crypto-bigint 240,940 | −120,560 (0.50×) | 68,158 vs halo2curves 72,365 | −4,207 (0.94×) |
| sqrt, constant time | Tom-256 base | 65,472 vs crypto-bigint 149,734 | −84,262 (0.44×) | 32,400 vs halo2curves 45,702 | −13,302 (0.71×) | 94,922 vs crypto-bigint 223,204 | −128,282 (0.43×) | 52,551 vs crypto-bigint 74,698 | −22,147 (0.70×) |
| sqrt, constant time | BLS12-381 base | 309,849 vs crypto-bigint 545,084 | −235,235 (0.57×) | 136,709 vs crypto-bigint 145,054 | −8,345 (0.94×) | 422,644 vs crypto-bigint 617,309 | −194,665 (0.68×) | 211,927 vs crypto-bigint 272,471 | −60,544 (0.78×) |
| sqrt, constant time | P-521 base | 325,144 vs crypto-bigint 1,528,245 | −1,203,101 (0.21×) | 135,728 vs crypto-bigint 296,403 | −160,675 (0.46×) | 504,081 vs crypto-bigint 1,565,745 | −1,061,664 (0.32×) | 272,366 vs crypto-bigint 591,360 | −318,994 (0.46×) |
| mul_u64, constant time | Mersenne-31 | 9.4 vs crypto-bigint 61.3 | −51.9 (0.15×) | 9.4 vs crypto-bigint 30.5 | −21.1 (0.31×) | 9.6 vs crypto-bigint 69.6 | −60.0 (0.14×) | 9.7 vs crypto-bigint 38.7 | −29.0 (0.25×) |
| mul_u64, constant time | BabyBear | 17.0 vs crypto-bigint 63.9 | −46.9 (0.27×) | 11.2 vs crypto-bigint 33.0 | −21.8 (0.34×) | 20.1 vs crypto-bigint 86.1 | −66.0 (0.23×) | 14.7 vs crypto-bigint 40.4 | −25.7 (0.36×) |
| mul_u64, constant time | Goldilocks | 19.0 vs winter-math 25.0 | −6.0 (0.76×) | 11.0 vs winter-math 10.8 | +0.2 (1.02×) | 21.1 vs winter-math 32.7 | −11.6 (0.65×) | 17.1 vs winter-math 13.5 | +3.6 (1.27×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 853 | −747 (0.12×) | 38.9 vs crypto-bigint 263 | −224 (0.15×) | 125 vs halo2curves 1,488 | −1,363 (0.08×) | 57.4 vs crypto-bigint 406 | −349 (0.14×) |
| mul_u64, constant time | secp256k1 base | 61.3 vs halo2curves 707 | −646 (0.09×) | 24.9 vs crypto-bigint 262 | −237 (0.10×) | 80.0 vs halo2curves 1,045 | −965 (0.08×) | 37.2 vs crypto-bigint 408 | −371 (0.09×) |
| mul_u64, constant time | P-256 base | 85.3 vs k256/p256 484 | −399 (0.18×) | 40.3 vs k256/p256 150 | −110 (0.27×) | 101 vs k256/p256 916 | −815 (0.11×) | 53.7 vs k256/p256 259 | −205 (0.21×) |
| mul_u64, constant time | P-256 scalar | 102 vs k256/p256 517 | −415 (0.20×) | 43.5 vs k256/p256 249 | −206 (0.17×) | 122 vs k256/p256 818 | −696 (0.15×) | 62.0 vs k256/p256 395 | −333 (0.16×) |
| mul_u64, constant time | Tom-256 base | 95.4 vs crypto-bigint 867 | −772 (0.11×) | 41.8 vs crypto-bigint 262 | −220 (0.16×) | 122 vs halo2curves 1,246 | −1,124 (0.10×) | 57.6 vs crypto-bigint 407 | −349 (0.14×) |
| mul_u64, constant time | BLS12-381 base | 148 vs crypto-bigint 1,920 | −1,772 (0.08×) | 65.8 vs halo2curves 481 | −415 (0.14×) | 190 vs crypto-bigint 2,493 | −2,303 (0.08×) | 85.9 vs crypto-bigint 1,034 | −948 (0.08×) |
| mul_u64, constant time | P-521 base | 138 vs crypto-bigint 3,011 | −2,873 (0.05×) | 69.3 vs crypto-bigint 984 | −915 (0.07×) | 138 vs crypto-bigint 3,977 | −3,839 (0.03×) | 95.7 vs crypto-bigint 1,825 | −1,729 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.4 vs crypto-bigint 60.5 | −51.1 (0.16×) | 9.4 vs crypto-bigint 30.5 | −21.1 (0.31×) | 9.7 vs crypto-bigint 72.3 | −62.6 (0.13×) | 9.7 vs crypto-bigint 38.9 | −29.2 (0.25×) |
| mul_uint, constant time | BabyBear | 17.1 vs crypto-bigint 63.9 | −46.8 (0.27×) | 11.2 vs crypto-bigint 33.1 | −21.9 (0.34×) | 20.1 vs crypto-bigint 85.5 | −65.4 (0.24×) | 14.7 vs crypto-bigint 40.5 | −25.8 (0.36×) |
| mul_uint, constant time | Goldilocks | 19.1 vs winter-math 24.9 | −5.8 (0.77×) | 11.0 vs winter-math 10.8 | +0.2 (1.02×) | 21.1 vs winter-math 32.7 | −11.6 (0.65×) | 17.1 vs winter-math 13.6 | +3.5 (1.26×) |
| mul_uint, constant time | BN254 scalar | 487 vs halo2curves 756 | −269 (0.64×) | 246 vs halo2curves 181 | +65.0 (1.36×) | 597 vs halo2curves 1,347 | −750 (0.44×) | 312 vs halo2curves 349 | −37.0 (0.89×) |
| mul_uint, constant time | secp256k1 base | 322 vs halo2curves 519 | −197 (0.62×) | 192 vs halo2curves 154 | +38.0 (1.25×) | 380 vs halo2curves 971 | −591 (0.39×) | 251 vs halo2curves 265 | −14.0 (0.95×) |
| mul_uint, constant time | P-256 base | 412 vs halo2curves 456 | −44.0 (0.90×) | 268 vs halo2curves 143 | +125 (1.87×) | 492 vs halo2curves 994 | −502 (0.49×) | 301 vs halo2curves 252 | +49.0 (1.19×) |
| mul_uint, constant time | P-256 scalar | 471 vs k256/p256 549 | −78.0 (0.86×) | 261 vs halo2curves 183 | +78.0 (1.43×) | 562 vs k256/p256 886 | −324 (0.63×) | 336 vs halo2curves 347 | −11.0 (0.97×) |
| mul_uint, constant time | Tom-256 base | 465 vs halo2curves 692 | −227 (0.67×) | 259 vs halo2curves 180 | +79.0 (1.44×) | 562 vs halo2curves 1,183 | −621 (0.48×) | 325 vs halo2curves 335 | −10.0 (0.97×) |
| mul_uint, constant time | BLS12-381 base | 1,084 vs crypto-bigint 1,903 | −819 (0.57×) | 513 vs halo2curves 382 | +131 (1.34×) | 1,694 vs crypto-bigint 2,501 | −807 (0.68×) | 689 vs crypto-bigint 940 | −251 (0.73×) |
| mul_uint, constant time | P-521 base | 1,993 vs crypto-bigint 3,010 | −1,017 (0.66×) | 974 vs crypto-bigint 1,008 | −34.0 (0.97×) | 2,923 vs crypto-bigint 3,978 | −1,055 (0.73×) | 1,496 vs crypto-bigint 1,899 | −403 (0.79×) |
| add_u64, constant time | Mersenne-31 | 8.1 vs crypto-bigint 36.1 | −28.0 (0.22×) | 8.1 vs crypto-bigint 30.4 | −22.3 (0.27×) | 9.1 vs crypto-bigint 40.2 | −31.1 (0.23×) | 9.2 vs crypto-bigint 30.7 | −21.5 (0.30×) |
| add_u64, constant time | BabyBear | 10.9 vs crypto-bigint 39.8 | −28.9 (0.27×) | 8.6 vs crypto-bigint 31.6 | −23.0 (0.27×) | 13.0 vs crypto-bigint 52.4 | −39.4 (0.25×) | 12.5 vs crypto-bigint 31.8 | −19.3 (0.39×) |
| add_u64, constant time | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 10.2 | +1.8 (1.18×) |
| add_u64, constant time | BN254 scalar | 135 vs crypto-bigint 506 | −371 (0.27×) | 58.3 vs crypto-bigint 221 | −163 (0.26×) | 163 vs halo2curves 797 | −634 (0.20×) | 81.4 vs crypto-bigint 311 | −230 (0.26×) |
| add_u64, constant time | secp256k1 base | 20.5 vs halo2curves 424 | −404 (0.05×) | 16.1 vs crypto-bigint 221 | −205 (0.07×) | 20.2 vs halo2curves 517 | −497 (0.04×) | 25.1 vs halo2curves 270 | −245 (0.09×) |
| add_u64, constant time | P-256 base | 90.1 vs k256/p256 275 | −185 (0.33×) | 58.7 vs k256/p256 96.6 | −37.9 (0.61×) | 104 vs k256/p256 491 | −387 (0.21×) | 71.5 vs k256/p256 159 | −87.5 (0.45×) |
| add_u64, constant time | P-256 scalar | 89.8 vs k256/p256 41.7 | +48.1 (2.15×) | 56.8 vs k256/p256 40.9 | +15.9 (1.39×) | 100 vs k256/p256 54.6 | +45.4 (1.83×) | 77.1 vs k256/p256 54.7 | +22.4 (1.41×) |
| add_u64, constant time | Tom-256 base | 115 vs crypto-bigint 505 | −390 (0.23×) | 62.6 vs crypto-bigint 221 | −158 (0.28×) | 132 vs halo2curves 737 | −605 (0.18×) | 85.2 vs crypto-bigint 313 | −228 (0.27×) |
| add_u64, constant time | BLS12-381 base | 192 vs crypto-bigint 1,081 | −889 (0.18×) | 93.2 vs halo2curves 323 | −230 (0.29×) | 236 vs crypto-bigint 1,384 | −1,148 (0.17×) | 127 vs halo2curves 632 | −505 (0.20×) |
| add_u64, constant time | P-521 base | 64.6 vs crypto-bigint 1,668 | −1,603 (0.04×) | 44.1 vs crypto-bigint 649 | −605 (0.07×) | 74.7 vs crypto-bigint 2,182 | −2,107 (0.03×) | 64.1 vs crypto-bigint 1,127 | −1,063 (0.06×) |
| add_uint, constant time | Mersenne-31 | 8.1 vs crypto-bigint 36.1 | −28.0 (0.22×) | 8.1 vs crypto-bigint 30.4 | −22.3 (0.27×) | 9.1 vs crypto-bigint 40.2 | −31.1 (0.23×) | 9.2 vs crypto-bigint 30.4 | −21.2 (0.30×) |
| add_uint, constant time | BabyBear | 10.9 vs crypto-bigint 39.8 | −28.9 (0.27×) | 8.6 vs crypto-bigint 31.7 | −23.1 (0.27×) | 13.0 vs crypto-bigint 52.4 | −39.4 (0.25×) | 12.5 vs crypto-bigint 31.8 | −19.3 (0.39×) |
| add_uint, constant time | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 10.2 | +1.8 (1.18×) |
| add_uint, constant time | BN254 scalar | 327 vs halo2curves 412 | −85.0 (0.79×) | 131 vs halo2curves 113 | +18.0 (1.16×) | 429 vs halo2curves 675 | −246 (0.64×) | 216 vs halo2curves 205 | +11.0 (1.05×) |
| add_uint, constant time | secp256k1 base | 41.3 vs halo2curves 223 | −182 (0.19×) | 29.4 vs halo2curves 85.4 | −56.0 (0.34×) | 37.5 vs halo2curves 425 | −388 (0.09×) | 44.4 vs halo2curves 140 | −95.6 (0.32×) |
| add_uint, constant time | P-256 base | 235 vs halo2curves 244 | −9.0 (0.96×) | 116 vs halo2curves 91.5 | +24.5 (1.27×) | 300 vs halo2curves 513 | −213 (0.58×) | 168 vs halo2curves 153 | +15.0 (1.10×) |
| add_uint, constant time | P-256 scalar | 302 vs k256/p256 94.6 | +207 (3.19×) | 129 vs k256/p256 94.8 | +34.2 (1.36×) | 364 vs k256/p256 120 | +244 (3.03×) | 192 vs k256/p256 117 | +75.0 (1.64×) |
| add_uint, constant time | Tom-256 base | 300 vs halo2curves 378 | −78.0 (0.79×) | 114 vs halo2curves 110 | +4.0 (1.04×) | 365 vs halo2curves 622 | −257 (0.59×) | 205 vs halo2curves 196 | +9.0 (1.05×) |
| add_uint, constant time | BLS12-381 base | 843 vs halo2curves 979 | −136 (0.86×) | 361 vs halo2curves 225 | +136 (1.60×) | 1,051 vs crypto-bigint 1,386 | −335 (0.76×) | 551 vs halo2curves 424 | +127 (1.30×) |
| add_uint, constant time | P-521 base | 135 vs crypto-bigint 1,693 | −1,558 (0.08×) | 105 vs crypto-bigint 666 | −561 (0.16×) | 142 vs crypto-bigint 2,189 | −2,047 (0.06×) | 138 vs crypto-bigint 1,193 | −1,055 (0.12×) |
| sub_u64, constant time | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.0 vs crypto-bigint 30.7 | −23.7 (0.23×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_u64, constant time | BabyBear | 11.0 vs crypto-bigint 27.9 | −16.9 (0.39×) | 8.5 vs crypto-bigint 19.5 | −11.0 (0.44×) | 15.8 vs crypto-bigint 42.2 | −26.4 (0.37×) | 12.5 vs crypto-bigint 21.8 | −9.3 (0.57×) |
| sub_u64, constant time | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 10.4 vs winter-math 19.0 | −8.6 (0.55×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_u64, constant time | BN254 scalar | 125 vs crypto-bigint 499 | −374 (0.25×) | 60.0 vs crypto-bigint 151 | −91.0 (0.40×) | 155 vs halo2curves 770 | −615 (0.20×) | 80.1 vs crypto-bigint 257 | −177 (0.31×) |
| sub_u64, constant time | secp256k1 base | 15.5 vs halo2curves 413 | −398 (0.04×) | 12.3 vs crypto-bigint 150 | −138 (0.08×) | 14.6 vs halo2curves 500 | −485 (0.03×) | 18.5 vs crypto-bigint 258 | −240 (0.07×) |
| sub_u64, constant time | P-256 base | 84.7 vs k256/p256 264 | −179 (0.32×) | 52.4 vs k256/p256 89.4 | −37.0 (0.59×) | 96.3 vs k256/p256 488 | −392 (0.20×) | 68.0 vs k256/p256 149 | −81.0 (0.46×) |
| sub_u64, constant time | P-256 scalar | 80.6 vs k256/p256 21.2 | +59.4 (3.80×) | 52.5 vs k256/p256 21.2 | +31.3 (2.48×) | 93.7 vs k256/p256 24.5 | +69.2 (3.82×) | 77.5 vs k256/p256 24.2 | +53.3 (3.20×) |
| sub_u64, constant time | Tom-256 base | 113 vs crypto-bigint 493 | −380 (0.23×) | 56.4 vs crypto-bigint 151 | −94.6 (0.37×) | 124 vs halo2curves 716 | −592 (0.17×) | 80.4 vs crypto-bigint 256 | −176 (0.31×) |
| sub_u64, constant time | BLS12-381 base | 183 vs crypto-bigint 1,083 | −900 (0.17×) | 86.5 vs halo2curves 308 | −222 (0.28×) | 242 vs crypto-bigint 1,358 | −1,116 (0.18×) | 118 vs halo2curves 612 | −494 (0.19×) |
| sub_u64, constant time | P-521 base | 42.2 vs crypto-bigint 1,673 | −1,631 (0.03×) | 34.1 vs crypto-bigint 622 | −588 (0.05×) | 52.3 vs crypto-bigint 2,159 | −2,107 (0.02×) | 44.2 vs crypto-bigint 1,139 | −1,095 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.0 vs crypto-bigint 30.7 | −23.7 (0.23×) | 7.1 vs crypto-bigint 20.9 | −13.8 (0.34×) |
| sub_uint, constant time | BabyBear | 11.1 vs crypto-bigint 27.9 | −16.8 (0.40×) | 8.5 vs crypto-bigint 19.5 | −11.0 (0.44×) | 15.8 vs crypto-bigint 42.2 | −26.4 (0.37×) | 12.5 vs crypto-bigint 21.8 | −9.3 (0.57×) |
| sub_uint, constant time | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 10.4 vs winter-math 19.0 | −8.6 (0.55×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_uint, constant time | BN254 scalar | 320 vs halo2curves 401 | −81.0 (0.80×) | 137 vs halo2curves 104 | +33.0 (1.32×) | 412 vs halo2curves 654 | −242 (0.63×) | 218 vs halo2curves 194 | +24.0 (1.12×) |
| sub_uint, constant time | secp256k1 base | 33.6 vs halo2curves 215 | −181 (0.16×) | 26.0 vs halo2curves 108 | −82.0 (0.24×) | 34.4 vs halo2curves 411 | −377 (0.08×) | 38.1 vs halo2curves 130 | −91.9 (0.29×) |
| sub_uint, constant time | P-256 base | 229 vs halo2curves 233 | −4.0 (0.98×) | 106 vs halo2curves 85.7 | +20.3 (1.24×) | 302 vs halo2curves 499 | −197 (0.61×) | 163 vs halo2curves 144 | +19.0 (1.13×) |
| sub_uint, constant time | P-256 scalar | 295 vs k256/p256 67.8 | +227 (4.35×) | 122 vs k256/p256 70.4 | +51.6 (1.73×) | 364 vs k256/p256 87.6 | +276 (4.16×) | 208 vs k256/p256 89.5 | +119 (2.32×) |
| sub_uint, constant time | Tom-256 base | 300 vs halo2curves 364 | −64.0 (0.82×) | 129 vs halo2curves 104 | +25.0 (1.24×) | 343 vs halo2curves 607 | −264 (0.57×) | 202 vs halo2curves 187 | +15.0 (1.08×) |
| sub_uint, constant time | BLS12-381 base | 828 vs halo2curves 972 | −144 (0.85×) | 364 vs halo2curves 216 | +148 (1.69×) | 1,034 vs crypto-bigint 1,355 | −321 (0.76×) | 547 vs halo2curves 407 | +140 (1.34×) |
| sub_uint, constant time | P-521 base | 127 vs crypto-bigint 1,677 | −1,550 (0.08×) | 86.9 vs crypto-bigint 635 | −548 (0.14×) | 127 vs crypto-bigint 2,168 | −2,041 (0.06×) | 114 vs crypto-bigint 1,164 | −1,050 (0.10×) |
| from_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 18.0 | −13.3 (0.26×) | 4.7 vs crypto-bigint 11.6 | −6.9 (0.41×) | 5.2 vs crypto-bigint 23.9 | −18.7 (0.22×) | 5.2 vs crypto-bigint 13.2 | −8.0 (0.39×) |
| from_u64, constant time | BabyBear | 13.5 vs crypto-bigint 21.6 | −8.1 (0.62×) | 10.3 vs crypto-bigint 12.6 | −2.3 (0.82×) | 14.0 vs crypto-bigint 35.1 | −21.1 (0.40×) | 12.7 vs crypto-bigint 15.0 | −2.3 (0.85×) |
| from_u64, constant time | Goldilocks | 14.9 vs winter-math 10.6 | +4.3 (1.41×) | 9.3 vs winter-math 5.9 | +3.4 (1.58×) | 17.4 vs winter-math 17.0 | +0.4 (1.02×) | 12.5 vs winter-math 7.0 | +5.5 (1.79×) |
| from_u64, constant time | BN254 scalar | 201 vs crypto-bigint 433 | −232 (0.46×) | 81.0 vs crypto-bigint 132 | −51.0 (0.61×) | 270 vs halo2curves 723 | −453 (0.37×) | 119 vs crypto-bigint 204 | −85.0 (0.58×) |
| from_u64, constant time | secp256k1 base | 3.4 vs k256/p256 6.0 | −2.6 (0.57×) | 3.4 vs k256/p256 6.0 | −2.6 (0.57×) | 3.4 vs k256/p256 5.9 | −2.5 (0.58×) | 3.4 vs k256/p256 6.0 | −2.6 (0.57×) |
| from_u64, constant time | P-256 base | 179 vs k256/p256 237 | −58.0 (0.76×) | 49.9 vs k256/p256 76.0 | −26.1 (0.66×) | 211 vs k256/p256 465 | −254 (0.45×) | 84.0 vs k256/p256 131 | −47.0 (0.64×) |
| from_u64, constant time | P-256 scalar | 249 vs k256/p256 4.0 | +245 (62.25×) | 81.5 vs k256/p256 4.0 | +77.5 (20.38×) | 268 vs k256/p256 4.0 | +264 (67.00×) | 117 vs k256/p256 4.0 | +113 (29.25×) |
| from_u64, constant time | Tom-256 base | 249 vs crypto-bigint 430 | −181 (0.58×) | 79.0 vs crypto-bigint 133 | −54.0 (0.59×) | 265 vs halo2curves 658 | −393 (0.40×) | 113 vs crypto-bigint 204 | −91.0 (0.55×) |
| from_u64, constant time | BLS12-381 base | 433 vs crypto-bigint 894 | −461 (0.48×) | 155 vs crypto-bigint 254 | −99.0 (0.61×) | 605 vs crypto-bigint 1,256 | −651 (0.48×) | 273 vs crypto-bigint 516 | −243 (0.53×) |
| from_u64, constant time | P-521 base | 6.0 vs crypto-bigint 1,460 | −1,454 (0.00×) | 6.0 vs crypto-bigint 506 | −500 (0.01×) | 6.0 vs crypto-bigint 1,964 | −1,958 (0.00×) | 6.0 vs crypto-bigint 889 | −883 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,356 vs crypto-bigint 28,523 | −16,167 (0.43×) | 9,500 vs crypto-bigint 24,187 | −14,687 (0.39×) | 17,229 vs crypto-bigint 37,822 | −20,593 (0.46×) | 12,948 vs crypto-bigint 31,291 | −18,343 (0.41×) |
| div_u64, constant time | secp256k1 base | 11,975 vs crypto-bigint 28,202 | −16,227 (0.42×) | 9,072 vs crypto-bigint 24,339 | −15,267 (0.37×) | 17,026 vs crypto-bigint 37,822 | −20,796 (0.45×) | 12,853 vs crypto-bigint 31,287 | −18,434 (0.41×) |
| div_u64, constant time | P-256 base | 11,826 vs crypto-bigint 28,340 | −16,514 (0.42×) | 9,157 vs k256/p256 22,360 | −13,203 (0.41×) | 17,317 vs crypto-bigint 37,757 | −20,440 (0.46×) | 12,501 vs crypto-bigint 31,256 | −18,755 (0.40×) |
| div_u64, constant time | P-256 scalar | 12,215 vs crypto-bigint 28,419 | −16,204 (0.43×) | 9,471 vs crypto-bigint 24,270 | −14,799 (0.39×) | 17,336 vs crypto-bigint 37,769 | −20,433 (0.46×) | 13,041 vs crypto-bigint 30,891 | −17,850 (0.42×) |
| div_u64, constant time | Tom-256 base | 12,193 vs crypto-bigint 28,484 | −16,291 (0.43×) | 9,268 vs crypto-bigint 24,352 | −15,084 (0.38×) | 16,891 vs crypto-bigint 37,679 | −20,788 (0.45×) | 12,990 vs crypto-bigint 31,209 | −18,219 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,787 vs crypto-bigint 51,013 | −26,226 (0.49×) | 16,844 vs crypto-bigint 43,619 | −26,775 (0.39×) | 34,605 vs crypto-bigint 67,015 | −32,410 (0.52×) | 24,119 vs crypto-bigint 55,120 | −31,001 (0.44×) |
| div_u64, constant time | P-521 base | 42,839 vs crypto-bigint 107,812 | −64,973 (0.40×) | 26,172 vs crypto-bigint 84,766 | −58,594 (0.31×) | 65,412 vs crypto-bigint 128,218 | −62,806 (0.51×) | 43,437 vs crypto-bigint 101,291 | −57,854 (0.43×) |
| div_uint, constant time | Mersenne-31 | 455 vs runtime-algebra 3,507 | −3,052 (0.13×) | 455 vs runtime-algebra 3,506 | −3,051 (0.13×) | 460 vs runtime-algebra 3,416 | −2,956 (0.13×) | 465 vs runtime-algebra 3,417 | −2,952 (0.14×) |
| div_uint, constant time | BabyBear | 546 vs runtime-algebra 3,567 | −3,021 (0.15×) | 466 vs runtime-algebra 3,535 | −3,069 (0.13×) | 611 vs runtime-algebra 3,419 | −2,808 (0.18×) | 459 vs runtime-algebra 3,417 | −2,958 (0.13×) |
| div_uint, constant time | Goldilocks | 1,313 vs winter-math 1,825 | −512 (0.72×) | 948 vs winter-math 946 | +2.0 (1.00×) | 1,581 vs winter-math 1,894 | −313 (0.83×) | 1,212 vs winter-math 1,047 | +165 (1.16×) |
| div_uint, constant time | BN254 scalar | 12,497 vs crypto-bigint 28,328 | −15,831 (0.44×) | 9,578 vs crypto-bigint 24,876 | −15,298 (0.39×) | 17,677 vs crypto-bigint 37,929 | −20,252 (0.47×) | 13,063 vs crypto-bigint 31,379 | −18,316 (0.42×) |
| div_uint, constant time | secp256k1 base | 11,942 vs crypto-bigint 28,433 | −16,491 (0.42×) | 9,100 vs crypto-bigint 24,783 | −15,683 (0.37×) | 16,863 vs crypto-bigint 37,610 | −20,747 (0.45×) | 12,854 vs crypto-bigint 31,295 | −18,441 (0.41×) |
| div_uint, constant time | P-256 base | 12,021 vs crypto-bigint 28,742 | −16,721 (0.42×) | 9,104 vs k256/p256 22,145 | −13,041 (0.41×) | 17,544 vs crypto-bigint 37,709 | −20,165 (0.47×) | 12,643 vs crypto-bigint 31,191 | −18,548 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,289 vs crypto-bigint 28,661 | −16,372 (0.43×) | 9,507 vs crypto-bigint 24,868 | −15,361 (0.38×) | 17,527 vs crypto-bigint 37,836 | −20,309 (0.46×) | 13,515 vs crypto-bigint 31,291 | −17,776 (0.43×) |
| div_uint, constant time | Tom-256 base | 12,387 vs crypto-bigint 28,962 | −16,575 (0.43×) | 9,458 vs crypto-bigint 24,963 | −15,505 (0.38×) | 16,923 vs crypto-bigint 37,794 | −20,871 (0.45×) | 13,094 vs crypto-bigint 31,268 | −18,174 (0.42×) |
| div_uint, constant time | BLS12-381 base | 25,420 vs crypto-bigint 51,141 | −25,721 (0.50×) | 16,995 vs crypto-bigint 43,537 | −26,542 (0.39×) | 35,347 vs crypto-bigint 66,970 | −31,623 (0.53×) | 24,508 vs crypto-bigint 54,952 | −30,444 (0.45×) |
| div_uint, constant time | P-521 base | 42,601 vs crypto-bigint 107,498 | −64,897 (0.40×) | 26,251 vs crypto-bigint 121,034 | −94,783 (0.22×) | 65,267 vs crypto-bigint 128,603 | −63,336 (0.51×) | 43,462 vs crypto-bigint 101,264 | −57,802 (0.43×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) | 7.7 vs runtime-algebra 17.8 | −10.1 (0.43×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add | BN254 scalar | 14.7 vs ark-ff 13.3 | +1.4 (1.11×) | 10.9 vs ark-ff 11.1 | −0.2 (0.98×) | 14.7 vs ark-ff 13.6 | +1.1 (1.08×) | 17.4 vs ark-ff 17.4 | +0.0 (1.00×) |
| add | secp256k1 base | 13.7 vs ark-ff 15.5 | −1.8 (0.88×) | 10.7 vs ark-ff 11.2 | −0.5 (0.96×) | 13.9 vs ark-ff 14.8 | −0.9 (0.94×) | 18.1 vs ark-ff 17.9 | +0.2 (1.01×) |
| add | P-256 base | 12.9 vs ark-ff 15.2 | −2.3 (0.85×) | 11.2 vs ark-ff 11.3 | −0.1 (0.99×) | 14.4 vs ark-ff 15.7 | −1.3 (0.92×) | 18.4 vs ark-ff 18.8 | −0.4 (0.98×) |
| add | P-256 scalar | 14.3 vs ark-ff 15.2 | −0.9 (0.94×) | 11.0 vs ark-ff 11.2 | −0.2 (0.98×) | 14.7 vs ark-ff 15.5 | −0.8 (0.95×) | 18.9 vs ark-ff 17.9 | +1.0 (1.06×) |
| add | Tom-256 base | 14.3 vs halo2curves 42.9 | −28.6 (0.33×) | 11.2 vs halo2curves 24.0 | −12.8 (0.47×) | 14.5 vs halo2curves 43.0 | −28.5 (0.34×) | 19.4 vs halo2curves 32.5 | −13.1 (0.60×) |
| add | BLS12-381 base | 25.0 vs ark-ff 43.9 | −18.9 (0.57×) | 18.8 vs halo2curves 34.1 | −15.3 (0.55×) | 24.8 vs ark-ff 45.4 | −20.6 (0.55×) | 29.8 vs ark-ff 44.9 | −15.1 (0.66×) |
| add | P-521 base | 46.0 vs crypto-bigint 183 | −137 (0.25×) | 28.9 vs crypto-bigint 179 | −150 (0.16×) | 46.6 vs crypto-bigint 247 | −200 (0.19×) | 47.5 vs crypto-bigint 243 | −196 (0.20×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 14.6 vs ark-ff 16.5 | −1.9 (0.88×) | 10.8 vs ark-ff 14.0 | −3.2 (0.77×) | 14.5 vs ark-ff 17.0 | −2.5 (0.85×) | 18.4 vs ark-ff 21.4 | −3.0 (0.86×) |
| sub | secp256k1 base | 16.5 vs ark-ff 16.6 | −0.1 (0.99×) | 10.7 vs ark-ff 14.0 | −3.3 (0.76×) | 13.5 vs ark-ff 17.1 | −3.6 (0.79×) | 17.8 vs halo2curves 21.0 | −3.2 (0.85×) |
| sub | P-256 base | 14.6 vs ark-ff 15.5 | −0.9 (0.94×) | 10.5 vs ark-ff 12.9 | −2.4 (0.81×) | 12.7 vs ark-ff 15.3 | −2.6 (0.83×) | 17.4 vs k256/p256 20.0 | −2.6 (0.87×) |
| sub | P-256 scalar | 15.2 vs ark-ff 16.4 | −1.2 (0.93×) | 10.7 vs ark-ff 13.9 | −3.2 (0.77×) | 14.0 vs ark-ff 16.5 | −2.5 (0.85×) | 18.4 vs halo2curves 21.5 | −3.1 (0.86×) |
| sub | Tom-256 base | 14.9 vs halo2curves 26.0 | −11.1 (0.57×) | 10.7 vs halo2curves 14.9 | −4.2 (0.72×) | 13.7 vs halo2curves 26.4 | −12.7 (0.52×) | 18.3 vs halo2curves 22.0 | −3.7 (0.83×) |
| sub | BLS12-381 base | 25.3 vs ark-ff 30.6 | −5.3 (0.83×) | 17.5 vs ark-ff 23.8 | −6.3 (0.74×) | 24.8 vs ark-ff 30.8 | −6.0 (0.81×) | 30.4 vs halo2curves 36.1 | −5.7 (0.84×) |
| sub | P-521 base | 43.5 vs crypto-bigint 159 | −116 (0.27×) | 27.1 vs crypto-bigint 145 | −118 (0.19×) | 43.1 vs crypto-bigint 212 | −169 (0.20×) | 45.6 vs crypto-bigint 171 | −125 (0.27×) |
| mul | Mersenne-31 | 12.0 vs crypto-bigint 46.5 | −34.5 (0.26×) | 12.0 vs crypto-bigint 26.9 | −14.9 (0.45×) | 12.0 vs crypto-bigint 50.0 | −38.0 (0.24×) | 12.0 vs crypto-bigint 30.8 | −18.8 (0.39×) |
| mul | BabyBear | 13.0 vs plonky3 12.2 | +0.8 (1.07×) | 11.0 vs plonky3 12.2 | −1.2 (0.90×) | 14.5 vs plonky3 13.0 | +1.5 (1.12×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| mul | Goldilocks | 18.0 vs winter-math 18.0 | +0.0 (1.00×) | 14.4 vs winter-math 14.5 | −0.1 (0.99×) | 22.0 vs winter-math 19.0 | +3.0 (1.16×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul | BN254 scalar | 228 vs ark-ff 316 | −88.0 (0.72×) | 90.8 vs halo2curves 93.5 | −2.7 (0.97×) | 365 vs ark-ff 380 | −15.0 (0.96×) | 151 vs halo2curves 178 | −27.0 (0.85×) |
| mul | secp256k1 base | 172 vs ark-ff 276 | −104 (0.62×) | 77.5 vs halo2curves 90.3 | −12.8 (0.86×) | 221 vs ark-ff 396 | −175 (0.56×) | 109 vs halo2curves 160 | −51.0 (0.68×) |
| mul | P-256 base | 190 vs k256/p256 241 | −51.0 (0.79×) | 72.5 vs halo2curves 75.6 | −3.1 (0.96×) | 292 vs k256/p256 461 | −169 (0.63×) | 118 vs k256/p256 131 | −13.0 (0.90×) |
| mul | P-256 scalar | 235 vs ark-ff 309 | −74.0 (0.76×) | 93.4 vs halo2curves 95.4 | −2.0 (0.98×) | 320 vs ark-ff 409 | −89.0 (0.78×) | 164 vs halo2curves 179 | −15.0 (0.92×) |
| mul | Tom-256 base | 236 vs halo2curves 342 | −106 (0.69×) | 87.0 vs halo2curves 94.2 | −7.2 (0.92×) | 312 vs halo2curves 588 | −276 (0.53×) | 160 vs halo2curves 171 | −11.0 (0.94×) |
| mul | BLS12-381 base | 722 vs ark-ff 683 | +39.0 (1.06×) | 244 vs halo2curves 193 | +51.0 (1.26×) | 908 vs ark-ff 1,053 | −145 (0.86×) | 430 vs halo2curves 376 | +54.0 (1.14×) |
| mul | P-521 base | 1,062 vs crypto-bigint 2,118 | −1,056 (0.50×) | 416 vs crypto-bigint 617 | −201 (0.67×) | 1,605 vs crypto-bigint 2,780 | −1,175 (0.58×) | 865 vs crypto-bigint 1,104 | −239 (0.78×) |
| square | Mersenne-31 | 12.0 vs crypto-bigint 45.0 | −33.0 (0.27×) | 12.0 vs crypto-bigint 27.0 | −15.0 (0.44×) | 12.0 vs crypto-bigint 47.7 | −35.7 (0.25×) | 12.0 vs crypto-bigint 30.1 | −18.1 (0.40×) |
| square | BabyBear | 13.0 vs plonky3 12.1 | +0.9 (1.07×) | 11.0 vs plonky3 11.9 | −0.9 (0.92×) | 14.0 vs plonky3 13.0 | +1.0 (1.08×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| square | Goldilocks | 17.3 vs winter-math 20.0 | −2.7 (0.86×) | 12.0 vs winter-math 12.8 | −0.8 (0.94×) | 21.2 vs winter-math 20.0 | +1.2 (1.06×) | 14.4 vs winter-math 14.3 | +0.1 (1.01×) |
| square | BN254 scalar | 206 vs ark-ff 263 | −57.0 (0.78×) | 90.0 vs halo2curves 95.8 | −5.8 (0.94×) | 363 vs ark-ff 345 | +18.0 (1.05×) | 141 vs halo2curves 144 | −3.0 (0.98×) |
| square | secp256k1 base | 154 vs halo2curves 222 | −68.0 (0.69×) | 69.9 vs halo2curves 84.1 | −14.2 (0.83×) | 198 vs ark-ff 298 | −100 (0.66×) | 82.9 vs halo2curves 122 | −39.1 (0.68×) |
| square | P-256 base | 155 vs halo2curves 178 | −23.0 (0.87×) | 62.2 vs halo2curves 61.3 | +0.9 (1.01×) | 225 vs ark-ff 330 | −105 (0.68×) | 104 vs halo2curves 107 | −3.0 (0.97×) |
| square | P-256 scalar | 217 vs ark-ff 259 | −42.0 (0.84×) | 89.5 vs halo2curves 92.5 | −3.0 (0.97×) | 240 vs ark-ff 343 | −103 (0.70×) | 150 vs halo2curves 140 | +10.0 (1.07×) |
| square | Tom-256 base | 201 vs halo2curves 267 | −66.0 (0.75×) | 83.0 vs halo2curves 89.5 | −6.5 (0.93×) | 234 vs halo2curves 474 | −240 (0.49×) | 142 vs halo2curves 136 | +6.0 (1.04×) |
| square | BLS12-381 base | 609 vs ark-ff 620 | −11.0 (0.98×) | 281 vs halo2curves 164 | +117 (1.71×) | 856 vs ark-ff 795 | +61.0 (1.08×) | 441 vs halo2curves 296 | +145 (1.49×) |
| square | P-521 base | 571 vs crypto-bigint 2,164 | −1,593 (0.26×) | 263 vs crypto-bigint 613 | −350 (0.43×) | 908 vs crypto-bigint 2,812 | −1,904 (0.32×) | 415 vs crypto-bigint 1,201 | −786 (0.35×) |
| mul_u64 | Mersenne-31 | 14.0 vs crypto-bigint 59.5 | −45.5 (0.24×) | 14.0 vs crypto-bigint 31.4 | −17.4 (0.45×) | 14.0 vs crypto-bigint 71.6 | −57.6 (0.20×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_u64 | BabyBear | 17.0 vs plonky3 34.2 | −17.2 (0.50×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 17.1 vs plonky3 45.0 | −27.9 (0.38×) | 13.1 vs plonky3 16.6 | −3.5 (0.79×) |
| mul_u64 | Goldilocks | 20.0 vs winter-math 23.6 | −3.6 (0.85×) | 12.0 vs winter-math 13.7 | −1.7 (0.88×) | 25.5 vs winter-math 32.1 | −6.6 (0.79×) | 17.1 vs winter-math 15.2 | +1.9 (1.13×) |
| mul_u64 | BN254 scalar | 91.7 vs ark-ff 659 | −567 (0.14×) | 34.5 vs crypto-bigint 262 | −228 (0.13×) | 120 vs ark-ff 811 | −691 (0.15×) | 45.1 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | secp256k1 base | 49.0 vs ark-ff 435 | −386 (0.11×) | 16.0 vs crypto-bigint 259 | −243 (0.06×) | 65.0 vs ark-ff 613 | −548 (0.11×) | 23.0 vs crypto-bigint 409 | −386 (0.06×) |
| mul_u64 | P-256 base | 75.0 vs k256/p256 487 | −412 (0.15×) | 33.0 vs k256/p256 151 | −118 (0.22×) | 89.1 vs ark-ff 848 | −759 (0.11×) | 41.8 vs k256/p256 258 | −216 (0.16×) |
| mul_u64 | P-256 scalar | 82.1 vs k256/p256 513 | −431 (0.16×) | 34.6 vs k256/p256 255 | −220 (0.14×) | 109 vs k256/p256 814 | −705 (0.13×) | 47.4 vs k256/p256 340 | −293 (0.14×) |
| mul_u64 | Tom-256 base | 84.9 vs crypto-bigint 843 | −758 (0.10×) | 33.0 vs crypto-bigint 259 | −226 (0.13×) | 105 vs halo2curves 1,284 | −1,179 (0.08×) | 45.3 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | BLS12-381 base | 123 vs ark-ff 1,627 | −1,504 (0.08×) | 47.8 vs halo2curves 486 | −438 (0.10×) | 158 vs ark-ff 2,137 | −1,979 (0.07×) | 62.1 vs crypto-bigint 1,038 | −976 (0.06×) |
| mul_u64 | P-521 base | 93.3 vs crypto-bigint 3,025 | −2,932 (0.03×) | 39.6 vs crypto-bigint 994 | −954 (0.04×) | 161 vs crypto-bigint 3,994 | −3,833 (0.04×) | 52.7 vs crypto-bigint 1,844 | −1,791 (0.03×) |
| mul_uint | Mersenne-31 | 14.0 vs crypto-bigint 59.6 | −45.6 (0.23×) | 14.0 vs crypto-bigint 31.5 | −17.5 (0.44×) | 14.0 vs crypto-bigint 72.9 | −58.9 (0.19×) | 14.0 vs crypto-bigint 38.7 | −24.7 (0.36×) |
| mul_uint | BabyBear | 17.0 vs plonky3 23.6 | −6.6 (0.72×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 17.0 vs plonky3 33.7 | −16.7 (0.50×) | 13.1 vs plonky3 16.6 | −3.5 (0.79×) |
| mul_uint | Goldilocks | 20.2 vs winter-math 23.5 | −3.3 (0.86×) | 12.0 vs winter-math 13.7 | −1.7 (0.88×) | 25.5 vs winter-math 32.1 | −6.6 (0.79×) | 17.1 vs winter-math 14.9 | +2.2 (1.15×) |
| mul_uint | BN254 scalar | 434 vs halo2curves 754 | −320 (0.58×) | 185 vs halo2curves 181 | +4.0 (1.02×) | 532 vs halo2curves 1,342 | −810 (0.40×) | 251 vs halo2curves 348 | −97.0 (0.72×) |
| mul_uint | secp256k1 base | 265 vs halo2curves 515 | −250 (0.51×) | 118 vs halo2curves 154 | −36.0 (0.77×) | 323 vs halo2curves 969 | −646 (0.33×) | 163 vs halo2curves 266 | −103 (0.61×) |
| mul_uint | P-256 base | 353 vs halo2curves 452 | −99.0 (0.78×) | 180 vs halo2curves 145 | +35.0 (1.24×) | 421 vs halo2curves 949 | −528 (0.44×) | 237 vs halo2curves 252 | −15.0 (0.94×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 548 | −123 (0.78×) | 189 vs halo2curves 185 | +4.0 (1.02×) | 474 vs k256/p256 880 | −406 (0.54×) | 263 vs halo2curves 346 | −83.0 (0.76×) |
| mul_uint | Tom-256 base | 409 vs halo2curves 693 | −284 (0.59×) | 177 vs halo2curves 178 | −1.0 (0.99×) | 484 vs halo2curves 1,175 | −691 (0.41×) | 255 vs halo2curves 335 | −80.0 (0.76×) |
| mul_uint | BLS12-381 base | 982 vs halo2curves 1,939 | −957 (0.51×) | 380 vs halo2curves 379 | +1.0 (1.00×) | 1,147 vs crypto-bigint 2,477 | −1,330 (0.46×) | 553 vs crypto-bigint 1,035 | −482 (0.53×) |
| mul_uint | P-521 base | 1,544 vs crypto-bigint 3,062 | −1,518 (0.50×) | 644 vs crypto-bigint 1,011 | −367 (0.64×) | 2,381 vs crypto-bigint 3,996 | −1,615 (0.60×) | 990 vs crypto-bigint 1,861 | −871 (0.53×) |
| add_u64 | Mersenne-31 | 8.6 vs crypto-bigint 35.4 | −26.8 (0.24×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.3 vs crypto-bigint 42.0 | −32.7 (0.22×) | 9.3 vs crypto-bigint 30.1 | −20.8 (0.31×) |
| add_u64 | BabyBear | 12.1 vs plonky3 30.7 | −18.6 (0.39×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 41.7 | −29.0 (0.30×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_u64 | Goldilocks | 9.1 vs winter-math 13.0 | −3.9 (0.70×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.7 vs winter-math 19.1 | −8.4 (0.56×) | 10.0 vs winter-math 9.3 | +0.7 (1.08×) |
| add_u64 | BN254 scalar | 117 vs ark-ff 339 | −222 (0.35×) | 44.5 vs crypto-bigint 228 | −184 (0.20×) | 142 vs ark-ff 398 | −256 (0.36×) | 65.2 vs halo2curves 299 | −234 (0.22×) |
| add_u64 | secp256k1 base | 7.0 vs ark-ff 158 | −151 (0.04×) | 6.0 vs ark-ff 119 | −113 (0.05×) | 8.0 vs ark-ff 222 | −214 (0.04×) | 10.0 vs ark-ff 184 | −174 (0.05×) |
| add_u64 | P-256 base | 68.9 vs ark-ff 261 | −192 (0.26×) | 35.2 vs k256/p256 92.5 | −57.3 (0.38×) | 77.5 vs ark-ff 400 | −323 (0.19×) | 57.4 vs k256/p256 158 | −101 (0.36×) |
| add_u64 | P-256 scalar | 69.2 vs k256/p256 57.4 | +11.8 (1.21×) | 38.4 vs k256/p256 56.9 | −18.5 (0.67×) | 74.2 vs k256/p256 63.0 | +11.2 (1.18×) | 65.7 vs k256/p256 63.0 | +2.7 (1.04×) |
| add_u64 | Tom-256 base | 88.4 vs crypto-bigint 510 | −422 (0.17×) | 39.7 vs halo2curves 223 | −183 (0.18×) | 100 vs halo2curves 705 | −605 (0.14×) | 64.8 vs crypto-bigint 309 | −244 (0.21×) |
| add_u64 | BLS12-381 base | 160 vs ark-ff 830 | −670 (0.19×) | 62.0 vs halo2curves 320 | −258 (0.19×) | 265 vs ark-ff 1,060 | −795 (0.25×) | 90.7 vs halo2curves 641 | −550 (0.14×) |
| add_u64 | P-521 base | 21.1 vs crypto-bigint 1,674 | −1,653 (0.01×) | 12.3 vs crypto-bigint 653 | −641 (0.02×) | 19.5 vs crypto-bigint 2,209 | −2,190 (0.01×) | 26.8 vs crypto-bigint 1,141 | −1,114 (0.02×) |
| add_uint | Mersenne-31 | 8.7 vs crypto-bigint 35.4 | −26.7 (0.25×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.2 vs crypto-bigint 43.4 | −34.2 (0.21×) | 9.3 vs crypto-bigint 30.5 | −21.2 (0.30×) |
| add_uint | BabyBear | 12.2 vs plonky3 21.5 | −9.3 (0.57×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 31.1 | −18.4 (0.41×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_uint | Goldilocks | 9.1 vs winter-math 13.0 | −3.9 (0.70×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.7 vs winter-math 19.1 | −8.4 (0.56×) | 10.0 vs winter-math 9.6 | +0.4 (1.04×) |
| add_uint | BN254 scalar | 284 vs halo2curves 392 | −108 (0.72×) | 136 vs halo2curves 111 | +25.0 (1.23×) | 421 vs halo2curves 681 | −260 (0.62×) | 200 vs halo2curves 205 | −5.0 (0.98×) |
| add_uint | secp256k1 base | 17.0 vs halo2curves 226 | −209 (0.08×) | 12.6 vs halo2curves 85.1 | −72.5 (0.15×) | 16.6 vs halo2curves 431 | −414 (0.04×) | 23.2 vs halo2curves 140 | −117 (0.17×) |
| add_uint | P-256 base | 197 vs halo2curves 246 | −49.0 (0.80×) | 88.0 vs halo2curves 90.1 | −2.1 (0.98×) | 316 vs halo2curves 512 | −196 (0.62×) | 161 vs halo2curves 153 | +8.0 (1.05×) |
| add_uint | P-256 scalar | 288 vs k256/p256 81.8 | +206 (3.52×) | 124 vs k256/p256 81.6 | +42.4 (1.52×) | 371 vs k256/p256 110 | +261 (3.37×) | 213 vs k256/p256 107 | +106 (1.99×) |
| add_uint | Tom-256 base | 289 vs halo2curves 380 | −91.0 (0.76×) | 124 vs halo2curves 110 | +14.0 (1.13×) | 363 vs halo2curves 624 | −261 (0.58×) | 204 vs halo2curves 196 | +8.0 (1.04×) |
| add_uint | BLS12-381 base | 807 vs halo2curves 994 | −187 (0.81×) | 320 vs halo2curves 224 | +96.0 (1.43×) | 1,002 vs crypto-bigint 1,388 | −386 (0.72×) | 505 vs halo2curves 425 | +80.0 (1.19×) |
| add_uint | P-521 base | 62.2 vs crypto-bigint 1,691 | −1,629 (0.04×) | 42.0 vs crypto-bigint 664 | −622 (0.06×) | 85.4 vs crypto-bigint 2,210 | −2,125 (0.04×) | 67.0 vs crypto-bigint 1,174 | −1,107 (0.06×) |
| sub_u64 | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.6 vs crypto-bigint 30.3 | −23.7 (0.22×) | 6.6 vs crypto-bigint 20.2 | −13.6 (0.33×) |
| sub_u64 | BabyBear | 11.9 vs crypto-bigint 28.7 | −16.8 (0.41×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 12.5 vs plonky3 41.8 | −29.3 (0.30×) | 10.9 vs plonky3 13.3 | −2.4 (0.82×) |
| sub_u64 | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.5 vs winter-math 7.1 | −0.6 (0.92×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.7 vs winter-math 8.7 | +1.0 (1.11×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 347 | −234 (0.33×) | 45.6 vs crypto-bigint 152 | −106 (0.30×) | 136 vs ark-ff 401 | −265 (0.34×) | 66.6 vs crypto-bigint 285 | −218 (0.23×) |
| sub_u64 | secp256k1 base | 7.0 vs ark-ff 156 | −149 (0.04×) | 6.1 vs ark-ff 121 | −115 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 9.5 vs ark-ff 182 | −173 (0.05×) |
| sub_u64 | P-256 base | 71.3 vs ark-ff 263 | −192 (0.27×) | 35.2 vs k256/p256 88.0 | −52.8 (0.40×) | 80.2 vs ark-ff 400 | −320 (0.20×) | 57.1 vs k256/p256 149 | −91.9 (0.38×) |
| sub_u64 | P-256 scalar | 68.8 vs k256/p256 21.2 | +47.6 (3.25×) | 37.3 vs k256/p256 21.1 | +16.2 (1.77×) | 73.6 vs k256/p256 24.4 | +49.2 (3.02×) | 65.4 vs k256/p256 24.7 | +40.7 (2.65×) |
| sub_u64 | Tom-256 base | 91.7 vs crypto-bigint 481 | −389 (0.19×) | 39.2 vs crypto-bigint 152 | −113 (0.26×) | 104 vs halo2curves 690 | −586 (0.15×) | 65.5 vs crypto-bigint 284 | −219 (0.23×) |
| sub_u64 | BLS12-381 base | 155 vs ark-ff 824 | −669 (0.19×) | 62.8 vs halo2curves 317 | −254 (0.20×) | 272 vs ark-ff 1,049 | −777 (0.26×) | 91.0 vs halo2curves 601 | −510 (0.15×) |
| sub_u64 | P-521 base | 21.1 vs crypto-bigint 1,680 | −1,659 (0.01×) | 15.2 vs crypto-bigint 627 | −612 (0.02×) | 22.2 vs crypto-bigint 2,185 | −2,163 (0.01×) | 21.7 vs crypto-bigint 1,106 | −1,084 (0.02×) |
| sub_uint | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.6 vs crypto-bigint 30.4 | −23.8 (0.22×) | 6.6 vs crypto-bigint 20.1 | −13.5 (0.33×) |
| sub_uint | BabyBear | 11.9 vs plonky3 22.0 | −10.1 (0.54×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 13.0 vs plonky3 31.3 | −18.3 (0.42×) | 10.9 vs plonky3 13.3 | −2.4 (0.82×) |
| sub_uint | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.5 vs winter-math 7.0 | −0.5 (0.93×) | 10.4 vs winter-math 18.1 | −7.7 (0.57×) | 9.7 vs winter-math 8.7 | +1.0 (1.11×) |
| sub_uint | BN254 scalar | 282 vs halo2curves 396 | −114 (0.71×) | 138 vs halo2curves 105 | +33.0 (1.31×) | 420 vs halo2curves 662 | −242 (0.63×) | 204 vs halo2curves 193 | +11.0 (1.06×) |
| sub_uint | secp256k1 base | 18.9 vs halo2curves 214 | −195 (0.09×) | 12.4 vs halo2curves 79.7 | −67.3 (0.16×) | 16.8 vs halo2curves 412 | −395 (0.04×) | 22.0 vs halo2curves 130 | −108 (0.17×) |
| sub_uint | P-256 base | 202 vs halo2curves 234 | −32.0 (0.86×) | 88.3 vs halo2curves 85.7 | +2.6 (1.03×) | 382 vs halo2curves 504 | −122 (0.76×) | 163 vs halo2curves 143 | +20.0 (1.14×) |
| sub_uint | P-256 scalar | 286 vs k256/p256 71.9 | +214 (3.98×) | 124 vs k256/p256 72.0 | +52.0 (1.72×) | 379 vs k256/p256 95.0 | +284 (3.99×) | 211 vs k256/p256 95.5 | +116 (2.21×) |
| sub_uint | Tom-256 base | 287 vs halo2curves 357 | −70.0 (0.80×) | 121 vs halo2curves 103 | +18.0 (1.17×) | 363 vs halo2curves 616 | −253 (0.59×) | 208 vs halo2curves 188 | +20.0 (1.11×) |
| sub_uint | BLS12-381 base | 809 vs halo2curves 972 | −163 (0.83×) | 324 vs halo2curves 216 | +108 (1.50×) | 999 vs crypto-bigint 1,361 | −362 (0.73×) | 505 vs halo2curves 408 | +97.0 (1.24×) |
| sub_uint | P-521 base | 67.4 vs crypto-bigint 1,680 | −1,613 (0.04×) | 38.7 vs crypto-bigint 638 | −599 (0.06×) | 75.2 vs crypto-bigint 2,198 | −2,123 (0.03×) | 67.4 vs crypto-bigint 1,123 | −1,056 (0.06×) |
| div_u64 | BN254 scalar | 1,208 vs halo2curves 13,474 | −12,266 (0.09×) | 842 vs halo2curves 6,985 | −6,143 (0.12×) | 1,432 vs halo2curves 19,327 | −17,895 (0.07×) | 988 vs halo2curves 11,726 | −10,738 (0.08×) |
| div_u64 | secp256k1 base | 1,017 vs halo2curves 13,047 | −12,030 (0.08×) | 777 vs halo2curves 6,988 | −6,211 (0.11×) | 1,228 vs halo2curves 18,889 | −17,661 (0.07×) | 874 vs halo2curves 11,513 | −10,639 (0.08×) |
| div_u64 | P-256 base | 1,086 vs halo2curves 13,117 | −12,031 (0.08×) | 887 vs halo2curves 6,910 | −6,023 (0.13×) | 1,342 vs halo2curves 18,888 | −17,546 (0.07×) | 944 vs halo2curves 11,506 | −10,562 (0.08×) |
| div_u64 | P-256 scalar | 1,180 vs halo2curves 13,381 | −12,201 (0.09×) | 899 vs halo2curves 6,964 | −6,065 (0.13×) | 1,419 vs halo2curves 19,216 | −17,797 (0.07×) | 994 vs halo2curves 11,689 | −10,695 (0.09×) |
| div_u64 | Tom-256 base | 1,181 vs halo2curves 13,342 | −12,161 (0.09×) | 854 vs halo2curves 6,927 | −6,073 (0.12×) | 1,484 vs halo2curves 19,081 | −17,597 (0.08×) | 966 vs halo2curves 11,657 | −10,691 (0.08×) |
| div_u64 | BLS12-381 base | 1,799 vs halo2curves 24,145 | −22,346 (0.07×) | 1,133 vs halo2curves 11,021 | −9,888 (0.10×) | 2,464 vs halo2curves 32,340 | −29,876 (0.08×) | 1,303 vs halo2curves 18,530 | −17,227 (0.07×) |
| div_u64 | P-521 base | 2,447 vs crypto-bigint 62,043 | −59,596 (0.04×) | 1,375 vs crypto-bigint 45,690 | −44,315 (0.03×) | 3,585 vs crypto-bigint 77,832 | −74,247 (0.05×) | 1,883 vs crypto-bigint 58,233 | −56,350 (0.03×) |
| div_uint | Mersenne-31 | 277 vs crypto-bigint 1,831 | −1,554 (0.15×) | 276 vs crypto-bigint 1,608 | −1,332 (0.17×) | 283 vs runtime-algebra 2,113 | −1,830 (0.13×) | 283 vs crypto-bigint 1,983 | −1,700 (0.14×) |
| div_uint | BabyBear | 279 vs plonky3 669 | −390 (0.42×) | 278 vs plonky3 655 | −377 (0.42×) | 287 vs plonky3 719 | −432 (0.40×) | 287 vs plonky3 723 | −436 (0.40×) |
| div_uint | Goldilocks | 553 vs winter-math 1,812 | −1,259 (0.31×) | 544 vs winter-math 927 | −383 (0.59×) | 591 vs winter-math 1,892 | −1,301 (0.31×) | 588 vs winter-math 1,028 | −440 (0.57×) |
| div_uint | BN254 scalar | 9,060 vs halo2curves 15,053 | −5,993 (0.60×) | 6,650 vs halo2curves 8,126 | −1,476 (0.82×) | 12,599 vs halo2curves 22,881 | −10,282 (0.55×) | 8,483 vs halo2curves 14,477 | −5,994 (0.59×) |
| div_uint | secp256k1 base | 8,605 vs halo2curves 14,888 | −6,283 (0.58×) | 6,042 vs halo2curves 8,168 | −2,126 (0.74×) | 11,968 vs halo2curves 22,606 | −10,638 (0.53×) | 8,098 vs halo2curves 14,389 | −6,291 (0.56×) |
| div_uint | P-256 base | 8,664 vs halo2curves 14,855 | −6,191 (0.58×) | 6,126 vs halo2curves 8,222 | −2,096 (0.75×) | 12,034 vs halo2curves 22,392 | −10,358 (0.54×) | 7,872 vs halo2curves 14,319 | −6,447 (0.55×) |
| div_uint | P-256 scalar | 9,062 vs halo2curves 15,050 | −5,988 (0.60×) | 6,538 vs halo2curves 8,246 | −1,708 (0.79×) | 12,578 vs halo2curves 23,101 | −10,523 (0.54×) | 8,639 vs halo2curves 14,648 | −6,009 (0.59×) |
| div_uint | Tom-256 base | 8,947 vs halo2curves 15,031 | −6,084 (0.60×) | 6,614 vs halo2curves 8,234 | −1,620 (0.80×) | 12,177 vs halo2curves 22,893 | −10,716 (0.53×) | 8,453 vs halo2curves 14,520 | −6,067 (0.58×) |
| div_uint | BLS12-381 base | 20,960 vs halo2curves 27,636 | −6,676 (0.76×) | 11,932 vs halo2curves 13,574 | −1,642 (0.88×) | 26,684 vs halo2curves 39,855 | −13,171 (0.67×) | 16,842 vs halo2curves 24,361 | −7,519 (0.69×) |
| div_uint | P-521 base | 34,184 vs crypto-bigint 68,267 | −34,083 (0.50×) | 18,379 vs crypto-bigint 52,443 | −34,064 (0.35×) | 49,547 vs crypto-bigint 83,242 | −33,695 (0.60×) | 30,667 vs crypto-bigint 63,655 | −32,988 (0.48×) |
| add, constant time | Mersenne-31 | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) | 7.7 vs runtime-algebra 17.8 | −10.1 (0.43×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add, constant time | BN254 scalar | 24.4 vs halo2curves 41.6 | −17.2 (0.59×) | 19.5 vs halo2curves 23.0 | −3.5 (0.85×) | 23.1 vs halo2curves 45.1 | −22.0 (0.51×) | 25.7 vs halo2curves 34.0 | −8.3 (0.76×) |
| add, constant time | secp256k1 base | 28.0 vs halo2curves 39.8 | −11.8 (0.70×) | 22.6 vs halo2curves 23.0 | −0.4 (0.98×) | 27.0 vs halo2curves 38.9 | −11.9 (0.69×) | 29.8 vs halo2curves 31.0 | −1.2 (0.96×) |
| add, constant time | P-256 base | 26.8 vs halo2curves 38.9 | −12.1 (0.69×) | 22.8 vs halo2curves 21.0 | +1.8 (1.09×) | 24.9 vs halo2curves 39.6 | −14.7 (0.63×) | 28.0 vs k256/p256 29.4 | −1.4 (0.95×) |
| add, constant time | P-256 scalar | 27.3 vs halo2curves 42.2 | −14.9 (0.65×) | 23.8 vs halo2curves 24.7 | −0.9 (0.96×) | 26.8 vs halo2curves 42.6 | −15.8 (0.63×) | 30.1 vs halo2curves 33.0 | −2.9 (0.91×) |
| add, constant time | Tom-256 base | 26.6 vs halo2curves 42.9 | −16.3 (0.62×) | 23.8 vs halo2curves 24.0 | −0.2 (0.99×) | 27.0 vs halo2curves 43.0 | −16.0 (0.63×) | 29.0 vs halo2curves 32.5 | −3.5 (0.89×) |
| add, constant time | BLS12-381 base | 41.8 vs halo2curves 63.3 | −21.5 (0.66×) | 28.5 vs halo2curves 34.1 | −5.6 (0.84×) | 35.3 vs halo2curves 67.0 | −31.7 (0.53×) | 41.0 vs halo2curves 54.0 | −13.0 (0.76×) |
| add, constant time | P-521 base | 77.8 vs crypto-bigint 183 | −105 (0.43×) | 48.6 vs crypto-bigint 179 | −130 (0.27×) | 80.4 vs crypto-bigint 247 | −167 (0.33×) | 74.0 vs crypto-bigint 243 | −169 (0.30×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs runtime-algebra 7.9 | −4.9 (0.38×) | 3.0 vs runtime-algebra 7.9 | −4.9 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 26.4 vs halo2curves 26.0 | +0.4 (1.02×) | 16.9 vs halo2curves 14.9 | +2.0 (1.13×) | 27.0 vs halo2curves 26.0 | +1.0 (1.04×) | 25.0 vs halo2curves 22.0 | +3.0 (1.14×) |
| sub, constant time | secp256k1 base | 21.2 vs halo2curves 25.2 | −4.0 (0.84×) | 15.5 vs halo2curves 14.5 | +1.0 (1.07×) | 20.4 vs halo2curves 25.2 | −4.8 (0.81×) | 21.5 vs halo2curves 21.0 | +0.5 (1.02×) |
| sub, constant time | P-256 base | 18.9 vs halo2curves 25.2 | −6.3 (0.75×) | 15.5 vs halo2curves 14.1 | +1.4 (1.10×) | 18.9 vs halo2curves 24.4 | −5.5 (0.77×) | 20.0 vs k256/p256 20.0 | +0.0 (1.00×) |
| sub, constant time | P-256 scalar | 21.6 vs k256/p256 24.7 | −3.1 (0.87×) | 15.8 vs halo2curves 15.1 | +0.7 (1.05×) | 18.8 vs k256/p256 26.9 | −8.1 (0.70×) | 21.5 vs halo2curves 21.5 | +0.0 (1.00×) |
| sub, constant time | Tom-256 base | 21.4 vs halo2curves 26.0 | −4.6 (0.82×) | 17.1 vs halo2curves 14.9 | +2.2 (1.15×) | 19.8 vs halo2curves 26.4 | −6.6 (0.75×) | 22.5 vs halo2curves 22.0 | +0.5 (1.02×) |
| sub, constant time | BLS12-381 base | 37.4 vs halo2curves 45.3 | −7.9 (0.83×) | 24.7 vs halo2curves 24.9 | −0.2 (0.99×) | 36.6 vs halo2curves 44.1 | −7.5 (0.83×) | 36.2 vs halo2curves 36.1 | +0.1 (1.00×) |
| sub, constant time | P-521 base | 57.5 vs crypto-bigint 159 | −102 (0.36×) | 39.9 vs crypto-bigint 145 | −105 (0.28×) | 59.9 vs crypto-bigint 212 | −152 (0.28×) | 53.0 vs crypto-bigint 171 | −118 (0.31×) |
| mul, constant time | Mersenne-31 | 12.0 vs crypto-bigint 46.5 | −34.5 (0.26×) | 12.0 vs crypto-bigint 26.9 | −14.9 (0.45×) | 12.0 vs crypto-bigint 50.0 | −38.0 (0.24×) | 12.0 vs crypto-bigint 30.8 | −18.8 (0.39×) |
| mul, constant time | BabyBear | 13.0 vs crypto-bigint 46.1 | −33.1 (0.28×) | 11.0 vs crypto-bigint 26.9 | −15.9 (0.41×) | 14.5 vs crypto-bigint 48.5 | −34.0 (0.30×) | 11.0 vs crypto-bigint 30.8 | −19.8 (0.36×) |
| mul, constant time | Goldilocks | 17.7 vs winter-math 18.0 | −0.3 (0.98×) | 14.4 vs winter-math 14.5 | −0.1 (0.99×) | 22.0 vs winter-math 19.0 | +3.0 (1.16×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul, constant time | BN254 scalar | 243 vs halo2curves 378 | −135 (0.64×) | 94.0 vs halo2curves 93.5 | +0.5 (1.01×) | 411 vs halo2curves 690 | −279 (0.60×) | 149 vs halo2curves 178 | −29.0 (0.84×) |
| mul, constant time | secp256k1 base | 171 vs halo2curves 306 | −135 (0.56×) | 77.5 vs halo2curves 90.3 | −12.8 (0.86×) | 226 vs halo2curves 587 | −361 (0.39×) | 108 vs halo2curves 160 | −52.0 (0.68×) |
| mul, constant time | P-256 base | 190 vs k256/p256 241 | −51.0 (0.79×) | 72.4 vs halo2curves 75.6 | −3.2 (0.96×) | 292 vs k256/p256 461 | −169 (0.63×) | 120 vs k256/p256 131 | −11.0 (0.92×) |
| mul, constant time | P-256 scalar | 234 vs halo2curves 351 | −117 (0.67×) | 93.5 vs halo2curves 95.4 | −1.9 (0.98×) | 320 vs halo2curves 693 | −373 (0.46×) | 164 vs halo2curves 179 | −15.0 (0.92×) |
| mul, constant time | Tom-256 base | 238 vs halo2curves 342 | −104 (0.70×) | 86.5 vs halo2curves 94.2 | −7.7 (0.92×) | 311 vs halo2curves 588 | −277 (0.53×) | 149 vs halo2curves 171 | −22.0 (0.87×) |
| mul, constant time | BLS12-381 base | 721 vs halo2curves 892 | −171 (0.81×) | 244 vs halo2curves 193 | +51.0 (1.26×) | 914 vs crypto-bigint 1,308 | −394 (0.70×) | 429 vs halo2curves 376 | +53.0 (1.14×) |
| mul, constant time | P-521 base | 1,064 vs crypto-bigint 2,118 | −1,054 (0.50×) | 417 vs crypto-bigint 617 | −200 (0.68×) | 1,604 vs crypto-bigint 2,780 | −1,176 (0.58×) | 866 vs crypto-bigint 1,104 | −238 (0.78×) |
| square, constant time | Mersenne-31 | 12.0 vs crypto-bigint 45.0 | −33.0 (0.27×) | 12.0 vs crypto-bigint 27.0 | −15.0 (0.44×) | 12.0 vs crypto-bigint 47.7 | −35.7 (0.25×) | 12.0 vs crypto-bigint 30.1 | −18.1 (0.40×) |
| square, constant time | BabyBear | 13.0 vs crypto-bigint 45.5 | −32.5 (0.29×) | 11.0 vs crypto-bigint 27.0 | −16.0 (0.41×) | 14.0 vs crypto-bigint 52.7 | −38.7 (0.27×) | 11.0 vs crypto-bigint 30.7 | −19.7 (0.36×) |
| square, constant time | Goldilocks | 17.3 vs winter-math 20.0 | −2.7 (0.86×) | 12.0 vs winter-math 12.8 | −0.8 (0.94×) | 21.2 vs winter-math 20.0 | +1.2 (1.06×) | 14.4 vs winter-math 14.3 | +0.1 (1.01×) |
| square, constant time | BN254 scalar | 219 vs halo2curves 309 | −90.0 (0.71×) | 82.5 vs halo2curves 95.8 | −13.3 (0.86×) | 220 vs halo2curves 555 | −335 (0.40×) | 147 vs halo2curves 144 | +3.0 (1.02×) |
| square, constant time | secp256k1 base | 154 vs halo2curves 222 | −68.0 (0.69×) | 69.9 vs halo2curves 84.1 | −14.2 (0.83×) | 198 vs halo2curves 456 | −258 (0.43×) | 95.7 vs halo2curves 122 | −26.3 (0.78×) |
| square, constant time | P-256 base | 155 vs halo2curves 178 | −23.0 (0.87×) | 62.1 vs halo2curves 61.3 | +0.8 (1.01×) | 227 vs halo2curves 341 | −114 (0.67×) | 104 vs halo2curves 107 | −3.0 (0.97×) |
| square, constant time | P-256 scalar | 222 vs halo2curves 266 | −44.0 (0.83×) | 89.1 vs halo2curves 92.5 | −3.4 (0.96×) | 238 vs halo2curves 551 | −313 (0.43×) | 150 vs halo2curves 140 | +10.0 (1.07×) |
| square, constant time | Tom-256 base | 200 vs halo2curves 267 | −67.0 (0.75×) | 83.0 vs halo2curves 89.5 | −6.5 (0.93×) | 234 vs halo2curves 474 | −240 (0.49×) | 142 vs halo2curves 136 | +6.0 (1.04×) |
| square, constant time | BLS12-381 base | 607 vs halo2curves 928 | −321 (0.65×) | 292 vs halo2curves 164 | +128 (1.78×) | 853 vs halo2curves 1,236 | −383 (0.69×) | 441 vs halo2curves 296 | +145 (1.49×) |
| square, constant time | P-521 base | 569 vs crypto-bigint 2,164 | −1,595 (0.26×) | 261 vs crypto-bigint 613 | −352 (0.43×) | 912 vs crypto-bigint 2,812 | −1,900 (0.32×) | 418 vs crypto-bigint 1,201 | −783 (0.35×) |
| mul_u64, constant time | Mersenne-31 | 14.0 vs crypto-bigint 59.5 | −45.5 (0.24×) | 14.0 vs crypto-bigint 31.4 | −17.4 (0.45×) | 14.0 vs crypto-bigint 71.6 | −57.6 (0.20×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_u64, constant time | BabyBear | 17.0 vs crypto-bigint 65.1 | −48.1 (0.26×) | 11.0 vs crypto-bigint 32.0 | −21.0 (0.34×) | 17.1 vs crypto-bigint 90.3 | −73.2 (0.19×) | 13.1 vs crypto-bigint 40.8 | −27.7 (0.32×) |
| mul_u64, constant time | Goldilocks | 20.1 vs winter-math 23.6 | −3.5 (0.85×) | 12.0 vs winter-math 13.7 | −1.7 (0.88×) | 25.5 vs winter-math 32.1 | −6.6 (0.79×) | 17.1 vs winter-math 15.2 | +1.9 (1.13×) |
| mul_u64, constant time | BN254 scalar | 110 vs crypto-bigint 853 | −743 (0.13×) | 50.9 vs crypto-bigint 262 | −211 (0.19×) | 132 vs halo2curves 1,469 | −1,337 (0.09×) | 58.0 vs crypto-bigint 410 | −352 (0.14×) |
| mul_u64, constant time | secp256k1 base | 68.9 vs halo2curves 690 | −621 (0.10×) | 39.0 vs crypto-bigint 259 | −220 (0.15×) | 85.1 vs halo2curves 1,048 | −963 (0.08×) | 45.5 vs crypto-bigint 409 | −364 (0.11×) |
| mul_u64, constant time | P-256 base | 90.3 vs k256/p256 487 | −397 (0.19×) | 53.2 vs k256/p256 151 | −97.8 (0.35×) | 103 vs k256/p256 890 | −787 (0.12×) | 59.4 vs k256/p256 258 | −199 (0.23×) |
| mul_u64, constant time | P-256 scalar | 106 vs k256/p256 513 | −407 (0.21×) | 55.0 vs k256/p256 255 | −200 (0.22×) | 127 vs k256/p256 814 | −687 (0.16×) | 67.3 vs k256/p256 340 | −273 (0.20×) |
| mul_u64, constant time | Tom-256 base | 100 vs crypto-bigint 843 | −743 (0.12×) | 52.0 vs crypto-bigint 259 | −207 (0.20×) | 122 vs halo2curves 1,284 | −1,162 (0.10×) | 64.0 vs crypto-bigint 410 | −346 (0.16×) |
| mul_u64, constant time | BLS12-381 base | 154 vs crypto-bigint 1,941 | −1,787 (0.08×) | 72.0 vs halo2curves 486 | −414 (0.15×) | 174 vs crypto-bigint 2,507 | −2,333 (0.07×) | 88.0 vs crypto-bigint 1,038 | −950 (0.08×) |
| mul_u64, constant time | P-521 base | 139 vs crypto-bigint 3,025 | −2,886 (0.05×) | 77.9 vs crypto-bigint 994 | −916 (0.08×) | 171 vs crypto-bigint 3,994 | −3,823 (0.04×) | 95.0 vs crypto-bigint 1,844 | −1,749 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 14.0 vs crypto-bigint 59.6 | −45.6 (0.23×) | 14.0 vs crypto-bigint 31.5 | −17.5 (0.44×) | 14.0 vs crypto-bigint 72.9 | −58.9 (0.19×) | 14.0 vs crypto-bigint 38.7 | −24.7 (0.36×) |
| mul_uint, constant time | BabyBear | 17.0 vs crypto-bigint 64.8 | −47.8 (0.26×) | 11.0 vs crypto-bigint 32.4 | −21.4 (0.34×) | 17.0 vs crypto-bigint 84.6 | −67.6 (0.20×) | 13.1 vs crypto-bigint 40.8 | −27.7 (0.32×) |
| mul_uint, constant time | Goldilocks | 20.1 vs winter-math 23.5 | −3.4 (0.86×) | 12.0 vs winter-math 13.7 | −1.7 (0.88×) | 25.5 vs winter-math 32.1 | −6.6 (0.79×) | 17.1 vs winter-math 14.9 | +2.2 (1.15×) |
| mul_uint, constant time | BN254 scalar | 479 vs halo2curves 754 | −275 (0.64×) | 251 vs halo2curves 181 | +70.0 (1.39×) | 594 vs halo2curves 1,342 | −748 (0.44×) | 311 vs halo2curves 348 | −37.0 (0.89×) |
| mul_uint, constant time | secp256k1 base | 316 vs halo2curves 515 | −199 (0.61×) | 192 vs halo2curves 154 | +38.0 (1.25×) | 379 vs halo2curves 969 | −590 (0.39×) | 252 vs halo2curves 266 | −14.0 (0.95×) |
| mul_uint, constant time | P-256 base | 408 vs halo2curves 452 | −44.0 (0.90×) | 247 vs halo2curves 145 | +102 (1.70×) | 489 vs halo2curves 949 | −460 (0.52×) | 306 vs halo2curves 252 | +54.0 (1.21×) |
| mul_uint, constant time | P-256 scalar | 475 vs k256/p256 548 | −73.0 (0.87×) | 264 vs halo2curves 185 | +79.0 (1.43×) | 544 vs k256/p256 880 | −336 (0.62×) | 333 vs halo2curves 346 | −13.0 (0.96×) |
| mul_uint, constant time | Tom-256 base | 465 vs halo2curves 693 | −228 (0.67×) | 257 vs halo2curves 178 | +79.0 (1.44×) | 546 vs halo2curves 1,175 | −629 (0.46×) | 324 vs halo2curves 335 | −11.0 (0.97×) |
| mul_uint, constant time | BLS12-381 base | 1,079 vs halo2curves 1,939 | −860 (0.56×) | 506 vs halo2curves 379 | +127 (1.34×) | 1,690 vs crypto-bigint 2,477 | −787 (0.68×) | 684 vs crypto-bigint 1,035 | −351 (0.66×) |
| mul_uint, constant time | P-521 base | 2,021 vs crypto-bigint 3,062 | −1,041 (0.66×) | 998 vs crypto-bigint 1,011 | −13.0 (0.99×) | 2,905 vs crypto-bigint 3,996 | −1,091 (0.73×) | 1,470 vs crypto-bigint 1,861 | −391 (0.79×) |
| add_u64, constant time | Mersenne-31 | 8.7 vs crypto-bigint 35.4 | −26.7 (0.25×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.3 vs crypto-bigint 42.0 | −32.7 (0.22×) | 9.3 vs crypto-bigint 30.1 | −20.8 (0.31×) |
| add_u64, constant time | BabyBear | 12.2 vs crypto-bigint 47.5 | −35.3 (0.26×) | 8.2 vs crypto-bigint 30.0 | −21.8 (0.27×) | 12.7 vs crypto-bigint 52.4 | −39.7 (0.24×) | 11.0 vs crypto-bigint 32.8 | −21.8 (0.34×) |
| add_u64, constant time | Goldilocks | 9.1 vs winter-math 13.0 | −3.9 (0.70×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.8 vs winter-math 19.1 | −8.3 (0.57×) | 10.0 vs winter-math 9.3 | +0.7 (1.08×) |
| add_u64, constant time | BN254 scalar | 140 vs crypto-bigint 527 | −387 (0.27×) | 56.2 vs crypto-bigint 228 | −172 (0.25×) | 148 vs halo2curves 792 | −644 (0.19×) | 81.6 vs halo2curves 299 | −217 (0.27×) |
| add_u64, constant time | secp256k1 base | 25.4 vs halo2curves 402 | −377 (0.06×) | 23.0 vs halo2curves 216 | −193 (0.11×) | 25.2 vs halo2curves 496 | −471 (0.05×) | 29.0 vs halo2curves 246 | −217 (0.12×) |
| add_u64, constant time | P-256 base | 89.5 vs k256/p256 278 | −189 (0.32×) | 56.3 vs k256/p256 92.5 | −36.2 (0.61×) | 95.8 vs k256/p256 501 | −405 (0.19×) | 74.3 vs k256/p256 158 | −83.7 (0.47×) |
| add_u64, constant time | P-256 scalar | 83.2 vs k256/p256 57.4 | +25.8 (1.45×) | 58.4 vs k256/p256 56.9 | +1.5 (1.03×) | 97.2 vs k256/p256 63.0 | +34.2 (1.54×) | 79.7 vs k256/p256 63.0 | +16.7 (1.27×) |
| add_u64, constant time | Tom-256 base | 119 vs crypto-bigint 510 | −391 (0.23×) | 62.0 vs halo2curves 223 | −161 (0.28×) | 133 vs halo2curves 705 | −572 (0.19×) | 84.9 vs crypto-bigint 309 | −224 (0.27×) |
| add_u64, constant time | BLS12-381 base | 185 vs crypto-bigint 1,088 | −903 (0.17×) | 96.0 vs halo2curves 320 | −224 (0.30×) | 236 vs crypto-bigint 1,370 | −1,134 (0.17×) | 122 vs halo2curves 641 | −519 (0.19×) |
| add_u64, constant time | P-521 base | 65.7 vs crypto-bigint 1,674 | −1,608 (0.04×) | 45.1 vs crypto-bigint 653 | −608 (0.07×) | 68.0 vs crypto-bigint 2,209 | −2,141 (0.03×) | 64.7 vs crypto-bigint 1,141 | −1,076 (0.06×) |
| add_uint, constant time | Mersenne-31 | 8.7 vs crypto-bigint 35.4 | −26.7 (0.25×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.3 vs crypto-bigint 43.4 | −34.1 (0.21×) | 9.3 vs crypto-bigint 30.5 | −21.2 (0.30×) |
| add_uint, constant time | BabyBear | 11.6 vs crypto-bigint 47.5 | −35.9 (0.24×) | 8.2 vs crypto-bigint 30.0 | −21.8 (0.27×) | 12.7 vs crypto-bigint 52.3 | −39.6 (0.24×) | 11.0 vs crypto-bigint 31.9 | −20.9 (0.34×) |
| add_uint, constant time | Goldilocks | 9.1 vs winter-math 13.0 | −3.9 (0.70×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.8 vs winter-math 19.1 | −8.3 (0.57×) | 10.0 vs winter-math 9.6 | +0.4 (1.04×) |
| add_uint, constant time | BN254 scalar | 314 vs halo2curves 392 | −78.0 (0.80×) | 129 vs halo2curves 111 | +18.0 (1.16×) | 369 vs halo2curves 681 | −312 (0.54×) | 205 vs halo2curves 205 | +0.0 (1.00×) |
| add_uint, constant time | secp256k1 base | 38.6 vs halo2curves 226 | −187 (0.17×) | 29.5 vs halo2curves 85.1 | −55.6 (0.35×) | 36.4 vs halo2curves 431 | −395 (0.08×) | 44.5 vs halo2curves 140 | −95.5 (0.32×) |
| add_uint, constant time | P-256 base | 218 vs halo2curves 246 | −28.0 (0.89×) | 108 vs halo2curves 90.1 | +17.9 (1.20×) | 299 vs halo2curves 512 | −213 (0.58×) | 160 vs halo2curves 153 | +7.0 (1.05×) |
| add_uint, constant time | P-256 scalar | 294 vs k256/p256 81.8 | +212 (3.59×) | 136 vs k256/p256 81.6 | +54.4 (1.67×) | 347 vs k256/p256 110 | +237 (3.15×) | 213 vs k256/p256 107 | +106 (1.99×) |
| add_uint, constant time | Tom-256 base | 290 vs halo2curves 380 | −90.0 (0.76×) | 128 vs halo2curves 110 | +18.0 (1.16×) | 364 vs halo2curves 624 | −260 (0.58×) | 207 vs halo2curves 196 | +11.0 (1.06×) |
| add_uint, constant time | BLS12-381 base | 841 vs halo2curves 994 | −153 (0.85×) | 351 vs halo2curves 224 | +127 (1.57×) | 1,025 vs crypto-bigint 1,388 | −363 (0.74×) | 544 vs halo2curves 425 | +119 (1.28×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,691 | −1,553 (0.08×) | 93.3 vs crypto-bigint 664 | −571 (0.14×) | 139 vs crypto-bigint 2,210 | −2,071 (0.06×) | 134 vs crypto-bigint 1,174 | −1,040 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.5 vs crypto-bigint 30.3 | −23.8 (0.21×) | 6.6 vs crypto-bigint 20.2 | −13.6 (0.33×) |
| sub_u64, constant time | BabyBear | 11.9 vs crypto-bigint 28.7 | −16.8 (0.41×) | 8.2 vs crypto-bigint 19.4 | −11.2 (0.42×) | 13.0 vs crypto-bigint 42.3 | −29.3 (0.31×) | 10.9 vs crypto-bigint 21.4 | −10.5 (0.51×) |
| sub_u64, constant time | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.6 vs winter-math 7.1 | −0.5 (0.93×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.4 vs winter-math 8.7 | +0.7 (1.08×) |
| sub_u64, constant time | BN254 scalar | 125 vs crypto-bigint 490 | −365 (0.26×) | 56.9 vs crypto-bigint 152 | −95.1 (0.37×) | 153 vs halo2curves 755 | −602 (0.20×) | 79.9 vs crypto-bigint 285 | −205 (0.28×) |
| sub_u64, constant time | secp256k1 base | 18.0 vs halo2curves 392 | −374 (0.05×) | 15.1 vs crypto-bigint 152 | −137 (0.10×) | 17.0 vs halo2curves 486 | −469 (0.03×) | 17.4 vs halo2curves 239 | −222 (0.07×) |
| sub_u64, constant time | P-256 base | 86.6 vs k256/p256 265 | −178 (0.33×) | 49.0 vs k256/p256 88.0 | −39.0 (0.56×) | 88.4 vs k256/p256 489 | −401 (0.18×) | 69.6 vs k256/p256 149 | −79.4 (0.47×) |
| sub_u64, constant time | P-256 scalar | 79.7 vs k256/p256 21.2 | +58.5 (3.76×) | 50.4 vs k256/p256 21.1 | +29.3 (2.39×) | 92.6 vs k256/p256 24.4 | +68.2 (3.80×) | 77.2 vs k256/p256 24.7 | +52.5 (3.13×) |
| sub_u64, constant time | Tom-256 base | 113 vs crypto-bigint 481 | −368 (0.23×) | 57.1 vs crypto-bigint 152 | −94.9 (0.38×) | 125 vs halo2curves 690 | −565 (0.18×) | 80.1 vs crypto-bigint 284 | −204 (0.28×) |
| sub_u64, constant time | BLS12-381 base | 175 vs crypto-bigint 1,093 | −918 (0.16×) | 90.0 vs halo2curves 317 | −227 (0.28×) | 229 vs crypto-bigint 1,365 | −1,136 (0.17×) | 119 vs halo2curves 601 | −482 (0.20×) |
| sub_u64, constant time | P-521 base | 45.8 vs crypto-bigint 1,680 | −1,634 (0.03×) | 35.4 vs crypto-bigint 627 | −592 (0.06×) | 49.8 vs crypto-bigint 2,185 | −2,135 (0.02×) | 46.3 vs crypto-bigint 1,106 | −1,060 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 5.4 vs crypto-bigint 22.1 | −16.7 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.6 vs crypto-bigint 30.4 | −23.8 (0.22×) | 6.5 vs crypto-bigint 20.1 | −13.6 (0.32×) |
| sub_uint, constant time | BabyBear | 12.0 vs crypto-bigint 28.7 | −16.7 (0.42×) | 8.2 vs crypto-bigint 19.4 | −11.2 (0.42×) | 13.3 vs crypto-bigint 42.3 | −29.0 (0.31×) | 10.8 vs crypto-bigint 21.9 | −11.1 (0.49×) |
| sub_uint, constant time | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.5 vs winter-math 7.0 | −0.5 (0.93×) | 10.8 vs winter-math 18.1 | −7.3 (0.60×) | 9.4 vs winter-math 8.7 | +0.7 (1.08×) |
| sub_uint, constant time | BN254 scalar | 313 vs halo2curves 396 | −83.0 (0.79×) | 133 vs halo2curves 105 | +28.0 (1.27×) | 372 vs halo2curves 662 | −290 (0.56×) | 219 vs halo2curves 193 | +26.0 (1.13×) |
| sub_uint, constant time | secp256k1 base | 32.2 vs halo2curves 214 | −182 (0.15×) | 24.9 vs halo2curves 79.7 | −54.8 (0.31×) | 31.6 vs halo2curves 412 | −380 (0.08×) | 37.7 vs halo2curves 130 | −92.3 (0.29×) |
| sub_uint, constant time | P-256 base | 225 vs halo2curves 234 | −9.0 (0.96×) | 98.2 vs halo2curves 85.7 | +12.5 (1.15×) | 291 vs halo2curves 504 | −213 (0.58×) | 158 vs halo2curves 143 | +15.0 (1.10×) |
| sub_uint, constant time | P-256 scalar | 296 vs k256/p256 71.9 | +224 (4.12×) | 127 vs k256/p256 72.0 | +55.0 (1.76×) | 343 vs k256/p256 95.0 | +248 (3.61×) | 207 vs k256/p256 95.5 | +112 (2.17×) |
| sub_uint, constant time | Tom-256 base | 289 vs halo2curves 357 | −68.0 (0.81×) | 127 vs halo2curves 103 | +24.0 (1.23×) | 362 vs halo2curves 616 | −254 (0.59×) | 193 vs halo2curves 188 | +5.0 (1.03×) |
| sub_uint, constant time | BLS12-381 base | 814 vs halo2curves 972 | −158 (0.84×) | 346 vs halo2curves 216 | +130 (1.60×) | 1,029 vs crypto-bigint 1,361 | −332 (0.76×) | 541 vs halo2curves 408 | +133 (1.33×) |
| sub_uint, constant time | P-521 base | 124 vs crypto-bigint 1,680 | −1,556 (0.07×) | 85.3 vs crypto-bigint 638 | −553 (0.13×) | 119 vs crypto-bigint 2,198 | −2,079 (0.05×) | 112 vs crypto-bigint 1,123 | −1,011 (0.10×) |
| div_u64, constant time | BN254 scalar | 12,417 vs crypto-bigint 29,683 | −17,266 (0.42×) | 9,574 vs crypto-bigint 25,834 | −16,260 (0.37×) | 17,184 vs crypto-bigint 37,709 | −20,525 (0.46×) | 12,896 vs crypto-bigint 31,382 | −18,486 (0.41×) |
| div_u64, constant time | secp256k1 base | 11,922 vs crypto-bigint 29,510 | −17,588 (0.40×) | 9,049 vs crypto-bigint 25,819 | −16,770 (0.35×) | 16,679 vs crypto-bigint 37,551 | −20,872 (0.44×) | 12,937 vs crypto-bigint 31,349 | −18,412 (0.41×) |
| div_u64, constant time | P-256 base | 11,832 vs crypto-bigint 29,445 | −17,613 (0.40×) | 9,131 vs k256/p256 23,992 | −14,861 (0.38×) | 17,350 vs crypto-bigint 37,689 | −20,339 (0.46×) | 12,483 vs crypto-bigint 31,375 | −18,892 (0.40×) |
| div_u64, constant time | P-256 scalar | 12,347 vs crypto-bigint 29,532 | −17,185 (0.42×) | 9,427 vs crypto-bigint 25,876 | −16,449 (0.36×) | 17,251 vs crypto-bigint 37,668 | −20,417 (0.46×) | 13,117 vs crypto-bigint 31,237 | −18,120 (0.42×) |
| div_u64, constant time | Tom-256 base | 12,203 vs crypto-bigint 29,575 | −17,372 (0.41×) | 9,343 vs crypto-bigint 25,865 | −16,522 (0.36×) | 16,988 vs crypto-bigint 37,566 | −20,578 (0.45×) | 12,925 vs crypto-bigint 31,377 | −18,452 (0.41×) |
| div_u64, constant time | BLS12-381 base | 24,797 vs crypto-bigint 51,387 | −26,590 (0.48×) | 16,774 vs crypto-bigint 43,318 | −26,544 (0.39×) | 34,568 vs crypto-bigint 66,876 | −32,308 (0.52×) | 24,124 vs crypto-bigint 54,655 | −30,531 (0.44×) |
| div_u64, constant time | P-521 base | 42,677 vs crypto-bigint 105,706 | −63,029 (0.40×) | 25,998 vs crypto-bigint 87,479 | −61,481 (0.30×) | 64,664 vs crypto-bigint 127,864 | −63,200 (0.51×) | 43,215 vs crypto-bigint 101,185 | −57,970 (0.43×) |
| div_uint, constant time | Mersenne-31 | 451 vs runtime-algebra 3,533 | −3,082 (0.13×) | 451 vs runtime-algebra 3,539 | −3,088 (0.13×) | 459 vs runtime-algebra 3,419 | −2,960 (0.13×) | 461 vs runtime-algebra 3,419 | −2,958 (0.13×) |
| div_uint, constant time | BabyBear | 545 vs runtime-algebra 3,525 | −2,980 (0.15×) | 437 vs runtime-algebra 3,526 | −3,089 (0.12×) | 607 vs runtime-algebra 3,419 | −2,812 (0.18×) | 450 vs runtime-algebra 3,419 | −2,969 (0.13×) |
| div_uint, constant time | Goldilocks | 1,308 vs winter-math 1,812 | −504 (0.72×) | 946 vs winter-math 927 | +19.0 (1.02×) | 1,576 vs winter-math 1,892 | −316 (0.83×) | 1,209 vs winter-math 1,028 | +181 (1.18×) |
| div_uint, constant time | BN254 scalar | 12,477 vs crypto-bigint 29,968 | −17,491 (0.42×) | 9,545 vs crypto-bigint 26,236 | −16,691 (0.36×) | 17,629 vs crypto-bigint 37,726 | −20,097 (0.47×) | 13,112 vs crypto-bigint 31,324 | −18,212 (0.42×) |
| div_uint, constant time | secp256k1 base | 11,932 vs crypto-bigint 29,691 | −17,759 (0.40×) | 9,054 vs crypto-bigint 26,195 | −17,141 (0.35×) | 16,858 vs crypto-bigint 37,631 | −20,773 (0.45×) | 12,900 vs crypto-bigint 31,440 | −18,540 (0.41×) |
| div_uint, constant time | P-256 base | 11,900 vs crypto-bigint 30,295 | −18,395 (0.39×) | 9,122 vs k256/p256 22,815 | −13,693 (0.40×) | 17,440 vs crypto-bigint 37,633 | −20,193 (0.46×) | 12,584 vs crypto-bigint 31,370 | −18,786 (0.40×) |
| div_uint, constant time | P-256 scalar | 12,390 vs crypto-bigint 29,718 | −17,328 (0.42×) | 9,496 vs crypto-bigint 26,208 | −16,712 (0.36×) | 17,609 vs crypto-bigint 37,578 | −19,969 (0.47×) | 13,334 vs crypto-bigint 31,428 | −18,094 (0.42×) |
| div_uint, constant time | Tom-256 base | 12,363 vs crypto-bigint 29,902 | −17,539 (0.41×) | 9,432 vs crypto-bigint 26,262 | −16,830 (0.36×) | 16,861 vs crypto-bigint 37,633 | −20,772 (0.45×) | 13,134 vs crypto-bigint 31,376 | −18,242 (0.42×) |
| div_uint, constant time | BLS12-381 base | 25,401 vs crypto-bigint 50,668 | −25,267 (0.50×) | 16,947 vs crypto-bigint 43,382 | −26,435 (0.39×) | 35,267 vs crypto-bigint 66,603 | −31,336 (0.53×) | 24,562 vs crypto-bigint 54,691 | −30,129 (0.45×) |
| div_uint, constant time | P-521 base | 42,935 vs crypto-bigint 105,569 | −62,634 (0.41×) | 26,146 vs crypto-bigint 85,263 | −59,117 (0.31×) | 65,297 vs crypto-bigint 128,164 | −62,867 (0.51×) | 43,287 vs crypto-bigint 101,383 | −58,096 (0.43×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 4.2 (2.2×) | 4.2 (2.2×) | 4.4 (2.3×) | 4.4 (2.3×) |
| add | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.1 (1.6×) | 3.2 (1.6×) |
| add | BN254 scalar | 5.0 | 16.7 (3.3×) | 12.3 (2.5×) | 17.1 (3.4×) | 19.0 (3.8×) |
| add | secp256k1 base | 5.8 | 14.1 (2.4×) | 11.8 (2.0×) | 14.9 (2.6×) | 20.5 (3.5×) |
| add | P-256 base | 5.8 | 15.1 (2.6×) | 12.3 (2.1×) | 15.0 (2.6×) | 20.0 (3.4×) |
| add | P-256 scalar | 5.8 | 15.3 (2.6×) | 11.9 (2.1×) | 16.1 (2.8×) | 21.4 (3.7×) |
| add | Tom-256 base | 5.9 | 15.0 (2.5×) | 11.7 (2.0×) | 15.8 (2.7×) | 20.5 (3.5×) |
| add | BLS12-381 base | 9.9 | 25.8 (2.6×) | 17.8 (1.8×) | 24.9 (2.5×) | 29.4 (3.0×) |
| add | P-521 base | 12.2 | 42.3 (3.5×) | 26.3 (2.2×) | 42.4 (3.5×) | 44.6 (3.7×) |
| sub | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BabyBear | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub | BN254 scalar | 5.2 | 16.6 (3.2×) | 11.9 (2.3×) | 15.1 (2.9×) | 19.6 (3.8×) |
| sub | secp256k1 base | 5.2 | 16.7 (3.2×) | 11.8 (2.3×) | 15.6 (3.0×) | 19.9 (3.8×) |
| sub | P-256 base | 5.4 | 15.2 (2.8×) | 11.9 (2.2×) | 14.9 (2.8×) | 18.8 (3.5×) |
| sub | P-256 scalar | 5.0 | 16.8 (3.4×) | 11.9 (2.4×) | 15.5 (3.1×) | 19.9 (4.0×) |
| sub | Tom-256 base | 5.4 | 16.5 (3.1×) | 11.9 (2.2×) | 15.5 (2.9×) | 19.6 (3.6×) |
| sub | BLS12-381 base | 10.0 | 25.4 (2.5×) | 17.9 (1.8×) | 24.8 (2.5×) | 29.2 (2.9×) |
| sub | P-521 base | 11.7 | 48.4 (4.1×) | 28.6 (2.4×) | 42.4 (3.6×) | 44.3 (3.8×) |
| mul | Mersenne-31 | 2.7 | 5.1 (1.9×) | 5.1 (1.9×) | 5.5 (2.0×) | 5.5 (2.0×) |
| mul | BabyBear | 3.6 | 7.8 (2.2×) | 5.2 (1.4×) | 7.8 (2.2×) | 5.0 (1.4×) |
| mul | Goldilocks | 3.7 | 12.2 (3.3×) | 7.0 (1.9×) | 12.4 (3.4×) | 9.0 (2.4×) |
| mul | BN254 scalar | 68.8 | 235 (3.4×) | 95.9 (1.4×) | 386 (5.6×) | 162 (2.4×) |
| mul | secp256k1 base | 44.3 | 162 (3.7×) | 64.1 (1.4×) | 218 (4.9×) | 108 (2.4×) |
| mul | P-256 base | 49.3 | 177 (3.6×) | 71.4 (1.4×) | 284 (5.8×) | 122 (2.5×) |
| mul | P-256 scalar | 72.7 | 241 (3.3×) | 86.6 (1.2×) | 319 (4.4×) | 172 (2.4×) |
| mul | Tom-256 base | 71.1 | 237 (3.3×) | 80.2 (1.1×) | 304 (4.3×) | 149 (2.1×) |
| mul | BLS12-381 base | 138 | 726 (5.3×) | 248 (1.8×) | 916 (6.6×) | 421 (3.1×) |
| mul | P-521 base | 171 | 1,020 (6.0×) | 399 (2.3×) | 1,637 (9.6×) | 846 (4.9×) |
| square | Mersenne-31 | 2.7 | 5.2 (1.9×) | 5.2 (1.9×) | 5.4 (2.0×) | 5.4 (2.0×) |
| square | BabyBear | 3.4 | 8.3 (2.4×) | 4.8 (1.4×) | 8.0 (2.4×) | 5.8 (1.7×) |
| square | Goldilocks | 3.7 | 10.1 (2.7×) | 6.0 (1.6×) | 12.0 (3.2×) | 9.7 (2.6×) |
| square | BN254 scalar | 63.2 | 215 (3.4×) | 95.0 (1.5×) | 374 (5.9×) | 160 (2.5×) |
| square | secp256k1 base | 32.7 | 143 (4.4×) | 60.8 (1.9×) | 192 (5.9×) | 99.5 (3.0×) |
| square | P-256 base | 40.1 | 159 (4.0×) | 78.3 (2.0×) | 220 (5.5×) | 116 (2.9×) |
| square | P-256 scalar | 64.4 | 219 (3.4×) | 83.6 (1.3×) | 281 (4.4×) | 144 (2.2×) |
| square | Tom-256 base | 56.9 | 205 (3.6×) | 75.9 (1.3×) | 226 (4.0×) | 158 (2.8×) |
| square | BLS12-381 base | 134 | 578 (4.3×) | 282 (2.1×) | 887 (6.6×) | 449 (3.4×) |
| square | P-521 base | 123 | 563 (4.6×) | 260 (2.1×) | 909 (7.4×) | 419 (3.4×) |
| invert | Mersenne-31 | 210 | 266 (1.3×) | 266 (1.3×) | 268 (1.3×) | 270 (1.3×) |
| invert | BabyBear | 212 | 269 (1.3×) | 268 (1.3×) | 277 (1.3×) | 277 (1.3×) |
| invert | Goldilocks | 431 | 537 (1.2×) | 537 (1.2×) | 563 (1.3×) | 571 (1.3×) |
| invert | BN254 scalar | 5,122 | 8,440 (1.6×) | 6,417 (1.3×) | 11,668 (2.3×) | 7,879 (1.5×) |
| invert | secp256k1 base | 4,842 | 8,502 (1.8×) | 6,031 (1.2×) | 11,702 (2.4×) | 7,960 (1.6×) |
| invert | P-256 base | 4,855 | 8,282 (1.7×) | 6,062 (1.2×) | 11,269 (2.3×) | 7,747 (1.6×) |
| invert | P-256 scalar | 5,046 | 8,482 (1.7×) | 6,399 (1.3×) | 11,980 (2.4×) | 8,223 (1.6×) |
| invert | Tom-256 base | 5,049 | 8,554 (1.7×) | 6,400 (1.3×) | 11,444 (2.3×) | 7,943 (1.6×) |
| invert | BLS12-381 base | 8,833 | 19,335 (2.2×) | 11,422 (1.3×) | 24,208 (2.7×) | 15,490 (1.8×) |
| invert | P-521 base | 14,210 | 33,221 (2.3×) | 18,196 (1.3×) | 47,873 (3.4×) | 29,337 (2.1×) |
| sqrt | Mersenne-31 | 136 | 329 (2.4×) | 329 (2.4×) | 330 (2.4×) | 330 (2.4×) |
| sqrt | BabyBear | 1,987 | 2,670 (1.3×) | 2,040 (1.0×) | 2,760 (1.4×) | 2,167 (1.1×) |
| sqrt | Goldilocks | 2,845 | 5,768 (2.0×) | 3,697 (1.3×) | 6,762 (2.4×) | 9,179 (3.2×) |
| sqrt | BN254 scalar | 34,616 | 108,085 (3.1×) | 47,313 (1.4×) | 196,081 (5.7×) | 81,894 (2.4×) |
| sqrt | secp256k1 base | 12,235 | 42,090 (3.4×) | 19,095 (1.6×) | 72,835 (6.0×) | 30,979 (2.5×) |
| sqrt | P-256 base | 12,111 | 44,757 (3.7×) | 16,334 (1.3×) | 78,047 (6.4×) | 27,589 (2.3×) |
| sqrt | P-256 scalar | 24,192 | 74,255 (3.1×) | 35,837 (1.5×) | 113,654 (4.7×) | 64,107 (2.6×) |
| sqrt | Tom-256 base | 21,349 | 65,526 (3.1×) | 21,187 (1.0×) | 94,927 (4.4×) | 55,442 (2.6×) |
| sqrt | BLS12-381 base | 65,946 | 311,855 (4.7×) | 133,690 (2.0×) | 423,946 (6.4×) | 212,164 (3.2×) |
| sqrt | P-521 base | 68,286 | 320,120 (4.7×) | 143,071 (2.1×) | 473,856 (6.9×) | 268,222 (3.9×) |
| mul_u64 | Mersenne-31 | 5.8 | 9.4 (1.6×) | 9.3 (1.6×) | 9.6 (1.7×) | 9.7 (1.7×) |
| mul_u64 | BabyBear | 8.9 | 17.0 (1.9×) | 11.2 (1.3×) | 20.1 (2.3×) | 14.7 (1.7×) |
| mul_u64 | Goldilocks | 6.9 | 19.1 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 17.1 (2.5×) |
| mul_u64 | BN254 scalar | 20.9 | 99.6 (4.8×) | 28.8 (1.4×) | 124 (5.9×) | 44.1 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 42.2 (4.4×) | 12.6 (1.3×) | 65.1 (6.7×) | 23.0 (2.4×) |
| mul_u64 | P-256 base | 18.9 | 72.2 (3.8×) | 27.3 (1.4×) | 94.2 (5.0×) | 42.1 (2.2×) |
| mul_u64 | P-256 scalar | 22.4 | 84.1 (3.8×) | 30.3 (1.4×) | 113 (5.0×) | 48.0 (2.1×) |
| mul_u64 | Tom-256 base | 20.4 | 82.2 (4.0×) | 30.0 (1.5×) | 115 (5.6×) | 44.3 (2.2×) |
| mul_u64 | BLS12-381 base | 28.9 | 131 (4.5×) | 44.2 (1.5×) | 167 (5.8×) | 61.5 (2.1×) |
| mul_u64 | P-521 base | 20.0 | 92.1 (4.6×) | 32.1 (1.6×) | 232 (11.6×) | 49.4 (2.5×) |
| mul_uint | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.7 (1.6×) | 9.7 (1.6×) |
| mul_uint | BabyBear | 8.9 | 17.0 (1.9×) | 11.2 (1.3×) | 20.0 (2.2×) | 14.7 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 19.1 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 17.1 (2.5×) |
| mul_uint | BN254 scalar | 103 | 433 (4.2×) | 177 (1.7×) | 542 (5.3×) | 253 (2.5×) |
| mul_uint | secp256k1 base | 56.8 | 266 (4.7×) | 113 (2.0×) | 324 (5.7×) | 164 (2.9×) |
| mul_uint | P-256 base | 101 | 353 (3.5×) | 173 (1.7×) | 430 (4.3×) | 236 (2.3×) |
| mul_uint | P-256 scalar | 110 | 426 (3.9×) | 188 (1.7×) | 473 (4.3×) | 263 (2.4×) |
| mul_uint | Tom-256 base | 108 | 410 (3.8×) | 178 (1.6×) | 488 (4.5×) | 254 (2.4×) |
| mul_uint | BLS12-381 base | 215 | 990 (4.6×) | 380 (1.8×) | 1,155 (5.4×) | 552 (2.6×) |
| mul_uint | P-521 base | 361 | 1,472 (4.1×) | 618 (1.7×) | 2,383 (6.6×) | 986 (2.7×) |
| add_u64 | Mersenne-31 | 4.6 | 8.1 (1.8×) | 8.1 (1.8×) | 9.2 (2.0×) | 9.1 (2.0×) |
| add_u64 | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_u64 | Goldilocks | 4.7 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_u64 | BN254 scalar | 24.9 | 122 (4.9×) | 53.3 (2.1×) | 178 (7.1×) | 86.7 (3.5×) |
| add_u64 | secp256k1 base | 5.9 | 8.0 (1.4×) | 8.0 (1.4×) | 8.0 (1.4×) | 11.5 (1.9×) |
| add_u64 | P-256 base | 25.9 | 72.3 (2.8×) | 36.8 (1.4×) | 85.7 (3.3×) | 67.5 (2.6×) |
| add_u64 | P-256 scalar | 25.4 | 70.2 (2.8×) | 40.4 (1.6×) | 72.5 (2.9×) | 83.3 (3.3×) |
| add_u64 | Tom-256 base | 26.4 | 89.6 (3.4×) | 41.6 (1.6×) | 109 (4.1×) | 71.1 (2.7×) |
| add_u64 | BLS12-381 base | 38.2 | 179 (4.7×) | 65.9 (1.7×) | 258 (6.8×) | 96.5 (2.5×) |
| add_u64 | P-521 base | 10.8 | 23.1 (2.1×) | 16.1 (1.5×) | 22.1 (2.0×) | 25.1 (2.3×) |
| add_uint | Mersenne-31 | 4.6 | 8.2 (1.8×) | 8.2 (1.8×) | 9.2 (2.0×) | 9.1 (2.0×) |
| add_uint | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_uint | Goldilocks | 4.8 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.3×) | 12.0 (2.5×) |
| add_uint | BN254 scalar | 87.8 | 290 (3.3×) | 142 (1.6×) | 414 (4.7×) | 212 (2.4×) |
| add_uint | secp256k1 base | 6.7 | 20.9 (3.1×) | 14.6 (2.2×) | 18.5 (2.8×) | 24.4 (3.6×) |
| add_uint | P-256 base | 61.7 | 204 (3.3×) | 87.2 (1.4×) | 333 (5.4×) | 165 (2.7×) |
| add_uint | P-256 scalar | 88.6 | 295 (3.3×) | 135 (1.5×) | 382 (4.3×) | 226 (2.6×) |
| add_uint | Tom-256 base | 87.3 | 294 (3.4×) | 134 (1.5×) | 374 (4.3×) | 212 (2.4×) |
| add_uint | BLS12-381 base | 170 | 836 (4.9×) | 343 (2.0×) | 1,036 (6.1×) | 521 (3.1×) |
| add_uint | P-521 base | 23.1 | 64.8 (2.8×) | 42.0 (1.8×) | 86.7 (3.8×) | 72.7 (3.1×) |
| sub_u64 | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.1 (1.5×) | 7.0 (1.5×) |
| sub_u64 | BabyBear | 6.0 | 11.0 (1.8×) | 8.5 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| sub_u64 | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.4 (2.7×) | 11.0 (2.6×) |
| sub_u64 | BN254 scalar | 24.8 | 113 (4.6×) | 51.3 (2.1×) | 139 (5.6×) | 73.6 (3.0×) |
| sub_u64 | secp256k1 base | 4.2 | 8.7 (2.1×) | 8.0 (1.9×) | 8.0 (1.9×) | 10.0 (2.4×) |
| sub_u64 | P-256 base | 20.0 | 71.4 (3.6×) | 36.8 (1.8×) | 80.9 (4.0×) | 80.9 (4.0×) |
| sub_u64 | P-256 scalar | 24.9 | 69.3 (2.8×) | 40.2 (1.6×) | 75.5 (3.0×) | 84.0 (3.4×) |
| sub_u64 | Tom-256 base | 24.4 | 88.1 (3.6×) | 50.1 (2.1×) | 106 (4.3×) | 72.2 (3.0×) |
| sub_u64 | BLS12-381 base | 37.9 | 162 (4.3×) | 67.6 (1.8×) | 250 (6.6×) | 96.3 (2.5×) |
| sub_u64 | P-521 base | 9.6 | 22.5 (2.3×) | 18.7 (1.9×) | 20.5 (2.1×) | 21.1 (2.2×) |
| sub_uint | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.1 (1.5×) | 7.0 (1.5×) |
| sub_uint | BabyBear | 5.9 | 11.0 (1.9×) | 8.5 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| sub_uint | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.3 (2.6×) | 11.0 (2.6×) |
| sub_uint | BN254 scalar | 84.2 | 287 (3.4×) | 146 (1.7×) | 417 (5.0×) | 216 (2.6×) |
| sub_uint | secp256k1 base | 5.2 | 21.2 (4.1×) | 14.6 (2.8×) | 19.3 (3.7×) | 23.5 (4.5×) |
| sub_uint | P-256 base | 62.0 | 206 (3.3×) | 87.8 (1.4×) | 376 (6.1×) | 165 (2.7×) |
| sub_uint | P-256 scalar | 81.8 | 290 (3.5×) | 133 (1.6×) | 391 (4.8×) | 227 (2.8×) |
| sub_uint | Tom-256 base | 83.9 | 294 (3.5×) | 131 (1.6×) | 369 (4.4×) | 217 (2.6×) |
| sub_uint | BLS12-381 base | 168 | 825 (4.9×) | 348 (2.1×) | 1,007 (6.0×) | 524 (3.1×) |
| sub_uint | P-521 base | 20.4 | 75.5 (3.7×) | 42.1 (2.1×) | 79.1 (3.9×) | 69.1 (3.4×) |
| from_u64 | Mersenne-31 | 3.2 | 4.7 (1.5×) | 4.7 (1.5×) | 5.2 (1.6×) | 5.2 (1.6×) |
| from_u64 | BabyBear | 6.3 | 11.1 (1.8×) | 7.3 (1.2×) | 12.9 (2.0×) | 10.3 (1.6×) |
| from_u64 | Goldilocks | 3.9 | 7.2 (1.8×) | 5.5 (1.4×) | 9.0 (2.3×) | 8.7 (2.2×) |
| from_u64 | BN254 scalar | 22.0 | 87.9 (4.0×) | 28.2 (1.3×) | 121 (5.5×) | 46.0 (2.1×) |
| from_u64 | secp256k1 base | 2.1 | 3.8 (1.8×) | 3.8 (1.8×) | 3.8 (1.8×) | 3.8 (1.8×) |
| from_u64 | P-256 base | 17.8 | 51.9 (2.9×) | 25.3 (1.4×) | 61.0 (3.4×) | 39.0 (2.2×) |
| from_u64 | P-256 scalar | 18.0 | 50.7 (2.8×) | 25.9 (1.4×) | 58.6 (3.3×) | 43.0 (2.4×) |
| from_u64 | Tom-256 base | 21.0 | 71.4 (3.4×) | 28.9 (1.4×) | 85.8 (4.1×) | 43.4 (2.1×) |
| from_u64 | BLS12-381 base | 29.2 | 137 (4.7×) | 44.6 (1.5×) | 217 (7.4×) | 64.0 (2.2×) |
| from_u64 | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64 | BN254 scalar | 611 | 1,215 (2.0×) | 834 (1.4×) | 1,432 (2.3×) | 982 (1.6×) |
| div_u64 | secp256k1 base | 568 | 1,000 (1.8×) | 766 (1.3×) | 1,233 (2.2×) | 914 (1.6×) |
| div_u64 | P-256 base | 601 | 1,089 (1.8×) | 867 (1.4×) | 1,350 (2.2×) | 948 (1.6×) |
| div_u64 | P-256 scalar | 618 | 1,183 (1.9×) | 891 (1.4×) | 1,422 (2.3×) | 1,004 (1.6×) |
| div_u64 | Tom-256 base | 606 | 1,192 (2.0×) | 823 (1.4×) | 1,493 (2.5×) | 972 (1.6×) |
| div_u64 | BLS12-381 base | 752 | 1,795 (2.4×) | 1,106 (1.5×) | 2,475 (3.3×) | 1,307 (1.7×) |
| div_u64 | P-521 base | 926 | 2,432 (2.6×) | 1,369 (1.5×) | 3,577 (3.9×) | 1,930 (2.1×) |
| div_uint | Mersenne-31 | 216 | 277 (1.3×) | 277 (1.3×) | 287 (1.3×) | 281 (1.3×) |
| div_uint | BabyBear | 219 | 278 (1.3×) | 278 (1.3×) | 284 (1.3×) | 291 (1.3×) |
| div_uint | Goldilocks | 435 | 550 (1.3×) | 544 (1.3×) | 594 (1.4×) | 586 (1.3×) |
| div_uint | BN254 scalar | 5,010 | 9,213 (1.8×) | 6,685 (1.3×) | 12,642 (2.5×) | 8,365 (1.7×) |
| div_uint | secp256k1 base | 4,894 | 8,585 (1.8×) | 6,068 (1.2×) | 11,908 (2.4×) | 8,118 (1.7×) |
| div_uint | P-256 base | 5,052 | 8,786 (1.7×) | 6,169 (1.2×) | 12,357 (2.4×) | 7,901 (1.6×) |
| div_uint | P-256 scalar | 4,963 | 9,231 (1.9×) | 6,641 (1.3×) | 12,629 (2.5×) | 8,667 (1.7×) |
| div_uint | Tom-256 base | 5,056 | 9,122 (1.8×) | 6,655 (1.3×) | 12,167 (2.4×) | 8,491 (1.7×) |
| div_uint | BLS12-381 base | 9,073 | 21,024 (2.3×) | 12,074 (1.3×) | 26,726 (2.9×) | 16,831 (1.9×) |
| div_uint | P-521 base | 14,964 | 34,627 (2.3×) | 18,428 (1.2×) | 49,325 (3.3×) | 30,683 (2.1×) |
| add, constant time | Mersenne-31 | 1.9 | 4.2 (2.2×) | 4.2 (2.2×) | 4.4 (2.3×) | 4.4 (2.3×) |
| add, constant time | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add, constant time | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.1 (1.6×) | 3.2 (1.6×) |
| add, constant time | BN254 scalar | 4.9 | 23.6 (4.8×) | 16.3 (3.3×) | 20.6 (4.2×) | 26.5 (5.4×) |
| add, constant time | secp256k1 base | 5.8 | 26.3 (4.5×) | 18.2 (3.1×) | 24.5 (4.2×) | 29.0 (5.0×) |
| add, constant time | P-256 base | 5.8 | 23.7 (4.1×) | 18.1 (3.1×) | 22.8 (3.9×) | 28.0 (4.8×) |
| add, constant time | P-256 scalar | 6.1 | 26.9 (4.4×) | 17.9 (2.9×) | 24.1 (4.0×) | 29.5 (4.8×) |
| add, constant time | Tom-256 base | 5.8 | 27.6 (4.8×) | 18.1 (3.1×) | 24.1 (4.2×) | 28.5 (4.9×) |
| add, constant time | BLS12-381 base | 9.6 | 42.9 (4.5×) | 25.7 (2.7×) | 35.4 (3.7×) | 41.1 (4.3×) |
| add, constant time | P-521 base | 12.2 | 76.7 (6.3×) | 46.5 (3.8×) | 77.3 (6.3×) | 74.0 (6.1×) |
| sub, constant time | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BabyBear | 1.9 | 2.5 (1.3×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub, constant time | BN254 scalar | 5.0 | 22.9 (4.6×) | 16.6 (3.3×) | 21.4 (4.3×) | 26.0 (5.2×) |
| sub, constant time | secp256k1 base | 5.2 | 19.8 (3.8×) | 14.4 (2.8×) | 18.6 (3.6×) | 22.5 (4.3×) |
| sub, constant time | P-256 base | 5.3 | 17.3 (3.3×) | 13.4 (2.5×) | 18.4 (3.5×) | 22.0 (4.2×) |
| sub, constant time | P-256 scalar | 5.0 | 21.2 (4.2×) | 15.0 (3.0×) | 22.2 (4.4×) | 24.0 (4.8×) |
| sub, constant time | Tom-256 base | 5.4 | 21.0 (3.9×) | 15.3 (2.8×) | 20.7 (3.8×) | 24.0 (4.4×) |
| sub, constant time | BLS12-381 base | 9.6 | 37.7 (3.9×) | 24.4 (2.5×) | 34.7 (3.6×) | 37.0 (3.9×) |
| sub, constant time | P-521 base | 12.0 | 56.2 (4.7×) | 37.2 (3.1×) | 58.0 (4.8×) | 53.0 (4.4×) |
| mul, constant time | Mersenne-31 | 2.7 | 5.1 (1.9×) | 5.2 (1.9×) | 5.5 (2.0×) | 5.5 (2.0×) |
| mul, constant time | BabyBear | 3.4 | 7.8 (2.3×) | 5.2 (1.5×) | 7.8 (2.3×) | 5.0 (1.5×) |
| mul, constant time | Goldilocks | 3.8 | 12.1 (3.2×) | 7.0 (1.8×) | 12.4 (3.3×) | 9.0 (2.4×) |
| mul, constant time | BN254 scalar | 67.7 | 236 (3.5×) | 81.1 (1.2×) | 415 (6.1×) | 145 (2.1×) |
| mul, constant time | secp256k1 base | 44.4 | 160 (3.6×) | 64.3 (1.4×) | 212 (4.8×) | 109 (2.5×) |
| mul, constant time | P-256 base | 49.0 | 177 (3.6×) | 72.0 (1.5×) | 281 (5.7×) | 120 (2.4×) |
| mul, constant time | P-256 scalar | 73.1 | 239 (3.3×) | 86.6 (1.2×) | 316 (4.3×) | 156 (2.1×) |
| mul, constant time | Tom-256 base | 69.9 | 236 (3.4×) | 80.1 (1.1×) | 304 (4.3×) | 156 (2.2×) |
| mul, constant time | BLS12-381 base | 139 | 727 (5.2×) | 248 (1.8×) | 916 (6.6×) | 421 (3.0×) |
| mul, constant time | P-521 base | 176 | 1,019 (5.8×) | 400 (2.3×) | 1,643 (9.3×) | 847 (4.8×) |
| square, constant time | Mersenne-31 | 2.7 | 5.2 (1.9×) | 5.2 (1.9×) | 5.4 (2.0×) | 5.4 (2.0×) |
| square, constant time | BabyBear | 3.6 | 8.3 (2.3×) | 4.9 (1.4×) | 8.0 (2.2×) | 5.8 (1.6×) |
| square, constant time | Goldilocks | 3.8 | 10.1 (2.7×) | 6.0 (1.6×) | 12.0 (3.2×) | 9.7 (2.6×) |
| square, constant time | BN254 scalar | 64.9 | 213 (3.3×) | 77.8 (1.2×) | 230 (3.5×) | 140 (2.2×) |
| square, constant time | secp256k1 base | 33.0 | 143 (4.3×) | 61.2 (1.9×) | 192 (5.8×) | 102 (3.1×) |
| square, constant time | P-256 base | 39.1 | 160 (4.1×) | 78.4 (2.0×) | 253 (6.5×) | 113 (2.9×) |
| square, constant time | P-256 scalar | 63.9 | 214 (3.3×) | 81.6 (1.3×) | 280 (4.4×) | 155 (2.4×) |
| square, constant time | Tom-256 base | 56.9 | 204 (3.6×) | 84.4 (1.5×) | 226 (4.0×) | 158 (2.8×) |
| square, constant time | BLS12-381 base | 137 | 577 (4.2×) | 282 (2.1×) | 889 (6.5×) | 448 (3.3×) |
| square, constant time | P-521 base | 121 | 559 (4.6×) | 262 (2.2×) | 915 (7.6×) | 420 (3.5×) |
| invert, constant time | Mersenne-31 | 251 | 438 (1.7×) | 438 (1.7×) | 430 (1.7×) | 428 (1.7×) |
| invert, constant time | BabyBear | 381 | 520 (1.4×) | 459 (1.2×) | 550 (1.4×) | 417 (1.1×) |
| invert, constant time | Goldilocks | 647 | 1,261 (1.9×) | 856 (1.3×) | 1,559 (2.4×) | 1,194 (1.8×) |
| invert, constant time | BN254 scalar | 6,759 | 12,021 (1.8×) | 9,515 (1.4×) | 16,834 (2.5×) | 12,527 (1.9×) |
| invert, constant time | secp256k1 base | 6,864 | 11,868 (1.7×) | 9,096 (1.3×) | 16,751 (2.4×) | 12,594 (1.8×) |
| invert, constant time | P-256 base | 6,510 | 11,532 (1.8×) | 8,923 (1.4×) | 16,376 (2.5×) | 12,309 (1.9×) |
| invert, constant time | P-256 scalar | 6,903 | 11,880 (1.7×) | 9,328 (1.4×) | 16,986 (2.5×) | 12,736 (1.8×) |
| invert, constant time | Tom-256 base | 7,006 | 11,796 (1.7×) | 9,293 (1.3×) | 16,889 (2.4×) | 12,702 (1.8×) |
| invert, constant time | BLS12-381 base | 12,601 | 24,008 (1.9×) | 16,592 (1.3×) | 32,855 (2.6×) | 24,079 (1.9×) |
| invert, constant time | P-521 base | 21,309 | 41,752 (2.0×) | 26,001 (1.2×) | 63,515 (3.0×) | 41,255 (1.9×) |
| sqrt, constant time | Mersenne-31 | 142 | 334 (2.4×) | 334 (2.4×) | 341 (2.4×) | 340 (2.4×) |
| sqrt, constant time | BabyBear | 5,597 | 7,466 (1.3×) | 6,456 (1.2×) | 7,849 (1.4×) | 6,489 (1.2×) |
| sqrt, constant time | Goldilocks | 7,148 | 12,876 (1.8×) | 8,887 (1.2×) | 15,824 (2.2×) | 21,641 (3.0×) |
| sqrt, constant time | BN254 scalar | 58,595 | 178,607 (3.0×) | 83,838 (1.4×) | 298,605 (5.1×) | 152,636 (2.6×) |
| sqrt, constant time | secp256k1 base | 12,287 | 42,199 (3.4×) | 19,399 (1.6×) | 68,083 (5.5×) | 31,139 (2.5×) |
| sqrt, constant time | P-256 base | 11,993 | 44,750 (3.7×) | 15,771 (1.3×) | 79,517 (6.6×) | 32,304 (2.7×) |
| sqrt, constant time | P-256 scalar | 25,618 | 78,923 (3.1×) | 37,110 (1.4×) | 120,380 (4.7×) | 68,158 (2.7×) |
| sqrt, constant time | Tom-256 base | 20,056 | 65,472 (3.3×) | 32,400 (1.6×) | 94,922 (4.7×) | 52,551 (2.6×) |
| sqrt, constant time | BLS12-381 base | 68,093 | 309,849 (4.6×) | 136,709 (2.0×) | 422,644 (6.2×) | 211,927 (3.1×) |
| sqrt, constant time | P-521 base | 68,425 | 325,144 (4.8×) | 135,728 (2.0×) | 504,081 (7.4×) | 272,366 (4.0×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.6 (1.6×) | 9.7 (1.6×) |
| mul_u64, constant time | BabyBear | 9.0 | 17.0 (1.9×) | 11.2 (1.2×) | 20.1 (2.2×) | 14.7 (1.6×) |
| mul_u64, constant time | Goldilocks | 6.9 | 19.0 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 17.1 (2.5×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.9 (1.8×) | 125 (5.7×) | 57.4 (2.6×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 61.3 (5.6×) | 24.9 (2.3×) | 80.0 (7.3×) | 37.2 (3.4×) |
| mul_u64, constant time | P-256 base | 24.5 | 85.3 (3.5×) | 40.3 (1.6×) | 101 (4.1×) | 53.7 (2.2×) |
| mul_u64, constant time | P-256 scalar | 22.9 | 102 (4.5×) | 43.5 (1.9×) | 122 (5.3×) | 62.0 (2.7×) |
| mul_u64, constant time | Tom-256 base | 22.4 | 95.4 (4.3×) | 41.8 (1.9×) | 122 (5.4×) | 57.6 (2.6×) |
| mul_u64, constant time | BLS12-381 base | 34.6 | 148 (4.3×) | 65.8 (1.9×) | 190 (5.5×) | 85.9 (2.5×) |
| mul_u64, constant time | P-521 base | 26.4 | 138 (5.2×) | 69.3 (2.6×) | 138 (5.2×) | 95.7 (3.6×) |
| mul_uint, constant time | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.7 (1.6×) | 9.7 (1.6×) |
| mul_uint, constant time | BabyBear | 9.0 | 17.1 (1.9×) | 11.2 (1.2×) | 20.1 (2.2×) | 14.7 (1.6×) |
| mul_uint, constant time | Goldilocks | 7.0 | 19.1 (2.7×) | 11.0 (1.6×) | 21.1 (3.0×) | 17.1 (2.4×) |
| mul_uint, constant time | BN254 scalar | 119 | 487 (4.1×) | 246 (2.1×) | 597 (5.0×) | 312 (2.6×) |
| mul_uint, constant time | secp256k1 base | 78.3 | 322 (4.1×) | 192 (2.5×) | 380 (4.9×) | 251 (3.2×) |
| mul_uint, constant time | P-256 base | 119 | 412 (3.5×) | 268 (2.3×) | 492 (4.1×) | 301 (2.5×) |
| mul_uint, constant time | P-256 scalar | 132 | 471 (3.6×) | 261 (2.0×) | 562 (4.3×) | 336 (2.5×) |
| mul_uint, constant time | Tom-256 base | 122 | 465 (3.8×) | 259 (2.1×) | 562 (4.6×) | 325 (2.7×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,084 (4.2×) | 513 (2.0×) | 1,694 (6.6×) | 689 (2.7×) |
| mul_uint, constant time | P-521 base | 434 | 1,993 (4.6×) | 974 (2.2×) | 2,923 (6.7×) | 1,496 (3.4×) |
| add_u64, constant time | Mersenne-31 | 4.6 | 8.1 (1.8×) | 8.1 (1.8×) | 9.1 (2.0×) | 9.2 (2.0×) |
| add_u64, constant time | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_u64, constant time | Goldilocks | 4.7 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_u64, constant time | BN254 scalar | 27.8 | 135 (4.9×) | 58.3 (2.1×) | 163 (5.9×) | 81.4 (2.9×) |
| add_u64, constant time | secp256k1 base | 5.9 | 20.5 (3.5×) | 16.1 (2.7×) | 20.2 (3.4×) | 25.1 (4.3×) |
| add_u64, constant time | P-256 base | 25.7 | 90.1 (3.5×) | 58.7 (2.3×) | 104 (4.0×) | 71.5 (2.8×) |
| add_u64, constant time | P-256 scalar | 27.1 | 89.8 (3.3×) | 56.8 (2.1×) | 100 (3.7×) | 77.1 (2.8×) |
| add_u64, constant time | Tom-256 base | 27.4 | 115 (4.2×) | 62.6 (2.3×) | 132 (4.8×) | 85.2 (3.1×) |
| add_u64, constant time | BLS12-381 base | 43.8 | 192 (4.4×) | 93.2 (2.1×) | 236 (5.4×) | 127 (2.9×) |
| add_u64, constant time | P-521 base | 10.8 | 64.6 (6.0×) | 44.1 (4.1×) | 74.7 (6.9×) | 64.1 (5.9×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 8.1 (1.8×) | 8.1 (1.8×) | 9.1 (2.0×) | 9.2 (2.0×) |
| add_uint, constant time | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_uint, constant time | Goldilocks | 4.7 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_uint, constant time | BN254 scalar | 92.8 | 327 (3.5×) | 131 (1.4×) | 429 (4.6×) | 216 (2.3×) |
| add_uint, constant time | secp256k1 base | 9.2 | 41.3 (4.5×) | 29.4 (3.2×) | 37.5 (4.1×) | 44.4 (4.8×) |
| add_uint, constant time | P-256 base | 70.3 | 235 (3.3×) | 116 (1.7×) | 300 (4.3×) | 168 (2.4×) |
| add_uint, constant time | P-256 scalar | 95.8 | 302 (3.2×) | 129 (1.3×) | 364 (3.8×) | 192 (2.0×) |
| add_uint, constant time | Tom-256 base | 84.3 | 300 (3.6×) | 114 (1.4×) | 365 (4.3×) | 205 (2.4×) |
| add_uint, constant time | BLS12-381 base | 176 | 843 (4.8×) | 361 (2.1×) | 1,051 (6.0×) | 551 (3.1×) |
| add_uint, constant time | P-521 base | 22.3 | 135 (6.1×) | 105 (4.7×) | 142 (6.4×) | 138 (6.2×) |
| sub_u64, constant time | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.0 (1.5×) |
| sub_u64, constant time | BabyBear | 6.0 | 11.0 (1.8×) | 8.5 (1.4×) | 15.8 (2.6×) | 12.5 (2.1×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 10.4 (2.4×) | 11.0 (2.6×) |
| sub_u64, constant time | BN254 scalar | 27.9 | 125 (4.5×) | 60.0 (2.2×) | 155 (5.6×) | 80.1 (2.9×) |
| sub_u64, constant time | secp256k1 base | 4.2 | 15.5 (3.7×) | 12.3 (2.9×) | 14.6 (3.5×) | 18.5 (4.4×) |
| sub_u64, constant time | P-256 base | 23.3 | 84.7 (3.6×) | 52.4 (2.2×) | 96.3 (4.1×) | 68.0 (2.9×) |
| sub_u64, constant time | P-256 scalar | 26.4 | 80.6 (3.1×) | 52.5 (2.0×) | 93.7 (3.5×) | 77.5 (2.9×) |
| sub_u64, constant time | Tom-256 base | 27.6 | 113 (4.1×) | 56.4 (2.0×) | 124 (4.5×) | 80.4 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 42.7 | 183 (4.3×) | 86.5 (2.0×) | 242 (5.7×) | 118 (2.8×) |
| sub_u64, constant time | P-521 base | 9.7 | 42.2 (4.4×) | 34.1 (3.5×) | 52.3 (5.4×) | 44.2 (4.6×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.1 (1.5×) |
| sub_uint, constant time | BabyBear | 6.0 | 11.1 (1.8×) | 8.5 (1.4×) | 15.8 (2.6×) | 12.5 (2.1×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 10.4 (2.4×) | 11.0 (2.6×) |
| sub_uint, constant time | BN254 scalar | 91.9 | 320 (3.5×) | 137 (1.5×) | 412 (4.5×) | 218 (2.4×) |
| sub_uint, constant time | secp256k1 base | 7.3 | 33.6 (4.6×) | 26.0 (3.6×) | 34.4 (4.7×) | 38.1 (5.2×) |
| sub_uint, constant time | P-256 base | 64.8 | 229 (3.5×) | 106 (1.6×) | 302 (4.7×) | 163 (2.5×) |
| sub_uint, constant time | P-256 scalar | 91.0 | 295 (3.2×) | 122 (1.3×) | 364 (4.0×) | 208 (2.3×) |
| sub_uint, constant time | Tom-256 base | 91.4 | 300 (3.3×) | 129 (1.4×) | 343 (3.8×) | 202 (2.2×) |
| sub_uint, constant time | BLS12-381 base | 175 | 828 (4.7×) | 364 (2.1×) | 1,034 (5.9×) | 547 (3.1×) |
| sub_uint, constant time | P-521 base | 19.1 | 127 (6.6×) | 86.9 (4.5×) | 127 (6.6×) | 114 (6.0×) |
| from_u64, constant time | Mersenne-31 | 3.2 | 4.7 (1.5×) | 4.7 (1.5×) | 5.2 (1.6×) | 5.2 (1.6×) |
| from_u64, constant time | BabyBear | 10.3 | 13.5 (1.3×) | 10.3 (1.0×) | 14.0 (1.4×) | 12.7 (1.2×) |
| from_u64, constant time | Goldilocks | 7.9 | 14.9 (1.9×) | 9.3 (1.2×) | 17.4 (2.2×) | 12.5 (1.6×) |
| from_u64, constant time | BN254 scalar | 60.6 | 201 (3.3×) | 81.0 (1.3×) | 270 (4.5×) | 119 (2.0×) |
| from_u64, constant time | secp256k1 base | 2.2 | 3.4 (1.5×) | 3.4 (1.5×) | 3.4 (1.5×) | 3.4 (1.5×) |
| from_u64, constant time | P-256 base | 30.6 | 179 (5.8×) | 49.9 (1.6×) | 211 (6.9×) | 84.0 (2.7×) |
| from_u64, constant time | P-256 scalar | 62.9 | 249 (4.0×) | 81.5 (1.3×) | 268 (4.3×) | 117 (1.9×) |
| from_u64, constant time | Tom-256 base | 55.0 | 249 (4.5×) | 79.0 (1.4×) | 265 (4.8×) | 113 (2.1×) |
| from_u64, constant time | BLS12-381 base | 148 | 433 (2.9×) | 155 (1.0×) | 605 (4.1×) | 273 (1.8×) |
| from_u64, constant time | P-521 base | 6.0 | 6.0 (1.0×) | 6.0 (1.0×) | 6.0 (1.0×) | 6.0 (1.0×) |
| div_u64, constant time | BN254 scalar | 6,843 | 12,356 (1.8×) | 9,500 (1.4×) | 17,229 (2.5×) | 12,948 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,837 | 11,975 (1.8×) | 9,072 (1.3×) | 17,026 (2.5×) | 12,853 (1.9×) |
| div_u64, constant time | P-256 base | 6,692 | 11,826 (1.8×) | 9,157 (1.4×) | 17,317 (2.6×) | 12,501 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,908 | 12,215 (1.8×) | 9,471 (1.4×) | 17,336 (2.5×) | 13,041 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,300 | 12,193 (1.7×) | 9,268 (1.3×) | 16,891 (2.3×) | 12,990 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,435 | 24,787 (2.0×) | 16,844 (1.4×) | 34,605 (2.8×) | 24,119 (1.9×) |
| div_u64, constant time | P-521 base | 20,370 | 42,839 (2.1×) | 26,172 (1.3×) | 65,412 (3.2×) | 43,437 (2.1×) |
| div_uint, constant time | Mersenne-31 | 260 | 455 (1.8×) | 455 (1.8×) | 460 (1.8×) | 465 (1.8×) |
| div_uint, constant time | BabyBear | 405 | 546 (1.3×) | 466 (1.2×) | 611 (1.5×) | 459 (1.1×) |
| div_uint, constant time | Goldilocks | 669 | 1,313 (2.0×) | 948 (1.4×) | 1,581 (2.4×) | 1,212 (1.8×) |
| div_uint, constant time | BN254 scalar | 6,894 | 12,497 (1.8×) | 9,578 (1.4×) | 17,677 (2.6×) | 13,063 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,776 | 11,942 (1.8×) | 9,100 (1.3×) | 16,863 (2.5×) | 12,854 (1.9×) |
| div_uint, constant time | P-256 base | 6,715 | 12,021 (1.8×) | 9,104 (1.4×) | 17,544 (2.6×) | 12,643 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,954 | 12,289 (1.8×) | 9,507 (1.4×) | 17,527 (2.5×) | 13,515 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,337 | 12,387 (1.7×) | 9,458 (1.3×) | 16,923 (2.3×) | 13,094 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,585 | 25,420 (2.0×) | 16,995 (1.4×) | 35,347 (2.8×) | 24,508 (1.9×) |
| div_uint, constant time | P-521 base | 20,388 | 42,601 (2.1×) | 26,251 (1.3×) | 65,267 (3.2×) | 43,462 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 7.5 (2.6×) | 7.5 (2.6×) | 7.7 (2.7×) | 7.7 (2.7×) |
| add | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.7 | 14.7 (2.2×) | 10.9 (1.6×) | 14.7 (2.2×) | 17.4 (2.6×) |
| add | secp256k1 base | 7.0 | 13.7 (2.0×) | 10.7 (1.5×) | 13.9 (2.0×) | 18.1 (2.6×) |
| add | P-256 base | 6.8 | 12.9 (1.9×) | 11.2 (1.6×) | 14.4 (2.1×) | 18.4 (2.7×) |
| add | P-256 scalar | 6.9 | 14.3 (2.1×) | 11.0 (1.6×) | 14.7 (2.1×) | 18.9 (2.7×) |
| add | Tom-256 base | 7.0 | 14.3 (2.0×) | 11.2 (1.6×) | 14.5 (2.1×) | 19.4 (2.8×) |
| add | BLS12-381 base | 9.0 | 25.0 (2.8×) | 18.8 (2.1×) | 24.8 (2.8×) | 29.8 (3.3×) |
| add | P-521 base | 14.9 | 46.0 (3.1×) | 28.9 (1.9×) | 46.6 (3.1×) | 47.5 (3.2×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.8 | 14.6 (2.1×) | 10.8 (1.6×) | 14.5 (2.1×) | 18.4 (2.7×) |
| sub | secp256k1 base | 7.8 | 16.5 (2.1×) | 10.7 (1.4×) | 13.5 (1.7×) | 17.8 (2.3×) |
| sub | P-256 base | 6.8 | 14.6 (2.1×) | 10.5 (1.5×) | 12.7 (1.9×) | 17.4 (2.6×) |
| sub | P-256 scalar | 6.8 | 15.2 (2.2×) | 10.7 (1.6×) | 14.0 (2.1×) | 18.4 (2.7×) |
| sub | Tom-256 base | 7.8 | 14.9 (1.9×) | 10.7 (1.4×) | 13.7 (1.8×) | 18.3 (2.3×) |
| sub | BLS12-381 base | 10.0 | 25.3 (2.5×) | 17.5 (1.8×) | 24.8 (2.5×) | 30.4 (3.0×) |
| sub | P-521 base | 12.4 | 43.5 (3.5×) | 27.1 (2.2×) | 43.1 (3.5×) | 45.6 (3.7×) |
| mul | Mersenne-31 | 6.9 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| mul | BabyBear | 10.9 | 13.0 (1.2×) | 11.0 (1.0×) | 14.5 (1.3×) | 11.0 (1.0×) |
| mul | Goldilocks | 9.0 | 18.0 (2.0×) | 14.4 (1.6×) | 22.0 (2.4×) | 14.0 (1.6×) |
| mul | BN254 scalar | 72.4 | 228 (3.1×) | 90.8 (1.3×) | 365 (5.0×) | 151 (2.1×) |
| mul | secp256k1 base | 54.8 | 172 (3.1×) | 77.5 (1.4×) | 221 (4.0×) | 109 (2.0×) |
| mul | P-256 base | 60.0 | 190 (3.2×) | 72.5 (1.2×) | 292 (4.9×) | 118 (2.0×) |
| mul | P-256 scalar | 83.9 | 235 (2.8×) | 93.4 (1.1×) | 320 (3.8×) | 164 (2.0×) |
| mul | Tom-256 base | 75.8 | 236 (3.1×) | 87.0 (1.1×) | 312 (4.1×) | 160 (2.1×) |
| mul | BLS12-381 base | 141 | 722 (5.1×) | 244 (1.7×) | 908 (6.4×) | 430 (3.0×) |
| mul | P-521 base | 190 | 1,062 (5.6×) | 416 (2.2×) | 1,605 (8.4×) | 865 (4.6×) |
| square | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| square | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 11.0 (1.0×) |
| square | Goldilocks | 9.0 | 17.3 (1.9×) | 12.0 (1.3×) | 21.2 (2.4×) | 14.4 (1.6×) |
| square | BN254 scalar | 66.9 | 206 (3.1×) | 90.0 (1.3×) | 363 (5.4×) | 141 (2.1×) |
| square | secp256k1 base | 42.9 | 154 (3.6×) | 69.9 (1.6×) | 198 (4.6×) | 82.9 (1.9×) |
| square | P-256 base | 43.9 | 155 (3.5×) | 62.2 (1.4×) | 225 (5.1×) | 104 (2.4×) |
| square | P-256 scalar | 72.9 | 217 (3.0×) | 89.5 (1.2×) | 240 (3.3×) | 150 (2.1×) |
| square | Tom-256 base | 64.4 | 201 (3.1×) | 83.0 (1.3×) | 234 (3.6×) | 142 (2.2×) |
| square | BLS12-381 base | 137 | 609 (4.4×) | 281 (2.1×) | 856 (6.2×) | 441 (3.2×) |
| square | P-521 base | 133 | 571 (4.3×) | 263 (2.0×) | 908 (6.8×) | 415 (3.1×) |
| mul_u64 | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_u64 | BabyBear | 10.9 | 17.0 (1.6×) | 11.0 (1.0×) | 17.1 (1.6×) | 13.1 (1.2×) |
| mul_u64 | Goldilocks | 9.1 | 20.0 (2.2×) | 12.0 (1.3×) | 25.5 (2.8×) | 17.1 (1.9×) |
| mul_u64 | BN254 scalar | 25.9 | 91.7 (3.5×) | 34.5 (1.3×) | 120 (4.6×) | 45.1 (1.7×) |
| mul_u64 | secp256k1 base | 14.9 | 49.0 (3.3×) | 16.0 (1.1×) | 65.0 (4.4×) | 23.0 (1.5×) |
| mul_u64 | P-256 base | 25.0 | 75.0 (3.0×) | 33.0 (1.3×) | 89.1 (3.6×) | 41.8 (1.7×) |
| mul_u64 | P-256 scalar | 27.9 | 82.1 (2.9×) | 34.6 (1.2×) | 109 (3.9×) | 47.4 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.9 (3.0×) | 33.0 (1.2×) | 105 (3.8×) | 45.3 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 123 (3.6×) | 47.8 (1.4×) | 158 (4.6×) | 62.1 (1.8×) |
| mul_u64 | P-521 base | 32.8 | 93.3 (2.8×) | 39.6 (1.2×) | 161 (4.9×) | 52.7 (1.6×) |
| mul_uint | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_uint | BabyBear | 10.9 | 17.0 (1.6×) | 11.0 (1.0×) | 17.0 (1.6×) | 13.1 (1.2×) |
| mul_uint | Goldilocks | 9.1 | 20.2 (2.2×) | 12.0 (1.3×) | 25.5 (2.8×) | 17.1 (1.9×) |
| mul_uint | BN254 scalar | 114 | 434 (3.8×) | 185 (1.6×) | 532 (4.7×) | 251 (2.2×) |
| mul_uint | secp256k1 base | 67.9 | 265 (3.9×) | 118 (1.7×) | 323 (4.8×) | 163 (2.4×) |
| mul_uint | P-256 base | 110 | 353 (3.2×) | 180 (1.6×) | 421 (3.8×) | 237 (2.2×) |
| mul_uint | P-256 scalar | 116 | 425 (3.7×) | 189 (1.6×) | 474 (4.1×) | 263 (2.3×) |
| mul_uint | Tom-256 base | 114 | 409 (3.6×) | 177 (1.6×) | 484 (4.2×) | 255 (2.2×) |
| mul_uint | BLS12-381 base | 222 | 982 (4.4×) | 380 (1.7×) | 1,147 (5.2×) | 553 (2.5×) |
| mul_uint | P-521 base | 392 | 1,544 (3.9×) | 644 (1.6×) | 2,381 (6.1×) | 990 (2.5×) |
| add_u64 | Mersenne-31 | 4.6 | 8.6 (1.9×) | 8.7 (1.9×) | 9.3 (2.0×) | 9.3 (2.0×) |
| add_u64 | BabyBear | 6.5 | 12.1 (1.9×) | 8.2 (1.3×) | 12.7 (2.0×) | 11.0 (1.7×) |
| add_u64 | Goldilocks | 4.7 | 9.1 (1.9×) | 7.3 (1.6×) | 10.7 (2.3×) | 10.0 (2.1×) |
| add_u64 | BN254 scalar | 24.9 | 117 (4.7×) | 44.5 (1.8×) | 142 (5.7×) | 65.2 (2.6×) |
| add_u64 | secp256k1 base | 7.3 | 7.0 (1.0×) | 6.0 (0.8×) | 8.0 (1.1×) | 10.0 (1.4×) |
| add_u64 | P-256 base | 22.0 | 68.9 (3.1×) | 35.2 (1.6×) | 77.5 (3.5×) | 57.4 (2.6×) |
| add_u64 | P-256 scalar | 22.7 | 69.2 (3.0×) | 38.4 (1.7×) | 74.2 (3.3×) | 65.7 (2.9×) |
| add_u64 | Tom-256 base | 26.0 | 88.4 (3.4×) | 39.7 (1.5×) | 100 (3.8×) | 64.8 (2.5×) |
| add_u64 | BLS12-381 base | 37.1 | 160 (4.3×) | 62.0 (1.7×) | 265 (7.1×) | 90.7 (2.4×) |
| add_u64 | P-521 base | 12.7 | 21.1 (1.7×) | 12.3 (1.0×) | 19.5 (1.5×) | 26.8 (2.1×) |
| add_uint | Mersenne-31 | 4.6 | 8.7 (1.9×) | 8.7 (1.9×) | 9.2 (2.0×) | 9.3 (2.0×) |
| add_uint | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.5 | 9.1 (2.0×) | 7.3 (1.6×) | 10.7 (2.4×) | 10.0 (2.2×) |
| add_uint | BN254 scalar | 84.0 | 284 (3.4×) | 136 (1.6×) | 421 (5.0×) | 200 (2.4×) |
| add_uint | secp256k1 base | 8.2 | 17.0 (2.1×) | 12.6 (1.5×) | 16.6 (2.0×) | 23.2 (2.8×) |
| add_uint | P-256 base | 62.4 | 197 (3.2×) | 88.0 (1.4×) | 316 (5.1×) | 161 (2.6×) |
| add_uint | P-256 scalar | 89.8 | 288 (3.2×) | 124 (1.4×) | 371 (4.1×) | 213 (2.4×) |
| add_uint | Tom-256 base | 83.5 | 289 (3.5×) | 124 (1.5×) | 363 (4.3×) | 204 (2.4×) |
| add_uint | BLS12-381 base | 166 | 807 (4.9×) | 320 (1.9×) | 1,002 (6.0×) | 505 (3.0×) |
| add_uint | P-521 base | 23.7 | 62.2 (2.6×) | 42.0 (1.8×) | 85.4 (3.6×) | 67.0 (2.8×) |
| sub_u64 | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.3 (1.2×) | 6.6 (1.5×) | 6.6 (1.5×) |
| sub_u64 | BabyBear | 6.9 | 11.9 (1.7×) | 8.2 (1.2×) | 12.5 (1.8×) | 10.9 (1.6×) |
| sub_u64 | Goldilocks | 4.2 | 8.9 (2.1×) | 6.5 (1.5×) | 10.7 (2.5×) | 9.7 (2.3×) |
| sub_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 45.6 (1.8×) | 136 (5.5×) | 66.6 (2.7×) |
| sub_u64 | secp256k1 base | 6.9 | 7.0 (1.0×) | 6.1 (0.9×) | 7.0 (1.0×) | 9.5 (1.4×) |
| sub_u64 | P-256 base | 21.3 | 71.3 (3.3×) | 35.2 (1.7×) | 80.2 (3.8×) | 57.1 (2.7×) |
| sub_u64 | P-256 scalar | 23.6 | 68.8 (2.9×) | 37.3 (1.6×) | 73.6 (3.1×) | 65.4 (2.8×) |
| sub_u64 | Tom-256 base | 25.0 | 91.7 (3.7×) | 39.2 (1.6×) | 104 (4.2×) | 65.5 (2.6×) |
| sub_u64 | BLS12-381 base | 38.6 | 155 (4.0×) | 62.8 (1.6×) | 272 (7.0×) | 91.0 (2.4×) |
| sub_u64 | P-521 base | 11.1 | 21.1 (1.9×) | 15.2 (1.4×) | 22.2 (2.0×) | 21.7 (2.0×) |
| sub_uint | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.3 (1.2×) | 6.6 (1.5×) | 6.6 (1.5×) |
| sub_uint | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 13.0 (1.9×) | 10.9 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.9 (2.1×) | 6.5 (1.5×) | 10.4 (2.5×) | 9.7 (2.3×) |
| sub_uint | BN254 scalar | 83.9 | 282 (3.4×) | 138 (1.6×) | 420 (5.0×) | 204 (2.4×) |
| sub_uint | secp256k1 base | 6.8 | 18.9 (2.8×) | 12.4 (1.8×) | 16.8 (2.5×) | 22.0 (3.2×) |
| sub_uint | P-256 base | 59.7 | 202 (3.4×) | 88.3 (1.5×) | 382 (6.4×) | 163 (2.7×) |
| sub_uint | P-256 scalar | 87.0 | 286 (3.3×) | 124 (1.4×) | 379 (4.4×) | 211 (2.4×) |
| sub_uint | Tom-256 base | 78.0 | 287 (3.7×) | 121 (1.6×) | 363 (4.7×) | 208 (2.7×) |
| sub_uint | BLS12-381 base | 169 | 809 (4.8×) | 324 (1.9×) | 999 (5.9×) | 505 (3.0×) |
| sub_uint | P-521 base | 15.8 | 67.4 (4.3×) | 38.7 (2.4×) | 75.2 (4.8×) | 67.4 (4.3×) |
| div_u64 | BN254 scalar | 608 | 1,208 (2.0×) | 842 (1.4×) | 1,432 (2.4×) | 988 (1.6×) |
| div_u64 | secp256k1 base | 567 | 1,017 (1.8×) | 777 (1.4×) | 1,228 (2.2×) | 874 (1.5×) |
| div_u64 | P-256 base | 609 | 1,086 (1.8×) | 887 (1.5×) | 1,342 (2.2×) | 944 (1.6×) |
| div_u64 | P-256 scalar | 620 | 1,180 (1.9×) | 899 (1.4×) | 1,419 (2.3×) | 994 (1.6×) |
| div_u64 | Tom-256 base | 610 | 1,181 (1.9×) | 854 (1.4×) | 1,484 (2.4×) | 966 (1.6×) |
| div_u64 | BLS12-381 base | 752 | 1,799 (2.4×) | 1,133 (1.5×) | 2,464 (3.3×) | 1,303 (1.7×) |
| div_u64 | P-521 base | 927 | 2,447 (2.6×) | 1,375 (1.5×) | 3,585 (3.9×) | 1,883 (2.0×) |
| div_uint | Mersenne-31 | 216 | 277 (1.3×) | 276 (1.3×) | 283 (1.3×) | 283 (1.3×) |
| div_uint | BabyBear | 216 | 279 (1.3×) | 278 (1.3×) | 287 (1.3×) | 287 (1.3×) |
| div_uint | Goldilocks | 428 | 553 (1.3×) | 544 (1.3×) | 591 (1.4×) | 588 (1.4×) |
| div_uint | BN254 scalar | 4,993 | 9,060 (1.8×) | 6,650 (1.3×) | 12,599 (2.5×) | 8,483 (1.7×) |
| div_uint | secp256k1 base | 4,878 | 8,605 (1.8×) | 6,042 (1.2×) | 11,968 (2.5×) | 8,098 (1.7×) |
| div_uint | P-256 base | 5,024 | 8,664 (1.7×) | 6,126 (1.2×) | 12,034 (2.4×) | 7,872 (1.6×) |
| div_uint | P-256 scalar | 4,949 | 9,062 (1.8×) | 6,538 (1.3×) | 12,578 (2.5×) | 8,639 (1.7×) |
| div_uint | Tom-256 base | 5,039 | 8,947 (1.8×) | 6,614 (1.3×) | 12,177 (2.4×) | 8,453 (1.7×) |
| div_uint | BLS12-381 base | 9,013 | 20,960 (2.3×) | 11,932 (1.3×) | 26,684 (3.0×) | 16,842 (1.9×) |
| div_uint | P-521 base | 14,943 | 34,184 (2.3×) | 18,379 (1.2×) | 49,547 (3.3×) | 30,667 (2.1×) |
| add, constant time | Mersenne-31 | 2.9 | 7.5 (2.6×) | 7.5 (2.6×) | 7.7 (2.7×) | 7.7 (2.7×) |
| add, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.7 | 24.4 (3.6×) | 19.5 (2.9×) | 23.1 (3.4×) | 25.7 (3.8×) |
| add, constant time | secp256k1 base | 6.7 | 28.0 (4.2×) | 22.6 (3.4×) | 27.0 (4.0×) | 29.8 (4.4×) |
| add, constant time | P-256 base | 6.9 | 26.8 (3.9×) | 22.8 (3.3×) | 24.9 (3.6×) | 28.0 (4.1×) |
| add, constant time | P-256 scalar | 6.7 | 27.3 (4.1×) | 23.8 (3.6×) | 26.8 (4.0×) | 30.1 (4.5×) |
| add, constant time | Tom-256 base | 6.6 | 26.6 (4.0×) | 23.8 (3.6×) | 27.0 (4.1×) | 29.0 (4.4×) |
| add, constant time | BLS12-381 base | 8.7 | 41.8 (4.8×) | 28.5 (3.3×) | 35.3 (4.1×) | 41.0 (4.7×) |
| add, constant time | P-521 base | 14.6 | 77.8 (5.3×) | 48.6 (3.3×) | 80.4 (5.5×) | 74.0 (5.1×) |
| sub, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 6.7 | 26.4 (3.9×) | 16.9 (2.5×) | 27.0 (4.0×) | 25.0 (3.7×) |
| sub, constant time | secp256k1 base | 7.8 | 21.2 (2.7×) | 15.5 (2.0×) | 20.4 (2.6×) | 21.5 (2.8×) |
| sub, constant time | P-256 base | 6.7 | 18.9 (2.8×) | 15.5 (2.3×) | 18.9 (2.8×) | 20.0 (3.0×) |
| sub, constant time | P-256 scalar | 6.8 | 21.6 (3.2×) | 15.8 (2.3×) | 18.8 (2.8×) | 21.5 (3.2×) |
| sub, constant time | Tom-256 base | 8.1 | 21.4 (2.6×) | 17.1 (2.1×) | 19.8 (2.4×) | 22.5 (2.8×) |
| sub, constant time | BLS12-381 base | 9.7 | 37.4 (3.9×) | 24.7 (2.5×) | 36.6 (3.8×) | 36.2 (3.7×) |
| sub, constant time | P-521 base | 12.3 | 57.5 (4.7×) | 39.9 (3.2×) | 59.9 (4.9×) | 53.0 (4.3×) |
| mul, constant time | Mersenne-31 | 6.9 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| mul, constant time | BabyBear | 10.9 | 13.0 (1.2×) | 11.0 (1.0×) | 14.5 (1.3×) | 11.0 (1.0×) |
| mul, constant time | Goldilocks | 9.0 | 17.7 (2.0×) | 14.4 (1.6×) | 22.0 (2.4×) | 14.0 (1.6×) |
| mul, constant time | BN254 scalar | 78.8 | 243 (3.1×) | 94.0 (1.2×) | 411 (5.2×) | 149 (1.9×) |
| mul, constant time | secp256k1 base | 54.8 | 171 (3.1×) | 77.5 (1.4×) | 226 (4.1×) | 108 (2.0×) |
| mul, constant time | P-256 base | 59.8 | 190 (3.2×) | 72.4 (1.2×) | 292 (4.9×) | 120 (2.0×) |
| mul, constant time | P-256 scalar | 83.8 | 234 (2.8×) | 93.5 (1.1×) | 320 (3.8×) | 164 (2.0×) |
| mul, constant time | Tom-256 base | 75.6 | 238 (3.1×) | 86.5 (1.1×) | 311 (4.1×) | 149 (2.0×) |
| mul, constant time | BLS12-381 base | 154 | 721 (4.7×) | 244 (1.6×) | 914 (5.9×) | 429 (2.8×) |
| mul, constant time | P-521 base | 193 | 1,064 (5.5×) | 417 (2.2×) | 1,604 (8.3×) | 866 (4.5×) |
| square, constant time | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| square, constant time | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 11.0 (1.0×) |
| square, constant time | Goldilocks | 9.0 | 17.3 (1.9×) | 12.0 (1.3×) | 21.2 (2.4×) | 14.4 (1.6×) |
| square, constant time | BN254 scalar | 66.9 | 219 (3.3×) | 82.5 (1.2×) | 220 (3.3×) | 147 (2.2×) |
| square, constant time | secp256k1 base | 42.9 | 154 (3.6×) | 69.9 (1.6×) | 198 (4.6×) | 95.7 (2.2×) |
| square, constant time | P-256 base | 43.9 | 155 (3.5×) | 62.1 (1.4×) | 227 (5.2×) | 104 (2.4×) |
| square, constant time | P-256 scalar | 72.9 | 222 (3.0×) | 89.1 (1.2×) | 238 (3.3×) | 150 (2.1×) |
| square, constant time | Tom-256 base | 64.4 | 200 (3.1×) | 83.0 (1.3×) | 234 (3.6×) | 142 (2.2×) |
| square, constant time | BLS12-381 base | 146 | 607 (4.2×) | 292 (2.0×) | 853 (5.8×) | 441 (3.0×) |
| square, constant time | P-521 base | 133 | 569 (4.3×) | 261 (2.0×) | 912 (6.9×) | 418 (3.1×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_u64, constant time | BabyBear | 10.9 | 17.0 (1.6×) | 11.0 (1.0×) | 17.1 (1.6×) | 13.1 (1.2×) |
| mul_u64, constant time | Goldilocks | 9.1 | 20.1 (2.2×) | 12.0 (1.3×) | 25.5 (2.8×) | 17.1 (1.9×) |
| mul_u64, constant time | BN254 scalar | 29.6 | 110 (3.7×) | 50.9 (1.7×) | 132 (4.5×) | 58.0 (2.0×) |
| mul_u64, constant time | secp256k1 base | 19.9 | 68.9 (3.5×) | 39.0 (2.0×) | 85.1 (4.3×) | 45.5 (2.3×) |
| mul_u64, constant time | P-256 base | 31.8 | 90.3 (2.8×) | 53.2 (1.7×) | 103 (3.2×) | 59.4 (1.9×) |
| mul_u64, constant time | P-256 scalar | 33.0 | 106 (3.2×) | 55.0 (1.7×) | 127 (3.8×) | 67.3 (2.0×) |
| mul_u64, constant time | Tom-256 base | 33.0 | 100 (3.0×) | 52.0 (1.6×) | 122 (3.7×) | 64.0 (1.9×) |
| mul_u64, constant time | BLS12-381 base | 39.6 | 154 (3.9×) | 72.0 (1.8×) | 174 (4.4×) | 88.0 (2.2×) |
| mul_u64, constant time | P-521 base | 44.1 | 139 (3.2×) | 77.9 (1.8×) | 171 (3.9×) | 95.0 (2.2×) |
| mul_uint, constant time | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_uint, constant time | BabyBear | 11.0 | 17.0 (1.5×) | 11.0 (1.0×) | 17.0 (1.5×) | 13.1 (1.2×) |
| mul_uint, constant time | Goldilocks | 9.1 | 20.1 (2.2×) | 12.0 (1.3×) | 25.5 (2.8×) | 17.1 (1.9×) |
| mul_uint, constant time | BN254 scalar | 141 | 479 (3.4×) | 251 (1.8×) | 594 (4.2×) | 311 (2.2×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 316 (3.8×) | 192 (2.3×) | 379 (4.5×) | 252 (3.0×) |
| mul_uint, constant time | P-256 base | 134 | 408 (3.0×) | 247 (1.8×) | 489 (3.6×) | 306 (2.3×) |
| mul_uint, constant time | P-256 scalar | 146 | 475 (3.3×) | 264 (1.8×) | 544 (3.7×) | 333 (2.3×) |
| mul_uint, constant time | Tom-256 base | 135 | 465 (3.4×) | 257 (1.9×) | 546 (4.0×) | 324 (2.4×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,079 (4.1×) | 506 (1.9×) | 1,690 (6.4×) | 684 (2.6×) |
| mul_uint, constant time | P-521 base | 471 | 2,021 (4.3×) | 998 (2.1×) | 2,905 (6.2×) | 1,470 (3.1×) |
| add_u64, constant time | Mersenne-31 | 4.6 | 8.7 (1.9×) | 8.7 (1.9×) | 9.3 (2.0×) | 9.3 (2.0×) |
| add_u64, constant time | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_u64, constant time | Goldilocks | 4.8 | 9.1 (1.9×) | 7.3 (1.5×) | 10.8 (2.3×) | 10.0 (2.1×) |
| add_u64, constant time | BN254 scalar | 28.9 | 140 (4.8×) | 56.2 (1.9×) | 148 (5.1×) | 81.6 (2.8×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.4 (3.4×) | 23.0 (3.1×) | 25.2 (3.4×) | 29.0 (3.9×) |
| add_u64, constant time | P-256 base | 24.0 | 89.5 (3.7×) | 56.3 (2.3×) | 95.8 (4.0×) | 74.3 (3.1×) |
| add_u64, constant time | P-256 scalar | 27.6 | 83.2 (3.0×) | 58.4 (2.1×) | 97.2 (3.5×) | 79.7 (2.9×) |
| add_u64, constant time | Tom-256 base | 27.1 | 119 (4.4×) | 62.0 (2.3×) | 133 (4.9×) | 84.9 (3.1×) |
| add_u64, constant time | BLS12-381 base | 44.8 | 185 (4.1×) | 96.0 (2.1×) | 236 (5.3×) | 122 (2.7×) |
| add_u64, constant time | P-521 base | 13.0 | 65.7 (5.1×) | 45.1 (3.5×) | 68.0 (5.2×) | 64.7 (5.0×) |
| add_uint, constant time | Mersenne-31 | 4.6 | 8.7 (1.9×) | 8.7 (1.9×) | 9.3 (2.0×) | 9.3 (2.0×) |
| add_uint, constant time | BabyBear | 7.0 | 11.6 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint, constant time | Goldilocks | 4.4 | 9.1 (2.1×) | 7.3 (1.7×) | 10.8 (2.5×) | 10.0 (2.3×) |
| add_uint, constant time | BN254 scalar | 91.8 | 314 (3.4×) | 129 (1.4×) | 369 (4.0×) | 205 (2.2×) |
| add_uint, constant time | secp256k1 base | 8.8 | 38.6 (4.4×) | 29.5 (3.4×) | 36.4 (4.1×) | 44.5 (5.1×) |
| add_uint, constant time | P-256 base | 69.0 | 218 (3.2×) | 108 (1.6×) | 299 (4.3×) | 160 (2.3×) |
| add_uint, constant time | P-256 scalar | 89.8 | 294 (3.3×) | 136 (1.5×) | 347 (3.9×) | 213 (2.4×) |
| add_uint, constant time | Tom-256 base | 86.7 | 290 (3.3×) | 128 (1.5×) | 364 (4.2×) | 207 (2.4×) |
| add_uint, constant time | BLS12-381 base | 179 | 841 (4.7×) | 351 (2.0×) | 1,025 (5.7×) | 544 (3.0×) |
| add_uint, constant time | P-521 base | 22.7 | 138 (6.1×) | 93.3 (4.1×) | 139 (6.1×) | 134 (5.9×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.3 (1.2×) | 6.5 (1.4×) | 6.6 (1.5×) |
| sub_u64, constant time | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 13.0 (1.9×) | 10.9 (1.6×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.9 (2.1×) | 6.6 (1.6×) | 10.7 (2.5×) | 9.4 (2.2×) |
| sub_u64, constant time | BN254 scalar | 29.8 | 125 (4.2×) | 56.9 (1.9×) | 153 (5.1×) | 79.9 (2.7×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.0 (2.6×) | 15.1 (2.2×) | 17.0 (2.5×) | 17.4 (2.5×) |
| sub_u64, constant time | P-256 base | 22.9 | 86.6 (3.8×) | 49.0 (2.1×) | 88.4 (3.9×) | 69.6 (3.0×) |
| sub_u64, constant time | P-256 scalar | 25.3 | 79.7 (3.2×) | 50.4 (2.0×) | 92.6 (3.7×) | 77.2 (3.1×) |
| sub_u64, constant time | Tom-256 base | 28.1 | 113 (4.0×) | 57.1 (2.0×) | 125 (4.4×) | 80.1 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 44.4 | 175 (3.9×) | 90.0 (2.0×) | 229 (5.2×) | 119 (2.7×) |
| sub_u64, constant time | P-521 base | 11.0 | 45.8 (4.2×) | 35.4 (3.2×) | 49.8 (4.5×) | 46.3 (4.2×) |
| sub_uint, constant time | Mersenne-31 | 4.5 | 5.4 (1.2×) | 5.3 (1.2×) | 6.6 (1.5×) | 6.5 (1.4×) |
| sub_uint, constant time | BabyBear | 6.9 | 12.0 (1.7×) | 8.2 (1.2×) | 13.3 (1.9×) | 10.8 (1.6×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.9 (2.1×) | 6.5 (1.5×) | 10.8 (2.6×) | 9.4 (2.2×) |
| sub_uint, constant time | BN254 scalar | 93.1 | 313 (3.4×) | 133 (1.4×) | 372 (4.0×) | 219 (2.4×) |
| sub_uint, constant time | secp256k1 base | 7.8 | 32.2 (4.1×) | 24.9 (3.2×) | 31.6 (4.1×) | 37.7 (4.8×) |
| sub_uint, constant time | P-256 base | 68.0 | 225 (3.3×) | 98.2 (1.4×) | 291 (4.3×) | 158 (2.3×) |
| sub_uint, constant time | P-256 scalar | 93.0 | 296 (3.2×) | 127 (1.4×) | 343 (3.7×) | 207 (2.2×) |
| sub_uint, constant time | Tom-256 base | 86.3 | 289 (3.3×) | 127 (1.5×) | 362 (4.2×) | 193 (2.2×) |
| sub_uint, constant time | BLS12-381 base | 172 | 814 (4.7×) | 346 (2.0×) | 1,029 (6.0×) | 541 (3.1×) |
| sub_uint, constant time | P-521 base | 19.5 | 124 (6.4×) | 85.3 (4.4×) | 119 (6.1×) | 112 (5.7×) |
| div_u64, constant time | BN254 scalar | 6,850 | 12,417 (1.8×) | 9,574 (1.4×) | 17,184 (2.5×) | 12,896 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,773 | 11,922 (1.8×) | 9,049 (1.3×) | 16,679 (2.5×) | 12,937 (1.9×) |
| div_u64, constant time | P-256 base | 6,687 | 11,832 (1.8×) | 9,131 (1.4×) | 17,350 (2.6×) | 12,483 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,894 | 12,347 (1.8×) | 9,427 (1.4×) | 17,251 (2.5×) | 13,117 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,290 | 12,203 (1.7×) | 9,343 (1.3×) | 16,988 (2.3×) | 12,925 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,396 | 24,797 (2.0×) | 16,774 (1.4×) | 34,568 (2.8×) | 24,124 (1.9×) |
| div_u64, constant time | P-521 base | 20,371 | 42,677 (2.1×) | 25,998 (1.3×) | 64,664 (3.2×) | 43,215 (2.1×) |
| div_uint, constant time | Mersenne-31 | 264 | 451 (1.7×) | 451 (1.7×) | 459 (1.7×) | 461 (1.7×) |
| div_uint, constant time | BabyBear | 405 | 545 (1.3×) | 437 (1.1×) | 607 (1.5×) | 450 (1.1×) |
| div_uint, constant time | Goldilocks | 655 | 1,308 (2.0×) | 946 (1.4×) | 1,576 (2.4×) | 1,209 (1.8×) |
| div_uint, constant time | BN254 scalar | 6,898 | 12,477 (1.8×) | 9,545 (1.4×) | 17,629 (2.6×) | 13,112 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,769 | 11,932 (1.8×) | 9,054 (1.3×) | 16,858 (2.5×) | 12,900 (1.9×) |
| div_uint, constant time | P-256 base | 6,748 | 11,900 (1.8×) | 9,122 (1.4×) | 17,440 (2.6×) | 12,584 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,916 | 12,390 (1.8×) | 9,496 (1.4×) | 17,609 (2.5×) | 13,334 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,328 | 12,363 (1.7×) | 9,432 (1.3×) | 16,861 (2.3×) | 13,134 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,511 | 25,401 (2.0×) | 16,947 (1.4×) | 35,267 (2.8×) | 24,562 (2.0×) |
| div_uint, constant time | P-521 base | 20,379 | 42,935 (2.1×) | 26,146 (1.3×) | 65,297 (3.2×) | 43,287 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
