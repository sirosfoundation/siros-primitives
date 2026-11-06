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

`prime-field` at commit `93e2036` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-06.

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
| Cost of a count, in WebAssembly, Chrome | 4,306 to 14,177 cycles and 3,081 instructions; its noise in a figure, 92.0 to 1,774 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,061 to 8,539 cycles and 3,081 instructions; its noise in a figure, 47.0 to 1,122 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 3,948 to 13,193 cycles and 1,885 instructions; its noise in a figure, 198 to 3,780 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,242 to 9,076 cycles and 1,885 instructions; its noise in a figure, 385 to 1,560 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 12.94 before, 15.79 after; its SMT sibling, CPU 28, 6% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 17 before, 8.82 after; its SMT sibling, CPU 28, 11% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 8.82 before, 10.13 after; its SMT sibling, CPU 28, 8% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 10.13 before, 13.03 after; its SMT sibling, CPU 28, 12% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add | BabyBear | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) |
| add | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) |
| add | BN254 scalar | 13.9 vs ark-ff 13.8 | +0.1 (1.01×) | 12.2 vs ark-ff 13.1 | −0.9 (0.93×) | 14.5 vs ark-ff 14.0 | +0.5 (1.04×) | 18.5 vs ark-ff 18.0 | +0.5 (1.03×) |
| add | secp256k1 base | 14.0 vs ark-ff 15.2 | −1.2 (0.92×) | 11.8 vs ark-ff 12.8 | −1.0 (0.92×) | 14.8 vs ark-ff 16.8 | −2.0 (0.88×) | 19.8 vs ark-ff 20.0 | −0.2 (0.99×) |
| add | P-256 base | 14.4 vs ark-ff 15.7 | −1.3 (0.92×) | 12.3 vs ark-ff 12.3 | +0.0 (1.00×) | 14.7 vs ark-ff 17.2 | −2.5 (0.85×) | 20.1 vs ark-ff 20.0 | +0.1 (1.01×) |
| add | P-256 scalar | 14.5 vs ark-ff 15.4 | −0.9 (0.94×) | 11.8 vs ark-ff 13.3 | −1.5 (0.89×) | 15.4 vs ark-ff 17.1 | −1.7 (0.90×) | 20.5 vs ark-ff 20.6 | −0.1 (1.00×) |
| add | Tom-256 base | 13.6 vs halo2curves 35.9 | −22.3 (0.38×) | 11.8 vs halo2curves 20.2 | −8.4 (0.58×) | 14.6 vs halo2curves 38.6 | −24.0 (0.38×) | 20.4 vs halo2curves 34.0 | −13.6 (0.60×) |
| add | BLS12-381 base | 22.0 vs ark-ff 35.4 | −13.4 (0.62×) | 17.8 vs ark-ff 30.0 | −12.2 (0.59×) | 22.4 vs ark-ff 39.4 | −17.0 (0.57×) | 29.4 vs ark-ff 42.7 | −13.3 (0.69×) |
| add | P-521 base | 42.5 vs crypto-bigint 145 | −103 (0.29×) | 25.8 vs crypto-bigint 135 | −109 (0.19×) | 43.3 vs crypto-bigint 180 | −137 (0.24×) | 44.7 vs crypto-bigint 185 | −140 (0.24×) |
| sub | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub | BabyBear | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) |
| sub | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 15.0 vs ark-ff 18.5 | −3.5 (0.81×) | 11.9 vs halo2curves 15.0 | −3.1 (0.79×) | 14.9 vs ark-ff 19.0 | −4.1 (0.78×) | 19.5 vs ark-ff 22.9 | −3.4 (0.85×) |
| sub | secp256k1 base | 16.6 vs ark-ff 18.4 | −1.8 (0.90×) | 11.8 vs halo2curves 14.1 | −2.3 (0.84×) | 15.4 vs ark-ff 20.0 | −4.6 (0.77×) | 19.9 vs halo2curves 22.0 | −2.1 (0.90×) |
| sub | P-256 base | 14.3 vs ark-ff 17.0 | −2.7 (0.84×) | 11.8 vs halo2curves 12.3 | −0.5 (0.96×) | 13.9 vs ark-ff 18.5 | −4.6 (0.75×) | 18.9 vs halo2curves 21.5 | −2.6 (0.88×) |
| sub | P-256 scalar | 16.1 vs ark-ff 18.0 | −1.9 (0.89×) | 11.8 vs halo2curves 14.4 | −2.6 (0.82×) | 15.0 vs ark-ff 18.7 | −3.7 (0.80×) | 19.5 vs ark-ff 23.1 | −3.6 (0.84×) |
| sub | Tom-256 base | 15.0 vs halo2curves 24.0 | −9.0 (0.62×) | 11.8 vs halo2curves 14.9 | −3.1 (0.79×) | 14.4 vs halo2curves 23.4 | −9.0 (0.62×) | 19.5 vs halo2curves 24.0 | −4.5 (0.81×) |
| sub | BLS12-381 base | 22.9 vs ark-ff 33.6 | −10.7 (0.68×) | 18.0 vs halo2curves 23.5 | −5.5 (0.77×) | 21.6 vs ark-ff 30.4 | −8.8 (0.71×) | 29.3 vs halo2curves 37.1 | −7.8 (0.79×) |
| sub | P-521 base | 47.0 vs crypto-bigint 88.5 | −41.5 (0.53×) | 28.7 vs crypto-bigint 64.6 | −35.9 (0.44×) | 42.5 vs crypto-bigint 123 | −80.5 (0.35×) | 44.7 vs crypto-bigint 129 | −84.3 (0.35×) |
| mul | Mersenne-31 | 3.6 vs crypto-bigint 29.3 | −25.7 (0.12×) | 3.6 vs crypto-bigint 15.8 | −12.2 (0.23×) | 4.0 vs crypto-bigint 38.8 | −34.8 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul | BabyBear | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 6.5 | −1.3 (0.80×) | 5.0 vs plonky3 6.5 | −1.5 (0.77×) |
| mul | Goldilocks | 10.0 vs winter-math 12.3 | −2.3 (0.81×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 14.6 vs winter-math 17.0 | −2.4 (0.86×) | 7.3 vs winter-math 7.8 | −0.5 (0.94×) |
| mul | BN254 scalar | 234 vs ark-ff 321 | −87.0 (0.73×) | 80.7 vs halo2curves 91.4 | −10.7 (0.88×) | 387 vs ark-ff 395 | −8.0 (0.98×) | 158 vs halo2curves 181 | −23.0 (0.87×) |
| mul | secp256k1 base | 161 vs ark-ff 279 | −118 (0.58×) | 63.4 vs halo2curves 91.0 | −27.6 (0.70×) | 247 vs ark-ff 410 | −163 (0.60×) | 109 vs halo2curves 163 | −54.0 (0.67×) |
| mul | P-256 base | 172 vs k256/p256 242 | −70.0 (0.71×) | 71.0 vs k256/p256 85.0 | −14.0 (0.84×) | 280 vs ark-ff 447 | −167 (0.63×) | 120 vs halo2curves 135 | −15.0 (0.89×) |
| mul | P-256 scalar | 239 vs ark-ff 318 | −79.0 (0.75×) | 85.3 vs halo2curves 93.8 | −8.5 (0.91×) | 312 vs ark-ff 413 | −101 (0.76×) | 171 vs halo2curves 181 | −10.0 (0.94×) |
| mul | Tom-256 base | 234 vs halo2curves 343 | −109 (0.68×) | 80.4 vs halo2curves 94.1 | −13.7 (0.85×) | 313 vs halo2curves 594 | −281 (0.53×) | 152 vs halo2curves 176 | −24.0 (0.86×) |
| mul | BLS12-381 base | 727 vs ark-ff 807 | −80.0 (0.90×) | 182 vs halo2curves 195 | −13.0 (0.93×) | 923 vs ark-ff 1,064 | −141 (0.87×) | 355 vs halo2curves 382 | −27.0 (0.93×) |
| mul | P-521 base | 667 vs crypto-bigint 2,116 | −1,449 (0.32×) | 253 vs crypto-bigint 600 | −347 (0.42×) | 1,280 vs crypto-bigint 2,738 | −1,458 (0.47×) | 443 vs crypto-bigint 1,064 | −621 (0.42×) |
| square | Mersenne-31 | 3.5 vs crypto-bigint 28.6 | −25.1 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 37.9 | −34.3 (0.09×) | 3.6 vs crypto-bigint 18.0 | −14.4 (0.20×) |
| square | BabyBear | 4.7 vs plonky3 5.1 | −0.4 (0.92×) | 4.8 vs plonky3 5.3 | −0.5 (0.91×) | 5.7 vs plonky3 5.8 | −0.1 (0.98×) | 5.4 vs plonky3 5.7 | −0.3 (0.95×) |
| square | Goldilocks | 8.7 vs winter-math 12.4 | −3.7 (0.70×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.0 vs winter-math 7.0 | +1.0 (1.14×) |
| square | BN254 scalar | 214 vs ark-ff 271 | −57.0 (0.79×) | 72.6 vs halo2curves 84.5 | −11.9 (0.86×) | 348 vs ark-ff 364 | −16.0 (0.96×) | 135 vs halo2curves 142 | −7.0 (0.95×) |
| square | secp256k1 base | 141 vs halo2curves 216 | −75.0 (0.65×) | 56.6 vs halo2curves 77.7 | −21.1 (0.73×) | 195 vs ark-ff 305 | −110 (0.64×) | 93.1 vs halo2curves 126 | −32.9 (0.74×) |
| square | P-256 base | 147 vs halo2curves 172 | −25.0 (0.85×) | 76.3 vs k256/p256 73.0 | +3.3 (1.05×) | 255 vs halo2curves 332 | −77.0 (0.77×) | 123 vs halo2curves 110 | +13.0 (1.12×) |
| square | P-256 scalar | 214 vs ark-ff 265 | −51.0 (0.81×) | 78.7 vs halo2curves 83.0 | −4.3 (0.95×) | 334 vs ark-ff 353 | −19.0 (0.95×) | 158 vs halo2curves 145 | +13.0 (1.09×) |
| square | Tom-256 base | 209 vs halo2curves 312 | −103 (0.67×) | 75.4 vs halo2curves 78.9 | −3.5 (0.96×) | 275 vs halo2curves 478 | −203 (0.58×) | 137 vs halo2curves 132 | +5.0 (1.04×) |
| square | BLS12-381 base | 547 vs ark-ff 634 | −87.0 (0.86×) | 176 vs halo2curves 163 | +13.0 (1.08×) | 822 vs ark-ff 796 | +26.0 (1.03×) | 328 vs halo2curves 287 | +41.0 (1.14×) |
| square | P-521 base | 558 vs crypto-bigint 2,155 | −1,597 (0.26×) | 246 vs crypto-bigint 615 | −369 (0.40×) | 912 vs crypto-bigint 2,837 | −1,925 (0.32×) | 394 vs crypto-bigint 1,107 | −713 (0.36×) |
| invert | Mersenne-31 | 266 vs plonky3 668 | −402 (0.40×) | 267 vs plonky3 678 | −411 (0.39×) | 268 vs plonky3 729 | −461 (0.37×) | 270 vs plonky3 724 | −454 (0.37×) |
| invert | BabyBear | 269 vs plonky3 704 | −435 (0.38×) | 269 vs plonky3 697 | −428 (0.39×) | 275 vs plonky3 696 | −421 (0.40×) | 271 vs plonky3 695 | −424 (0.39×) |
| invert | Goldilocks | 539 vs winter-math 1,762 | −1,223 (0.31×) | 539 vs winter-math 912 | −373 (0.59×) | 558 vs winter-math 1,831 | −1,273 (0.30×) | 559 vs winter-math 1,005 | −446 (0.56×) |
| invert | BN254 scalar | 8,467 vs halo2curves 14,235 | −5,768 (0.59×) | 6,530 vs halo2curves 8,004 | −1,474 (0.82×) | 11,735 vs halo2curves 21,464 | −9,729 (0.55×) | 7,878 vs halo2curves 13,983 | −6,105 (0.56×) |
| invert | secp256k1 base | 8,473 vs halo2curves 14,253 | −5,780 (0.59×) | 6,071 vs halo2curves 8,032 | −1,961 (0.76×) | 11,773 vs halo2curves 21,530 | −9,757 (0.55×) | 7,973 vs halo2curves 14,088 | −6,115 (0.57×) |
| invert | P-256 base | 8,280 vs halo2curves 14,244 | −5,964 (0.58×) | 6,088 vs halo2curves 8,045 | −1,957 (0.76×) | 11,332 vs halo2curves 21,610 | −10,278 (0.52×) | 7,709 vs halo2curves 14,081 | −6,372 (0.55×) |
| invert | P-256 scalar | 8,497 vs halo2curves 14,326 | −5,829 (0.59×) | 6,401 vs halo2curves 8,032 | −1,631 (0.80×) | 11,679 vs halo2curves 21,532 | −9,853 (0.54×) | 8,201 vs halo2curves 14,093 | −5,892 (0.58×) |
| invert | Tom-256 base | 8,375 vs halo2curves 14,335 | −5,960 (0.58×) | 6,430 vs halo2curves 8,054 | −1,624 (0.80×) | 11,384 vs halo2curves 21,637 | −10,253 (0.53×) | 7,961 vs halo2curves 14,098 | −6,137 (0.56×) |
| invert | BLS12-381 base | 19,326 vs halo2curves 25,439 | −6,113 (0.76×) | 11,416 vs halo2curves 13,083 | −1,667 (0.87×) | 24,344 vs halo2curves 37,999 | −13,655 (0.64×) | 15,495 vs halo2curves 23,697 | −8,202 (0.65×) |
| invert | P-521 base | 33,222 vs crypto-bigint 65,124 | −31,902 (0.51×) | 18,188 vs crypto-bigint 50,093 | −31,905 (0.36×) | 48,743 vs crypto-bigint 80,807 | −32,064 (0.60×) | 29,440 vs crypto-bigint 62,077 | −32,637 (0.47×) |
| sqrt | Mersenne-31 | 159 vs plonky3 324 | −165 (0.49×) | 159 vs plonky3 324 | −165 (0.49×) | 160 vs plonky3 329 | −169 (0.49×) | 160 vs plonky3 329 | −169 (0.49×) |
| sqrt | BabyBear | 2,115 vs plonky3 3,083 | −968 (0.69×) | 2,027 vs plonky3 3,090 | −1,063 (0.66×) | 2,149 vs plonky3 2,960 | −811 (0.73×) | 2,060 vs plonky3 2,993 | −933 (0.69×) |
| sqrt | Goldilocks | 4,825 vs crypto-bigint 48,599 | −43,774 (0.10×) | 3,489 vs crypto-bigint 28,837 | −25,348 (0.12×) | 6,105 vs crypto-bigint 54,489 | −48,384 (0.11×) | 5,101 vs crypto-bigint 32,850 | −27,749 (0.16×) |
| sqrt | BN254 scalar | 110,434 vs ark-ff 149,018 | −38,584 (0.74×) | 55,793 vs halo2curves 102,154 | −46,361 (0.55×) | 180,083 vs ark-ff 204,243 | −24,160 (0.88×) | 93,453 vs ark-ff 147,815 | −54,362 (0.63×) |
| sqrt | secp256k1 base | 40,902 vs ark-ff 118,053 | −77,151 (0.35×) | 18,739 vs halo2curves 43,366 | −24,627 (0.43×) | 56,499 vs ark-ff 173,947 | −117,448 (0.32×) | 28,354 vs crypto-bigint 79,559 | −51,205 (0.36×) |
| sqrt | P-256 base | 43,201 vs ark-ff 59,750 | −16,549 (0.72×) | 20,261 vs k256/p256 21,681 | −1,420 (0.93×) | 78,043 vs ark-ff 99,001 | −20,958 (0.79×) | 25,769 vs k256/p256 33,574 | −7,805 (0.77×) |
| sqrt | P-256 scalar | 74,311 vs ark-ff 122,952 | −48,641 (0.60×) | 35,225 vs halo2curves 49,711 | −14,486 (0.71×) | 112,404 vs ark-ff 158,099 | −45,695 (0.71×) | 64,824 vs halo2curves 72,503 | −7,679 (0.89×) |
| sqrt | Tom-256 base | 65,989 vs crypto-bigint 148,928 | −82,939 (0.44×) | 22,503 vs halo2curves 45,495 | −22,992 (0.49×) | 99,201 vs crypto-bigint 231,054 | −131,853 (0.43×) | 54,764 vs crypto-bigint 74,387 | −19,623 (0.74×) |
| sqrt | BLS12-381 base | 296,265 vs ark-ff 420,588 | −124,323 (0.70×) | 97,532 vs crypto-bigint 144,949 | −47,417 (0.67×) | 393,110 vs ark-ff 577,252 | −184,142 (0.68×) | 222,608 vs crypto-bigint 273,194 | −50,586 (0.81×) |
| sqrt | P-521 base | 319,464 vs crypto-bigint 1,531,110 | −1,211,646 (0.21×) | 127,191 vs crypto-bigint 296,892 | −169,701 (0.43×) | 472,407 vs crypto-bigint 1,563,440 | −1,091,033 (0.30×) | 259,168 vs crypto-bigint 597,804 | −338,636 (0.43×) |
| mul_u64 | Mersenne-31 | 6.7 vs crypto-bigint 61.2 | −54.5 (0.11×) | 6.7 vs crypto-bigint 30.4 | −23.7 (0.22×) | 7.6 vs crypto-bigint 72.6 | −65.0 (0.10×) | 7.7 vs crypto-bigint 39.0 | −31.3 (0.20×) |
| mul_u64 | BabyBear | 13.0 vs plonky3 35.1 | −22.1 (0.37×) | 11.2 vs plonky3 11.0 | +0.2 (1.02×) | 18.0 vs plonky3 46.0 | −28.0 (0.39×) | 14.7 vs plonky3 18.0 | −3.3 (0.82×) |
| mul_u64 | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.4 | −6.6 (0.79×) | 9.0 vs winter-math 13.5 | −4.5 (0.67×) |
| mul_u64 | BN254 scalar | 99.5 vs ark-ff 695 | −596 (0.14×) | 28.8 vs crypto-bigint 263 | −234 (0.11×) | 124 vs ark-ff 832 | −708 (0.15×) | 44.1 vs crypto-bigint 406 | −362 (0.11×) |
| mul_u64 | secp256k1 base | 42.2 vs ark-ff 449 | −407 (0.09×) | 12.6 vs crypto-bigint 263 | −250 (0.05×) | 65.2 vs ark-ff 612 | −547 (0.11×) | 24.0 vs halo2curves 383 | −359 (0.06×) |
| mul_u64 | P-256 base | 72.3 vs k256/p256 485 | −413 (0.15×) | 27.3 vs k256/p256 150 | −123 (0.18×) | 94.1 vs ark-ff 828 | −734 (0.11×) | 41.1 vs k256/p256 258 | −217 (0.16×) |
| mul_u64 | P-256 scalar | 84.1 vs k256/p256 519 | −435 (0.16×) | 30.3 vs k256/p256 253 | −223 (0.12×) | 112 vs k256/p256 826 | −714 (0.14×) | 48.7 vs k256/p256 338 | −289 (0.14×) |
| mul_u64 | Tom-256 base | 82.1 vs crypto-bigint 876 | −794 (0.09×) | 30.0 vs crypto-bigint 263 | −233 (0.11×) | 115 vs halo2curves 1,259 | −1,144 (0.09×) | 44.5 vs crypto-bigint 408 | −364 (0.11×) |
| mul_u64 | BLS12-381 base | 131 vs ark-ff 1,660 | −1,529 (0.08×) | 44.2 vs halo2curves 487 | −443 (0.09×) | 167 vs ark-ff 2,110 | −1,943 (0.08×) | 61.5 vs crypto-bigint 1,026 | −965 (0.06×) |
| mul_u64 | P-521 base | 92.3 vs crypto-bigint 3,011 | −2,919 (0.03×) | 31.8 vs crypto-bigint 991 | −959 (0.03×) | 249 vs crypto-bigint 3,999 | −3,750 (0.06×) | 50.1 vs crypto-bigint 1,831 | −1,781 (0.03×) |
| mul_uint | Mersenne-31 | 6.7 vs crypto-bigint 61.0 | −54.3 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 74.6 | −67.0 (0.10×) | 7.7 vs crypto-bigint 38.9 | −31.2 (0.20×) |
| mul_uint | BabyBear | 13.0 vs plonky3 23.1 | −10.1 (0.56×) | 11.3 vs plonky3 12.1 | −0.8 (0.93×) | 18.0 vs plonky3 34.5 | −16.5 (0.52×) | 14.7 vs plonky3 18.0 | −3.3 (0.82×) |
| mul_uint | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.5 | −6.7 (0.79×) | 9.0 vs winter-math 13.5 | −4.5 (0.67×) |
| mul_uint | BN254 scalar | 439 vs halo2curves 756 | −317 (0.58×) | 181 vs halo2curves 182 | −1.0 (0.99×) | 547 vs halo2curves 1,316 | −769 (0.42×) | 287 vs halo2curves 349 | −62.0 (0.82×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 519 | −253 (0.51×) | 62.5 vs halo2curves 153 | −90.5 (0.41×) | 319 vs halo2curves 969 | −650 (0.33×) | 110 vs halo2curves 265 | −155 (0.42×) |
| mul_uint | P-256 base | 353 vs halo2curves 456 | −103 (0.77×) | 125 vs halo2curves 143 | −18.0 (0.87×) | 435 vs halo2curves 947 | −512 (0.46×) | 233 vs halo2curves 297 | −64.0 (0.78×) |
| mul_uint | P-256 scalar | 426 vs k256/p256 557 | −131 (0.76×) | 180 vs halo2curves 183 | −3.0 (0.98×) | 478 vs k256/p256 887 | −409 (0.54×) | 343 vs halo2curves 346 | −3.0 (0.99×) |
| mul_uint | Tom-256 base | 412 vs halo2curves 696 | −284 (0.59×) | 162 vs halo2curves 182 | −20.0 (0.89×) | 486 vs halo2curves 1,191 | −705 (0.41×) | 323 vs halo2curves 335 | −12.0 (0.96×) |
| mul_uint | BLS12-381 base | 969 vs crypto-bigint 1,925 | −956 (0.50×) | 370 vs halo2curves 381 | −11.0 (0.97×) | 1,145 vs crypto-bigint 2,517 | −1,372 (0.45×) | 925 vs crypto-bigint 927 | −2.0 (1.00×) |
| mul_uint | P-521 base | 1,545 vs crypto-bigint 3,026 | −1,481 (0.51×) | 266 vs crypto-bigint 990 | −724 (0.27×) | 2,384 vs crypto-bigint 4,025 | −1,641 (0.59×) | 541 vs crypto-bigint 1,872 | −1,331 (0.29×) |
| add_u64 | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.3 | −25.1 (0.17×) | 5.7 vs crypto-bigint 41.4 | −35.7 (0.14×) | 5.4 vs crypto-bigint 30.5 | −25.1 (0.18×) |
| add_u64 | BabyBear | 10.9 vs plonky3 32.1 | −21.2 (0.34×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 44.2 | −31.2 (0.29×) | 12.6 vs plonky3 15.0 | −2.4 (0.84×) |
| add_u64 | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.0 vs winter-math 7.4 | −0.4 (0.95×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 10.0 vs winter-math 10.2 | −0.2 (0.98×) |
| add_u64 | BN254 scalar | 118 vs ark-ff 362 | −244 (0.33×) | 53.0 vs halo2curves 209 | −156 (0.25×) | 142 vs ark-ff 419 | −277 (0.34×) | 94.3 vs halo2curves 297 | −203 (0.32×) |
| add_u64 | secp256k1 base | 8.0 vs ark-ff 162 | −154 (0.05×) | 8.0 vs ark-ff 121 | −113 (0.07×) | 8.0 vs ark-ff 227 | −219 (0.04×) | 12.0 vs ark-ff 184 | −172 (0.07×) |
| add_u64 | P-256 base | 71.3 vs k256/p256 276 | −205 (0.26×) | 35.9 vs k256/p256 97.0 | −61.1 (0.37×) | 84.6 vs ark-ff 405 | −320 (0.21×) | 67.1 vs k256/p256 159 | −91.9 (0.42×) |
| add_u64 | P-256 scalar | 70.7 vs k256/p256 41.1 | +29.6 (1.72×) | 40.7 vs k256/p256 41.2 | −0.5 (0.99×) | 74.6 vs k256/p256 54.5 | +20.1 (1.37×) | 84.3 vs k256/p256 54.7 | +29.6 (1.54×) |
| add_u64 | Tom-256 base | 88.5 vs crypto-bigint 506 | −418 (0.17×) | 41.7 vs halo2curves 218 | −176 (0.19×) | 108 vs halo2curves 698 | −590 (0.15×) | 71.8 vs halo2curves 291 | −219 (0.25×) |
| add_u64 | BLS12-381 base | 169 vs ark-ff 835 | −666 (0.20×) | 65.6 vs halo2curves 322 | −256 (0.20×) | 254 vs ark-ff 1,080 | −826 (0.24×) | 95.4 vs crypto-bigint 649 | −554 (0.15×) |
| add_u64 | P-521 base | 20.9 vs crypto-bigint 1,672 | −1,651 (0.01×) | 15.4 vs crypto-bigint 647 | −632 (0.02×) | 22.1 vs crypto-bigint 2,228 | −2,206 (0.01×) | 23.4 vs crypto-bigint 1,113 | −1,090 (0.02×) |
| add_uint | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.7 vs crypto-bigint 41.9 | −36.2 (0.14×) | 5.4 vs crypto-bigint 30.8 | −25.4 (0.18×) |
| add_uint | BabyBear | 10.9 vs plonky3 22.0 | −11.1 (0.50×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 34.3 | −21.3 (0.38×) | 12.6 vs plonky3 15.0 | −2.4 (0.84×) |
| add_uint | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 10.0 vs winter-math 10.2 | −0.2 (0.98×) |
| add_uint | BN254 scalar | 246 vs halo2curves 412 | −166 (0.60×) | 99.5 vs halo2curves 112 | −12.5 (0.89×) | 356 vs halo2curves 685 | −329 (0.52×) | 178 vs halo2curves 205 | −27.0 (0.87×) |
| add_uint | secp256k1 base | 21.0 vs halo2curves 224 | −203 (0.09×) | 14.6 vs halo2curves 85.3 | −70.7 (0.17×) | 18.3 vs halo2curves 426 | −408 (0.04×) | 23.8 vs halo2curves 140 | −116 (0.17×) |
| add_uint | P-256 base | 188 vs halo2curves 246 | −58.0 (0.76×) | 79.4 vs halo2curves 91.5 | −12.1 (0.87×) | 325 vs halo2curves 511 | −186 (0.64×) | 147 vs halo2curves 154 | −7.0 (0.95×) |
| add_uint | P-256 scalar | 260 vs k256/p256 94.6 | +165 (2.75×) | 109 vs k256/p256 94.8 | +14.2 (1.15×) | 331 vs k256/p256 118 | +213 (2.81×) | 175 vs k256/p256 118 | +57.0 (1.48×) |
| add_uint | Tom-256 base | 251 vs halo2curves 380 | −129 (0.66×) | 102 vs halo2curves 110 | −8.0 (0.93×) | 330 vs halo2curves 636 | −306 (0.52×) | 182 vs halo2curves 196 | −14.0 (0.93×) |
| add_uint | BLS12-381 base | 769 vs halo2curves 981 | −212 (0.78×) | 214 vs halo2curves 225 | −11.0 (0.95×) | 930 vs crypto-bigint 1,392 | −462 (0.67×) | 385 vs halo2curves 566 | −181 (0.68×) |
| add_uint | P-521 base | 65.1 vs crypto-bigint 1,684 | −1,619 (0.04×) | 41.8 vs crypto-bigint 648 | −606 (0.06×) | 86.7 vs crypto-bigint 2,229 | −2,142 (0.04×) | 71.6 vs crypto-bigint 1,135 | −1,063 (0.06×) |
| sub_u64 | Mersenne-31 | 5.0 vs crypto-bigint 23.4 | −18.4 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_u64 | BabyBear | 11.1 vs crypto-bigint 27.9 | −16.8 (0.40×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs plonky3 43.6 | −30.6 (0.30×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| sub_u64 | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.6 vs winter-math 19.0 | −7.4 (0.61×) | 9.3 vs winter-math 9.5 | −0.2 (0.98×) |
| sub_u64 | BN254 scalar | 109 vs ark-ff 353 | −244 (0.31×) | 55.6 vs crypto-bigint 151 | −95.4 (0.37×) | 139 vs ark-ff 417 | −278 (0.33×) | 73.9 vs crypto-bigint 261 | −187 (0.28×) |
| sub_u64 | secp256k1 base | 8.7 vs ark-ff 161 | −152 (0.05×) | 8.0 vs ark-ff 124 | −116 (0.06×) | 8.0 vs ark-ff 231 | −223 (0.03×) | 11.0 vs ark-ff 193 | −182 (0.06×) |
| sub_u64 | P-256 base | 70.5 vs k256/p256 264 | −194 (0.27×) | 35.7 vs k256/p256 88.9 | −53.2 (0.40×) | 80.1 vs ark-ff 408 | −328 (0.20×) | 60.8 vs k256/p256 149 | −88.2 (0.41×) |
| sub_u64 | P-256 scalar | 67.9 vs k256/p256 21.2 | +46.7 (3.20×) | 40.0 vs k256/p256 21.2 | +18.8 (1.89×) | 75.9 vs k256/p256 24.4 | +51.5 (3.11×) | 84.1 vs k256/p256 24.3 | +59.8 (3.46×) |
| sub_u64 | Tom-256 base | 87.4 vs crypto-bigint 501 | −414 (0.17×) | 49.6 vs crypto-bigint 151 | −101 (0.33×) | 104 vs halo2curves 687 | −583 (0.15×) | 72.0 vs crypto-bigint 259 | −187 (0.28×) |
| sub_u64 | BLS12-381 base | 151 vs ark-ff 835 | −684 (0.18×) | 66.6 vs halo2curves 316 | −249 (0.21×) | 251 vs ark-ff 1,052 | −801 (0.24×) | 96.3 vs crypto-bigint 617 | −521 (0.16×) |
| sub_u64 | P-521 base | 22.2 vs crypto-bigint 1,659 | −1,637 (0.01×) | 18.7 vs crypto-bigint 622 | −603 (0.03×) | 20.2 vs crypto-bigint 2,211 | −2,191 (0.01×) | 21.1 vs crypto-bigint 1,098 | −1,077 (0.02×) |
| sub_uint | Mersenne-31 | 5.0 vs crypto-bigint 23.5 | −18.5 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_uint | BabyBear | 11.1 vs plonky3 23.0 | −11.9 (0.48×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs plonky3 34.3 | −21.3 (0.38×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| sub_uint | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.7 vs winter-math 19.0 | −7.3 (0.62×) | 9.2 vs winter-math 9.5 | −0.3 (0.97×) |
| sub_uint | BN254 scalar | 246 vs halo2curves 401 | −155 (0.61×) | 93.8 vs halo2curves 103 | −9.2 (0.91×) | 364 vs halo2curves 665 | −301 (0.55×) | 178 vs halo2curves 194 | −16.0 (0.92×) |
| sub_uint | secp256k1 base | 21.2 vs halo2curves 207 | −186 (0.10×) | 14.6 vs halo2curves 109 | −94.4 (0.13×) | 18.9 vs halo2curves 410 | −391 (0.05×) | 24.4 vs halo2curves 130 | −106 (0.19×) |
| sub_uint | P-256 base | 175 vs halo2curves 232 | −57.0 (0.75×) | 86.3 vs halo2curves 85.1 | +1.2 (1.01×) | 270 vs halo2curves 497 | −227 (0.54×) | 138 vs halo2curves 144 | −6.0 (0.96×) |
| sub_uint | P-256 scalar | 249 vs k256/p256 67.3 | +182 (3.70×) | 103 vs k256/p256 68.0 | +35.0 (1.51×) | 325 vs k256/p256 87.4 | +238 (3.72×) | 184 vs k256/p256 88.6 | +95.4 (2.08×) |
| sub_uint | Tom-256 base | 252 vs halo2curves 365 | −113 (0.69×) | 98.3 vs halo2curves 104 | −5.7 (0.95×) | 319 vs halo2curves 623 | −304 (0.51×) | 177 vs halo2curves 187 | −10.0 (0.95×) |
| sub_uint | BLS12-381 base | 765 vs halo2curves 952 | −187 (0.80×) | 206 vs halo2curves 216 | −10.0 (0.95×) | 929 vs crypto-bigint 1,368 | −439 (0.68×) | 383 vs halo2curves 544 | −161 (0.70×) |
| sub_uint | P-521 base | 75.5 vs crypto-bigint 1,673 | −1,598 (0.05×) | 41.9 vs crypto-bigint 620 | −578 (0.07×) | 80.3 vs crypto-bigint 2,211 | −2,131 (0.04×) | 68.7 vs crypto-bigint 1,105 | −1,036 (0.06×) |
| from_u64 | Mersenne-31 | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.0 vs plonky3 3.8 | +0.2 (1.05×) | 4.3 vs plonky3 3.9 | +0.4 (1.10×) |
| from_u64 | BabyBear | 11.1 vs crypto-bigint 21.5 | −10.4 (0.52×) | 7.3 vs plonky3 7.8 | −0.5 (0.94×) | 12.1 vs crypto-bigint 35.3 | −23.2 (0.34×) | 10.3 vs plonky3 11.8 | −1.5 (0.87×) |
| from_u64 | Goldilocks | 7.4 vs winter-math 10.6 | −3.2 (0.70×) | 5.9 vs winter-math 5.9 | +0.0 (1.00×) | 9.0 vs winter-math 17.1 | −8.1 (0.53×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) |
| from_u64 | BN254 scalar | 101 vs ark-ff 320 | −219 (0.32×) | 29.1 vs crypto-bigint 133 | −104 (0.22×) | 121 vs ark-ff 371 | −250 (0.33×) | 50.1 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64 | secp256k1 base | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) |
| from_u64 | P-256 base | 56.6 vs ark-ff 230 | −173 (0.25×) | 24.7 vs k256/p256 75.7 | −51.0 (0.33×) | 69.9 vs ark-ff 376 | −306 (0.19×) | 41.0 vs k256/p256 130 | −89.0 (0.32×) |
| from_u64 | P-256 scalar | 55.0 vs k256/p256 3.8 | +51.2 (14.47×) | 26.3 vs k256/p256 3.8 | +22.5 (6.92×) | 58.3 vs k256/p256 3.7 | +54.6 (15.76×) | 48.1 vs k256/p256 3.8 | +44.3 (12.66×) |
| from_u64 | Tom-256 base | 74.5 vs crypto-bigint 435 | −361 (0.17×) | 28.2 vs crypto-bigint 133 | −105 (0.21×) | 86.1 vs halo2curves 647 | −561 (0.13×) | 50.0 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64 | BLS12-381 base | 144 vs ark-ff 743 | −599 (0.19×) | 50.3 vs crypto-bigint 258 | −208 (0.19×) | 183 vs ark-ff 1,008 | −825 (0.18×) | 68.0 vs crypto-bigint 473 | −405 (0.14×) |
| from_u64 | P-521 base | 6.1 vs crypto-bigint 1,445 | −1,439 (0.00×) | 6.0 vs crypto-bigint 499 | −493 (0.01×) | 6.0 vs crypto-bigint 1,991 | −1,985 (0.00×) | 6.0 vs crypto-bigint 808 | −802 (0.01×) |
| div_u64 | BN254 scalar | 1,214 vs halo2curves 13,466 | −12,252 (0.09×) | 843 vs halo2curves 6,978 | −6,135 (0.12×) | 1,378 vs halo2curves 19,442 | −18,064 (0.07×) | 1,106 vs halo2curves 11,759 | −10,653 (0.09×) |
| div_u64 | secp256k1 base | 1,010 vs halo2curves 13,071 | −12,061 (0.08×) | 716 vs halo2curves 6,883 | −6,167 (0.10×) | 1,216 vs halo2curves 18,933 | −17,717 (0.06×) | 817 vs halo2curves 11,676 | −10,859 (0.07×) |
| div_u64 | P-256 base | 1,091 vs halo2curves 13,085 | −11,994 (0.08×) | 804 vs halo2curves 6,886 | −6,082 (0.12×) | 1,345 vs halo2curves 18,936 | −17,591 (0.07×) | 1,010 vs halo2curves 11,586 | −10,576 (0.09×) |
| div_u64 | P-256 scalar | 1,187 vs halo2curves 13,395 | −12,208 (0.09×) | 867 vs halo2curves 6,953 | −6,086 (0.12×) | 1,440 vs halo2curves 19,417 | −17,977 (0.07×) | 1,138 vs halo2curves 11,702 | −10,564 (0.10×) |
| div_u64 | Tom-256 base | 1,209 vs halo2curves 13,278 | −12,069 (0.09×) | 865 vs halo2curves 6,926 | −6,061 (0.12×) | 1,469 vs halo2curves 19,119 | −17,650 (0.08×) | 1,120 vs halo2curves 11,842 | −10,722 (0.09×) |
| div_u64 | BLS12-381 base | 1,813 vs halo2curves 24,039 | −22,226 (0.08×) | 1,082 vs halo2curves 11,025 | −9,943 (0.10×) | 2,463 vs halo2curves 32,975 | −30,512 (0.07×) | 1,776 vs halo2curves 18,596 | −16,820 (0.10×) |
| div_u64 | P-521 base | 2,457 vs crypto-bigint 62,297 | −59,840 (0.04×) | 1,010 vs crypto-bigint 45,175 | −44,165 (0.02×) | 3,589 vs crypto-bigint 77,911 | −74,322 (0.05×) | 1,513 vs crypto-bigint 58,035 | −56,522 (0.03×) |
| div_uint | Mersenne-31 | 273 vs crypto-bigint 1,813 | −1,540 (0.15×) | 274 vs crypto-bigint 1,611 | −1,337 (0.17×) | 279 vs runtime-algebra 2,097 | −1,818 (0.13×) | 279 vs crypto-bigint 1,972 | −1,693 (0.14×) |
| div_uint | BabyBear | 276 vs plonky3 676 | −400 (0.41×) | 275 vs plonky3 656 | −381 (0.42×) | 279 vs plonky3 740 | −461 (0.38×) | 281 vs plonky3 713 | −432 (0.39×) |
| div_uint | Goldilocks | 552 vs winter-math 1,818 | −1,266 (0.30×) | 540 vs winter-math 952 | −412 (0.57×) | 591 vs winter-math 1,882 | −1,291 (0.31×) | 577 vs winter-math 1,041 | −464 (0.55×) |
| div_uint | BN254 scalar | 9,242 vs halo2curves 15,058 | −5,816 (0.61×) | 6,697 vs halo2curves 8,192 | −1,495 (0.82×) | 12,726 vs halo2curves 22,886 | −10,160 (0.56×) | 8,387 vs halo2curves 14,506 | −6,119 (0.58×) |
| div_uint | secp256k1 base | 8,575 vs halo2curves 14,824 | −6,249 (0.58×) | 6,044 vs halo2curves 8,257 | −2,213 (0.73×) | 12,029 vs halo2curves 22,759 | −10,730 (0.53×) | 8,170 vs halo2curves 14,457 | −6,287 (0.57×) |
| div_uint | P-256 base | 8,789 vs halo2curves 14,878 | −6,089 (0.59×) | 6,162 vs halo2curves 8,197 | −2,035 (0.75×) | 12,120 vs halo2curves 22,659 | −10,539 (0.53×) | 7,960 vs halo2curves 14,426 | −6,466 (0.55×) |
| div_uint | P-256 scalar | 9,151 vs halo2curves 15,110 | −5,959 (0.61×) | 6,620 vs halo2curves 8,230 | −1,610 (0.80×) | 12,491 vs halo2curves 23,061 | −10,570 (0.54×) | 8,662 vs halo2curves 14,631 | −5,969 (0.59×) |
| div_uint | Tom-256 base | 9,086 vs halo2curves 15,040 | −5,954 (0.60×) | 6,640 vs halo2curves 8,258 | −1,618 (0.80×) | 12,437 vs halo2curves 22,973 | −10,536 (0.54×) | 8,376 vs halo2curves 14,552 | −6,176 (0.58×) |
| div_uint | BLS12-381 base | 21,059 vs halo2curves 27,618 | −6,559 (0.76×) | 12,225 vs halo2curves 13,664 | −1,439 (0.89×) | 26,412 vs halo2curves 40,364 | −13,952 (0.65×) | 17,165 vs halo2curves 24,411 | −7,246 (0.70×) |
| div_uint | P-521 base | 34,261 vs crypto-bigint 68,106 | −33,845 (0.50×) | 18,489 vs crypto-bigint 51,270 | −32,781 (0.36×) | 49,508 vs crypto-bigint 83,902 | −34,394 (0.59×) | 29,987 vs crypto-bigint 63,565 | −33,578 (0.47×) |
| add, constant time | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add, constant time | BabyBear | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 3.0 vs crypto-bigint 10.8 | −7.8 (0.28×) | 3.0 vs runtime-algebra 10.9 | −7.9 (0.28×) |
| add, constant time | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) | 3.1 vs winter-math 4.0 | −0.9 (0.78×) |
| add, constant time | BN254 scalar | 23.6 vs halo2curves 36.5 | −12.9 (0.65×) | 16.3 vs halo2curves 20.6 | −4.3 (0.79×) | 20.7 vs halo2curves 39.4 | −18.7 (0.53×) | 26.5 vs halo2curves 35.0 | −8.5 (0.76×) |
| add, constant time | secp256k1 base | 26.3 vs halo2curves 35.1 | −8.8 (0.75×) | 18.2 vs halo2curves 19.2 | −1.0 (0.95×) | 24.9 vs halo2curves 35.7 | −10.8 (0.70×) | 29.0 vs halo2curves 33.1 | −4.1 (0.88×) |
| add, constant time | P-256 base | 23.5 vs halo2curves 30.5 | −7.0 (0.77×) | 18.1 vs halo2curves 17.9 | +0.2 (1.01×) | 22.7 vs halo2curves 33.5 | −10.8 (0.68×) | 28.0 vs k256/p256 31.5 | −3.5 (0.89×) |
| add, constant time | P-256 scalar | 27.1 vs halo2curves 35.5 | −8.4 (0.76×) | 17.9 vs halo2curves 20.5 | −2.6 (0.87×) | 24.0 vs halo2curves 38.1 | −14.1 (0.63×) | 29.5 vs halo2curves 35.0 | −5.5 (0.84×) |
| add, constant time | Tom-256 base | 27.8 vs halo2curves 35.9 | −8.1 (0.77×) | 18.0 vs halo2curves 20.2 | −2.2 (0.89×) | 24.0 vs halo2curves 38.6 | −14.6 (0.62×) | 28.5 vs halo2curves 34.0 | −5.5 (0.84×) |
| add, constant time | BLS12-381 base | 42.7 vs halo2curves 63.6 | −20.9 (0.67×) | 25.5 vs halo2curves 32.5 | −7.0 (0.78×) | 35.2 vs halo2curves 69.9 | −34.7 (0.50×) | 41.2 vs halo2curves 54.1 | −12.9 (0.76×) |
| add, constant time | P-521 base | 74.6 vs crypto-bigint 145 | −70.4 (0.51×) | 46.6 vs crypto-bigint 135 | −88.4 (0.35×) | 79.0 vs crypto-bigint 180 | −101 (0.44×) | 74.1 vs crypto-bigint 185 | −111 (0.40×) |
| sub, constant time | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | BabyBear | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.1 vs halo2curves 24.0 | −2.9 (0.88×) | 15.4 vs halo2curves 15.0 | +0.4 (1.03×) | 20.8 vs halo2curves 23.2 | −2.4 (0.90×) | 24.0 vs halo2curves 24.0 | +0.0 (1.00×) |
| sub, constant time | secp256k1 base | 19.1 vs halo2curves 22.4 | −3.3 (0.85×) | 14.4 vs halo2curves 14.1 | +0.3 (1.02×) | 18.6 vs halo2curves 20.3 | −1.7 (0.92×) | 22.5 vs halo2curves 22.0 | +0.5 (1.02×) |
| sub, constant time | P-256 base | 17.3 vs halo2curves 20.3 | −3.0 (0.85×) | 13.4 vs halo2curves 12.3 | +1.1 (1.09×) | 18.4 vs halo2curves 21.1 | −2.7 (0.87×) | 22.0 vs halo2curves 21.5 | +0.5 (1.02×) |
| sub, constant time | P-256 scalar | 21.0 vs k256/p256 23.2 | −2.2 (0.91×) | 14.7 vs halo2curves 14.4 | +0.3 (1.02×) | 22.0 vs halo2curves 23.1 | −1.1 (0.95×) | 24.0 vs halo2curves 23.1 | +0.9 (1.04×) |
| sub, constant time | Tom-256 base | 21.1 vs halo2curves 24.0 | −2.9 (0.88×) | 15.3 vs halo2curves 14.9 | +0.4 (1.03×) | 21.0 vs halo2curves 23.4 | −2.4 (0.90×) | 24.0 vs halo2curves 24.0 | +0.0 (1.00×) |
| sub, constant time | BLS12-381 base | 37.8 vs halo2curves 43.4 | −5.6 (0.87×) | 24.5 vs halo2curves 23.5 | +1.0 (1.04×) | 34.7 vs halo2curves 44.1 | −9.4 (0.79×) | 37.0 vs halo2curves 37.1 | −0.1 (1.00×) |
| sub, constant time | P-521 base | 56.2 vs crypto-bigint 88.5 | −32.3 (0.64×) | 37.1 vs crypto-bigint 64.6 | −27.5 (0.57×) | 58.0 vs crypto-bigint 123 | −65.0 (0.47×) | 53.1 vs crypto-bigint 129 | −75.9 (0.41×) |
| mul, constant time | Mersenne-31 | 3.6 vs crypto-bigint 29.3 | −25.7 (0.12×) | 3.6 vs crypto-bigint 15.8 | −12.2 (0.23×) | 4.0 vs crypto-bigint 38.8 | −34.8 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul, constant time | BabyBear | 5.2 vs crypto-bigint 29.3 | −24.1 (0.18×) | 5.2 vs crypto-bigint 15.8 | −10.6 (0.33×) | 5.2 vs crypto-bigint 38.8 | −33.6 (0.13×) | 5.0 vs crypto-bigint 19.2 | −14.2 (0.26×) |
| mul, constant time | Goldilocks | 10.0 vs winter-math 12.3 | −2.3 (0.81×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 14.6 vs winter-math 17.0 | −2.4 (0.86×) | 7.3 vs winter-math 7.8 | −0.5 (0.94×) |
| mul, constant time | BN254 scalar | 235 vs halo2curves 378 | −143 (0.62×) | 82.1 vs halo2curves 91.4 | −9.3 (0.90×) | 414 vs halo2curves 698 | −284 (0.59×) | 156 vs halo2curves 181 | −25.0 (0.86×) |
| mul, constant time | secp256k1 base | 160 vs halo2curves 303 | −143 (0.53×) | 63.1 vs halo2curves 91.0 | −27.9 (0.69×) | 246 vs halo2curves 585 | −339 (0.42×) | 110 vs halo2curves 163 | −53.0 (0.67×) |
| mul, constant time | P-256 base | 173 vs k256/p256 242 | −69.0 (0.71×) | 71.3 vs k256/p256 85.0 | −13.7 (0.84×) | 279 vs k256/p256 487 | −208 (0.57×) | 120 vs halo2curves 135 | −15.0 (0.89×) |
| mul, constant time | P-256 scalar | 241 vs halo2curves 343 | −102 (0.70×) | 85.4 vs halo2curves 93.8 | −8.4 (0.91×) | 311 vs halo2curves 705 | −394 (0.44×) | 164 vs halo2curves 181 | −17.0 (0.91×) |
| mul, constant time | Tom-256 base | 233 vs halo2curves 343 | −110 (0.68×) | 80.1 vs halo2curves 94.1 | −14.0 (0.85×) | 305 vs halo2curves 594 | −289 (0.51×) | 154 vs halo2curves 176 | −22.0 (0.88×) |
| mul, constant time | BLS12-381 base | 725 vs crypto-bigint 903 | −178 (0.80×) | 185 vs halo2curves 195 | −10.0 (0.95×) | 919 vs crypto-bigint 1,305 | −386 (0.70×) | 341 vs halo2curves 382 | −41.0 (0.89×) |
| mul, constant time | P-521 base | 666 vs crypto-bigint 2,116 | −1,450 (0.31×) | 253 vs crypto-bigint 600 | −347 (0.42×) | 1,281 vs crypto-bigint 2,738 | −1,457 (0.47×) | 443 vs crypto-bigint 1,064 | −621 (0.42×) |
| square, constant time | Mersenne-31 | 3.5 vs crypto-bigint 28.6 | −25.1 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 37.9 | −34.3 (0.09×) | 3.6 vs crypto-bigint 18.0 | −14.4 (0.20×) |
| square, constant time | BabyBear | 4.7 vs crypto-bigint 28.5 | −23.8 (0.16×) | 4.8 vs crypto-bigint 15.6 | −10.8 (0.31×) | 5.7 vs crypto-bigint 38.3 | −32.6 (0.15×) | 5.4 vs crypto-bigint 18.0 | −12.6 (0.30×) |
| square, constant time | Goldilocks | 8.7 vs winter-math 12.4 | −3.7 (0.70×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.0 vs winter-math 7.0 | +1.0 (1.14×) |
| square, constant time | BN254 scalar | 214 vs halo2curves 309 | −95.0 (0.69×) | 72.8 vs halo2curves 84.5 | −11.7 (0.86×) | 303 vs halo2curves 556 | −253 (0.54×) | 159 vs halo2curves 142 | +17.0 (1.12×) |
| square, constant time | secp256k1 base | 141 vs halo2curves 216 | −75.0 (0.65×) | 56.6 vs halo2curves 77.7 | −21.1 (0.73×) | 173 vs halo2curves 459 | −286 (0.38×) | 89.4 vs halo2curves 126 | −36.6 (0.71×) |
| square, constant time | P-256 base | 147 vs halo2curves 172 | −25.0 (0.85×) | 76.3 vs k256/p256 73.0 | +3.3 (1.05×) | 237 vs halo2curves 332 | −95.0 (0.71×) | 116 vs halo2curves 110 | +6.0 (1.05×) |
| square, constant time | P-256 scalar | 214 vs halo2curves 306 | −92.0 (0.70×) | 79.2 vs halo2curves 83.0 | −3.8 (0.95×) | 334 vs halo2curves 546 | −212 (0.61×) | 157 vs halo2curves 145 | +12.0 (1.08×) |
| square, constant time | Tom-256 base | 209 vs halo2curves 312 | −103 (0.67×) | 75.4 vs halo2curves 78.9 | −3.5 (0.96×) | 275 vs halo2curves 478 | −203 (0.58×) | 136 vs halo2curves 132 | +4.0 (1.03×) |
| square, constant time | BLS12-381 base | 567 vs halo2curves 659 | −92.0 (0.86×) | 176 vs halo2curves 163 | +13.0 (1.08×) | 822 vs halo2curves 1,285 | −463 (0.64×) | 321 vs halo2curves 287 | +34.0 (1.12×) |
| square, constant time | P-521 base | 563 vs crypto-bigint 2,155 | −1,592 (0.26×) | 246 vs crypto-bigint 615 | −369 (0.40×) | 909 vs crypto-bigint 2,837 | −1,928 (0.32×) | 394 vs crypto-bigint 1,107 | −713 (0.36×) |
| invert, constant time | Mersenne-31 | 263 vs runtime-algebra 3,341 | −3,078 (0.08×) | 264 vs runtime-algebra 3,342 | −3,078 (0.08×) | 268 vs runtime-algebra 3,329 | −3,061 (0.08×) | 266 vs runtime-algebra 3,335 | −3,069 (0.08×) |
| invert, constant time | BabyBear | 451 vs runtime-algebra 3,305 | −2,854 (0.14×) | 458 vs runtime-algebra 3,344 | −2,886 (0.14×) | 438 vs runtime-algebra 3,334 | −2,896 (0.13×) | 417 vs runtime-algebra 3,330 | −2,913 (0.13×) |
| invert, constant time | Goldilocks | 1,154 vs winter-math 1,762 | −608 (0.65×) | 841 vs winter-math 912 | −71.0 (0.92×) | 1,416 vs winter-math 1,831 | −415 (0.77×) | 932 vs winter-math 1,005 | −73.0 (0.93×) |
| invert, constant time | BN254 scalar | 12,000 vs crypto-bigint 29,617 | −17,617 (0.41×) | 9,454 vs crypto-bigint 25,166 | −15,712 (0.38×) | 16,754 vs crypto-bigint 35,999 | −19,245 (0.47×) | 12,515 vs crypto-bigint 30,680 | −18,165 (0.41×) |
| invert, constant time | secp256k1 base | 11,814 vs crypto-bigint 28,975 | −17,161 (0.41×) | 9,110 vs crypto-bigint 25,131 | −16,021 (0.36×) | 16,689 vs crypto-bigint 36,202 | −19,513 (0.46×) | 12,599 vs crypto-bigint 30,677 | −18,078 (0.41×) |
| invert, constant time | P-256 base | 11,522 vs crypto-bigint 28,839 | −17,317 (0.40×) | 8,957 vs k256/p256 22,171 | −13,214 (0.40×) | 16,344 vs crypto-bigint 36,152 | −19,808 (0.45×) | 12,302 vs crypto-bigint 30,693 | −18,391 (0.40×) |
| invert, constant time | P-256 scalar | 11,908 vs crypto-bigint 29,157 | −17,249 (0.41×) | 9,363 vs crypto-bigint 25,218 | −15,855 (0.37×) | 16,953 vs crypto-bigint 36,292 | −19,339 (0.47×) | 12,804 vs crypto-bigint 30,707 | −17,903 (0.42×) |
| invert, constant time | Tom-256 base | 11,809 vs crypto-bigint 29,390 | −17,581 (0.40×) | 9,247 vs crypto-bigint 25,258 | −16,011 (0.37×) | 17,029 vs crypto-bigint 36,405 | −19,376 (0.47×) | 12,699 vs crypto-bigint 30,723 | −18,024 (0.41×) |
| invert, constant time | BLS12-381 base | 23,998 vs crypto-bigint 49,213 | −25,215 (0.49×) | 16,671 vs crypto-bigint 42,608 | −25,937 (0.39×) | 32,888 vs crypto-bigint 66,251 | −33,363 (0.50×) | 24,146 vs crypto-bigint 53,757 | −29,611 (0.45×) |
| invert, constant time | P-521 base | 41,737 vs crypto-bigint 101,838 | −60,101 (0.41×) | 25,894 vs crypto-bigint 81,838 | −55,944 (0.32×) | 63,372 vs crypto-bigint 123,433 | −60,061 (0.51×) | 41,461 vs crypto-bigint 99,992 | −58,531 (0.41×) |
| sqrt, constant time | Mersenne-31 | 159 vs runtime-algebra 1,740 | −1,581 (0.09×) | 159 vs crypto-bigint 1,641 | −1,482 (0.10×) | 172 vs runtime-algebra 1,748 | −1,576 (0.10×) | 172 vs runtime-algebra 1,730 | −1,558 (0.10×) |
| sqrt, constant time | BabyBear | 7,133 vs crypto-bigint 33,829 | −26,696 (0.21×) | 6,475 vs crypto-bigint 20,220 | −13,745 (0.32×) | 6,802 vs crypto-bigint 38,402 | −31,600 (0.18×) | 6,524 vs crypto-bigint 22,997 | −16,473 (0.28×) |
| sqrt, constant time | Goldilocks | 10,859 vs crypto-bigint 48,599 | −37,740 (0.22×) | 8,338 vs crypto-bigint 28,837 | −20,499 (0.29×) | 14,673 vs crypto-bigint 54,489 | −39,816 (0.27×) | 10,417 vs crypto-bigint 32,850 | −22,433 (0.32×) |
| sqrt, constant time | BN254 scalar | 180,175 vs crypto-bigint 328,575 | −148,400 (0.55×) | 83,297 vs halo2curves 102,154 | −18,857 (0.82×) | 289,145 vs halo2curves 527,404 | −238,259 (0.55×) | 148,720 vs halo2curves 181,444 | −32,724 (0.82×) |
| sqrt, constant time | secp256k1 base | 41,156 vs crypto-bigint 159,173 | −118,017 (0.26×) | 18,851 vs halo2curves 43,366 | −24,515 (0.43×) | 65,022 vs crypto-bigint 252,479 | −187,457 (0.26×) | 27,999 vs crypto-bigint 79,559 | −51,560 (0.35×) |
| sqrt, constant time | P-256 base | 42,910 vs k256/p256 64,557 | −21,647 (0.66×) | 15,404 vs k256/p256 21,681 | −6,277 (0.71×) | 74,509 vs k256/p256 125,486 | −50,977 (0.59×) | 30,745 vs k256/p256 33,574 | −2,829 (0.92×) |
| sqrt, constant time | P-256 scalar | 77,925 vs halo2curves 152,676 | −74,751 (0.51×) | 36,678 vs halo2curves 49,711 | −13,033 (0.74×) | 116,129 vs crypto-bigint 249,548 | −133,419 (0.47×) | 66,567 vs halo2curves 72,503 | −5,936 (0.92×) |
| sqrt, constant time | Tom-256 base | 65,986 vs crypto-bigint 148,928 | −82,942 (0.44×) | 31,865 vs halo2curves 45,495 | −13,630 (0.70×) | 99,847 vs crypto-bigint 231,054 | −131,207 (0.43×) | 50,129 vs crypto-bigint 74,387 | −24,258 (0.67×) |
| sqrt, constant time | BLS12-381 base | 297,738 vs crypto-bigint 542,380 | −244,642 (0.55×) | 97,462 vs crypto-bigint 144,949 | −47,487 (0.67×) | 396,487 vs crypto-bigint 627,103 | −230,616 (0.63×) | 213,597 vs crypto-bigint 273,194 | −59,597 (0.78×) |
| sqrt, constant time | P-521 base | 324,679 vs crypto-bigint 1,531,110 | −1,206,431 (0.21×) | 131,004 vs crypto-bigint 296,892 | −165,888 (0.44×) | 507,930 vs crypto-bigint 1,563,440 | −1,055,510 (0.32×) | 256,756 vs crypto-bigint 597,804 | −341,048 (0.43×) |
| mul_u64, constant time | Mersenne-31 | 6.7 vs crypto-bigint 61.2 | −54.5 (0.11×) | 6.7 vs crypto-bigint 30.4 | −23.7 (0.22×) | 7.6 vs crypto-bigint 72.6 | −65.0 (0.10×) | 7.6 vs crypto-bigint 39.0 | −31.4 (0.19×) |
| mul_u64, constant time | BabyBear | 13.0 vs crypto-bigint 63.7 | −50.7 (0.20×) | 11.2 vs crypto-bigint 33.0 | −21.8 (0.34×) | 18.0 vs crypto-bigint 83.5 | −65.5 (0.22×) | 14.7 vs crypto-bigint 40.2 | −25.5 (0.37×) |
| mul_u64, constant time | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.4 | −6.6 (0.79×) | 9.0 vs winter-math 13.5 | −4.5 (0.67×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 871 | −765 (0.12×) | 38.9 vs crypto-bigint 263 | −224 (0.15×) | 125 vs halo2curves 1,428 | −1,303 (0.09×) | 57.4 vs crypto-bigint 406 | −349 (0.14×) |
| mul_u64, constant time | secp256k1 base | 61.3 vs halo2curves 692 | −631 (0.09×) | 24.9 vs crypto-bigint 263 | −238 (0.09×) | 80.0 vs halo2curves 1,062 | −982 (0.08×) | 37.2 vs halo2curves 383 | −346 (0.10×) |
| mul_u64, constant time | P-256 base | 85.5 vs k256/p256 485 | −400 (0.18×) | 40.3 vs k256/p256 150 | −110 (0.27×) | 102 vs k256/p256 951 | −849 (0.11×) | 57.1 vs k256/p256 258 | −201 (0.22×) |
| mul_u64, constant time | P-256 scalar | 102 vs k256/p256 519 | −417 (0.20×) | 43.5 vs k256/p256 253 | −210 (0.17×) | 122 vs k256/p256 826 | −704 (0.15×) | 60.9 vs k256/p256 338 | −277 (0.18×) |
| mul_u64, constant time | Tom-256 base | 95.3 vs crypto-bigint 876 | −781 (0.11×) | 41.7 vs crypto-bigint 263 | −221 (0.16×) | 122 vs halo2curves 1,259 | −1,137 (0.10×) | 59.3 vs crypto-bigint 408 | −349 (0.15×) |
| mul_u64, constant time | BLS12-381 base | 149 vs crypto-bigint 1,928 | −1,779 (0.08×) | 65.9 vs halo2curves 487 | −421 (0.14×) | 168 vs crypto-bigint 2,523 | −2,355 (0.07×) | 85.8 vs crypto-bigint 1,026 | −940 (0.08×) |
| mul_u64, constant time | P-521 base | 138 vs crypto-bigint 3,011 | −2,873 (0.05×) | 69.2 vs crypto-bigint 991 | −922 (0.07×) | 165 vs crypto-bigint 3,999 | −3,834 (0.04×) | 95.8 vs crypto-bigint 1,831 | −1,735 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 6.7 vs crypto-bigint 61.0 | −54.3 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 74.6 | −67.0 (0.10×) | 7.7 vs crypto-bigint 38.9 | −31.2 (0.20×) |
| mul_uint, constant time | BabyBear | 13.0 vs crypto-bigint 63.9 | −50.9 (0.20×) | 11.2 vs crypto-bigint 33.1 | −21.9 (0.34×) | 18.0 vs crypto-bigint 85.4 | −67.4 (0.21×) | 14.7 vs crypto-bigint 40.2 | −25.5 (0.37×) |
| mul_uint, constant time | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.8 | −3.5 (0.68×) | 24.8 vs winter-math 31.5 | −6.7 (0.79×) | 9.0 vs winter-math 13.5 | −4.5 (0.67×) |
| mul_uint, constant time | BN254 scalar | 487 vs halo2curves 756 | −269 (0.64×) | 181 vs halo2curves 182 | −1.0 (0.99×) | 605 vs halo2curves 1,316 | −711 (0.46×) | 308 vs halo2curves 349 | −41.0 (0.88×) |
| mul_uint, constant time | secp256k1 base | 320 vs halo2curves 519 | −199 (0.62×) | 88.1 vs halo2curves 153 | −64.9 (0.58×) | 377 vs halo2curves 969 | −592 (0.39×) | 119 vs halo2curves 265 | −146 (0.45×) |
| mul_uint, constant time | P-256 base | 412 vs halo2curves 456 | −44.0 (0.90×) | 131 vs halo2curves 143 | −12.0 (0.92×) | 492 vs halo2curves 947 | −455 (0.52×) | 224 vs halo2curves 297 | −73.0 (0.75×) |
| mul_uint, constant time | P-256 scalar | 471 vs k256/p256 557 | −86.0 (0.85×) | 180 vs halo2curves 183 | −3.0 (0.98×) | 565 vs k256/p256 887 | −322 (0.64×) | 341 vs halo2curves 346 | −5.0 (0.99×) |
| mul_uint, constant time | Tom-256 base | 461 vs halo2curves 696 | −235 (0.66×) | 165 vs halo2curves 182 | −17.0 (0.91×) | 557 vs halo2curves 1,191 | −634 (0.47×) | 306 vs halo2curves 335 | −29.0 (0.91×) |
| mul_uint, constant time | BLS12-381 base | 1,083 vs crypto-bigint 1,925 | −842 (0.56×) | 366 vs halo2curves 381 | −15.0 (0.96×) | 1,679 vs crypto-bigint 2,517 | −838 (0.67×) | 926 vs crypto-bigint 927 | −1.0 (1.00×) |
| mul_uint, constant time | P-521 base | 1,997 vs crypto-bigint 3,026 | −1,029 (0.66×) | 315 vs crypto-bigint 990 | −675 (0.32×) | 2,920 vs crypto-bigint 4,025 | −1,105 (0.73×) | 623 vs crypto-bigint 1,872 | −1,249 (0.33×) |
| add_u64, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.3 | −25.1 (0.17×) | 5.7 vs crypto-bigint 41.4 | −35.7 (0.14×) | 5.4 vs crypto-bigint 30.5 | −25.1 (0.18×) |
| add_u64, constant time | BabyBear | 10.3 vs crypto-bigint 39.9 | −29.6 (0.26×) | 8.4 vs crypto-bigint 31.6 | −23.2 (0.27×) | 12.8 vs crypto-bigint 53.8 | −41.0 (0.24×) | 12.2 vs crypto-bigint 31.6 | −19.4 (0.39×) |
| add_u64, constant time | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 10.0 vs winter-math 10.2 | −0.2 (0.98×) |
| add_u64, constant time | BN254 scalar | 93.6 vs crypto-bigint 501 | −407 (0.19×) | 50.1 vs halo2curves 209 | −159 (0.24×) | 108 vs halo2curves 774 | −666 (0.14×) | 73.9 vs halo2curves 297 | −223 (0.25×) |
| add_u64, constant time | secp256k1 base | 22.4 vs halo2curves 398 | −376 (0.06×) | 16.1 vs halo2curves 203 | −187 (0.08×) | 20.2 vs halo2curves 477 | −457 (0.04×) | 25.0 vs halo2curves 243 | −218 (0.10×) |
| add_u64, constant time | P-256 base | 70.8 vs k256/p256 276 | −205 (0.26×) | 44.0 vs k256/p256 97.0 | −53.0 (0.45×) | 77.1 vs k256/p256 510 | −433 (0.15×) | 60.3 vs k256/p256 159 | −98.7 (0.38×) |
| add_u64, constant time | P-256 scalar | 92.2 vs k256/p256 41.1 | +51.1 (2.24×) | 50.4 vs k256/p256 41.2 | +9.2 (1.22×) | 112 vs k256/p256 54.5 | +57.5 (2.06×) | 72.6 vs k256/p256 54.7 | +17.9 (1.33×) |
| add_u64, constant time | Tom-256 base | 94.4 vs crypto-bigint 506 | −412 (0.19×) | 48.7 vs halo2curves 218 | −169 (0.22×) | 111 vs halo2curves 698 | −587 (0.16×) | 67.9 vs halo2curves 291 | −223 (0.23×) |
| add_u64, constant time | BLS12-381 base | 147 vs crypto-bigint 1,066 | −919 (0.14×) | 98.6 vs halo2curves 322 | −223 (0.31×) | 172 vs crypto-bigint 1,408 | −1,236 (0.12×) | 114 vs crypto-bigint 649 | −535 (0.18×) |
| add_u64, constant time | P-521 base | 65.0 vs crypto-bigint 1,672 | −1,607 (0.04×) | 44.2 vs crypto-bigint 647 | −603 (0.07×) | 69.5 vs crypto-bigint 2,228 | −2,159 (0.03×) | 64.2 vs crypto-bigint 1,113 | −1,049 (0.06×) |
| add_uint, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.7 vs crypto-bigint 41.9 | −36.2 (0.14×) | 5.4 vs crypto-bigint 30.8 | −25.4 (0.18×) |
| add_uint, constant time | BabyBear | 10.4 vs crypto-bigint 39.9 | −29.5 (0.26×) | 8.5 vs crypto-bigint 31.6 | −23.1 (0.27×) | 12.8 vs crypto-bigint 53.7 | −40.9 (0.24×) | 12.1 vs crypto-bigint 32.4 | −20.3 (0.37×) |
| add_uint, constant time | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.0 vs winter-math 7.4 | −0.4 (0.95×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 10.0 vs winter-math 10.2 | −0.2 (0.98×) |
| add_uint, constant time | BN254 scalar | 249 vs halo2curves 412 | −163 (0.60×) | 102 vs halo2curves 112 | −10.0 (0.91×) | 352 vs halo2curves 685 | −333 (0.51×) | 182 vs halo2curves 205 | −23.0 (0.89×) |
| add_uint, constant time | secp256k1 base | 41.7 vs halo2curves 224 | −182 (0.19×) | 29.5 vs halo2curves 85.3 | −55.8 (0.35×) | 37.0 vs halo2curves 426 | −389 (0.09×) | 44.4 vs halo2curves 140 | −95.6 (0.32×) |
| add_uint, constant time | P-256 base | 180 vs halo2curves 246 | −66.0 (0.73×) | 104 vs halo2curves 91.5 | +12.5 (1.14×) | 283 vs halo2curves 511 | −228 (0.55×) | 145 vs halo2curves 154 | −9.0 (0.94×) |
| add_uint, constant time | P-256 scalar | 260 vs k256/p256 94.6 | +165 (2.75×) | 108 vs k256/p256 94.8 | +13.2 (1.14×) | 319 vs k256/p256 118 | +201 (2.70×) | 175 vs k256/p256 118 | +57.0 (1.48×) |
| add_uint, constant time | Tom-256 base | 251 vs halo2curves 380 | −129 (0.66×) | 102 vs halo2curves 110 | −8.0 (0.93×) | 317 vs halo2curves 636 | −319 (0.50×) | 182 vs halo2curves 196 | −14.0 (0.93×) |
| add_uint, constant time | BLS12-381 base | 769 vs halo2curves 981 | −212 (0.78×) | 208 vs halo2curves 225 | −17.0 (0.92×) | 931 vs crypto-bigint 1,392 | −461 (0.67×) | 388 vs halo2curves 566 | −178 (0.69×) |
| add_uint, constant time | P-521 base | 135 vs crypto-bigint 1,684 | −1,549 (0.08×) | 105 vs crypto-bigint 648 | −543 (0.16×) | 144 vs crypto-bigint 2,229 | −2,085 (0.06×) | 136 vs crypto-bigint 1,135 | −999 (0.12×) |
| sub_u64, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.4 | −18.4 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.1 vs crypto-bigint 31.2 | −26.1 (0.16×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_u64, constant time | BabyBear | 11.2 vs crypto-bigint 27.9 | −16.7 (0.40×) | 8.7 vs crypto-bigint 19.5 | −10.8 (0.45×) | 14.0 vs crypto-bigint 43.7 | −29.7 (0.32×) | 12.0 vs crypto-bigint 21.8 | −9.8 (0.55×) |
| sub_u64, constant time | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.3 vs winter-math 9.5 | −0.2 (0.98×) |
| sub_u64, constant time | BN254 scalar | 91.2 vs crypto-bigint 493 | −402 (0.18×) | 42.4 vs crypto-bigint 151 | −109 (0.28×) | 109 vs halo2curves 762 | −653 (0.14×) | 71.9 vs crypto-bigint 261 | −189 (0.28×) |
| sub_u64, constant time | secp256k1 base | 15.5 vs halo2curves 371 | −356 (0.04×) | 12.3 vs crypto-bigint 150 | −138 (0.08×) | 14.6 vs halo2curves 465 | −450 (0.03×) | 18.5 vs halo2curves 237 | −219 (0.08×) |
| sub_u64, constant time | P-256 base | 63.6 vs k256/p256 264 | −200 (0.24×) | 36.1 vs k256/p256 88.9 | −52.8 (0.41×) | 75.4 vs k256/p256 493 | −418 (0.15×) | 54.8 vs k256/p256 149 | −94.2 (0.37×) |
| sub_u64, constant time | P-256 scalar | 88.4 vs k256/p256 21.2 | +67.2 (4.17×) | 43.9 vs k256/p256 21.2 | +22.7 (2.07×) | 108 vs k256/p256 24.4 | +83.6 (4.43×) | 69.6 vs k256/p256 24.3 | +45.3 (2.86×) |
| sub_u64, constant time | Tom-256 base | 93.0 vs crypto-bigint 501 | −408 (0.19×) | 43.3 vs crypto-bigint 151 | −108 (0.29×) | 106 vs halo2curves 687 | −581 (0.15×) | 67.5 vs crypto-bigint 259 | −192 (0.26×) |
| sub_u64, constant time | BLS12-381 base | 142 vs crypto-bigint 1,076 | −934 (0.13×) | 93.5 vs halo2curves 316 | −223 (0.30×) | 177 vs crypto-bigint 1,372 | −1,195 (0.13×) | 105 vs crypto-bigint 617 | −512 (0.17×) |
| sub_u64, constant time | P-521 base | 42.2 vs crypto-bigint 1,659 | −1,617 (0.03×) | 34.1 vs crypto-bigint 622 | −588 (0.05×) | 52.0 vs crypto-bigint 2,211 | −2,159 (0.02×) | 45.5 vs crypto-bigint 1,098 | −1,053 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.5 | −18.5 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_uint, constant time | BabyBear | 11.2 vs crypto-bigint 28.0 | −16.8 (0.40×) | 8.7 vs crypto-bigint 19.5 | −10.8 (0.45×) | 14.0 vs crypto-bigint 43.9 | −29.9 (0.32×) | 12.0 vs crypto-bigint 21.8 | −9.8 (0.55×) |
| sub_uint, constant time | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.3 vs winter-math 9.5 | −0.2 (0.98×) |
| sub_uint, constant time | BN254 scalar | 253 vs halo2curves 401 | −148 (0.63×) | 93.5 vs halo2curves 103 | −9.5 (0.91×) | 338 vs halo2curves 665 | −327 (0.51×) | 175 vs halo2curves 194 | −19.0 (0.90×) |
| sub_uint, constant time | secp256k1 base | 32.9 vs halo2curves 207 | −174 (0.16×) | 26.0 vs halo2curves 109 | −83.0 (0.24×) | 34.3 vs halo2curves 410 | −376 (0.08×) | 37.5 vs halo2curves 130 | −92.5 (0.29×) |
| sub_uint, constant time | P-256 base | 171 vs halo2curves 232 | −61.0 (0.74×) | 95.0 vs halo2curves 85.1 | +9.9 (1.12×) | 274 vs halo2curves 497 | −223 (0.55×) | 136 vs halo2curves 144 | −8.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 248 vs k256/p256 67.3 | +181 (3.68×) | 106 vs k256/p256 68.0 | +38.0 (1.56×) | 333 vs k256/p256 87.4 | +246 (3.81×) | 189 vs k256/p256 88.6 | +100 (2.13×) |
| sub_uint, constant time | Tom-256 base | 252 vs halo2curves 365 | −113 (0.69×) | 97.2 vs halo2curves 104 | −6.8 (0.93×) | 321 vs halo2curves 623 | −302 (0.52×) | 179 vs halo2curves 187 | −8.0 (0.96×) |
| sub_uint, constant time | BLS12-381 base | 761 vs halo2curves 952 | −191 (0.80×) | 209 vs halo2curves 216 | −7.0 (0.97×) | 928 vs crypto-bigint 1,368 | −440 (0.68×) | 382 vs halo2curves 544 | −162 (0.70×) |
| sub_uint, constant time | P-521 base | 127 vs crypto-bigint 1,673 | −1,546 (0.08×) | 86.9 vs crypto-bigint 620 | −533 (0.14×) | 124 vs crypto-bigint 2,211 | −2,087 (0.06×) | 114 vs crypto-bigint 1,105 | −991 (0.10×) |
| from_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 18.0 | −13.3 (0.26×) | 4.7 vs crypto-bigint 11.6 | −6.9 (0.41×) | 5.2 vs crypto-bigint 25.1 | −19.9 (0.21×) | 5.2 vs crypto-bigint 13.2 | −8.0 (0.39×) |
| from_u64, constant time | BabyBear | 10.0 vs crypto-bigint 21.5 | −11.5 (0.47×) | 6.8 vs crypto-bigint 12.6 | −5.8 (0.54×) | 13.3 vs crypto-bigint 35.3 | −22.0 (0.38×) | 10.5 vs crypto-bigint 15.5 | −5.0 (0.68×) |
| from_u64, constant time | Goldilocks | 7.4 vs winter-math 10.6 | −3.2 (0.70×) | 6.0 vs winter-math 5.9 | +0.1 (1.02×) | 9.0 vs winter-math 17.1 | −8.1 (0.53×) | 7.0 vs winter-math 7.0 | +0.0 (1.00×) |
| from_u64, constant time | BN254 scalar | 83.6 vs crypto-bigint 433 | −349 (0.19×) | 34.7 vs crypto-bigint 133 | −98.3 (0.26×) | 98.1 vs halo2curves 713 | −615 (0.14×) | 50.0 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64, constant time | secp256k1 base | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.7 vs k256/p256 5.5 | −1.8 (0.67×) |
| from_u64, constant time | P-256 base | 51.5 vs k256/p256 238 | −187 (0.22×) | 26.7 vs k256/p256 75.7 | −49.0 (0.35×) | 61.3 vs k256/p256 477 | −416 (0.13×) | 38.5 vs k256/p256 130 | −91.5 (0.30×) |
| from_u64, constant time | P-256 scalar | 84.1 vs k256/p256 3.8 | +80.3 (22.13×) | 34.6 vs k256/p256 3.8 | +30.8 (9.11×) | 96.8 vs k256/p256 3.7 | +93.1 (26.16×) | 50.0 vs k256/p256 3.8 | +46.2 (13.16×) |
| from_u64, constant time | Tom-256 base | 82.0 vs crypto-bigint 435 | −353 (0.19×) | 34.8 vs crypto-bigint 133 | −98.2 (0.26×) | 100 vs halo2curves 647 | −547 (0.15×) | 49.0 vs crypto-bigint 204 | −155 (0.24×) |
| from_u64, constant time | BLS12-381 base | 145 vs crypto-bigint 916 | −771 (0.16×) | 56.6 vs crypto-bigint 258 | −201 (0.22×) | 198 vs crypto-bigint 1,245 | −1,047 (0.16×) | 66.1 vs crypto-bigint 473 | −407 (0.14×) |
| from_u64, constant time | P-521 base | 6.0 vs crypto-bigint 1,445 | −1,439 (0.00×) | 6.0 vs crypto-bigint 499 | −493 (0.01×) | 6.0 vs crypto-bigint 1,991 | −1,985 (0.00×) | 6.0 vs crypto-bigint 808 | −802 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,341 vs crypto-bigint 28,131 | −15,790 (0.44×) | 9,385 vs crypto-bigint 24,433 | −15,048 (0.38×) | 17,656 vs crypto-bigint 37,477 | −19,821 (0.47×) | 12,998 vs crypto-bigint 31,394 | −18,396 (0.41×) |
| div_u64, constant time | secp256k1 base | 11,913 vs crypto-bigint 28,549 | −16,636 (0.42×) | 9,038 vs crypto-bigint 24,625 | −15,587 (0.37×) | 16,890 vs crypto-bigint 37,625 | −20,735 (0.45×) | 12,813 vs crypto-bigint 31,381 | −18,568 (0.41×) |
| div_u64, constant time | P-256 base | 11,610 vs crypto-bigint 28,324 | −16,714 (0.41×) | 9,169 vs k256/p256 22,976 | −13,807 (0.40×) | 16,906 vs crypto-bigint 37,446 | −20,540 (0.45×) | 12,446 vs crypto-bigint 31,569 | −19,123 (0.39×) |
| div_u64, constant time | P-256 scalar | 12,254 vs crypto-bigint 28,817 | −16,563 (0.43×) | 9,514 vs crypto-bigint 24,372 | −14,858 (0.39×) | 17,121 vs crypto-bigint 37,520 | −20,399 (0.46×) | 13,110 vs crypto-bigint 31,271 | −18,161 (0.42×) |
| div_u64, constant time | Tom-256 base | 12,219 vs crypto-bigint 28,495 | −16,276 (0.43×) | 9,271 vs crypto-bigint 24,372 | −15,101 (0.38×) | 16,953 vs crypto-bigint 37,547 | −20,594 (0.45×) | 12,893 vs crypto-bigint 31,444 | −18,551 (0.41×) |
| div_u64, constant time | BLS12-381 base | 24,771 vs crypto-bigint 51,948 | −27,177 (0.48×) | 16,799 vs crypto-bigint 43,670 | −26,871 (0.38×) | 34,564 vs crypto-bigint 68,968 | −34,404 (0.50×) | 24,347 vs crypto-bigint 54,858 | −30,511 (0.44×) |
| div_u64, constant time | P-521 base | 42,464 vs crypto-bigint 106,623 | −64,159 (0.40×) | 25,848 vs crypto-bigint 83,105 | −57,257 (0.31×) | 64,233 vs crypto-bigint 125,651 | −61,418 (0.51×) | 42,966 vs crypto-bigint 101,243 | −58,277 (0.42×) |
| div_uint, constant time | Mersenne-31 | 272 vs runtime-algebra 3,506 | −3,234 (0.08×) | 272 vs runtime-algebra 3,505 | −3,233 (0.08×) | 280 vs runtime-algebra 3,524 | −3,244 (0.08×) | 273 vs runtime-algebra 3,418 | −3,145 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,558 | −3,089 (0.13×) | 465 vs runtime-algebra 3,558 | −3,093 (0.13×) | 474 vs runtime-algebra 3,519 | −3,045 (0.13×) | 447 vs runtime-algebra 3,411 | −2,964 (0.13×) |
| div_uint, constant time | Goldilocks | 1,146 vs winter-math 1,818 | −672 (0.63×) | 863 vs winter-math 952 | −89.0 (0.91×) | 1,437 vs winter-math 1,882 | −445 (0.76×) | 958 vs winter-math 1,041 | −83.0 (0.92×) |
| div_uint, constant time | BN254 scalar | 12,492 vs crypto-bigint 28,660 | −16,168 (0.44×) | 9,611 vs crypto-bigint 24,211 | −14,600 (0.40×) | 17,824 vs crypto-bigint 37,458 | −19,634 (0.48×) | 13,360 vs crypto-bigint 31,458 | −18,098 (0.42×) |
| div_uint, constant time | secp256k1 base | 11,984 vs crypto-bigint 28,685 | −16,701 (0.42×) | 9,095 vs crypto-bigint 24,281 | −15,186 (0.37×) | 16,891 vs crypto-bigint 37,474 | −20,583 (0.45×) | 12,869 vs crypto-bigint 31,385 | −18,516 (0.41×) |
| div_uint, constant time | P-256 base | 12,021 vs crypto-bigint 29,427 | −17,406 (0.41×) | 9,105 vs k256/p256 23,618 | −14,513 (0.39×) | 17,274 vs crypto-bigint 37,259 | −19,985 (0.46×) | 12,625 vs crypto-bigint 31,416 | −18,791 (0.40×) |
| div_uint, constant time | P-256 scalar | 12,352 vs crypto-bigint 28,627 | −16,275 (0.43×) | 9,506 vs crypto-bigint 24,118 | −14,612 (0.39×) | 17,361 vs crypto-bigint 37,374 | −20,013 (0.46×) | 13,524 vs crypto-bigint 31,126 | −17,602 (0.43×) |
| div_uint, constant time | Tom-256 base | 12,384 vs crypto-bigint 28,769 | −16,385 (0.43×) | 9,459 vs crypto-bigint 24,580 | −15,121 (0.38×) | 16,998 vs crypto-bigint 37,574 | −20,576 (0.45×) | 13,046 vs crypto-bigint 31,403 | −18,357 (0.42×) |
| div_uint, constant time | BLS12-381 base | 25,380 vs crypto-bigint 51,927 | −26,547 (0.49×) | 16,845 vs crypto-bigint 42,528 | −25,683 (0.40×) | 35,281 vs crypto-bigint 68,728 | −33,447 (0.51×) | 24,786 vs crypto-bigint 54,613 | −29,827 (0.45×) |
| div_uint, constant time | P-521 base | 42,465 vs crypto-bigint 105,544 | −63,079 (0.40×) | 26,245 vs crypto-bigint 83,801 | −57,556 (0.31×) | 64,579 vs crypto-bigint 126,102 | −61,523 (0.51×) | 43,052 vs crypto-bigint 101,370 | −58,318 (0.42×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.9 | −14.9 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) |
| add | BN254 scalar | 13.1 vs ark-ff 13.4 | −0.3 (0.98×) | 10.8 vs ark-ff 11.2 | −0.4 (0.96×) | 13.4 vs ark-ff 13.6 | −0.2 (0.99×) | 17.9 vs ark-ff 16.4 | +1.5 (1.09×) |
| add | secp256k1 base | 13.7 vs ark-ff 15.5 | −1.8 (0.88×) | 10.7 vs ark-ff 11.2 | −0.5 (0.96×) | 13.7 vs ark-ff 14.9 | −1.2 (0.92×) | 18.3 vs ark-ff 17.9 | +0.4 (1.02×) |
| add | P-256 base | 12.2 vs ark-ff 15.2 | −3.0 (0.80×) | 11.2 vs ark-ff 11.2 | +0.0 (1.00×) | 13.0 vs ark-ff 15.7 | −2.7 (0.83×) | 18.8 vs ark-ff 19.3 | −0.5 (0.97×) |
| add | P-256 scalar | 13.6 vs ark-ff 15.2 | −1.6 (0.89×) | 11.1 vs ark-ff 11.2 | −0.1 (0.99×) | 14.5 vs ark-ff 15.5 | −1.0 (0.94×) | 19.5 vs ark-ff 17.9 | +1.6 (1.09×) |
| add | Tom-256 base | 13.3 vs halo2curves 42.9 | −29.6 (0.31×) | 11.2 vs halo2curves 24.0 | −12.8 (0.47×) | 13.1 vs halo2curves 42.5 | −29.4 (0.31×) | 18.4 vs halo2curves 33.0 | −14.6 (0.56×) |
| add | BLS12-381 base | 22.0 vs ark-ff 44.1 | −22.1 (0.50×) | 18.7 vs halo2curves 34.1 | −15.4 (0.55×) | 23.2 vs ark-ff 48.0 | −24.8 (0.48×) | 30.4 vs ark-ff 45.8 | −15.4 (0.66×) |
| add | P-521 base | 41.4 vs crypto-bigint 181 | −140 (0.23×) | 26.8 vs crypto-bigint 179 | −152 (0.15×) | 47.0 vs crypto-bigint 246 | −199 (0.19×) | 48.4 vs crypto-bigint 245 | −197 (0.20×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 13.1 vs ark-ff 16.5 | −3.4 (0.79×) | 10.7 vs ark-ff 13.9 | −3.2 (0.77×) | 13.3 vs ark-ff 16.6 | −3.3 (0.80×) | 18.0 vs ark-ff 21.4 | −3.4 (0.84×) |
| sub | secp256k1 base | 16.6 vs ark-ff 16.6 | +0.0 (1.00×) | 10.8 vs ark-ff 13.9 | −3.1 (0.78×) | 13.9 vs ark-ff 17.1 | −3.2 (0.81×) | 17.9 vs halo2curves 21.0 | −3.1 (0.85×) |
| sub | P-256 base | 13.4 vs ark-ff 16.4 | −3.0 (0.82×) | 10.4 vs ark-ff 12.9 | −2.5 (0.81×) | 12.5 vs ark-ff 15.2 | −2.7 (0.82×) | 17.5 vs k256/p256 20.0 | −2.5 (0.88×) |
| sub | P-256 scalar | 14.3 vs ark-ff 16.4 | −2.1 (0.87×) | 10.7 vs ark-ff 14.0 | −3.3 (0.76×) | 13.0 vs ark-ff 16.6 | −3.6 (0.78×) | 17.8 vs halo2curves 21.5 | −3.7 (0.83×) |
| sub | Tom-256 base | 13.1 vs halo2curves 26.1 | −13.0 (0.50×) | 10.8 vs halo2curves 15.0 | −4.2 (0.72×) | 12.7 vs halo2curves 26.5 | −13.8 (0.48×) | 18.6 vs halo2curves 22.0 | −3.4 (0.85×) |
| sub | BLS12-381 base | 22.8 vs ark-ff 30.5 | −7.7 (0.75×) | 17.5 vs ark-ff 23.6 | −6.1 (0.74×) | 22.6 vs ark-ff 30.1 | −7.5 (0.75×) | 30.4 vs halo2curves 36.2 | −5.8 (0.84×) |
| sub | P-521 base | 43.2 vs crypto-bigint 159 | −116 (0.27×) | 26.9 vs crypto-bigint 144 | −117 (0.19×) | 43.6 vs crypto-bigint 174 | −130 (0.25×) | 45.9 vs crypto-bigint 217 | −171 (0.21×) |
| mul | Mersenne-31 | 7.0 vs crypto-bigint 46.5 | −39.5 (0.15×) | 7.0 vs crypto-bigint 27.0 | −20.0 (0.26×) | 7.0 vs crypto-bigint 49.0 | −42.0 (0.14×) | 7.0 vs crypto-bigint 30.3 | −23.3 (0.23×) |
| mul | BabyBear | 12.0 vs plonky3 12.2 | −0.2 (0.98×) | 11.0 vs plonky3 12.1 | −1.1 (0.91×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| mul | Goldilocks | 16.5 vs winter-math 18.0 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.4 vs winter-math 19.0 | −0.6 (0.97×) | 12.3 vs winter-math 14.0 | −1.7 (0.88×) |
| mul | BN254 scalar | 227 vs ark-ff 316 | −89.0 (0.72×) | 92.7 vs halo2curves 91.4 | +1.3 (1.01×) | 367 vs ark-ff 388 | −21.0 (0.95×) | 152 vs halo2curves 178 | −26.0 (0.85×) |
| mul | secp256k1 base | 173 vs ark-ff 275 | −102 (0.63×) | 77.5 vs halo2curves 90.5 | −13.0 (0.86×) | 218 vs ark-ff 404 | −186 (0.54×) | 105 vs halo2curves 159 | −54.0 (0.66×) |
| mul | P-256 base | 188 vs k256/p256 243 | −55.0 (0.77×) | 71.7 vs halo2curves 74.5 | −2.8 (0.96×) | 292 vs ark-ff 439 | −147 (0.67×) | 119 vs halo2curves 130 | −11.0 (0.92×) |
| mul | P-256 scalar | 234 vs ark-ff 311 | −77.0 (0.75×) | 93.2 vs halo2curves 95.3 | −2.1 (0.98×) | 319 vs ark-ff 407 | −88.0 (0.78×) | 170 vs halo2curves 176 | −6.0 (0.97×) |
| mul | Tom-256 base | 235 vs halo2curves 344 | −109 (0.68×) | 86.6 vs halo2curves 93.5 | −6.9 (0.93×) | 329 vs halo2curves 602 | −273 (0.55×) | 151 vs halo2curves 172 | −21.0 (0.88×) |
| mul | BLS12-381 base | 723 vs ark-ff 682 | +41.0 (1.06×) | 186 vs halo2curves 194 | −8.0 (0.96×) | 928 vs ark-ff 1,062 | −134 (0.87×) | 336 vs halo2curves 377 | −41.0 (0.89×) |
| mul | P-521 base | 684 vs crypto-bigint 2,131 | −1,447 (0.32×) | 247 vs crypto-bigint 618 | −371 (0.40×) | 1,312 vs crypto-bigint 2,795 | −1,483 (0.47×) | 444 vs crypto-bigint 1,117 | −673 (0.40×) |
| square | Mersenne-31 | 7.0 vs crypto-bigint 45.4 | −38.4 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.5 | −43.5 (0.14×) | 7.0 vs crypto-bigint 29.9 | −22.9 (0.23×) |
| square | BabyBear | 12.0 vs plonky3 12.1 | −0.1 (0.99×) | 11.0 vs plonky3 12.0 | −1.0 (0.92×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| square | Goldilocks | 15.0 vs winter-math 19.6 | −4.6 (0.77×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.5 vs winter-math 20.0 | −1.5 (0.93×) | 12.7 vs winter-math 14.3 | −1.6 (0.89×) |
| square | BN254 scalar | 210 vs ark-ff 265 | −55.0 (0.79×) | 83.0 vs halo2curves 96.6 | −13.6 (0.86×) | 326 vs ark-ff 355 | −29.0 (0.92×) | 140 vs halo2curves 142 | −2.0 (0.99×) |
| square | secp256k1 base | 150 vs halo2curves 222 | −72.0 (0.68×) | 67.2 vs halo2curves 84.8 | −17.6 (0.79×) | 181 vs ark-ff 295 | −114 (0.61×) | 87.1 vs halo2curves 119 | −31.9 (0.73×) |
| square | P-256 base | 160 vs halo2curves 176 | −16.0 (0.91×) | 62.0 vs halo2curves 61.0 | +1.0 (1.02×) | 237 vs ark-ff 334 | −97.0 (0.71×) | 102 vs halo2curves 108 | −6.0 (0.94×) |
| square | P-256 scalar | 221 vs ark-ff 258 | −37.0 (0.86×) | 88.2 vs halo2curves 90.1 | −1.9 (0.98×) | 279 vs ark-ff 342 | −63.0 (0.82×) | 143 vs halo2curves 140 | +3.0 (1.02×) |
| square | Tom-256 base | 212 vs halo2curves 270 | −58.0 (0.79×) | 82.3 vs halo2curves 88.6 | −6.3 (0.93×) | 211 vs halo2curves 471 | −260 (0.45×) | 126 vs halo2curves 136 | −10.0 (0.93×) |
| square | BLS12-381 base | 570 vs ark-ff 621 | −51.0 (0.92×) | 169 vs halo2curves 168 | +1.0 (1.01×) | 794 vs ark-ff 819 | −25.0 (0.97×) | 294 vs halo2curves 296 | −2.0 (0.99×) |
| square | P-521 base | 570 vs crypto-bigint 2,159 | −1,589 (0.26×) | 219 vs crypto-bigint 633 | −414 (0.35×) | 825 vs crypto-bigint 2,854 | −2,029 (0.29×) | 391 vs crypto-bigint 1,152 | −761 (0.34×) |
| mul_u64 | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 31.6 | −22.6 (0.28×) | 9.0 vs crypto-bigint 70.4 | −61.4 (0.13×) | 9.0 vs crypto-bigint 38.8 | −29.8 (0.23×) |
| mul_u64 | BabyBear | 16.1 vs plonky3 34.2 | −18.1 (0.47×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.7 vs plonky3 45.1 | −29.4 (0.35×) | 13.0 vs plonky3 16.6 | −3.6 (0.78×) |
| mul_u64 | Goldilocks | 17.8 vs winter-math 23.3 | −5.5 (0.76×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.2 vs winter-math 34.5 | −9.3 (0.73×) | 15.1 vs winter-math 15.5 | −0.4 (0.97×) |
| mul_u64 | BN254 scalar | 89.0 vs ark-ff 662 | −573 (0.13×) | 34.5 vs crypto-bigint 261 | −227 (0.13×) | 121 vs ark-ff 835 | −714 (0.14×) | 45.1 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | secp256k1 base | 49.0 vs ark-ff 433 | −384 (0.11×) | 16.0 vs crypto-bigint 259 | −243 (0.06×) | 65.0 vs ark-ff 605 | −540 (0.11×) | 23.0 vs halo2curves 386 | −363 (0.06×) |
| mul_u64 | P-256 base | 74.9 vs k256/p256 486 | −411 (0.15×) | 33.0 vs k256/p256 151 | −118 (0.22×) | 95.2 vs ark-ff 828 | −733 (0.11×) | 42.3 vs k256/p256 258 | −216 (0.16×) |
| mul_u64 | P-256 scalar | 81.8 vs k256/p256 516 | −434 (0.16×) | 34.5 vs k256/p256 255 | −221 (0.14×) | 105 vs ark-ff 822 | −717 (0.13×) | 47.6 vs k256/p256 338 | −290 (0.14×) |
| mul_u64 | Tom-256 base | 84.6 vs crypto-bigint 844 | −759 (0.10×) | 33.0 vs crypto-bigint 258 | −225 (0.13×) | 105 vs halo2curves 1,260 | −1,155 (0.08×) | 45.4 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | BLS12-381 base | 124 vs ark-ff 1,634 | −1,510 (0.08×) | 47.1 vs halo2curves 490 | −443 (0.10×) | 177 vs ark-ff 2,097 | −1,920 (0.08×) | 62.0 vs crypto-bigint 1,012 | −950 (0.06×) |
| mul_u64 | P-521 base | 94.2 vs crypto-bigint 3,059 | −2,965 (0.03×) | 39.5 vs crypto-bigint 1,000 | −961 (0.04×) | 199 vs crypto-bigint 3,999 | −3,800 (0.05×) | 52.5 vs crypto-bigint 1,846 | −1,794 (0.03×) |
| mul_uint | Mersenne-31 | 9.0 vs crypto-bigint 59.6 | −50.6 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 73.8 | −64.8 (0.12×) | 9.0 vs crypto-bigint 38.2 | −29.2 (0.24×) |
| mul_uint | BabyBear | 16.1 vs plonky3 23.6 | −7.5 (0.68×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.6 vs plonky3 33.7 | −18.1 (0.46×) | 13.0 vs plonky3 16.6 | −3.6 (0.78×) |
| mul_uint | Goldilocks | 17.8 vs winter-math 23.6 | −5.8 (0.75×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.2 vs winter-math 34.5 | −9.3 (0.73×) | 15.2 vs winter-math 14.3 | +0.9 (1.06×) |
| mul_uint | BN254 scalar | 440 vs halo2curves 749 | −309 (0.59×) | 167 vs halo2curves 181 | −14.0 (0.92×) | 532 vs halo2curves 1,313 | −781 (0.41×) | 298 vs crypto-bigint 406 | −108 (0.73×) |
| mul_uint | secp256k1 base | 265 vs halo2curves 517 | −252 (0.51×) | 74.6 vs halo2curves 154 | −79.4 (0.48×) | 320 vs halo2curves 968 | −648 (0.33×) | 108 vs halo2curves 264 | −156 (0.41×) |
| mul_uint | P-256 base | 355 vs halo2curves 454 | −99.0 (0.78×) | 137 vs halo2curves 146 | −9.0 (0.94×) | 423 vs halo2curves 953 | −530 (0.44×) | 232 vs halo2curves 256 | −24.0 (0.91×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 551 | −126 (0.77×) | 178 vs halo2curves 185 | −7.0 (0.96×) | 472 vs k256/p256 880 | −408 (0.54×) | 293 vs halo2curves 351 | −58.0 (0.83×) |
| mul_uint | Tom-256 base | 410 vs halo2curves 696 | −286 (0.59×) | 171 vs halo2curves 181 | −10.0 (0.94×) | 487 vs halo2curves 1,177 | −690 (0.41×) | 287 vs halo2curves 335 | −48.0 (0.86×) |
| mul_uint | BLS12-381 base | 982 vs halo2curves 1,905 | −923 (0.52×) | 360 vs halo2curves 377 | −17.0 (0.95×) | 1,141 vs crypto-bigint 2,499 | −1,358 (0.46×) | 897 vs crypto-bigint 1,039 | −142 (0.86×) |
| mul_uint | P-521 base | 1,573 vs crypto-bigint 3,051 | −1,478 (0.52×) | 253 vs crypto-bigint 1,009 | −756 (0.25×) | 2,381 vs crypto-bigint 4,014 | −1,633 (0.59×) | 538 vs crypto-bigint 1,858 | −1,320 (0.29×) |
| add_u64 | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.8 vs crypto-bigint 43.3 | −38.5 (0.11×) | 4.7 vs crypto-bigint 30.5 | −25.8 (0.15×) |
| add_u64 | BabyBear | 12.2 vs plonky3 30.6 | −18.4 (0.40×) | 8.2 vs plonky3 9.1 | −0.9 (0.90×) | 12.7 vs plonky3 41.8 | −29.1 (0.30×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_u64 | Goldilocks | 8.5 vs winter-math 13.2 | −4.7 (0.64×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.6 vs winter-math 19.1 | −7.5 (0.61×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_u64 | BN254 scalar | 113 vs ark-ff 340 | −227 (0.33×) | 44.7 vs halo2curves 210 | −165 (0.21×) | 140 vs ark-ff 402 | −262 (0.35×) | 65.7 vs halo2curves 299 | −233 (0.22×) |
| add_u64 | secp256k1 base | 7.0 vs ark-ff 158 | −151 (0.04×) | 6.0 vs ark-ff 119 | −113 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 10.0 vs ark-ff 185 | −175 (0.05×) |
| add_u64 | P-256 base | 67.7 vs ark-ff 262 | −194 (0.26×) | 34.9 vs k256/p256 92.5 | −57.6 (0.38×) | 77.5 vs ark-ff 398 | −321 (0.19×) | 54.9 vs k256/p256 158 | −103 (0.35×) |
| add_u64 | P-256 scalar | 66.8 vs k256/p256 56.2 | +10.6 (1.19×) | 38.0 vs k256/p256 56.0 | −18.0 (0.68×) | 74.7 vs k256/p256 62.4 | +12.3 (1.20×) | 64.9 vs k256/p256 62.0 | +2.9 (1.05×) |
| add_u64 | Tom-256 base | 83.9 vs crypto-bigint 512 | −428 (0.16×) | 39.7 vs halo2curves 217 | −177 (0.18×) | 99.6 vs halo2curves 702 | −602 (0.14×) | 65.2 vs halo2curves 293 | −228 (0.22×) |
| add_u64 | BLS12-381 base | 161 vs ark-ff 830 | −669 (0.19×) | 62.6 vs halo2curves 325 | −262 (0.19×) | 248 vs ark-ff 1,067 | −819 (0.23×) | 89.5 vs crypto-bigint 648 | −559 (0.14×) |
| add_u64 | P-521 base | 21.4 vs crypto-bigint 1,679 | −1,658 (0.01×) | 11.9 vs crypto-bigint 655 | −643 (0.02×) | 19.5 vs crypto-bigint 2,237 | −2,218 (0.01×) | 27.6 vs crypto-bigint 1,146 | −1,118 (0.02×) |
| add_uint | Mersenne-31 | 4.6 vs crypto-bigint 35.4 | −30.8 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.8 vs crypto-bigint 41.4 | −36.6 (0.12×) | 4.7 vs crypto-bigint 30.4 | −25.7 (0.15×) |
| add_uint | BabyBear | 12.2 vs plonky3 21.5 | −9.3 (0.57×) | 8.2 vs plonky3 9.3 | −1.1 (0.88×) | 12.7 vs plonky3 31.1 | −18.4 (0.41×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_uint | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.6 vs winter-math 19.2 | −7.6 (0.60×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_uint | BN254 scalar | 224 vs halo2curves 421 | −197 (0.53×) | 98.2 vs halo2curves 112 | −13.8 (0.88×) | 365 vs halo2curves 665 | −300 (0.55×) | 181 vs halo2curves 205 | −24.0 (0.88×) |
| add_uint | secp256k1 base | 17.3 vs halo2curves 225 | −208 (0.08×) | 12.5 vs halo2curves 85.2 | −72.7 (0.15×) | 16.5 vs halo2curves 432 | −416 (0.04×) | 23.1 vs halo2curves 141 | −118 (0.16×) |
| add_uint | P-256 base | 180 vs halo2curves 245 | −65.0 (0.73×) | 78.3 vs halo2curves 92.2 | −13.9 (0.85×) | 326 vs halo2curves 518 | −192 (0.63×) | 149 vs halo2curves 153 | −4.0 (0.97×) |
| add_uint | P-256 scalar | 259 vs k256/p256 80.5 | +179 (3.22×) | 106 vs k256/p256 83.1 | +22.9 (1.28×) | 310 vs k256/p256 108 | +202 (2.87×) | 190 vs k256/p256 108 | +82.0 (1.76×) |
| add_uint | Tom-256 base | 253 vs halo2curves 374 | −121 (0.68×) | 104 vs halo2curves 111 | −7.0 (0.94×) | 326 vs halo2curves 638 | −312 (0.51×) | 181 vs halo2curves 196 | −15.0 (0.92×) |
| add_uint | BLS12-381 base | 761 vs halo2curves 965 | −204 (0.79×) | 214 vs halo2curves 226 | −12.0 (0.95×) | 914 vs crypto-bigint 1,379 | −465 (0.66×) | 383 vs halo2curves 511 | −128 (0.75×) |
| add_uint | P-521 base | 61.5 vs crypto-bigint 1,696 | −1,635 (0.04×) | 41.5 vs crypto-bigint 664 | −623 (0.06×) | 86.4 vs crypto-bigint 2,228 | −2,142 (0.04×) | 66.7 vs crypto-bigint 1,154 | −1,087 (0.06×) |
| sub_u64 | Mersenne-31 | 4.6 vs crypto-bigint 22.1 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.6 | −25.7 (0.16×) | 4.9 vs crypto-bigint 19.9 | −15.0 (0.25×) |
| sub_u64 | BabyBear | 12.0 vs crypto-bigint 28.7 | −16.7 (0.42×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 12.0 vs plonky3 41.6 | −29.6 (0.29×) | 11.2 vs plonky3 13.3 | −2.1 (0.84×) |
| sub_u64 | Goldilocks | 8.1 vs winter-math 11.1 | −3.0 (0.73×) | 6.1 vs winter-math 7.0 | −0.9 (0.87×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 8.7 vs winter-math 8.7 | +0.0 (1.00×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 347 | −234 (0.33×) | 45.7 vs crypto-bigint 151 | −105 (0.30×) | 135 vs ark-ff 404 | −269 (0.33×) | 67.3 vs crypto-bigint 286 | −219 (0.24×) |
| sub_u64 | secp256k1 base | 7.0 vs ark-ff 156 | −149 (0.04×) | 6.1 vs ark-ff 120 | −114 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 9.0 vs ark-ff 183 | −174 (0.05×) |
| sub_u64 | P-256 base | 69.4 vs ark-ff 264 | −195 (0.26×) | 34.8 vs k256/p256 88.0 | −53.2 (0.40×) | 78.2 vs ark-ff 401 | −323 (0.20×) | 55.8 vs k256/p256 149 | −93.2 (0.37×) |
| sub_u64 | P-256 scalar | 66.8 vs k256/p256 21.2 | +45.6 (3.15×) | 37.6 vs k256/p256 21.2 | +16.4 (1.77×) | 71.6 vs k256/p256 24.7 | +46.9 (2.90×) | 64.8 vs k256/p256 25.0 | +39.8 (2.59×) |
| sub_u64 | Tom-256 base | 85.0 vs crypto-bigint 479 | −394 (0.18×) | 39.2 vs crypto-bigint 152 | −113 (0.26×) | 97.1 vs halo2curves 689 | −592 (0.14×) | 65.7 vs crypto-bigint 284 | −218 (0.23×) |
| sub_u64 | BLS12-381 base | 153 vs ark-ff 826 | −673 (0.19×) | 62.3 vs halo2curves 310 | −248 (0.20×) | 263 vs ark-ff 1,043 | −780 (0.25×) | 91.9 vs crypto-bigint 624 | −532 (0.15×) |
| sub_u64 | P-521 base | 21.1 vs crypto-bigint 1,666 | −1,645 (0.01×) | 15.0 vs crypto-bigint 628 | −613 (0.02×) | 22.2 vs crypto-bigint 2,199 | −2,177 (0.01×) | 22.0 vs crypto-bigint 1,113 | −1,091 (0.02×) |
| sub_uint | Mersenne-31 | 4.6 vs crypto-bigint 22.2 | −17.6 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.4 | −25.5 (0.16×) | 4.9 vs crypto-bigint 20.2 | −15.3 (0.24×) |
| sub_uint | BabyBear | 11.9 vs plonky3 22.0 | −10.1 (0.54×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 12.0 vs plonky3 31.3 | −19.3 (0.38×) | 11.2 vs plonky3 13.3 | −2.1 (0.84×) |
| sub_uint | Goldilocks | 8.1 vs winter-math 11.1 | −3.0 (0.73×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 8.7 vs winter-math 8.7 | +0.0 (1.00×) |
| sub_uint | BN254 scalar | 228 vs halo2curves 397 | −169 (0.57×) | 97.2 vs halo2curves 105 | −7.8 (0.93×) | 352 vs halo2curves 660 | −308 (0.53×) | 181 vs halo2curves 195 | −14.0 (0.93×) |
| sub_uint | secp256k1 base | 18.9 vs halo2curves 212 | −193 (0.09×) | 12.4 vs halo2curves 80.0 | −67.6 (0.15×) | 16.7 vs halo2curves 415 | −398 (0.04×) | 21.9 vs halo2curves 130 | −108 (0.17×) |
| sub_uint | P-256 base | 174 vs halo2curves 233 | −59.0 (0.75×) | 79.9 vs halo2curves 85.7 | −5.8 (0.93×) | 245 vs halo2curves 497 | −252 (0.49×) | 136 vs halo2curves 143 | −7.0 (0.95×) |
| sub_uint | P-256 scalar | 253 vs k256/p256 71.9 | +181 (3.52×) | 103 vs k256/p256 72.2 | +30.8 (1.43×) | 330 vs k256/p256 95.0 | +235 (3.47×) | 186 vs k256/p256 95.3 | +90.7 (1.95×) |
| sub_uint | Tom-256 base | 253 vs halo2curves 365 | −112 (0.69×) | 98.3 vs halo2curves 103 | −4.7 (0.95×) | 329 vs halo2curves 619 | −290 (0.53×) | 176 vs halo2curves 188 | −12.0 (0.94×) |
| sub_uint | BLS12-381 base | 756 vs halo2curves 968 | −212 (0.78×) | 203 vs halo2curves 215 | −12.0 (0.94×) | 923 vs crypto-bigint 1,382 | −459 (0.67×) | 374 vs halo2curves 411 | −37.0 (0.91×) |
| sub_uint | P-521 base | 66.1 vs crypto-bigint 1,667 | −1,601 (0.04×) | 38.9 vs crypto-bigint 640 | −601 (0.06×) | 76.2 vs crypto-bigint 2,205 | −2,129 (0.03×) | 67.2 vs crypto-bigint 1,150 | −1,083 (0.06×) |
| div_u64 | BN254 scalar | 1,209 vs halo2curves 13,494 | −12,285 (0.09×) | 842 vs halo2curves 6,982 | −6,140 (0.12×) | 1,372 vs halo2curves 19,386 | −18,014 (0.07×) | 1,105 vs halo2curves 11,913 | −10,808 (0.09×) |
| div_u64 | secp256k1 base | 1,015 vs halo2curves 13,078 | −12,063 (0.08×) | 722 vs halo2curves 6,905 | −6,183 (0.10×) | 1,206 vs halo2curves 18,965 | −17,759 (0.06×) | 810 vs halo2curves 11,613 | −10,803 (0.07×) |
| div_u64 | P-256 base | 1,083 vs halo2curves 13,047 | −11,964 (0.08×) | 801 vs halo2curves 6,860 | −6,059 (0.12×) | 1,338 vs halo2curves 18,924 | −17,586 (0.07×) | 987 vs halo2curves 11,645 | −10,658 (0.08×) |
| div_u64 | P-256 scalar | 1,181 vs halo2curves 13,395 | −12,214 (0.09×) | 856 vs halo2curves 6,951 | −6,095 (0.12×) | 1,436 vs halo2curves 19,466 | −18,030 (0.07×) | 1,130 vs halo2curves 11,732 | −10,602 (0.10×) |
| div_u64 | Tom-256 base | 1,201 vs halo2curves 13,342 | −12,141 (0.09×) | 858 vs halo2curves 6,964 | −6,106 (0.12×) | 1,465 vs halo2curves 19,239 | −17,774 (0.08×) | 1,113 vs halo2curves 11,718 | −10,605 (0.09×) |
| div_u64 | BLS12-381 base | 1,790 vs halo2curves 23,980 | −22,190 (0.07×) | 1,086 vs halo2curves 11,065 | −9,979 (0.10×) | 2,460 vs halo2curves 32,861 | −30,401 (0.07×) | 1,756 vs halo2curves 18,749 | −16,993 (0.09×) |
| div_u64 | P-521 base | 2,442 vs crypto-bigint 62,201 | −59,759 (0.04×) | 1,014 vs crypto-bigint 45,300 | −44,286 (0.02×) | 3,618 vs crypto-bigint 78,308 | −74,690 (0.05×) | 1,524 vs crypto-bigint 57,892 | −56,368 (0.03×) |
| div_uint | Mersenne-31 | 274 vs crypto-bigint 1,820 | −1,546 (0.15×) | 274 vs crypto-bigint 1,601 | −1,327 (0.17×) | 278 vs runtime-algebra 2,098 | −1,820 (0.13×) | 281 vs crypto-bigint 1,966 | −1,685 (0.14×) |
| div_uint | BabyBear | 275 vs plonky3 675 | −400 (0.41×) | 277 vs plonky3 651 | −374 (0.43×) | 277 vs plonky3 720 | −443 (0.38×) | 281 vs plonky3 722 | −441 (0.39×) |
| div_uint | Goldilocks | 549 vs winter-math 1,791 | −1,242 (0.31×) | 539 vs winter-math 927 | −388 (0.58×) | 585 vs winter-math 1,881 | −1,296 (0.31×) | 569 vs winter-math 1,030 | −461 (0.55×) |
| div_uint | BN254 scalar | 9,078 vs halo2curves 15,111 | −6,033 (0.60×) | 6,600 vs halo2curves 8,173 | −1,573 (0.81×) | 12,765 vs halo2curves 22,876 | −10,111 (0.56×) | 8,344 vs halo2curves 14,406 | −6,062 (0.58×) |
| div_uint | secp256k1 base | 8,569 vs halo2curves 14,887 | −6,318 (0.58×) | 6,001 vs halo2curves 8,205 | −2,204 (0.73×) | 12,029 vs halo2curves 22,605 | −10,576 (0.53×) | 8,123 vs halo2curves 14,447 | −6,324 (0.56×) |
| div_uint | P-256 base | 8,760 vs halo2curves 14,844 | −6,084 (0.59×) | 6,175 vs halo2curves 8,214 | −2,039 (0.75×) | 11,942 vs halo2curves 22,580 | −10,638 (0.53×) | 7,931 vs halo2curves 14,457 | −6,526 (0.55×) |
| div_uint | P-256 scalar | 9,129 vs halo2curves 15,047 | −5,918 (0.61×) | 6,596 vs halo2curves 8,253 | −1,657 (0.80×) | 12,456 vs halo2curves 23,049 | −10,593 (0.54×) | 8,622 vs halo2curves 14,537 | −5,915 (0.59×) |
| div_uint | Tom-256 base | 8,877 vs halo2curves 15,050 | −6,173 (0.59×) | 6,449 vs halo2curves 8,178 | −1,729 (0.79×) | 12,494 vs halo2curves 22,871 | −10,377 (0.55×) | 8,415 vs halo2curves 14,615 | −6,200 (0.58×) |
| div_uint | BLS12-381 base | 20,855 vs halo2curves 27,620 | −6,765 (0.76×) | 12,241 vs halo2curves 13,641 | −1,400 (0.90×) | 26,355 vs halo2curves 40,145 | −13,790 (0.66×) | 17,092 vs halo2curves 24,373 | −7,281 (0.70×) |
| div_uint | P-521 base | 34,495 vs crypto-bigint 68,456 | −33,961 (0.50×) | 18,119 vs crypto-bigint 51,170 | −33,051 (0.35×) | 49,309 vs crypto-bigint 84,519 | −35,210 (0.58×) | 30,112 vs crypto-bigint 63,735 | −33,623 (0.47×) |
| add, constant time | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.9 | −14.9 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) |
| add, constant time | BN254 scalar | 25.1 vs halo2curves 41.9 | −16.8 (0.60×) | 19.5 vs halo2curves 23.4 | −3.9 (0.83×) | 23.1 vs halo2curves 45.0 | −21.9 (0.51×) | 25.8 vs halo2curves 34.0 | −8.2 (0.76×) |
| add, constant time | secp256k1 base | 28.3 vs halo2curves 39.8 | −11.5 (0.71×) | 22.6 vs halo2curves 23.0 | −0.4 (0.98×) | 27.0 vs halo2curves 37.7 | −10.7 (0.72×) | 29.8 vs halo2curves 31.0 | −1.2 (0.96×) |
| add, constant time | P-256 base | 24.6 vs halo2curves 38.9 | −14.3 (0.63×) | 22.8 vs halo2curves 21.0 | +1.8 (1.09×) | 25.0 vs k256/p256 39.5 | −14.5 (0.63×) | 27.7 vs k256/p256 29.5 | −1.8 (0.94×) |
| add, constant time | P-256 scalar | 27.7 vs halo2curves 42.2 | −14.5 (0.66×) | 23.8 vs halo2curves 24.4 | −0.6 (0.98×) | 26.8 vs halo2curves 42.5 | −15.7 (0.63×) | 30.1 vs halo2curves 33.0 | −2.9 (0.91×) |
| add, constant time | Tom-256 base | 27.2 vs halo2curves 42.9 | −15.7 (0.63×) | 23.8 vs halo2curves 24.0 | −0.2 (0.99×) | 26.9 vs halo2curves 42.5 | −15.6 (0.63×) | 29.1 vs halo2curves 33.0 | −3.9 (0.88×) |
| add, constant time | BLS12-381 base | 41.4 vs halo2curves 63.4 | −22.0 (0.65×) | 28.6 vs halo2curves 34.1 | −5.5 (0.84×) | 35.3 vs halo2curves 67.1 | −31.8 (0.53×) | 41.1 vs halo2curves 54.1 | −13.0 (0.76×) |
| add, constant time | P-521 base | 76.4 vs crypto-bigint 181 | −105 (0.42×) | 47.2 vs crypto-bigint 179 | −132 (0.26×) | 81.4 vs crypto-bigint 246 | −165 (0.33×) | 74.1 vs crypto-bigint 245 | −171 (0.30×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.5 vs halo2curves 26.0 | −4.5 (0.83×) | 16.5 vs halo2curves 14.9 | +1.6 (1.11×) | 20.8 vs halo2curves 25.5 | −4.7 (0.82×) | 22.0 vs halo2curves 22.0 | +0.0 (1.00×) |
| sub, constant time | secp256k1 base | 21.4 vs halo2curves 25.2 | −3.8 (0.85×) | 15.5 vs halo2curves 14.5 | +1.0 (1.07×) | 19.6 vs halo2curves 24.0 | −4.4 (0.82×) | 22.0 vs halo2curves 21.0 | +1.0 (1.05×) |
| sub, constant time | P-256 base | 19.1 vs halo2curves 25.1 | −6.0 (0.76×) | 15.5 vs halo2curves 14.0 | +1.5 (1.11×) | 18.5 vs halo2curves 24.0 | −5.5 (0.77×) | 20.0 vs k256/p256 20.0 | +0.0 (1.00×) |
| sub, constant time | P-256 scalar | 21.6 vs k256/p256 24.3 | −2.7 (0.89×) | 15.9 vs halo2curves 15.3 | +0.6 (1.04×) | 18.5 vs k256/p256 27.0 | −8.5 (0.69×) | 21.5 vs halo2curves 21.5 | +0.0 (1.00×) |
| sub, constant time | Tom-256 base | 21.4 vs halo2curves 26.1 | −4.7 (0.82×) | 17.1 vs halo2curves 15.0 | +2.1 (1.14×) | 19.8 vs halo2curves 26.5 | −6.7 (0.75×) | 23.0 vs halo2curves 22.0 | +1.0 (1.05×) |
| sub, constant time | BLS12-381 base | 37.5 vs halo2curves 45.3 | −7.8 (0.83×) | 24.1 vs halo2curves 25.0 | −0.9 (0.96×) | 36.9 vs halo2curves 44.1 | −7.2 (0.84×) | 36.1 vs halo2curves 36.2 | −0.1 (1.00×) |
| sub, constant time | P-521 base | 57.4 vs crypto-bigint 159 | −102 (0.36×) | 37.2 vs crypto-bigint 144 | −107 (0.26×) | 59.8 vs crypto-bigint 174 | −114 (0.34×) | 53.1 vs crypto-bigint 217 | −164 (0.24×) |
| mul, constant time | Mersenne-31 | 7.0 vs crypto-bigint 46.5 | −39.5 (0.15×) | 7.0 vs crypto-bigint 27.0 | −20.0 (0.26×) | 7.0 vs crypto-bigint 49.0 | −42.0 (0.14×) | 7.0 vs crypto-bigint 30.3 | −23.3 (0.23×) |
| mul, constant time | BabyBear | 12.0 vs crypto-bigint 46.8 | −34.8 (0.26×) | 11.0 vs crypto-bigint 27.0 | −16.0 (0.41×) | 12.0 vs crypto-bigint 50.1 | −38.1 (0.24×) | 11.0 vs crypto-bigint 29.3 | −18.3 (0.38×) |
| mul, constant time | Goldilocks | 16.5 vs winter-math 18.0 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 12.3 vs winter-math 14.0 | −1.7 (0.88×) |
| mul, constant time | BN254 scalar | 244 vs halo2curves 381 | −137 (0.64×) | 93.0 vs halo2curves 91.4 | +1.6 (1.02×) | 413 vs halo2curves 696 | −283 (0.59×) | 153 vs halo2curves 178 | −25.0 (0.86×) |
| mul, constant time | secp256k1 base | 172 vs halo2curves 307 | −135 (0.56×) | 77.5 vs halo2curves 90.5 | −13.0 (0.86×) | 242 vs halo2curves 587 | −345 (0.41×) | 105 vs halo2curves 159 | −54.0 (0.66×) |
| mul, constant time | P-256 base | 189 vs k256/p256 243 | −54.0 (0.78×) | 72.4 vs halo2curves 74.5 | −2.1 (0.97×) | 292 vs k256/p256 476 | −184 (0.61×) | 120 vs halo2curves 130 | −10.0 (0.92×) |
| mul, constant time | P-256 scalar | 234 vs halo2curves 350 | −116 (0.67×) | 93.2 vs halo2curves 95.3 | −2.1 (0.98×) | 326 vs halo2curves 674 | −348 (0.48×) | 167 vs halo2curves 176 | −9.0 (0.95×) |
| mul, constant time | Tom-256 base | 237 vs halo2curves 344 | −107 (0.69×) | 86.7 vs halo2curves 93.5 | −6.8 (0.93×) | 332 vs halo2curves 602 | −270 (0.55×) | 158 vs halo2curves 172 | −14.0 (0.92×) |
| mul, constant time | BLS12-381 base | 718 vs halo2curves 907 | −189 (0.79×) | 188 vs halo2curves 194 | −6.0 (0.97×) | 916 vs crypto-bigint 1,326 | −410 (0.69×) | 337 vs halo2curves 377 | −40.0 (0.89×) |
| mul, constant time | P-521 base | 687 vs crypto-bigint 2,131 | −1,444 (0.32×) | 247 vs crypto-bigint 618 | −371 (0.40×) | 1,316 vs crypto-bigint 2,795 | −1,479 (0.47×) | 445 vs crypto-bigint 1,117 | −672 (0.40×) |
| square, constant time | Mersenne-31 | 7.0 vs crypto-bigint 45.4 | −38.4 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.5 | −43.5 (0.14×) | 7.0 vs crypto-bigint 29.9 | −22.9 (0.23×) |
| square, constant time | BabyBear | 12.0 vs crypto-bigint 45.5 | −33.5 (0.26×) | 11.0 vs crypto-bigint 27.1 | −16.1 (0.41×) | 12.0 vs crypto-bigint 52.0 | −40.0 (0.23×) | 11.0 vs crypto-bigint 30.6 | −19.6 (0.36×) |
| square, constant time | Goldilocks | 15.0 vs winter-math 19.6 | −4.6 (0.77×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.5 vs winter-math 20.0 | −1.5 (0.93×) | 12.7 vs winter-math 14.3 | −1.6 (0.89×) |
| square, constant time | BN254 scalar | 219 vs halo2curves 304 | −85.0 (0.72×) | 85.2 vs halo2curves 96.6 | −11.4 (0.88×) | 199 vs halo2curves 551 | −352 (0.36×) | 140 vs halo2curves 142 | −2.0 (0.99×) |
| square, constant time | secp256k1 base | 151 vs halo2curves 222 | −71.0 (0.68×) | 67.1 vs halo2curves 84.8 | −17.7 (0.79×) | 181 vs halo2curves 449 | −268 (0.40×) | 89.4 vs halo2curves 119 | −29.6 (0.75×) |
| square, constant time | P-256 base | 160 vs halo2curves 176 | −16.0 (0.91×) | 57.1 vs halo2curves 61.0 | −3.9 (0.94×) | 234 vs halo2curves 340 | −106 (0.69×) | 102 vs halo2curves 108 | −6.0 (0.94×) |
| square, constant time | P-256 scalar | 217 vs halo2curves 266 | −49.0 (0.82×) | 88.2 vs halo2curves 90.1 | −1.9 (0.98×) | 249 vs halo2curves 552 | −303 (0.45×) | 132 vs halo2curves 140 | −8.0 (0.94×) |
| square, constant time | Tom-256 base | 213 vs halo2curves 270 | −57.0 (0.79×) | 82.6 vs halo2curves 88.6 | −6.0 (0.93×) | 218 vs halo2curves 471 | −253 (0.46×) | 136 vs halo2curves 136 | +0.0 (1.00×) |
| square, constant time | BLS12-381 base | 563 vs halo2curves 916 | −353 (0.61×) | 168 vs halo2curves 168 | +0.0 (1.00×) | 789 vs halo2curves 1,238 | −449 (0.64×) | 295 vs halo2curves 296 | −1.0 (1.00×) |
| square, constant time | P-521 base | 571 vs crypto-bigint 2,159 | −1,588 (0.26×) | 221 vs crypto-bigint 633 | −412 (0.35×) | 909 vs crypto-bigint 2,854 | −1,945 (0.32×) | 392 vs crypto-bigint 1,152 | −760 (0.34×) |
| mul_u64, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 31.6 | −22.6 (0.28×) | 9.0 vs crypto-bigint 70.4 | −61.4 (0.13×) | 9.0 vs crypto-bigint 38.8 | −29.8 (0.23×) |
| mul_u64, constant time | BabyBear | 16.1 vs crypto-bigint 65.3 | −49.2 (0.25×) | 11.0 vs crypto-bigint 32.6 | −21.6 (0.34×) | 15.6 vs crypto-bigint 89.7 | −74.1 (0.17×) | 13.1 vs crypto-bigint 40.7 | −27.6 (0.32×) |
| mul_u64, constant time | Goldilocks | 17.8 vs winter-math 23.3 | −5.5 (0.76×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.3 vs winter-math 34.5 | −9.2 (0.73×) | 15.2 vs winter-math 15.5 | −0.3 (0.98×) |
| mul_u64, constant time | BN254 scalar | 109 vs crypto-bigint 840 | −731 (0.13×) | 50.9 vs crypto-bigint 261 | −210 (0.20×) | 132 vs halo2curves 1,418 | −1,286 (0.09×) | 58.0 vs crypto-bigint 410 | −352 (0.14×) |
| mul_u64, constant time | secp256k1 base | 68.9 vs halo2curves 691 | −622 (0.10×) | 39.0 vs crypto-bigint 259 | −220 (0.15×) | 85.1 vs halo2curves 1,066 | −981 (0.08×) | 45.5 vs halo2curves 386 | −341 (0.12×) |
| mul_u64, constant time | P-256 base | 89.4 vs k256/p256 486 | −397 (0.18×) | 53.2 vs k256/p256 151 | −97.8 (0.35×) | 103 vs k256/p256 947 | −844 (0.11×) | 58.7 vs k256/p256 258 | −199 (0.23×) |
| mul_u64, constant time | P-256 scalar | 105 vs k256/p256 516 | −411 (0.20×) | 55.0 vs k256/p256 255 | −200 (0.22×) | 121 vs k256/p256 831 | −710 (0.15×) | 68.5 vs k256/p256 338 | −270 (0.20×) |
| mul_u64, constant time | Tom-256 base | 100 vs crypto-bigint 844 | −744 (0.12×) | 52.0 vs crypto-bigint 258 | −206 (0.20×) | 121 vs halo2curves 1,260 | −1,139 (0.10×) | 60.8 vs crypto-bigint 410 | −349 (0.15×) |
| mul_u64, constant time | BLS12-381 base | 155 vs crypto-bigint 1,932 | −1,777 (0.08×) | 70.6 vs halo2curves 490 | −419 (0.14×) | 184 vs crypto-bigint 2,547 | −2,363 (0.07×) | 89.4 vs crypto-bigint 1,012 | −923 (0.09×) |
| mul_u64, constant time | P-521 base | 139 vs crypto-bigint 3,059 | −2,920 (0.05×) | 78.6 vs crypto-bigint 1,000 | −921 (0.08×) | 170 vs crypto-bigint 3,999 | −3,829 (0.04×) | 95.1 vs crypto-bigint 1,846 | −1,751 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.6 | −50.6 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 73.8 | −64.8 (0.12×) | 9.0 vs crypto-bigint 38.2 | −29.2 (0.24×) |
| mul_uint, constant time | BabyBear | 16.1 vs crypto-bigint 65.1 | −49.0 (0.25×) | 11.0 vs crypto-bigint 32.3 | −21.3 (0.34×) | 15.8 vs crypto-bigint 91.2 | −75.4 (0.17×) | 13.0 vs crypto-bigint 40.8 | −27.8 (0.32×) |
| mul_uint, constant time | Goldilocks | 17.6 vs winter-math 23.6 | −6.0 (0.75×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.3 vs winter-math 34.5 | −9.2 (0.73×) | 15.2 vs winter-math 14.3 | +0.9 (1.06×) |
| mul_uint, constant time | BN254 scalar | 484 vs halo2curves 749 | −265 (0.65×) | 168 vs halo2curves 181 | −13.0 (0.93×) | 597 vs halo2curves 1,313 | −716 (0.45×) | 283 vs crypto-bigint 406 | −123 (0.70×) |
| mul_uint, constant time | secp256k1 base | 318 vs halo2curves 517 | −199 (0.62×) | 76.7 vs halo2curves 154 | −77.3 (0.50×) | 376 vs halo2curves 968 | −592 (0.39×) | 122 vs halo2curves 264 | −142 (0.46×) |
| mul_uint, constant time | P-256 base | 407 vs halo2curves 454 | −47.0 (0.90×) | 133 vs halo2curves 146 | −13.0 (0.91×) | 487 vs halo2curves 953 | −466 (0.51×) | 220 vs halo2curves 256 | −36.0 (0.86×) |
| mul_uint, constant time | P-256 scalar | 476 vs k256/p256 551 | −75.0 (0.86×) | 177 vs halo2curves 185 | −8.0 (0.96×) | 544 vs k256/p256 880 | −336 (0.62×) | 292 vs halo2curves 351 | −59.0 (0.83×) |
| mul_uint, constant time | Tom-256 base | 467 vs halo2curves 696 | −229 (0.67×) | 172 vs halo2curves 181 | −9.0 (0.95×) | 553 vs halo2curves 1,177 | −624 (0.47×) | 320 vs halo2curves 335 | −15.0 (0.96×) |
| mul_uint, constant time | BLS12-381 base | 1,075 vs halo2curves 1,905 | −830 (0.56×) | 360 vs halo2curves 377 | −17.0 (0.95×) | 1,689 vs crypto-bigint 2,499 | −810 (0.68×) | 900 vs crypto-bigint 1,039 | −139 (0.87×) |
| mul_uint, constant time | P-521 base | 2,018 vs crypto-bigint 3,051 | −1,033 (0.66×) | 303 vs crypto-bigint 1,009 | −706 (0.30×) | 2,913 vs crypto-bigint 4,014 | −1,101 (0.73×) | 530 vs crypto-bigint 1,858 | −1,328 (0.29×) |
| add_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.7 vs crypto-bigint 43.3 | −38.6 (0.11×) | 4.7 vs crypto-bigint 30.5 | −25.8 (0.15×) |
| add_u64, constant time | BabyBear | 11.7 vs crypto-bigint 47.4 | −35.7 (0.25×) | 8.5 vs crypto-bigint 30.0 | −21.5 (0.28×) | 13.0 vs crypto-bigint 52.4 | −39.4 (0.25×) | 10.8 vs crypto-bigint 33.0 | −22.2 (0.33×) |
| add_u64, constant time | Goldilocks | 8.5 vs winter-math 13.2 | −4.7 (0.64×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_u64, constant time | BN254 scalar | 93.2 vs crypto-bigint 512 | −419 (0.18×) | 44.6 vs halo2curves 210 | −165 (0.21×) | 108 vs halo2curves 765 | −657 (0.14×) | 69.6 vs halo2curves 299 | −229 (0.23×) |
| add_u64, constant time | secp256k1 base | 25.4 vs halo2curves 399 | −374 (0.06×) | 23.0 vs halo2curves 208 | −185 (0.11×) | 25.0 vs halo2curves 483 | −458 (0.05×) | 29.0 vs halo2curves 244 | −215 (0.12×) |
| add_u64, constant time | P-256 base | 66.4 vs k256/p256 277 | −211 (0.24×) | 42.6 vs k256/p256 92.5 | −49.9 (0.46×) | 80.1 vs k256/p256 507 | −427 (0.16×) | 60.7 vs k256/p256 158 | −97.3 (0.38×) |
| add_u64, constant time | P-256 scalar | 103 vs k256/p256 56.2 | +46.8 (1.83×) | 49.4 vs k256/p256 56.0 | −6.6 (0.88×) | 109 vs k256/p256 62.4 | +46.6 (1.75×) | 75.1 vs k256/p256 62.0 | +13.1 (1.21×) |
| add_u64, constant time | Tom-256 base | 97.6 vs crypto-bigint 512 | −414 (0.19×) | 47.2 vs halo2curves 217 | −170 (0.22×) | 109 vs halo2curves 702 | −593 (0.16×) | 70.6 vs halo2curves 293 | −222 (0.24×) |
| add_u64, constant time | BLS12-381 base | 136 vs crypto-bigint 1,082 | −946 (0.13×) | 69.2 vs halo2curves 325 | −256 (0.21×) | 163 vs crypto-bigint 1,389 | −1,226 (0.12×) | 110 vs crypto-bigint 648 | −538 (0.17×) |
| add_u64, constant time | P-521 base | 65.9 vs crypto-bigint 1,679 | −1,613 (0.04×) | 45.0 vs crypto-bigint 655 | −610 (0.07×) | 69.6 vs crypto-bigint 2,237 | −2,167 (0.03×) | 64.8 vs crypto-bigint 1,146 | −1,081 (0.06×) |
| add_uint, constant time | Mersenne-31 | 4.6 vs crypto-bigint 35.4 | −30.8 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.7 vs crypto-bigint 41.4 | −36.7 (0.11×) | 4.7 vs crypto-bigint 30.4 | −25.7 (0.15×) |
| add_uint, constant time | BabyBear | 11.6 vs crypto-bigint 47.6 | −36.0 (0.24×) | 8.1 vs crypto-bigint 30.0 | −21.9 (0.27×) | 13.0 vs crypto-bigint 52.4 | −39.4 (0.25×) | 10.8 vs crypto-bigint 32.9 | −22.1 (0.33×) |
| add_uint, constant time | Goldilocks | 8.5 vs winter-math 13.1 | −4.6 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.8 vs winter-math 19.2 | −7.4 (0.61×) | 9.3 vs winter-math 9.6 | −0.3 (0.97×) |
| add_uint, constant time | BN254 scalar | 238 vs halo2curves 421 | −183 (0.57×) | 97.8 vs halo2curves 112 | −14.2 (0.87×) | 367 vs halo2curves 665 | −298 (0.55×) | 181 vs halo2curves 205 | −24.0 (0.88×) |
| add_uint, constant time | secp256k1 base | 38.5 vs halo2curves 225 | −187 (0.17×) | 29.3 vs halo2curves 85.2 | −55.9 (0.34×) | 36.3 vs halo2curves 432 | −396 (0.08×) | 44.5 vs halo2curves 141 | −96.5 (0.32×) |
| add_uint, constant time | P-256 base | 179 vs halo2curves 245 | −66.0 (0.73×) | 83.7 vs halo2curves 92.2 | −8.5 (0.91×) | 276 vs halo2curves 518 | −242 (0.53×) | 144 vs halo2curves 153 | −9.0 (0.94×) |
| add_uint, constant time | P-256 scalar | 261 vs k256/p256 80.5 | +181 (3.24×) | 106 vs k256/p256 83.1 | +22.9 (1.28×) | 317 vs k256/p256 108 | +209 (2.94×) | 178 vs k256/p256 108 | +70.0 (1.65×) |
| add_uint, constant time | Tom-256 base | 253 vs halo2curves 374 | −121 (0.68×) | 103 vs halo2curves 111 | −8.0 (0.93×) | 322 vs halo2curves 638 | −316 (0.50×) | 177 vs halo2curves 196 | −19.0 (0.90×) |
| add_uint, constant time | BLS12-381 base | 762 vs halo2curves 965 | −203 (0.79×) | 206 vs halo2curves 226 | −20.0 (0.91×) | 909 vs crypto-bigint 1,379 | −470 (0.66×) | 384 vs halo2curves 511 | −127 (0.75×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,696 | −1,558 (0.08×) | 93.0 vs crypto-bigint 664 | −571 (0.14×) | 142 vs crypto-bigint 2,228 | −2,086 (0.06×) | 134 vs crypto-bigint 1,154 | −1,020 (0.12×) |
| sub_u64, constant time | Mersenne-31 | 4.6 vs crypto-bigint 22.1 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.6 | −25.7 (0.16×) | 4.9 vs crypto-bigint 19.9 | −15.0 (0.25×) |
| sub_u64, constant time | BabyBear | 12.1 vs crypto-bigint 28.7 | −16.6 (0.42×) | 8.2 vs crypto-bigint 19.5 | −11.3 (0.42×) | 13.0 vs crypto-bigint 42.3 | −29.3 (0.31×) | 11.6 vs crypto-bigint 21.5 | −9.9 (0.54×) |
| sub_u64, constant time | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 7.0 | −0.9 (0.87×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.7 vs winter-math 8.7 | +0.0 (1.00×) |
| sub_u64, constant time | BN254 scalar | 95.1 vs crypto-bigint 483 | −388 (0.20×) | 41.5 vs crypto-bigint 151 | −110 (0.27×) | 107 vs halo2curves 755 | −648 (0.14×) | 65.7 vs crypto-bigint 286 | −220 (0.23×) |
| sub_u64, constant time | secp256k1 base | 18.0 vs halo2curves 384 | −366 (0.05×) | 15.2 vs crypto-bigint 152 | −137 (0.10×) | 17.0 vs halo2curves 468 | −451 (0.04×) | 17.4 vs halo2curves 238 | −221 (0.07×) |
| sub_u64, constant time | P-256 base | 61.4 vs k256/p256 265 | −204 (0.23×) | 36.0 vs k256/p256 88.0 | −52.0 (0.41×) | 74.6 vs k256/p256 498 | −423 (0.15×) | 54.9 vs k256/p256 149 | −94.1 (0.37×) |
| sub_u64, constant time | P-256 scalar | 96.1 vs k256/p256 21.2 | +74.9 (4.53×) | 45.1 vs k256/p256 21.2 | +23.9 (2.13×) | 107 vs k256/p256 24.7 | +82.3 (4.33×) | 70.7 vs k256/p256 25.0 | +45.7 (2.83×) |
| sub_u64, constant time | Tom-256 base | 92.0 vs crypto-bigint 479 | −387 (0.19×) | 43.1 vs crypto-bigint 152 | −109 (0.28×) | 105 vs halo2curves 689 | −584 (0.15×) | 69.3 vs crypto-bigint 284 | −215 (0.24×) |
| sub_u64, constant time | BLS12-381 base | 138 vs crypto-bigint 1,086 | −948 (0.13×) | 66.6 vs halo2curves 310 | −243 (0.21×) | 172 vs crypto-bigint 1,380 | −1,208 (0.12×) | 108 vs crypto-bigint 624 | −516 (0.17×) |
| sub_u64, constant time | P-521 base | 45.4 vs crypto-bigint 1,666 | −1,621 (0.03×) | 35.0 vs crypto-bigint 628 | −593 (0.06×) | 49.8 vs crypto-bigint 2,199 | −2,149 (0.02×) | 46.3 vs crypto-bigint 1,113 | −1,067 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 4.7 vs crypto-bigint 22.2 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.4 | −25.5 (0.16×) | 4.9 vs crypto-bigint 20.2 | −15.3 (0.24×) |
| sub_uint, constant time | BabyBear | 11.9 vs crypto-bigint 28.6 | −16.7 (0.42×) | 8.2 vs crypto-bigint 19.3 | −11.1 (0.42×) | 13.0 vs crypto-bigint 42.3 | −29.3 (0.31×) | 11.6 vs crypto-bigint 21.5 | −9.9 (0.54×) |
| sub_uint, constant time | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.7 vs winter-math 8.7 | +0.0 (1.00×) |
| sub_uint, constant time | BN254 scalar | 241 vs halo2curves 397 | −156 (0.61×) | 97.3 vs halo2curves 105 | −7.7 (0.93×) | 287 vs halo2curves 660 | −373 (0.43×) | 176 vs halo2curves 195 | −19.0 (0.90×) |
| sub_uint, constant time | secp256k1 base | 32.3 vs halo2curves 212 | −180 (0.15×) | 24.9 vs halo2curves 80.0 | −55.1 (0.31×) | 32.4 vs halo2curves 415 | −383 (0.08×) | 37.5 vs halo2curves 130 | −92.5 (0.29×) |
| sub_uint, constant time | P-256 base | 176 vs halo2curves 233 | −57.0 (0.76×) | 78.3 vs halo2curves 85.7 | −7.4 (0.91×) | 274 vs halo2curves 497 | −223 (0.55×) | 135 vs halo2curves 143 | −8.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 260 vs k256/p256 71.9 | +188 (3.62×) | 102 vs k256/p256 72.2 | +29.8 (1.41×) | 330 vs k256/p256 95.0 | +235 (3.47×) | 177 vs k256/p256 95.3 | +81.7 (1.86×) |
| sub_uint, constant time | Tom-256 base | 248 vs halo2curves 365 | −117 (0.68×) | 97.9 vs halo2curves 103 | −5.1 (0.95×) | 329 vs halo2curves 619 | −290 (0.53×) | 174 vs halo2curves 188 | −14.0 (0.93×) |
| sub_uint, constant time | BLS12-381 base | 760 vs halo2curves 968 | −208 (0.79×) | 203 vs halo2curves 215 | −12.0 (0.94×) | 907 vs crypto-bigint 1,382 | −475 (0.66×) | 381 vs halo2curves 411 | −30.0 (0.93×) |
| sub_uint, constant time | P-521 base | 123 vs crypto-bigint 1,667 | −1,544 (0.07×) | 85.2 vs crypto-bigint 640 | −555 (0.13×) | 122 vs crypto-bigint 2,205 | −2,083 (0.06×) | 113 vs crypto-bigint 1,150 | −1,037 (0.10×) |
| div_u64, constant time | BN254 scalar | 12,396 vs crypto-bigint 30,139 | −17,743 (0.41×) | 9,401 vs crypto-bigint 25,494 | −16,093 (0.37×) | 17,470 vs crypto-bigint 37,661 | −20,191 (0.46×) | 13,319 vs crypto-bigint 31,306 | −17,987 (0.43×) |
| div_u64, constant time | secp256k1 base | 11,997 vs crypto-bigint 30,128 | −18,131 (0.40×) | 9,060 vs crypto-bigint 25,376 | −16,316 (0.36×) | 16,768 vs crypto-bigint 37,764 | −20,996 (0.44×) | 12,861 vs crypto-bigint 31,448 | −18,587 (0.41×) |
| div_u64, constant time | P-256 base | 11,860 vs crypto-bigint 30,101 | −18,241 (0.39×) | 9,125 vs k256/p256 22,283 | −13,158 (0.41×) | 16,959 vs crypto-bigint 37,553 | −20,594 (0.45×) | 12,446 vs crypto-bigint 31,447 | −19,001 (0.40×) |
| div_u64, constant time | P-256 scalar | 12,343 vs crypto-bigint 30,058 | −17,715 (0.41×) | 9,428 vs crypto-bigint 25,464 | −16,036 (0.37×) | 17,090 vs crypto-bigint 37,723 | −20,633 (0.45×) | 13,076 vs crypto-bigint 31,240 | −18,164 (0.42×) |
| div_u64, constant time | Tom-256 base | 12,244 vs crypto-bigint 30,182 | −17,938 (0.41×) | 9,283 vs crypto-bigint 25,486 | −16,203 (0.36×) | 16,830 vs crypto-bigint 37,696 | −20,866 (0.45×) | 12,824 vs crypto-bigint 31,123 | −18,299 (0.41×) |
| div_u64, constant time | BLS12-381 base | 24,766 vs crypto-bigint 51,177 | −26,411 (0.48×) | 16,683 vs crypto-bigint 43,712 | −27,029 (0.38×) | 34,547 vs crypto-bigint 68,843 | −34,296 (0.50×) | 24,288 vs crypto-bigint 55,094 | −30,806 (0.44×) |
| div_u64, constant time | P-521 base | 43,059 vs crypto-bigint 106,620 | −63,561 (0.40×) | 26,100 vs crypto-bigint 84,346 | −58,246 (0.31×) | 64,242 vs crypto-bigint 126,530 | −62,288 (0.51×) | 42,893 vs crypto-bigint 101,079 | −58,186 (0.42×) |
| div_uint, constant time | Mersenne-31 | 275 vs runtime-algebra 3,535 | −3,260 (0.08×) | 275 vs runtime-algebra 3,537 | −3,262 (0.08×) | 274 vs runtime-algebra 3,528 | −3,254 (0.08×) | 274 vs runtime-algebra 3,416 | −3,142 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,558 | −3,089 (0.13×) | 435 vs runtime-algebra 3,563 | −3,128 (0.12×) | 470 vs runtime-algebra 3,527 | −3,057 (0.13×) | 441 vs runtime-algebra 3,416 | −2,975 (0.13×) |
| div_uint, constant time | Goldilocks | 1,138 vs winter-math 1,791 | −653 (0.64×) | 859 vs winter-math 927 | −68.0 (0.93×) | 1,431 vs winter-math 1,881 | −450 (0.76×) | 953 vs winter-math 1,030 | −77.0 (0.93×) |
| div_uint, constant time | BN254 scalar | 12,409 vs crypto-bigint 29,921 | −17,512 (0.41×) | 9,411 vs crypto-bigint 25,772 | −16,361 (0.37×) | 17,816 vs crypto-bigint 37,715 | −19,899 (0.47×) | 13,425 vs crypto-bigint 31,344 | −17,919 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,952 vs crypto-bigint 29,770 | −17,818 (0.40×) | 9,069 vs crypto-bigint 25,856 | −16,787 (0.35×) | 16,832 vs crypto-bigint 37,609 | −20,777 (0.45×) | 12,787 vs crypto-bigint 31,306 | −18,519 (0.41×) |
| div_uint, constant time | P-256 base | 11,892 vs crypto-bigint 29,583 | −17,691 (0.40×) | 9,122 vs k256/p256 22,336 | −13,214 (0.41×) | 17,216 vs crypto-bigint 37,714 | −20,498 (0.46×) | 12,673 vs crypto-bigint 31,211 | −18,538 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,530 vs crypto-bigint 29,626 | −17,096 (0.42×) | 9,421 vs crypto-bigint 25,760 | −16,339 (0.37×) | 17,412 vs crypto-bigint 37,453 | −20,041 (0.46×) | 13,365 vs crypto-bigint 31,094 | −17,729 (0.43×) |
| div_uint, constant time | Tom-256 base | 12,429 vs crypto-bigint 30,098 | −17,669 (0.41×) | 9,474 vs crypto-bigint 25,739 | −16,265 (0.37×) | 17,036 vs crypto-bigint 37,607 | −20,571 (0.45×) | 13,052 vs crypto-bigint 31,125 | −18,073 (0.42×) |
| div_uint, constant time | BLS12-381 base | 25,397 vs crypto-bigint 51,102 | −25,705 (0.50×) | 17,062 vs crypto-bigint 42,953 | −25,891 (0.40×) | 35,191 vs crypto-bigint 68,688 | −33,497 (0.51×) | 24,664 vs crypto-bigint 54,853 | −30,189 (0.45×) |
| div_uint, constant time | P-521 base | 42,915 vs crypto-bigint 104,665 | −61,750 (0.41×) | 25,999 vs crypto-bigint 84,345 | −58,346 (0.31×) | 63,827 vs crypto-bigint 127,116 | −63,289 (0.50×) | 42,935 vs crypto-bigint 101,019 | −58,084 (0.43×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.6×) | 3.1 (1.6×) |
| add | BN254 scalar | 4.9 | 13.9 (2.8×) | 12.2 (2.5×) | 14.5 (3.0×) | 18.5 (3.8×) |
| add | secp256k1 base | 5.8 | 14.0 (2.4×) | 11.8 (2.0×) | 14.8 (2.6×) | 19.8 (3.4×) |
| add | P-256 base | 6.0 | 14.4 (2.4×) | 12.3 (2.1×) | 14.7 (2.4×) | 20.1 (3.4×) |
| add | P-256 scalar | 5.8 | 14.5 (2.5×) | 11.8 (2.0×) | 15.4 (2.7×) | 20.5 (3.5×) |
| add | Tom-256 base | 6.0 | 13.6 (2.3×) | 11.8 (2.0×) | 14.6 (2.4×) | 20.4 (3.4×) |
| add | BLS12-381 base | 9.6 | 22.0 (2.3×) | 17.8 (1.9×) | 22.4 (2.3×) | 29.4 (3.1×) |
| add | P-521 base | 12.3 | 42.5 (3.5×) | 25.8 (2.1×) | 43.3 (3.5×) | 44.7 (3.6×) |
| sub | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BabyBear | 2.0 | 2.5 (1.2×) | 2.5 (1.2×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub | BN254 scalar | 4.9 | 15.0 (3.1×) | 11.9 (2.4×) | 14.9 (3.0×) | 19.5 (4.0×) |
| sub | secp256k1 base | 5.4 | 16.6 (3.1×) | 11.8 (2.2×) | 15.4 (2.9×) | 19.9 (3.7×) |
| sub | P-256 base | 5.5 | 14.3 (2.6×) | 11.8 (2.1×) | 13.9 (2.5×) | 18.9 (3.4×) |
| sub | P-256 scalar | 5.0 | 16.1 (3.2×) | 11.8 (2.4×) | 15.0 (3.0×) | 19.5 (3.9×) |
| sub | Tom-256 base | 5.5 | 15.0 (2.7×) | 11.8 (2.1×) | 14.4 (2.6×) | 19.5 (3.5×) |
| sub | BLS12-381 base | 9.6 | 22.9 (2.4×) | 18.0 (1.9×) | 21.6 (2.3×) | 29.3 (3.1×) |
| sub | P-521 base | 11.9 | 47.0 (3.9×) | 28.7 (2.4×) | 42.5 (3.6×) | 44.7 (3.8×) |
| mul | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 14.6 (3.8×) | 7.3 (1.9×) |
| mul | BN254 scalar | 68.8 | 234 (3.4×) | 80.7 (1.2×) | 387 (5.6×) | 158 (2.3×) |
| mul | secp256k1 base | 44.5 | 161 (3.6×) | 63.4 (1.4×) | 247 (5.6×) | 109 (2.4×) |
| mul | P-256 base | 49.5 | 172 (3.5×) | 71.0 (1.4×) | 280 (5.7×) | 120 (2.4×) |
| mul | P-256 scalar | 74.8 | 239 (3.2×) | 85.3 (1.1×) | 312 (4.2×) | 171 (2.3×) |
| mul | Tom-256 base | 70.9 | 234 (3.3×) | 80.4 (1.1×) | 313 (4.4×) | 152 (2.1×) |
| mul | BLS12-381 base | 138 | 727 (5.3×) | 182 (1.3×) | 923 (6.7×) | 355 (2.6×) |
| mul | P-521 base | 176 | 667 (3.8×) | 253 (1.4×) | 1,280 (7.3×) | 443 (2.5×) |
| square | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.6 (1.3×) |
| square | BabyBear | 3.4 | 4.7 (1.4×) | 4.8 (1.4×) | 5.7 (1.7×) | 5.4 (1.6×) |
| square | Goldilocks | 3.7 | 8.7 (2.4×) | 6.0 (1.6×) | 13.0 (3.5×) | 8.0 (2.2×) |
| square | BN254 scalar | 63.1 | 214 (3.4×) | 72.6 (1.2×) | 348 (5.5×) | 135 (2.1×) |
| square | secp256k1 base | 33.8 | 141 (4.2×) | 56.6 (1.7×) | 195 (5.8×) | 93.1 (2.8×) |
| square | P-256 base | 40.0 | 147 (3.7×) | 76.3 (1.9×) | 255 (6.4×) | 123 (3.1×) |
| square | P-256 scalar | 63.9 | 214 (3.3×) | 78.7 (1.2×) | 334 (5.2×) | 158 (2.5×) |
| square | Tom-256 base | 57.0 | 209 (3.7×) | 75.4 (1.3×) | 275 (4.8×) | 137 (2.4×) |
| square | BLS12-381 base | 131 | 547 (4.2×) | 176 (1.3×) | 822 (6.3×) | 328 (2.5×) |
| square | P-521 base | 117 | 558 (4.8×) | 246 (2.1×) | 912 (7.8×) | 394 (3.4×) |
| invert | Mersenne-31 | 214 | 266 (1.2×) | 267 (1.2×) | 268 (1.3×) | 270 (1.3×) |
| invert | BabyBear | 213 | 269 (1.3×) | 269 (1.3×) | 275 (1.3×) | 271 (1.3×) |
| invert | Goldilocks | 437 | 539 (1.2×) | 539 (1.2×) | 558 (1.3×) | 559 (1.3×) |
| invert | BN254 scalar | 5,057 | 8,467 (1.7×) | 6,530 (1.3×) | 11,735 (2.3×) | 7,878 (1.6×) |
| invert | secp256k1 base | 4,922 | 8,473 (1.7×) | 6,071 (1.2×) | 11,773 (2.4×) | 7,973 (1.6×) |
| invert | P-256 base | 4,837 | 8,280 (1.7×) | 6,088 (1.3×) | 11,332 (2.3×) | 7,709 (1.6×) |
| invert | P-256 scalar | 5,041 | 8,497 (1.7×) | 6,401 (1.3×) | 11,679 (2.3×) | 8,201 (1.6×) |
| invert | Tom-256 base | 5,018 | 8,375 (1.7×) | 6,430 (1.3×) | 11,384 (2.3×) | 7,961 (1.6×) |
| invert | BLS12-381 base | 8,808 | 19,326 (2.2×) | 11,416 (1.3×) | 24,344 (2.8×) | 15,495 (1.8×) |
| invert | P-521 base | 14,227 | 33,222 (2.3×) | 18,188 (1.3×) | 48,743 (3.4×) | 29,440 (2.1×) |
| sqrt | Mersenne-31 | 135 | 159 (1.2×) | 159 (1.2×) | 160 (1.2×) | 160 (1.2×) |
| sqrt | BabyBear | 1,971 | 2,115 (1.1×) | 2,027 (1.0×) | 2,149 (1.1×) | 2,060 (1.0×) |
| sqrt | Goldilocks | 2,833 | 4,825 (1.7×) | 3,489 (1.2×) | 6,105 (2.2×) | 5,101 (1.8×) |
| sqrt | BN254 scalar | 33,810 | 110,434 (3.3×) | 55,793 (1.7×) | 180,083 (5.3×) | 93,453 (2.8×) |
| sqrt | secp256k1 base | 12,304 | 40,902 (3.3×) | 18,739 (1.5×) | 56,499 (4.6×) | 28,354 (2.3×) |
| sqrt | P-256 base | 11,796 | 43,201 (3.7×) | 20,261 (1.7×) | 78,043 (6.6×) | 25,769 (2.2×) |
| sqrt | P-256 scalar | 24,127 | 74,311 (3.1×) | 35,225 (1.5×) | 112,404 (4.7×) | 64,824 (2.7×) |
| sqrt | Tom-256 base | 21,264 | 65,989 (3.1×) | 22,503 (1.1×) | 99,201 (4.7×) | 54,764 (2.6×) |
| sqrt | BLS12-381 base | 65,940 | 296,265 (4.5×) | 97,532 (1.5×) | 393,110 (6.0×) | 222,608 (3.4×) |
| sqrt | P-521 base | 68,261 | 319,464 (4.7×) | 127,191 (1.9×) | 472,407 (6.9×) | 259,168 (3.8×) |
| mul_u64 | Mersenne-31 | 5.8 | 6.7 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_u64 | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_u64 | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_u64 | BN254 scalar | 20.7 | 99.5 (4.8×) | 28.8 (1.4×) | 124 (6.0×) | 44.1 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 42.2 (4.4×) | 12.6 (1.3×) | 65.2 (6.7×) | 24.0 (2.5×) |
| mul_u64 | P-256 base | 18.9 | 72.3 (3.8×) | 27.3 (1.4×) | 94.1 (5.0×) | 41.1 (2.2×) |
| mul_u64 | P-256 scalar | 22.3 | 84.1 (3.8×) | 30.3 (1.4×) | 112 (5.0×) | 48.7 (2.2×) |
| mul_u64 | Tom-256 base | 20.5 | 82.1 (4.0×) | 30.0 (1.5×) | 115 (5.6×) | 44.5 (2.2×) |
| mul_u64 | BLS12-381 base | 28.8 | 131 (4.5×) | 44.2 (1.5×) | 167 (5.8×) | 61.5 (2.1×) |
| mul_u64 | P-521 base | 19.7 | 92.3 (4.7×) | 31.8 (1.6×) | 249 (12.6×) | 50.1 (2.5×) |
| mul_uint | Mersenne-31 | 5.9 | 6.7 (1.1×) | 6.7 (1.1×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_uint | BabyBear | 8.9 | 13.0 (1.5×) | 11.3 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_uint | BN254 scalar | 103 | 439 (4.3×) | 181 (1.8×) | 547 (5.3×) | 287 (2.8×) |
| mul_uint | secp256k1 base | 56.9 | 266 (4.7×) | 62.5 (1.1×) | 319 (5.6×) | 110 (1.9×) |
| mul_uint | P-256 base | 102 | 353 (3.5×) | 125 (1.2×) | 435 (4.3×) | 233 (2.3×) |
| mul_uint | P-256 scalar | 110 | 426 (3.9×) | 180 (1.6×) | 478 (4.3×) | 343 (3.1×) |
| mul_uint | Tom-256 base | 108 | 412 (3.8×) | 162 (1.5×) | 486 (4.5×) | 323 (3.0×) |
| mul_uint | BLS12-381 base | 211 | 969 (4.6×) | 370 (1.8×) | 1,145 (5.4×) | 925 (4.4×) |
| mul_uint | P-521 base | 350 | 1,545 (4.4×) | 266 (0.8×) | 2,384 (6.8×) | 541 (1.5×) |
| add_u64 | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.7 (1.2×) | 5.4 (1.2×) |
| add_u64 | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.6 (2.1×) |
| add_u64 | Goldilocks | 4.8 | 9.3 (1.9×) | 7.0 (1.5×) | 11.8 (2.5×) | 10.0 (2.1×) |
| add_u64 | BN254 scalar | 24.8 | 118 (4.8×) | 53.0 (2.1×) | 142 (5.7×) | 94.3 (3.8×) |
| add_u64 | secp256k1 base | 5.8 | 8.0 (1.4×) | 8.0 (1.4×) | 8.0 (1.4×) | 12.0 (2.1×) |
| add_u64 | P-256 base | 25.7 | 71.3 (2.8×) | 35.9 (1.4×) | 84.6 (3.3×) | 67.1 (2.6×) |
| add_u64 | P-256 scalar | 25.4 | 70.7 (2.8×) | 40.7 (1.6×) | 74.6 (2.9×) | 84.3 (3.3×) |
| add_u64 | Tom-256 base | 26.1 | 88.5 (3.4×) | 41.7 (1.6×) | 108 (4.1×) | 71.8 (2.8×) |
| add_u64 | BLS12-381 base | 38.2 | 169 (4.4×) | 65.6 (1.7×) | 254 (6.6×) | 95.4 (2.5×) |
| add_u64 | P-521 base | 10.7 | 20.9 (2.0×) | 15.4 (1.4×) | 22.1 (2.1×) | 23.4 (2.2×) |
| add_uint | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.7 (1.2×) | 5.4 (1.2×) |
| add_uint | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.6 (2.1×) |
| add_uint | Goldilocks | 4.7 | 9.3 (2.0×) | 7.1 (1.5×) | 11.8 (2.5×) | 10.0 (2.1×) |
| add_uint | BN254 scalar | 72.0 | 246 (3.4×) | 99.5 (1.4×) | 356 (4.9×) | 178 (2.5×) |
| add_uint | secp256k1 base | 6.8 | 21.0 (3.1×) | 14.6 (2.1×) | 18.3 (2.7×) | 23.8 (3.5×) |
| add_uint | P-256 base | 57.3 | 188 (3.3×) | 79.4 (1.4×) | 325 (5.7×) | 147 (2.6×) |
| add_uint | P-256 scalar | 81.0 | 260 (3.2×) | 109 (1.3×) | 331 (4.1×) | 175 (2.2×) |
| add_uint | Tom-256 base | 73.8 | 251 (3.4×) | 102 (1.4×) | 330 (4.5×) | 182 (2.5×) |
| add_uint | BLS12-381 base | 152 | 769 (5.1×) | 214 (1.4×) | 930 (6.1×) | 385 (2.5×) |
| add_uint | P-521 base | 23.2 | 65.1 (2.8×) | 41.8 (1.8×) | 86.7 (3.7×) | 71.6 (3.1×) |
| sub_u64 | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.1 (1.1×) |
| sub_u64 | BabyBear | 5.9 | 11.1 (1.9×) | 8.5 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| sub_u64 | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.6 (2.7×) | 9.3 (2.2×) |
| sub_u64 | BN254 scalar | 24.8 | 109 (4.4×) | 55.6 (2.2×) | 139 (5.6×) | 73.9 (3.0×) |
| sub_u64 | secp256k1 base | 4.2 | 8.7 (2.1×) | 8.0 (1.9×) | 8.0 (1.9×) | 11.0 (2.6×) |
| sub_u64 | P-256 base | 19.8 | 70.5 (3.6×) | 35.7 (1.8×) | 80.1 (4.0×) | 60.8 (3.1×) |
| sub_u64 | P-256 scalar | 24.9 | 67.9 (2.7×) | 40.0 (1.6×) | 75.9 (3.0×) | 84.1 (3.4×) |
| sub_u64 | Tom-256 base | 24.6 | 87.4 (3.6×) | 49.6 (2.0×) | 104 (4.2×) | 72.0 (2.9×) |
| sub_u64 | BLS12-381 base | 37.8 | 151 (4.0×) | 66.6 (1.8×) | 251 (6.6×) | 96.3 (2.5×) |
| sub_u64 | P-521 base | 9.6 | 22.2 (2.3×) | 18.7 (1.9×) | 20.2 (2.1×) | 21.1 (2.2×) |
| sub_uint | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.1 (1.1×) |
| sub_uint | BabyBear | 5.8 | 11.1 (1.9×) | 8.5 (1.5×) | 13.0 (2.2×) | 12.5 (2.2×) |
| sub_uint | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.7 (2.7×) | 9.2 (2.1×) |
| sub_uint | BN254 scalar | 74.6 | 246 (3.3×) | 93.8 (1.3×) | 364 (4.9×) | 178 (2.4×) |
| sub_uint | secp256k1 base | 5.4 | 21.2 (3.9×) | 14.6 (2.7×) | 18.9 (3.5×) | 24.4 (4.5×) |
| sub_uint | P-256 base | 53.1 | 175 (3.3×) | 86.3 (1.6×) | 270 (5.1×) | 138 (2.6×) |
| sub_uint | P-256 scalar | 79.9 | 249 (3.1×) | 103 (1.3×) | 325 (4.1×) | 184 (2.3×) |
| sub_uint | Tom-256 base | 76.0 | 252 (3.3×) | 98.3 (1.3×) | 319 (4.2×) | 177 (2.3×) |
| sub_uint | BLS12-381 base | 155 | 765 (4.9×) | 206 (1.3×) | 929 (6.0×) | 383 (2.5×) |
| sub_uint | P-521 base | 19.9 | 75.5 (3.8×) | 41.9 (2.1×) | 80.3 (4.0×) | 68.7 (3.5×) |
| from_u64 | Mersenne-31 | 3.2 | 4.0 (1.2×) | 4.0 (1.2×) | 4.0 (1.2×) | 4.3 (1.3×) |
| from_u64 | BabyBear | 6.3 | 11.1 (1.8×) | 7.3 (1.2×) | 12.1 (1.9×) | 10.3 (1.6×) |
| from_u64 | Goldilocks | 3.4 | 7.4 (2.2×) | 5.9 (1.7×) | 9.0 (2.6×) | 7.0 (2.1×) |
| from_u64 | BN254 scalar | 22.0 | 101 (4.6×) | 29.1 (1.3×) | 121 (5.5×) | 50.1 (2.3×) |
| from_u64 | secp256k1 base | 2.0 | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) |
| from_u64 | P-256 base | 17.6 | 56.6 (3.2×) | 24.7 (1.4×) | 69.9 (4.0×) | 41.0 (2.3×) |
| from_u64 | P-256 scalar | 18.0 | 55.0 (3.1×) | 26.3 (1.5×) | 58.3 (3.2×) | 48.1 (2.7×) |
| from_u64 | Tom-256 base | 21.0 | 74.5 (3.5×) | 28.2 (1.3×) | 86.1 (4.1×) | 50.0 (2.4×) |
| from_u64 | BLS12-381 base | 29.1 | 144 (4.9×) | 50.3 (1.7×) | 183 (6.3×) | 68.0 (2.3×) |
| from_u64 | P-521 base | 5.0 | 6.1 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64 | BN254 scalar | 612 | 1,214 (2.0×) | 843 (1.4×) | 1,378 (2.3×) | 1,106 (1.8×) |
| div_u64 | secp256k1 base | 564 | 1,010 (1.8×) | 716 (1.3×) | 1,216 (2.2×) | 817 (1.4×) |
| div_u64 | P-256 base | 602 | 1,091 (1.8×) | 804 (1.3×) | 1,345 (2.2×) | 1,010 (1.7×) |
| div_u64 | P-256 scalar | 619 | 1,187 (1.9×) | 867 (1.4×) | 1,440 (2.3×) | 1,138 (1.8×) |
| div_u64 | Tom-256 base | 608 | 1,209 (2.0×) | 865 (1.4×) | 1,469 (2.4×) | 1,120 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,813 (2.4×) | 1,082 (1.4×) | 2,463 (3.3×) | 1,776 (2.4×) |
| div_u64 | P-521 base | 925 | 2,457 (2.7×) | 1,010 (1.1×) | 3,589 (3.9×) | 1,513 (1.6×) |
| div_uint | Mersenne-31 | 215 | 273 (1.3×) | 274 (1.3×) | 279 (1.3×) | 279 (1.3×) |
| div_uint | BabyBear | 215 | 276 (1.3×) | 275 (1.3×) | 279 (1.3×) | 281 (1.3×) |
| div_uint | Goldilocks | 435 | 552 (1.3×) | 540 (1.2×) | 591 (1.4×) | 577 (1.3×) |
| div_uint | BN254 scalar | 5,014 | 9,242 (1.8×) | 6,697 (1.3×) | 12,726 (2.5×) | 8,387 (1.7×) |
| div_uint | secp256k1 base | 4,901 | 8,575 (1.7×) | 6,044 (1.2×) | 12,029 (2.5×) | 8,170 (1.7×) |
| div_uint | P-256 base | 5,029 | 8,789 (1.7×) | 6,162 (1.2×) | 12,120 (2.4×) | 7,960 (1.6×) |
| div_uint | P-256 scalar | 4,958 | 9,151 (1.8×) | 6,620 (1.3×) | 12,491 (2.5×) | 8,662 (1.7×) |
| div_uint | Tom-256 base | 5,057 | 9,086 (1.8×) | 6,640 (1.3×) | 12,437 (2.5×) | 8,376 (1.7×) |
| div_uint | BLS12-381 base | 9,020 | 21,059 (2.3×) | 12,225 (1.4×) | 26,412 (2.9×) | 17,165 (1.9×) |
| div_uint | P-521 base | 14,921 | 34,261 (2.3×) | 18,489 (1.2×) | 49,508 (3.3×) | 29,987 (2.0×) |
| add, constant time | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add, constant time | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add, constant time | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.6×) | 3.1 (1.6×) |
| add, constant time | BN254 scalar | 4.9 | 23.6 (4.8×) | 16.3 (3.3×) | 20.7 (4.2×) | 26.5 (5.4×) |
| add, constant time | secp256k1 base | 5.8 | 26.3 (4.5×) | 18.2 (3.1×) | 24.9 (4.3×) | 29.0 (5.0×) |
| add, constant time | P-256 base | 6.0 | 23.5 (3.9×) | 18.1 (3.0×) | 22.7 (3.8×) | 28.0 (4.7×) |
| add, constant time | P-256 scalar | 5.8 | 27.1 (4.7×) | 17.9 (3.1×) | 24.0 (4.1×) | 29.5 (5.1×) |
| add, constant time | Tom-256 base | 6.0 | 27.8 (4.6×) | 18.0 (3.0×) | 24.0 (4.0×) | 28.5 (4.8×) |
| add, constant time | BLS12-381 base | 9.6 | 42.7 (4.4×) | 25.5 (2.7×) | 35.2 (3.7×) | 41.2 (4.3×) |
| add, constant time | P-521 base | 12.2 | 74.6 (6.1×) | 46.6 (3.8×) | 79.0 (6.5×) | 74.1 (6.1×) |
| sub, constant time | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BabyBear | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub, constant time | BN254 scalar | 5.0 | 21.1 (4.2×) | 15.4 (3.1×) | 20.8 (4.2×) | 24.0 (4.8×) |
| sub, constant time | secp256k1 base | 5.3 | 19.1 (3.6×) | 14.4 (2.7×) | 18.6 (3.5×) | 22.5 (4.2×) |
| sub, constant time | P-256 base | 5.3 | 17.3 (3.3×) | 13.4 (2.5×) | 18.4 (3.5×) | 22.0 (4.2×) |
| sub, constant time | P-256 scalar | 5.0 | 21.0 (4.2×) | 14.7 (2.9×) | 22.0 (4.4×) | 24.0 (4.8×) |
| sub, constant time | Tom-256 base | 5.5 | 21.1 (3.8×) | 15.3 (2.8×) | 21.0 (3.8×) | 24.0 (4.4×) |
| sub, constant time | BLS12-381 base | 9.7 | 37.8 (3.9×) | 24.5 (2.5×) | 34.7 (3.6×) | 37.0 (3.8×) |
| sub, constant time | P-521 base | 11.7 | 56.2 (4.8×) | 37.1 (3.2×) | 58.0 (5.0×) | 53.1 (4.5×) |
| mul, constant time | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul, constant time | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul, constant time | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 14.6 (3.8×) | 7.3 (1.9×) |
| mul, constant time | BN254 scalar | 67.8 | 235 (3.5×) | 82.1 (1.2×) | 414 (6.1×) | 156 (2.3×) |
| mul, constant time | secp256k1 base | 44.6 | 160 (3.6×) | 63.1 (1.4×) | 246 (5.5×) | 110 (2.5×) |
| mul, constant time | P-256 base | 49.6 | 173 (3.5×) | 71.3 (1.4×) | 279 (5.6×) | 120 (2.4×) |
| mul, constant time | P-256 scalar | 75.0 | 241 (3.2×) | 85.4 (1.1×) | 311 (4.1×) | 164 (2.2×) |
| mul, constant time | Tom-256 base | 70.0 | 233 (3.3×) | 80.1 (1.1×) | 305 (4.4×) | 154 (2.2×) |
| mul, constant time | BLS12-381 base | 141 | 725 (5.1×) | 185 (1.3×) | 919 (6.5×) | 341 (2.4×) |
| mul, constant time | P-521 base | 176 | 666 (3.8×) | 253 (1.4×) | 1,281 (7.3×) | 443 (2.5×) |
| square, constant time | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.6 (1.3×) |
| square, constant time | BabyBear | 3.5 | 4.7 (1.3×) | 4.8 (1.4×) | 5.7 (1.6×) | 5.4 (1.5×) |
| square, constant time | Goldilocks | 3.8 | 8.7 (2.3×) | 6.0 (1.6×) | 13.0 (3.4×) | 8.0 (2.1×) |
| square, constant time | BN254 scalar | 64.9 | 214 (3.3×) | 72.8 (1.1×) | 303 (4.7×) | 159 (2.4×) |
| square, constant time | secp256k1 base | 33.8 | 141 (4.2×) | 56.6 (1.7×) | 173 (5.1×) | 89.4 (2.6×) |
| square, constant time | P-256 base | 40.3 | 147 (3.6×) | 76.3 (1.9×) | 237 (5.9×) | 116 (2.9×) |
| square, constant time | P-256 scalar | 64.4 | 214 (3.3×) | 79.2 (1.2×) | 334 (5.2×) | 157 (2.4×) |
| square, constant time | Tom-256 base | 57.0 | 209 (3.7×) | 75.4 (1.3×) | 275 (4.8×) | 136 (2.4×) |
| square, constant time | BLS12-381 base | 134 | 567 (4.2×) | 176 (1.3×) | 822 (6.1×) | 321 (2.4×) |
| square, constant time | P-521 base | 118 | 563 (4.8×) | 246 (2.1×) | 909 (7.7×) | 394 (3.3×) |
| invert, constant time | Mersenne-31 | 243 | 263 (1.1×) | 264 (1.1×) | 268 (1.1×) | 266 (1.1×) |
| invert, constant time | BabyBear | 383 | 451 (1.2×) | 458 (1.2×) | 438 (1.1×) | 417 (1.1×) |
| invert, constant time | Goldilocks | 648 | 1,154 (1.8×) | 841 (1.3×) | 1,416 (2.2×) | 932 (1.4×) |
| invert, constant time | BN254 scalar | 6,758 | 12,000 (1.8×) | 9,454 (1.4×) | 16,754 (2.5×) | 12,515 (1.9×) |
| invert, constant time | secp256k1 base | 6,860 | 11,814 (1.7×) | 9,110 (1.3×) | 16,689 (2.4×) | 12,599 (1.8×) |
| invert, constant time | P-256 base | 6,485 | 11,522 (1.8×) | 8,957 (1.4×) | 16,344 (2.5×) | 12,302 (1.9×) |
| invert, constant time | P-256 scalar | 6,909 | 11,908 (1.7×) | 9,363 (1.4×) | 16,953 (2.5×) | 12,804 (1.9×) |
| invert, constant time | Tom-256 base | 6,973 | 11,809 (1.7×) | 9,247 (1.3×) | 17,029 (2.4×) | 12,699 (1.8×) |
| invert, constant time | BLS12-381 base | 12,456 | 23,998 (1.9×) | 16,671 (1.3×) | 32,888 (2.6×) | 24,146 (1.9×) |
| invert, constant time | P-521 base | 20,025 | 41,737 (2.1×) | 25,894 (1.3×) | 63,372 (3.2×) | 41,461 (2.1×) |
| sqrt, constant time | Mersenne-31 | 142 | 159 (1.1×) | 159 (1.1×) | 172 (1.2×) | 172 (1.2×) |
| sqrt, constant time | BabyBear | 5,592 | 7,133 (1.3×) | 6,475 (1.2×) | 6,802 (1.2×) | 6,524 (1.2×) |
| sqrt, constant time | Goldilocks | 7,149 | 10,859 (1.5×) | 8,338 (1.2×) | 14,673 (2.1×) | 10,417 (1.5×) |
| sqrt, constant time | BN254 scalar | 58,556 | 180,175 (3.1×) | 83,297 (1.4×) | 289,145 (4.9×) | 148,720 (2.5×) |
| sqrt, constant time | secp256k1 base | 12,274 | 41,156 (3.4×) | 18,851 (1.5×) | 65,022 (5.3×) | 27,999 (2.3×) |
| sqrt, constant time | P-256 base | 12,041 | 42,910 (3.6×) | 15,404 (1.3×) | 74,509 (6.2×) | 30,745 (2.6×) |
| sqrt, constant time | P-256 scalar | 25,625 | 77,925 (3.0×) | 36,678 (1.4×) | 116,129 (4.5×) | 66,567 (2.6×) |
| sqrt, constant time | Tom-256 base | 20,048 | 65,986 (3.3×) | 31,865 (1.6×) | 99,847 (5.0×) | 50,129 (2.5×) |
| sqrt, constant time | BLS12-381 base | 69,784 | 297,738 (4.3×) | 97,462 (1.4×) | 396,487 (5.7×) | 213,597 (3.1×) |
| sqrt, constant time | P-521 base | 67,949 | 324,679 (4.8×) | 131,004 (1.9×) | 507,930 (7.5×) | 256,756 (3.8×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 6.7 (1.1×) | 6.7 (1.1×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_u64, constant time | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_u64, constant time | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.9 (1.8×) | 125 (5.7×) | 57.4 (2.6×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 61.3 (5.6×) | 24.9 (2.3×) | 80.0 (7.3×) | 37.2 (3.4×) |
| mul_u64, constant time | P-256 base | 24.4 | 85.5 (3.5×) | 40.3 (1.7×) | 102 (4.2×) | 57.1 (2.3×) |
| mul_u64, constant time | P-256 scalar | 22.9 | 102 (4.5×) | 43.5 (1.9×) | 122 (5.3×) | 60.9 (2.7×) |
| mul_u64, constant time | Tom-256 base | 22.4 | 95.3 (4.3×) | 41.7 (1.9×) | 122 (5.4×) | 59.3 (2.6×) |
| mul_u64, constant time | BLS12-381 base | 34.9 | 149 (4.3×) | 65.9 (1.9×) | 168 (4.8×) | 85.8 (2.5×) |
| mul_u64, constant time | P-521 base | 26.1 | 138 (5.3×) | 69.2 (2.7×) | 165 (6.3×) | 95.8 (3.7×) |
| mul_uint, constant time | Mersenne-31 | 5.8 | 6.7 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.7 (1.3×) |
| mul_uint, constant time | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_uint, constant time | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_uint, constant time | BN254 scalar | 119 | 487 (4.1×) | 181 (1.5×) | 605 (5.1×) | 308 (2.6×) |
| mul_uint, constant time | secp256k1 base | 78.2 | 320 (4.1×) | 88.1 (1.1×) | 377 (4.8×) | 119 (1.5×) |
| mul_uint, constant time | P-256 base | 118 | 412 (3.5×) | 131 (1.1×) | 492 (4.2×) | 224 (1.9×) |
| mul_uint, constant time | P-256 scalar | 132 | 471 (3.6×) | 180 (1.4×) | 565 (4.3×) | 341 (2.6×) |
| mul_uint, constant time | Tom-256 base | 122 | 461 (3.8×) | 165 (1.4×) | 557 (4.6×) | 306 (2.5×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,083 (4.2×) | 366 (1.4×) | 1,679 (6.6×) | 926 (3.6×) |
| mul_uint, constant time | P-521 base | 426 | 1,997 (4.7×) | 315 (0.7×) | 2,920 (6.9×) | 623 (1.5×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.7 (1.3×) | 5.4 (1.2×) |
| add_u64, constant time | BabyBear | 5.4 | 10.3 (1.9×) | 8.4 (1.6×) | 12.8 (2.4×) | 12.2 (2.3×) |
| add_u64, constant time | Goldilocks | 4.8 | 9.3 (1.9×) | 7.1 (1.5×) | 12.0 (2.5×) | 10.0 (2.1×) |
| add_u64, constant time | BN254 scalar | 23.5 | 93.6 (4.0×) | 50.1 (2.1×) | 108 (4.6×) | 73.9 (3.1×) |
| add_u64, constant time | secp256k1 base | 5.8 | 22.4 (3.9×) | 16.1 (2.8×) | 20.2 (3.5×) | 25.0 (4.3×) |
| add_u64, constant time | P-256 base | 16.9 | 70.8 (4.2×) | 44.0 (2.6×) | 77.1 (4.6×) | 60.3 (3.6×) |
| add_u64, constant time | P-256 scalar | 23.9 | 92.2 (3.9×) | 50.4 (2.1×) | 112 (4.7×) | 72.6 (3.0×) |
| add_u64, constant time | Tom-256 base | 22.3 | 94.4 (4.2×) | 48.7 (2.2×) | 111 (5.0×) | 67.9 (3.0×) |
| add_u64, constant time | BLS12-381 base | 36.6 | 147 (4.0×) | 98.6 (2.7×) | 172 (4.7×) | 114 (3.1×) |
| add_u64, constant time | P-521 base | 10.7 | 65.0 (6.1×) | 44.2 (4.1×) | 69.5 (6.5×) | 64.2 (6.0×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.7 (1.3×) | 5.4 (1.2×) |
| add_uint, constant time | BabyBear | 5.4 | 10.4 (1.9×) | 8.5 (1.6×) | 12.8 (2.4×) | 12.1 (2.2×) |
| add_uint, constant time | Goldilocks | 4.7 | 9.3 (2.0×) | 7.0 (1.5×) | 12.0 (2.6×) | 10.0 (2.1×) |
| add_uint, constant time | BN254 scalar | 75.8 | 249 (3.3×) | 102 (1.3×) | 352 (4.6×) | 182 (2.4×) |
| add_uint, constant time | secp256k1 base | 9.3 | 41.7 (4.5×) | 29.5 (3.2×) | 37.0 (4.0×) | 44.4 (4.8×) |
| add_uint, constant time | P-256 base | 57.2 | 180 (3.1×) | 104 (1.8×) | 283 (4.9×) | 145 (2.5×) |
| add_uint, constant time | P-256 scalar | 83.2 | 260 (3.1×) | 108 (1.3×) | 319 (3.8×) | 175 (2.1×) |
| add_uint, constant time | Tom-256 base | 73.7 | 251 (3.4×) | 102 (1.4×) | 317 (4.3×) | 182 (2.5×) |
| add_uint, constant time | BLS12-381 base | 156 | 769 (4.9×) | 208 (1.3×) | 931 (6.0×) | 388 (2.5×) |
| add_uint, constant time | P-521 base | 22.3 | 135 (6.1×) | 105 (4.7×) | 144 (6.5×) | 136 (6.1×) |
| sub_u64, constant time | Mersenne-31 | 4.6 | 5.0 (1.1×) | 5.0 (1.1×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64, constant time | BabyBear | 5.2 | 11.2 (2.2×) | 8.7 (1.7×) | 14.0 (2.7×) | 12.0 (2.3×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.3 (2.2×) |
| sub_u64, constant time | BN254 scalar | 25.4 | 91.2 (3.6×) | 42.4 (1.7×) | 109 (4.3×) | 71.9 (2.8×) |
| sub_u64, constant time | secp256k1 base | 4.2 | 15.5 (3.7×) | 12.3 (2.9×) | 14.6 (3.5×) | 18.5 (4.4×) |
| sub_u64, constant time | P-256 base | 16.2 | 63.6 (3.9×) | 36.1 (2.2×) | 75.4 (4.7×) | 54.8 (3.4×) |
| sub_u64, constant time | P-256 scalar | 24.1 | 88.4 (3.7×) | 43.9 (1.8×) | 108 (4.5×) | 69.6 (2.9×) |
| sub_u64, constant time | Tom-256 base | 22.9 | 93.0 (4.1×) | 43.3 (1.9×) | 106 (4.6×) | 67.5 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 36.5 | 142 (3.9×) | 93.5 (2.6×) | 177 (4.8×) | 105 (2.9×) |
| sub_u64, constant time | P-521 base | 9.6 | 42.2 (4.4×) | 34.1 (3.6×) | 52.0 (5.4×) | 45.5 (4.7×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.1 (1.1×) |
| sub_uint, constant time | BabyBear | 5.2 | 11.2 (2.2×) | 8.7 (1.7×) | 14.0 (2.7×) | 12.0 (2.3×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.3 (2.2×) |
| sub_uint, constant time | BN254 scalar | 77.7 | 253 (3.3×) | 93.5 (1.2×) | 338 (4.4×) | 175 (2.3×) |
| sub_uint, constant time | secp256k1 base | 7.2 | 32.9 (4.6×) | 26.0 (3.6×) | 34.3 (4.8×) | 37.5 (5.2×) |
| sub_uint, constant time | P-256 base | 53.1 | 171 (3.2×) | 95.0 (1.8×) | 274 (5.2×) | 136 (2.6×) |
| sub_uint, constant time | P-256 scalar | 80.0 | 248 (3.1×) | 106 (1.3×) | 333 (4.2×) | 189 (2.4×) |
| sub_uint, constant time | Tom-256 base | 74.9 | 252 (3.4×) | 97.2 (1.3×) | 321 (4.3×) | 179 (2.4×) |
| sub_uint, constant time | BLS12-381 base | 154 | 761 (4.9×) | 209 (1.4×) | 928 (6.0×) | 382 (2.5×) |
| sub_uint, constant time | P-521 base | 19.0 | 127 (6.7×) | 86.9 (4.6×) | 124 (6.5×) | 114 (6.0×) |
| from_u64, constant time | Mersenne-31 | 3.8 | 4.7 (1.2×) | 4.7 (1.2×) | 5.2 (1.4×) | 5.2 (1.4×) |
| from_u64, constant time | BabyBear | 4.9 | 10.0 (2.0×) | 6.8 (1.4×) | 13.3 (2.7×) | 10.5 (2.1×) |
| from_u64, constant time | Goldilocks | 3.4 | 7.4 (2.2×) | 6.0 (1.8×) | 9.0 (2.6×) | 7.0 (2.1×) |
| from_u64, constant time | BN254 scalar | 26.2 | 83.6 (3.2×) | 34.7 (1.3×) | 98.1 (3.7×) | 50.0 (1.9×) |
| from_u64, constant time | secp256k1 base | 2.0 | 3.8 (1.9×) | 3.8 (1.9×) | 3.8 (1.9×) | 3.7 (1.9×) |
| from_u64, constant time | P-256 base | 18.6 | 51.5 (2.8×) | 26.7 (1.4×) | 61.3 (3.3×) | 38.5 (2.1×) |
| from_u64, constant time | P-256 scalar | 27.2 | 84.1 (3.1×) | 34.6 (1.3×) | 96.8 (3.6×) | 50.0 (1.8×) |
| from_u64, constant time | Tom-256 base | 23.5 | 82.0 (3.5×) | 34.8 (1.5×) | 100 (4.3×) | 49.0 (2.1×) |
| from_u64, constant time | BLS12-381 base | 41.9 | 145 (3.5×) | 56.6 (1.4×) | 198 (4.7×) | 66.1 (1.6×) |
| from_u64, constant time | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64, constant time | BN254 scalar | 6,851 | 12,341 (1.8×) | 9,385 (1.4×) | 17,656 (2.6×) | 12,998 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,776 | 11,913 (1.8×) | 9,038 (1.3×) | 16,890 (2.5×) | 12,813 (1.9×) |
| div_u64, constant time | P-256 base | 6,666 | 11,610 (1.7×) | 9,169 (1.4×) | 16,906 (2.5×) | 12,446 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,956 | 12,254 (1.8×) | 9,514 (1.4×) | 17,121 (2.5×) | 13,110 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,282 | 12,219 (1.7×) | 9,271 (1.3×) | 16,953 (2.3×) | 12,893 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,384 | 24,771 (2.0×) | 16,799 (1.4×) | 34,564 (2.8×) | 24,347 (2.0×) |
| div_u64, constant time | P-521 base | 20,382 | 42,464 (2.1×) | 25,848 (1.3×) | 64,233 (3.2×) | 42,966 (2.1×) |
| div_uint, constant time | Mersenne-31 | 262 | 272 (1.0×) | 272 (1.0×) | 280 (1.1×) | 273 (1.0×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 465 (1.2×) | 474 (1.2×) | 447 (1.1×) |
| div_uint, constant time | Goldilocks | 668 | 1,146 (1.7×) | 863 (1.3×) | 1,437 (2.2×) | 958 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,895 | 12,492 (1.8×) | 9,611 (1.4×) | 17,824 (2.6×) | 13,360 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,771 | 11,984 (1.8×) | 9,095 (1.3×) | 16,891 (2.5×) | 12,869 (1.9×) |
| div_uint, constant time | P-256 base | 6,740 | 12,021 (1.8×) | 9,105 (1.4×) | 17,274 (2.6×) | 12,625 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,952 | 12,352 (1.8×) | 9,506 (1.4×) | 17,361 (2.5×) | 13,524 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,337 | 12,384 (1.7×) | 9,459 (1.3×) | 16,998 (2.3×) | 13,046 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,490 | 25,380 (2.0×) | 16,845 (1.3×) | 35,281 (2.8×) | 24,786 (2.0×) |
| div_uint, constant time | P-521 base | 20,380 | 42,465 (2.1×) | 26,245 (1.3×) | 64,579 (3.2×) | 43,052 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.7 | 13.1 (2.0×) | 10.8 (1.6×) | 13.4 (2.0×) | 17.9 (2.7×) |
| add | secp256k1 base | 6.9 | 13.7 (2.0×) | 10.7 (1.6×) | 13.7 (2.0×) | 18.3 (2.7×) |
| add | P-256 base | 6.9 | 12.2 (1.8×) | 11.2 (1.6×) | 13.0 (1.9×) | 18.8 (2.7×) |
| add | P-256 scalar | 6.9 | 13.6 (2.0×) | 11.1 (1.6×) | 14.5 (2.1×) | 19.5 (2.8×) |
| add | Tom-256 base | 6.9 | 13.3 (1.9×) | 11.2 (1.6×) | 13.1 (1.9×) | 18.4 (2.7×) |
| add | BLS12-381 base | 8.9 | 22.0 (2.5×) | 18.7 (2.1×) | 23.2 (2.6×) | 30.4 (3.4×) |
| add | P-521 base | 14.7 | 41.4 (2.8×) | 26.8 (1.8×) | 47.0 (3.2×) | 48.4 (3.3×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.7 | 13.1 (2.0×) | 10.7 (1.6×) | 13.3 (2.0×) | 18.0 (2.7×) |
| sub | secp256k1 base | 8.0 | 16.6 (2.1×) | 10.8 (1.4×) | 13.9 (1.7×) | 17.9 (2.2×) |
| sub | P-256 base | 7.0 | 13.4 (1.9×) | 10.4 (1.5×) | 12.5 (1.8×) | 17.5 (2.5×) |
| sub | P-256 scalar | 6.9 | 14.3 (2.1×) | 10.7 (1.6×) | 13.0 (1.9×) | 17.8 (2.6×) |
| sub | Tom-256 base | 8.0 | 13.1 (1.6×) | 10.8 (1.4×) | 12.7 (1.6×) | 18.6 (2.3×) |
| sub | BLS12-381 base | 9.9 | 22.8 (2.3×) | 17.5 (1.8×) | 22.6 (2.3×) | 30.4 (3.1×) |
| sub | P-521 base | 12.4 | 43.2 (3.5×) | 26.9 (2.2×) | 43.6 (3.5×) | 45.9 (3.7×) |
| mul | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul | BabyBear | 10.9 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul | Goldilocks | 8.9 | 16.5 (1.9×) | 12.5 (1.4×) | 18.4 (2.1×) | 12.3 (1.4×) |
| mul | BN254 scalar | 72.5 | 227 (3.1×) | 92.7 (1.3×) | 367 (5.1×) | 152 (2.1×) |
| mul | secp256k1 base | 54.8 | 173 (3.2×) | 77.5 (1.4×) | 218 (4.0×) | 105 (1.9×) |
| mul | P-256 base | 52.5 | 188 (3.6×) | 71.7 (1.4×) | 292 (5.6×) | 119 (2.3×) |
| mul | P-256 scalar | 84.0 | 234 (2.8×) | 93.2 (1.1×) | 319 (3.8×) | 170 (2.0×) |
| mul | Tom-256 base | 76.0 | 235 (3.1×) | 86.6 (1.1×) | 329 (4.3×) | 151 (2.0×) |
| mul | BLS12-381 base | 141 | 723 (5.1×) | 186 (1.3×) | 928 (6.6×) | 336 (2.4×) |
| mul | P-521 base | 191 | 684 (3.6×) | 247 (1.3×) | 1,312 (6.9×) | 444 (2.3×) |
| square | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square | Goldilocks | 9.0 | 15.0 (1.7×) | 11.0 (1.2×) | 18.5 (2.1×) | 12.7 (1.4×) |
| square | BN254 scalar | 66.7 | 210 (3.1×) | 83.0 (1.2×) | 326 (4.9×) | 140 (2.1×) |
| square | secp256k1 base | 42.9 | 150 (3.5×) | 67.2 (1.6×) | 181 (4.2×) | 87.1 (2.0×) |
| square | P-256 base | 44.0 | 160 (3.6×) | 62.0 (1.4×) | 237 (5.4×) | 102 (2.3×) |
| square | P-256 scalar | 72.9 | 221 (3.0×) | 88.2 (1.2×) | 279 (3.8×) | 143 (2.0×) |
| square | Tom-256 base | 64.5 | 212 (3.3×) | 82.3 (1.3×) | 211 (3.3×) | 126 (2.0×) |
| square | BLS12-381 base | 137 | 570 (4.2×) | 169 (1.2×) | 794 (5.8×) | 294 (2.1×) |
| square | P-521 base | 129 | 570 (4.4×) | 219 (1.7×) | 825 (6.4×) | 391 (3.0×) |
| mul_u64 | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64 | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 13.0 (1.2×) |
| mul_u64 | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.2 (2.8×) | 15.1 (1.7×) |
| mul_u64 | BN254 scalar | 25.9 | 89.0 (3.4×) | 34.5 (1.3×) | 121 (4.7×) | 45.1 (1.7×) |
| mul_u64 | secp256k1 base | 14.9 | 49.0 (3.3×) | 16.0 (1.1×) | 65.0 (4.4×) | 23.0 (1.5×) |
| mul_u64 | P-256 base | 24.9 | 74.9 (3.0×) | 33.0 (1.3×) | 95.2 (3.8×) | 42.3 (1.7×) |
| mul_u64 | P-256 scalar | 27.9 | 81.8 (2.9×) | 34.5 (1.2×) | 105 (3.8×) | 47.6 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.6 (3.0×) | 33.0 (1.2×) | 105 (3.8×) | 45.4 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 124 (3.6×) | 47.1 (1.4×) | 177 (5.1×) | 62.0 (1.8×) |
| mul_u64 | P-521 base | 32.7 | 94.2 (2.9×) | 39.5 (1.2×) | 199 (6.1×) | 52.5 (1.6×) |
| mul_uint | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.6 (1.4×) | 13.0 (1.2×) |
| mul_uint | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.2 (2.8×) | 15.2 (1.7×) |
| mul_uint | BN254 scalar | 114 | 440 (3.9×) | 167 (1.5×) | 532 (4.7×) | 298 (2.6×) |
| mul_uint | secp256k1 base | 67.9 | 265 (3.9×) | 74.6 (1.1×) | 320 (4.7×) | 108 (1.6×) |
| mul_uint | P-256 base | 110 | 355 (3.2×) | 137 (1.2×) | 423 (3.8×) | 232 (2.1×) |
| mul_uint | P-256 scalar | 116 | 425 (3.7×) | 178 (1.5×) | 472 (4.1×) | 293 (2.5×) |
| mul_uint | Tom-256 base | 114 | 410 (3.6×) | 171 (1.5×) | 487 (4.3×) | 287 (2.5×) |
| mul_uint | BLS12-381 base | 222 | 982 (4.4×) | 360 (1.6×) | 1,141 (5.1×) | 897 (4.0×) |
| mul_uint | P-521 base | 406 | 1,573 (3.9×) | 253 (0.6×) | 2,381 (5.9×) | 538 (1.3×) |
| add_u64 | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.8 (1.1×) | 4.7 (1.0×) |
| add_u64 | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_u64 | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.6 (2.5×) | 9.2 (2.0×) |
| add_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 44.7 (1.8×) | 140 (5.6×) | 65.7 (2.6×) |
| add_u64 | secp256k1 base | 7.4 | 7.0 (0.9×) | 6.0 (0.8×) | 7.0 (0.9×) | 10.0 (1.4×) |
| add_u64 | P-256 base | 22.1 | 67.7 (3.1×) | 34.9 (1.6×) | 77.5 (3.5×) | 54.9 (2.5×) |
| add_u64 | P-256 scalar | 22.7 | 66.8 (2.9×) | 38.0 (1.7×) | 74.7 (3.3×) | 64.9 (2.9×) |
| add_u64 | Tom-256 base | 26.0 | 83.9 (3.2×) | 39.7 (1.5×) | 99.6 (3.8×) | 65.2 (2.5×) |
| add_u64 | BLS12-381 base | 37.5 | 161 (4.3×) | 62.6 (1.7×) | 248 (6.6×) | 89.5 (2.4×) |
| add_u64 | P-521 base | 12.7 | 21.4 (1.7×) | 11.9 (0.9×) | 19.5 (1.5×) | 27.6 (2.2×) |
| add_uint | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.8 (1.1×) | 4.7 (1.0×) |
| add_uint | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.6 (2.5×) | 9.2 (2.0×) |
| add_uint | BN254 scalar | 73.1 | 224 (3.1×) | 98.2 (1.3×) | 365 (5.0×) | 181 (2.5×) |
| add_uint | secp256k1 base | 8.5 | 17.3 (2.0×) | 12.5 (1.5×) | 16.5 (1.9×) | 23.1 (2.7×) |
| add_uint | P-256 base | 57.8 | 180 (3.1×) | 78.3 (1.4×) | 326 (5.6×) | 149 (2.6×) |
| add_uint | P-256 scalar | 78.1 | 259 (3.3×) | 106 (1.4×) | 310 (4.0×) | 190 (2.4×) |
| add_uint | Tom-256 base | 75.2 | 253 (3.4×) | 104 (1.4×) | 326 (4.3×) | 181 (2.4×) |
| add_uint | BLS12-381 base | 151 | 761 (5.0×) | 214 (1.4×) | 914 (6.1×) | 383 (2.5×) |
| add_uint | P-521 base | 23.6 | 61.5 (2.6×) | 41.5 (1.8×) | 86.4 (3.7×) | 66.7 (2.8×) |
| sub_u64 | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_u64 | BabyBear | 7.0 | 12.0 (1.7×) | 8.2 (1.2×) | 12.0 (1.7×) | 11.2 (1.6×) |
| sub_u64 | Goldilocks | 4.2 | 8.1 (1.9×) | 6.1 (1.5×) | 10.5 (2.5×) | 8.7 (2.1×) |
| sub_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 45.7 (1.8×) | 135 (5.4×) | 67.3 (2.7×) |
| sub_u64 | secp256k1 base | 6.9 | 7.0 (1.0×) | 6.1 (0.9×) | 7.0 (1.0×) | 9.0 (1.3×) |
| sub_u64 | P-256 base | 21.7 | 69.4 (3.2×) | 34.8 (1.6×) | 78.2 (3.6×) | 55.8 (2.6×) |
| sub_u64 | P-256 scalar | 23.7 | 66.8 (2.8×) | 37.6 (1.6×) | 71.6 (3.0×) | 64.8 (2.7×) |
| sub_u64 | Tom-256 base | 25.0 | 85.0 (3.4×) | 39.2 (1.6×) | 97.1 (3.9×) | 65.7 (2.6×) |
| sub_u64 | BLS12-381 base | 38.6 | 153 (4.0×) | 62.3 (1.6×) | 263 (6.8×) | 91.9 (2.4×) |
| sub_u64 | P-521 base | 11.0 | 21.1 (1.9×) | 15.0 (1.4×) | 22.2 (2.0×) | 22.0 (2.0×) |
| sub_uint | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_uint | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 12.0 (1.7×) | 11.2 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.1 (1.9×) | 6.1 (1.5×) | 10.5 (2.5×) | 8.7 (2.1×) |
| sub_uint | BN254 scalar | 73.1 | 228 (3.1×) | 97.2 (1.3×) | 352 (4.8×) | 181 (2.5×) |
| sub_uint | secp256k1 base | 6.8 | 18.9 (2.8×) | 12.4 (1.8×) | 16.7 (2.5×) | 21.9 (3.2×) |
| sub_uint | P-256 base | 56.8 | 174 (3.1×) | 79.9 (1.4×) | 245 (4.3×) | 136 (2.4×) |
| sub_uint | P-256 scalar | 77.7 | 253 (3.3×) | 103 (1.3×) | 330 (4.2×) | 186 (2.4×) |
| sub_uint | Tom-256 base | 72.6 | 253 (3.5×) | 98.3 (1.4×) | 329 (4.5×) | 176 (2.4×) |
| sub_uint | BLS12-381 base | 155 | 756 (4.9×) | 203 (1.3×) | 923 (6.0×) | 374 (2.4×) |
| sub_uint | P-521 base | 15.5 | 66.1 (4.3×) | 38.9 (2.5×) | 76.2 (4.9×) | 67.2 (4.3×) |
| div_u64 | BN254 scalar | 604 | 1,209 (2.0×) | 842 (1.4×) | 1,372 (2.3×) | 1,105 (1.8×) |
| div_u64 | secp256k1 base | 566 | 1,015 (1.8×) | 722 (1.3×) | 1,206 (2.1×) | 810 (1.4×) |
| div_u64 | P-256 base | 603 | 1,083 (1.8×) | 801 (1.3×) | 1,338 (2.2×) | 987 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,181 (1.9×) | 856 (1.4×) | 1,436 (2.3×) | 1,130 (1.8×) |
| div_u64 | Tom-256 base | 609 | 1,201 (2.0×) | 858 (1.4×) | 1,465 (2.4×) | 1,113 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,790 (2.4×) | 1,086 (1.4×) | 2,460 (3.3×) | 1,756 (2.3×) |
| div_u64 | P-521 base | 924 | 2,442 (2.6×) | 1,014 (1.1×) | 3,618 (3.9×) | 1,524 (1.6×) |
| div_uint | Mersenne-31 | 217 | 274 (1.3×) | 274 (1.3×) | 278 (1.3×) | 281 (1.3×) |
| div_uint | BabyBear | 217 | 275 (1.3×) | 277 (1.3×) | 277 (1.3×) | 281 (1.3×) |
| div_uint | Goldilocks | 435 | 549 (1.3×) | 539 (1.2×) | 585 (1.3×) | 569 (1.3×) |
| div_uint | BN254 scalar | 4,982 | 9,078 (1.8×) | 6,600 (1.3×) | 12,765 (2.6×) | 8,344 (1.7×) |
| div_uint | secp256k1 base | 4,882 | 8,569 (1.8×) | 6,001 (1.2×) | 12,029 (2.5×) | 8,123 (1.7×) |
| div_uint | P-256 base | 5,018 | 8,760 (1.7×) | 6,175 (1.2×) | 11,942 (2.4×) | 7,931 (1.6×) |
| div_uint | P-256 scalar | 4,943 | 9,129 (1.8×) | 6,596 (1.3×) | 12,456 (2.5×) | 8,622 (1.7×) |
| div_uint | Tom-256 base | 5,051 | 8,877 (1.8×) | 6,449 (1.3×) | 12,494 (2.5×) | 8,415 (1.7×) |
| div_uint | BLS12-381 base | 9,014 | 20,855 (2.3×) | 12,241 (1.4×) | 26,355 (2.9×) | 17,092 (1.9×) |
| div_uint | P-521 base | 14,980 | 34,495 (2.3×) | 18,119 (1.2×) | 49,309 (3.3×) | 30,112 (2.0×) |
| add, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.7 | 25.1 (3.7×) | 19.5 (2.9×) | 23.1 (3.4×) | 25.8 (3.9×) |
| add, constant time | secp256k1 base | 6.9 | 28.3 (4.1×) | 22.6 (3.3×) | 27.0 (3.9×) | 29.8 (4.3×) |
| add, constant time | P-256 base | 7.0 | 24.6 (3.5×) | 22.8 (3.3×) | 25.0 (3.6×) | 27.7 (4.0×) |
| add, constant time | P-256 scalar | 6.8 | 27.7 (4.1×) | 23.8 (3.5×) | 26.8 (3.9×) | 30.1 (4.4×) |
| add, constant time | Tom-256 base | 7.0 | 27.2 (3.9×) | 23.8 (3.4×) | 26.9 (3.8×) | 29.1 (4.2×) |
| add, constant time | BLS12-381 base | 8.9 | 41.4 (4.7×) | 28.6 (3.2×) | 35.3 (4.0×) | 41.1 (4.6×) |
| add, constant time | P-521 base | 15.2 | 76.4 (5.0×) | 47.2 (3.1×) | 81.4 (5.4×) | 74.1 (4.9×) |
| sub, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 6.8 | 21.5 (3.2×) | 16.5 (2.4×) | 20.8 (3.1×) | 22.0 (3.2×) |
| sub, constant time | secp256k1 base | 8.0 | 21.4 (2.7×) | 15.5 (1.9×) | 19.6 (2.5×) | 22.0 (2.8×) |
| sub, constant time | P-256 base | 6.7 | 19.1 (2.9×) | 15.5 (2.3×) | 18.5 (2.8×) | 20.0 (3.0×) |
| sub, constant time | P-256 scalar | 6.8 | 21.6 (3.2×) | 15.9 (2.3×) | 18.5 (2.7×) | 21.5 (3.2×) |
| sub, constant time | Tom-256 base | 8.3 | 21.4 (2.6×) | 17.1 (2.1×) | 19.8 (2.4×) | 23.0 (2.8×) |
| sub, constant time | BLS12-381 base | 9.6 | 37.5 (3.9×) | 24.1 (2.5×) | 36.9 (3.8×) | 36.1 (3.8×) |
| sub, constant time | P-521 base | 12.3 | 57.4 (4.7×) | 37.2 (3.0×) | 59.8 (4.9×) | 53.1 (4.3×) |
| mul, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul, constant time | Goldilocks | 8.9 | 16.5 (1.9×) | 12.5 (1.4×) | 18.0 (2.0×) | 12.3 (1.4×) |
| mul, constant time | BN254 scalar | 78.8 | 244 (3.1×) | 93.0 (1.2×) | 413 (5.2×) | 153 (1.9×) |
| mul, constant time | secp256k1 base | 55.0 | 172 (3.1×) | 77.5 (1.4×) | 242 (4.4×) | 105 (1.9×) |
| mul, constant time | P-256 base | 52.5 | 189 (3.6×) | 72.4 (1.4×) | 292 (5.6×) | 120 (2.3×) |
| mul, constant time | P-256 scalar | 84.0 | 234 (2.8×) | 93.2 (1.1×) | 326 (3.9×) | 167 (2.0×) |
| mul, constant time | Tom-256 base | 76.0 | 237 (3.1×) | 86.7 (1.1×) | 332 (4.4×) | 158 (2.1×) |
| mul, constant time | BLS12-381 base | 154 | 718 (4.7×) | 188 (1.2×) | 916 (5.9×) | 337 (2.2×) |
| mul, constant time | P-521 base | 188 | 687 (3.7×) | 247 (1.3×) | 1,316 (7.0×) | 445 (2.4×) |
| square, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square, constant time | Goldilocks | 9.0 | 15.0 (1.7×) | 11.0 (1.2×) | 18.5 (2.1×) | 12.7 (1.4×) |
| square, constant time | BN254 scalar | 66.9 | 219 (3.3×) | 85.2 (1.3×) | 199 (3.0×) | 140 (2.1×) |
| square, constant time | secp256k1 base | 42.9 | 151 (3.5×) | 67.1 (1.6×) | 181 (4.2×) | 89.4 (2.1×) |
| square, constant time | P-256 base | 44.0 | 160 (3.6×) | 57.1 (1.3×) | 234 (5.3×) | 102 (2.3×) |
| square, constant time | P-256 scalar | 72.9 | 217 (3.0×) | 88.2 (1.2×) | 249 (3.4×) | 132 (1.8×) |
| square, constant time | Tom-256 base | 64.5 | 213 (3.3×) | 82.6 (1.3×) | 218 (3.4×) | 136 (2.1×) |
| square, constant time | BLS12-381 base | 145 | 563 (3.9×) | 168 (1.2×) | 789 (5.4×) | 295 (2.0×) |
| square, constant time | P-521 base | 129 | 571 (4.4×) | 221 (1.7×) | 909 (7.0×) | 392 (3.0×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64, constant time | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.6 (1.4×) | 13.1 (1.2×) |
| mul_u64, constant time | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 15.2 (1.7×) |
| mul_u64, constant time | BN254 scalar | 30.4 | 109 (3.6×) | 50.9 (1.7×) | 132 (4.3×) | 58.0 (1.9×) |
| mul_u64, constant time | secp256k1 base | 19.8 | 68.9 (3.5×) | 39.0 (2.0×) | 85.1 (4.3×) | 45.5 (2.3×) |
| mul_u64, constant time | P-256 base | 33.8 | 89.4 (2.6×) | 53.2 (1.6×) | 103 (3.0×) | 58.7 (1.7×) |
| mul_u64, constant time | P-256 scalar | 33.9 | 105 (3.1×) | 55.0 (1.6×) | 121 (3.6×) | 68.5 (2.0×) |
| mul_u64, constant time | Tom-256 base | 32.9 | 100 (3.0×) | 52.0 (1.6×) | 121 (3.7×) | 60.8 (1.8×) |
| mul_u64, constant time | BLS12-381 base | 41.6 | 155 (3.7×) | 70.6 (1.7×) | 184 (4.4×) | 89.4 (2.1×) |
| mul_u64, constant time | P-521 base | 45.7 | 139 (3.0×) | 78.6 (1.7×) | 170 (3.7×) | 95.1 (2.1×) |
| mul_uint, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint, constant time | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.8 (1.4×) | 13.0 (1.2×) |
| mul_uint, constant time | Goldilocks | 9.0 | 17.6 (2.0×) | 14.1 (1.6×) | 25.3 (2.8×) | 15.2 (1.7×) |
| mul_uint, constant time | BN254 scalar | 141 | 484 (3.4×) | 168 (1.2×) | 597 (4.2×) | 283 (2.0×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 318 (3.8×) | 76.7 (0.9×) | 376 (4.5×) | 122 (1.5×) |
| mul_uint, constant time | P-256 base | 134 | 407 (3.0×) | 133 (1.0×) | 487 (3.6×) | 220 (1.6×) |
| mul_uint, constant time | P-256 scalar | 146 | 476 (3.3×) | 177 (1.2×) | 544 (3.7×) | 292 (2.0×) |
| mul_uint, constant time | Tom-256 base | 135 | 467 (3.5×) | 172 (1.3×) | 553 (4.1×) | 320 (2.4×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,075 (4.0×) | 360 (1.4×) | 1,689 (6.3×) | 900 (3.4×) |
| mul_uint, constant time | P-521 base | 472 | 2,018 (4.3×) | 303 (0.6×) | 2,913 (6.2×) | 530 (1.1×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) |
| add_u64, constant time | BabyBear | 4.9 | 11.7 (2.4×) | 8.5 (1.7×) | 13.0 (2.7×) | 10.8 (2.2×) |
| add_u64, constant time | Goldilocks | 5.0 | 8.5 (1.7×) | 6.5 (1.3×) | 11.8 (2.4×) | 9.2 (1.8×) |
| add_u64, constant time | BN254 scalar | 21.9 | 93.2 (4.3×) | 44.6 (2.0×) | 108 (4.9×) | 69.6 (3.2×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.4 (3.4×) | 23.0 (3.1×) | 25.0 (3.4×) | 29.0 (3.9×) |
| add_u64, constant time | P-256 base | 17.1 | 66.4 (3.9×) | 42.6 (2.5×) | 80.1 (4.7×) | 60.7 (3.5×) |
| add_u64, constant time | P-256 scalar | 24.8 | 103 (4.2×) | 49.4 (2.0×) | 109 (4.4×) | 75.1 (3.0×) |
| add_u64, constant time | Tom-256 base | 22.3 | 97.6 (4.4×) | 47.2 (2.1×) | 109 (4.9×) | 70.6 (3.2×) |
| add_u64, constant time | BLS12-381 base | 35.8 | 136 (3.8×) | 69.2 (1.9×) | 163 (4.6×) | 110 (3.1×) |
| add_u64, constant time | P-521 base | 12.9 | 65.9 (5.1×) | 45.0 (3.5×) | 69.6 (5.4×) | 64.8 (5.0×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) |
| add_uint, constant time | BabyBear | 4.9 | 11.6 (2.4×) | 8.1 (1.7×) | 13.0 (2.7×) | 10.8 (2.2×) |
| add_uint, constant time | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.8 (2.5×) | 9.3 (2.0×) |
| add_uint, constant time | BN254 scalar | 74.8 | 238 (3.2×) | 97.8 (1.3×) | 367 (4.9×) | 181 (2.4×) |
| add_uint, constant time | secp256k1 base | 8.8 | 38.5 (4.4×) | 29.3 (3.3×) | 36.3 (4.1×) | 44.5 (5.1×) |
| add_uint, constant time | P-256 base | 57.8 | 179 (3.1×) | 83.7 (1.4×) | 276 (4.8×) | 144 (2.5×) |
| add_uint, constant time | P-256 scalar | 78.8 | 261 (3.3×) | 106 (1.3×) | 317 (4.0×) | 178 (2.3×) |
| add_uint, constant time | Tom-256 base | 75.1 | 253 (3.4×) | 103 (1.4×) | 322 (4.3×) | 177 (2.4×) |
| add_uint, constant time | BLS12-381 base | 154 | 762 (4.9×) | 206 (1.3×) | 909 (5.9×) | 384 (2.5×) |
| add_uint, constant time | P-521 base | 22.6 | 138 (6.1×) | 93.0 (4.1×) | 142 (6.3×) | 134 (5.9×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_u64, constant time | BabyBear | 4.8 | 12.1 (2.5×) | 8.2 (1.7×) | 13.0 (2.7×) | 11.6 (2.4×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.7 (2.1×) |
| sub_u64, constant time | BN254 scalar | 21.9 | 95.1 (4.3×) | 41.5 (1.9×) | 107 (4.9×) | 65.7 (3.0×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.0 (2.6×) | 15.2 (2.2×) | 17.0 (2.5×) | 17.4 (2.5×) |
| sub_u64, constant time | P-256 base | 16.1 | 61.4 (3.8×) | 36.0 (2.2×) | 74.6 (4.6×) | 54.9 (3.4×) |
| sub_u64, constant time | P-256 scalar | 22.7 | 96.1 (4.2×) | 45.1 (2.0×) | 107 (4.7×) | 70.7 (3.1×) |
| sub_u64, constant time | Tom-256 base | 23.9 | 92.0 (3.8×) | 43.1 (1.8×) | 105 (4.4×) | 69.3 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 35.1 | 138 (3.9×) | 66.6 (1.9×) | 172 (4.9×) | 108 (3.1×) |
| sub_u64, constant time | P-521 base | 11.0 | 45.4 (4.1×) | 35.0 (3.2×) | 49.8 (4.5×) | 46.3 (4.2×) |
| sub_uint, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_uint, constant time | BabyBear | 4.8 | 11.9 (2.5×) | 8.2 (1.7×) | 13.0 (2.7×) | 11.6 (2.4×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.7 (2.1×) |
| sub_uint, constant time | BN254 scalar | 74.1 | 241 (3.3×) | 97.3 (1.3×) | 287 (3.9×) | 176 (2.4×) |
| sub_uint, constant time | secp256k1 base | 7.8 | 32.3 (4.1×) | 24.9 (3.2×) | 32.4 (4.2×) | 37.5 (4.8×) |
| sub_uint, constant time | P-256 base | 56.8 | 176 (3.1×) | 78.3 (1.4×) | 274 (4.8×) | 135 (2.4×) |
| sub_uint, constant time | P-256 scalar | 76.9 | 260 (3.4×) | 102 (1.3×) | 330 (4.3×) | 177 (2.3×) |
| sub_uint, constant time | Tom-256 base | 72.8 | 248 (3.4×) | 97.9 (1.3×) | 329 (4.5×) | 174 (2.4×) |
| sub_uint, constant time | BLS12-381 base | 156 | 760 (4.9×) | 203 (1.3×) | 907 (5.8×) | 381 (2.4×) |
| sub_uint, constant time | P-521 base | 19.5 | 123 (6.3×) | 85.2 (4.4×) | 122 (6.3×) | 113 (5.8×) |
| div_u64, constant time | BN254 scalar | 6,843 | 12,396 (1.8×) | 9,401 (1.4×) | 17,470 (2.6×) | 13,319 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,780 | 11,997 (1.8×) | 9,060 (1.3×) | 16,768 (2.5×) | 12,861 (1.9×) |
| div_u64, constant time | P-256 base | 6,654 | 11,860 (1.8×) | 9,125 (1.4×) | 16,959 (2.5×) | 12,446 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,895 | 12,343 (1.8×) | 9,428 (1.4×) | 17,090 (2.5×) | 13,076 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,274 | 12,244 (1.7×) | 9,283 (1.3×) | 16,830 (2.3×) | 12,824 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,367 | 24,766 (2.0×) | 16,683 (1.3×) | 34,547 (2.8×) | 24,288 (2.0×) |
| div_u64, constant time | P-521 base | 20,375 | 43,059 (2.1×) | 26,100 (1.3×) | 64,242 (3.2×) | 42,893 (2.1×) |
| div_uint, constant time | Mersenne-31 | 250 | 275 (1.1×) | 275 (1.1×) | 274 (1.1×) | 274 (1.1×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 435 (1.1×) | 470 (1.2×) | 441 (1.1×) |
| div_uint, constant time | Goldilocks | 665 | 1,138 (1.7×) | 859 (1.3×) | 1,431 (2.2×) | 953 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,894 | 12,409 (1.8×) | 9,411 (1.4×) | 17,816 (2.6×) | 13,425 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,779 | 11,952 (1.8×) | 9,069 (1.3×) | 16,832 (2.5×) | 12,787 (1.9×) |
| div_uint, constant time | P-256 base | 6,690 | 11,892 (1.8×) | 9,122 (1.4×) | 17,216 (2.6×) | 12,673 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,912 | 12,530 (1.8×) | 9,421 (1.4×) | 17,412 (2.5×) | 13,365 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,335 | 12,429 (1.7×) | 9,474 (1.3×) | 17,036 (2.3×) | 13,052 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,516 | 25,397 (2.0×) | 17,062 (1.4×) | 35,191 (2.8×) | 24,664 (2.0×) |
| div_uint, constant time | P-521 base | 20,375 | 42,915 (2.1×) | 25,999 (1.3×) | 63,827 (3.1×) | 42,935 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
