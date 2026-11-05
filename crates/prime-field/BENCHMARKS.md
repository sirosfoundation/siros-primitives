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

## Results in WebAssembly, Chrome

### add: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | – | 11.0 / 19.1 | – | – | 4.4 / 7.5\* | – | 10.8 / 18.0 | – |
| BabyBear | **2.4** / **3.0** | – | 11.0 / 21.1 | – | – | **2.4** / **3.0** | – | 10.8 / 18.0 | – |
| Goldilocks | **3.3** / **3.0** | – | 10.9 / 20.2 | – | – | 3.5 / 6.0 | **3.3** / 4.0 | 10.8 / 18.0 | – |
| BN254 scalar | 16.7 / 14.7 | **13.6** / **13.3** | 53.1 / 83.0 | – | 36.2 / 41.6 | – | – | 62.2 / 104 | – |
| secp256k1 base | **14.1** / **13.7** | 15.2 / 15.5 | 53.0 / 83.0 | 22.4 / 14.5 | 35.1 / 39.8 | – | – | 62.1 / 104 | – |
| P-256 base | **15.1** / **12.9** | 15.6 / 15.2 | 53.2 / 82.9 | 30.5 / 39.6 | 30.5 / 38.9 | – | – | 62.2 / 103 | 440 / – |
| P-256 scalar | **15.3** / **14.3** | 15.4 / 15.2 | 53.1 / 82.9 | 41.4 / 57.8 | 35.5 / 42.2 | – | – | 62.1 / 103 | 443 / – |
| Tom-256 base | **15.0** / **14.3** | – | 53.1 / 83.0 | – | 36.6 / 42.9 | – | – | 62.2 / 104 | – |
| BLS12-381 base | **25.8** / **25.0** | 35.4 / 43.9 | 107 / 113 | – | 63.5 / 63.3 | – | – | 115 / 158 | – |
| P-521 base | **42.3** / **46.0** | – | 142 / 183 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### sub: a − b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.6 / 8.1 | – |
| BabyBear | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 2.7 / **3.0** | – | 8.5 / 7.9 | – |
| Goldilocks | **2.6** / **3.0** | – | 4.5 / 8.0 | – | – | 3.5 / 5.0 | 2.8 / **3.0** | 10.4 / 8.1 | – |
| BN254 scalar | **16.6** / **14.6** | 18.4 / 16.5 | 31.0 / 32.0 | – | 24.0 / 26.0 | – | – | 37.3 / 77.1 | – |
| secp256k1 base | **16.7** / **16.5** | 18.5 / 16.6 | 31.0 / 29.0 | 25.5 / 19.7 | 22.3 / 25.2 | – | – | 37.6 / 77.1 | – |
| P-256 base | **15.2** / **14.6** | 16.9 / 15.5 | 30.9 / 27.5 | 20.8 / 26.6 | 20.5 / 25.2 | – | – | 37.5 / 77.4 | 230 / – |
| P-256 scalar | **16.8** / **15.2** | 17.9 / 16.4 | 31.0 / 31.0 | 23.2 / 24.7 | 23.6 / 26.8 | – | – | 36.9 / 77.1 | 233 / – |
| Tom-256 base | **16.5** / **14.9** | – | 31.0 / 32.0 | – | 23.8 / 26.0 | – | – | 37.7 / 77.0 | – |
| BLS12-381 base | **25.4** / **25.3** | 33.5 / 30.6 | 54.0 / 101 | – | 43.3 / 45.3 | – | – | 73.0 / 103 | – |
| P-521 base | **48.4** / **43.5** | – | 88.7 / 159 | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | – | 29.3 / 46.5 | – | – | 5.2 / 12.0 | – | 51.4 / 59.0 | – |
| BabyBear | 7.8 / 13.0 | – | 29.3 / 46.1 | – | – | **4.1** / **12.2** | – | 51.3 / 59.0 | – |
| Goldilocks | **12.2** / **18.0** | – | 32.5 / 47.1 | – | – | 10.0\* / 16.6\* | 12.3 / **18.0** | 51.4 / 59.0 | – |
| BN254 scalar | **235** / **228** | 318 / 316 | 431 / 441 | – | 375 / 378 | – | – | 897 / 938 | – |
| secp256k1 base | **162** / **172** | 279 / 276 | 436 / 436 | 158\* / 162\* | 303 / 306 | – | – | 910 / 944 | – |
| P-256 base | **177** / **190** | 262 / 258 | 429 / 437 | 241 / 241 | 243 / 243 | – | – | 558 / 589 | 1,081 / – |
| P-256 scalar | **241** / **235** | 314 / 309 | 430 / 429 | 517 / 512 | 344 / 351 | – | – | 911 / 938 | 1,075 / – |
| Tom-256 base | **237** / **236** | – | 443 / 443 | – | 344 / 342 | – | – | 909 / 943 | – |
| BLS12-381 base | **726** / 722 | 805 / **683** | 912 / 986 | – | 884 / 892 | – | – | 2,009 / 2,023 | – |
| P-521 base | **1,020** / **1,062** | – | 2,114 / 2,118 | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | – | 28.5 / 45.0 | – | – | 5.1\* / 12.0\* | – | 46.5 / 55.3 | – |
| BabyBear | 8.3 / 13.0 | – | 28.5 / 45.5 | – | – | **5.1** / **12.1** | – | 46.6 / 58.7 | – |
| Goldilocks | **10.1** / **17.3** | – | 32.7 / 46.5 | – | – | 8.7\* / 16.3\* | 12.4 / 20.0 | 46.5 / 58.7 | – |
| BN254 scalar | **215** / **206** | 271 / 263 | 318 / 351 | – | 310 / 309 | – | – | 899 / 925 | – |
| secp256k1 base | **143** / **154** | 232 / 225 | 325 / 348 | 98.6\* / 116\* | 215 / 222 | – | – | 914 / 937 | – |
| P-256 base | **159** / **155** | 214 / 210 | 325 / 350 | 240 / 244 | 200 / 178 | – | – | 560 / 590 | 1,086 / – |
| P-256 scalar | **219** / **217** | 266 / 259 | 323 / 349 | 504 / 515 | 269 / 266 | – | – | 915 / 941 | 1,090 / – |
| Tom-256 base | **205** / **201** | – | 323 / 350 | – | 317 / 267 | – | – | 914 / 943 | – |
| BLS12-381 base | **578** / **609** | 633 / 620 | 903 / 974 | – | 670 / 928 | – | – | 2,010 / 2,024 | – |
| P-521 base | **563** / **571** | – | 2,154 / 2,164 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **266** / – | – | 1,731 / – | – | – | 676 / – | – | 2,214 / – | – |
| BabyBear | **269** / – | – | 1,713 / – | – | – | 701 / – | – | 2,358 / – | – |
| Goldilocks | **537** / – | – | 3,455 / – | – | – | 1,369 / – | 1,772 / – | 4,423 / – | – |
| BN254 scalar | **8,440** / – | 24,008 / – | 18,492 / – | – | 14,085 / – | – | – | 42,132 / – | – |
| secp256k1 base | **8,502** / – | 22,273 / – | 18,512 / – | 33,509 / – | 14,123 / – | – | – | 42,531 / – | – |
| P-256 base | **8,282** / – | 21,861 / – | 18,511 / – | 66,059 / – | 14,147 / – | – | – | 41,441 / – | 248,166 / – |
| P-256 scalar | **8,482** / – | 24,160 / – | 18,514 / – | 46,542 / – | 14,142 / – | – | – | 42,313 / – | 253,059 / – |
| Tom-256 base | **8,554** / – | – | 18,478 / – | – | 14,150 / – | – | – | 42,309 / – | – |
| BLS12-381 base | **19,335** / – | 46,919 / – | 32,746 / – | – | 25,564 / – | – | – | 98,059 / – | – |
| P-521 base | **33,221** / – | – | 65,268 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 329 / – | – | 3,422 / – | – | – | **324** / – | 1,741 / – | – |
| BabyBear | **2,670** / – | – | 34,149 / – | – | – | 3,090 / – | – | – |
| Goldilocks | **5,768** / – | – | 48,789 / – | – | – | 5,571\* / – | – | – |
| BN254 scalar | **108,085** / – | 147,910 / – | 330,187 / – | – | 380,651 / – | – | – | – |
| secp256k1 base | **42,090** / – | 130,955 / – | 160,451 / – | 33,149\* / – | 158,742 / – | – | 474,951 / – | – |
| P-256 base | **44,757** / – | 59,370 / – | 135,828 / – | 64,488 / – | 129,911 / – | – | 171,337 / – | 309,100 / – |
| P-256 scalar | **74,255** / – | 115,519 / – | 158,074 / – | 240,007 / – | 153,029 / – | – | – | – |
| Tom-256 base | **65,526** / – | – | 149,734 / – | – | 183,196 / – | – | 338,342 / – | – |
| BLS12-381 base | **311,855** / – | 419,409 / – | 545,084 / – | – | 797,158 / – | – | 1,236,367 / – | – |
| P-521 base | **320,120** / – | – | 1,528,245 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | – | 61.3 / 59.5 | – | – | 9.1\* / 12.5\* | – | – |
| BabyBear | **17.0** / **17.0** | – | 63.9 / 65.1 | – | – | 34.9 / 34.2 | – | – |
| Goldilocks | **19.1** / **20.0** | – | 72.4 / 74.2 | – | – | 10.0\* / 16.6\* | 25.0 / 23.6 | – |
| BN254 scalar | **99.6** / **91.7** | 694 / 659 | 853 / 853 | – | 1,035 / 1,050 | – | – | 1,842 / 1,840 |
| secp256k1 base | **42.2** / **49.0** | 451 / 435 | 858 / 861 | 157 / 165 | 707 / 690 | – | – | 1,835 / 1,846 |
| P-256 base | **72.2** / **75.0** | 609 / 530 | 868 / 855 | 484 / 487 | 719 / 735 | – | – | 1,209 / 1,215 |
| P-256 scalar | **84.1** / **82.1** | 634 / 620 | 858 / 862 | 517 / 513 | 974 / 991 | – | – | 1,858 / 1,855 |
| Tom-256 base | **82.2** / **84.9** | – | 867 / 843 | – | 937 / 947 | – | – | 1,855 / 1,864 |
| BLS12-381 base | **131** / **123** | 1,636 / 1,627 | 1,920 / 1,941 | – | 2,380 / 2,378 | – | – | 4,106 / 4,101 |
| P-521 base | **92.1** / **93.3** | – | 3,011 / 3,025 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 60.5 / 59.6 | – | – | 8.9\* / 12.6\* | – |
| BabyBear | **17.0** / **17.0** | 63.9 / 64.8 | – | – | 23.0 / 23.6 | – |
| Goldilocks | **19.1** / **20.2** | 72.4 / 74.3 | – | – | 10.0\* / 16.6\* | 24.9 / 23.5 |
| BN254 scalar | **433** / **434** | 858 / 870 | – | 756 / 754 | – | – |
| secp256k1 base | **266** / **265** | 866 / 873 | – | 519 / 515 | – | – |
| P-256 base | **353** / **353** | 866 / 866 | – | 456 / 452 | – | – |
| P-256 scalar | **426** / **425** | 862 / 869 | 549 / 548 | 728 / 725 | – | – |
| Tom-256 base | **410** / **409** | 858 / 861 | – | 692 / 693 | – | – |
| BLS12-381 base | **990** / **982** | 1,903 / 1,940 | – | 1,957 / 1,939 | – | – |
| P-521 base | **1,472** / **1,544** | 3,010 / 3,062 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.6** | – | 36.1 / 35.4 | – | – | 7.6\* / 8.1\* | – | – |
| BabyBear | **10.9** / **12.1** | – | 39.8 / 47.5 | – | – | 32.1 / 30.7 | – | – |
| Goldilocks | **9.9** / **9.1** | – | 50.8 / 58.3 | – | – | 3.5\* / 6.0\* | 13.3 / 13.0 | – |
| BN254 scalar | **122** / **117** | 349 / 339 | 506 / 527 | – | 691 / 663 | – | – | 1,057 / 1,056 |
| secp256k1 base | **8.0** / **7.0** | 162 / 158 | 500 / 502 | 16.8 / 5.3\* | 424 / 402 | – | – | 1,022 / 1,027 |
| P-256 base | **72.3** / **68.9** | 268 / 261 | 496 / 512 | 275 / 278 | 488 / 487 | – | – | 702 / 692 |
| P-256 scalar | 70.2 / 69.2 | 339 / 335 | 499 / 507 | **41.7** / **57.4** | 678 / 655 | – | – | 1,053 / 1,052 |
| Tom-256 base | **89.6** / **88.4** | – | 505 / 510 | – | 626 / 609 | – | – | 1,055 / 1,048 |
| BLS12-381 base | **179** / **160** | 836 / 830 | 1,081 / 1,088 | – | 1,395 / 1,379 | – | – | 2,216 / 2,220 |
| P-521 base | **23.1** / **21.1** | – | 1,668 / 1,674 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **8.7** | 36.1 / 35.4 | – | – | 7.6\* / 8.0\* | – |
| BabyBear | **10.9** / **12.2** | 39.8 / 47.5 | – | – | 22.0 / 21.5 | – |
| Goldilocks | **9.9** / **9.1** | 50.8 / 58.2 | – | – | 3.5\* / 6.0\* | 13.3 / 13.0 |
| BN254 scalar | **290** / **284** | 509 / 514 | – | 412 / 392 | – | – |
| secp256k1 base | **20.9** / **17.0** | 500 / 507 | – | 223 / 226 | – | – |
| P-256 base | **204** / **197** | 502 / 513 | – | 244 / 246 | – | – |
| P-256 scalar | 295 / 288 | 499 / 507 | **94.6** / **81.8** | 389 / 387 | – | – |
| Tom-256 base | **294** / **289** | 503 / 511 | – | 378 / 380 | – | – |
| BLS12-381 base | **836** / **807** | 1,072 / 1,087 | – | 979 / 994 | – | – |
| P-521 base | **64.8** / **62.2** | 1,693 / 1,691 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | – | 23.4 / 22.1 | – | – | 6.1\* / 6.1 | – | – |
| BabyBear | **11.0** / **11.9** | – | 27.9 / 28.7 | – | – | 32.2 / 31.8 | – | – |
| Goldilocks | **9.9** / **8.9** | – | 36.7 / 36.3 | – | – | 3.5\* / 5.0\* | 11.5 / 11.1 | – |
| BN254 scalar | **113** / **113** | 352 / 347 | 499 / 490 | – | 666 / 652 | – | – | 1,003 / 991 |
| secp256k1 base | **8.7** / **7.0** | 161 / 156 | 498 / 492 | 14.0 / 11.0 | 413 / 392 | – | – | 979 / 980 |
| P-256 base | **71.4** / **71.3** | 272 / 263 | 485 / 486 | 264 / 265 | 493 / 470 | – | – | 645 / 643 |
| P-256 scalar | 69.3 / 68.8 | 339 / 336 | 499 / 487 | **21.2** / **21.2** | 660 / 641 | – | – | 1,003 / 998 |
| Tom-256 base | **88.1** / **91.7** | – | 493 / 481 | – | 631 / 599 | – | – | 1,002 / 992 |
| BLS12-381 base | **162** / **155** | 837 / 824 | 1,083 / 1,093 | – | 1,381 / 1,332 | – | – | 2,160 / 2,163 |
| P-521 base | **22.5** / **21.1** | – | 1,673 / 1,680 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 23.4 / 22.1 | – | – | 6.1\* / 6.1 | – |
| BabyBear | **11.0** / **11.9** | 27.9 / 28.7 | – | – | 23.0 / 22.0 | – |
| Goldilocks | **9.9** / **8.9** | 36.7 / 36.3 | – | – | 3.5\* / 5.0\* | 11.5 / 11.1 |
| BN254 scalar | **287** / **282** | 497 / 501 | – | 401 / 396 | – | – |
| secp256k1 base | **21.2** / **18.9** | 501 / 488 | – | 215 / 214 | – | – |
| P-256 base | **206** / **202** | 493 / 488 | – | 233 / 234 | – | – |
| P-256 scalar | 290 / 286 | 500 / 490 | **67.8** / **71.9** | 373 / 370 | – | – |
| Tom-256 base | **294** / **287** | 499 / 486 | – | 364 / 357 | – | – |
| BLS12-381 base | **825** / **809** | 1,074 / 1,077 | – | 972 / 972 | – | – |
| P-521 base | **75.5** / **67.4** | 1,677 / 1,680 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 18.0 / – | – | – | **4.1** / – | – | – |
| BabyBear | **11.1** / – | – | 21.6 / – | – | – | 31.0 / – | – | – |
| Goldilocks | **7.2** / – | – | 29.3 / – | – | – | 2.1\* / – | 10.6 / – | – |
| BN254 scalar | **87.9** / – | 317 / – | 433 / – | – | 612 / – | – | – | 935 / – |
| secp256k1 base | **3.8** / – | 150 / – | 432 / – | 6.0 / – | 352 / – | – | – | 923 / – |
| P-256 base | **51.9** / – | 241 / – | 433 / – | 237 / – | 445 / – | – | – | 558 / – |
| P-256 scalar | 50.7 / – | 311 / – | 427 / – | **4.0** / – | 605 / – | – | – | 930 / – |
| Tom-256 base | **71.4** / – | – | 430 / – | – | 561 / – | – | – | 928 / – |
| BLS12-381 base | **137** / – | 780 / – | 894 / – | – | 1,257 / – | – | – | 2,051 / – |
| P-521 base | **6.0** / – | – | 1,460 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **1,215** / **1,208** | 8,057 / 7,944 | 25,047 / 25,084 | 18,046 / 18,019 | – | 13,498 / 13,474 | 33,415 / 33,200 |
| secp256k1 base | **1,000** / **1,017** | 7,606 / 7,632 | 18,844 / 18,865 | 17,943 / 17,886 | 33,693 / 33,608 | 13,076 / 13,047 | 34,304 / 33,319 |
| P-256 base | **1,089** / **1,086** | 7,562 / 7,458 | 22,410 / 22,564 | 17,952 / 17,934 | 66,720 / 66,686 | 13,089 / 13,117 | 33,325 / 32,879 |
| P-256 scalar | **1,183** / **1,180** | 8,045 / 8,059 | 24,390 / 24,363 | 17,899 / 17,982 | 35,648 / 35,606 | 13,320 / 13,381 | 33,751 / 32,855 |
| Tom-256 base | **1,192** / **1,181** | 8,083 / 7,920 | – | 17,761 / 17,950 | – | 13,332 / 13,342 | 34,189 / 33,782 |
| BLS12-381 base | **1,795** / **1,799** | 18,161 / 18,195 | 48,325 / 48,207 | 31,321 / 31,271 | – | 24,070 / 24,145 | 73,439 / 73,095 |
| P-521 base | **2,432** / **2,447** | 30,238 / 30,490 | – | 62,460 / 62,043 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 277 / 277 | **274** / **272** | – | 1,816 / 1,831 | – | – | 675 / 677 | – | 2,410 / 2,386 |
| BabyBear | **278** / **279** | 297 / 296 | – | 1,810 / 1,829 | – | – | 680 / 669 | – | 2,540 / 2,518 |
| Goldilocks | **550** / **553** | 580 / 580 | – | 3,572 / 3,584 | – | – | 1,378 / 1,381 | 1,825 / 1,812 | 4,743 / 4,728 |
| BN254 scalar | 9,213 / **9,060** | **9,122** / 9,137 | 24,956 / 25,146 | 19,271 / 19,292 | – | 15,095 / 15,053 | – | – | 43,697 / 44,254 |
| secp256k1 base | 8,585 / 8,605 | **8,572** / **8,538** | 22,522 / 22,671 | 19,542 / 19,440 | 33,059 / 33,877 | 14,909 / 14,888 | – | – | 44,288 / 44,555 |
| P-256 base | 8,786 / **8,664** | **8,716** / 8,700 | 22,878 / 22,732 | 19,562 / 19,460 | 67,049 / 66,830 | 14,816 / 14,855 | – | – | 42,598 / 43,069 |
| P-256 scalar | 9,231 / **9,062** | **9,139** / 9,069 | 25,017 / 24,982 | 19,444 / 19,518 | 47,776 / 47,606 | 15,092 / 15,050 | – | – | 44,474 / 44,496 |
| Tom-256 base | **9,122** / **8,947** | 9,171 / 9,098 | – | 19,415 / 19,400 | – | 15,034 / 15,031 | – | – | 44,398 / 44,368 |
| BLS12-381 base | 21,024 / 20,960 | **20,926** / **20,760** | 49,157 / 48,875 | 34,774 / 34,294 | – | 27,656 / 27,636 | – | – | 101,446 / 101,871 |
| P-521 base | 34,627 / **34,184** | **34,586** / 34,613 | – | 68,848 / 68,267 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | 11.0 / 19.1 | – | – | – | 10.8 / 18.0 |
| BabyBear | **2.4** / **3.0** | 11.0 / 21.1 | – | – | – | 10.8 / 18.0 |
| Goldilocks | **3.3** / **3.0** | 10.9 / 20.2 | – | – | **3.3** / 4.0 | 10.8 / 18.0 |
| BN254 scalar | **23.6** / **24.4** | 53.1 / 83.0 | – | 36.2 / 41.6 | – | 62.2 / 104 |
| secp256k1 base | **26.3** / **28.0** | 53.0 / 83.0 | 22.4\* / 14.5\* | 35.1 / 39.8 | – | 62.1 / 104 |
| P-256 base | **23.7** / **26.8** | 53.2 / 82.9 | 30.5 / 39.6 | 30.5 / 38.9 | – | 62.2 / 103 |
| P-256 scalar | **26.9** / **27.3** | 53.1 / 82.9 | 41.4 / 57.8 | 35.5 / 42.2 | – | 62.1 / 103 |
| Tom-256 base | **27.6** / **26.6** | 53.1 / 83.0 | – | 36.6 / 42.9 | – | 62.2 / 104 |
| BLS12-381 base | **42.9** / **41.8** | 107 / 113 | – | 63.5 / 63.3 | – | 115 / 158 |
| P-521 base | **76.7** / **77.8** | 142 / 183 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.6 / 8.1 |
| BabyBear | **2.5** / **3.0** | 4.5 / 8.0 | – | – | – | – | 8.5 / 7.9 |
| Goldilocks | **2.6** / **3.0** | 4.5 / 8.0 | – | – | – | 2.8 / **3.0** | 10.4 / 8.1 |
| BN254 scalar | **22.9** / 26.4 | 31.0 / 32.0 | – | 24.0 / **26.0** | – | – | 37.3 / 77.1 |
| secp256k1 base | **19.8** / **21.2** | 31.0 / 29.0 | 25.5 / 19.7\* | 22.3 / 25.2 | – | – | 37.6 / 77.1 |
| P-256 base | **17.3** / **18.9** | 30.9 / 27.5 | 20.8 / 26.6 | 20.5 / 25.2 | – | – | 37.5 / 77.4 |
| P-256 scalar | **21.2** / **21.6** | 31.0 / 31.0 | 23.2 / 24.7 | 23.6 / 26.8 | – | – | 36.9 / 77.1 |
| Tom-256 base | **21.0** / **21.4** | 31.0 / 32.0 | – | 23.8 / 26.0 | – | – | 37.7 / 77.0 |
| BLS12-381 base | **37.7** / **37.4** | 54.0 / 101 | – | 43.3 / 45.3 | – | – | 73.0 / 103 |
| P-521 base | **56.2** / **57.5** | 88.7 / 159 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | 29.3 / 46.5 | – | – | – | 51.4 / 59.0 |
| BabyBear | **7.8** / **13.0** | 29.3 / 46.1 | – | – | – | 51.3 / 59.0 |
| Goldilocks | **12.1** / **17.7** | 32.5 / 47.1 | – | – | 12.3 / 18.0 | 51.4 / 59.0 |
| BN254 scalar | **236** / **243** | 431 / 441 | – | 375 / 378 | – | 897 / 938 |
| secp256k1 base | **160** / **171** | 436 / 436 | 158\* / 162\* | 303 / 306 | – | 910 / 944 |
| P-256 base | **177** / **190** | 429 / 437 | 241 / 241 | 243 / 243 | – | 558 / 589 |
| P-256 scalar | **239** / **234** | 430 / 429 | 517 / 512 | 344 / 351 | – | 911 / 938 |
| Tom-256 base | **236** / **238** | 443 / 443 | – | 344 / 342 | – | 909 / 943 |
| BLS12-381 base | **727** / **721** | 912 / 986 | – | 884 / 892 | – | 2,009 / 2,023 |
| P-521 base | **1,019** / **1,064** | 2,114 / 2,118 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square, constant time: a², in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | 28.5 / 45.0 | – | – | – | 46.5 / 55.3 |
| BabyBear | **8.3** / **13.0** | 28.5 / 45.5 | – | – | – | 46.6 / 58.7 |
| Goldilocks | **10.1** / **17.3** | 32.7 / 46.5 | – | – | 12.4 / 20.0 | 46.5 / 58.7 |
| BN254 scalar | **213** / **219** | 318 / 351 | – | 310 / 309 | – | 899 / 925 |
| secp256k1 base | **143** / **154** | 325 / 348 | 98.6\* / 116\* | 215 / 222 | – | 914 / 937 |
| P-256 base | **160** / **155** | 325 / 350 | 240 / 244 | 200 / 178 | – | 560 / 590 |
| P-256 scalar | **214** / **222** | 323 / 349 | 504 / 515 | 269 / 266 | – | 915 / 941 |
| Tom-256 base | **204** / **200** | 323 / 350 | – | 317 / 267 | – | 914 / 943 |
| BLS12-381 base | **577** / **607** | 903 / 974 | – | 670 / 928 | – | 2,010 / 2,024 |
| P-521 base | **559** / **569** | 2,154 / 2,164 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert, constant time: a⁻¹, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **438** / – | 5,729 / – | – | – | – | 3,341 / – |
| BabyBear | **520** / – | 5,775 / – | – | – | – | 3,302 / – |
| Goldilocks | **1,261** / – | 5,736 / – | – | – | 1,772 / – | 7,031 / – |
| BN254 scalar | **12,021** / – | 28,421 / – | – | 232,969 / – | – | 361,392 / – |
| secp256k1 base | **11,868** / – | 28,539 / – | 33,509 / – | 199,247 / – | – | 477,924 / – |
| P-256 base | **11,532** / – | 29,241 / – | 66,059 / – | 170,867 / – | – | 226,095 / – |
| P-256 scalar | **11,880** / – | 28,615 / – | 218,570 / – | 233,387 / – | – | 406,562 / – |
| Tom-256 base | **11,796** / – | 28,767 / – | – | 216,815 / – | – | 340,952 / – |
| BLS12-381 base | **24,008** / – | 49,507 / – | – | 861,046 / – | – | 1,240,089 / – |
| P-521 base | **41,752** / – | 101,758 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **334** / – | 3,422 / – | – | – | 1,741 / – |
| BabyBear | **7,466** / – | 34,149 / – | – | – | – |
| Goldilocks | **12,876** / – | 48,789 / – | – | – | – |
| BN254 scalar | **178,607** / – | 330,187 / – | – | 380,651 / – | – |
| secp256k1 base | **42,199** / – | 160,451 / – | 33,149\* / – | 158,742 / – | 474,951 / – |
| P-256 base | **44,750** / – | 135,828 / – | 64,488 / – | 129,911 / – | 171,337 / – |
| P-256 scalar | **78,923** / – | 158,074 / – | 240,007 / – | 153,029 / – | – |
| Tom-256 base | **65,472** / – | 149,734 / – | – | 183,196 / – | 338,342 / – |
| BLS12-381 base | **309,849** / – | 545,084 / – | – | 797,158 / – | 1,236,367 / – |
| P-521 base | **325,144** / – | 1,528,245 / – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 61.3 / 59.5 | – | – | – |
| BabyBear | **17.0** / **17.0** | 63.9 / 65.1 | – | – | – |
| Goldilocks | **19.0** / **20.1** | 72.4 / 74.2 | – | – | 25.0 / 23.6 |
| BN254 scalar | **106** / **110** | 853 / 853 | – | 1,035 / 1,050 | – |
| secp256k1 base | **61.3** / **68.9** | 858 / 861 | 157 / 165 | 707 / 690 | – |
| P-256 base | **85.3** / **90.3** | 868 / 855 | 484 / 487 | 719 / 735 | – |
| P-256 scalar | **102** / **106** | 858 / 862 | 517 / 513 | 974 / 991 | – |
| Tom-256 base | **95.4** / **100.0** | 867 / 843 | – | 937 / 947 | – |
| BLS12-381 base | **148** / **154** | 1,920 / 1,941 | – | 2,380 / 2,378 | – |
| P-521 base | **138** / **139** | 3,011 / 3,025 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 60.5 / 59.6 | – | – | – |
| BabyBear | **17.1** / **17.0** | 63.9 / 64.8 | – | – | – |
| Goldilocks | **19.1** / **20.1** | 72.4 / 74.3 | – | – | 24.9 / 23.5 |
| BN254 scalar | **487** / **479** | 858 / 870 | – | 756 / 754 | – |
| secp256k1 base | **322** / **316** | 866 / 873 | – | 519 / 515 | – |
| P-256 base | **412** / **408** | 866 / 866 | – | 456 / 452 | – |
| P-256 scalar | **471** / **475** | 862 / 869 | 549 / 548 | 728 / 725 | – |
| Tom-256 base | **465** / **465** | 858 / 861 | – | 692 / 693 | – |
| BLS12-381 base | **1,084** / **1,079** | 1,903 / 1,940 | – | 1,957 / 1,939 | – |
| P-521 base | **1,993** / **2,021** | 3,010 / 3,062 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 36.1 / 35.4 | – | – | – |
| BabyBear | **10.9** / **12.2** | 39.8 / 47.5 | – | – | – |
| Goldilocks | **9.9** / **9.1** | 50.8 / 58.3 | – | – | 13.3 / 13.0 |
| BN254 scalar | **135** / **140** | 506 / 527 | – | 691 / 663 | – |
| secp256k1 base | **20.5** / **25.4** | 500 / 502 | 16.8\* / 5.3\* | 424 / 402 | – |
| P-256 base | **90.1** / **89.5** | 496 / 512 | 275 / 278 | 488 / 487 | – |
| P-256 scalar | 89.8 / 83.2 | 499 / 507 | **41.7** / **57.4** | 678 / 655 | – |
| Tom-256 base | **115** / **119** | 505 / 510 | – | 626 / 609 | – |
| BLS12-381 base | **192** / **185** | 1,081 / 1,088 | – | 1,395 / 1,379 | – |
| P-521 base | **64.6** / **65.7** | 1,668 / 1,674 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 36.1 / 35.4 | – | – | – |
| BabyBear | **10.9** / **11.6** | 39.8 / 47.5 | – | – | – |
| Goldilocks | **9.9** / **9.1** | 50.8 / 58.2 | – | – | 13.3 / 13.0 |
| BN254 scalar | **327** / **314** | 509 / 514 | – | 412 / 392 | – |
| secp256k1 base | **41.3** / **38.6** | 500 / 507 | – | 223 / 226 | – |
| P-256 base | **235** / **218** | 502 / 513 | – | 244 / 246 | – |
| P-256 scalar | 302 / 294 | 499 / 507 | **94.6** / **81.8** | 389 / 387 | – |
| Tom-256 base | **300** / **290** | 503 / 511 | – | 378 / 380 | – |
| BLS12-381 base | **843** / **841** | 1,072 / 1,087 | – | 979 / 994 | – |
| P-521 base | **135** / **138** | 1,693 / 1,691 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 23.4 / 22.1 | – | – | – |
| BabyBear | **11.0** / **11.9** | 27.9 / 28.7 | – | – | – |
| Goldilocks | **9.9** / **8.9** | 36.7 / 36.3 | – | – | 11.5 / 11.1 |
| BN254 scalar | **125** / **125** | 499 / 490 | – | 666 / 652 | – |
| secp256k1 base | **15.5** / **18.0** | 498 / 492 | 14.0\* / 11.0\* | 413 / 392 | – |
| P-256 base | **84.7** / **86.6** | 485 / 486 | 264 / 265 | 493 / 470 | – |
| P-256 scalar | 80.6 / 79.7 | 499 / 487 | **21.2** / **21.2** | 660 / 641 | – |
| Tom-256 base | **113** / **113** | 493 / 481 | – | 631 / 599 | – |
| BLS12-381 base | **183** / **175** | 1,083 / 1,093 | – | 1,381 / 1,332 | – |
| P-521 base | **42.2** / **45.8** | 1,673 / 1,680 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.4** | 23.4 / 22.1 | – | – | – |
| BabyBear | **11.1** / **12.0** | 27.9 / 28.7 | – | – | – |
| Goldilocks | **9.9** / **8.9** | 36.7 / 36.3 | – | – | 11.5 / 11.1 |
| BN254 scalar | **320** / **313** | 497 / 501 | – | 401 / 396 | – |
| secp256k1 base | **33.6** / **32.2** | 501 / 488 | – | 215 / 214 | – |
| P-256 base | **229** / **225** | 493 / 488 | – | 233 / 234 | – |
| P-256 scalar | 295 / 296 | 500 / 490 | **67.8** / **71.9** | 373 / 370 | – |
| Tom-256 base | **300** / **289** | 499 / 486 | – | 364 / 357 | – |
| BLS12-381 base | **828** / **814** | 1,074 / 1,077 | – | 972 / 972 | – |
| P-521 base | **127** / **124** | 1,677 / 1,680 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / – | 18.0 / – | – | – | – |
| BabyBear | **13.5** / – | 21.6 / – | – | – | – |
| Goldilocks | 14.9 / – | 29.3 / – | – | – | **10.6** / – |
| BN254 scalar | **201** / – | 433 / – | – | 612 / – | – |
| secp256k1 base | **3.4** / – | 432 / – | 6.0 / – | 352 / – | – |
| P-256 base | **179** / – | 433 / – | 237 / – | 445 / – | – |
| P-256 scalar | 249 / – | 427 / – | **4.0** / – | 605 / – | – |
| Tom-256 base | **249** / – | 430 / – | – | 561 / – | – |
| BLS12-381 base | **433** / – | 894 / – | – | 1,257 / – | – |
| P-521 base | **6.0** / – | 1,460 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **12,356** / 12,417 | 12,452 / **12,408** | 28,523 / 29,683 | – | 233,313 / 234,318 | 351,604 / 349,714 |
| secp256k1 base | 11,975 / 11,922 | **11,945** / **11,864** | 28,202 / 29,510 | 33,693 / 33,608 | 201,299 / 201,259 | 466,493 / 464,971 |
| P-256 base | **11,826** / **11,832** | 11,948 / 11,860 | 28,340 / 29,445 | 66,720 / 66,686 | 165,894 / 164,896 | 226,037 / 226,683 |
| P-256 scalar | **12,215** / 12,347 | 12,266 / **12,242** | 28,419 / 29,532 | 219,294 / 222,531 | 234,068 / 232,819 | 396,076 / 395,765 |
| Tom-256 base | **12,193** / **12,203** | 12,409 / 12,372 | 28,484 / 29,575 | – | 215,197 / 215,524 | 330,648 / 328,205 |
| BLS12-381 base | **24,787** / **24,797** | 24,832 / 24,856 | 51,013 / 51,387 | – | 874,859 / 874,968 | 1,241,613 / 1,242,284 |
| P-521 base | **42,839** / **42,677** | 43,060 / 42,963 | 107,812 / 105,706 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **455** / **451** | 457 / 454 | 5,959 / 5,530 | – | – | – | 3,507 / 3,533 |
| BabyBear | **546** / **545** | 552 / 553 | 5,995 / 5,677 | – | – | – | 3,567 / 3,525 |
| Goldilocks | 1,313 / **1,308** | **1,308** / 1,310 | 5,955 / 5,610 | – | – | 1,825 / 1,812 | 7,360 / 7,373 |
| BN254 scalar | **12,497** / **12,477** | 12,628 / 12,633 | 28,328 / 29,968 | – | 234,469 / 231,884 | – | 352,590 / 351,888 |
| secp256k1 base | **11,942** / **11,932** | 11,975 / 12,006 | 28,433 / 29,691 | 33,059 / 33,877 | 200,582 / 201,933 | – | 470,012 / 466,725 |
| P-256 base | **12,021** / **11,900** | 12,120 / 12,038 | 28,742 / 30,295 | 67,049 / 66,830 | 165,667 / 163,231 | – | 226,866 / 227,555 |
| P-256 scalar | **12,289** / **12,390** | 12,546 / 12,637 | 28,661 / 29,718 | 219,596 / 222,717 | 233,916 / 234,255 | – | 396,267 / 395,990 |
| Tom-256 base | **12,387** / **12,363** | 12,583 / 12,571 | 28,962 / 29,902 | – | 216,220 / 215,473 | – | 330,827 / 330,389 |
| BLS12-381 base | **25,420** / **25,401** | 25,457 / 25,517 | 51,141 / 50,668 | – | 874,198 / 874,075 | – | 1,239,908 / 1,240,884 |
| P-521 base | **42,601** / **42,935** | 43,149 / 43,011 | 107,498 / 105,569 | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Chrome

