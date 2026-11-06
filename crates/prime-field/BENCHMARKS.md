# Benchmarks

`prime-field` at commit `a0a67b8` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-05.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 20.58 before, 19.94 after; its SMT sibling, CPU 29, 27% busy |
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
| Mersenne-31 | **1.9** / **2.9** | – | 9.6 / 17.9 | – | – | 2.0 / 3.0 | – | 2.5 / 5.0 | – |
| BabyBear | **1.9** / **3.0** | – | 9.6 / 17.7 | – | – | 2.0 / **3.0** | – | 2.6 / 5.0 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 18.9 | – | – | 1.9\* / 2.9\* | 2.4 / **2.9** | 2.5 / 5.0 | – |
| BN254 scalar | **4.9** / **6.7** | 16.9 / 11.7 | 66.4 / 87.1 | – | 13.0 / 21.8 | – | – | 7.0 / 10.6 | – |
| secp256k1 base | **5.8** / **6.9** | 17.3 / 13.0 | 66.3 / 89.9 | 4.1\* / 3.0\* | 14.1 / 21.3 | – | – | 7.0 / 11.0 | – |
| P-256 base | **6.0** / **6.9** | 18.2 / 12.9 | 66.3 / 84.2 | 10.8 / 18.7 | 10.9 / 18.6 | – | – | 7.0 / 10.5 | 323 / – |
| P-256 scalar | **5.8** / **6.9** | 17.2 / 13.3 | 66.3 / 87.9 | 17.5 / 21.0 | 14.6 / 21.0 | – | – | 7.0 / 10.9 | 328 / – |
| Tom-256 base | **6.0** / **6.9** | – | 66.5 / 86.2 | – | 11.5 / 20.5 | – | – | 7.0 / 11.5 | – |
| BLS12-381 base | **9.6** / **8.9** | 20.3 / 13.8 | 87.2 / 112 | – | 20.2 / 27.3 | – | – | 16.9 / 12.9 | – |
| P-521 base | **12.3** / **14.7** | – | 116 / 154 | – | – | – | – | 24.8 / 53.0 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | **1.9** / 4.0 | – |
| BabyBear | **2.0** / **2.9** | – | 4.2 / 11.9 | – | – | **2.0** / 3.0 | – | **2.0** / 4.0 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.3 / 11.7 | – | – | 1.9 / 2.9 | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **4.9** / **6.7** | 16.0 / 12.0 | 49.5 / 50.8 | – | 8.3 / 11.8 | – | – | 6.9 / 8.0 | – |
| secp256k1 base | **5.4** / **8.0** | 16.6 / 13.1 | 49.8 / 48.4 | 4.7\* / 3.7\* | 7.5 / 13.0 | – | – | 7.0 / **8.0** | – |
| P-256 base | **5.5** / **7.0** | 16.2 / 11.7 | 49.6 / 43.6 | 8.0 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 | 148 / – |
| P-256 scalar | **5.0** / **6.9** | 15.7 / 12.5 | 49.6 / 47.7 | 7.9 / 13.0 | 8.1 / 13.0 | – | – | 7.0 / 8.0 | 149 / – |
| Tom-256 base | **5.5** / **8.0** | – | 49.8 / 55.0 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **9.6** / **9.9** | 20.6 / 15.7 | 82.9 / 87.8 | – | 15.2 / 19.9 | – | – | 16.9 / 11.1 | – |
| P-521 base | **11.9** / **12.4** | – | 121 / 139 | – | – | – | – | 27.5 / 50.4 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.8 / 35.1 | – | – | 2.8 / 7.0\* | – | 7.0 / 17.0 | – |
| BabyBear | **3.4** / **10.9** | – | 16.8 / 34.7 | – | – | 4.7 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | 3.8 / **8.9** | – | 17.6 / 34.6 | – | – | 3.2\* / 9.9 | **3.7** / 10.9 | 7.0 / 17.0 | – |
| BN254 scalar | **68.8** / **72.5** | 74.1 / 104 | 118 / 152 | – | 91.4 / 93.8 | – | – | 94.2 / 126 | – |
| secp256k1 base | **44.5** / **54.8** | 88.3 / 114 | 118 / 152 | 86.8 / 88.4 | 88.1 / 96.0 | – | – | 94.2 / 128 | – |
| P-256 base | **49.5** / **52.5** | 65.0 / 86.5 | 118 / 152 | 68.1 / 72.1 | 66.8 / 73.0 | – | – | 50.7 / 55.5 | 572 / – |
| P-256 scalar | **74.8** / **84.0** | 90.8 / 113 | 118 / 152 | 140 / 141 | 93.9 / 100.0 | – | – | 94.3 / 128 | 649 / – |
| Tom-256 base | **70.9** / **76.0** | – | 119 / 152 | – | 89.8 / 94.8 | – | – | 94.3 / 128 | – |
| BLS12-381 base | **138** / **141** | 162 / 194 | 270 / 288 | – | 187 / 194 | – | – | 255 / 262 | – |
| P-521 base | **176** / **191** | – | 528 / 546 | – | – | – | – | 465 / 474 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 33.9 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.7 / 33.9 | – | – | 4.7 / 13.0 | – | 7.6 / 17.0 | – |
| Goldilocks | **3.7** / **9.0** | – | 17.5 / 35.2 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.5 / 17.0 | – |
| BN254 scalar | **63.1** / **66.7** | 73.4 / 105 | 115 / 152 | – | 74.2 / 89.4 | – | – | 93.8 / 125 | – |
| secp256k1 base | **33.8** / **42.9** | 83.9 / 101 | 112 / 151 | 57.9 / 80.0 | 70.4 / 84.9 | – | – | 93.9 / 126 | – |
| P-256 base | **40.0** / **44.0** | 63.9 / 84.7 | 118 / 150 | 60.4 / 72.0 | 54.7 / 58.0 | – | – | 50.5 / 57.9 | 572 / – |
| P-256 scalar | **63.9** / **72.9** | 87.1 / 108 | 112 / 152 | 129 / 147 | 77.1 / 89.2 | – | – | 93.9 / 128 | 637 / – |
| Tom-256 base | **57.0** / **64.5** | – | 114 / 151 | – | 69.5 / 82.9 | – | – | 93.7 / 129 | – |
| BLS12-381 base | **131** / **137** | 140 / 167 | 268 / 289 | – | 151 / 166 | – | – | 254 / 255 | – |
| P-521 base | **117** / **129** | – | 536 / 563 | – | – | – | – | 463 / 463 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **214** / – | – | 1,427 / – | – | – | 636 / – | – | 880 / – | – |
| BabyBear | **213** / – | – | 1,397 / – | – | – | 622 / – | – | 1,047 / – | – |
| Goldilocks | **437** / – | – | 2,868 / – | – | – | 1,295 / – | 691 / – | 1,701 / – | – |
| BN254 scalar | **5,057** / – | 16,468 / – | 14,297 / – | – | 7,144 / – | – | – | 14,887 / – | – |
| secp256k1 base | **4,922** / – | 15,636 / – | 14,380 / – | 24,899 / – | 7,207 / – | – | – | 14,918 / – | – |
| P-256 base | **4,837** / – | 15,276 / – | 14,307 / – | 29,613 / – | 7,198 / – | – | – | 14,569 / – | 88,652 / – |
| P-256 scalar | **5,041** / – | 16,466 / – | 14,343 / – | 29,645 / – | 7,201 / – | – | – | 15,012 / – | 88,817 / – |
| Tom-256 base | **5,018** / – | – | 14,311 / – | – | 7,198 / – | – | – | 14,966 / – | – |
| BLS12-381 base | **8,808** / – | 36,666 / – | 25,072 / – | – | 12,084 / – | – | – | 34,939 / – | – |
| P-521 base | **14,227** / – | – | 44,164 / – | – | – | – | – | 71,220 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **135** / – | – | 1,646 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,971** / – | – | 18,509 / – | – | – | 2,739 / – | – | – |
| Goldilocks | **2,833** / – | – | 26,585 / – | – | – | 3,154 / – | – | – |
| BN254 scalar | **33,810** / – | 44,181 / – | 133,683 / – | – | 84,211 / – | – | – | – |
| secp256k1 base | **12,304** / – | 42,716 / – | 52,062 / – | 26,170 / – | 47,445 / – | – | 68,906 / – | – |
| P-256 base | **11,796** / – | 18,774 / – | 43,178 / – | 28,136 / – | 33,629 / – | – | 29,408 / – | 164,166 / – |
| P-256 scalar | **24,127** / – | 39,359 / – | 51,683 / – | 82,133 / – | 42,065 / – | – | – | – |
| Tom-256 base | **21,264** / – | – | 48,654 / – | – | 44,138 / – | – | 51,397 / – | – |
| BLS12-381 base | **65,940** / – | 89,696 / – | 133,942 / – | – | 139,909 / – | – | 176,512 / – | – |
| P-521 base | **68,261** / – | – | 266,107 / – | – | – | – | 257,106 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.8** / **8.9** | – | 34.5 / 36.5 | – | – | 5.0\* / 6.9\* | – | – |
| BabyBear | **8.9** / **10.9** | – | 34.3 / 36.5 | – | – | 11.9 / 15.0 | – | – |
| Goldilocks | **6.9** / **9.1** | – | 38.0 / 38.6 | – | – | 3.2\* / 9.9 | 7.2 / 10.9 | – |
| BN254 scalar | **20.7** / **25.9** | 236 / 235 | 266 / 266 | – | 236 / 234 | – | – | 252 / 250 |
| secp256k1 base | **9.7** / **14.9** | 233 / 233 | 264 / 262 | 81.8 / 90.1 | 208 / 211 | – | – | 251 / 250 |
| P-256 base | **18.9** / **24.9** | 206 / 210 | 268 / 267 | 148 / 136 | 182 / 183 | – | – | 131 / 133 |
| P-256 scalar | **22.3** / **27.9** | 260 / 259 | 260 / 265 | 139 / 146 | 248 / 242 | – | – | 253 / 250 |
| Tom-256 base | **20.5** / **27.9** | – | 274 / 265 | – | 223 / 230 | – | – | 252 / 251 |
| BLS12-381 base | **28.8** / **34.5** | 423 / 426 | 543 / 552 | – | 429 / 430 | – | – | 571 / 577 |
| P-521 base | **19.7** / **32.7** | – | 903 / 918 | – | – | – | – | 960 / 969 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.6 / 36.5 | – | – | 5.4\* / 6.9\* | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | 11.9 / 13.9 | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 37.5 | – | – | 3.1\* / 9.9 | 7.3 / 10.9 |
| BN254 scalar | **103** / **114** | 277 / 268 | – | 185 / 185 | – | – |
| secp256k1 base | **56.9** / **67.9** | 276 / 275 | – | 157 / 155 | – | – |
| P-256 base | **102** / **110** | 281 / 264 | – | 128 / 134 | – | – |
| P-256 scalar | **110** / **116** | 280 / 270 | 181 / 186 | 191 / 188 | – | – |
| Tom-256 base | **108** / **114** | 279 / 280 | – | 170 / 173 | – | – |
| BLS12-381 base | **211** / **222** | 549 / 549 | – | 372 / 374 | – | – |
| P-521 base | **350** / **406** | 909 / 919 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | – | 31.6 / 26.5 | – | – | 4.1\* / 4.0\* | – | – |
| BabyBear | **5.9** / **7.0** | – | 32.0 / 30.5 | – | – | 8.9 / 7.9 | – | – |
| Goldilocks | **4.8** / **4.7** | – | 38.0 / 34.9 | – | – | 1.9\* / 2.9\* | 6.1 / 5.9 | – |
| BN254 scalar | **24.8** / **24.9** | 128 / 124 | 231 / 221 | – | 141 / 150 | – | – | 114 / 113 |
| secp256k1 base | **5.8** / **7.4** | 109 / 104 | 232 / 227 | 3.6\* / 1.9\* | 129 / 137 | – | – | 114 / 113 |
| P-256 base | **25.7** / **22.1** | 107 / 103 | 231 / 215 | 76.5 / 85.1 | 119 / 129 | – | – | 59.0 / 63.0 |
| P-256 scalar | 25.4 / 22.7 | 134 / 129 | 231 / 223 | **14.1** / **20.7** | 152 / 152 | – | – | 115 / 113 |
| Tom-256 base | **26.1** / **26.0** | – | 231 / 230 | – | 141 / 149 | – | – | 115 / 113 |
| BLS12-381 base | **38.2** / **37.5** | 224 / 224 | 374 / 375 | – | 262 / 260 | – | – | 307 / 308 |
| P-521 base | **10.7** / **12.7** | – | 593 / 595 | – | – | – | – | 530 / 538 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | 33.4 / 26.6 | – | – | 4.1\* / 4.0\* | – |
| BabyBear | **5.9** / **7.0** | 32.4 / 30.4 | – | – | 8.9 / 7.9 | – |
| Goldilocks | **4.7** / **4.7** | 38.9 / 35.0 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 |
| BN254 scalar | **72.0** / **73.1** | 233 / 232 | – | 101 / 108 | – | – |
| secp256k1 base | **6.8** / **8.5** | 231 / 232 | – | 83.8 / 94.8 | – | – |
| P-256 base | **57.3** / **57.8** | 232 / 230 | – | 75.7 / 81.5 | – | – |
| P-256 scalar | 81.0 / 78.1 | 232 / 233 | **48.2** / **43.9** | 108 / 116 | – | – |
| Tom-256 base | **73.8** / **75.2** | 233 / 230 | – | 92.7 / 107 | – | – |
| BLS12-381 base | **152** / **151** | 375 / 380 | – | 206 / 213 | – | – |
| P-521 base | **23.2** / **23.6** | 596 / 603 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | – | 16.5 / 17.9 | – | – | 4.2\* / 4.1\* | – | – |
| BabyBear | **5.9** / **7.0** | – | 17.5 / 18.4 | – | – | 8.9 / 8.9 | – | – |
| Goldilocks | **4.3** / **4.2** | – | 26.8 / 22.2 | – | – | 1.9\* / 2.9\* | 4.8 / 5.6 | – |
| BN254 scalar | **24.8** / **24.9** | 129 / 120 | 179 / 183 | – | 145 / 143 | – | – | 112 / 111 |
| secp256k1 base | **4.2** / **6.9** | 109 / 103 | 189 / 177 | 3.8\* / 2.8\* | 123 / 124 | – | – | 112 / 111 |
| P-256 base | **19.8** / **21.7** | 106 / 95.9 | 178 / 178 | 80.8 / 78.8 | 120 / 117 | – | – | 61.3 / 61.1 |
| P-256 scalar | 24.9 / 23.7 | 132 / 127 | 186 / 176 | **6.5** / **12.4** | 149 / 150 | – | – | 112 / 111 |
| Tom-256 base | **24.6** / **25.0** | – | 183 / 182 | – | 142 / 143 | – | – | 113 / 111 |
| BLS12-381 base | **37.8** / **38.6** | 222 / 226 | 361 / 361 | – | 254 / 255 | – | – | 317 / 311 |
| P-521 base | **9.6** / **11.0** | – | 586 / 595 | – | – | – | – | 534 / 545 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 17.9 | – | – | 3.9\* / 4.1\* | – |
| BabyBear | **5.8** / **7.0** | 17.6 / 18.4 | – | – | 8.9 / 8.9 | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 1.9\* / 2.9\* | 4.9 / 5.3 |
| BN254 scalar | **74.6** / **73.1** | 185 / 185 | – | 111 / 105 | – | – |
| secp256k1 base | **5.4** / **6.8** | 180 / 180 | – | 80.8 / 82.8 | – | – |
| P-256 base | **53.1** / **56.8** | 178 / 179 | – | 78.5 / 77.9 | – | – |
| P-256 scalar | 79.9 / 77.7 | 187 / 178 | **43.3** / **36.8** | 106 / 102 | – | – |
| Tom-256 base | **76.0** / **72.6** | 181 / 186 | – | 95.3 / 96.0 | – | – |
| BLS12-381 base | **155** / **155** | 363 / 366 | – | 208 / 203 | – | – |
| P-521 base | **19.9** / **15.5** | 583 / 605 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / – | – | 12.3 / – | – | – | **3.2** / – | – | – |
| BabyBear | **6.3** / – | – | 13.1 / – | – | – | 8.0 / – | – | – |
| Goldilocks | 3.4 / – | – | 17.5 / – | – | – | 1.0\* / – | **3.2** / – | – |
| BN254 scalar | **22.0** / – | 137 / – | 118 / – | – | 127 / – | – | – | 95.0 / – |
| secp256k1 base | **2.0** / – | 127 / – | 118 / – | 2.2 / – | 108 / – | – | – | 94.8 / – |
| P-256 base | **17.6** / – | 120 / – | 117 / – | 58.9 / – | 106 / – | – | – | 54.6 / – |
| P-256 scalar | 18.0 / – | 148 / – | 117 / – | **2.2** / – | 134 / – | – | – | 94.9 / – |
| Tom-256 base | **21.0** / – | – | 114 / – | – | 124 / – | – | – | 94.8 / – |
| BLS12-381 base | **29.1** / – | 226 / – | 256 / – | – | 227 / – | – | – | 336 / – |
| P-521 base | **5.0** / – | – | 442 / – | – | – | – | – | 483 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **612** / **604** | 4,165 / 4,127 | 16,439 / 16,522 | 13,316 / 13,274 | – | 6,195 / 6,181 | 10,573 / 10,593 |
| secp256k1 base | **564** / **566** | 4,054 / 4,043 | 13,257 / 13,201 | 13,360 / 13,224 | 24,973 / 24,780 | 6,150 / 6,134 | 10,608 / 10,603 |
| P-256 base | **602** / **603** | 4,162 / 4,150 | 14,810 / 14,850 | 13,310 / 13,269 | 29,578 / 30,296 | 6,134 / 6,129 | 10,495 / 10,468 |
| P-256 scalar | **619** / **619** | 4,089 / 4,083 | 16,058 / 15,962 | 13,290 / 13,264 | 21,911 / 21,946 | 6,188 / 6,181 | 10,608 / 10,607 |
| Tom-256 base | **608** / **609** | 4,164 / 4,185 | – | 13,355 / 13,240 | – | 6,172 / 6,172 | 10,605 / 10,599 |
| BLS12-381 base | **751** / **751** | 7,109 / 7,088 | 34,537 / 34,799 | 23,256 / 23,107 | – | 9,838 / 9,832 | 26,012 / 25,663 |
| P-521 base | **925** / **924** | 11,954 / 11,977 | – | 39,605 / 39,638 | – | – | 55,022 / 54,824 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **215** / **217** | 216 / **217** | – | 1,461 / 1,436 | – | – | 636 / 636 | – | 889 / 892 |
| BabyBear | **215** / **217** | 233 / 233 | – | 1,435 / 1,437 | – | – | 628 / 621 | – | 1,065 / 1,066 |
| Goldilocks | **435** / **435** | 445 / 445 | – | 2,900 / 2,899 | – | – | 1,295 / 1,291 | 704 / 706 | 1,723 / 1,720 |
| BN254 scalar | 5,014 / **4,982** | **5,012** / 5,027 | 16,800 / 16,664 | 14,605 / 14,544 | – | 7,372 / 7,354 | – | – | 14,262 / 14,359 |
| secp256k1 base | 4,901 / **4,882** | **4,888** / 4,885 | 15,651 / 15,642 | 14,656 / 14,557 | 27,316 / 25,162 | 7,402 / 7,369 | – | – | 14,465 / 14,411 |
| P-256 base | **5,029** / **5,018** | 5,072 / 5,041 | 15,447 / 15,487 | 14,628 / 14,620 | 29,121 / 28,820 | 7,349 / 7,363 | – | – | 14,224 / 14,237 |
| P-256 scalar | **4,958** / **4,943** | 4,978 / 4,962 | 16,721 / 16,681 | 14,650 / 14,598 | 29,909 / 29,861 | 7,391 / 7,404 | – | – | 14,465 / 14,483 |
| Tom-256 base | **5,057** / **5,051** | 5,099 / 5,071 | – | 14,693 / 14,604 | – | 7,412 / 7,414 | – | – | 14,500 / 14,406 |
| BLS12-381 base | **9,020** / **9,014** | 9,169 / 9,141 | 36,515 / 36,897 | 26,059 / 25,945 | – | 12,373 / 12,347 | – | – | 35,263 / 34,839 |
| P-521 base | **14,921** / 14,980 | 14,948 / **14,942** | – | 45,212 / 44,726 | – | – | – | – | 72,147 / 71,864 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 9.6 / 17.9 | – | – | – | 2.5 / 5.0 |
| BabyBear | **1.9** / **2.9** | 9.6 / 17.7 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 18.9 | – | – | 2.4 / **2.9** | 2.5 / 5.0 |
| BN254 scalar | **4.9** / **6.7** | 66.4 / 87.1 | – | 13.0 / 21.8 | – | 7.0 / 10.6 |
| secp256k1 base | **5.8** / **6.9** | 66.3 / 89.9 | 4.1\* / 3.0\* | 14.1 / 21.3 | – | 7.0 / 11.0 |
| P-256 base | **6.0** / **7.0** | 66.3 / 84.2 | 10.8 / 18.7 | 10.9 / 18.6 | – | 7.0 / 10.5 |
| P-256 scalar | **5.8** / **6.8** | 66.3 / 87.9 | 17.5 / 21.0 | 14.6 / 21.0 | – | 7.0 / 10.9 |
| Tom-256 base | **6.0** / **7.0** | 66.5 / 86.2 | – | 11.5 / 20.5 | – | 7.0 / 11.5 |
| BLS12-381 base | **9.6** / **8.9** | 87.2 / 112 | – | 20.2 / 27.3 | – | 16.9 / 12.9 |
| P-521 base | **12.2** / **15.2** | 116 / 154 | – | – | – | 24.8 / 53.0 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | **1.9** / 4.0 |
| BabyBear | **1.9** / **3.0** | 4.2 / 11.9 | – | – | – | – | 2.0 / 4.0 |
| Goldilocks | **1.9** / **2.9** | 4.3 / 11.7 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.0** / **6.8** | 49.5 / 50.8 | – | 8.3 / 11.8 | – | – | 6.9 / 8.0 |
| secp256k1 base | **5.3** / **8.0** | 49.8 / 48.4 | 4.7\* / 3.7\* | 7.5 / 13.0 | – | – | 7.0 / **8.0** |
| P-256 base | **5.3** / **6.7** | 49.6 / 43.6 | 8.0 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.0** / **6.8** | 49.6 / 47.7 | 7.9 / 13.0 | 8.1 / 13.0 | – | – | 7.0 / 8.0 |
| Tom-256 base | **5.5** / 8.3 | 49.8 / 55.0 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **9.7** / **9.6** | 82.9 / 87.8 | – | 15.2 / 19.9 | – | – | 16.9 / 11.1 |
| P-521 base | **11.7** / **12.3** | 121 / 139 | – | – | – | – | 27.5 / 50.4 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.8 / 35.1 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.4** / **11.0** | 16.8 / 34.7 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **8.9** | 17.6 / 34.6 | – | – | **3.7** / 10.9 | 7.0 / 17.0 |
| BN254 scalar | **67.8** / **78.8** | 118 / 152 | – | 91.4 / 93.8 | – | 94.2 / 126 |
| secp256k1 base | **44.6** / **55.0** | 118 / 152 | 86.8 / 88.4 | 88.1 / 96.0 | – | 94.2 / 128 |
| P-256 base | **49.6** / **52.5** | 118 / 152 | 68.1 / 72.1 | 66.8 / 73.0 | – | 50.7 / 55.5 |
| P-256 scalar | **75.0** / **84.0** | 118 / 152 | 140 / 141 | 93.9 / 100.0 | – | 94.3 / 128 |
| Tom-256 base | **70.0** / **76.0** | 119 / 152 | – | 89.8 / 94.8 | – | 94.3 / 128 |
| BLS12-381 base | **141** / **154** | 270 / 288 | – | 187 / 194 | – | 255 / 262 |
| P-521 base | **176** / **188** | 528 / 546 | – | – | – | 465 / 474 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 33.9 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.7 / 33.9 | – | – | – | 7.6 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 35.2 | – | – | **3.7** / 11.0 | 7.5 / 17.0 |
| BN254 scalar | **64.9** / **66.9** | 115 / 152 | – | 74.2 / 89.4 | – | 93.8 / 125 |
| secp256k1 base | **33.8** / **42.9** | 112 / 151 | 57.9 / 80.0 | 70.4 / 84.9 | – | 93.9 / 126 |
| P-256 base | **40.3** / **44.0** | 118 / 150 | 60.4 / 72.0 | 54.7 / 58.0 | – | 50.5 / 57.9 |
| P-256 scalar | **64.4** / **72.9** | 112 / 152 | 129 / 147 | 77.1 / 89.2 | – | 93.9 / 128 |
| Tom-256 base | **57.0** / **64.5** | 114 / 151 | – | 69.5 / 82.9 | – | 93.7 / 129 |
| BLS12-381 base | **134** / **145** | 268 / 289 | – | 151 / 166 | – | 254 / 255 |
| P-521 base | **118** / **129** | 536 / 563 | – | – | – | 463 / 463 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **243** / – | 3,852 / – | – | – | – | 953 / – |
| BabyBear | **383** / – | 3,852 / – | – | – | – | 944 / – |
| Goldilocks | **648** / – | 3,850 / – | – | – | 691 / – | 2,067 / – |
| BN254 scalar | **6,758** / – | 19,328 / – | – | 46,611 / – | – | 55,420 / – |
| secp256k1 base | **6,860** / – | 19,294 / – | 24,899 / – | 46,580 / – | – | 69,946 / – |
| P-256 base | **6,485** / – | 19,317 / – | 29,613 / – | 33,423 / – | – | 34,703 / – |
| P-256 scalar | **6,909** / – | 19,239 / – | 78,723 / – | 48,919 / – | – | 60,479 / – |
| Tom-256 base | **6,973** / – | 19,266 / – | – | 44,802 / – | – | 52,313 / – |
| BLS12-381 base | **12,456** / – | 33,900 / – | – | 141,950 / – | – | 178,624 / – |
| P-521 base | **20,025** / – | 64,105 / – | – | – | – | 494,523 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,646 / – | – | – | 446 / – |
| BabyBear | **5,592** / – | 18,509 / – | – | – | – |
| Goldilocks | **7,149** / – | 26,585 / – | – | – | – |
| BN254 scalar | **58,556** / – | 133,683 / – | – | 84,211 / – | – |
| secp256k1 base | **12,274** / – | 52,062 / – | 26,170 / – | 47,445 / – | 68,906 / – |
| P-256 base | **12,041** / – | 43,178 / – | 28,136 / – | 33,629 / – | 29,408 / – |
| P-256 scalar | **25,625** / – | 51,683 / – | 82,133 / – | 42,065 / – | – |
| Tom-256 base | **20,048** / – | 48,654 / – | – | 44,138 / – | 51,397 / – |
| BLS12-381 base | **69,784** / – | 133,942 / – | – | 139,909 / – | 176,512 / – |
| P-521 base | **67,949** / – | 266,107 / – | – | – | 257,106 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.5 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.5 | – | – | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 38.6 | – | – | 7.2 / 10.9 |
| BN254 scalar | **21.9** / **30.4** | 266 / 266 | – | 236 / 234 | – |
| secp256k1 base | **11.0** / **19.8** | 264 / 262 | 81.8 / 90.1 | 208 / 211 | – |
| P-256 base | **24.4** / **33.8** | 268 / 267 | 148 / 136 | 182 / 183 | – |
| P-256 scalar | **22.9** / **33.9** | 260 / 265 | 139 / 146 | 248 / 242 | – |
| Tom-256 base | **22.4** / **32.9** | 274 / 265 | – | 223 / 230 | – |
| BLS12-381 base | **34.9** / **41.6** | 543 / 552 | – | 429 / 430 | – |
| P-521 base | **26.1** / **45.7** | 903 / 918 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.8** / **8.9** | 34.6 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | – |
| Goldilocks | **6.9** / **9.0** | 38.0 / 37.5 | – | – | 7.3 / 10.9 |
| BN254 scalar | **119** / **141** | 277 / 268 | – | 185 / 185 | – |
| secp256k1 base | **78.2** / **83.8** | 276 / 275 | – | 157 / 155 | – |
| P-256 base | **118** / **134** | 281 / 264 | – | 128 / **134** | – |
| P-256 scalar | **132** / **146** | 280 / 270 | 181 / 186 | 191 / 188 | – |
| Tom-256 base | **122** / **135** | 279 / 280 | – | 170 / 173 | – |
| BLS12-381 base | **256** / **266** | 549 / 549 | – | 372 / 374 | – |
| P-521 base | **426** / **472** | 909 / 919 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.5** / **4.5** | 31.6 / 26.5 | – | – | – |
| BabyBear | **5.4** / **4.9** | 32.0 / 30.5 | – | – | – |
| Goldilocks | **4.8** / **5.0** | 38.0 / 34.9 | – | – | 6.1 / 5.9 |
| BN254 scalar | **23.5** / **21.9** | 231 / 221 | – | 141 / 150 | – |
| secp256k1 base | **5.8** / **7.4** | 232 / 227 | 3.6\* / 1.9\* | 129 / 137 | – |
| P-256 base | **16.9** / **17.1** | 231 / 215 | 76.5 / 85.1 | 119 / 129 | – |
| P-256 scalar | 23.9 / 24.8 | 231 / 223 | **14.1** / **20.7** | 152 / 152 | – |
| Tom-256 base | **22.3** / **22.3** | 231 / 230 | – | 141 / 149 | – |
| BLS12-381 base | **36.6** / **35.8** | 374 / 375 | – | 262 / 260 | – |
| P-521 base | **10.7** / **12.9** | 593 / 595 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.5** / **4.5** | 33.4 / 26.6 | – | – | – |
| BabyBear | **5.4** / **4.9** | 32.4 / 30.4 | – | – | – |
| Goldilocks | **4.7** / **4.7** | 38.9 / 35.0 | – | – | 6.0 / 5.9 |
| BN254 scalar | **75.8** / **74.8** | 233 / 232 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **8.8** | 231 / 232 | – | 83.8 / 94.8 | – |
| P-256 base | **57.2** / **57.8** | 232 / 230 | – | 75.7 / 81.5 | – |
| P-256 scalar | 83.2 / 78.8 | 232 / 233 | **48.2** / **43.9** | 108 / 116 | – |
| Tom-256 base | **73.7** / **75.1** | 233 / 230 | – | 92.7 / 107 | – |
| BLS12-381 base | **156** / **154** | 375 / 380 | – | 206 / 213 | – |
| P-521 base | **22.3** / **22.6** | 596 / 603 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | 16.5 / 17.9 | – | – | – |
| BabyBear | **5.2** / **4.8** | 17.5 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 4.8 / 5.6 |
| BN254 scalar | **25.4** / **21.9** | 179 / 183 | – | 145 / 143 | – |
| secp256k1 base | **4.2** / **6.9** | 189 / 177 | 3.8\* / 2.8\* | 123 / 124 | – |
| P-256 base | **16.2** / **16.1** | 178 / 178 | 80.8 / 78.8 | 120 / 117 | – |
| P-256 scalar | 24.1 / 22.7 | 186 / 176 | **6.5** / **12.4** | 149 / 150 | – |
| Tom-256 base | **22.9** / **23.9** | 183 / 182 | – | 142 / 143 | – |
| BLS12-381 base | **36.5** / **35.1** | 361 / 361 | – | 254 / 255 | – |
| P-521 base | **9.6** / **11.0** | 586 / 595 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 17.9 | – | – | – |
| BabyBear | **5.2** / **4.8** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.2 | – | – | 4.9 / 5.3 |
| BN254 scalar | **77.7** / **74.1** | 185 / 185 | – | 111 / 105 | – |
| secp256k1 base | **7.2** / **7.8** | 180 / 180 | – | 80.8 / 82.8 | – |
| P-256 base | **53.1** / **56.8** | 178 / 179 | – | 78.5 / 77.9 | – |
| P-256 scalar | 80.0 / 76.9 | 187 / 178 | **43.3** / **36.8** | 106 / 102 | – |
| Tom-256 base | **74.9** / **72.8** | 181 / 186 | – | 95.3 / 96.0 | – |
| BLS12-381 base | **154** / **156** | 363 / 366 | – | 208 / 203 | – |
| P-521 base | **19.0** / **19.5** | 583 / 605 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.8** / – | 12.3 / – | – | – | – |
| BabyBear | **4.9** / – | 13.1 / – | – | – | – |
| Goldilocks | 3.4 / – | 17.5 / – | – | – | **3.2** / – |
| BN254 scalar | **26.2** / – | 118 / – | – | 127 / – | – |
| secp256k1 base | **2.0** / – | 118 / – | 2.2 / – | 108 / – | – |
| P-256 base | **18.6** / – | 117 / – | 58.9 / – | 106 / – | – |
| P-256 scalar | 27.2 / – | 117 / – | **2.2** / – | 134 / – | – |
| Tom-256 base | **23.5** / – | 114 / – | – | 124 / – | – |
| BLS12-381 base | **41.9** / – | 256 / – | – | 227 / – | – |
| P-521 base | **5.0** / – | 442 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **6,851** / **6,843** | 6,890 / 6,887 | 19,605 / 19,586 | – | 46,076 / 46,048 | 55,082 / 54,822 |
| secp256k1 base | **6,776** / **6,780** | 6,832 / 6,824 | 19,616 / 19,607 | 24,973 / 24,780 | 46,768 / 46,765 | 69,507 / 69,480 |
| P-256 base | **6,666** / **6,654** | 6,700 / 6,700 | 19,595 / 19,571 | 29,578 / 30,296 | 33,197 / 33,200 | 34,737 / 34,694 |
| P-256 scalar | 6,956 / 6,895 | **6,940** / **6,879** | 19,606 / 19,551 | 78,935 / 78,431 | 49,369 / 48,823 | 60,184 / 60,057 |
| Tom-256 base | **7,282** / **7,274** | 7,323 / 7,326 | 19,586 / 19,567 | – | 44,199 / 43,792 | 51,762 / 51,621 |
| BLS12-381 base | **12,384** / **12,367** | 12,417 / 12,450 | 34,809 / 34,894 | – | 140,493 / 140,324 | 182,342 / 178,214 |
| P-521 base | **20,382** / **20,375** | 20,420 / 20,412 | 64,797 / 64,848 | – | – | 497,728 / 497,401 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **262** / **250** | 274 / 268 | 3,893 / 3,894 | – | – | – | 993 / 993 |
| BabyBear | **403** / **403** | 417 / 412 | 3,894 / 3,896 | – | – | – | 993 / 993 |
| Goldilocks | 668 / **665** | **666** / 667 | 3,892 / 3,898 | – | – | 704 / 706 | 2,115 / 2,114 |
| BN254 scalar | **6,895** / **6,894** | 6,943 / 6,935 | 19,587 / 19,567 | – | 46,002 / 46,010 | – | 54,989 / 54,784 |
| secp256k1 base | **6,771** / **6,779** | 6,829 / 6,824 | 19,589 / 19,513 | 27,316 / 25,162 | 46,713 / 46,722 | – | 69,289 / 69,431 |
| P-256 base | **6,740** / **6,690** | 6,757 / 6,755 | 19,584 / 19,567 | 29,121 / 28,820 | 33,145 / 33,153 | – | 34,638 / 34,356 |
| P-256 scalar | **6,952** / **6,912** | 6,995 / 7,001 | 19,674 / 19,549 | 78,954 / 78,467 | 48,969 / 49,081 | – | 60,318 / 60,008 |
| Tom-256 base | **7,337** / **7,335** | 7,381 / 7,376 | 19,591 / 19,542 | – | 44,088 / 43,766 | – | 51,769 / 51,572 |
| BLS12-381 base | **12,490** / **12,516** | 12,562 / 12,532 | 34,744 / 35,027 | – | 141,904 / 140,224 | – | 180,912 / 178,557 |
| P-521 base | **20,380** / **20,375** | 20,419 / 20,417 | 64,635 / 64,720 | – | – | – | 498,774 / 496,589 |

