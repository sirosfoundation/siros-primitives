# Benchmarks

`prime-field` at commit `13c9a8a` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 29.28 before, 41.75 after; its SMT sibling, CPU 29, 62% busy |
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
| from_u64 | – | – | `new` | `from_u64`, `From<u64>` | `From<u64>` | – | `new` | – |
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
| Mersenne-31 | **1.9** / **3.0** | – | 9.7 / 16.9 | – | – | 2.0 / 3.0\* | – | 2.5 / 5.0 | – |
| BabyBear | **1.9** / **2.9** | – | 9.7 / 17.2 | – | – | 2.0 / 3.0 | – | 2.5 / 5.0 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 19.0 | – | – | 1.9\* / 2.9\* | 2.4 / **2.9** | 2.5 / 4.9 | – |
| BN254 scalar | **5.1** / **6.7** | 17.2 / 12.1 | 66.6 / 94.0 | – | 12.8 / 21.8 | – | – | 7.0 / 10.1 | – |
| secp256k1 base | **5.9** / **7.0** | 17.3 / 12.8 | 66.6 / 85.9 | 2.8\* / 2.9\* | 14.1 / 21.1 | – | – | 6.8 / 10.0 | – |
| P-256 base | **5.9** / **7.0** | 18.4 / 12.7 | 66.6 / 76.9 | 10.8 / 18.6 | 10.7 / 19.5 | – | – | 7.0 / 10.4 | 316 / – |
| P-256 scalar | **5.8** / **7.0** | 17.4 / 13.6 | 66.6 / 94.5 | 17.6 / 21.0 | 14.5 / 21.0 | – | – | 7.0 / 10.2 | 318 / – |
| Tom-256 base | **6.0** / **7.0** | – | 66.5 / 80.6 | – | 11.8 / 20.5 | – | – | 7.0 / 10.4 | – |
| BLS12-381 base | **10.0** / **8.9** | 19.9 / 14.3 | 87.5 / 112 | – | 20.5 / 27.3 | – | – | 16.9 / 12.8 | – |
| P-521 base | **12.4** / **14.8** | – | 116 / 155 | – | – | – | – | 24.9 / 52.4 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.2 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| BabyBear | **1.9** / **3.0** | – | 4.2 / 11.9 | – | – | 2.0 / **3.0** | – | 2.0 / 4.0 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.2 / 11.7 | – | – | 1.9\* / 2.9\* | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.1** / **7.0** | 16.1 / 12.2 | 49.6 / 50.9 | – | 8.5 / 12.0 | – | – | 6.9 / 8.0 | – |
| secp256k1 base | **5.3** / **7.8** | 17.0 / 12.9 | 49.6 / 48.0 | 3.6\* / 3.7\* | 7.7 / 13.0 | – | – | 7.0 / 8.0 | – |
| P-256 base | **5.5** / **7.0** | 16.3 / 11.4 | 49.7 / 43.5 | 8.0 / 13.0 | 7.6 / 12.0 | – | – | 7.0 / 8.0 | 150 / – |
| P-256 scalar | **4.9** / **7.0** | 15.6 / 12.7 | 49.7 / 47.7 | 7.8 / 13.0 | 8.1 / 12.9 | – | – | 7.0 / 8.0 | 150 / – |
| Tom-256 base | **5.5** / 8.1 | – | 49.7 / 54.9 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **10.0** / **10.0** | 20.9 / 15.2 | 82.7 / 88.0 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 | – |
| P-521 base | **11.8** / **12.7** | – | 121 / 138 | – | – | – | – | 27.3 / 50.8 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **6.9** | – | 16.9 / 34.4 | – | – | 2.8 / 7.0 | – | 7.0 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.9 / 34.9 | – | – | 4.7 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.6 / 34.7 | – | – | 3.2\* / 9.9 | **3.7** / 11.0 | 7.0 / 17.0 | – |
| BN254 scalar | **68.8** / **72.4** | 74.0 / 104 | 119 / 151 | – | 91.4 / 93.8 | – | – | 94.1 / 129 | – |
| secp256k1 base | **44.6** / **54.9** | 89.2 / 114 | 118 / 152 | 84.8 / 88.4 | 88.1 / 96.0 | – | – | 94.1 / 129 | – |
| P-256 base | **49.4** / 60.0 | 66.2 / 86.6 | 117 / 152 | 68.1 / 72.2 | 67.9 / 73.0 | – | – | 50.7 / **56.7** | 563 / – |
| P-256 scalar | **75.0** / **84.0** | 91.9 / 113 | 117 / 152 | 140 / 141 | 94.0 / 100.0 | – | – | 93.7 / 128 | 630 / – |
| Tom-256 base | **71.1** / **76.0** | – | 117 / 152 | – | 89.8 / 94.8 | – | – | 93.9 / 128 | – |
| BLS12-381 base | **138** / **141** | 165 / 195 | 270 / 290 | – | 188 / 192 | – | – | 277 / 262 | – |
| P-521 base | **176** / **189** | – | 529 / 545 | – | – | – | – | 462 / 477 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 33.9 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.3** / **11.0** | – | 16.7 / 31.9 | – | – | 4.7 / 13.0 | – | 7.5 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.5 / 36.2 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.6 / 17.0 | – |
| BN254 scalar | **63.1** / **66.5** | 72.4 / 104 | 114 / 154 | – | 74.2 / 89.4 | – | – | 93.3 / 127 | – |
| secp256k1 base | **32.3** / **43.0** | 83.3 / 102 | 113 / 154 | 57.9 / 79.9 | 65.1 / 85.0 | – | – | 93.3 / 128 | – |
| P-256 base | **39.7** / **44.0** | 63.7 / 85.3 | 115 / 153 | 60.4 / 72.0 | 54.7 / 58.0 | – | – | 50.4 / 57.9 | 563 / – |
| P-256 scalar | **63.9** / **73.0** | 88.4 / 111 | 114 / 154 | 129 / 147 | 77.1 / 93.5 | – | – | 93.3 / 129 | 629 / – |
| Tom-256 base | **57.0** / **64.5** | – | 114 / 154 | – | 69.5 / 81.0 | – | – | 93.4 / 125 | – |
| BLS12-381 base | **132** / **137** | 140 / 167 | 268 / 289 | – | 152 / 166 | – | – | 276 / 277 | – |
| P-521 base | **118** / **133** | – | 536 / 553 | – | – | – | – | 463 / 469 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **210** / – | – | 1,430 / – | – | – | 630 / – | – | 870 / – | – |
| BabyBear | **211** / – | – | 1,402 / – | – | – | 621 / – | – | 1,050 / – | – |
| Goldilocks | **436** / – | – | 2,853 / – | – | – | 1,286 / – | 691 / – | 1,707 / – | – |
| BN254 scalar | **5,038** / – | 16,469 / – | 14,415 / – | – | 7,147 / – | – | – | 14,042 / – | – |
| secp256k1 base | **4,925** / – | 15,429 / – | 14,474 / – | 24,677 / – | 7,186 / – | – | – | 14,197 / – | – |
| P-256 base | **4,856** / – | 15,242 / – | 14,541 / – | 28,712 / – | 7,221 / – | – | – | 14,040 / – | 89,138 / – |
| P-256 scalar | **5,014** / – | 16,601 / – | 14,500 / – | 29,543 / – | 7,175 / – | – | – | 14,149 / – | 89,456 / – |
| Tom-256 base | **5,018** / – | – | 14,479 / – | – | 7,186 / – | – | – | 14,158 / – | – |
| BLS12-381 base | **8,836** / – | 36,458 / – | 25,092 / – | – | 12,039 / – | – | – | 35,258 / – | – |
| P-521 base | **14,222** / – | – | 43,943 / – | – | – | – | – | 71,176 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **135** / – | – | 1,652 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,984** / – | – | 18,660 / – | – | – | 2,706 / – | – | – |
| Goldilocks | **2,846** / – | – | 26,624 / – | – | – | 3,146 / – | – | – |
| BN254 scalar | **34,168** / – | 45,010 / – | 132,723 / – | – | 84,541 / – | – | – | – |
| secp256k1 base | **12,281** / – | 42,338 / – | 52,662 / – | 24,055 / – | 47,190 / – | – | 69,026 / – | – |
| P-256 base | **11,928** / – | 18,820 / – | 43,147 / – | 28,416 / – | 33,630 / – | – | 29,183 / – | 164,476 / – |
| P-256 scalar | **24,154** / – | 38,789 / – | 51,557 / – | 81,949 / – | 40,472 / – | – | – | – |
| Tom-256 base | **21,277** / – | – | 48,736 / – | – | 44,125 / – | – | 51,401 / – | – |
| BLS12-381 base | **66,001** / – | 89,738 / – | 133,746 / – | – | 139,998 / – | – | 178,541 / – | – |
| P-521 base | **68,234** / – | – | 265,813 / – | – | – | – | 257,722 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **8.9** | – | 34.6 / 36.3 | – | – | 5.5\* / 6.9\* | – | – |
| BabyBear | **8.9** / **10.9** | – | 34.4 / 36.4 | – | – | 11.9 / 14.5 | – | – |
| Goldilocks | **7.1** / **9.9** | – | 38.1 / 38.0 | – | – | 3.2\* / 9.9\* | 7.2 / 10.9 | – |
| BN254 scalar | **20.8** / **25.9** | 236 / 235 | 269 / 269 | – | 236 / 234 | – | – | 253 / 248 |
| secp256k1 base | **9.7** / **14.9** | 231 / 231 | 272 / 270 | 81.7 / 90.0 | 205 / 210 | – | – | 252 / 251 |
| P-256 base | **18.9** / **24.9** | 204 / 207 | 265 / 271 | 148 / 129 | 183 / 180 | – | – | 133 / 137 |
| P-256 scalar | **22.3** / **27.9** | 261 / 259 | 261 / 263 | 139 / 146 | 228 / 234 | – | – | 253 / 249 |
| Tom-256 base | **20.5** / **27.9** | – | 274 / 267 | – | 226 / 224 | – | – | 253 / 250 |
| BLS12-381 base | **28.9** / **34.5** | 421 / 430 | 543 / 543 | – | 424 / 431 | – | – | 578 / 596 |
| P-521 base | **19.6** / **32.8** | – | 908 / 916 | – | – | – | – | 963 / 967 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **8.9** | 34.6 / 36.5 | – | – | 5.5\* / 6.9\* | – |
| BabyBear | **8.9** / **10.9** | 34.5 / 36.4 | – | – | 11.9 / 14.1 | – |
| Goldilocks | **7.1** / **9.9** | 38.0 / 38.4 | – | – | 3.2\* / 9.9\* | 7.3 / 10.9 |
| BN254 scalar | **103** / **114** | 279 / 279 | – | 186 / 185 | – | – |
| secp256k1 base | **56.9** / **68.1** | 272 / 272 | – | 158 / 155 | – | – |
| P-256 base | **102** / **111** | 274 / 274 | – | 128 / 134 | – | – |
| P-256 scalar | **110** / **116** | 281 / 280 | 182 / 186 | 181 / 188 | – | – |
| Tom-256 base | **108** / **114** | 279 / 269 | – | 170 / 173 | – | – |
| BLS12-381 base | **214** / **222** | 545 / 551 | – | 371 / 376 | – | – |
| P-521 base | **350** / **405** | 912 / 921 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | – | 32.5 / 26.5 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **5.9** / **7.0** | – | 31.7 / 29.6 | – | – | 9.0 / 7.9 | – | – |
| Goldilocks | **5.0** / **4.7** | – | 38.7 / 35.1 | – | – | 1.9\* / 2.9\* | 6.1 / 5.9 | – |
| BN254 scalar | **24.7** / **24.9** | 125 / 127 | 231 / 228 | – | 143 / 150 | – | – | 114 / 113 |
| secp256k1 base | **5.8** / **7.4** | 106 / 103 | 231 / 226 | 2.8\* / 2.1\* | 131 / 137 | – | – | 114 / 113 |
| P-256 base | **25.8** / **22.1** | 106 / 103 | 231 / 220 | 76.3 / 84.3 | 118 / 128 | – | – | 59.1 / 64.0 |
| P-256 scalar | 25.3 / 22.8 | 133 / 129 | 231 / 224 | **14.3** / **20.7** | 152 / 163 | – | – | 114 / 113 |
| Tom-256 base | **25.9** / **25.8** | – | 231 / 228 | – | 141 / 148 | – | – | 114 / 113 |
| BLS12-381 base | **38.4** / **38.7** | 223 / 222 | 374 / 379 | – | 263 / 261 | – | – | 330 / 313 |
| P-521 base | **10.8** / **12.7** | – | 595 / 591 | – | – | – | – | 525 / 539 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 32.7 / 26.6 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **5.9** / **7.0** | 32.6 / 29.5 | – | – | 9.0 / 8.0 | – |
| Goldilocks | **5.0** / **4.7** | 39.0 / 35.2 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 |
| BN254 scalar | **87.6** / **83.9** | 234 / 230 | – | 101 / 108 | – | – |
| secp256k1 base | **6.8** / **8.2** | 233 / 231 | – | 84.5 / 94.8 | – | – |
| P-256 base | **62.0** / **62.4** | 229 / 232 | – | 75.9 / 81.5 | – | – |
| P-256 scalar | 88.6 / 89.8 | 234 / 232 | **48.7** / **45.5** | 108 / 116 | – | – |
| Tom-256 base | **87.3** / **83.5** | 234 / 231 | – | 92.7 / 107 | – | – |
| BLS12-381 base | **170** / **166** | 376 / 381 | – | 206 / 213 | – | – |
| P-521 base | **23.4** / **23.9** | 597 / 603 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | – | 16.5 / 18.0 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **5.9** / **6.9** | – | 17.6 / 18.4 | – | – | 8.9 / 8.9 | – | – |
| Goldilocks | **4.7** / **4.4** | – | 26.9 / 22.3 | – | – | 1.9\* / 2.9\* | 4.9 / 5.5 | – |
| BN254 scalar | **24.6** / **24.9** | 124 / 120 | 180 / 185 | – | 145 / 145 | – | – | 112 / 110 |
| secp256k1 base | **4.2** / **6.9** | 101 / 102 | 190 / 180 | 2.9\* / 2.9\* | 125 / 125 | – | – | 112 / 110 |
| P-256 base | **20.2** / **21.6** | 103 / 97.0 | 180 / 181 | 79.2 / 77.6 | 121 / 119 | – | – | 63.0 / 61.6 |
| P-256 scalar | 24.9 / 23.6 | 130 / 127 | 187 / 182 | **6.5** / **12.4** | 149 / 149 | – | – | 112 / 110 |
| Tom-256 base | **24.9** / **25.1** | – | 183 / 183 | – | 142 / 146 | – | – | 113 / 110 |
| BLS12-381 base | **37.8** / **38.8** | 222 / 225 | 363 / 356 | – | 255 / 255 | – | – | 335 / 310 |
| P-521 base | **9.7** / **11.0** | – | 586 / 603 | – | – | – | – | 526 / 539 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.0 | – | – | 4.1\* / 4.2\* | – |
| BabyBear | **5.9** / **6.9** | 17.6 / 18.4 | – | – | 8.9 / 8.9 | – |
| Goldilocks | **4.7** / **4.4** | 26.8 / 22.3 | – | – | 1.9\* / 2.9\* | 4.9 / 4.6 |
| BN254 scalar | **87.8** / **84.0** | 182 / 183 | – | 109 / 105 | – | – |
| secp256k1 base | **5.4** / **6.8** | 176 / 180 | – | 80.8 / 82.8 | – | – |
| P-256 base | **62.0** / **62.0** | 178 / 177 | – | 78.8 / 78.0 | – | – |
| P-256 scalar | 89.1 / 86.9 | 188 / 178 | **43.6** / **36.7** | 106 / 101 | – | – |
| Tom-256 base | **83.9** / **81.7** | 187 / 184 | – | 103 / 96.0 | – | – |
| BLS12-381 base | **168** / **169** | 363 / 367 | – | 208 / 203 | – | – |
| P-521 base | **20.2** / **16.0** | 586 / 608 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 12.3 / – | – | – | **3.2** / – | – | – |
| BabyBear | **6.3** / – | – | 13.1 / – | – | – | 8.0 / – | – | – |
| Goldilocks | 4.1 / – | – | 17.5 / – | – | – | 1.0\* / – | **3.4** / – | – |
| BN254 scalar | **22.0** / – | 129 / – | 119 / – | – | 126 / – | – | – | 95.5 / – |
| secp256k1 base | **1.9** / – | 121 / – | 119 / – | 2.2 / – | 109 / – | – | – | 95.7 / – |
| P-256 base | **17.8** / – | 112 / – | 116 / – | 59.0 / – | 107 / – | – | – | 55.3 / – |
| P-256 scalar | 18.0 / – | 137 / – | 118 / – | **2.2** / – | 134 / – | – | – | 95.5 / – |
| Tom-256 base | **21.0** / – | – | 117 / – | – | 128 / – | – | – | 95.6 / – |
| BLS12-381 base | **29.1** / – | 229 / – | 260 / – | – | 227 / – | – | – | 326 / – |
| P-521 base | **5.0** / – | – | 451 / – | – | – | – | – | 491 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **618** / **620** | 4,175 / 4,149 | 16,540 / 16,534 | 13,409 / 13,344 | – | 6,249 / 6,209 | 10,597 / 10,617 |
| secp256k1 base | **573** / **573** | 4,059 / 4,039 | 13,250 / 13,125 | 13,341 / 13,275 | 23,758 / 23,950 | 6,155 / 6,154 | 10,608 / 10,634 |
| P-256 base | **617** / **619** | 4,150 / 4,142 | 15,079 / 14,827 | 13,422 / 13,403 | 27,893 / 27,889 | 6,129 / 6,112 | 10,610 / 10,463 |
| P-256 scalar | **630** / **629** | 4,077 / 4,076 | 15,870 / 15,866 | 13,351 / 13,269 | 22,008 / 21,984 | 6,210 / 6,181 | 10,584 / 10,668 |
| Tom-256 base | **623** / **626** | 4,196 / 4,192 | – | 13,368 / 13,338 | – | 6,164 / 6,174 | 10,608 / 10,634 |
| BLS12-381 base | **767** / **770** | 7,091 / 7,091 | 34,795 / 34,689 | 23,118 / 22,975 | – | 9,832 / 9,825 | 26,000 / 25,875 |
| P-521 base | **951** / **945** | 11,966 / 12,010 | – | 40,128 / 39,838 | – | – | 54,889 / 54,896 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 217 / 215 | **214** / **213** | – | 1,463 / 1,462 | – | – | 640 / 639 | – | 895 / 892 |
| BabyBear | **216** / **214** | 233 / 230 | – | 1,397 / 1,414 | – | – | 622 / 623 | – | 1,069 / 1,065 |
| Goldilocks | **438** / **431** | 455 / 455 | – | 2,921 / 2,921 | – | – | 1,286 / 1,284 | 704 / 709 | 1,784 / 1,740 |
| BN254 scalar | **5,006** / **5,007** | 5,038 / 5,021 | 16,659 / 16,707 | 14,552 / 14,400 | – | 7,379 / 7,388 | – | – | 14,271 / 14,296 |
| secp256k1 base | **4,889** / **4,870** | 4,918 / 4,873 | 15,906 / 15,654 | 14,634 / 14,686 | 25,392 / 25,450 | 7,416 / 7,400 | – | – | 14,375 / 14,446 |
| P-256 base | 5,087 / **5,026** | **5,081** / 5,061 | 15,534 / 15,536 | 14,672 / 14,631 | 29,988 / 28,686 | 7,351 / 7,353 | – | – | 14,699 / 14,254 |
| P-256 scalar | 4,966 / **4,953** | **4,957** / 4,978 | 16,799 / 16,740 | 14,709 / 14,659 | 30,069 / 29,736 | 7,410 / 7,390 | – | – | 14,400 / 14,412 |
| Tom-256 base | **5,051** / **5,055** | 5,075 / 5,073 | – | 14,668 / 14,623 | – | 7,402 / 7,379 | – | – | 14,500 / 14,490 |
| BLS12-381 base | **9,038** / **8,990** | 9,125 / 9,090 | 37,006 / 36,912 | 25,842 / 25,811 | – | 12,349 / 12,339 | – | – | 35,527 / 34,666 |
| P-521 base | 15,005 / 14,974 | **14,971** / **14,957** | – | 45,589 / 44,468 | – | – | – | – | 72,447 / 71,828 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 9.7 / 16.9 | – | – | – | 2.5 / 5.0 |
| BabyBear | **1.9** / **2.9** | 9.7 / 17.2 | – | – | – | 2.5 / 5.0 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 19.0 | – | – | 2.4 / **2.9** | 2.5 / 4.9 |
| BN254 scalar | **4.9** / **6.9** | 66.6 / 94.0 | – | 12.8 / 21.8 | – | 7.0 / 10.1 |
| secp256k1 base | **6.0** / **6.9** | 66.6 / 85.9 | 2.8\* / 2.9\* | 14.1 / 21.1 | – | 6.8 / 10.0 |
| P-256 base | **5.8** / **6.9** | 66.6 / 76.9 | 10.8 / 18.6 | 10.7 / 19.5 | – | 7.0 / 10.4 |
| P-256 scalar | **5.8** / **6.7** | 66.6 / 94.5 | 17.6 / 21.0 | 14.5 / 21.0 | – | 7.0 / 10.2 |
| Tom-256 base | **5.9** / **6.9** | 66.5 / 80.6 | – | 11.8 / 20.5 | – | 7.0 / 10.4 |
| BLS12-381 base | **9.7** / **8.7** | 87.5 / 112 | – | 20.5 / 27.3 | – | 16.9 / 12.8 |
| P-521 base | **12.3** / **15.1** | 116 / 155 | – | – | – | 24.9 / 52.4 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.2 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 |
| BabyBear | **1.9** / **2.9** | 4.2 / 11.9 | – | – | – | – | 2.0 / 4.0 |
| Goldilocks | **1.9** / **2.9** | 4.2 / 11.7 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.0** / **6.8** | 49.6 / 50.9 | – | 8.5 / 12.0 | – | – | 6.9 / 8.0 |
| secp256k1 base | **5.3** / **7.8** | 49.6 / 48.0 | 3.6\* / 3.7\* | 7.7 / 13.0 | – | – | 7.0 / 8.0 |
| P-256 base | **5.5** / **7.0** | 49.7 / 43.5 | 8.0 / 13.0 | 7.6 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.0** / **6.8** | 49.7 / 47.7 | 7.8 / 13.0 | 8.1 / 12.9 | – | – | 7.0 / 8.0 |
| Tom-256 base | **5.5** / 8.1 | 49.7 / 54.9 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **10.0** / **9.9** | 82.7 / 88.0 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 |
| P-521 base | **11.8** / **12.5** | 121 / 138 | – | – | – | – | 27.3 / 50.8 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **6.9** | 16.9 / 34.4 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.3** / **11.0** | 16.9 / 34.9 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **8.9** | 17.6 / 34.7 | – | – | **3.7** / 11.0 | 7.0 / 17.0 |
| BN254 scalar | **67.9** / **78.9** | 119 / 151 | – | 91.4 / 93.8 | – | 94.1 / 129 |
| secp256k1 base | **44.6** / **55.0** | 118 / 152 | 84.8 / 88.4 | 88.1 / 96.0 | – | 94.1 / 129 |
| P-256 base | **49.5** / 60.0 | 117 / 152 | 68.1 / 72.2 | 67.9 / 73.0 | – | 50.7 / **56.7** |
| P-256 scalar | **71.3** / **83.8** | 117 / 152 | 140 / 141 | 94.0 / 100.0 | – | 93.7 / 128 |
| Tom-256 base | **71.0** / **76.0** | 117 / 152 | – | 89.8 / 94.8 | – | 93.9 / 128 |
| BLS12-381 base | **141** / **154** | 270 / 290 | – | 188 / 192 | – | 277 / 262 |
| P-521 base | **176** / **191** | 529 / 545 | – | – | – | 462 / 477 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 33.9 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.7 / 31.9 | – | – | – | 7.5 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 36.2 | – | – | **3.7** / 11.0 | 7.6 / 17.0 |
| BN254 scalar | **64.9** / **66.9** | 114 / 154 | – | 74.2 / 89.4 | – | 93.3 / 127 |
| secp256k1 base | **33.8** / **42.9** | 113 / 154 | 57.9 / 79.9 | 65.1 / 85.0 | – | 93.3 / 128 |
| P-256 base | **39.5** / **43.9** | 115 / 153 | 60.4 / 72.0 | 54.7 / 58.0 | – | 50.4 / 57.9 |
| P-256 scalar | **63.9** / **72.9** | 114 / 154 | 129 / 147 | 77.1 / 93.5 | – | 93.3 / 129 |
| Tom-256 base | **57.0** / **64.4** | 114 / 154 | – | 69.5 / 81.0 | – | 93.4 / 125 |
| BLS12-381 base | **135** / **145** | 268 / 289 | – | 152 / 166 | – | 276 / 277 |
| P-521 base | **118** / **133** | 536 / 553 | – | – | – | 463 / 469 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **252** / – | 3,853 / – | – | – | – | 953 / – |
| BabyBear | **383** / – | 3,851 / – | – | – | – | 944 / – |
| Goldilocks | **648** / – | 3,853 / – | – | – | 691 / – | 2,067 / – |
| BN254 scalar | **6,754** / – | 19,272 / – | – | 46,597 / – | – | 55,445 / – |
| secp256k1 base | **6,858** / – | 19,257 / – | 24,677 / – | 46,574 / – | – | 70,084 / – |
| P-256 base | **6,502** / – | 19,257 / – | 28,712 / – | 33,421 / – | – | 35,895 / – |
| P-256 scalar | **6,905** / – | 19,317 / – | 78,893 / – | 48,920 / – | – | 60,718 / – |
| Tom-256 base | **6,981** / – | 19,278 / – | – | 44,801 / – | – | 52,291 / – |
| BLS12-381 base | **12,519** / – | 34,254 / – | – | 141,473 / – | – | 180,393 / – |
| P-521 base | **19,965** / – | 64,404 / – | – | – | – | 496,994 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,652 / – | – | – | 446 / – |
| BabyBear | **5,594** / – | 18,660 / – | – | – | – |
| Goldilocks | **7,151** / – | 26,624 / – | – | – | – |
| BN254 scalar | **58,578** / – | 132,723 / – | – | 84,541 / – | – |
| secp256k1 base | **12,269** / – | 52,662 / – | 24,055 / – | 47,190 / – | 69,026 / – |
| P-256 base | **12,001** / – | 43,147 / – | 28,416 / – | 33,630 / – | 29,183 / – |
| P-256 scalar | **25,656** / – | 51,557 / – | 81,949 / – | 40,472 / – | – |
| Tom-256 base | **20,084** / – | 48,736 / – | – | 44,125 / – | 51,401 / – |
| BLS12-381 base | **69,801** / – | 133,746 / – | – | 139,998 / – | 178,541 / – |
| P-521 base | **68,418** / – | 265,813 / – | – | – | 257,722 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **9.0** | 34.6 / 36.3 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.4 / 36.4 | – | – | – |
| Goldilocks | **7.1** / **10.9** | 38.1 / 38.0 | – | – | 7.2 / **10.9** |
| BN254 scalar | **21.8** / **30.3** | 269 / 269 | – | 236 / 234 | – |
| secp256k1 base | **11.0** / **19.9** | 272 / 270 | 81.7 / 90.0 | 205 / 210 | – |
| P-256 base | **24.4** / **31.6** | 265 / 271 | 148 / 129 | 183 / 180 | – |
| P-256 scalar | **22.8** / **34.6** | 261 / 263 | 139 / 146 | 228 / 234 | – |
| Tom-256 base | **22.3** / **32.9** | 274 / 267 | – | 226 / 224 | – |
| BLS12-381 base | **34.6** / **39.9** | 543 / 543 | – | 424 / 431 | – |
| P-521 base | **26.4** / **44.4** | 908 / 916 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **8.9** | 34.6 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.5 / 36.4 | – | – | – |
| Goldilocks | **7.1** / **9.9** | 38.0 / 38.4 | – | – | 7.3 / 10.9 |
| BN254 scalar | **119** / **141** | 279 / 279 | – | 186 / 185 | – |
| secp256k1 base | **78.3** / **83.8** | 272 / 272 | – | 158 / 155 | – |
| P-256 base | **119** / **134** | 274 / 274 | – | 128 / **134** | – |
| P-256 scalar | **132** / **146** | 281 / 280 | 182 / 186 | 181 / 188 | – |
| Tom-256 base | **122** / **135** | 279 / 269 | – | 170 / 173 | – |
| BLS12-381 base | **253** / **266** | 545 / 551 | – | 371 / 376 | – |
| P-521 base | **426** / **473** | 912 / 921 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 32.5 / 26.5 | – | – | – |
| BabyBear | **5.9** / **7.1** | 31.7 / 29.6 | – | – | – |
| Goldilocks | **5.2** / **4.7** | 38.7 / 35.1 | – | – | 6.1 / 5.9 |
| BN254 scalar | **26.9** / **28.9** | 231 / 228 | – | 143 / 150 | – |
| secp256k1 base | **5.9** / **7.4** | 231 / 226 | 2.8\* / 2.1\* | 131 / 137 | – |
| P-256 base | **25.7** / **23.9** | 231 / 220 | 76.3 / 84.3 | 118 / 128 | – |
| P-256 scalar | 29.2 / 27.6 | 231 / 224 | **14.3** / **20.7** | 152 / 163 | – |
| Tom-256 base | **27.2** / **27.1** | 231 / 228 | – | 141 / 148 | – |
| BLS12-381 base | **43.8** / **44.8** | 374 / 379 | – | 263 / 261 | – |
| P-521 base | **11.3** / **12.7** | 595 / 591 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 32.7 / 26.6 | – | – | – |
| BabyBear | **5.9** / **7.0** | 32.6 / 29.5 | – | – | – |
| Goldilocks | **5.0** / **4.7** | 39.0 / 35.2 | – | – | 6.0 / 5.9 |
| BN254 scalar | **92.8** / **91.8** | 234 / 230 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **8.7** | 233 / 231 | – | 84.5 / 94.8 | – |
| P-256 base | **70.3** / **69.0** | 229 / 232 | – | 75.9 / 81.5 | – |
| P-256 scalar | 95.8 / 89.8 | 234 / 232 | **48.7** / **45.5** | 108 / 116 | – |
| Tom-256 base | **84.4** / **86.7** | 234 / 231 | – | 92.7 / 107 | – |
| BLS12-381 base | **176** / **179** | 376 / 381 | – | 206 / 213 | – |
| P-521 base | **22.4** / **22.3** | 597 / 603 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.0 | – | – | – |
| BabyBear | **5.8** / **6.9** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 26.9 / 22.3 | – | – | 4.9 / 5.5 |
| BN254 scalar | **27.9** / **28.2** | 180 / 185 | – | 145 / 145 | – |
| secp256k1 base | **4.1** / **6.9** | 190 / 180 | 2.9\* / 2.9\* | 125 / 125 | – |
| P-256 base | **23.1** / **22.9** | 180 / 181 | 79.2 / 77.6 | 121 / 119 | – |
| P-256 scalar | 26.4 / 22.6 | 187 / 182 | **6.5** / **12.4** | 149 / 149 | – |
| Tom-256 base | **28.0** / **28.2** | 183 / 183 | – | 142 / 146 | – |
| BLS12-381 base | **42.5** / **44.3** | 363 / 356 | – | 255 / 255 | – |
| P-521 base | **9.5** / **11.1** | 586 / 603 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.0 | – | – | – |
| BabyBear | **5.8** / **6.9** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.6** / **4.4** | 26.8 / 22.3 | – | – | 4.9 / 4.6 |
| BN254 scalar | **91.8** / **93.1** | 182 / 183 | – | 109 / 105 | – |
| secp256k1 base | **7.3** / **7.8** | 176 / 180 | – | 80.8 / 82.8 | – |
| P-256 base | **64.8** / **68.0** | 178 / 177 | – | 78.8 / 78.0 | – |
| P-256 scalar | 90.7 / 93.0 | 188 / 178 | **43.6** / **36.7** | 106 / 101 | – |
| Tom-256 base | **91.5** / **86.3** | 187 / 184 | – | 103 / 96.0 | – |
| BLS12-381 base | **174** / **171** | 363 / 367 | – | 208 / 203 | – |
| P-521 base | **19.1** / **19.5** | 586 / 608 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **12.3** / – | – | – | – |
| BabyBear | **13.1** / – | – | – | – |
| Goldilocks | 17.5 / – | – | – | **3.4** / – |
| BN254 scalar | **119** / – | – | 126 / – | – |
| secp256k1 base | 119 / – | **2.2** / – | 109 / – | – |
| P-256 base | 116 / – | **59.0** / – | 107 / – | – |
| P-256 scalar | 118 / – | **2.2** / – | 134 / – | – |
| Tom-256 base | **117** / – | – | 128 / – | – |
| BLS12-381 base | 260 / – | – | **227** / – | – |
| P-521 base | **451** / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **6,841** / **6,816** | 6,902 / 6,891 | 19,560 / 19,555 | – | 46,314 / 46,079 | 56,021 / 54,769 |
| secp256k1 base | **6,782** / **6,786** | 6,825 / 6,825 | 19,701 / 19,556 | 23,758 / 23,950 | 46,772 / 46,795 | 70,527 / 69,021 |
| P-256 base | **6,709** / **6,686** | 6,749 / 6,706 | 19,607 / 19,599 | 27,893 / 27,889 | 33,183 / 33,199 | 34,611 / 34,807 |
| P-256 scalar | **6,877** / **6,894** | 6,934 / 6,938 | 19,523 / 19,558 | 78,913 / 76,915 | 49,120 / 49,110 | 61,038 / 60,068 |
| Tom-256 base | **7,283** / **7,291** | 7,335 / 7,333 | 19,579 / 19,501 | – | 44,433 / 43,813 | 52,729 / 51,876 |
| BLS12-381 base | **12,391** / **12,397** | 12,512 / 12,465 | 35,390 / 34,899 | – | 140,343 / 140,341 | 179,063 / 177,178 |
| P-521 base | **20,367** / **20,408** | 20,419 / 20,425 | 64,873 / 64,672 | – | – | 521,717 / 494,510 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **271** / **248** | 274 / 267 | 3,886 / 3,890 | – | – | – | 992 / 992 |
| BabyBear | **405** / **405** | 421 / 416 | 4,225 / 3,896 | – | – | – | 990 / 992 |
| Goldilocks | 664 / **663** | **652** / 671 | 3,890 / 3,890 | – | – | 704 / 709 | 2,114 / 2,114 |
| BN254 scalar | **6,890** / **6,895** | 6,963 / 6,954 | 19,547 / 19,592 | – | 46,033 / 46,012 | – | 55,955 / 54,805 |
| secp256k1 base | **6,893** / **6,738** | 6,915 / 6,824 | 19,594 / 19,628 | 25,392 / 25,450 | 46,713 / 46,741 | – | 70,827 / 69,155 |
| P-256 base | **6,773** / **6,712** | 6,777 / 6,795 | 19,616 / 19,654 | 29,988 / 28,686 | 33,181 / 33,169 | – | 35,099 / 34,303 |
| P-256 scalar | **6,940** / **6,935** | 7,004 / 6,994 | 19,683 / 19,619 | 78,253 / 78,855 | 49,165 / 49,061 | – | 61,041 / 60,672 |
| Tom-256 base | **7,321** / **7,331** | 7,389 / 7,393 | 19,605 / 19,554 | – | 44,255 / 44,021 | – | 52,682 / 51,845 |
| BLS12-381 base | **12,547** / **12,494** | 12,623 / 12,578 | 35,248 / 35,126 | – | 140,198 / 140,286 | – | 179,048 / 177,643 |
| P-521 base | **20,382** / 20,419 | 20,504 / **20,413** | 64,699 / 64,598 | – | – | – | 518,879 / 494,551 |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