### add: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | – | 11.7 / 19.3 | – | – | 4.4 / 7.5\* | – | 10.8 / 18.0 | – |
| BabyBear | **2.4** / **3.0** | – | 11.7 / 19.7 | – | – | **2.4** / **3.0** | – | 10.8 / 18.0 | – |
| Goldilocks | **3.3** / **3.0** | – | 11.3 / 19.1 | – | – | 4.5 / 8.0 | **3.3** / 4.0 | 10.8 / 18.0 | – |
| BN254 scalar | **12.3** / **10.9** | 13.2 / 11.1 | 42.3 / 87.0 | – | 20.7 / 23.0 | – | – | 62.1 / 103 | – |
| secp256k1 base | **11.8** / **10.7** | 12.8 / 11.2 | 42.4 / 87.0 | 22.5 / 14.5 | 19.4 / 23.0 | – | – | 62.2 / 104 | – |
| P-256 base | **12.3** / **11.2** | **12.3** / 11.3 | 42.4 / 87.0 | 17.9 / 21.6 | 18.0 / 21.0 | – | – | 62.1 / 104 | 440 / – |
| P-256 scalar | **11.9** / **11.0** | 13.4 / 11.2 | 42.4 / 87.1 | 41.4 / 58.0 | 20.6 / 24.7 | – | – | 61.9 / 103 | 456 / – |
| Tom-256 base | **11.7** / **11.2** | – | 42.3 / 87.3 | – | 20.2 / 24.0 | – | – | 62.1 / 103 | – |
| BLS12-381 base | **17.8** / **18.8** | 30.2 / 38.0 | 106 / 116 | – | 32.5 / 34.1 | – | – | 115 / 156 | – |
| P-521 base | **26.3** / **28.9** | – | 134 / 179 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### sub: a − b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.9 / 8.1 | – |
| BabyBear | **2.5** / **3.0** | – | 4.5 / 8.0 | – | – | 2.8 / **3.0** | – | 8.8 / 7.9 | – |
| Goldilocks | **2.6** / **3.0** | – | 4.5 / 8.0 | – | – | 3.5 / 5.0 | 2.8 / **3.0** | 10.3 / 8.1 | – |
| BN254 scalar | **11.9** / **10.8** | 17.1 / 14.0 | 26.7 / 27.0 | – | 15.0 / 14.9 | – | – | 37.4 / 76.8 | – |
| secp256k1 base | **11.8** / **10.7** | 16.5 / 14.0 | 26.7 / 27.2 | 25.6 / 19.7 | 14.2 / 14.5 | – | – | 37.7 / 77.1 | – |
| P-256 base | **11.9** / **10.5** | 16.0 / 12.9 | 26.7 / 25.8 | 13.4 / 14.6 | 12.4 / 14.1 | – | – | 37.8 / 77.4 | 231 / – |
| P-256 scalar | **11.9** / **10.7** | 17.1 / 13.9 | 26.7 / 29.0 | 23.1 / 24.3 | 14.4 / 15.1 | – | – | 37.5 / 76.8 | 234 / – |
| Tom-256 base | **11.9** / **10.7** | – | 26.7 / 27.0 | – | 15.0 / 14.9 | – | – | 37.6 / 77.4 | – |
| BLS12-381 base | **17.9** / **17.5** | 27.4 / 23.8 | 43.8 / 97.3 | – | 23.5 / 24.9 | – | – | 73.7 / 104 | – |
| P-521 base | **28.6** / **27.1** | – | 65.0 / 145 | – | – | – | – | – | – |

