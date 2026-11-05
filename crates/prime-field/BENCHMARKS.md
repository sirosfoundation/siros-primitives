# Benchmarks

`prime-field` at commit `6dd3098` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 20.84 before, 22.37 after; its SMT sibling, CPU 29, 30% busy |
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
| Mersenne-31 | **1.9** / **2.9** | – | 9.6 / 16.2 | – | – | 2.0 / 3.0 | – | 2.5 / 5.0 | – |
| BabyBear | **1.9** / **3.0** | – | 9.6 / 17.9 | – | – | 2.0 / **3.0** | – | 2.6 / 5.0 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 19.0 | – | – | 1.9\* / 2.9\* | 2.4 / **2.9** | 2.6 / 5.0 | – |
| BN254 scalar | **5.1** / **6.8** | 17.0 / 12.1 | 66.5 / 79.9 | – | 12.7 / 22.0 | – | – | 7.0 / 11.0 | – |
| secp256k1 base | **5.8** / **6.9** | 17.3 / 13.1 | 66.5 / 81.3 | 4.3\* / 3.4\* | 14.3 / 21.1 | – | – | 7.0 / 11.0 | – |
| P-256 base | **5.8** / **6.9** | 18.5 / 13.0 | 66.5 / 85.0 | 10.6 / 18.7 | 10.8 / 19.0 | – | – | 7.1 / 11.0 | 335 / – |
| P-256 scalar | **6.0** / **7.0** | 17.3 / 13.7 | 66.5 / 82.1 | 17.6 / 21.0 | 14.5 / 21.0 | – | – | 7.0 / 9.8 | 336 / – |
| Tom-256 base | **6.0** / **6.9** | – | 66.6 / 81.9 | – | 11.5 / 20.5 | – | – | 7.1 / 10.5 | – |
| BLS12-381 base | **9.9** / **8.9** | 19.2 / 13.6 | 87.3 / 112 | – | 20.4 / 27.3 | – | – | 17.0 / 12.9 | – |
| P-521 base | **12.5** / **15.1** | – | 117 / 154 | – | – | – | – | 24.8 / 52.3 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| BabyBear | **1.9** / **3.0** | – | 4.2 / 11.9 | – | – | 2.0 / **3.0** | – | 2.0 / 4.0 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.3 / 11.7 | – | – | 1.9 / 2.9 | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.1** / **6.8** | 16.1 / 12.6 | 49.6 / 50.8 | – | 8.3 / 11.8 | – | – | 7.0 / 8.0 | – |
| secp256k1 base | **5.4** / **8.0** | 16.9 / 13.1 | 49.5 / 48.0 | 4.8\* / 3.8\* | 7.5 / 12.8 | – | – | 7.0 / **8.0** | – |
| P-256 base | **5.3** / **7.0** | 16.1 / 11.8 | 49.9 / 43.6 | 7.8 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 | 151 / – |
| P-256 scalar | **5.1** / **7.0** | 15.5 / 12.8 | 49.8 / 47.4 | 7.9 / 13.0 | 8.0 / 13.0 | – | – | 7.0 / 8.0 | 151 / – |
| Tom-256 base | **5.5** / 8.1 | – | 49.9 / 52.6 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **10.0** / **9.9** | 21.3 / 15.8 | 82.9 / 87.7 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 | – |
| P-521 base | **12.2** / **12.5** | – | 121 / 138 | – | – | – | – | 27.4 / 51.0 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.9 / 30.9 | – | – | 2.8 / 7.0\* | – | 6.9 / 17.0 | – |
| BabyBear | **3.3** / **10.9** | – | 16.9 / 38.3 | – | – | 4.6 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.6 / 34.7 | – | – | 3.2\* / 9.9 | **3.7** / 10.9 | 7.0 / 17.0 | – |
| BN254 scalar | **68.6** / **72.4** | 75.2 / 104 | 118 / 152 | – | 92.3 / 93.8 | – | – | 93.9 / 127 | – |
| secp256k1 base | **44.6** / **54.8** | 90.1 / 114 | 118 / 153 | 84.7 / 88.5 | 88.0 / 96.0 | – | – | 93.9 / 128 | – |
| P-256 base | **49.4** / **60.0** | 67.4 / 86.4 | 118 / 153 | 67.8 / 71.9 | 67.9 / 73.0 | – | – | 50.7 / 62.7 | 563 / – |
| P-256 scalar | **75.0** / **84.0** | 90.4 / 113 | 118 / 151 | 140 / 141 | 94.0 / 100.0 | – | – | 94.1 / 128 | 628 / – |
| Tom-256 base | **71.0** / **76.0** | – | 118 / 153 | – | 89.9 / 94.8 | – | – | 93.9 / 126 | – |
| BLS12-381 base | **139** / **141** | 164 / 195 | 270 / 287 | – | 189 / 193 | – | – | 251 / 270 | – |
| P-521 base | **175** / **189** | – | 529 / 545 | – | – | – | – | 463 / 475 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 34.3 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.7 / 33.3 | – | – | 4.6 / 13.0 | – | 7.6 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.5 / 36.2 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.6 / 17.0 | – |
| BN254 scalar | **63.2** / **66.5** | 78.4 / 105 | 113 / 154 | – | 74.2 / 89.4 | – | – | 93.4 / 126 | – |
| secp256k1 base | **33.8** / **42.9** | 87.1 / 101 | 113 / 154 | 58.0 / 80.2 | 70.4 / 84.9 | – | – | 93.5 / 128 | – |
| P-256 base | **40.0** / **44.0** | 65.5 / 82.8 | 117 / 153 | 60.6 / 72.0 | 54.7 / 58.0 | – | – | 50.5 / 58.0 | 562 / – |
| P-256 scalar | **64.1** / **73.0** | 91.1 / 109 | 114 / 154 | 129 / 147 | 77.4 / 93.5 | – | – | 93.4 / 126 | 632 / – |
| Tom-256 base | **57.0** / **64.5** | – | 112 / 154 | – | 69.4 / 82.9 | – | – | 93.4 / 128 | – |
| BLS12-381 base | **132** / **137** | 140 / 167 | 268 / 290 | – | 151 / 166 | – | – | 251 / 262 | – |
| P-521 base | **121** / **133** | – | 537 / 559 | – | – | – | – | 462 / 470 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **216** / – | – | 1,415 / – | – | – | 633 / – | – | 878 / – | – |
| BabyBear | **223** / – | – | 1,403 / – | – | – | 622 / – | – | 1,054 / – | – |
| Goldilocks | **435** / – | – | 2,857 / – | – | – | 1,301 / – | 691 / – | 1,714 / – | – |
| BN254 scalar | **5,042** / – | 16,380 / – | 14,281 / – | – | 7,136 / – | – | – | 13,994 / – | – |
| secp256k1 base | **4,917** / – | 15,436 / – | 14,344 / – | 26,453 / – | 7,179 / – | – | – | 14,108 / – | – |
| P-256 base | **4,844** / – | 15,206 / – | 14,327 / – | 30,558 / – | 7,202 / – | – | – | 14,072 / – | 88,986 / – |
| P-256 scalar | **5,048** / – | 16,448 / – | 14,371 / – | 29,638 / – | 7,212 / – | – | – | 14,170 / – | 89,161 / – |
| Tom-256 base | **5,026** / – | – | 14,350 / – | – | 7,186 / – | – | – | 14,123 / – | – |
| BLS12-381 base | **8,838** / – | 36,329 / – | 25,124 / – | – | 12,075 / – | – | – | 34,092 / – | – |
| P-521 base | **14,234** / – | – | 43,690 / – | – | – | – | – | 71,837 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **135** / – | – | 1,648 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,974** / – | – | 18,597 / – | – | – | 2,712 / – | – | – |
| Goldilocks | **2,824** / – | – | 26,987 / – | – | – | 3,120 / – | – | – |
| BN254 scalar | **33,878** / – | 43,602 / – | 133,361 / – | – | 84,029 / – | – | – | – |
| secp256k1 base | **12,290** / – | 43,212 / – | 52,326 / – | 24,377 / – | 47,452 / – | – | 69,041 / – | – |
| P-256 base | **11,937** / – | 18,684 / – | 43,205 / – | 30,278 / – | 33,627 / – | – | 29,687 / – | 163,982 / – |
| P-256 scalar | **24,065** / – | 38,931 / – | 51,702 / – | 81,765 / – | 42,811 / – | – | – | – |
| Tom-256 base | **21,265** / – | – | 48,666 / – | – | 44,135 / – | – | 51,656 / – | – |
| BLS12-381 base | **65,900** / – | 90,156 / – | 133,907 / – | – | 139,126 / – | – | 178,331 / – | – |
| P-521 base | **68,261** / – | – | 264,554 / – | – | – | – | 256,618 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | – | 34.5 / 36.5 | – | – | 5.3\* / 6.9\* | – | – |
| BabyBear | **8.9** / **11.0** | – | 34.3 / 36.4 | – | – | 11.9 / 14.3 | – | – |
| Goldilocks | **7.0** / **9.1** | – | 38.0 / 38.7 | – | – | 3.2\* / 9.9 | 7.2 / 10.9 | – |
| BN254 scalar | **20.8** / **25.9** | 237 / 240 | 273 / 271 | – | 236 / 233 | – | – | 252 / 249 |
| secp256k1 base | **9.7** / **14.9** | 234 / 243 | 276 / 270 | 82.8 / 90.3 | 210 / 211 | – | – | 253 / 249 |
| P-256 base | **18.9** / **24.9** | 207 / 220 | 267 / 273 | 147 / 135 | 184 / 182 | – | – | 150 / 136 |
| P-256 scalar | **22.4** / **27.8** | 260 / 267 | 262 / 270 | 139 / 146 | 242 / 239 | – | – | 252 / 249 |
| Tom-256 base | **20.5** / **27.9** | – | 277 / 265 | – | 221 / 235 | – | – | 253 / 248 |
| BLS12-381 base | **28.8** / **34.5** | 439 / 427 | 547 / 541 | – | 431 / 430 | – | – | 586 / 598 |
| P-521 base | **19.7** / **32.8** | – | 896 / 900 | – | – | – | – | 996 / 959 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **9.0** | 34.5 / 36.5 | – | – | 5.3\* / 6.9\* | – |
| BabyBear | **8.9** / **10.9** | 34.4 / 36.5 | – | – | 12.0 / 15.0 | – |
| Goldilocks | **6.9** / **8.8** | 38.0 / 38.5 | – | – | 3.2\* / 9.9 | 7.3 / 10.9 |
| BN254 scalar | **103** / **114** | 277 / 274 | – | 186 / 186 | – | – |
| secp256k1 base | **56.6** / **68.3** | 276 / 276 | – | 158 / 155 | – | – |
| P-256 base | **102** / **110** | 280 / 277 | – | 128 / 134 | – | – |
| P-256 scalar | **110** / **116** | 282 / 269 | 180 / 187 | 191 / 189 | – | – |
| Tom-256 base | **108** / **114** | 282 / 279 | – | 170 / 173 | – | – |
| BLS12-381 base | **214** / **222** | 547 / 548 | – | 371 / 378 | – | – |
| P-521 base | **352** / **407** | 910 / 919 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | – | 31.7 / 26.6 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **6.0** / **7.4** | – | 32.1 / 30.4 | – | – | 8.9 / 7.9 | – | – |
| Goldilocks | **4.7** / **4.4** | – | 39.3 / 35.0 | – | – | 1.9\* / 2.9\* | 6.1 / 5.9 | – |
| BN254 scalar | **24.8** / **24.9** | 123 / 135 | 230 / 226 | – | 143 / 150 | – | – | 116 / 112 |
| secp256k1 base | **5.9** / **7.4** | 107 / 104 | 232 / 228 | 3.5\* / 2.2\* | 129 / 137 | – | – | 116 / 112 |
| P-256 base | **25.8** / **22.1** | 103 / 110 | 230 / 230 | 75.9 / 84.7 | 116 / 127 | – | – | 59.1 / 63.6 |
| P-256 scalar | 25.3 / 22.7 | 129 / 136 | 230 / 219 | **14.2** / **20.7** | 152 / 163 | – | – | 116 / 112 |
| Tom-256 base | **26.2** / **26.0** | – | 230 / 224 | – | 141 / 149 | – | – | 116 / 112 |
| BLS12-381 base | **38.3** / **38.7** | 225 / 218 | 374 / 377 | – | 262 / 259 | – | – | 329 / 326 |
| P-521 base | **10.8** / **12.8** | – | 595 / 602 | – | – | – | – | 524 / 541 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | 32.5 / 26.6 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **5.9** / **7.0** | 32.6 / 30.5 | – | – | 9.0 / 8.0 | – |
| Goldilocks | **4.7** / **4.4** | 38.9 / 35.1 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 |
| BN254 scalar | **87.8** / **83.8** | 228 / 228 | – | 101 / 108 | – | – |
| secp256k1 base | **6.8** / **8.5** | 230 / 231 | – | 83.8 / 93.8 | – | – |
| P-256 base | **61.2** / **62.2** | 230 / 230 | – | 75.7 / 81.5 | – | – |
| P-256 scalar | 88.6 / 89.8 | 230 / 231 | **45.3** / **43.9** | 108 / 116 | – | – |
| Tom-256 base | **87.1** / **83.3** | 230 / 229 | – | 92.8 / 106 | – | – |
| BLS12-381 base | **169** / **166** | 378 / 381 | – | 206 / 213 | – | – |
| P-521 base | **22.8** / **23.6** | 595 / 603 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | – | 16.5 / 17.9 | – | – | 3.9\* / 4.1\* | – | – |
| BabyBear | **6.0** / **6.4** | – | 17.6 / 18.4 | – | – | 8.9 / 8.5 | – | – |
| Goldilocks | **4.3** / **4.2** | – | 26.8 / 22.2 | – | – | 1.9\* / 2.9\* | 4.9 / 5.6 | – |
| BN254 scalar | **24.5** / **24.9** | 122 / 123 | 180 / 184 | – | 143 / 144 | – | – | 114 / 110 |
| secp256k1 base | **4.2** / **6.9** | 101 / 100.0 | 188 / 181 | 4.0\* / 3.1\* | 123 / 125 | – | – | 114 / 110 |
| P-256 base | **20.1** / **21.5** | 100 / 100 | 181 / 176 | 82.0 / 78.9 | 120 / 119 | – | – | 61.2 / 61.3 |
| P-256 scalar | 24.8 / 23.6 | 129 / 132 | 185 / 181 | **5.6** / **12.3** | 149 / 150 | – | – | 114 / 110 |
| Tom-256 base | **24.6** / **25.0** | – | 184 / 181 | – | 140 / 147 | – | – | 114 / 110 |
| BLS12-381 base | **38.1** / **38.7** | 224 / 216 | 362 / 360 | – | 254 / 253 | – | – | 319 / 318 |
| P-521 base | **9.6** / **11.1** | – | 586 / 601 | – | – | – | – | 526 / 544 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 18.0 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **6.0** / **7.0** | 17.6 / 18.4 | – | – | 9.0 / 9.0 | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 1.9\* / 2.9\* | 4.9 / 5.3 |
| BN254 scalar | **87.8** / **83.9** | 183 / 185 | – | 111 / 105 | – | – |
| secp256k1 base | **5.4** / **6.8** | 189 / 178 | – | 80.8 / 82.8 | – | – |
| P-256 base | **59.8** / **61.4** | 181 / 174 | – | 78.8 / 78.0 | – | – |
| P-256 scalar | 86.3 / 86.1 | 187 / 175 | **43.5** / **36.4** | 106 / 101 | – | – |
| Tom-256 base | **83.7** / **81.5** | 180 / 184 | – | 103 / 96.0 | – | – |
| BLS12-381 base | **168** / **169** | 364 / 366 | – | 208 / 203 | – | – |
| P-521 base | **20.2** / **15.6** | 586 / 604 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / – | – | 12.3 / – | – | – | **3.2** / – | – | – |
| BabyBear | **6.2** / – | – | 13.1 / – | – | – | 8.0 / – | – | – |
| Goldilocks | **3.3** / – | – | 17.5 / – | – | – | 1.0\* / – | **3.3** / – | – |
| BN254 scalar | **22.0** / – | 129 / – | 118 / – | – | 127 / – | – | – | 95.5 / – |
| secp256k1 base | 2.2 / – | 120 / – | 118 / – | **2.1** / – | 109 / – | – | – | 95.5 / – |
| P-256 base | **17.6** / – | 112 / – | 116 / – | 59.0 / – | 104 / – | – | – | 54.1 / – |
| P-256 scalar | 18.0 / – | 139 / – | 118 / – | **2.0** / – | 133 / – | – | – | 95.6 / – |
| Tom-256 base | **21.1** / – | – | 118 / – | – | 128 / – | – | – | 95.6 / – |
| BLS12-381 base | **29.1** / – | 226 / – | 256 / – | – | 226 / – | – | – | 332 / – |
| P-521 base | **5.0** / – | – | 436 / – | – | – | – | – | 490 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **615** / **605** | 4,157 / 4,134 | 16,488 / 16,462 | 13,593 / 13,331 | – | 6,202 / 6,220 | 10,701 / 10,617 |
| secp256k1 base | **563** / **567** | 4,042 / 4,056 | 13,304 / 13,072 | 13,336 / 13,172 | 24,787 / 24,833 | 6,238 / 6,161 | 10,624 / 10,613 |
| P-256 base | **601** / **603** | 4,179 / 4,139 | 14,872 / 14,869 | 13,460 / 13,276 | 30,243 / 29,709 | 6,135 / 6,121 | 11,381 / 10,260 |
| P-256 scalar | **619** / **619** | 4,082 / 4,071 | 15,880 / 15,748 | 13,304 / 13,289 | 21,965 / 21,933 | 6,215 / 6,175 | 10,631 / 10,543 |
| Tom-256 base | **613** / **612** | 4,198 / 4,172 | – | 13,342 / 13,260 | – | 6,168 / 6,172 | 10,631 / 10,618 |
| BLS12-381 base | **751** / **754** | 7,089 / 7,121 | 35,409 / 34,749 | 23,245 / 23,092 | – | 9,845 / 9,853 | 26,176 / 25,397 |
| P-521 base | **927** / **922** | 11,967 / 11,991 | – | 39,750 / 39,921 | – | – | 54,337 / 55,222 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 220 / 219 | **217** / **217** | – | 1,472 / 1,446 | – | – | 629 / 637 | – | 920 / 897 |
| BabyBear | **217** / **218** | 233 / 234 | – | 1,441 / 1,441 | – | – | 630 / 622 | – | 1,052 / 1,062 |
| Goldilocks | **440** / **438** | 446 / 446 | – | 2,906 / 2,892 | – | – | 1,285 / 1,284 | 704 / 706 | 1,709 / 1,711 |
| BN254 scalar | 5,030 / **4,997** | **5,019** / 5,019 | 17,416 / 16,694 | 14,629 / 14,558 | – | 7,341 / 7,397 | – | – | 14,544 / 14,395 |
| secp256k1 base | **4,880** / **4,867** | 4,933 / 4,899 | 15,790 / 15,738 | 14,630 / 14,567 | 25,382 / 25,408 | 7,424 / 7,370 | – | – | 14,492 / 14,465 |
| P-256 base | 5,136 / **5,079** | **5,077** / 5,082 | 15,469 / 15,523 | 15,050 / 14,518 | 28,949 / 29,060 | 7,372 / 7,332 | – | – | 15,233 / 14,638 |
| P-256 scalar | **4,966** / **4,934** | 4,993 / 4,979 | 16,654 / 16,695 | 14,602 / 14,629 | 30,228 / 29,667 | 7,433 / 7,381 | – | – | 14,463 / 14,566 |
| Tom-256 base | **5,047** / **5,043** | 5,087 / 5,117 | – | 14,619 / 14,683 | – | 7,457 / 7,390 | – | – | 14,792 / 14,252 |
| BLS12-381 base | **8,963** / **9,060** | 9,119 / 9,077 | 36,958 / 36,884 | 26,064 / 25,982 | – | 12,415 / 12,349 | – | – | 35,688 / 35,056 |
| P-521 base | **14,948** / **14,914** | 14,992 / 14,995 | – | 45,188 / 44,678 | – | – | – | – | 72,159 / 71,997 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **3.0** | 9.6 / 16.2 | – | – | – | 2.5 / 5.0 |
| BabyBear | **1.9** / **3.0** | 9.6 / 17.9 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **2.0** / 3.0 | 9.8 / 19.0 | – | – | 2.4 / **2.9** | 2.6 / 5.0 |
| BN254 scalar | **5.0** / **6.9** | 66.5 / 79.9 | – | 12.7 / 22.0 | – | 7.0 / 11.0 |
| secp256k1 base | **5.8** / **6.9** | 66.5 / 81.3 | 4.3\* / 3.4\* | 14.3 / 21.1 | – | 7.0 / 11.0 |
| P-256 base | **6.0** / **7.0** | 66.5 / 85.0 | 10.6 / 18.7 | 10.8 / 19.0 | – | 7.1 / 11.0 |
| P-256 scalar | **5.9** / **6.7** | 66.5 / 82.1 | 17.6 / 21.0 | 14.5 / 21.0 | – | 7.0 / 9.8 |
| Tom-256 base | **6.0** / **7.0** | 66.6 / 81.9 | – | 11.5 / 20.5 | – | 7.1 / 10.5 |
| BLS12-381 base | **9.9** / **8.9** | 87.3 / 112 | – | 20.4 / 27.3 | – | 17.0 / 12.9 |
| P-521 base | **12.7** / **14.9** | 117 / 154 | – | – | – | 24.8 / 52.3 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 4.3 / 12.0 | – | – | 2.0\* / 3.0 | – | **2.0** / 4.0 |
| BabyBear | **2.0** / **3.0** | 4.2 / 11.9 | – | – | – | – | **2.0** / 4.0 |
| Goldilocks | **1.9** / 3.0 | 4.3 / 11.7 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.1** / **7.0** | 49.6 / 50.8 | – | 8.3 / 11.8 | – | – | 7.0 / 8.0 |
| secp256k1 base | **5.3** / **8.0** | 49.5 / 48.0 | 4.8\* / 3.8\* | 7.5 / 12.8 | – | – | 7.0 / **8.0** |
| P-256 base | **5.3** / **7.0** | 49.9 / 43.6 | 7.8 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.0** / **6.8** | 49.8 / 47.4 | 7.9 / 13.0 | 8.0 / 13.0 | – | – | 7.0 / 8.0 |
| Tom-256 base | **5.5** / 8.2 | 49.9 / 52.6 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **9.9** / **9.9** | 82.9 / 87.7 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 |
| P-521 base | **12.1** / **12.6** | 121 / 138 | – | – | – | – | 27.4 / 51.0 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 30.9 | – | – | – | 6.9 / 17.0 |
| BabyBear | **3.4** / **11.0** | 16.9 / 38.3 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **8.9** | 17.6 / 34.7 | – | – | **3.7** / 10.9 | 7.0 / 17.0 |
| BN254 scalar | **67.8** / **79.0** | 118 / 152 | – | 92.3 / 93.8 | – | 93.9 / 127 |
| secp256k1 base | **44.6** / **55.0** | 118 / 153 | 84.7 / 88.5 | 88.0 / 96.0 | – | 93.9 / 128 |
| P-256 base | **49.2** / **60.0** | 118 / 153 | 67.8 / 71.9 | 67.9 / 73.0 | – | 50.7 / 62.7 |
| P-256 scalar | **74.8** / **84.0** | 118 / 151 | 140 / 141 | 94.0 / 100.0 | – | 94.1 / 128 |
| Tom-256 base | **71.0** / **76.0** | 118 / 153 | – | 89.9 / 94.8 | – | 93.9 / 126 |
| BLS12-381 base | **142** / **154** | 270 / 287 | – | 189 / 193 | – | 251 / 270 |
| P-521 base | **173** / **188** | 529 / 545 | – | – | – | 463 / 475 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 34.3 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.7** / **11.0** | 16.7 / 33.3 | – | – | – | 7.6 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 36.2 | – | – | **3.7** / 11.0 | 7.6 / 17.0 |
| BN254 scalar | **65.0** / **66.9** | 113 / 154 | – | 74.2 / 89.4 | – | 93.4 / 126 |
| secp256k1 base | **33.9** / **43.0** | 113 / 154 | 58.0 / 80.2 | 70.4 / 84.9 | – | 93.5 / 128 |
| P-256 base | **40.2** / **44.0** | 117 / 153 | 60.6 / 72.0 | 54.7 / 58.0 | – | 50.5 / 58.0 |
| P-256 scalar | **63.9** / **73.0** | 114 / 154 | 129 / 147 | 77.4 / 93.5 | – | 93.4 / 126 |
| Tom-256 base | **57.0** / **64.5** | 112 / 154 | – | 69.4 / 82.9 | – | 93.4 / 128 |
| BLS12-381 base | **137** / **145** | 268 / 290 | – | 151 / 166 | – | 251 / 262 |
| P-521 base | **123** / **133** | 537 / 559 | – | – | – | 462 / 470 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **248** / – | 3,846 / – | – | – | – | 953 / – |
| BabyBear | **383** / – | 3,847 / – | – | – | – | 944 / – |
| Goldilocks | **645** / – | 3,852 / – | – | – | 691 / – | 2,067 / – |
| BN254 scalar | **6,760** / – | 19,310 / – | – | 46,612 / – | – | 54,996 / – |
| secp256k1 base | **6,858** / – | 19,302 / – | 26,453 / – | 46,592 / – | – | 69,369 / – |
| P-256 base | **6,512** / – | 19,234 / – | 30,558 / – | 33,423 / – | – | 34,288 / – |
| P-256 scalar | **6,917** / – | 19,435 / – | 78,251 / – | 48,920 / – | – | 59,997 / – |
| Tom-256 base | **6,974** / – | 19,424 / – | – | 44,812 / – | – | 51,797 / – |
| BLS12-381 base | **12,573** / – | 34,197 / – | – | 141,664 / – | – | 177,288 / – |
| P-521 base | **20,025** / – | 64,189 / – | – | – | – | 511,427 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,648 / – | – | – | 446 / – |
| BabyBear | **5,592** / – | 18,597 / – | – | – | – |
| Goldilocks | **7,152** / – | 26,987 / – | – | – | – |
| BN254 scalar | **58,580** / – | 133,361 / – | – | 84,029 / – | – |
| secp256k1 base | **12,273** / – | 52,326 / – | 24,377 / – | 47,452 / – | 69,041 / – |
| P-256 base | **11,984** / – | 43,205 / – | 30,278 / – | 33,627 / – | 29,687 / – |
| P-256 scalar | **25,624** / – | 51,702 / – | 81,765 / – | 42,811 / – | – |
| Tom-256 base | **20,056** / – | 48,666 / – | – | 44,135 / – | 51,656 / – |
| BLS12-381 base | **69,780** / – | 133,907 / – | – | 139,126 / – | 178,331 / – |
| P-521 base | **68,425** / – | 264,554 / – | – | – | 256,618 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.5 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | – |
| Goldilocks | **6.9** / **8.9** | 38.0 / 38.7 | – | – | 7.2 / 10.9 |
| BN254 scalar | **21.9** / **30.3** | 273 / 271 | – | 236 / 233 | – |
| secp256k1 base | **11.0** / **19.9** | 276 / 270 | 82.8 / 90.3 | 210 / 211 | – |
| P-256 base | **24.4** / **31.9** | 267 / 273 | 147 / 135 | 184 / 182 | – |
| P-256 scalar | **22.9** / **33.5** | 262 / 270 | 139 / 146 | 242 / 239 | – |
| Tom-256 base | **22.4** / **32.9** | 277 / 265 | – | 221 / 235 | – |
| BLS12-381 base | **34.7** / **42.1** | 547 / 541 | – | 431 / 430 | – |
| P-521 base | **26.4** / **45.1** | 896 / 900 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **9.0** | 34.5 / 36.5 | – | – | – |
| BabyBear | **9.0** / **11.0** | 34.4 / 36.5 | – | – | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 38.5 | – | – | 7.3 / 10.9 |
| BN254 scalar | **120** / **141** | 277 / 274 | – | 186 / 186 | – |
| secp256k1 base | **78.2** / **83.8** | 276 / 276 | – | 158 / 155 | – |
| P-256 base | **119** / **134** | 280 / 277 | – | 128 / **134** | – |
| P-256 scalar | **132** / **146** | 282 / 269 | 180 / 187 | 191 / 189 | – |
| Tom-256 base | **122** / **135** | 282 / 279 | – | 170 / 173 | – |
| BLS12-381 base | **256** / **266** | 547 / 548 | – | 371 / 378 | – |
| P-521 base | **427** / **472** | 910 / 919 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 31.7 / 26.6 | – | – | – |
| BabyBear | **5.9** / **7.0** | 32.1 / 30.4 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 39.3 / 35.0 | – | – | 6.1 / 5.9 |
| BN254 scalar | **26.9** / **28.8** | 230 / 226 | – | 143 / 150 | – |
| secp256k1 base | **5.9** / **7.4** | 232 / 228 | 3.5\* / 2.2\* | 129 / 137 | – |
| P-256 base | **25.7** / **24.0** | 230 / 230 | 75.9 / 84.7 | 116 / 127 | – |
| P-256 scalar | 27.1 / 26.0 | 230 / 219 | **14.2** / **20.7** | 152 / 163 | – |
| Tom-256 base | **27.4** / **27.1** | 230 / 224 | – | 141 / 149 | – |
| BLS12-381 base | **43.8** / **44.9** | 374 / 377 | – | 262 / 259 | – |
| P-521 base | **10.8** / **12.7** | 595 / 602 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 32.5 / 26.6 | – | – | – |
| BabyBear | **6.0** / **7.1** | 32.6 / 30.5 | – | – | – |
| Goldilocks | **4.8** / **4.4** | 38.9 / 35.1 | – | – | 6.0 / 5.9 |
| BN254 scalar | **92.8** / **91.8** | 228 / 228 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **8.7** | 230 / 231 | – | 83.8 / 93.8 | – |
| P-256 base | **70.3** / **68.9** | 230 / 230 | – | 75.7 / 81.5 | – |
| P-256 scalar | 95.8 / 89.8 | 230 / 231 | **45.3** / **43.9** | 108 / 116 | – |
| Tom-256 base | **84.3** / **86.6** | 230 / 229 | – | 92.8 / 106 | – |
| BLS12-381 base | **176** / **179** | 378 / 381 | – | 206 / 213 | – |
| P-521 base | **22.4** / **22.7** | 595 / 603 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 17.9 | – | – | – |
| BabyBear | **6.0** / **7.0** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 4.9 / 5.6 |
| BN254 scalar | **27.9** / **28.3** | 180 / 184 | – | 143 / 144 | – |
| secp256k1 base | **4.2** / **6.9** | 188 / 181 | 4.0\* / 3.1\* | 123 / 125 | – |
| P-256 base | **23.3** / **22.9** | 181 / 176 | 82.0 / 78.9 | 120 / 119 | – |
| P-256 scalar | 26.5 / 22.2 | 185 / 181 | **5.6** / **12.3** | 149 / 150 | – |
| Tom-256 base | **28.1** / **28.1** | 184 / 181 | – | 140 / 147 | – |
| BLS12-381 base | **42.6** / **44.5** | 362 / 360 | – | 254 / 253 | – |
| P-521 base | **9.6** / **11.1** | 586 / 601 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.6** | 16.5 / 18.0 | – | – | – |
| BabyBear | **6.1** / **7.1** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 4.9 / 5.3 |
| BN254 scalar | **91.4** / **93.1** | 183 / 185 | – | 111 / 105 | – |
| secp256k1 base | **7.2** / **7.7** | 189 / 178 | – | 80.8 / 82.8 | – |
| P-256 base | **64.8** / **68.0** | 181 / 174 | – | 78.8 / 78.0 | – |
| P-256 scalar | 90.8 / 96.0 | 187 / 175 | **43.5** / **36.4** | 106 / 101 | – |
| Tom-256 base | **91.4** / **86.3** | 180 / 184 | – | 103 / 96.0 | – |
| BLS12-381 base | **175** / **172** | 364 / 366 | – | 208 / 203 | – |
| P-521 base | **19.1** / **19.6** | 586 / 604 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.3** / – | 12.3 / – | – | – | – |
| BabyBear | **10.3** / – | 13.1 / – | – | – | – |
| Goldilocks | 3.4 / – | 17.5 / – | – | – | **3.3** / – |
| BN254 scalar | **60.6** / – | 118 / – | – | 127 / – | – |
| secp256k1 base | 2.2 / – | 118 / – | **2.1** / – | 109 / – | – |
| P-256 base | **30.6** / – | 116 / – | 59.0 / – | 104 / – | – |
| P-256 scalar | 62.7 / – | 118 / – | **2.0** / – | 133 / – | – |
| Tom-256 base | **55.8** / – | 118 / – | – | 128 / – | – |
| BLS12-381 base | **142** / – | 256 / – | – | 226 / – | – |
| P-521 base | **5.0** / – | 436 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **6,930** / 6,950 | 6,967 / **6,917** | 19,595 / 19,659 | – | 46,047 / 46,054 | 56,432 / 54,805 |
| secp256k1 base | **6,782** / **6,790** | 6,886 / 6,838 | 19,505 / 19,595 | 24,787 / 24,833 | 46,778 / 46,788 | 72,027 / 68,607 |
| P-256 base | **6,695** / **6,707** | 6,751 / 6,708 | 19,614 / 19,584 | 30,243 / 29,709 | 33,222 / 33,208 | 34,555 / 34,562 |
| P-256 scalar | **6,905** / **6,903** | 6,952 / 6,933 | 19,632 / 19,639 | 78,847 / 78,877 | 49,155 / 49,121 | 61,756 / 59,954 |
| Tom-256 base | 7,409 / **7,295** | **7,332** / 7,339 | 19,547 / 19,589 | – | 44,300 / 43,513 | 52,926 / 51,448 |
| BLS12-381 base | **12,404** / **12,417** | 12,459 / 12,449 | 34,664 / 34,676 | – | 140,754 / 138,815 | 180,432 / 178,499 |
| P-521 base | **20,372** / **20,361** | 20,430 / 20,456 | 64,759 / 64,763 | – | – | 510,974 / 497,501 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **267** / **269** | 275 / 277 | 3,886 / 3,892 | – | – | – | 994 / 997 |
| BabyBear | **408** / **405** | 421 / 416 | 3,897 / 3,897 | – | – | – | 990 / 981 |
| Goldilocks | 707 / **664** | **666** / 672 | 3,894 / 3,906 | – | – | 704 / 706 | 2,102 / 2,102 |
| BN254 scalar | 7,010 / **6,863** | **6,976** / 6,962 | 19,618 / 19,364 | – | 46,031 / 46,026 | – | 55,880 / 55,141 |
| secp256k1 base | **6,807** / **6,807** | 6,879 / 6,828 | 19,640 / 19,614 | 25,382 / 25,408 | 46,713 / 46,723 | – | 70,736 / 68,703 |
| P-256 base | **6,719** / **6,719** | 6,760 / 6,850 | 19,625 / 19,594 | 28,949 / 29,060 | 33,156 / 33,157 | – | 34,852 / 34,144 |
| P-256 scalar | **6,954** / **6,965** | 7,011 / 7,010 | 19,606 / 19,258 | 78,774 / 78,512 | 49,464 / 49,101 | – | 61,707 / 59,987 |
| Tom-256 base | **7,337** / **7,280** | 7,407 / 7,408 | 19,639 / 19,586 | – | 44,302 / 43,483 | – | 52,848 / 51,620 |
| BLS12-381 base | **12,513** / 12,518 | 12,580 / **12,476** | 34,824 / 34,601 | – | 140,766 / 140,309 | – | 181,731 / 178,637 |
| P-521 base | 20,756 / **20,372** | **20,635** / 20,433 | 64,756 / 65,003 | – | – | – | 498,293 / 494,502 |