## WebAssembly

`prime-field` at commit `c7970be` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-06.

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
| Cost of a count, in WebAssembly, Chrome | 4,207 to 11,107 cycles and 3,081 instructions; its noise in a figure, 45.0 to 4,570 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,216 to 13,839 cycles and 3,081 instructions; its noise in a figure, 87.0 to 3,099 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 6,923 to 27,410 cycles and 1,885 instructions; its noise in a figure, 959 to 5,880 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 8,585 to 20,307 cycles and 1,885 instructions; its noise in a figure, 658 to 1,986 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 18.29 before, 36.08 after; its SMT sibling, CPU 28, 36% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 36.08 before, 41.25 after; its SMT sibling, CPU 28, 52% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 39.38 before, 53.49 after; its SMT sibling, CPU 28, 55% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 55.05 before, 38.19 after; its SMT sibling, CPU 28, 72% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add | BabyBear | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 2.7 vs plonky3 2.6 | +0.1 (1.04×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) | 3.0 vs plonky3 2.6 | +0.4 (1.15×) |
| add | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.4 vs winter-math 3.4 | +0.0 (1.00×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add | BN254 scalar | 13.9 vs ark-ff 13.8 | +0.1 (1.01×) | 12.8 vs ark-ff 14.1 | −1.3 (0.91×) | 14.8 vs ark-ff 14.1 | +0.7 (1.05×) | 18.9 vs ark-ff 18.7 | +0.2 (1.01×) |
| add | secp256k1 base | 14.0 vs ark-ff 15.3 | −1.3 (0.92×) | 11.8 vs ark-ff 12.7 | −0.9 (0.93×) | 14.9 vs ark-ff 16.8 | −1.9 (0.89×) | 20.5 vs ark-ff 20.1 | +0.4 (1.02×) |
| add | P-256 base | 14.4 vs ark-ff 15.6 | −1.2 (0.92×) | 12.2 vs ark-ff 13.3 | −1.1 (0.92×) | 14.5 vs ark-ff 17.3 | −2.8 (0.84×) | 20.3 vs ark-ff 20.5 | −0.2 (0.99×) |
| add | P-256 scalar | 14.5 vs ark-ff 15.4 | −0.9 (0.94×) | 11.8 vs ark-ff 13.4 | −1.6 (0.88×) | 15.7 vs ark-ff 17.3 | −1.6 (0.91×) | 20.6 vs ark-ff 19.6 | +1.0 (1.05×) |
| add | Tom-256 base | 13.6 vs halo2curves 35.9 | −22.3 (0.38×) | 11.8 vs halo2curves 20.4 | −8.6 (0.58×) | 14.3 vs halo2curves 38.7 | −24.4 (0.37×) | 20.3 vs halo2curves 34.1 | −13.8 (0.60×) |
| add | BLS12-381 base | 22.0 vs ark-ff 35.1 | −13.1 (0.63×) | 18.6 vs ark-ff 30.0 | −11.4 (0.62×) | 22.3 vs ark-ff 39.3 | −17.0 (0.57×) | 29.7 vs ark-ff 41.6 | −11.9 (0.71×) |
| add | P-521 base | 42.2 vs crypto-bigint 144 | −102 (0.29×) | 26.4 vs crypto-bigint 138 | −112 (0.19×) | 43.6 vs crypto-bigint 218 | −174 (0.20×) | 44.6 vs crypto-bigint 195 | −150 (0.23×) |
| sub | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub | BabyBear | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.6 vs plonky3 2.8 | −0.2 (0.93×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) |
| sub | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.9 | −0.3 (0.90×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 15.0 vs ark-ff 18.4 | −3.4 (0.82×) | 11.7 vs halo2curves 14.9 | −3.2 (0.79×) | 15.0 vs ark-ff 18.9 | −3.9 (0.79×) | 19.2 vs ark-ff 22.9 | −3.7 (0.84×) |
| sub | secp256k1 base | 16.7 vs ark-ff 18.4 | −1.7 (0.91×) | 11.8 vs halo2curves 14.4 | −2.6 (0.82×) | 15.7 vs ark-ff 19.4 | −3.7 (0.81×) | 18.8 vs halo2curves 22.0 | −3.2 (0.85×) |
| sub | P-256 base | 14.3 vs ark-ff 17.0 | −2.7 (0.84×) | 12.2 vs halo2curves 12.3 | −0.1 (0.99×) | 14.0 vs ark-ff 18.5 | −4.5 (0.76×) | 18.3 vs halo2curves 21.6 | −3.3 (0.85×) |
| sub | P-256 scalar | 16.1 vs ark-ff 17.9 | −1.8 (0.90×) | 11.8 vs halo2curves 14.5 | −2.7 (0.81×) | 15.0 vs ark-ff 18.7 | −3.7 (0.80×) | 18.6 vs halo2curves 23.2 | −4.6 (0.80×) |
| sub | Tom-256 base | 15.0 vs halo2curves 23.9 | −8.9 (0.63×) | 11.6 vs halo2curves 15.1 | −3.5 (0.77×) | 14.3 vs halo2curves 23.5 | −9.2 (0.61×) | 19.2 vs halo2curves 24.2 | −5.0 (0.79×) |
| sub | BLS12-381 base | 22.9 vs ark-ff 33.4 | −10.5 (0.69×) | 17.2 vs halo2curves 24.4 | −7.2 (0.70×) | 21.8 vs ark-ff 31.1 | −9.3 (0.70×) | 29.5 vs halo2curves 37.0 | −7.5 (0.80×) |
| sub | P-521 base | 46.9 vs crypto-bigint 88.4 | −41.5 (0.53×) | 26.6 vs crypto-bigint 64.3 | −37.7 (0.41×) | 42.2 vs crypto-bigint 122 | −79.8 (0.35×) | 43.3 vs crypto-bigint 128 | −84.7 (0.34×) |
| mul | Mersenne-31 | 3.6 vs crypto-bigint 29.3 | −25.7 (0.12×) | 3.6 vs crypto-bigint 15.9 | −12.3 (0.23×) | 4.0 vs crypto-bigint 39.2 | −35.2 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul | BabyBear | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 6.5 | −1.3 (0.80×) | 5.0 vs plonky3 6.5 | −1.5 (0.77×) |
| mul | Goldilocks | 10.0 vs winter-math 12.4 | −2.4 (0.81×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 15.0 vs winter-math 17.1 | −2.1 (0.88×) | 7.0 vs winter-math 7.7 | −0.7 (0.91×) |
| mul | BN254 scalar | 234 vs ark-ff 322 | −88.0 (0.73×) | 88.2 vs halo2curves 91.4 | −3.2 (0.96×) | 385 vs ark-ff 405 | −20.0 (0.95×) | 173 vs halo2curves 182 | −9.0 (0.95×) |
| mul | secp256k1 base | 160 vs ark-ff 280 | −120 (0.57×) | 59.6 vs halo2curves 91.3 | −31.7 (0.65×) | 239 vs ark-ff 410 | −171 (0.58×) | 104 vs halo2curves 164 | −60.0 (0.63×) |
| mul | P-256 base | 179 vs k256/p256 241 | −62.0 (0.74×) | 71.8 vs k256/p256 85.2 | −13.4 (0.84×) | 278 vs ark-ff 443 | −165 (0.63×) | 126 vs halo2curves 135 | −9.0 (0.93×) |
| mul | P-256 scalar | 244 vs ark-ff 321 | −77.0 (0.76×) | 84.2 vs halo2curves 93.7 | −9.5 (0.90×) | 329 vs ark-ff 414 | −85.0 (0.79×) | 170 vs halo2curves 182 | −12.0 (0.93×) |
| mul | Tom-256 base | 234 vs halo2curves 343 | −109 (0.68×) | 80.7 vs halo2curves 94.2 | −13.5 (0.86×) | 318 vs halo2curves 584 | −266 (0.54×) | 178 vs halo2curves 176 | +2.0 (1.01×) |
| mul | BLS12-381 base | 727 vs ark-ff 806 | −79.0 (0.90×) | 188 vs halo2curves 195 | −7.0 (0.96×) | 914 vs ark-ff 1,079 | −165 (0.85×) | 348 vs halo2curves 383 | −35.0 (0.91×) |
| mul | P-521 base | 669 vs crypto-bigint 2,124 | −1,455 (0.31×) | 245 vs crypto-bigint 600 | −355 (0.41×) | 1,288 vs crypto-bigint 2,702 | −1,414 (0.48×) | 532 vs crypto-bigint 1,059 | −527 (0.50×) |
| square | Mersenne-31 | 3.5 vs crypto-bigint 28.6 | −25.1 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 38.0 | −34.4 (0.09×) | 3.7 vs crypto-bigint 17.8 | −14.1 (0.21×) |
| square | BabyBear | 4.7 vs plonky3 5.5 | −0.8 (0.85×) | 4.8 vs plonky3 5.2 | −0.4 (0.92×) | 5.7 vs plonky3 5.9 | −0.2 (0.97×) | 5.4 vs plonky3 5.8 | −0.4 (0.93×) |
| square | Goldilocks | 8.7 vs winter-math 12.4 | −3.7 (0.70×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.0 vs winter-math 7.1 | +0.9 (1.13×) |
| square | BN254 scalar | 214 vs ark-ff 271 | −57.0 (0.79×) | 81.6 vs halo2curves 84.7 | −3.1 (0.96×) | 347 vs ark-ff 361 | −14.0 (0.96×) | 160 vs halo2curves 148 | +12.0 (1.08×) |
| square | secp256k1 base | 142 vs ark-ff 234 | −92.0 (0.61×) | 53.1 vs halo2curves 76.9 | −23.8 (0.69×) | 175 vs ark-ff 307 | −132 (0.57×) | 91.0 vs halo2curves 138 | −47.0 (0.66×) |
| square | P-256 base | 152 vs halo2curves 201 | −49.0 (0.76×) | 74.3 vs k256/p256 74.2 | +0.1 (1.00×) | 238 vs ark-ff 342 | −104 (0.70×) | 120 vs halo2curves 120 | +0.0 (1.00×) |
| square | P-256 scalar | 213 vs ark-ff 265 | −52.0 (0.80×) | 84.9 vs halo2curves 83.1 | +1.8 (1.02×) | 250 vs ark-ff 356 | −106 (0.70×) | 158 vs halo2curves 149 | +9.0 (1.06×) |
| square | Tom-256 base | 210 vs halo2curves 312 | −102 (0.67×) | 72.2 vs halo2curves 78.6 | −6.4 (0.92×) | 219 vs halo2curves 485 | −266 (0.45×) | 140 vs halo2curves 142 | −2.0 (0.99×) |
| square | BLS12-381 base | 568 vs ark-ff 638 | −70.0 (0.89×) | 156 vs halo2curves 165 | −9.0 (0.95×) | 823 vs ark-ff 797 | +26.0 (1.03×) | 330 vs halo2curves 329 | +1.0 (1.00×) |
| square | P-521 base | 565 vs crypto-bigint 2,156 | −1,591 (0.26×) | 237 vs crypto-bigint 617 | −380 (0.38×) | 907 vs crypto-bigint 2,838 | −1,931 (0.32×) | 391 vs crypto-bigint 1,099 | −708 (0.36×) |
| invert | Mersenne-31 | 266 vs plonky3 671 | −405 (0.40×) | 267 vs plonky3 677 | −410 (0.39×) | 275 vs plonky3 729 | −454 (0.38×) | 275 vs plonky3 723 | −448 (0.38×) |
| invert | BabyBear | 269 vs plonky3 705 | −436 (0.38×) | 268 vs plonky3 699 | −431 (0.38×) | 276 vs plonky3 720 | −444 (0.38×) | 278 vs plonky3 717 | −439 (0.39×) |
| invert | Goldilocks | 539 vs winter-math 1,773 | −1,234 (0.30×) | 540 vs winter-math 912 | −372 (0.59×) | 571 vs winter-math 1,835 | −1,264 (0.31×) | 571 vs winter-math 1,001 | −430 (0.57×) |
| invert | BN254 scalar | 8,468 vs halo2curves 14,269 | −5,801 (0.59×) | 6,489 vs halo2curves 8,044 | −1,555 (0.81×) | 11,684 vs halo2curves 21,688 | −10,004 (0.54×) | 7,852 vs halo2curves 13,985 | −6,133 (0.56×) |
| invert | secp256k1 base | 8,477 vs halo2curves 14,283 | −5,806 (0.59×) | 6,020 vs halo2curves 8,048 | −2,028 (0.75×) | 12,134 vs halo2curves 21,744 | −9,610 (0.56×) | 7,911 vs halo2curves 14,305 | −6,394 (0.55×) |
| invert | P-256 base | 8,295 vs halo2curves 14,303 | −6,008 (0.58×) | 6,104 vs halo2curves 7,997 | −1,893 (0.76×) | 11,383 vs halo2curves 21,684 | −10,301 (0.52×) | 7,739 vs halo2curves 14,180 | −6,441 (0.55×) |
| invert | P-256 scalar | 8,512 vs halo2curves 14,307 | −5,795 (0.59×) | 6,405 vs halo2curves 8,058 | −1,653 (0.79×) | 12,064 vs halo2curves 21,890 | −9,826 (0.55×) | 8,190 vs halo2curves 14,269 | −6,079 (0.57×) |
| invert | Tom-256 base | 8,533 vs halo2curves 14,314 | −5,781 (0.60×) | 6,434 vs halo2curves 8,021 | −1,587 (0.80×) | 11,414 vs halo2curves 21,793 | −10,379 (0.52×) | 8,039 vs halo2curves 14,312 | −6,273 (0.56×) |
| invert | BLS12-381 base | 19,379 vs halo2curves 25,398 | −6,019 (0.76×) | 11,415 vs halo2curves 13,155 | −1,740 (0.87×) | 24,467 vs halo2curves 37,601 | −13,134 (0.65×) | 15,947 vs halo2curves 23,734 | −7,787 (0.67×) |
| invert | P-521 base | 33,340 vs crypto-bigint 65,405 | −32,065 (0.51×) | 18,119 vs crypto-bigint 50,298 | −32,179 (0.36×) | 48,645 vs crypto-bigint 80,272 | −31,627 (0.61×) | 29,588 vs crypto-bigint 61,110 | −31,522 (0.48×) |
| sqrt | Mersenne-31 | 159 vs plonky3 324 | −165 (0.49×) | 159 vs plonky3 324 | −165 (0.49×) | 160 vs plonky3 329 | −169 (0.49×) | 163 vs plonky3 329 | −166 (0.50×) |
| sqrt | BabyBear | 2,132 vs plonky3 3,103 | −971 (0.69×) | 2,058 vs plonky3 3,106 | −1,048 (0.66×) | 2,150 vs plonky3 2,995 | −845 (0.72×) | 2,087 vs plonky3 3,035 | −948 (0.69×) |
| sqrt | Goldilocks | 4,828 vs crypto-bigint 48,578 | −43,750 (0.10×) | 3,497 vs crypto-bigint 28,861 | −25,364 (0.12×) | 6,129 vs crypto-bigint 54,636 | −48,507 (0.11×) | 5,090 vs crypto-bigint 32,697 | −27,607 (0.16×) |
| sqrt | BN254 scalar | 114,468 vs ark-ff 147,848 | −33,380 (0.77×) | 51,670 vs halo2curves 102,319 | −50,649 (0.50×) | 180,838 vs ark-ff 203,027 | −22,189 (0.89×) | 98,533 vs ark-ff 141,796 | −43,263 (0.69×) |
| sqrt | secp256k1 base | 40,917 vs ark-ff 119,054 | −78,137 (0.34×) | 16,595 vs halo2curves 43,343 | −26,748 (0.38×) | 65,357 vs ark-ff 177,774 | −112,417 (0.37×) | 27,701 vs crypto-bigint 82,288 | −54,587 (0.34×) |
| sqrt | P-256 base | 43,092 vs ark-ff 60,067 | −16,975 (0.72×) | 14,467 vs k256/p256 21,710 | −7,243 (0.67×) | 78,209 vs ark-ff 97,746 | −19,537 (0.80×) | 30,838 vs k256/p256 33,696 | −2,858 (0.92×) |
| sqrt | P-256 scalar | 73,381 vs ark-ff 123,113 | −49,732 (0.60×) | 33,306 vs halo2curves 49,331 | −16,025 (0.68×) | 114,062 vs ark-ff 159,504 | −45,442 (0.72×) | 64,307 vs halo2curves 72,464 | −8,157 (0.89×) |
| sqrt | Tom-256 base | 65,879 vs crypto-bigint 149,260 | −83,381 (0.44×) | 18,318 vs halo2curves 45,397 | −27,079 (0.40×) | 99,285 vs crypto-bigint 231,678 | −132,393 (0.43×) | 51,931 vs crypto-bigint 77,298 | −25,367 (0.67×) |
| sqrt | BLS12-381 base | 297,168 vs ark-ff 421,050 | −123,882 (0.71×) | 93,867 vs crypto-bigint 145,067 | −51,200 (0.65×) | 395,495 vs ark-ff 572,766 | −177,271 (0.69×) | 217,458 vs crypto-bigint 268,646 | −51,188 (0.81×) |
| sqrt | P-521 base | 317,910 vs crypto-bigint 1,529,900 | −1,211,990 (0.21×) | 126,485 vs crypto-bigint 297,160 | −170,675 (0.43×) | 473,179 vs crypto-bigint 1,577,707 | −1,104,528 (0.30×) | 267,041 vs crypto-bigint 601,836 | −334,795 (0.44×) |
| mul_u64 | Mersenne-31 | 6.7 vs crypto-bigint 62.4 | −55.7 (0.11×) | 6.7 vs crypto-bigint 30.4 | −23.7 (0.22×) | 7.6 vs crypto-bigint 72.8 | −65.2 (0.10×) | 7.7 vs crypto-bigint 39.2 | −31.5 (0.20×) |
| mul_u64 | BabyBear | 13.0 vs plonky3 35.0 | −22.0 (0.37×) | 11.2 vs plonky3 12.1 | −0.9 (0.93×) | 18.0 vs plonky3 45.6 | −27.6 (0.39×) | 15.9 vs plonky3 17.9 | −2.0 (0.89×) |
| mul_u64 | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.3 vs winter-math 13.6 | −4.3 (0.68×) |
| mul_u64 | BN254 scalar | 101 vs ark-ff 691 | −590 (0.15×) | 28.8 vs crypto-bigint 262 | −233 (0.11×) | 124 vs ark-ff 853 | −729 (0.15×) | 43.7 vs crypto-bigint 407 | −363 (0.11×) |
| mul_u64 | secp256k1 base | 42.5 vs ark-ff 446 | −404 (0.10×) | 12.6 vs halo2curves 269 | −256 (0.05×) | 65.9 vs ark-ff 620 | −554 (0.11×) | 24.6 vs halo2curves 375 | −350 (0.07×) |
| mul_u64 | P-256 base | 72.4 vs k256/p256 488 | −416 (0.15×) | 27.3 vs k256/p256 150 | −123 (0.18×) | 98.5 vs ark-ff 853 | −755 (0.12×) | 43.0 vs k256/p256 260 | −217 (0.17×) |
| mul_u64 | P-256 scalar | 84.1 vs k256/p256 518 | −434 (0.16×) | 30.3 vs k256/p256 253 | −223 (0.12×) | 113 vs ark-ff 837 | −724 (0.14×) | 48.6 vs k256/p256 388 | −339 (0.13×) |
| mul_u64 | Tom-256 base | 82.1 vs crypto-bigint 871 | −789 (0.09×) | 30.0 vs crypto-bigint 263 | −233 (0.11×) | 116 vs halo2curves 1,307 | −1,191 (0.09×) | 45.7 vs crypto-bigint 408 | −362 (0.11×) |
| mul_u64 | BLS12-381 base | 131 vs ark-ff 1,648 | −1,517 (0.08×) | 44.2 vs halo2curves 473 | −429 (0.09×) | 188 vs ark-ff 2,128 | −1,940 (0.09×) | 58.8 vs crypto-bigint 1,036 | −977 (0.06×) |
| mul_u64 | P-521 base | 92.1 vs crypto-bigint 3,025 | −2,933 (0.03×) | 31.9 vs crypto-bigint 991 | −959 (0.03×) | 257 vs crypto-bigint 3,959 | −3,702 (0.06×) | 50.3 vs crypto-bigint 1,826 | −1,776 (0.03×) |
| mul_uint | Mersenne-31 | 6.7 vs crypto-bigint 60.1 | −53.4 (0.11×) | 6.8 vs crypto-bigint 30.4 | −23.6 (0.22×) | 7.6 vs crypto-bigint 76.2 | −68.6 (0.10×) | 7.6 vs crypto-bigint 39.2 | −31.6 (0.19×) |
| mul_uint | BabyBear | 13.1 vs plonky3 23.0 | −9.9 (0.57×) | 11.2 vs plonky3 11.6 | −0.4 (0.97×) | 18.0 vs plonky3 34.5 | −16.5 (0.52×) | 15.3 vs plonky3 17.7 | −2.4 (0.86×) |
| mul_uint | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.0 | −6.2 (0.80×) | 9.1 vs winter-math 13.5 | −4.4 (0.67×) |
| mul_uint | BN254 scalar | 437 vs halo2curves 760 | −323 (0.57×) | 171 vs halo2curves 181 | −10.0 (0.94×) | 541 vs halo2curves 1,376 | −835 (0.39×) | 290 vs halo2curves 350 | −60.0 (0.83×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 520 | −254 (0.51×) | 59.7 vs halo2curves 153 | −93.3 (0.39×) | 319 vs halo2curves 970 | −651 (0.33×) | 108 vs halo2curves 266 | −158 (0.41×) |
| mul_uint | P-256 base | 353 vs halo2curves 455 | −102 (0.78×) | 125 vs halo2curves 144 | −19.0 (0.87×) | 429 vs halo2curves 928 | −499 (0.46×) | 254 vs halo2curves 252 | +2.0 (1.01×) |
| mul_uint | P-256 scalar | 427 vs k256/p256 554 | −127 (0.77×) | 173 vs halo2curves 186 | −13.0 (0.93×) | 480 vs k256/p256 870 | −390 (0.55×) | 369 vs crypto-bigint 409 | −40.0 (0.90×) |
| mul_uint | Tom-256 base | 413 vs halo2curves 693 | −280 (0.60×) | 170 vs halo2curves 180 | −10.0 (0.94×) | 493 vs halo2curves 1,183 | −690 (0.42×) | 340 vs halo2curves 398 | −58.0 (0.85×) |
| mul_uint | BLS12-381 base | 987 vs crypto-bigint 1,949 | −962 (0.51×) | 363 vs halo2curves 382 | −19.0 (0.95×) | 1,161 vs crypto-bigint 2,570 | −1,409 (0.45×) | 904 vs crypto-bigint 1,010 | −106 (0.90×) |
| mul_uint | P-521 base | 1,564 vs crypto-bigint 3,032 | −1,468 (0.52×) | 258 vs crypto-bigint 990 | −732 (0.26×) | 2,388 vs crypto-bigint 4,058 | −1,670 (0.59×) | 463 vs crypto-bigint 1,840 | −1,377 (0.25×) |
| add_u64 | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.3 vs crypto-bigint 42.1 | −36.8 (0.13×) | 5.4 vs crypto-bigint 30.4 | −25.0 (0.18×) |
| add_u64 | BabyBear | 10.9 vs plonky3 32.2 | −21.3 (0.34×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 43.6 | −30.6 (0.30×) | 12.7 vs plonky3 16.1 | −3.4 (0.79×) |
| add_u64 | Goldilocks | 9.2 vs winter-math 13.4 | −4.2 (0.69×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 10.0 vs winter-math 10.1 | −0.1 (0.99×) |
| add_u64 | BN254 scalar | 118 vs ark-ff 360 | −242 (0.33×) | 52.8 vs halo2curves 209 | −156 (0.25×) | 143 vs ark-ff 430 | −287 (0.33×) | 89.3 vs halo2curves 294 | −205 (0.30×) |
| add_u64 | secp256k1 base | 8.0 vs ark-ff 163 | −155 (0.05×) | 8.0 vs ark-ff 121 | −113 (0.07×) | 8.0 vs ark-ff 228 | −220 (0.04×) | 11.0 vs ark-ff 185 | −174 (0.06×) |
| add_u64 | P-256 base | 71.3 vs k256/p256 277 | −206 (0.26×) | 36.0 vs k256/p256 96.7 | −60.7 (0.37×) | 87.0 vs ark-ff 408 | −321 (0.21×) | 66.6 vs k256/p256 159 | −92.4 (0.42×) |
| add_u64 | P-256 scalar | 70.6 vs k256/p256 41.1 | +29.5 (1.72×) | 40.6 vs k256/p256 41.3 | −0.7 (0.98×) | 74.3 vs k256/p256 55.6 | +18.7 (1.34×) | 84.2 vs k256/p256 55.8 | +28.4 (1.51×) |
| add_u64 | Tom-256 base | 88.5 vs crypto-bigint 502 | −414 (0.18×) | 42.0 vs halo2curves 220 | −178 (0.19×) | 108 vs halo2curves 710 | −602 (0.15×) | 71.6 vs halo2curves 288 | −216 (0.25×) |
| add_u64 | BLS12-381 base | 167 vs ark-ff 835 | −668 (0.20×) | 66.7 vs halo2curves 324 | −257 (0.21×) | 255 vs ark-ff 1,087 | −832 (0.23×) | 95.8 vs crypto-bigint 651 | −555 (0.15×) |
| add_u64 | P-521 base | 20.8 vs crypto-bigint 1,678 | −1,657 (0.01×) | 15.4 vs crypto-bigint 648 | −633 (0.02×) | 22.1 vs crypto-bigint 2,183 | −2,161 (0.01×) | 23.4 vs crypto-bigint 1,122 | −1,099 (0.02×) |
| add_uint | Mersenne-31 | 5.2 vs crypto-bigint 36.2 | −31.0 (0.14×) | 5.2 vs crypto-bigint 30.3 | −25.1 (0.17×) | 5.4 vs crypto-bigint 42.1 | −36.7 (0.13×) | 5.4 vs crypto-bigint 30.4 | −25.0 (0.18×) |
| add_uint | BabyBear | 10.9 vs plonky3 22.1 | −11.2 (0.49×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 34.3 | −21.3 (0.38×) | 12.5 vs plonky3 16.0 | −3.5 (0.78×) |
| add_uint | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.0 vs winter-math 7.4 | −0.4 (0.95×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 10.0 vs winter-math 10.1 | −0.1 (0.99×) |
| add_uint | BN254 scalar | 248 vs halo2curves 412 | −164 (0.60×) | 91.4 vs halo2curves 113 | −21.6 (0.81×) | 360 vs halo2curves 695 | −335 (0.52×) | 191 vs halo2curves 205 | −14.0 (0.93×) |
| add_uint | secp256k1 base | 20.9 vs halo2curves 225 | −204 (0.09×) | 14.6 vs halo2curves 86.6 | −72.0 (0.17×) | 18.1 vs halo2curves 410 | −392 (0.04×) | 24.3 vs halo2curves 140 | −116 (0.17×) |
| add_uint | P-256 base | 188 vs halo2curves 246 | −58.0 (0.76×) | 74.9 vs halo2curves 91.9 | −17.0 (0.82×) | 325 vs halo2curves 518 | −193 (0.63×) | 147 vs halo2curves 153 | −6.0 (0.96×) |
| add_uint | P-256 scalar | 260 vs k256/p256 95.0 | +165 (2.74×) | 96.7 vs k256/p256 94.9 | +1.8 (1.02×) | 336 vs k256/p256 118 | +218 (2.85×) | 223 vs k256/p256 117 | +106 (1.91×) |
| add_uint | Tom-256 base | 251 vs halo2curves 379 | −128 (0.66×) | 99.2 vs halo2curves 111 | −11.8 (0.89×) | 333 vs halo2curves 636 | −303 (0.52×) | 190 vs halo2curves 196 | −6.0 (0.97×) |
| add_uint | BLS12-381 base | 768 vs halo2curves 984 | −216 (0.78×) | 199 vs halo2curves 224 | −25.0 (0.89×) | 951 vs crypto-bigint 1,422 | −471 (0.67×) | 409 vs halo2curves 494 | −85.0 (0.83×) |
| add_uint | P-521 base | 65.4 vs crypto-bigint 1,682 | −1,617 (0.04×) | 41.9 vs crypto-bigint 647 | −605 (0.06×) | 89.8 vs crypto-bigint 2,240 | −2,150 (0.04×) | 76.3 vs crypto-bigint 1,159 | −1,083 (0.07×) |
| sub_u64 | Mersenne-31 | 5.0 vs crypto-bigint 23.5 | −18.5 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.1 vs crypto-bigint 31.2 | −26.1 (0.16×) | 5.1 vs crypto-bigint 21.2 | −16.1 (0.24×) |
| sub_u64 | BabyBear | 11.0 vs crypto-bigint 28.0 | −17.0 (0.39×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs crypto-bigint 43.9 | −30.9 (0.30×) | 13.0 vs plonky3 15.1 | −2.1 (0.86×) |
| sub_u64 | Goldilocks | 9.5 vs winter-math 11.5 | −2.0 (0.83×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.6 vs winter-math 19.0 | −7.4 (0.61×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| sub_u64 | BN254 scalar | 109 vs ark-ff 352 | −243 (0.31×) | 54.7 vs crypto-bigint 151 | −96.3 (0.36×) | 137 vs ark-ff 418 | −281 (0.33×) | 73.3 vs crypto-bigint 254 | −181 (0.29×) |
| sub_u64 | secp256k1 base | 8.7 vs ark-ff 162 | −153 (0.05×) | 7.2 vs ark-ff 124 | −117 (0.06×) | 7.5 vs ark-ff 232 | −225 (0.03×) | 9.2 vs ark-ff 193 | −184 (0.05×) |
| sub_u64 | P-256 base | 70.6 vs k256/p256 265 | −194 (0.27×) | 34.9 vs k256/p256 90.0 | −55.1 (0.39×) | 80.0 vs ark-ff 408 | −328 (0.20×) | 61.3 vs k256/p256 150 | −88.7 (0.41×) |
| sub_u64 | P-256 scalar | 68.0 vs k256/p256 21.2 | +46.8 (3.21×) | 39.1 vs k256/p256 21.2 | +17.9 (1.84×) | 74.2 vs k256/p256 24.4 | +49.8 (3.04×) | 87.3 vs k256/p256 24.2 | +63.1 (3.61×) |
| sub_u64 | Tom-256 base | 87.7 vs crypto-bigint 494 | −406 (0.18×) | 50.2 vs crypto-bigint 151 | −101 (0.33×) | 104 vs halo2curves 698 | −594 (0.15×) | 74.1 vs halo2curves 282 | −208 (0.26×) |
| sub_u64 | BLS12-381 base | 153 vs ark-ff 837 | −684 (0.18×) | 65.4 vs halo2curves 316 | −251 (0.21×) | 250 vs ark-ff 1,078 | −828 (0.23×) | 98.9 vs crypto-bigint 602 | −503 (0.16×) |
| sub_u64 | P-521 base | 22.3 vs crypto-bigint 1,667 | −1,645 (0.01×) | 18.0 vs crypto-bigint 621 | −603 (0.03×) | 20.2 vs crypto-bigint 2,167 | −2,147 (0.01×) | 20.1 vs crypto-bigint 1,105 | −1,085 (0.02×) |
| sub_uint | Mersenne-31 | 5.0 vs crypto-bigint 23.6 | −18.6 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.1 vs crypto-bigint 31.2 | −26.1 (0.16×) | 5.2 vs crypto-bigint 21.8 | −16.6 (0.24×) |
| sub_uint | BabyBear | 11.1 vs plonky3 23.0 | −11.9 (0.48×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs plonky3 34.5 | −21.5 (0.38×) | 13.1 vs plonky3 15.3 | −2.2 (0.86×) |
| sub_uint | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.6 vs winter-math 19.0 | −7.4 (0.61×) | 9.2 vs winter-math 9.5 | −0.3 (0.97×) |
| sub_uint | BN254 scalar | 246 vs halo2curves 401 | −155 (0.61×) | 92.3 vs halo2curves 104 | −11.7 (0.89×) | 364 vs halo2curves 650 | −286 (0.56×) | 191 vs halo2curves 193 | −2.0 (0.99×) |
| sub_uint | secp256k1 base | 21.2 vs halo2curves 212 | −191 (0.10×) | 13.6 vs halo2curves 104 | −90.4 (0.13×) | 19.3 vs halo2curves 408 | −389 (0.05×) | 23.4 vs halo2curves 137 | −114 (0.17×) |
| sub_uint | P-256 base | 170 vs halo2curves 232 | −62.0 (0.73×) | 80.9 vs halo2curves 85.6 | −4.7 (0.95×) | 295 vs halo2curves 496 | −201 (0.59×) | 143 vs halo2curves 143 | +0.0 (1.00×) |
| sub_uint | P-256 scalar | 249 vs k256/p256 70.3 | +179 (3.54×) | 103 vs k256/p256 65.9 | +37.1 (1.56×) | 331 vs k256/p256 88.9 | +242 (3.72×) | 199 vs k256/p256 90.7 | +108 (2.19×) |
| sub_uint | Tom-256 base | 251 vs halo2curves 366 | −115 (0.69×) | 93.5 vs halo2curves 104 | −10.5 (0.90×) | 325 vs halo2curves 621 | −296 (0.52×) | 191 vs halo2curves 192 | −1.0 (0.99×) |
| sub_uint | BLS12-381 base | 762 vs halo2curves 952 | −190 (0.80×) | 208 vs halo2curves 216 | −8.0 (0.96×) | 939 vs crypto-bigint 1,395 | −456 (0.67×) | 428 vs halo2curves 411 | +17.0 (1.04×) |
| sub_uint | P-521 base | 75.4 vs crypto-bigint 1,677 | −1,602 (0.04×) | 40.0 vs crypto-bigint 621 | −581 (0.06×) | 89.0 vs crypto-bigint 2,206 | −2,117 (0.04×) | 69.2 vs crypto-bigint 1,122 | −1,053 (0.06×) |
| from_u64 | Mersenne-31 | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.1 vs plonky3 3.9 | +0.2 (1.05×) | 4.1 vs plonky3 3.9 | +0.2 (1.05×) |
| from_u64 | BabyBear | 11.1 vs crypto-bigint 21.6 | −10.5 (0.51×) | 7.3 vs plonky3 8.0 | −0.7 (0.91×) | 12.7 vs crypto-bigint 36.0 | −23.3 (0.35×) | 10.3 vs plonky3 11.9 | −1.6 (0.87×) |
| from_u64 | Goldilocks | 7.4 vs winter-math 10.6 | −3.2 (0.70×) | 5.9 vs winter-math 5.9 | +0.0 (1.00×) | 9.1 vs winter-math 17.0 | −7.9 (0.54×) | 7.1 vs winter-math 7.1 | +0.0 (1.00×) |
| from_u64 | BN254 scalar | 101 vs ark-ff 315 | −214 (0.32×) | 29.2 vs crypto-bigint 133 | −104 (0.22×) | 117 vs ark-ff 380 | −263 (0.31×) | 50.2 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64 | secp256k1 base | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.6 | −2.2 (0.61×) |
| from_u64 | P-256 base | 56.7 vs k256/p256 239 | −182 (0.24×) | 24.6 vs k256/p256 76.2 | −51.6 (0.32×) | 70.3 vs ark-ff 378 | −308 (0.19×) | 41.1 vs k256/p256 130 | −88.9 (0.32×) |
| from_u64 | P-256 scalar | 55.1 vs k256/p256 3.7 | +51.4 (14.89×) | 26.2 vs k256/p256 3.7 | +22.5 (7.08×) | 58.3 vs k256/p256 3.8 | +54.5 (15.34×) | 49.4 vs k256/p256 3.7 | +45.7 (13.35×) |
| from_u64 | Tom-256 base | 74.6 vs crypto-bigint 428 | −353 (0.17×) | 28.2 vs crypto-bigint 133 | −105 (0.21×) | 86.1 vs halo2curves 663 | −577 (0.13×) | 48.8 vs crypto-bigint 205 | −156 (0.24×) |
| from_u64 | BLS12-381 base | 144 vs ark-ff 741 | −597 (0.19×) | 50.0 vs crypto-bigint 252 | −202 (0.20×) | 182 vs ark-ff 1,012 | −830 (0.18×) | 70.6 vs crypto-bigint 565 | −494 (0.12×) |
| from_u64 | P-521 base | 6.0 vs crypto-bigint 1,456 | −1,450 (0.00×) | 6.0 vs crypto-bigint 498 | −492 (0.01×) | 6.0 vs crypto-bigint 1,994 | −1,988 (0.00×) | 6.1 vs crypto-bigint 918 | −912 (0.01×) |
| div_u64 | BN254 scalar | 1,212 vs halo2curves 13,705 | −12,493 (0.09×) | 839 vs halo2curves 7,045 | −6,206 (0.12×) | 1,365 vs halo2curves 19,363 | −17,998 (0.07×) | 1,144 vs halo2curves 11,826 | −10,682 (0.10×) |
| div_u64 | secp256k1 base | 1,008 vs halo2curves 13,125 | −12,117 (0.08×) | 712 vs halo2curves 6,937 | −6,225 (0.10×) | 1,234 vs halo2curves 18,896 | −17,662 (0.07×) | 801 vs halo2curves 11,578 | −10,777 (0.07×) |
| div_u64 | P-256 base | 1,091 vs halo2curves 13,091 | −12,000 (0.08×) | 804 vs halo2curves 6,885 | −6,081 (0.12×) | 1,353 vs halo2curves 18,919 | −17,566 (0.07×) | 981 vs halo2curves 11,669 | −10,688 (0.08×) |
| div_u64 | P-256 scalar | 1,188 vs halo2curves 13,755 | −12,567 (0.09×) | 847 vs halo2curves 6,973 | −6,126 (0.12×) | 1,427 vs halo2curves 19,277 | −17,850 (0.07×) | 1,129 vs halo2curves 11,601 | −10,472 (0.10×) |
| div_u64 | Tom-256 base | 1,210 vs halo2curves 13,274 | −12,064 (0.09×) | 849 vs halo2curves 6,954 | −6,105 (0.12×) | 1,499 vs halo2curves 18,958 | −17,459 (0.08×) | 1,064 vs halo2curves 11,775 | −10,711 (0.09×) |
| div_u64 | BLS12-381 base | 1,803 vs halo2curves 24,376 | −22,573 (0.07×) | 1,076 vs halo2curves 10,984 | −9,908 (0.10×) | 2,479 vs halo2curves 33,108 | −30,629 (0.07×) | 1,789 vs halo2curves 18,458 | −16,669 (0.10×) |
| div_u64 | P-521 base | 2,452 vs crypto-bigint 67,351 | −64,899 (0.04×) | 1,001 vs crypto-bigint 46,366 | −45,365 (0.02×) | 3,585 vs crypto-bigint 78,294 | −74,709 (0.05×) | 1,454 vs crypto-bigint 57,788 | −56,334 (0.03×) |
| div_uint | Mersenne-31 | 273 vs crypto-bigint 1,825 | −1,552 (0.15×) | 275 vs crypto-bigint 1,607 | −1,332 (0.17×) | 282 vs runtime-algebra 2,112 | −1,830 (0.13×) | 285 vs crypto-bigint 2,044 | −1,759 (0.14×) |
| div_uint | BabyBear | 276 vs plonky3 676 | −400 (0.41×) | 274 vs plonky3 656 | −382 (0.42×) | 282 vs plonky3 739 | −457 (0.38×) | 280 vs plonky3 722 | −442 (0.39×) |
| div_uint | Goldilocks | 554 vs winter-math 1,836 | −1,282 (0.30×) | 540 vs winter-math 933 | −393 (0.58×) | 591 vs winter-math 1,887 | −1,296 (0.31×) | 575 vs winter-math 1,055 | −480 (0.55×) |
| div_uint | BN254 scalar | 9,251 vs halo2curves 15,063 | −5,812 (0.61×) | 6,586 vs halo2curves 8,225 | −1,639 (0.80×) | 12,719 vs halo2curves 22,812 | −10,093 (0.56×) | 8,516 vs halo2curves 14,572 | −6,056 (0.58×) |
| div_uint | secp256k1 base | 8,607 vs halo2curves 14,827 | −6,220 (0.58×) | 6,038 vs halo2curves 8,236 | −2,198 (0.73×) | 12,338 vs halo2curves 22,470 | −10,132 (0.55×) | 8,186 vs halo2curves 14,655 | −6,469 (0.56×) |
| div_uint | P-256 base | 8,817 vs halo2curves 14,999 | −6,182 (0.59×) | 6,202 vs halo2curves 8,242 | −2,040 (0.75×) | 12,107 vs halo2curves 22,475 | −10,368 (0.54×) | 7,880 vs halo2curves 14,404 | −6,524 (0.55×) |
| div_uint | P-256 scalar | 9,217 vs halo2curves 15,111 | −5,894 (0.61×) | 6,565 vs halo2curves 8,242 | −1,677 (0.80×) | 12,531 vs halo2curves 23,146 | −10,615 (0.54×) | 8,660 vs halo2curves 14,669 | −6,009 (0.59×) |
| div_uint | Tom-256 base | 9,097 vs halo2curves 15,055 | −5,958 (0.60×) | 6,614 vs halo2curves 8,259 | −1,645 (0.80×) | 12,210 vs halo2curves 22,838 | −10,628 (0.53×) | 8,421 vs halo2curves 14,707 | −6,286 (0.57×) |
| div_uint | BLS12-381 base | 20,969 vs halo2curves 27,734 | −6,765 (0.76×) | 12,030 vs halo2curves 13,623 | −1,593 (0.88×) | 26,343 vs halo2curves 40,886 | −14,543 (0.64×) | 17,298 vs halo2curves 24,919 | −7,621 (0.69×) |
| div_uint | P-521 base | 34,083 vs crypto-bigint 71,694 | −37,611 (0.48×) | 18,423 vs crypto-bigint 53,030 | −34,607 (0.35×) | 49,809 vs crypto-bigint 84,973 | −35,164 (0.59×) | 30,655 vs crypto-bigint 64,875 | −34,220 (0.47×) |
| add, constant time | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add, constant time | BabyBear | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 2.7 vs runtime-algebra 11.3 | −8.6 (0.24×) | 3.0 vs crypto-bigint 10.8 | −7.8 (0.28×) | 3.0 vs runtime-algebra 10.8 | −7.8 (0.28×) |
| add, constant time | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.4 | −0.1 (0.97×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add, constant time | BN254 scalar | 23.7 vs halo2curves 36.4 | −12.7 (0.65×) | 16.2 vs halo2curves 20.6 | −4.4 (0.79×) | 20.7 vs halo2curves 39.3 | −18.6 (0.53×) | 26.1 vs halo2curves 35.1 | −9.0 (0.74×) |
| add, constant time | secp256k1 base | 26.2 vs halo2curves 35.1 | −8.9 (0.75×) | 15.9 vs halo2curves 19.7 | −3.8 (0.81×) | 24.5 vs halo2curves 36.0 | −11.5 (0.68×) | 28.0 vs halo2curves 33.0 | −5.0 (0.85×) |
| add, constant time | P-256 base | 23.6 vs k256/p256 30.8 | −7.2 (0.77×) | 15.6 vs k256/p256 18.0 | −2.4 (0.87×) | 22.5 vs halo2curves 33.4 | −10.9 (0.67×) | 27.0 vs k256/p256 31.5 | −4.5 (0.86×) |
| add, constant time | P-256 scalar | 27.1 vs halo2curves 35.5 | −8.4 (0.76×) | 16.1 vs halo2curves 20.8 | −4.7 (0.77×) | 23.9 vs halo2curves 38.1 | −14.2 (0.63×) | 28.5 vs halo2curves 35.2 | −6.7 (0.81×) |
| add, constant time | Tom-256 base | 27.7 vs halo2curves 35.9 | −8.2 (0.77×) | 16.1 vs halo2curves 20.4 | −4.3 (0.79×) | 24.2 vs halo2curves 38.7 | −14.5 (0.63×) | 29.1 vs halo2curves 34.1 | −5.0 (0.85×) |
| add, constant time | BLS12-381 base | 42.8 vs halo2curves 63.5 | −20.7 (0.67×) | 24.3 vs halo2curves 32.5 | −8.2 (0.75×) | 35.6 vs halo2curves 69.9 | −34.3 (0.51×) | 40.1 vs halo2curves 54.1 | −14.0 (0.74×) |
| add, constant time | P-521 base | 76.8 vs crypto-bigint 144 | −67.2 (0.53×) | 40.7 vs crypto-bigint 138 | −97.3 (0.29×) | 75.7 vs crypto-bigint 218 | −142 (0.35×) | 72.0 vs crypto-bigint 195 | −123 (0.37×) |
| sub, constant time | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | BabyBear | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.6 vs crypto-bigint 4.7 | −2.1 (0.55×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.9 | −0.3 (0.90×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.1 vs winter-math 3.0 | +0.1 (1.03×) |
| sub, constant time | BN254 scalar | 21.1 vs halo2curves 24.0 | −2.9 (0.88×) | 14.5 vs halo2curves 14.9 | −0.4 (0.97×) | 20.6 vs halo2curves 23.3 | −2.7 (0.88×) | 23.1 vs halo2curves 24.0 | −0.9 (0.96×) |
| sub, constant time | secp256k1 base | 19.1 vs halo2curves 22.4 | −3.3 (0.85×) | 13.6 vs halo2curves 14.4 | −0.8 (0.94×) | 18.7 vs halo2curves 20.3 | −1.6 (0.92×) | 22.0 vs halo2curves 22.0 | +0.0 (1.00×) |
| sub, constant time | P-256 base | 17.3 vs halo2curves 20.5 | −3.2 (0.84×) | 12.5 vs halo2curves 12.3 | +0.2 (1.02×) | 18.4 vs halo2curves 21.2 | −2.8 (0.87×) | 21.0 vs halo2curves 21.6 | −0.6 (0.97×) |
| sub, constant time | P-256 scalar | 21.0 vs k256/p256 23.2 | −2.2 (0.91×) | 14.3 vs halo2curves 14.5 | −0.2 (0.99×) | 21.0 vs halo2curves 23.2 | −2.2 (0.91×) | 23.0 vs halo2curves 23.2 | −0.2 (0.99×) |
| sub, constant time | Tom-256 base | 21.0 vs halo2curves 23.9 | −2.9 (0.88×) | 15.0 vs halo2curves 15.1 | −0.1 (0.99×) | 20.9 vs halo2curves 23.5 | −2.6 (0.89×) | 23.1 vs halo2curves 24.2 | −1.1 (0.95×) |
| sub, constant time | BLS12-381 base | 37.6 vs halo2curves 43.3 | −5.7 (0.87×) | 21.3 vs halo2curves 24.4 | −3.1 (0.87×) | 34.7 vs halo2curves 44.2 | −9.5 (0.79×) | 36.0 vs halo2curves 37.0 | −1.0 (0.97×) |
| sub, constant time | P-521 base | 56.3 vs crypto-bigint 88.4 | −32.1 (0.64×) | 31.2 vs crypto-bigint 64.3 | −33.1 (0.49×) | 58.7 vs crypto-bigint 122 | −63.3 (0.48×) | 51.0 vs crypto-bigint 128 | −77.0 (0.40×) |
| mul, constant time | Mersenne-31 | 3.6 vs crypto-bigint 29.3 | −25.7 (0.12×) | 3.6 vs crypto-bigint 15.9 | −12.3 (0.23×) | 4.0 vs crypto-bigint 39.2 | −35.2 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul, constant time | BabyBear | 5.2 vs crypto-bigint 29.3 | −24.1 (0.18×) | 5.2 vs crypto-bigint 15.9 | −10.7 (0.33×) | 5.2 vs crypto-bigint 39.2 | −34.0 (0.13×) | 5.0 vs crypto-bigint 19.2 | −14.2 (0.26×) |
| mul, constant time | Goldilocks | 10.0 vs winter-math 12.4 | −2.4 (0.81×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 15.0 vs winter-math 17.1 | −2.1 (0.88×) | 7.0 vs winter-math 7.7 | −0.7 (0.91×) |
| mul, constant time | BN254 scalar | 237 vs halo2curves 377 | −140 (0.63×) | 88.1 vs halo2curves 91.4 | −3.3 (0.96×) | 416 vs halo2curves 706 | −290 (0.59×) | 161 vs halo2curves 182 | −21.0 (0.88×) |
| mul, constant time | secp256k1 base | 160 vs halo2curves 304 | −144 (0.53×) | 59.6 vs halo2curves 91.3 | −31.7 (0.65×) | 226 vs halo2curves 594 | −368 (0.38×) | 107 vs halo2curves 164 | −57.0 (0.65×) |
| mul, constant time | P-256 base | 171 vs k256/p256 241 | −70.0 (0.71×) | 71.6 vs k256/p256 85.2 | −13.6 (0.84×) | 286 vs k256/p256 484 | −198 (0.59×) | 121 vs halo2curves 135 | −14.0 (0.90×) |
| mul, constant time | P-256 scalar | 239 vs halo2curves 340 | −101 (0.70×) | 89.8 vs halo2curves 93.7 | −3.9 (0.96×) | 332 vs halo2curves 703 | −371 (0.47×) | 175 vs halo2curves 182 | −7.0 (0.96×) |
| mul, constant time | Tom-256 base | 236 vs halo2curves 343 | −107 (0.69×) | 80.7 vs halo2curves 94.2 | −13.5 (0.86×) | 317 vs halo2curves 584 | −267 (0.54×) | 164 vs halo2curves 176 | −12.0 (0.93×) |
| mul, constant time | BLS12-381 base | 725 vs crypto-bigint 900 | −175 (0.81×) | 187 vs halo2curves 195 | −8.0 (0.96×) | 916 vs crypto-bigint 1,312 | −396 (0.70×) | 333 vs halo2curves 383 | −50.0 (0.87×) |
| mul, constant time | P-521 base | 667 vs crypto-bigint 2,124 | −1,457 (0.31×) | 245 vs crypto-bigint 600 | −355 (0.41×) | 1,281 vs crypto-bigint 2,702 | −1,421 (0.47×) | 534 vs crypto-bigint 1,059 | −525 (0.50×) |
| square, constant time | Mersenne-31 | 3.5 vs crypto-bigint 28.6 | −25.1 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 38.0 | −34.4 (0.09×) | 3.7 vs crypto-bigint 17.8 | −14.1 (0.21×) |
| square, constant time | BabyBear | 4.7 vs crypto-bigint 28.7 | −24.0 (0.16×) | 4.8 vs crypto-bigint 15.6 | −10.8 (0.31×) | 5.7 vs crypto-bigint 38.1 | −32.4 (0.15×) | 5.4 vs crypto-bigint 18.1 | −12.7 (0.30×) |
| square, constant time | Goldilocks | 8.6 vs winter-math 12.4 | −3.8 (0.69×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.1 vs winter-math 7.1 | +1.0 (1.14×) |
| square, constant time | BN254 scalar | 213 vs halo2curves 312 | −99.0 (0.68×) | 86.4 vs halo2curves 84.7 | +1.7 (1.02×) | 177 vs halo2curves 549 | −372 (0.32×) | 159 vs halo2curves 148 | +11.0 (1.07×) |
| square, constant time | secp256k1 base | 144 vs halo2curves 267 | −123 (0.54×) | 53.1 vs halo2curves 76.9 | −23.8 (0.69×) | 171 vs halo2curves 461 | −290 (0.37×) | 91.5 vs halo2curves 138 | −46.5 (0.66×) |
| square, constant time | P-256 base | 148 vs halo2curves 201 | −53.0 (0.74×) | 74.3 vs k256/p256 74.2 | +0.1 (1.00×) | 259 vs halo2curves 346 | −87.0 (0.75×) | 125 vs halo2curves 120 | +5.0 (1.04×) |
| square, constant time | P-256 scalar | 215 vs halo2curves 271 | −56.0 (0.79×) | 77.9 vs halo2curves 83.1 | −5.2 (0.94×) | 250 vs halo2curves 551 | −301 (0.45×) | 166 vs halo2curves 149 | +17.0 (1.11×) |
| square, constant time | Tom-256 base | 207 vs halo2curves 312 | −105 (0.66×) | 72.8 vs halo2curves 78.6 | −5.8 (0.93×) | 227 vs halo2curves 485 | −258 (0.47×) | 129 vs halo2curves 142 | −13.0 (0.91×) |
| square, constant time | BLS12-381 base | 567 vs halo2curves 698 | −131 (0.81×) | 158 vs halo2curves 165 | −7.0 (0.96×) | 824 vs halo2curves 1,352 | −528 (0.61×) | 296 vs halo2curves 329 | −33.0 (0.90×) |
| square, constant time | P-521 base | 563 vs crypto-bigint 2,156 | −1,593 (0.26×) | 238 vs crypto-bigint 617 | −379 (0.39×) | 912 vs crypto-bigint 2,838 | −1,926 (0.32×) | 391 vs crypto-bigint 1,099 | −708 (0.36×) |
| invert, constant time | Mersenne-31 | 263 vs runtime-algebra 3,340 | −3,077 (0.08×) | 264 vs runtime-algebra 3,340 | −3,076 (0.08×) | 271 vs runtime-algebra 3,330 | −3,059 (0.08×) | 267 vs runtime-algebra 3,386 | −3,119 (0.08×) |
| invert, constant time | BabyBear | 451 vs runtime-algebra 3,298 | −2,847 (0.14×) | 458 vs runtime-algebra 3,317 | −2,859 (0.14×) | 447 vs runtime-algebra 3,303 | −2,856 (0.14×) | 418 vs runtime-algebra 3,400 | −2,982 (0.12×) |
| invert, constant time | Goldilocks | 1,154 vs winter-math 1,773 | −619 (0.65×) | 841 vs winter-math 912 | −71.0 (0.92×) | 1,420 vs winter-math 1,835 | −415 (0.77×) | 936 vs winter-math 1,001 | −65.0 (0.94×) |
| invert, constant time | BN254 scalar | 11,983 vs crypto-bigint 29,664 | −17,681 (0.40×) | 9,358 vs crypto-bigint 25,355 | −15,997 (0.37×) | 17,132 vs crypto-bigint 35,662 | −18,530 (0.48×) | 12,564 vs crypto-bigint 30,802 | −18,238 (0.41×) |
| invert, constant time | secp256k1 base | 11,830 vs crypto-bigint 29,415 | −17,585 (0.40×) | 9,051 vs crypto-bigint 25,455 | −16,404 (0.36×) | 16,792 vs crypto-bigint 35,836 | −19,044 (0.47×) | 12,577 vs crypto-bigint 30,376 | −17,799 (0.41×) |
| invert, constant time | P-256 base | 11,489 vs crypto-bigint 28,999 | −17,510 (0.40×) | 8,945 vs k256/p256 22,131 | −13,186 (0.40×) | 16,378 vs crypto-bigint 35,749 | −19,371 (0.46×) | 12,433 vs crypto-bigint 30,680 | −18,247 (0.41×) |
| invert, constant time | P-256 scalar | 11,870 vs crypto-bigint 29,453 | −17,583 (0.40×) | 9,413 vs crypto-bigint 25,455 | −16,042 (0.37×) | 17,061 vs crypto-bigint 35,773 | −18,712 (0.48×) | 12,746 vs crypto-bigint 30,631 | −17,885 (0.42×) |
| invert, constant time | Tom-256 base | 11,943 vs crypto-bigint 29,535 | −17,592 (0.40×) | 9,282 vs crypto-bigint 25,296 | −16,014 (0.37×) | 16,839 vs crypto-bigint 35,619 | −18,780 (0.47×) | 12,806 vs crypto-bigint 30,438 | −17,632 (0.42×) |
| invert, constant time | BLS12-381 base | 24,033 vs crypto-bigint 48,926 | −24,893 (0.49×) | 16,674 vs crypto-bigint 42,420 | −25,746 (0.39×) | 32,779 vs crypto-bigint 64,342 | −31,563 (0.51×) | 24,086 vs crypto-bigint 54,065 | −29,979 (0.45×) |
| invert, constant time | P-521 base | 41,710 vs crypto-bigint 100,770 | −59,060 (0.41×) | 26,021 vs crypto-bigint 81,885 | −55,864 (0.32×) | 62,818 vs crypto-bigint 123,099 | −60,281 (0.51×) | 41,805 vs crypto-bigint 99,778 | −57,973 (0.42×) |
| sqrt, constant time | Mersenne-31 | 159 vs runtime-algebra 1,741 | −1,582 (0.09×) | 159 vs crypto-bigint 1,631 | −1,472 (0.10×) | 170 vs runtime-algebra 1,740 | −1,570 (0.10×) | 171 vs runtime-algebra 1,795 | −1,624 (0.10×) |
| sqrt, constant time | BabyBear | 7,130 vs crypto-bigint 34,067 | −26,937 (0.21×) | 6,446 vs crypto-bigint 20,216 | −13,770 (0.32×) | 6,762 vs crypto-bigint 38,437 | −31,675 (0.18×) | 6,529 vs crypto-bigint 22,827 | −16,298 (0.29×) |
| sqrt, constant time | Goldilocks | 10,865 vs crypto-bigint 48,578 | −37,713 (0.22×) | 8,377 vs crypto-bigint 28,861 | −20,484 (0.29×) | 14,626 vs crypto-bigint 54,636 | −40,010 (0.27×) | 10,334 vs crypto-bigint 32,697 | −22,363 (0.32×) |
| sqrt, constant time | BN254 scalar | 180,163 vs crypto-bigint 329,141 | −148,978 (0.55×) | 84,788 vs halo2curves 102,319 | −17,531 (0.83×) | 293,067 vs halo2curves 533,959 | −240,892 (0.55×) | 149,268 vs halo2curves 178,948 | −29,680 (0.83×) |
| sqrt, constant time | secp256k1 base | 41,221 vs crypto-bigint 159,105 | −117,884 (0.26×) | 16,810 vs halo2curves 43,343 | −26,533 (0.39×) | 66,520 vs crypto-bigint 249,569 | −183,049 (0.27×) | 26,663 vs crypto-bigint 82,288 | −55,625 (0.32×) |
| sqrt, constant time | P-256 base | 43,007 vs k256/p256 64,303 | −21,296 (0.67×) | 15,245 vs k256/p256 21,710 | −6,465 (0.70×) | 74,750 vs k256/p256 123,598 | −48,848 (0.60×) | 31,055 vs k256/p256 33,696 | −2,641 (0.92×) |
| sqrt, constant time | P-256 scalar | 77,950 vs halo2curves 154,092 | −76,142 (0.51×) | 34,643 vs halo2curves 49,331 | −14,688 (0.70×) | 117,621 vs crypto-bigint 246,820 | −129,199 (0.48×) | 66,290 vs halo2curves 72,464 | −6,174 (0.91×) |
| sqrt, constant time | Tom-256 base | 66,073 vs crypto-bigint 149,260 | −83,187 (0.44×) | 29,280 vs halo2curves 45,397 | −16,117 (0.64×) | 98,912 vs crypto-bigint 231,678 | −132,766 (0.43×) | 50,967 vs crypto-bigint 77,298 | −26,331 (0.66×) |
| sqrt, constant time | BLS12-381 base | 297,973 vs crypto-bigint 543,621 | −245,648 (0.55×) | 93,625 vs crypto-bigint 145,067 | −51,442 (0.65×) | 392,535 vs crypto-bigint 619,742 | −227,207 (0.63×) | 206,963 vs crypto-bigint 268,646 | −61,683 (0.77×) |
| sqrt, constant time | P-521 base | 324,093 vs crypto-bigint 1,529,900 | −1,205,807 (0.21×) | 124,267 vs crypto-bigint 297,160 | −172,893 (0.42×) | 507,512 vs crypto-bigint 1,577,707 | −1,070,195 (0.32×) | 255,664 vs crypto-bigint 601,836 | −346,172 (0.42×) |
| mul_u64, constant time | Mersenne-31 | 6.7 vs crypto-bigint 62.4 | −55.7 (0.11×) | 6.7 vs crypto-bigint 30.4 | −23.7 (0.22×) | 7.6 vs crypto-bigint 72.8 | −65.2 (0.10×) | 7.7 vs crypto-bigint 39.2 | −31.5 (0.20×) |
| mul_u64, constant time | BabyBear | 13.1 vs crypto-bigint 64.0 | −50.9 (0.20×) | 11.2 vs crypto-bigint 33.3 | −22.1 (0.34×) | 18.0 vs crypto-bigint 84.2 | −66.2 (0.21×) | 15.1 vs crypto-bigint 42.4 | −27.3 (0.36×) |
| mul_u64, constant time | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.2 vs winter-math 13.6 | −4.4 (0.68×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 865 | −759 (0.12×) | 38.1 vs crypto-bigint 262 | −224 (0.15×) | 125 vs halo2curves 1,467 | −1,342 (0.09×) | 56.3 vs crypto-bigint 407 | −351 (0.14×) |
| mul_u64, constant time | secp256k1 base | 61.5 vs halo2curves 690 | −629 (0.09×) | 22.3 vs halo2curves 269 | −247 (0.08×) | 79.9 vs halo2curves 1,038 | −958 (0.08×) | 36.2 vs halo2curves 375 | −339 (0.10×) |
| mul_u64, constant time | P-256 base | 85.4 vs k256/p256 488 | −403 (0.18×) | 37.4 vs k256/p256 150 | −113 (0.25×) | 103 vs k256/p256 957 | −854 (0.11×) | 59.5 vs k256/p256 260 | −201 (0.23×) |
| mul_u64, constant time | P-256 scalar | 102 vs k256/p256 518 | −416 (0.20×) | 41.4 vs k256/p256 253 | −212 (0.16×) | 122 vs k256/p256 841 | −719 (0.15×) | 66.2 vs k256/p256 388 | −322 (0.17×) |
| mul_u64, constant time | Tom-256 base | 95.2 vs crypto-bigint 871 | −776 (0.11×) | 41.0 vs crypto-bigint 263 | −222 (0.16×) | 122 vs halo2curves 1,307 | −1,185 (0.09×) | 71.0 vs crypto-bigint 408 | −337 (0.17×) |
| mul_u64, constant time | BLS12-381 base | 149 vs crypto-bigint 1,940 | −1,791 (0.08×) | 62.6 vs halo2curves 473 | −410 (0.13×) | 287 vs crypto-bigint 2,584 | −2,297 (0.11×) | 118 vs crypto-bigint 1,036 | −918 (0.11×) |
| mul_u64, constant time | P-521 base | 138 vs crypto-bigint 3,025 | −2,887 (0.05×) | 64.3 vs crypto-bigint 991 | −927 (0.06×) | 192 vs crypto-bigint 3,959 | −3,767 (0.05×) | 90.0 vs crypto-bigint 1,826 | −1,736 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 6.7 vs crypto-bigint 60.1 | −53.4 (0.11×) | 6.7 vs crypto-bigint 30.4 | −23.7 (0.22×) | 7.6 vs crypto-bigint 76.2 | −68.6 (0.10×) | 7.7 vs crypto-bigint 39.2 | −31.5 (0.20×) |
| mul_uint, constant time | BabyBear | 13.0 vs crypto-bigint 64.1 | −51.1 (0.20×) | 11.1 vs crypto-bigint 33.0 | −21.9 (0.34×) | 18.0 vs crypto-bigint 82.8 | −64.8 (0.22×) | 15.0 vs crypto-bigint 39.8 | −24.8 (0.38×) |
| mul_uint, constant time | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.0 | −6.2 (0.80×) | 9.0 vs winter-math 13.5 | −4.5 (0.67×) |
| mul_uint, constant time | BN254 scalar | 486 vs halo2curves 760 | −274 (0.64×) | 167 vs halo2curves 181 | −14.0 (0.92×) | 592 vs halo2curves 1,376 | −784 (0.43×) | 310 vs halo2curves 350 | −40.0 (0.89×) |
| mul_uint, constant time | secp256k1 base | 322 vs halo2curves 520 | −198 (0.62×) | 81.1 vs halo2curves 153 | −71.9 (0.53×) | 379 vs halo2curves 970 | −591 (0.39×) | 125 vs halo2curves 266 | −141 (0.47×) |
| mul_uint, constant time | P-256 base | 413 vs halo2curves 455 | −42.0 (0.91×) | 131 vs halo2curves 144 | −13.0 (0.91×) | 493 vs halo2curves 928 | −435 (0.53×) | 227 vs halo2curves 252 | −25.0 (0.90×) |
| mul_uint, constant time | P-256 scalar | 473 vs k256/p256 554 | −81.0 (0.85×) | 181 vs halo2curves 186 | −5.0 (0.97×) | 548 vs k256/p256 870 | −322 (0.63×) | 380 vs crypto-bigint 409 | −29.0 (0.93×) |
| mul_uint, constant time | Tom-256 base | 465 vs halo2curves 693 | −228 (0.67×) | 154 vs halo2curves 180 | −26.0 (0.86×) | 554 vs halo2curves 1,183 | −629 (0.47×) | 329 vs halo2curves 398 | −69.0 (0.83×) |
| mul_uint, constant time | BLS12-381 base | 1,095 vs crypto-bigint 1,949 | −854 (0.56×) | 377 vs halo2curves 382 | −5.0 (0.99×) | 1,701 vs crypto-bigint 2,570 | −869 (0.66×) | 935 vs crypto-bigint 1,010 | −75.0 (0.93×) |
| mul_uint, constant time | P-521 base | 1,999 vs crypto-bigint 3,032 | −1,033 (0.66×) | 290 vs crypto-bigint 990 | −700 (0.29×) | 2,936 vs crypto-bigint 4,058 | −1,122 (0.72×) | 500 vs crypto-bigint 1,840 | −1,340 (0.27×) |
| add_u64, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.7 vs crypto-bigint 42.1 | −36.4 (0.14×) | 5.5 vs crypto-bigint 30.4 | −24.9 (0.18×) |
| add_u64, constant time | BabyBear | 10.3 vs crypto-bigint 39.9 | −29.6 (0.26×) | 7.4 vs crypto-bigint 31.7 | −24.3 (0.23×) | 14.8 vs crypto-bigint 53.8 | −39.0 (0.28×) | 11.6 vs crypto-bigint 32.1 | −20.5 (0.36×) |
| add_u64, constant time | Goldilocks | 9.3 vs winter-math 13.4 | −4.1 (0.69×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 10.0 vs winter-math 10.1 | −0.1 (0.99×) |
| add_u64, constant time | BN254 scalar | 93.5 vs crypto-bigint 502 | −409 (0.19×) | 42.0 vs halo2curves 209 | −167 (0.20×) | 111 vs halo2curves 757 | −646 (0.15×) | 81.9 vs halo2curves 294 | −212 (0.28×) |
| add_u64, constant time | secp256k1 base | 22.4 vs halo2curves 401 | −379 (0.06×) | 14.2 vs halo2curves 203 | −189 (0.07×) | 20.2 vs halo2curves 484 | −464 (0.04×) | 24.1 vs halo2curves 247 | −223 (0.10×) |
| add_u64, constant time | P-256 base | 71.0 vs k256/p256 277 | −206 (0.26×) | 36.0 vs k256/p256 96.7 | −60.7 (0.37×) | 78.9 vs k256/p256 517 | −438 (0.15×) | 58.0 vs k256/p256 159 | −101 (0.36×) |
| add_u64, constant time | P-256 scalar | 92.2 vs k256/p256 41.1 | +51.1 (2.24×) | 43.1 vs k256/p256 41.3 | +1.8 (1.04×) | 113 vs k256/p256 55.6 | +57.4 (2.03×) | 87.0 vs k256/p256 55.8 | +31.2 (1.56×) |
| add_u64, constant time | Tom-256 base | 94.4 vs crypto-bigint 502 | −408 (0.19×) | 40.9 vs halo2curves 220 | −179 (0.19×) | 111 vs halo2curves 710 | −599 (0.16×) | 77.6 vs halo2curves 288 | −210 (0.27×) |
| add_u64, constant time | BLS12-381 base | 147 vs crypto-bigint 1,082 | −935 (0.14×) | 69.4 vs halo2curves 324 | −255 (0.21×) | 170 vs crypto-bigint 1,423 | −1,253 (0.12×) | 112 vs crypto-bigint 651 | −539 (0.17×) |
| add_u64, constant time | P-521 base | 64.2 vs crypto-bigint 1,678 | −1,614 (0.04×) | 39.1 vs crypto-bigint 648 | −609 (0.06×) | 70.3 vs crypto-bigint 2,183 | −2,113 (0.03×) | 62.1 vs crypto-bigint 1,122 | −1,060 (0.06×) |
| add_uint, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.2 | −31.0 (0.14×) | 5.2 vs crypto-bigint 30.3 | −25.1 (0.17×) | 5.7 vs crypto-bigint 42.1 | −36.4 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) |
| add_uint, constant time | BabyBear | 10.4 vs crypto-bigint 39.9 | −29.5 (0.26×) | 7.4 vs crypto-bigint 31.6 | −24.2 (0.23×) | 14.8 vs crypto-bigint 53.7 | −38.9 (0.28×) | 12.1 vs crypto-bigint 32.3 | −20.2 (0.37×) |
| add_uint, constant time | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 9.9 vs winter-math 10.1 | −0.2 (0.98×) |
| add_uint, constant time | BN254 scalar | 249 vs halo2curves 412 | −163 (0.60×) | 93.3 vs halo2curves 113 | −19.7 (0.83×) | 303 vs halo2curves 695 | −392 (0.44×) | 186 vs halo2curves 205 | −19.0 (0.91×) |
| add_uint, constant time | secp256k1 base | 41.7 vs halo2curves 225 | −183 (0.19×) | 25.0 vs halo2curves 86.6 | −61.6 (0.29×) | 37.5 vs halo2curves 410 | −373 (0.09×) | 40.9 vs halo2curves 140 | −99.1 (0.29×) |
| add_uint, constant time | P-256 base | 185 vs halo2curves 246 | −61.0 (0.75×) | 96.2 vs halo2curves 91.9 | +4.3 (1.05×) | 265 vs halo2curves 518 | −253 (0.51×) | 149 vs halo2curves 153 | −4.0 (0.97×) |
| add_uint, constant time | P-256 scalar | 260 vs k256/p256 95.0 | +165 (2.74×) | 101 vs k256/p256 94.9 | +6.1 (1.06×) | 325 vs k256/p256 118 | +207 (2.75×) | 196 vs k256/p256 117 | +79.0 (1.68×) |
| add_uint, constant time | Tom-256 base | 251 vs halo2curves 379 | −128 (0.66×) | 94.8 vs halo2curves 111 | −16.2 (0.85×) | 333 vs halo2curves 636 | −303 (0.52×) | 173 vs halo2curves 196 | −23.0 (0.88×) |
| add_uint, constant time | BLS12-381 base | 769 vs halo2curves 984 | −215 (0.78×) | 204 vs halo2curves 224 | −20.0 (0.91×) | 961 vs crypto-bigint 1,422 | −461 (0.68×) | 426 vs halo2curves 494 | −68.0 (0.86×) |
| add_uint, constant time | P-521 base | 135 vs crypto-bigint 1,682 | −1,547 (0.08×) | 80.3 vs crypto-bigint 647 | −567 (0.12×) | 135 vs crypto-bigint 2,240 | −2,105 (0.06×) | 129 vs crypto-bigint 1,159 | −1,030 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.5 | −18.5 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.1 vs crypto-bigint 31.2 | −26.1 (0.16×) | 5.1 vs crypto-bigint 21.2 | −16.1 (0.24×) |
| sub_u64, constant time | BabyBear | 11.3 vs crypto-bigint 28.0 | −16.7 (0.40×) | 7.5 vs crypto-bigint 19.5 | −12.0 (0.38×) | 14.0 vs crypto-bigint 43.9 | −29.9 (0.32×) | 10.8 vs crypto-bigint 21.9 | −11.1 (0.49×) |
| sub_u64, constant time | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.6 vs winter-math 9.6 | +0.0 (1.00×) |
| sub_u64, constant time | BN254 scalar | 91.3 vs crypto-bigint 501 | −410 (0.18×) | 38.0 vs crypto-bigint 151 | −113 (0.25×) | 110 vs halo2curves 741 | −631 (0.15×) | 67.6 vs crypto-bigint 254 | −186 (0.27×) |
| sub_u64, constant time | secp256k1 base | 15.6 vs halo2curves 372 | −356 (0.04×) | 11.1 vs crypto-bigint 151 | −140 (0.07×) | 14.6 vs halo2curves 471 | −456 (0.03×) | 17.8 vs halo2curves 237 | −219 (0.08×) |
| sub_u64, constant time | P-256 base | 64.1 vs k256/p256 265 | −201 (0.24×) | 31.8 vs k256/p256 90.0 | −58.2 (0.35×) | 75.5 vs k256/p256 492 | −417 (0.15×) | 53.1 vs k256/p256 150 | −96.9 (0.35×) |
| sub_u64, constant time | P-256 scalar | 88.4 vs k256/p256 21.2 | +67.2 (4.17×) | 37.5 vs k256/p256 21.2 | +16.3 (1.77×) | 107 vs k256/p256 24.4 | +82.6 (4.39×) | 67.0 vs k256/p256 24.2 | +42.8 (2.77×) |
| sub_u64, constant time | Tom-256 base | 94.5 vs crypto-bigint 494 | −400 (0.19×) | 37.8 vs crypto-bigint 151 | −113 (0.25×) | 107 vs halo2curves 698 | −591 (0.15×) | 66.4 vs halo2curves 282 | −216 (0.24×) |
| sub_u64, constant time | BLS12-381 base | 143 vs crypto-bigint 1,080 | −937 (0.13×) | 60.1 vs halo2curves 316 | −256 (0.19×) | 185 vs crypto-bigint 1,404 | −1,219 (0.13×) | 117 vs crypto-bigint 602 | −485 (0.19×) |
| sub_u64, constant time | P-521 base | 42.4 vs crypto-bigint 1,667 | −1,625 (0.03×) | 27.7 vs crypto-bigint 621 | −593 (0.04×) | 52.0 vs crypto-bigint 2,167 | −2,115 (0.02×) | 40.5 vs crypto-bigint 1,105 | −1,065 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.6 | −18.6 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.2 vs crypto-bigint 21.8 | −16.6 (0.24×) |
| sub_uint, constant time | BabyBear | 11.3 vs crypto-bigint 28.0 | −16.7 (0.40×) | 7.5 vs crypto-bigint 19.6 | −12.1 (0.38×) | 14.0 vs crypto-bigint 43.9 | −29.9 (0.32×) | 11.4 vs crypto-bigint 21.8 | −10.4 (0.52×) |
| sub_uint, constant time | Goldilocks | 9.5 vs winter-math 11.5 | −2.0 (0.83×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.4 vs winter-math 9.5 | −0.1 (0.99×) |
| sub_uint, constant time | BN254 scalar | 248 vs halo2curves 401 | −153 (0.62×) | 92.3 vs halo2curves 104 | −11.7 (0.89×) | 380 vs halo2curves 650 | −270 (0.58×) | 176 vs halo2curves 193 | −17.0 (0.91×) |
| sub_uint, constant time | secp256k1 base | 32.8 vs halo2curves 212 | −179 (0.15×) | 21.5 vs halo2curves 104 | −82.5 (0.21×) | 33.8 vs halo2curves 408 | −374 (0.08×) | 35.7 vs halo2curves 137 | −101 (0.26×) |
| sub_uint, constant time | P-256 base | 171 vs halo2curves 232 | −61.0 (0.74×) | 89.6 vs halo2curves 85.6 | +4.0 (1.05×) | 297 vs halo2curves 496 | −199 (0.60×) | 140 vs halo2curves 143 | −3.0 (0.98×) |
| sub_uint, constant time | P-256 scalar | 250 vs k256/p256 70.3 | +180 (3.56×) | 101 vs k256/p256 65.9 | +35.1 (1.53×) | 362 vs k256/p256 88.9 | +273 (4.07×) | 202 vs k256/p256 90.7 | +111 (2.23×) |
| sub_uint, constant time | Tom-256 base | 253 vs halo2curves 366 | −113 (0.69×) | 96.2 vs halo2curves 104 | −7.8 (0.93×) | 323 vs halo2curves 621 | −298 (0.52×) | 186 vs halo2curves 192 | −6.0 (0.97×) |
| sub_uint, constant time | BLS12-381 base | 757 vs halo2curves 952 | −195 (0.80×) | 202 vs halo2curves 216 | −14.0 (0.94×) | 950 vs crypto-bigint 1,395 | −445 (0.68×) | 363 vs halo2curves 411 | −48.0 (0.88×) |
| sub_uint, constant time | P-521 base | 126 vs crypto-bigint 1,677 | −1,551 (0.08×) | 66.8 vs crypto-bigint 621 | −554 (0.11×) | 123 vs crypto-bigint 2,206 | −2,083 (0.06×) | 110 vs crypto-bigint 1,122 | −1,012 (0.10×) |
| from_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 18.0 | −13.3 (0.26×) | 4.7 vs crypto-bigint 11.6 | −6.9 (0.41×) | 5.2 vs crypto-bigint 25.5 | −20.3 (0.20×) | 5.2 vs crypto-bigint 13.7 | −8.5 (0.38×) |
| from_u64, constant time | BabyBear | 10.0 vs crypto-bigint 21.6 | −11.6 (0.46×) | 6.8 vs crypto-bigint 12.6 | −5.8 (0.54×) | 13.3 vs crypto-bigint 36.0 | −22.7 (0.37×) | 10.5 vs crypto-bigint 15.6 | −5.1 (0.67×) |
| from_u64, constant time | Goldilocks | 7.4 vs winter-math 10.6 | −3.2 (0.70×) | 5.9 vs winter-math 5.9 | +0.0 (1.00×) | 9.0 vs winter-math 17.0 | −8.0 (0.53×) | 7.1 vs winter-math 7.1 | +0.0 (1.00×) |
| from_u64, constant time | BN254 scalar | 83.8 vs crypto-bigint 430 | −346 (0.19×) | 34.7 vs crypto-bigint 133 | −98.3 (0.26×) | 99.3 vs halo2curves 709 | −610 (0.14×) | 50.4 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64, constant time | secp256k1 base | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.9 vs k256/p256 5.5 | −1.6 (0.71×) | 3.8 vs k256/p256 5.6 | −1.8 (0.68×) |
| from_u64, constant time | P-256 base | 51.3 vs k256/p256 239 | −188 (0.21×) | 26.7 vs k256/p256 76.2 | −49.5 (0.35×) | 61.4 vs k256/p256 480 | −419 (0.13×) | 38.7 vs k256/p256 130 | −91.3 (0.30×) |
| from_u64, constant time | P-256 scalar | 84.2 vs k256/p256 3.7 | +80.5 (22.76×) | 34.5 vs k256/p256 3.7 | +30.8 (9.32×) | 97.0 vs k256/p256 3.8 | +93.2 (25.53×) | 50.5 vs k256/p256 3.7 | +46.8 (13.65×) |
| from_u64, constant time | Tom-256 base | 82.0 vs crypto-bigint 428 | −346 (0.19×) | 35.4 vs crypto-bigint 133 | −97.6 (0.27×) | 92.5 vs halo2curves 663 | −571 (0.14×) | 49.2 vs crypto-bigint 205 | −156 (0.24×) |
| from_u64, constant time | BLS12-381 base | 145 vs crypto-bigint 910 | −765 (0.16×) | 56.6 vs crypto-bigint 252 | −195 (0.22×) | 173 vs crypto-bigint 1,264 | −1,091 (0.14×) | 95.5 vs crypto-bigint 565 | −470 (0.17×) |
| from_u64, constant time | P-521 base | 6.0 vs crypto-bigint 1,456 | −1,450 (0.00×) | 6.0 vs crypto-bigint 498 | −492 (0.01×) | 6.0 vs crypto-bigint 1,994 | −1,988 (0.00×) | 6.1 vs crypto-bigint 918 | −912 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,276 vs crypto-bigint 28,543 | −16,267 (0.43×) | 9,430 vs crypto-bigint 24,276 | −14,846 (0.39×) | 17,544 vs crypto-bigint 38,362 | −20,818 (0.46×) | 13,374 vs crypto-bigint 30,759 | −17,385 (0.43×) |
| div_u64, constant time | secp256k1 base | 11,959 vs crypto-bigint 28,665 | −16,706 (0.42×) | 9,110 vs crypto-bigint 24,244 | −15,134 (0.38×) | 17,272 vs crypto-bigint 38,175 | −20,903 (0.45×) | 12,802 vs crypto-bigint 31,771 | −18,969 (0.40×) |
| div_u64, constant time | P-256 base | 11,815 vs crypto-bigint 28,263 | −16,448 (0.42×) | 9,118 vs k256/p256 23,953 | −14,835 (0.38×) | 17,179 vs crypto-bigint 38,296 | −21,117 (0.45×) | 12,457 vs crypto-bigint 30,927 | −18,470 (0.40×) |
| div_u64, constant time | P-256 scalar | 12,280 vs crypto-bigint 28,533 | −16,253 (0.43×) | 9,404 vs crypto-bigint 24,371 | −14,967 (0.39×) | 17,658 vs crypto-bigint 38,241 | −20,583 (0.46×) | 13,396 vs crypto-bigint 30,841 | −17,445 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,086 vs crypto-bigint 28,734 | −16,648 (0.42×) | 9,318 vs crypto-bigint 24,278 | −14,960 (0.38×) | 16,929 vs crypto-bigint 38,582 | −21,653 (0.44×) | 12,962 vs crypto-bigint 31,084 | −18,122 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,775 vs crypto-bigint 51,677 | −26,902 (0.48×) | 16,918 vs crypto-bigint 43,345 | −26,427 (0.39×) | 34,672 vs crypto-bigint 69,011 | −34,339 (0.50×) | 24,767 vs crypto-bigint 55,533 | −30,766 (0.45×) |
| div_u64, constant time | P-521 base | 42,336 vs crypto-bigint 116,228 | −73,892 (0.36×) | 25,940 vs crypto-bigint 84,761 | −58,821 (0.31×) | 64,180 vs crypto-bigint 127,091 | −62,911 (0.50×) | 43,108 vs crypto-bigint 102,031 | −58,923 (0.42×) |
| div_uint, constant time | Mersenne-31 | 272 vs runtime-algebra 3,501 | −3,229 (0.08×) | 272 vs runtime-algebra 3,512 | −3,240 (0.08×) | 281 vs runtime-algebra 3,447 | −3,166 (0.08×) | 280 vs runtime-algebra 3,437 | −3,157 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,555 | −3,086 (0.13×) | 465 vs runtime-algebra 3,564 | −3,099 (0.13×) | 473 vs runtime-algebra 3,446 | −2,973 (0.14×) | 445 vs runtime-algebra 3,419 | −2,974 (0.13×) |
| div_uint, constant time | Goldilocks | 1,145 vs winter-math 1,836 | −691 (0.62×) | 862 vs winter-math 933 | −71.0 (0.92×) | 1,438 vs winter-math 1,887 | −449 (0.76×) | 960 vs winter-math 1,055 | −95.0 (0.91×) |
| div_uint, constant time | BN254 scalar | 12,518 vs crypto-bigint 28,743 | −16,225 (0.44×) | 9,819 vs crypto-bigint 24,215 | −14,396 (0.41×) | 17,747 vs crypto-bigint 38,434 | −20,687 (0.46×) | 13,379 vs crypto-bigint 31,024 | −17,645 (0.43×) |
| div_uint, constant time | secp256k1 base | 12,005 vs crypto-bigint 28,471 | −16,466 (0.42×) | 9,096 vs crypto-bigint 24,226 | −15,130 (0.38×) | 17,159 vs crypto-bigint 38,664 | −21,505 (0.44×) | 12,836 vs crypto-bigint 31,243 | −18,407 (0.41×) |
| div_uint, constant time | P-256 base | 12,045 vs crypto-bigint 29,289 | −17,244 (0.41×) | 9,092 vs k256/p256 24,168 | −15,076 (0.38×) | 17,578 vs crypto-bigint 38,162 | −20,584 (0.46×) | 12,614 vs crypto-bigint 30,443 | −17,829 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,316 vs crypto-bigint 28,533 | −16,217 (0.43×) | 9,611 vs crypto-bigint 24,285 | −14,674 (0.40×) | 17,822 vs crypto-bigint 38,203 | −20,381 (0.47×) | 13,368 vs crypto-bigint 31,468 | −18,100 (0.42×) |
| div_uint, constant time | Tom-256 base | 12,498 vs crypto-bigint 28,557 | −16,059 (0.44×) | 9,455 vs crypto-bigint 24,141 | −14,686 (0.39×) | 16,958 vs crypto-bigint 38,552 | −21,594 (0.44×) | 13,197 vs crypto-bigint 30,851 | −17,654 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,340 vs crypto-bigint 51,879 | −26,539 (0.49×) | 16,962 vs crypto-bigint 43,428 | −26,466 (0.39×) | 35,456 vs crypto-bigint 69,796 | −34,340 (0.51×) | 24,761 vs crypto-bigint 58,175 | −33,414 (0.43×) |
| div_uint, constant time | P-521 base | 42,595 vs crypto-bigint 110,979 | −68,384 (0.38×) | 26,114 vs crypto-bigint 86,425 | −60,311 (0.30×) | 65,040 vs crypto-bigint 127,819 | −62,779 (0.51×) | 43,253 vs crypto-bigint 103,688 | −60,435 (0.42×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 18.2 | −15.2 (0.16×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add | BN254 scalar | 13.2 vs ark-ff 13.3 | −0.1 (0.99×) | 10.9 vs ark-ff 11.3 | −0.4 (0.96×) | 13.7 vs ark-ff 13.7 | +0.0 (1.00×) | 17.4 vs ark-ff 16.4 | +1.0 (1.06×) |
| add | secp256k1 base | 13.8 vs ark-ff 15.5 | −1.7 (0.89×) | 10.8 vs ark-ff 11.3 | −0.5 (0.96×) | 13.9 vs ark-ff 15.1 | −1.2 (0.92×) | 18.3 vs ark-ff 17.9 | +0.4 (1.02×) |
| add | P-256 base | 12.3 vs ark-ff 15.1 | −2.8 (0.81×) | 11.2 vs ark-ff 11.4 | −0.2 (0.98×) | 13.0 vs ark-ff 15.6 | −2.6 (0.83×) | 18.7 vs ark-ff 19.3 | −0.6 (0.97×) |
| add | P-256 scalar | 13.6 vs ark-ff 15.2 | −1.6 (0.89×) | 11.1 vs ark-ff 11.7 | −0.6 (0.95×) | 14.0 vs ark-ff 15.6 | −1.6 (0.90×) | 19.4 vs ark-ff 17.9 | +1.5 (1.08×) |
| add | Tom-256 base | 13.3 vs halo2curves 43.0 | −29.7 (0.31×) | 11.3 vs halo2curves 24.5 | −13.2 (0.46×) | 13.2 vs halo2curves 43.4 | −30.2 (0.30×) | 18.6 vs halo2curves 33.0 | −14.4 (0.56×) |
| add | BLS12-381 base | 21.9 vs ark-ff 44.0 | −22.1 (0.50×) | 18.9 vs halo2curves 34.5 | −15.6 (0.55×) | 23.0 vs ark-ff 47.2 | −24.2 (0.49×) | 30.1 vs ark-ff 46.1 | −16.0 (0.65×) |
| add | P-521 base | 43.3 vs crypto-bigint 180 | −137 (0.24×) | 27.4 vs crypto-bigint 181 | −154 (0.15×) | 48.9 vs crypto-bigint 246 | −197 (0.20×) | 48.0 vs crypto-bigint 246 | −198 (0.20×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 13.1 vs ark-ff 16.4 | −3.3 (0.80×) | 10.7 vs ark-ff 13.9 | −3.2 (0.77×) | 12.7 vs ark-ff 16.9 | −4.2 (0.75×) | 17.4 vs ark-ff 21.9 | −4.5 (0.79×) |
| sub | secp256k1 base | 16.6 vs ark-ff 16.6 | +0.0 (1.00×) | 10.4 vs ark-ff 14.1 | −3.7 (0.74×) | 13.5 vs ark-ff 17.2 | −3.7 (0.78×) | 16.9 vs halo2curves 21.0 | −4.1 (0.80×) |
| sub | P-256 base | 13.3 vs ark-ff 16.4 | −3.1 (0.81×) | 10.4 vs ark-ff 13.0 | −2.6 (0.80×) | 12.5 vs ark-ff 15.2 | −2.7 (0.82×) | 16.5 vs k256/p256 20.0 | −3.5 (0.82×) |
| sub | P-256 scalar | 14.3 vs ark-ff 16.5 | −2.2 (0.87×) | 10.4 vs ark-ff 14.7 | −4.3 (0.71×) | 13.6 vs ark-ff 16.6 | −3.0 (0.82×) | 16.8 vs ark-ff 22.0 | −5.2 (0.76×) |
| sub | Tom-256 base | 13.1 vs halo2curves 26.0 | −12.9 (0.50×) | 10.7 vs halo2curves 14.8 | −4.1 (0.72×) | 12.8 vs halo2curves 26.5 | −13.7 (0.48×) | 17.4 vs halo2curves 22.1 | −4.7 (0.79×) |
| sub | BLS12-381 base | 22.8 vs ark-ff 30.7 | −7.9 (0.74×) | 17.3 vs ark-ff 25.1 | −7.8 (0.69×) | 22.3 vs ark-ff 30.2 | −7.9 (0.74×) | 29.6 vs halo2curves 36.2 | −6.6 (0.82×) |
| sub | P-521 base | 43.6 vs crypto-bigint 159 | −115 (0.27×) | 25.6 vs crypto-bigint 144 | −118 (0.18×) | 44.2 vs crypto-bigint 195 | −151 (0.23×) | 44.1 vs crypto-bigint 175 | −131 (0.25×) |
| mul | Mersenne-31 | 7.0 vs crypto-bigint 46.8 | −39.8 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.9 | −43.9 (0.14×) | 7.0 vs crypto-bigint 31.0 | −24.0 (0.23×) |
| mul | BabyBear | 12.0 vs plonky3 12.2 | −0.2 (0.98×) | 11.0 vs plonky3 12.2 | −1.2 (0.90×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| mul | Goldilocks | 16.5 vs winter-math 18.0 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 12.5 vs winter-math 14.1 | −1.6 (0.89×) |
| mul | BN254 scalar | 229 vs ark-ff 320 | −91.0 (0.72×) | 91.6 vs halo2curves 95.2 | −3.6 (0.96×) | 366 vs ark-ff 391 | −25.0 (0.94×) | 159 vs halo2curves 178 | −19.0 (0.89×) |
| mul | secp256k1 base | 173 vs ark-ff 277 | −104 (0.62×) | 68.9 vs halo2curves 90.7 | −21.8 (0.76×) | 230 vs ark-ff 399 | −169 (0.58×) | 106 vs halo2curves 160 | −54.0 (0.66×) |
| mul | P-256 base | 191 vs k256/p256 241 | −50.0 (0.79×) | 65.2 vs halo2curves 75.6 | −10.4 (0.86×) | 291 vs ark-ff 431 | −140 (0.68×) | 119 vs k256/p256 131 | −12.0 (0.91×) |
| mul | P-256 scalar | 240 vs ark-ff 307 | −67.0 (0.78×) | 88.5 vs halo2curves 95.9 | −7.4 (0.92×) | 331 vs ark-ff 405 | −74.0 (0.82×) | 159 vs halo2curves 176 | −17.0 (0.90×) |
| mul | Tom-256 base | 238 vs halo2curves 345 | −107 (0.69×) | 82.6 vs halo2curves 93.9 | −11.3 (0.88×) | 300 vs halo2curves 598 | −298 (0.50×) | 152 vs halo2curves 172 | −20.0 (0.88×) |
| mul | BLS12-381 base | 721 vs ark-ff 691 | +30.0 (1.04×) | 185 vs halo2curves 195 | −10.0 (0.95×) | 919 vs ark-ff 1,048 | −129 (0.88×) | 314 vs halo2curves 378 | −64.0 (0.83×) |
| mul | P-521 base | 687 vs crypto-bigint 2,146 | −1,459 (0.32×) | 240 vs crypto-bigint 619 | −379 (0.39×) | 1,324 vs crypto-bigint 2,791 | −1,467 (0.47×) | 438 vs crypto-bigint 1,109 | −671 (0.39×) |
| square | Mersenne-31 | 7.0 vs crypto-bigint 45.2 | −38.2 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 49.0 | −42.0 (0.14×) | 7.0 vs crypto-bigint 31.2 | −24.2 (0.22×) |
| square | BabyBear | 12.0 vs plonky3 12.0 | +0.0 (1.00×) | 11.0 vs plonky3 12.0 | −1.0 (0.92×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| square | Goldilocks | 15.0 vs winter-math 20.0 | −5.0 (0.75×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.7 vs winter-math 19.0 | −0.3 (0.98×) | 12.5 vs winter-math 14.4 | −1.9 (0.87×) |
| square | BN254 scalar | 211 vs ark-ff 263 | −52.0 (0.80×) | 82.4 vs halo2curves 97.0 | −14.6 (0.85×) | 325 vs ark-ff 348 | −23.0 (0.93×) | 153 vs halo2curves 149 | +4.0 (1.03×) |
| square | secp256k1 base | 151 vs halo2curves 222 | −71.0 (0.68×) | 58.8 vs halo2curves 84.3 | −25.5 (0.70×) | 181 vs ark-ff 297 | −116 (0.61×) | 88.5 vs halo2curves 118 | −29.5 (0.75×) |
| square | P-256 base | 159 vs halo2curves 180 | −21.0 (0.88×) | 55.6 vs halo2curves 61.8 | −6.2 (0.90×) | 212 vs ark-ff 330 | −118 (0.64×) | 99.9 vs halo2curves 108 | −8.1 (0.93×) |
| square | P-256 scalar | 218 vs ark-ff 258 | −40.0 (0.84×) | 81.4 vs halo2curves 92.5 | −11.1 (0.88×) | 230 vs ark-ff 345 | −115 (0.67×) | 145 vs halo2curves 143 | +2.0 (1.01×) |
| square | Tom-256 base | 213 vs halo2curves 268 | −55.0 (0.79×) | 79.3 vs halo2curves 88.4 | −9.1 (0.90×) | 212 vs halo2curves 488 | −276 (0.43×) | 136 vs halo2curves 135 | +1.0 (1.01×) |
| square | BLS12-381 base | 568 vs ark-ff 621 | −53.0 (0.91×) | 164 vs halo2curves 161 | +3.0 (1.02×) | 808 vs ark-ff 819 | −11.0 (0.99×) | 325 vs halo2curves 296 | +29.0 (1.10×) |
| square | P-521 base | 570 vs crypto-bigint 2,174 | −1,604 (0.26×) | 210 vs crypto-bigint 625 | −415 (0.34×) | 916 vs crypto-bigint 2,858 | −1,942 (0.32×) | 390 vs crypto-bigint 1,140 | −750 (0.34×) |
| mul_u64 | Mersenne-31 | 9.0 vs crypto-bigint 60.1 | −51.1 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 75.1 | −66.1 (0.12×) | 9.0 vs crypto-bigint 38.3 | −29.3 (0.23×) |
| mul_u64 | BabyBear | 16.1 vs plonky3 34.2 | −18.1 (0.47×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.4 vs plonky3 45.0 | −29.6 (0.34×) | 13.0 vs plonky3 17.2 | −4.2 (0.76×) |
| mul_u64 | Goldilocks | 17.8 vs winter-math 23.5 | −5.7 (0.76×) | 14.1 vs winter-math 13.9 | +0.2 (1.01×) | 25.3 vs winter-math 34.5 | −9.2 (0.73×) | 15.0 vs winter-math 14.6 | +0.4 (1.03×) |
| mul_u64 | BN254 scalar | 91.8 vs ark-ff 655 | −563 (0.14×) | 34.5 vs crypto-bigint 262 | −228 (0.13×) | 121 vs ark-ff 823 | −702 (0.15×) | 45.1 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | secp256k1 base | 49.0 vs ark-ff 432 | −383 (0.11×) | 16.0 vs crypto-bigint 259 | −243 (0.06×) | 65.1 vs ark-ff 604 | −539 (0.11×) | 23.1 vs halo2curves 387 | −364 (0.06×) |
| mul_u64 | P-256 base | 75.2 vs k256/p256 487 | −412 (0.15×) | 33.0 vs k256/p256 154 | −121 (0.21×) | 90.1 vs ark-ff 839 | −749 (0.11×) | 42.0 vs k256/p256 258 | −216 (0.16×) |
| mul_u64 | P-256 scalar | 81.8 vs k256/p256 518 | −436 (0.16×) | 34.5 vs k256/p256 255 | −221 (0.14×) | 112 vs k256/p256 810 | −698 (0.14×) | 48.2 vs k256/p256 338 | −290 (0.14×) |
| mul_u64 | Tom-256 base | 84.2 vs crypto-bigint 863 | −779 (0.10×) | 32.9 vs crypto-bigint 259 | −226 (0.13×) | 111 vs halo2curves 1,315 | −1,204 (0.08×) | 45.4 vs crypto-bigint 411 | −366 (0.11×) |
| mul_u64 | BLS12-381 base | 124 vs ark-ff 1,636 | −1,512 (0.08×) | 47.9 vs halo2curves 489 | −441 (0.10×) | 196 vs ark-ff 2,129 | −1,933 (0.09×) | 61.7 vs crypto-bigint 1,045 | −983 (0.06×) |
| mul_u64 | P-521 base | 94.9 vs crypto-bigint 3,070 | −2,975 (0.03×) | 39.6 vs crypto-bigint 996 | −956 (0.04×) | 185 vs crypto-bigint 4,093 | −3,908 (0.05×) | 53.1 vs crypto-bigint 1,846 | −1,793 (0.03×) |
| mul_uint | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 32.5 | −23.5 (0.28×) | 9.0 vs crypto-bigint 75.2 | −66.2 (0.12×) | 9.0 vs crypto-bigint 38.8 | −29.8 (0.23×) |
| mul_uint | BabyBear | 16.1 vs plonky3 23.6 | −7.5 (0.68×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.7 vs plonky3 34.1 | −18.4 (0.46×) | 12.8 vs plonky3 16.7 | −3.9 (0.77×) |
| mul_uint | Goldilocks | 17.8 vs winter-math 23.4 | −5.6 (0.76×) | 14.1 vs winter-math 13.7 | +0.4 (1.03×) | 25.3 vs winter-math 34.5 | −9.2 (0.73×) | 15.0 vs winter-math 14.6 | +0.4 (1.03×) |
| mul_uint | BN254 scalar | 440 vs halo2curves 758 | −318 (0.58×) | 157 vs halo2curves 178 | −21.0 (0.88×) | 538 vs halo2curves 1,318 | −780 (0.41×) | 346 vs halo2curves 353 | −7.0 (0.98×) |
| mul_uint | secp256k1 base | 265 vs halo2curves 517 | −252 (0.51×) | 68.9 vs halo2curves 154 | −85.1 (0.45×) | 319 vs halo2curves 962 | −643 (0.33×) | 107 vs halo2curves 266 | −159 (0.40×) |
| mul_uint | P-256 base | 354 vs halo2curves 452 | −98.0 (0.78×) | 130 vs halo2curves 146 | −16.0 (0.89×) | 429 vs halo2curves 978 | −549 (0.44×) | 230 vs halo2curves 257 | −27.0 (0.89×) |
| mul_uint | P-256 scalar | 424 vs k256/p256 553 | −129 (0.77×) | 176 vs halo2curves 186 | −10.0 (0.95×) | 475 vs k256/p256 877 | −402 (0.54×) | 333 vs halo2curves 347 | −14.0 (0.96×) |
| mul_uint | Tom-256 base | 409 vs halo2curves 694 | −285 (0.59×) | 160 vs halo2curves 179 | −19.0 (0.89×) | 490 vs halo2curves 1,178 | −688 (0.42×) | 341 vs halo2curves 338 | +3.0 (1.01×) |
| mul_uint | BLS12-381 base | 970 vs halo2curves 1,898 | −928 (0.51×) | 353 vs halo2curves 380 | −27.0 (0.93×) | 1,137 vs crypto-bigint 2,585 | −1,448 (0.44×) | 943 vs crypto-bigint 1,044 | −101 (0.90×) |
| mul_uint | P-521 base | 1,562 vs crypto-bigint 3,062 | −1,500 (0.51×) | 248 vs crypto-bigint 1,009 | −761 (0.25×) | 2,388 vs crypto-bigint 4,011 | −1,623 (0.60×) | 456 vs crypto-bigint 1,892 | −1,436 (0.24×) |
| add_u64 | Mersenne-31 | 4.6 vs crypto-bigint 35.4 | −30.8 (0.13×) | 4.7 vs crypto-bigint 30.1 | −25.4 (0.16×) | 4.8 vs crypto-bigint 43.4 | −38.6 (0.11×) | 5.0 vs crypto-bigint 31.0 | −26.0 (0.16×) |
| add_u64 | BabyBear | 12.3 vs plonky3 30.6 | −18.3 (0.40×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 41.7 | −29.0 (0.30×) | 10.9 vs plonky3 13.1 | −2.2 (0.83×) |
| add_u64 | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.5 | −1.0 (0.87×) | 12.0 vs winter-math 19.0 | −7.0 (0.63×) | 9.3 vs winter-math 9.8 | −0.5 (0.95×) |
| add_u64 | BN254 scalar | 113 vs ark-ff 340 | −227 (0.33×) | 44.6 vs halo2curves 213 | −168 (0.21×) | 139 vs ark-ff 408 | −269 (0.34×) | 65.6 vs halo2curves 299 | −233 (0.22×) |
| add_u64 | secp256k1 base | 7.0 vs ark-ff 157 | −150 (0.04×) | 6.0 vs ark-ff 120 | −114 (0.05×) | 8.0 vs ark-ff 222 | −214 (0.04×) | 10.5 vs ark-ff 183 | −173 (0.06×) |
| add_u64 | P-256 base | 67.7 vs ark-ff 261 | −193 (0.26×) | 34.9 vs k256/p256 92.6 | −57.7 (0.38×) | 77.7 vs ark-ff 401 | −323 (0.19×) | 56.1 vs k256/p256 158 | −102 (0.36×) |
| add_u64 | P-256 scalar | 66.7 vs k256/p256 56.3 | +10.4 (1.18×) | 38.2 vs k256/p256 56.1 | −17.9 (0.68×) | 74.7 vs k256/p256 62.2 | +12.5 (1.20×) | 66.3 vs k256/p256 62.5 | +3.8 (1.06×) |
| add_u64 | Tom-256 base | 84.3 vs crypto-bigint 511 | −427 (0.16×) | 39.9 vs halo2curves 217 | −177 (0.18×) | 94.4 vs halo2curves 706 | −612 (0.13×) | 65.0 vs halo2curves 287 | −222 (0.23×) |
| add_u64 | BLS12-381 base | 161 vs ark-ff 829 | −668 (0.19×) | 62.2 vs halo2curves 323 | −261 (0.19×) | 249 vs ark-ff 1,079 | −830 (0.23×) | 89.7 vs crypto-bigint 657 | −567 (0.14×) |
| add_u64 | P-521 base | 21.4 vs crypto-bigint 1,677 | −1,656 (0.01×) | 12.3 vs crypto-bigint 651 | −639 (0.02×) | 19.5 vs crypto-bigint 2,257 | −2,238 (0.01×) | 30.2 vs crypto-bigint 1,144 | −1,114 (0.03×) |
| add_uint | Mersenne-31 | 4.6 vs crypto-bigint 35.4 | −30.8 (0.13×) | 4.7 vs crypto-bigint 30.1 | −25.4 (0.16×) | 4.8 vs crypto-bigint 43.5 | −38.7 (0.11×) | 4.8 vs crypto-bigint 30.0 | −25.2 (0.16×) |
| add_uint | BabyBear | 12.2 vs plonky3 21.5 | −9.3 (0.57×) | 8.2 vs plonky3 9.3 | −1.1 (0.88×) | 12.7 vs plonky3 31.3 | −18.6 (0.41×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_uint | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 9.1 vs winter-math 9.7 | −0.6 (0.94×) |
| add_uint | BN254 scalar | 225 vs halo2curves 417 | −192 (0.54×) | 99.6 vs halo2curves 112 | −12.4 (0.89×) | 374 vs halo2curves 679 | −305 (0.55×) | 176 vs halo2curves 206 | −30.0 (0.85×) |
| add_uint | secp256k1 base | 17.3 vs halo2curves 225 | −208 (0.08×) | 12.6 vs halo2curves 83.5 | −70.9 (0.15×) | 16.9 vs halo2curves 419 | −402 (0.04×) | 23.0 vs halo2curves 141 | −118 (0.16×) |
| add_uint | P-256 base | 180 vs halo2curves 249 | −69.0 (0.72×) | 73.4 vs halo2curves 91.8 | −18.4 (0.80×) | 326 vs halo2curves 514 | −188 (0.63×) | 148 vs halo2curves 154 | −6.0 (0.96×) |
| add_uint | P-256 scalar | 260 vs k256/p256 77.1 | +183 (3.37×) | 99.1 vs k256/p256 82.1 | +17.0 (1.21×) | 306 vs k256/p256 107 | +199 (2.86×) | 200 vs k256/p256 115 | +85.0 (1.74×) |
| add_uint | Tom-256 base | 253 vs halo2curves 373 | −120 (0.68×) | 96.1 vs halo2curves 110 | −13.9 (0.87×) | 329 vs halo2curves 636 | −307 (0.52×) | 177 vs halo2curves 197 | −20.0 (0.90×) |
| add_uint | BLS12-381 base | 765 vs halo2curves 955 | −190 (0.80×) | 196 vs halo2curves 223 | −27.0 (0.88×) | 953 vs crypto-bigint 1,415 | −462 (0.67×) | 389 vs halo2curves 443 | −54.0 (0.88×) |
| add_uint | P-521 base | 61.8 vs crypto-bigint 1,689 | −1,627 (0.04×) | 41.5 vs crypto-bigint 666 | −625 (0.06×) | 85.8 vs crypto-bigint 2,215 | −2,129 (0.04×) | 66.5 vs crypto-bigint 1,197 | −1,131 (0.06×) |
| sub_u64 | Mersenne-31 | 4.7 vs crypto-bigint 22.2 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.3 | −25.4 (0.16×) | 4.8 vs crypto-bigint 20.3 | −15.5 (0.24×) |
| sub_u64 | BabyBear | 12.0 vs crypto-bigint 28.8 | −16.8 (0.42×) | 8.2 vs plonky3 9.3 | −1.1 (0.88×) | 12.1 vs plonky3 41.8 | −29.7 (0.29×) | 10.9 vs plonky3 13.6 | −2.7 (0.80×) |
| sub_u64 | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 8.7 vs winter-math 8.6 | +0.1 (1.01×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 347 | −234 (0.33×) | 45.8 vs crypto-bigint 152 | −106 (0.30×) | 136 vs ark-ff 410 | −274 (0.33×) | 66.1 vs crypto-bigint 288 | −222 (0.23×) |
| sub_u64 | secp256k1 base | 7.0 vs ark-ff 156 | −149 (0.04×) | 6.0 vs ark-ff 123 | −117 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 9.1 vs ark-ff 190 | −181 (0.05×) |
| sub_u64 | P-256 base | 69.5 vs ark-ff 264 | −195 (0.26×) | 34.0 vs k256/p256 88.0 | −54.0 (0.39×) | 79.0 vs ark-ff 393 | −314 (0.20×) | 57.2 vs k256/p256 150 | −92.8 (0.38×) |
| sub_u64 | P-256 scalar | 66.9 vs k256/p256 21.2 | +45.7 (3.16×) | 36.7 vs k256/p256 21.3 | +15.4 (1.72×) | 72.5 vs k256/p256 24.7 | +47.8 (2.94×) | 61.9 vs k256/p256 24.7 | +37.2 (2.51×) |
| sub_u64 | Tom-256 base | 85.2 vs crypto-bigint 496 | −411 (0.17×) | 39.6 vs crypto-bigint 152 | −112 (0.26×) | 97.3 vs halo2curves 694 | −597 (0.14×) | 66.1 vs halo2curves 278 | −212 (0.24×) |
| sub_u64 | BLS12-381 base | 151 vs ark-ff 824 | −673 (0.18×) | 60.3 vs halo2curves 315 | −255 (0.19×) | 265 vs ark-ff 1,053 | −788 (0.25×) | 90.5 vs crypto-bigint 645 | −555 (0.14×) |
| sub_u64 | P-521 base | 21.1 vs crypto-bigint 1,664 | −1,643 (0.01×) | 13.1 vs crypto-bigint 630 | −617 (0.02×) | 21.4 vs crypto-bigint 2,233 | −2,212 (0.01×) | 20.9 vs crypto-bigint 1,139 | −1,118 (0.02×) |
| sub_uint | Mersenne-31 | 4.6 vs crypto-bigint 22.1 | −17.5 (0.21×) | 4.7 vs crypto-bigint 18.3 | −13.6 (0.26×) | 4.9 vs crypto-bigint 30.5 | −25.6 (0.16×) | 4.9 vs crypto-bigint 20.4 | −15.5 (0.24×) |
| sub_uint | BabyBear | 12.0 vs plonky3 22.0 | −10.0 (0.55×) | 8.2 vs plonky3 9.1 | −0.9 (0.90×) | 12.1 vs plonky3 31.2 | −19.1 (0.39×) | 11.3 vs plonky3 13.8 | −2.5 (0.82×) |
| sub_uint | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 9.0 vs winter-math 8.3 | +0.7 (1.08×) |
| sub_uint | BN254 scalar | 232 vs halo2curves 401 | −169 (0.58×) | 95.7 vs halo2curves 106 | −10.3 (0.90×) | 353 vs halo2curves 660 | −307 (0.53×) | 181 vs halo2curves 195 | −14.0 (0.93×) |
| sub_uint | secp256k1 base | 18.9 vs halo2curves 206 | −187 (0.09×) | 12.0 vs halo2curves 80.7 | −68.7 (0.15×) | 16.4 vs halo2curves 408 | −392 (0.04×) | 21.3 vs halo2curves 130 | −109 (0.16×) |
| sub_uint | P-256 base | 175 vs halo2curves 235 | −60.0 (0.74×) | 70.3 vs halo2curves 85.8 | −15.5 (0.82×) | 248 vs halo2curves 502 | −254 (0.49×) | 136 vs halo2curves 144 | −8.0 (0.94×) |
| sub_uint | P-256 scalar | 253 vs k256/p256 72.0 | +181 (3.51×) | 98.8 vs k256/p256 77.7 | +21.1 (1.27×) | 329 vs k256/p256 94.9 | +234 (3.47×) | 192 vs k256/p256 98.4 | +93.6 (1.95×) |
| sub_uint | Tom-256 base | 250 vs halo2curves 366 | −116 (0.68×) | 93.0 vs halo2curves 103 | −10.0 (0.90×) | 328 vs halo2curves 608 | −280 (0.54×) | 165 vs halo2curves 188 | −23.0 (0.88×) |
| sub_uint | BLS12-381 base | 761 vs halo2curves 959 | −198 (0.79×) | 196 vs halo2curves 215 | −19.0 (0.91×) | 934 vs crypto-bigint 1,413 | −479 (0.66×) | 384 vs halo2curves 509 | −125 (0.75×) |
| sub_uint | P-521 base | 67.4 vs crypto-bigint 1,688 | −1,621 (0.04×) | 37.6 vs crypto-bigint 639 | −601 (0.06×) | 76.6 vs crypto-bigint 2,196 | −2,119 (0.03×) | 65.1 vs crypto-bigint 1,135 | −1,070 (0.06×) |
| div_u64 | BN254 scalar | 1,203 vs halo2curves 13,702 | −12,499 (0.09×) | 839 vs halo2curves 6,988 | −6,149 (0.12×) | 1,369 vs halo2curves 19,363 | −17,994 (0.07×) | 1,117 vs halo2curves 11,884 | −10,767 (0.09×) |
| div_u64 | secp256k1 base | 1,013 vs halo2curves 13,043 | −12,030 (0.08×) | 715 vs halo2curves 6,962 | −6,247 (0.10×) | 1,230 vs halo2curves 18,809 | −17,579 (0.07×) | 806 vs halo2curves 11,733 | −10,927 (0.07×) |
| div_u64 | P-256 base | 1,091 vs halo2curves 13,158 | −12,067 (0.08×) | 800 vs halo2curves 6,938 | −6,138 (0.12×) | 1,343 vs halo2curves 18,859 | −17,516 (0.07×) | 977 vs halo2curves 11,507 | −10,530 (0.08×) |
| div_u64 | P-256 scalar | 1,184 vs halo2curves 13,527 | −12,343 (0.09×) | 842 vs halo2curves 6,956 | −6,114 (0.12×) | 1,418 vs halo2curves 19,386 | −17,968 (0.07×) | 1,127 vs halo2curves 11,787 | −10,660 (0.10×) |
| div_u64 | Tom-256 base | 1,199 vs halo2curves 13,319 | −12,120 (0.09×) | 843 vs halo2curves 6,967 | −6,124 (0.12×) | 1,492 vs halo2curves 19,173 | −17,681 (0.08×) | 1,104 vs halo2curves 11,719 | −10,615 (0.09×) |
| div_u64 | BLS12-381 base | 1,790 vs halo2curves 24,128 | −22,338 (0.07×) | 1,075 vs halo2curves 11,063 | −9,988 (0.10×) | 2,466 vs halo2curves 33,179 | −30,713 (0.07×) | 1,799 vs halo2curves 18,796 | −16,997 (0.10×) |
| div_u64 | P-521 base | 2,453 vs crypto-bigint 63,346 | −60,893 (0.04×) | 998 vs crypto-bigint 46,270 | −45,272 (0.02×) | 3,622 vs crypto-bigint 78,770 | −75,148 (0.05×) | 1,518 vs crypto-bigint 57,638 | −56,120 (0.03×) |
| div_uint | Mersenne-31 | 273 vs crypto-bigint 1,827 | −1,554 (0.15×) | 275 vs crypto-bigint 1,611 | −1,336 (0.17×) | 278 vs runtime-algebra 2,103 | −1,825 (0.13×) | 282 vs crypto-bigint 2,025 | −1,743 (0.14×) |
| div_uint | BabyBear | 275 vs plonky3 675 | −400 (0.41×) | 278 vs plonky3 651 | −373 (0.43×) | 277 vs plonky3 738 | −461 (0.38×) | 280 vs plonky3 713 | −433 (0.39×) |
| div_uint | Goldilocks | 550 vs winter-math 1,817 | −1,267 (0.30×) | 538 vs winter-math 928 | −390 (0.58×) | 584 vs winter-math 1,898 | −1,314 (0.31×) | 569 vs winter-math 1,031 | −462 (0.55×) |
| div_uint | BN254 scalar | 9,073 vs halo2curves 15,068 | −5,995 (0.60×) | 6,605 vs halo2curves 8,170 | −1,565 (0.81×) | 12,557 vs halo2curves 22,692 | −10,135 (0.55×) | 8,481 vs halo2curves 14,586 | −6,105 (0.58×) |
| div_uint | secp256k1 base | 8,598 vs halo2curves 14,878 | −6,280 (0.58×) | 6,099 vs halo2curves 8,220 | −2,121 (0.74×) | 12,189 vs halo2curves 22,573 | −10,384 (0.54×) | 8,095 vs halo2curves 14,530 | −6,435 (0.56×) |
| div_uint | P-256 base | 8,725 vs halo2curves 14,858 | −6,133 (0.59×) | 6,170 vs halo2curves 8,177 | −2,007 (0.75×) | 12,069 vs halo2curves 22,538 | −10,469 (0.54×) | 7,902 vs halo2curves 14,498 | −6,596 (0.55×) |
| div_uint | P-256 scalar | 9,144 vs halo2curves 15,106 | −5,962 (0.61×) | 6,556 vs halo2curves 8,244 | −1,688 (0.80×) | 12,449 vs halo2curves 23,261 | −10,812 (0.54×) | 8,667 vs halo2curves 14,548 | −5,881 (0.60×) |
| div_uint | Tom-256 base | 8,902 vs halo2curves 14,993 | −6,091 (0.59×) | 6,467 vs halo2curves 8,186 | −1,719 (0.79×) | 12,217 vs halo2curves 22,638 | −10,421 (0.54×) | 8,368 vs halo2curves 14,553 | −6,185 (0.58×) |
| div_uint | BLS12-381 base | 20,889 vs halo2curves 27,609 | −6,720 (0.76×) | 12,113 vs halo2curves 13,673 | −1,560 (0.89×) | 26,257 vs halo2curves 40,690 | −14,433 (0.65×) | 17,084 vs halo2curves 24,764 | −7,680 (0.69×) |
| div_uint | P-521 base | 34,490 vs crypto-bigint 69,543 | −35,053 (0.50×) | 18,103 vs crypto-bigint 52,439 | −34,336 (0.35×) | 49,529 vs crypto-bigint 83,769 | −34,240 (0.59×) | 30,346 vs crypto-bigint 63,669 | −33,323 (0.48×) |
| add, constant time | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 18.2 | −15.2 (0.16×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) | 3.0 vs runtime-algebra 18.2 | −15.2 (0.16×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add, constant time | BN254 scalar | 25.5 vs halo2curves 42.5 | −17.0 (0.60×) | 16.0 vs halo2curves 23.6 | −7.6 (0.68×) | 23.1 vs halo2curves 45.0 | −21.9 (0.51×) | 26.1 vs halo2curves 34.0 | −7.9 (0.77×) |
| add, constant time | secp256k1 base | 28.2 vs halo2curves 39.6 | −11.4 (0.71×) | 15.6 vs halo2curves 23.0 | −7.4 (0.68×) | 26.8 vs halo2curves 37.8 | −11.0 (0.71×) | 28.5 vs halo2curves 31.1 | −2.6 (0.92×) |
| add, constant time | P-256 base | 26.0 vs halo2curves 38.9 | −12.9 (0.67×) | 15.0 vs halo2curves 21.0 | −6.0 (0.71×) | 24.7 vs k256/p256 39.1 | −14.4 (0.63×) | 27.1 vs k256/p256 29.5 | −2.4 (0.92×) |
| add, constant time | P-256 scalar | 27.4 vs halo2curves 42.2 | −14.8 (0.65×) | 17.7 vs halo2curves 24.9 | −7.2 (0.71×) | 26.9 vs halo2curves 42.5 | −15.6 (0.63×) | 28.4 vs halo2curves 33.1 | −4.7 (0.86×) |
| add, constant time | Tom-256 base | 27.7 vs halo2curves 43.0 | −15.3 (0.64×) | 16.6 vs halo2curves 24.5 | −7.9 (0.68×) | 27.4 vs halo2curves 43.4 | −16.0 (0.63×) | 28.4 vs halo2curves 33.0 | −4.6 (0.86×) |
| add, constant time | BLS12-381 base | 41.4 vs halo2curves 63.3 | −21.9 (0.65×) | 23.3 vs halo2curves 34.5 | −11.2 (0.68×) | 35.2 vs halo2curves 67.2 | −32.0 (0.52×) | 40.6 vs halo2curves 54.0 | −13.4 (0.75×) |
| add, constant time | P-521 base | 76.5 vs crypto-bigint 180 | −104 (0.42×) | 39.5 vs crypto-bigint 181 | −142 (0.22×) | 82.6 vs crypto-bigint 246 | −163 (0.34×) | 72.0 vs crypto-bigint 246 | −174 (0.29×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs runtime-algebra 7.8 | −4.8 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.5 vs halo2curves 26.0 | −4.5 (0.83×) | 13.3 vs halo2curves 15.1 | −1.8 (0.88×) | 20.8 vs halo2curves 25.5 | −4.7 (0.82×) | 21.5 vs halo2curves 22.1 | −0.6 (0.97×) |
| sub, constant time | secp256k1 base | 21.3 vs halo2curves 25.2 | −3.9 (0.85×) | 12.3 vs halo2curves 15.1 | −2.8 (0.81×) | 20.3 vs halo2curves 24.0 | −3.7 (0.85×) | 21.0 vs halo2curves 21.0 | +0.0 (1.00×) |
| sub, constant time | P-256 base | 19.2 vs halo2curves 25.1 | −5.9 (0.76×) | 12.8 vs halo2curves 14.1 | −1.3 (0.91×) | 19.6 vs halo2curves 23.9 | −4.3 (0.82×) | 19.0 vs k256/p256 20.0 | −1.0 (0.95×) |
| sub, constant time | P-256 scalar | 21.6 vs k256/p256 25.0 | −3.4 (0.86×) | 13.8 vs halo2curves 15.6 | −1.8 (0.88×) | 19.2 vs k256/p256 27.8 | −8.6 (0.69×) | 21.0 vs halo2curves 22.0 | −1.0 (0.95×) |
| sub, constant time | Tom-256 base | 21.3 vs halo2curves 26.0 | −4.7 (0.82×) | 13.3 vs halo2curves 14.8 | −1.5 (0.90×) | 19.9 vs halo2curves 26.5 | −6.6 (0.75×) | 21.1 vs halo2curves 22.1 | −1.0 (0.95×) |
| sub, constant time | BLS12-381 base | 37.4 vs halo2curves 45.3 | −7.9 (0.83×) | 21.8 vs halo2curves 25.5 | −3.7 (0.85×) | 36.9 vs halo2curves 44.1 | −7.2 (0.84×) | 35.1 vs halo2curves 36.2 | −1.1 (0.97×) |
| sub, constant time | P-521 base | 56.4 vs crypto-bigint 159 | −103 (0.35×) | 32.3 vs crypto-bigint 144 | −112 (0.22×) | 60.1 vs crypto-bigint 195 | −135 (0.31×) | 51.1 vs crypto-bigint 175 | −124 (0.29×) |
| mul, constant time | Mersenne-31 | 7.0 vs crypto-bigint 46.8 | −39.8 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.9 | −43.9 (0.14×) | 7.0 vs crypto-bigint 31.0 | −24.0 (0.23×) |
| mul, constant time | BabyBear | 12.0 vs crypto-bigint 46.6 | −34.6 (0.26×) | 11.0 vs crypto-bigint 26.9 | −15.9 (0.41×) | 12.0 vs crypto-bigint 50.1 | −38.1 (0.24×) | 11.0 vs crypto-bigint 30.1 | −19.1 (0.37×) |
| mul, constant time | Goldilocks | 16.5 vs winter-math 18.0 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 12.5 vs winter-math 14.1 | −1.6 (0.89×) |
| mul, constant time | BN254 scalar | 241 vs halo2curves 384 | −143 (0.63×) | 93.5 vs halo2curves 95.2 | −1.7 (0.98×) | 413 vs halo2curves 706 | −293 (0.58×) | 149 vs halo2curves 178 | −29.0 (0.84×) |
| mul, constant time | secp256k1 base | 174 vs halo2curves 307 | −133 (0.57×) | 68.8 vs halo2curves 90.7 | −21.9 (0.76×) | 237 vs halo2curves 584 | −347 (0.41×) | 106 vs halo2curves 160 | −54.0 (0.66×) |
| mul, constant time | P-256 base | 191 vs k256/p256 241 | −50.0 (0.79×) | 65.0 vs halo2curves 75.6 | −10.6 (0.86×) | 273 vs k256/p256 475 | −202 (0.57×) | 118 vs k256/p256 131 | −13.0 (0.90×) |
| mul, constant time | P-256 scalar | 234 vs halo2curves 352 | −118 (0.66×) | 87.9 vs halo2curves 95.9 | −8.0 (0.92×) | 329 vs halo2curves 697 | −368 (0.47×) | 157 vs halo2curves 176 | −19.0 (0.89×) |
| mul, constant time | Tom-256 base | 236 vs halo2curves 345 | −109 (0.68×) | 79.8 vs halo2curves 93.9 | −14.1 (0.85×) | 314 vs halo2curves 598 | −284 (0.53×) | 167 vs halo2curves 172 | −5.0 (0.97×) |
| mul, constant time | BLS12-381 base | 720 vs halo2curves 905 | −185 (0.80×) | 186 vs halo2curves 195 | −9.0 (0.95×) | 910 vs crypto-bigint 1,320 | −410 (0.69×) | 360 vs halo2curves 378 | −18.0 (0.95×) |
| mul, constant time | P-521 base | 689 vs crypto-bigint 2,146 | −1,457 (0.32×) | 241 vs crypto-bigint 619 | −378 (0.39×) | 1,314 vs crypto-bigint 2,791 | −1,477 (0.47×) | 439 vs crypto-bigint 1,109 | −670 (0.40×) |
| square, constant time | Mersenne-31 | 7.0 vs crypto-bigint 45.2 | −38.2 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 49.0 | −42.0 (0.14×) | 7.0 vs crypto-bigint 31.2 | −24.2 (0.22×) |
| square, constant time | BabyBear | 12.0 vs crypto-bigint 45.2 | −33.2 (0.27×) | 11.0 vs crypto-bigint 27.1 | −16.1 (0.41×) | 12.0 vs crypto-bigint 50.4 | −38.4 (0.24×) | 11.0 vs crypto-bigint 30.0 | −19.0 (0.37×) |
| square, constant time | Goldilocks | 15.0 vs winter-math 20.0 | −5.0 (0.75×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.7 vs winter-math 19.0 | −0.3 (0.98×) | 12.5 vs winter-math 14.4 | −1.9 (0.87×) |
| square, constant time | BN254 scalar | 220 vs halo2curves 308 | −88.0 (0.71×) | 82.1 vs halo2curves 97.0 | −14.9 (0.85×) | 203 vs halo2curves 540 | −337 (0.38×) | 131 vs halo2curves 149 | −18.0 (0.88×) |
| square, constant time | secp256k1 base | 151 vs halo2curves 222 | −71.0 (0.68×) | 58.7 vs halo2curves 84.3 | −25.6 (0.70×) | 181 vs halo2curves 454 | −273 (0.40×) | 87.2 vs halo2curves 118 | −30.8 (0.74×) |
| square, constant time | P-256 base | 161 vs halo2curves 180 | −19.0 (0.89×) | 55.7 vs halo2curves 61.8 | −6.1 (0.90×) | 210 vs halo2curves 337 | −127 (0.62×) | 105 vs halo2curves 108 | −3.0 (0.97×) |
| square, constant time | P-256 scalar | 222 vs halo2curves 266 | −44.0 (0.83×) | 81.2 vs halo2curves 92.5 | −11.3 (0.88×) | 230 vs halo2curves 557 | −327 (0.41×) | 144 vs halo2curves 143 | +1.0 (1.01×) |
| square, constant time | Tom-256 base | 213 vs halo2curves 268 | −55.0 (0.79×) | 77.3 vs halo2curves 88.4 | −11.1 (0.87×) | 235 vs halo2curves 488 | −253 (0.48×) | 134 vs halo2curves 135 | −1.0 (0.99×) |
| square, constant time | BLS12-381 base | 560 vs halo2curves 936 | −376 (0.60×) | 161 vs halo2curves 161 | +0.0 (1.00×) | 810 vs halo2curves 1,246 | −436 (0.65×) | 322 vs halo2curves 296 | +26.0 (1.09×) |
| square, constant time | P-521 base | 571 vs crypto-bigint 2,174 | −1,603 (0.26×) | 207 vs crypto-bigint 625 | −418 (0.33×) | 914 vs crypto-bigint 2,858 | −1,944 (0.32×) | 389 vs crypto-bigint 1,140 | −751 (0.34×) |
| mul_u64, constant time | Mersenne-31 | 9.0 vs crypto-bigint 60.1 | −51.1 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 75.1 | −66.1 (0.12×) | 9.0 vs crypto-bigint 38.3 | −29.3 (0.23×) |
| mul_u64, constant time | BabyBear | 16.1 vs crypto-bigint 65.2 | −49.1 (0.25×) | 11.0 vs crypto-bigint 32.1 | −21.1 (0.34×) | 15.7 vs crypto-bigint 89.1 | −73.4 (0.18×) | 12.9 vs crypto-bigint 40.9 | −28.0 (0.32×) |
| mul_u64, constant time | Goldilocks | 17.8 vs winter-math 23.5 | −5.7 (0.76×) | 14.1 vs winter-math 13.9 | +0.2 (1.01×) | 25.3 vs winter-math 34.5 | −9.2 (0.73×) | 14.9 vs winter-math 14.6 | +0.3 (1.02×) |
| mul_u64, constant time | BN254 scalar | 109 vs crypto-bigint 854 | −745 (0.13×) | 48.3 vs crypto-bigint 262 | −214 (0.18×) | 132 vs halo2curves 1,450 | −1,318 (0.09×) | 52.4 vs crypto-bigint 410 | −358 (0.13×) |
| mul_u64, constant time | secp256k1 base | 68.9 vs halo2curves 692 | −623 (0.10×) | 29.0 vs crypto-bigint 259 | −230 (0.11×) | 85.1 vs halo2curves 1,034 | −949 (0.08×) | 40.8 vs halo2curves 387 | −346 (0.11×) |
| mul_u64, constant time | P-256 base | 90.2 vs k256/p256 487 | −397 (0.19×) | 46.5 vs k256/p256 154 | −108 (0.30×) | 106 vs k256/p256 950 | −844 (0.11×) | 70.4 vs k256/p256 258 | −188 (0.27×) |
| mul_u64, constant time | P-256 scalar | 104 vs k256/p256 518 | −414 (0.20×) | 46.4 vs k256/p256 255 | −209 (0.18×) | 147 vs k256/p256 810 | −663 (0.18×) | 70.7 vs k256/p256 338 | −267 (0.21×) |
| mul_u64, constant time | Tom-256 base | 99.9 vs crypto-bigint 863 | −763 (0.12×) | 48.1 vs crypto-bigint 259 | −211 (0.19×) | 121 vs halo2curves 1,315 | −1,194 (0.09×) | 72.7 vs crypto-bigint 411 | −338 (0.18×) |
| mul_u64, constant time | BLS12-381 base | 154 vs crypto-bigint 1,929 | −1,775 (0.08×) | 64.2 vs halo2curves 489 | −425 (0.13×) | 195 vs crypto-bigint 2,589 | −2,394 (0.08×) | 87.3 vs crypto-bigint 1,045 | −958 (0.08×) |
| mul_u64, constant time | P-521 base | 139 vs crypto-bigint 3,070 | −2,931 (0.05×) | 65.1 vs crypto-bigint 996 | −931 (0.07×) | 192 vs crypto-bigint 4,093 | −3,901 (0.05×) | 90.3 vs crypto-bigint 1,846 | −1,756 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 32.5 | −23.5 (0.28×) | 9.0 vs crypto-bigint 75.2 | −66.2 (0.12×) | 9.0 vs crypto-bigint 38.8 | −29.8 (0.23×) |
| mul_uint, constant time | BabyBear | 16.1 vs crypto-bigint 65.4 | −49.3 (0.25×) | 11.0 vs crypto-bigint 32.8 | −21.8 (0.34×) | 15.8 vs crypto-bigint 91.0 | −75.2 (0.17×) | 14.0 vs crypto-bigint 40.7 | −26.7 (0.34×) |
| mul_uint, constant time | Goldilocks | 17.8 vs winter-math 23.4 | −5.6 (0.76×) | 14.1 vs winter-math 13.7 | +0.4 (1.03×) | 25.2 vs winter-math 34.5 | −9.3 (0.73×) | 15.2 vs winter-math 14.6 | +0.6 (1.04×) |
| mul_uint, constant time | BN254 scalar | 482 vs halo2curves 758 | −276 (0.64×) | 164 vs halo2curves 178 | −14.0 (0.92×) | 592 vs halo2curves 1,318 | −726 (0.45×) | 300 vs halo2curves 353 | −53.0 (0.85×) |
| mul_uint, constant time | secp256k1 base | 318 vs halo2curves 517 | −199 (0.62×) | 70.7 vs halo2curves 154 | −83.3 (0.46×) | 380 vs halo2curves 962 | −582 (0.40×) | 117 vs halo2curves 266 | −149 (0.44×) |
| mul_uint, constant time | P-256 base | 409 vs halo2curves 452 | −43.0 (0.90×) | 126 vs halo2curves 146 | −20.0 (0.86×) | 488 vs halo2curves 978 | −490 (0.50×) | 225 vs halo2curves 257 | −32.0 (0.88×) |
| mul_uint, constant time | P-256 scalar | 474 vs k256/p256 553 | −79.0 (0.86×) | 173 vs halo2curves 186 | −13.0 (0.93×) | 541 vs k256/p256 877 | −336 (0.62×) | 318 vs halo2curves 347 | −29.0 (0.92×) |
| mul_uint, constant time | Tom-256 base | 461 vs halo2curves 694 | −233 (0.66×) | 163 vs halo2curves 179 | −16.0 (0.91×) | 555 vs halo2curves 1,178 | −623 (0.47×) | 349 vs halo2curves 338 | +11.0 (1.03×) |
| mul_uint, constant time | BLS12-381 base | 1,076 vs halo2curves 1,898 | −822 (0.57×) | 354 vs halo2curves 380 | −26.0 (0.93×) | 1,693 vs crypto-bigint 2,585 | −892 (0.65×) | 930 vs crypto-bigint 1,044 | −114 (0.89×) |
| mul_uint, constant time | P-521 base | 2,021 vs crypto-bigint 3,062 | −1,041 (0.66×) | 285 vs crypto-bigint 1,009 | −724 (0.28×) | 2,937 vs crypto-bigint 4,011 | −1,074 (0.73×) | 594 vs crypto-bigint 1,892 | −1,298 (0.31×) |
| add_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.7 vs crypto-bigint 30.1 | −25.4 (0.16×) | 4.8 vs crypto-bigint 43.4 | −38.6 (0.11×) | 4.9 vs crypto-bigint 31.0 | −26.1 (0.16×) |
| add_u64, constant time | BabyBear | 11.9 vs crypto-bigint 47.5 | −35.6 (0.25×) | 6.9 vs crypto-bigint 30.0 | −23.1 (0.23×) | 13.0 vs crypto-bigint 51.6 | −38.6 (0.25×) | 11.3 vs crypto-bigint 31.8 | −20.5 (0.36×) |
| add_u64, constant time | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.5 | −1.0 (0.87×) | 11.7 vs winter-math 19.0 | −7.3 (0.62×) | 9.3 vs winter-math 9.8 | −0.5 (0.95×) |
| add_u64, constant time | BN254 scalar | 93.0 vs crypto-bigint 504 | −411 (0.18×) | 40.8 vs halo2curves 213 | −172 (0.19×) | 108 vs halo2curves 757 | −649 (0.14×) | 66.3 vs halo2curves 299 | −233 (0.22×) |
| add_u64, constant time | secp256k1 base | 25.5 vs halo2curves 389 | −364 (0.07×) | 15.0 vs halo2curves 207 | −192 (0.07×) | 25.1 vs halo2curves 489 | −464 (0.05×) | 24.1 vs halo2curves 239 | −215 (0.10×) |
| add_u64, constant time | P-256 base | 66.5 vs k256/p256 277 | −211 (0.24×) | 35.5 vs k256/p256 92.6 | −57.1 (0.38×) | 80.1 vs k256/p256 510 | −430 (0.16×) | 58.0 vs k256/p256 158 | −100 (0.37×) |
| add_u64, constant time | P-256 scalar | 104 vs k256/p256 56.3 | +47.7 (1.85×) | 40.7 vs k256/p256 56.1 | −15.4 (0.73×) | 109 vs k256/p256 62.2 | +46.8 (1.75×) | 83.2 vs k256/p256 62.5 | +20.7 (1.33×) |
| add_u64, constant time | Tom-256 base | 97.6 vs crypto-bigint 511 | −413 (0.19×) | 40.2 vs halo2curves 217 | −177 (0.19×) | 110 vs halo2curves 706 | −596 (0.16×) | 70.3 vs halo2curves 287 | −217 (0.24×) |
| add_u64, constant time | BLS12-381 base | 136 vs crypto-bigint 1,090 | −954 (0.12×) | 63.8 vs halo2curves 323 | −259 (0.20×) | 177 vs crypto-bigint 1,393 | −1,216 (0.13×) | 102 vs crypto-bigint 657 | −555 (0.16×) |
| add_u64, constant time | P-521 base | 65.8 vs crypto-bigint 1,677 | −1,611 (0.04×) | 36.5 vs crypto-bigint 651 | −615 (0.06×) | 66.8 vs crypto-bigint 2,257 | −2,190 (0.03×) | 62.1 vs crypto-bigint 1,144 | −1,082 (0.05×) |
| add_uint, constant time | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.7 vs crypto-bigint 43.5 | −38.8 (0.11×) | 4.8 vs crypto-bigint 30.0 | −25.2 (0.16×) |
| add_uint, constant time | BabyBear | 11.9 vs crypto-bigint 47.6 | −35.7 (0.25×) | 7.2 vs crypto-bigint 30.0 | −22.8 (0.24×) | 13.0 vs crypto-bigint 51.8 | −38.8 (0.25×) | 11.4 vs crypto-bigint 31.7 | −20.3 (0.36×) |
| add_uint, constant time | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.3 vs winter-math 9.7 | −0.4 (0.96×) |
| add_uint, constant time | BN254 scalar | 238 vs halo2curves 417 | −179 (0.57×) | 96.6 vs halo2curves 112 | −15.4 (0.86×) | 327 vs halo2curves 679 | −352 (0.48×) | 188 vs halo2curves 206 | −18.0 (0.91×) |
| add_uint, constant time | secp256k1 base | 38.1 vs halo2curves 225 | −187 (0.17×) | 25.1 vs halo2curves 83.5 | −58.4 (0.30×) | 36.4 vs halo2curves 419 | −383 (0.09×) | 40.5 vs halo2curves 141 | −101 (0.29×) |
| add_uint, constant time | P-256 base | 179 vs halo2curves 249 | −70.0 (0.72×) | 74.4 vs halo2curves 91.8 | −17.4 (0.81×) | 268 vs halo2curves 514 | −246 (0.52×) | 142 vs halo2curves 154 | −12.0 (0.92×) |
| add_uint, constant time | P-256 scalar | 260 vs k256/p256 77.1 | +183 (3.37×) | 101 vs k256/p256 82.1 | +18.9 (1.23×) | 310 vs k256/p256 107 | +203 (2.90×) | 201 vs k256/p256 115 | +86.0 (1.75×) |
| add_uint, constant time | Tom-256 base | 252 vs halo2curves 373 | −121 (0.68×) | 95.3 vs halo2curves 110 | −14.7 (0.87×) | 328 vs halo2curves 636 | −308 (0.52×) | 179 vs halo2curves 197 | −18.0 (0.91×) |
| add_uint, constant time | BLS12-381 base | 756 vs halo2curves 955 | −199 (0.79×) | 191 vs halo2curves 223 | −32.0 (0.86×) | 941 vs crypto-bigint 1,415 | −474 (0.67×) | 408 vs halo2curves 443 | −35.0 (0.92×) |
| add_uint, constant time | P-521 base | 137 vs crypto-bigint 1,689 | −1,552 (0.08×) | 78.2 vs crypto-bigint 666 | −588 (0.12×) | 138 vs crypto-bigint 2,215 | −2,077 (0.06×) | 132 vs crypto-bigint 1,197 | −1,065 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 22.2 | −17.5 (0.21×) | 4.7 vs crypto-bigint 18.2 | −13.5 (0.26×) | 4.9 vs crypto-bigint 30.3 | −25.4 (0.16×) | 4.9 vs crypto-bigint 20.3 | −15.4 (0.24×) |
| sub_u64, constant time | BabyBear | 11.9 vs crypto-bigint 28.8 | −16.9 (0.41×) | 7.1 vs crypto-bigint 19.3 | −12.2 (0.37×) | 12.9 vs crypto-bigint 42.2 | −29.3 (0.31×) | 10.6 vs crypto-bigint 21.8 | −11.2 (0.49×) |
| sub_u64, constant time | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.8 vs winter-math 8.6 | +0.2 (1.02×) |
| sub_u64, constant time | BN254 scalar | 94.8 vs crypto-bigint 480 | −385 (0.20×) | 37.0 vs crypto-bigint 152 | −115 (0.24×) | 107 vs halo2curves 742 | −635 (0.14×) | 66.7 vs crypto-bigint 288 | −221 (0.23×) |
| sub_u64, constant time | secp256k1 base | 18.1 vs halo2curves 386 | −368 (0.05×) | 12.0 vs crypto-bigint 152 | −140 (0.08×) | 17.0 vs halo2curves 469 | −452 (0.04×) | 15.1 vs halo2curves 238 | −223 (0.06×) |
| sub_u64, constant time | P-256 base | 61.4 vs k256/p256 265 | −204 (0.23×) | 31.6 vs k256/p256 88.0 | −56.4 (0.36×) | 74.6 vs k256/p256 486 | −411 (0.15×) | 53.3 vs k256/p256 150 | −96.7 (0.36×) |
| sub_u64, constant time | P-256 scalar | 96.3 vs k256/p256 21.2 | +75.1 (4.54×) | 37.4 vs k256/p256 21.3 | +16.1 (1.76×) | 107 vs k256/p256 24.7 | +82.3 (4.33×) | 69.8 vs k256/p256 24.7 | +45.1 (2.83×) |
| sub_u64, constant time | Tom-256 base | 92.0 vs crypto-bigint 496 | −404 (0.19×) | 36.6 vs crypto-bigint 152 | −115 (0.24×) | 105 vs halo2curves 694 | −589 (0.15×) | 74.3 vs halo2curves 278 | −204 (0.27×) |
| sub_u64, constant time | BLS12-381 base | 140 vs crypto-bigint 1,094 | −954 (0.13×) | 59.9 vs halo2curves 315 | −255 (0.19×) | 165 vs crypto-bigint 1,414 | −1,249 (0.12×) | 111 vs crypto-bigint 645 | −534 (0.17×) |
| sub_u64, constant time | P-521 base | 45.7 vs crypto-bigint 1,664 | −1,618 (0.03×) | 27.1 vs crypto-bigint 630 | −603 (0.04×) | 49.8 vs crypto-bigint 2,233 | −2,183 (0.02×) | 40.2 vs crypto-bigint 1,139 | −1,099 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 4.6 vs crypto-bigint 22.1 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.3 | −13.7 (0.25×) | 4.9 vs crypto-bigint 30.5 | −25.6 (0.16×) | 5.0 vs crypto-bigint 20.4 | −15.4 (0.25×) |
| sub_uint, constant time | BabyBear | 12.1 vs crypto-bigint 28.7 | −16.6 (0.42×) | 7.1 vs crypto-bigint 19.4 | −12.3 (0.37×) | 13.0 vs crypto-bigint 42.4 | −29.4 (0.31×) | 10.6 vs crypto-bigint 21.9 | −11.3 (0.48×) |
| sub_uint, constant time | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.7 vs winter-math 8.3 | +0.4 (1.05×) |
| sub_uint, constant time | BN254 scalar | 249 vs halo2curves 401 | −152 (0.62×) | 92.6 vs halo2curves 106 | −13.4 (0.87×) | 307 vs halo2curves 660 | −353 (0.47×) | 182 vs halo2curves 195 | −13.0 (0.93×) |
| sub_uint, constant time | secp256k1 base | 32.3 vs halo2curves 206 | −174 (0.16×) | 20.6 vs halo2curves 80.7 | −60.1 (0.26×) | 31.8 vs halo2curves 408 | −376 (0.08×) | 35.6 vs halo2curves 130 | −94.4 (0.27×) |
| sub_uint, constant time | P-256 base | 176 vs halo2curves 235 | −59.0 (0.75×) | 71.1 vs halo2curves 85.8 | −14.7 (0.83×) | 260 vs halo2curves 502 | −242 (0.52×) | 136 vs halo2curves 144 | −8.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 253 vs k256/p256 72.0 | +181 (3.51×) | 97.1 vs k256/p256 77.7 | +19.4 (1.25×) | 330 vs k256/p256 94.9 | +235 (3.48×) | 189 vs k256/p256 98.4 | +90.6 (1.92×) |
| sub_uint, constant time | Tom-256 base | 248 vs halo2curves 366 | −118 (0.68×) | 97.4 vs halo2curves 103 | −5.6 (0.95×) | 333 vs halo2curves 608 | −275 (0.55×) | 192 vs halo2curves 188 | +4.0 (1.02×) |
| sub_uint, constant time | BLS12-381 base | 760 vs halo2curves 959 | −199 (0.79×) | 198 vs halo2curves 215 | −17.0 (0.92×) | 932 vs crypto-bigint 1,413 | −481 (0.66×) | 415 vs halo2curves 509 | −94.0 (0.82×) |
| sub_uint, constant time | P-521 base | 125 vs crypto-bigint 1,688 | −1,563 (0.07×) | 67.2 vs crypto-bigint 639 | −572 (0.11×) | 122 vs crypto-bigint 2,196 | −2,074 (0.06×) | 108 vs crypto-bigint 1,135 | −1,027 (0.10×) |
| div_u64, constant time | BN254 scalar | 12,436 vs crypto-bigint 29,962 | −17,526 (0.42×) | 9,395 vs crypto-bigint 25,595 | −16,200 (0.37×) | 17,184 vs crypto-bigint 38,255 | −21,071 (0.45×) | 13,246 vs crypto-bigint 31,033 | −17,787 (0.43×) |
| div_u64, constant time | secp256k1 base | 11,953 vs crypto-bigint 30,062 | −18,109 (0.40×) | 9,020 vs crypto-bigint 25,461 | −16,441 (0.35×) | 17,156 vs crypto-bigint 38,211 | −21,055 (0.45×) | 12,727 vs crypto-bigint 30,865 | −18,138 (0.41×) |
| div_u64, constant time | P-256 base | 11,868 vs crypto-bigint 30,223 | −18,355 (0.39×) | 9,116 vs k256/p256 22,225 | −13,109 (0.41×) | 16,968 vs crypto-bigint 38,228 | −21,260 (0.44×) | 12,488 vs crypto-bigint 30,624 | −18,136 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,309 vs crypto-bigint 30,103 | −17,794 (0.41×) | 9,353 vs crypto-bigint 25,511 | −16,158 (0.37×) | 17,558 vs crypto-bigint 38,503 | −20,945 (0.46×) | 13,125 vs crypto-bigint 30,760 | −17,635 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,189 vs crypto-bigint 30,079 | −17,890 (0.41×) | 9,276 vs crypto-bigint 25,622 | −16,346 (0.36×) | 16,895 vs crypto-bigint 38,243 | −21,348 (0.44×) | 13,006 vs crypto-bigint 30,720 | −17,714 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,805 vs crypto-bigint 51,041 | −26,236 (0.49×) | 16,887 vs crypto-bigint 43,512 | −26,625 (0.39×) | 34,839 vs crypto-bigint 69,107 | −34,268 (0.50×) | 24,262 vs crypto-bigint 54,808 | −30,546 (0.44×) |
| div_u64, constant time | P-521 base | 42,990 vs crypto-bigint 114,304 | −71,314 (0.38×) | 26,086 vs crypto-bigint 85,085 | −58,999 (0.31×) | 65,028 vs crypto-bigint 126,951 | −61,923 (0.51×) | 43,122 vs crypto-bigint 101,071 | −57,949 (0.43×) |
| div_uint, constant time | Mersenne-31 | 275 vs runtime-algebra 3,534 | −3,259 (0.08×) | 275 vs runtime-algebra 3,539 | −3,264 (0.08×) | 274 vs runtime-algebra 3,451 | −3,177 (0.08×) | 274 vs runtime-algebra 3,417 | −3,143 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,570 | −3,101 (0.13×) | 435 vs runtime-algebra 3,565 | −3,130 (0.12×) | 470 vs runtime-algebra 3,447 | −2,977 (0.14×) | 434 vs runtime-algebra 3,408 | −2,974 (0.13×) |
| div_uint, constant time | Goldilocks | 1,138 vs winter-math 1,817 | −679 (0.63×) | 860 vs winter-math 928 | −68.0 (0.93×) | 1,434 vs winter-math 1,898 | −464 (0.76×) | 950 vs winter-math 1,031 | −81.0 (0.92×) |
| div_uint, constant time | BN254 scalar | 12,367 vs crypto-bigint 29,926 | −17,559 (0.41×) | 9,359 vs crypto-bigint 25,834 | −16,475 (0.36×) | 17,713 vs crypto-bigint 38,337 | −20,624 (0.46×) | 13,362 vs crypto-bigint 30,758 | −17,396 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,962 vs crypto-bigint 29,873 | −17,911 (0.40×) | 9,056 vs crypto-bigint 25,941 | −16,885 (0.35×) | 16,891 vs crypto-bigint 38,143 | −21,252 (0.44×) | 12,821 vs crypto-bigint 30,599 | −17,778 (0.42×) |
| div_uint, constant time | P-256 base | 11,953 vs crypto-bigint 29,921 | −17,968 (0.40×) | 9,091 vs k256/p256 22,554 | −13,463 (0.40×) | 17,483 vs crypto-bigint 38,170 | −20,687 (0.46×) | 12,627 vs crypto-bigint 30,745 | −18,118 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,589 vs crypto-bigint 30,110 | −17,521 (0.42×) | 9,453 vs crypto-bigint 25,837 | −16,384 (0.37×) | 17,919 vs crypto-bigint 38,574 | −20,655 (0.46×) | 13,356 vs crypto-bigint 30,657 | −17,301 (0.44×) |
| div_uint, constant time | Tom-256 base | 12,408 vs crypto-bigint 29,829 | −17,421 (0.42×) | 9,389 vs crypto-bigint 25,851 | −16,462 (0.36×) | 17,014 vs crypto-bigint 38,248 | −21,234 (0.44×) | 13,142 vs crypto-bigint 30,886 | −17,744 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,405 vs crypto-bigint 51,991 | −26,586 (0.49×) | 17,020 vs crypto-bigint 43,448 | −26,428 (0.39×) | 35,338 vs crypto-bigint 69,062 | −33,724 (0.51×) | 24,760 vs crypto-bigint 54,113 | −29,353 (0.46×) |
| div_uint, constant time | P-521 base | 43,026 vs crypto-bigint 106,686 | −63,660 (0.40×) | 26,311 vs crypto-bigint 88,444 | −62,133 (0.30×) | 64,427 vs crypto-bigint 126,906 | −62,479 (0.51×) | 43,137 vs crypto-bigint 98,787 | −55,650 (0.44×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add | BabyBear | 1.9 | 2.4 (1.3×) | 2.7 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add | Goldilocks | 2.0 | 3.3 (1.6×) | 3.4 (1.7×) | 3.1 (1.6×) | 3.2 (1.6×) |
| add | BN254 scalar | 4.9 | 13.9 (2.8×) | 12.8 (2.6×) | 14.8 (3.0×) | 18.9 (3.9×) |
| add | secp256k1 base | 5.8 | 14.0 (2.4×) | 11.8 (2.0×) | 14.9 (2.6×) | 20.5 (3.5×) |
| add | P-256 base | 6.0 | 14.4 (2.4×) | 12.2 (2.0×) | 14.5 (2.4×) | 20.3 (3.4×) |
| add | P-256 scalar | 5.8 | 14.5 (2.5×) | 11.8 (2.0×) | 15.7 (2.7×) | 20.6 (3.6×) |
| add | Tom-256 base | 6.0 | 13.6 (2.3×) | 11.8 (2.0×) | 14.3 (2.4×) | 20.3 (3.4×) |
| add | BLS12-381 base | 9.6 | 22.0 (2.3×) | 18.6 (1.9×) | 22.3 (2.3×) | 29.7 (3.1×) |
| add | P-521 base | 12.3 | 42.2 (3.4×) | 26.4 (2.1×) | 43.6 (3.5×) | 44.6 (3.6×) |
| sub | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BabyBear | 2.0 | 2.5 (1.2×) | 2.6 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub | BN254 scalar | 4.9 | 15.0 (3.1×) | 11.7 (2.4×) | 15.0 (3.1×) | 19.2 (3.9×) |
| sub | secp256k1 base | 5.4 | 16.7 (3.1×) | 11.8 (2.2×) | 15.7 (2.9×) | 18.8 (3.5×) |
| sub | P-256 base | 5.5 | 14.3 (2.6×) | 12.2 (2.2×) | 14.0 (2.5×) | 18.3 (3.3×) |
| sub | P-256 scalar | 5.0 | 16.1 (3.2×) | 11.8 (2.4×) | 15.0 (3.0×) | 18.6 (3.7×) |
| sub | Tom-256 base | 5.5 | 15.0 (2.7×) | 11.6 (2.1×) | 14.3 (2.6×) | 19.2 (3.5×) |
| sub | BLS12-381 base | 9.6 | 22.9 (2.4×) | 17.2 (1.8×) | 21.8 (2.3×) | 29.5 (3.1×) |
| sub | P-521 base | 11.9 | 46.9 (3.9×) | 26.6 (2.2×) | 42.2 (3.5×) | 43.3 (3.6×) |
| mul | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 15.0 (3.9×) | 7.0 (1.8×) |
| mul | BN254 scalar | 68.8 | 234 (3.4×) | 88.2 (1.3×) | 385 (5.6×) | 173 (2.5×) |
| mul | secp256k1 base | 44.5 | 160 (3.6×) | 59.6 (1.3×) | 239 (5.4×) | 104 (2.3×) |
| mul | P-256 base | 49.5 | 179 (3.6×) | 71.8 (1.5×) | 278 (5.6×) | 126 (2.5×) |
| mul | P-256 scalar | 74.8 | 244 (3.3×) | 84.2 (1.1×) | 329 (4.4×) | 170 (2.3×) |
| mul | Tom-256 base | 70.9 | 234 (3.3×) | 80.7 (1.1×) | 318 (4.5×) | 178 (2.5×) |
| mul | BLS12-381 base | 138 | 727 (5.3×) | 188 (1.4×) | 914 (6.6×) | 348 (2.5×) |
| mul | P-521 base | 176 | 669 (3.8×) | 245 (1.4×) | 1,288 (7.3×) | 532 (3.0×) |
| square | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.7 (1.4×) |
| square | BabyBear | 3.4 | 4.7 (1.4×) | 4.8 (1.4×) | 5.7 (1.7×) | 5.4 (1.6×) |
| square | Goldilocks | 3.7 | 8.7 (2.4×) | 6.0 (1.6×) | 13.0 (3.5×) | 8.0 (2.2×) |
| square | BN254 scalar | 63.1 | 214 (3.4×) | 81.6 (1.3×) | 347 (5.5×) | 160 (2.5×) |
| square | secp256k1 base | 33.8 | 142 (4.2×) | 53.1 (1.6×) | 175 (5.2×) | 91.0 (2.7×) |
| square | P-256 base | 40.0 | 152 (3.8×) | 74.3 (1.9×) | 238 (6.0×) | 120 (3.0×) |
| square | P-256 scalar | 63.9 | 213 (3.3×) | 84.9 (1.3×) | 250 (3.9×) | 158 (2.5×) |
| square | Tom-256 base | 57.0 | 210 (3.7×) | 72.2 (1.3×) | 219 (3.8×) | 140 (2.5×) |
| square | BLS12-381 base | 131 | 568 (4.3×) | 156 (1.2×) | 823 (6.3×) | 330 (2.5×) |
| square | P-521 base | 117 | 565 (4.8×) | 237 (2.0×) | 907 (7.8×) | 391 (3.3×) |
| invert | Mersenne-31 | 214 | 266 (1.2×) | 267 (1.2×) | 275 (1.3×) | 275 (1.3×) |
| invert | BabyBear | 213 | 269 (1.3×) | 268 (1.3×) | 276 (1.3×) | 278 (1.3×) |
| invert | Goldilocks | 437 | 539 (1.2×) | 540 (1.2×) | 571 (1.3×) | 571 (1.3×) |
| invert | BN254 scalar | 5,057 | 8,468 (1.7×) | 6,489 (1.3×) | 11,684 (2.3×) | 7,852 (1.6×) |
| invert | secp256k1 base | 4,922 | 8,477 (1.7×) | 6,020 (1.2×) | 12,134 (2.5×) | 7,911 (1.6×) |
| invert | P-256 base | 4,837 | 8,295 (1.7×) | 6,104 (1.3×) | 11,383 (2.4×) | 7,739 (1.6×) |
| invert | P-256 scalar | 5,041 | 8,512 (1.7×) | 6,405 (1.3×) | 12,064 (2.4×) | 8,190 (1.6×) |
| invert | Tom-256 base | 5,018 | 8,533 (1.7×) | 6,434 (1.3×) | 11,414 (2.3×) | 8,039 (1.6×) |
| invert | BLS12-381 base | 8,808 | 19,379 (2.2×) | 11,415 (1.3×) | 24,467 (2.8×) | 15,947 (1.8×) |
| invert | P-521 base | 14,227 | 33,340 (2.3×) | 18,119 (1.3×) | 48,645 (3.4×) | 29,588 (2.1×) |
| sqrt | Mersenne-31 | 135 | 159 (1.2×) | 159 (1.2×) | 160 (1.2×) | 163 (1.2×) |
| sqrt | BabyBear | 1,971 | 2,132 (1.1×) | 2,058 (1.0×) | 2,150 (1.1×) | 2,087 (1.1×) |
| sqrt | Goldilocks | 2,833 | 4,828 (1.7×) | 3,497 (1.2×) | 6,129 (2.2×) | 5,090 (1.8×) |
| sqrt | BN254 scalar | 33,810 | 114,468 (3.4×) | 51,670 (1.5×) | 180,838 (5.3×) | 98,533 (2.9×) |
| sqrt | secp256k1 base | 12,304 | 40,917 (3.3×) | 16,595 (1.3×) | 65,357 (5.3×) | 27,701 (2.3×) |
| sqrt | P-256 base | 11,796 | 43,092 (3.7×) | 14,467 (1.2×) | 78,209 (6.6×) | 30,838 (2.6×) |
| sqrt | P-256 scalar | 24,127 | 73,381 (3.0×) | 33,306 (1.4×) | 114,062 (4.7×) | 64,307 (2.7×) |
| sqrt | Tom-256 base | 21,264 | 65,879 (3.1×) | 18,318 (0.9×) | 99,285 (4.7×) | 51,931 (2.4×) |
| sqrt | BLS12-381 base | 65,940 | 297,168 (4.5×) | 93,867 (1.4×) | 395,495 (6.0×) | 217,458 (3.3×) |
| sqrt | P-521 base | 68,261 | 317,910 (4.7×) | 126,485 (1.9×) | 473,179 (6.9×) | 267,041 (3.9×) |
| mul_u64 | Mersenne-31 | 5.8 | 6.7 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_u64 | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 15.9 (1.8×) |
| mul_u64 | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.3 (1.3×) |
| mul_u64 | BN254 scalar | 20.7 | 101 (4.9×) | 28.8 (1.4×) | 124 (6.0×) | 43.7 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 42.5 (4.4×) | 12.6 (1.3×) | 65.9 (6.8×) | 24.6 (2.5×) |
| mul_u64 | P-256 base | 18.9 | 72.4 (3.8×) | 27.3 (1.4×) | 98.5 (5.2×) | 43.0 (2.3×) |
| mul_u64 | P-256 scalar | 22.3 | 84.1 (3.8×) | 30.3 (1.4×) | 113 (5.1×) | 48.6 (2.2×) |
| mul_u64 | Tom-256 base | 20.5 | 82.1 (4.0×) | 30.0 (1.5×) | 116 (5.7×) | 45.7 (2.2×) |
| mul_u64 | BLS12-381 base | 28.8 | 131 (4.5×) | 44.2 (1.5×) | 188 (6.5×) | 58.8 (2.0×) |
| mul_u64 | P-521 base | 19.7 | 92.1 (4.7×) | 31.9 (1.6×) | 257 (13.0×) | 50.3 (2.6×) |
| mul_uint | Mersenne-31 | 5.9 | 6.7 (1.1×) | 6.8 (1.2×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_uint | BabyBear | 8.9 | 13.1 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 15.3 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.1 (1.3×) |
| mul_uint | BN254 scalar | 103 | 437 (4.2×) | 171 (1.7×) | 541 (5.3×) | 290 (2.8×) |
| mul_uint | secp256k1 base | 56.9 | 266 (4.7×) | 59.7 (1.0×) | 319 (5.6×) | 108 (1.9×) |
| mul_uint | P-256 base | 102 | 353 (3.5×) | 125 (1.2×) | 429 (4.2×) | 254 (2.5×) |
| mul_uint | P-256 scalar | 110 | 427 (3.9×) | 173 (1.6×) | 480 (4.4×) | 369 (3.4×) |
| mul_uint | Tom-256 base | 108 | 413 (3.8×) | 170 (1.6×) | 493 (4.6×) | 340 (3.1×) |
| mul_uint | BLS12-381 base | 211 | 987 (4.7×) | 363 (1.7×) | 1,161 (5.5×) | 904 (4.3×) |
| mul_uint | P-521 base | 350 | 1,564 (4.5×) | 258 (0.7×) | 2,388 (6.8×) | 463 (1.3×) |
| add_u64 | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.3 (1.2×) | 5.4 (1.2×) |
| add_u64 | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.7 (2.2×) |
| add_u64 | Goldilocks | 4.8 | 9.2 (1.9×) | 7.1 (1.5×) | 11.8 (2.5×) | 10.0 (2.1×) |
| add_u64 | BN254 scalar | 24.8 | 118 (4.8×) | 52.8 (2.1×) | 143 (5.8×) | 89.3 (3.6×) |
| add_u64 | secp256k1 base | 5.8 | 8.0 (1.4×) | 8.0 (1.4×) | 8.0 (1.4×) | 11.0 (1.9×) |
| add_u64 | P-256 base | 25.7 | 71.3 (2.8×) | 36.0 (1.4×) | 87.0 (3.4×) | 66.6 (2.6×) |
| add_u64 | P-256 scalar | 25.4 | 70.6 (2.8×) | 40.6 (1.6×) | 74.3 (2.9×) | 84.2 (3.3×) |
| add_u64 | Tom-256 base | 26.1 | 88.5 (3.4×) | 42.0 (1.6×) | 108 (4.1×) | 71.6 (2.7×) |
| add_u64 | BLS12-381 base | 38.2 | 167 (4.4×) | 66.7 (1.7×) | 255 (6.7×) | 95.8 (2.5×) |
| add_u64 | P-521 base | 10.7 | 20.8 (1.9×) | 15.4 (1.4×) | 22.1 (2.1×) | 23.4 (2.2×) |
| add_uint | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.4 (1.2×) | 5.4 (1.2×) |
| add_uint | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_uint | Goldilocks | 4.7 | 9.3 (2.0×) | 7.0 (1.5×) | 11.8 (2.5×) | 10.0 (2.1×) |
| add_uint | BN254 scalar | 72.0 | 248 (3.4×) | 91.4 (1.3×) | 360 (5.0×) | 191 (2.7×) |
| add_uint | secp256k1 base | 6.8 | 20.9 (3.1×) | 14.6 (2.1×) | 18.1 (2.7×) | 24.3 (3.6×) |
| add_uint | P-256 base | 57.3 | 188 (3.3×) | 74.9 (1.3×) | 325 (5.7×) | 147 (2.6×) |
| add_uint | P-256 scalar | 81.0 | 260 (3.2×) | 96.7 (1.2×) | 336 (4.1×) | 223 (2.8×) |
| add_uint | Tom-256 base | 73.8 | 251 (3.4×) | 99.2 (1.3×) | 333 (4.5×) | 190 (2.6×) |
| add_uint | BLS12-381 base | 152 | 768 (5.1×) | 199 (1.3×) | 951 (6.3×) | 409 (2.7×) |
| add_uint | P-521 base | 23.2 | 65.4 (2.8×) | 41.9 (1.8×) | 89.8 (3.9×) | 76.3 (3.3×) |
| sub_u64 | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64 | BabyBear | 5.9 | 11.0 (1.9×) | 8.5 (1.4×) | 13.0 (2.2×) | 13.0 (2.2×) |
| sub_u64 | Goldilocks | 4.3 | 9.5 (2.2×) | 6.7 (1.6×) | 11.6 (2.7×) | 9.2 (2.1×) |
| sub_u64 | BN254 scalar | 24.8 | 109 (4.4×) | 54.7 (2.2×) | 137 (5.5×) | 73.3 (3.0×) |
| sub_u64 | secp256k1 base | 4.2 | 8.7 (2.1×) | 7.2 (1.7×) | 7.5 (1.8×) | 9.2 (2.2×) |
| sub_u64 | P-256 base | 19.8 | 70.6 (3.6×) | 34.9 (1.8×) | 80.0 (4.0×) | 61.3 (3.1×) |
| sub_u64 | P-256 scalar | 24.9 | 68.0 (2.7×) | 39.1 (1.6×) | 74.2 (3.0×) | 87.3 (3.5×) |
| sub_u64 | Tom-256 base | 24.6 | 87.7 (3.6×) | 50.2 (2.0×) | 104 (4.2×) | 74.1 (3.0×) |
| sub_u64 | BLS12-381 base | 37.8 | 153 (4.0×) | 65.4 (1.7×) | 250 (6.6×) | 98.9 (2.6×) |
| sub_u64 | P-521 base | 9.6 | 22.3 (2.3×) | 18.0 (1.9×) | 20.2 (2.1×) | 20.1 (2.1×) |
| sub_uint | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.1 (1.1×) | 5.2 (1.1×) |
| sub_uint | BabyBear | 5.8 | 11.1 (1.9×) | 8.5 (1.5×) | 13.0 (2.2×) | 13.1 (2.3×) |
| sub_uint | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.6 (2.7×) | 9.2 (2.1×) |
| sub_uint | BN254 scalar | 74.6 | 246 (3.3×) | 92.3 (1.2×) | 364 (4.9×) | 191 (2.6×) |
| sub_uint | secp256k1 base | 5.4 | 21.2 (3.9×) | 13.6 (2.5×) | 19.3 (3.6×) | 23.4 (4.3×) |
| sub_uint | P-256 base | 53.1 | 170 (3.2×) | 80.9 (1.5×) | 295 (5.6×) | 143 (2.7×) |
| sub_uint | P-256 scalar | 79.9 | 249 (3.1×) | 103 (1.3×) | 331 (4.1×) | 199 (2.5×) |
| sub_uint | Tom-256 base | 76.0 | 251 (3.3×) | 93.5 (1.2×) | 325 (4.3×) | 191 (2.5×) |
| sub_uint | BLS12-381 base | 155 | 762 (4.9×) | 208 (1.3×) | 939 (6.1×) | 428 (2.8×) |
| sub_uint | P-521 base | 19.9 | 75.4 (3.8×) | 40.0 (2.0×) | 89.0 (4.5×) | 69.2 (3.5×) |
| from_u64 | Mersenne-31 | 3.2 | 4.0 (1.2×) | 4.0 (1.2×) | 4.1 (1.3×) | 4.1 (1.3×) |
| from_u64 | BabyBear | 6.3 | 11.1 (1.8×) | 7.3 (1.2×) | 12.7 (2.0×) | 10.3 (1.6×) |
| from_u64 | Goldilocks | 3.4 | 7.4 (2.2×) | 5.9 (1.7×) | 9.1 (2.7×) | 7.1 (2.1×) |
| from_u64 | BN254 scalar | 22.0 | 101 (4.6×) | 29.2 (1.3×) | 117 (5.3×) | 50.2 (2.3×) |
| from_u64 | secp256k1 base | 2.0 | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) |
| from_u64 | P-256 base | 17.6 | 56.7 (3.2×) | 24.6 (1.4×) | 70.3 (4.0×) | 41.1 (2.3×) |
| from_u64 | P-256 scalar | 18.0 | 55.1 (3.1×) | 26.2 (1.5×) | 58.3 (3.2×) | 49.4 (2.7×) |
| from_u64 | Tom-256 base | 21.0 | 74.6 (3.6×) | 28.2 (1.3×) | 86.1 (4.1×) | 48.8 (2.3×) |
| from_u64 | BLS12-381 base | 29.1 | 144 (4.9×) | 50.0 (1.7×) | 182 (6.3×) | 70.6 (2.4×) |
| from_u64 | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.1 (1.2×) |
| div_u64 | BN254 scalar | 612 | 1,212 (2.0×) | 839 (1.4×) | 1,365 (2.2×) | 1,144 (1.9×) |
| div_u64 | secp256k1 base | 564 | 1,008 (1.8×) | 712 (1.3×) | 1,234 (2.2×) | 801 (1.4×) |
| div_u64 | P-256 base | 602 | 1,091 (1.8×) | 804 (1.3×) | 1,353 (2.2×) | 981 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,188 (1.9×) | 847 (1.4×) | 1,427 (2.3×) | 1,129 (1.8×) |
| div_u64 | Tom-256 base | 608 | 1,210 (2.0×) | 849 (1.4×) | 1,499 (2.5×) | 1,064 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,803 (2.4×) | 1,076 (1.4×) | 2,479 (3.3×) | 1,789 (2.4×) |
| div_u64 | P-521 base | 925 | 2,452 (2.7×) | 1,001 (1.1×) | 3,585 (3.9×) | 1,454 (1.6×) |
| div_uint | Mersenne-31 | 215 | 273 (1.3×) | 275 (1.3×) | 282 (1.3×) | 285 (1.3×) |
| div_uint | BabyBear | 215 | 276 (1.3×) | 274 (1.3×) | 282 (1.3×) | 280 (1.3×) |
| div_uint | Goldilocks | 435 | 554 (1.3×) | 540 (1.2×) | 591 (1.4×) | 575 (1.3×) |
| div_uint | BN254 scalar | 5,014 | 9,251 (1.8×) | 6,586 (1.3×) | 12,719 (2.5×) | 8,516 (1.7×) |
| div_uint | secp256k1 base | 4,901 | 8,607 (1.8×) | 6,038 (1.2×) | 12,338 (2.5×) | 8,186 (1.7×) |
| div_uint | P-256 base | 5,029 | 8,817 (1.8×) | 6,202 (1.2×) | 12,107 (2.4×) | 7,880 (1.6×) |
| div_uint | P-256 scalar | 4,958 | 9,217 (1.9×) | 6,565 (1.3×) | 12,531 (2.5×) | 8,660 (1.7×) |
| div_uint | Tom-256 base | 5,057 | 9,097 (1.8×) | 6,614 (1.3×) | 12,210 (2.4×) | 8,421 (1.7×) |
| div_uint | BLS12-381 base | 9,020 | 20,969 (2.3×) | 12,030 (1.3×) | 26,343 (2.9×) | 17,298 (1.9×) |
| div_uint | P-521 base | 14,921 | 34,083 (2.3×) | 18,423 (1.2×) | 49,809 (3.3×) | 30,655 (2.1×) |
| add, constant time | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add, constant time | BabyBear | 1.9 | 2.4 (1.3×) | 2.7 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add, constant time | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.1 (1.6×) | 3.2 (1.6×) |
| add, constant time | BN254 scalar | 4.9 | 23.7 (4.8×) | 16.2 (3.3×) | 20.7 (4.2×) | 26.1 (5.3×) |
| add, constant time | secp256k1 base | 5.8 | 26.2 (4.5×) | 15.9 (2.7×) | 24.5 (4.2×) | 28.0 (4.8×) |
| add, constant time | P-256 base | 6.0 | 23.6 (3.9×) | 15.6 (2.6×) | 22.5 (3.8×) | 27.0 (4.5×) |
| add, constant time | P-256 scalar | 5.8 | 27.1 (4.7×) | 16.1 (2.8×) | 23.9 (4.1×) | 28.5 (4.9×) |
| add, constant time | Tom-256 base | 6.0 | 27.7 (4.6×) | 16.1 (2.7×) | 24.2 (4.0×) | 29.1 (4.9×) |
| add, constant time | BLS12-381 base | 9.6 | 42.8 (4.5×) | 24.3 (2.5×) | 35.6 (3.7×) | 40.1 (4.2×) |
| add, constant time | P-521 base | 12.2 | 76.8 (6.3×) | 40.7 (3.3×) | 75.7 (6.2×) | 72.0 (5.9×) |
| sub, constant time | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BabyBear | 1.9 | 2.5 (1.3×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.1 (1.6×) |
| sub, constant time | BN254 scalar | 5.0 | 21.1 (4.2×) | 14.5 (2.9×) | 20.6 (4.1×) | 23.1 (4.6×) |
| sub, constant time | secp256k1 base | 5.3 | 19.1 (3.6×) | 13.6 (2.6×) | 18.7 (3.5×) | 22.0 (4.2×) |
| sub, constant time | P-256 base | 5.3 | 17.3 (3.3×) | 12.5 (2.4×) | 18.4 (3.5×) | 21.0 (4.0×) |
| sub, constant time | P-256 scalar | 5.0 | 21.0 (4.2×) | 14.3 (2.9×) | 21.0 (4.2×) | 23.0 (4.6×) |
| sub, constant time | Tom-256 base | 5.5 | 21.0 (3.8×) | 15.0 (2.7×) | 20.9 (3.8×) | 23.1 (4.2×) |
| sub, constant time | BLS12-381 base | 9.7 | 37.6 (3.9×) | 21.3 (2.2×) | 34.7 (3.6×) | 36.0 (3.7×) |
| sub, constant time | P-521 base | 11.7 | 56.3 (4.8×) | 31.2 (2.7×) | 58.7 (5.0×) | 51.0 (4.4×) |
| mul, constant time | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul, constant time | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul, constant time | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 15.0 (3.9×) | 7.0 (1.8×) |
| mul, constant time | BN254 scalar | 67.8 | 237 (3.5×) | 88.1 (1.3×) | 416 (6.1×) | 161 (2.4×) |
| mul, constant time | secp256k1 base | 44.6 | 160 (3.6×) | 59.6 (1.3×) | 226 (5.1×) | 107 (2.4×) |
| mul, constant time | P-256 base | 49.6 | 171 (3.4×) | 71.6 (1.4×) | 286 (5.8×) | 121 (2.4×) |
| mul, constant time | P-256 scalar | 75.0 | 239 (3.2×) | 89.8 (1.2×) | 332 (4.4×) | 175 (2.3×) |
| mul, constant time | Tom-256 base | 70.0 | 236 (3.4×) | 80.7 (1.2×) | 317 (4.5×) | 164 (2.3×) |
| mul, constant time | BLS12-381 base | 141 | 725 (5.1×) | 187 (1.3×) | 916 (6.5×) | 333 (2.4×) |
| mul, constant time | P-521 base | 176 | 667 (3.8×) | 245 (1.4×) | 1,281 (7.3×) | 534 (3.0×) |
| square, constant time | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.7 (1.4×) |
| square, constant time | BabyBear | 3.5 | 4.7 (1.3×) | 4.8 (1.4×) | 5.7 (1.6×) | 5.4 (1.5×) |
| square, constant time | Goldilocks | 3.8 | 8.6 (2.3×) | 6.0 (1.6×) | 13.0 (3.4×) | 8.1 (2.1×) |
| square, constant time | BN254 scalar | 64.9 | 213 (3.3×) | 86.4 (1.3×) | 177 (2.7×) | 159 (2.4×) |
| square, constant time | secp256k1 base | 33.8 | 144 (4.3×) | 53.1 (1.6×) | 171 (5.1×) | 91.5 (2.7×) |
| square, constant time | P-256 base | 40.3 | 148 (3.7×) | 74.3 (1.8×) | 259 (6.4×) | 125 (3.1×) |
| square, constant time | P-256 scalar | 64.4 | 215 (3.3×) | 77.9 (1.2×) | 250 (3.9×) | 166 (2.6×) |
| square, constant time | Tom-256 base | 57.0 | 207 (3.6×) | 72.8 (1.3×) | 227 (4.0×) | 129 (2.3×) |
| square, constant time | BLS12-381 base | 134 | 567 (4.2×) | 158 (1.2×) | 824 (6.1×) | 296 (2.2×) |
| square, constant time | P-521 base | 118 | 563 (4.8×) | 238 (2.0×) | 912 (7.7×) | 391 (3.3×) |
| invert, constant time | Mersenne-31 | 243 | 263 (1.1×) | 264 (1.1×) | 271 (1.1×) | 267 (1.1×) |
| invert, constant time | BabyBear | 383 | 451 (1.2×) | 458 (1.2×) | 447 (1.2×) | 418 (1.1×) |
| invert, constant time | Goldilocks | 648 | 1,154 (1.8×) | 841 (1.3×) | 1,420 (2.2×) | 936 (1.4×) |
| invert, constant time | BN254 scalar | 6,758 | 11,983 (1.8×) | 9,358 (1.4×) | 17,132 (2.5×) | 12,564 (1.9×) |
| invert, constant time | secp256k1 base | 6,860 | 11,830 (1.7×) | 9,051 (1.3×) | 16,792 (2.4×) | 12,577 (1.8×) |
| invert, constant time | P-256 base | 6,485 | 11,489 (1.8×) | 8,945 (1.4×) | 16,378 (2.5×) | 12,433 (1.9×) |
| invert, constant time | P-256 scalar | 6,909 | 11,870 (1.7×) | 9,413 (1.4×) | 17,061 (2.5×) | 12,746 (1.8×) |
| invert, constant time | Tom-256 base | 6,973 | 11,943 (1.7×) | 9,282 (1.3×) | 16,839 (2.4×) | 12,806 (1.8×) |
| invert, constant time | BLS12-381 base | 12,456 | 24,033 (1.9×) | 16,674 (1.3×) | 32,779 (2.6×) | 24,086 (1.9×) |
| invert, constant time | P-521 base | 20,025 | 41,710 (2.1×) | 26,021 (1.3×) | 62,818 (3.1×) | 41,805 (2.1×) |
| sqrt, constant time | Mersenne-31 | 142 | 159 (1.1×) | 159 (1.1×) | 170 (1.2×) | 171 (1.2×) |
| sqrt, constant time | BabyBear | 5,592 | 7,130 (1.3×) | 6,446 (1.2×) | 6,762 (1.2×) | 6,529 (1.2×) |
| sqrt, constant time | Goldilocks | 7,149 | 10,865 (1.5×) | 8,377 (1.2×) | 14,626 (2.0×) | 10,334 (1.4×) |
| sqrt, constant time | BN254 scalar | 58,556 | 180,163 (3.1×) | 84,788 (1.4×) | 293,067 (5.0×) | 149,268 (2.5×) |
| sqrt, constant time | secp256k1 base | 12,274 | 41,221 (3.4×) | 16,810 (1.4×) | 66,520 (5.4×) | 26,663 (2.2×) |
| sqrt, constant time | P-256 base | 12,041 | 43,007 (3.6×) | 15,245 (1.3×) | 74,750 (6.2×) | 31,055 (2.6×) |
| sqrt, constant time | P-256 scalar | 25,625 | 77,950 (3.0×) | 34,643 (1.4×) | 117,621 (4.6×) | 66,290 (2.6×) |
| sqrt, constant time | Tom-256 base | 20,048 | 66,073 (3.3×) | 29,280 (1.5×) | 98,912 (4.9×) | 50,967 (2.5×) |
| sqrt, constant time | BLS12-381 base | 69,784 | 297,973 (4.3×) | 93,625 (1.3×) | 392,535 (5.6×) | 206,963 (3.0×) |
| sqrt, constant time | P-521 base | 67,949 | 324,093 (4.8×) | 124,267 (1.8×) | 507,512 (7.5×) | 255,664 (3.8×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 6.7 (1.1×) | 6.7 (1.1×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_u64, constant time | BabyBear | 8.9 | 13.1 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 15.1 (1.7×) |
| mul_u64, constant time | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.2 (1.3×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.1 (1.7×) | 125 (5.7×) | 56.3 (2.6×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 61.5 (5.6×) | 22.3 (2.0×) | 79.9 (7.3×) | 36.2 (3.3×) |
| mul_u64, constant time | P-256 base | 24.4 | 85.4 (3.5×) | 37.4 (1.5×) | 103 (4.2×) | 59.5 (2.4×) |
| mul_u64, constant time | P-256 scalar | 22.9 | 102 (4.5×) | 41.4 (1.8×) | 122 (5.3×) | 66.2 (2.9×) |
| mul_u64, constant time | Tom-256 base | 22.4 | 95.2 (4.2×) | 41.0 (1.8×) | 122 (5.4×) | 71.0 (3.2×) |
| mul_u64, constant time | BLS12-381 base | 34.9 | 149 (4.3×) | 62.6 (1.8×) | 287 (8.2×) | 118 (3.4×) |
| mul_u64, constant time | P-521 base | 26.1 | 138 (5.3×) | 64.3 (2.5×) | 192 (7.4×) | 90.0 (3.4×) |
| mul_uint, constant time | Mersenne-31 | 5.8 | 6.7 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_uint, constant time | BabyBear | 8.9 | 13.0 (1.5×) | 11.1 (1.2×) | 18.0 (2.0×) | 15.0 (1.7×) |
| mul_uint, constant time | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_uint, constant time | BN254 scalar | 119 | 486 (4.1×) | 167 (1.4×) | 592 (5.0×) | 310 (2.6×) |
| mul_uint, constant time | secp256k1 base | 78.2 | 322 (4.1×) | 81.1 (1.0×) | 379 (4.8×) | 125 (1.6×) |
| mul_uint, constant time | P-256 base | 118 | 413 (3.5×) | 131 (1.1×) | 493 (4.2×) | 227 (1.9×) |
| mul_uint, constant time | P-256 scalar | 132 | 473 (3.6×) | 181 (1.4×) | 548 (4.2×) | 380 (2.9×) |
| mul_uint, constant time | Tom-256 base | 122 | 465 (3.8×) | 154 (1.3×) | 554 (4.5×) | 329 (2.7×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,095 (4.3×) | 377 (1.5×) | 1,701 (6.6×) | 935 (3.7×) |
| mul_uint, constant time | P-521 base | 426 | 1,999 (4.7×) | 290 (0.7×) | 2,936 (6.9×) | 500 (1.2×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.7 (1.3×) | 5.5 (1.2×) |
| add_u64, constant time | BabyBear | 5.4 | 10.3 (1.9×) | 7.4 (1.4×) | 14.8 (2.7×) | 11.6 (2.1×) |
| add_u64, constant time | Goldilocks | 4.8 | 9.3 (1.9×) | 7.1 (1.5×) | 12.0 (2.5×) | 10.0 (2.1×) |
| add_u64, constant time | BN254 scalar | 23.5 | 93.5 (4.0×) | 42.0 (1.8×) | 111 (4.7×) | 81.9 (3.5×) |
| add_u64, constant time | secp256k1 base | 5.8 | 22.4 (3.9×) | 14.2 (2.4×) | 20.2 (3.5×) | 24.1 (4.2×) |
| add_u64, constant time | P-256 base | 16.9 | 71.0 (4.2×) | 36.0 (2.1×) | 78.9 (4.7×) | 58.0 (3.4×) |
| add_u64, constant time | P-256 scalar | 23.9 | 92.2 (3.9×) | 43.1 (1.8×) | 113 (4.7×) | 87.0 (3.6×) |
| add_u64, constant time | Tom-256 base | 22.3 | 94.4 (4.2×) | 40.9 (1.8×) | 111 (5.0×) | 77.6 (3.5×) |
| add_u64, constant time | BLS12-381 base | 36.6 | 147 (4.0×) | 69.4 (1.9×) | 170 (4.6×) | 112 (3.1×) |
| add_u64, constant time | P-521 base | 10.7 | 64.2 (6.0×) | 39.1 (3.7×) | 70.3 (6.6×) | 62.1 (5.8×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.7 (1.3×) | 5.2 (1.2×) |
| add_uint, constant time | BabyBear | 5.4 | 10.4 (1.9×) | 7.4 (1.4×) | 14.8 (2.7×) | 12.1 (2.2×) |
| add_uint, constant time | Goldilocks | 4.7 | 9.3 (2.0×) | 7.1 (1.5×) | 12.0 (2.6×) | 9.9 (2.1×) |
| add_uint, constant time | BN254 scalar | 75.8 | 249 (3.3×) | 93.3 (1.2×) | 303 (4.0×) | 186 (2.5×) |
| add_uint, constant time | secp256k1 base | 9.3 | 41.7 (4.5×) | 25.0 (2.7×) | 37.5 (4.0×) | 40.9 (4.4×) |
| add_uint, constant time | P-256 base | 57.2 | 185 (3.2×) | 96.2 (1.7×) | 265 (4.6×) | 149 (2.6×) |
| add_uint, constant time | P-256 scalar | 83.2 | 260 (3.1×) | 101 (1.2×) | 325 (3.9×) | 196 (2.4×) |
| add_uint, constant time | Tom-256 base | 73.7 | 251 (3.4×) | 94.8 (1.3×) | 333 (4.5×) | 173 (2.3×) |
| add_uint, constant time | BLS12-381 base | 156 | 769 (4.9×) | 204 (1.3×) | 961 (6.2×) | 426 (2.7×) |
| add_uint, constant time | P-521 base | 22.3 | 135 (6.1×) | 80.3 (3.6×) | 135 (6.1×) | 129 (5.8×) |
| sub_u64, constant time | Mersenne-31 | 4.6 | 5.0 (1.1×) | 5.0 (1.1×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64, constant time | BabyBear | 5.2 | 11.3 (2.2×) | 7.5 (1.4×) | 14.0 (2.7×) | 10.8 (2.1×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.6 (2.2×) |
| sub_u64, constant time | BN254 scalar | 25.4 | 91.3 (3.6×) | 38.0 (1.5×) | 110 (4.3×) | 67.6 (2.7×) |
| sub_u64, constant time | secp256k1 base | 4.2 | 15.6 (3.7×) | 11.1 (2.6×) | 14.6 (3.5×) | 17.8 (4.2×) |
| sub_u64, constant time | P-256 base | 16.2 | 64.1 (4.0×) | 31.8 (2.0×) | 75.5 (4.7×) | 53.1 (3.3×) |
| sub_u64, constant time | P-256 scalar | 24.1 | 88.4 (3.7×) | 37.5 (1.6×) | 107 (4.4×) | 67.0 (2.8×) |
| sub_u64, constant time | Tom-256 base | 22.9 | 94.5 (4.1×) | 37.8 (1.7×) | 107 (4.7×) | 66.4 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 36.5 | 143 (3.9×) | 60.1 (1.6×) | 185 (5.1×) | 117 (3.2×) |
| sub_u64, constant time | P-521 base | 9.6 | 42.4 (4.4×) | 27.7 (2.9×) | 52.0 (5.4×) | 40.5 (4.2×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.2 (1.1×) |
| sub_uint, constant time | BabyBear | 5.2 | 11.3 (2.2×) | 7.5 (1.4×) | 14.0 (2.7×) | 11.4 (2.2×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.5 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.4 (2.2×) |
| sub_uint, constant time | BN254 scalar | 77.7 | 248 (3.2×) | 92.3 (1.2×) | 380 (4.9×) | 176 (2.3×) |
| sub_uint, constant time | secp256k1 base | 7.2 | 32.8 (4.6×) | 21.5 (3.0×) | 33.8 (4.7×) | 35.7 (5.0×) |
| sub_uint, constant time | P-256 base | 53.1 | 171 (3.2×) | 89.6 (1.7×) | 297 (5.6×) | 140 (2.6×) |
| sub_uint, constant time | P-256 scalar | 80.0 | 250 (3.1×) | 101 (1.3×) | 362 (4.5×) | 202 (2.5×) |
| sub_uint, constant time | Tom-256 base | 74.9 | 253 (3.4×) | 96.2 (1.3×) | 323 (4.3×) | 186 (2.5×) |
| sub_uint, constant time | BLS12-381 base | 154 | 757 (4.9×) | 202 (1.3×) | 950 (6.2×) | 363 (2.4×) |
| sub_uint, constant time | P-521 base | 19.0 | 126 (6.6×) | 66.8 (3.5×) | 123 (6.5×) | 110 (5.8×) |
| from_u64, constant time | Mersenne-31 | 3.8 | 4.7 (1.2×) | 4.7 (1.2×) | 5.2 (1.4×) | 5.2 (1.4×) |
| from_u64, constant time | BabyBear | 4.9 | 10.0 (2.0×) | 6.8 (1.4×) | 13.3 (2.7×) | 10.5 (2.1×) |
| from_u64, constant time | Goldilocks | 3.4 | 7.4 (2.2×) | 5.9 (1.7×) | 9.0 (2.6×) | 7.1 (2.1×) |
| from_u64, constant time | BN254 scalar | 26.2 | 83.8 (3.2×) | 34.7 (1.3×) | 99.3 (3.8×) | 50.4 (1.9×) |
| from_u64, constant time | secp256k1 base | 2.0 | 3.8 (1.9×) | 3.8 (1.9×) | 3.9 (1.9×) | 3.8 (1.9×) |
| from_u64, constant time | P-256 base | 18.6 | 51.3 (2.8×) | 26.7 (1.4×) | 61.4 (3.3×) | 38.7 (2.1×) |
| from_u64, constant time | P-256 scalar | 27.2 | 84.2 (3.1×) | 34.5 (1.3×) | 97.0 (3.6×) | 50.5 (1.9×) |
| from_u64, constant time | Tom-256 base | 23.5 | 82.0 (3.5×) | 35.4 (1.5×) | 92.5 (3.9×) | 49.2 (2.1×) |
| from_u64, constant time | BLS12-381 base | 41.9 | 145 (3.5×) | 56.6 (1.4×) | 173 (4.1×) | 95.5 (2.3×) |
| from_u64, constant time | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.1 (1.2×) |
| div_u64, constant time | BN254 scalar | 6,851 | 12,276 (1.8×) | 9,430 (1.4×) | 17,544 (2.6×) | 13,374 (2.0×) |
| div_u64, constant time | secp256k1 base | 6,776 | 11,959 (1.8×) | 9,110 (1.3×) | 17,272 (2.5×) | 12,802 (1.9×) |
| div_u64, constant time | P-256 base | 6,666 | 11,815 (1.8×) | 9,118 (1.4×) | 17,179 (2.6×) | 12,457 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,956 | 12,280 (1.8×) | 9,404 (1.4×) | 17,658 (2.5×) | 13,396 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,282 | 12,086 (1.7×) | 9,318 (1.3×) | 16,929 (2.3×) | 12,962 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,384 | 24,775 (2.0×) | 16,918 (1.4×) | 34,672 (2.8×) | 24,767 (2.0×) |
| div_u64, constant time | P-521 base | 20,382 | 42,336 (2.1×) | 25,940 (1.3×) | 64,180 (3.1×) | 43,108 (2.1×) |
| div_uint, constant time | Mersenne-31 | 262 | 272 (1.0×) | 272 (1.0×) | 281 (1.1×) | 280 (1.1×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 465 (1.2×) | 473 (1.2×) | 445 (1.1×) |
| div_uint, constant time | Goldilocks | 668 | 1,145 (1.7×) | 862 (1.3×) | 1,438 (2.2×) | 960 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,895 | 12,518 (1.8×) | 9,819 (1.4×) | 17,747 (2.6×) | 13,379 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,771 | 12,005 (1.8×) | 9,096 (1.3×) | 17,159 (2.5×) | 12,836 (1.9×) |
| div_uint, constant time | P-256 base | 6,740 | 12,045 (1.8×) | 9,092 (1.3×) | 17,578 (2.6×) | 12,614 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,952 | 12,316 (1.8×) | 9,611 (1.4×) | 17,822 (2.6×) | 13,368 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,337 | 12,498 (1.7×) | 9,455 (1.3×) | 16,958 (2.3×) | 13,197 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,490 | 25,340 (2.0×) | 16,962 (1.4×) | 35,456 (2.8×) | 24,761 (2.0×) |
| div_uint, constant time | P-521 base | 20,380 | 42,595 (2.1×) | 26,114 (1.3×) | 65,040 (3.2×) | 43,253 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.1 (1.1×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.7 | 13.2 (2.0×) | 10.9 (1.6×) | 13.7 (2.0×) | 17.4 (2.6×) |
| add | secp256k1 base | 6.9 | 13.8 (2.0×) | 10.8 (1.6×) | 13.9 (2.0×) | 18.3 (2.7×) |
| add | P-256 base | 6.9 | 12.3 (1.8×) | 11.2 (1.6×) | 13.0 (1.9×) | 18.7 (2.7×) |
| add | P-256 scalar | 6.9 | 13.6 (2.0×) | 11.1 (1.6×) | 14.0 (2.0×) | 19.4 (2.8×) |
| add | Tom-256 base | 6.9 | 13.3 (1.9×) | 11.3 (1.6×) | 13.2 (1.9×) | 18.6 (2.7×) |
| add | BLS12-381 base | 8.9 | 21.9 (2.5×) | 18.9 (2.1×) | 23.0 (2.6×) | 30.1 (3.4×) |
| add | P-521 base | 14.7 | 43.3 (2.9×) | 27.4 (1.9×) | 48.9 (3.3×) | 48.0 (3.3×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.7 | 13.1 (2.0×) | 10.7 (1.6×) | 12.7 (1.9×) | 17.4 (2.6×) |
| sub | secp256k1 base | 8.0 | 16.6 (2.1×) | 10.4 (1.3×) | 13.5 (1.7×) | 16.9 (2.1×) |
| sub | P-256 base | 7.0 | 13.3 (1.9×) | 10.4 (1.5×) | 12.5 (1.8×) | 16.5 (2.4×) |
| sub | P-256 scalar | 6.9 | 14.3 (2.1×) | 10.4 (1.5×) | 13.6 (2.0×) | 16.8 (2.4×) |
| sub | Tom-256 base | 8.0 | 13.1 (1.6×) | 10.7 (1.3×) | 12.8 (1.6×) | 17.4 (2.2×) |
| sub | BLS12-381 base | 9.9 | 22.8 (2.3×) | 17.3 (1.7×) | 22.3 (2.3×) | 29.6 (3.0×) |
| sub | P-521 base | 12.4 | 43.6 (3.5×) | 25.6 (2.1×) | 44.2 (3.6×) | 44.1 (3.6×) |
| mul | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul | BabyBear | 10.9 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul | Goldilocks | 8.9 | 16.5 (1.9×) | 12.5 (1.4×) | 18.0 (2.0×) | 12.5 (1.4×) |
| mul | BN254 scalar | 72.5 | 229 (3.2×) | 91.6 (1.3×) | 366 (5.0×) | 159 (2.2×) |
| mul | secp256k1 base | 54.8 | 173 (3.2×) | 68.9 (1.3×) | 230 (4.2×) | 106 (1.9×) |
| mul | P-256 base | 52.5 | 191 (3.6×) | 65.2 (1.2×) | 291 (5.5×) | 119 (2.3×) |
| mul | P-256 scalar | 84.0 | 240 (2.9×) | 88.5 (1.1×) | 331 (3.9×) | 159 (1.9×) |
| mul | Tom-256 base | 76.0 | 238 (3.1×) | 82.6 (1.1×) | 300 (3.9×) | 152 (2.0×) |
| mul | BLS12-381 base | 141 | 721 (5.1×) | 185 (1.3×) | 919 (6.5×) | 314 (2.2×) |
| mul | P-521 base | 191 | 687 (3.6×) | 240 (1.3×) | 1,324 (6.9×) | 438 (2.3×) |
| square | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square | Goldilocks | 9.0 | 15.0 (1.7×) | 11.0 (1.2×) | 18.7 (2.1×) | 12.5 (1.4×) |
| square | BN254 scalar | 66.7 | 211 (3.2×) | 82.4 (1.2×) | 325 (4.9×) | 153 (2.3×) |
| square | secp256k1 base | 42.9 | 151 (3.5×) | 58.8 (1.4×) | 181 (4.2×) | 88.5 (2.1×) |
| square | P-256 base | 44.0 | 159 (3.6×) | 55.6 (1.3×) | 212 (4.8×) | 99.9 (2.3×) |
| square | P-256 scalar | 72.9 | 218 (3.0×) | 81.4 (1.1×) | 230 (3.2×) | 145 (2.0×) |
| square | Tom-256 base | 64.5 | 213 (3.3×) | 79.3 (1.2×) | 212 (3.3×) | 136 (2.1×) |
| square | BLS12-381 base | 137 | 568 (4.1×) | 164 (1.2×) | 808 (5.9×) | 325 (2.4×) |
| square | P-521 base | 129 | 570 (4.4×) | 210 (1.6×) | 916 (7.1×) | 390 (3.0×) |
| mul_u64 | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64 | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.4 (1.4×) | 13.0 (1.2×) |
| mul_u64 | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 15.0 (1.6×) |
| mul_u64 | BN254 scalar | 25.9 | 91.8 (3.5×) | 34.5 (1.3×) | 121 (4.7×) | 45.1 (1.7×) |
| mul_u64 | secp256k1 base | 14.9 | 49.0 (3.3×) | 16.0 (1.1×) | 65.1 (4.4×) | 23.1 (1.6×) |
| mul_u64 | P-256 base | 24.9 | 75.2 (3.0×) | 33.0 (1.3×) | 90.1 (3.6×) | 42.0 (1.7×) |
| mul_u64 | P-256 scalar | 27.9 | 81.8 (2.9×) | 34.5 (1.2×) | 112 (4.0×) | 48.2 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.2 (3.0×) | 32.9 (1.2×) | 111 (4.0×) | 45.4 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 124 (3.6×) | 47.9 (1.4×) | 196 (5.7×) | 61.7 (1.8×) |
| mul_u64 | P-521 base | 32.7 | 94.9 (2.9×) | 39.6 (1.2×) | 185 (5.7×) | 53.1 (1.6×) |
| mul_uint | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 12.8 (1.2×) |
| mul_uint | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 15.0 (1.6×) |
| mul_uint | BN254 scalar | 114 | 440 (3.9×) | 157 (1.4×) | 538 (4.7×) | 346 (3.0×) |
| mul_uint | secp256k1 base | 67.9 | 265 (3.9×) | 68.9 (1.0×) | 319 (4.7×) | 107 (1.6×) |
| mul_uint | P-256 base | 110 | 354 (3.2×) | 130 (1.2×) | 429 (3.9×) | 230 (2.1×) |
| mul_uint | P-256 scalar | 116 | 424 (3.7×) | 176 (1.5×) | 475 (4.1×) | 333 (2.9×) |
| mul_uint | Tom-256 base | 114 | 409 (3.6×) | 160 (1.4×) | 490 (4.3×) | 341 (3.0×) |
| mul_uint | BLS12-381 base | 222 | 970 (4.4×) | 353 (1.6×) | 1,137 (5.1×) | 943 (4.2×) |
| mul_uint | P-521 base | 406 | 1,562 (3.8×) | 248 (0.6×) | 2,388 (5.9×) | 456 (1.1×) |
| add_u64 | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) | 5.0 (1.1×) |
| add_u64 | BabyBear | 7.0 | 12.3 (1.8×) | 8.2 (1.2×) | 12.7 (1.8×) | 10.9 (1.6×) |
| add_u64 | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 12.0 (2.6×) | 9.3 (2.0×) |
| add_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 44.6 (1.8×) | 139 (5.6×) | 65.6 (2.6×) |
| add_u64 | secp256k1 base | 7.4 | 7.0 (0.9×) | 6.0 (0.8×) | 8.0 (1.1×) | 10.5 (1.4×) |
| add_u64 | P-256 base | 22.1 | 67.7 (3.1×) | 34.9 (1.6×) | 77.7 (3.5×) | 56.1 (2.5×) |
| add_u64 | P-256 scalar | 22.7 | 66.7 (2.9×) | 38.2 (1.7×) | 74.7 (3.3×) | 66.3 (2.9×) |
| add_u64 | Tom-256 base | 26.0 | 84.3 (3.2×) | 39.9 (1.5×) | 94.4 (3.6×) | 65.0 (2.5×) |
| add_u64 | BLS12-381 base | 37.5 | 161 (4.3×) | 62.2 (1.7×) | 249 (6.6×) | 89.7 (2.4×) |
| add_u64 | P-521 base | 12.7 | 21.4 (1.7×) | 12.3 (1.0×) | 19.5 (1.5×) | 30.2 (2.4×) |
| add_uint | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) | 4.8 (1.1×) |
| add_uint | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 12.0 (2.6×) | 9.1 (1.9×) |
| add_uint | BN254 scalar | 73.1 | 225 (3.1×) | 99.6 (1.4×) | 374 (5.1×) | 176 (2.4×) |
| add_uint | secp256k1 base | 8.5 | 17.3 (2.0×) | 12.6 (1.5×) | 16.9 (2.0×) | 23.0 (2.7×) |
| add_uint | P-256 base | 57.8 | 180 (3.1×) | 73.4 (1.3×) | 326 (5.6×) | 148 (2.6×) |
| add_uint | P-256 scalar | 78.1 | 260 (3.3×) | 99.1 (1.3×) | 306 (3.9×) | 200 (2.6×) |
| add_uint | Tom-256 base | 75.2 | 253 (3.4×) | 96.1 (1.3×) | 329 (4.4×) | 177 (2.4×) |
| add_uint | BLS12-381 base | 151 | 765 (5.1×) | 196 (1.3×) | 953 (6.3×) | 389 (2.6×) |
| add_uint | P-521 base | 23.6 | 61.8 (2.6×) | 41.5 (1.8×) | 85.8 (3.6×) | 66.5 (2.8×) |
| sub_u64 | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.8 (1.1×) |
| sub_u64 | BabyBear | 7.0 | 12.0 (1.7×) | 8.2 (1.2×) | 12.1 (1.7×) | 10.9 (1.6×) |
| sub_u64 | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 10.5 (2.5×) | 8.7 (2.1×) |
| sub_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 45.8 (1.8×) | 136 (5.5×) | 66.1 (2.7×) |
| sub_u64 | secp256k1 base | 6.9 | 7.0 (1.0×) | 6.0 (0.9×) | 7.0 (1.0×) | 9.1 (1.3×) |
| sub_u64 | P-256 base | 21.7 | 69.5 (3.2×) | 34.0 (1.6×) | 79.0 (3.6×) | 57.2 (2.6×) |
| sub_u64 | P-256 scalar | 23.7 | 66.9 (2.8×) | 36.7 (1.5×) | 72.5 (3.1×) | 61.9 (2.6×) |
| sub_u64 | Tom-256 base | 25.0 | 85.2 (3.4×) | 39.6 (1.6×) | 97.3 (3.9×) | 66.1 (2.6×) |
| sub_u64 | BLS12-381 base | 38.6 | 151 (3.9×) | 60.3 (1.6×) | 265 (6.9×) | 90.5 (2.3×) |
| sub_u64 | P-521 base | 11.0 | 21.1 (1.9×) | 13.1 (1.2×) | 21.4 (1.9×) | 20.9 (1.9×) |
| sub_uint | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.7 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_uint | BabyBear | 7.0 | 12.0 (1.7×) | 8.2 (1.2×) | 12.1 (1.7×) | 11.3 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 10.5 (2.5×) | 9.0 (2.1×) |
| sub_uint | BN254 scalar | 73.1 | 232 (3.2×) | 95.7 (1.3×) | 353 (4.8×) | 181 (2.5×) |
| sub_uint | secp256k1 base | 6.8 | 18.9 (2.8×) | 12.0 (1.8×) | 16.4 (2.4×) | 21.3 (3.1×) |
| sub_uint | P-256 base | 56.8 | 175 (3.1×) | 70.3 (1.2×) | 248 (4.4×) | 136 (2.4×) |
| sub_uint | P-256 scalar | 77.7 | 253 (3.3×) | 98.8 (1.3×) | 329 (4.2×) | 192 (2.5×) |
| sub_uint | Tom-256 base | 72.6 | 250 (3.4×) | 93.0 (1.3×) | 328 (4.5×) | 165 (2.3×) |
| sub_uint | BLS12-381 base | 155 | 761 (4.9×) | 196 (1.3×) | 934 (6.0×) | 384 (2.5×) |
| sub_uint | P-521 base | 15.5 | 67.4 (4.3×) | 37.6 (2.4×) | 76.6 (4.9×) | 65.1 (4.2×) |
| div_u64 | BN254 scalar | 604 | 1,203 (2.0×) | 839 (1.4×) | 1,369 (2.3×) | 1,117 (1.8×) |
| div_u64 | secp256k1 base | 566 | 1,013 (1.8×) | 715 (1.3×) | 1,230 (2.2×) | 806 (1.4×) |
| div_u64 | P-256 base | 603 | 1,091 (1.8×) | 800 (1.3×) | 1,343 (2.2×) | 977 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,184 (1.9×) | 842 (1.4×) | 1,418 (2.3×) | 1,127 (1.8×) |
| div_u64 | Tom-256 base | 609 | 1,199 (2.0×) | 843 (1.4×) | 1,492 (2.4×) | 1,104 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,790 (2.4×) | 1,075 (1.4×) | 2,466 (3.3×) | 1,799 (2.4×) |
| div_u64 | P-521 base | 924 | 2,453 (2.7×) | 998 (1.1×) | 3,622 (3.9×) | 1,518 (1.6×) |
| div_uint | Mersenne-31 | 217 | 273 (1.3×) | 275 (1.3×) | 278 (1.3×) | 282 (1.3×) |
| div_uint | BabyBear | 217 | 275 (1.3×) | 278 (1.3×) | 277 (1.3×) | 280 (1.3×) |
| div_uint | Goldilocks | 435 | 550 (1.3×) | 538 (1.2×) | 584 (1.3×) | 569 (1.3×) |
| div_uint | BN254 scalar | 4,982 | 9,073 (1.8×) | 6,605 (1.3×) | 12,557 (2.5×) | 8,481 (1.7×) |
| div_uint | secp256k1 base | 4,882 | 8,598 (1.8×) | 6,099 (1.2×) | 12,189 (2.5×) | 8,095 (1.7×) |
| div_uint | P-256 base | 5,018 | 8,725 (1.7×) | 6,170 (1.2×) | 12,069 (2.4×) | 7,902 (1.6×) |
| div_uint | P-256 scalar | 4,943 | 9,144 (1.8×) | 6,556 (1.3×) | 12,449 (2.5×) | 8,667 (1.8×) |
| div_uint | Tom-256 base | 5,051 | 8,902 (1.8×) | 6,467 (1.3×) | 12,217 (2.4×) | 8,368 (1.7×) |
| div_uint | BLS12-381 base | 9,014 | 20,889 (2.3×) | 12,113 (1.3×) | 26,257 (2.9×) | 17,084 (1.9×) |
| div_uint | P-521 base | 14,980 | 34,490 (2.3×) | 18,103 (1.2×) | 49,529 (3.3×) | 30,346 (2.0×) |
| add, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.7 | 25.5 (3.8×) | 16.0 (2.4×) | 23.1 (3.4×) | 26.1 (3.9×) |
| add, constant time | secp256k1 base | 6.9 | 28.2 (4.1×) | 15.6 (2.3×) | 26.8 (3.9×) | 28.5 (4.1×) |
| add, constant time | P-256 base | 7.0 | 26.0 (3.7×) | 15.0 (2.1×) | 24.7 (3.5×) | 27.1 (3.9×) |
| add, constant time | P-256 scalar | 6.8 | 27.4 (4.0×) | 17.7 (2.6×) | 26.9 (4.0×) | 28.4 (4.2×) |
| add, constant time | Tom-256 base | 7.0 | 27.7 (4.0×) | 16.6 (2.4×) | 27.4 (3.9×) | 28.4 (4.1×) |
| add, constant time | BLS12-381 base | 8.9 | 41.4 (4.7×) | 23.3 (2.6×) | 35.2 (4.0×) | 40.6 (4.6×) |
| add, constant time | P-521 base | 15.2 | 76.5 (5.0×) | 39.5 (2.6×) | 82.6 (5.4×) | 72.0 (4.7×) |
| sub, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 6.8 | 21.5 (3.2×) | 13.3 (2.0×) | 20.8 (3.1×) | 21.5 (3.2×) |
| sub, constant time | secp256k1 base | 8.0 | 21.3 (2.7×) | 12.3 (1.5×) | 20.3 (2.5×) | 21.0 (2.6×) |
| sub, constant time | P-256 base | 6.7 | 19.2 (2.9×) | 12.8 (1.9×) | 19.6 (2.9×) | 19.0 (2.8×) |
| sub, constant time | P-256 scalar | 6.8 | 21.6 (3.2×) | 13.8 (2.0×) | 19.2 (2.8×) | 21.0 (3.1×) |
| sub, constant time | Tom-256 base | 8.3 | 21.3 (2.6×) | 13.3 (1.6×) | 19.9 (2.4×) | 21.1 (2.5×) |
| sub, constant time | BLS12-381 base | 9.6 | 37.4 (3.9×) | 21.8 (2.3×) | 36.9 (3.8×) | 35.1 (3.7×) |
| sub, constant time | P-521 base | 12.3 | 56.4 (4.6×) | 32.3 (2.6×) | 60.1 (4.9×) | 51.1 (4.2×) |
| mul, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul, constant time | Goldilocks | 8.9 | 16.5 (1.9×) | 12.5 (1.4×) | 18.0 (2.0×) | 12.5 (1.4×) |
| mul, constant time | BN254 scalar | 78.8 | 241 (3.1×) | 93.5 (1.2×) | 413 (5.2×) | 149 (1.9×) |
| mul, constant time | secp256k1 base | 55.0 | 174 (3.2×) | 68.8 (1.3×) | 237 (4.3×) | 106 (1.9×) |
| mul, constant time | P-256 base | 52.5 | 191 (3.6×) | 65.0 (1.2×) | 273 (5.2×) | 118 (2.2×) |
| mul, constant time | P-256 scalar | 84.0 | 234 (2.8×) | 87.9 (1.0×) | 329 (3.9×) | 157 (1.9×) |
| mul, constant time | Tom-256 base | 76.0 | 236 (3.1×) | 79.8 (1.1×) | 314 (4.1×) | 167 (2.2×) |
| mul, constant time | BLS12-381 base | 154 | 720 (4.7×) | 186 (1.2×) | 910 (5.9×) | 360 (2.3×) |
| mul, constant time | P-521 base | 188 | 689 (3.7×) | 241 (1.3×) | 1,314 (7.0×) | 439 (2.3×) |
| square, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square, constant time | Goldilocks | 9.0 | 15.0 (1.7×) | 11.0 (1.2×) | 18.7 (2.1×) | 12.5 (1.4×) |
| square, constant time | BN254 scalar | 66.9 | 220 (3.3×) | 82.1 (1.2×) | 203 (3.0×) | 131 (2.0×) |
| square, constant time | secp256k1 base | 42.9 | 151 (3.5×) | 58.7 (1.4×) | 181 (4.2×) | 87.2 (2.0×) |
| square, constant time | P-256 base | 44.0 | 161 (3.7×) | 55.7 (1.3×) | 210 (4.8×) | 105 (2.4×) |
| square, constant time | P-256 scalar | 72.9 | 222 (3.0×) | 81.2 (1.1×) | 230 (3.2×) | 144 (2.0×) |
| square, constant time | Tom-256 base | 64.5 | 213 (3.3×) | 77.3 (1.2×) | 235 (3.6×) | 134 (2.1×) |
| square, constant time | BLS12-381 base | 145 | 560 (3.9×) | 161 (1.1×) | 810 (5.6×) | 322 (2.2×) |
| square, constant time | P-521 base | 129 | 571 (4.4×) | 207 (1.6×) | 914 (7.1×) | 389 (3.0×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64, constant time | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 12.9 (1.2×) |
| mul_u64, constant time | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 14.9 (1.6×) |
| mul_u64, constant time | BN254 scalar | 30.4 | 109 (3.6×) | 48.3 (1.6×) | 132 (4.3×) | 52.4 (1.7×) |
| mul_u64, constant time | secp256k1 base | 19.8 | 68.9 (3.5×) | 29.0 (1.5×) | 85.1 (4.3×) | 40.8 (2.1×) |
| mul_u64, constant time | P-256 base | 33.8 | 90.2 (2.7×) | 46.5 (1.4×) | 106 (3.1×) | 70.4 (2.1×) |
| mul_u64, constant time | P-256 scalar | 33.9 | 104 (3.1×) | 46.4 (1.4×) | 147 (4.3×) | 70.7 (2.1×) |
| mul_u64, constant time | Tom-256 base | 32.9 | 99.9 (3.0×) | 48.1 (1.5×) | 121 (3.7×) | 72.7 (2.2×) |
| mul_u64, constant time | BLS12-381 base | 41.6 | 154 (3.7×) | 64.2 (1.5×) | 195 (4.7×) | 87.3 (2.1×) |
| mul_u64, constant time | P-521 base | 45.7 | 139 (3.0×) | 65.1 (1.4×) | 192 (4.2×) | 90.3 (2.0×) |
| mul_uint, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint, constant time | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.8 (1.4×) | 14.0 (1.3×) |
| mul_uint, constant time | Goldilocks | 9.0 | 17.8 (2.0×) | 14.1 (1.6×) | 25.2 (2.8×) | 15.2 (1.7×) |
| mul_uint, constant time | BN254 scalar | 141 | 482 (3.4×) | 164 (1.2×) | 592 (4.2×) | 300 (2.1×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 318 (3.8×) | 70.7 (0.8×) | 380 (4.5×) | 117 (1.4×) |
| mul_uint, constant time | P-256 base | 134 | 409 (3.1×) | 126 (0.9×) | 488 (3.6×) | 225 (1.7×) |
| mul_uint, constant time | P-256 scalar | 146 | 474 (3.2×) | 173 (1.2×) | 541 (3.7×) | 318 (2.2×) |
| mul_uint, constant time | Tom-256 base | 135 | 461 (3.4×) | 163 (1.2×) | 555 (4.1×) | 349 (2.6×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,076 (4.0×) | 354 (1.3×) | 1,693 (6.4×) | 930 (3.5×) |
| mul_uint, constant time | P-521 base | 472 | 2,021 (4.3×) | 285 (0.6×) | 2,937 (6.2×) | 594 (1.3×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) | 4.9 (1.1×) |
| add_u64, constant time | BabyBear | 4.9 | 11.9 (2.4×) | 6.9 (1.4×) | 13.0 (2.7×) | 11.3 (2.3×) |
| add_u64, constant time | Goldilocks | 5.0 | 8.5 (1.7×) | 6.5 (1.3×) | 11.7 (2.3×) | 9.3 (1.9×) |
| add_u64, constant time | BN254 scalar | 21.9 | 93.0 (4.2×) | 40.8 (1.9×) | 108 (4.9×) | 66.3 (3.0×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.5 (3.4×) | 15.0 (2.0×) | 25.1 (3.4×) | 24.1 (3.3×) |
| add_u64, constant time | P-256 base | 17.1 | 66.5 (3.9×) | 35.5 (2.1×) | 80.1 (4.7×) | 58.0 (3.4×) |
| add_u64, constant time | P-256 scalar | 24.8 | 104 (4.2×) | 40.7 (1.6×) | 109 (4.4×) | 83.2 (3.4×) |
| add_u64, constant time | Tom-256 base | 22.3 | 97.6 (4.4×) | 40.2 (1.8×) | 110 (4.9×) | 70.3 (3.2×) |
| add_u64, constant time | BLS12-381 base | 35.8 | 136 (3.8×) | 63.8 (1.8×) | 177 (4.9×) | 102 (2.8×) |
| add_u64, constant time | P-521 base | 12.9 | 65.8 (5.1×) | 36.5 (2.8×) | 66.8 (5.2×) | 62.1 (4.8×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) |
| add_uint, constant time | BabyBear | 4.9 | 11.9 (2.4×) | 7.2 (1.5×) | 13.0 (2.7×) | 11.4 (2.3×) |
| add_uint, constant time | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.8 (2.5×) | 9.3 (2.0×) |
| add_uint, constant time | BN254 scalar | 74.8 | 238 (3.2×) | 96.6 (1.3×) | 327 (4.4×) | 188 (2.5×) |
| add_uint, constant time | secp256k1 base | 8.8 | 38.1 (4.3×) | 25.1 (2.9×) | 36.4 (4.1×) | 40.5 (4.6×) |
| add_uint, constant time | P-256 base | 57.8 | 179 (3.1×) | 74.4 (1.3×) | 268 (4.6×) | 142 (2.5×) |
| add_uint, constant time | P-256 scalar | 78.8 | 260 (3.3×) | 101 (1.3×) | 310 (3.9×) | 201 (2.6×) |
| add_uint, constant time | Tom-256 base | 75.1 | 252 (3.4×) | 95.3 (1.3×) | 328 (4.4×) | 179 (2.4×) |
| add_uint, constant time | BLS12-381 base | 154 | 756 (4.9×) | 191 (1.2×) | 941 (6.1×) | 408 (2.6×) |
| add_uint, constant time | P-521 base | 22.6 | 137 (6.1×) | 78.2 (3.5×) | 138 (6.1×) | 132 (5.8×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.7 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_u64, constant time | BabyBear | 4.8 | 11.9 (2.5×) | 7.1 (1.5×) | 12.9 (2.7×) | 10.6 (2.2×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.8 (2.1×) |
| sub_u64, constant time | BN254 scalar | 21.9 | 94.8 (4.3×) | 37.0 (1.7×) | 107 (4.9×) | 66.7 (3.0×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.1 (2.6×) | 12.0 (1.7×) | 17.0 (2.5×) | 15.1 (2.2×) |
| sub_u64, constant time | P-256 base | 16.1 | 61.4 (3.8×) | 31.6 (2.0×) | 74.6 (4.6×) | 53.3 (3.3×) |
| sub_u64, constant time | P-256 scalar | 22.7 | 96.3 (4.2×) | 37.4 (1.6×) | 107 (4.7×) | 69.8 (3.1×) |
| sub_u64, constant time | Tom-256 base | 23.9 | 92.0 (3.8×) | 36.6 (1.5×) | 105 (4.4×) | 74.3 (3.1×) |
| sub_u64, constant time | BLS12-381 base | 35.1 | 140 (4.0×) | 59.9 (1.7×) | 165 (4.7×) | 111 (3.2×) |
| sub_u64, constant time | P-521 base | 11.0 | 45.7 (4.2×) | 27.1 (2.5×) | 49.8 (4.5×) | 40.2 (3.7×) |
| sub_uint, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 5.0 (1.1×) |
| sub_uint, constant time | BabyBear | 4.8 | 12.1 (2.5×) | 7.1 (1.5×) | 13.0 (2.7×) | 10.6 (2.2×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.7 (2.1×) |
| sub_uint, constant time | BN254 scalar | 74.1 | 249 (3.4×) | 92.6 (1.2×) | 307 (4.1×) | 182 (2.5×) |
| sub_uint, constant time | secp256k1 base | 7.8 | 32.3 (4.1×) | 20.6 (2.6×) | 31.8 (4.1×) | 35.6 (4.6×) |
| sub_uint, constant time | P-256 base | 56.8 | 176 (3.1×) | 71.1 (1.3×) | 260 (4.6×) | 136 (2.4×) |
| sub_uint, constant time | P-256 scalar | 76.9 | 253 (3.3×) | 97.1 (1.3×) | 330 (4.3×) | 189 (2.5×) |
| sub_uint, constant time | Tom-256 base | 72.8 | 248 (3.4×) | 97.4 (1.3×) | 333 (4.6×) | 192 (2.6×) |
| sub_uint, constant time | BLS12-381 base | 156 | 760 (4.9×) | 198 (1.3×) | 932 (6.0×) | 415 (2.7×) |
| sub_uint, constant time | P-521 base | 19.5 | 125 (6.4×) | 67.2 (3.4×) | 122 (6.3×) | 108 (5.5×) |
| div_u64, constant time | BN254 scalar | 6,843 | 12,436 (1.8×) | 9,395 (1.4×) | 17,184 (2.5×) | 13,246 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,780 | 11,953 (1.8×) | 9,020 (1.3×) | 17,156 (2.5×) | 12,727 (1.9×) |
| div_u64, constant time | P-256 base | 6,654 | 11,868 (1.8×) | 9,116 (1.4×) | 16,968 (2.6×) | 12,488 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,895 | 12,309 (1.8×) | 9,353 (1.4×) | 17,558 (2.5×) | 13,125 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,274 | 12,189 (1.7×) | 9,276 (1.3×) | 16,895 (2.3×) | 13,006 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,367 | 24,805 (2.0×) | 16,887 (1.4×) | 34,839 (2.8×) | 24,262 (2.0×) |
| div_u64, constant time | P-521 base | 20,375 | 42,990 (2.1×) | 26,086 (1.3×) | 65,028 (3.2×) | 43,122 (2.1×) |
| div_uint, constant time | Mersenne-31 | 250 | 275 (1.1×) | 275 (1.1×) | 274 (1.1×) | 274 (1.1×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 435 (1.1×) | 470 (1.2×) | 434 (1.1×) |
| div_uint, constant time | Goldilocks | 665 | 1,138 (1.7×) | 860 (1.3×) | 1,434 (2.2×) | 950 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,894 | 12,367 (1.8×) | 9,359 (1.4×) | 17,713 (2.6×) | 13,362 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,779 | 11,962 (1.8×) | 9,056 (1.3×) | 16,891 (2.5×) | 12,821 (1.9×) |
| div_uint, constant time | P-256 base | 6,690 | 11,953 (1.8×) | 9,091 (1.4×) | 17,483 (2.6×) | 12,627 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,912 | 12,589 (1.8×) | 9,453 (1.4×) | 17,919 (2.6×) | 13,356 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,335 | 12,408 (1.7×) | 9,389 (1.3×) | 17,014 (2.3×) | 13,142 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,516 | 25,405 (2.0×) | 17,020 (1.4×) | 35,338 (2.8×) | 24,760 (2.0×) |
| div_uint, constant time | P-521 base | 20,375 | 43,026 (2.1×) | 26,311 (1.3×) | 64,427 (3.2×) | 43,137 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