### mul: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.1** / **12.0** | – | 15.8 / 26.9 | – | – | 5.2 / 12.0 | – | 51.3 / 59.0 | – |
| BabyBear | 5.2 / **11.0** | – | 15.8 / 26.9 | – | – | **4.1** / 12.2 | – | 51.3 / 59.0 | – |
| Goldilocks | 7.0 / **14.4** | – | 16.2 / 27.0 | – | – | 11.6 / 17.7 | **6.5** / 14.5 | 51.4 / 59.0 | – |
| BN254 scalar | 95.9 / **90.8** | 249 / 249 | 133 / 135 | – | **91.4** / 93.5 | – | – | 896 / 931 | – |
| secp256k1 base | **64.1** / **77.5** | 203 / 203 | 133 / 135 | 158 / 161 | 91.0 / 90.3 | – | – | 907 / 943 | – |
| P-256 base | **71.4** / **72.5** | 181 / 181 | 133 / 134 | 85.0 / 78.3 | 86.3 / 75.6 | – | – | 549 / 582 | 1,070 / – |
| P-256 scalar | **86.6** / **93.4** | 248 / 245 | 133 / 135 | 258 / 250 | 93.5 / 95.4 | – | – | 909 / 943 | 1,065 / – |
| Tom-256 base | **80.2** / **87.0** | – | 133 / 135 | – | 94.2 / 94.2 | – | – | 910 / 937 | – |
| BLS12-381 base | 248 / 244 | 553 / 550 | 265 / 311 | – | **195** / **193** | – | – | 2,010 / 2,033 | – |
| P-521 base | **399** / **416** | – | 600 / 617 | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | – | 15.6 / 27.0 | – | – | 5.1\* / 12.0\* | – | 46.5 / 56.4 | – |
| BabyBear | **4.8** / **11.0** | – | 15.6 / 27.0 | – | – | 5.2 / 11.9 | – | 46.5 / 54.3 | – |
| Goldilocks | **6.0** / **12.0** | – | 15.9 / 27.0 | – | – | 9.1 / 18.0 | **6.0** / 12.8 | 46.6 / 55.5 | – |
| BN254 scalar | 95.0 / **90.0** | 201 / 202 | 110 / 109 | – | **84.3** / 95.8 | – | – | 899 / 924 | – |
| secp256k1 base | **60.8** / **69.9** | 162 / 161 | 110 / 108 | 98.6 / 116 | 77.9 / 84.1 | – | – | 916 / 936 | – |
| P-256 base | 78.3 / 62.2 | 136 / 135 | 109 / 108 | **73.6** / 76.7 | 75.9 / **61.3** | – | – | 559 / 593 | 1,072 / – |
| P-256 scalar | 83.6 / **89.5** | 186 / 185 | 110 / 108 | 248 / 254 | **83.0** / 92.5 | – | – | 918 / 939 | 1,066 / – |
| Tom-256 base | **75.9** / **83.0** | – | 109 / 108 | – | 79.8 / 89.5 | – | – | 917 / 938 | – |
| BLS12-381 base | 282 / 281 | 439 / 438 | 296 / 304 | – | **163** / **164** | – | – | 2,011 / 2,030 | – |
| P-521 base | **260** / **263** | – | 614 / 613 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### invert: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **266** / – | – | 1,587 / – | – | – | 672 / – | – | 2,204 / – | – |
| BabyBear | **268** / – | – | 1,545 / – | – | – | 700 / – | – | 2,357 / – | – |
| Goldilocks | **537** / – | – | 3,128 / – | – | – | 1,392 / – | 912 / – | 4,441 / – | – |
| BN254 scalar | **6,417** / – | 22,474 / – | 15,204 / – | – | 8,010 / – | – | – | 42,069 / – | – |
| secp256k1 base | **6,031** / – | 20,417 / – | 15,237 / – | 33,482 / – | 8,001 / – | – | – | 42,556 / – | – |
| P-256 base | **6,062** / – | 20,777 / – | 15,236 / – | 22,180 / – | 8,013 / – | – | – | 40,744 / – | 245,048 / – |
| P-256 scalar | **6,399** / – | 23,069 / – | 15,252 / – | 46,636 / – | 8,029 / – | – | – | 42,589 / – | 248,821 / – |
| Tom-256 base | **6,400** / – | – | 15,253 / – | – | 8,023 / – | – | – | 42,485 / – | – |
| BLS12-381 base | **11,422** / – | 42,184 / – | 26,321 / – | – | 13,127 / – | – | – | 97,980 / – | – |
| P-521 base | **18,196** / – | – | 50,293 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 329 / – | – | 1,637 / – | – | – | **324** / – | 1,741 / – | – |
| BabyBear | **2,040** / – | – | 20,193 / – | – | – | 3,118 / – | – | – |
| Goldilocks | **3,697** / – | – | 28,889 / – | – | – | 5,739 / – | – | – |
| BN254 scalar | **47,313** / – | 123,084 / – | 133,963 / – | – | 102,211 / – | – | – | – |
| secp256k1 base | **19,095** / – | 91,221 / – | 55,374 / – | 33,129 / – | 42,787 / – | – | 475,590 / – | – |
| P-256 base | **16,334** / – | 40,004 / – | 47,122 / – | 21,580 / – | 38,148 / – | – | 170,891 / – | 305,084 / – |
| P-256 scalar | **35,837** / – | 88,463 / – | 54,754 / – | 115,422 / – | 49,826 / – | – | – | – |
| Tom-256 base | **21,187** / – | – | 51,934 / – | – | 45,702 / – | – | 336,024 / – | – |
| BLS12-381 base | **133,690** / – | 284,163 / – | 145,054 / – | – | 154,972 / – | – | 1,234,379 / – | – |
| P-521 base | **143,071** / – | – | 296,403 / – | – | – | – | – | – |

