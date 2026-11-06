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
| Cost of a count, in WebAssembly, Chrome | 10,005 to 15,298 cycles and 3,081 instructions; its noise in a figure, 788 to 3,909 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,148 to 7,862 cycles and 3,081 instructions; its noise in a figure, 53.0 to 458 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 7,196 to 10,270 cycles and 1,885 instructions; its noise in a figure, 793 to 2,389 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,703 to 14,690 cycles and 1,885 instructions; its noise in a figure, 135 to 2,178 cycles |
| Load, in WebAssembly, Chrome | CPU 12; 1-minute load 16.18 before, 20.59 after; its SMT sibling, CPU 28, 54% busy |
| Load, in WebAssembly with wide arithmetic, Chrome | CPU 12; 1-minute load 22.61 before, 26.52 after; its SMT sibling, CPU 28, 20% busy |
| Load, in WebAssembly, Firefox | CPU 12; 1-minute load 25.68 before, 19.49 after; its SMT sibling, CPU 28, 20% busy |
| Load, in WebAssembly with wide arithmetic, Firefox | CPU 12; 1-minute load 19.49 before, 21.5 after; its SMT sibling, CPU 28, 14% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs crypto-bigint 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add | BabyBear | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 2.4 vs plonky3 2.4 | +0.0 (1.00×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) | 3.0 vs plonky3 2.7 | +0.3 (1.11×) |
| add | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add | BN254 scalar | 14.0 vs ark-ff 13.8 | +0.2 (1.01×) | 12.4 vs ark-ff 13.2 | −0.8 (0.94×) | 14.5 vs ark-ff 14.2 | +0.3 (1.02×) | 19.0 vs ark-ff 18.0 | +1.0 (1.06×) |
| add | secp256k1 base | 14.0 vs ark-ff 15.4 | −1.4 (0.91×) | 11.8 vs ark-ff 12.8 | −1.0 (0.92×) | 14.8 vs ark-ff 17.1 | −2.3 (0.87×) | 20.2 vs ark-ff 19.5 | +0.7 (1.04×) |
| add | P-256 base | 14.4 vs ark-ff 15.8 | −1.4 (0.91×) | 12.3 vs ark-ff 12.2 | +0.1 (1.01×) | 14.5 vs ark-ff 17.4 | −2.9 (0.83×) | 20.0 vs ark-ff 19.9 | +0.1 (1.01×) |
| add | P-256 scalar | 14.5 vs ark-ff 15.3 | −0.8 (0.95×) | 11.7 vs ark-ff 13.4 | −1.7 (0.87×) | 15.5 vs ark-ff 17.3 | −1.8 (0.90×) | 20.5 vs ark-ff 19.4 | +1.1 (1.06×) |
| add | Tom-256 base | 13.5 vs halo2curves 35.7 | −22.2 (0.38×) | 11.8 vs halo2curves 20.2 | −8.4 (0.58×) | 14.7 vs halo2curves 38.8 | −24.1 (0.38×) | 20.4 vs halo2curves 34.1 | −13.7 (0.60×) |
| add | BLS12-381 base | 21.9 vs ark-ff 35.1 | −13.2 (0.62×) | 18.0 vs ark-ff 30.0 | −12.0 (0.60×) | 22.5 vs ark-ff 38.9 | −16.4 (0.58×) | 29.2 vs ark-ff 42.6 | −13.4 (0.69×) |
| add | P-521 base | 42.6 vs crypto-bigint 147 | −104 (0.29×) | 26.7 vs crypto-bigint 136 | −109 (0.20×) | 42.7 vs crypto-bigint 193 | −150 (0.22×) | 44.8 vs crypto-bigint 187 | −142 (0.24×) |
| sub | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub | BabyBear | 2.5 vs plonky3 2.8 | −0.3 (0.89×) | 2.6 vs plonky3 2.8 | −0.2 (0.93×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) | 2.7 vs plonky3 3.0 | −0.3 (0.90×) |
| sub | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 15.1 vs ark-ff 18.5 | −3.4 (0.82×) | 11.8 vs halo2curves 15.0 | −3.2 (0.79×) | 14.5 vs ark-ff 19.1 | −4.6 (0.76×) | 19.4 vs ark-ff 23.8 | −4.4 (0.82×) |
| sub | secp256k1 base | 16.5 vs ark-ff 18.6 | −2.1 (0.89×) | 11.8 vs halo2curves 14.3 | −2.5 (0.83×) | 15.7 vs ark-ff 19.6 | −3.9 (0.80×) | 18.7 vs halo2curves 22.1 | −3.4 (0.85×) |
| sub | P-256 base | 14.3 vs ark-ff 16.9 | −2.6 (0.85×) | 11.8 vs halo2curves 12.3 | −0.5 (0.96×) | 14.1 vs ark-ff 18.3 | −4.2 (0.77×) | 18.2 vs halo2curves 21.5 | −3.3 (0.85×) |
| sub | P-256 scalar | 16.0 vs ark-ff 18.0 | −2.0 (0.89×) | 11.8 vs halo2curves 14.4 | −2.6 (0.82×) | 15.0 vs ark-ff 18.8 | −3.8 (0.80×) | 19.2 vs halo2curves 23.1 | −3.9 (0.83×) |
| sub | Tom-256 base | 15.0 vs halo2curves 24.0 | −9.0 (0.62×) | 11.8 vs halo2curves 14.9 | −3.1 (0.79×) | 14.6 vs halo2curves 23.4 | −8.8 (0.62×) | 19.5 vs halo2curves 24.1 | −4.6 (0.81×) |
| sub | BLS12-381 base | 23.0 vs ark-ff 33.4 | −10.4 (0.69×) | 17.1 vs halo2curves 23.6 | −6.5 (0.72×) | 21.8 vs ark-ff 31.6 | −9.8 (0.69×) | 28.5 vs halo2curves 37.1 | −8.6 (0.77×) |
| sub | P-521 base | 47.2 vs crypto-bigint 88.8 | −41.6 (0.53×) | 26.3 vs crypto-bigint 64.4 | −38.1 (0.41×) | 43.1 vs crypto-bigint 121 | −77.9 (0.36×) | 43.3 vs crypto-bigint 128 | −84.7 (0.34×) |
| mul | Mersenne-31 | 3.7 vs crypto-bigint 29.4 | −25.7 (0.13×) | 3.6 vs crypto-bigint 15.8 | −12.2 (0.23×) | 4.0 vs crypto-bigint 38.9 | −34.9 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul | BabyBear | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 4.1 | +1.1 (1.27×) | 5.2 vs plonky3 6.6 | −1.4 (0.79×) | 5.0 vs plonky3 6.5 | −1.5 (0.77×) |
| mul | Goldilocks | 10.0 vs winter-math 12.2 | −2.2 (0.82×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 15.0 vs winter-math 17.0 | −2.0 (0.88×) | 7.0 vs winter-math 7.7 | −0.7 (0.91×) |
| mul | BN254 scalar | 233 vs ark-ff 322 | −89.0 (0.72×) | 79.7 vs halo2curves 91.8 | −12.1 (0.87×) | 387 vs ark-ff 398 | −11.0 (0.97×) | 149 vs halo2curves 182 | −33.0 (0.82×) |
| mul | secp256k1 base | 162 vs ark-ff 282 | −120 (0.57×) | 59.6 vs halo2curves 91.1 | −31.5 (0.65×) | 237 vs ark-ff 418 | −181 (0.57×) | 107 vs halo2curves 163 | −56.0 (0.66×) |
| mul | P-256 base | 182 vs k256/p256 244 | −62.0 (0.75×) | 72.0 vs k256/p256 85.0 | −13.0 (0.85×) | 278 vs ark-ff 449 | −171 (0.62×) | 119 vs halo2curves 135 | −16.0 (0.88×) |
| mul | P-256 scalar | 247 vs ark-ff 324 | −77.0 (0.76×) | 84.8 vs halo2curves 93.9 | −9.1 (0.90×) | 329 vs ark-ff 420 | −91.0 (0.78×) | 167 vs halo2curves 181 | −14.0 (0.92×) |
| mul | Tom-256 base | 239 vs halo2curves 345 | −106 (0.69×) | 74.8 vs halo2curves 94.1 | −19.3 (0.79×) | 314 vs halo2curves 624 | −310 (0.50×) | 163 vs halo2curves 176 | −13.0 (0.93×) |
| mul | BLS12-381 base | 726 vs ark-ff 816 | −90.0 (0.89×) | 181 vs halo2curves 195 | −14.0 (0.93×) | 930 vs ark-ff 1,076 | −146 (0.86×) | 322 vs halo2curves 382 | −60.0 (0.84×) |
| mul | P-521 base | 667 vs crypto-bigint 2,123 | −1,456 (0.31×) | 244 vs crypto-bigint 601 | −357 (0.41×) | 1,273 vs crypto-bigint 2,712 | −1,439 (0.47×) | 534 vs crypto-bigint 1,053 | −519 (0.51×) |
| square | Mersenne-31 | 3.5 vs crypto-bigint 28.7 | −25.2 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 38.3 | −34.7 (0.09×) | 3.6 vs crypto-bigint 18.1 | −14.5 (0.20×) |
| square | BabyBear | 4.7 vs plonky3 5.3 | −0.6 (0.89×) | 4.8 vs plonky3 5.2 | −0.4 (0.92×) | 5.7 vs plonky3 5.7 | +0.0 (1.00×) | 5.8 vs plonky3 5.7 | +0.1 (1.02×) |
| square | Goldilocks | 8.7 vs winter-math 12.5 | −3.8 (0.70×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.0 vs winter-math 7.0 | +1.0 (1.14×) |
| square | BN254 scalar | 217 vs ark-ff 271 | −54.0 (0.80×) | 78.4 vs halo2curves 84.8 | −6.4 (0.92×) | 348 vs ark-ff 358 | −10.0 (0.97×) | 142 vs halo2curves 144 | −2.0 (0.99×) |
| square | secp256k1 base | 145 vs ark-ff 232 | −87.0 (0.62×) | 53.0 vs halo2curves 78.6 | −25.6 (0.67×) | 202 vs ark-ff 306 | −104 (0.66×) | 92.0 vs halo2curves 123 | −31.0 (0.75×) |
| square | P-256 base | 160 vs halo2curves 188 | −28.0 (0.85×) | 74.3 vs k256/p256 73.1 | +1.2 (1.02×) | 262 vs halo2curves 334 | −72.0 (0.78×) | 123 vs halo2curves 114 | +9.0 (1.08×) |
| square | P-256 scalar | 234 vs ark-ff 266 | −32.0 (0.88×) | 75.8 vs halo2curves 83.1 | −7.3 (0.91×) | 269 vs ark-ff 351 | −82.0 (0.77×) | 151 vs halo2curves 141 | +10.0 (1.07×) |
| square | Tom-256 base | 233 vs halo2curves 312 | −79.0 (0.75×) | 72.4 vs halo2curves 78.0 | −5.6 (0.93×) | 282 vs halo2curves 474 | −192 (0.59×) | 160 vs halo2curves 136 | +24.0 (1.18×) |
| square | BLS12-381 base | 567 vs ark-ff 635 | −68.0 (0.89×) | 158 vs halo2curves 163 | −5.0 (0.97×) | 828 vs ark-ff 831 | −3.0 (1.00×) | 309 vs halo2curves 309 | +0.0 (1.00×) |
| square | P-521 base | 567 vs crypto-bigint 2,163 | −1,596 (0.26×) | 238 vs crypto-bigint 616 | −378 (0.39×) | 914 vs crypto-bigint 2,837 | −1,923 (0.32×) | 398 vs crypto-bigint 1,124 | −726 (0.35×) |
| invert | Mersenne-31 | 267 vs plonky3 672 | −405 (0.40×) | 267 vs plonky3 676 | −409 (0.39×) | 269 vs plonky3 750 | −481 (0.36×) | 268 vs plonky3 739 | −471 (0.36×) |
| invert | BabyBear | 269 vs plonky3 706 | −437 (0.38×) | 268 vs plonky3 699 | −431 (0.38×) | 271 vs plonky3 716 | −445 (0.38×) | 271 vs plonky3 720 | −449 (0.38×) |
| invert | Goldilocks | 539 vs winter-math 1,748 | −1,209 (0.31×) | 540 vs winter-math 912 | −372 (0.59×) | 564 vs winter-math 1,834 | −1,270 (0.31×) | 567 vs winter-math 998 | −431 (0.57×) |
| invert | BN254 scalar | 8,523 vs halo2curves 14,300 | −5,777 (0.60×) | 6,468 vs halo2curves 8,008 | −1,540 (0.81×) | 11,802 vs halo2curves 21,416 | −9,614 (0.55×) | 7,853 vs halo2curves 13,807 | −5,954 (0.57×) |
| invert | secp256k1 base | 8,497 vs halo2curves 14,304 | −5,807 (0.59×) | 6,045 vs halo2curves 8,052 | −2,007 (0.75×) | 11,550 vs halo2curves 21,470 | −9,920 (0.54×) | 7,990 vs halo2curves 14,084 | −6,094 (0.57×) |
| invert | P-256 base | 8,309 vs halo2curves 14,323 | −6,014 (0.58×) | 6,072 vs halo2curves 8,031 | −1,959 (0.76×) | 11,486 vs halo2curves 21,615 | −10,129 (0.53×) | 7,614 vs halo2curves 13,877 | −6,263 (0.55×) |
| invert | P-256 scalar | 8,470 vs halo2curves 14,291 | −5,821 (0.59×) | 6,382 vs halo2curves 8,038 | −1,656 (0.79×) | 11,866 vs halo2curves 21,471 | −9,605 (0.55×) | 8,190 vs halo2curves 14,021 | −5,831 (0.58×) |
| invert | Tom-256 base | 8,522 vs halo2curves 14,324 | −5,802 (0.59×) | 6,464 vs halo2curves 7,996 | −1,532 (0.81×) | 11,670 vs halo2curves 21,701 | −10,031 (0.54×) | 8,034 vs halo2curves 14,113 | −6,079 (0.57×) |
| invert | BLS12-381 base | 19,535 vs halo2curves 25,592 | −6,057 (0.76×) | 11,520 vs halo2curves 13,089 | −1,569 (0.88×) | 24,270 vs halo2curves 37,394 | −13,124 (0.65×) | 15,715 vs halo2curves 23,468 | −7,753 (0.67×) |
| invert | P-521 base | 33,389 vs crypto-bigint 64,940 | −31,551 (0.51×) | 18,109 vs crypto-bigint 50,387 | −32,278 (0.36×) | 48,629 vs crypto-bigint 80,215 | −31,586 (0.61×) | 29,193 vs crypto-bigint 62,198 | −33,005 (0.47×) |
| sqrt | Mersenne-31 | 159 vs plonky3 324 | −165 (0.49×) | 159 vs plonky3 324 | −165 (0.49×) | 160 vs plonky3 329 | −169 (0.49×) | 160 vs plonky3 329 | −169 (0.49×) |
| sqrt | BabyBear | 2,104 vs plonky3 3,110 | −1,006 (0.68×) | 2,058 vs plonky3 3,100 | −1,042 (0.66×) | 2,142 vs plonky3 2,995 | −853 (0.72×) | 2,173 vs plonky3 2,990 | −817 (0.73×) |
| sqrt | Goldilocks | 4,834 vs crypto-bigint 48,577 | −43,743 (0.10×) | 3,485 vs crypto-bigint 28,840 | −25,355 (0.12×) | 6,077 vs crypto-bigint 54,379 | −48,302 (0.11×) | 5,043 vs crypto-bigint 32,894 | −27,851 (0.15×) |
| sqrt | BN254 scalar | 115,714 vs ark-ff 152,799 | −37,085 (0.76×) | 52,938 vs halo2curves 102,662 | −49,724 (0.52×) | 176,779 vs ark-ff 208,083 | −31,304 (0.85×) | 95,027 vs ark-ff 151,628 | −56,601 (0.63×) |
| sqrt | secp256k1 base | 40,862 vs ark-ff 119,334 | −78,472 (0.34×) | 17,817 vs halo2curves 43,325 | −25,508 (0.41×) | 66,119 vs ark-ff 175,641 | −109,522 (0.38×) | 27,371 vs crypto-bigint 81,855 | −54,484 (0.33×) |
| sqrt | P-256 base | 43,265 vs ark-ff 60,305 | −17,040 (0.72×) | 18,396 vs k256/p256 21,667 | −3,271 (0.85×) | 78,732 vs ark-ff 97,551 | −18,819 (0.81×) | 30,386 vs k256/p256 33,586 | −3,200 (0.90×) |
| sqrt | P-256 scalar | 74,437 vs ark-ff 122,376 | −47,939 (0.61×) | 33,335 vs halo2curves 49,381 | −16,046 (0.68×) | 113,724 vs ark-ff 159,393 | −45,669 (0.71×) | 63,482 vs halo2curves 72,405 | −8,923 (0.88×) |
| sqrt | Tom-256 base | 65,944 vs crypto-bigint 148,892 | −82,948 (0.44×) | 18,298 vs halo2curves 45,656 | −27,358 (0.40×) | 71,968 vs crypto-bigint 225,627 | −153,659 (0.32×) | 51,996 vs crypto-bigint 76,147 | −24,151 (0.68×) |
| sqrt | BLS12-381 base | 296,814 vs ark-ff 421,077 | −124,263 (0.70×) | 93,874 vs crypto-bigint 145,084 | −51,210 (0.65×) | 396,124 vs ark-ff 574,395 | −178,271 (0.69×) | 216,536 vs crypto-bigint 268,366 | −51,830 (0.81×) |
| sqrt | P-521 base | 319,421 vs crypto-bigint 1,534,942 | −1,215,521 (0.21×) | 125,421 vs crypto-bigint 296,458 | −171,037 (0.42×) | 474,076 vs crypto-bigint 1,563,666 | −1,089,590 (0.30×) | 256,633 vs crypto-bigint 591,904 | −335,271 (0.43×) |
| mul_u64 | Mersenne-31 | 6.7 vs crypto-bigint 61.5 | −54.8 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 72.6 | −65.0 (0.10×) | 7.6 vs crypto-bigint 38.7 | −31.1 (0.20×) |
| mul_u64 | BabyBear | 13.0 vs plonky3 35.2 | −22.2 (0.37×) | 11.2 vs plonky3 11.4 | −0.2 (0.98×) | 18.0 vs plonky3 46.1 | −28.1 (0.39×) | 14.7 vs plonky3 17.8 | −3.1 (0.83×) |
| mul_u64 | Goldilocks | 17.2 vs winter-math 25.1 | −7.9 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.0 vs winter-math 13.6 | −4.6 (0.66×) |
| mul_u64 | BN254 scalar | 100 vs ark-ff 702 | −602 (0.14×) | 28.8 vs crypto-bigint 263 | −234 (0.11×) | 124 vs ark-ff 815 | −691 (0.15×) | 44.1 vs crypto-bigint 407 | −363 (0.11×) |
| mul_u64 | secp256k1 base | 42.4 vs ark-ff 447 | −405 (0.09×) | 12.6 vs crypto-bigint 263 | −250 (0.05×) | 65.7 vs ark-ff 620 | −554 (0.11×) | 23.0 vs crypto-bigint 408 | −385 (0.06×) |
| mul_u64 | P-256 base | 71.8 vs k256/p256 484 | −412 (0.15×) | 27.3 vs k256/p256 150 | −123 (0.18×) | 94.0 vs ark-ff 861 | −767 (0.11×) | 41.2 vs k256/p256 258 | −217 (0.16×) |
| mul_u64 | P-256 scalar | 84.3 vs k256/p256 521 | −437 (0.16×) | 30.3 vs k256/p256 254 | −224 (0.12×) | 112 vs k256/p256 830 | −718 (0.13×) | 47.4 vs k256/p256 337 | −290 (0.14×) |
| mul_u64 | Tom-256 base | 82.2 vs crypto-bigint 866 | −784 (0.09×) | 30.0 vs crypto-bigint 263 | −233 (0.11×) | 116 vs halo2curves 1,271 | −1,155 (0.09×) | 45.3 vs crypto-bigint 408 | −363 (0.11×) |
| mul_u64 | BLS12-381 base | 131 vs ark-ff 1,654 | −1,523 (0.08×) | 44.2 vs halo2curves 485 | −441 (0.09×) | 175 vs ark-ff 2,143 | −1,968 (0.08×) | 61.5 vs crypto-bigint 1,032 | −971 (0.06×) |
| mul_u64 | P-521 base | 92.2 vs crypto-bigint 3,015 | −2,923 (0.03×) | 31.8 vs crypto-bigint 992 | −960 (0.03×) | 236 vs crypto-bigint 4,010 | −3,774 (0.06×) | 50.2 vs crypto-bigint 1,831 | −1,781 (0.03×) |
| mul_uint | Mersenne-31 | 6.7 vs crypto-bigint 63.0 | −56.3 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 72.8 | −65.2 (0.10×) | 7.6 vs crypto-bigint 38.7 | −31.1 (0.20×) |
| mul_uint | BabyBear | 13.0 vs plonky3 23.9 | −10.9 (0.54×) | 11.2 vs plonky3 11.2 | +0.0 (1.00×) | 18.0 vs plonky3 35.1 | −17.1 (0.51×) | 14.7 vs plonky3 17.9 | −3.2 (0.82×) |
| mul_uint | Goldilocks | 17.3 vs winter-math 25.0 | −7.7 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.0 vs winter-math 13.6 | −4.6 (0.66×) |
| mul_uint | BN254 scalar | 441 vs halo2curves 776 | −335 (0.57×) | 168 vs halo2curves 182 | −14.0 (0.92×) | 534 vs halo2curves 1,356 | −822 (0.39×) | 337 vs halo2curves 349 | −12.0 (0.97×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 519 | −253 (0.51×) | 59.7 vs halo2curves 153 | −93.3 (0.39×) | 323 vs halo2curves 962 | −639 (0.34×) | 109 vs halo2curves 266 | −157 (0.41×) |
| mul_uint | P-256 base | 354 vs halo2curves 454 | −100 (0.78×) | 137 vs halo2curves 143 | −6.0 (0.96×) | 435 vs halo2curves 946 | −511 (0.46×) | 224 vs halo2curves 253 | −29.0 (0.89×) |
| mul_uint | P-256 scalar | 427 vs k256/p256 554 | −127 (0.77×) | 168 vs halo2curves 186 | −18.0 (0.90×) | 474 vs k256/p256 902 | −428 (0.53×) | 307 vs halo2curves 347 | −40.0 (0.88×) |
| mul_uint | Tom-256 base | 412 vs halo2curves 697 | −285 (0.59×) | 155 vs halo2curves 180 | −25.0 (0.86×) | 490 vs halo2curves 1,194 | −704 (0.41×) | 306 vs halo2curves 336 | −30.0 (0.91×) |
| mul_uint | BLS12-381 base | 1,006 vs halo2curves 1,919 | −913 (0.52×) | 365 vs halo2curves 381 | −16.0 (0.96×) | 1,148 vs crypto-bigint 2,570 | −1,422 (0.45×) | 897 vs crypto-bigint 1,037 | −140 (0.86×) |
| mul_uint | P-521 base | 1,539 vs crypto-bigint 3,000 | −1,461 (0.51×) | 257 vs crypto-bigint 990 | −733 (0.26×) | 2,379 vs crypto-bigint 4,047 | −1,668 (0.59×) | 525 vs crypto-bigint 1,841 | −1,316 (0.29×) |
| add_u64 | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.4 vs crypto-bigint 41.4 | −36.0 (0.13×) | 5.4 vs crypto-bigint 30.4 | −25.0 (0.18×) |
| add_u64 | BabyBear | 10.9 vs plonky3 32.3 | −21.4 (0.34×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 44.2 | −31.2 (0.29×) | 12.5 vs plonky3 16.0 | −3.5 (0.78×) |
| add_u64 | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 9.4 vs winter-math 11.0 | −1.6 (0.85×) |
| add_u64 | BN254 scalar | 119 vs ark-ff 363 | −244 (0.33×) | 53.1 vs halo2curves 210 | −157 (0.25×) | 142 vs ark-ff 409 | −267 (0.35×) | 94.6 vs halo2curves 298 | −203 (0.32×) |
| add_u64 | secp256k1 base | 8.1 vs ark-ff 163 | −155 (0.05×) | 8.0 vs ark-ff 121 | −113 (0.07×) | 8.0 vs ark-ff 228 | −220 (0.04×) | 11.0 vs ark-ff 185 | −174 (0.06×) |
| add_u64 | P-256 base | 71.4 vs k256/p256 277 | −206 (0.26×) | 35.7 vs k256/p256 95.2 | −59.5 (0.38×) | 84.7 vs ark-ff 400 | −315 (0.21×) | 68.0 vs k256/p256 159 | −91.0 (0.43×) |
| add_u64 | P-256 scalar | 70.7 vs k256/p256 41.1 | +29.6 (1.72×) | 40.5 vs k256/p256 41.0 | −0.5 (0.99×) | 74.7 vs k256/p256 54.2 | +20.5 (1.38×) | 83.9 vs k256/p256 54.9 | +29.0 (1.53×) |
| add_u64 | Tom-256 base | 88.8 vs crypto-bigint 505 | −416 (0.18×) | 41.4 vs halo2curves 219 | −178 (0.19×) | 108 vs halo2curves 703 | −595 (0.15×) | 71.9 vs halo2curves 291 | −219 (0.25×) |
| add_u64 | BLS12-381 base | 169 vs ark-ff 841 | −672 (0.20×) | 66.7 vs halo2curves 322 | −255 (0.21×) | 256 vs ark-ff 1,094 | −838 (0.23×) | 95.2 vs crypto-bigint 650 | −555 (0.15×) |
| add_u64 | P-521 base | 21.0 vs crypto-bigint 1,688 | −1,667 (0.01×) | 15.4 vs crypto-bigint 648 | −633 (0.02×) | 22.1 vs crypto-bigint 2,205 | −2,183 (0.01×) | 24.1 vs crypto-bigint 1,148 | −1,124 (0.02×) |
| add_uint | Mersenne-31 | 5.2 vs crypto-bigint 36.3 | −31.1 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.4 vs crypto-bigint 41.4 | −36.0 (0.13×) | 5.4 vs crypto-bigint 30.8 | −25.4 (0.18×) |
| add_uint | BabyBear | 10.9 vs plonky3 22.9 | −12.0 (0.48×) | 8.6 vs plonky3 10.0 | −1.4 (0.86×) | 13.0 vs plonky3 34.3 | −21.3 (0.38×) | 12.5 vs plonky3 16.0 | −3.5 (0.78×) |
| add_uint | Goldilocks | 9.3 vs winter-math 13.5 | −4.2 (0.69×) | 7.1 vs winter-math 7.4 | −0.3 (0.96×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 9.4 vs winter-math 11.0 | −1.6 (0.85×) |
| add_uint | BN254 scalar | 246 vs halo2curves 410 | −164 (0.60×) | 97.2 vs halo2curves 113 | −15.8 (0.86×) | 365 vs halo2curves 684 | −319 (0.53×) | 185 vs halo2curves 205 | −20.0 (0.90×) |
| add_uint | secp256k1 base | 20.9 vs halo2curves 222 | −201 (0.09×) | 14.6 vs halo2curves 84.9 | −70.3 (0.17×) | 18.3 vs halo2curves 415 | −397 (0.04×) | 23.8 vs halo2curves 140 | −116 (0.17×) |
| add_uint | P-256 base | 188 vs halo2curves 246 | −58.0 (0.76×) | 74.6 vs halo2curves 91.4 | −16.8 (0.82×) | 326 vs halo2curves 516 | −190 (0.63×) | 147 vs halo2curves 153 | −6.0 (0.96×) |
| add_uint | P-256 scalar | 258 vs k256/p256 95.0 | +163 (2.72×) | 96.1 vs k256/p256 94.7 | +1.4 (1.01×) | 322 vs k256/p256 119 | +203 (2.71×) | 171 vs k256/p256 119 | +52.0 (1.44×) |
| add_uint | Tom-256 base | 251 vs halo2curves 379 | −128 (0.66×) | 99.6 vs halo2curves 110 | −10.4 (0.91×) | 323 vs halo2curves 615 | −292 (0.53×) | 181 vs halo2curves 196 | −15.0 (0.92×) |
| add_uint | BLS12-381 base | 772 vs halo2curves 984 | −212 (0.78×) | 204 vs halo2curves 224 | −20.0 (0.91×) | 949 vs crypto-bigint 1,424 | −475 (0.67×) | 384 vs halo2curves 424 | −40.0 (0.91×) |
| add_uint | P-521 base | 65.2 vs crypto-bigint 1,686 | −1,621 (0.04×) | 42.0 vs crypto-bigint 647 | −605 (0.06×) | 88.4 vs crypto-bigint 2,246 | −2,158 (0.04×) | 75.0 vs crypto-bigint 1,157 | −1,082 (0.06×) |
| sub_u64 | Mersenne-31 | 5.0 vs crypto-bigint 23.4 | −18.4 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_u64 | BabyBear | 11.1 vs crypto-bigint 27.9 | −16.8 (0.40×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs crypto-bigint 42.3 | −29.3 (0.31×) | 12.5 vs plonky3 15.1 | −2.6 (0.83×) |
| sub_u64 | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.3 vs winter-math 9.5 | −0.2 (0.98×) |
| sub_u64 | BN254 scalar | 110 vs ark-ff 354 | −244 (0.31×) | 54.8 vs crypto-bigint 151 | −96.2 (0.36×) | 137 vs ark-ff 403 | −266 (0.34×) | 74.0 vs crypto-bigint 283 | −209 (0.26×) |
| sub_u64 | secp256k1 base | 8.7 vs ark-ff 162 | −153 (0.05×) | 7.2 vs ark-ff 124 | −117 (0.06×) | 7.5 vs ark-ff 232 | −225 (0.03×) | 10.0 vs ark-ff 193 | −183 (0.05×) |
| sub_u64 | P-256 base | 70.7 vs k256/p256 266 | −195 (0.27×) | 34.7 vs k256/p256 88.8 | −54.1 (0.39×) | 80.2 vs ark-ff 398 | −318 (0.20×) | 61.1 vs k256/p256 149 | −87.9 (0.41×) |
| sub_u64 | P-256 scalar | 67.8 vs k256/p256 21.3 | +46.5 (3.18×) | 39.1 vs k256/p256 21.2 | +17.9 (1.84×) | 75.1 vs k256/p256 24.4 | +50.7 (3.08×) | 86.5 vs k256/p256 24.0 | +62.5 (3.60×) |
| sub_u64 | Tom-256 base | 88.0 vs crypto-bigint 514 | −426 (0.17×) | 49.2 vs crypto-bigint 151 | −102 (0.33×) | 105 vs halo2curves 697 | −592 (0.15×) | 70.2 vs crypto-bigint 284 | −214 (0.25×) |
| sub_u64 | BLS12-381 base | 152 vs ark-ff 835 | −683 (0.18×) | 65.5 vs halo2curves 316 | −251 (0.21×) | 246 vs ark-ff 1,079 | −833 (0.23×) | 92.7 vs crypto-bigint 617 | −524 (0.15×) |
| sub_u64 | P-521 base | 22.5 vs crypto-bigint 1,665 | −1,643 (0.01×) | 17.9 vs crypto-bigint 620 | −602 (0.03×) | 20.3 vs crypto-bigint 2,183 | −2,163 (0.01×) | 20.0 vs crypto-bigint 1,099 | −1,079 (0.02×) |
| sub_uint | Mersenne-31 | 5.0 vs crypto-bigint 23.7 | −18.7 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.2 vs crypto-bigint 20.9 | −15.7 (0.25×) |
| sub_uint | BabyBear | 11.0 vs plonky3 23.8 | −12.8 (0.46×) | 8.5 vs plonky3 10.7 | −2.2 (0.79×) | 13.0 vs plonky3 34.4 | −21.4 (0.38×) | 12.5 vs plonky3 15.0 | −2.5 (0.83×) |
| sub_uint | Goldilocks | 9.5 vs winter-math 11.5 | −2.0 (0.83×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.4 vs winter-math 19.0 | −7.6 (0.60×) | 9.3 vs winter-math 9.5 | −0.2 (0.98×) |
| sub_uint | BN254 scalar | 246 vs halo2curves 402 | −156 (0.61×) | 93.6 vs halo2curves 104 | −10.4 (0.90×) | 371 vs halo2curves 660 | −289 (0.56×) | 180 vs halo2curves 194 | −14.0 (0.93×) |
| sub_uint | secp256k1 base | 21.7 vs halo2curves 211 | −189 (0.10×) | 13.6 vs halo2curves 109 | −95.4 (0.12×) | 18.4 vs halo2curves 405 | −387 (0.05×) | 23.3 vs halo2curves 130 | −107 (0.18×) |
| sub_uint | P-256 base | 179 vs halo2curves 231 | −52.0 (0.77×) | 80.8 vs halo2curves 85.9 | −5.1 (0.94×) | 283 vs halo2curves 496 | −213 (0.57×) | 140 vs halo2curves 144 | −4.0 (0.97×) |
| sub_uint | P-256 scalar | 270 vs k256/p256 69.3 | +201 (3.90×) | 94.9 vs k256/p256 64.9 | +30.0 (1.46×) | 332 vs k256/p256 85.6 | +246 (3.88×) | 187 vs k256/p256 90.3 | +96.7 (2.07×) |
| sub_uint | Tom-256 base | 253 vs halo2curves 364 | −111 (0.70×) | 93.1 vs halo2curves 103 | −9.9 (0.90×) | 318 vs halo2curves 613 | −295 (0.52×) | 170 vs halo2curves 187 | −17.0 (0.91×) |
| sub_uint | BLS12-381 base | 759 vs halo2curves 961 | −202 (0.79×) | 205 vs halo2curves 216 | −11.0 (0.95×) | 948 vs crypto-bigint 1,363 | −415 (0.70×) | 396 vs halo2curves 408 | −12.0 (0.97×) |
| sub_uint | P-521 base | 75.5 vs crypto-bigint 1,678 | −1,603 (0.04×) | 40.7 vs crypto-bigint 620 | −579 (0.07×) | 81.0 vs crypto-bigint 2,217 | −2,136 (0.04×) | 68.5 vs crypto-bigint 1,107 | −1,039 (0.06×) |
| from_u64 | Mersenne-31 | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.0 vs plonky3 4.1 | −0.1 (0.98×) | 4.3 vs plonky3 3.9 | +0.4 (1.10×) | 4.0 vs plonky3 3.9 | +0.1 (1.03×) |
| from_u64 | BabyBear | 11.1 vs crypto-bigint 21.6 | −10.5 (0.51×) | 7.3 vs plonky3 8.3 | −1.0 (0.88×) | 12.4 vs crypto-bigint 35.3 | −22.9 (0.35×) | 10.3 vs plonky3 11.8 | −1.5 (0.87×) |
| from_u64 | Goldilocks | 7.4 vs winter-math 11.2 | −3.8 (0.66×) | 5.9 vs winter-math 5.9 | +0.0 (1.00×) | 9.0 vs winter-math 17.0 | −8.0 (0.53×) | 7.1 vs winter-math 7.0 | +0.1 (1.01×) |
| from_u64 | BN254 scalar | 101 vs ark-ff 316 | −215 (0.32×) | 29.1 vs crypto-bigint 133 | −104 (0.22×) | 121 vs ark-ff 375 | −254 (0.32×) | 50.1 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64 | secp256k1 base | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) | 3.4 vs k256/p256 5.5 | −2.1 (0.62×) |
| from_u64 | P-256 base | 56.8 vs k256/p256 240 | −183 (0.24×) | 24.6 vs k256/p256 76.0 | −51.4 (0.32×) | 70.6 vs ark-ff 376 | −305 (0.19×) | 44.8 vs k256/p256 130 | −85.2 (0.34×) |
| from_u64 | P-256 scalar | 55.1 vs k256/p256 3.8 | +51.3 (14.50×) | 26.1 vs k256/p256 3.8 | +22.3 (6.87×) | 59.9 vs k256/p256 3.8 | +56.1 (15.76×) | 48.1 vs k256/p256 3.8 | +44.3 (12.66×) |
| from_u64 | Tom-256 base | 74.5 vs crypto-bigint 430 | −356 (0.17×) | 28.2 vs crypto-bigint 133 | −105 (0.21×) | 86.2 vs halo2curves 665 | −579 (0.13×) | 49.0 vs crypto-bigint 205 | −156 (0.24×) |
| from_u64 | BLS12-381 base | 144 vs ark-ff 741 | −597 (0.19×) | 50.6 vs crypto-bigint 252 | −201 (0.20×) | 179 vs ark-ff 1,025 | −846 (0.17×) | 68.1 vs crypto-bigint 525 | −457 (0.13×) |
| from_u64 | P-521 base | 6.1 vs crypto-bigint 1,444 | −1,438 (0.00×) | 6.0 vs crypto-bigint 446 | −440 (0.01×) | 6.0 vs crypto-bigint 1,976 | −1,970 (0.00×) | 6.0 vs crypto-bigint 920 | −914 (0.01×) |
| div_u64 | BN254 scalar | 1,212 vs halo2curves 13,540 | −12,328 (0.09×) | 844 vs halo2curves 6,993 | −6,149 (0.12×) | 1,374 vs halo2curves 19,377 | −18,003 (0.07×) | 1,116 vs halo2curves 11,923 | −10,807 (0.09×) |
| div_u64 | secp256k1 base | 1,011 vs halo2curves 13,038 | −12,027 (0.08×) | 712 vs halo2curves 6,920 | −6,208 (0.10×) | 1,230 vs halo2curves 18,855 | −17,625 (0.07×) | 823 vs halo2curves 11,578 | −10,755 (0.07×) |
| div_u64 | P-256 base | 1,090 vs halo2curves 13,149 | −12,059 (0.08×) | 805 vs halo2curves 6,909 | −6,104 (0.12×) | 1,354 vs halo2curves 18,907 | −17,553 (0.07×) | 982 vs halo2curves 11,715 | −10,733 (0.08×) |
| div_u64 | P-256 scalar | 1,191 vs halo2curves 13,412 | −12,221 (0.09×) | 848 vs halo2curves 6,945 | −6,097 (0.12×) | 1,408 vs halo2curves 19,293 | −17,885 (0.07×) | 1,132 vs halo2curves 11,874 | −10,742 (0.10×) |
| div_u64 | Tom-256 base | 1,207 vs halo2curves 13,287 | −12,080 (0.09×) | 849 vs halo2curves 6,926 | −6,077 (0.12×) | 1,495 vs halo2curves 19,212 | −17,717 (0.08×) | 1,043 vs halo2curves 11,679 | −10,636 (0.09×) |
| div_u64 | BLS12-381 base | 1,804 vs halo2curves 24,206 | −22,402 (0.07×) | 1,069 vs halo2curves 11,051 | −9,982 (0.10×) | 2,469 vs halo2curves 32,677 | −30,208 (0.08×) | 1,752 vs halo2curves 18,849 | −17,097 (0.09×) |
| div_u64 | P-521 base | 2,457 vs crypto-bigint 62,572 | −60,115 (0.04×) | 1,002 vs crypto-bigint 45,200 | −44,198 (0.02×) | 3,596 vs crypto-bigint 77,783 | −74,187 (0.05×) | 1,489 vs crypto-bigint 57,875 | −56,386 (0.03×) |
| div_uint | Mersenne-31 | 274 vs crypto-bigint 1,811 | −1,537 (0.15×) | 275 vs crypto-bigint 1,599 | −1,324 (0.17×) | 283 vs runtime-algebra 2,129 | −1,846 (0.13×) | 283 vs crypto-bigint 2,017 | −1,734 (0.14×) |
| div_uint | BabyBear | 276 vs plonky3 676 | −400 (0.41×) | 274 vs plonky3 658 | −384 (0.42×) | 286 vs plonky3 720 | −434 (0.40×) | 290 vs plonky3 732 | −442 (0.40×) |
| div_uint | Goldilocks | 552 vs winter-math 1,825 | −1,273 (0.30×) | 541 vs winter-math 945 | −404 (0.57×) | 583 vs winter-math 1,894 | −1,311 (0.31×) | 578 vs winter-math 1,053 | −475 (0.55×) |
| div_uint | BN254 scalar | 9,251 vs halo2curves 15,088 | −5,837 (0.61×) | 6,600 vs halo2curves 8,183 | −1,583 (0.81×) | 12,660 vs halo2curves 23,044 | −10,384 (0.55×) | 8,480 vs halo2curves 14,736 | −6,256 (0.58×) |
| div_uint | secp256k1 base | 8,621 vs halo2curves 14,914 | −6,293 (0.58×) | 6,043 vs halo2curves 8,204 | −2,161 (0.74×) | 12,041 vs halo2curves 22,868 | −10,827 (0.53×) | 8,124 vs halo2curves 14,643 | −6,519 (0.55×) |
| div_uint | P-256 base | 8,822 vs halo2curves 14,867 | −6,045 (0.59×) | 6,207 vs halo2curves 8,198 | −1,991 (0.76×) | 12,166 vs halo2curves 22,902 | −10,736 (0.53×) | 8,046 vs halo2curves 14,655 | −6,609 (0.55×) |
| div_uint | P-256 scalar | 9,209 vs halo2curves 15,128 | −5,919 (0.61×) | 6,545 vs halo2curves 8,267 | −1,722 (0.79×) | 12,668 vs halo2curves 23,140 | −10,472 (0.55×) | 8,637 vs halo2curves 14,838 | −6,201 (0.58×) |
| div_uint | Tom-256 base | 9,148 vs halo2curves 15,039 | −5,891 (0.61×) | 6,642 vs halo2curves 8,260 | −1,618 (0.80×) | 12,264 vs halo2curves 23,104 | −10,840 (0.53×) | 8,484 vs halo2curves 14,682 | −6,198 (0.58×) |
| div_uint | BLS12-381 base | 20,907 vs halo2curves 27,536 | −6,629 (0.76×) | 11,913 vs halo2curves 13,703 | −1,790 (0.87×) | 26,343 vs halo2curves 40,735 | −14,392 (0.65×) | 17,035 vs halo2curves 24,855 | −7,820 (0.69×) |
| div_uint | P-521 base | 34,180 vs crypto-bigint 67,775 | −33,595 (0.50×) | 18,562 vs crypto-bigint 51,278 | −32,716 (0.36×) | 49,727 vs crypto-bigint 84,345 | −34,618 (0.59×) | 30,259 vs crypto-bigint 64,118 | −33,859 (0.47×) |
| add, constant time | Mersenne-31 | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.8 | −8.1 (0.25×) | 2.7 vs crypto-bigint 10.8 | −8.1 (0.25×) | 2.7 vs runtime-algebra 10.9 | −8.2 (0.25×) |
| add, constant time | BabyBear | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 2.4 vs runtime-algebra 10.8 | −8.4 (0.22×) | 3.0 vs crypto-bigint 10.8 | −7.8 (0.28×) | 3.0 vs runtime-algebra 10.9 | −7.9 (0.28×) |
| add, constant time | Goldilocks | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.3 vs winter-math 3.3 | +0.0 (1.00×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) | 3.2 vs winter-math 4.0 | −0.8 (0.80×) |
| add, constant time | BN254 scalar | 23.7 vs halo2curves 36.4 | −12.7 (0.65×) | 16.1 vs halo2curves 20.6 | −4.5 (0.78×) | 20.8 vs halo2curves 39.5 | −18.7 (0.53×) | 26.1 vs halo2curves 35.1 | −9.0 (0.74×) |
| add, constant time | secp256k1 base | 26.3 vs halo2curves 35.1 | −8.8 (0.75×) | 15.7 vs halo2curves 19.3 | −3.6 (0.81×) | 25.0 vs halo2curves 35.6 | −10.6 (0.70×) | 28.0 vs halo2curves 33.3 | −5.3 (0.84×) |
| add, constant time | P-256 base | 23.5 vs k256/p256 30.5 | −7.0 (0.77×) | 15.6 vs k256/p256 17.9 | −2.3 (0.87×) | 22.8 vs halo2curves 33.8 | −11.0 (0.67×) | 27.0 vs k256/p256 31.5 | −4.5 (0.86×) |
| add, constant time | P-256 scalar | 27.2 vs halo2curves 35.5 | −8.3 (0.77×) | 16.3 vs halo2curves 20.7 | −4.4 (0.79×) | 24.0 vs halo2curves 38.0 | −14.0 (0.63×) | 28.5 vs halo2curves 35.1 | −6.6 (0.81×) |
| add, constant time | Tom-256 base | 27.8 vs halo2curves 35.7 | −7.9 (0.78×) | 16.1 vs halo2curves 20.2 | −4.1 (0.80×) | 24.3 vs halo2curves 38.8 | −14.5 (0.63×) | 29.1 vs halo2curves 34.1 | −5.0 (0.85×) |
| add, constant time | BLS12-381 base | 42.9 vs halo2curves 63.7 | −20.8 (0.67×) | 23.9 vs halo2curves 32.4 | −8.5 (0.74×) | 35.7 vs halo2curves 69.9 | −34.2 (0.51×) | 40.1 vs halo2curves 54.1 | −14.0 (0.74×) |
| add, constant time | P-521 base | 76.7 vs crypto-bigint 147 | −70.3 (0.52×) | 40.7 vs crypto-bigint 136 | −95.3 (0.30×) | 77.1 vs crypto-bigint 193 | −116 (0.40×) | 72.0 vs crypto-bigint 187 | −115 (0.39×) |
| sub, constant time | Mersenne-31 | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | BabyBear | 2.5 vs crypto-bigint 4.5 | −2.0 (0.56×) | 2.6 vs crypto-bigint 4.5 | −1.9 (0.58×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) | 2.7 vs crypto-bigint 5.0 | −2.3 (0.54×) |
| sub, constant time | Goldilocks | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 2.6 vs winter-math 2.8 | −0.2 (0.93×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.3 vs halo2curves 24.0 | −2.7 (0.89×) | 14.1 vs halo2curves 15.0 | −0.9 (0.94×) | 20.7 vs halo2curves 23.3 | −2.6 (0.89×) | 23.1 vs halo2curves 24.1 | −1.0 (0.96×) |
| sub, constant time | secp256k1 base | 19.2 vs halo2curves 22.3 | −3.1 (0.86×) | 13.6 vs halo2curves 14.3 | −0.7 (0.95×) | 18.6 vs halo2curves 20.4 | −1.8 (0.91×) | 22.0 vs halo2curves 22.1 | −0.1 (1.00×) |
| sub, constant time | P-256 base | 17.3 vs halo2curves 20.5 | −3.2 (0.84×) | 12.1 vs halo2curves 12.3 | −0.2 (0.98×) | 18.4 vs halo2curves 21.2 | −2.8 (0.87×) | 21.1 vs halo2curves 21.5 | −0.4 (0.98×) |
| sub, constant time | P-256 scalar | 21.1 vs k256/p256 23.1 | −2.0 (0.91×) | 14.1 vs halo2curves 14.4 | −0.3 (0.98×) | 22.4 vs halo2curves 23.4 | −1.0 (0.96×) | 23.0 vs halo2curves 23.1 | −0.1 (1.00×) |
| sub, constant time | Tom-256 base | 21.1 vs halo2curves 24.0 | −2.9 (0.88×) | 14.1 vs halo2curves 14.9 | −0.8 (0.95×) | 21.0 vs halo2curves 23.4 | −2.4 (0.90×) | 23.0 vs halo2curves 24.1 | −1.1 (0.95×) |
| sub, constant time | BLS12-381 base | 37.6 vs halo2curves 43.4 | −5.8 (0.87×) | 21.4 vs halo2curves 23.6 | −2.2 (0.91×) | 34.9 vs halo2curves 44.2 | −9.3 (0.79×) | 36.1 vs halo2curves 37.1 | −1.0 (0.97×) |
| sub, constant time | P-521 base | 56.2 vs crypto-bigint 88.8 | −32.6 (0.63×) | 31.2 vs crypto-bigint 64.4 | −33.2 (0.48×) | 58.1 vs crypto-bigint 121 | −62.9 (0.48×) | 51.1 vs crypto-bigint 128 | −76.9 (0.40×) |
| mul, constant time | Mersenne-31 | 3.7 vs crypto-bigint 29.4 | −25.7 (0.13×) | 3.6 vs crypto-bigint 15.8 | −12.2 (0.23×) | 4.0 vs crypto-bigint 38.9 | −34.9 (0.10×) | 4.0 vs crypto-bigint 19.2 | −15.2 (0.21×) |
| mul, constant time | BabyBear | 5.2 vs crypto-bigint 29.3 | −24.1 (0.18×) | 5.2 vs crypto-bigint 15.8 | −10.6 (0.33×) | 5.2 vs crypto-bigint 39.2 | −34.0 (0.13×) | 5.0 vs crypto-bigint 19.2 | −14.2 (0.26×) |
| mul, constant time | Goldilocks | 10.0 vs winter-math 12.2 | −2.2 (0.82×) | 6.3 vs winter-math 6.5 | −0.2 (0.97×) | 15.0 vs winter-math 17.0 | −2.0 (0.88×) | 7.0 vs winter-math 7.7 | −0.7 (0.91×) |
| mul, constant time | BN254 scalar | 236 vs halo2curves 374 | −138 (0.63×) | 79.6 vs halo2curves 91.8 | −12.2 (0.87×) | 418 vs halo2curves 701 | −283 (0.60×) | 164 vs halo2curves 182 | −18.0 (0.90×) |
| mul, constant time | secp256k1 base | 162 vs halo2curves 303 | −141 (0.53×) | 59.6 vs halo2curves 91.1 | −31.5 (0.65×) | 232 vs halo2curves 591 | −359 (0.39×) | 109 vs halo2curves 163 | −54.0 (0.67×) |
| mul, constant time | P-256 base | 177 vs k256/p256 244 | −67.0 (0.73×) | 71.9 vs k256/p256 85.0 | −13.1 (0.85×) | 316 vs k256/p256 472 | −156 (0.67×) | 119 vs halo2curves 135 | −16.0 (0.88×) |
| mul, constant time | P-256 scalar | 246 vs halo2curves 343 | −97.0 (0.72×) | 84.3 vs halo2curves 93.9 | −9.6 (0.90×) | 329 vs halo2curves 701 | −372 (0.47×) | 163 vs halo2curves 181 | −18.0 (0.90×) |
| mul, constant time | Tom-256 base | 241 vs halo2curves 345 | −104 (0.70×) | 74.6 vs halo2curves 94.1 | −19.5 (0.79×) | 310 vs halo2curves 624 | −314 (0.50×) | 163 vs halo2curves 176 | −13.0 (0.93×) |
| mul, constant time | BLS12-381 base | 729 vs crypto-bigint 914 | −185 (0.80×) | 183 vs halo2curves 195 | −12.0 (0.94×) | 929 vs crypto-bigint 1,311 | −382 (0.71×) | 321 vs halo2curves 382 | −61.0 (0.84×) |
| mul, constant time | P-521 base | 670 vs crypto-bigint 2,123 | −1,453 (0.32×) | 244 vs crypto-bigint 601 | −357 (0.41×) | 1,279 vs crypto-bigint 2,712 | −1,433 (0.47×) | 533 vs crypto-bigint 1,053 | −520 (0.51×) |
| square, constant time | Mersenne-31 | 3.5 vs crypto-bigint 28.7 | −25.2 (0.12×) | 3.5 vs crypto-bigint 15.6 | −12.1 (0.22×) | 3.6 vs crypto-bigint 38.3 | −34.7 (0.09×) | 3.6 vs crypto-bigint 18.1 | −14.5 (0.20×) |
| square, constant time | BabyBear | 4.7 vs crypto-bigint 28.7 | −24.0 (0.16×) | 4.8 vs crypto-bigint 15.5 | −10.7 (0.31×) | 5.7 vs crypto-bigint 38.3 | −32.6 (0.15×) | 5.8 vs crypto-bigint 17.8 | −12.0 (0.33×) |
| square, constant time | Goldilocks | 8.8 vs winter-math 12.5 | −3.7 (0.70×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 17.0 | −4.0 (0.76×) | 8.0 vs winter-math 7.0 | +1.0 (1.14×) |
| square, constant time | BN254 scalar | 218 vs crypto-bigint 325 | −107 (0.67×) | 84.5 vs halo2curves 84.8 | −0.3 (1.00×) | 296 vs halo2curves 540 | −244 (0.55×) | 138 vs halo2curves 144 | −6.0 (0.96×) |
| square, constant time | secp256k1 base | 144 vs halo2curves 266 | −122 (0.54×) | 53.1 vs halo2curves 78.6 | −25.5 (0.68×) | 192 vs halo2curves 461 | −269 (0.42×) | 88.4 vs halo2curves 123 | −34.6 (0.72×) |
| square, constant time | P-256 base | 166 vs halo2curves 188 | −22.0 (0.88×) | 74.2 vs k256/p256 73.1 | +1.1 (1.02×) | 260 vs halo2curves 334 | −74.0 (0.78×) | 118 vs halo2curves 114 | +4.0 (1.04×) |
| square, constant time | P-256 scalar | 220 vs halo2curves 306 | −86.0 (0.72×) | 75.7 vs halo2curves 83.1 | −7.4 (0.91×) | 290 vs halo2curves 547 | −257 (0.53×) | 135 vs halo2curves 141 | −6.0 (0.96×) |
| square, constant time | Tom-256 base | 223 vs halo2curves 312 | −89.0 (0.71×) | 72.4 vs halo2curves 78.0 | −5.6 (0.93×) | 282 vs halo2curves 474 | −192 (0.59×) | 164 vs halo2curves 136 | +28.0 (1.21×) |
| square, constant time | BLS12-381 base | 570 vs halo2curves 683 | −113 (0.83×) | 157 vs halo2curves 163 | −6.0 (0.96×) | 826 vs halo2curves 1,297 | −471 (0.64×) | 278 vs halo2curves 309 | −31.0 (0.90×) |
| square, constant time | P-521 base | 564 vs crypto-bigint 2,163 | −1,599 (0.26×) | 238 vs crypto-bigint 616 | −378 (0.39×) | 914 vs crypto-bigint 2,837 | −1,923 (0.32×) | 394 vs crypto-bigint 1,124 | −730 (0.35×) |
| invert, constant time | Mersenne-31 | 263 vs runtime-algebra 3,344 | −3,081 (0.08×) | 264 vs runtime-algebra 3,342 | −3,078 (0.08×) | 267 vs runtime-algebra 3,323 | −3,056 (0.08×) | 267 vs runtime-algebra 3,339 | −3,072 (0.08×) |
| invert, constant time | BabyBear | 451 vs runtime-algebra 3,316 | −2,865 (0.14×) | 458 vs runtime-algebra 3,302 | −2,844 (0.14×) | 446 vs runtime-algebra 3,341 | −2,895 (0.13×) | 417 vs runtime-algebra 3,333 | −2,916 (0.13×) |
| invert, constant time | Goldilocks | 1,156 vs winter-math 1,748 | −592 (0.66×) | 841 vs winter-math 912 | −71.0 (0.92×) | 1,417 vs winter-math 1,834 | −417 (0.77×) | 937 vs winter-math 998 | −61.0 (0.94×) |
| invert, constant time | BN254 scalar | 11,998 vs crypto-bigint 29,487 | −17,489 (0.41×) | 9,522 vs crypto-bigint 25,374 | −15,852 (0.38×) | 16,747 vs crypto-bigint 36,557 | −19,810 (0.46×) | 12,585 vs crypto-bigint 30,776 | −18,191 (0.41×) |
| invert, constant time | secp256k1 base | 11,885 vs crypto-bigint 29,196 | −17,311 (0.41×) | 9,101 vs crypto-bigint 25,344 | −16,243 (0.36×) | 16,868 vs crypto-bigint 36,254 | −19,386 (0.47×) | 12,456 vs crypto-bigint 30,760 | −18,304 (0.40×) |
| invert, constant time | P-256 base | 11,508 vs crypto-bigint 28,917 | −17,409 (0.40×) | 8,910 vs k256/p256 22,127 | −13,217 (0.40×) | 16,410 vs crypto-bigint 36,410 | −20,000 (0.45×) | 12,253 vs crypto-bigint 30,820 | −18,567 (0.40×) |
| invert, constant time | P-256 scalar | 11,847 vs crypto-bigint 29,581 | −17,734 (0.40×) | 9,385 vs crypto-bigint 25,376 | −15,991 (0.37×) | 17,022 vs crypto-bigint 36,212 | −19,190 (0.47×) | 12,800 vs crypto-bigint 30,933 | −18,133 (0.41×) |
| invert, constant time | Tom-256 base | 11,824 vs crypto-bigint 29,072 | −17,248 (0.41×) | 9,271 vs crypto-bigint 25,335 | −16,064 (0.37×) | 16,657 vs crypto-bigint 36,131 | −19,474 (0.46×) | 12,803 vs crypto-bigint 30,851 | −18,048 (0.41×) |
| invert, constant time | BLS12-381 base | 24,064 vs crypto-bigint 49,228 | −25,164 (0.49×) | 16,804 vs crypto-bigint 42,680 | −25,876 (0.39×) | 32,675 vs crypto-bigint 66,553 | −33,878 (0.49×) | 24,014 vs crypto-bigint 54,189 | −30,175 (0.44×) |
| invert, constant time | P-521 base | 41,920 vs crypto-bigint 101,592 | −59,672 (0.41×) | 25,931 vs crypto-bigint 82,195 | −56,264 (0.32×) | 64,412 vs crypto-bigint 122,905 | −58,493 (0.52×) | 41,531 vs crypto-bigint 99,851 | −58,320 (0.42×) |
| sqrt, constant time | Mersenne-31 | 159 vs runtime-algebra 1,704 | −1,545 (0.09×) | 159 vs crypto-bigint 1,644 | −1,485 (0.10×) | 170 vs runtime-algebra 1,741 | −1,571 (0.10×) | 172 vs runtime-algebra 1,746 | −1,574 (0.10×) |
| sqrt, constant time | BabyBear | 7,140 vs crypto-bigint 33,589 | −26,449 (0.21×) | 6,465 vs crypto-bigint 20,184 | −13,719 (0.32×) | 6,697 vs crypto-bigint 38,292 | −31,595 (0.17×) | 6,492 vs crypto-bigint 23,012 | −16,520 (0.28×) |
| sqrt, constant time | Goldilocks | 11,050 vs crypto-bigint 48,577 | −37,527 (0.23×) | 8,393 vs crypto-bigint 28,840 | −20,447 (0.29×) | 14,306 vs crypto-bigint 54,379 | −40,073 (0.26×) | 10,377 vs crypto-bigint 32,894 | −22,517 (0.32×) |
| sqrt, constant time | BN254 scalar | 180,334 vs crypto-bigint 329,653 | −149,319 (0.55×) | 81,774 vs halo2curves 102,662 | −20,888 (0.80×) | 291,304 vs halo2curves 535,296 | −243,992 (0.54×) | 150,680 vs halo2curves 179,719 | −29,039 (0.84×) |
| sqrt, constant time | secp256k1 base | 41,194 vs crypto-bigint 158,491 | −117,297 (0.26×) | 16,526 vs halo2curves 43,325 | −26,799 (0.38×) | 65,297 vs crypto-bigint 249,032 | −183,735 (0.26×) | 26,766 vs crypto-bigint 81,855 | −55,089 (0.33×) |
| sqrt, constant time | P-256 base | 42,953 vs k256/p256 64,890 | −21,937 (0.66×) | 14,535 vs k256/p256 21,667 | −7,132 (0.67×) | 74,361 vs k256/p256 119,414 | −45,053 (0.62×) | 31,088 vs k256/p256 33,586 | −2,498 (0.93×) |
| sqrt, constant time | P-256 scalar | 77,786 vs halo2curves 153,191 | −75,405 (0.51×) | 34,601 vs halo2curves 49,381 | −14,780 (0.70×) | 117,332 vs crypto-bigint 246,924 | −129,592 (0.48×) | 65,489 vs halo2curves 72,405 | −6,916 (0.90×) |
| sqrt, constant time | Tom-256 base | 65,835 vs crypto-bigint 148,892 | −83,057 (0.44×) | 27,142 vs halo2curves 45,656 | −18,514 (0.59×) | 98,940 vs crypto-bigint 225,627 | −126,687 (0.44×) | 50,439 vs crypto-bigint 76,147 | −25,708 (0.66×) |
| sqrt, constant time | BLS12-381 base | 299,234 vs crypto-bigint 546,579 | −247,345 (0.55×) | 93,230 vs crypto-bigint 145,084 | −51,854 (0.64×) | 395,250 vs crypto-bigint 619,276 | −224,026 (0.64×) | 206,810 vs crypto-bigint 268,366 | −61,556 (0.77×) |
| sqrt, constant time | P-521 base | 324,126 vs crypto-bigint 1,534,942 | −1,210,816 (0.21×) | 124,826 vs crypto-bigint 296,458 | −171,632 (0.42×) | 506,222 vs crypto-bigint 1,563,666 | −1,057,444 (0.32×) | 257,676 vs crypto-bigint 591,904 | −334,228 (0.44×) |
| mul_u64, constant time | Mersenne-31 | 6.8 vs crypto-bigint 61.5 | −54.7 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 72.6 | −65.0 (0.10×) | 7.6 vs crypto-bigint 38.7 | −31.1 (0.20×) |
| mul_u64, constant time | BabyBear | 13.0 vs crypto-bigint 63.7 | −50.7 (0.20×) | 11.2 vs crypto-bigint 33.0 | −21.8 (0.34×) | 18.0 vs crypto-bigint 85.1 | −67.1 (0.21×) | 14.7 vs crypto-bigint 40.4 | −25.7 (0.36×) |
| mul_u64, constant time | Goldilocks | 17.3 vs winter-math 25.1 | −7.8 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.0 vs winter-math 13.6 | −4.6 (0.66×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 845 | −739 (0.13×) | 38.1 vs crypto-bigint 263 | −225 (0.14×) | 125 vs halo2curves 1,424 | −1,299 (0.09×) | 56.1 vs crypto-bigint 407 | −351 (0.14×) |
| mul_u64, constant time | secp256k1 base | 61.6 vs halo2curves 689 | −627 (0.09×) | 22.3 vs crypto-bigint 263 | −241 (0.08×) | 80.1 vs halo2curves 1,046 | −966 (0.08×) | 35.9 vs crypto-bigint 408 | −372 (0.09×) |
| mul_u64, constant time | P-256 base | 86.1 vs k256/p256 484 | −398 (0.18×) | 37.2 vs k256/p256 150 | −113 (0.25×) | 99.1 vs k256/p256 945 | −846 (0.10×) | 55.3 vs k256/p256 258 | −203 (0.21×) |
| mul_u64, constant time | P-256 scalar | 102 vs k256/p256 521 | −419 (0.20×) | 41.4 vs k256/p256 254 | −213 (0.16×) | 122 vs k256/p256 830 | −708 (0.15×) | 56.9 vs k256/p256 337 | −280 (0.17×) |
| mul_u64, constant time | Tom-256 base | 95.3 vs crypto-bigint 866 | −771 (0.11×) | 41.0 vs crypto-bigint 263 | −222 (0.16×) | 123 vs halo2curves 1,271 | −1,148 (0.10×) | 60.9 vs crypto-bigint 408 | −347 (0.15×) |
| mul_u64, constant time | BLS12-381 base | 154 vs crypto-bigint 1,909 | −1,755 (0.08×) | 62.4 vs halo2curves 485 | −423 (0.13×) | 203 vs crypto-bigint 2,575 | −2,372 (0.08×) | 113 vs crypto-bigint 1,032 | −919 (0.11×) |
| mul_u64, constant time | P-521 base | 141 vs crypto-bigint 3,015 | −2,874 (0.05×) | 64.3 vs crypto-bigint 992 | −928 (0.06×) | 165 vs crypto-bigint 4,010 | −3,845 (0.04×) | 89.8 vs crypto-bigint 1,831 | −1,741 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 6.8 vs crypto-bigint 63.0 | −56.2 (0.11×) | 6.7 vs crypto-bigint 30.5 | −23.8 (0.22×) | 7.6 vs crypto-bigint 72.8 | −65.2 (0.10×) | 7.6 vs crypto-bigint 38.7 | −31.1 (0.20×) |
| mul_uint, constant time | BabyBear | 13.0 vs crypto-bigint 63.7 | −50.7 (0.20×) | 11.2 vs crypto-bigint 33.0 | −21.8 (0.34×) | 18.0 vs crypto-bigint 85.2 | −67.2 (0.21×) | 14.7 vs crypto-bigint 40.5 | −25.8 (0.36×) |
| mul_uint, constant time | Goldilocks | 17.2 vs winter-math 25.0 | −7.8 (0.69×) | 7.3 vs winter-math 10.9 | −3.6 (0.67×) | 24.8 vs winter-math 31.7 | −6.9 (0.78×) | 9.0 vs winter-math 13.6 | −4.6 (0.66×) |
| mul_uint, constant time | BN254 scalar | 490 vs halo2curves 776 | −286 (0.63×) | 167 vs halo2curves 182 | −15.0 (0.92×) | 595 vs halo2curves 1,356 | −761 (0.44×) | 331 vs halo2curves 349 | −18.0 (0.95×) |
| mul_uint, constant time | secp256k1 base | 322 vs halo2curves 519 | −197 (0.62×) | 81.7 vs halo2curves 153 | −71.3 (0.53×) | 381 vs halo2curves 962 | −581 (0.40×) | 117 vs halo2curves 266 | −149 (0.44×) |
| mul_uint, constant time | P-256 base | 413 vs halo2curves 454 | −41.0 (0.91×) | 127 vs halo2curves 143 | −16.0 (0.89×) | 491 vs halo2curves 946 | −455 (0.52×) | 236 vs halo2curves 253 | −17.0 (0.93×) |
| mul_uint, constant time | P-256 scalar | 468 vs k256/p256 554 | −86.0 (0.84×) | 165 vs halo2curves 186 | −21.0 (0.89×) | 549 vs k256/p256 902 | −353 (0.61×) | 346 vs halo2curves 347 | −1.0 (1.00×) |
| mul_uint, constant time | Tom-256 base | 469 vs halo2curves 697 | −228 (0.67×) | 167 vs halo2curves 180 | −13.0 (0.93×) | 553 vs halo2curves 1,194 | −641 (0.46×) | 329 vs halo2curves 336 | −7.0 (0.98×) |
| mul_uint, constant time | BLS12-381 base | 1,088 vs halo2curves 1,919 | −831 (0.57×) | 368 vs halo2curves 381 | −13.0 (0.97×) | 1,688 vs crypto-bigint 2,570 | −882 (0.66×) | 889 vs crypto-bigint 1,037 | −148 (0.86×) |
| mul_uint, constant time | P-521 base | 2,008 vs crypto-bigint 3,000 | −992 (0.67×) | 291 vs crypto-bigint 990 | −699 (0.29×) | 2,914 vs crypto-bigint 4,047 | −1,133 (0.72×) | 499 vs crypto-bigint 1,841 | −1,342 (0.27×) |
| add_u64, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.1 | −30.9 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.4 vs crypto-bigint 41.4 | −36.0 (0.13×) | 5.4 vs crypto-bigint 30.4 | −25.0 (0.18×) |
| add_u64, constant time | BabyBear | 10.4 vs crypto-bigint 39.9 | −29.5 (0.26×) | 7.4 vs crypto-bigint 31.7 | −24.3 (0.23×) | 14.8 vs crypto-bigint 53.8 | −39.0 (0.28×) | 11.6 vs crypto-bigint 31.8 | −20.2 (0.36×) |
| add_u64, constant time | Goldilocks | 9.3 vs winter-math 13.3 | −4.0 (0.70×) | 7.0 vs winter-math 7.4 | −0.4 (0.95×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.4 vs winter-math 11.0 | −1.6 (0.85×) |
| add_u64, constant time | BN254 scalar | 93.4 vs crypto-bigint 505 | −412 (0.18×) | 41.9 vs halo2curves 210 | −168 (0.20×) | 108 vs halo2curves 757 | −649 (0.14×) | 75.5 vs halo2curves 298 | −223 (0.25×) |
| add_u64, constant time | secp256k1 base | 22.5 vs halo2curves 406 | −384 (0.06×) | 14.2 vs halo2curves 198 | −184 (0.07×) | 20.2 vs halo2curves 485 | −465 (0.04×) | 24.0 vs halo2curves 280 | −256 (0.09×) |
| add_u64, constant time | P-256 base | 70.9 vs k256/p256 277 | −206 (0.26×) | 35.9 vs k256/p256 95.2 | −59.3 (0.38×) | 77.5 vs k256/p256 516 | −439 (0.15×) | 58.0 vs k256/p256 159 | −101 (0.36×) |
| add_u64, constant time | P-256 scalar | 91.9 vs k256/p256 41.1 | +50.8 (2.24×) | 43.1 vs k256/p256 41.0 | +2.1 (1.05×) | 112 vs k256/p256 54.2 | +57.8 (2.07×) | 73.2 vs k256/p256 54.9 | +18.3 (1.33×) |
| add_u64, constant time | Tom-256 base | 94.3 vs crypto-bigint 505 | −411 (0.19×) | 40.9 vs halo2curves 219 | −178 (0.19×) | 111 vs halo2curves 703 | −592 (0.16×) | 65.3 vs halo2curves 291 | −226 (0.22×) |
| add_u64, constant time | BLS12-381 base | 147 vs crypto-bigint 1,081 | −934 (0.14×) | 69.8 vs halo2curves 322 | −252 (0.22×) | 171 vs crypto-bigint 1,415 | −1,244 (0.12×) | 116 vs crypto-bigint 650 | −534 (0.18×) |
| add_u64, constant time | P-521 base | 65.1 vs crypto-bigint 1,688 | −1,623 (0.04×) | 39.1 vs crypto-bigint 648 | −609 (0.06×) | 72.2 vs crypto-bigint 2,205 | −2,133 (0.03×) | 62.0 vs crypto-bigint 1,148 | −1,086 (0.05×) |
| add_uint, constant time | Mersenne-31 | 5.2 vs crypto-bigint 36.3 | −31.1 (0.14×) | 5.2 vs crypto-bigint 30.4 | −25.2 (0.17×) | 5.4 vs crypto-bigint 41.4 | −36.0 (0.13×) | 5.4 vs crypto-bigint 30.8 | −25.4 (0.18×) |
| add_uint, constant time | BabyBear | 10.4 vs crypto-bigint 39.9 | −29.5 (0.26×) | 7.4 vs crypto-bigint 31.6 | −24.2 (0.23×) | 14.8 vs crypto-bigint 55.2 | −40.4 (0.27×) | 11.6 vs crypto-bigint 31.6 | −20.0 (0.37×) |
| add_uint, constant time | Goldilocks | 9.3 vs winter-math 13.5 | −4.2 (0.69×) | 7.0 vs winter-math 7.4 | −0.4 (0.95×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.4 vs winter-math 11.0 | −1.6 (0.85×) |
| add_uint, constant time | BN254 scalar | 261 vs halo2curves 410 | −149 (0.64×) | 95.2 vs halo2curves 113 | −17.8 (0.84×) | 294 vs halo2curves 684 | −390 (0.43×) | 184 vs halo2curves 205 | −21.0 (0.90×) |
| add_uint, constant time | secp256k1 base | 41.9 vs halo2curves 222 | −180 (0.19×) | 24.9 vs halo2curves 84.9 | −60.0 (0.29×) | 36.5 vs halo2curves 415 | −379 (0.09×) | 40.6 vs halo2curves 140 | −99.4 (0.29×) |
| add_uint, constant time | P-256 base | 188 vs halo2curves 246 | −58.0 (0.76×) | 96.1 vs halo2curves 91.4 | +4.7 (1.05×) | 287 vs halo2curves 516 | −229 (0.56×) | 141 vs halo2curves 153 | −12.0 (0.92×) |
| add_uint, constant time | P-256 scalar | 256 vs k256/p256 95.0 | +161 (2.69×) | 95.9 vs k256/p256 94.7 | +1.2 (1.01×) | 337 vs k256/p256 119 | +218 (2.83×) | 186 vs k256/p256 119 | +67.0 (1.56×) |
| add_uint, constant time | Tom-256 base | 254 vs halo2curves 379 | −125 (0.67×) | 92.3 vs halo2curves 110 | −17.7 (0.84×) | 330 vs halo2curves 615 | −285 (0.54×) | 180 vs halo2curves 196 | −16.0 (0.92×) |
| add_uint, constant time | BLS12-381 base | 769 vs halo2curves 984 | −215 (0.78×) | 204 vs halo2curves 224 | −20.0 (0.91×) | 959 vs crypto-bigint 1,424 | −465 (0.67×) | 390 vs halo2curves 424 | −34.0 (0.92×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,686 | −1,548 (0.08×) | 80.4 vs crypto-bigint 647 | −567 (0.12×) | 147 vs crypto-bigint 2,246 | −2,099 (0.07×) | 128 vs crypto-bigint 1,157 | −1,029 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.4 | −18.4 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.1 vs crypto-bigint 31.2 | −26.1 (0.16×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_u64, constant time | BabyBear | 11.3 vs crypto-bigint 27.9 | −16.6 (0.41×) | 7.6 vs crypto-bigint 19.5 | −11.9 (0.39×) | 14.0 vs crypto-bigint 42.3 | −28.3 (0.33×) | 10.7 vs crypto-bigint 21.8 | −11.1 (0.49×) |
| sub_u64, constant time | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.6 vs winter-math 19.0 | −7.4 (0.61×) | 9.2 vs winter-math 9.5 | −0.3 (0.97×) |
| sub_u64, constant time | BN254 scalar | 91.2 vs crypto-bigint 501 | −410 (0.18×) | 37.9 vs crypto-bigint 151 | −113 (0.25×) | 109 vs halo2curves 756 | −647 (0.14×) | 67.4 vs crypto-bigint 283 | −216 (0.24×) |
| sub_u64, constant time | secp256k1 base | 15.6 vs halo2curves 383 | −367 (0.04×) | 11.0 vs crypto-bigint 150 | −139 (0.07×) | 14.6 vs halo2curves 477 | −462 (0.03×) | 18.0 vs halo2curves 237 | −219 (0.08×) |
| sub_u64, constant time | P-256 base | 64.0 vs k256/p256 266 | −202 (0.24×) | 31.8 vs k256/p256 88.8 | −57.0 (0.36×) | 75.4 vs k256/p256 485 | −410 (0.16×) | 53.0 vs k256/p256 149 | −96.0 (0.36×) |
| sub_u64, constant time | P-256 scalar | 88.5 vs k256/p256 21.3 | +67.2 (4.15×) | 37.4 vs k256/p256 21.2 | +16.2 (1.76×) | 108 vs k256/p256 24.4 | +83.6 (4.43×) | 67.7 vs k256/p256 24.0 | +43.7 (2.82×) |
| sub_u64, constant time | Tom-256 base | 93.6 vs crypto-bigint 514 | −420 (0.18×) | 37.7 vs crypto-bigint 151 | −113 (0.25×) | 107 vs halo2curves 697 | −590 (0.15×) | 65.5 vs crypto-bigint 284 | −219 (0.23×) |
| sub_u64, constant time | BLS12-381 base | 148 vs crypto-bigint 1,082 | −934 (0.14×) | 59.6 vs halo2curves 316 | −256 (0.19×) | 203 vs crypto-bigint 1,402 | −1,199 (0.14×) | 105 vs crypto-bigint 617 | −512 (0.17×) |
| sub_u64, constant time | P-521 base | 42.7 vs crypto-bigint 1,665 | −1,622 (0.03×) | 27.6 vs crypto-bigint 620 | −592 (0.04×) | 52.3 vs crypto-bigint 2,183 | −2,131 (0.02×) | 40.1 vs crypto-bigint 1,099 | −1,059 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 5.0 vs crypto-bigint 23.7 | −18.7 (0.21×) | 5.0 vs crypto-bigint 17.9 | −12.9 (0.28×) | 5.2 vs crypto-bigint 31.2 | −26.0 (0.17×) | 5.1 vs crypto-bigint 20.9 | −15.8 (0.24×) |
| sub_uint, constant time | BabyBear | 11.3 vs crypto-bigint 28.0 | −16.7 (0.40×) | 7.6 vs crypto-bigint 19.5 | −11.9 (0.39×) | 14.0 vs crypto-bigint 44.5 | −30.5 (0.31×) | 10.7 vs crypto-bigint 21.8 | −11.1 (0.49×) |
| sub_uint, constant time | Goldilocks | 9.4 vs winter-math 11.5 | −2.1 (0.82×) | 6.7 vs winter-math 7.0 | −0.3 (0.96×) | 11.7 vs winter-math 19.0 | −7.3 (0.62×) | 9.2 vs winter-math 9.5 | −0.3 (0.97×) |
| sub_uint, constant time | BN254 scalar | 266 vs halo2curves 402 | −136 (0.66×) | 96.5 vs halo2curves 104 | −7.5 (0.93×) | 330 vs halo2curves 660 | −330 (0.50×) | 180 vs halo2curves 194 | −14.0 (0.93×) |
| sub_uint, constant time | secp256k1 base | 32.9 vs halo2curves 211 | −178 (0.16×) | 21.5 vs halo2curves 109 | −87.5 (0.20×) | 34.2 vs halo2curves 405 | −371 (0.08×) | 34.8 vs halo2curves 130 | −95.2 (0.27×) |
| sub_uint, constant time | P-256 base | 185 vs halo2curves 231 | −46.0 (0.80×) | 89.8 vs halo2curves 85.9 | +3.9 (1.05×) | 288 vs halo2curves 496 | −208 (0.58×) | 136 vs halo2curves 144 | −8.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 244 vs k256/p256 69.3 | +175 (3.52×) | 100 vs k256/p256 64.9 | +35.1 (1.54×) | 332 vs k256/p256 85.6 | +246 (3.88×) | 180 vs k256/p256 90.3 | +89.7 (1.99×) |
| sub_uint, constant time | Tom-256 base | 251 vs halo2curves 364 | −113 (0.69×) | 92.4 vs halo2curves 103 | −10.6 (0.90×) | 322 vs halo2curves 613 | −291 (0.53×) | 182 vs halo2curves 187 | −5.0 (0.97×) |
| sub_uint, constant time | BLS12-381 base | 761 vs halo2curves 961 | −200 (0.79×) | 202 vs halo2curves 216 | −14.0 (0.94×) | 942 vs crypto-bigint 1,363 | −421 (0.69×) | 377 vs halo2curves 408 | −31.0 (0.92×) |
| sub_uint, constant time | P-521 base | 127 vs crypto-bigint 1,678 | −1,551 (0.08×) | 66.6 vs crypto-bigint 620 | −553 (0.11×) | 123 vs crypto-bigint 2,217 | −2,094 (0.06×) | 108 vs crypto-bigint 1,107 | −999 (0.10×) |
| from_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 18.1 | −13.4 (0.26×) | 4.7 vs crypto-bigint 11.6 | −6.9 (0.41×) | 5.2 vs crypto-bigint 25.2 | −20.0 (0.21×) | 5.2 vs crypto-bigint 13.2 | −8.0 (0.39×) |
| from_u64, constant time | BabyBear | 10.0 vs crypto-bigint 21.6 | −11.6 (0.46×) | 6.8 vs crypto-bigint 12.6 | −5.8 (0.54×) | 13.3 vs crypto-bigint 35.3 | −22.0 (0.38×) | 10.5 vs crypto-bigint 15.5 | −5.0 (0.68×) |
| from_u64, constant time | Goldilocks | 7.4 vs winter-math 11.2 | −3.8 (0.66×) | 6.0 vs winter-math 5.9 | +0.1 (1.02×) | 9.0 vs winter-math 17.0 | −8.0 (0.53×) | 7.1 vs winter-math 7.0 | +0.1 (1.01×) |
| from_u64, constant time | BN254 scalar | 83.8 vs crypto-bigint 432 | −348 (0.19×) | 34.7 vs crypto-bigint 133 | −98.3 (0.26×) | 99.3 vs halo2curves 710 | −611 (0.14×) | 50.0 vs crypto-bigint 204 | −154 (0.25×) |
| from_u64, constant time | secp256k1 base | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) | 3.8 vs k256/p256 5.5 | −1.7 (0.69×) |
| from_u64, constant time | P-256 base | 52.1 vs k256/p256 240 | −188 (0.22×) | 26.6 vs k256/p256 76.0 | −49.4 (0.35×) | 61.1 vs k256/p256 475 | −414 (0.13×) | 38.5 vs k256/p256 130 | −91.5 (0.30×) |
| from_u64, constant time | P-256 scalar | 84.5 vs k256/p256 3.8 | +80.7 (22.24×) | 34.5 vs k256/p256 3.8 | +30.7 (9.08×) | 89.6 vs k256/p256 3.8 | +85.8 (23.58×) | 50.1 vs k256/p256 3.8 | +46.3 (13.18×) |
| from_u64, constant time | Tom-256 base | 82.2 vs crypto-bigint 430 | −348 (0.19×) | 35.4 vs crypto-bigint 133 | −97.6 (0.27×) | 97.7 vs halo2curves 665 | −567 (0.15×) | 49.0 vs crypto-bigint 205 | −156 (0.24×) |
| from_u64, constant time | BLS12-381 base | 147 vs crypto-bigint 972 | −825 (0.15×) | 56.7 vs crypto-bigint 252 | −195 (0.23×) | 194 vs crypto-bigint 1,279 | −1,085 (0.15×) | 90.4 vs crypto-bigint 525 | −435 (0.17×) |
| from_u64, constant time | P-521 base | 6.0 vs crypto-bigint 1,444 | −1,438 (0.00×) | 6.0 vs crypto-bigint 446 | −440 (0.01×) | 6.0 vs crypto-bigint 1,976 | −1,970 (0.00×) | 6.1 vs crypto-bigint 920 | −914 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,281 vs crypto-bigint 28,239 | −15,958 (0.43×) | 9,492 vs crypto-bigint 24,277 | −14,785 (0.39×) | 17,516 vs crypto-bigint 37,707 | −20,191 (0.46×) | 12,945 vs crypto-bigint 30,887 | −17,942 (0.42×) |
| div_u64, constant time | secp256k1 base | 11,984 vs crypto-bigint 28,430 | −16,446 (0.42×) | 9,093 vs crypto-bigint 24,439 | −15,346 (0.37×) | 16,969 vs crypto-bigint 37,656 | −20,687 (0.45×) | 12,950 vs crypto-bigint 30,775 | −17,825 (0.42×) |
| div_u64, constant time | P-256 base | 11,703 vs crypto-bigint 28,646 | −16,943 (0.41×) | 9,231 vs k256/p256 22,808 | −13,577 (0.40×) | 16,841 vs crypto-bigint 37,642 | −20,801 (0.45×) | 12,492 vs crypto-bigint 30,679 | −18,187 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,262 vs crypto-bigint 28,535 | −16,273 (0.43×) | 9,463 vs crypto-bigint 24,323 | −14,860 (0.39×) | 17,475 vs crypto-bigint 37,689 | −20,214 (0.46×) | 13,548 vs crypto-bigint 30,866 | −17,318 (0.44×) |
| div_u64, constant time | Tom-256 base | 12,096 vs crypto-bigint 28,345 | −16,249 (0.43×) | 9,302 vs crypto-bigint 24,276 | −14,974 (0.38×) | 16,729 vs crypto-bigint 37,783 | −21,054 (0.44×) | 13,005 vs crypto-bigint 30,760 | −17,755 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,742 vs crypto-bigint 51,010 | −26,268 (0.49×) | 16,879 vs crypto-bigint 43,327 | −26,448 (0.39×) | 34,783 vs crypto-bigint 67,032 | −32,249 (0.52×) | 24,131 vs crypto-bigint 53,786 | −29,655 (0.45×) |
| div_u64, constant time | P-521 base | 42,369 vs crypto-bigint 107,166 | −64,797 (0.40×) | 25,912 vs crypto-bigint 83,589 | −57,677 (0.31×) | 64,618 vs crypto-bigint 127,364 | −62,746 (0.51×) | 43,159 vs crypto-bigint 100,709 | −57,550 (0.43×) |
| div_uint, constant time | Mersenne-31 | 272 vs runtime-algebra 3,505 | −3,233 (0.08×) | 272 vs runtime-algebra 3,508 | −3,236 (0.08×) | 280 vs runtime-algebra 3,445 | −3,165 (0.08×) | 280 vs runtime-algebra 3,447 | −3,167 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,559 | −3,090 (0.13×) | 465 vs runtime-algebra 3,561 | −3,096 (0.13×) | 474 vs runtime-algebra 3,445 | −2,971 (0.14×) | 442 vs runtime-algebra 3,447 | −3,005 (0.13×) |
| div_uint, constant time | Goldilocks | 1,146 vs winter-math 1,825 | −679 (0.63×) | 862 vs winter-math 945 | −83.0 (0.91×) | 1,437 vs winter-math 1,894 | −457 (0.76×) | 969 vs winter-math 1,053 | −84.0 (0.92×) |
| div_uint, constant time | BN254 scalar | 12,555 vs crypto-bigint 28,881 | −16,326 (0.43×) | 9,501 vs crypto-bigint 24,116 | −14,615 (0.39×) | 17,700 vs crypto-bigint 37,689 | −19,989 (0.47×) | 13,226 vs crypto-bigint 30,836 | −17,610 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,996 vs crypto-bigint 28,431 | −16,435 (0.42×) | 9,093 vs crypto-bigint 24,290 | −15,197 (0.37×) | 16,991 vs crypto-bigint 37,605 | −20,614 (0.45×) | 12,681 vs crypto-bigint 30,703 | −18,022 (0.41×) |
| div_uint, constant time | P-256 base | 11,996 vs crypto-bigint 29,025 | −17,029 (0.41×) | 9,129 vs k256/p256 23,711 | −14,582 (0.39×) | 17,413 vs crypto-bigint 37,745 | −20,332 (0.46×) | 12,560 vs crypto-bigint 30,743 | −18,183 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,334 vs crypto-bigint 28,679 | −16,345 (0.43×) | 9,543 vs crypto-bigint 24,116 | −14,573 (0.40×) | 17,880 vs crypto-bigint 37,826 | −19,946 (0.47×) | 13,834 vs crypto-bigint 30,941 | −17,107 (0.45×) |
| div_uint, constant time | Tom-256 base | 12,388 vs crypto-bigint 28,247 | −15,859 (0.44×) | 9,441 vs crypto-bigint 24,210 | −14,769 (0.39×) | 17,218 vs crypto-bigint 37,723 | −20,505 (0.46×) | 13,151 vs crypto-bigint 30,820 | −17,669 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,420 vs crypto-bigint 51,313 | −25,893 (0.50×) | 17,124 vs crypto-bigint 43,574 | −26,450 (0.39×) | 35,327 vs crypto-bigint 66,997 | −31,670 (0.53×) | 24,762 vs crypto-bigint 53,623 | −28,861 (0.46×) |
| div_uint, constant time | P-521 base | 42,588 vs crypto-bigint 104,619 | −62,031 (0.41×) | 26,212 vs crypto-bigint 84,021 | −57,809 (0.31×) | 65,037 vs crypto-bigint 127,288 | −62,251 (0.51×) | 43,022 vs crypto-bigint 100,555 | −57,533 (0.43×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add | BN254 scalar | 13.3 vs ark-ff 13.4 | −0.1 (0.99×) | 10.9 vs ark-ff 11.1 | −0.2 (0.98×) | 13.4 vs ark-ff 13.7 | −0.3 (0.98×) | 18.0 vs ark-ff 16.4 | +1.6 (1.10×) |
| add | secp256k1 base | 13.7 vs ark-ff 15.5 | −1.8 (0.88×) | 10.7 vs ark-ff 11.2 | −0.5 (0.96×) | 13.3 vs ark-ff 15.0 | −1.7 (0.89×) | 18.8 vs ark-ff 17.9 | +0.9 (1.05×) |
| add | P-256 base | 12.4 vs ark-ff 15.2 | −2.8 (0.82×) | 11.3 vs ark-ff 11.2 | +0.1 (1.01×) | 12.9 vs ark-ff 15.6 | −2.7 (0.83×) | 18.4 vs ark-ff 18.9 | −0.5 (0.97×) |
| add | P-256 scalar | 13.6 vs ark-ff 15.3 | −1.7 (0.89×) | 11.1 vs ark-ff 11.2 | −0.1 (0.99×) | 14.0 vs ark-ff 15.9 | −1.9 (0.88×) | 19.0 vs ark-ff 18.3 | +0.7 (1.04×) |
| add | Tom-256 base | 13.4 vs halo2curves 43.1 | −29.7 (0.31×) | 11.2 vs halo2curves 24.0 | −12.8 (0.47×) | 13.5 vs halo2curves 42.6 | −29.1 (0.32×) | 18.6 vs halo2curves 32.5 | −13.9 (0.57×) |
| add | BLS12-381 base | 21.9 vs ark-ff 44.1 | −22.2 (0.50×) | 18.7 vs halo2curves 34.0 | −15.3 (0.55×) | 23.0 vs ark-ff 47.1 | −24.1 (0.49×) | 29.8 vs ark-ff 46.4 | −16.6 (0.64×) |
| add | P-521 base | 42.2 vs crypto-bigint 181 | −139 (0.23×) | 27.2 vs crypto-bigint 179 | −152 (0.15×) | 48.6 vs crypto-bigint 245 | −196 (0.20×) | 47.9 vs crypto-bigint 246 | −198 (0.19×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 13.2 vs ark-ff 16.5 | −3.3 (0.80×) | 10.8 vs ark-ff 13.9 | −3.1 (0.78×) | 12.4 vs ark-ff 17.0 | −4.6 (0.73×) | 17.4 vs ark-ff 21.4 | −4.0 (0.81×) |
| sub | secp256k1 base | 16.6 vs ark-ff 16.7 | −0.1 (0.99×) | 10.4 vs ark-ff 13.9 | −3.5 (0.75×) | 13.9 vs ark-ff 17.2 | −3.3 (0.81×) | 16.8 vs halo2curves 21.0 | −4.2 (0.80×) |
| sub | P-256 base | 13.4 vs ark-ff 16.4 | −3.0 (0.82×) | 10.4 vs ark-ff 12.9 | −2.5 (0.81×) | 12.4 vs ark-ff 16.0 | −3.6 (0.78×) | 16.4 vs k256/p256 20.0 | −3.6 (0.82×) |
| sub | P-256 scalar | 14.3 vs ark-ff 16.6 | −2.3 (0.86×) | 10.4 vs ark-ff 13.9 | −3.5 (0.75×) | 13.9 vs ark-ff 16.7 | −2.8 (0.83×) | 17.3 vs ark-ff 21.4 | −4.1 (0.81×) |
| sub | Tom-256 base | 13.1 vs halo2curves 26.1 | −13.0 (0.50×) | 10.7 vs halo2curves 14.8 | −4.1 (0.72×) | 12.4 vs halo2curves 26.5 | −14.1 (0.47×) | 17.3 vs halo2curves 22.0 | −4.7 (0.79×) |
| sub | BLS12-381 base | 22.8 vs ark-ff 30.5 | −7.7 (0.75×) | 17.3 vs ark-ff 23.8 | −6.5 (0.73×) | 22.7 vs ark-ff 30.7 | −8.0 (0.74×) | 29.5 vs halo2curves 36.1 | −6.6 (0.82×) |
| sub | P-521 base | 43.5 vs crypto-bigint 159 | −116 (0.27×) | 25.7 vs crypto-bigint 144 | −118 (0.18×) | 44.9 vs crypto-bigint 193 | −148 (0.23×) | 44.0 vs crypto-bigint 225 | −181 (0.20×) |
| mul | Mersenne-31 | 7.0 vs crypto-bigint 46.6 | −39.6 (0.15×) | 7.0 vs crypto-bigint 27.0 | −20.0 (0.26×) | 7.0 vs crypto-bigint 50.3 | −43.3 (0.14×) | 7.0 vs crypto-bigint 30.6 | −23.6 (0.23×) |
| mul | BabyBear | 12.0 vs plonky3 12.0 | +0.0 (1.00×) | 11.0 vs plonky3 12.2 | −1.2 (0.90×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| mul | Goldilocks | 16.6 vs winter-math 18.1 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 12.3 vs winter-math 14.0 | −1.7 (0.88×) |
| mul | BN254 scalar | 227 vs ark-ff 320 | −93.0 (0.71×) | 88.5 vs halo2curves 95.0 | −6.5 (0.93×) | 368 vs ark-ff 383 | −15.0 (0.96×) | 153 vs halo2curves 178 | −25.0 (0.86×) |
| mul | secp256k1 base | 174 vs ark-ff 275 | −101 (0.63×) | 69.2 vs halo2curves 90.3 | −21.1 (0.77×) | 241 vs ark-ff 401 | −160 (0.60×) | 106 vs halo2curves 159 | −53.0 (0.67×) |
| mul | P-256 base | 190 vs k256/p256 243 | −53.0 (0.78×) | 65.0 vs halo2curves 73.7 | −8.7 (0.88×) | 292 vs ark-ff 441 | −149 (0.66×) | 118 vs halo2curves 130 | −12.0 (0.91×) |
| mul | P-256 scalar | 245 vs ark-ff 314 | −69.0 (0.78×) | 88.0 vs halo2curves 95.9 | −7.9 (0.92×) | 326 vs ark-ff 405 | −79.0 (0.80×) | 160 vs halo2curves 176 | −16.0 (0.91×) |
| mul | Tom-256 base | 234 vs halo2curves 347 | −113 (0.67×) | 80.7 vs halo2curves 93.9 | −13.2 (0.86×) | 314 vs halo2curves 591 | −277 (0.53×) | 158 vs halo2curves 171 | −13.0 (0.92×) |
| mul | BLS12-381 base | 722 vs ark-ff 690 | +32.0 (1.05×) | 187 vs halo2curves 192 | −5.0 (0.97×) | 928 vs ark-ff 1,050 | −122 (0.88×) | 323 vs halo2curves 379 | −56.0 (0.85×) |
| mul | P-521 base | 686 vs crypto-bigint 2,151 | −1,465 (0.32×) | 240 vs crypto-bigint 618 | −378 (0.39×) | 1,294 vs crypto-bigint 2,755 | −1,461 (0.47×) | 439 vs crypto-bigint 1,101 | −662 (0.40×) |
| square | Mersenne-31 | 7.0 vs crypto-bigint 45.4 | −38.4 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.6 | −43.6 (0.14×) | 7.0 vs crypto-bigint 30.8 | −23.8 (0.23×) |
| square | BabyBear | 12.0 vs plonky3 12.1 | −0.1 (0.99×) | 11.0 vs plonky3 12.0 | −1.0 (0.92×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) | 11.0 vs plonky3 13.0 | −2.0 (0.85×) |
| square | Goldilocks | 15.0 vs winter-math 19.5 | −4.5 (0.77×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.6 vs winter-math 19.0 | −0.4 (0.98×) | 12.5 vs winter-math 14.7 | −2.2 (0.85×) |
| square | BN254 scalar | 210 vs ark-ff 265 | −55.0 (0.79×) | 81.4 vs halo2curves 97.0 | −15.6 (0.84×) | 324 vs ark-ff 346 | −22.0 (0.94×) | 133 vs halo2curves 143 | −10.0 (0.93×) |
| square | secp256k1 base | 152 vs halo2curves 214 | −62.0 (0.71×) | 58.9 vs halo2curves 82.8 | −23.9 (0.71×) | 160 vs ark-ff 295 | −135 (0.54×) | 88.3 vs halo2curves 125 | −36.7 (0.71×) |
| square | P-256 base | 160 vs halo2curves 193 | −33.0 (0.83×) | 55.6 vs halo2curves 62.6 | −7.0 (0.89×) | 246 vs ark-ff 327 | −81.0 (0.75×) | 108 vs halo2curves 109 | −1.0 (0.99×) |
| square | P-256 scalar | 220 vs ark-ff 260 | −40.0 (0.85×) | 81.7 vs halo2curves 92.2 | −10.5 (0.89×) | 242 vs ark-ff 343 | −101 (0.71×) | 141 vs halo2curves 142 | −1.0 (0.99×) |
| square | Tom-256 base | 213 vs halo2curves 280 | −67.0 (0.76×) | 77.7 vs halo2curves 89.1 | −11.4 (0.87×) | 210 vs halo2curves 476 | −266 (0.44×) | 124 vs halo2curves 130 | −6.0 (0.95×) |
| square | BLS12-381 base | 569 vs ark-ff 620 | −51.0 (0.92×) | 161 vs halo2curves 162 | −1.0 (0.99×) | 800 vs ark-ff 818 | −18.0 (0.98×) | 276 vs halo2curves 292 | −16.0 (0.95×) |
| square | P-521 base | 571 vs crypto-bigint 2,174 | −1,603 (0.26×) | 197 vs crypto-bigint 627 | −430 (0.31×) | 905 vs crypto-bigint 2,832 | −1,927 (0.32×) | 387 vs crypto-bigint 1,127 | −740 (0.34×) |
| mul_u64 | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 31.6 | −22.6 (0.28×) | 9.0 vs crypto-bigint 73.5 | −64.5 (0.12×) | 9.0 vs crypto-bigint 38.7 | −29.7 (0.23×) |
| mul_u64 | BabyBear | 16.1 vs plonky3 34.3 | −18.2 (0.47×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.7 vs plonky3 45.2 | −29.5 (0.35×) | 13.0 vs plonky3 16.6 | −3.6 (0.78×) |
| mul_u64 | Goldilocks | 17.8 vs winter-math 24.0 | −6.2 (0.74×) | 14.1 vs winter-math 13.7 | +0.4 (1.03×) | 25.3 vs winter-math 34.1 | −8.8 (0.74×) | 15.1 vs winter-math 14.7 | +0.4 (1.03×) |
| mul_u64 | BN254 scalar | 91.7 vs ark-ff 647 | −555 (0.14×) | 34.5 vs crypto-bigint 258 | −224 (0.13×) | 120 vs ark-ff 803 | −683 (0.15×) | 45.3 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | secp256k1 base | 49.0 vs ark-ff 436 | −387 (0.11×) | 16.0 vs crypto-bigint 258 | −242 (0.06×) | 65.0 vs ark-ff 614 | −549 (0.11×) | 23.0 vs crypto-bigint 410 | −387 (0.06×) |
| mul_u64 | P-256 base | 75.0 vs k256/p256 487 | −412 (0.15×) | 33.0 vs k256/p256 154 | −121 (0.21×) | 89.2 vs ark-ff 853 | −764 (0.10×) | 41.8 vs k256/p256 257 | −215 (0.16×) |
| mul_u64 | P-256 scalar | 82.7 vs k256/p256 508 | −425 (0.16×) | 34.5 vs k256/p256 255 | −221 (0.14×) | 105 vs k256/p256 815 | −710 (0.13×) | 47.4 vs k256/p256 339 | −292 (0.14×) |
| mul_u64 | Tom-256 base | 84.5 vs crypto-bigint 834 | −750 (0.10×) | 33.0 vs crypto-bigint 258 | −225 (0.13×) | 105 vs halo2curves 1,273 | −1,168 (0.08×) | 45.4 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | BLS12-381 base | 125 vs ark-ff 1,626 | −1,501 (0.08×) | 47.9 vs halo2curves 488 | −440 (0.10×) | 177 vs ark-ff 2,129 | −1,952 (0.08×) | 62.1 vs crypto-bigint 1,039 | −977 (0.06×) |
| mul_u64 | P-521 base | 96.8 vs crypto-bigint 3,050 | −2,953 (0.03×) | 39.5 vs crypto-bigint 1,002 | −963 (0.04×) | 196 vs crypto-bigint 4,041 | −3,845 (0.05×) | 52.8 vs crypto-bigint 1,860 | −1,807 (0.03×) |
| mul_uint | Mersenne-31 | 9.0 vs crypto-bigint 60.0 | −51.0 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 73.0 | −64.0 (0.12×) | 9.0 vs crypto-bigint 39.0 | −30.0 (0.23×) |
| mul_uint | BabyBear | 16.2 vs plonky3 23.7 | −7.5 (0.68×) | 11.0 vs plonky3 13.3 | −2.3 (0.83×) | 15.7 vs plonky3 34.1 | −18.4 (0.46×) | 13.0 vs plonky3 16.6 | −3.6 (0.78×) |
| mul_uint | Goldilocks | 17.8 vs winter-math 24.3 | −6.5 (0.73×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.3 vs winter-math 34.0 | −8.7 (0.74×) | 15.0 vs winter-math 14.5 | +0.5 (1.03×) |
| mul_uint | BN254 scalar | 441 vs halo2curves 762 | −321 (0.58×) | 162 vs halo2curves 180 | −18.0 (0.90×) | 497 vs halo2curves 1,343 | −846 (0.37×) | 298 vs crypto-bigint 407 | −109 (0.73×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 526 | −260 (0.51×) | 68.9 vs halo2curves 154 | −85.1 (0.45×) | 320 vs halo2curves 952 | −632 (0.34×) | 106 vs halo2curves 264 | −158 (0.40×) |
| mul_uint | P-256 base | 352 vs halo2curves 460 | −108 (0.77×) | 127 vs halo2curves 145 | −18.0 (0.88×) | 424 vs halo2curves 940 | −516 (0.45×) | 223 vs halo2curves 251 | −28.0 (0.89×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 549 | −124 (0.77×) | 162 vs halo2curves 185 | −23.0 (0.88×) | 479 vs k256/p256 883 | −404 (0.54×) | 315 vs halo2curves 346 | −31.0 (0.91×) |
| mul_uint | Tom-256 base | 410 vs halo2curves 694 | −284 (0.59×) | 160 vs halo2curves 180 | −20.0 (0.89×) | 488 vs halo2curves 1,179 | −691 (0.41×) | 305 vs halo2curves 341 | −36.0 (0.89×) |
| mul_uint | BLS12-381 base | 974 vs halo2curves 1,899 | −925 (0.51×) | 352 vs halo2curves 379 | −27.0 (0.93×) | 1,139 vs crypto-bigint 2,547 | −1,408 (0.45×) | 889 vs crypto-bigint 1,037 | −148 (0.86×) |
| mul_uint | P-521 base | 1,567 vs crypto-bigint 3,068 | −1,501 (0.51×) | 248 vs crypto-bigint 1,009 | −761 (0.25×) | 2,381 vs crypto-bigint 3,996 | −1,615 (0.60×) | 458 vs crypto-bigint 1,871 | −1,413 (0.24×) |
| add_u64 | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.7 vs crypto-bigint 43.6 | −38.9 (0.11×) | 4.8 vs crypto-bigint 29.8 | −25.0 (0.16×) |
| add_u64 | BabyBear | 12.2 vs plonky3 30.8 | −18.6 (0.40×) | 8.2 vs plonky3 9.2 | −1.0 (0.89×) | 12.7 vs plonky3 41.9 | −29.2 (0.30×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_u64 | Goldilocks | 8.5 vs winter-math 13.0 | −4.5 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 12.0 vs winter-math 19.1 | −7.1 (0.63×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_u64 | BN254 scalar | 113 vs ark-ff 343 | −230 (0.33×) | 42.9 vs halo2curves 212 | −169 (0.20×) | 138 vs ark-ff 418 | −280 (0.33×) | 65.6 vs halo2curves 300 | −234 (0.22×) |
| add_u64 | secp256k1 base | 7.0 vs ark-ff 159 | −152 (0.04×) | 6.0 vs ark-ff 118 | −112 (0.05×) | 7.0 vs ark-ff 220 | −213 (0.03×) | 10.0 vs ark-ff 183 | −173 (0.05×) |
| add_u64 | P-256 base | 67.5 vs ark-ff 263 | −196 (0.26×) | 34.8 vs k256/p256 92.5 | −57.7 (0.38×) | 77.7 vs ark-ff 397 | −319 (0.20×) | 56.3 vs k256/p256 158 | −102 (0.36×) |
| add_u64 | P-256 scalar | 66.8 vs k256/p256 56.3 | +10.5 (1.19×) | 38.0 vs k256/p256 56.3 | −18.3 (0.67×) | 74.7 vs k256/p256 62.1 | +12.6 (1.20×) | 65.1 vs k256/p256 62.0 | +3.1 (1.05×) |
| add_u64 | Tom-256 base | 84.5 vs crypto-bigint 508 | −424 (0.17×) | 39.6 vs halo2curves 217 | −177 (0.18×) | 99.6 vs halo2curves 686 | −586 (0.15×) | 66.0 vs halo2curves 293 | −227 (0.23×) |
| add_u64 | BLS12-381 base | 161 vs ark-ff 829 | −668 (0.19×) | 62.2 vs halo2curves 322 | −260 (0.19×) | 254 vs ark-ff 1,087 | −833 (0.23×) | 89.4 vs halo2curves 642 | −553 (0.14×) |
| add_u64 | P-521 base | 21.1 vs crypto-bigint 1,690 | −1,669 (0.01×) | 12.2 vs crypto-bigint 653 | −641 (0.02×) | 19.4 vs crypto-bigint 2,209 | −2,190 (0.01×) | 21.4 vs crypto-bigint 1,167 | −1,146 (0.02×) |
| add_uint | Mersenne-31 | 4.7 vs crypto-bigint 35.5 | −30.8 (0.13×) | 4.7 vs crypto-bigint 30.1 | −25.4 (0.16×) | 4.7 vs crypto-bigint 41.9 | −37.2 (0.11×) | 4.7 vs crypto-bigint 30.5 | −25.8 (0.15×) |
| add_uint | BabyBear | 12.2 vs plonky3 21.5 | −9.3 (0.57×) | 8.2 vs plonky3 9.1 | −0.9 (0.90×) | 12.7 vs plonky3 31.4 | −18.7 (0.40×) | 11.0 vs plonky3 13.1 | −2.1 (0.84×) |
| add_uint | Goldilocks | 8.5 vs winter-math 13.2 | −4.7 (0.64×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.7 vs winter-math 19.1 | −7.4 (0.61×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_uint | BN254 scalar | 226 vs halo2curves 410 | −184 (0.55×) | 96.8 vs halo2curves 112 | −15.2 (0.86×) | 370 vs halo2curves 690 | −320 (0.54×) | 179 vs halo2curves 206 | −27.0 (0.87×) |
| add_uint | secp256k1 base | 17.5 vs halo2curves 228 | −211 (0.08×) | 12.5 vs halo2curves 85.1 | −72.6 (0.15×) | 16.8 vs halo2curves 417 | −400 (0.04×) | 23.1 vs halo2curves 140 | −117 (0.17×) |
| add_uint | P-256 base | 180 vs halo2curves 250 | −70.0 (0.72×) | 73.5 vs halo2curves 91.0 | −17.5 (0.81×) | 325 vs halo2curves 507 | −182 (0.64×) | 147 vs halo2curves 153 | −6.0 (0.96×) |
| add_uint | P-256 scalar | 251 vs k256/p256 84.0 | +167 (2.99×) | 99.1 vs k256/p256 82.6 | +16.5 (1.20×) | 322 vs k256/p256 108 | +214 (2.98×) | 186 vs k256/p256 108 | +78.0 (1.72×) |
| add_uint | Tom-256 base | 257 vs halo2curves 375 | −118 (0.69×) | 96.0 vs halo2curves 110 | −14.0 (0.87×) | 327 vs halo2curves 626 | −299 (0.52×) | 180 vs halo2curves 196 | −16.0 (0.92×) |
| add_uint | BLS12-381 base | 764 vs halo2curves 961 | −197 (0.80×) | 195 vs halo2curves 224 | −29.0 (0.87×) | 942 vs crypto-bigint 1,415 | −473 (0.67×) | 389 vs halo2curves 425 | −36.0 (0.92×) |
| add_uint | P-521 base | 62.0 vs crypto-bigint 1,712 | −1,650 (0.04×) | 41.6 vs crypto-bigint 666 | −624 (0.06×) | 85.0 vs crypto-bigint 2,217 | −2,132 (0.04×) | 66.9 vs crypto-bigint 1,182 | −1,115 (0.06×) |
| sub_u64 | Mersenne-31 | 4.7 vs crypto-bigint 22.4 | −17.7 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.5 | −25.6 (0.16×) | 4.9 vs crypto-bigint 19.8 | −14.9 (0.25×) |
| sub_u64 | BabyBear | 11.9 vs crypto-bigint 28.9 | −17.0 (0.41×) | 8.2 vs plonky3 9.1 | −0.9 (0.90×) | 12.0 vs plonky3 41.6 | −29.6 (0.29×) | 10.8 vs plonky3 13.2 | −2.4 (0.82×) |
| sub_u64 | Goldilocks | 8.2 vs winter-math 11.2 | −3.0 (0.73×) | 6.1 vs winter-math 6.9 | −0.8 (0.88×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 8.7 vs winter-math 8.6 | +0.1 (1.01×) |
| sub_u64 | BN254 scalar | 113 vs ark-ff 346 | −233 (0.33×) | 44.9 vs crypto-bigint 152 | −107 (0.30×) | 136 vs ark-ff 400 | −264 (0.34×) | 66.5 vs crypto-bigint 286 | −220 (0.23×) |
| sub_u64 | secp256k1 base | 7.0 vs ark-ff 156 | −149 (0.04×) | 6.0 vs ark-ff 119 | −113 (0.05×) | 7.0 vs ark-ff 221 | −214 (0.03×) | 9.0 vs ark-ff 183 | −174 (0.05×) |
| sub_u64 | P-256 base | 69.5 vs ark-ff 266 | −197 (0.26×) | 34.1 vs k256/p256 88.1 | −54.0 (0.39×) | 78.1 vs ark-ff 393 | −315 (0.20×) | 54.3 vs k256/p256 149 | −94.7 (0.36×) |
| sub_u64 | P-256 scalar | 66.8 vs k256/p256 21.3 | +45.5 (3.14×) | 36.6 vs k256/p256 21.2 | +15.4 (1.73×) | 72.6 vs k256/p256 24.7 | +47.9 (2.94×) | 61.5 vs k256/p256 24.7 | +36.8 (2.49×) |
| sub_u64 | Tom-256 base | 85.4 vs crypto-bigint 484 | −399 (0.18×) | 39.5 vs crypto-bigint 151 | −112 (0.26×) | 96.7 vs halo2curves 692 | −595 (0.14×) | 64.3 vs halo2curves 285 | −221 (0.23×) |
| sub_u64 | BLS12-381 base | 155 vs ark-ff 829 | −674 (0.19×) | 60.2 vs halo2curves 318 | −258 (0.19×) | 264 vs ark-ff 1,065 | −801 (0.25×) | 89.8 vs crypto-bigint 623 | −533 (0.14×) |
| sub_u64 | P-521 base | 21.2 vs crypto-bigint 1,669 | −1,648 (0.01×) | 13.1 vs crypto-bigint 637 | −624 (0.02×) | 21.6 vs crypto-bigint 2,214 | −2,192 (0.01×) | 20.7 vs crypto-bigint 1,110 | −1,089 (0.02×) |
| sub_uint | Mersenne-31 | 4.7 vs crypto-bigint 22.2 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.7 | −25.8 (0.16×) | 4.9 vs crypto-bigint 20.4 | −15.5 (0.24×) |
| sub_uint | BabyBear | 11.9 vs plonky3 22.1 | −10.2 (0.54×) | 8.2 vs plonky3 9.0 | −0.8 (0.91×) | 12.0 vs plonky3 31.2 | −19.2 (0.38×) | 10.9 vs plonky3 13.3 | −2.4 (0.82×) |
| sub_uint | Goldilocks | 8.2 vs winter-math 11.1 | −2.9 (0.74×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 10.5 vs winter-math 18.1 | −7.6 (0.58×) | 8.7 vs winter-math 8.6 | +0.1 (1.01×) |
| sub_uint | BN254 scalar | 233 vs halo2curves 396 | −163 (0.59×) | 92.7 vs halo2curves 105 | −12.3 (0.88×) | 349 vs halo2curves 670 | −321 (0.52×) | 177 vs halo2curves 194 | −17.0 (0.91×) |
| sub_uint | secp256k1 base | 19.0 vs halo2curves 210 | −191 (0.09×) | 12.0 vs halo2curves 80.7 | −68.7 (0.15×) | 16.4 vs halo2curves 399 | −383 (0.04×) | 20.9 vs halo2curves 130 | −109 (0.16×) |
| sub_uint | P-256 base | 177 vs halo2curves 235 | −58.0 (0.75×) | 71.0 vs halo2curves 85.7 | −14.7 (0.83×) | 251 vs halo2curves 499 | −248 (0.50×) | 133 vs halo2curves 143 | −10.0 (0.93×) |
| sub_uint | P-256 scalar | 261 vs k256/p256 72.0 | +189 (3.62×) | 95.9 vs k256/p256 72.4 | +23.5 (1.32×) | 328 vs k256/p256 95.4 | +233 (3.44×) | 175 vs k256/p256 95.6 | +79.4 (1.83×) |
| sub_uint | Tom-256 base | 252 vs halo2curves 365 | −113 (0.69×) | 93.1 vs halo2curves 103 | −9.9 (0.90×) | 324 vs halo2curves 612 | −288 (0.53×) | 172 vs halo2curves 187 | −15.0 (0.92×) |
| sub_uint | BLS12-381 base | 756 vs halo2curves 953 | −197 (0.79×) | 197 vs halo2curves 217 | −20.0 (0.91×) | 937 vs crypto-bigint 1,382 | −445 (0.68×) | 394 vs halo2curves 485 | −91.0 (0.81×) |
| sub_uint | P-521 base | 67.4 vs crypto-bigint 1,687 | −1,620 (0.04×) | 37.2 vs crypto-bigint 638 | −601 (0.06×) | 80.1 vs crypto-bigint 2,202 | −2,122 (0.04×) | 65.7 vs crypto-bigint 1,135 | −1,069 (0.06×) |
| div_u64 | BN254 scalar | 1,205 vs halo2curves 13,493 | −12,288 (0.09×) | 838 vs halo2curves 6,973 | −6,135 (0.12×) | 1,372 vs halo2curves 19,259 | −17,887 (0.07×) | 1,115 vs halo2curves 11,850 | −10,735 (0.09×) |
| div_u64 | secp256k1 base | 1,015 vs halo2curves 13,077 | −12,062 (0.08×) | 717 vs halo2curves 6,953 | −6,236 (0.10×) | 1,224 vs halo2curves 18,931 | −17,707 (0.06×) | 821 vs halo2curves 11,643 | −10,822 (0.07×) |
| div_u64 | P-256 base | 1,085 vs halo2curves 13,099 | −12,014 (0.08×) | 797 vs halo2curves 6,929 | −6,132 (0.12×) | 1,343 vs halo2curves 19,032 | −17,689 (0.07×) | 977 vs halo2curves 11,604 | −10,627 (0.08×) |
| div_u64 | P-256 scalar | 1,187 vs halo2curves 13,382 | −12,195 (0.09×) | 844 vs halo2curves 6,965 | −6,121 (0.12×) | 1,397 vs halo2curves 19,380 | −17,983 (0.07×) | 1,099 vs halo2curves 11,814 | −10,715 (0.09×) |
| div_u64 | Tom-256 base | 1,201 vs halo2curves 13,301 | −12,100 (0.09×) | 840 vs halo2curves 6,941 | −6,101 (0.12×) | 1,496 vs halo2curves 19,089 | −17,593 (0.08×) | 1,034 vs halo2curves 11,835 | −10,801 (0.09×) |
| div_u64 | BLS12-381 base | 1,801 vs halo2curves 24,023 | −22,222 (0.07×) | 1,074 vs halo2curves 11,036 | −9,962 (0.10×) | 2,459 vs halo2curves 32,547 | −30,088 (0.08×) | 1,752 vs halo2curves 18,783 | −17,031 (0.09×) |
| div_u64 | P-521 base | 2,453 vs crypto-bigint 62,284 | −59,831 (0.04×) | 999 vs crypto-bigint 45,178 | −44,179 (0.02×) | 3,626 vs crypto-bigint 77,792 | −74,166 (0.05×) | 1,513 vs crypto-bigint 57,953 | −56,440 (0.03×) |
| div_uint | Mersenne-31 | 274 vs crypto-bigint 1,806 | −1,532 (0.15×) | 273 vs crypto-bigint 1,611 | −1,338 (0.17×) | 278 vs runtime-algebra 2,117 | −1,839 (0.13×) | 281 vs crypto-bigint 2,005 | −1,724 (0.14×) |
| div_uint | BabyBear | 275 vs plonky3 680 | −405 (0.40×) | 278 vs plonky3 654 | −376 (0.43×) | 277 vs plonky3 711 | −434 (0.39×) | 280 vs plonky3 703 | −423 (0.40×) |
| div_uint | Goldilocks | 549 vs winter-math 1,810 | −1,261 (0.30×) | 539 vs winter-math 928 | −389 (0.58×) | 583 vs winter-math 1,899 | −1,316 (0.31×) | 571 vs winter-math 1,033 | −462 (0.55×) |
| div_uint | BN254 scalar | 9,123 vs halo2curves 15,118 | −5,995 (0.60×) | 6,619 vs halo2curves 8,187 | −1,568 (0.81×) | 12,788 vs halo2curves 23,052 | −10,264 (0.55×) | 8,467 vs halo2curves 14,628 | −6,161 (0.58×) |
| div_uint | secp256k1 base | 8,564 vs halo2curves 14,843 | −6,279 (0.58×) | 6,015 vs halo2curves 8,228 | −2,213 (0.73×) | 11,939 vs halo2curves 22,495 | −10,556 (0.53×) | 8,053 vs halo2curves 14,547 | −6,494 (0.55×) |
| div_uint | P-256 base | 8,755 vs halo2curves 14,857 | −6,102 (0.59×) | 6,139 vs halo2curves 8,208 | −2,069 (0.75×) | 12,180 vs halo2curves 22,811 | −10,631 (0.53×) | 7,993 vs halo2curves 14,516 | −6,523 (0.55×) |
| div_uint | P-256 scalar | 9,091 vs halo2curves 15,083 | −5,992 (0.60×) | 6,497 vs halo2curves 8,271 | −1,774 (0.79×) | 12,657 vs halo2curves 23,169 | −10,512 (0.55×) | 8,634 vs halo2curves 14,706 | −6,072 (0.59×) |
| div_uint | Tom-256 base | 8,995 vs halo2curves 15,029 | −6,034 (0.60×) | 6,532 vs halo2curves 8,256 | −1,724 (0.79×) | 12,246 vs halo2curves 23,026 | −10,780 (0.53×) | 8,458 vs halo2curves 14,744 | −6,286 (0.57×) |
| div_uint | BLS12-381 base | 20,789 vs halo2curves 27,559 | −6,770 (0.75×) | 12,136 vs halo2curves 13,603 | −1,467 (0.89×) | 26,283 vs halo2curves 40,669 | −14,386 (0.65×) | 16,833 vs halo2curves 24,649 | −7,816 (0.68×) |
| div_uint | P-521 base | 34,468 vs crypto-bigint 68,326 | −33,858 (0.50×) | 18,158 vs crypto-bigint 51,658 | −33,500 (0.35×) | 49,587 vs crypto-bigint 84,402 | −34,815 (0.59×) | 30,275 vs crypto-bigint 63,843 | −33,568 (0.47×) |
| add, constant time | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.8 | −14.8 (0.17×) | 3.0 vs runtime-algebra 17.7 | −14.7 (0.17×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add, constant time | BN254 scalar | 24.3 vs halo2curves 42.9 | −18.6 (0.57×) | 15.7 vs halo2curves 23.2 | −7.5 (0.68×) | 23.2 vs halo2curves 45.1 | −21.9 (0.51×) | 26.0 vs halo2curves 34.1 | −8.1 (0.76×) |
| add, constant time | secp256k1 base | 28.3 vs halo2curves 39.9 | −11.6 (0.71×) | 15.4 vs halo2curves 23.0 | −7.6 (0.67×) | 26.8 vs halo2curves 38.1 | −11.3 (0.70×) | 28.4 vs halo2curves 31.0 | −2.6 (0.92×) |
| add, constant time | P-256 base | 26.5 vs halo2curves 38.9 | −12.4 (0.68×) | 15.0 vs halo2curves 21.0 | −6.0 (0.71×) | 25.0 vs halo2curves 39.0 | −14.0 (0.64×) | 27.5 vs k256/p256 29.6 | −2.1 (0.93×) |
| add, constant time | P-256 scalar | 27.3 vs halo2curves 42.2 | −14.9 (0.65×) | 17.9 vs halo2curves 24.7 | −6.8 (0.72×) | 27.2 vs halo2curves 42.5 | −15.3 (0.64×) | 29.1 vs halo2curves 33.0 | −3.9 (0.88×) |
| add, constant time | Tom-256 base | 27.7 vs halo2curves 43.1 | −15.4 (0.64×) | 15.9 vs halo2curves 24.0 | −8.1 (0.66×) | 27.0 vs halo2curves 42.6 | −15.6 (0.63×) | 29.0 vs halo2curves 32.5 | −3.5 (0.89×) |
| add, constant time | BLS12-381 base | 41.3 vs halo2curves 63.6 | −22.3 (0.65×) | 23.4 vs halo2curves 34.0 | −10.6 (0.69×) | 35.4 vs halo2curves 67.1 | −31.7 (0.53×) | 40.7 vs halo2curves 54.0 | −13.3 (0.75×) |
| add, constant time | P-521 base | 77.5 vs crypto-bigint 181 | −104 (0.43×) | 39.8 vs crypto-bigint 179 | −139 (0.22×) | 80.5 vs crypto-bigint 245 | −165 (0.33×) | 72.1 vs crypto-bigint 246 | −174 (0.29×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.6 vs halo2curves 26.0 | −4.4 (0.83×) | 13.3 vs halo2curves 15.0 | −1.7 (0.89×) | 20.8 vs halo2curves 25.6 | −4.8 (0.81×) | 21.5 vs halo2curves 23.0 | −1.5 (0.93×) |
| sub, constant time | secp256k1 base | 21.4 vs halo2curves 25.3 | −3.9 (0.85×) | 12.4 vs halo2curves 14.5 | −2.1 (0.86×) | 20.5 vs halo2curves 24.0 | −3.5 (0.85×) | 21.0 vs halo2curves 21.0 | +0.0 (1.00×) |
| sub, constant time | P-256 base | 19.2 vs halo2curves 25.2 | −6.0 (0.76×) | 12.6 vs halo2curves 14.1 | −1.5 (0.89×) | 18.5 vs halo2curves 24.0 | −5.5 (0.77×) | 19.0 vs k256/p256 20.0 | −1.0 (0.95×) |
| sub, constant time | P-256 scalar | 21.7 vs k256/p256 24.8 | −3.1 (0.88×) | 13.6 vs halo2curves 14.8 | −1.2 (0.92×) | 20.6 vs k256/p256 27.0 | −6.4 (0.76×) | 21.0 vs halo2curves 22.0 | −1.0 (0.95×) |
| sub, constant time | Tom-256 base | 21.4 vs halo2curves 26.1 | −4.7 (0.82×) | 13.3 vs halo2curves 14.8 | −1.5 (0.90×) | 19.7 vs halo2curves 26.5 | −6.8 (0.74×) | 21.5 vs halo2curves 22.0 | −0.5 (0.98×) |
| sub, constant time | BLS12-381 base | 37.3 vs halo2curves 45.4 | −8.1 (0.82×) | 22.0 vs halo2curves 25.1 | −3.1 (0.88×) | 37.0 vs halo2curves 44.1 | −7.1 (0.84×) | 35.1 vs halo2curves 36.1 | −1.0 (0.97×) |
| sub, constant time | P-521 base | 57.6 vs crypto-bigint 159 | −101 (0.36×) | 31.8 vs crypto-bigint 144 | −112 (0.22×) | 60.1 vs crypto-bigint 193 | −133 (0.31×) | 51.1 vs crypto-bigint 225 | −174 (0.23×) |
| mul, constant time | Mersenne-31 | 7.0 vs crypto-bigint 46.6 | −39.6 (0.15×) | 7.0 vs crypto-bigint 27.0 | −20.0 (0.26×) | 7.0 vs crypto-bigint 50.3 | −43.3 (0.14×) | 7.0 vs crypto-bigint 30.6 | −23.6 (0.23×) |
| mul, constant time | BabyBear | 12.0 vs crypto-bigint 46.4 | −34.4 (0.26×) | 11.0 vs crypto-bigint 26.9 | −15.9 (0.41×) | 12.0 vs crypto-bigint 49.5 | −37.5 (0.24×) | 11.0 vs crypto-bigint 30.2 | −19.2 (0.36×) |
| mul, constant time | Goldilocks | 16.6 vs winter-math 18.1 | −1.5 (0.92×) | 12.5 vs winter-math 14.5 | −2.0 (0.86×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 12.3 vs winter-math 14.0 | −1.7 (0.88×) |
| mul, constant time | BN254 scalar | 242 vs halo2curves 384 | −142 (0.63×) | 89.2 vs halo2curves 95.0 | −5.8 (0.94×) | 413 vs halo2curves 693 | −280 (0.60×) | 151 vs halo2curves 178 | −27.0 (0.85×) |
| mul, constant time | secp256k1 base | 174 vs halo2curves 308 | −134 (0.56×) | 69.1 vs halo2curves 90.3 | −21.2 (0.77×) | 233 vs halo2curves 585 | −352 (0.40×) | 106 vs halo2curves 159 | −53.0 (0.67×) |
| mul, constant time | P-256 base | 192 vs k256/p256 243 | −51.0 (0.79×) | 64.5 vs halo2curves 73.7 | −9.2 (0.88×) | 295 vs k256/p256 455 | −160 (0.65×) | 119 vs halo2curves 130 | −11.0 (0.92×) |
| mul, constant time | P-256 scalar | 243 vs halo2curves 351 | −108 (0.69×) | 88.1 vs halo2curves 95.9 | −7.8 (0.92×) | 331 vs halo2curves 698 | −367 (0.47×) | 159 vs halo2curves 176 | −17.0 (0.90×) |
| mul, constant time | Tom-256 base | 239 vs halo2curves 347 | −108 (0.69×) | 80.2 vs halo2curves 93.9 | −13.7 (0.85×) | 309 vs halo2curves 591 | −282 (0.52×) | 153 vs halo2curves 171 | −18.0 (0.89×) |
| mul, constant time | BLS12-381 base | 722 vs halo2curves 954 | −232 (0.76×) | 185 vs halo2curves 192 | −7.0 (0.96×) | 910 vs crypto-bigint 1,330 | −420 (0.68×) | 322 vs halo2curves 379 | −57.0 (0.85×) |
| mul, constant time | P-521 base | 686 vs crypto-bigint 2,151 | −1,465 (0.32×) | 239 vs crypto-bigint 618 | −379 (0.39×) | 1,296 vs crypto-bigint 2,755 | −1,459 (0.47×) | 438 vs crypto-bigint 1,101 | −663 (0.40×) |
| square, constant time | Mersenne-31 | 7.0 vs crypto-bigint 45.4 | −38.4 (0.15×) | 7.0 vs crypto-bigint 27.1 | −20.1 (0.26×) | 7.0 vs crypto-bigint 50.6 | −43.6 (0.14×) | 7.0 vs crypto-bigint 30.8 | −23.8 (0.23×) |
| square, constant time | BabyBear | 12.0 vs crypto-bigint 45.3 | −33.3 (0.26×) | 11.0 vs crypto-bigint 27.0 | −16.0 (0.41×) | 12.0 vs crypto-bigint 52.9 | −40.9 (0.23×) | 11.0 vs crypto-bigint 31.2 | −20.2 (0.35×) |
| square, constant time | Goldilocks | 15.1 vs winter-math 19.5 | −4.4 (0.77×) | 11.0 vs winter-math 12.8 | −1.8 (0.86×) | 18.5 vs winter-math 19.0 | −0.5 (0.97×) | 12.5 vs winter-math 14.7 | −2.2 (0.85×) |
| square, constant time | BN254 scalar | 223 vs halo2curves 321 | −98.0 (0.69×) | 81.4 vs halo2curves 97.0 | −15.6 (0.84×) | 185 vs halo2curves 537 | −352 (0.34×) | 133 vs halo2curves 143 | −10.0 (0.93×) |
| square, constant time | secp256k1 base | 152 vs halo2curves 214 | −62.0 (0.71×) | 58.8 vs halo2curves 82.8 | −24.0 (0.71×) | 159 vs halo2curves 450 | −291 (0.35×) | 88.2 vs halo2curves 125 | −36.8 (0.71×) |
| square, constant time | P-256 base | 160 vs halo2curves 193 | −33.0 (0.83×) | 55.5 vs halo2curves 62.6 | −7.1 (0.89×) | 237 vs halo2curves 347 | −110 (0.68×) | 108 vs halo2curves 109 | −1.0 (0.99×) |
| square, constant time | P-256 scalar | 220 vs halo2curves 282 | −62.0 (0.78×) | 81.5 vs halo2curves 92.2 | −10.7 (0.88×) | 286 vs halo2curves 548 | −262 (0.52×) | 141 vs halo2curves 142 | −1.0 (0.99×) |
| square, constant time | Tom-256 base | 211 vs halo2curves 280 | −69.0 (0.75×) | 77.4 vs halo2curves 89.1 | −11.7 (0.87×) | 210 vs halo2curves 476 | −266 (0.44×) | 125 vs halo2curves 130 | −5.0 (0.96×) |
| square, constant time | BLS12-381 base | 567 vs halo2curves 932 | −365 (0.61×) | 160 vs halo2curves 162 | −2.0 (0.99×) | 798 vs halo2curves 1,214 | −416 (0.66×) | 280 vs halo2curves 292 | −12.0 (0.96×) |
| square, constant time | P-521 base | 571 vs crypto-bigint 2,174 | −1,603 (0.26×) | 196 vs crypto-bigint 627 | −431 (0.31×) | 906 vs crypto-bigint 2,832 | −1,926 (0.32×) | 388 vs crypto-bigint 1,127 | −739 (0.34×) |
| mul_u64, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.5 | −50.5 (0.15×) | 9.0 vs crypto-bigint 31.6 | −22.6 (0.28×) | 9.0 vs crypto-bigint 73.5 | −64.5 (0.12×) | 9.0 vs crypto-bigint 38.7 | −29.7 (0.23×) |
| mul_u64, constant time | BabyBear | 16.2 vs crypto-bigint 64.5 | −48.3 (0.25×) | 11.0 vs crypto-bigint 32.6 | −21.6 (0.34×) | 15.5 vs crypto-bigint 89.8 | −74.3 (0.17×) | 13.0 vs crypto-bigint 40.8 | −27.8 (0.32×) |
| mul_u64, constant time | Goldilocks | 17.8 vs winter-math 24.0 | −6.2 (0.74×) | 14.1 vs winter-math 13.7 | +0.4 (1.03×) | 25.3 vs winter-math 34.1 | −8.8 (0.74×) | 13.6 vs winter-math 14.7 | −1.1 (0.93×) |
| mul_u64, constant time | BN254 scalar | 109 vs crypto-bigint 874 | −765 (0.12×) | 48.4 vs crypto-bigint 258 | −210 (0.19×) | 132 vs halo2curves 1,433 | −1,301 (0.09×) | 59.5 vs crypto-bigint 410 | −351 (0.15×) |
| mul_u64, constant time | secp256k1 base | 68.6 vs halo2curves 688 | −619 (0.10×) | 29.0 vs crypto-bigint 258 | −229 (0.11×) | 85.1 vs halo2curves 1,050 | −965 (0.08×) | 39.5 vs crypto-bigint 410 | −371 (0.10×) |
| mul_u64, constant time | P-256 base | 88.2 vs k256/p256 487 | −399 (0.18×) | 46.5 vs k256/p256 154 | −108 (0.30×) | 104 vs k256/p256 932 | −828 (0.11×) | 60.4 vs k256/p256 257 | −197 (0.24×) |
| mul_u64, constant time | P-256 scalar | 105 vs k256/p256 508 | −403 (0.21×) | 47.9 vs k256/p256 255 | −207 (0.19×) | 139 vs k256/p256 815 | −676 (0.17×) | 70.3 vs k256/p256 339 | −269 (0.21×) |
| mul_u64, constant time | Tom-256 base | 100 vs crypto-bigint 834 | −734 (0.12×) | 47.9 vs crypto-bigint 258 | −210 (0.19×) | 122 vs halo2curves 1,273 | −1,151 (0.10×) | 67.9 vs crypto-bigint 410 | −342 (0.17×) |
| mul_u64, constant time | BLS12-381 base | 163 vs crypto-bigint 1,943 | −1,780 (0.08×) | 64.3 vs halo2curves 488 | −424 (0.13×) | 195 vs crypto-bigint 2,582 | −2,387 (0.08×) | 81.1 vs crypto-bigint 1,039 | −958 (0.08×) |
| mul_u64, constant time | P-521 base | 146 vs crypto-bigint 3,050 | −2,904 (0.05×) | 65.1 vs crypto-bigint 1,002 | −937 (0.06×) | 188 vs crypto-bigint 4,041 | −3,853 (0.05×) | 90.3 vs crypto-bigint 1,860 | −1,770 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.0 vs crypto-bigint 60.0 | −51.0 (0.15×) | 9.0 vs crypto-bigint 31.8 | −22.8 (0.28×) | 9.0 vs crypto-bigint 73.0 | −64.0 (0.12×) | 9.0 vs crypto-bigint 39.0 | −30.0 (0.23×) |
| mul_uint, constant time | BabyBear | 16.1 vs crypto-bigint 64.1 | −48.0 (0.25×) | 11.0 vs crypto-bigint 32.7 | −21.7 (0.34×) | 15.7 vs crypto-bigint 88.9 | −73.2 (0.18×) | 13.0 vs crypto-bigint 40.8 | −27.8 (0.32×) |
| mul_uint, constant time | Goldilocks | 17.8 vs winter-math 24.3 | −6.5 (0.73×) | 14.1 vs winter-math 13.8 | +0.3 (1.02×) | 25.3 vs winter-math 34.0 | −8.7 (0.74×) | 15.0 vs winter-math 14.5 | +0.5 (1.03×) |
| mul_uint, constant time | BN254 scalar | 482 vs halo2curves 762 | −280 (0.63×) | 162 vs halo2curves 180 | −18.0 (0.90×) | 594 vs halo2curves 1,343 | −749 (0.44×) | 298 vs crypto-bigint 407 | −109 (0.73×) |
| mul_uint, constant time | secp256k1 base | 319 vs halo2curves 526 | −207 (0.61×) | 71.3 vs halo2curves 154 | −82.7 (0.46×) | 382 vs halo2curves 952 | −570 (0.40×) | 117 vs halo2curves 264 | −147 (0.44×) |
| mul_uint, constant time | P-256 base | 406 vs halo2curves 460 | −54.0 (0.88×) | 124 vs halo2curves 145 | −21.0 (0.86×) | 489 vs halo2curves 940 | −451 (0.52×) | 232 vs halo2curves 251 | −19.0 (0.92×) |
| mul_uint, constant time | P-256 scalar | 477 vs k256/p256 549 | −72.0 (0.87×) | 167 vs halo2curves 185 | −18.0 (0.90×) | 540 vs k256/p256 883 | −343 (0.61×) | 315 vs halo2curves 346 | −31.0 (0.91×) |
| mul_uint, constant time | Tom-256 base | 466 vs halo2curves 694 | −228 (0.67×) | 161 vs halo2curves 180 | −19.0 (0.89×) | 554 vs halo2curves 1,179 | −625 (0.47×) | 308 vs halo2curves 341 | −33.0 (0.90×) |
| mul_uint, constant time | BLS12-381 base | 1,088 vs halo2curves 1,899 | −811 (0.57×) | 351 vs halo2curves 379 | −28.0 (0.93×) | 1,695 vs crypto-bigint 2,547 | −852 (0.67×) | 890 vs crypto-bigint 1,037 | −147 (0.86×) |
| mul_uint, constant time | P-521 base | 2,021 vs crypto-bigint 3,068 | −1,047 (0.66×) | 283 vs crypto-bigint 1,009 | −726 (0.28×) | 2,900 vs crypto-bigint 3,996 | −1,096 (0.73×) | 589 vs crypto-bigint 1,871 | −1,282 (0.31×) |
| add_u64, constant time | Mersenne-31 | 4.7 vs crypto-bigint 35.4 | −30.7 (0.13×) | 4.6 vs crypto-bigint 30.1 | −25.5 (0.15×) | 4.7 vs crypto-bigint 43.6 | −38.9 (0.11×) | 4.8 vs crypto-bigint 29.8 | −25.0 (0.16×) |
| add_u64, constant time | BabyBear | 11.9 vs crypto-bigint 47.5 | −35.6 (0.25×) | 7.2 vs crypto-bigint 30.0 | −22.8 (0.24×) | 13.0 vs crypto-bigint 53.4 | −40.4 (0.24×) | 11.3 vs crypto-bigint 31.9 | −20.6 (0.35×) |
| add_u64, constant time | Goldilocks | 8.5 vs winter-math 13.0 | −4.5 (0.65×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_u64, constant time | BN254 scalar | 93.0 vs crypto-bigint 513 | −420 (0.18×) | 40.7 vs halo2curves 212 | −171 (0.19×) | 107 vs halo2curves 764 | −657 (0.14×) | 69.8 vs halo2curves 300 | −230 (0.23×) |
| add_u64, constant time | secp256k1 base | 25.4 vs halo2curves 394 | −369 (0.06×) | 15.0 vs halo2curves 204 | −189 (0.07×) | 25.2 vs halo2curves 494 | −469 (0.05×) | 24.0 vs halo2curves 244 | −220 (0.10×) |
| add_u64, constant time | P-256 base | 66.5 vs k256/p256 277 | −211 (0.24×) | 35.4 vs k256/p256 92.5 | −57.1 (0.38×) | 80.1 vs k256/p256 510 | −430 (0.16×) | 58.1 vs k256/p256 158 | −99.9 (0.37×) |
| add_u64, constant time | P-256 scalar | 105 vs k256/p256 56.3 | +48.7 (1.87×) | 40.7 vs k256/p256 56.3 | −15.6 (0.72×) | 109 vs k256/p256 62.1 | +46.9 (1.76×) | 73.2 vs k256/p256 62.0 | +11.2 (1.18×) |
| add_u64, constant time | Tom-256 base | 97.7 vs crypto-bigint 508 | −410 (0.19×) | 40.4 vs halo2curves 217 | −177 (0.19×) | 110 vs halo2curves 686 | −576 (0.16×) | 70.2 vs halo2curves 293 | −223 (0.24×) |
| add_u64, constant time | BLS12-381 base | 142 vs crypto-bigint 1,092 | −950 (0.13×) | 63.8 vs halo2curves 322 | −258 (0.20×) | 158 vs crypto-bigint 1,395 | −1,237 (0.11×) | 107 vs halo2curves 642 | −535 (0.17×) |
| add_u64, constant time | P-521 base | 66.1 vs crypto-bigint 1,690 | −1,624 (0.04×) | 36.6 vs crypto-bigint 653 | −616 (0.06×) | 68.6 vs crypto-bigint 2,209 | −2,140 (0.03×) | 62.0 vs crypto-bigint 1,167 | −1,105 (0.05×) |
| add_uint, constant time | Mersenne-31 | 4.7 vs crypto-bigint 35.5 | −30.8 (0.13×) | 4.7 vs crypto-bigint 30.1 | −25.4 (0.16×) | 4.7 vs crypto-bigint 41.9 | −37.2 (0.11×) | 4.7 vs crypto-bigint 30.5 | −25.8 (0.15×) |
| add_uint, constant time | BabyBear | 11.6 vs crypto-bigint 47.6 | −36.0 (0.24×) | 7.2 vs crypto-bigint 30.1 | −22.9 (0.24×) | 13.0 vs crypto-bigint 51.3 | −38.3 (0.25×) | 11.3 vs crypto-bigint 32.9 | −21.6 (0.34×) |
| add_uint, constant time | Goldilocks | 8.5 vs winter-math 13.2 | −4.7 (0.64×) | 6.5 vs winter-math 7.4 | −0.9 (0.88×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 9.2 vs winter-math 9.6 | −0.4 (0.96×) |
| add_uint, constant time | BN254 scalar | 255 vs halo2curves 410 | −155 (0.62×) | 96.6 vs halo2curves 112 | −15.4 (0.86×) | 317 vs halo2curves 690 | −373 (0.46×) | 181 vs halo2curves 206 | −25.0 (0.88×) |
| add_uint, constant time | secp256k1 base | 38.4 vs halo2curves 228 | −190 (0.17×) | 25.1 vs halo2curves 85.1 | −60.0 (0.29×) | 36.3 vs halo2curves 417 | −381 (0.09×) | 40.6 vs halo2curves 140 | −99.4 (0.29×) |
| add_uint, constant time | P-256 base | 179 vs halo2curves 250 | −71.0 (0.72×) | 74.3 vs halo2curves 91.0 | −16.7 (0.82×) | 249 vs halo2curves 507 | −258 (0.49×) | 141 vs halo2curves 153 | −12.0 (0.92×) |
| add_uint, constant time | P-256 scalar | 263 vs k256/p256 84.0 | +179 (3.13×) | 99.1 vs k256/p256 82.6 | +16.5 (1.20×) | 318 vs k256/p256 108 | +210 (2.94×) | 193 vs k256/p256 108 | +85.0 (1.79×) |
| add_uint, constant time | Tom-256 base | 255 vs halo2curves 375 | −120 (0.68×) | 96.8 vs halo2curves 110 | −13.2 (0.88×) | 326 vs halo2curves 626 | −300 (0.52×) | 181 vs halo2curves 196 | −15.0 (0.92×) |
| add_uint, constant time | BLS12-381 base | 757 vs halo2curves 961 | −204 (0.79×) | 196 vs halo2curves 224 | −28.0 (0.88×) | 947 vs crypto-bigint 1,415 | −468 (0.67×) | 380 vs halo2curves 425 | −45.0 (0.89×) |
| add_uint, constant time | P-521 base | 144 vs crypto-bigint 1,712 | −1,568 (0.08×) | 78.2 vs crypto-bigint 666 | −588 (0.12×) | 147 vs crypto-bigint 2,217 | −2,070 (0.07×) | 131 vs crypto-bigint 1,182 | −1,051 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 4.6 vs crypto-bigint 22.4 | −17.8 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.5 | −25.6 (0.16×) | 4.8 vs crypto-bigint 19.8 | −15.0 (0.24×) |
| sub_u64, constant time | BabyBear | 12.2 vs crypto-bigint 28.9 | −16.7 (0.42×) | 7.1 vs crypto-bigint 19.3 | −12.2 (0.37×) | 13.0 vs crypto-bigint 42.3 | −29.3 (0.31×) | 10.7 vs crypto-bigint 21.3 | −10.6 (0.50×) |
| sub_u64, constant time | Goldilocks | 8.2 vs winter-math 11.2 | −3.0 (0.73×) | 6.1 vs winter-math 6.9 | −0.8 (0.88×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.7 vs winter-math 8.6 | +0.1 (1.01×) |
| sub_u64, constant time | BN254 scalar | 94.8 vs crypto-bigint 492 | −397 (0.19×) | 36.7 vs crypto-bigint 152 | −115 (0.24×) | 107 vs halo2curves 745 | −638 (0.14×) | 66.9 vs crypto-bigint 286 | −219 (0.23×) |
| sub_u64, constant time | secp256k1 base | 18.0 vs halo2curves 377 | −359 (0.05×) | 12.0 vs crypto-bigint 152 | −140 (0.08×) | 17.0 vs halo2curves 477 | −460 (0.04×) | 15.3 vs halo2curves 237 | −222 (0.06×) |
| sub_u64, constant time | P-256 base | 61.4 vs k256/p256 269 | −208 (0.23×) | 31.3 vs k256/p256 88.1 | −56.8 (0.36×) | 74.6 vs k256/p256 491 | −416 (0.15×) | 53.1 vs k256/p256 149 | −95.9 (0.36×) |
| sub_u64, constant time | P-256 scalar | 95.9 vs k256/p256 21.3 | +74.6 (4.50×) | 37.3 vs k256/p256 21.2 | +16.1 (1.76×) | 105 vs k256/p256 24.7 | +80.3 (4.25×) | 64.0 vs k256/p256 24.7 | +39.3 (2.59×) |
| sub_u64, constant time | Tom-256 base | 91.9 vs crypto-bigint 484 | −392 (0.19×) | 36.9 vs crypto-bigint 151 | −114 (0.24×) | 105 vs halo2curves 692 | −587 (0.15×) | 66.0 vs halo2curves 285 | −219 (0.23×) |
| sub_u64, constant time | BLS12-381 base | 138 vs crypto-bigint 1,091 | −953 (0.13×) | 59.6 vs halo2curves 318 | −258 (0.19×) | 174 vs crypto-bigint 1,407 | −1,233 (0.12×) | 103 vs crypto-bigint 623 | −520 (0.17×) |
| sub_u64, constant time | P-521 base | 45.9 vs crypto-bigint 1,669 | −1,623 (0.03×) | 27.1 vs crypto-bigint 637 | −610 (0.04×) | 49.7 vs crypto-bigint 2,214 | −2,164 (0.02×) | 40.2 vs crypto-bigint 1,110 | −1,070 (0.04×) |
| sub_uint, constant time | Mersenne-31 | 4.7 vs crypto-bigint 22.2 | −17.5 (0.21×) | 4.6 vs crypto-bigint 18.2 | −13.6 (0.25×) | 4.9 vs crypto-bigint 30.7 | −25.8 (0.16×) | 4.9 vs crypto-bigint 20.4 | −15.5 (0.24×) |
| sub_uint, constant time | BabyBear | 12.1 vs crypto-bigint 28.7 | −16.6 (0.42×) | 7.1 vs crypto-bigint 19.4 | −12.3 (0.37×) | 13.0 vs crypto-bigint 42.4 | −29.4 (0.31×) | 10.6 vs crypto-bigint 21.4 | −10.8 (0.50×) |
| sub_uint, constant time | Goldilocks | 8.3 vs winter-math 11.1 | −2.8 (0.75×) | 6.1 vs winter-math 6.8 | −0.7 (0.90×) | 11.0 vs winter-math 18.1 | −7.1 (0.61×) | 8.7 vs winter-math 8.6 | +0.1 (1.01×) |
| sub_uint, constant time | BN254 scalar | 241 vs halo2curves 396 | −155 (0.61×) | 91.8 vs halo2curves 105 | −13.2 (0.87×) | 287 vs halo2curves 670 | −383 (0.43×) | 179 vs halo2curves 194 | −15.0 (0.92×) |
| sub_uint, constant time | secp256k1 base | 32.4 vs halo2curves 210 | −178 (0.15×) | 20.5 vs halo2curves 80.7 | −60.2 (0.25×) | 31.7 vs halo2curves 399 | −367 (0.08×) | 35.4 vs halo2curves 130 | −94.6 (0.27×) |
| sub_uint, constant time | P-256 base | 180 vs halo2curves 235 | −55.0 (0.77×) | 71.6 vs halo2curves 85.7 | −14.1 (0.84×) | 247 vs halo2curves 499 | −252 (0.49×) | 132 vs halo2curves 143 | −11.0 (0.92×) |
| sub_uint, constant time | P-256 scalar | 252 vs k256/p256 72.0 | +180 (3.50×) | 98.5 vs k256/p256 72.4 | +26.1 (1.36×) | 331 vs k256/p256 95.4 | +236 (3.47×) | 181 vs k256/p256 95.6 | +85.4 (1.89×) |
| sub_uint, constant time | Tom-256 base | 246 vs halo2curves 365 | −119 (0.67×) | 94.4 vs halo2curves 103 | −8.6 (0.92×) | 328 vs halo2curves 612 | −284 (0.54×) | 177 vs halo2curves 187 | −10.0 (0.95×) |
| sub_uint, constant time | BLS12-381 base | 761 vs halo2curves 953 | −192 (0.80×) | 197 vs halo2curves 217 | −20.0 (0.91×) | 941 vs crypto-bigint 1,382 | −441 (0.68×) | 379 vs halo2curves 485 | −106 (0.78×) |
| sub_uint, constant time | P-521 base | 128 vs crypto-bigint 1,687 | −1,559 (0.08×) | 67.3 vs crypto-bigint 638 | −571 (0.11×) | 121 vs crypto-bigint 2,202 | −2,081 (0.05×) | 108 vs crypto-bigint 1,135 | −1,027 (0.10×) |
| div_u64, constant time | BN254 scalar | 12,358 vs crypto-bigint 29,996 | −17,638 (0.41×) | 9,433 vs crypto-bigint 25,532 | −16,099 (0.37×) | 17,330 vs crypto-bigint 37,698 | −20,368 (0.46×) | 13,038 vs crypto-bigint 30,412 | −17,374 (0.43×) |
| div_u64, constant time | secp256k1 base | 11,889 vs crypto-bigint 30,189 | −18,300 (0.39×) | 9,059 vs crypto-bigint 25,514 | −16,455 (0.36×) | 16,945 vs crypto-bigint 37,868 | −20,923 (0.45×) | 12,582 vs crypto-bigint 30,571 | −17,989 (0.41×) |
| div_u64, constant time | P-256 base | 11,866 vs crypto-bigint 30,105 | −18,239 (0.39×) | 9,122 vs k256/p256 22,342 | −13,220 (0.41×) | 17,021 vs crypto-bigint 37,735 | −20,714 (0.45×) | 12,387 vs crypto-bigint 30,496 | −18,109 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,240 vs crypto-bigint 29,933 | −17,693 (0.41×) | 9,376 vs crypto-bigint 25,609 | −16,233 (0.37×) | 17,515 vs crypto-bigint 37,854 | −20,339 (0.46×) | 13,344 vs crypto-bigint 30,368 | −17,024 (0.44×) |
| div_u64, constant time | Tom-256 base | 12,216 vs crypto-bigint 30,171 | −17,955 (0.40×) | 9,267 vs crypto-bigint 25,519 | −16,252 (0.36×) | 16,743 vs crypto-bigint 37,715 | −20,972 (0.44×) | 12,968 vs crypto-bigint 30,642 | −17,674 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,761 vs crypto-bigint 51,061 | −26,300 (0.48×) | 16,913 vs crypto-bigint 43,623 | −26,710 (0.39×) | 34,565 vs crypto-bigint 66,479 | −31,914 (0.52×) | 24,110 vs crypto-bigint 54,008 | −29,898 (0.45×) |
| div_u64, constant time | P-521 base | 43,062 vs crypto-bigint 106,291 | −63,229 (0.41×) | 26,297 vs crypto-bigint 85,676 | −59,379 (0.31×) | 64,852 vs crypto-bigint 126,893 | −62,041 (0.51×) | 43,359 vs crypto-bigint 99,470 | −56,111 (0.44×) |
| div_uint, constant time | Mersenne-31 | 275 vs runtime-algebra 3,530 | −3,255 (0.08×) | 275 vs runtime-algebra 3,539 | −3,264 (0.08×) | 274 vs runtime-algebra 3,448 | −3,174 (0.08×) | 274 vs runtime-algebra 3,446 | −3,172 (0.08×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,532 | −3,063 (0.13×) | 435 vs runtime-algebra 3,536 | −3,101 (0.12×) | 470 vs runtime-algebra 3,445 | −2,975 (0.14×) | 435 vs runtime-algebra 3,450 | −3,015 (0.13×) |
| div_uint, constant time | Goldilocks | 1,138 vs winter-math 1,810 | −672 (0.63×) | 859 vs winter-math 928 | −69.0 (0.93×) | 1,431 vs winter-math 1,899 | −468 (0.75×) | 966 vs winter-math 1,033 | −67.0 (0.94×) |
| div_uint, constant time | BN254 scalar | 12,371 vs crypto-bigint 30,014 | −17,643 (0.41×) | 9,492 vs crypto-bigint 25,782 | −16,290 (0.37×) | 17,469 vs crypto-bigint 37,815 | −20,346 (0.46×) | 13,228 vs crypto-bigint 30,511 | −17,283 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,870 vs crypto-bigint 29,908 | −18,038 (0.40×) | 9,086 vs crypto-bigint 25,774 | −16,688 (0.35×) | 16,834 vs crypto-bigint 37,802 | −20,968 (0.45×) | 12,859 vs crypto-bigint 30,475 | −17,616 (0.42×) |
| div_uint, constant time | P-256 base | 11,898 vs crypto-bigint 29,863 | −17,965 (0.40×) | 9,073 vs k256/p256 22,296 | −13,223 (0.41×) | 17,266 vs crypto-bigint 37,985 | −20,719 (0.45×) | 12,619 vs crypto-bigint 30,644 | −18,025 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,592 vs crypto-bigint 30,097 | −17,505 (0.42×) | 9,424 vs crypto-bigint 25,759 | −16,335 (0.37×) | 17,650 vs crypto-bigint 37,957 | −20,307 (0.46×) | 13,662 vs crypto-bigint 30,612 | −16,950 (0.45×) |
| div_uint, constant time | Tom-256 base | 12,437 vs crypto-bigint 29,851 | −17,414 (0.42×) | 9,389 vs crypto-bigint 25,930 | −16,541 (0.36×) | 17,060 vs crypto-bigint 37,908 | −20,848 (0.45×) | 13,198 vs crypto-bigint 30,505 | −17,307 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,368 vs crypto-bigint 50,051 | −24,683 (0.51×) | 16,957 vs crypto-bigint 43,405 | −26,448 (0.39×) | 35,309 vs crypto-bigint 66,783 | −31,474 (0.53×) | 24,696 vs crypto-bigint 53,719 | −29,023 (0.46×) |
| div_uint, constant time | P-521 base | 42,958 vs crypto-bigint 105,088 | −62,130 (0.41×) | 26,111 vs crypto-bigint 84,268 | −58,157 (0.31×) | 64,813 vs crypto-bigint 126,450 | −61,637 (0.51×) | 42,943 vs crypto-bigint 99,551 | −56,608 (0.43×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.6×) | 3.2 (1.6×) |
| add | BN254 scalar | 4.9 | 14.0 (2.9×) | 12.4 (2.5×) | 14.5 (3.0×) | 19.0 (3.9×) |
| add | secp256k1 base | 5.8 | 14.0 (2.4×) | 11.8 (2.0×) | 14.8 (2.6×) | 20.2 (3.5×) |
| add | P-256 base | 6.0 | 14.4 (2.4×) | 12.3 (2.1×) | 14.5 (2.4×) | 20.0 (3.3×) |
| add | P-256 scalar | 5.8 | 14.5 (2.5×) | 11.7 (2.0×) | 15.5 (2.7×) | 20.5 (3.5×) |
| add | Tom-256 base | 6.0 | 13.5 (2.2×) | 11.8 (2.0×) | 14.7 (2.4×) | 20.4 (3.4×) |
| add | BLS12-381 base | 9.6 | 21.9 (2.3×) | 18.0 (1.9×) | 22.5 (2.3×) | 29.2 (3.0×) |
| add | P-521 base | 12.3 | 42.6 (3.5×) | 26.7 (2.2×) | 42.7 (3.5×) | 44.8 (3.6×) |
| sub | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BabyBear | 2.0 | 2.5 (1.2×) | 2.6 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub | BN254 scalar | 4.9 | 15.1 (3.1×) | 11.8 (2.4×) | 14.5 (3.0×) | 19.4 (4.0×) |
| sub | secp256k1 base | 5.4 | 16.5 (3.1×) | 11.8 (2.2×) | 15.7 (2.9×) | 18.7 (3.5×) |
| sub | P-256 base | 5.5 | 14.3 (2.6×) | 11.8 (2.1×) | 14.1 (2.6×) | 18.2 (3.3×) |
| sub | P-256 scalar | 5.0 | 16.0 (3.2×) | 11.8 (2.4×) | 15.0 (3.0×) | 19.2 (3.8×) |
| sub | Tom-256 base | 5.5 | 15.0 (2.7×) | 11.8 (2.1×) | 14.6 (2.7×) | 19.5 (3.5×) |
| sub | BLS12-381 base | 9.6 | 23.0 (2.4×) | 17.1 (1.8×) | 21.8 (2.3×) | 28.5 (3.0×) |
| sub | P-521 base | 11.9 | 47.2 (4.0×) | 26.3 (2.2×) | 43.1 (3.6×) | 43.3 (3.6×) |
| mul | Mersenne-31 | 2.7 | 3.7 (1.4×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 15.0 (3.9×) | 7.0 (1.8×) |
| mul | BN254 scalar | 68.8 | 233 (3.4×) | 79.7 (1.2×) | 387 (5.6×) | 149 (2.2×) |
| mul | secp256k1 base | 44.5 | 162 (3.6×) | 59.6 (1.3×) | 237 (5.3×) | 107 (2.4×) |
| mul | P-256 base | 49.5 | 182 (3.7×) | 72.0 (1.5×) | 278 (5.6×) | 119 (2.4×) |
| mul | P-256 scalar | 74.8 | 247 (3.3×) | 84.8 (1.1×) | 329 (4.4×) | 167 (2.2×) |
| mul | Tom-256 base | 70.9 | 239 (3.4×) | 74.8 (1.1×) | 314 (4.4×) | 163 (2.3×) |
| mul | BLS12-381 base | 138 | 726 (5.3×) | 181 (1.3×) | 930 (6.7×) | 322 (2.3×) |
| mul | P-521 base | 176 | 667 (3.8×) | 244 (1.4×) | 1,273 (7.2×) | 534 (3.0×) |
| square | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.6 (1.3×) |
| square | BabyBear | 3.4 | 4.7 (1.4×) | 4.8 (1.4×) | 5.7 (1.7×) | 5.8 (1.7×) |
| square | Goldilocks | 3.7 | 8.7 (2.4×) | 6.0 (1.6×) | 13.0 (3.5×) | 8.0 (2.2×) |
| square | BN254 scalar | 63.1 | 217 (3.4×) | 78.4 (1.2×) | 348 (5.5×) | 142 (2.3×) |
| square | secp256k1 base | 33.8 | 145 (4.3×) | 53.0 (1.6×) | 202 (6.0×) | 92.0 (2.7×) |
| square | P-256 base | 40.0 | 160 (4.0×) | 74.3 (1.9×) | 262 (6.5×) | 123 (3.1×) |
| square | P-256 scalar | 63.9 | 234 (3.7×) | 75.8 (1.2×) | 269 (4.2×) | 151 (2.4×) |
| square | Tom-256 base | 57.0 | 233 (4.1×) | 72.4 (1.3×) | 282 (4.9×) | 160 (2.8×) |
| square | BLS12-381 base | 131 | 567 (4.3×) | 158 (1.2×) | 828 (6.3×) | 309 (2.4×) |
| square | P-521 base | 117 | 567 (4.8×) | 238 (2.0×) | 914 (7.8×) | 398 (3.4×) |
| invert | Mersenne-31 | 214 | 267 (1.2×) | 267 (1.2×) | 269 (1.3×) | 268 (1.3×) |
| invert | BabyBear | 213 | 269 (1.3×) | 268 (1.3×) | 271 (1.3×) | 271 (1.3×) |
| invert | Goldilocks | 437 | 539 (1.2×) | 540 (1.2×) | 564 (1.3×) | 567 (1.3×) |
| invert | BN254 scalar | 5,057 | 8,523 (1.7×) | 6,468 (1.3×) | 11,802 (2.3×) | 7,853 (1.6×) |
| invert | secp256k1 base | 4,922 | 8,497 (1.7×) | 6,045 (1.2×) | 11,550 (2.3×) | 7,990 (1.6×) |
| invert | P-256 base | 4,837 | 8,309 (1.7×) | 6,072 (1.3×) | 11,486 (2.4×) | 7,614 (1.6×) |
| invert | P-256 scalar | 5,041 | 8,470 (1.7×) | 6,382 (1.3×) | 11,866 (2.4×) | 8,190 (1.6×) |
| invert | Tom-256 base | 5,018 | 8,522 (1.7×) | 6,464 (1.3×) | 11,670 (2.3×) | 8,034 (1.6×) |
| invert | BLS12-381 base | 8,808 | 19,535 (2.2×) | 11,520 (1.3×) | 24,270 (2.8×) | 15,715 (1.8×) |
| invert | P-521 base | 14,227 | 33,389 (2.3×) | 18,109 (1.3×) | 48,629 (3.4×) | 29,193 (2.1×) |
| sqrt | Mersenne-31 | 135 | 159 (1.2×) | 159 (1.2×) | 160 (1.2×) | 160 (1.2×) |
| sqrt | BabyBear | 1,971 | 2,104 (1.1×) | 2,058 (1.0×) | 2,142 (1.1×) | 2,173 (1.1×) |
| sqrt | Goldilocks | 2,833 | 4,834 (1.7×) | 3,485 (1.2×) | 6,077 (2.1×) | 5,043 (1.8×) |
| sqrt | BN254 scalar | 33,810 | 115,714 (3.4×) | 52,938 (1.6×) | 176,779 (5.2×) | 95,027 (2.8×) |
| sqrt | secp256k1 base | 12,304 | 40,862 (3.3×) | 17,817 (1.4×) | 66,119 (5.4×) | 27,371 (2.2×) |
| sqrt | P-256 base | 11,796 | 43,265 (3.7×) | 18,396 (1.6×) | 78,732 (6.7×) | 30,386 (2.6×) |
| sqrt | P-256 scalar | 24,127 | 74,437 (3.1×) | 33,335 (1.4×) | 113,724 (4.7×) | 63,482 (2.6×) |
| sqrt | Tom-256 base | 21,264 | 65,944 (3.1×) | 18,298 (0.9×) | 71,968 (3.4×) | 51,996 (2.4×) |
| sqrt | BLS12-381 base | 65,940 | 296,814 (4.5×) | 93,874 (1.4×) | 396,124 (6.0×) | 216,536 (3.3×) |
| sqrt | P-521 base | 68,261 | 319,421 (4.7×) | 125,421 (1.8×) | 474,076 (6.9×) | 256,633 (3.8×) |
| mul_u64 | Mersenne-31 | 5.8 | 6.7 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_u64 | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_u64 | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_u64 | BN254 scalar | 20.7 | 100 (4.8×) | 28.8 (1.4×) | 124 (6.0×) | 44.1 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 42.4 (4.4×) | 12.6 (1.3×) | 65.7 (6.8×) | 23.0 (2.4×) |
| mul_u64 | P-256 base | 18.9 | 71.8 (3.8×) | 27.3 (1.4×) | 94.0 (5.0×) | 41.2 (2.2×) |
| mul_u64 | P-256 scalar | 22.3 | 84.3 (3.8×) | 30.3 (1.4×) | 112 (5.0×) | 47.4 (2.1×) |
| mul_u64 | Tom-256 base | 20.5 | 82.2 (4.0×) | 30.0 (1.5×) | 116 (5.7×) | 45.3 (2.2×) |
| mul_u64 | BLS12-381 base | 28.8 | 131 (4.5×) | 44.2 (1.5×) | 175 (6.1×) | 61.5 (2.1×) |
| mul_u64 | P-521 base | 19.7 | 92.2 (4.7×) | 31.8 (1.6×) | 236 (12.0×) | 50.2 (2.5×) |
| mul_uint | Mersenne-31 | 5.9 | 6.7 (1.1×) | 6.7 (1.1×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_uint | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 17.3 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_uint | BN254 scalar | 103 | 441 (4.3×) | 168 (1.6×) | 534 (5.2×) | 337 (3.3×) |
| mul_uint | secp256k1 base | 56.9 | 266 (4.7×) | 59.7 (1.0×) | 323 (5.7×) | 109 (1.9×) |
| mul_uint | P-256 base | 102 | 354 (3.5×) | 137 (1.3×) | 435 (4.3×) | 224 (2.2×) |
| mul_uint | P-256 scalar | 110 | 427 (3.9×) | 168 (1.5×) | 474 (4.3×) | 307 (2.8×) |
| mul_uint | Tom-256 base | 108 | 412 (3.8×) | 155 (1.4×) | 490 (4.5×) | 306 (2.8×) |
| mul_uint | BLS12-381 base | 211 | 1,006 (4.8×) | 365 (1.7×) | 1,148 (5.4×) | 897 (4.3×) |
| mul_uint | P-521 base | 350 | 1,539 (4.4×) | 257 (0.7×) | 2,379 (6.8×) | 525 (1.5×) |
| add_u64 | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.4 (1.2×) | 5.4 (1.2×) |
| add_u64 | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_u64 | Goldilocks | 4.8 | 9.3 (1.9×) | 7.1 (1.5×) | 12.0 (2.5×) | 9.4 (2.0×) |
| add_u64 | BN254 scalar | 24.8 | 119 (4.8×) | 53.1 (2.1×) | 142 (5.7×) | 94.6 (3.8×) |
| add_u64 | secp256k1 base | 5.8 | 8.1 (1.4×) | 8.0 (1.4×) | 8.0 (1.4×) | 11.0 (1.9×) |
| add_u64 | P-256 base | 25.7 | 71.4 (2.8×) | 35.7 (1.4×) | 84.7 (3.3×) | 68.0 (2.6×) |
| add_u64 | P-256 scalar | 25.4 | 70.7 (2.8×) | 40.5 (1.6×) | 74.7 (2.9×) | 83.9 (3.3×) |
| add_u64 | Tom-256 base | 26.1 | 88.8 (3.4×) | 41.4 (1.6×) | 108 (4.1×) | 71.9 (2.8×) |
| add_u64 | BLS12-381 base | 38.2 | 169 (4.4×) | 66.7 (1.7×) | 256 (6.7×) | 95.2 (2.5×) |
| add_u64 | P-521 base | 10.7 | 21.0 (2.0×) | 15.4 (1.4×) | 22.1 (2.1×) | 24.1 (2.3×) |
| add_uint | Mersenne-31 | 4.6 | 5.2 (1.1×) | 5.2 (1.1×) | 5.4 (1.2×) | 5.4 (1.2×) |
| add_uint | BabyBear | 5.9 | 10.9 (1.8×) | 8.6 (1.5×) | 13.0 (2.2×) | 12.5 (2.1×) |
| add_uint | Goldilocks | 4.7 | 9.3 (2.0×) | 7.1 (1.5×) | 12.0 (2.6×) | 9.4 (2.0×) |
| add_uint | BN254 scalar | 72.0 | 246 (3.4×) | 97.2 (1.4×) | 365 (5.1×) | 185 (2.6×) |
| add_uint | secp256k1 base | 6.8 | 20.9 (3.1×) | 14.6 (2.1×) | 18.3 (2.7×) | 23.8 (3.5×) |
| add_uint | P-256 base | 57.3 | 188 (3.3×) | 74.6 (1.3×) | 326 (5.7×) | 147 (2.6×) |
| add_uint | P-256 scalar | 81.0 | 258 (3.2×) | 96.1 (1.2×) | 322 (4.0×) | 171 (2.1×) |
| add_uint | Tom-256 base | 73.8 | 251 (3.4×) | 99.6 (1.3×) | 323 (4.4×) | 181 (2.5×) |
| add_uint | BLS12-381 base | 152 | 772 (5.1×) | 204 (1.3×) | 949 (6.2×) | 384 (2.5×) |
| add_uint | P-521 base | 23.2 | 65.2 (2.8×) | 42.0 (1.8×) | 88.4 (3.8×) | 75.0 (3.2×) |
| sub_u64 | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.1 (1.1×) |
| sub_u64 | BabyBear | 5.9 | 11.1 (1.9×) | 8.5 (1.4×) | 13.0 (2.2×) | 12.5 (2.1×) |
| sub_u64 | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.3 (2.2×) |
| sub_u64 | BN254 scalar | 24.8 | 110 (4.4×) | 54.8 (2.2×) | 137 (5.5×) | 74.0 (3.0×) |
| sub_u64 | secp256k1 base | 4.2 | 8.7 (2.1×) | 7.2 (1.7×) | 7.5 (1.8×) | 10.0 (2.4×) |
| sub_u64 | P-256 base | 19.8 | 70.7 (3.6×) | 34.7 (1.8×) | 80.2 (4.1×) | 61.1 (3.1×) |
| sub_u64 | P-256 scalar | 24.9 | 67.8 (2.7×) | 39.1 (1.6×) | 75.1 (3.0×) | 86.5 (3.5×) |
| sub_u64 | Tom-256 base | 24.6 | 88.0 (3.6×) | 49.2 (2.0×) | 105 (4.3×) | 70.2 (2.9×) |
| sub_u64 | BLS12-381 base | 37.8 | 152 (4.0×) | 65.5 (1.7×) | 246 (6.5×) | 92.7 (2.5×) |
| sub_u64 | P-521 base | 9.6 | 22.5 (2.3×) | 17.9 (1.9×) | 20.3 (2.1×) | 20.0 (2.1×) |
| sub_uint | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.2 (1.1×) |
| sub_uint | BabyBear | 5.8 | 11.0 (1.9×) | 8.5 (1.5×) | 13.0 (2.2×) | 12.5 (2.2×) |
| sub_uint | Goldilocks | 4.3 | 9.5 (2.2×) | 6.7 (1.6×) | 11.4 (2.7×) | 9.3 (2.2×) |
| sub_uint | BN254 scalar | 74.6 | 246 (3.3×) | 93.6 (1.3×) | 371 (5.0×) | 180 (2.4×) |
| sub_uint | secp256k1 base | 5.4 | 21.7 (4.0×) | 13.6 (2.5×) | 18.4 (3.4×) | 23.3 (4.3×) |
| sub_uint | P-256 base | 53.1 | 179 (3.4×) | 80.8 (1.5×) | 283 (5.3×) | 140 (2.6×) |
| sub_uint | P-256 scalar | 79.9 | 270 (3.4×) | 94.9 (1.2×) | 332 (4.2×) | 187 (2.3×) |
| sub_uint | Tom-256 base | 76.0 | 253 (3.3×) | 93.1 (1.2×) | 318 (4.2×) | 170 (2.2×) |
| sub_uint | BLS12-381 base | 155 | 759 (4.9×) | 205 (1.3×) | 948 (6.1×) | 396 (2.6×) |
| sub_uint | P-521 base | 19.9 | 75.5 (3.8×) | 40.7 (2.0×) | 81.0 (4.1×) | 68.5 (3.4×) |
| from_u64 | Mersenne-31 | 3.2 | 4.0 (1.2×) | 4.0 (1.2×) | 4.3 (1.3×) | 4.0 (1.2×) |
| from_u64 | BabyBear | 6.3 | 11.1 (1.8×) | 7.3 (1.2×) | 12.4 (2.0×) | 10.3 (1.6×) |
| from_u64 | Goldilocks | 3.4 | 7.4 (2.2×) | 5.9 (1.7×) | 9.0 (2.6×) | 7.1 (2.1×) |
| from_u64 | BN254 scalar | 22.0 | 101 (4.6×) | 29.1 (1.3×) | 121 (5.5×) | 50.1 (2.3×) |
| from_u64 | secp256k1 base | 2.0 | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) | 3.4 (1.7×) |
| from_u64 | P-256 base | 17.6 | 56.8 (3.2×) | 24.6 (1.4×) | 70.6 (4.0×) | 44.8 (2.5×) |
| from_u64 | P-256 scalar | 18.0 | 55.1 (3.1×) | 26.1 (1.5×) | 59.9 (3.3×) | 48.1 (2.7×) |
| from_u64 | Tom-256 base | 21.0 | 74.5 (3.5×) | 28.2 (1.3×) | 86.2 (4.1×) | 49.0 (2.3×) |
| from_u64 | BLS12-381 base | 29.1 | 144 (4.9×) | 50.6 (1.7×) | 179 (6.2×) | 68.1 (2.3×) |
| from_u64 | P-521 base | 5.0 | 6.1 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64 | BN254 scalar | 612 | 1,212 (2.0×) | 844 (1.4×) | 1,374 (2.2×) | 1,116 (1.8×) |
| div_u64 | secp256k1 base | 564 | 1,011 (1.8×) | 712 (1.3×) | 1,230 (2.2×) | 823 (1.5×) |
| div_u64 | P-256 base | 602 | 1,090 (1.8×) | 805 (1.3×) | 1,354 (2.2×) | 982 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,191 (1.9×) | 848 (1.4×) | 1,408 (2.3×) | 1,132 (1.8×) |
| div_u64 | Tom-256 base | 608 | 1,207 (2.0×) | 849 (1.4×) | 1,495 (2.5×) | 1,043 (1.7×) |
| div_u64 | BLS12-381 base | 751 | 1,804 (2.4×) | 1,069 (1.4×) | 2,469 (3.3×) | 1,752 (2.3×) |
| div_u64 | P-521 base | 925 | 2,457 (2.7×) | 1,002 (1.1×) | 3,596 (3.9×) | 1,489 (1.6×) |
| div_uint | Mersenne-31 | 215 | 274 (1.3×) | 275 (1.3×) | 283 (1.3×) | 283 (1.3×) |
| div_uint | BabyBear | 215 | 276 (1.3×) | 274 (1.3×) | 286 (1.3×) | 290 (1.3×) |
| div_uint | Goldilocks | 435 | 552 (1.3×) | 541 (1.2×) | 583 (1.3×) | 578 (1.3×) |
| div_uint | BN254 scalar | 5,014 | 9,251 (1.8×) | 6,600 (1.3×) | 12,660 (2.5×) | 8,480 (1.7×) |
| div_uint | secp256k1 base | 4,901 | 8,621 (1.8×) | 6,043 (1.2×) | 12,041 (2.5×) | 8,124 (1.7×) |
| div_uint | P-256 base | 5,029 | 8,822 (1.8×) | 6,207 (1.2×) | 12,166 (2.4×) | 8,046 (1.6×) |
| div_uint | P-256 scalar | 4,958 | 9,209 (1.9×) | 6,545 (1.3×) | 12,668 (2.6×) | 8,637 (1.7×) |
| div_uint | Tom-256 base | 5,057 | 9,148 (1.8×) | 6,642 (1.3×) | 12,264 (2.4×) | 8,484 (1.7×) |
| div_uint | BLS12-381 base | 9,020 | 20,907 (2.3×) | 11,913 (1.3×) | 26,343 (2.9×) | 17,035 (1.9×) |
| div_uint | P-521 base | 14,921 | 34,180 (2.3×) | 18,562 (1.2×) | 49,727 (3.3×) | 30,259 (2.0×) |
| add, constant time | Mersenne-31 | 1.9 | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add, constant time | BabyBear | 1.9 | 2.4 (1.3×) | 2.4 (1.3×) | 3.0 (1.6×) | 3.0 (1.6×) |
| add, constant time | Goldilocks | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.6×) | 3.2 (1.6×) |
| add, constant time | BN254 scalar | 4.9 | 23.7 (4.8×) | 16.1 (3.3×) | 20.8 (4.2×) | 26.1 (5.3×) |
| add, constant time | secp256k1 base | 5.8 | 26.3 (4.5×) | 15.7 (2.7×) | 25.0 (4.3×) | 28.0 (4.8×) |
| add, constant time | P-256 base | 6.0 | 23.5 (3.9×) | 15.6 (2.6×) | 22.8 (3.8×) | 27.0 (4.5×) |
| add, constant time | P-256 scalar | 5.8 | 27.2 (4.7×) | 16.3 (2.8×) | 24.0 (4.1×) | 28.5 (4.9×) |
| add, constant time | Tom-256 base | 6.0 | 27.8 (4.6×) | 16.1 (2.7×) | 24.3 (4.0×) | 29.1 (4.9×) |
| add, constant time | BLS12-381 base | 9.6 | 42.9 (4.5×) | 23.9 (2.5×) | 35.7 (3.7×) | 40.1 (4.2×) |
| add, constant time | P-521 base | 12.2 | 76.7 (6.3×) | 40.7 (3.3×) | 77.1 (6.3×) | 72.0 (5.9×) |
| sub, constant time | Mersenne-31 | 1.9 | 2.5 (1.3×) | 2.5 (1.3×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BabyBear | 1.9 | 2.5 (1.3×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 3.0 (1.6×) | 3.0 (1.6×) |
| sub, constant time | BN254 scalar | 5.0 | 21.3 (4.3×) | 14.1 (2.8×) | 20.7 (4.1×) | 23.1 (4.6×) |
| sub, constant time | secp256k1 base | 5.3 | 19.2 (3.6×) | 13.6 (2.6×) | 18.6 (3.5×) | 22.0 (4.2×) |
| sub, constant time | P-256 base | 5.3 | 17.3 (3.3×) | 12.1 (2.3×) | 18.4 (3.5×) | 21.1 (4.0×) |
| sub, constant time | P-256 scalar | 5.0 | 21.1 (4.2×) | 14.1 (2.8×) | 22.4 (4.5×) | 23.0 (4.6×) |
| sub, constant time | Tom-256 base | 5.5 | 21.1 (3.8×) | 14.1 (2.6×) | 21.0 (3.8×) | 23.0 (4.2×) |
| sub, constant time | BLS12-381 base | 9.7 | 37.6 (3.9×) | 21.4 (2.2×) | 34.9 (3.6×) | 36.1 (3.7×) |
| sub, constant time | P-521 base | 11.7 | 56.2 (4.8×) | 31.2 (2.7×) | 58.1 (5.0×) | 51.1 (4.4×) |
| mul, constant time | Mersenne-31 | 2.7 | 3.7 (1.4×) | 3.6 (1.3×) | 4.0 (1.5×) | 4.0 (1.5×) |
| mul, constant time | BabyBear | 3.4 | 5.2 (1.5×) | 5.2 (1.5×) | 5.2 (1.5×) | 5.0 (1.5×) |
| mul, constant time | Goldilocks | 3.8 | 10.0 (2.6×) | 6.3 (1.7×) | 15.0 (3.9×) | 7.0 (1.8×) |
| mul, constant time | BN254 scalar | 67.8 | 236 (3.5×) | 79.6 (1.2×) | 418 (6.2×) | 164 (2.4×) |
| mul, constant time | secp256k1 base | 44.6 | 162 (3.6×) | 59.6 (1.3×) | 232 (5.2×) | 109 (2.4×) |
| mul, constant time | P-256 base | 49.6 | 177 (3.6×) | 71.9 (1.4×) | 316 (6.4×) | 119 (2.4×) |
| mul, constant time | P-256 scalar | 75.0 | 246 (3.3×) | 84.3 (1.1×) | 329 (4.4×) | 163 (2.2×) |
| mul, constant time | Tom-256 base | 70.0 | 241 (3.4×) | 74.6 (1.1×) | 310 (4.4×) | 163 (2.3×) |
| mul, constant time | BLS12-381 base | 141 | 729 (5.2×) | 183 (1.3×) | 929 (6.6×) | 321 (2.3×) |
| mul, constant time | P-521 base | 176 | 670 (3.8×) | 244 (1.4×) | 1,279 (7.3×) | 533 (3.0×) |
| square, constant time | Mersenne-31 | 2.7 | 3.5 (1.3×) | 3.5 (1.3×) | 3.6 (1.3×) | 3.6 (1.3×) |
| square, constant time | BabyBear | 3.5 | 4.7 (1.3×) | 4.8 (1.4×) | 5.7 (1.6×) | 5.8 (1.7×) |
| square, constant time | Goldilocks | 3.8 | 8.8 (2.3×) | 6.0 (1.6×) | 13.0 (3.4×) | 8.0 (2.1×) |
| square, constant time | BN254 scalar | 64.9 | 218 (3.4×) | 84.5 (1.3×) | 296 (4.6×) | 138 (2.1×) |
| square, constant time | secp256k1 base | 33.8 | 144 (4.3×) | 53.1 (1.6×) | 192 (5.7×) | 88.4 (2.6×) |
| square, constant time | P-256 base | 40.3 | 166 (4.1×) | 74.2 (1.8×) | 260 (6.5×) | 118 (2.9×) |
| square, constant time | P-256 scalar | 64.4 | 220 (3.4×) | 75.7 (1.2×) | 290 (4.5×) | 135 (2.1×) |
| square, constant time | Tom-256 base | 57.0 | 223 (3.9×) | 72.4 (1.3×) | 282 (4.9×) | 164 (2.9×) |
| square, constant time | BLS12-381 base | 134 | 570 (4.3×) | 157 (1.2×) | 826 (6.2×) | 278 (2.1×) |
| square, constant time | P-521 base | 118 | 564 (4.8×) | 238 (2.0×) | 914 (7.7×) | 394 (3.3×) |
| invert, constant time | Mersenne-31 | 243 | 263 (1.1×) | 264 (1.1×) | 267 (1.1×) | 267 (1.1×) |
| invert, constant time | BabyBear | 383 | 451 (1.2×) | 458 (1.2×) | 446 (1.2×) | 417 (1.1×) |
| invert, constant time | Goldilocks | 648 | 1,156 (1.8×) | 841 (1.3×) | 1,417 (2.2×) | 937 (1.4×) |
| invert, constant time | BN254 scalar | 6,758 | 11,998 (1.8×) | 9,522 (1.4×) | 16,747 (2.5×) | 12,585 (1.9×) |
| invert, constant time | secp256k1 base | 6,860 | 11,885 (1.7×) | 9,101 (1.3×) | 16,868 (2.5×) | 12,456 (1.8×) |
| invert, constant time | P-256 base | 6,485 | 11,508 (1.8×) | 8,910 (1.4×) | 16,410 (2.5×) | 12,253 (1.9×) |
| invert, constant time | P-256 scalar | 6,909 | 11,847 (1.7×) | 9,385 (1.4×) | 17,022 (2.5×) | 12,800 (1.9×) |
| invert, constant time | Tom-256 base | 6,973 | 11,824 (1.7×) | 9,271 (1.3×) | 16,657 (2.4×) | 12,803 (1.8×) |
| invert, constant time | BLS12-381 base | 12,456 | 24,064 (1.9×) | 16,804 (1.3×) | 32,675 (2.6×) | 24,014 (1.9×) |
| invert, constant time | P-521 base | 20,025 | 41,920 (2.1×) | 25,931 (1.3×) | 64,412 (3.2×) | 41,531 (2.1×) |
| sqrt, constant time | Mersenne-31 | 142 | 159 (1.1×) | 159 (1.1×) | 170 (1.2×) | 172 (1.2×) |
| sqrt, constant time | BabyBear | 5,592 | 7,140 (1.3×) | 6,465 (1.2×) | 6,697 (1.2×) | 6,492 (1.2×) |
| sqrt, constant time | Goldilocks | 7,149 | 11,050 (1.5×) | 8,393 (1.2×) | 14,306 (2.0×) | 10,377 (1.5×) |
| sqrt, constant time | BN254 scalar | 58,556 | 180,334 (3.1×) | 81,774 (1.4×) | 291,304 (5.0×) | 150,680 (2.6×) |
| sqrt, constant time | secp256k1 base | 12,274 | 41,194 (3.4×) | 16,526 (1.3×) | 65,297 (5.3×) | 26,766 (2.2×) |
| sqrt, constant time | P-256 base | 12,041 | 42,953 (3.6×) | 14,535 (1.2×) | 74,361 (6.2×) | 31,088 (2.6×) |
| sqrt, constant time | P-256 scalar | 25,625 | 77,786 (3.0×) | 34,601 (1.4×) | 117,332 (4.6×) | 65,489 (2.6×) |
| sqrt, constant time | Tom-256 base | 20,048 | 65,835 (3.3×) | 27,142 (1.4×) | 98,940 (4.9×) | 50,439 (2.5×) |
| sqrt, constant time | BLS12-381 base | 69,784 | 299,234 (4.3×) | 93,230 (1.3×) | 395,250 (5.7×) | 206,810 (3.0×) |
| sqrt, constant time | P-521 base | 67,949 | 324,126 (4.8×) | 124,826 (1.8×) | 506,222 (7.5×) | 257,676 (3.8×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 6.8 (1.2×) | 6.7 (1.1×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_u64, constant time | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_u64, constant time | Goldilocks | 6.9 | 17.3 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.1 (1.7×) | 125 (5.7×) | 56.1 (2.6×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 61.6 (5.6×) | 22.3 (2.0×) | 80.1 (7.3×) | 35.9 (3.3×) |
| mul_u64, constant time | P-256 base | 24.4 | 86.1 (3.5×) | 37.2 (1.5×) | 99.1 (4.1×) | 55.3 (2.3×) |
| mul_u64, constant time | P-256 scalar | 22.9 | 102 (4.5×) | 41.4 (1.8×) | 122 (5.3×) | 56.9 (2.5×) |
| mul_u64, constant time | Tom-256 base | 22.4 | 95.3 (4.3×) | 41.0 (1.8×) | 123 (5.5×) | 60.9 (2.7×) |
| mul_u64, constant time | BLS12-381 base | 34.9 | 154 (4.4×) | 62.4 (1.8×) | 203 (5.8×) | 113 (3.2×) |
| mul_u64, constant time | P-521 base | 26.1 | 141 (5.4×) | 64.3 (2.5×) | 165 (6.3×) | 89.8 (3.4×) |
| mul_uint, constant time | Mersenne-31 | 5.8 | 6.8 (1.2×) | 6.7 (1.2×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_uint, constant time | BabyBear | 8.9 | 13.0 (1.5×) | 11.2 (1.3×) | 18.0 (2.0×) | 14.7 (1.7×) |
| mul_uint, constant time | Goldilocks | 6.9 | 17.2 (2.5×) | 7.3 (1.1×) | 24.8 (3.6×) | 9.0 (1.3×) |
| mul_uint, constant time | BN254 scalar | 119 | 490 (4.1×) | 167 (1.4×) | 595 (5.0×) | 331 (2.8×) |
| mul_uint, constant time | secp256k1 base | 78.2 | 322 (4.1×) | 81.7 (1.0×) | 381 (4.9×) | 117 (1.5×) |
| mul_uint, constant time | P-256 base | 118 | 413 (3.5×) | 127 (1.1×) | 491 (4.2×) | 236 (2.0×) |
| mul_uint, constant time | P-256 scalar | 132 | 468 (3.5×) | 165 (1.2×) | 549 (4.2×) | 346 (2.6×) |
| mul_uint, constant time | Tom-256 base | 122 | 469 (3.8×) | 167 (1.4×) | 553 (4.5×) | 329 (2.7×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,088 (4.2×) | 368 (1.4×) | 1,688 (6.6×) | 889 (3.5×) |
| mul_uint, constant time | P-521 base | 426 | 2,008 (4.7×) | 291 (0.7×) | 2,914 (6.8×) | 499 (1.2×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.4 (1.2×) | 5.4 (1.2×) |
| add_u64, constant time | BabyBear | 5.4 | 10.4 (1.9×) | 7.4 (1.4×) | 14.8 (2.7×) | 11.6 (2.1×) |
| add_u64, constant time | Goldilocks | 4.8 | 9.3 (1.9×) | 7.0 (1.5×) | 11.8 (2.5×) | 9.4 (2.0×) |
| add_u64, constant time | BN254 scalar | 23.5 | 93.4 (4.0×) | 41.9 (1.8×) | 108 (4.6×) | 75.5 (3.2×) |
| add_u64, constant time | secp256k1 base | 5.8 | 22.5 (3.9×) | 14.2 (2.4×) | 20.2 (3.5×) | 24.0 (4.1×) |
| add_u64, constant time | P-256 base | 16.9 | 70.9 (4.2×) | 35.9 (2.1×) | 77.5 (4.6×) | 58.0 (3.4×) |
| add_u64, constant time | P-256 scalar | 23.9 | 91.9 (3.8×) | 43.1 (1.8×) | 112 (4.7×) | 73.2 (3.1×) |
| add_u64, constant time | Tom-256 base | 22.3 | 94.3 (4.2×) | 40.9 (1.8×) | 111 (5.0×) | 65.3 (2.9×) |
| add_u64, constant time | BLS12-381 base | 36.6 | 147 (4.0×) | 69.8 (1.9×) | 171 (4.7×) | 116 (3.2×) |
| add_u64, constant time | P-521 base | 10.7 | 65.1 (6.1×) | 39.1 (3.7×) | 72.2 (6.7×) | 62.0 (5.8×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 5.2 (1.2×) | 5.2 (1.2×) | 5.4 (1.2×) | 5.4 (1.2×) |
| add_uint, constant time | BabyBear | 5.4 | 10.4 (1.9×) | 7.4 (1.4×) | 14.8 (2.7×) | 11.6 (2.1×) |
| add_uint, constant time | Goldilocks | 4.7 | 9.3 (2.0×) | 7.0 (1.5×) | 11.8 (2.5×) | 9.4 (2.0×) |
| add_uint, constant time | BN254 scalar | 75.8 | 261 (3.4×) | 95.2 (1.3×) | 294 (3.9×) | 184 (2.4×) |
| add_uint, constant time | secp256k1 base | 9.3 | 41.9 (4.5×) | 24.9 (2.7×) | 36.5 (3.9×) | 40.6 (4.4×) |
| add_uint, constant time | P-256 base | 57.2 | 188 (3.3×) | 96.1 (1.7×) | 287 (5.0×) | 141 (2.5×) |
| add_uint, constant time | P-256 scalar | 83.2 | 256 (3.1×) | 95.9 (1.2×) | 337 (4.1×) | 186 (2.2×) |
| add_uint, constant time | Tom-256 base | 73.7 | 254 (3.4×) | 92.3 (1.3×) | 330 (4.5×) | 180 (2.4×) |
| add_uint, constant time | BLS12-381 base | 156 | 769 (4.9×) | 204 (1.3×) | 959 (6.1×) | 390 (2.5×) |
| add_uint, constant time | P-521 base | 22.3 | 138 (6.2×) | 80.4 (3.6×) | 147 (6.6×) | 128 (5.7×) |
| sub_u64, constant time | Mersenne-31 | 4.6 | 5.0 (1.1×) | 5.0 (1.1×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64, constant time | BabyBear | 5.2 | 11.3 (2.2×) | 7.6 (1.5×) | 14.0 (2.7×) | 10.7 (2.1×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.6 (2.7×) | 9.2 (2.1×) |
| sub_u64, constant time | BN254 scalar | 25.4 | 91.2 (3.6×) | 37.9 (1.5×) | 109 (4.3×) | 67.4 (2.7×) |
| sub_u64, constant time | secp256k1 base | 4.2 | 15.6 (3.7×) | 11.0 (2.6×) | 14.6 (3.5×) | 18.0 (4.3×) |
| sub_u64, constant time | P-256 base | 16.2 | 64.0 (4.0×) | 31.8 (2.0×) | 75.4 (4.7×) | 53.0 (3.3×) |
| sub_u64, constant time | P-256 scalar | 24.1 | 88.5 (3.7×) | 37.4 (1.6×) | 108 (4.5×) | 67.7 (2.8×) |
| sub_u64, constant time | Tom-256 base | 22.9 | 93.6 (4.1×) | 37.7 (1.6×) | 107 (4.7×) | 65.5 (2.9×) |
| sub_u64, constant time | BLS12-381 base | 36.5 | 148 (4.1×) | 59.6 (1.6×) | 203 (5.6×) | 105 (2.9×) |
| sub_u64, constant time | P-521 base | 9.6 | 42.7 (4.4×) | 27.6 (2.9×) | 52.3 (5.4×) | 40.1 (4.2×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 5.0 (1.1×) | 5.0 (1.1×) | 5.2 (1.1×) | 5.1 (1.1×) |
| sub_uint, constant time | BabyBear | 5.2 | 11.3 (2.2×) | 7.6 (1.5×) | 14.0 (2.7×) | 10.7 (2.1×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 6.7 (1.6×) | 11.7 (2.7×) | 9.2 (2.1×) |
| sub_uint, constant time | BN254 scalar | 77.7 | 266 (3.4×) | 96.5 (1.2×) | 330 (4.2×) | 180 (2.3×) |
| sub_uint, constant time | secp256k1 base | 7.2 | 32.9 (4.6×) | 21.5 (3.0×) | 34.2 (4.8×) | 34.8 (4.8×) |
| sub_uint, constant time | P-256 base | 53.1 | 185 (3.5×) | 89.8 (1.7×) | 288 (5.4×) | 136 (2.6×) |
| sub_uint, constant time | P-256 scalar | 80.0 | 244 (3.0×) | 100 (1.2×) | 332 (4.2×) | 180 (2.2×) |
| sub_uint, constant time | Tom-256 base | 74.9 | 251 (3.4×) | 92.4 (1.2×) | 322 (4.3×) | 182 (2.4×) |
| sub_uint, constant time | BLS12-381 base | 154 | 761 (4.9×) | 202 (1.3×) | 942 (6.1×) | 377 (2.4×) |
| sub_uint, constant time | P-521 base | 19.0 | 127 (6.7×) | 66.6 (3.5×) | 123 (6.5×) | 108 (5.7×) |
| from_u64, constant time | Mersenne-31 | 3.8 | 4.7 (1.2×) | 4.7 (1.2×) | 5.2 (1.4×) | 5.2 (1.4×) |
| from_u64, constant time | BabyBear | 4.9 | 10.0 (2.0×) | 6.8 (1.4×) | 13.3 (2.7×) | 10.5 (2.1×) |
| from_u64, constant time | Goldilocks | 3.4 | 7.4 (2.2×) | 6.0 (1.8×) | 9.0 (2.6×) | 7.1 (2.1×) |
| from_u64, constant time | BN254 scalar | 26.2 | 83.8 (3.2×) | 34.7 (1.3×) | 99.3 (3.8×) | 50.0 (1.9×) |
| from_u64, constant time | secp256k1 base | 2.0 | 3.8 (1.9×) | 3.8 (1.9×) | 3.8 (1.9×) | 3.8 (1.9×) |
| from_u64, constant time | P-256 base | 18.6 | 52.1 (2.8×) | 26.6 (1.4×) | 61.1 (3.3×) | 38.5 (2.1×) |
| from_u64, constant time | P-256 scalar | 27.2 | 84.5 (3.1×) | 34.5 (1.3×) | 89.6 (3.3×) | 50.1 (1.8×) |
| from_u64, constant time | Tom-256 base | 23.5 | 82.2 (3.5×) | 35.4 (1.5×) | 97.7 (4.2×) | 49.0 (2.1×) |
| from_u64, constant time | BLS12-381 base | 41.9 | 147 (3.5×) | 56.7 (1.4×) | 194 (4.6×) | 90.4 (2.2×) |
| from_u64, constant time | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.1 (1.2×) |
| div_u64, constant time | BN254 scalar | 6,851 | 12,281 (1.8×) | 9,492 (1.4×) | 17,516 (2.6×) | 12,945 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,776 | 11,984 (1.8×) | 9,093 (1.3×) | 16,969 (2.5×) | 12,950 (1.9×) |
| div_u64, constant time | P-256 base | 6,666 | 11,703 (1.8×) | 9,231 (1.4×) | 16,841 (2.5×) | 12,492 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,956 | 12,262 (1.8×) | 9,463 (1.4×) | 17,475 (2.5×) | 13,548 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,282 | 12,096 (1.7×) | 9,302 (1.3×) | 16,729 (2.3×) | 13,005 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,384 | 24,742 (2.0×) | 16,879 (1.4×) | 34,783 (2.8×) | 24,131 (1.9×) |
| div_u64, constant time | P-521 base | 20,382 | 42,369 (2.1×) | 25,912 (1.3×) | 64,618 (3.2×) | 43,159 (2.1×) |
| div_uint, constant time | Mersenne-31 | 262 | 272 (1.0×) | 272 (1.0×) | 280 (1.1×) | 280 (1.1×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 465 (1.2×) | 474 (1.2×) | 442 (1.1×) |
| div_uint, constant time | Goldilocks | 668 | 1,146 (1.7×) | 862 (1.3×) | 1,437 (2.2×) | 969 (1.5×) |
| div_uint, constant time | BN254 scalar | 6,895 | 12,555 (1.8×) | 9,501 (1.4×) | 17,700 (2.6×) | 13,226 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,771 | 11,996 (1.8×) | 9,093 (1.3×) | 16,991 (2.5×) | 12,681 (1.9×) |
| div_uint, constant time | P-256 base | 6,740 | 11,996 (1.8×) | 9,129 (1.4×) | 17,413 (2.6×) | 12,560 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,952 | 12,334 (1.8×) | 9,543 (1.4×) | 17,880 (2.6×) | 13,834 (2.0×) |
| div_uint, constant time | Tom-256 base | 7,337 | 12,388 (1.7×) | 9,441 (1.3×) | 17,218 (2.3×) | 13,151 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,490 | 25,420 (2.0×) | 17,124 (1.4×) | 35,327 (2.8×) | 24,762 (2.0×) |
| div_uint, constant time | P-521 base | 20,380 | 42,588 (2.1×) | 26,212 (1.3×) | 65,037 (3.2×) | 43,022 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.7 | 13.3 (2.0×) | 10.9 (1.6×) | 13.4 (2.0×) | 18.0 (2.7×) |
| add | secp256k1 base | 6.9 | 13.7 (2.0×) | 10.7 (1.6×) | 13.3 (1.9×) | 18.8 (2.7×) |
| add | P-256 base | 6.9 | 12.4 (1.8×) | 11.3 (1.6×) | 12.9 (1.9×) | 18.4 (2.7×) |
| add | P-256 scalar | 6.9 | 13.6 (2.0×) | 11.1 (1.6×) | 14.0 (2.0×) | 19.0 (2.8×) |
| add | Tom-256 base | 6.9 | 13.4 (1.9×) | 11.2 (1.6×) | 13.5 (2.0×) | 18.6 (2.7×) |
| add | BLS12-381 base | 8.9 | 21.9 (2.5×) | 18.7 (2.1×) | 23.0 (2.6×) | 29.8 (3.3×) |
| add | P-521 base | 14.7 | 42.2 (2.9×) | 27.2 (1.9×) | 48.6 (3.3×) | 47.9 (3.3×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.7 | 13.2 (2.0×) | 10.8 (1.6×) | 12.4 (1.9×) | 17.4 (2.6×) |
| sub | secp256k1 base | 8.0 | 16.6 (2.1×) | 10.4 (1.3×) | 13.9 (1.7×) | 16.8 (2.1×) |
| sub | P-256 base | 7.0 | 13.4 (1.9×) | 10.4 (1.5×) | 12.4 (1.8×) | 16.4 (2.3×) |
| sub | P-256 scalar | 6.9 | 14.3 (2.1×) | 10.4 (1.5×) | 13.9 (2.0×) | 17.3 (2.5×) |
| sub | Tom-256 base | 8.0 | 13.1 (1.6×) | 10.7 (1.3×) | 12.4 (1.6×) | 17.3 (2.2×) |
| sub | BLS12-381 base | 9.9 | 22.8 (2.3×) | 17.3 (1.7×) | 22.7 (2.3×) | 29.5 (3.0×) |
| sub | P-521 base | 12.4 | 43.5 (3.5×) | 25.7 (2.1×) | 44.9 (3.6×) | 44.0 (3.5×) |
| mul | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul | BabyBear | 10.9 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul | Goldilocks | 8.9 | 16.6 (1.9×) | 12.5 (1.4×) | 18.0 (2.0×) | 12.3 (1.4×) |
| mul | BN254 scalar | 72.5 | 227 (3.1×) | 88.5 (1.2×) | 368 (5.1×) | 153 (2.1×) |
| mul | secp256k1 base | 54.8 | 174 (3.2×) | 69.2 (1.3×) | 241 (4.4×) | 106 (1.9×) |
| mul | P-256 base | 52.5 | 190 (3.6×) | 65.0 (1.2×) | 292 (5.6×) | 118 (2.2×) |
| mul | P-256 scalar | 84.0 | 245 (2.9×) | 88.0 (1.0×) | 326 (3.9×) | 160 (1.9×) |
| mul | Tom-256 base | 76.0 | 234 (3.1×) | 80.7 (1.1×) | 314 (4.1×) | 158 (2.1×) |
| mul | BLS12-381 base | 141 | 722 (5.1×) | 187 (1.3×) | 928 (6.6×) | 323 (2.3×) |
| mul | P-521 base | 191 | 686 (3.6×) | 240 (1.3×) | 1,294 (6.8×) | 439 (2.3×) |
| square | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square | Goldilocks | 9.0 | 15.0 (1.7×) | 11.0 (1.2×) | 18.6 (2.1×) | 12.5 (1.4×) |
| square | BN254 scalar | 66.7 | 210 (3.1×) | 81.4 (1.2×) | 324 (4.9×) | 133 (2.0×) |
| square | secp256k1 base | 42.9 | 152 (3.5×) | 58.9 (1.4×) | 160 (3.7×) | 88.3 (2.1×) |
| square | P-256 base | 44.0 | 160 (3.6×) | 55.6 (1.3×) | 246 (5.6×) | 108 (2.5×) |
| square | P-256 scalar | 72.9 | 220 (3.0×) | 81.7 (1.1×) | 242 (3.3×) | 141 (1.9×) |
| square | Tom-256 base | 64.5 | 213 (3.3×) | 77.7 (1.2×) | 210 (3.3×) | 124 (1.9×) |
| square | BLS12-381 base | 137 | 569 (4.2×) | 161 (1.2×) | 800 (5.8×) | 276 (2.0×) |
| square | P-521 base | 129 | 571 (4.4×) | 197 (1.5×) | 905 (7.0×) | 387 (3.0×) |
| mul_u64 | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64 | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 13.0 (1.2×) |
| mul_u64 | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 15.1 (1.7×) |
| mul_u64 | BN254 scalar | 25.9 | 91.7 (3.5×) | 34.5 (1.3×) | 120 (4.6×) | 45.3 (1.7×) |
| mul_u64 | secp256k1 base | 14.9 | 49.0 (3.3×) | 16.0 (1.1×) | 65.0 (4.4×) | 23.0 (1.5×) |
| mul_u64 | P-256 base | 24.9 | 75.0 (3.0×) | 33.0 (1.3×) | 89.2 (3.6×) | 41.8 (1.7×) |
| mul_u64 | P-256 scalar | 27.9 | 82.7 (3.0×) | 34.5 (1.2×) | 105 (3.8×) | 47.4 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.5 (3.0×) | 33.0 (1.2×) | 105 (3.8×) | 45.4 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 125 (3.6×) | 47.9 (1.4×) | 177 (5.1×) | 62.1 (1.8×) |
| mul_u64 | P-521 base | 32.7 | 96.8 (3.0×) | 39.5 (1.2×) | 196 (6.0×) | 52.8 (1.6×) |
| mul_uint | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint | BabyBear | 10.9 | 16.2 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 13.0 (1.2×) |
| mul_uint | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 15.0 (1.6×) |
| mul_uint | BN254 scalar | 114 | 441 (3.9×) | 162 (1.4×) | 497 (4.4×) | 298 (2.6×) |
| mul_uint | secp256k1 base | 67.9 | 266 (3.9×) | 68.9 (1.0×) | 320 (4.7×) | 106 (1.6×) |
| mul_uint | P-256 base | 110 | 352 (3.2×) | 127 (1.2×) | 424 (3.9×) | 223 (2.0×) |
| mul_uint | P-256 scalar | 116 | 425 (3.7×) | 162 (1.4×) | 479 (4.1×) | 315 (2.7×) |
| mul_uint | Tom-256 base | 114 | 410 (3.6×) | 160 (1.4×) | 488 (4.3×) | 305 (2.7×) |
| mul_uint | BLS12-381 base | 222 | 974 (4.4×) | 352 (1.6×) | 1,139 (5.1×) | 889 (4.0×) |
| mul_uint | P-521 base | 406 | 1,567 (3.9×) | 248 (0.6×) | 2,381 (5.9×) | 458 (1.1×) |
| add_u64 | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) |
| add_u64 | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_u64 | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 12.0 (2.6×) | 9.2 (2.0×) |
| add_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 42.9 (1.7×) | 138 (5.5×) | 65.6 (2.6×) |
| add_u64 | secp256k1 base | 7.4 | 7.0 (0.9×) | 6.0 (0.8×) | 7.0 (0.9×) | 10.0 (1.4×) |
| add_u64 | P-256 base | 22.1 | 67.5 (3.1×) | 34.8 (1.6×) | 77.7 (3.5×) | 56.3 (2.5×) |
| add_u64 | P-256 scalar | 22.7 | 66.8 (2.9×) | 38.0 (1.7×) | 74.7 (3.3×) | 65.1 (2.9×) |
| add_u64 | Tom-256 base | 26.0 | 84.5 (3.2×) | 39.6 (1.5×) | 99.6 (3.8×) | 66.0 (2.5×) |
| add_u64 | BLS12-381 base | 37.5 | 161 (4.3×) | 62.2 (1.7×) | 254 (6.8×) | 89.4 (2.4×) |
| add_u64 | P-521 base | 12.7 | 21.1 (1.7×) | 12.2 (1.0×) | 19.4 (1.5×) | 21.4 (1.7×) |
| add_uint | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) |
| add_uint | BabyBear | 7.0 | 12.2 (1.7×) | 8.2 (1.2×) | 12.7 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.7 (2.5×) | 9.2 (2.0×) |
| add_uint | BN254 scalar | 73.1 | 226 (3.1×) | 96.8 (1.3×) | 370 (5.1×) | 179 (2.4×) |
| add_uint | secp256k1 base | 8.5 | 17.5 (2.1×) | 12.5 (1.5×) | 16.8 (2.0×) | 23.1 (2.7×) |
| add_uint | P-256 base | 57.8 | 180 (3.1×) | 73.5 (1.3×) | 325 (5.6×) | 147 (2.5×) |
| add_uint | P-256 scalar | 78.1 | 251 (3.2×) | 99.1 (1.3×) | 322 (4.1×) | 186 (2.4×) |
| add_uint | Tom-256 base | 75.2 | 257 (3.4×) | 96.0 (1.3×) | 327 (4.3×) | 180 (2.4×) |
| add_uint | BLS12-381 base | 151 | 764 (5.1×) | 195 (1.3×) | 942 (6.2×) | 389 (2.6×) |
| add_uint | P-521 base | 23.6 | 62.0 (2.6×) | 41.6 (1.8×) | 85.0 (3.6×) | 66.9 (2.8×) |
| sub_u64 | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_u64 | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 12.0 (1.7×) | 10.8 (1.5×) |
| sub_u64 | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 10.5 (2.5×) | 8.7 (2.1×) |
| sub_u64 | BN254 scalar | 24.9 | 113 (4.5×) | 44.9 (1.8×) | 136 (5.5×) | 66.5 (2.7×) |
| sub_u64 | secp256k1 base | 6.9 | 7.0 (1.0×) | 6.0 (0.9×) | 7.0 (1.0×) | 9.0 (1.3×) |
| sub_u64 | P-256 base | 21.7 | 69.5 (3.2×) | 34.1 (1.6×) | 78.1 (3.6×) | 54.3 (2.5×) |
| sub_u64 | P-256 scalar | 23.7 | 66.8 (2.8×) | 36.6 (1.5×) | 72.6 (3.1×) | 61.5 (2.6×) |
| sub_u64 | Tom-256 base | 25.0 | 85.4 (3.4×) | 39.5 (1.6×) | 96.7 (3.9×) | 64.3 (2.6×) |
| sub_u64 | BLS12-381 base | 38.6 | 155 (4.0×) | 60.2 (1.6×) | 264 (6.8×) | 89.8 (2.3×) |
| sub_u64 | P-521 base | 11.0 | 21.2 (1.9×) | 13.1 (1.2×) | 21.6 (2.0×) | 20.7 (1.9×) |
| sub_uint | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_uint | BabyBear | 7.0 | 11.9 (1.7×) | 8.2 (1.2×) | 12.0 (1.7×) | 10.9 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 10.5 (2.5×) | 8.7 (2.1×) |
| sub_uint | BN254 scalar | 73.1 | 233 (3.2×) | 92.7 (1.3×) | 349 (4.8×) | 177 (2.4×) |
| sub_uint | secp256k1 base | 6.8 | 19.0 (2.8×) | 12.0 (1.8×) | 16.4 (2.4×) | 20.9 (3.1×) |
| sub_uint | P-256 base | 56.8 | 177 (3.1×) | 71.0 (1.2×) | 251 (4.4×) | 133 (2.3×) |
| sub_uint | P-256 scalar | 77.7 | 261 (3.4×) | 95.9 (1.2×) | 328 (4.2×) | 175 (2.3×) |
| sub_uint | Tom-256 base | 72.6 | 252 (3.5×) | 93.1 (1.3×) | 324 (4.5×) | 172 (2.4×) |
| sub_uint | BLS12-381 base | 155 | 756 (4.9×) | 197 (1.3×) | 937 (6.0×) | 394 (2.5×) |
| sub_uint | P-521 base | 15.5 | 67.4 (4.3×) | 37.2 (2.4×) | 80.1 (5.2×) | 65.7 (4.2×) |
| div_u64 | BN254 scalar | 604 | 1,205 (2.0×) | 838 (1.4×) | 1,372 (2.3×) | 1,115 (1.8×) |
| div_u64 | secp256k1 base | 566 | 1,015 (1.8×) | 717 (1.3×) | 1,224 (2.2×) | 821 (1.5×) |
| div_u64 | P-256 base | 603 | 1,085 (1.8×) | 797 (1.3×) | 1,343 (2.2×) | 977 (1.6×) |
| div_u64 | P-256 scalar | 619 | 1,187 (1.9×) | 844 (1.4×) | 1,397 (2.3×) | 1,099 (1.8×) |
| div_u64 | Tom-256 base | 609 | 1,201 (2.0×) | 840 (1.4×) | 1,496 (2.5×) | 1,034 (1.7×) |
| div_u64 | BLS12-381 base | 751 | 1,801 (2.4×) | 1,074 (1.4×) | 2,459 (3.3×) | 1,752 (2.3×) |
| div_u64 | P-521 base | 924 | 2,453 (2.7×) | 999 (1.1×) | 3,626 (3.9×) | 1,513 (1.6×) |
| div_uint | Mersenne-31 | 217 | 274 (1.3×) | 273 (1.3×) | 278 (1.3×) | 281 (1.3×) |
| div_uint | BabyBear | 217 | 275 (1.3×) | 278 (1.3×) | 277 (1.3×) | 280 (1.3×) |
| div_uint | Goldilocks | 435 | 549 (1.3×) | 539 (1.2×) | 583 (1.3×) | 571 (1.3×) |
| div_uint | BN254 scalar | 4,982 | 9,123 (1.8×) | 6,619 (1.3×) | 12,788 (2.6×) | 8,467 (1.7×) |
| div_uint | secp256k1 base | 4,882 | 8,564 (1.8×) | 6,015 (1.2×) | 11,939 (2.4×) | 8,053 (1.6×) |
| div_uint | P-256 base | 5,018 | 8,755 (1.7×) | 6,139 (1.2×) | 12,180 (2.4×) | 7,993 (1.6×) |
| div_uint | P-256 scalar | 4,943 | 9,091 (1.8×) | 6,497 (1.3×) | 12,657 (2.6×) | 8,634 (1.7×) |
| div_uint | Tom-256 base | 5,051 | 8,995 (1.8×) | 6,532 (1.3×) | 12,246 (2.4×) | 8,458 (1.7×) |
| div_uint | BLS12-381 base | 9,014 | 20,789 (2.3×) | 12,136 (1.3×) | 26,283 (2.9×) | 16,833 (1.9×) |
| div_uint | P-521 base | 14,980 | 34,468 (2.3×) | 18,158 (1.2×) | 49,587 (3.3×) | 30,275 (2.0×) |
| add, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.7 | 24.3 (3.6×) | 15.7 (2.3×) | 23.2 (3.5×) | 26.0 (3.9×) |
| add, constant time | secp256k1 base | 6.9 | 28.3 (4.1×) | 15.4 (2.2×) | 26.8 (3.9×) | 28.4 (4.1×) |
| add, constant time | P-256 base | 7.0 | 26.5 (3.8×) | 15.0 (2.1×) | 25.0 (3.6×) | 27.5 (3.9×) |
| add, constant time | P-256 scalar | 6.8 | 27.3 (4.0×) | 17.9 (2.6×) | 27.2 (4.0×) | 29.1 (4.3×) |
| add, constant time | Tom-256 base | 7.0 | 27.7 (4.0×) | 15.9 (2.3×) | 27.0 (3.9×) | 29.0 (4.1×) |
| add, constant time | BLS12-381 base | 8.9 | 41.3 (4.6×) | 23.4 (2.6×) | 35.4 (4.0×) | 40.7 (4.6×) |
| add, constant time | P-521 base | 15.2 | 77.5 (5.1×) | 39.8 (2.6×) | 80.5 (5.3×) | 72.1 (4.7×) |
| sub, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 6.8 | 21.6 (3.2×) | 13.3 (2.0×) | 20.8 (3.1×) | 21.5 (3.2×) |
| sub, constant time | secp256k1 base | 8.0 | 21.4 (2.7×) | 12.4 (1.6×) | 20.5 (2.6×) | 21.0 (2.6×) |
| sub, constant time | P-256 base | 6.7 | 19.2 (2.9×) | 12.6 (1.9×) | 18.5 (2.8×) | 19.0 (2.8×) |
| sub, constant time | P-256 scalar | 6.8 | 21.7 (3.2×) | 13.6 (2.0×) | 20.6 (3.0×) | 21.0 (3.1×) |
| sub, constant time | Tom-256 base | 8.3 | 21.4 (2.6×) | 13.3 (1.6×) | 19.7 (2.4×) | 21.5 (2.6×) |
| sub, constant time | BLS12-381 base | 9.6 | 37.3 (3.9×) | 22.0 (2.3×) | 37.0 (3.9×) | 35.1 (3.7×) |
| sub, constant time | P-521 base | 12.3 | 57.6 (4.7×) | 31.8 (2.6×) | 60.1 (4.9×) | 51.1 (4.2×) |
| mul, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| mul, constant time | Goldilocks | 8.9 | 16.6 (1.9×) | 12.5 (1.4×) | 18.0 (2.0×) | 12.3 (1.4×) |
| mul, constant time | BN254 scalar | 78.8 | 242 (3.1×) | 89.2 (1.1×) | 413 (5.2×) | 151 (1.9×) |
| mul, constant time | secp256k1 base | 55.0 | 174 (3.2×) | 69.1 (1.3×) | 233 (4.2×) | 106 (1.9×) |
| mul, constant time | P-256 base | 52.5 | 192 (3.7×) | 64.5 (1.2×) | 295 (5.6×) | 119 (2.3×) |
| mul, constant time | P-256 scalar | 84.0 | 243 (2.9×) | 88.1 (1.0×) | 331 (3.9×) | 159 (1.9×) |
| mul, constant time | Tom-256 base | 76.0 | 239 (3.1×) | 80.2 (1.1×) | 309 (4.1×) | 153 (2.0×) |
| mul, constant time | BLS12-381 base | 154 | 722 (4.7×) | 185 (1.2×) | 910 (5.9×) | 322 (2.1×) |
| mul, constant time | P-521 base | 188 | 686 (3.6×) | 239 (1.3×) | 1,296 (6.9×) | 438 (2.3×) |
| square, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square, constant time | BabyBear | 11.0 | 12.0 (1.1×) | 11.0 (1.0×) | 12.0 (1.1×) | 11.0 (1.0×) |
| square, constant time | Goldilocks | 9.0 | 15.1 (1.7×) | 11.0 (1.2×) | 18.5 (2.1×) | 12.5 (1.4×) |
| square, constant time | BN254 scalar | 66.9 | 223 (3.3×) | 81.4 (1.2×) | 185 (2.8×) | 133 (2.0×) |
| square, constant time | secp256k1 base | 42.9 | 152 (3.5×) | 58.8 (1.4×) | 159 (3.7×) | 88.2 (2.1×) |
| square, constant time | P-256 base | 44.0 | 160 (3.6×) | 55.5 (1.3×) | 237 (5.4×) | 108 (2.5×) |
| square, constant time | P-256 scalar | 72.9 | 220 (3.0×) | 81.5 (1.1×) | 286 (3.9×) | 141 (1.9×) |
| square, constant time | Tom-256 base | 64.5 | 211 (3.3×) | 77.4 (1.2×) | 210 (3.3×) | 125 (1.9×) |
| square, constant time | BLS12-381 base | 145 | 567 (3.9×) | 160 (1.1×) | 798 (5.5×) | 280 (1.9×) |
| square, constant time | P-521 base | 129 | 571 (4.4×) | 196 (1.5×) | 906 (7.0×) | 388 (3.0×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64, constant time | BabyBear | 10.9 | 16.2 (1.5×) | 11.0 (1.0×) | 15.5 (1.4×) | 13.0 (1.2×) |
| mul_u64, constant time | Goldilocks | 9.1 | 17.8 (2.0×) | 14.1 (1.5×) | 25.3 (2.8×) | 13.6 (1.5×) |
| mul_u64, constant time | BN254 scalar | 30.4 | 109 (3.6×) | 48.4 (1.6×) | 132 (4.3×) | 59.5 (2.0×) |
| mul_u64, constant time | secp256k1 base | 19.8 | 68.6 (3.5×) | 29.0 (1.5×) | 85.1 (4.3×) | 39.5 (2.0×) |
| mul_u64, constant time | P-256 base | 33.8 | 88.2 (2.6×) | 46.5 (1.4×) | 104 (3.1×) | 60.4 (1.8×) |
| mul_u64, constant time | P-256 scalar | 33.9 | 105 (3.1×) | 47.9 (1.4×) | 139 (4.1×) | 70.3 (2.1×) |
| mul_u64, constant time | Tom-256 base | 32.9 | 100 (3.0×) | 47.9 (1.5×) | 122 (3.7×) | 67.9 (2.1×) |
| mul_u64, constant time | BLS12-381 base | 41.6 | 163 (3.9×) | 64.3 (1.5×) | 195 (4.7×) | 81.1 (1.9×) |
| mul_u64, constant time | P-521 base | 45.7 | 146 (3.2×) | 65.1 (1.4×) | 188 (4.1×) | 90.3 (2.0×) |
| mul_uint, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint, constant time | BabyBear | 10.9 | 16.1 (1.5×) | 11.0 (1.0×) | 15.7 (1.4×) | 13.0 (1.2×) |
| mul_uint, constant time | Goldilocks | 9.0 | 17.8 (2.0×) | 14.1 (1.6×) | 25.3 (2.8×) | 15.0 (1.7×) |
| mul_uint, constant time | BN254 scalar | 141 | 482 (3.4×) | 162 (1.1×) | 594 (4.2×) | 298 (2.1×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 319 (3.8×) | 71.3 (0.9×) | 382 (4.6×) | 117 (1.4×) |
| mul_uint, constant time | P-256 base | 134 | 406 (3.0×) | 124 (0.9×) | 489 (3.6×) | 232 (1.7×) |
| mul_uint, constant time | P-256 scalar | 146 | 477 (3.3×) | 167 (1.1×) | 540 (3.7×) | 315 (2.2×) |
| mul_uint, constant time | Tom-256 base | 135 | 466 (3.5×) | 161 (1.2×) | 554 (4.1×) | 308 (2.3×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,088 (4.1×) | 351 (1.3×) | 1,695 (6.4×) | 890 (3.3×) |
| mul_uint, constant time | P-521 base | 472 | 2,021 (4.3×) | 283 (0.6×) | 2,900 (6.1×) | 589 (1.2×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.7 (1.0×) | 4.8 (1.1×) |
| add_u64, constant time | BabyBear | 4.9 | 11.9 (2.4×) | 7.2 (1.5×) | 13.0 (2.7×) | 11.3 (2.3×) |
| add_u64, constant time | Goldilocks | 5.0 | 8.5 (1.7×) | 6.5 (1.3×) | 11.8 (2.4×) | 9.2 (1.8×) |
| add_u64, constant time | BN254 scalar | 21.9 | 93.0 (4.2×) | 40.7 (1.9×) | 107 (4.9×) | 69.8 (3.2×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.4 (3.4×) | 15.0 (2.0×) | 25.2 (3.4×) | 24.0 (3.2×) |
| add_u64, constant time | P-256 base | 17.1 | 66.5 (3.9×) | 35.4 (2.1×) | 80.1 (4.7×) | 58.1 (3.4×) |
| add_u64, constant time | P-256 scalar | 24.8 | 105 (4.2×) | 40.7 (1.6×) | 109 (4.4×) | 73.2 (3.0×) |
| add_u64, constant time | Tom-256 base | 22.3 | 97.7 (4.4×) | 40.4 (1.8×) | 110 (4.9×) | 70.2 (3.1×) |
| add_u64, constant time | BLS12-381 base | 35.8 | 142 (4.0×) | 63.8 (1.8×) | 158 (4.4×) | 107 (3.0×) |
| add_u64, constant time | P-521 base | 12.9 | 66.1 (5.1×) | 36.6 (2.8×) | 68.6 (5.3×) | 62.0 (4.8×) |
| add_uint, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) | 4.7 (1.0×) |
| add_uint, constant time | BabyBear | 4.9 | 11.6 (2.4×) | 7.2 (1.5×) | 13.0 (2.7×) | 11.3 (2.3×) |
| add_uint, constant time | Goldilocks | 4.7 | 8.5 (1.8×) | 6.5 (1.4×) | 11.8 (2.5×) | 9.2 (2.0×) |
| add_uint, constant time | BN254 scalar | 74.8 | 255 (3.4×) | 96.6 (1.3×) | 317 (4.2×) | 181 (2.4×) |
| add_uint, constant time | secp256k1 base | 8.8 | 38.4 (4.4×) | 25.1 (2.9×) | 36.3 (4.1×) | 40.6 (4.6×) |
| add_uint, constant time | P-256 base | 57.8 | 179 (3.1×) | 74.3 (1.3×) | 249 (4.3×) | 141 (2.4×) |
| add_uint, constant time | P-256 scalar | 78.8 | 263 (3.3×) | 99.1 (1.3×) | 318 (4.0×) | 193 (2.4×) |
| add_uint, constant time | Tom-256 base | 75.1 | 255 (3.4×) | 96.8 (1.3×) | 326 (4.3×) | 181 (2.4×) |
| add_uint, constant time | BLS12-381 base | 154 | 757 (4.9×) | 196 (1.3×) | 947 (6.1×) | 380 (2.5×) |
| add_uint, constant time | P-521 base | 22.6 | 144 (6.4×) | 78.2 (3.5×) | 147 (6.5×) | 131 (5.8×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.8 (1.1×) |
| sub_u64, constant time | BabyBear | 4.8 | 12.2 (2.5×) | 7.1 (1.5×) | 13.0 (2.7×) | 10.7 (2.2×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.2 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.7 (2.1×) |
| sub_u64, constant time | BN254 scalar | 21.9 | 94.8 (4.3×) | 36.7 (1.7×) | 107 (4.9×) | 66.9 (3.1×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.0 (2.6×) | 12.0 (1.7×) | 17.0 (2.5×) | 15.3 (2.2×) |
| sub_u64, constant time | P-256 base | 16.1 | 61.4 (3.8×) | 31.3 (1.9×) | 74.6 (4.6×) | 53.1 (3.3×) |
| sub_u64, constant time | P-256 scalar | 22.7 | 95.9 (4.2×) | 37.3 (1.6×) | 105 (4.6×) | 64.0 (2.8×) |
| sub_u64, constant time | Tom-256 base | 23.9 | 91.9 (3.8×) | 36.9 (1.5×) | 105 (4.4×) | 66.0 (2.8×) |
| sub_u64, constant time | BLS12-381 base | 35.1 | 138 (3.9×) | 59.6 (1.7×) | 174 (5.0×) | 103 (2.9×) |
| sub_u64, constant time | P-521 base | 11.0 | 45.9 (4.2×) | 27.1 (2.5×) | 49.7 (4.5×) | 40.2 (3.7×) |
| sub_uint, constant time | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 4.9 (1.1×) | 4.9 (1.1×) |
| sub_uint, constant time | BabyBear | 4.8 | 12.1 (2.5×) | 7.1 (1.5×) | 13.0 (2.7×) | 10.6 (2.2×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.3 (2.0×) | 6.1 (1.5×) | 11.0 (2.6×) | 8.7 (2.1×) |
| sub_uint, constant time | BN254 scalar | 74.1 | 241 (3.3×) | 91.8 (1.2×) | 287 (3.9×) | 179 (2.4×) |
| sub_uint, constant time | secp256k1 base | 7.8 | 32.4 (4.2×) | 20.5 (2.6×) | 31.7 (4.1×) | 35.4 (4.5×) |
| sub_uint, constant time | P-256 base | 56.8 | 180 (3.2×) | 71.6 (1.3×) | 247 (4.3×) | 132 (2.3×) |
| sub_uint, constant time | P-256 scalar | 76.9 | 252 (3.3×) | 98.5 (1.3×) | 331 (4.3×) | 181 (2.4×) |
| sub_uint, constant time | Tom-256 base | 72.8 | 246 (3.4×) | 94.4 (1.3×) | 328 (4.5×) | 177 (2.4×) |
| sub_uint, constant time | BLS12-381 base | 156 | 761 (4.9×) | 197 (1.3×) | 941 (6.0×) | 379 (2.4×) |
| sub_uint, constant time | P-521 base | 19.5 | 128 (6.6×) | 67.3 (3.5×) | 121 (6.2×) | 108 (5.5×) |
| div_u64, constant time | BN254 scalar | 6,843 | 12,358 (1.8×) | 9,433 (1.4×) | 17,330 (2.5×) | 13,038 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,780 | 11,889 (1.8×) | 9,059 (1.3×) | 16,945 (2.5×) | 12,582 (1.9×) |
| div_u64, constant time | P-256 base | 6,654 | 11,866 (1.8×) | 9,122 (1.4×) | 17,021 (2.6×) | 12,387 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,895 | 12,240 (1.8×) | 9,376 (1.4×) | 17,515 (2.5×) | 13,344 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,274 | 12,216 (1.7×) | 9,267 (1.3×) | 16,743 (2.3×) | 12,968 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,367 | 24,761 (2.0×) | 16,913 (1.4×) | 34,565 (2.8×) | 24,110 (1.9×) |
| div_u64, constant time | P-521 base | 20,375 | 43,062 (2.1×) | 26,297 (1.3×) | 64,852 (3.2×) | 43,359 (2.1×) |
| div_uint, constant time | Mersenne-31 | 250 | 275 (1.1×) | 275 (1.1×) | 274 (1.1×) | 274 (1.1×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 435 (1.1×) | 470 (1.2×) | 435 (1.1×) |
| div_uint, constant time | Goldilocks | 665 | 1,138 (1.7×) | 859 (1.3×) | 1,431 (2.2×) | 966 (1.5×) |
| div_uint, constant time | BN254 scalar | 6,894 | 12,371 (1.8×) | 9,492 (1.4×) | 17,469 (2.5×) | 13,228 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,779 | 11,870 (1.8×) | 9,086 (1.3×) | 16,834 (2.5×) | 12,859 (1.9×) |
| div_uint, constant time | P-256 base | 6,690 | 11,898 (1.8×) | 9,073 (1.4×) | 17,266 (2.6×) | 12,619 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,912 | 12,592 (1.8×) | 9,424 (1.4×) | 17,650 (2.6×) | 13,662 (2.0×) |
| div_uint, constant time | Tom-256 base | 7,335 | 12,437 (1.7×) | 9,389 (1.3×) | 17,060 (2.3×) | 13,198 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,516 | 25,368 (2.0×) | 16,957 (1.4×) | 35,309 (2.8×) | 24,696 (2.0×) |
| div_uint, constant time | P-521 base | 20,375 | 42,958 (2.1×) | 26,111 (1.3×) | 64,813 (3.2×) | 42,943 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox>   # the WebAssembly builds
cargo run -p prime-field-bench-report -- report   # this file
```