## WebAssembly

`prime-field` at commit `6825d17` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-05.

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
| Cost of a count, in WebAssembly, Chrome | 4,459 to 9,016 cycles and 3,081 instructions; its noise in a figure, 133 to 3,658 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,114 to 8,647 cycles and 3,081 instructions; its noise in a figure, 49.0 to 381 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 6,381 to 22,055 cycles and 1,885 instructions; its noise in a figure, 368 to 8,624 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 8,875 to 18,204 cycles and 1,885 instructions; its noise in a figure, 691 to 5,183 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 12.98 before, 21.35 after; its SMT sibling, CPU 28, 24% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 21.57 before, 23.17 after; its SMT sibling, CPU 28, 27% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 23.17 before, 28.61 after; its SMT sibling, CPU 28, 47% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 28.61 before, 19.93 after; its SMT sibling, CPU 28, 46% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.4 vs runtime-algebra 10.8 | −6.4 (0.41×) | 4.4 vs runtime-algebra 10.9 | −6.5 (0.40×) |
| add | BabyBear | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) |
| add | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) |
| add | BN254 scalar | 17.1 vs ark-ff 13.5 | +3.6 (1.27×) | 12.3 vs ark-ff 13.2 | −0.9 (0.93×) | 16.7 vs ark-ff 14.1 | +2.6 (1.18×) | 19.0 vs ark-ff 18.2 | +0.8 (1.04×) |
| add | secp256k1 base | 14.0 vs ark-ff 15.2 | −1.2 (0.92×) | 11.8 vs ark-ff 12.8 | −1.0 (0.92×) | 14.8 vs ark-ff 17.1 | −2.3 (0.87×) | 20.4 vs ark-ff 20.1 | +0.3 (1.01×) |
| add | P-256 base | 14.7 vs ark-ff 15.6 | −0.9 (0.94×) | 12.3 vs ark-ff 12.3 | +0.0 (1.00×) | 14.8 vs ark-ff 17.4 | −2.6 (0.85×) | 19.9 vs ark-ff 19.9 | +0.0 (1.00×) |
| add | P-256 scalar | 15.4 vs ark-ff 15.3 | +0.1 (1.01×) | 11.8 vs ark-ff 13.3 | −1.5 (0.89×) | 16.2 vs ark-ff 17.2 | −1.0 (0.94×) | 21.4 vs ark-ff 20.6 | +0.8 (1.04×) |
| add | Tom-256 base | 15.0 vs halo2curves 35.8 | −20.8 (0.42×) | 11.7 vs halo2curves 20.2 | −8.5 (0.58×) | 15.8 vs halo2curves 38.2 | −22.4 (0.41×) | 20.6 vs halo2curves 34.0 | −13.4 (0.61×) |
| add | BLS12-381 base | 26.1 vs ark-ff 35.5 | −9.4 (0.74×) | 17.8 vs ark-ff 30.0 | −12.2 (0.59×) | 25.0 vs ark-ff 39.2 | −14.2 (0.64×) | 29.5 vs ark-ff 42.3 | −12.8 (0.70×) |
| add | P-521 base | 43.2 vs crypto-bigint 146 | −103 (0.30×) | 26.1 vs crypto-bigint 133 | −107 (0.20×) | 42.5 vs crypto-bigint 176 | −134 (0.24×) | 43.9 vs crypto-bigint 184 | −140 (0.24×) |
| sub | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub | BabyBear | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) |
| sub | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 16.4 vs ark-ff 18.4 | −2.0 (0.89×) | 11.7 vs halo2curves 15.0 | −3.3 (0.78×) | 15.5 vs ark-ff 19.1 | −3.6 (0.81×) | 19.8 vs ark-ff 23.7 | −3.9 (0.84×) |
| sub | secp256k1 base | 16.6 vs ark-ff 18.4 | −1.8 (0.90×) | 11.8 vs halo2curves 14.2 | −2.4 (0.83×) | 15.3 vs ark-ff 19.5 | −4.2 (0.78×) | 19.6 vs halo2curves 22.0 | −2.4 (0.89×) |
| sub | P-256 base | 15.2 vs ark-ff 17.0 | −1.8 (0.89×) | 11.9 vs halo2curves 12.3 | −0.4 (0.97×) | 15.1 vs ark-ff 18.1 | −3.0 (0.83×) | 18.9 vs halo2curves 21.5 | −2.6 (0.88×) |
| sub | P-256 scalar | 16.5 vs ark-ff 17.8 | −1.3 (0.93×) | 11.9 vs halo2curves 14.4 | −2.5 (0.83×) | 15.5 vs ark-ff 18.8 | −3.3 (0.82×) | 19.7 vs ark-ff 23.0 | −3.3 (0.86×) |
| sub | Tom-256 base | 16.2 vs halo2curves 24.0 | −7.8 (0.67×) | 11.8 vs halo2curves 15.0 | −3.2 (0.79×) | 16.0 vs halo2curves 23.3 | −7.3 (0.69×) | 19.4 vs halo2curves 24.1 | −4.7 (0.80×) |
| sub | BLS12-381 base | 25.5 vs ark-ff 33.7 | −8.2 (0.76×) | 17.9 vs halo2curves 23.5 | −5.6 (0.76×) | 24.1 vs ark-ff 31.6 | −7.5 (0.76×) | 29.5 vs halo2curves 37.0 | −7.5 (0.80×) |
| sub | P-521 base | 47.5 vs crypto-bigint 88.8 | −41.3 (0.53×) | 28.8 vs crypto-bigint 64.4 | −35.6 (0.45×) | 42.3 vs crypto-bigint 122 | −79.7 (0.35×) | 44.2 vs crypto-bigint 127 | −82.8 (0.35×) |
| mul | Mersenne-31 | 5.1 vs crypto-bigint 29.3 | −24.2 (0.17×) | 5.1 vs crypto-bigint 15.8 | −10.7 (0.32×) | 5.5 vs crypto-bigint 38.8 | −33.3 (0.14×) | 5.5 vs crypto-bigint 19.2 | −13.7 (0.29×) |
| mul | BabyBear | 7.8 vs plonky3 4.1 | +3.7 (1.90×) | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 7.8 vs plonky3 6.6 | +1.2 (1.18×) | 5.0 vs plonky3 6.5 | −1.5 (0.77×) |
| mul | Goldilocks | 12.1 vs winter-math 12.3 | −0.2 (0.98×) | 7.0 vs winter-math 6.5 | +0.5 (1.08×) | 12.4 vs winter-math 17.1 | −4.7 (0.73×) | 9.0 vs winter-math 7.8 | +1.2 (1.15×) |
| mul | BN254 scalar | 234 vs ark-ff 322 | −88.0 (0.73×) | 95.6 vs halo2curves 91.7 | +3.9 (1.04×) | 386 vs ark-ff 449 | −63.0 (0.86×) | 162 vs halo2curves 182 | −20.0 (0.89×) |
| mul | secp256k1 base | 160 vs halo2curves 303 | −143 (0.53×) | 63.4 vs halo2curves 91.3 | −27.9 (0.69×) | 289 vs ark-ff 417 | −128 (0.69×) | 112 vs halo2curves 163 | −51.0 (0.69×) |
| mul | P-256 base | 176 vs k256/p256 242 | −66.0 (0.73×) | 71.4 vs k256/p256 84.9 | −13.5 (0.84×) | 317 vs ark-ff 449 | −132 (0.71×) | 119 vs halo2curves 135 | −16.0 (0.88×) |
| mul | P-256 scalar | 239 vs ark-ff 314 | −75.0 (0.76×) | 85.2 vs halo2curves 93.5 | −8.3 (0.91×) | 342 vs ark-ff 431 | −89.0 (0.79×) | 161 vs halo2curves 181 | −20.0 (0.89×) |
| mul | Tom-256 base | 237 vs halo2curves 343 | −106 (0.69×) | 85.5 vs halo2curves 93.7 | −8.2 (0.91×) | 336 vs halo2curves 607 | −271 (0.55×) | 154 vs halo2curves 176 | −22.0 (0.88×) |
| mul | BLS12-381 base | 727 vs ark-ff 807 | −80.0 (0.90×) | 199 vs halo2curves 194 | +5.0 (1.03×) | 944 vs ark-ff 1,093 | −149 (0.86×) | 352 vs halo2curves 381 | −29.0 (0.92×) |
| mul | P-521 base | 665 vs crypto-bigint 2,112 | −1,447 (0.31×) | 253 vs crypto-bigint 600 | −347 (0.42×) | 1,293 vs crypto-bigint 2,726 | −1,433 (0.47×) | 444 vs crypto-bigint 1,045 | −601 (0.42×) |
| square | Mersenne-31 | 5.2 vs crypto-bigint 28.4 | −23.2 (0.18×) | 5.2 vs crypto-bigint 15.6 | −10.4 (0.33×) | 5.4 vs crypto-bigint 38.3 | −32.9 (0.14×) | 5.4 vs crypto-bigint 18.2 | −12.8 (0.30×) |
| square | BabyBear | 8.3 vs plonky3 5.2 | +3.1 (1.60×) | 4.8 vs plonky3 5.2 | −0.4 (0.92×) | 8.0 vs plonky3 5.7 | +2.3 (1.40×) | 5.8 vs plonky3 5.7 | +0.1 (1.02×) |
| square | Goldilocks | 10.1 vs winter-math 12.4 | −2.3 (0.81×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 12.0 vs winter-math 17.0 | −5.0 (0.71×) | 9.7 vs winter-math 7.0 | +2.7 (1.39×) |
| square | BN254 scalar | 214 vs ark-ff 271 | −57.0 (0.79×) | 93.9 vs halo2curves 84.6 | +9.3 (1.11×) | 375 vs ark-ff 373 | +2.0 (1.01×) | 163 vs halo2curves 142 | +21.0 (1.15×) |
| square | secp256k1 base | 144 vs ark-ff 233 | −89.0 (0.62×) | 61.4 vs halo2curves 77.5 | −16.1 (0.79×) | 217 vs ark-ff 304 | −87.0 (0.71×) | 104 vs halo2curves 126 | −22.0 (0.83×) |
| square | P-256 base | 147 vs halo2curves 172 | −25.0 (0.85×) | 78.4 vs k256/p256 74.5 | +3.9 (1.05×) | 258 vs ark-ff 340 | −82.0 (0.76×) | 129 vs halo2curves 107 | +22.0 (1.21×) |
| square | P-256 scalar | 214 vs ark-ff 266 | −52.0 (0.80×) | 80.5 vs halo2curves 83.0 | −2.5 (0.97×) | 290 vs ark-ff 353 | −63.0 (0.82×) | 163 vs halo2curves 144 | +19.0 (1.13×) |
| square | Tom-256 base | 208 vs halo2curves 312 | −104 (0.67×) | 76.3 vs halo2curves 79.6 | −3.3 (0.96×) | 321 vs halo2curves 466 | −145 (0.69×) | 148 vs halo2curves 136 | +12.0 (1.09×) |
| square | BLS12-381 base | 586 vs ark-ff 633 | −47.0 (0.93×) | 189 vs halo2curves 170 | +19.0 (1.11×) | 894 vs ark-ff 844 | +50.0 (1.06×) | 320 vs halo2curves 331 | −11.0 (0.97×) |
| square | P-521 base | 560 vs crypto-bigint 2,157 | −1,597 (0.26×) | 246 vs crypto-bigint 615 | −369 (0.40×) | 923 vs crypto-bigint 2,841 | −1,918 (0.32×) | 396 vs crypto-bigint 1,099 | −703 (0.36×) |
| invert | Mersenne-31 | 266 vs plonky3 676 | −410 (0.39×) | 266 vs plonky3 676 | −410 (0.39×) | 276 vs plonky3 731 | −455 (0.38×) | 271 vs plonky3 723 | −452 (0.37×) |
| invert | BabyBear | 269 vs plonky3 704 | −435 (0.38×) | 268 vs plonky3 699 | −431 (0.38×) | 281 vs plonky3 691 | −410 (0.41×) | 271 vs plonky3 699 | −428 (0.39×) |
| invert | Goldilocks | 537 vs winter-math 1,760 | −1,223 (0.31×) | 538 vs winter-math 912 | −374 (0.59×) | 557 vs winter-math 1,831 | −1,274 (0.30×) | 563 vs winter-math 998 | −435 (0.56×) |
| invert | BN254 scalar | 8,463 vs halo2curves 14,214 | −5,751 (0.60×) | 6,509 vs halo2curves 8,011 | −1,502 (0.81×) | 11,871 vs halo2curves 21,599 | −9,728 (0.55×) | 7,866 vs halo2curves 14,052 | −6,186 (0.56×) |
| invert | secp256k1 base | 8,472 vs halo2curves 14,247 | −5,775 (0.59×) | 6,024 vs halo2curves 8,022 | −1,998 (0.75×) | 11,640 vs halo2curves 21,650 | −10,010 (0.54×) | 8,006 vs halo2curves 14,124 | −6,118 (0.57×) |
| invert | P-256 base | 8,313 vs halo2curves 14,272 | −5,959 (0.58×) | 6,056 vs halo2curves 7,976 | −1,920 (0.76×) | 11,342 vs halo2curves 21,674 | −10,332 (0.52×) | 7,651 vs halo2curves 14,180 | −6,529 (0.54×) |
| invert | P-256 scalar | 8,474 vs halo2curves 14,276 | −5,802 (0.59×) | 6,404 vs halo2curves 8,022 | −1,618 (0.80×) | 12,069 vs halo2curves 21,680 | −9,611 (0.56×) | 8,189 vs halo2curves 14,220 | −6,031 (0.58×) |
| invert | Tom-256 base | 8,506 vs halo2curves 14,312 | −5,806 (0.59×) | 6,469 vs halo2curves 8,026 | −1,557 (0.81×) | 11,385 vs halo2curves 21,645 | −10,260 (0.53×) | 7,989 vs halo2curves 14,113 | −6,124 (0.57×) |
| invert | BLS12-381 base | 19,365 vs halo2curves 25,486 | −6,121 (0.76×) | 11,508 vs halo2curves 13,186 | −1,678 (0.87×) | 24,350 vs halo2curves 38,146 | −13,796 (0.64×) | 16,049 vs halo2curves 23,734 | −7,685 (0.68×) |
| invert | P-521 base | 33,347 vs crypto-bigint 64,815 | −31,468 (0.51×) | 18,149 vs crypto-bigint 50,519 | −32,370 (0.36×) | 48,398 vs crypto-bigint 80,712 | −32,314 (0.60×) | 29,621 vs crypto-bigint 62,079 | −32,458 (0.48×) |
| sqrt | Mersenne-31 | 329 vs plonky3 324 | +5.0 (1.02×) | 329 vs plonky3 324 | +5.0 (1.02×) | 330 vs plonky3 329 | +1.0 (1.00×) | 329 vs plonky3 329 | +0.0 (1.00×) |
| sqrt | BabyBear | 2,679 vs plonky3 3,103 | −424 (0.86×) | 2,032 vs plonky3 3,103 | −1,071 (0.65×) | 2,959 vs plonky3 3,007 | −48.0 (0.98×) | 2,065 vs plonky3 2,997 | −932 (0.69×) |
| sqrt | Goldilocks | 5,778 vs crypto-bigint 48,300 | −42,522 (0.12×) | 3,689 vs crypto-bigint 28,889 | −25,200 (0.13×) | 6,745 vs crypto-bigint 54,744 | −47,999 (0.12×) | 9,289 vs crypto-bigint 32,626 | −23,337 (0.28×) |
| sqrt | BN254 scalar | 108,172 vs ark-ff 148,991 | −40,819 (0.73×) | 46,725 vs halo2curves 102,173 | −55,448 (0.46×) | 198,613 vs ark-ff 205,733 | −7,120 (0.97×) | 82,256 vs ark-ff 142,771 | −60,515 (0.58×) |
| sqrt | secp256k1 base | 42,077 vs ark-ff 133,936 | −91,859 (0.31×) | 19,615 vs halo2curves 43,341 | −23,726 (0.45×) | 61,004 vs ark-ff 174,395 | −113,391 (0.35×) | 31,141 vs crypto-bigint 84,300 | −53,159 (0.37×) |
| sqrt | P-256 base | 44,705 vs ark-ff 59,760 | −15,055 (0.75×) | 19,828 vs k256/p256 21,666 | −1,838 (0.92×) | 78,041 vs ark-ff 97,377 | −19,336 (0.80×) | 31,414 vs k256/p256 33,665 | −2,251 (0.93×) |
| sqrt | P-256 scalar | 74,031 vs ark-ff 115,229 | −41,198 (0.64×) | 35,781 vs halo2curves 49,988 | −14,207 (0.72×) | 114,957 vs ark-ff 160,627 | −45,670 (0.72×) | 63,922 vs halo2curves 72,417 | −8,495 (0.88×) |
| sqrt | Tom-256 base | 65,540 vs crypto-bigint 148,468 | −82,928 (0.44×) | 19,242 vs halo2curves 45,267 | −26,025 (0.43×) | 94,543 vs crypto-bigint 232,503 | −137,960 (0.41×) | 49,342 vs crypto-bigint 77,789 | −28,447 (0.63×) |
| sqrt | BLS12-381 base | 315,485 vs ark-ff 419,279 | −103,794 (0.75×) | 87,392 vs crypto-bigint 145,031 | −57,639 (0.60×) | 420,212 vs ark-ff 581,434 | −161,222 (0.72×) | 203,949 vs crypto-bigint 267,711 | −63,762 (0.76×) |
| sqrt | P-521 base | 320,271 vs crypto-bigint 1,524,172 | −1,203,901 (0.21×) | 131,305 vs crypto-bigint 296,736 | −165,431 (0.44×) | 473,617 vs crypto-bigint 1,560,230 | −1,086,613 (0.30×) | 260,337 vs crypto-bigint 590,262 | −329,925 (0.44×) |
| mul_u64 | Mersenne-31 | 9.4 vs crypto-bigint 61.4 | −52.0 (0.15×) | 9.4 vs crypto-bigint 30.6 | −21.2 (0.31×) | 9.8 vs crypto-bigint 74.4 | −64.6 (0.13×) | 9.8 vs crypto-bigint 38.9 | −29.1 (0.25×) |
| mul_u64 | BabyBear | 17.1 vs plonky3 35.0 | −17.9 (0.49×) | 11.2 vs plonky3 12.0 | −0.8 (0.93×) | 20.1 vs plonky3 45.6 | −25.5 (0.44×) | 15.0 vs plonky3 18.0 | −3.0 (0.83×) |
| mul_u64 | Goldilocks | 19.0 vs winter-math 25.0 | −6.0 (0.76×) | 11.0 vs winter-math 10.9 | +0.1 (1.01×) | 21.1 vs winter-math 31.1 | −10.0 (0.68×) | 16.7 vs winter-math 13.6 | +3.1 (1.23×) |
| mul_u64 | BN254 scalar | 101 vs ark-ff 696 | −595 (0.15×) | 28.8 vs crypto-bigint 263 | −234 (0.11×) | 122 vs ark-ff 814 | −692 (0.15×) | 44.2 vs crypto-bigint 407 | −363 (0.11×) |
| mul_u64 | secp256k1 base | 43.5 vs ark-ff 447 | −404 (0.10×) | 12.6 vs crypto-bigint 263 | −250 (0.05×) | 66.8 vs ark-ff 627 | −560 (0.11×) | 24.0 vs crypto-bigint 408 | −384 (0.06×) |
| mul_u64 | P-256 base | 72.3 vs k256/p256 485 | −413 (0.15×) | 27.3 vs k256/p256 150 | −123 (0.18×) | 98.1 vs ark-ff 835 | −737 (0.12×) | 41.3 vs k256/p256 258 | −217 (0.16×) |
| mul_u64 | P-256 scalar | 84.1 vs k256/p256 516 | −432 (0.16×) | 30.3 vs k256/p256 253 | −223 (0.12×) | 113 vs k256/p256 816 | −703 (0.14×) | 48.2 vs k256/p256 390 | −342 (0.12×) |
| mul_u64 | Tom-256 base | 82.1 vs crypto-bigint 868 | −786 (0.09×) | 30.0 vs crypto-bigint 263 | −233 (0.11×) | 115 vs halo2curves 1,283 | −1,168 (0.09×) | 44.6 vs crypto-bigint 406 | −361 (0.11×) |
| mul_u64 | BLS12-381 base | 131 vs ark-ff 1,641 | −1,510 (0.08×) | 44.1 vs halo2curves 483 | −439 (0.09×) | 194 vs ark-ff 2,130 | −1,936 (0.09×) | 61.6 vs crypto-bigint 1,020 | −958 (0.06×) |
| mul_u64 | P-521 base | 92.3 vs crypto-bigint 3,038 | −2,946 (0.03×) | 32.0 vs crypto-bigint 991 | −959 (0.03×) | 228 vs crypto-bigint 3,974 | −3,746 (0.06×) | 49.3 vs crypto-bigint 1,831 | −1,782 (0.03×) |
| mul_uint | Mersenne-31 | 9.4 vs crypto-bigint 61.9 | −52.5 (0.15×) | 9.4 vs crypto-bigint 30.7 | −21.3 (0.31×) | 9.8 vs crypto-bigint 72.8 | −63.0 (0.13×) | 9.8 vs crypto-bigint 38.8 | −29.0 (0.25×) |
| mul_uint | BabyBear | 17.1 vs plonky3 23.1 | −6.0 (0.74×) | 11.2 vs plonky3 11.7 | −0.5 (0.96×) | 20.1 vs plonky3 35.0 | −14.9 (0.57×) | 15.0 vs plonky3 17.9 | −2.9 (0.84×) |
| mul_uint | Goldilocks | 19.2 vs winter-math 25.0 | −5.8 (0.77×) | 11.0 vs winter-math 10.9 | +0.1 (1.01×) | 21.1 vs winter-math 31.0 | −9.9 (0.68×) | 16.7 vs winter-math 13.6 | +3.1 (1.23×) |
| mul_uint | BN254 scalar | 434 vs halo2curves 769 | −335 (0.56×) | 179 vs halo2curves 180 | −1.0 (0.99×) | 523 vs halo2curves 1,348 | −825 (0.39×) | 303 vs halo2curves 349 | −46.0 (0.87×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 519 | −253 (0.51×) | 114 vs halo2curves 154 | −40.0 (0.74×) | 317 vs halo2curves 959 | −642 (0.33×) | 164 vs halo2curves 266 | −102 (0.62×) |
| mul_uint | P-256 base | 353 vs halo2curves 458 | −105 (0.77×) | 134 vs halo2curves 143 | −9.0 (0.94×) | 422 vs halo2curves 954 | −532 (0.44×) | 243 vs halo2curves 252 | −9.0 (0.96×) |
| mul_uint | P-256 scalar | 426 vs k256/p256 547 | −121 (0.78×) | 180 vs halo2curves 186 | −6.0 (0.97×) | 478 vs k256/p256 882 | −404 (0.54×) | 295 vs crypto-bigint 409 | −114 (0.72×) |
| mul_uint | Tom-256 base | 411 vs halo2curves 695 | −284 (0.59×) | 166 vs halo2curves 180 | −14.0 (0.92×) | 493 vs halo2curves 1,176 | −683 (0.42×) | 351 vs halo2curves 335 | +16.0 (1.05×) |
| mul_uint | BLS12-381 base | 990 vs halo2curves 1,940 | −950 (0.51×) | 371 vs halo2curves 381 | −10.0 (0.97×) | 1,139 vs crypto-bigint 2,519 | −1,380 (0.45×) | 856 vs crypto-bigint 982 | −126 (0.87×) |
| mul_uint | P-521 base | 1,543 vs crypto-bigint 3,030 | −1,487 (0.51×) | 619 vs crypto-bigint 990 | −371 (0.63×) | 2,383 vs crypto-bigint 4,016 | −1,633 (0.59×) | 982 vs crypto-bigint 1,835 | −853 (0.54×) |
| add_u64 | Mersenne-31 | 8.2 vs crypto-bigint 36.1 | −27.9 (0.23×) | 8.2 vs crypto-bigint 30.4 | −22.2 (0.27×) | 9.1 vs crypto-bigint 42.2 | −33.1 (0.22×) | 9.2 vs crypto-bigint 30.4 | −21.2 (0.30×) |
| add_u64 | BabyBear | 10.9 vs plonky3 32.1 | −21.2 (0.34×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 43.4 | −30.4 (0.30×) | 12.5 vs plonky3 16.0 | −3.5 (0.78×) |
| add_u64 | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 11.0 | +1.0 (1.09×) |
| add_u64 | BN254 scalar | 120 vs ark-ff 348 | −228 (0.34×) | 55.8 vs halo2curves 210 | −154 (0.27×) | 179 vs ark-ff 412 | −233 (0.43×) | 91.5 vs halo2curves 295 | −204 (0.31×) |
| add_u64 | secp256k1 base | 8.0 vs ark-ff 163 | −155 (0.05×) | 8.0 vs ark-ff 121 | −113 (0.07×) | 8.0 vs ark-ff 229 | −221 (0.03×) | 12.0 vs ark-ff 185 | −173 (0.06×) |
| add_u64 | P-256 base | 72.3 vs ark-ff 273 | −201 (0.26×) | 35.8 vs k256/p256 95.2 | −59.4 (0.38×) | 86.7 vs ark-ff 407 | −320 (0.21×) | 68.5 vs k256/p256 160 | −91.5 (0.43×) |
| add_u64 | P-256 scalar | 70.2 vs k256/p256 41.3 | +28.9 (1.70×) | 40.4 vs k256/p256 41.3 | −0.9 (0.98×) | 72.5 vs k256/p256 54.6 | +17.9 (1.33×) | 83.3 vs k256/p256 55.0 | +28.3 (1.51×) |
| add_u64 | Tom-256 base | 89.7 vs crypto-bigint 504 | −414 (0.18×) | 41.7 vs halo2curves 218 | −176 (0.19×) | 109 vs halo2curves 686 | −577 (0.16×) | 71.7 vs crypto-bigint 316 | −244 (0.23×) |
| add_u64 | BLS12-381 base | 178 vs ark-ff 842 | −664 (0.21×) | 66.6 vs halo2curves 324 | −257 (0.21×) | 262 vs ark-ff 1,093 | −831 (0.24×) | 95.3 vs crypto-bigint 650 | −555 (0.15×) |
| add_u64 | P-521 base | 23.1 vs crypto-bigint 1,688 | −1,665 (0.01×) | 16.1 vs crypto-bigint 651 | −635 (0.02×) | 22.3 vs crypto-bigint 2,197 | −2,175 (0.01×) | 25.1 vs crypto-bigint 1,117 | −1,092 (0.02×) |
| add_uint | Mersenne-31 | 8.2 vs crypto-bigint 36.0 | −27.8 (0.23×) | 8.2 vs crypto-bigint 30.4 | −22.2 (0.27×) | 9.1 vs crypto-bigint 41.5 | −32.4 (0.22×) | 9.2 vs crypto-bigint 30.6 | −21.4 (0.30×) |
| add_uint | BabyBear | 10.9 vs plonky3 22.0 | −11.1 (0.50×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 34.3 | −21.3 (0.38×) | 12.6 vs plonky3 16.0 | −3.4 (0.79×) |
| add_uint | Goldilocks | 10.0 vs winter-math 13.3 | −3.3 (0.75×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 11.0 | +1.0 (1.09×) |
| add_uint | BN254 scalar | 252 vs halo2curves 415 | −163 (0.61×) | 119 vs halo2curves 112 | +7.0 (1.06×) | 387 vs halo2curves 678 | −291 (0.57×) | 190 vs halo2curves 205 | −15.0 (0.93×) |
| add_uint | secp256k1 base | 21.0 vs halo2curves 224 | −203 (0.09×) | 14.6 vs halo2curves 84.9 | −70.3 (0.17×) | 18.4 vs halo2curves 415 | −397 (0.04×) | 24.0 vs halo2curves 140 | −116 (0.17×) |
| add_uint | P-256 base | 189 vs halo2curves 248 | −59.0 (0.76×) | 79.7 vs halo2curves 91.5 | −11.8 (0.87×) | 327 vs halo2curves 509 | −182 (0.64×) | 148 vs halo2curves 154 | −6.0 (0.96×) |
| add_uint | P-256 scalar | 261 vs k256/p256 94.7 | +166 (2.76×) | 116 vs k256/p256 94.9 | +21.1 (1.22×) | 350 vs k256/p256 118 | +232 (2.97×) | 211 vs k256/p256 119 | +92.0 (1.77×) |
| add_uint | Tom-256 base | 249 vs halo2curves 380 | −131 (0.66×) | 109 vs halo2curves 110 | −1.0 (0.99×) | 339 vs halo2curves 629 | −290 (0.54×) | 198 vs halo2curves 196 | +2.0 (1.01×) |
| add_uint | BLS12-381 base | 775 vs halo2curves 974 | −199 (0.80×) | 205 vs halo2curves 225 | −20.0 (0.91×) | 970 vs crypto-bigint 1,395 | −425 (0.70×) | 419 vs halo2curves 562 | −143 (0.75×) |
| add_uint | P-521 base | 65.0 vs crypto-bigint 1,676 | −1,611 (0.04×) | 41.8 vs crypto-bigint 648 | −606 (0.06×) | 88.7 vs crypto-bigint 2,227 | −2,138 (0.04×) | 71.5 vs crypto-bigint 1,127 | −1,056 (0.06×) |
| sub_u64 | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.0 vs crypto-bigint 31.3 | −24.3 (0.22×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_u64 | BabyBear | 11.1 vs crypto-bigint 27.9 | −16.8 (0.40×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 15.8 vs plonky3 44.0 | −28.2 (0.36×) | 12.6 vs plonky3 15.0 | −2.4 (0.84×) |
| sub_u64 | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 11.0 vs winter-math 9.6 | +1.4 (1.15×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 354 | −241 (0.32×) | 53.9 vs crypto-bigint 151 | −97.1 (0.36×) | 138 vs ark-ff 415 | −277 (0.33×) | 74.2 vs crypto-bigint 279 | −205 (0.27×) |
| sub_u64 | secp256k1 base | 8.7 vs ark-ff 161 | −152 (0.05×) | 8.0 vs ark-ff 123 | −115 (0.07×) | 7.5 vs ark-ff 232 | −225 (0.03×) | 11.0 vs ark-ff 193 | −182 (0.06×) |
| sub_u64 | P-256 base | 72.1 vs k256/p256 266 | −194 (0.27×) | 35.9 vs k256/p256 89.1 | −53.2 (0.40×) | 81.0 vs ark-ff 408 | −327 (0.20×) | 61.7 vs k256/p256 149 | −87.3 (0.41×) |
| sub_u64 | P-256 scalar | 69.0 vs k256/p256 21.2 | +47.8 (3.25×) | 40.1 vs k256/p256 21.2 | +18.9 (1.89×) | 75.9 vs k256/p256 24.4 | +51.5 (3.11×) | 82.7 vs k256/p256 24.3 | +58.4 (3.40×) |
| sub_u64 | Tom-256 base | 88.7 vs crypto-bigint 506 | −417 (0.18×) | 51.3 vs crypto-bigint 152 | −101 (0.34×) | 106 vs halo2curves 665 | −559 (0.16×) | 71.8 vs crypto-bigint 259 | −187 (0.28×) |
| sub_u64 | BLS12-381 base | 163 vs ark-ff 844 | −681 (0.19×) | 67.6 vs halo2curves 317 | −249 (0.21×) | 254 vs ark-ff 1,067 | −813 (0.24×) | 96.7 vs crypto-bigint 618 | −521 (0.16×) |
| sub_u64 | P-521 base | 22.2 vs crypto-bigint 1,674 | −1,652 (0.01×) | 18.8 vs crypto-bigint 623 | −604 (0.03×) | 20.3 vs crypto-bigint 2,184 | −2,164 (0.01×) | 21.2 vs crypto-bigint 1,124 | −1,103 (0.02×) |
| sub_uint | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 18.2 | −12.0 (0.34×) | 7.0 vs crypto-bigint 31.3 | −24.3 (0.22×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_uint | BabyBear | 11.1 vs plonky3 23.0 | −11.9 (0.48×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 15.8 vs plonky3 34.4 | −18.6 (0.46×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| sub_uint | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_uint | BN254 scalar | 253 vs halo2curves 403 | −150 (0.63×) | 118 vs halo2curves 103 | +15.0 (1.15×) | 384 vs halo2curves 684 | −300 (0.56×) | 191 vs halo2curves 194 | −3.0 (0.98×) |
| sub_uint | secp256k1 base | 21.2 vs halo2curves 209 | −188 (0.10×) | 14.6 vs halo2curves 110 | −95.4 (0.13×) | 18.2 vs halo2curves 409 | −391 (0.04×) | 24.2 vs halo2curves 130 | −106 (0.19×) |
| sub_uint | P-256 base | 193 vs halo2curves 233 | −40.0 (0.83×) | 88.0 vs halo2curves 85.3 | +2.7 (1.03×) | 359 vs halo2curves 494 | −135 (0.73×) | 150 vs halo2curves 143 | +7.0 (1.05×) |
| sub_uint | P-256 scalar | 267 vs k256/p256 64.8 | +202 (4.12×) | 116 vs k256/p256 65.0 | +51.0 (1.78×) | 357 vs k256/p256 89.0 | +268 (4.01×) | 212 vs k256/p256 88.9 | +123 (2.38×) |
| sub_uint | Tom-256 base | 254 vs halo2curves 365 | −111 (0.70×) | 114 vs halo2curves 103 | +11.0 (1.11×) | 339 vs halo2curves 617 | −278 (0.55×) | 202 vs halo2curves 187 | +15.0 (1.08×) |
| sub_uint | BLS12-381 base | 758 vs halo2curves 962 | −204 (0.79×) | 212 vs halo2curves 215 | −3.0 (0.99×) | 949 vs crypto-bigint 1,373 | −424 (0.69×) | 402 vs halo2curves 408 | −6.0 (0.99×) |
| sub_uint | P-521 base | 75.9 vs crypto-bigint 1,674 | −1,598 (0.05×) | 42.3 vs crypto-bigint 621 | −579 (0.07×) | 81.1 vs crypto-bigint 2,196 | −2,115 (0.04×) | 69.0 vs crypto-bigint 1,104 | −1,035 (0.06×) |
| from_u64 | Mersenne-31 | 4.7 vs plonky3 4.1 | +0.6 (1.15×) | 4.7 vs plonky3 4.1 | +0.6 (1.15×) | 5.2 vs plonky3 3.9 | +1.3 (1.33×) | 5.3 vs plonky3 3.9 | +1.4 (1.36×) |
| from_u64 | BabyBear | 11.1 vs crypto-bigint 21.6 | −10.5 (0.51×) | 7.3 vs plonky3 7.8 | −0.5 (0.94×) | 12.9 vs crypto-bigint 36.1 | −23.2 (0.36×) | 10.3 vs plonky3 11.8 | −1.5 (0.87×) |
| from_u64 | Goldilocks | 7.2 vs winter-math 10.6 | −3.4 (0.68×) | 5.5 vs winter-math 6.0 | −0.5 (0.92×) | 9.0 vs winter-math 17.0 | −8.0 (0.53×) | 8.7 vs winter-math 7.0 | +1.7 (1.24×) |
| from_u64 | BN254 scalar | 87.0 vs ark-ff 315 | −228 (0.28×) | 28.2 vs crypto-bigint 133 | −105 (0.21×) | 121 vs ark-ff 375 | −254 (0.32×) | 45.0 vs crypto-bigint 205 | −160 (0.22×) |
| from_u64 | secp256k1 base | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) |
| from_u64 | P-256 base | 51.4 vs k256/p256 239 | −188 (0.22×) | 23.7 vs k256/p256 76.2 | −52.5 (0.31×) | 61.4 vs ark-ff 376 | −315 (0.16×) | 36.1 vs k256/p256 130 | −93.9 (0.28×) |
| from_u64 | P-256 scalar | 51.5 vs k256/p256 3.8 | +47.7 (13.55×) | 25.9 vs k256/p256 3.8 | +22.1 (6.82×) | 59.2 vs k256/p256 3.7 | +55.5 (16.00×) | 43.1 vs k256/p256 3.8 | +39.3 (11.34×) |
| from_u64 | Tom-256 base | 72.0 vs crypto-bigint 428 | −356 (0.17×) | 28.6 vs crypto-bigint 133 | −104 (0.22×) | 85.7 vs halo2curves 667 | −581 (0.13×) | 43.4 vs crypto-bigint 205 | −162 (0.21×) |
| from_u64 | BLS12-381 base | 141 vs ark-ff 811 | −670 (0.17×) | 44.6 vs crypto-bigint 252 | −207 (0.18×) | 234 vs ark-ff 1,022 | −788 (0.23×) | 63.1 vs crypto-bigint 479 | −416 (0.13×) |
| from_u64 | P-521 base | 6.1 vs crypto-bigint 1,448 | −1,442 (0.00×) | 6.0 vs crypto-bigint 496 | −490 (0.01×) | 6.0 vs crypto-bigint 1,995 | −1,989 (0.00×) | 6.1 vs crypto-bigint 850 | −844 (0.01×) |
| div_u64 | BN254 scalar | 1,214 vs halo2curves 13,461 | −12,247 (0.09×) | 821 vs halo2curves 7,020 | −6,199 (0.12×) | 1,419 vs halo2curves 19,554 | −18,135 (0.07×) | 1,049 vs halo2curves 11,740 | −10,691 (0.09×) |
| div_u64 | secp256k1 base | 1,000 vs halo2curves 13,021 | −12,021 (0.08×) | 766 vs halo2curves 6,977 | −6,211 (0.11×) | 1,217 vs halo2curves 19,115 | −17,898 (0.06×) | 877 vs halo2curves 11,503 | −10,626 (0.08×) |
| div_u64 | P-256 base | 1,089 vs halo2curves 13,112 | −12,023 (0.08×) | 808 vs halo2curves 6,914 | −6,106 (0.12×) | 1,356 vs halo2curves 18,926 | −17,570 (0.07×) | 987 vs halo2curves 11,529 | −10,542 (0.09×) |
| div_u64 | P-256 scalar | 1,185 vs halo2curves 13,366 | −12,181 (0.09×) | 864 vs halo2curves 6,950 | −6,086 (0.12×) | 1,405 vs halo2curves 19,572 | −18,167 (0.07×) | 1,152 vs halo2curves 11,679 | −10,527 (0.10×) |
| div_u64 | Tom-256 base | 1,188 vs halo2curves 13,321 | −12,133 (0.09×) | 864 vs halo2curves 6,977 | −6,113 (0.12×) | 1,478 vs halo2curves 19,296 | −17,818 (0.08×) | 1,036 vs halo2curves 11,639 | −10,603 (0.09×) |
| div_u64 | BLS12-381 base | 1,795 vs halo2curves 24,322 | −22,527 (0.07×) | 1,033 vs halo2curves 11,012 | −9,979 (0.09×) | 2,510 vs halo2curves 32,685 | −30,175 (0.08×) | 1,750 vs halo2curves 18,871 | −17,121 (0.09×) |
| div_u64 | P-521 base | 2,450 vs crypto-bigint 62,554 | −60,104 (0.04×) | 1,369 vs crypto-bigint 45,420 | −44,051 (0.03×) | 3,622 vs crypto-bigint 77,929 | −74,307 (0.05×) | 1,878 vs crypto-bigint 58,185 | −56,307 (0.03×) |
| div_uint | Mersenne-31 | 276 vs crypto-bigint 1,816 | −1,540 (0.15×) | 278 vs crypto-bigint 1,620 | −1,342 (0.17×) | 281 vs runtime-algebra 2,131 | −1,850 (0.13×) | 287 vs crypto-bigint 2,023 | −1,736 (0.14×) |
| div_uint | BabyBear | 277 vs plonky3 682 | −405 (0.41×) | 278 vs plonky3 666 | −388 (0.42×) | 290 vs plonky3 728 | −438 (0.40×) | 283 vs plonky3 733 | −450 (0.39×) |
| div_uint | Goldilocks | 551 vs winter-math 1,800 | −1,249 (0.31×) | 545 vs winter-math 948 | −403 (0.57×) | 593 vs winter-math 1,902 | −1,309 (0.31×) | 596 vs winter-math 1,045 | −449 (0.57×) |
| div_uint | BN254 scalar | 9,221 vs halo2curves 15,070 | −5,849 (0.61×) | 6,797 vs halo2curves 8,220 | −1,423 (0.83×) | 12,816 vs halo2curves 22,812 | −9,996 (0.56×) | 8,430 vs halo2curves 14,464 | −6,034 (0.58×) |
| div_uint | secp256k1 base | 8,563 vs halo2curves 14,880 | −6,317 (0.58×) | 6,022 vs halo2curves 8,223 | −2,201 (0.73×) | 11,918 vs halo2curves 22,915 | −10,997 (0.52×) | 8,094 vs halo2curves 14,441 | −6,347 (0.56×) |
| div_uint | P-256 base | 8,830 vs halo2curves 14,899 | −6,069 (0.59×) | 6,165 vs halo2curves 8,238 | −2,073 (0.75×) | 12,053 vs halo2curves 22,569 | −10,516 (0.53×) | 7,932 vs halo2curves 14,291 | −6,359 (0.56×) |
| div_uint | P-256 scalar | 9,235 vs halo2curves 15,137 | −5,902 (0.61×) | 6,623 vs halo2curves 8,260 | −1,637 (0.80×) | 12,560 vs halo2curves 23,001 | −10,441 (0.55×) | 8,739 vs halo2curves 14,616 | −5,877 (0.60×) |
| div_uint | Tom-256 base | 9,148 vs halo2curves 15,038 | −5,890 (0.61×) | 6,658 vs halo2curves 8,256 | −1,598 (0.81×) | 12,378 vs halo2curves 22,975 | −10,597 (0.54×) | 8,327 vs halo2curves 14,527 | −6,200 (0.57×) |
| div_uint | BLS12-381 base | 20,940 vs halo2curves 27,763 | −6,823 (0.75×) | 12,042 vs halo2curves 13,709 | −1,667 (0.88×) | 26,464 vs halo2curves 40,634 | −14,170 (0.65×) | 16,866 vs halo2curves 24,359 | −7,493 (0.69×) |
| div_uint | P-521 base | 34,250 vs crypto-bigint 68,362 | −34,112 (0.50×) | 18,513 vs crypto-bigint 51,058 | −32,545 (0.36×) | 50,018 vs crypto-bigint 84,804 | −34,786 (0.59×) | 29,895 vs crypto-bigint 63,936 | −34,041 (0.47×) |
| add, constant time | Mersenne-31 | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.2 vs runtime-algebra 10.8 | −6.6 (0.39×) | 4.4 vs runtime-algebra 10.8 | −6.4 (0.41×) | 4.4 vs runtime-algebra 10.9 | −6.5 (0.40×) |
| add, constant time | BabyBear | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 3.0 vs runtime-algebra 10.8 | −7.8 (0.28×) | 3.0 vs runtime-algebra 10.9 | −7.9 (0.28×) |
| add, constant time | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) |
| add, constant time | BN254 scalar | 23.6 vs halo2curves 36.4 | −12.8 (0.65×) | 16.3 vs halo2curves 20.6 | −4.3 (0.79×) | 20.7 vs halo2curves 39.4 | −18.7 (0.53×) | 27.1 vs halo2curves 35.0 | −7.9 (0.77×) |
| add, constant time | secp256k1 base | 26.3 vs halo2curves 35.0 | −8.7 (0.75×) | 18.1 vs halo2curves 19.3 | −1.2 (0.94×) | 24.5 vs halo2curves 35.8 | −11.3 (0.68×) | 29.0 vs halo2curves 32.9 | −3.9 (0.88×) |
| add, constant time | P-256 base | 23.6 vs halo2curves 30.5 | −6.9 (0.77×) | 18.1 vs halo2curves 17.9 | +0.2 (1.01×) | 22.9 vs halo2curves 33.7 | −10.8 (0.68×) | 28.0 vs halo2curves 31.5 | −3.5 (0.89×) |
| add, constant time | P-256 scalar | 27.1 vs halo2curves 35.6 | −8.5 (0.76×) | 17.9 vs halo2curves 20.6 | −2.7 (0.87×) | 24.1 vs halo2curves 38.0 | −13.9 (0.63×) | 29.5 vs halo2curves 34.9 | −5.4 (0.85×) |
| add, constant time | Tom-256 base | 27.6 vs halo2curves 35.8 | −8.2 (0.77×) | 18.0 vs halo2curves 20.2 | −2.2 (0.89×) | 23.8 vs halo2curves 38.2 | −14.4 (0.62×) | 28.5 vs halo2curves 34.0 | −5.5 (0.84×) |
| add, constant time | BLS12-381 base | 42.7 vs halo2curves 63.7 | −21.0 (0.67×) | 25.6 vs halo2curves 32.7 | −7.1 (0.78×) | 35.7 vs halo2curves 69.9 | −34.2 (0.51×) | 41.2 vs halo2curves 54.0 | −12.8 (0.76×) |
| add, constant time | P-521 base | 76.6 vs crypto-bigint 146 | −69.4 (0.52×) | 46.6 vs crypto-bigint 133 | −86.4 (0.35×) | 84.8 vs crypto-bigint 176 | −91.2 (0.48×) | 74.1 vs crypto-bigint 184 | −110 (0.40×) |
| sub, constant time | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | BabyBear | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 22.9 vs halo2curves 24.0 | −1.1 (0.95×) | 16.5 vs halo2curves 15.0 | +1.5 (1.10×) | 21.4 vs halo2curves 23.3 | −1.9 (0.92×) | 26.0 vs halo2curves 24.1 | +1.9 (1.08×) |
| sub, constant time | secp256k1 base | 19.2 vs halo2curves 22.5 | −3.3 (0.85×) | 14.4 vs halo2curves 14.2 | +0.2 (1.01×) | 18.7 vs halo2curves 20.3 | −1.6 (0.92×) | 22.5 vs halo2curves 22.0 | +0.5 (1.02×) |
| sub, constant time | P-256 base | 17.3 vs halo2curves 20.5 | −3.2 (0.84×) | 13.4 vs halo2curves 12.3 | +1.1 (1.09×) | 18.5 vs halo2curves 21.1 | −2.6 (0.88×) | 22.0 vs halo2curves 21.5 | +0.5 (1.02×) |
| sub, constant time | P-256 scalar | 21.0 vs k256/p256 23.1 | −2.1 (0.91×) | 14.7 vs halo2curves 14.4 | +0.3 (1.02×) | 22.1 vs halo2curves 23.6 | −1.5 (0.94×) | 23.5 vs halo2curves 23.0 | +0.5 (1.02×) |
| sub, constant time | Tom-256 base | 21.1 vs halo2curves 24.0 | −2.9 (0.88×) | 15.2 vs halo2curves 15.0 | +0.2 (1.01×) | 21.0 vs halo2curves 23.3 | −2.3 (0.90×) | 24.0 vs halo2curves 24.1 | −0.1 (1.00×) |
| sub, constant time | BLS12-381 base | 37.7 vs halo2curves 43.3 | −5.6 (0.87×) | 24.4 vs halo2curves 23.5 | +0.9 (1.04×) | 35.3 vs halo2curves 44.2 | −8.9 (0.80×) | 37.3 vs halo2curves 37.0 | +0.3 (1.01×) |
| sub, constant time | P-521 base | 56.3 vs crypto-bigint 88.8 | −32.5 (0.63×) | 37.2 vs crypto-bigint 64.4 | −27.2 (0.58×) | 58.2 vs crypto-bigint 122 | −63.8 (0.48×) | 53.1 vs crypto-bigint 127 | −73.9 (0.42×) |
| mul, constant time | Mersenne-31 | 5.2 vs crypto-bigint 29.3 | −24.1 (0.18×) | 5.1 vs crypto-bigint 15.8 | −10.7 (0.32×) | 5.5 vs crypto-bigint 38.8 | −33.3 (0.14×) | 5.5 vs crypto-bigint 19.2 | −13.7 (0.29×) |
| mul, constant time | BabyBear | 7.8 vs crypto-bigint 29.3 | −21.5 (0.27×) | 5.2 vs crypto-bigint 15.8 | −10.6 (0.33×) | 7.8 vs crypto-bigint 39.2 | −31.4 (0.20×) | 5.0 vs crypto-bigint 19.2 | −14.2 (0.26×) |
| mul, constant time | Goldilocks | 12.1 vs winter-math 12.3 | −0.2 (0.98×) | 7.0 vs winter-math 6.5 | +0.5 (1.08×) | 12.4 vs winter-math 17.1 | −4.7 (0.73×) | 9.0 vs winter-math 7.8 | +1.2 (1.15×) |
| mul, constant time | BN254 scalar | 236 vs halo2curves 378 | −142 (0.62×) | 86.7 vs halo2curves 91.7 | −5.0 (0.95×) | 418 vs halo2curves 709 | −291 (0.59×) | 151 vs halo2curves 182 | −31.0 (0.83×) |
| mul, constant time | secp256k1 base | 160 vs halo2curves 303 | −143 (0.53×) | 63.6 vs halo2curves 91.3 | −27.7 (0.70×) | 275 vs halo2curves 589 | −314 (0.47×) | 108 vs halo2curves 163 | −55.0 (0.66×) |
| mul, constant time | P-256 base | 173 vs k256/p256 242 | −69.0 (0.71×) | 71.8 vs k256/p256 84.9 | −13.1 (0.85×) | 321 vs k256/p256 484 | −163 (0.66×) | 121 vs halo2curves 135 | −14.0 (0.90×) |
| mul, constant time | P-256 scalar | 244 vs halo2curves 343 | −99.0 (0.71×) | 85.4 vs halo2curves 93.5 | −8.1 (0.91×) | 343 vs halo2curves 691 | −348 (0.50×) | 168 vs halo2curves 181 | −13.0 (0.93×) |
| mul, constant time | Tom-256 base | 235 vs halo2curves 343 | −108 (0.69×) | 85.8 vs halo2curves 93.7 | −7.9 (0.92×) | 331 vs halo2curves 607 | −276 (0.55×) | 157 vs halo2curves 176 | −19.0 (0.89×) |
| mul, constant time | BLS12-381 base | 728 vs crypto-bigint 903 | −175 (0.81×) | 184 vs halo2curves 194 | −10.0 (0.95×) | 935 vs crypto-bigint 1,327 | −392 (0.70×) | 337 vs halo2curves 381 | −44.0 (0.88×) |
| mul, constant time | P-521 base | 666 vs crypto-bigint 2,112 | −1,446 (0.32×) | 252 vs crypto-bigint 600 | −348 (0.42×) | 1,287 vs crypto-bigint 2,726 | −1,439 (0.47×) | 444 vs crypto-bigint 1,045 | −601 (0.42×) |
| square, constant time | Mersenne-31 | 5.2 vs crypto-bigint 28.4 | −23.2 (0.18×) | 5.2 vs crypto-bigint 15.6 | −10.4 (0.33×) | 5.4 vs crypto-bigint 38.3 | −32.9 (0.14×) | 5.4 vs crypto-bigint 18.2 | −12.8 (0.30×) |
| square, constant time | BabyBear | 8.3 vs crypto-bigint 28.5 | −20.2 (0.29×) | 4.9 vs crypto-bigint 15.6 | −10.7 (0.31×) | 8.0 vs crypto-bigint 38.5 | −30.5 (0.21×) | 5.8 vs crypto-bigint 18.3 | −12.5 (0.32×) |
| square, constant time | Goldilocks | 10.1 vs winter-math 12.4 | −2.3 (0.81×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 12.0 vs winter-math 17.0 | −5.0 (0.71×) | 9.7 vs winter-math 7.0 | +2.7 (1.39×) |
| square, constant time | BN254 scalar | 213 vs crypto-bigint 324 | −111 (0.66×) | 78.1 vs halo2curves 84.6 | −6.5 (0.92×) | 370 vs halo2curves 559 | −189 (0.66×) | 147 vs halo2curves 142 | +5.0 (1.04×) |
| square, constant time | secp256k1 base | 143 vs halo2curves 267 | −124 (0.54×) | 61.3 vs halo2curves 77.5 | −16.2 (0.79×) | 217 vs halo2curves 463 | −246 (0.47×) | 98.1 vs halo2curves 126 | −27.9 (0.78×) |
| square, constant time | P-256 base | 151 vs halo2curves 172 | −21.0 (0.88×) | 78.4 vs k256/p256 74.5 | +3.9 (1.05×) | 253 vs halo2curves 349 | −96.0 (0.72×) | 131 vs halo2curves 107 | +24.0 (1.22×) |
| square, constant time | P-256 scalar | 214 vs halo2curves 306 | −92.0 (0.70×) | 80.7 vs halo2curves 83.0 | −2.3 (0.97×) | 288 vs halo2curves 561 | −273 (0.51×) | 140 vs halo2curves 144 | −4.0 (0.97×) |
| square, constant time | Tom-256 base | 203 vs halo2curves 312 | −109 (0.65×) | 77.2 vs halo2curves 79.6 | −2.4 (0.97×) | 321 vs halo2curves 466 | −145 (0.69×) | 166 vs halo2curves 136 | +30.0 (1.22×) |
| square, constant time | BLS12-381 base | 582 vs halo2curves 660 | −78.0 (0.88×) | 172 vs halo2curves 170 | +2.0 (1.01×) | 886 vs halo2curves 1,324 | −438 (0.67×) | 308 vs halo2curves 331 | −23.0 (0.93×) |
| square, constant time | P-521 base | 563 vs crypto-bigint 2,157 | −1,594 (0.26×) | 246 vs crypto-bigint 615 | −369 (0.40×) | 917 vs crypto-bigint 2,841 | −1,924 (0.32×) | 398 vs crypto-bigint 1,099 | −701 (0.36×) |
| invert, constant time | Mersenne-31 | 438 vs runtime-algebra 3,343 | −2,905 (0.13×) | 441 vs runtime-algebra 3,342 | −2,901 (0.13×) | 431 vs runtime-algebra 3,417 | −2,986 (0.13×) | 430 vs runtime-algebra 3,319 | −2,889 (0.13×) |
| invert, constant time | BabyBear | 520 vs runtime-algebra 3,323 | −2,803 (0.16×) | 458 vs runtime-algebra 3,301 | −2,843 (0.14×) | 550 vs runtime-algebra 3,382 | −2,832 (0.16×) | 417 vs runtime-algebra 3,370 | −2,953 (0.12×) |
| invert, constant time | Goldilocks | 1,263 vs winter-math 1,760 | −497 (0.72×) | 858 vs winter-math 912 | −54.0 (0.94×) | 1,564 vs winter-math 1,831 | −267 (0.85×) | 1,183 vs winter-math 998 | +185 (1.19×) |
| invert, constant time | BN254 scalar | 12,015 vs crypto-bigint 29,062 | −17,047 (0.41×) | 9,244 vs crypto-bigint 25,295 | −16,051 (0.37×) | 16,857 vs crypto-bigint 36,220 | −19,363 (0.47×) | 12,578 vs crypto-bigint 30,674 | −18,096 (0.41×) |
| invert, constant time | secp256k1 base | 11,831 vs crypto-bigint 29,572 | −17,741 (0.40×) | 9,093 vs crypto-bigint 25,390 | −16,297 (0.36×) | 16,945 vs crypto-bigint 36,301 | −19,356 (0.47×) | 12,668 vs crypto-bigint 30,676 | −18,008 (0.41×) |
| invert, constant time | P-256 base | 11,533 vs crypto-bigint 29,002 | −17,469 (0.40×) | 8,967 vs k256/p256 22,033 | −13,066 (0.41×) | 16,307 vs crypto-bigint 36,230 | −19,923 (0.45×) | 12,319 vs crypto-bigint 30,742 | −18,423 (0.40×) |
| invert, constant time | P-256 scalar | 11,904 vs crypto-bigint 29,654 | −17,750 (0.40×) | 9,379 vs crypto-bigint 25,464 | −16,085 (0.37×) | 17,148 vs crypto-bigint 36,454 | −19,306 (0.47×) | 12,779 vs crypto-bigint 30,623 | −17,844 (0.42×) |
| invert, constant time | Tom-256 base | 11,773 vs crypto-bigint 29,588 | −17,815 (0.40×) | 9,286 vs crypto-bigint 25,633 | −16,347 (0.36×) | 16,992 vs crypto-bigint 36,176 | −19,184 (0.47×) | 12,767 vs crypto-bigint 30,737 | −17,970 (0.42×) |
| invert, constant time | BLS12-381 base | 23,954 vs crypto-bigint 49,528 | −25,574 (0.48×) | 16,698 vs crypto-bigint 42,668 | −25,970 (0.39×) | 32,822 vs crypto-bigint 66,505 | −33,683 (0.49×) | 24,072 vs crypto-bigint 54,035 | −29,963 (0.45×) |
| invert, constant time | P-521 base | 41,739 vs crypto-bigint 101,433 | −59,694 (0.41×) | 26,139 vs crypto-bigint 82,132 | −55,993 (0.32×) | 63,686 vs crypto-bigint 123,744 | −60,058 (0.51×) | 41,552 vs crypto-bigint 100,426 | −58,874 (0.41×) |
| sqrt, constant time | Mersenne-31 | 334 vs runtime-algebra 1,740 | −1,406 (0.19×) | 334 vs crypto-bigint 1,647 | −1,313 (0.20×) | 341 vs runtime-algebra 1,748 | −1,407 (0.20×) | 340 vs runtime-algebra 1,679 | −1,339 (0.20×) |
| sqrt, constant time | BabyBear | 7,468 vs crypto-bigint 34,215 | −26,747 (0.22×) | 6,464 vs crypto-bigint 20,220 | −13,756 (0.32×) | 7,850 vs crypto-bigint 38,485 | −30,635 (0.20×) | 6,531 vs crypto-bigint 22,801 | −16,270 (0.29×) |
| sqrt, constant time | Goldilocks | 12,879 vs crypto-bigint 48,300 | −35,421 (0.27×) | 8,921 vs crypto-bigint 28,889 | −19,968 (0.31×) | 15,858 vs crypto-bigint 54,744 | −38,886 (0.29×) | 21,585 vs crypto-bigint 32,626 | −11,041 (0.66×) |
| sqrt, constant time | BN254 scalar | 178,697 vs crypto-bigint 328,345 | −149,648 (0.54×) | 83,897 vs halo2curves 102,173 | −18,276 (0.82×) | 297,160 vs halo2curves 530,608 | −233,448 (0.56×) | 152,719 vs halo2curves 180,044 | −27,325 (0.85×) |
| sqrt, constant time | secp256k1 base | 42,420 vs crypto-bigint 159,037 | −116,617 (0.27×) | 19,829 vs halo2curves 43,341 | −23,512 (0.46×) | 70,690 vs crypto-bigint 259,339 | −188,649 (0.27×) | 31,261 vs crypto-bigint 84,300 | −53,039 (0.37×) |
| sqrt, constant time | P-256 base | 44,657 vs k256/p256 64,688 | −20,031 (0.69×) | 19,094 vs k256/p256 21,666 | −2,572 (0.88×) | 79,907 vs k256/p256 126,183 | −46,276 (0.63×) | 32,459 vs k256/p256 33,665 | −1,206 (0.96×) |
| sqrt, constant time | P-256 scalar | 78,912 vs halo2curves 153,248 | −74,336 (0.51×) | 37,108 vs halo2curves 49,988 | −12,880 (0.74×) | 121,136 vs crypto-bigint 250,825 | −129,689 (0.48×) | 68,607 vs halo2curves 72,417 | −3,810 (0.95×) |
| sqrt, constant time | Tom-256 base | 65,867 vs crypto-bigint 148,468 | −82,601 (0.44×) | 19,373 vs halo2curves 45,267 | −25,894 (0.43×) | 94,906 vs crypto-bigint 232,503 | −137,597 (0.41×) | 50,847 vs crypto-bigint 77,789 | −26,942 (0.65×) |
| sqrt, constant time | BLS12-381 base | 314,938 vs crypto-bigint 542,867 | −227,929 (0.58×) | 97,430 vs crypto-bigint 145,031 | −47,601 (0.67×) | 423,132 vs crypto-bigint 623,651 | −200,519 (0.68×) | 214,113 vs crypto-bigint 267,711 | −53,598 (0.80×) |
| sqrt, constant time | P-521 base | 324,637 vs crypto-bigint 1,524,172 | −1,199,535 (0.21×) | 129,843 vs crypto-bigint 296,736 | −166,893 (0.44×) | 507,844 vs crypto-bigint 1,560,230 | −1,052,386 (0.33×) | 266,374 vs crypto-bigint 590,262 | −323,888 (0.45×) |
| mul_u64, constant time | Mersenne-31 | 9.4 vs crypto-bigint 61.4 | −52.0 (0.15×) | 9.4 vs crypto-bigint 30.6 | −21.2 (0.31×) | 9.8 vs crypto-bigint 74.4 | −64.6 (0.13×) | 9.8 vs crypto-bigint 38.9 | −29.1 (0.25×) |
| mul_u64, constant time | BabyBear | 17.1 vs crypto-bigint 64.0 | −46.9 (0.27×) | 11.2 vs crypto-bigint 33.1 | −21.9 (0.34×) | 20.0 vs crypto-bigint 86.0 | −66.0 (0.23×) | 15.0 vs crypto-bigint 40.4 | −25.4 (0.37×) |
| mul_u64, constant time | Goldilocks | 19.0 vs winter-math 25.0 | −6.0 (0.76×) | 11.0 vs winter-math 10.9 | +0.1 (1.01×) | 21.1 vs winter-math 31.1 | −10.0 (0.68×) | 16.7 vs winter-math 13.6 | +3.1 (1.23×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 866 | −760 (0.12×) | 38.9 vs crypto-bigint 263 | −224 (0.15×) | 125 vs halo2curves 1,417 | −1,292 (0.09×) | 57.3 vs crypto-bigint 407 | −350 (0.14×) |
| mul_u64, constant time | secp256k1 base | 61.6 vs halo2curves 689 | −627 (0.09×) | 24.9 vs crypto-bigint 263 | −238 (0.09×) | 80.1 vs halo2curves 1,061 | −981 (0.08×) | 37.3 vs crypto-bigint 408 | −371 (0.09×) |
| mul_u64, constant time | P-256 base | 85.5 vs k256/p256 485 | −400 (0.18×) | 40.3 vs k256/p256 150 | −110 (0.27×) | 103 vs k256/p256 957 | −854 (0.11×) | 57.2 vs k256/p256 258 | −201 (0.22×) |
| mul_u64, constant time | P-256 scalar | 102 vs k256/p256 516 | −414 (0.20×) | 43.6 vs k256/p256 253 | −209 (0.17×) | 122 vs k256/p256 816 | −694 (0.15×) | 62.7 vs k256/p256 390 | −327 (0.16×) |
| mul_u64, constant time | Tom-256 base | 95.3 vs crypto-bigint 868 | −773 (0.11×) | 41.8 vs crypto-bigint 263 | −221 (0.16×) | 123 vs halo2curves 1,283 | −1,160 (0.10×) | 63.4 vs crypto-bigint 406 | −343 (0.16×) |
| mul_u64, constant time | BLS12-381 base | 149 vs crypto-bigint 1,931 | −1,782 (0.08×) | 65.7 vs halo2curves 483 | −417 (0.14×) | 160 vs crypto-bigint 2,535 | −2,375 (0.06×) | 85.9 vs crypto-bigint 1,020 | −934 (0.08×) |
| mul_u64, constant time | P-521 base | 138 vs crypto-bigint 3,038 | −2,900 (0.05×) | 69.2 vs crypto-bigint 991 | −922 (0.07×) | 138 vs crypto-bigint 3,974 | −3,836 (0.03×) | 96.0 vs crypto-bigint 1,831 | −1,735 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.4 vs crypto-bigint 61.9 | −52.5 (0.15×) | 9.4 vs crypto-bigint 30.7 | −21.3 (0.31×) | 9.7 vs crypto-bigint 72.8 | −63.1 (0.13×) | 9.8 vs crypto-bigint 38.8 | −29.0 (0.25×) |
| mul_uint, constant time | BabyBear | 17.1 vs crypto-bigint 63.8 | −46.7 (0.27×) | 11.2 vs crypto-bigint 33.1 | −21.9 (0.34×) | 20.1 vs crypto-bigint 85.1 | −65.0 (0.24×) | 15.0 vs crypto-bigint 40.6 | −25.6 (0.37×) |
| mul_uint, constant time | Goldilocks | 19.1 vs winter-math 25.0 | −5.9 (0.76×) | 11.0 vs winter-math 10.9 | +0.1 (1.01×) | 21.1 vs winter-math 31.0 | −9.9 (0.68×) | 16.8 vs winter-math 13.6 | +3.2 (1.24×) |
| mul_uint, constant time | BN254 scalar | 484 vs halo2curves 769 | −285 (0.63×) | 173 vs halo2curves 180 | −7.0 (0.96×) | 599 vs halo2curves 1,348 | −749 (0.44×) | 312 vs halo2curves 349 | −37.0 (0.89×) |
| mul_uint, constant time | secp256k1 base | 322 vs halo2curves 519 | −197 (0.62×) | 192 vs halo2curves 154 | +38.0 (1.25×) | 381 vs halo2curves 959 | −578 (0.40×) | 251 vs halo2curves 266 | −15.0 (0.94×) |
| mul_uint, constant time | P-256 base | 413 vs halo2curves 458 | −45.0 (0.90×) | 136 vs halo2curves 143 | −7.0 (0.95×) | 492 vs halo2curves 954 | −462 (0.52×) | 233 vs halo2curves 252 | −19.0 (0.92×) |
| mul_uint, constant time | P-256 scalar | 473 vs k256/p256 547 | −74.0 (0.86×) | 186 vs halo2curves 186 | +0.0 (1.00×) | 562 vs k256/p256 882 | −320 (0.64×) | 304 vs crypto-bigint 409 | −105 (0.74×) |
| mul_uint, constant time | Tom-256 base | 467 vs halo2curves 695 | −228 (0.67×) | 163 vs halo2curves 180 | −17.0 (0.91×) | 553 vs halo2curves 1,176 | −623 (0.47×) | 293 vs halo2curves 335 | −42.0 (0.87×) |
| mul_uint, constant time | BLS12-381 base | 1,066 vs halo2curves 1,940 | −874 (0.55×) | 369 vs halo2curves 381 | −12.0 (0.97×) | 1,678 vs crypto-bigint 2,519 | −841 (0.67×) | 918 vs crypto-bigint 982 | −64.0 (0.93×) |
| mul_uint, constant time | P-521 base | 1,989 vs crypto-bigint 3,030 | −1,041 (0.66×) | 975 vs crypto-bigint 990 | −15.0 (0.98×) | 2,921 vs crypto-bigint 4,016 | −1,095 (0.73×) | 1,477 vs crypto-bigint 1,835 | −358 (0.80×) |
| add_u64, constant time | Mersenne-31 | 8.2 vs crypto-bigint 36.1 | −27.9 (0.23×) | 8.1 vs crypto-bigint 30.4 | −22.3 (0.27×) | 9.1 vs crypto-bigint 42.2 | −33.1 (0.22×) | 9.2 vs crypto-bigint 30.4 | −21.2 (0.30×) |
| add_u64, constant time | BabyBear | 10.9 vs crypto-bigint 39.9 | −29.0 (0.27×) | 8.6 vs crypto-bigint 31.6 | −23.0 (0.27×) | 13.0 vs crypto-bigint 53.6 | −40.6 (0.24×) | 12.6 vs crypto-bigint 31.9 | −19.3 (0.39×) |
| add_u64, constant time | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 11.0 | +1.0 (1.09×) |
| add_u64, constant time | BN254 scalar | 136 vs crypto-bigint 504 | −368 (0.27×) | 78.3 vs halo2curves 210 | −132 (0.37×) | 163 vs halo2curves 758 | −595 (0.22×) | 81.4 vs halo2curves 295 | −214 (0.28×) |
| add_u64, constant time | secp256k1 base | 20.5 vs halo2curves 380 | −360 (0.05×) | 16.1 vs halo2curves 203 | −187 (0.08×) | 19.5 vs halo2curves 488 | −469 (0.04×) | 25.1 vs halo2curves 243 | −218 (0.10×) |
| add_u64, constant time | P-256 base | 90.4 vs k256/p256 276 | −186 (0.33×) | 57.7 vs k256/p256 95.2 | −37.5 (0.61×) | 103 vs k256/p256 510 | −407 (0.20×) | 73.0 vs k256/p256 160 | −87.0 (0.46×) |
| add_u64, constant time | P-256 scalar | 90.2 vs k256/p256 41.3 | +48.9 (2.18×) | 56.4 vs k256/p256 41.3 | +15.1 (1.37×) | 100 vs k256/p256 54.6 | +45.4 (1.83×) | 93.4 vs k256/p256 55.0 | +38.4 (1.70×) |
| add_u64, constant time | Tom-256 base | 116 vs crypto-bigint 504 | −388 (0.23×) | 61.5 vs halo2curves 218 | −157 (0.28×) | 132 vs halo2curves 686 | −554 (0.19×) | 93.8 vs crypto-bigint 316 | −222 (0.30×) |
| add_u64, constant time | BLS12-381 base | 211 vs crypto-bigint 1,075 | −864 (0.20×) | 92.1 vs halo2curves 324 | −232 (0.28×) | 268 vs crypto-bigint 1,387 | −1,119 (0.19×) | 125 vs crypto-bigint 650 | −525 (0.19×) |
| add_u64, constant time | P-521 base | 64.3 vs crypto-bigint 1,688 | −1,624 (0.04×) | 44.3 vs crypto-bigint 651 | −607 (0.07×) | 65.7 vs crypto-bigint 2,197 | −2,131 (0.03×) | 64.1 vs crypto-bigint 1,117 | −1,053 (0.06×) |
| add_uint, constant time | Mersenne-31 | 8.1 vs crypto-bigint 36.0 | −27.9 (0.22×) | 8.2 vs crypto-bigint 30.4 | −22.2 (0.27×) | 9.1 vs crypto-bigint 41.5 | −32.4 (0.22×) | 9.1 vs crypto-bigint 30.6 | −21.5 (0.30×) |
| add_uint, constant time | BabyBear | 10.9 vs crypto-bigint 39.9 | −29.0 (0.27×) | 8.6 vs crypto-bigint 31.7 | −23.1 (0.27×) | 13.0 vs crypto-bigint 53.7 | −40.7 (0.24×) | 12.6 vs crypto-bigint 31.6 | −19.0 (0.40×) |
| add_uint, constant time | Goldilocks | 9.9 vs winter-math 13.3 | −3.4 (0.74×) | 7.7 vs winter-math 7.4 | +0.3 (1.04×) | 11.1 vs winter-math 19.1 | −8.0 (0.58×) | 12.0 vs winter-math 11.0 | +1.0 (1.09×) |
| add_uint, constant time | BN254 scalar | 249 vs halo2curves 415 | −166 (0.60×) | 99.1 vs halo2curves 112 | −12.9 (0.88×) | 344 vs halo2curves 678 | −334 (0.51×) | 182 vs halo2curves 205 | −23.0 (0.89×) |
| add_uint, constant time | secp256k1 base | 41.7 vs halo2curves 224 | −182 (0.19×) | 29.4 vs halo2curves 84.9 | −55.5 (0.35×) | 37.6 vs halo2curves 415 | −377 (0.09×) | 44.5 vs halo2curves 140 | −95.5 (0.32×) |
| add_uint, constant time | P-256 base | 177 vs halo2curves 248 | −71.0 (0.71×) | 98.7 vs halo2curves 91.5 | +7.2 (1.08×) | 286 vs halo2curves 509 | −223 (0.56×) | 150 vs halo2curves 154 | −4.0 (0.97×) |
| add_uint, constant time | P-256 scalar | 256 vs k256/p256 94.7 | +161 (2.70×) | 106 vs k256/p256 94.9 | +11.1 (1.12×) | 316 vs k256/p256 118 | +198 (2.68×) | 202 vs k256/p256 119 | +83.0 (1.70×) |
| add_uint, constant time | Tom-256 base | 251 vs halo2curves 380 | −129 (0.66×) | 102 vs halo2curves 110 | −8.0 (0.93×) | 329 vs halo2curves 629 | −300 (0.52×) | 187 vs halo2curves 196 | −9.0 (0.95×) |
| add_uint, constant time | BLS12-381 base | 770 vs halo2curves 974 | −204 (0.79×) | 209 vs halo2curves 225 | −16.0 (0.93×) | 949 vs crypto-bigint 1,395 | −446 (0.68×) | 411 vs halo2curves 562 | −151 (0.73×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,676 | −1,538 (0.08×) | 105 vs crypto-bigint 648 | −543 (0.16×) | 155 vs crypto-bigint 2,227 | −2,072 (0.07×) | 138 vs crypto-bigint 1,127 | −989 (0.12×) |
| sub_u64, constant time | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 17.9 | −11.7 (0.35×) | 7.0 vs crypto-bigint 31.3 | −24.3 (0.22×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_u64, constant time | BabyBear | 10.8 vs crypto-bigint 27.9 | −17.1 (0.39×) | 8.4 vs crypto-bigint 19.5 | −11.1 (0.43×) | 13.0 vs crypto-bigint 44.5 | −31.5 (0.29×) | 12.5 vs crypto-bigint 21.9 | −9.4 (0.57×) |
| sub_u64, constant time | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 11.0 vs winter-math 9.6 | +1.4 (1.15×) |
| sub_u64, constant time | BN254 scalar | 126 vs crypto-bigint 509 | −383 (0.25×) | 60.0 vs crypto-bigint 151 | −91.0 (0.40×) | 155 vs halo2curves 763 | −608 (0.20×) | 80.0 vs crypto-bigint 279 | −199 (0.29×) |
| sub_u64, constant time | secp256k1 base | 15.5 vs halo2curves 381 | −366 (0.04×) | 12.3 vs crypto-bigint 150 | −138 (0.08×) | 14.6 vs halo2curves 474 | −459 (0.03×) | 18.5 vs halo2curves 238 | −220 (0.08×) |
| sub_u64, constant time | P-256 base | 84.8 vs k256/p256 266 | −181 (0.32×) | 51.9 vs k256/p256 89.1 | −37.2 (0.58×) | 97.8 vs k256/p256 507 | −409 (0.19×) | 67.7 vs k256/p256 149 | −81.3 (0.45×) |
| sub_u64, constant time | P-256 scalar | 80.6 vs k256/p256 21.2 | +59.4 (3.80×) | 52.7 vs k256/p256 21.2 | +31.5 (2.49×) | 93.6 vs k256/p256 24.4 | +69.2 (3.84×) | 79.2 vs k256/p256 24.3 | +54.9 (3.26×) |
| sub_u64, constant time | Tom-256 base | 113 vs crypto-bigint 506 | −393 (0.22×) | 58.9 vs crypto-bigint 152 | −93.1 (0.39×) | 124 vs halo2curves 665 | −541 (0.19×) | 80.1 vs crypto-bigint 259 | −179 (0.31×) |
| sub_u64, constant time | BLS12-381 base | 187 vs crypto-bigint 1,090 | −903 (0.17×) | 87.6 vs halo2curves 317 | −229 (0.28×) | 250 vs crypto-bigint 1,383 | −1,133 (0.18×) | 118 vs crypto-bigint 618 | −500 (0.19×) |
| sub_u64, constant time | P-521 base | 42.4 vs crypto-bigint 1,674 | −1,632 (0.03×) | 34.4 vs crypto-bigint 623 | −589 (0.06×) | 52.4 vs crypto-bigint 2,184 | −2,132 (0.02×) | 44.5 vs crypto-bigint 1,124 | −1,080 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 6.2 vs crypto-bigint 23.4 | −17.2 (0.26×) | 6.2 vs crypto-bigint 18.2 | −12.0 (0.34×) | 7.0 vs crypto-bigint 31.3 | −24.3 (0.22×) | 7.0 vs crypto-bigint 20.9 | −13.9 (0.33×) |
| sub_uint, constant time | BabyBear | 10.9 vs crypto-bigint 27.9 | −17.0 (0.39×) | 8.5 vs crypto-bigint 19.6 | −11.1 (0.43×) | 13.0 vs crypto-bigint 43.6 | −30.6 (0.30×) | 12.5 vs crypto-bigint 21.8 | −9.3 (0.57×) |
| sub_uint, constant time | Goldilocks | 9.9 vs winter-math 11.5 | −1.6 (0.86×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 11.0 vs winter-math 9.5 | +1.5 (1.16×) |
| sub_uint, constant time | BN254 scalar | 262 vs halo2curves 403 | −141 (0.65×) | 101 vs halo2curves 103 | −2.0 (0.98×) | 347 vs halo2curves 684 | −337 (0.51×) | 188 vs halo2curves 194 | −6.0 (0.97×) |
| sub_uint, constant time | secp256k1 base | 33.5 vs halo2curves 209 | −176 (0.16×) | 26.1 vs halo2curves 110 | −83.9 (0.24×) | 34.4 vs halo2curves 409 | −375 (0.08×) | 38.5 vs halo2curves 130 | −91.5 (0.30×) |
| sub_uint, constant time | P-256 base | 178 vs halo2curves 233 | −55.0 (0.76×) | 86.4 vs halo2curves 85.3 | +1.1 (1.01×) | 275 vs halo2curves 494 | −219 (0.56×) | 139 vs halo2curves 143 | −4.0 (0.97×) |
| sub_uint, constant time | P-256 scalar | 250 vs k256/p256 64.8 | +185 (3.86×) | 106 vs k256/p256 65.0 | +41.0 (1.63×) | 328 vs k256/p256 89.0 | +239 (3.69×) | 184 vs k256/p256 88.9 | +95.1 (2.07×) |
| sub_uint, constant time | Tom-256 base | 253 vs halo2curves 365 | −112 (0.69×) | 101 vs halo2curves 103 | −2.0 (0.98×) | 317 vs halo2curves 617 | −300 (0.51×) | 188 vs halo2curves 187 | +1.0 (1.01×) |
| sub_uint, constant time | BLS12-381 base | 762 vs halo2curves 962 | −200 (0.79×) | 207 vs halo2curves 215 | −8.0 (0.96×) | 942 vs crypto-bigint 1,373 | −431 (0.69×) | 375 vs halo2curves 408 | −33.0 (0.92×) |
| sub_uint, constant time | P-521 base | 127 vs crypto-bigint 1,674 | −1,547 (0.08×) | 86.8 vs crypto-bigint 621 | −534 (0.14×) | 126 vs crypto-bigint 2,196 | −2,070 (0.06×) | 113 vs crypto-bigint 1,104 | −991 (0.10×) |
| from_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 18.0 | −13.3 (0.26×) | 4.7 vs crypto-bigint 11.6 | −6.9 (0.41×) | 5.2 vs crypto-bigint 25.3 | −20.1 (0.21×) | 5.3 vs crypto-bigint 13.1 | −7.8 (0.40×) |
| from_u64, constant time | BabyBear | 13.5 vs crypto-bigint 21.6 | −8.1 (0.62×) | 10.3 vs crypto-bigint 12.7 | −2.4 (0.81×) | 14.0 vs crypto-bigint 36.1 | −22.1 (0.39×) | 12.7 vs crypto-bigint 15.5 | −2.8 (0.82×) |
| from_u64, constant time | Goldilocks | 15.0 vs winter-math 10.6 | +4.4 (1.42×) | 9.3 vs winter-math 6.0 | +3.3 (1.55×) | 17.4 vs winter-math 17.0 | +0.4 (1.02×) | 12.5 vs winter-math 7.0 | +5.5 (1.79×) |
| from_u64, constant time | BN254 scalar | 201 vs crypto-bigint 433 | −232 (0.46×) | 80.9 vs crypto-bigint 133 | −52.1 (0.61×) | 275 vs halo2curves 708 | −433 (0.39×) | 119 vs crypto-bigint 205 | −86.0 (0.58×) |
| from_u64, constant time | secp256k1 base | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) |
| from_u64, constant time | P-256 base | 178 vs k256/p256 239 | −61.0 (0.74×) | 50.0 vs k256/p256 76.2 | −26.2 (0.66×) | 210 vs k256/p256 480 | −270 (0.44×) | 84.1 vs k256/p256 130 | −45.9 (0.65×) |
| from_u64, constant time | P-256 scalar | 255 vs k256/p256 3.8 | +251 (67.11×) | 81.8 vs k256/p256 3.8 | +78.0 (21.53×) | 268 vs k256/p256 3.7 | +264 (72.43×) | 118 vs k256/p256 3.8 | +114 (31.05×) |
| from_u64, constant time | Tom-256 base | 253 vs crypto-bigint 428 | −175 (0.59×) | 79.1 vs crypto-bigint 133 | −53.9 (0.59×) | 271 vs halo2curves 667 | −396 (0.41×) | 114 vs crypto-bigint 205 | −91.0 (0.56×) |
| from_u64, constant time | BLS12-381 base | 431 vs crypto-bigint 908 | −477 (0.47×) | 155 vs crypto-bigint 252 | −97.0 (0.62×) | 607 vs crypto-bigint 1,247 | −640 (0.49×) | 273 vs crypto-bigint 479 | −206 (0.57×) |
| from_u64, constant time | P-521 base | 6.1 vs crypto-bigint 1,448 | −1,442 (0.00×) | 6.0 vs crypto-bigint 496 | −490 (0.01×) | 6.1 vs crypto-bigint 1,995 | −1,989 (0.00×) | 6.0 vs crypto-bigint 850 | −844 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,376 vs crypto-bigint 28,754 | −16,378 (0.43×) | 9,508 vs crypto-bigint 24,345 | −14,837 (0.39×) | 17,577 vs crypto-bigint 37,623 | −20,046 (0.47×) | 12,966 vs crypto-bigint 30,669 | −17,703 (0.42×) |
| div_u64, constant time | secp256k1 base | 11,879 vs crypto-bigint 28,725 | −16,846 (0.41×) | 9,055 vs crypto-bigint 24,391 | −15,336 (0.37×) | 17,081 vs crypto-bigint 37,254 | −20,173 (0.46×) | 12,739 vs crypto-bigint 30,808 | −18,069 (0.41×) |
| div_u64, constant time | P-256 base | 11,796 vs crypto-bigint 28,746 | −16,950 (0.41×) | 9,146 vs k256/p256 22,874 | −13,728 (0.40×) | 17,486 vs crypto-bigint 37,818 | −20,332 (0.46×) | 12,560 vs crypto-bigint 30,710 | −18,150 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,320 vs crypto-bigint 28,624 | −16,304 (0.43×) | 9,427 vs crypto-bigint 24,225 | −14,798 (0.39×) | 17,554 vs crypto-bigint 37,322 | −19,768 (0.47×) | 13,160 vs crypto-bigint 30,804 | −17,644 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,206 vs crypto-bigint 28,652 | −16,446 (0.43×) | 9,479 vs crypto-bigint 24,316 | −14,837 (0.39×) | 16,908 vs crypto-bigint 37,719 | −20,811 (0.45×) | 12,938 vs crypto-bigint 30,720 | −17,782 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,806 vs crypto-bigint 51,455 | −26,649 (0.48×) | 17,035 vs crypto-bigint 43,435 | −26,400 (0.39×) | 35,168 vs crypto-bigint 68,726 | −33,558 (0.51×) | 24,433 vs crypto-bigint 53,670 | −29,237 (0.46×) |
| div_u64, constant time | P-521 base | 42,369 vs crypto-bigint 105,869 | −63,500 (0.40×) | 25,993 vs crypto-bigint 83,064 | −57,071 (0.31×) | 64,051 vs crypto-bigint 127,692 | −63,641 (0.50×) | 42,942 vs crypto-bigint 100,983 | −58,041 (0.43×) |
| div_uint, constant time | Mersenne-31 | 455 vs runtime-algebra 3,501 | −3,046 (0.13×) | 455 vs runtime-algebra 3,505 | −3,050 (0.13×) | 469 vs runtime-algebra 3,423 | −2,954 (0.14×) | 465 vs runtime-algebra 3,418 | −2,953 (0.14×) |
| div_uint, constant time | BabyBear | 546 vs runtime-algebra 3,560 | −3,014 (0.15×) | 466 vs runtime-algebra 3,558 | −3,092 (0.13×) | 612 vs runtime-algebra 3,421 | −2,809 (0.18×) | 460 vs runtime-algebra 3,420 | −2,960 (0.13×) |
| div_uint, constant time | Goldilocks | 1,308 vs winter-math 1,800 | −492 (0.73×) | 950 vs winter-math 948 | +2.0 (1.00×) | 1,576 vs winter-math 1,902 | −326 (0.83×) | 1,211 vs winter-math 1,045 | +166 (1.16×) |
| div_uint, constant time | BN254 scalar | 12,579 vs crypto-bigint 28,689 | −16,110 (0.44×) | 9,691 vs crypto-bigint 24,283 | −14,592 (0.40×) | 17,813 vs crypto-bigint 37,914 | −20,101 (0.47×) | 13,321 vs crypto-bigint 30,779 | −17,458 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,955 vs crypto-bigint 28,138 | −16,183 (0.42×) | 9,092 vs crypto-bigint 24,270 | −15,178 (0.37×) | 17,052 vs crypto-bigint 37,604 | −20,552 (0.45×) | 12,806 vs crypto-bigint 30,636 | −17,830 (0.42×) |
| div_uint, constant time | P-256 base | 12,024 vs crypto-bigint 29,068 | −17,044 (0.41×) | 9,118 vs k256/p256 23,746 | −14,628 (0.38×) | 17,545 vs crypto-bigint 37,452 | −19,907 (0.47×) | 12,681 vs crypto-bigint 30,857 | −18,176 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,378 vs crypto-bigint 28,892 | −16,514 (0.43×) | 9,541 vs crypto-bigint 24,219 | −14,678 (0.39×) | 17,746 vs crypto-bigint 37,474 | −19,728 (0.47×) | 13,496 vs crypto-bigint 30,742 | −17,246 (0.44×) |
| div_uint, constant time | Tom-256 base | 12,457 vs crypto-bigint 28,763 | −16,306 (0.43×) | 9,428 vs crypto-bigint 24,174 | −14,746 (0.39×) | 17,276 vs crypto-bigint 37,817 | −20,541 (0.46×) | 13,089 vs crypto-bigint 30,871 | −17,782 (0.42×) |
| div_uint, constant time | BLS12-381 base | 25,374 vs crypto-bigint 51,902 | −26,528 (0.49×) | 17,063 vs crypto-bigint 43,276 | −26,213 (0.39×) | 35,743 vs crypto-bigint 68,918 | −33,175 (0.52×) | 24,870 vs crypto-bigint 54,164 | −29,294 (0.46×) |
| div_uint, constant time | P-521 base | 42,567 vs crypto-bigint 106,524 | −63,957 (0.40×) | 26,008 vs crypto-bigint 83,713 | −57,705 (0.31×) | 65,187 vs crypto-bigint 127,977 | −62,790 (0.51×) | 43,336 vs crypto-bigint 100,960 | −57,624 (0.43×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) |
| add | BN254 scalar | 14.6 vs ark-ff 14.0 | +0.6 (1.04×) | 10.9 vs ark-ff 11.1 | −0.2 (0.98×) | 15.6 vs ark-ff 13.6 | +2.0 (1.15×) | 17.4 vs ark-ff 16.4 | +1.0 (1.06×) |
| add | secp256k1 base | 13.7 vs ark-ff 15.5 | −1.8 (0.88×) | 10.7 vs ark-ff 11.2 | −0.5 (0.96×) | 13.3 vs ark-ff 15.0 | −1.7 (0.89×) | 18.1 vs ark-ff 17.8 | +0.3 (1.02×) |
| add | P-256 base | 12.9 vs ark-ff 15.1 | −2.2 (0.85×) | 11.2 vs ark-ff 11.2 | +0.0 (1.00×) | 13.8 vs ark-ff 15.6 | −1.8 (0.88×) | 17.9 vs ark-ff 19.4 | −1.5 (0.92×) |
| add | P-256 scalar | 14.4 vs ark-ff 15.3 | −0.9 (0.94×) | 11.1 vs ark-ff 11.3 | −0.2 (0.98×) | 15.4 vs ark-ff 15.8 | −0.4 (0.97×) | 18.9 vs ark-ff 17.9 | +1.0 (1.06×) |
| add | Tom-256 base | 14.3 vs halo2curves 42.9 | −28.6 (0.33×) | 11.2 vs halo2curves 24.0 | −12.8 (0.47×) | 15.0 vs halo2curves 42.5 | −27.5 (0.35×) | 18.4 vs halo2curves 32.5 | −14.1 (0.57×) |
| add | BLS12-381 base | 24.9 vs ark-ff 43.9 | −19.0 (0.57×) | 18.7 vs halo2curves 34.1 | −15.4 (0.55×) | 24.7 vs ark-ff 47.5 | −22.8 (0.52×) | 29.8 vs ark-ff 46.1 | −16.3 (0.65×) |
| add | P-521 base | 42.4 vs crypto-bigint 181 | −139 (0.23×) | 27.7 vs crypto-bigint 179 | −151 (0.15×) | 47.5 vs crypto-bigint 244 | −197 (0.19×) | 47.5 vs crypto-bigint 245 | −198 (0.19×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 14.5 vs ark-ff 16.5 | −2.0 (0.88×) | 10.7 vs ark-ff 14.0 | −3.3 (0.76×) | 13.8 vs ark-ff 16.7 | −2.9 (0.83×) | 18.5 vs ark-ff 21.5 | −3.0 (0.86×) |
| sub | secp256k1 base | 16.4 vs ark-ff 16.6 | −0.2 (0.99×) | 10.7 vs ark-ff 14.0 | −3.3 (0.76×) | 13.1 vs ark-ff 17.1 | −4.0 (0.77×) | 17.8 vs halo2curves 21.0 | −3.2 (0.85×) |
| sub | P-256 base | 14.2 vs ark-ff 15.6 | −1.4 (0.91×) | 10.4 vs ark-ff 12.9 | −2.5 (0.81×) | 12.9 vs ark-ff 16.0 | −3.1 (0.81×) | 17.4 vs k256/p256 20.0 | −2.6 (0.87×) |
| sub | P-256 scalar | 15.3 vs ark-ff 16.6 | −1.3 (0.92×) | 10.7 vs ark-ff 13.9 | −3.2 (0.77×) | 13.7 vs ark-ff 16.6 | −2.9 (0.83×) | 17.9 vs ark-ff 21.3 | −3.4 (0.84×) |
| sub | Tom-256 base | 15.0 vs halo2curves 26.0 | −11.0 (0.58×) | 10.8 vs halo2curves 14.7 | −3.9 (0.73×) | 14.0 vs halo2curves 26.5 | −12.5 (0.53×) | 18.5 vs halo2curves 22.1 | −3.6 (0.84×) |
| sub | BLS12-381 base | 25.3 vs ark-ff 31.8 | −6.5 (0.80×) | 17.5 vs ark-ff 23.6 | −6.1 (0.74×) | 24.4 vs ark-ff 30.8 | −6.4 (0.79×) | 30.6 vs halo2curves 36.1 | −5.5 (0.85×) |
| sub | P-521 base | 44.9 vs crypto-bigint 159 | −114 (0.28×) | 26.8 vs crypto-bigint 144 | −117 (0.19×) | 43.3 vs crypto-bigint 194 | −151 (0.22×) | 45.5 vs crypto-bigint 225 | −180 (0.20×) |
| mul | Mersenne-31 | 12.0 vs crypto-bigint 46.7 | −34.7 (0.26×) | 12.0 vs crypto-bigint 27.0 | −15.0 (0.44×) | 12.0 vs crypto-bigint 50.8 | −38.8 (0.24×) | 12.0 vs crypto-bigint 30.9 | −18.9 (0.39×) |
| mul | BabyBear | 13.0 vs plonky3 12.2 | +0.8 (1.07×) | 11.0 vs plonky3 12.1 | −1.1 (0.91×) | 14.5 vs plonky3 13.0 | +1.5 (1.12×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| mul | Goldilocks | 18.0 vs winter-math 18.0 | +0.0 (1.00×) | 14.4 vs winter-math 14.5 | −0.1 (0.99×) | 21.0 vs winter-math 19.0 | +2.0 (1.11×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul | BN254 scalar | 227 vs ark-ff 317 | −90.0 (0.72×) | 90.9 vs halo2curves 95.1 | −4.2 (0.96×) | 368 vs ark-ff 423 | −55.0 (0.87×) | 152 vs halo2curves 178 | −26.0 (0.85×) |
| mul | secp256k1 base | 173 vs ark-ff 276 | −103 (0.63×) | 77.6 vs halo2curves 90.2 | −12.6 (0.86×) | 250 vs ark-ff 400 | −150 (0.62×) | 106 vs halo2curves 159 | −53.0 (0.67×) |
| mul | P-256 base | 190 vs k256/p256 243 | −53.0 (0.78×) | 72.3 vs halo2curves 74.9 | −2.6 (0.97×) | 315 vs ark-ff 449 | −134 (0.70×) | 121 vs halo2curves 130 | −9.0 (0.93×) |
| mul | P-256 scalar | 239 vs ark-ff 310 | −71.0 (0.77×) | 93.4 vs halo2curves 95.9 | −2.5 (0.97×) | 325 vs ark-ff 407 | −82.0 (0.80×) | 164 vs halo2curves 176 | −12.0 (0.93×) |
| mul | Tom-256 base | 239 vs halo2curves 344 | −105 (0.69×) | 84.9 vs halo2curves 93.9 | −9.0 (0.90×) | 332 vs halo2curves 595 | −263 (0.56×) | 159 vs halo2curves 172 | −13.0 (0.92×) |
| mul | BLS12-381 base | 719 vs ark-ff 686 | +33.0 (1.05×) | 185 vs halo2curves 193 | −8.0 (0.96×) | 917 vs ark-ff 1,050 | −133 (0.87×) | 337 vs halo2curves 376 | −39.0 (0.90×) |
| mul | P-521 base | 686 vs crypto-bigint 2,137 | −1,451 (0.32×) | 247 vs crypto-bigint 619 | −372 (0.40×) | 1,292 vs crypto-bigint 2,804 | −1,512 (0.46×) | 446 vs crypto-bigint 1,099 | −653 (0.41×) |
| square | Mersenne-31 | 12.0 vs crypto-bigint 45.4 | −33.4 (0.26×) | 12.0 vs crypto-bigint 27.1 | −15.1 (0.44×) | 12.0 vs crypto-bigint 52.0 | −40.0 (0.23×) | 12.0 vs crypto-bigint 30.0 | −18.0 (0.40×) |
| square | BabyBear | 13.0 vs plonky3 12.1 | +0.9 (1.07×) | 11.0 vs plonky3 12.1 | −1.1 (0.91×) | 14.0 vs plonky3 13.0 | +1.0 (1.08×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| square | Goldilocks | 17.3 vs winter-math 19.7 | −2.4 (0.88×) | 12.0 vs winter-math 12.8 | −0.8 (0.94×) | 21.3 vs winter-math 19.0 | +2.3 (1.12×) | 14.4 vs winter-math 14.7 | −0.3 (0.98×) |
| square | BN254 scalar | 208 vs ark-ff 264 | −56.0 (0.79×) | 89.9 vs halo2curves 96.4 | −6.5 (0.93×) | 365 vs ark-ff 355 | +10.0 (1.03×) | 142 vs halo2curves 144 | −2.0 (0.99×) |
| square | secp256k1 base | 154 vs halo2curves 222 | −68.0 (0.69×) | 69.9 vs halo2curves 83.6 | −13.7 (0.84×) | 183 vs ark-ff 298 | −115 (0.61×) | 95.5 vs halo2curves 126 | −30.5 (0.76×) |
| square | P-256 base | 155 vs halo2curves 178 | −23.0 (0.87×) | 62.2 vs halo2curves 62.9 | −0.7 (0.99×) | 213 vs ark-ff 331 | −118 (0.64×) | 114 vs halo2curves 107 | +7.0 (1.07×) |
| square | P-256 scalar | 223 vs ark-ff 259 | −36.0 (0.86×) | 89.1 vs halo2curves 93.8 | −4.7 (0.95×) | 240 vs ark-ff 347 | −107 (0.69×) | 150 vs halo2curves 137 | +13.0 (1.09×) |
| square | Tom-256 base | 201 vs halo2curves 268 | −67.0 (0.75×) | 83.0 vs halo2curves 88.5 | −5.5 (0.94×) | 236 vs halo2curves 451 | −215 (0.52×) | 142 vs halo2curves 135 | +7.0 (1.05×) |
| square | BLS12-381 base | 609 vs ark-ff 619 | −10.0 (0.98×) | 179 vs halo2curves 163 | +16.0 (1.10×) | 870 vs ark-ff 838 | +32.0 (1.04×) | 294 vs halo2curves 292 | +2.0 (1.01×) |
| square | P-521 base | 571 vs crypto-bigint 2,140 | −1,569 (0.27×) | 210 vs crypto-bigint 634 | −424 (0.33×) | 913 vs crypto-bigint 2,840 | −1,927 (0.32×) | 390 vs crypto-bigint 1,139 | −749 (0.34×) |
| mul_u64 | Mersenne-31 | 14.0 vs crypto-bigint 60.6 | −46.6 (0.23×) | 14.0 vs crypto-bigint 32.2 | −18.2 (0.43×) | 14.0 vs crypto-bigint 71.3 | −57.3 (0.20×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_u64 | BabyBear | 17.0 vs plonky3 34.2 | −17.2 (0.50×) | 11.1 vs plonky3 13.3 | −2.2 (0.83×) | 19.3 vs plonky3 45.0 | −25.7 (0.43×) | 13.1 vs plonky3 16.5 | −3.4 (0.79×) |
| mul_u64 | Goldilocks | 20.4 vs winter-math 23.4 | −3.0 (0.87×) | 12.0 vs winter-math 13.2 | −1.2 (0.91×) | 25.0 vs winter-math 34.6 | −9.6 (0.72×) | 17.2 vs winter-math 14.5 | +2.7 (1.19×) |
| mul_u64 | BN254 scalar | 91.3 vs ark-ff 662 | −571 (0.14×) | 34.5 vs crypto-bigint 262 | −228 (0.13×) | 120 vs ark-ff 818 | −698 (0.15×) | 45.4 vs crypto-bigint 412 | −367 (0.11×) |
| mul_u64 | secp256k1 base | 49.0 vs ark-ff 438 | −389 (0.11×) | 16.0 vs crypto-bigint 259 | −243 (0.06×) | 65.1 vs ark-ff 609 | −544 (0.11×) | 23.0 vs crypto-bigint 410 | −387 (0.06×) |
| mul_u64 | P-256 base | 74.9 vs k256/p256 487 | −412 (0.15×) | 33.0 vs k256/p256 153 | −120 (0.22×) | 89.1 vs ark-ff 831 | −742 (0.11×) | 42.4 vs k256/p256 258 | −216 (0.16×) |
| mul_u64 | P-256 scalar | 82.2 vs k256/p256 516 | −434 (0.16×) | 34.6 vs k256/p256 255 | −220 (0.14×) | 106 vs ark-ff 816 | −710 (0.13×) | 48.1 vs k256/p256 340 | −292 (0.14×) |
| mul_u64 | Tom-256 base | 84.9 vs crypto-bigint 873 | −788 (0.10×) | 32.9 vs crypto-bigint 261 | −228 (0.13×) | 105 vs halo2curves 1,273 | −1,168 (0.08×) | 45.4 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | BLS12-381 base | 125 vs ark-ff 1,627 | −1,502 (0.08×) | 47.9 vs halo2curves 485 | −437 (0.10×) | 183 vs ark-ff 2,124 | −1,941 (0.09×) | 62.1 vs crypto-bigint 1,050 | −988 (0.06×) |
| mul_u64 | P-521 base | 95.7 vs crypto-bigint 3,015 | −2,919 (0.03×) | 39.5 vs crypto-bigint 990 | −951 (0.04×) | 191 vs crypto-bigint 4,018 | −3,827 (0.05×) | 51.8 vs crypto-bigint 1,843 | −1,791 (0.03×) |
| mul_uint | Mersenne-31 | 14.0 vs crypto-bigint 60.4 | −46.4 (0.23×) | 14.0 vs crypto-bigint 31.7 | −17.7 (0.44×) | 14.0 vs crypto-bigint 73.4 | −59.4 (0.19×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_uint | BabyBear | 17.0 vs plonky3 23.6 | −6.6 (0.72×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 19.3 vs plonky3 33.9 | −14.6 (0.57×) | 13.1 vs plonky3 16.5 | −3.4 (0.79×) |
| mul_uint | Goldilocks | 20.5 vs winter-math 23.6 | −3.1 (0.87×) | 12.1 vs winter-math 13.8 | −1.7 (0.88×) | 25.0 vs winter-math 34.5 | −9.5 (0.72×) | 17.2 vs winter-math 14.5 | +2.7 (1.19×) |
| mul_uint | BN254 scalar | 439 vs halo2curves 773 | −334 (0.57×) | 173 vs halo2curves 182 | −9.0 (0.95×) | 533 vs halo2curves 1,317 | −784 (0.40×) | 294 vs halo2curves 354 | −60.0 (0.83×) |
| mul_uint | secp256k1 base | 265 vs halo2curves 516 | −251 (0.51×) | 117 vs halo2curves 154 | −37.0 (0.76×) | 319 vs halo2curves 959 | −640 (0.33×) | 164 vs halo2curves 265 | −101 (0.62×) |
| mul_uint | P-256 base | 356 vs halo2curves 460 | −104 (0.77×) | 138 vs halo2curves 145 | −7.0 (0.95×) | 424 vs halo2curves 947 | −523 (0.45×) | 233 vs halo2curves 252 | −19.0 (0.92×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 551 | −126 (0.77×) | 178 vs halo2curves 186 | −8.0 (0.96×) | 474 vs k256/p256 863 | −389 (0.55×) | 301 vs halo2curves 350 | −49.0 (0.86×) |
| mul_uint | Tom-256 base | 411 vs halo2curves 695 | −284 (0.59×) | 172 vs halo2curves 180 | −8.0 (0.96×) | 497 vs halo2curves 1,179 | −682 (0.42×) | 324 vs halo2curves 335 | −11.0 (0.97×) |
| mul_uint | BLS12-381 base | 956 vs halo2curves 1,928 | −972 (0.50×) | 357 vs halo2curves 379 | −22.0 (0.94×) | 1,134 vs crypto-bigint 2,549 | −1,415 (0.44×) | 832 vs halo2curves 1,040 | −208 (0.80×) |
| mul_uint | P-521 base | 1,566 vs crypto-bigint 3,057 | −1,491 (0.51×) | 643 vs crypto-bigint 1,010 | −367 (0.64×) | 2,382 vs crypto-bigint 4,023 | −1,641 (0.59×) | 989 vs crypto-bigint 1,874 | −885 (0.53×) |
| add_u64 | Mersenne-31 | 8.7 vs crypto-bigint 35.4 | −26.7 (0.25×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.3 vs crypto-bigint 43.6 | −34.3 (0.21×) | 9.3 vs crypto-bigint 30.6 | −21.3 (0.30×) |
| add_u64 | BabyBear | 12.2 vs plonky3 30.7 | −18.5 (0.40×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 41.7 | −29.0 (0.30×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_u64 | Goldilocks | 9.1 vs winter-math 13.1 | −4.0 (0.69×) | 7.3 vs winter-math 7.6 | −0.3 (0.96×) | 10.8 vs winter-math 19.1 | −8.3 (0.57×) | 10.0 vs winter-math 9.1 | +0.9 (1.10×) |
| add_u64 | BN254 scalar | 117 vs ark-ff 341 | −224 (0.34×) | 44.3 vs halo2curves 213 | −169 (0.21×) | 142 vs ark-ff 403 | −261 (0.35×) | 65.5 vs halo2curves 298 | −233 (0.22×) |
| add_u64 | secp256k1 base | 7.0 vs ark-ff 157 | −150 (0.04×) | 6.0 vs ark-ff 118 | −112 (0.05×) | 7.0 vs ark-ff 222 | −215 (0.03×) | 10.0 vs ark-ff 183 | −173 (0.05×) |
| add_u64 | P-256 base | 68.6 vs ark-ff 267 | −198 (0.26×) | 34.9 vs k256/p256 92.6 | −57.7 (0.38×) | 77.7 vs ark-ff 400 | −322 (0.19×) | 55.7 vs k256/p256 158 | −102 (0.35×) |
| add_u64 | P-256 scalar | 69.3 vs k256/p256 56.0 | +13.3 (1.24×) | 38.4 vs k256/p256 56.2 | −17.8 (0.68×) | 72.9 vs k256/p256 61.9 | +11.0 (1.18×) | 65.1 vs k256/p256 62.9 | +2.2 (1.03×) |
| add_u64 | Tom-256 base | 88.0 vs crypto-bigint 511 | −423 (0.17×) | 39.8 vs halo2curves 217 | −177 (0.18×) | 98.4 vs halo2curves 708 | −610 (0.14×) | 65.8 vs halo2curves 290 | −224 (0.23×) |
| add_u64 | BLS12-381 base | 160 vs ark-ff 830 | −670 (0.19×) | 62.4 vs halo2curves 323 | −261 (0.19×) | 270 vs ark-ff 1,075 | −805 (0.25×) | 89.0 vs crypto-bigint 646 | −557 (0.14×) |
| add_u64 | P-521 base | 21.2 vs crypto-bigint 1,686 | −1,665 (0.01×) | 12.5 vs crypto-bigint 653 | −641 (0.02×) | 19.4 vs crypto-bigint 2,236 | −2,217 (0.01×) | 27.7 vs crypto-bigint 1,135 | −1,107 (0.02×) |
| add_uint | Mersenne-31 | 8.6 vs crypto-bigint 35.4 | −26.8 (0.24×) | 8.6 vs crypto-bigint 30.1 | −21.5 (0.29×) | 9.3 vs crypto-bigint 41.8 | −32.5 (0.22×) | 9.3 vs crypto-bigint 30.5 | −21.2 (0.30×) |
| add_uint | BabyBear | 12.2 vs plonky3 21.5 | −9.3 (0.57×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 31.3 | −18.6 (0.41×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| add_uint | Goldilocks | 9.1 vs winter-math 13.1 | −4.0 (0.69×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.8 vs winter-math 19.1 | −8.3 (0.57×) | 10.1 vs winter-math 9.1 | +1.0 (1.11×) |
| add_uint | BN254 scalar | 230 vs halo2curves 413 | −183 (0.56×) | 104 vs halo2curves 111 | −7.0 (0.94×) | 382 vs halo2curves 696 | −314 (0.55×) | 174 vs halo2curves 206 | −32.0 (0.84×) |
| add_uint | secp256k1 base | 17.0 vs halo2curves 225 | −208 (0.08×) | 12.5 vs halo2curves 86.4 | −73.9 (0.14×) | 16.7 vs halo2curves 421 | −404 (0.04×) | 23.4 vs halo2curves 140 | −117 (0.17×) |
| add_uint | P-256 base | 181 vs halo2curves 244 | −63.0 (0.74×) | 78.1 vs halo2curves 90.9 | −12.8 (0.86×) | 328 vs halo2curves 512 | −184 (0.64×) | 149 vs halo2curves 153 | −4.0 (0.97×) |
| add_uint | P-256 scalar | 258 vs k256/p256 85.1 | +173 (3.03×) | 106 vs k256/p256 82.0 | +24.0 (1.29×) | 342 vs k256/p256 107 | +235 (3.20×) | 201 vs k256/p256 107 | +94.0 (1.88×) |
| add_uint | Tom-256 base | 245 vs halo2curves 373 | −128 (0.66×) | 108 vs halo2curves 111 | −3.0 (0.97×) | 332 vs halo2curves 634 | −302 (0.52×) | 196 vs halo2curves 196 | +0.0 (1.00×) |
| add_uint | BLS12-381 base | 763 vs halo2curves 976 | −213 (0.78×) | 196 vs halo2curves 225 | −29.0 (0.87×) | 955 vs crypto-bigint 1,413 | −458 (0.68×) | 375 vs halo2curves 425 | −50.0 (0.88×) |
| add_uint | P-521 base | 62.6 vs crypto-bigint 1,698 | −1,635 (0.04×) | 42.1 vs crypto-bigint 664 | −622 (0.06×) | 85.0 vs crypto-bigint 2,220 | −2,135 (0.04×) | 67.2 vs crypto-bigint 1,153 | −1,086 (0.06×) |
| sub_u64 | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.5 vs crypto-bigint 30.4 | −23.9 (0.21×) | 6.7 vs crypto-bigint 20.2 | −13.5 (0.33×) |
| sub_u64 | BabyBear | 11.9 vs crypto-bigint 28.7 | −16.8 (0.41×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 12.7 vs plonky3 41.8 | −29.1 (0.30×) | 11.1 vs plonky3 13.4 | −2.3 (0.83×) |
| sub_u64 | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.6 vs winter-math 6.5 | +0.1 (1.02×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.4 vs winter-math 8.8 | +0.6 (1.07×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 345 | −232 (0.33×) | 45.8 vs crypto-bigint 153 | −107 (0.30×) | 137 vs ark-ff 402 | −265 (0.34×) | 67.1 vs crypto-bigint 286 | −219 (0.23×) |
| sub_u64 | secp256k1 base | 7.0 vs ark-ff 157 | −150 (0.04×) | 6.1 vs ark-ff 120 | −114 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 9.0 vs ark-ff 183 | −174 (0.05×) |
| sub_u64 | P-256 base | 71.3 vs k256/p256 265 | −194 (0.27×) | 34.7 vs k256/p256 88.0 | −53.3 (0.39×) | 79.8 vs ark-ff 395 | −315 (0.20×) | 55.6 vs k256/p256 149 | −93.4 (0.37×) |
| sub_u64 | P-256 scalar | 68.7 vs k256/p256 21.2 | +47.5 (3.24×) | 37.9 vs k256/p256 21.1 | +16.8 (1.80×) | 73.5 vs k256/p256 24.7 | +48.8 (2.98×) | 64.8 vs k256/p256 24.4 | +40.4 (2.66×) |
| sub_u64 | Tom-256 base | 91.6 vs crypto-bigint 488 | −396 (0.19×) | 39.3 vs crypto-bigint 152 | −113 (0.26×) | 105 vs halo2curves 699 | −594 (0.15×) | 66.6 vs crypto-bigint 284 | −217 (0.23×) |
| sub_u64 | BLS12-381 base | 155 vs ark-ff 831 | −676 (0.19×) | 62.2 vs halo2curves 315 | −253 (0.20×) | 266 vs ark-ff 1,055 | −789 (0.25×) | 92.1 vs crypto-bigint 624 | −532 (0.15×) |
| sub_u64 | P-521 base | 21.1 vs crypto-bigint 1,672 | −1,651 (0.01×) | 15.3 vs crypto-bigint 631 | −616 (0.02×) | 21.7 vs crypto-bigint 2,194 | −2,172 (0.01×) | 21.9 vs crypto-bigint 1,108 | −1,086 (0.02×) |
| sub_uint | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.4 vs crypto-bigint 18.2 | −12.8 (0.30×) | 6.5 vs crypto-bigint 30.6 | −24.1 (0.21×) | 6.6 vs crypto-bigint 20.3 | −13.7 (0.33×) |
| sub_uint | BabyBear | 12.0 vs plonky3 22.0 | −10.0 (0.55×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 13.1 vs plonky3 31.0 | −17.9 (0.42×) | 11.2 vs plonky3 13.1 | −1.9 (0.85×) |
| sub_uint | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.6 vs winter-math 7.0 | −0.4 (0.94×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.4 vs winter-math 8.7 | +0.7 (1.08×) |
| sub_uint | BN254 scalar | 228 vs halo2curves 397 | −169 (0.57×) | 103 vs halo2curves 105 | −2.0 (0.98×) | 367 vs halo2curves 672 | −305 (0.55×) | 177 vs halo2curves 194 | −17.0 (0.91×) |
| sub_uint | secp256k1 base | 18.9 vs halo2curves 210 | −191 (0.09×) | 12.4 vs halo2curves 80.7 | −68.3 (0.15×) | 16.5 vs halo2curves 403 | −387 (0.04×) | 21.9 vs halo2curves 130 | −108 (0.17×) |
| sub_uint | P-256 base | 185 vs halo2curves 233 | −48.0 (0.79×) | 78.9 vs halo2curves 85.7 | −6.8 (0.92×) | 372 vs halo2curves 504 | −132 (0.74×) | 145 vs halo2curves 143 | +2.0 (1.01×) |
| sub_uint | P-256 scalar | 257 vs k256/p256 72.0 | +185 (3.57×) | 109 vs k256/p256 72.0 | +37.0 (1.51×) | 340 vs k256/p256 95.2 | +245 (3.57×) | 202 vs k256/p256 95.0 | +107 (2.13×) |
| sub_uint | Tom-256 base | 246 vs halo2curves 359 | −113 (0.69×) | 105 vs halo2curves 103 | +2.0 (1.02×) | 332 vs halo2curves 613 | −281 (0.54×) | 196 vs halo2curves 187 | +9.0 (1.05×) |
| sub_uint | BLS12-381 base | 755 vs halo2curves 953 | −198 (0.79×) | 194 vs halo2curves 216 | −22.0 (0.90×) | 934 vs crypto-bigint 1,390 | −456 (0.67×) | 379 vs halo2curves 409 | −30.0 (0.93×) |
| sub_uint | P-521 base | 67.1 vs crypto-bigint 1,696 | −1,629 (0.04×) | 38.7 vs crypto-bigint 644 | −605 (0.06×) | 76.9 vs crypto-bigint 2,202 | −2,125 (0.03×) | 67.7 vs crypto-bigint 1,149 | −1,081 (0.06×) |
| div_u64 | BN254 scalar | 1,213 vs halo2curves 13,496 | −12,283 (0.09×) | 819 vs halo2curves 6,992 | −6,173 (0.12×) | 1,421 vs halo2curves 19,620 | −18,199 (0.07×) | 1,046 vs halo2curves 11,717 | −10,671 (0.09×) |
| div_u64 | secp256k1 base | 1,019 vs halo2curves 13,078 | −12,059 (0.08×) | 777 vs halo2curves 6,970 | −6,193 (0.11×) | 1,216 vs halo2curves 18,989 | −17,773 (0.06×) | 871 vs halo2curves 11,469 | −10,598 (0.08×) |
| div_u64 | P-256 base | 1,094 vs halo2curves 13,091 | −11,997 (0.08×) | 798 vs halo2curves 6,920 | −6,122 (0.12×) | 1,342 vs halo2curves 18,986 | −17,644 (0.07×) | 982 vs halo2curves 11,481 | −10,499 (0.09×) |
| div_u64 | P-256 scalar | 1,179 vs halo2curves 13,391 | −12,212 (0.09×) | 861 vs halo2curves 6,973 | −6,112 (0.12×) | 1,400 vs halo2curves 19,429 | −18,029 (0.07×) | 1,132 vs halo2curves 11,674 | −10,542 (0.10×) |
| div_u64 | Tom-256 base | 1,183 vs halo2curves 13,294 | −12,111 (0.09×) | 859 vs halo2curves 6,930 | −6,071 (0.12×) | 1,486 vs halo2curves 19,216 | −17,730 (0.08×) | 1,082 vs halo2curves 11,707 | −10,625 (0.09×) |
| div_u64 | BLS12-381 base | 1,802 vs halo2curves 24,152 | −22,350 (0.07×) | 1,031 vs halo2curves 11,019 | −9,988 (0.09×) | 2,484 vs halo2curves 32,568 | −30,084 (0.08×) | 1,753 vs halo2curves 18,582 | −16,829 (0.09×) |
| div_u64 | P-521 base | 2,454 vs crypto-bigint 62,363 | −59,909 (0.04×) | 1,373 vs crypto-bigint 45,439 | −44,066 (0.03×) | 3,629 vs crypto-bigint 77,369 | −73,740 (0.05×) | 1,867 vs crypto-bigint 57,961 | −56,094 (0.03×) |
| div_uint | Mersenne-31 | 276 vs crypto-bigint 1,808 | −1,532 (0.15×) | 277 vs crypto-bigint 1,613 | −1,336 (0.17×) | 287 vs runtime-algebra 2,123 | −1,836 (0.14×) | 281 vs crypto-bigint 2,005 | −1,724 (0.14×) |
| div_uint | BabyBear | 278 vs plonky3 672 | −394 (0.41×) | 278 vs plonky3 652 | −374 (0.43×) | 286 vs plonky3 728 | −442 (0.39×) | 283 vs plonky3 726 | −443 (0.39×) |
| div_uint | Goldilocks | 553 vs winter-math 1,809 | −1,256 (0.31×) | 543 vs winter-math 927 | −384 (0.59×) | 593 vs winter-math 1,875 | −1,282 (0.32×) | 579 vs winter-math 1,034 | −455 (0.56×) |
| div_uint | BN254 scalar | 9,042 vs halo2curves 15,087 | −6,045 (0.60×) | 6,541 vs halo2curves 8,206 | −1,665 (0.80×) | 12,999 vs halo2curves 22,942 | −9,943 (0.57×) | 8,439 vs halo2curves 14,422 | −5,983 (0.59×) |
| div_uint | secp256k1 base | 8,481 vs halo2curves 14,868 | −6,387 (0.57×) | 6,033 vs halo2curves 8,217 | −2,184 (0.73×) | 12,094 vs halo2curves 22,500 | −10,406 (0.54×) | 8,085 vs halo2curves 14,385 | −6,300 (0.56×) |
| div_uint | P-256 base | 8,763 vs halo2curves 14,915 | −6,152 (0.59×) | 6,170 vs halo2curves 8,220 | −2,050 (0.75×) | 12,245 vs halo2curves 22,615 | −10,370 (0.54×) | 7,914 vs halo2curves 14,350 | −6,436 (0.55×) |
| div_uint | P-256 scalar | 9,221 vs halo2curves 15,134 | −5,913 (0.61×) | 6,605 vs halo2curves 8,215 | −1,610 (0.80×) | 12,426 vs halo2curves 23,280 | −10,854 (0.53×) | 8,667 vs halo2curves 14,530 | −5,863 (0.60×) |
| div_uint | Tom-256 base | 8,984 vs halo2curves 15,040 | −6,056 (0.60×) | 6,489 vs halo2curves 8,218 | −1,729 (0.79×) | 12,329 vs halo2curves 23,030 | −10,701 (0.54×) | 8,467 vs halo2curves 14,603 | −6,136 (0.58×) |
| div_uint | BLS12-381 base | 20,824 vs halo2curves 27,811 | −6,987 (0.75×) | 12,045 vs halo2curves 13,611 | −1,566 (0.88×) | 26,517 vs halo2curves 40,627 | −14,110 (0.65×) | 16,955 vs halo2curves 24,277 | −7,322 (0.70×) |
| div_uint | P-521 base | 34,490 vs crypto-bigint 68,460 | −33,970 (0.50×) | 18,121 vs crypto-bigint 51,107 | −32,986 (0.35×) | 50,364 vs crypto-bigint 84,483 | −34,119 (0.60×) | 30,006 vs crypto-bigint 63,601 | −33,595 (0.47×) |
| add, constant time | Mersenne-31 | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.5 vs runtime-algebra 18.0 | −10.5 (0.42×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) | 7.7 vs runtime-algebra 17.7 | −10.0 (0.44×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) |
| add, constant time | BN254 scalar | 24.8 vs halo2curves 40.8 | −16.0 (0.61×) | 19.4 vs halo2curves 23.0 | −3.6 (0.84×) | 23.1 vs halo2curves 45.0 | −21.9 (0.51×) | 25.7 vs halo2curves 34.0 | −8.3 (0.76×) |
| add, constant time | secp256k1 base | 28.1 vs halo2curves 40.2 | −12.1 (0.70×) | 22.6 vs halo2curves 23.0 | −0.4 (0.98×) | 27.0 vs halo2curves 38.0 | −11.0 (0.71×) | 29.8 vs halo2curves 31.0 | −1.2 (0.96×) |
| add, constant time | P-256 base | 26.8 vs halo2curves 38.9 | −12.1 (0.69×) | 22.7 vs halo2curves 21.0 | +1.7 (1.08×) | 24.9 vs halo2curves 38.0 | −13.1 (0.66×) | 28.1 vs k256/p256 29.4 | −1.3 (0.96×) |
| add, constant time | P-256 scalar | 26.6 vs halo2curves 42.2 | −15.6 (0.63×) | 23.7 vs halo2curves 24.8 | −1.1 (0.96×) | 26.9 vs halo2curves 42.6 | −15.7 (0.63×) | 30.1 vs halo2curves 33.0 | −2.9 (0.91×) |
| add, constant time | Tom-256 base | 27.9 vs halo2curves 42.9 | −15.0 (0.65×) | 23.8 vs halo2curves 24.0 | −0.2 (0.99×) | 27.1 vs halo2curves 42.5 | −15.4 (0.64×) | 29.1 vs halo2curves 32.5 | −3.4 (0.90×) |
| add, constant time | BLS12-381 base | 41.4 vs halo2curves 63.3 | −21.9 (0.65×) | 28.3 vs halo2curves 34.1 | −5.8 (0.83×) | 35.5 vs halo2curves 66.8 | −31.3 (0.53×) | 41.1 vs halo2curves 54.1 | −13.0 (0.76×) |
| add, constant time | P-521 base | 76.7 vs crypto-bigint 181 | −104 (0.42×) | 47.2 vs crypto-bigint 179 | −132 (0.26×) | 80.0 vs crypto-bigint 244 | −164 (0.33×) | 74.1 vs crypto-bigint 245 | −171 (0.30×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs runtime-algebra 7.9 | −4.9 (0.38×) | 3.0 vs runtime-algebra 7.9 | −4.9 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 26.4 vs halo2curves 26.0 | +0.4 (1.02×) | 16.9 vs halo2curves 14.9 | +2.0 (1.13×) | 27.0 vs halo2curves 25.5 | +1.5 (1.06×) | 25.0 vs halo2curves 22.0 | +3.0 (1.14×) |
| sub, constant time | secp256k1 base | 21.4 vs halo2curves 25.3 | −3.9 (0.85×) | 15.5 vs halo2curves 14.5 | +1.0 (1.07×) | 20.5 vs halo2curves 26.4 | −5.9 (0.78×) | 21.5 vs halo2curves 21.0 | +0.5 (1.02×) |
| sub, constant time | P-256 base | 19.1 vs halo2curves 25.1 | −6.0 (0.76×) | 15.6 vs halo2curves 14.0 | +1.6 (1.11×) | 18.6 vs halo2curves 24.0 | −5.4 (0.78×) | 20.0 vs k256/p256 20.0 | +0.0 (1.00×) |
| sub, constant time | P-256 scalar | 21.6 vs k256/p256 24.8 | −3.2 (0.87×) | 15.7 vs halo2curves 15.7 | +0.0 (1.00×) | 18.7 vs halo2curves 25.9 | −7.2 (0.72×) | 22.0 vs halo2curves 21.6 | +0.4 (1.02×) |
| sub, constant time | Tom-256 base | 21.2 vs halo2curves 26.0 | −4.8 (0.82×) | 17.1 vs halo2curves 14.7 | +2.4 (1.16×) | 19.8 vs halo2curves 26.5 | −6.7 (0.75×) | 23.0 vs halo2curves 22.1 | +0.9 (1.04×) |
| sub, constant time | BLS12-381 base | 37.4 vs halo2curves 45.3 | −7.9 (0.83×) | 24.2 vs halo2curves 24.9 | −0.7 (0.97×) | 36.9 vs halo2curves 44.1 | −7.2 (0.84×) | 36.3 vs halo2curves 36.1 | +0.2 (1.01×) |
| sub, constant time | P-521 base | 57.1 vs crypto-bigint 159 | −102 (0.36×) | 37.2 vs crypto-bigint 144 | −107 (0.26×) | 60.3 vs crypto-bigint 194 | −134 (0.31×) | 53.1 vs crypto-bigint 225 | −172 (0.24×) |
| mul, constant time | Mersenne-31 | 12.0 vs crypto-bigint 46.7 | −34.7 (0.26×) | 12.0 vs crypto-bigint 27.0 | −15.0 (0.44×) | 12.0 vs crypto-bigint 50.8 | −38.8 (0.24×) | 12.0 vs crypto-bigint 30.9 | −18.9 (0.39×) |
| mul, constant time | BabyBear | 13.0 vs crypto-bigint 46.4 | −33.4 (0.28×) | 11.0 vs crypto-bigint 27.0 | −16.0 (0.41×) | 14.5 vs crypto-bigint 50.1 | −35.6 (0.29×) | 11.0 vs crypto-bigint 29.9 | −18.9 (0.37×) |
| mul, constant time | Goldilocks | 18.1 vs winter-math 18.0 | +0.1 (1.01×) | 14.4 vs winter-math 14.5 | −0.1 (0.99×) | 21.0 vs winter-math 19.0 | +2.0 (1.11×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul, constant time | BN254 scalar | 242 vs halo2curves 378 | −136 (0.64×) | 94.8 vs halo2curves 95.1 | −0.3 (1.00×) | 415 vs halo2curves 677 | −262 (0.61×) | 154 vs halo2curves 178 | −24.0 (0.87×) |
| mul, constant time | secp256k1 base | 173 vs halo2curves 306 | −133 (0.57×) | 77.6 vs halo2curves 90.2 | −12.6 (0.86×) | 276 vs halo2curves 583 | −307 (0.47×) | 106 vs halo2curves 159 | −53.0 (0.67×) |
| mul, constant time | P-256 base | 189 vs k256/p256 243 | −54.0 (0.78×) | 72.2 vs halo2curves 74.9 | −2.7 (0.96×) | 316 vs k256/p256 467 | −151 (0.68×) | 120 vs halo2curves 130 | −10.0 (0.92×) |
| mul, constant time | P-256 scalar | 233 vs halo2curves 349 | −116 (0.67×) | 94.4 vs halo2curves 95.9 | −1.5 (0.98×) | 335 vs halo2curves 680 | −345 (0.49×) | 158 vs halo2curves 176 | −18.0 (0.90×) |
| mul, constant time | Tom-256 base | 237 vs halo2curves 344 | −107 (0.69×) | 88.2 vs halo2curves 93.9 | −5.7 (0.94×) | 336 vs halo2curves 595 | −259 (0.56×) | 151 vs halo2curves 172 | −21.0 (0.88×) |
| mul, constant time | BLS12-381 base | 720 vs halo2curves 937 | −217 (0.77×) | 189 vs halo2curves 193 | −4.0 (0.98×) | 930 vs crypto-bigint 1,333 | −403 (0.70×) | 337 vs halo2curves 376 | −39.0 (0.90×) |
| mul, constant time | P-521 base | 687 vs crypto-bigint 2,137 | −1,450 (0.32×) | 247 vs crypto-bigint 619 | −372 (0.40×) | 1,300 vs crypto-bigint 2,804 | −1,504 (0.46×) | 445 vs crypto-bigint 1,099 | −654 (0.40×) |
| square, constant time | Mersenne-31 | 12.0 vs crypto-bigint 45.4 | −33.4 (0.26×) | 12.0 vs crypto-bigint 27.1 | −15.1 (0.44×) | 12.0 vs crypto-bigint 52.0 | −40.0 (0.23×) | 12.0 vs crypto-bigint 30.0 | −18.0 (0.40×) |
| square, constant time | BabyBear | 13.0 vs crypto-bigint 45.3 | −32.3 (0.29×) | 11.0 vs crypto-bigint 27.0 | −16.0 (0.41×) | 14.0 vs crypto-bigint 52.7 | −38.7 (0.27×) | 11.0 vs crypto-bigint 30.9 | −19.9 (0.36×) |
| square, constant time | Goldilocks | 17.3 vs winter-math 19.7 | −2.4 (0.88×) | 12.0 vs winter-math 12.8 | −0.8 (0.94×) | 21.1 vs winter-math 19.0 | +2.1 (1.11×) | 14.4 vs winter-math 14.7 | −0.3 (0.98×) |
| square, constant time | BN254 scalar | 219 vs halo2curves 309 | −90.0 (0.71×) | 83.7 vs halo2curves 96.4 | −12.7 (0.87×) | 329 vs halo2curves 551 | −222 (0.60×) | 138 vs halo2curves 144 | −6.0 (0.96×) |
| square, constant time | secp256k1 base | 153 vs halo2curves 222 | −69.0 (0.69×) | 69.9 vs halo2curves 83.6 | −13.7 (0.84×) | 168 vs halo2curves 441 | −273 (0.38×) | 97.4 vs halo2curves 126 | −28.6 (0.77×) |
| square, constant time | P-256 base | 155 vs halo2curves 178 | −23.0 (0.87×) | 62.6 vs halo2curves 62.9 | −0.3 (1.00×) | 227 vs halo2curves 338 | −111 (0.67×) | 113 vs halo2curves 107 | +6.0 (1.06×) |
| square, constant time | P-256 scalar | 206 vs halo2curves 266 | −60.0 (0.77×) | 89.8 vs halo2curves 93.8 | −4.0 (0.96×) | 242 vs halo2curves 567 | −325 (0.43×) | 149 vs halo2curves 137 | +12.0 (1.09×) |
| square, constant time | Tom-256 base | 200 vs halo2curves 268 | −68.0 (0.75×) | 83.1 vs halo2curves 88.5 | −5.4 (0.94×) | 290 vs halo2curves 451 | −161 (0.64×) | 142 vs halo2curves 135 | +7.0 (1.05×) |
| square, constant time | BLS12-381 base | 608 vs halo2curves 919 | −311 (0.66×) | 169 vs halo2curves 163 | +6.0 (1.04×) | 867 vs halo2curves 1,266 | −399 (0.68×) | 297 vs halo2curves 292 | +5.0 (1.02×) |
| square, constant time | P-521 base | 571 vs crypto-bigint 2,140 | −1,569 (0.27×) | 208 vs crypto-bigint 634 | −426 (0.33×) | 907 vs crypto-bigint 2,840 | −1,933 (0.32×) | 392 vs crypto-bigint 1,139 | −747 (0.34×) |
| mul_u64, constant time | Mersenne-31 | 14.0 vs crypto-bigint 60.6 | −46.6 (0.23×) | 14.0 vs crypto-bigint 32.2 | −18.2 (0.43×) | 14.0 vs crypto-bigint 71.3 | −57.3 (0.20×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_u64, constant time | BabyBear | 17.0 vs crypto-bigint 65.2 | −48.2 (0.26×) | 11.0 vs crypto-bigint 32.7 | −21.7 (0.34×) | 19.3 vs crypto-bigint 86.1 | −66.8 (0.22×) | 13.0 vs crypto-bigint 41.0 | −28.0 (0.32×) |
| mul_u64, constant time | Goldilocks | 20.4 vs winter-math 23.4 | −3.0 (0.87×) | 12.1 vs winter-math 13.2 | −1.1 (0.92×) | 25.0 vs winter-math 34.6 | −9.6 (0.72×) | 17.2 vs winter-math 14.5 | +2.7 (1.19×) |
| mul_u64, constant time | BN254 scalar | 108 vs crypto-bigint 862 | −754 (0.13×) | 50.9 vs crypto-bigint 262 | −211 (0.19×) | 132 vs halo2curves 1,439 | −1,307 (0.09×) | 58.0 vs crypto-bigint 412 | −354 (0.14×) |
| mul_u64, constant time | secp256k1 base | 68.9 vs halo2curves 690 | −621 (0.10×) | 39.0 vs crypto-bigint 259 | −220 (0.15×) | 85.1 vs halo2curves 1,053 | −968 (0.08×) | 44.3 vs crypto-bigint 410 | −366 (0.11×) |
| mul_u64, constant time | P-256 base | 87.6 vs k256/p256 487 | −399 (0.18×) | 53.2 vs k256/p256 153 | −99.8 (0.35×) | 105 vs k256/p256 919 | −814 (0.11×) | 59.5 vs k256/p256 258 | −199 (0.23×) |
| mul_u64, constant time | P-256 scalar | 105 vs k256/p256 516 | −411 (0.20×) | 55.0 vs k256/p256 255 | −200 (0.22×) | 133 vs k256/p256 820 | −687 (0.16×) | 65.5 vs k256/p256 340 | −275 (0.19×) |
| mul_u64, constant time | Tom-256 base | 99.7 vs crypto-bigint 873 | −773 (0.11×) | 52.0 vs crypto-bigint 261 | −209 (0.20×) | 122 vs halo2curves 1,273 | −1,151 (0.10×) | 61.4 vs crypto-bigint 410 | −349 (0.15×) |
| mul_u64, constant time | BLS12-381 base | 154 vs crypto-bigint 1,953 | −1,799 (0.08×) | 70.8 vs halo2curves 485 | −414 (0.15×) | 195 vs crypto-bigint 2,511 | −2,316 (0.08×) | 88.5 vs crypto-bigint 1,050 | −962 (0.08×) |
| mul_u64, constant time | P-521 base | 139 vs crypto-bigint 3,015 | −2,876 (0.05×) | 78.0 vs crypto-bigint 990 | −912 (0.08×) | 170 vs crypto-bigint 4,018 | −3,848 (0.04×) | 94.9 vs crypto-bigint 1,843 | −1,748 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 14.0 vs crypto-bigint 60.4 | −46.4 (0.23×) | 14.0 vs crypto-bigint 31.7 | −17.7 (0.44×) | 14.0 vs crypto-bigint 73.4 | −59.4 (0.19×) | 14.0 vs crypto-bigint 38.8 | −24.8 (0.36×) |
| mul_uint, constant time | BabyBear | 17.0 vs crypto-bigint 65.6 | −48.6 (0.26×) | 11.0 vs crypto-bigint 32.3 | −21.3 (0.34×) | 19.4 vs crypto-bigint 89.1 | −69.7 (0.22×) | 13.1 vs crypto-bigint 40.8 | −27.7 (0.32×) |
| mul_uint, constant time | Goldilocks | 20.4 vs winter-math 23.6 | −3.2 (0.86×) | 12.1 vs winter-math 13.8 | −1.7 (0.88×) | 25.0 vs winter-math 34.5 | −9.5 (0.72×) | 17.2 vs winter-math 14.5 | +2.7 (1.19×) |
| mul_uint, constant time | BN254 scalar | 481 vs halo2curves 773 | −292 (0.62×) | 167 vs halo2curves 182 | −15.0 (0.92×) | 593 vs halo2curves 1,317 | −724 (0.45×) | 312 vs halo2curves 354 | −42.0 (0.88×) |
| mul_uint, constant time | secp256k1 base | 317 vs halo2curves 516 | −199 (0.61×) | 193 vs halo2curves 154 | +39.0 (1.25×) | 380 vs halo2curves 959 | −579 (0.40×) | 253 vs halo2curves 265 | −12.0 (0.95×) |
| mul_uint, constant time | P-256 base | 406 vs halo2curves 460 | −54.0 (0.88×) | 137 vs halo2curves 145 | −8.0 (0.94×) | 489 vs halo2curves 947 | −458 (0.52×) | 235 vs halo2curves 252 | −17.0 (0.93×) |
| mul_uint, constant time | P-256 scalar | 478 vs k256/p256 551 | −73.0 (0.87×) | 178 vs halo2curves 186 | −8.0 (0.96×) | 540 vs k256/p256 863 | −323 (0.63×) | 314 vs halo2curves 350 | −36.0 (0.90×) |
| mul_uint, constant time | Tom-256 base | 467 vs halo2curves 695 | −228 (0.67×) | 172 vs halo2curves 180 | −8.0 (0.96×) | 554 vs halo2curves 1,179 | −625 (0.47×) | 300 vs halo2curves 335 | −35.0 (0.90×) |
| mul_uint, constant time | BLS12-381 base | 1,082 vs halo2curves 1,928 | −846 (0.56×) | 370 vs halo2curves 379 | −9.0 (0.98×) | 1,699 vs crypto-bigint 2,549 | −850 (0.67×) | 912 vs halo2curves 1,040 | −128 (0.88×) |
| mul_uint, constant time | P-521 base | 2,017 vs crypto-bigint 3,057 | −1,040 (0.66×) | 997 vs crypto-bigint 1,010 | −13.0 (0.99×) | 2,901 vs crypto-bigint 4,023 | −1,122 (0.72×) | 1,465 vs crypto-bigint 1,874 | −409 (0.78×) |
| add_u64, constant time | Mersenne-31 | 8.6 vs crypto-bigint 35.4 | −26.8 (0.24×) | 8.7 vs crypto-bigint 30.1 | −21.4 (0.29×) | 9.3 vs crypto-bigint 43.6 | −34.3 (0.21×) | 9.3 vs crypto-bigint 30.6 | −21.3 (0.30×) |
| add_u64, constant time | BabyBear | 12.2 vs crypto-bigint 47.6 | −35.4 (0.26×) | 8.2 vs crypto-bigint 30.1 | −21.9 (0.27×) | 12.7 vs crypto-bigint 53.0 | −40.3 (0.24×) | 11.0 vs crypto-bigint 31.8 | −20.8 (0.35×) |
| add_u64, constant time | Goldilocks | 9.1 vs winter-math 13.1 | −4.0 (0.69×) | 7.3 vs winter-math 7.6 | −0.3 (0.96×) | 10.7 vs winter-math 19.1 | −8.4 (0.56×) | 10.0 vs winter-math 9.1 | +0.9 (1.10×) |
| add_u64, constant time | BN254 scalar | 132 vs crypto-bigint 490 | −358 (0.27×) | 55.9 vs halo2curves 213 | −157 (0.26×) | 151 vs halo2curves 746 | −595 (0.20×) | 81.8 vs halo2curves 298 | −216 (0.27×) |
| add_u64, constant time | secp256k1 base | 25.4 vs halo2curves 401 | −376 (0.06×) | 23.0 vs halo2curves 218 | −195 (0.11×) | 25.1 vs halo2curves 494 | −469 (0.05×) | 29.0 vs halo2curves 247 | −218 (0.12×) |
| add_u64, constant time | P-256 base | 89.1 vs k256/p256 278 | −189 (0.32×) | 55.6 vs k256/p256 92.6 | −37.0 (0.60×) | 94.9 vs k256/p256 507 | −412 (0.19×) | 74.6 vs k256/p256 158 | −83.4 (0.47×) |
| add_u64, constant time | P-256 scalar | 85.7 vs k256/p256 56.0 | +29.7 (1.53×) | 58.6 vs k256/p256 56.2 | +2.4 (1.04×) | 98.4 vs k256/p256 61.9 | +36.5 (1.59×) | 89.6 vs k256/p256 62.9 | +26.7 (1.42×) |
| add_u64, constant time | Tom-256 base | 119 vs crypto-bigint 511 | −392 (0.23×) | 59.6 vs halo2curves 217 | −157 (0.27×) | 133 vs halo2curves 708 | −575 (0.19×) | 85.1 vs halo2curves 290 | −205 (0.29×) |
| add_u64, constant time | BLS12-381 base | 182 vs crypto-bigint 1,084 | −902 (0.17×) | 96.1 vs halo2curves 323 | −227 (0.30×) | 233 vs crypto-bigint 1,376 | −1,143 (0.17×) | 124 vs crypto-bigint 646 | −522 (0.19×) |
| add_u64, constant time | P-521 base | 65.6 vs crypto-bigint 1,686 | −1,620 (0.04×) | 47.0 vs crypto-bigint 653 | −606 (0.07×) | 69.0 vs crypto-bigint 2,236 | −2,167 (0.03×) | 64.9 vs crypto-bigint 1,135 | −1,070 (0.06×) |
| add_uint, constant time | Mersenne-31 | 8.7 vs crypto-bigint 35.4 | −26.7 (0.25×) | 8.6 vs crypto-bigint 30.1 | −21.5 (0.29×) | 9.3 vs crypto-bigint 41.8 | −32.5 (0.22×) | 9.3 vs crypto-bigint 30.5 | −21.2 (0.30×) |
| add_uint, constant time | BabyBear | 12.4 vs crypto-bigint 47.5 | −35.1 (0.26×) | 8.2 vs crypto-bigint 30.1 | −21.9 (0.27×) | 12.7 vs crypto-bigint 51.1 | −38.4 (0.25×) | 11.0 vs crypto-bigint 31.9 | −20.9 (0.34×) |
| add_uint, constant time | Goldilocks | 9.0 vs winter-math 13.1 | −4.1 (0.69×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 10.7 vs winter-math 19.1 | −8.4 (0.56×) | 10.0 vs winter-math 9.1 | +0.9 (1.10×) |
| add_uint, constant time | BN254 scalar | 230 vs halo2curves 413 | −183 (0.56×) | 98.1 vs halo2curves 111 | −12.9 (0.88×) | 297 vs halo2curves 696 | −399 (0.43×) | 181 vs halo2curves 206 | −25.0 (0.88×) |
| add_uint, constant time | secp256k1 base | 38.3 vs halo2curves 225 | −187 (0.17×) | 29.4 vs halo2curves 86.4 | −57.0 (0.34×) | 36.3 vs halo2curves 421 | −385 (0.09×) | 44.2 vs halo2curves 140 | −95.8 (0.32×) |
| add_uint, constant time | P-256 base | 178 vs halo2curves 244 | −66.0 (0.73×) | 84.7 vs halo2curves 90.9 | −6.2 (0.93×) | 255 vs halo2curves 512 | −257 (0.50×) | 144 vs halo2curves 153 | −9.0 (0.94×) |
| add_uint, constant time | P-256 scalar | 260 vs k256/p256 85.1 | +175 (3.06×) | 107 vs k256/p256 82.0 | +25.0 (1.30×) | 311 vs k256/p256 107 | +204 (2.91×) | 197 vs k256/p256 107 | +90.0 (1.84×) |
| add_uint, constant time | Tom-256 base | 255 vs halo2curves 373 | −118 (0.68×) | 104 vs halo2curves 111 | −7.0 (0.94×) | 332 vs halo2curves 634 | −302 (0.52×) | 182 vs halo2curves 196 | −14.0 (0.93×) |
| add_uint, constant time | BLS12-381 base | 764 vs halo2curves 976 | −212 (0.78×) | 206 vs halo2curves 225 | −19.0 (0.92×) | 941 vs crypto-bigint 1,413 | −472 (0.67×) | 369 vs halo2curves 425 | −56.0 (0.87×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,698 | −1,560 (0.08×) | 93.5 vs crypto-bigint 664 | −571 (0.14×) | 147 vs crypto-bigint 2,220 | −2,073 (0.07×) | 134 vs crypto-bigint 1,153 | −1,019 (0.12×) |
| sub_u64, constant time | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.4 vs crypto-bigint 18.2 | −12.8 (0.30×) | 6.6 vs crypto-bigint 30.4 | −23.8 (0.22×) | 6.6 vs crypto-bigint 20.2 | −13.6 (0.33×) |
| sub_u64, constant time | BabyBear | 11.9 vs crypto-bigint 28.7 | −16.8 (0.41×) | 8.2 vs crypto-bigint 19.3 | −11.1 (0.42×) | 13.1 vs crypto-bigint 42.3 | −29.2 (0.31×) | 11.2 vs crypto-bigint 21.8 | −10.6 (0.51×) |
| sub_u64, constant time | Goldilocks | 8.8 vs winter-math 11.1 | −2.3 (0.79×) | 6.6 vs winter-math 6.5 | +0.1 (1.02×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.4 vs winter-math 8.8 | +0.6 (1.07×) |
| sub_u64, constant time | BN254 scalar | 125 vs crypto-bigint 484 | −359 (0.26×) | 56.9 vs crypto-bigint 153 | −96.1 (0.37×) | 153 vs halo2curves 743 | −590 (0.21×) | 80.0 vs crypto-bigint 286 | −206 (0.28×) |
| sub_u64, constant time | secp256k1 base | 18.0 vs halo2curves 380 | −362 (0.05×) | 15.1 vs crypto-bigint 151 | −136 (0.10×) | 17.0 vs halo2curves 481 | −464 (0.04×) | 17.3 vs halo2curves 239 | −222 (0.07×) |
| sub_u64, constant time | P-256 base | 85.7 vs k256/p256 265 | −179 (0.32×) | 50.6 vs k256/p256 88.0 | −37.4 (0.58×) | 88.2 vs k256/p256 491 | −403 (0.18×) | 67.2 vs k256/p256 149 | −81.8 (0.45×) |
| sub_u64, constant time | P-256 scalar | 81.3 vs k256/p256 21.2 | +60.1 (3.83×) | 50.8 vs k256/p256 21.1 | +29.7 (2.41×) | 93.2 vs k256/p256 24.7 | +68.5 (3.77×) | 79.8 vs k256/p256 24.4 | +55.4 (3.27×) |
| sub_u64, constant time | Tom-256 base | 113 vs crypto-bigint 488 | −375 (0.23×) | 56.2 vs crypto-bigint 152 | −95.8 (0.37×) | 125 vs halo2curves 699 | −574 (0.18×) | 80.5 vs crypto-bigint 284 | −204 (0.28×) |
| sub_u64, constant time | BLS12-381 base | 181 vs crypto-bigint 1,092 | −911 (0.17×) | 90.0 vs halo2curves 315 | −225 (0.29×) | 223 vs crypto-bigint 1,387 | −1,164 (0.16×) | 120 vs crypto-bigint 624 | −504 (0.19×) |
| sub_u64, constant time | P-521 base | 45.5 vs crypto-bigint 1,672 | −1,627 (0.03×) | 36.1 vs crypto-bigint 631 | −595 (0.06×) | 49.8 vs crypto-bigint 2,194 | −2,144 (0.02×) | 46.3 vs crypto-bigint 1,108 | −1,062 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 5.3 vs crypto-bigint 22.1 | −16.8 (0.24×) | 5.3 vs crypto-bigint 18.2 | −12.9 (0.29×) | 6.6 vs crypto-bigint 30.6 | −24.0 (0.22×) | 6.6 vs crypto-bigint 20.3 | −13.7 (0.33×) |
| sub_uint, constant time | BabyBear | 11.9 vs crypto-bigint 28.7 | −16.8 (0.41×) | 8.2 vs crypto-bigint 19.4 | −11.2 (0.42×) | 13.0 vs crypto-bigint 42.4 | −29.4 (0.31×) | 11.2 vs crypto-bigint 21.3 | −10.1 (0.53×) |
| sub_uint, constant time | Goldilocks | 8.9 vs winter-math 11.1 | −2.2 (0.80×) | 6.5 vs winter-math 7.0 | −0.5 (0.93×) | 10.7 vs winter-math 18.1 | −7.4 (0.59×) | 9.4 vs winter-math 8.7 | +0.7 (1.08×) |
| sub_uint, constant time | BN254 scalar | 242 vs halo2curves 397 | −155 (0.61×) | 99.0 vs halo2curves 105 | −6.0 (0.94×) | 283 vs halo2curves 672 | −389 (0.42×) | 180 vs halo2curves 194 | −14.0 (0.93×) |
| sub_uint, constant time | secp256k1 base | 32.2 vs halo2curves 210 | −178 (0.15×) | 25.0 vs halo2curves 80.7 | −55.7 (0.31×) | 31.8 vs halo2curves 403 | −371 (0.08×) | 38.3 vs halo2curves 130 | −91.7 (0.29×) |
| sub_uint, constant time | P-256 base | 176 vs halo2curves 233 | −57.0 (0.76×) | 80.3 vs halo2curves 85.7 | −5.4 (0.94×) | 251 vs halo2curves 504 | −253 (0.50×) | 138 vs halo2curves 143 | −5.0 (0.97×) |
| sub_uint, constant time | P-256 scalar | 253 vs k256/p256 72.0 | +181 (3.51×) | 102 vs k256/p256 72.0 | +30.0 (1.42×) | 331 vs k256/p256 95.2 | +236 (3.48×) | 189 vs k256/p256 95.0 | +94.0 (1.99×) |
| sub_uint, constant time | Tom-256 base | 248 vs halo2curves 359 | −111 (0.69×) | 98.4 vs halo2curves 103 | −4.6 (0.96×) | 333 vs halo2curves 613 | −280 (0.54×) | 177 vs halo2curves 187 | −10.0 (0.95×) |
| sub_uint, constant time | BLS12-381 base | 754 vs halo2curves 953 | −199 (0.79×) | 204 vs halo2curves 216 | −12.0 (0.94×) | 946 vs crypto-bigint 1,390 | −444 (0.68×) | 376 vs halo2curves 409 | −33.0 (0.92×) |
| sub_uint, constant time | P-521 base | 124 vs crypto-bigint 1,696 | −1,572 (0.07×) | 85.4 vs crypto-bigint 644 | −559 (0.13×) | 121 vs crypto-bigint 2,202 | −2,081 (0.05×) | 113 vs crypto-bigint 1,149 | −1,036 (0.10×) |
| div_u64, constant time | BN254 scalar | 12,371 vs crypto-bigint 30,027 | −17,656 (0.41×) | 9,487 vs crypto-bigint 25,481 | −15,994 (0.37×) | 17,513 vs crypto-bigint 37,804 | −20,291 (0.46×) | 12,967 vs crypto-bigint 30,530 | −17,563 (0.42×) |
| div_u64, constant time | secp256k1 base | 11,948 vs crypto-bigint 30,315 | −18,367 (0.39×) | 8,999 vs crypto-bigint 25,508 | −16,509 (0.35×) | 17,024 vs crypto-bigint 37,939 | −20,915 (0.45×) | 12,708 vs crypto-bigint 30,532 | −17,824 (0.42×) |
| div_u64, constant time | P-256 base | 11,856 vs crypto-bigint 29,686 | −17,830 (0.40×) | 9,177 vs k256/p256 22,383 | −13,206 (0.41×) | 17,369 vs crypto-bigint 37,657 | −20,288 (0.46×) | 12,549 vs crypto-bigint 30,555 | −18,006 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,313 vs crypto-bigint 30,112 | −17,799 (0.41×) | 9,442 vs crypto-bigint 25,573 | −16,131 (0.37×) | 17,333 vs crypto-bigint 37,588 | −20,255 (0.46×) | 13,173 vs crypto-bigint 30,438 | −17,265 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,164 vs crypto-bigint 29,997 | −17,833 (0.41×) | 9,294 vs crypto-bigint 25,532 | −16,238 (0.36×) | 16,956 vs crypto-bigint 37,743 | −20,787 (0.45×) | 12,902 vs crypto-bigint 30,560 | −17,658 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,774 vs crypto-bigint 51,874 | −27,100 (0.48×) | 16,947 vs crypto-bigint 43,315 | −26,368 (0.39×) | 35,027 vs crypto-bigint 68,818 | −33,791 (0.51×) | 24,420 vs crypto-bigint 54,007 | −29,587 (0.45×) |
| div_u64, constant time | P-521 base | 43,048 vs crypto-bigint 106,113 | −63,065 (0.41×) | 26,016 vs crypto-bigint 84,937 | −58,921 (0.31×) | 64,693 vs crypto-bigint 127,003 | −62,310 (0.51×) | 43,074 vs crypto-bigint 99,817 | −56,743 (0.43×) |
| div_uint, constant time | Mersenne-31 | 451 vs runtime-algebra 3,532 | −3,081 (0.13×) | 451 vs runtime-algebra 3,542 | −3,091 (0.13×) | 459 vs runtime-algebra 3,417 | −2,958 (0.13×) | 462 vs runtime-algebra 3,417 | −2,955 (0.14×) |
| div_uint, constant time | BabyBear | 545 vs runtime-algebra 3,557 | −3,012 (0.15×) | 436 vs runtime-algebra 3,526 | −3,090 (0.12×) | 607 vs runtime-algebra 3,414 | −2,807 (0.18×) | 450 vs runtime-algebra 3,418 | −2,968 (0.13×) |
| div_uint, constant time | Goldilocks | 1,308 vs winter-math 1,809 | −501 (0.72×) | 947 vs winter-math 927 | +20.0 (1.02×) | 1,577 vs winter-math 1,875 | −298 (0.84×) | 1,207 vs winter-math 1,034 | +173 (1.17×) |
| div_uint, constant time | BN254 scalar | 12,397 vs crypto-bigint 29,764 | −17,367 (0.42×) | 9,514 vs crypto-bigint 25,794 | −16,280 (0.37×) | 17,589 vs crypto-bigint 37,624 | −20,035 (0.47×) | 13,298 vs crypto-bigint 30,615 | −17,317 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,902 vs crypto-bigint 29,969 | −18,067 (0.40×) | 9,074 vs crypto-bigint 25,885 | −16,811 (0.35×) | 16,852 vs crypto-bigint 37,805 | −20,953 (0.45×) | 12,843 vs crypto-bigint 30,472 | −17,629 (0.42×) |
| div_uint, constant time | P-256 base | 11,985 vs crypto-bigint 29,958 | −17,973 (0.40×) | 9,149 vs k256/p256 22,389 | −13,240 (0.41×) | 17,590 vs crypto-bigint 37,639 | −20,049 (0.47×) | 12,643 vs crypto-bigint 30,481 | −17,838 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,486 vs crypto-bigint 29,972 | −17,486 (0.42×) | 9,436 vs crypto-bigint 25,862 | −16,426 (0.36×) | 17,771 vs crypto-bigint 37,629 | −19,858 (0.47×) | 13,263 vs crypto-bigint 30,620 | −17,357 (0.43×) |
| div_uint, constant time | Tom-256 base | 12,377 vs crypto-bigint 30,082 | −17,705 (0.41×) | 9,442 vs crypto-bigint 25,996 | −16,554 (0.36×) | 17,138 vs crypto-bigint 38,199 | −21,061 (0.45×) | 13,092 vs crypto-bigint 30,579 | −17,487 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,361 vs crypto-bigint 51,625 | −26,264 (0.49×) | 16,780 vs crypto-bigint 43,064 | −26,284 (0.39×) | 35,352 vs crypto-bigint 68,948 | −33,596 (0.51×) | 24,779 vs crypto-bigint 53,885 | −29,106 (0.46×) |
| div_uint, constant time | P-521 base | 42,851 vs crypto-bigint 104,924 | −62,073 (0.41×) | 25,984 vs crypto-bigint 84,638 | −58,654 (0.31×) | 65,678 vs crypto-bigint 127,407 | −61,729 (0.52×) | 43,052 vs crypto-bigint 100,068 | −57,016 (0.43×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 4.2 (2.2×) | 4.2 (2.2×) | 4.4 (2.3×) | 4.4 (2.3×) |
| add | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.1 (1.6×) | 3.1 (1.6×) |
| add | BN254 scalar | 5.1 | 17.1 (3.4×) | 12.3 (2.4×) | 16.7 (3.3×) | 19.0 (3.7×) |
| add | secp256k1 base | 5.8 | 14.0 (2.4×) | 11.8 (2.0×) | 14.8 (2.6×) | 20.4 (3.5×) |
| add | P-256 base | 5.8 | 14.7 (2.5×) | 12.3 (2.1×) | 14.8 (2.6×) | 19.9 (3.4×) |
| add | P-256 scalar | 6.0 | 15.4 (2.6×) | 11.8 (2.0×) | 16.2 (2.7×) | 21.4 (3.6×) |
| add | Tom-256 base | 6.0 | 15.0 (2.5×) | 11.7 (1.9×) | 15.8 (2.6×) | 20.6 (3.4×) |
| add | BLS12-381 base | 9.9 | 26.1 (2.6×) | 17.8 (1.8×) | 25.0 (2.5×) | 29.5 (3.0×) |
| add | P-521 base | 12.5 | 43.2 (3.5×) | 26.1 (2.1×) | 42.5 (3.4×) | 43.9 (3.5×) |
| sub | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BabyBear | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub | BN254 scalar | 5.1 | 16.4 (3.2×) | 11.7 (2.3×) | 15.5 (3.0×) | 19.8 (3.9×) |
| sub | secp256k1 base | 5.4 | 16.6 (3.1×) | 11.8 (2.2×) | 15.3 (2.8×) | 19.6 (3.6×) |
| sub | P-256 base | 5.3 | 15.2 (2.9×) | 11.9 (2.2×) | 15.1 (2.8×) | 18.9 (3.6×) |
| sub | P-256 scalar | 5.1 | 16.5 (3.2×) | 11.9 (2.3×) | 15.5 (3.0×) | 19.7 (3.9×) |
| sub | Tom-256 base | 5.5 | 16.2 (2.9×) | 11.8 (2.1×) | 16.0 (2.9×) | 19.4 (3.5×) |
| sub | BLS12-381 base | 10.0 | 25.5 (2.5×) | 17.9 (1.8×) | 24.1 (2.4×) | 29.5 (3.0×) |
| sub | P-521 base | 12.2 | 47.5 (3.9×) | 28.8 (2.4×) | 42.3 (3.5×) | 44.2 (3.6×) |
| mul | Mersenne-31 | 2.7 | 5.1 (1.9×) | 5.1 (1.9×) | 5.5 (2.0×) | 5.5 (2.0×) |
| mul | BabyBear | 3.3 | 7.8 (2.4×) | 5.2 (1.6×) | 7.8 (2.4×) | 5.0 (1.5×) |
| mul | Goldilocks | 3.8 | 12.1 (3.2×) | 7.0 (1.8×) | 12.4 (3.3×) | 9.0 (2.4×) |
| mul | BN254 scalar | 68.6 | 234 (3.4×) | 95.6 (1.4×) | 386 (5.6×) | 162 (2.4×) |
| mul | secp256k1 base | 44.6 | 160 (3.6×) | 63.4 (1.4×) | 289 (6.5×) | 112 (2.5×) |
| mul | P-256 base | 49.4 | 176 (3.6×) | 71.4 (1.4×) | 317 (6.4×) | 119 (2.4×) |
| mul | P-256 scalar | 75.0 | 239 (3.2×) | 85.2 (1.1×) | 342 (4.6×) | 161 (2.1×) |
| mul | Tom-256 base | 71.0 | 237 (3.3×) | 85.5 (1.2×) | 336 (4.7×) | 154 (2.2×) |
| mul | BLS12-381 base | 139 | 727 (5.2×) | 199 (1.4×) | 944 (6.8×) | 352 (2.5×) |
| mul | P-521 base | 175 | 665 (3.8×) | 253 (1.4×) | 1,293 (7.4×) | 444 (2.5×) |
| square | Mersenne-31 | 2.7 | 5.2 (1.9×) | 5.2 (1.9×) | 5.4 (2.0×) | 5.4 (2.0×) |
| square | BabyBear | 3.4 | 8.3 (2.4×) | 4.8 (1.4×) | 8.0 (2.4×) | 5.8 (1.7×) |
| square | Goldilocks | 3.8 | 10.1 (2.7×) | 6.0 (1.6×) | 12.0 (3.2×) | 9.7 (2.6×) |
| square | BN254 scalar | 63.2 | 214 (3.4×) | 93.9 (1.5×) | 375 (5.9×) | 163 (2.6×) |
| square | secp256k1 base | 33.8 | 144 (4.3×) | 61.4 (1.8×) | 217 (6.4×) | 104 (3.1×) |
| square | P-256 base | 40.0 | 147 (3.7×) | 78.4 (2.0×) | 258 (6.5×) | 129 (3.2×) |
| square | P-256 scalar | 64.1 | 214 (3.3×) | 80.5 (1.3×) | 290 (4.5×) | 163 (2.5×) |
| square | Tom-256 base | 57.0 | 208 (3.6×) | 76.3 (1.3×) | 321 (5.6×) | 148 (2.6×) |
| square | BLS12-381 base | 132 | 586 (4.4×) | 189 (1.4×) | 894 (6.8×) | 320 (2.4×) |
| square | P-521 base | 121 | 560 (4.6×) | 246 (2.0×) | 923 (7.6×) | 396 (3.3×) |
| invert | Mersenne-31 | 216 | 266 (1.2×) | 266 (1.2×) | 276 (1.3×) | 271 (1.3×) |
| invert | BabyBear | 223 | 269 (1.2×) | 268 (1.2×) | 281 (1.3×) | 271 (1.2×) |
| invert | Goldilocks | 435 | 537 (1.2×) | 538 (1.2×) | 557 (1.3×) | 563 (1.3×) |
| invert | BN254 scalar | 5,042 | 8,463 (1.7×) | 6,509 (1.3×) | 11,871 (2.4×) | 7,866 (1.6×) |
| invert | secp256k1 base | 4,917 | 8,472 (1.7×) | 6,024 (1.2×) | 11,640 (2.4×) | 8,006 (1.6×) |
| invert | P-256 base | 4,844 | 8,313 (1.7×) | 6,056 (1.3×) | 11,342 (2.3×) | 7,651 (1.6×) |
| invert | P-256 scalar | 5,048 | 8,474 (1.7×) | 6,404 (1.3×) | 12,069 (2.4×) | 8,189 (1.6×) |
| invert | Tom-256 base | 5,026 | 8,506 (1.7×) | 6,469 (1.3×) | 11,385 (2.3×) | 7,989 (1.6×) |
| invert | BLS12-381 base | 8,838 | 19,365 (2.2×) | 11,508 (1.3×) | 24,350 (2.8×) | 16,049 (1.8×) |
| invert | P-521 base | 14,234 | 33,347 (2.3×) | 18,149 (1.3×) | 48,398 (3.4×) | 29,621 (2.1×) |
| sqrt | Mersenne-31 | 135 | 329 (2.4×) | 329 (2.4×) | 330 (2.4×) | 329 (2.4×) |
| sqrt | BabyBear | 1,974 | 2,679 (1.4×) | 2,032 (1.0×) | 2,959 (1.5×) | 2,065 (1.0×) |
| sqrt | Goldilocks | 2,824 | 5,778 (2.0×) | 3,689 (1.3×) | 6,745 (2.4×) | 9,289 (3.3×) |
| sqrt | BN254 scalar | 33,878 | 108,172 (3.2×) | 46,725 (1.4×) | 198,613 (5.9×) | 82,256 (2.4×) |
| sqrt | secp256k1 base | 12,290 | 42,077 (3.4×) | 19,615 (1.6×) | 61,004 (5.0×) | 31,141 (2.5×) |
| sqrt | P-256 base | 11,937 | 44,705 (3.7×) | 19,828 (1.7×) | 78,041 (6.5×) | 31,414 (2.6×) |
| sqrt | P-256 scalar | 24,065 | 74,031 (3.1×) | 35,781 (1.5×) | 114,957 (4.8×) | 63,922 (2.7×) |
| sqrt | Tom-256 base | 21,265 | 65,540 (3.1×) | 19,242 (0.9×) | 94,543 (4.4×) | 49,342 (2.3×) |
| sqrt | BLS12-381 base | 65,900 | 315,485 (4.8×) | 87,392 (1.3×) | 420,212 (6.4×) | 203,949 (3.1×) |
| sqrt | P-521 base | 68,261 | 320,271 (4.7×) | 131,305 (1.9×) | 473,617 (6.9×) | 260,337 (3.8×) |
| mul_u64 | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.8 (1.7×) | 9.8 (1.7×) |
| mul_u64 | BabyBear | 8.9 | 17.1 (1.9×) | 11.2 (1.3×) | 20.1 (2.3×) | 15.0 (1.7×) |
| mul_u64 | Goldilocks | 7.0 | 19.0 (2.7×) | 11.0 (1.6×) | 21.1 (3.0×) | 16.7 (2.4×) |
| mul_u64 | BN254 scalar | 20.8 | 101 (4.9×) | 28.8 (1.4×) | 122 (5.9×) | 44.2 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 43.5 (4.5×) | 12.6 (1.3×) | 66.8 (6.9×) | 24.0 (2.5×) |
| mul_u64 | P-256 base | 18.9 | 72.3 (3.8×) | 27.3 (1.4×) | 98.1 (5.2×) | 41.3 (2.2×) |
| mul_u64 | P-256 scalar | 22.4 | 84.1 (3.8×) | 30.3 (1.4×) | 113 (5.0×) | 48.2 (2.2×) |
| mul_u64 | Tom-256 base | 20.5 | 82.1 (4.0×) | 30.0 (1.5×) | 115 (5.6×) | 44.6 (2.2×) |
| mul_u64 | BLS12-381 base | 28.8 | 131 (4.5×) | 44.1 (1.5×) | 194 (6.7×) | 61.6 (2.1×) |
| mul_u64 | P-521 base | 19.7 | 92.3 (4.7×) | 32.0 (1.6×) | 228 (11.6×) | 49.3 (2.5×) |
| mul_uint | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.8 (1.7×) | 9.8 (1.7×) |
| mul_uint | BabyBear | 8.9 | 17.1 (1.9×) | 11.2 (1.3×) | 20.1 (2.3×) | 15.0 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 19.2 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 16.7 (2.4×) |
| mul_uint | BN254 scalar | 103 | 434 (4.2×) | 179 (1.7×) | 523 (5.1×) | 303 (2.9×) |
| mul_uint | secp256k1 base | 56.6 | 266 (4.7×) | 114 (2.0×) | 317 (5.6×) | 164 (2.9×) |
| mul_uint | P-256 base | 102 | 353 (3.5×) | 134 (1.3×) | 422 (4.1×) | 243 (2.4×) |
| mul_uint | P-256 scalar | 110 | 426 (3.9×) | 180 (1.6×) | 478 (4.3×) | 295 (2.7×) |
| mul_uint | Tom-256 base | 108 | 411 (3.8×) | 166 (1.5×) | 493 (4.6×) | 351 (3.2×) |
| mul_uint | BLS12-381 base | 214 | 990 (4.6×) | 371 (1.7×) | 1,139 (5.3×) | 856 (4.0×) |
| mul_uint | P-521 base | 352 | 1,543 (4.4×) | 619 (1.8×) | 2,383 (6.8×) | 982 (2.8×) |
| add_u64 | Mersenne-31 | 4.6 | 8.2 (1.8×) | 8.2 (1.8×) | 9.1 (2.0×) | 9.2 (2.0×) |
| add_u64 | BabyBear | 6.0 | 10.9 (1.8×) | 8.6 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_u64 | Goldilocks | 4.7 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_u64 | BN254 scalar | 24.8 | 120 (4.8×) | 55.8 (2.2×) | 179 (7.2×) | 91.5 (3.7×) |
| add_u64 | secp256k1 base | 5.9 | 8.0 (1.4×) | 8.0 (1.4×) | 8.0 (1.4×) | 12.0 (2.0×) |
| add_u64 | P-256 base | 25.8 | 72.3 (2.8×) | 35.8 (1.4×) | 86.7 (3.4×) | 68.5 (2.7×) |
| add_u64 | P-256 scalar | 25.3 | 70.2 (2.8×) | 40.4 (1.6×) | 72.5 (2.9×) | 83.3 (3.3×) |
| add_u64 | Tom-256 base | 26.2 | 89.7 (3.4×) | 41.7 (1.6×) | 109 (4.2×) | 71.7 (2.7×) |
| add_u64 | BLS12-381 base | 38.3 | 178 (4.6×) | 66.6 (1.7×) | 262 (6.8×) | 95.3 (2.5×) |
| add_u64 | P-521 base | 10.8 | 23.1 (2.1×) | 16.1 (1.5×) | 22.3 (2.1×) | 25.1 (2.3×) |
| add_uint | Mersenne-31 | 4.6 | 8.2 (1.8×) | 8.2 (1.8×) | 9.1 (2.0×) | 9.2 (2.0×) |
| add_uint | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.6 (2.1×) |
| add_uint | Goldilocks | 4.7 | 10.0 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_uint | BN254 scalar | 87.8 | 252 (2.9×) | 119 (1.4×) | 387 (4.4×) | 190 (2.2×) |
| add_uint | secp256k1 base | 6.8 | 21.0 (3.1×) | 14.6 (2.1×) | 18.4 (2.7×) | 24.0 (3.5×) |
| add_uint | P-256 base | 61.2 | 189 (3.1×) | 79.7 (1.3×) | 327 (5.3×) | 148 (2.4×) |
| add_uint | P-256 scalar | 88.6 | 261 (2.9×) | 116 (1.3×) | 350 (4.0×) | 211 (2.4×) |
| add_uint | Tom-256 base | 87.1 | 249 (2.9×) | 109 (1.3×) | 339 (3.9×) | 198 (2.3×) |
| add_uint | BLS12-381 base | 169 | 775 (4.6×) | 205 (1.2×) | 970 (5.7×) | 419 (2.5×) |
| add_uint | P-521 base | 22.8 | 65.0 (2.9×) | 41.8 (1.8×) | 88.7 (3.9×) | 71.5 (3.1×) |
| sub_u64 | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.0 (1.5×) |
| sub_u64 | BabyBear | 6.0 | 11.1 (1.8×) | 8.5 (1.4×) | 15.8 (2.6×) | 12.6 (2.1×) |
| sub_u64 | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.4 (2.7×) | 11.0 (2.6×) |
| sub_u64 | BN254 scalar | 24.5 | 113 (4.6×) | 53.9 (2.2×) | 138 (5.6×) | 74.2 (3.0×) |
| sub_u64 | secp256k1 base | 4.2 | 8.7 (2.1×) | 8.0 (1.9×) | 7.5 (1.8×) | 11.0 (2.6×) |
| sub_u64 | P-256 base | 20.1 | 72.1 (3.6×) | 35.9 (1.8×) | 81.0 (4.0×) | 61.7 (3.1×) |
| sub_u64 | P-256 scalar | 24.8 | 69.0 (2.8×) | 40.1 (1.6×) | 75.9 (3.1×) | 82.7 (3.3×) |
| sub_u64 | Tom-256 base | 24.6 | 88.7 (3.6×) | 51.3 (2.1×) | 106 (4.3×) | 71.8 (2.9×) |
| sub_u64 | BLS12-381 base | 38.1 | 163 (4.3×) | 67.6 (1.8×) | 254 (6.7×) | 96.7 (2.5×) |
| sub_u64 | P-521 base | 9.6 | 22.2 (2.3×) | 18.8 (2.0×) | 20.3 (2.1×) | 21.2 (2.2×) |
| sub_uint | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.0 (1.5×) |
| sub_uint | BabyBear | 6.0 | 11.1 (1.8×) | 8.5 (1.4×) | 15.8 (2.6×) | 12.5 (2.1×) |
| sub_uint | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.4 (2.7×) | 11.0 (2.6×) |
| sub_uint | BN254 scalar | 87.8 | 253 (2.9×) | 118 (1.3×) | 384 (4.4×) | 191 (2.2×) |
| sub_uint | secp256k1 base | 5.4 | 21.2 (3.9×) | 14.6 (2.7×) | 18.2 (3.4×) | 24.2 (4.5×) |
| sub_uint | P-256 base | 59.8 | 193 (3.2×) | 88.0 (1.5×) | 359 (6.0×) | 150 (2.5×) |
| sub_uint | P-256 scalar | 86.3 | 267 (3.1×) | 116 (1.3×) | 357 (4.1×) | 212 (2.5×) |
| sub_uint | Tom-256 base | 83.7 | 254 (3.0×) | 114 (1.4×) | 339 (4.1×) | 202 (2.4×) |
| sub_uint | BLS12-381 base | 168 | 758 (4.5×) | 212 (1.3×) | 949 (5.6×) | 402 (2.4×) |
| sub_uint | P-521 base | 20.2 | 75.9 (3.8×) | 42.3 (2.1×) | 81.1 (4.0×) | 69.0 (3.4×) |
| from_u64 | Mersenne-31 | 3.2 | 4.7 (1.5×) | 4.7 (1.5×) | 5.2 (1.6×) | 5.3 (1.7×) |
| from_u64 | BabyBear | 6.2 | 11.1 (1.8×) | 7.3 (1.2×) | 12.9 (2.1×) | 10.3 (1.7×) |
| from_u64 | Goldilocks | 3.3 | 7.2 (2.2×) | 5.5 (1.7×) | 9.0 (2.7×) | 8.7 (2.6×) |
| from_u64 | BN254 scalar | 22.0 | 87.0 (4.0×) | 28.2 (1.3×) | 121 (5.5×) | 45.0 (2.0×) |
| from_u64 | secp256k1 base | 2.2 | 3.8 (1.7×) | 3.8 (1.7×) | 3.8 (1.7×) | 3.8 (1.7×) |
| from_u64 | P-256 base | 17.6 | 51.4 (2.9×) | 23.7 (1.3×) | 61.4 (3.5×) | 36.1 (2.1×) |
| from_u64 | P-256 scalar | 18.0 | 51.5 (2.9×) | 25.9 (1.4×) | 59.2 (3.3×) | 43.1 (2.4×) |
| from_u64 | Tom-256 base | 21.1 | 72.0 (3.4×) | 28.6 (1.4×) | 85.7 (4.1×) | 43.4 (2.1×) |
| from_u64 | BLS12-381 base | 29.1 | 141 (4.8×) | 44.6 (1.5×) | 234 (8.0×) | 63.1 (2.2×) |
| from_u64 | P-521 base | 5.0 | 6.1 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.1 (1.2×) |
| div_u64 | BN254 scalar | 615 | 1,214 (2.0×) | 821 (1.3×) | 1,419 (2.3×) | 1,049 (1.7×) |
| div_u64 | secp256k1 base | 563 | 1,000 (1.8×) | 766 (1.4×) | 1,217 (2.2×) | 877 (1.6×) |
| div_u64 | P-256 base | 601 | 1,089 (1.8×) | 808 (1.3×) | 1,356 (2.3×) | 987 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,185 (1.9×) | 864 (1.4×) | 1,405 (2.3×) | 1,152 (1.9×) |
| div_u64 | Tom-256 base | 613 | 1,188 (1.9×) | 864 (1.4×) | 1,478 (2.4×) | 1,036 (1.7×) |
| div_u64 | BLS12-381 base | 751 | 1,795 (2.4×) | 1,033 (1.4×) | 2,510 (3.3×) | 1,750 (2.3×) |
| div_u64 | P-521 base | 927 | 2,450 (2.6×) | 1,369 (1.5×) | 3,622 (3.9×) | 1,878 (2.0×) |
| div_uint | Mersenne-31 | 220 | 276 (1.3×) | 278 (1.3×) | 281 (1.3×) | 287 (1.3×) |
| div_uint | BabyBear | 217 | 277 (1.3×) | 278 (1.3×) | 290 (1.3×) | 283 (1.3×) |
| div_uint | Goldilocks | 440 | 551 (1.3×) | 545 (1.2×) | 593 (1.3×) | 596 (1.4×) |
| div_uint | BN254 scalar | 5,030 | 9,221 (1.8×) | 6,797 (1.4×) | 12,816 (2.5×) | 8,430 (1.7×) |
| div_uint | secp256k1 base | 4,880 | 8,563 (1.8×) | 6,022 (1.2×) | 11,918 (2.4×) | 8,094 (1.7×) |
| div_uint | P-256 base | 5,136 | 8,830 (1.7×) | 6,165 (1.2×) | 12,053 (2.3×) | 7,932 (1.5×) |
| div_uint | P-256 scalar | 4,966 | 9,235 (1.9×) | 6,623 (1.3×) | 12,560 (2.5×) | 8,739 (1.8×) |
| div_uint | Tom-256 base | 5,047 | 9,148 (1.8×) | 6,658 (1.3×) | 12,378 (2.5×) | 8,327 (1.6×) |
| div_uint | BLS12-381 base | 8,963 | 20,940 (2.3×) | 12,042 (1.3×) | 26,464 (3.0×) | 16,866 (1.9×) |
| div_uint | P-521 base | 14,948 | 34,250 (2.3×) | 18,513 (1.2×) | 50,018 (3.3×) | 29,895 (2.0×) |
| add, constant time | Mersenne-31 | 1.9 | 4.2 (2.2×) | 4.2 (2.2×) | 4.4 (2.3×) | 4.4 (2.3×) |
| add, constant time | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add, constant time | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.1 (1.6×) | 3.1 (1.6×) |
| add, constant time | BN254 scalar | 5.0 | 23.6 (4.7×) | 16.3 (3.3×) | 20.7 (4.1×) | 27.1 (5.4×) |
| add, constant time | secp256k1 base | 5.8 | 26.3 (4.5×) | 18.1 (3.1×) | 24.5 (4.2×) | 29.0 (5.0×) |
| add, constant time | P-256 base | 6.0 | 23.6 (3.9×) | 18.1 (3.0×) | 22.9 (3.8×) | 28.0 (4.7×) |
| add, constant time | P-256 scalar | 5.9 | 27.1 (4.6×) | 17.9 (3.0×) | 24.1 (4.1×) | 29.5 (5.0×) |
| add, constant time | Tom-256 base | 6.0 | 27.6 (4.6×) | 18.0 (3.0×) | 23.8 (4.0×) | 28.5 (4.8×) |
| add, constant time | BLS12-381 base | 9.9 | 42.7 (4.3×) | 25.6 (2.6×) | 35.7 (3.6×) | 41.2 (4.2×) |
| add, constant time | P-521 base | 12.7 | 76.6 (6.0×) | 46.6 (3.7×) | 84.8 (6.7×) | 74.1 (5.8×) |
| sub, constant time | Mersenne-31 | 2.0 | 2.5 (1.2×) | 2.5 (1.2×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BabyBear | 2.0 | 2.5 (1.2×) | 2.5 (1.2×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub, constant time | BN254 scalar | 5.1 | 22.9 (4.5×) | 16.5 (3.2×) | 21.4 (4.2×) | 26.0 (5.1×) |
| sub, constant time | secp256k1 base | 5.3 | 19.2 (3.6×) | 14.4 (2.7×) | 18.7 (3.5×) | 22.5 (4.2×) |
| sub, constant time | P-256 base | 5.3 | 17.3 (3.3×) | 13.4 (2.5×) | 18.5 (3.5×) | 22.0 (4.2×) |
| sub, constant time | P-256 scalar | 5.0 | 21.0 (4.2×) | 14.7 (2.9×) | 22.1 (4.4×) | 23.5 (4.7×) |
| sub, constant time | Tom-256 base | 5.5 | 21.1 (3.8×) | 15.2 (2.8×) | 21.0 (3.8×) | 24.0 (4.4×) |
| sub, constant time | BLS12-381 base | 9.9 | 37.7 (3.8×) | 24.4 (2.5×) | 35.3 (3.6×) | 37.3 (3.8×) |
| sub, constant time | P-521 base | 12.1 | 56.3 (4.7×) | 37.2 (3.1×) | 58.2 (4.8×) | 53.1 (4.4×) |
| mul, constant time | Mersenne-31 | 2.7 | 5.2 (1.9×) | 5.1 (1.9×) | 5.5 (2.0×) | 5.5 (2.0×) |
| mul, constant time | BabyBear | 3.4 | 7.8 (2.3×) | 5.2 (1.5×) | 7.8 (2.3×) | 5.0 (1.5×) |
| mul, constant time | Goldilocks | 3.8 | 12.1 (3.2×) | 7.0 (1.8×) | 12.4 (3.3×) | 9.0 (2.4×) |
| mul, constant time | BN254 scalar | 67.8 | 236 (3.5×) | 86.7 (1.3×) | 418 (6.2×) | 151 (2.2×) |
| mul, constant time | secp256k1 base | 44.6 | 160 (3.6×) | 63.6 (1.4×) | 275 (6.2×) | 108 (2.4×) |
| mul, constant time | P-256 base | 49.2 | 173 (3.5×) | 71.8 (1.5×) | 321 (6.5×) | 121 (2.5×) |
| mul, constant time | P-256 scalar | 74.8 | 244 (3.3×) | 85.4 (1.1×) | 343 (4.6×) | 168 (2.2×) |
| mul, constant time | Tom-256 base | 71.0 | 235 (3.3×) | 85.8 (1.2×) | 331 (4.7×) | 157 (2.2×) |
| mul, constant time | BLS12-381 base | 142 | 728 (5.1×) | 184 (1.3×) | 935 (6.6×) | 337 (2.4×) |
| mul, constant time | P-521 base | 173 | 666 (3.8×) | 252 (1.5×) | 1,287 (7.4×) | 444 (2.6×) |
| square, constant time | Mersenne-31 | 2.7 | 5.2 (1.9×) | 5.2 (1.9×) | 5.4 (2.0×) | 5.4 (2.0×) |
| square, constant time | BabyBear | 3.7 | 8.3 (2.2×) | 4.9 (1.3×) | 8.0 (2.2×) | 5.8 (1.6×) |
| square, constant time | Goldilocks | 3.8 | 10.1 (2.7×) | 6.0 (1.6×) | 12.0 (3.2×) | 9.7 (2.6×) |
| square, constant time | BN254 scalar | 65.0 | 213 (3.3×) | 78.1 (1.2×) | 370 (5.7×) | 147 (2.3×) |
| square, constant time | secp256k1 base | 33.9 | 143 (4.2×) | 61.3 (1.8×) | 217 (6.4×) | 98.1 (2.9×) |
| square, constant time | P-256 base | 40.2 | 151 (3.8×) | 78.4 (2.0×) | 253 (6.3×) | 131 (3.3×) |
| square, constant time | P-256 scalar | 63.9 | 214 (3.3×) | 80.7 (1.3×) | 288 (4.5×) | 140 (2.2×) |
| square, constant time | Tom-256 base | 57.0 | 203 (3.6×) | 77.2 (1.4×) | 321 (5.6×) | 166 (2.9×) |
| square, constant time | BLS12-381 base | 137 | 582 (4.2×) | 172 (1.3×) | 886 (6.5×) | 308 (2.2×) |
| square, constant time | P-521 base | 123 | 563 (4.6×) | 246 (2.0×) | 917 (7.5×) | 398 (3.2×) |
| invert, constant time | Mersenne-31 | 248 | 438 (1.8×) | 441 (1.8×) | 431 (1.7×) | 430 (1.7×) |
| invert, constant time | BabyBear | 383 | 520 (1.4×) | 458 (1.2×) | 550 (1.4×) | 417 (1.1×) |
| invert, constant time | Goldilocks | 645 | 1,263 (2.0×) | 858 (1.3×) | 1,564 (2.4×) | 1,183 (1.8×) |
| invert, constant time | BN254 scalar | 6,760 | 12,015 (1.8×) | 9,244 (1.4×) | 16,857 (2.5×) | 12,578 (1.9×) |
| invert, constant time | secp256k1 base | 6,858 | 11,831 (1.7×) | 9,093 (1.3×) | 16,945 (2.5×) | 12,668 (1.8×) |
| invert, constant time | P-256 base | 6,512 | 11,533 (1.8×) | 8,967 (1.4×) | 16,307 (2.5×) | 12,319 (1.9×) |
| invert, constant time | P-256 scalar | 6,917 | 11,904 (1.7×) | 9,379 (1.4×) | 17,148 (2.5×) | 12,779 (1.8×) |
| invert, constant time | Tom-256 base | 6,974 | 11,773 (1.7×) | 9,286 (1.3×) | 16,992 (2.4×) | 12,767 (1.8×) |
| invert, constant time | BLS12-381 base | 12,573 | 23,954 (1.9×) | 16,698 (1.3×) | 32,822 (2.6×) | 24,072 (1.9×) |
| invert, constant time | P-521 base | 20,025 | 41,739 (2.1×) | 26,139 (1.3×) | 63,686 (3.2×) | 41,552 (2.1×) |
| sqrt, constant time | Mersenne-31 | 142 | 334 (2.4×) | 334 (2.4×) | 341 (2.4×) | 340 (2.4×) |
| sqrt, constant time | BabyBear | 5,592 | 7,468 (1.3×) | 6,464 (1.2×) | 7,850 (1.4×) | 6,531 (1.2×) |
| sqrt, constant time | Goldilocks | 7,152 | 12,879 (1.8×) | 8,921 (1.2×) | 15,858 (2.2×) | 21,585 (3.0×) |
| sqrt, constant time | BN254 scalar | 58,580 | 178,697 (3.1×) | 83,897 (1.4×) | 297,160 (5.1×) | 152,719 (2.6×) |
| sqrt, constant time | secp256k1 base | 12,273 | 42,420 (3.5×) | 19,829 (1.6×) | 70,690 (5.8×) | 31,261 (2.5×) |
| sqrt, constant time | P-256 base | 11,984 | 44,657 (3.7×) | 19,094 (1.6×) | 79,907 (6.7×) | 32,459 (2.7×) |
| sqrt, constant time | P-256 scalar | 25,624 | 78,912 (3.1×) | 37,108 (1.4×) | 121,136 (4.7×) | 68,607 (2.7×) |
| sqrt, constant time | Tom-256 base | 20,056 | 65,867 (3.3×) | 19,373 (1.0×) | 94,906 (4.7×) | 50,847 (2.5×) |
| sqrt, constant time | BLS12-381 base | 69,780 | 314,938 (4.5×) | 97,430 (1.4×) | 423,132 (6.1×) | 214,113 (3.1×) |
| sqrt, constant time | P-521 base | 68,425 | 324,637 (4.7×) | 129,843 (1.9×) | 507,844 (7.4×) | 266,374 (3.9×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.8 (1.7×) | 9.8 (1.7×) |
| mul_u64, constant time | BabyBear | 8.9 | 17.1 (1.9×) | 11.2 (1.3×) | 20.0 (2.2×) | 15.0 (1.7×) |
| mul_u64, constant time | Goldilocks | 6.9 | 19.0 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 16.7 (2.4×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.9 (1.8×) | 125 (5.7×) | 57.3 (2.6×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 61.6 (5.6×) | 24.9 (2.3×) | 80.1 (7.3×) | 37.3 (3.4×) |
| mul_u64, constant time | P-256 base | 24.4 | 85.5 (3.5×) | 40.3 (1.7×) | 103 (4.2×) | 57.2 (2.3×) |
| mul_u64, constant time | P-256 scalar | 22.9 | 102 (4.5×) | 43.6 (1.9×) | 122 (5.3×) | 62.7 (2.7×) |
| mul_u64, constant time | Tom-256 base | 22.4 | 95.3 (4.3×) | 41.8 (1.9×) | 123 (5.5×) | 63.4 (2.8×) |
| mul_u64, constant time | BLS12-381 base | 34.7 | 149 (4.3×) | 65.7 (1.9×) | 160 (4.6×) | 85.9 (2.5×) |
| mul_u64, constant time | P-521 base | 26.4 | 138 (5.2×) | 69.2 (2.6×) | 138 (5.2×) | 96.0 (3.6×) |
| mul_uint, constant time | Mersenne-31 | 5.9 | 9.4 (1.6×) | 9.4 (1.6×) | 9.7 (1.6×) | 9.8 (1.7×) |
| mul_uint, constant time | BabyBear | 9.0 | 17.1 (1.9×) | 11.2 (1.2×) | 20.1 (2.2×) | 15.0 (1.7×) |
| mul_uint, constant time | Goldilocks | 6.9 | 19.1 (2.8×) | 11.0 (1.6×) | 21.1 (3.1×) | 16.8 (2.4×) |
| mul_uint, constant time | BN254 scalar | 120 | 484 (4.0×) | 173 (1.4×) | 599 (5.0×) | 312 (2.6×) |
| mul_uint, constant time | secp256k1 base | 78.2 | 322 (4.1×) | 192 (2.5×) | 381 (4.9×) | 251 (3.2×) |
| mul_uint, constant time | P-256 base | 119 | 413 (3.5×) | 136 (1.1×) | 492 (4.1×) | 233 (2.0×) |
| mul_uint, constant time | P-256 scalar | 132 | 473 (3.6×) | 186 (1.4×) | 562 (4.3×) | 304 (2.3×) |
| mul_uint, constant time | Tom-256 base | 122 | 467 (3.8×) | 163 (1.3×) | 553 (4.5×) | 293 (2.4×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,066 (4.2×) | 369 (1.4×) | 1,678 (6.6×) | 918 (3.6×) |
| mul_uint, constant time | P-521 base | 427 | 1,989 (4.7×) | 975 (2.3×) | 2,921 (6.8×) | 1,477 (3.5×) |
| add_u64, constant time | Mersenne-31 | 4.6 | 8.2 (1.8×) | 8.1 (1.8×) | 9.1 (2.0×) | 9.2 (2.0×) |
| add_u64, constant time | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.6 (2.1×) |
| add_u64, constant time | Goldilocks | 4.7 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.4×) | 12.0 (2.6×) |
| add_u64, constant time | BN254 scalar | 26.9 | 136 (5.1×) | 78.3 (2.9×) | 163 (6.1×) | 81.4 (3.0×) |
| add_u64, constant time | secp256k1 base | 5.9 | 20.5 (3.5×) | 16.1 (2.7×) | 19.5 (3.3×) | 25.1 (4.3×) |
| add_u64, constant time | P-256 base | 25.7 | 90.4 (3.5×) | 57.7 (2.2×) | 103 (4.0×) | 73.0 (2.8×) |
| add_u64, constant time | P-256 scalar | 27.1 | 90.2 (3.3×) | 56.4 (2.1×) | 100 (3.7×) | 93.4 (3.4×) |
| add_u64, constant time | Tom-256 base | 27.4 | 116 (4.2×) | 61.5 (2.2×) | 132 (4.8×) | 93.8 (3.4×) |
| add_u64, constant time | BLS12-381 base | 43.8 | 211 (4.8×) | 92.1 (2.1×) | 268 (6.1×) | 125 (2.9×) |
| add_u64, constant time | P-521 base | 10.8 | 64.3 (6.0×) | 44.3 (4.1×) | 65.7 (6.1×) | 64.1 (5.9×) |
| add_uint, constant time | Mersenne-31 | 4.6 | 8.1 (1.8×) | 8.2 (1.8×) | 9.1 (2.0×) | 9.1 (2.0×) |
| add_uint, constant time | BabyBear | 6.0 | 10.9 (1.8×) | 8.6 (1.4×) | 13.0 (2.2×) | 12.6 (2.1×) |
| add_uint, constant time | Goldilocks | 4.8 | 9.9 (2.1×) | 7.7 (1.6×) | 11.1 (2.3×) | 12.0 (2.5×) |
| add_uint, constant time | BN254 scalar | 92.8 | 249 (2.7×) | 99.1 (1.1×) | 344 (3.7×) | 182 (2.0×) |
| add_uint, constant time | secp256k1 base | 9.3 | 41.7 (4.5×) | 29.4 (3.2×) | 37.6 (4.0×) | 44.5 (4.8×) |
| add_uint, constant time | P-256 base | 70.3 | 177 (2.5×) | 98.7 (1.4×) | 286 (4.1×) | 150 (2.1×) |
| add_uint, constant time | P-256 scalar | 95.8 | 256 (2.7×) | 106 (1.1×) | 316 (3.3×) | 202 (2.1×) |
| add_uint, constant time | Tom-256 base | 84.3 | 251 (3.0×) | 102 (1.2×) | 329 (3.9×) | 187 (2.2×) |
| add_uint, constant time | BLS12-381 base | 176 | 770 (4.4×) | 209 (1.2×) | 949 (5.4×) | 411 (2.3×) |
| add_uint, constant time | P-521 base | 22.4 | 138 (6.2×) | 105 (4.7×) | 155 (6.9×) | 138 (6.2×) |
| sub_u64, constant time | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.0 (1.5×) |
| sub_u64, constant time | BabyBear | 6.0 | 10.8 (1.8×) | 8.4 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.4 (2.7×) | 11.0 (2.6×) |
| sub_u64, constant time | BN254 scalar | 27.9 | 126 (4.5×) | 60.0 (2.2×) | 155 (5.6×) | 80.0 (2.9×) |
| sub_u64, constant time | secp256k1 base | 4.2 | 15.5 (3.7×) | 12.3 (2.9×) | 14.6 (3.5×) | 18.5 (4.4×) |
| sub_u64, constant time | P-256 base | 23.3 | 84.8 (3.6×) | 51.9 (2.2×) | 97.8 (4.2×) | 67.7 (2.9×) |
| sub_u64, constant time | P-256 scalar | 26.5 | 80.6 (3.0×) | 52.7 (2.0×) | 93.6 (3.5×) | 79.2 (3.0×) |
| sub_u64, constant time | Tom-256 base | 28.1 | 113 (4.0×) | 58.9 (2.1×) | 124 (4.4×) | 80.1 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 42.6 | 187 (4.4×) | 87.6 (2.1×) | 250 (5.9×) | 118 (2.8×) |
| sub_u64, constant time | P-521 base | 9.6 | 42.4 (4.4×) | 34.4 (3.6×) | 52.4 (5.5×) | 44.5 (4.6×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 6.2 (1.3×) | 6.2 (1.3×) | 7.0 (1.5×) | 7.0 (1.5×) |
| sub_uint, constant time | BabyBear | 6.1 | 10.9 (1.8×) | 8.5 (1.4×) | 13.0 (2.1×) | 12.5 (2.0×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.9 (2.3×) | 7.0 (1.6×) | 11.4 (2.7×) | 11.0 (2.6×) |
| sub_uint, constant time | BN254 scalar | 91.4 | 262 (2.9×) | 101 (1.1×) | 347 (3.8×) | 188 (2.1×) |
| sub_uint, constant time | secp256k1 base | 7.2 | 33.5 (4.7×) | 26.1 (3.6×) | 34.4 (4.8×) | 38.5 (5.3×) |
| sub_uint, constant time | P-256 base | 64.8 | 178 (2.7×) | 86.4 (1.3×) | 275 (4.2×) | 139 (2.1×) |
| sub_uint, constant time | P-256 scalar | 90.8 | 250 (2.8×) | 106 (1.2×) | 328 (3.6×) | 184 (2.0×) |
| sub_uint, constant time | Tom-256 base | 91.4 | 253 (2.8×) | 101 (1.1×) | 317 (3.5×) | 188 (2.1×) |
| sub_uint, constant time | BLS12-381 base | 175 | 762 (4.4×) | 207 (1.2×) | 942 (5.4×) | 375 (2.1×) |
| sub_uint, constant time | P-521 base | 19.1 | 127 (6.6×) | 86.8 (4.5×) | 126 (6.6×) | 113 (5.9×) |
| from_u64, constant time | Mersenne-31 | 3.3 | 4.7 (1.4×) | 4.7 (1.4×) | 5.2 (1.6×) | 5.3 (1.6×) |
| from_u64, constant time | BabyBear | 10.3 | 13.5 (1.3×) | 10.3 (1.0×) | 14.0 (1.4×) | 12.7 (1.2×) |
| from_u64, constant time | Goldilocks | 3.4 | 15.0 (4.4×) | 9.3 (2.7×) | 17.4 (5.1×) | 12.5 (3.7×) |
| from_u64, constant time | BN254 scalar | 60.6 | 201 (3.3×) | 80.9 (1.3×) | 275 (4.5×) | 119 (2.0×) |
| from_u64, constant time | secp256k1 base | 2.2 | 3.4 (1.5×) | 3.4 (1.5×) | 3.4 (1.5×) | 3.4 (1.5×) |
| from_u64, constant time | P-256 base | 30.6 | 178 (5.8×) | 50.0 (1.6×) | 210 (6.9×) | 84.1 (2.7×) |
| from_u64, constant time | P-256 scalar | 62.7 | 255 (4.1×) | 81.8 (1.3×) | 268 (4.3×) | 118 (1.9×) |
| from_u64, constant time | Tom-256 base | 55.8 | 253 (4.5×) | 79.1 (1.4×) | 271 (4.9×) | 114 (2.0×) |
| from_u64, constant time | BLS12-381 base | 142 | 431 (3.0×) | 155 (1.1×) | 607 (4.3×) | 273 (1.9×) |
| from_u64, constant time | P-521 base | 5.0 | 6.1 (1.2×) | 6.0 (1.2×) | 6.1 (1.2×) | 6.0 (1.2×) |
| div_u64, constant time | BN254 scalar | 6,930 | 12,376 (1.8×) | 9,508 (1.4×) | 17,577 (2.5×) | 12,966 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,782 | 11,879 (1.8×) | 9,055 (1.3×) | 17,081 (2.5×) | 12,739 (1.9×) |
| div_u64, constant time | P-256 base | 6,695 | 11,796 (1.8×) | 9,146 (1.4×) | 17,486 (2.6×) | 12,560 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,905 | 12,320 (1.8×) | 9,427 (1.4×) | 17,554 (2.5×) | 13,160 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,409 | 12,206 (1.6×) | 9,479 (1.3×) | 16,908 (2.3×) | 12,938 (1.7×) |
| div_u64, constant time | BLS12-381 base | 12,404 | 24,806 (2.0×) | 17,035 (1.4×) | 35,168 (2.8×) | 24,433 (2.0×) |
| div_u64, constant time | P-521 base | 20,372 | 42,369 (2.1×) | 25,993 (1.3×) | 64,051 (3.1×) | 42,942 (2.1×) |
| div_uint, constant time | Mersenne-31 | 267 | 455 (1.7×) | 455 (1.7×) | 469 (1.8×) | 465 (1.7×) |
| div_uint, constant time | BabyBear | 408 | 546 (1.3×) | 466 (1.1×) | 612 (1.5×) | 460 (1.1×) |
| div_uint, constant time | Goldilocks | 707 | 1,308 (1.9×) | 950 (1.3×) | 1,576 (2.2×) | 1,211 (1.7×) |
| div_uint, constant time | BN254 scalar | 7,010 | 12,579 (1.8×) | 9,691 (1.4×) | 17,813 (2.5×) | 13,321 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,807 | 11,955 (1.8×) | 9,092 (1.3×) | 17,052 (2.5×) | 12,806 (1.9×) |
| div_uint, constant time | P-256 base | 6,719 | 12,024 (1.8×) | 9,118 (1.4×) | 17,545 (2.6×) | 12,681 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,954 | 12,378 (1.8×) | 9,541 (1.4×) | 17,746 (2.6×) | 13,496 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,337 | 12,457 (1.7×) | 9,428 (1.3×) | 17,276 (2.4×) | 13,089 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,513 | 25,374 (2.0×) | 17,063 (1.4×) | 35,743 (2.9×) | 24,870 (2.0×) |
| div_uint, constant time | P-521 base | 20,756 | 42,567 (2.1×) | 26,008 (1.3×) | 65,187 (3.1×) | 43,336 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 7.5 (2.6×) | 7.5 (2.6×) | 7.7 (2.7×) | 7.7 (2.7×) |
| add | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.8 | 14.6 (2.1×) | 10.9 (1.6×) | 15.6 (2.3×) | 17.4 (2.6×) |
| add | secp256k1 base | 6.9 | 13.7 (2.0×) | 10.7 (1.6×) | 13.3 (1.9×) | 18.1 (2.6×) |
| add | P-256 base | 6.9 | 12.9 (1.9×) | 11.2 (1.6×) | 13.8 (2.0×) | 17.9 (2.6×) |
| add | P-256 scalar | 7.0 | 14.4 (2.1×) | 11.1 (1.6×) | 15.4 (2.2×) | 18.9 (2.7×) |
| add | Tom-256 base | 6.9 | 14.3 (2.1×) | 11.2 (1.6×) | 15.0 (2.2×) | 18.4 (2.7×) |
| add | BLS12-381 base | 8.9 | 24.9 (2.8×) | 18.7 (2.1×) | 24.7 (2.8×) | 29.8 (3.3×) |
| add | P-521 base | 15.1 | 42.4 (2.8×) | 27.7 (1.8×) | 47.5 (3.1×) | 47.5 (3.1×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.8 | 14.5 (2.1×) | 10.7 (1.6×) | 13.8 (2.0×) | 18.5 (2.7×) |
| sub | secp256k1 base | 8.0 | 16.4 (2.0×) | 10.7 (1.3×) | 13.1 (1.6×) | 17.8 (2.2×) |
| sub | P-256 base | 7.0 | 14.2 (2.0×) | 10.4 (1.5×) | 12.9 (1.8×) | 17.4 (2.5×) |
| sub | P-256 scalar | 7.0 | 15.3 (2.2×) | 10.7 (1.5×) | 13.7 (2.0×) | 17.9 (2.6×) |
| sub | Tom-256 base | 8.1 | 15.0 (1.9×) | 10.8 (1.3×) | 14.0 (1.7×) | 18.5 (2.3×) |
| sub | BLS12-381 base | 9.9 | 25.3 (2.6×) | 17.5 (1.8×) | 24.4 (2.5×) | 30.6 (3.1×) |
| sub | P-521 base | 12.5 | 44.9 (3.6×) | 26.8 (2.1×) | 43.3 (3.5×) | 45.5 (3.6×) |
| mul | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| mul | BabyBear | 10.9 | 13.0 (1.2×) | 11.0 (1.0×) | 14.5 (1.3×) | 11.0 (1.0×) |
| mul | Goldilocks | 9.0 | 18.0 (2.0×) | 14.4 (1.6×) | 21.0 (2.3×) | 14.0 (1.6×) |
| mul | BN254 scalar | 72.4 | 227 (3.1×) | 90.9 (1.3×) | 368 (5.1×) | 152 (2.1×) |
| mul | secp256k1 base | 54.8 | 173 (3.2×) | 77.6 (1.4×) | 250 (4.6×) | 106 (1.9×) |
| mul | P-256 base | 60.0 | 190 (3.2×) | 72.3 (1.2×) | 315 (5.2×) | 121 (2.0×) |
| mul | P-256 scalar | 84.0 | 239 (2.8×) | 93.4 (1.1×) | 325 (3.9×) | 164 (2.0×) |
| mul | Tom-256 base | 76.0 | 239 (3.1×) | 84.9 (1.1×) | 332 (4.4×) | 159 (2.1×) |
| mul | BLS12-381 base | 141 | 719 (5.1×) | 185 (1.3×) | 917 (6.5×) | 337 (2.4×) |
| mul | P-521 base | 189 | 686 (3.6×) | 247 (1.3×) | 1,292 (6.8×) | 446 (2.4×) |
| square | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| square | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 11.0 (1.0×) |
| square | Goldilocks | 9.0 | 17.3 (1.9×) | 12.0 (1.3×) | 21.3 (2.4×) | 14.4 (1.6×) |
| square | BN254 scalar | 66.5 | 208 (3.1×) | 89.9 (1.4×) | 365 (5.5×) | 142 (2.1×) |
| square | secp256k1 base | 42.9 | 154 (3.6×) | 69.9 (1.6×) | 183 (4.3×) | 95.5 (2.2×) |
| square | P-256 base | 44.0 | 155 (3.5×) | 62.2 (1.4×) | 213 (4.8×) | 114 (2.6×) |
| square | P-256 scalar | 73.0 | 223 (3.1×) | 89.1 (1.2×) | 240 (3.3×) | 150 (2.1×) |
| square | Tom-256 base | 64.5 | 201 (3.1×) | 83.0 (1.3×) | 236 (3.7×) | 142 (2.2×) |
| square | BLS12-381 base | 137 | 609 (4.4×) | 179 (1.3×) | 870 (6.4×) | 294 (2.1×) |
| square | P-521 base | 133 | 571 (4.3×) | 210 (1.6×) | 913 (6.9×) | 390 (2.9×) |
| mul_u64 | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_u64 | BabyBear | 11.0 | 17.0 (1.5×) | 11.1 (1.0×) | 19.3 (1.8×) | 13.1 (1.2×) |
| mul_u64 | Goldilocks | 9.1 | 20.4 (2.2×) | 12.0 (1.3×) | 25.0 (2.7×) | 17.2 (1.9×) |
| mul_u64 | BN254 scalar | 25.9 | 91.3 (3.5×) | 34.5 (1.3×) | 120 (4.6×) | 45.4 (1.8×) |
| mul_u64 | secp256k1 base | 14.9 | 49.0 (3.3×) | 16.0 (1.1×) | 65.1 (4.4×) | 23.0 (1.5×) |
| mul_u64 | P-256 base | 24.9 | 74.9 (3.0×) | 33.0 (1.3×) | 89.1 (3.6×) | 42.4 (1.7×) |
| mul_u64 | P-256 scalar | 27.8 | 82.2 (3.0×) | 34.6 (1.2×) | 106 (3.8×) | 48.1 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.9 (3.0×) | 32.9 (1.2×) | 105 (3.8×) | 45.4 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 125 (3.6×) | 47.9 (1.4×) | 183 (5.3×) | 62.1 (1.8×) |
| mul_u64 | P-521 base | 32.8 | 95.7 (2.9×) | 39.5 (1.2×) | 191 (5.8×) | 51.8 (1.6×) |
| mul_uint | Mersenne-31 | 9.0 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_uint | BabyBear | 10.9 | 17.0 (1.6×) | 11.0 (1.0×) | 19.3 (1.8×) | 13.1 (1.2×) |
| mul_uint | Goldilocks | 8.8 | 20.5 (2.3×) | 12.1 (1.4×) | 25.0 (2.8×) | 17.2 (2.0×) |
| mul_uint | BN254 scalar | 114 | 439 (3.9×) | 173 (1.5×) | 533 (4.7×) | 294 (2.6×) |
| mul_uint | secp256k1 base | 68.3 | 265 (3.9×) | 117 (1.7×) | 319 (4.7×) | 164 (2.4×) |
| mul_uint | P-256 base | 110 | 356 (3.2×) | 138 (1.3×) | 424 (3.9×) | 233 (2.1×) |
| mul_uint | P-256 scalar | 116 | 425 (3.7×) | 178 (1.5×) | 474 (4.1×) | 301 (2.6×) |
| mul_uint | Tom-256 base | 114 | 411 (3.6×) | 172 (1.5×) | 497 (4.4×) | 324 (2.8×) |
| mul_uint | BLS12-381 base | 222 | 956 (4.3×) | 357 (1.6×) | 1,134 (5.1×) | 832 (3.7×) |
| mul_uint | P-521 base | 407 | 1,566 (3.8×) | 643 (1.6×) | 2,382 (5.9×) | 989 (2.4×) |
| add_u64 | Mersenne-31 | 4.5 | 8.7 (1.9×) | 8.7 (1.9×) | 9.3 (2.1×) | 9.3 (2.1×) |
| add_u64 | BabyBear | 7.4 | 12.2 (1.6×) | 8.2 (1.1×) | 12.7 (1.7×) | 11.0 (1.5×) |
| add_u64 | Goldilocks | 4.4 | 9.1 (2.1×) | 7.3 (1.7×) | 10.8 (2.5×) | 10.0 (2.3×) |
| add_u64 | BN254 scalar | 24.9 | 117 (4.7×) | 44.3 (1.8×) | 142 (5.7×) | 65.5 (2.6×) |
| add_u64 | secp256k1 base | 7.4 | 7.0 (0.9×) | 6.0 (0.8×) | 7.0 (0.9×) | 10.0 (1.4×) |
| add_u64 | P-256 base | 22.1 | 68.6 (3.1×) | 34.9 (1.6×) | 77.7 (3.5×) | 55.7 (2.5×) |
| add_u64 | P-256 scalar | 22.7 | 69.3 (3.1×) | 38.4 (1.7×) | 72.9 (3.2×) | 65.1 (2.9×) |
| add_u64 | Tom-256 base | 26.0 | 88.0 (3.4×) | 39.8 (1.5×) | 98.4 (3.8×) | 65.8 (2.5×) |
| add_u64 | BLS12-381 base | 38.7 | 160 (4.1×) | 62.4 (1.6×) | 270 (7.0×) | 89.0 (2.3×) |
| add_u64 | P-521 base | 12.8 | 21.2 (1.7×) | 12.5 (1.0×) | 19.4 (1.5×) | 27.7 (2.2×) |
| add_uint | Mersenne-31 | 4.5 | 8.6 (1.9×) | 8.6 (1.9×) | 9.3 (2.1×) | 9.3 (2.1×) |
| add_uint | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.4 | 9.1 (2.1×) | 7.3 (1.7×) | 10.8 (2.5×) | 10.1 (2.3×) |
| add_uint | BN254 scalar | 83.8 | 230 (2.7×) | 104 (1.2×) | 382 (4.6×) | 174 (2.1×) |
| add_uint | secp256k1 base | 8.5 | 17.0 (2.0×) | 12.5 (1.5×) | 16.7 (2.0×) | 23.4 (2.8×) |
| add_uint | P-256 base | 62.2 | 181 (2.9×) | 78.1 (1.3×) | 328 (5.3×) | 149 (2.4×) |
| add_uint | P-256 scalar | 89.8 | 258 (2.9×) | 106 (1.2×) | 342 (3.8×) | 201 (2.2×) |
| add_uint | Tom-256 base | 83.3 | 245 (2.9×) | 108 (1.3×) | 332 (4.0×) | 196 (2.4×) |
| add_uint | BLS12-381 base | 166 | 763 (4.6×) | 196 (1.2×) | 955 (5.8×) | 375 (2.3×) |
| add_uint | P-521 base | 23.6 | 62.6 (2.7×) | 42.1 (1.8×) | 85.0 (3.6×) | 67.2 (2.8×) |
| sub_u64 | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.3 (1.2×) | 6.5 (1.4×) | 6.7 (1.5×) |
| sub_u64 | BabyBear | 6.4 | 11.9 (1.9×) | 8.2 (1.3×) | 12.7 (2.0×) | 11.1 (1.7×) |
| sub_u64 | Goldilocks | 4.2 | 8.9 (2.1×) | 6.6 (1.6×) | 10.7 (2.5×) | 9.4 (2.2×) |
| sub_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 45.8 (1.8×) | 137 (5.5×) | 67.1 (2.7×) |
| sub_u64 | secp256k1 base | 6.9 | 7.0 (1.0×) | 6.1 (0.9×) | 7.0 (1.0×) | 9.0 (1.3×) |
| sub_u64 | P-256 base | 21.5 | 71.3 (3.3×) | 34.7 (1.6×) | 79.8 (3.7×) | 55.6 (2.6×) |
| sub_u64 | P-256 scalar | 23.6 | 68.7 (2.9×) | 37.9 (1.6×) | 73.5 (3.1×) | 64.8 (2.7×) |
| sub_u64 | Tom-256 base | 25.0 | 91.6 (3.7×) | 39.3 (1.6×) | 105 (4.2×) | 66.6 (2.7×) |
| sub_u64 | BLS12-381 base | 38.7 | 155 (4.0×) | 62.2 (1.6×) | 266 (6.9×) | 92.1 (2.4×) |
| sub_u64 | P-521 base | 11.1 | 21.1 (1.9×) | 15.3 (1.4×) | 21.7 (2.0×) | 21.9 (2.0×) |
| sub_uint | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.4 (1.2×) | 6.5 (1.4×) | 6.6 (1.5×) |
| sub_uint | BabyBear | 7.0 | 12.0 (1.7×) | 8.2 (1.2×) | 13.1 (1.9×) | 11.2 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.9 (2.1×) | 6.6 (1.6×) | 10.7 (2.5×) | 9.4 (2.2×) |
| sub_uint | BN254 scalar | 83.9 | 228 (2.7×) | 103 (1.2×) | 367 (4.4×) | 177 (2.1×) |
| sub_uint | secp256k1 base | 6.8 | 18.9 (2.8×) | 12.4 (1.8×) | 16.5 (2.4×) | 21.9 (3.2×) |
| sub_uint | P-256 base | 61.4 | 185 (3.0×) | 78.9 (1.3×) | 372 (6.1×) | 145 (2.4×) |
| sub_uint | P-256 scalar | 86.1 | 257 (3.0×) | 109 (1.3×) | 340 (3.9×) | 202 (2.3×) |
| sub_uint | Tom-256 base | 81.5 | 246 (3.0×) | 105 (1.3×) | 332 (4.1×) | 196 (2.4×) |
| sub_uint | BLS12-381 base | 169 | 755 (4.5×) | 194 (1.1×) | 934 (5.5×) | 379 (2.2×) |
| sub_uint | P-521 base | 15.6 | 67.1 (4.3×) | 38.7 (2.5×) | 76.9 (4.9×) | 67.7 (4.3×) |
| div_u64 | BN254 scalar | 605 | 1,213 (2.0×) | 819 (1.4×) | 1,421 (2.3×) | 1,046 (1.7×) |
| div_u64 | secp256k1 base | 567 | 1,019 (1.8×) | 777 (1.4×) | 1,216 (2.1×) | 871 (1.5×) |
| div_u64 | P-256 base | 603 | 1,094 (1.8×) | 798 (1.3×) | 1,342 (2.2×) | 982 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,179 (1.9×) | 861 (1.4×) | 1,400 (2.3×) | 1,132 (1.8×) |
| div_u64 | Tom-256 base | 612 | 1,183 (1.9×) | 859 (1.4×) | 1,486 (2.4×) | 1,082 (1.8×) |
| div_u64 | BLS12-381 base | 754 | 1,802 (2.4×) | 1,031 (1.4×) | 2,484 (3.3×) | 1,753 (2.3×) |
| div_u64 | P-521 base | 922 | 2,454 (2.7×) | 1,373 (1.5×) | 3,629 (3.9×) | 1,867 (2.0×) |
| div_uint | Mersenne-31 | 219 | 276 (1.3×) | 277 (1.3×) | 287 (1.3×) | 281 (1.3×) |
| div_uint | BabyBear | 218 | 278 (1.3×) | 278 (1.3×) | 286 (1.3×) | 283 (1.3×) |
| div_uint | Goldilocks | 438 | 553 (1.3×) | 543 (1.2×) | 593 (1.4×) | 579 (1.3×) |
| div_uint | BN254 scalar | 4,997 | 9,042 (1.8×) | 6,541 (1.3×) | 12,999 (2.6×) | 8,439 (1.7×) |
| div_uint | secp256k1 base | 4,867 | 8,481 (1.7×) | 6,033 (1.2×) | 12,094 (2.5×) | 8,085 (1.7×) |
| div_uint | P-256 base | 5,079 | 8,763 (1.7×) | 6,170 (1.2×) | 12,245 (2.4×) | 7,914 (1.6×) |
| div_uint | P-256 scalar | 4,934 | 9,221 (1.9×) | 6,605 (1.3×) | 12,426 (2.5×) | 8,667 (1.8×) |
| div_uint | Tom-256 base | 5,043 | 8,984 (1.8×) | 6,489 (1.3×) | 12,329 (2.4×) | 8,467 (1.7×) |
| div_uint | BLS12-381 base | 9,060 | 20,824 (2.3×) | 12,045 (1.3×) | 26,517 (2.9×) | 16,955 (1.9×) |
| div_uint | P-521 base | 14,914 | 34,490 (2.3×) | 18,121 (1.2×) | 50,364 (3.4×) | 30,006 (2.0×) |
| add, constant time | Mersenne-31 | 3.0 | 7.5 (2.5×) | 7.5 (2.5×) | 7.7 (2.6×) | 7.7 (2.6×) |
| add, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.9 | 24.8 (3.6×) | 19.4 (2.8×) | 23.1 (3.3×) | 25.7 (3.7×) |
| add, constant time | secp256k1 base | 6.9 | 28.1 (4.1×) | 22.6 (3.3×) | 27.0 (3.9×) | 29.8 (4.3×) |
| add, constant time | P-256 base | 7.0 | 26.8 (3.8×) | 22.7 (3.2×) | 24.9 (3.6×) | 28.1 (4.0×) |
| add, constant time | P-256 scalar | 6.7 | 26.6 (4.0×) | 23.7 (3.5×) | 26.9 (4.0×) | 30.1 (4.5×) |
| add, constant time | Tom-256 base | 7.0 | 27.9 (4.0×) | 23.8 (3.4×) | 27.1 (3.9×) | 29.1 (4.2×) |
| add, constant time | BLS12-381 base | 8.9 | 41.4 (4.7×) | 28.3 (3.2×) | 35.5 (4.0×) | 41.1 (4.6×) |
| add, constant time | P-521 base | 14.9 | 76.7 (5.1×) | 47.2 (3.2×) | 80.0 (5.4×) | 74.1 (5.0×) |
| sub, constant time | Mersenne-31 | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 7.0 | 26.4 (3.8×) | 16.9 (2.4×) | 27.0 (3.9×) | 25.0 (3.6×) |
| sub, constant time | secp256k1 base | 8.0 | 21.4 (2.7×) | 15.5 (1.9×) | 20.5 (2.6×) | 21.5 (2.7×) |
| sub, constant time | P-256 base | 7.0 | 19.1 (2.7×) | 15.6 (2.2×) | 18.6 (2.7×) | 20.0 (2.9×) |
| sub, constant time | P-256 scalar | 6.8 | 21.6 (3.2×) | 15.7 (2.3×) | 18.7 (2.8×) | 22.0 (3.2×) |
| sub, constant time | Tom-256 base | 8.2 | 21.2 (2.6×) | 17.1 (2.1×) | 19.8 (2.4×) | 23.0 (2.8×) |
| sub, constant time | BLS12-381 base | 9.9 | 37.4 (3.8×) | 24.2 (2.4×) | 36.9 (3.7×) | 36.3 (3.7×) |
| sub, constant time | P-521 base | 12.6 | 57.1 (4.5×) | 37.2 (3.0×) | 60.3 (4.8×) | 53.1 (4.2×) |
| mul, constant time | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| mul, constant time | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.5 (1.3×) | 11.0 (1.0×) |
| mul, constant time | Goldilocks | 8.9 | 18.1 (2.0×) | 14.4 (1.6×) | 21.0 (2.4×) | 14.0 (1.6×) |
| mul, constant time | BN254 scalar | 79.0 | 242 (3.1×) | 94.8 (1.2×) | 415 (5.3×) | 154 (1.9×) |
| mul, constant time | secp256k1 base | 55.0 | 173 (3.1×) | 77.6 (1.4×) | 276 (5.0×) | 106 (1.9×) |
| mul, constant time | P-256 base | 60.0 | 189 (3.1×) | 72.2 (1.2×) | 316 (5.3×) | 120 (2.0×) |
| mul, constant time | P-256 scalar | 84.0 | 233 (2.8×) | 94.4 (1.1×) | 335 (4.0×) | 158 (1.9×) |
| mul, constant time | Tom-256 base | 76.0 | 237 (3.1×) | 88.2 (1.2×) | 336 (4.4×) | 151 (2.0×) |
| mul, constant time | BLS12-381 base | 154 | 720 (4.7×) | 189 (1.2×) | 930 (6.0×) | 337 (2.2×) |
| mul, constant time | P-521 base | 188 | 687 (3.7×) | 247 (1.3×) | 1,300 (6.9×) | 445 (2.4×) |
| square, constant time | Mersenne-31 | 7.0 | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) | 12.0 (1.7×) |
| square, constant time | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 11.0 (1.0×) |
| square, constant time | Goldilocks | 9.0 | 17.3 (1.9×) | 12.0 (1.3×) | 21.1 (2.3×) | 14.4 (1.6×) |
| square, constant time | BN254 scalar | 66.9 | 219 (3.3×) | 83.7 (1.3×) | 329 (4.9×) | 138 (2.1×) |
| square, constant time | secp256k1 base | 43.0 | 153 (3.6×) | 69.9 (1.6×) | 168 (3.9×) | 97.4 (2.3×) |
| square, constant time | P-256 base | 44.0 | 155 (3.5×) | 62.6 (1.4×) | 227 (5.2×) | 113 (2.6×) |
| square, constant time | P-256 scalar | 73.0 | 206 (2.8×) | 89.8 (1.2×) | 242 (3.3×) | 149 (2.0×) |
| square, constant time | Tom-256 base | 64.5 | 200 (3.1×) | 83.1 (1.3×) | 290 (4.5×) | 142 (2.2×) |
| square, constant time | BLS12-381 base | 145 | 608 (4.2×) | 169 (1.2×) | 867 (6.0×) | 297 (2.0×) |
| square, constant time | P-521 base | 133 | 571 (4.3×) | 208 (1.6×) | 907 (6.8×) | 392 (2.9×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_u64, constant time | BabyBear | 10.9 | 17.0 (1.6×) | 11.0 (1.0×) | 19.3 (1.8×) | 13.0 (1.2×) |
| mul_u64, constant time | Goldilocks | 8.9 | 20.4 (2.3×) | 12.1 (1.4×) | 25.0 (2.8×) | 17.2 (1.9×) |
| mul_u64, constant time | BN254 scalar | 30.3 | 108 (3.6×) | 50.9 (1.7×) | 132 (4.4×) | 58.0 (1.9×) |
| mul_u64, constant time | secp256k1 base | 19.9 | 68.9 (3.5×) | 39.0 (2.0×) | 85.1 (4.3×) | 44.3 (2.2×) |
| mul_u64, constant time | P-256 base | 31.9 | 87.6 (2.7×) | 53.2 (1.7×) | 105 (3.3×) | 59.5 (1.9×) |
| mul_u64, constant time | P-256 scalar | 33.5 | 105 (3.1×) | 55.0 (1.6×) | 133 (4.0×) | 65.5 (2.0×) |
| mul_u64, constant time | Tom-256 base | 32.9 | 99.7 (3.0×) | 52.0 (1.6×) | 122 (3.7×) | 61.4 (1.9×) |
| mul_u64, constant time | BLS12-381 base | 42.1 | 154 (3.7×) | 70.8 (1.7×) | 195 (4.6×) | 88.5 (2.1×) |
| mul_u64, constant time | P-521 base | 45.1 | 139 (3.1×) | 78.0 (1.7×) | 170 (3.8×) | 94.9 (2.1×) |
| mul_uint, constant time | Mersenne-31 | 9.0 | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) | 14.0 (1.6×) |
| mul_uint, constant time | BabyBear | 11.0 | 17.0 (1.5×) | 11.0 (1.0×) | 19.4 (1.8×) | 13.1 (1.2×) |
| mul_uint, constant time | Goldilocks | 9.1 | 20.4 (2.2×) | 12.1 (1.3×) | 25.0 (2.7×) | 17.2 (1.9×) |
| mul_uint, constant time | BN254 scalar | 141 | 481 (3.4×) | 167 (1.2×) | 593 (4.2×) | 312 (2.2×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 317 (3.8×) | 193 (2.3×) | 380 (4.5×) | 253 (3.0×) |
| mul_uint, constant time | P-256 base | 134 | 406 (3.0×) | 137 (1.0×) | 489 (3.6×) | 235 (1.8×) |
| mul_uint, constant time | P-256 scalar | 146 | 478 (3.3×) | 178 (1.2×) | 540 (3.7×) | 314 (2.2×) |
| mul_uint, constant time | Tom-256 base | 135 | 467 (3.5×) | 172 (1.3×) | 554 (4.1×) | 300 (2.2×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,082 (4.1×) | 370 (1.4×) | 1,699 (6.4×) | 912 (3.4×) |
| mul_uint, constant time | P-521 base | 472 | 2,017 (4.3×) | 997 (2.1×) | 2,901 (6.1×) | 1,465 (3.1×) |
| add_u64, constant time | Mersenne-31 | 4.6 | 8.6 (1.9×) | 8.7 (1.9×) | 9.3 (2.0×) | 9.3 (2.0×) |
| add_u64, constant time | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_u64, constant time | Goldilocks | 4.4 | 9.1 (2.1×) | 7.3 (1.7×) | 10.7 (2.4×) | 10.0 (2.3×) |
| add_u64, constant time | BN254 scalar | 28.8 | 132 (4.6×) | 55.9 (1.9×) | 151 (5.2×) | 81.8 (2.8×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.4 (3.4×) | 23.0 (3.1×) | 25.1 (3.4×) | 29.0 (3.9×) |
| add_u64, constant time | P-256 base | 24.0 | 89.1 (3.7×) | 55.6 (2.3×) | 94.9 (4.0×) | 74.6 (3.1×) |
| add_u64, constant time | P-256 scalar | 26.0 | 85.7 (3.3×) | 58.6 (2.3×) | 98.4 (3.8×) | 89.6 (3.4×) |
| add_u64, constant time | Tom-256 base | 27.1 | 119 (4.4×) | 59.6 (2.2×) | 133 (4.9×) | 85.1 (3.1×) |
| add_u64, constant time | BLS12-381 base | 44.9 | 182 (4.1×) | 96.1 (2.1×) | 233 (5.2×) | 124 (2.8×) |
| add_u64, constant time | P-521 base | 12.7 | 65.6 (5.2×) | 47.0 (3.7×) | 69.0 (5.4×) | 64.9 (5.1×) |
| add_uint, constant time | Mersenne-31 | 4.6 | 8.7 (1.9×) | 8.6 (1.9×) | 9.3 (2.0×) | 9.3 (2.0×) |
| add_uint, constant time | BabyBear | 7.1 | 12.4 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.5×) |
| add_uint, constant time | Goldilocks | 4.4 | 9.0 (2.0×) | 7.3 (1.7×) | 10.7 (2.4×) | 10.0 (2.3×) |
| add_uint, constant time | BN254 scalar | 91.8 | 230 (2.5×) | 98.1 (1.1×) | 297 (3.2×) | 181 (2.0×) |
| add_uint, constant time | secp256k1 base | 8.7 | 38.3 (4.4×) | 29.4 (3.4×) | 36.3 (4.2×) | 44.2 (5.1×) |
| add_uint, constant time | P-256 base | 68.9 | 178 (2.6×) | 84.7 (1.2×) | 255 (3.7×) | 144 (2.1×) |
| add_uint, constant time | P-256 scalar | 89.8 | 260 (2.9×) | 107 (1.2×) | 311 (3.5×) | 197 (2.2×) |
| add_uint, constant time | Tom-256 base | 86.6 | 255 (2.9×) | 104 (1.2×) | 332 (3.8×) | 182 (2.1×) |
| add_uint, constant time | BLS12-381 base | 179 | 764 (4.3×) | 206 (1.2×) | 941 (5.3×) | 369 (2.1×) |
| add_uint, constant time | P-521 base | 22.7 | 138 (6.1×) | 93.5 (4.1×) | 147 (6.5×) | 134 (5.9×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 5.3 (1.2×) | 5.4 (1.2×) | 6.6 (1.5×) | 6.6 (1.5×) |
| sub_u64, constant time | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 13.1 (1.9×) | 11.2 (1.6×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.8 (2.1×) | 6.6 (1.6×) | 10.7 (2.5×) | 9.4 (2.2×) |
| sub_u64, constant time | BN254 scalar | 28.3 | 125 (4.4×) | 56.9 (2.0×) | 153 (5.4×) | 80.0 (2.8×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.0 (2.6×) | 15.1 (2.2×) | 17.0 (2.5×) | 17.3 (2.5×) |
| sub_u64, constant time | P-256 base | 22.9 | 85.7 (3.7×) | 50.6 (2.2×) | 88.2 (3.9×) | 67.2 (2.9×) |
| sub_u64, constant time | P-256 scalar | 22.2 | 81.3 (3.7×) | 50.8 (2.3×) | 93.2 (4.2×) | 79.8 (3.6×) |
| sub_u64, constant time | Tom-256 base | 28.1 | 113 (4.0×) | 56.2 (2.0×) | 125 (4.4×) | 80.5 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 44.5 | 181 (4.1×) | 90.0 (2.0×) | 223 (5.0×) | 120 (2.7×) |
| sub_u64, constant time | P-521 base | 11.1 | 45.5 (4.1×) | 36.1 (3.3×) | 49.8 (4.5×) | 46.3 (4.2×) |
| sub_uint, constant time | Mersenne-31 | 4.6 | 5.3 (1.2×) | 5.3 (1.2×) | 6.6 (1.4×) | 6.6 (1.4×) |
| sub_uint, constant time | BabyBear | 7.1 | 11.9 (1.7×) | 8.2 (1.2×) | 13.0 (1.8×) | 11.2 (1.6×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.9 (2.1×) | 6.5 (1.5×) | 10.7 (2.5×) | 9.4 (2.2×) |
| sub_uint, constant time | BN254 scalar | 93.1 | 242 (2.6×) | 99.0 (1.1×) | 283 (3.0×) | 180 (1.9×) |
| sub_uint, constant time | secp256k1 base | 7.7 | 32.2 (4.2×) | 25.0 (3.2×) | 31.8 (4.1×) | 38.3 (5.0×) |
| sub_uint, constant time | P-256 base | 68.0 | 176 (2.6×) | 80.3 (1.2×) | 251 (3.7×) | 138 (2.0×) |
| sub_uint, constant time | P-256 scalar | 96.0 | 253 (2.6×) | 102 (1.1×) | 331 (3.4×) | 189 (2.0×) |
| sub_uint, constant time | Tom-256 base | 86.3 | 248 (2.9×) | 98.4 (1.1×) | 333 (3.9×) | 177 (2.1×) |
| sub_uint, constant time | BLS12-381 base | 172 | 754 (4.4×) | 204 (1.2×) | 946 (5.5×) | 376 (2.2×) |
| sub_uint, constant time | P-521 base | 19.6 | 124 (6.3×) | 85.4 (4.4×) | 121 (6.2×) | 113 (5.8×) |
| div_u64, constant time | BN254 scalar | 6,950 | 12,371 (1.8×) | 9,487 (1.4×) | 17,513 (2.5×) | 12,967 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,790 | 11,948 (1.8×) | 8,999 (1.3×) | 17,024 (2.5×) | 12,708 (1.9×) |
| div_u64, constant time | P-256 base | 6,707 | 11,856 (1.8×) | 9,177 (1.4×) | 17,369 (2.6×) | 12,549 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,903 | 12,313 (1.8×) | 9,442 (1.4×) | 17,333 (2.5×) | 13,173 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,295 | 12,164 (1.7×) | 9,294 (1.3×) | 16,956 (2.3×) | 12,902 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,417 | 24,774 (2.0×) | 16,947 (1.4×) | 35,027 (2.8×) | 24,420 (2.0×) |
| div_u64, constant time | P-521 base | 20,361 | 43,048 (2.1×) | 26,016 (1.3×) | 64,693 (3.2×) | 43,074 (2.1×) |
| div_uint, constant time | Mersenne-31 | 269 | 451 (1.7×) | 451 (1.7×) | 459 (1.7×) | 462 (1.7×) |
| div_uint, constant time | BabyBear | 405 | 545 (1.3×) | 436 (1.1×) | 607 (1.5×) | 450 (1.1×) |
| div_uint, constant time | Goldilocks | 664 | 1,308 (2.0×) | 947 (1.4×) | 1,577 (2.4×) | 1,207 (1.8×) |
| div_uint, constant time | BN254 scalar | 6,863 | 12,397 (1.8×) | 9,514 (1.4×) | 17,589 (2.6×) | 13,298 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,807 | 11,902 (1.7×) | 9,074 (1.3×) | 16,852 (2.5×) | 12,843 (1.9×) |
| div_uint, constant time | P-256 base | 6,719 | 11,985 (1.8×) | 9,149 (1.4×) | 17,590 (2.6×) | 12,643 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,965 | 12,486 (1.8×) | 9,436 (1.4×) | 17,771 (2.6×) | 13,263 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,280 | 12,377 (1.7×) | 9,442 (1.3×) | 17,138 (2.4×) | 13,092 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,518 | 25,361 (2.0×) | 16,780 (1.3×) | 35,352 (2.8×) | 24,779 (2.0×) |
| div_uint, constant time | P-521 base | 20,372 | 42,851 (2.1×) | 25,984 (1.3×) | 65,678 (3.2×) | 43,052 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