### mul_u64: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.3** / **14.0** | – | 30.5 / 31.4 | – | – | 9.1\* / 12.6\* | – | – |
| BabyBear | **11.2** / **11.0** | – | 33.0 / 32.0 | – | – | 12.0 / 13.3 | – | – |
| Goldilocks | 11.0 / **12.0** | – | 34.6 / 35.8 | – | – | 11.7 / 17.0 | **10.8** / 13.7 | – |
| BN254 scalar | **28.8** / **34.5** | 514 / 519 | 263 / 262 | – | 284 / 291 | – | – | 1,840 / 1,854 |
| secp256k1 base | **12.6** / **16.0** | 336 / 336 | 262 / 259 | 157 / 163 | 269 / 280 | – | – | 1,833 / 1,849 |
| P-256 base | **27.3** / **33.0** | 351 / 351 | 262 / 258 | 150 / 151 | 239 / 251 | – | – | 1,215 / 1,214 |
| P-256 scalar | **30.3** / **34.6** | 492 / 489 | 263 / 259 | 249 / 255 | 287 / 297 | – | – | 1,851 / 1,864 |
| Tom-256 base | **30.0** / **33.0** | – | 262 / 259 | – | 290 / 299 | – | – | 1,832 / 1,860 |
| BLS12-381 base | **44.2** / **47.8** | 1,123 / 1,121 | 509 / 592 | – | 481 / 486 | – | – | 4,086 / 4,094 |
| P-521 base | **32.1** / **39.6** | – | 984 / 994 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.5 / 31.5 | – | – | 9.1\* / 12.6\* | – |
| BabyBear | **11.2** / **11.0** | 33.1 / 32.4 | – | – | 11.6 / 13.3 | – |
| Goldilocks | 11.0 / **12.0** | 34.7 / 35.5 | – | – | 11.6 / 17.8 | **10.8** / 13.7 |
| BN254 scalar | **177** / 185 | 262 / 260 | – | 181 / **181** | – | – |
| secp256k1 base | **113** / **118** | 262 / 260 | – | 154 / 154 | – | – |
| P-256 base | 173 / 180 | 263 / 259 | – | **143** / **145** | – | – |
| P-256 scalar | 188 / 189 | 262 / 260 | 296 / 289 | **183** / **185** | – | – |
| Tom-256 base | **178** / **177** | 262 / 260 | – | 180 / 178 | – | – |
| BLS12-381 base | **380** / 380 | 517 / 593 | – | 382 / **379** | – | – |
| P-521 base | **618** / **644** | 1,008 / 1,011 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | – | 30.4 / 30.1 | – | – | 7.6\* / 8.1\* | – | – |
| BabyBear | **8.6** / **8.2** | – | 31.6 / 30.0 | – | – | 10.0 / 9.2 | – | – |
| Goldilocks | 7.7 / **7.3** | – | 32.5 / 30.9 | – | – | 4.5\* / 8.0 | **7.4** / 7.4 | – |
| BN254 scalar | **53.3** / **44.5** | 272 / 264 | 221 / 228 | – | 238 / 228 | – | – | 1,060 / 1,050 |
| secp256k1 base | **8.0** / **6.0** | 121 / 119 | 221 / 229 | 14.6 / 5.3\* | 233 / 216 | – | – | 1,018 / 1,028 |
| P-256 base | **36.8** / **35.2** | 171 / 162 | 224 / 228 | 96.6 / 92.5 | 214 / 192 | – | – | 706 / 694 |
| P-256 scalar | **40.4** / **38.4** | 249 / 242 | 221 / 229 | 40.9 / 56.9 | 239 / 214 | – | – | 1,054 / 1,051 |
| Tom-256 base | **41.6** / **39.7** | – | 221 / 229 | – | 246 / 223 | – | – | 1,053 / 1,049 |
| BLS12-381 base | **65.9** / **62.0** | 615 / 612 | 406 / 413 | – | 323 / 320 | – | – | 2,218 / 2,219 |
| P-521 base | **16.1** / **12.3** | – | 649 / 653 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **8.7** | 30.4 / 30.1 | – | – | 7.6\* / 8.1\* | – |
| BabyBear | **8.6** / **8.2** | 31.7 / 30.0 | – | – | 10.0 / 9.2 | – |
| Goldilocks | 7.7 / **7.3** | 32.5 / 31.0 | – | – | 4.5\* / 8.0 | **7.4** / 7.4 |
| BN254 scalar | 142 / 136 | 237 / 231 | – | **113** / **111** | – | – |
| secp256k1 base | **14.6** / **12.6** | 238 / 232 | – | 85.4 / 85.1 | – | – |
| P-256 base | **87.2** / **88.0** | 237 / 232 | – | 91.5 / 90.1 | – | – |
| P-256 scalar | 135 / 124 | 238 / 231 | **94.8** / **81.6** | 113 / 113 | – | – |
| Tom-256 base | 134 / 124 | 237 / 232 | – | **110** / **110** | – | – |
| BLS12-381 base | 343 / 320 | 408 / 402 | – | **225** / **224** | – | – |
| P-521 base | **42.0** / **42.0** | 666 / 664 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | – | 17.9 / 18.2 | – | – | 6.1\* / 6.1 | – | – |
| BabyBear | **8.5** / **8.2** | – | 19.5 / 19.4 | – | – | 10.7 / 9.0 | – | – |
| Goldilocks | **7.0** / **6.5** | – | 23.2 / 23.2 | – | – | 3.5\* / 5.0\* | **7.0** / 7.1 | – |
| BN254 scalar | **51.3** / **45.6** | 276 / 266 | 151 / 152 | – | 244 / 214 | – | – | 1,001 / 996 |
| secp256k1 base | **8.0** / **6.1** | 124 / 121 | 150 / 152 | 14.0 / 11.0 | 246 / 213 | – | – | 976 / 973 |
| P-256 base | **36.8** / **35.2** | 172 / 165 | 150 / 150 | 89.4 / 88.0 | 224 / 201 | – | – | 643 / 644 |
| P-256 scalar | 40.2 / 37.3 | 251 / 243 | 152 / 151 | **21.2** / **21.1** | 235 / 215 | – | – | 1,003 / 989 |
| Tom-256 base | **50.1** / **39.2** | – | 151 / 152 | – | 240 / 219 | – | – | 1,001 / 989 |
| BLS12-381 base | **67.6** / **62.8** | 609 / 600 | 386 / 381 | – | 308 / 317 | – | – | 2,153 / 2,168 |
| P-521 base | **18.7** / **15.2** | – | 622 / 627 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 17.9 / 18.2 | – | – | 6.1\* / 6.1 | – |
| BabyBear | **8.5** / **8.2** | 19.5 / 19.4 | – | – | 10.7 / 9.0 | – |
| Goldilocks | **7.0** / **6.5** | 23.2 / 23.2 | – | – | 3.5\* / 5.0\* | **7.0** / 7.0 |
| BN254 scalar | 146 / 138 | 153 / 150 | – | **104** / **105** | – | – |
| secp256k1 base | **14.6** / **12.4** | 152 / 150 | – | 108 / 79.7 | – | – |
| P-256 base | 87.8 / 88.3 | 152 / 149 | – | **85.7** / **85.7** | – | – |
| P-256 scalar | 133 / 124 | 153 / 149 | **70.4** / **72.0** | 105 / 105 | – | – |
| Tom-256 base | 131 / 121 | 153 / 150 | – | **104** / **103** | – | – |
| BLS12-381 base | 348 / 324 | 387 / 390 | – | **216** / **216** | – | – |
| P-521 base | **42.1** / **38.7** | 635 / 638 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 11.6 / – | – | – | **4.1** / – | – | – |
| BabyBear | **7.3** / – | – | 12.6 / – | – | – | 7.5 / – | – | – |
| Goldilocks | **5.5** / – | – | 16.9 / – | – | – | 2.2\* / – | 5.9 / – | – |
| BN254 scalar | **28.2** / – | 248 / – | 132 / – | – | 189 / – | – | – | 925 / – |
| secp256k1 base | **3.8** / – | 114 / – | 133 / – | 6.0 / – | 169 / – | – | – | 928 / – |
| P-256 base | **25.3** / – | 148 / – | 134 / – | 76.0 / – | 159 / – | – | – | 567 / – |
| P-256 scalar | 25.9 / – | 225 / – | 133 / – | **4.0** / – | 190 / – | – | – | 931 / – |
| Tom-256 base | **28.9** / – | – | 133 / – | – | 191 / – | – | – | 934 / – |
| BLS12-381 base | **44.6** / – | 567 / – | 254 / – | – | 291 / – | – | – | 2,051 / – |
| P-521 base | **6.0** / – | – | 506 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **834** / **842** | 5,563 / 5,589 | 22,943 / 23,124 | 14,135 / 14,122 | – | 7,017 / 6,985 | 33,645 / 33,610 |
| secp256k1 base | **766** / **777** | 5,035 / 5,038 | 17,338 / 17,316 | 14,178 / 14,153 | 34,061 / 34,139 | 6,923 / 6,988 | 34,340 / 33,502 |
| P-256 base | **867** / **887** | 5,172 / 5,113 | 20,686 / 20,729 | 14,087 / 14,130 | 22,360 / 23,992 | 6,881 / 6,910 | 32,328 / 32,846 |
| P-256 scalar | **891** / **899** | 5,498 / 5,444 | 22,634 / 22,537 | 14,047 / 14,168 | 36,193 / 35,256 | 6,955 / 6,964 | 34,945 / 33,855 |
| Tom-256 base | **823** / **854** | 5,373 / 5,405 | – | 14,161 / 14,108 | – | 6,918 / 6,927 | 33,930 / 34,243 |
| BLS12-381 base | **1,106** / **1,133** | 9,886 / 9,866 | 42,303 / 42,395 | 23,914 / 23,888 | – | 11,006 / 11,021 | 73,398 / 73,613 |
| P-521 base | **1,369** / **1,375** | 14,914 / 14,903 | – | 45,412 / 45,690 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 277 / 276 | **274** / **272** | – | 1,606 / 1,608 | – | – | 675 / 670 | – | 2,412 / 2,388 |
| BabyBear | **278** / **278** | 289 / 288 | – | 1,582 / 1,593 | – | – | 662 / 655 | – | 2,544 / 2,520 |
| Goldilocks | **544** / **544** | 562 / 562 | – | 3,162 / 3,161 | – | – | 1,384 / 1,384 | 946 / 927 | 4,740 / 4,717 |
| BN254 scalar | 6,685 / **6,650** | **6,601** / 6,686 | 22,797 / 23,014 | 15,648 / 15,586 | – | 8,226 / 8,126 | – | – | 44,156 / 44,045 |
| secp256k1 base | **6,068** / 6,042 | 6,072 / **6,029** | 20,760 / 20,749 | 15,623 / 15,556 | 35,214 / 34,145 | 8,204 / 8,168 | – | – | 44,125 / 44,449 |
| P-256 base | **6,169** / **6,126** | 6,243 / 6,171 | 21,193 / 21,250 | 15,655 / 15,618 | 22,145 / 22,815 | 8,220 / 8,222 | – | – | 42,573 / 42,516 |
| P-256 scalar | **6,641** / **6,538** | 6,673 / 6,657 | 23,587 / 23,602 | 15,693 / 15,568 | 47,473 / 46,910 | 8,240 / 8,246 | – | – | 44,137 / 44,322 |
| Tom-256 base | **6,655** / 6,614 | 6,656 / **6,567** | – | 15,674 / 15,587 | – | 8,247 / 8,234 | – | – | 44,469 / 44,388 |
| BLS12-381 base | 12,074 / **11,932** | **12,043** / 12,010 | 43,376 / 43,316 | 27,031 / 27,021 | – | 13,593 / 13,574 | – | – | 101,731 / 101,507 |
| P-521 base | **18,428** / 18,379 | 18,716 / **18,085** | – | 52,233 / 52,443 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.2** / **7.5** | 11.7 / 19.3 | – | – | – | 10.8 / 18.0 |
| BabyBear | **2.4** / **3.0** | 11.7 / 19.7 | – | – | – | 10.8 / 18.0 |
| Goldilocks | **3.3** / **3.0** | 11.3 / 19.1 | – | – | **3.3** / 4.0 | 10.8 / 18.0 |
| BN254 scalar | **16.3** / **19.5** | 42.3 / 87.0 | – | 20.7 / 23.0 | – | 62.1 / 103 |
| secp256k1 base | **18.2** / **22.6** | 42.4 / 87.0 | 22.5 / 14.5\* | 19.4 / 23.0 | – | 62.2 / 104 |
| P-256 base | 18.1 / 22.8 | 42.4 / 87.0 | **17.9** / 21.6 | 18.0 / **21.0** | – | 62.1 / 104 |
| P-256 scalar | **17.9** / **23.8** | 42.4 / 87.1 | 41.4 / 58.0 | 20.6 / 24.7 | – | 61.9 / 103 |
| Tom-256 base | **18.1** / **23.8** | 42.3 / 87.3 | – | 20.2 / 24.0 | – | 62.1 / 103 |
| BLS12-381 base | **25.7** / **28.5** | 106 / 116 | – | 32.5 / 34.1 | – | 115 / 156 |
| P-521 base | **46.5** / **48.6** | 134 / 179 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.6** / **3.0** | 4.5 / 8.0 | – | – | 3.0 / 5.0 | – | 8.9 / 8.1 |
| BabyBear | **2.6** / **3.0** | 4.5 / 8.0 | – | – | – | – | 8.8 / 7.9 |
| Goldilocks | **2.6** / **3.0** | 4.5 / 8.0 | – | – | – | 2.8 / **3.0** | 10.3 / 8.1 |
| BN254 scalar | 16.6 / 16.9 | 26.7 / 27.0 | – | **15.0** / **14.9** | – | – | 37.4 / 76.8 |
| secp256k1 base | 14.4 / 15.5 | 26.7 / 27.2 | 25.6 / 19.7 | **14.2** / **14.5** | – | – | 37.7 / 77.1 |
| P-256 base | 13.4 / 15.5 | 26.7 / 25.8 | 13.4 / 14.6 | **12.4** / **14.1** | – | – | 37.8 / 77.4 |
| P-256 scalar | 15.0 / 15.8 | 26.7 / 29.0 | 23.1 / 24.3 | **14.4** / **15.1** | – | – | 37.5 / 76.8 |
| Tom-256 base | 15.3 / 17.1 | 26.7 / 27.0 | – | **15.0** / **14.9** | – | – | 37.6 / 77.4 |
| BLS12-381 base | 24.4 / **24.7** | 43.8 / 97.3 | – | **23.5** / 24.9 | – | – | 73.7 / 104 |
| P-521 base | **37.2** / **39.9** | 65.0 / 145 | – | – | – | – | – |

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | 15.8 / 26.9 | – | – | – | 51.3 / 59.0 |
| BabyBear | **5.2** / **11.0** | 15.8 / 26.9 | – | – | – | 51.3 / 59.0 |
| Goldilocks | 7.0 / **14.4** | 16.2 / 27.0 | – | – | **6.5** / 14.5 | 51.4 / 59.0 |
| BN254 scalar | **81.1** / 94.0 | 133 / 135 | – | 91.4 / **93.5** | – | 896 / 931 |
| secp256k1 base | **64.3** / **77.5** | 133 / 135 | 158 / 161 | 91.0 / 90.3 | – | 907 / 943 |
| P-256 base | **72.0** / **72.4** | 133 / 134 | 85.0 / 78.3 | 86.3 / 75.6 | – | 549 / 582 |
| P-256 scalar | **86.6** / **93.5** | 133 / 135 | 258 / 250 | 93.5 / 95.4 | – | 909 / 943 |
| Tom-256 base | **80.1** / **86.5** | 133 / 135 | – | 94.2 / 94.2 | – | 910 / 937 |
| BLS12-381 base | 248 / 244 | 265 / 311 | – | **195** / **193** | – | 2,010 / 2,033 |
| P-521 base | **400** / **417** | 600 / 617 | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / **12.0** | 15.6 / 27.0 | – | – | – | 46.5 / 56.4 |
| BabyBear | **4.9** / **11.0** | 15.6 / 27.0 | – | – | – | 46.5 / 54.3 |
| Goldilocks | **6.0** / **12.0** | 15.9 / 27.0 | – | – | **6.0** / 12.8 | 46.6 / 55.5 |
| BN254 scalar | **77.8** / **82.5** | 110 / 109 | – | 84.3 / 95.8 | – | 899 / 924 |
| secp256k1 base | **61.2** / **69.9** | 110 / 108 | 98.6 / 116 | 77.9 / 84.1 | – | 916 / 936 |
| P-256 base | 78.4 / 62.1 | 109 / 108 | **73.6** / 76.7 | 75.9 / **61.3** | – | 559 / 593 |
| P-256 scalar | **81.6** / **89.1** | 110 / 108 | 248 / 254 | 83.0 / 92.5 | – | 918 / 939 |
| Tom-256 base | 84.4 / **83.0** | 109 / 108 | – | **79.8** / 89.5 | – | 917 / 938 |
| BLS12-381 base | 282 / 292 | 296 / 304 | – | **163** / **164** | – | 2,011 / 2,030 |
| P-521 base | **262** / **261** | 614 / 613 | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **438** / – | 5,299 / – | – | – | – | 3,341 / – |
| BabyBear | **459** / – | 5,336 / – | – | – | – | 3,305 / – |
| Goldilocks | **856** / – | 5,380 / – | – | – | 912 / – | 7,026 / – |
| BN254 scalar | **9,515** / – | 25,649 / – | – | 59,812 / – | – | 361,576 / – |
| secp256k1 base | **9,096** / – | 25,562 / – | 33,482 / – | 52,631 / – | – | 477,019 / – |
| P-256 base | **8,923** / – | 25,223 / – | 22,180 / – | 43,495 / – | – | 224,813 / – |
| P-256 scalar | **9,328** / – | 25,677 / – | 109,613 / – | 59,939 / – | – | 406,957 / – |
| Tom-256 base | **9,293** / – | 25,568 / – | – | 58,354 / – | – | 340,567 / – |
| BLS12-381 base | **16,592** / – | 42,655 / – | – | 166,073 / – | – | 1,242,751 / – |
| P-521 base | **26,001** / – | 84,120 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **334** / – | 1,637 / – | – | – | 1,741 / – |
| BabyBear | **6,456** / – | 20,193 / – | – | – | – |
| Goldilocks | **8,887** / – | 28,889 / – | – | – | – |
| BN254 scalar | **83,838** / – | 133,963 / – | – | 102,211 / – | – |
| secp256k1 base | **19,399** / – | 55,374 / – | 33,129 / – | 42,787 / – | 475,590 / – |
| P-256 base | **15,771** / – | 47,122 / – | 21,580 / – | 38,148 / – | 170,891 / – |
| P-256 scalar | **37,110** / – | 54,754 / – | 115,422 / – | 49,826 / – | – |
| Tom-256 base | **32,400** / – | 51,934 / – | – | 45,702 / – | 336,024 / – |
| BLS12-381 base | **136,709** / – | 145,054 / – | – | 154,972 / – | 1,234,379 / – |
| P-521 base | **135,728** / – | 296,403 / – | – | – | – |

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.5 / 31.4 | – | – | – |
| BabyBear | **11.2** / **11.0** | 33.0 / 32.0 | – | – | – |
| Goldilocks | 11.0 / **12.0** | 34.6 / 35.8 | – | – | **10.8** / 13.7 |
| BN254 scalar | **38.9** / **50.9** | 263 / 262 | – | 284 / 291 | – |
| secp256k1 base | **24.9** / **39.0** | 262 / 259 | 157 / 163 | 269 / 280 | – |
| P-256 base | **40.3** / **53.2** | 262 / 258 | 150 / 151 | 239 / 251 | – |
| P-256 scalar | **43.5** / **55.0** | 263 / 259 | 249 / 255 | 287 / 297 | – |
| Tom-256 base | **41.8** / **52.0** | 262 / 259 | – | 290 / 299 | – |
| BLS12-381 base | **65.8** / **72.0** | 509 / 592 | – | 481 / 486 | – |
| P-521 base | **69.3** / **77.9** | 984 / 994 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **14.0** | 30.5 / 31.5 | – | – | – |
| BabyBear | **11.2** / **11.0** | 33.1 / 32.4 | – | – | – |
| Goldilocks | 11.0 / **12.0** | 34.7 / 35.5 | – | – | **10.8** / 13.7 |
| BN254 scalar | 246 / 251 | 262 / 260 | – | **181** / **181** | – |
| secp256k1 base | 192 / 192 | 262 / 260 | – | **154** / **154** | – |
| P-256 base | 268 / 247 | 263 / 259 | – | **143** / **145** | – |
| P-256 scalar | 261 / 264 | 262 / 260 | 296 / 289 | **183** / **185** | – |
| Tom-256 base | 259 / 257 | 262 / 260 | – | **180** / **178** | – |
| BLS12-381 base | 513 / 506 | 517 / 593 | – | **382** / **379** | – |
| P-521 base | **974** / **998** | 1,008 / 1,011 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 30.4 / 30.1 | – | – | – |
| BabyBear | **8.6** / **8.2** | 31.6 / 30.0 | – | – | – |
| Goldilocks | 7.7 / **7.3** | 32.5 / 30.9 | – | – | **7.4** / 7.4 |
| BN254 scalar | **58.3** / **56.2** | 221 / 228 | – | 238 / 228 | – |
| secp256k1 base | **16.1** / **23.0** | 221 / 229 | 14.6\* / 5.3\* | 233 / 216 | – |
| P-256 base | **58.7** / **56.3** | 224 / 228 | 96.6 / 92.5 | 214 / 192 | – |
| P-256 scalar | 56.8 / 58.4 | 221 / 229 | **40.9** / **56.9** | 239 / 214 | – |
| Tom-256 base | **62.6** / **62.0** | 221 / 229 | – | 246 / 223 | – |
| BLS12-381 base | **93.2** / **96.0** | 406 / 413 | – | 323 / 320 | – |
| P-521 base | **44.1** / **45.1** | 649 / 653 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **8.7** | 30.4 / 30.1 | – | – | – |
| BabyBear | **8.6** / **8.2** | 31.7 / 30.0 | – | – | – |
| Goldilocks | 7.7 / **7.3** | 32.5 / 31.0 | – | – | **7.4** / 7.4 |
| BN254 scalar | 131 / 129 | 237 / 231 | – | **113** / **111** | – |
| secp256k1 base | **29.4** / **29.5** | 238 / 232 | – | 85.4 / 85.1 | – |
| P-256 base | 116 / 108 | 237 / 232 | – | **91.5** / **90.1** | – |
| P-256 scalar | 129 / 136 | 238 / 231 | **94.8** / **81.6** | 113 / 113 | – |
| Tom-256 base | 114 / 128 | 237 / 232 | – | **110** / **110** | – |
| BLS12-381 base | 361 / 351 | 408 / 402 | – | **225** / **224** | – |
| P-521 base | **105** / **93.3** | 666 / 664 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 17.9 / 18.2 | – | – | – |
| BabyBear | **8.5** / **8.2** | 19.5 / 19.4 | – | – | – |
| Goldilocks | **7.0** / **6.6** | 23.2 / 23.2 | – | – | **7.0** / 7.1 |
| BN254 scalar | **60.0** / **56.9** | 151 / 152 | – | 244 / 214 | – |
| secp256k1 base | **12.3** / **15.1** | 150 / 152 | 14.0 / 11.0\* | 246 / 213 | – |
| P-256 base | **52.4** / **49.0** | 150 / 150 | 89.4 / 88.0 | 224 / 201 | – |
| P-256 scalar | 52.5 / 50.4 | 152 / 151 | **21.2** / **21.1** | 235 / 215 | – |
| Tom-256 base | **56.4** / **57.1** | 151 / 152 | – | 240 / 219 | – |
| BLS12-381 base | **86.5** / **90.0** | 386 / 381 | – | 308 / 317 | – |
| P-521 base | **34.1** / **35.4** | 622 / 627 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **5.3** | 17.9 / 18.2 | – | – | – |
| BabyBear | **8.5** / **8.2** | 19.5 / 19.4 | – | – | – |
| Goldilocks | **7.0** / **6.5** | 23.2 / 23.2 | – | – | **7.0** / 7.0 |
| BN254 scalar | 137 / 133 | 153 / 150 | – | **104** / **105** | – |
| secp256k1 base | **26.0** / **24.9** | 152 / 150 | – | 108 / 79.7 | – |
| P-256 base | 106 / 98.2 | 152 / 149 | – | **85.7** / **85.7** | – |
| P-256 scalar | 122 / 127 | 153 / 149 | **70.4** / **72.0** | 105 / 105 | – |
| Tom-256 base | 129 / 127 | 153 / 150 | – | **104** / **103** | – |
| BLS12-381 base | 364 / 346 | 387 / 390 | – | **216** / **216** | – |
| P-521 base | **86.9** / **85.3** | 635 / 638 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / – | 11.6 / – | – | – | – |
| BabyBear | **10.3** / – | 12.6 / – | – | – | – |
| Goldilocks | 9.3 / – | 16.9 / – | – | – | **5.9** / – |
| BN254 scalar | **81.0** / – | 132 / – | – | 189 / – | – |
| secp256k1 base | **3.4** / – | 133 / – | 6.0 / – | 169 / – | – |
| P-256 base | **49.9** / – | 134 / – | 76.0 / – | 159 / – | – |
| P-256 scalar | 81.5 / – | 133 / – | **4.0** / – | 190 / – | – |
| Tom-256 base | **79.0** / – | 133 / – | – | 191 / – | – |
| BLS12-381 base | **155** / – | 254 / – | – | 291 / – | – |
| P-521 base | **6.0** / – | 506 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **9,500** / 9,574 | 9,573 / **9,449** | 24,187 / 25,834 | – | 61,606 / 61,645 | 352,051 / 351,425 |
| secp256k1 base | **9,072** / **9,049** | 9,111 / 9,093 | 24,339 / 25,819 | 34,061 / 34,139 | 53,609 / 53,612 | 466,788 / 460,817 |
| P-256 base | **9,157** / **9,131** | 9,189 / 9,157 | 24,375 / 25,533 | 22,360 / 23,992 | 44,366 / 43,287 | 226,539 / 225,070 |
| P-256 scalar | **9,471** / **9,427** | 9,641 / 9,594 | 24,270 / 25,876 | 109,928 / 144,510 | 59,900 / 59,700 | 396,123 / 395,065 |
| Tom-256 base | **9,268** / 9,343 | 9,542 / **9,325** | 24,352 / 25,865 | – | 58,464 / 57,996 | 331,044 / 329,381 |
| BLS12-381 base | **16,844** / **16,774** | 17,012 / 16,864 | 43,619 / 43,318 | – | 162,134 / 162,028 | 1,242,613 / 1,242,346 |
| P-521 base | **26,172** / **25,998** | 26,715 / 26,801 | 84,766 / 87,479 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Chrome

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **455** / **451** | 457 / 454 | 5,551 / 5,146 | – | – | – | 3,506 / 3,539 |
| BabyBear | 466 / **437** | **439** / 441 | 5,548 / 5,240 | – | – | – | 3,535 / 3,526 |
| Goldilocks | 948 / 946 | 953 / 948 | 5,564 / 5,197 | – | – | **946** / **927** | 7,361 / 7,457 |
| BN254 scalar | 9,578 / **9,545** | **9,503** / 9,618 | 24,876 / 26,236 | – | 62,215 / 61,402 | – | 352,629 / 350,955 |
| secp256k1 base | **9,100** / **9,054** | 9,135 / 9,102 | 24,783 / 26,195 | 35,214 / 34,145 | 53,312 / 53,585 | – | 468,433 / 466,757 |
| P-256 base | **9,104** / **9,122** | 9,291 / 9,188 | 24,534 / 25,533 | 22,145 / 22,815 | 44,472 / 43,090 | – | 226,451 / 226,604 |
| P-256 scalar | **9,507** / **9,496** | 9,654 / 9,642 | 24,868 / 26,208 | 109,797 / 143,166 | 60,540 / 59,633 | – | 396,090 / 395,036 |
| Tom-256 base | **9,458** / 9,432 | 9,494 / **9,281** | 24,963 / 26,262 | – | 58,239 / 56,483 | – | 331,118 / 331,397 |
| BLS12-381 base | **16,995** / **16,947** | 17,178 / 17,157 | 43,537 / 43,382 | – | 160,480 / 160,883 | – | 1,244,751 / 1,241,073 |
| P-521 base | **26,251** / **26,146** | 26,970 / 26,690 | 121,034 / 85,263 | – | – | – | – |

## Results in WebAssembly, Firefox

### add: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | – | 10.8 / 19.5 | – | – | 4.4\* / 7.8 | – | 10.9 / 17.7 | – |
| BabyBear | 3.0 / **3.0** | – | 10.8 / 19.8 | – | – | **2.7** / **3.0** | – | 10.9 / 17.8 | – |
| Goldilocks | **3.1** / **3.0** | – | 12.0 / 19.2 | – | – | 4.0 / 6.0 | 4.0 / 4.7 | 10.9 / 17.7 | – |
| BN254 scalar | 17.1 / 14.7 | **14.1** / **13.6** | 63.4 / 90.6 | – | 39.4 / 45.1 | – | – | 76.6 / 116 | – |
| secp256k1 base | **14.9** / **13.9** | 16.7 / 14.8 | 64.4 / 90.4 | 16.1 / 15.6 | 35.3 / 38.9 | – | – | 76.4 / 115 | – |
| P-256 base | **15.0** / **14.4** | 17.4 / 15.7 | 64.2 / 90.4 | 33.7 / 39.8 | 33.6 / 39.6 | – | – | 76.7 / 134 | 526 / – |
| P-256 scalar | **16.1** / **14.7** | 17.1 / 15.5 | 64.2 / 90.5 | 55.8 / 60.1 | 38.1 / 42.6 | – | – | 76.5 / 134 | 525 / – |
| Tom-256 base | **15.8** / **14.5** | – | 62.7 / 89.5 | – | 38.6 / 43.0 | – | – | 76.7 / 116 | – |
| BLS12-381 base | **24.9** / **24.8** | 39.8 / 45.4 | 95.6 / 141 | – | 71.9 / 67.0 | – | – | 128 / 173 | – |
| P-521 base | **42.4** / **46.6** | – | 178 / 247 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### sub: a − b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 16.6 / 15.0 | – |
| BabyBear | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / **3.0** | – | 16.8 / 14.8 | – |
| Goldilocks | **3.0** / **3.0** | – | 5.0 / 8.0 | – | – | 4.3 / 5.0 | **3.0** / **3.0** | 18.5 / 13.0 | – |
| BN254 scalar | **15.1** / **14.5** | 19.0 / 17.0 | 34.5 / 30.0 | – | 23.3 / 26.0 | – | – | 54.5 / 73.8 | – |
| secp256k1 base | **15.6** / **13.5** | 19.6 / 17.1 | 34.5 / 32.0 | 15.8 / 18.1 | 20.2 / 25.2 | – | – | 54.9 / 74.2 | – |
| P-256 base | **14.9** / **12.7** | 18.5 / 15.3 | 34.6 / 28.8 | 21.7 / 24.6 | 21.1 / 24.4 | – | – | 54.4 / 91.6 | 269 / – |
| P-256 scalar | **15.5** / **14.0** | 18.7 / 16.5 | 34.6 / 33.6 | 26.6 / 26.9 | 23.1 / 27.2 | – | – | 54.6 / 91.7 | 273 / – |
| Tom-256 base | **15.5** / **13.7** | – | 34.5 / 30.2 | – | 23.4 / 26.4 | – | – | 54.5 / 73.9 | – |
| BLS12-381 base | **24.8** / **24.8** | 31.4 / 30.8 | 60.6 / 110 | – | 44.9 / 44.1 | – | – | 76.9 / 104 | – |
| P-521 base | **42.4** / **43.1** | – | 124 / 212 | – | – | – | – | – | – |

### mul: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | – | 38.8 / 50.0 | – | – | 5.6 / 12.0 | – | 51.6 / 56.0 | – |
| BabyBear | 7.8 / 14.5 | – | 39.2 / 48.5 | – | – | **6.5** / **13.0** | – | 51.6 / 56.0 | – |
| Goldilocks | **12.4** / 22.0 | – | 37.8 / 52.1 | – | – | 13.1 / 18.0\* | 17.1 / **19.0** | 51.5 / 56.1 | – |
| BN254 scalar | **386** / **365** | 404 / 380 | 901 / 891 | – | 698 / 690 | – | – | 1,059 / 1,075 | – |
| secp256k1 base | **218** / **221** | 407 / 396 | 892 / 886 | 171\* / 171\* | 594 / 587 | – | – | 1,069 / 1,086 | – |
| P-256 base | **284** / **292** | 484 / 470 | 886 / 881 | 473 / 461 | 486 / 483 | – | – | 851 / 837 | 1,289 / – |
| P-256 scalar | **319** / **320** | 422 / 409 | 881 / 884 | 828 / 813 | 710 / 693 | – | – | 1,065 / 1,082 | 1,289 / – |
| Tom-256 base | **304** / **312** | – | 892 / 885 | – | 608 / 588 | – | – | 1,069 / 1,081 | – |
| BLS12-381 base | **916** / **908** | 1,061 / 1,053 | 1,290 / 1,308 | – | 1,799 / 1,737 | – | – | 2,177 / 2,173 | – |
| P-521 base | **1,637** / **1,605** | – | 2,716 / 2,780 | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | – | 38.3 / 47.7 | – | – | 5.4\* / 12.0\* | – | 49.1 / 56.0 | – |
| BabyBear | 8.0 / 14.0 | – | 38.2 / 52.7 | – | – | **5.7** / **13.0** | – | 49.0 / 56.0 | – |
| Goldilocks | **12.0** / 21.2 | – | 39.5 / 52.0 | – | – | 11.2\* / 18.0\* | 17.0 / **20.0** | 49.1 / 56.0 | – |
| BN254 scalar | 374 / 363 | **359** / **345** | 643 / 650 | – | 543 / 555 | – | – | 1,066 / 1,062 | – |
| secp256k1 base | **192** / **198** | 306 / 298 | 654 / 644 | 116\* / 136\* | 430 / 456 | – | – | 1,076 / 1,072 | – |
| P-256 base | **220** / **225** | 341 / 330 | 647 / 646 | 456 / 456 | 341 / 341 | – | – | 817 / 670 | 1,288 / – |
| P-256 scalar | **281** / **240** | 350 / 343 | 659 / 645 | 818 / 818 | 554 / 551 | – | – | 1,073 / 1,074 | 1,281 / – |
| Tom-256 base | **226** / **234** | – | 662 / 660 | – | 494 / 474 | – | – | 1,076 / 1,075 | – |
| BLS12-381 base | 887 / 856 | **825** / **795** | 1,343 / 1,336 | – | 1,296 / 1,236 | – | – | 2,175 / 2,154 | – |
| P-521 base | **909** / **908** | – | 2,798 / 2,812 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **268** / – | – | 2,127 / – | – | – | 718 / – | – | 1,987 / – | – |
| BabyBear | **277** / – | – | 2,108 / – | – | – | 728 / – | – | 2,068 / – | – |
| Goldilocks | **563** / – | – | 4,214 / – | – | – | 1,514 / – | 1,837 / – | 4,003 / – | – |
| BN254 scalar | **11,668** / – | 25,225 / – | 23,554 / – | – | 21,266 / – | – | – | 52,264 / – | – |
| secp256k1 base | **11,702** / – | 24,007 / – | 23,617 / – | 37,370 / – | 21,577 / – | – | – | 52,251 / – | – |
| P-256 base | **11,269** / – | 24,393 / – | 23,624 / – | 123,557 / – | 21,387 / – | – | – | 50,986 / – | 255,326 / – |
| P-256 scalar | **11,980** / – | 25,792 / – | 23,597 / – | 63,566 / – | 21,472 / – | – | – | 52,705 / – | 255,460 / – |
| Tom-256 base | **11,444** / – | – | 23,584 / – | – | 21,426 / – | – | – | 52,609 / – | – |
| BLS12-381 base | **24,208** / – | 49,792 / – | 43,920 / – | – | 37,675 / – | – | – | 124,735 / – | – |
| P-521 base | **47,873** / – | – | 79,649 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 330 / – | – | 3,728 / – | – | – | **329** / – | 1,747 / – | – |
| BabyBear | **2,760** / – | – | 38,424 / – | – | – | 2,989 / – | – | – |
| Goldilocks | **6,762** / – | – | 54,915 / – | – | – | 6,042\* / – | – | – |
| BN254 scalar | **196,081** / – | 201,732 / – | 583,732 / – | – | 521,248 / – | – | – | – |
| secp256k1 base | **72,835** / – | 176,941 / – | 244,994 / – | 37,006\* / – | 278,949 / – | – | 534,545 / – | – |
| P-256 base | **78,047** / – | 98,628 / – | 192,719 / – | 121,574 / – | 212,390 / – | – | 241,212 / – | 370,757 / – |
| P-256 scalar | **113,654** / – | 157,635 / – | 240,940 / – | 364,463 / – | 273,774 / – | – | – | – |
| Tom-256 base | **94,927** / – | – | 223,204 / – | – | 289,606 / – | – | 375,453 / – | – |
| BLS12-381 base | **423,946** / – | 567,436 / – | 617,309 / – | – | 1,206,737 / – | – | 1,323,194 / – | – |
| P-521 base | **473,856** / – | – | 1,565,745 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.6** / **14.0** | – | 69.6 / 71.6 | – | – | 9.6\* / 12.8\* | – | – |
| BabyBear | **20.1** / **17.1** | – | 86.1 / 90.3 | – | – | 45.9 / 45.0 | – | – |
| Goldilocks | **21.1** / **25.5** | – | 92.3 / 92.1 | – | – | 13.3\* / 18.0\* | 32.7 / 32.1 | – |
| BN254 scalar | **124** / **120** | 828 / 811 | 1,760 / 1,688 | – | 1,488 / 1,469 | – | – | 2,151 / 2,154 |
| secp256k1 base | **65.1** / **65.0** | 633 / 613 | 1,754 / 1,740 | 167 / 175 | 1,045 / 1,048 | – | – | 2,150 / 2,149 |
| P-256 base | **94.2** / **89.1** | 878 / 848 | 1,768 / 1,727 | 916 / 890 | 1,068 / 1,037 | – | – | 1,448 / 1,447 |
| P-256 scalar | **113** / **109** | 808 / 816 | 1,762 / 1,735 | 818 / 814 | 1,446 / 1,474 | – | – | 2,277 / 2,331 |
| Tom-256 base | **115** / **105** | – | 1,761 / 1,717 | – | 1,246 / 1,284 | – | – | 2,155 / 2,162 |
| BLS12-381 base | **167** / **158** | 2,139 / 2,137 | 2,493 / 2,507 | – | 3,571 / 3,511 | – | – | 4,844 / 4,578 |
| P-521 base | **232** / **161** | – | 3,977 / 3,994 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | 72.3 / 72.9 | – | – | 9.6\* / 12.8\* | – |
| BabyBear | **20.0** / **17.0** | 85.5 / 84.6 | – | – | 35.1 / 33.7 | – |
| Goldilocks | **21.1** / **25.5** | 94.1 / 88.6 | – | – | 13.3\* / 18.0\* | 32.7 / 32.1 |
| BN254 scalar | **542** / **532** | 1,771 / 1,732 | – | 1,347 / 1,342 | – | – |
| secp256k1 base | **324** / **323** | 1,783 / 1,731 | – | 971 / 969 | – | – |
| P-256 base | **430** / **421** | 1,781 / 1,738 | – | 994 / 949 | – | – |
| P-256 scalar | **473** / **474** | 1,773 / 1,746 | 886 / 880 | 1,445 / 1,380 | – | – |
| Tom-256 base | **488** / **484** | 1,780 / 1,743 | – | 1,183 / 1,175 | – | – |
| BLS12-381 base | **1,155** / **1,147** | 2,501 / 2,477 | – | 3,545 / 3,520 | – | – |
| P-521 base | **2,383** / **2,381** | 3,978 / 3,996 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.3** | – | 40.2 / 42.0 | – | – | 7.8\* / 7.1\* | – | – |
| BabyBear | **13.0** / **12.7** | – | 52.4 / 52.4 | – | – | 44.0 / 41.7 | – | – |
| Goldilocks | **11.1** / **10.7** | – | 58.2 / 61.1 | – | – | 4.0\* / 6.0\* | 19.1 / 19.1 | – |
| BN254 scalar | **178** / **142** | 429 / 398 | 965 / 936 | – | 797 / 792 | – | – | 1,224 / 1,208 |
| secp256k1 base | **8.0** / **8.0** | 228 / 222 | 962 / 937 | 16.0 / 5.4\* | 517 / 496 | – | – | 1,197 / 1,191 |
| P-256 base | **85.7** / **77.5** | 409 / 400 | 966 / 938 | 491 / 501 | 597 / 600 | – | – | 982 / 983 |
| P-256 scalar | 72.5 / 74.2 | 406 / 398 | 968 / 941 | **54.6** / **63.0** | 855 / 829 | – | – | 1,307 / 1,320 |
| Tom-256 base | **109** / **100** | – | 974 / 945 | – | 737 / 705 | – | – | 1,218 / 1,211 |
| BLS12-381 base | **258** / **265** | 1,071 / 1,060 | 1,384 / 1,370 | – | 1,681 / 1,691 | – | – | 2,509 / 2,441 |
| P-521 base | **22.1** / **19.5** | – | 2,182 / 2,209 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.2** | 40.2 / 43.4 | – | – | 7.8\* / 7.2\* | – |
| BabyBear | **13.0** / **12.7** | 52.4 / 52.3 | – | – | 34.8 / 31.1 | – |
| Goldilocks | **11.1** / **10.7** | 57.1 / 61.0 | – | – | 4.0\* / 6.0\* | 19.1 / 19.1 |
| BN254 scalar | **414** / **421** | 979 / 939 | – | 675 / 681 | – | – |
| secp256k1 base | **18.5** / **16.6** | 977 / 957 | – | 425 / 431 | – | – |
| P-256 base | **333** / **316** | 977 / 957 | – | 513 / 512 | – | – |
| P-256 scalar | 382 / 371 | 983 / 950 | **120** / **110** | 717 / 716 | – | – |
| Tom-256 base | **374** / **363** | 975 / 952 | – | 622 / 624 | – | – |
| BLS12-381 base | **1,036** / **1,002** | 1,386 / 1,388 | – | 1,579 / 1,587 | – | – |
| P-521 base | **86.7** / **85.4** | 2,189 / 2,210 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **6.6** | – | 30.7 / 30.3 | – | – | 6.3\* / 6.1\* | – | – |
| BabyBear | **13.0** / **12.5** | – | 42.2 / 42.3 | – | – | 44.1 / 41.8 | – | – |
| Goldilocks | **11.4** / **10.7** | – | 46.0 / 48.1 | – | – | 4.3\* / 5.0\* | 19.0 / 18.1 | – |
| BN254 scalar | **139** / **136** | 402 / 401 | 950 / 977 | – | 770 / 755 | – | – | 1,172 / 1,161 |
| secp256k1 base | **8.0** / **7.0** | 231 / 221 | 958 / 949 | 14.0 / 11.0 | 500 / 486 | – | – | 1,136 / 1,139 |
| P-256 base | **80.9** / **80.2** | 404 / 400 | 931 / 950 | 488 / 489 | 584 / 587 | – | – | 931 / 913 |
| P-256 scalar | 75.5 / 73.6 | 400 / 396 | 949 / 969 | **24.5** / **24.4** | 841 / 813 | – | – | 1,269 / 1,298 |
| Tom-256 base | **106** / **104** | – | 935 / 970 | – | 716 / 690 | – | – | 1,167 / 1,164 |
| BLS12-381 base | **250** / **272** | 1,106 / 1,049 | 1,358 / 1,365 | – | 1,651 / 1,677 | – | – | 2,433 / 2,376 |
| P-521 base | **20.5** / **22.2** | – | 2,159 / 2,185 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **6.6** | 30.7 / 30.4 | – | – | 6.3\* / 6.0\* | – |
| BabyBear | **13.0** / **13.0** | 42.2 / 42.3 | – | – | 34.9 / 31.3 | – |
| Goldilocks | **11.3** / **10.4** | 47.4 / 47.4 | – | – | 4.3\* / 5.0\* | 19.0 / 18.1 |
| BN254 scalar | **417** / **420** | 956 / 974 | – | 654 / 662 | – | – |
| secp256k1 base | **19.3** / **16.8** | 940 / 953 | – | 411 / 412 | – | – |
| P-256 base | **376** / **382** | 928 / 941 | – | 499 / 504 | – | – |
| P-256 scalar | 391 / 379 | 956 / 963 | **87.6** / **95.0** | 692 / 689 | – | – |
| Tom-256 base | **369** / **363** | 932 / 963 | – | 607 / 616 | – | – |
| BLS12-381 base | **1,007** / **999** | 1,355 / 1,361 | – | 1,587 / 1,562 | – | – |
| P-521 base | **79.1** / **75.2** | 2,168 / 2,198 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 5.2 / – | – | 23.9 / – | – | – | **3.8** / – | – | – |
| BabyBear | **12.9** / – | – | 35.1 / – | – | – | 41.1 / – | – | – |
| Goldilocks | **9.0** / – | – | 39.2 / – | – | – | 2.2\* / – | 17.0 / – | – |
| BN254 scalar | **121** / – | 370 / – | 881 / – | – | 723 / – | – | – | 1,080 / – |
| secp256k1 base | **3.8** / – | 222 / – | 891 / – | 5.9 / – | 448 / – | – | – | 1,068 / – |
| P-256 base | **61.0** / – | 380 / – | 887 / – | 465 / – | 554 / – | – | – | 807 / – |
| P-256 scalar | 58.6 / – | 377 / – | 886 / – | **4.0** / – | 741 / – | – | – | 1,084 / – |
| Tom-256 base | **85.8** / – | – | 874 / – | – | 658 / – | – | – | 1,079 / – |
| BLS12-381 base | **217** / – | 998 / – | 1,256 / – | – | 1,647 / – | – | – | 2,321 / – |
| P-521 base | **6.0** / – | – | 1,964 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **1,432** / **1,432** | 11,576 / 11,572 | 26,560 / 26,678 | 23,890 / 23,899 | – | 19,335 / 19,327 | 44,334 / 43,619 |
| secp256k1 base | **1,233** / **1,228** | 11,095 / 11,007 | 20,546 / 20,507 | 23,859 / 23,824 | 37,487 / 37,435 | 18,951 / 18,889 | 45,099 / 44,425 |
| P-256 base | **1,350** / **1,342** | 11,082 / 10,943 | 25,033 / 25,109 | 23,811 / 23,834 | 126,830 / 126,017 | 18,888 / 18,888 | 43,247 / 43,592 |
| P-256 scalar | **1,422** / **1,419** | 11,384 / 11,391 | 25,838 / 25,830 | 23,839 / 23,863 | 48,935 / 48,737 | 19,404 / 19,216 | 45,117 / 44,162 |
| Tom-256 base | **1,493** / **1,484** | 11,022 / 10,969 | – | 23,884 / 23,872 | – | 19,200 / 19,081 | 44,655 / 44,614 |
| BLS12-381 base | **2,475** / **2,464** | 23,672 / 23,710 | 50,487 / 50,840 | 43,162 / 43,266 | – | 32,205 / 32,340 | 101,898 / 102,299 |
| P-521 base | **3,577** / **3,585** | 44,669 / 44,768 | – | 77,902 / 77,832 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **287** / **283** | **287** / 285 | – | 2,230 / 2,231 | – | – | 736 / 724 | – | 2,098 / 2,113 |
| BabyBear | **284** / **287** | 309 / 307 | – | 2,230 / 2,222 | – | – | 753 / 719 | – | 2,184 / 2,172 |
| Goldilocks | **594** / **591** | 623 / 619 | – | 4,318 / 4,282 | – | – | 1,527 / 1,508 | 1,894 / 1,892 | 4,127 / 4,135 |
| BN254 scalar | **12,642** / 12,599 | 12,770 / **12,574** | 26,625 / 26,740 | 25,326 / 25,404 | – | 23,032 / 22,881 | – | – | 54,445 / 54,137 |
| secp256k1 base | **11,908** / 11,968 | 12,078 / **11,898** | 24,412 / 24,580 | 25,389 / 25,488 | 37,595 / 37,525 | 22,760 / 22,606 | – | – | 54,659 / 54,949 |
| P-256 base | 12,357 / **12,034** | **12,252** / 12,248 | 25,500 / 25,623 | 25,450 / 25,406 | 126,719 / 127,328 | 22,601 / 22,392 | – | – | 54,430 / 54,284 |
| P-256 scalar | **12,629** / 12,578 | 12,715 / **12,557** | 26,871 / 26,846 | 25,466 / 25,438 | 64,278 / 64,058 | 23,112 / 23,101 | – | – | 54,301 / 54,848 |
| Tom-256 base | **12,167** / **12,177** | 12,314 / 12,187 | – | 25,471 / 25,425 | – | 23,023 / 22,893 | – | – | 54,481 / 54,752 |
| BLS12-381 base | **26,726** / 26,684 | 26,727 / **26,566** | 51,557 / 52,051 | 46,524 / 46,559 | – | 40,048 / 39,855 | – | – | 128,504 / 128,570 |
| P-521 base | **49,325** / 49,547 | 49,593 / **49,318** | – | 84,182 / 83,242 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | 10.8 / 19.5 | – | – | – | 10.9 / 17.7 |
| BabyBear | **3.0** / **3.0** | 10.8 / 19.8 | – | – | – | 10.9 / 17.8 |
| Goldilocks | **3.1** / **3.0** | 12.0 / 19.2 | – | – | 4.0 / 4.7 | 10.9 / 17.7 |
| BN254 scalar | **20.6** / **23.1** | 63.4 / 90.6 | – | 39.4 / 45.1 | – | 76.6 / 116 |
| secp256k1 base | **24.5** / **27.0** | 64.4 / 90.4 | 16.1\* / 15.6\* | 35.3 / 38.9 | – | 76.4 / 115 |
| P-256 base | **22.8** / **24.9** | 64.2 / 90.4 | 33.7 / 39.8 | 33.6 / 39.6 | – | 76.7 / 134 |
| P-256 scalar | **24.1** / **26.8** | 64.2 / 90.5 | 55.8 / 60.1 | 38.1 / 42.6 | – | 76.5 / 134 |
| Tom-256 base | **24.1** / **27.0** | 62.7 / 89.5 | – | 38.6 / 43.0 | – | 76.7 / 116 |
| BLS12-381 base | **35.4** / **35.3** | 95.6 / 141 | – | 71.9 / 67.0 | – | 128 / 173 |
| P-521 base | **77.3** / **80.4** | 178 / 247 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 16.6 / 15.0 |
| BabyBear | **2.7** / **3.0** | 5.0 / 8.0 | – | – | – | – | 16.8 / 14.8 |
| Goldilocks | **3.0** / **3.0** | 5.0 / 8.0 | – | – | – | **3.0** / **3.0** | 18.5 / 13.0 |
| BN254 scalar | **21.4** / 27.0 | 34.5 / 30.0 | – | 23.3 / **26.0** | – | – | 54.5 / 73.8 |
| secp256k1 base | **18.6** / **20.4** | 34.5 / 32.0 | 15.8\* / 18.1\* | 20.2 / 25.2 | – | – | 54.9 / 74.2 |
| P-256 base | **18.4** / **18.9** | 34.6 / 28.8 | 21.7 / 24.6 | 21.1 / 24.4 | – | – | 54.4 / 91.6 |
| P-256 scalar | **22.2** / **18.8** | 34.6 / 33.6 | 26.6 / 26.9 | 23.1 / 27.2 | – | – | 54.6 / 91.7 |
| Tom-256 base | **20.7** / **19.8** | 34.5 / 30.2 | – | 23.4 / 26.4 | – | – | 54.5 / 73.9 |
| BLS12-381 base | **34.7** / **36.6** | 60.6 / 110 | – | 44.9 / 44.1 | – | – | 76.9 / 104 |
| P-521 base | **58.0** / **59.9** | 124 / 212 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | 38.8 / 50.0 | – | – | – | 51.6 / 56.0 |
| BabyBear | **7.8** / **14.5** | 39.2 / 48.5 | – | – | – | 51.6 / 56.0 |
| Goldilocks | **12.4** / 22.0 | 37.8 / 52.1 | – | – | 17.1 / **19.0** | 51.5 / 56.1 |
| BN254 scalar | **415** / **411** | 901 / 891 | – | 698 / 690 | – | 1,059 / 1,075 |
| secp256k1 base | **212** / **226** | 892 / 886 | 171\* / 171\* | 594 / 587 | – | 1,069 / 1,086 |
| P-256 base | **281** / **292** | 886 / 881 | 473 / 461 | 486 / 483 | – | 851 / 837 |
| P-256 scalar | **316** / **320** | 881 / 884 | 828 / 813 | 710 / 693 | – | 1,065 / 1,082 |
| Tom-256 base | **304** / **311** | 892 / 885 | – | 608 / 588 | – | 1,069 / 1,081 |
| BLS12-381 base | **916** / **914** | 1,290 / 1,308 | – | 1,799 / 1,737 | – | 2,177 / 2,173 |
| P-521 base | **1,643** / **1,604** | 2,716 / 2,780 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### square, constant time: a², in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | 38.3 / 47.7 | – | – | – | 49.1 / 56.0 |
| BabyBear | **8.0** / **14.0** | 38.2 / 52.7 | – | – | – | 49.0 / 56.0 |
| Goldilocks | **12.0** / 21.2 | 39.5 / 52.0 | – | – | 17.0 / **20.0** | 49.1 / 56.0 |
| BN254 scalar | **230** / **220** | 643 / 650 | – | 543 / 555 | – | 1,066 / 1,062 |
| secp256k1 base | **192** / **198** | 654 / 644 | 116\* / 136\* | 430 / 456 | – | 1,076 / 1,072 |
| P-256 base | **253** / **227** | 647 / 646 | 456 / 456 | 341 / 341 | – | 817 / 670 |
| P-256 scalar | **280** / **238** | 659 / 645 | 818 / 818 | 554 / 551 | – | 1,073 / 1,074 |
| Tom-256 base | **226** / **234** | 662 / 660 | – | 494 / 474 | – | 1,076 / 1,075 |
| BLS12-381 base | **889** / **853** | 1,343 / 1,336 | – | 1,296 / 1,236 | – | 2,175 / 2,154 |
| P-521 base | **915** / **912** | 2,798 / 2,812 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### invert, constant time: a⁻¹, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **430** / – | 6,828 / – | – | – | – | 3,315 / – |
| BabyBear | **550** / – | 6,846 / – | – | – | – | 3,329 / – |
| Goldilocks | **1,559** / – | 6,861 / – | – | – | 1,837 / – | 7,055 / – |
| BN254 scalar | **16,834** / – | 35,997 / – | – | 325,515 / – | – | 403,685 / – |
| secp256k1 base | **16,751** / – | 35,964 / – | 37,370 / – | 275,356 / – | – | 538,663 / – |
| P-256 base | **16,376** / – | 36,083 / – | 123,557 / – | 218,697 / – | – | 320,724 / – |
| P-256 scalar | **16,986** / – | 35,927 / – | 348,095 / – | 329,795 / – | – | 454,729 / – |
| Tom-256 base | **16,889** / – | 35,871 / – | – | 286,153 / – | – | 378,788 / – |
| BLS12-381 base | **32,855** / – | 65,893 / – | – | 1,202,142 / – | – | 1,329,055 / – |
| P-521 base | **63,515** / – | 122,580 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **341** / – | 3,728 / – | – | – | 1,747 / – |
| BabyBear | **7,849** / – | 38,424 / – | – | – | – |
| Goldilocks | **15,824** / – | 54,915 / – | – | – | – |
| BN254 scalar | **298,605** / – | 583,732 / – | – | 521,248 / – | – |
| secp256k1 base | **68,083** / – | 244,994 / – | 37,006\* / – | 278,949 / – | 534,545 / – |
| P-256 base | **79,517** / – | 192,719 / – | 121,574 / – | 212,390 / – | 241,212 / – |
| P-256 scalar | **120,380** / – | 240,940 / – | 364,463 / – | 273,774 / – | – |
| Tom-256 base | **94,922** / – | 223,204 / – | – | 289,606 / – | 375,453 / – |
| BLS12-381 base | **422,644** / – | 617,309 / – | – | 1,206,737 / – | 1,323,194 / – |
| P-521 base | **504,081** / – | 1,565,745 / – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final normalization; the result is not canonical.

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.6** / **14.0** | 69.6 / 71.6 | – | – | – |
| BabyBear | **20.1** / **17.1** | 86.1 / 90.3 | – | – | – |
| Goldilocks | **21.1** / **25.5** | 92.3 / 92.1 | – | – | 32.7 / 32.1 |
| BN254 scalar | **125** / **132** | 1,760 / 1,688 | – | 1,488 / 1,469 | – |
| secp256k1 base | **80.0** / **85.1** | 1,754 / 1,740 | 167 / 175 | 1,045 / 1,048 | – |
| P-256 base | **101** / **103** | 1,768 / 1,727 | 916 / 890 | 1,068 / 1,037 | – |
| P-256 scalar | **122** / **127** | 1,762 / 1,735 | 818 / 814 | 1,446 / 1,474 | – |
| Tom-256 base | **122** / **122** | 1,761 / 1,717 | – | 1,246 / 1,284 | – |
| BLS12-381 base | **190** / **174** | 2,493 / 2,507 | – | 3,571 / 3,511 | – |
| P-521 base | **138** / **171** | 3,977 / 3,994 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | 72.3 / 72.9 | – | – | – |
| BabyBear | **20.1** / **17.0** | 85.5 / 84.6 | – | – | – |
| Goldilocks | **21.1** / **25.5** | 94.1 / 88.6 | – | – | 32.7 / 32.1 |
| BN254 scalar | **597** / **594** | 1,771 / 1,732 | – | 1,347 / 1,342 | – |
| secp256k1 base | **380** / **379** | 1,783 / 1,731 | – | 971 / 969 | – |
| P-256 base | **492** / **489** | 1,781 / 1,738 | – | 994 / 949 | – |
| P-256 scalar | **562** / **544** | 1,773 / 1,746 | 886 / 880 | 1,445 / 1,380 | – |
| Tom-256 base | **562** / **546** | 1,780 / 1,743 | – | 1,183 / 1,175 | – |
| BLS12-381 base | **1,694** / **1,690** | 2,501 / 2,477 | – | 3,545 / 3,520 | – |
| P-521 base | **2,923** / **2,905** | 3,978 / 3,996 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 40.2 / 42.0 | – | – | – |
| BabyBear | **13.0** / **12.7** | 52.4 / 52.4 | – | – | – |
| Goldilocks | **11.1** / **10.8** | 58.2 / 61.1 | – | – | 19.1 / 19.1 |
| BN254 scalar | **163** / **148** | 965 / 936 | – | 797 / 792 | – |
| secp256k1 base | **20.2** / **25.2** | 962 / 937 | 16.0\* / 5.4\* | 517 / 496 | – |
| P-256 base | **104** / **95.8** | 966 / 938 | 491 / 501 | 597 / 600 | – |
| P-256 scalar | 100 / 97.2 | 968 / 941 | **54.6** / **63.0** | 855 / 829 | – |
| Tom-256 base | **132** / **133** | 974 / 945 | – | 737 / 705 | – |
| BLS12-381 base | **236** / **236** | 1,384 / 1,370 | – | 1,681 / 1,691 | – |
| P-521 base | **74.7** / **68.0** | 2,182 / 2,209 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 40.2 / 43.4 | – | – | – |
| BabyBear | **13.0** / **12.7** | 52.4 / 52.3 | – | – | – |
| Goldilocks | **11.1** / **10.8** | 57.1 / 61.0 | – | – | 19.1 / 19.1 |
| BN254 scalar | **429** / **369** | 979 / 939 | – | 675 / 681 | – |
| secp256k1 base | **37.5** / **36.4** | 977 / 957 | – | 425 / 431 | – |
| P-256 base | **300** / **299** | 977 / 957 | – | 513 / 512 | – |
| P-256 scalar | 364 / 347 | 983 / 950 | **120** / **110** | 717 / 716 | – |
| Tom-256 base | **365** / **364** | 975 / 952 | – | 622 / 624 | – |
| BLS12-381 base | **1,051** / **1,025** | 1,386 / 1,388 | – | 1,579 / 1,587 | – |
| P-521 base | **142** / **139** | 2,189 / 2,210 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.5** | 30.7 / 30.3 | – | – | – |
| BabyBear | **15.8** / **13.0** | 42.2 / 42.3 | – | – | – |
| Goldilocks | **10.4** / **10.7** | 46.0 / 48.1 | – | – | 19.0 / 18.1 |
| BN254 scalar | **155** / **153** | 950 / 977 | – | 770 / 755 | – |
| secp256k1 base | **14.6** / **17.0** | 958 / 949 | 14.0\* / 11.0\* | 500 / 486 | – |
| P-256 base | **96.3** / **88.4** | 931 / 950 | 488 / 489 | 584 / 587 | – |
| P-256 scalar | 93.7 / 92.6 | 949 / 969 | **24.5** / **24.4** | 841 / 813 | – |
| Tom-256 base | **124** / **125** | 935 / 970 | – | 716 / 690 | – |
| BLS12-381 base | **242** / **229** | 1,358 / 1,365 | – | 1,651 / 1,677 | – |
| P-521 base | **52.3** / **49.8** | 2,159 / 2,185 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | 30.7 / 30.4 | – | – | – |
| BabyBear | **15.8** / **13.3** | 42.2 / 42.3 | – | – | – |
| Goldilocks | **10.4** / **10.8** | 47.4 / 47.4 | – | – | 19.0 / 18.1 |
| BN254 scalar | **412** / **372** | 956 / 974 | – | 654 / 662 | – |
| secp256k1 base | **34.4** / **31.6** | 940 / 953 | – | 411 / 412 | – |
| P-256 base | **302** / **291** | 928 / 941 | – | 499 / 504 | – |
| P-256 scalar | 364 / 343 | 956 / 963 | **87.6** / **95.0** | 692 / 689 | – |
| Tom-256 base | **343** / **362** | 932 / 963 | – | 607 / 616 | – |
| BLS12-381 base | **1,034** / **1,029** | 1,355 / 1,361 | – | 1,587 / 1,562 | – |
| P-521 base | **127** / **119** | 2,168 / 2,198 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / – | 23.9 / – | – | – | – |
| BabyBear | **14.0** / – | 35.1 / – | – | – | – |
| Goldilocks | 17.4 / – | 39.2 / – | – | – | **17.0** / – |
| BN254 scalar | **270** / – | 881 / – | – | 723 / – | – |
| secp256k1 base | **3.4** / – | 891 / – | 5.9 / – | 448 / – | – |
| P-256 base | **211** / – | 887 / – | 465 / – | 554 / – | – |
| P-256 scalar | 268 / – | 886 / – | **4.0** / – | 741 / – | – |
| Tom-256 base | **265** / – | 874 / – | – | 658 / – | – |
| BLS12-381 base | **605** / – | 1,256 / – | – | 1,647 / – | – |
| P-521 base | **6.0** / – | 1,964 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | 17,229 / **17,184** | **17,218** / 17,364 | 37,822 / 37,709 | – | 330,910 / 329,717 | 405,064 / 406,058 |
| secp256k1 base | 17,026 / **16,679** | **16,776** / 16,942 | 37,822 / 37,551 | 37,487 / 37,435 | 282,771 / 285,691 | 537,096 / 537,534 |
| P-256 base | **17,317** / 17,350 | 17,346 / **17,292** | 37,757 / 37,689 | 126,830 / 126,017 | 220,740 / 217,931 | 332,060 / 330,315 |
| P-256 scalar | **17,336** / 17,251 | 17,387 / **17,225** | 37,769 / 37,668 | 348,210 / 348,269 | 330,836 / 329,295 | 454,945 / 455,824 |
| Tom-256 base | 16,891 / 16,988 | **16,883** / **16,925** | 37,679 / 37,566 | – | 289,112 / 288,194 | 379,735 / 379,744 |
| BLS12-381 base | **34,605** / **34,568** | 34,721 / 34,640 | 67,015 / 66,876 | – | 1,187,878 / 1,177,421 | 1,329,048 / 1,327,893 |
| P-521 base | 65,412 / **64,664** | **64,838** / 65,765 | 128,218 / 127,864 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **460** / **459** | 464 / 464 | 6,894 / 6,888 | – | – | – | 3,416 / 3,419 |
| BabyBear | **611** / **607** | 615 / 612 | 6,980 / 6,887 | – | – | – | 3,419 / 3,419 |
| Goldilocks | 1,581 / **1,576** | **1,574** / 1,579 | 6,946 / 6,883 | – | – | 1,894 / 1,892 | 7,112 / 7,114 |
| BN254 scalar | **17,677** / 17,629 | 17,835 / **17,542** | 37,929 / 37,726 | – | 331,640 / 331,036 | – | 404,477 / 404,849 |
| secp256k1 base | **16,863** / **16,858** | 17,029 / 16,881 | 37,610 / 37,631 | 37,595 / 37,525 | 293,323 / 283,036 | – | 538,652 / 539,007 |
| P-256 base | **17,544** / **17,440** | 17,613 / 17,661 | 37,709 / 37,633 | 126,719 / 127,328 | 218,431 / 217,334 | – | 331,130 / 331,544 |
| P-256 scalar | **17,527** / **17,609** | 17,668 / 17,737 | 37,836 / 37,578 | 347,472 / 348,172 | 328,076 / 330,011 | – | 454,675 / 453,792 |
| Tom-256 base | **16,923** / **16,861** | 17,373 / 17,326 | 37,794 / 37,633 | – | 291,104 / 288,248 | – | 378,961 / 378,497 |
| BLS12-381 base | **35,347** / **35,267** | 35,598 / 35,472 | 66,970 / 66,603 | – | 1,176,943 / 1,173,365 | – | 1,325,631 / 1,325,816 |
| P-521 base | **65,267** / **65,297** | 65,469 / 65,307 | 128,603 / 128,164 | – | – | – | – |

## Results in WebAssembly with wide arithmetic, Firefox

### add: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | – | 11.4 / 20.7 | – | – | 4.4\* / 7.7 | – | 10.9 / 17.8 | – |
| BabyBear | 3.0 / **3.0** | – | 11.4 / 19.4 | – | – | **2.7** / **3.0** | – | 10.9 / 17.8 | – |
| Goldilocks | **3.2** / **3.0** | – | 10.9 / 18.7 | – | – | 6.7 / 8.0 | 4.0 / 4.4 | 10.9 / 17.8 | – |
| BN254 scalar | 19.0 / **17.4** | **18.5** / **17.4** | 60.1 / 92.5 | – | 35.0 / 34.0 | – | – | 76.6 / 115 | – |
| secp256k1 base | 20.5 / 18.1 | **19.9** / **17.9** | 60.1 / 92.7 | 16.0\* / 15.7\* | 33.0 / 31.0 | – | – | 76.5 / 115 | – |
| P-256 base | **20.0** / **18.4** | 20.5 / 18.8 | 60.1 / 89.3 | 31.5 / 29.4 | 31.5 / 30.0 | – | – | 76.5 / 119 | 535 / – |
| P-256 scalar | 21.4 / 18.9 | **19.5** / **17.9** | 60.1 / 92.8 | 56.4 / 60.4 | 35.0 / 33.0 | – | – | 76.5 / 118 | 534 / – |
| Tom-256 base | **20.5** / **19.4** | – | 60.1 / 93.0 | – | 34.0 / 32.5 | – | – | 76.5 / 116 | – |
| BLS12-381 base | **29.4** / **29.8** | 41.8 / 44.9 | 85.5 / 136 | – | 54.0 / 54.0 | – | – | 136 / 165 | – |
| P-521 base | **44.6** / **47.5** | – | 175 / 243 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 17.0 / 14.5 | – |
| BabyBear | **2.7** / **3.0** | – | 5.0 / 8.0 | – | – | 3.0 / **3.0** | – | 16.9 / 14.1 | – |
| Goldilocks | **3.0** / **3.0** | – | 5.0 / 8.0 | – | – | 4.0 / 5.0 | **3.0** / **3.0** | 18.7 / 12.9 | – |
| BN254 scalar | **19.6** / **18.4** | 22.8 / 21.4 | 37.0 / 30.6 | – | 24.0 / 22.0 | – | – | 54.9 / 75.2 | – |
| secp256k1 base | **19.9** / **17.8** | 24.5 / 21.9 | 37.0 / 29.3 | 15.8\* / 17.6\* | 22.0 / 21.0 | – | – | 54.1 / 75.1 | – |
| P-256 base | **18.8** / **17.4** | 22.9 / 21.4 | 37.1 / 28.2 | 22.0 / 20.0 | 21.5 / 20.0 | – | – | 53.8 / 75.1 | 274 / – |
| P-256 scalar | **19.9** / **18.4** | 23.4 / 21.9 | 37.0 / 30.1 | 26.4 / 27.0 | 23.0 / 21.5 | – | – | 54.2 / 75.3 | 268 / – |
| Tom-256 base | **19.6** / **18.3** | – | 37.1 / 30.5 | – | 24.0 / 22.0 | – | – | 54.7 / 75.6 | – |
| BLS12-381 base | **29.2** / **30.4** | 38.8 / 36.9 | 57.3 / 105 | – | 37.0 / 36.1 | – | – | 69.7 / 102 | – |
| P-521 base | **44.3** / **45.6** | – | 128 / 171 | – | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | – | 19.2 / 30.8 | – | – | 5.6 / 12.0 | – | 49.8 / 57.0 | – |
| BabyBear | **5.0** / **11.0** | – | 19.2 / 30.8 | – | – | 6.5 / 13.0 | – | 49.9 / 57.0 | – |
| Goldilocks | 9.0 / **14.0** | – | 19.3 / 32.7 | – | – | 13.7 / 19.0 | **7.7** / **14.0** | 49.8 / 57.0 | – |
| BN254 scalar | **162** / **151** | 276 / 276 | 204 / 211 | – | 182 / 178 | – | – | 1,066 / 1,073 | – |
| secp256k1 base | **108** / **109** | 259 / 257 | 204 / 211 | 175 / 175 | 163 / 160 | – | – | 1,073 / 1,078 | – |
| P-256 base | **122** / **118** | 235 / 235 | 203 / 212 | 135 / 131 | 135 / 132 | – | – | 798 / 777 | 1,273 / – |
| P-256 scalar | **172** / **164** | 293 / 295 | 204 / 212 | 371 / 335 | 181 / 179 | – | – | 1,067 / 1,075 | 1,281 / – |
| Tom-256 base | **149** / **160** | – | 204 / 211 | – | 176 / 171 | – | – | 1,071 / 1,080 | – |
| BLS12-381 base | **421** / 430 | 815 / 786 | 512 / 534 | – | 473 / **376** | – | – | 2,173 / 2,180 | – |
| P-521 base | **846** / **865** | – | 1,054 / 1,104 | – | – | – | – | – | – |

### square: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | – | 18.0 / 30.1 | – | – | 5.5 / 12.0\* | – | 49.0 / 56.0 | – |
| BabyBear | 5.8 / **11.0** | – | 18.2 / 30.7 | – | – | **5.7** / 13.0 | – | 49.0 / 56.0 | – |
| Goldilocks | 9.7 / 14.4 | – | 18.8 / 30.6 | – | – | 11.0 / 19.0 | **7.0** / **14.3** | 49.0 / 56.0 | – |
| BN254 scalar | 160 / **141** | 235 / 233 | 172 / 176 | – | **144** / 144 | – | – | 1,057 / 1,062 | – |
| secp256k1 base | **99.5** / **82.9** | 217 / 214 | 175 / 176 | 115 / 136 | 121 / 122 | – | – | 1,070 / 1,072 | – |
| P-256 base | 116 / **104** | 189 / 188 | 175 / 176 | 128 / 131 | **107** / 107 | – | – | 763 / 693 | 1,292 / – |
| P-256 scalar | 144 / 150 | 259 / 259 | 175 / 176 | 335 / 337 | **140** / **140** | – | – | 1,069 / 1,069 | 1,298 / – |
| Tom-256 base | 158 / 142 | – | 175 / 177 | – | **134** / **136** | – | – | 1,070 / 1,073 | – |
| BLS12-381 base | 449 / 441 | 632 / 555 | 519 / 531 | – | **286** / **296** | – | – | 2,172 / 2,158 | – |
| P-521 base | **419** / **415** | – | 1,123 / 1,201 | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

### invert: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **270** / – | – | 1,966 / – | – | – | 729 / – | – | 1,990 / – | – |
| BabyBear | **277** / – | – | 1,889 / – | – | – | 718 / – | – | 2,084 / – | – |
| Goldilocks | **571** / – | – | 3,806 / – | – | – | 1,526 / – | 998 / – | 4,031 / – | – |
| BN254 scalar | **7,879** / – | 26,505 / – | 19,414 / – | – | 14,337 / – | – | – | 51,712 / – | – |
| secp256k1 base | **7,960** / – | 25,438 / – | 19,039 / – | 37,465 / – | 14,385 / – | – | – | 52,384 / – | – |
| P-256 base | **7,747** / – | 26,251 / – | 19,161 / – | 34,425 / – | 14,263 / – | – | – | 51,719 / – | 272,936 / – |
| P-256 scalar | **8,223** / – | 27,212 / – | 19,219 / – | 63,289 / – | 14,303 / – | – | – | 52,572 / – | 267,215 / – |
| Tom-256 base | **7,943** / – | – | 19,234 / – | – | 14,362 / – | – | – | 52,612 / – | – |
| BLS12-381 base | **15,490** / – | 52,701 / – | 33,923 / – | – | 23,613 / – | – | – | 123,971 / – | – |
| P-521 base | **29,337** / – | – | 62,401 / – | – | – | – | – | – | – |

### sqrt: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 330 / – | – | 2,163 / – | – | – | **329** / – | 1,679 / – | – |
| BabyBear | **2,167** / – | – | 23,175 / – | – | – | 3,000 / – | – | – |
| Goldilocks | **9,179** / – | – | 32,998 / – | – | – | 6,215\* / – | – | – |
| BN254 scalar | **81,894** / – | 140,915 / – | 203,433 / – | – | 179,561 / – | – | – | – |
| secp256k1 base | **30,979** / – | 121,854 / – | 79,732 / – | 37,132 / – | 87,682 / – | – | 531,665 / – | – |
| P-256 base | **27,589** / – | 56,280 / – | 67,732 / – | 34,036 / – | 70,166 / – | – | 247,873 / – | 370,455 / – |
| P-256 scalar | **64,107** / – | 129,219 / – | 79,070 / – | 154,418 / – | 72,365 / – | – | – | – |
| Tom-256 base | **55,442** / – | – | 74,698 / – | – | 93,462 / – | – | 374,513 / – | – |
| BLS12-381 base | **212,164** / – | 431,088 / – | 272,471 / – | – | 370,734 / – | – | 1,320,972 / – | – |
| P-521 base | **268,222** / – | – | 591,360 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_u64: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | – | 38.7 / 38.8 | – | – | 9.5\* / 12.8\* | – | – |
| BabyBear | **14.7** / **13.1** | – | 40.4 / 40.8 | – | – | 17.7 / 16.6 | – | – |
| Goldilocks | 17.1 / 17.1 | – | 50.8 / 49.8 | – | – | 13.7\* / 19.0 | **13.5** / **15.2** | – |
| BN254 scalar | **44.1** / **45.1** | 679 / 648 | 406 / 410 | – | 514 / 504 | – | – | 2,159 / 2,152 |
| secp256k1 base | **23.0** / **23.0** | 480 / 427 | 408 / 409 | 167 / 175 | 437 / 430 | – | – | 2,153 / 2,148 |
| P-256 base | **42.1** / **41.8** | 584 / 564 | 406 / 409 | 259 / 258 | 452 / 432 | – | – | 1,363 / 1,408 |
| P-256 scalar | **48.0** / **47.4** | 720 / 701 | 407 / 411 | 395 / 340 | 442 / 511 | – | – | 2,188 / 2,187 |
| Tom-256 base | **44.3** / **45.3** | – | 407 / 410 | – | 518 / 481 | – | – | 2,161 / 2,158 |
| BLS12-381 base | **61.5** / **62.1** | 1,659 / 1,644 | 1,034 / 1,038 | – | 1,176 / 1,186 | – | – | 4,425 / 4,397 |
| P-521 base | **49.4** / **52.7** | – | 1,825 / 1,844 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | 38.9 / 38.7 | – | – | 9.6\* / 12.8\* | – |
| BabyBear | **14.7** / **13.1** | 40.5 / 40.8 | – | – | 17.7 / 16.6 | – |
| Goldilocks | 17.1 / 17.1 | 49.8 / 50.0 | – | – | 13.7\* / 19.0 | **13.6** / **14.9** |
| BN254 scalar | **253** / **251** | 409 / 408 | – | 349 / 348 | – | – |
| secp256k1 base | **164** / **163** | 408 / 407 | – | 265 / 266 | – | – |
| P-256 base | **236** / **237** | 408 / 408 | – | 252 / 252 | – | – |
| P-256 scalar | **263** / **263** | 408 / 407 | 406 / 406 | 347 / 346 | – | – |
| Tom-256 base | **254** / **255** | 408 / 409 | – | 335 / 335 | – | – |
| BLS12-381 base | **552** / **553** | 940 / 1,035 | – | 1,092 / 1,064 | – | – |
| P-521 base | **986** / **990** | 1,899 / 1,861 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | – | 30.7 / 30.1 | – | – | 7.8\* / 7.1\* | – | – |
| BabyBear | **12.5** / **11.0** | – | 31.8 / 32.8 | – | – | 15.0 / 13.1 | – | – |
| Goldilocks | 12.0 / 10.0 | – | 36.4 / 33.6 | – | – | 6.7\* / 8.1\* | **10.2** / **9.3** | – |
| BN254 scalar | **86.7** / **65.2** | 330 / 325 | 311 / 310 | – | 323 / 299 | – | – | 1,212 / 1,210 |
| secp256k1 base | **11.5** / **10.0** | 184 / 184 | 314 / 310 | 15.2 / 5.4\* | 270 / 246 | – | – | 1,187 / 1,179 |
| P-256 base | **67.5** / **57.4** | 284 / 271 | 311 / 310 | 159 / 158 | 274 / 250 | – | – | 988 / 990 |
| P-256 scalar | 83.3 / 65.7 | 362 / 333 | 312 / 310 | **54.7** / **63.0** | 323 / 299 | – | – | 1,229 / 1,224 |
| Tom-256 base | **71.1** / **64.8** | – | 313 / 309 | – | 318 / 318 | – | – | 1,214 / 1,208 |
| BLS12-381 base | **96.5** / **90.7** | 843 / 838 | 649 / 647 | – | 632 / 641 | – | – | 2,408 / 2,381 |
| P-521 base | **25.1** / **26.8** | – | 1,127 / 1,141 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.3** | 30.4 / 30.5 | – | – | 7.8\* / 7.1\* | – |
| BabyBear | **12.5** / **11.0** | 31.8 / 31.9 | – | – | 15.0 / 13.1 | – |
| Goldilocks | 12.0 / 10.0 | 36.5 / 33.6 | – | – | 6.7\* / 8.1\* | **10.2** / **9.6** |
| BN254 scalar | 212 / **200** | 314 / 313 | – | **205** / 205 | – | – |
| secp256k1 base | **24.4** / **23.2** | 315 / 311 | – | 140 / 140 | – | – |
| P-256 base | 165 / 161 | 314 / 311 | – | **153** / **153** | – | – |
| P-256 scalar | 226 / 213 | 314 / 312 | **117** / **107** | 204 / 204 | – | – |
| Tom-256 base | 212 / 204 | 314 / 313 | – | **196** / **196** | – | – |
| BLS12-381 base | 521 / 505 | 652 / 649 | – | **424** / **425** | – | – |
| P-521 base | **72.7** / **67.0** | 1,193 / 1,174 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | – | 20.9 / 20.2 | – | – | 6.3\* / 6.1\* | – | – |
| BabyBear | **12.5** / **10.9** | – | 21.8 / 21.4 | – | – | 15.1 / 13.3 | – | – |
| Goldilocks | 11.0 / 9.7 | – | 27.5 / 24.0 | – | – | 4.3\* / 5.0\* | **9.5** / **8.7** | – |
| BN254 scalar | **73.6** / **66.6** | 338 / 325 | 257 / 285 | – | 317 / 290 | – | – | 1,174 / 1,163 |
| secp256k1 base | **10.0** / **9.5** | 193 / 182 | 258 / 286 | 14.0 / 11.0 | 263 / 239 | – | – | 1,150 / 1,139 |
| P-256 base | **80.9** / **57.1** | 291 / 277 | 228 / 286 | 149 / 149 | 266 / 243 | – | – | 933 / 905 |
| P-256 scalar | 84.0 / 65.4 | 382 / 342 | 256 / 293 | **24.2** / **24.7** | 315 / 288 | – | – | 1,187 / 1,191 |
| Tom-256 base | **72.2** / **65.5** | – | 256 / 284 | – | 311 / 286 | – | – | 1,174 / 1,164 |
| BLS12-381 base | **96.3** / **91.0** | 886 / 835 | 617 / 624 | – | 612 / 601 | – | – | 2,342 / 2,317 |
| P-521 base | **21.1** / **21.7** | – | 1,139 / 1,106 | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | 20.9 / 20.1 | – | – | 6.3\* / 6.0\* | – |
| BabyBear | **12.5** / **10.9** | 21.8 / 21.9 | – | – | 15.1 / 13.3 | – |
| Goldilocks | 11.0 / 9.7 | 28.4 / 24.0 | – | – | 4.3\* / 5.0\* | **9.5** / **8.7** |
| BN254 scalar | 216 / 204 | 257 / 282 | – | **194** / **193** | – | – |
| secp256k1 base | **23.5** / **22.0** | 290 / 281 | – | 130 / 130 | – | – |
| P-256 base | 165 / 163 | 229 / 282 | – | **144** / **143** | – | – |
| P-256 scalar | 227 / 211 | 258 / 285 | **89.5** / **95.5** | 193 / 194 | – | – |
| Tom-256 base | 217 / 208 | 258 / 282 | – | **187** / **188** | – | – |
| BLS12-381 base | 524 / 505 | 618 / 623 | – | **407** / **408** | – | – |
| P-521 base | **69.1** / **67.4** | 1,164 / 1,123 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 5.2 / – | – | 13.2 / – | – | – | **3.9** / – | – | – |
| BabyBear | **10.3** / – | – | 15.0 / – | – | – | 11.8 / – | – | – |
| Goldilocks | 8.7 / – | – | 17.0 / – | – | – | 2.1\* / – | **7.0** / – | – |
| BN254 scalar | **46.0** / – | 299 / – | 204 / – | – | 266 / – | – | – | 1,080 / – |
| secp256k1 base | **3.8** / – | 181 / – | 204 / – | 6.0 / – | 215 / – | – | – | 1,079 / – |
| P-256 base | **39.0** / – | 254 / – | 203 / – | 131 / – | 219 / – | – | – | 820 / – |
| P-256 scalar | 43.0 / – | 315 / – | 204 / – | **4.0** / – | 266 / – | – | – | 1,077 / – |
| Tom-256 base | **43.4** / – | – | 204 / – | – | 260 / – | – | – | 1,076 / – |
| BLS12-381 base | **64.0** / – | 778 / – | 516 / – | – | 584 / – | – | – | 2,230 / – |
| P-521 base | **6.0** / – | – | 889 / – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **982** / **988** | 7,292 / 7,213 | 27,240 / 27,178 | 17,740 / 17,743 | – | 11,753 / 11,726 | 43,639 / 43,830 |
| secp256k1 base | **914** / **874** | 7,071 / 7,012 | 21,642 / 21,544 | 17,722 / 17,675 | 37,512 / 37,432 | 11,588 / 11,513 | 44,227 / 44,606 |
| P-256 base | **948** / **944** | 6,796 / 6,783 | 26,242 / 26,240 | 17,751 / 17,681 | 34,651 / 34,686 | 11,565 / 11,506 | 43,255 / 42,960 |
| P-256 scalar | **1,004** / **994** | 7,331 / 7,373 | 27,032 / 27,122 | 17,726 / 17,657 | 48,462 / 48,263 | 11,679 / 11,689 | 44,290 / 43,969 |
| Tom-256 base | **972** / **966** | 7,227 / 7,189 | – | 17,728 / 17,727 | – | 11,645 / 11,657 | 43,366 / 44,059 |
| BLS12-381 base | **1,307** / **1,303** | 14,206 / 14,157 | 53,203 / 53,118 | 31,415 / 31,539 | – | 18,616 / 18,530 | 102,215 / 101,682 |
| P-521 base | **1,930** / **1,883** | 26,291 / 26,372 | – | 57,887 / 58,233 | – | – | – |

### div_uint: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **281** / **283** | 286 / **283** | – | 1,976 / 1,983 | – | – | 744 / 702 | – | 2,096 / 2,099 |
| BabyBear | **291** / **287** | 303 / 297 | – | 1,965 / 1,958 | – | – | 716 / 723 | – | 2,197 / 2,195 |
| Goldilocks | **586** / **588** | 614 / 609 | – | 3,868 / 3,863 | – | – | 1,562 / 1,564 | 1,047 / 1,028 | 4,163 / 4,151 |
| BN254 scalar | **8,365** / 8,483 | 8,422 / **8,475** | 27,093 / 27,046 | 19,187 / 19,207 | – | 14,515 / 14,477 | – | – | 54,257 / 54,102 |
| secp256k1 base | **8,118** / **8,098** | 8,121 / 8,100 | 25,670 / 26,010 | 19,216 / 19,250 | 37,611 / 37,532 | 14,549 / 14,389 | – | – | 54,504 / 54,609 |
| P-256 base | **7,901** / **7,872** | 7,983 / 7,927 | 26,725 / 26,781 | 19,231 / 19,239 | 34,698 / 34,729 | 14,404 / 14,319 | – | – | 53,883 / 53,835 |
| P-256 scalar | 8,667 / **8,639** | **8,579** / 8,653 | 27,990 / 28,138 | 19,281 / 19,258 | 64,300 / 63,942 | 14,686 / 14,648 | – | – | 54,719 / 54,484 |
| Tom-256 base | 8,491 / **8,453** | **8,485** / 8,454 | – | 19,281 / 19,256 | – | 14,559 / 14,520 | – | – | 54,579 / 54,682 |
| BLS12-381 base | 16,831 / **16,842** | **16,804** / 16,933 | 53,287 / 53,743 | 34,629 / 34,575 | – | 24,437 / 24,361 | – | – | 129,069 / 128,887 |
| P-521 base | 30,683 / 30,667 | **30,601** / **30,323** | – | 63,807 / 63,655 | – | – | – | – | – |

### add, constant time: a + b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.4** / **7.7** | 11.4 / 20.7 | – | – | – | 10.9 / 17.8 |
| BabyBear | **3.0** / **3.0** | 11.4 / 19.4 | – | – | – | 10.9 / 17.8 |
| Goldilocks | **3.2** / **3.0** | 10.9 / 18.7 | – | – | 4.0 / 4.4 | 10.9 / 17.8 |
| BN254 scalar | **26.5** / **25.7** | 60.1 / 92.5 | – | 35.0 / 34.0 | – | 76.6 / 115 |
| secp256k1 base | **29.0** / **29.8** | 60.1 / 92.7 | 16.0\* / 15.7\* | 33.0 / 31.0 | – | 76.5 / 115 |
| P-256 base | **28.0** / **28.0** | 60.1 / 89.3 | 31.5 / 29.4 | 31.5 / 30.0 | – | 76.5 / 119 |
| P-256 scalar | **29.5** / **30.1** | 60.1 / 92.8 | 56.4 / 60.4 | 35.0 / 33.0 | – | 76.5 / 118 |
| Tom-256 base | **28.5** / **29.0** | 60.1 / 93.0 | – | 34.0 / 32.5 | – | 76.5 / 116 |
| BLS12-381 base | **41.1** / **41.0** | 85.5 / 136 | – | 54.0 / 54.0 | – | 136 / 165 |
| P-521 base | **74.0** / **74.0** | 175 / 243 | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **3.0** | 5.0 / 8.0 | – | – | 3.0 / 5.0 | – | 17.0 / 14.5 |
| BabyBear | **2.7** / **3.0** | 5.0 / 8.0 | – | – | – | – | 16.9 / 14.1 |
| Goldilocks | **3.0** / **3.0** | 5.0 / 8.0 | – | – | – | **3.0** / **3.0** | 18.7 / 12.9 |
| BN254 scalar | 26.0 / 25.0 | 37.0 / 30.6 | – | **24.0** / **22.0** | – | – | 54.9 / 75.2 |
| secp256k1 base | 22.5 / 21.5 | 37.0 / 29.3 | 15.8\* / 17.6\* | **22.0** / **21.0** | – | – | 54.1 / 75.1 |
| P-256 base | 22.0 / **20.0** | 37.1 / 28.2 | 22.0 / **20.0** | **21.5** / **20.0** | – | – | 53.8 / 75.1 |
| P-256 scalar | 24.0 / **21.5** | 37.0 / 30.1 | 26.4 / 27.0 | **23.0** / **21.5** | – | – | 54.2 / 75.3 |
| Tom-256 base | **24.0** / 22.5 | 37.1 / 30.5 | – | **24.0** / **22.0** | – | – | 54.7 / 75.6 |
| BLS12-381 base | **37.0** / 36.2 | 57.3 / 105 | – | **37.0** / **36.1** | – | – | 69.7 / 102 |
| P-521 base | **53.0** / **53.0** | 128 / 171 | – | – | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **12.0** | 19.2 / 30.8 | – | – | – | 49.8 / 57.0 |
| BabyBear | **5.0** / **11.0** | 19.2 / 30.8 | – | – | – | 49.9 / 57.0 |
| Goldilocks | 9.0 / **14.0** | 19.3 / 32.7 | – | – | **7.7** / **14.0** | 49.8 / 57.0 |
| BN254 scalar | **145** / **149** | 204 / 211 | – | 182 / 178 | – | 1,066 / 1,073 |
| secp256k1 base | **109** / **108** | 204 / 211 | 175 / 175 | 163 / 160 | – | 1,073 / 1,078 |
| P-256 base | **120** / **120** | 203 / 212 | 135 / 131 | 135 / 132 | – | 798 / 777 |
| P-256 scalar | **156** / **164** | 204 / 212 | 371 / 335 | 181 / 179 | – | 1,067 / 1,075 |
| Tom-256 base | **156** / **149** | 204 / 211 | – | 176 / 171 | – | 1,071 / 1,080 |
| BLS12-381 base | **421** / 429 | 512 / 534 | – | 473 / **376** | – | 2,173 / 2,180 |
| P-521 base | **847** / **866** | 1,054 / 1,104 | – | – | – | – |

### square, constant time: a², in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.4** / **12.0** | 18.0 / 30.1 | – | – | – | 49.0 / 56.0 |
| BabyBear | **5.8** / **11.0** | 18.2 / 30.7 | – | – | – | 49.0 / 56.0 |
| Goldilocks | 9.7 / 14.4 | 18.8 / 30.6 | – | – | **7.0** / **14.3** | 49.0 / 56.0 |
| BN254 scalar | **140** / 147 | 172 / 176 | – | 144 / **144** | – | 1,057 / 1,062 |
| secp256k1 base | **102** / **95.7** | 175 / 176 | 115 / 136 | 121 / 122 | – | 1,070 / 1,072 |
| P-256 base | 113 / **104** | 175 / 176 | 128 / 131 | **107** / 107 | – | 763 / 693 |
| P-256 scalar | 155 / 150 | 175 / 176 | 335 / 337 | **140** / **140** | – | 1,069 / 1,069 |
| Tom-256 base | 158 / 142 | 175 / 177 | – | **134** / **136** | – | 1,070 / 1,073 |
| BLS12-381 base | 448 / 441 | 519 / 531 | – | **286** / **296** | – | 2,172 / 2,158 |
| P-521 base | **420** / **418** | 1,123 / 1,201 | – | – | – | – |

### invert, constant time: a⁻¹, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **428** / – | 6,441 / – | – | – | – | 3,332 / – |
| BabyBear | **417** / – | 6,455 / – | – | – | – | 3,342 / – |
| Goldilocks | 1,194 / – | 6,463 / – | – | – | **998** / – | 7,060 / – |
| BN254 scalar | **12,527** / – | 30,794 / – | – | 99,877 / – | – | 403,140 / – |
| secp256k1 base | **12,594** / – | 30,446 / – | 37,465 / – | 109,486 / – | – | 537,289 / – |
| P-256 base | **12,309** / – | 30,452 / – | 34,425 / – | 76,459 / – | – | 316,661 / – |
| P-256 scalar | **12,736** / – | 30,685 / – | 147,110 / – | 98,594 / – | – | 452,861 / – |
| Tom-256 base | **12,702** / – | 30,692 / – | – | 96,809 / – | – | 377,185 / – |
| BLS12-381 base | **24,079** / – | 53,633 / – | – | 379,094 / – | – | 1,324,724 / – |
| P-521 base | **41,255** / – | 101,412 / – | – | – | – | – |

### sqrt, constant time: √a, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **340** / – | 2,163 / – | – | – | 1,679 / – |
| BabyBear | **6,489** / – | 23,175 / – | – | – | – |
| Goldilocks | **21,641** / – | 32,998 / – | – | – | – |
| BN254 scalar | **152,636** / – | 203,433 / – | – | 179,561 / – | – |
| secp256k1 base | **31,139** / – | 79,732 / – | 37,132 / – | 87,682 / – | 531,665 / – |
| P-256 base | **32,304** / – | 67,732 / – | 34,036 / – | 70,166 / – | 247,873 / – |
| P-256 scalar | **68,158** / – | 79,070 / – | 154,418 / – | 72,365 / – | – |
| Tom-256 base | **52,551** / – | 74,698 / – | – | 93,462 / – | 374,513 / – |
| BLS12-381 base | **211,927** / – | 272,471 / – | – | 370,734 / – | 1,320,972 / – |
| P-521 base | **272,366** / – | 591,360 / – | – | – | – |

### mul_u64, constant time: a × k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | 38.7 / 38.8 | – | – | – |
| BabyBear | **14.7** / **13.1** | 40.4 / 40.8 | – | – | – |
| Goldilocks | 17.1 / 17.1 | 50.8 / 49.8 | – | – | **13.5** / **15.2** |
| BN254 scalar | **57.4** / **58.0** | 406 / 410 | – | 514 / 504 | – |
| secp256k1 base | **37.2** / **45.5** | 408 / 409 | 167 / 175 | 437 / 430 | – |
| P-256 base | **53.7** / **59.4** | 406 / 409 | 259 / 258 | 452 / 432 | – |
| P-256 scalar | **62.0** / **67.3** | 407 / 411 | 395 / 340 | 442 / 511 | – |
| Tom-256 base | **57.6** / **64.0** | 407 / 410 | – | 518 / 481 | – |
| BLS12-381 base | **85.9** / **88.0** | 1,034 / 1,038 | – | 1,176 / 1,186 | – |
| P-521 base | **95.7** / **95.0** | 1,825 / 1,844 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.7** / **14.0** | 38.9 / 38.7 | – | – | – |
| BabyBear | **14.7** / **13.1** | 40.5 / 40.8 | – | – | – |
| Goldilocks | 17.1 / 17.1 | 49.8 / 50.0 | – | – | **13.6** / **14.9** |
| BN254 scalar | **312** / **311** | 409 / 408 | – | 349 / 348 | – |
| secp256k1 base | **251** / **252** | 408 / 407 | – | 265 / 266 | – |
| P-256 base | 301 / 306 | 408 / 408 | – | **252** / **252** | – |
| P-256 scalar | **336** / **333** | 408 / 407 | 406 / 406 | 347 / 346 | – |
| Tom-256 base | **325** / **324** | 408 / 409 | – | 335 / 335 | – |
| BLS12-381 base | **689** / **684** | 940 / 1,035 | – | 1,092 / 1,064 | – |
| P-521 base | **1,496** / **1,470** | 1,899 / 1,861 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.3** | 30.7 / 30.1 | – | – | – |
| BabyBear | **12.5** / **11.0** | 31.8 / 32.8 | – | – | – |
| Goldilocks | 12.0 / 10.0 | 36.4 / 33.6 | – | – | **10.2** / **9.3** |
| BN254 scalar | **81.4** / **81.6** | 311 / 310 | – | 323 / 299 | – |
| secp256k1 base | **25.1** / **29.0** | 314 / 310 | 15.2\* / 5.4\* | 270 / 246 | – |
| P-256 base | **71.5** / **74.3** | 311 / 310 | 159 / 158 | 274 / 250 | – |
| P-256 scalar | 77.1 / 79.7 | 312 / 310 | **54.7** / **63.0** | 323 / 299 | – |
| Tom-256 base | **85.2** / **84.9** | 313 / 309 | – | 318 / 318 | – |
| BLS12-381 base | **127** / **122** | 649 / 647 | – | 632 / 641 | – |
| P-521 base | **64.1** / **64.7** | 1,127 / 1,141 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.2** / **9.3** | 30.4 / 30.5 | – | – | – |
| BabyBear | **12.5** / **11.0** | 31.8 / 31.9 | – | – | – |
| Goldilocks | 12.0 / 10.0 | 36.5 / 33.6 | – | – | **10.2** / **9.6** |
| BN254 scalar | 216 / **205** | 314 / 313 | – | **205** / **205** | – |
| secp256k1 base | **44.4** / **44.5** | 315 / 311 | – | 140 / 140 | – |
| P-256 base | 168 / 160 | 314 / 311 | – | **153** / **153** | – |
| P-256 scalar | 192 / 213 | 314 / 312 | **117** / **107** | 204 / 204 | – |
| Tom-256 base | 205 / 207 | 314 / 313 | – | **196** / **196** | – |
| BLS12-381 base | 551 / 544 | 652 / 649 | – | **424** / **425** | – |
| P-521 base | **138** / **134** | 1,193 / 1,174 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **6.6** | 20.9 / 20.2 | – | – | – |
| BabyBear | **12.5** / **10.9** | 21.8 / 21.4 | – | – | – |
| Goldilocks | 11.0 / 9.4 | 27.5 / 24.0 | – | – | **9.5** / **8.7** |
| BN254 scalar | **80.1** / **79.9** | 257 / 285 | – | 317 / 290 | – |
| secp256k1 base | **18.5** / **17.4** | 258 / 286 | 14.0\* / 11.0\* | 263 / 239 | – |
| P-256 base | **68.0** / **69.6** | 228 / 286 | 149 / 149 | 266 / 243 | – |
| P-256 scalar | 77.5 / 77.2 | 256 / 293 | **24.2** / **24.7** | 315 / 288 | – |
| Tom-256 base | **80.4** / **80.1** | 256 / 284 | – | 311 / 286 | – |
| BLS12-381 base | **118** / **119** | 617 / 624 | – | 612 / 601 | – |
| P-521 base | **44.2** / **46.2** | 1,139 / 1,106 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **6.5** | 20.9 / 20.1 | – | – | – |
| BabyBear | **12.5** / **10.8** | 21.8 / 21.9 | – | – | – |
| Goldilocks | 11.0 / 9.4 | 28.4 / 24.0 | – | – | **9.5** / **8.7** |
| BN254 scalar | 218 / 219 | 257 / 282 | – | **194** / **193** | – |
| secp256k1 base | **38.1** / **37.7** | 290 / 281 | – | 130 / 130 | – |
| P-256 base | 163 / 158 | 229 / 282 | – | **144** / **143** | – |
| P-256 scalar | 208 / 207 | 258 / 285 | **89.5** / **95.5** | 193 / 194 | – |
| Tom-256 base | 202 / 193 | 258 / 282 | – | **187** / **188** | – |
| BLS12-381 base | 547 / 541 | 618 / 623 | – | **407** / **408** | – |
| P-521 base | **114** / **112** | 1,164 / 1,123 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.2** / – | 13.2 / – | – | – | – |
| BabyBear | **12.7** / – | 15.0 / – | – | – | – |
| Goldilocks | 12.5 / – | 17.0 / – | – | – | **7.0** / – |
| BN254 scalar | **119** / – | 204 / – | – | 266 / – | – |
| secp256k1 base | **3.4** / – | 204 / – | 6.0 / – | 215 / – | – |
| P-256 base | **84.0** / – | 203 / – | 131 / – | 219 / – | – |
| P-256 scalar | 117 / – | 204 / – | **4.0** / – | 266 / – | – |
| Tom-256 base | **113** / – | 204 / – | – | 260 / – | – |
| BLS12-381 base | **273** / – | 516 / – | – | 584 / – | – |
| P-521 base | **6.0** / – | 889 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | 12,948 / **12,896** | **12,915** / 12,952 | 31,291 / 31,382 | – | 110,969 / 110,896 | 405,988 / 405,334 |
| secp256k1 base | **12,853** / 12,937 | 12,930 / **12,794** | 31,287 / 31,349 | 37,512 / 37,432 | 93,052 / 93,056 | 537,603 / 537,115 |
| P-256 base | **12,501** / **12,483** | 12,513 / 12,486 | 31,256 / 31,375 | 34,651 / 34,686 | 96,968 / 98,259 | 325,721 / 325,293 |
| P-256 scalar | **13,041** / **13,117** | 13,211 / 13,189 | 30,891 / 31,237 | 142,605 / 142,686 | 114,513 / 114,407 | 456,526 / 456,316 |
| Tom-256 base | **12,990** / **12,925** | 13,052 / 12,957 | 31,209 / 31,377 | – | 110,718 / 110,570 | 379,800 / 379,429 |
| BLS12-381 base | **24,119** / **24,124** | 24,204 / 24,125 | 55,120 / 54,655 | – | 400,957 / 400,667 | 1,329,828 / 1,327,806 |
| P-521 base | 43,437 / **43,215** | **43,373** / 43,328 | 101,291 / 101,185 | – | – | – |

### div_uint, constant time: a / k, k of one bit less than p, in WebAssembly with wide arithmetic, Firefox

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 465 / 461 | **464** / **459** | 6,520 / 6,439 | – | – | – | 3,417 / 3,419 |
| BabyBear | **459** / **450** | 462 / **450** | 6,539 / 6,433 | – | – | – | 3,417 / 3,419 |
| Goldilocks | 1,212 / 1,209 | 1,222 / 1,214 | 6,510 / 6,453 | – | – | **1,047** / **1,028** | 7,111 / 7,115 |
| BN254 scalar | **13,063** / **13,112** | 13,295 / 13,119 | 31,379 / 31,324 | – | 110,359 / 110,733 | – | 405,031 / 404,545 |
| secp256k1 base | **12,854** / 12,900 | 12,942 / **12,850** | 31,295 / 31,440 | 37,611 / 37,532 | 92,822 / 92,811 | – | 539,965 / 538,481 |
| P-256 base | **12,643** / **12,584** | 12,729 / 12,683 | 31,191 / 31,370 | 34,698 / 34,729 | 97,750 / 97,361 | – | 326,167 / 328,102 |
| P-256 scalar | **13,515** / **13,334** | 13,642 / 13,431 | 31,291 / 31,428 | 142,627 / 142,613 | 114,660 / 114,613 | – | 455,698 / 455,326 |
| Tom-256 base | **13,094** / **13,134** | 13,227 / 13,170 | 31,268 / 31,376 | – | 110,662 / 110,544 | – | 379,618 / 378,997 |
| BLS12-381 base | **24,508** / **24,562** | 24,662 / 24,651 | 54,952 / 54,691 | – | 400,596 / 400,562 | – | 1,328,282 / 1,326,774 |
| P-521 base | **43,462** / **43,287** | 43,793 / 43,563 | 101,264 / 101,383 | – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
