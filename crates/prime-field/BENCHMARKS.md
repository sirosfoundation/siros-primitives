# Benchmarks

`prime-field` at commit `18979ad` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-08.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Run 1 | `806f19c`, 2026-11-08: 906 cells; CPU 9; 1-minute load 12.3 before, 11.4 after; its SMT sibling, CPU 25, 7% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each native figure: over 31 repetitions, the fewest cycles of 2n operations less the fewest of n, over n. A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more, up to 3 times. Each such round ran every case again, in the order of the first round, and kept the counts of that case alone.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands: canonical residues, the same for every crate.
- Integer operands `k`: uniform of exactly 64 bits, or of exactly the bit length of `p`, which may reach `p`. A crate without an operation on an integer runs its own conversion of `k` and then its operation on two elements.
- Divisors `k`: nonzero, uniform of exactly 64 bits over the fields of more than 64 bits, or of exactly one bit less than `p`. A crate without a division by an integer runs its own conversion of `k`, its inverse and a product; `prime-field × invert` is that composition in `prime-field`.
- Every result matches `num-bigint` before it is measured.
- `prime-field` runs `Fp` in the tables without a label and `ConstantTime` in the constant-time tables.
- `prime-field` holds each element in Montgomery form, which favours `mul`. `k256/p256` holds a P-256 scalar as a plain integer, which favours `add_u64`, `sub_u64`, `add_uint`, `sub_uint` and `from_u64` over that field.
- Over BabyBear in WebAssembly without wide arithmetic, `mul` takes the reduction with the fewest cycles summed over Chrome and Firefox, which leaves Chrome's throughput and the latency of both browsers behind `plonky3`.
- Over Goldilocks with wide arithmetic, `sub_u64` and `sub_uint` take the correction with the higher throughput, which costs Firefox latency against `winter-math`.

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
| Mersenne-31 | **1.9** / **2.9** | – | 9.6 / 17.5 | – | – | 2.0 / 3.0 | – | 2.5 / 4.9 | – |
| BabyBear | **1.9** / **2.9** | – | 9.6 / 17.5 | – | – | 2.0 / 3.0 | – | 2.5 / 5.0 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 18.4 | – | – | 1.9\* / 2.9\* | 2.4 / **2.9** | 2.5 / 5.0 | – |
| BN254 scalar | **4.9** / **6.8** | 17.1 / 12.2 | 66.2 / 82.6 | – | 12.8 / 21.8 | – | – | 6.9 / 10.1 | – |
| secp256k1 base | **5.9** / **6.9** | 17.3 / 12.6 | 66.3 / 88.8 | 3.8\* / 3.0\* | 14.1 / 21.3 | – | – | 7.0 / 10.4 | – |
| P-256 base | **5.9** / **6.9** | 18.1 / 12.9 | 66.2 / 89.5 | 10.6 / 18.5 | 10.8 / 18.4 | – | – | 7.0 / 10.5 | 327 / – |
| P-256 scalar | **5.8** / **6.9** | 17.8 / 13.6 | 66.3 / 92.3 | 17.5 / 21.0 | 14.5 / 21.0 | – | – | 7.0 / 10.8 | 328 / – |
| Tom-256 base | **6.0** / **6.9** | – | 66.3 / 91.2 | – | 11.5 / 20.5 | – | – | 7.0 / 10.7 | – |
| BLS12-381 base | **9.6** / **8.9** | 19.5 / 13.2 | 87.2 / 112 | – | 20.2 / 27.3 | – | – | 16.8 / 12.7 | – |
| P-521 base | **12.2** / **14.8** | – | 117 / 154 | – | – | – | – | 24.8 / 52.8 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| BabyBear | **1.9** / **2.9** | – | 4.2 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.2 / 11.7 | – | – | 1.9\* / 2.9\* | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.0** / **6.7** | 16.1 / 12.3 | 49.5 / 50.8 | – | 8.3 / 11.8 | – | – | 6.9 / 8.0 | – |
| secp256k1 base | **5.2** / **8.0** | 16.8 / 13.0 | 49.5 / 48.4 | 3.2\* / 3.5\* | 7.5 / 12.8 | – | – | 7.0 / **8.0** | – |
| P-256 base | **5.1** / **6.5** | 15.7 / 11.4 | 49.5 / 43.7 | 8.0 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 | 148 / – |
| P-256 scalar | **5.0** / **6.9** | 16.2 / 12.5 | 49.5 / 47.6 | 7.9 / 13.0 | 8.1 / 13.0 | – | – | 7.0 / 8.0 | 150 / – |
| Tom-256 base | **5.5** / 8.1 | – | 49.6 / 55.0 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **9.7** / **9.9** | 21.6 / 15.2 | 82.6 / 88.1 | – | 15.2 / 19.9 | – | – | 16.9 / 11.0 | – |
| P-521 base | **11.7** / **12.1** | – | 120 / 138 | – | – | – | – | 27.2 / 50.5 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.9 / 35.0 | – | – | 2.8 / 7.0\* | – | 7.0 / 17.0 | – |
| BabyBear | **3.4** / **10.9** | – | 16.9 / 34.9 | – | – | 4.6 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.6 / 34.8 | – | – | 3.2\* / 9.9 | **3.7** / 10.9 | 7.0 / 17.0 | – |
| BN254 scalar | **68.6** / **72.5** | 74.3 / 104 | 118 / 152 | – | 92.4 / 93.8 | – | – | 93.5 / 128 | – |
| secp256k1 base | **44.5** / **54.8** | 88.3 / 114 | 118 / 152 | 84.3 / 88.4 | 88.1 / 96.0 | – | – | 93.4 / 128 | – |
| P-256 base | **49.6** / **52.5** | 66.9 / 86.8 | 118 / 152 | 67.7 / 71.8 | 68.0 / 73.0 | – | – | 50.8 / 57.3 | 563 / – |
| P-256 scalar | **74.9** / **84.0** | 91.1 / 113 | 119 / 152 | 140 / 141 | 94.2 / 100.0 | – | – | 93.8 / 127 | 630 / – |
| Tom-256 base | **70.9** / **76.0** | – | 118 / 152 | – | 90.1 / 94.8 | – | – | 93.8 / 128 | – |
| BLS12-381 base | **138** / **141** | 163 / 194 | 270 / 286 | – | 190 / 194 | – | – | 255 / 269 | – |
| P-521 base | **176** / **191** | – | 530 / 547 | – | – | – | – | 462 / 471 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 31.9 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.7 / 33.5 | – | – | 4.7 / 13.0 | – | 7.6 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.5 / 35.3 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.6 / 17.0 | – |
| BN254 scalar | **63.2** / **66.5** | 73.1 / 105 | 114 / 151 | – | 74.2 / 89.4 | – | – | 93.3 / 127 | – |
| secp256k1 base | **33.9** / **42.9** | 83.6 / 101 | 115 / 151 | 57.9 / 80.0 | 70.4 / 84.9 | – | – | 93.5 / 124 | – |
| P-256 base | **39.7** / **44.0** | 63.3 / 81.9 | 117 / 150 | 60.8 / 72.0 | 54.8 / 58.0 | – | – | 50.7 / 59.9 | 562 / – |
| P-256 scalar | **63.9** / **73.0** | 88.1 / 109 | 113 / 151 | 129 / 147 | 77.4 / 93.5 | – | – | 93.6 / 128 | 631 / – |
| Tom-256 base | **57.0** / **64.5** | – | 113 / 151 | – | 69.5 / 82.9 | – | – | 93.4 / 127 | – |
| BLS12-381 base | **131** / **136** | 140 / 167 | 269 / 288 | – | 151 / 166 | – | – | 254 / 265 | – |
| P-521 base | **118** / **133** | – | 537 / 558 | – | – | – | – | 462 / 472 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **221** / – | – | 1,433 / – | – | – | 633 / – | – | 878 / – | – |
| BabyBear | **210** / – | – | 1,414 / – | – | – | 618 / – | – | 1,051 / – | – |
| Goldilocks | **434** / – | – | 2,848 / – | – | – | 1,307 / – | 691 / – | 1,699 / – | – |
| BN254 scalar | **5,041** / – | 16,285 / – | 14,267 / – | – | 7,145 / – | – | – | 14,072 / – | – |
| secp256k1 base | **4,922** / – | 15,410 / – | 14,311 / – | 26,450 / – | 7,176 / – | – | – | 14,157 / – | – |
| P-256 base | **4,833** / – | 15,173 / – | 14,265 / – | 30,358 / – | 7,196 / – | – | – | 14,791 / – | 88,709 / – |
| P-256 scalar | **5,033** / – | 16,372 / – | 14,278 / – | 29,475 / – | 7,199 / – | – | – | 14,137 / – | 88,975 / – |
| Tom-256 base | **5,024** / – | – | 14,341 / – | – | 7,194 / – | – | – | 14,162 / – | – |
| BLS12-381 base | **8,807** / – | 36,241 / – | 25,195 / – | – | 12,058 / – | – | – | 34,843 / – | – |
| P-521 base | **14,723** / – | – | 43,659 / – | – | – | – | – | 71,362 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **135** / – | – | 1,652 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,971** / – | – | 18,677 / – | – | – | 2,712 / – | – | – |
| Goldilocks | **2,829** / – | – | 26,887 / – | – | – | 3,148 / – | – | – |
| BN254 scalar | **33,510** / – | 43,643 / – | 132,639 / – | – | 84,156 / – | – | – | – |
| secp256k1 base | **12,289** / – | 42,734 / – | 52,349 / – | 24,333 / – | 47,453 / – | – | 69,074 / – | – |
| P-256 base | **11,790** / – | 18,600 / – | 43,148 / – | 27,672 / – | 33,630 / – | – | 30,242 / – | 163,933 / – |
| P-256 scalar | **24,058** / – | 39,216 / – | 51,572 / – | 81,760 / – | 41,601 / – | – | – | – |
| Tom-256 base | **21,265** / – | – | 48,661 / – | – | 44,126 / – | – | 51,790 / – | – |
| BLS12-381 base | **65,953** / – | 90,368 / – | 133,875 / – | – | 139,245 / – | – | 176,589 / – | – |
| P-521 base | **68,259** / – | – | 264,450 / – | – | – | – | 256,510 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | – | 34.5 / 36.5 | – | – | 5.2\* / 6.9\* | – | – |
| BabyBear | **8.9** / **10.9** | – | 34.3 / 36.4 | – | – | 11.9 / 13.7 | – | – |
| Goldilocks | **6.9** / **9.1** | – | 37.9 / 39.0 | – | – | 3.2\* / 9.9 | 7.3 / 10.9 | – |
| BN254 scalar | **20.7** / **25.8** | 237 / 235 | 266 / 268 | – | 236 / 233 | – | – | 252 / 251 |
| secp256k1 base | **9.7** / **14.9** | 235 / 233 | 266 / 271 | 81.7 / 89.6 | 208 / 210 | – | – | 254 / 251 |
| P-256 base | **18.8** / **24.9** | 207 / 207 | 266 / 269 | 147 / 136 | 175 / 180 | – | – | 131 / 133 |
| P-256 scalar | **22.2** / **27.9** | 260 / 259 | 273 / 268 | 139 / 146 | 243 / 236 | – | – | 252 / 249 |
| Tom-256 base | **20.4** / **27.9** | – | 259 / 269 | – | 220 / 227 | – | – | 252 / 250 |
| BLS12-381 base | **28.6** / **34.5** | 425 / 425 | 545 / 542 | – | 425 / 430 | – | – | 585 / 596 |
| P-521 base | **19.7** / **32.9** | – | 900 / 914 | – | – | – | – | 959 / 960 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.6 / 36.5 | – | – | 5.2\* / 6.9\* | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | 11.9 / 14.0 | – |
| Goldilocks | **6.9** / **9.1** | 38.0 / 37.6 | – | – | 3.2\* / 9.9 | 7.2 / 10.9 |
| BN254 scalar | **103** / **114** | 272 / 273 | – | 186 / 185 | – | – |
| secp256k1 base | **57.0** / **68.0** | 279 / 267 | – | 158 / 155 | – | – |
| P-256 base | **102** / **110** | 278 / 276 | – | 128 / 134 | – | – |
| P-256 scalar | **110** / **116** | 267 / 271 | 186 / 186 | 191 / 188 | – | – |
| Tom-256 base | **108** / **114** | 279 / 272 | – | 170 / 173 | – | – |
| BLS12-381 base | **214** / **222** | 540 / 546 | – | 371 / 378 | – | – |
| P-521 base | **359** / **394** | 911 / 920 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.5** | – | 31.9 / 26.5 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **5.9** / **7.0** | – | 32.1 / 30.5 | – | – | 8.9 / 7.9 | – | – |
| Goldilocks | **4.8** / **4.4** | – | 38.7 / 34.0 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 | – |
| BN254 scalar | **24.9** / **24.8** | 123 / 125 | 231 / 226 | – | 141 / 150 | – | – | 116 / 113 |
| secp256k1 base | **5.8** / **7.4** | 106 / 105 | 231 / 227 | 2.9\* / 2.0\* | 130 / 137 | – | – | 117 / 113 |
| P-256 base | **25.8** / **22.1** | 103 / 103 | 231 / 218 | 76.0 / 85.0 | 119 / 128 | – | – | 59.0 / 64.9 |
| P-256 scalar | 25.4 / 22.7 | 129 / 130 | 232 / 227 | **14.2** / **20.6** | 152 / 162 | – | – | 117 / 113 |
| Tom-256 base | **25.9** / **26.0** | – | 231 / 226 | – | 141 / 149 | – | – | 117 / 113 |
| BLS12-381 base | **38.2** / **39.4** | 225 / 224 | 374 / 375 | – | 259 / 259 | – | – | 324 / 311 |
| P-521 base | **10.8** / **12.7** | – | 595 / 596 | – | – | – | – | 528 / 539 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 31.8 / 26.6 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **5.9** / **7.0** | 32.1 / 29.9 | – | – | 9.0 / 7.9 | – |
| Goldilocks | **4.7** / **4.4** | 39.9 / 35.1 | – | – | 1.9\* / 2.9\* | 6.0 / 5.9 |
| BN254 scalar | **71.9** / **73.2** | 236 / 230 | – | 101 / 108 | – | – |
| secp256k1 base | **6.7** / **8.5** | 233 / 230 | – | 83.7 / 94.8 | – | – |
| P-256 base | **57.2** / **58.0** | 229 / 234 | – | 75.5 / 81.5 | – | – |
| P-256 scalar | 81.0 / 78.1 | 238 / 231 | **48.2** / **44.1** | 108 / 116 | – | – |
| Tom-256 base | **73.8** / **75.3** | 233 / 232 | – | 92.7 / 104 | – | – |
| BLS12-381 base | **151** / **152** | 375 / 381 | – | 206 / 213 | – | – |
| P-521 base | **23.1** / **23.3** | 596 / 604 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | – | 16.4 / 18.0 | – | – | 4.2\* / 4.1\* | – | – |
| BabyBear | **5.8** / **7.0** | – | 17.6 / 18.4 | – | – | 8.9 / 8.7 | – | – |
| Goldilocks | **4.3** / **4.2** | – | 26.9 / 22.3 | – | – | 1.9\* / 2.9\* | 4.8 / 5.2 | – |
| BN254 scalar | **24.5** / **24.8** | 122 / 120 | 179 / 179 | – | 144 / 144 | – | – | 116 / 110 |
| secp256k1 base | **4.0** / **6.9** | 101 / 103 | 190 / 177 | 3.4\* / 3.0\* | 123 / 125 | – | – | 116 / 110 |
| P-256 base | **20.2** / **21.3** | 101 / 96.1 | 178 / 179 | 81.3 / 78.8 | 120 / 119 | – | – | 61.3 / 61.5 |
| P-256 scalar | 24.8 / 23.6 | 129 / 126 | 186 / 182 | **6.5** / **12.3** | 149 / 150 | – | – | 116 / 110 |
| Tom-256 base | **24.7** / **25.0** | – | 182 / 182 | – | 140 / 147 | – | – | 116 / 110 |
| BLS12-381 base | **37.8** / **38.6** | 224 / 227 | 360 / 352 | – | 253 / 254 | – | – | 314 / 310 |
| P-521 base | **9.5** / **11.0** | – | 576 / 598 | – | – | – | – | 540 / 540 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 17.9 | – | – | 3.8\* / 4.1\* | – |
| BabyBear | **5.9** / **6.9** | 17.6 / 18.4 | – | – | 8.9 / 8.7 | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.3 | – | – | 1.9\* / 2.9\* | 4.9 / 5.3 |
| BN254 scalar | **73.7** / **73.2** | 182 / 183 | – | 110 / 103 | – | – |
| secp256k1 base | **5.4** / **6.8** | 184 / 178 | – | 80.7 / 82.8 | – | – |
| P-256 base | **53.7** / **56.8** | 181 / 173 | – | 78.6 / 78.0 | – | – |
| P-256 scalar | 80.0 / 79.2 | 187 / 177 | **43.4** / **36.5** | 106 / 102 | – | – |
| Tom-256 base | **75.4** / **72.8** | 174 / 187 | – | 103 / 96.0 | – | – |
| BLS12-381 base | **155** / **154** | 363 / 366 | – | 208 / 203 | – | – |
| P-521 base | **20.2** / **15.6** | 588 / 601 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / – | – | 12.3 / – | – | – | **3.2** / – | – | – |
| BabyBear | **6.3** / – | – | 13.1 / – | – | – | 8.0 / – | – | – |
| Goldilocks | **3.3** / – | – | 17.5 / – | – | – | 1.0\* / – | 3.4 / – | – |
| BN254 scalar | **22.0** / – | 129 / – | 119 / – | – | 126 / – | – | – | 94.7 / – |
| secp256k1 base | **2.0** / – | 120 / – | 118 / – | 2.3 / – | 110 / – | – | – | 94.7 / – |
| P-256 base | **18.0** / – | 112 / – | 116 / – | 59.0 / – | 102 / – | – | – | 54.8 / – |
| P-256 scalar | 18.0 / – | 139 / – | 118 / – | **2.4** / – | 133 / – | – | – | 94.5 / – |
| Tom-256 base | **21.0** / – | – | 117 / – | – | 128 / – | – | – | 94.9 / – |
| BLS12-381 base | **29.1** / – | 226 / – | 261 / – | – | 227 / – | – | – | 325 / – |
| P-521 base | **5.0** / – | – | 436 / – | – | – | – | – | 487 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **609** / **604** | 4,149 / 4,138 | 16,497 / 16,425 | 13,373 / 13,294 | – | 6,190 / 6,209 | 10,593 / 10,560 |
| secp256k1 base | **565** / **566** | 4,056 / 4,050 | 13,210 / 13,218 | 13,323 / 13,202 | 24,850 / 24,764 | 6,139 / 6,130 | 10,595 / 10,583 |
| P-256 base | **602** / **602** | 4,190 / 4,116 | 14,881 / 14,828 | 13,311 / 13,236 | 29,992 / 28,737 | 6,146 / 6,103 | 10,467 / 10,419 |
| P-256 scalar | **618** / **618** | 4,093 / 4,082 | 15,945 / 15,907 | 13,376 / 13,301 | 21,911 / 21,756 | 6,184 / 6,181 | 10,591 / 10,514 |
| Tom-256 base | **609** / **604** | 4,175 / 4,166 | – | 13,309 / 13,255 | – | 6,170 / 6,148 | 10,582 / 10,550 |
| BLS12-381 base | **751** / **751** | 7,108 / 7,083 | 34,584 / 34,837 | 23,010 / 23,093 | – | 9,851 / 9,838 | 25,947 / 25,724 |
| P-521 base | **922** / **923** | 11,987 / 11,975 | – | 40,069 / 40,829 | – | – | 54,819 / 54,812 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 224 / 218 | **222** / **216** | – | 1,443 / 1,437 | – | – | 638 / 637 | – | 887 / 896 |
| BabyBear | **214** / **213** | 239 / 237 | – | 1,432 / 1,427 | – | – | 624 / 624 | – | 1,063 / 1,064 |
| Goldilocks | **435** / **434** | 445 / 444 | – | 2,912 / 2,921 | – | – | 1,288 / 1,292 | 704 / 706 | 1,734 / 1,740 |
| BN254 scalar | **4,983** / **4,994** | 5,025 / 5,024 | 16,691 / 16,624 | 14,594 / 14,584 | – | 7,377 / 7,382 | – | – | 14,287 / 14,281 |
| secp256k1 base | 4,886 / **4,880** | **4,876** / 4,884 | 15,638 / 15,620 | 14,612 / 14,619 | 27,011 / 25,406 | 7,391 / 7,369 | – | – | 14,437 / 14,357 |
| P-256 base | **5,031** / **5,016** | 5,066 / 5,040 | 15,437 / 15,419 | 14,574 / 14,583 | 28,368 / 29,191 | 7,358 / 7,343 | – | – | 14,227 / 14,638 |
| P-256 scalar | **4,964** / **4,951** | 4,994 / 4,967 | 16,719 / 16,638 | 14,613 / 14,548 | 29,733 / 29,760 | 7,380 / 7,397 | – | – | 14,368 / 14,349 |
| Tom-256 base | **5,056** / **5,051** | 5,080 / 5,067 | – | 14,626 / 14,630 | – | 7,388 / 7,378 | – | – | 14,448 / 14,351 |
| BLS12-381 base | **9,044** / **9,025** | 9,145 / 9,131 | 36,736 / 36,868 | 25,905 / 25,816 | – | 12,379 / 12,326 | – | – | 35,404 / 35,390 |
| P-521 base | **14,901** / **14,953** | 14,954 / 14,957 | – | 44,615 / 44,622 | – | – | – | – | 71,125 / 72,072 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 9.6 / 17.5 | – | – | – | 2.5 / 4.9 |
| BabyBear | **1.9** / **3.0** | 9.6 / 17.5 | – | – | – | 2.5 / 5.0 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 18.4 | – | – | 2.4 / **2.9** | 2.5 / 5.0 |
| BN254 scalar | **4.9** / **6.8** | 66.2 / 82.6 | – | 12.8 / 21.8 | – | 6.9 / 10.1 |
| secp256k1 base | **6.0** / **6.8** | 66.3 / 88.8 | 3.8\* / 3.0\* | 14.1 / 21.3 | – | 7.0 / 10.4 |
| P-256 base | **5.8** / **6.9** | 66.2 / 89.5 | 10.6 / 18.5 | 10.8 / 18.4 | – | 7.0 / 10.5 |
| P-256 scalar | **5.8** / **6.7** | 66.3 / 92.3 | 17.5 / 21.0 | 14.5 / 21.0 | – | 7.0 / 10.8 |
| Tom-256 base | **5.8** / **6.9** | 66.3 / 91.2 | – | 11.5 / 20.5 | – | 7.0 / 10.7 |
| BLS12-381 base | **9.6** / **8.9** | 87.2 / 112 | – | 20.2 / 27.3 | – | 16.8 / 12.7 |
| P-521 base | **12.2** / **14.7** | 117 / 154 | – | – | – | 24.8 / 52.8 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 |
| BabyBear | **1.9** / **2.9** | 4.2 / 11.9 | – | – | – | – | 2.0 / 4.0 |
| Goldilocks | **1.9** / **2.9** | 4.2 / 11.7 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **4.9** / **6.8** | 49.5 / 50.8 | – | 8.3 / 11.8 | – | – | 6.9 / 8.0 |
| secp256k1 base | **5.3** / **8.0** | 49.5 / 48.4 | 3.2\* / 3.5\* | 7.5 / 12.8 | – | – | 7.0 / **8.0** |
| P-256 base | **5.3** / **6.8** | 49.5 / 43.7 | 8.0 / 13.0 | 7.7 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.0** / **6.8** | 49.5 / 47.6 | 7.9 / 13.0 | 8.1 / 13.0 | – | – | 7.0 / 8.0 |
| Tom-256 base | **5.5** / 8.1 | 49.6 / 55.0 | – | 8.3 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **9.7** / **9.7** | 82.6 / 88.1 | – | 15.2 / 19.9 | – | – | 16.9 / 11.0 |
| P-521 base | **11.7** / **12.2** | 120 / 138 | – | – | – | – | 27.2 / 50.5 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 35.0 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.9 / 34.9 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **8.9** | 17.6 / 34.8 | – | – | **3.7** / 10.9 | 7.0 / 17.0 |
| BN254 scalar | **68.0** / **79.0** | 118 / 152 | – | 92.4 / 93.8 | – | 93.5 / 128 |
| secp256k1 base | **44.7** / **55.0** | 118 / 152 | 84.3 / 88.4 | 88.1 / 96.0 | – | 93.4 / 128 |
| P-256 base | **49.5** / **52.5** | 118 / 152 | 67.7 / 71.8 | 68.0 / 73.0 | – | 50.8 / 57.3 |
| P-256 scalar | **74.9** / **84.0** | 119 / 152 | 140 / 141 | 94.2 / 100.0 | – | 93.8 / 127 |
| Tom-256 base | **71.0** / **76.0** | 118 / 152 | – | 90.1 / 94.8 | – | 93.8 / 128 |
| BLS12-381 base | **141** / **154** | 270 / 286 | – | 190 / 194 | – | 255 / 269 |
| P-521 base | **176** / **191** | 530 / 547 | – | – | – | 462 / 471 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 31.9 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.4** / **11.0** | 16.7 / 33.5 | – | – | – | 7.6 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 35.3 | – | – | **3.7** / 11.0 | 7.6 / 17.0 |
| BN254 scalar | **64.9** / **66.9** | 114 / 151 | – | 74.2 / 89.4 | – | 93.3 / 127 |
| secp256k1 base | **33.8** / **43.0** | 115 / 151 | 57.9 / 80.0 | 70.4 / 84.9 | – | 93.5 / 124 |
| P-256 base | **40.3** / **44.0** | 117 / 150 | 60.8 / 72.0 | 54.8 / 58.0 | – | 50.7 / 59.9 |
| P-256 scalar | **63.9** / **73.0** | 113 / 151 | 129 / 147 | 77.4 / 93.5 | – | 93.6 / 128 |
| Tom-256 base | **57.0** / **64.5** | 113 / 151 | – | 69.5 / 82.9 | – | 93.4 / 127 |
| BLS12-381 base | **134** / **145** | 269 / 288 | – | 151 / 166 | – | 254 / 265 |
| P-521 base | **118** / **133** | 537 / 558 | – | – | – | 462 / 472 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **248** / – | 3,851 / – | – | – | – | 953 / – |
| BabyBear | **383** / – | 3,853 / – | – | – | – | 944 / – |
| Goldilocks | **646** / – | 3,853 / – | – | – | 691 / – | 2,067 / – |
| BN254 scalar | **6,761** / – | 19,334 / – | – | 46,612 / – | – | 54,944 / – |
| secp256k1 base | **6,858** / – | 19,331 / – | 26,450 / – | 46,592 / – | – | 69,247 / – |
| P-256 base | **6,516** / – | 19,346 / – | 30,358 / – | 33,419 / – | – | 34,287 / – |
| P-256 scalar | **6,911** / – | 19,341 / – | 78,255 / – | 48,911 / – | – | 60,105 / – |
| Tom-256 base | **6,985** / – | 19,333 / – | – | 44,800 / – | – | 51,808 / – |
| BLS12-381 base | **12,456** / – | 34,165 / – | – | 141,458 / – | – | 179,131 / – |
| P-521 base | **19,938** / – | 64,310 / – | – | – | – | 495,236 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,652 / – | – | – | 446 / – |
| BabyBear | **5,592** / – | 18,677 / – | – | – | – |
| Goldilocks | **7,152** / – | 26,887 / – | – | – | – |
| BN254 scalar | **58,556** / – | 132,639 / – | – | 84,156 / – | – |
| secp256k1 base | **12,274** / – | 52,349 / – | 24,333 / – | 47,453 / – | 69,074 / – |
| P-256 base | **12,039** / – | 43,148 / – | 27,672 / – | 33,630 / – | 30,242 / – |
| P-256 scalar | **25,620** / – | 51,572 / – | 81,760 / – | 41,601 / – | – |
| Tom-256 base | **20,051** / – | 48,661 / – | – | 44,126 / – | 51,790 / – |
| BLS12-381 base | **69,785** / – | 133,875 / – | – | 139,245 / – | 176,589 / – |
| P-521 base | **68,436** / – | 264,450 / – | – | – | 256,510 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **8.9** | 34.5 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | – |
| Goldilocks | **7.0** / **9.1** | 37.9 / 39.0 | – | – | 7.3 / 10.9 |
| BN254 scalar | **21.9** / **30.4** | 266 / 268 | – | 236 / 233 | – |
| secp256k1 base | **11.0** / **19.9** | 266 / 271 | 81.7 / 89.6 | 208 / 210 | – |
| P-256 base | **24.4** / **31.9** | 266 / 269 | 147 / 136 | 175 / 180 | – |
| P-256 scalar | **22.8** / **33.9** | 273 / 268 | 139 / 146 | 243 / 236 | – |
| Tom-256 base | **22.5** / **32.9** | 259 / 269 | – | 220 / 227 | – |
| BLS12-381 base | **34.6** / **42.1** | 545 / 542 | – | 425 / 430 | – |
| P-521 base | **26.3** / **45.3** | 900 / 914 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.9** / **9.0** | 34.6 / 36.5 | – | – | – |
| BabyBear | **8.9** / **10.9** | 34.3 / 36.4 | – | – | – |
| Goldilocks | **6.9** / **8.8** | 38.0 / 37.6 | – | – | 7.2 / 10.9 |
| BN254 scalar | **120** / **141** | 272 / 273 | – | 186 / 185 | – |
| secp256k1 base | **78.0** / **83.8** | 279 / 267 | – | 158 / 155 | – |
| P-256 base | **118** / **134** | 278 / 276 | – | 128 / **134** | – |
| P-256 scalar | **131** / **146** | 267 / 271 | 186 / 186 | 191 / 188 | – |
| Tom-256 base | **122** / **135** | 279 / 272 | – | 170 / 173 | – |
| BLS12-381 base | **256** / **266** | 540 / 546 | – | 371 / 378 | – |
| P-521 base | **426** / **472** | 911 / 920 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.5** / **4.5** | 31.9 / 26.5 | – | – | – |
| BabyBear | **5.4** / **4.9** | 32.1 / 30.5 | – | – | – |
| Goldilocks | **4.7** / **4.8** | 38.7 / 34.0 | – | – | 6.0 / 5.9 |
| BN254 scalar | **22.8** / **21.9** | 231 / 226 | – | 141 / 150 | – |
| secp256k1 base | **5.8** / **7.4** | 231 / 227 | 2.9\* / 2.0\* | 130 / 137 | – |
| P-256 base | **17.0** / **17.1** | 231 / 218 | 76.0 / 85.0 | 119 / 128 | – |
| P-256 scalar | 23.9 / 24.7 | 232 / 227 | **14.2** / **20.6** | 152 / 162 | – |
| Tom-256 base | **22.3** / **22.3** | 231 / 226 | – | 141 / 149 | – |
| BLS12-381 base | **36.3** / **35.8** | 374 / 375 | – | 259 / 259 | – |
| P-521 base | **10.8** / **12.7** | 595 / 596 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.6** / **4.6** | 31.8 / 26.6 | – | – | – |
| BabyBear | **5.4** / **4.9** | 32.1 / 29.9 | – | – | – |
| Goldilocks | **4.7** / **4.0** | 39.9 / 35.1 | – | – | 6.0 / 5.9 |
| BN254 scalar | **75.9** / **74.8** | 236 / 230 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **8.7** | 233 / 230 | – | 83.7 / 94.8 | – |
| P-256 base | **57.2** / **57.8** | 229 / 234 | – | 75.5 / 81.5 | – |
| P-256 scalar | 81.0 / 78.1 | 238 / 231 | **48.2** / **44.1** | 108 / 116 | – |
| Tom-256 base | **73.8** / **75.2** | 233 / 232 | – | 92.7 / 104 | – |
| BLS12-381 base | **157** / **152** | 375 / 381 | – | 206 / 213 | – |
| P-521 base | **22.5** / **22.8** | 596 / 604 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.4 / 18.0 | – | – | – |
| BabyBear | **5.2** / **4.8** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.9 / 22.3 | – | – | 4.8 / 5.2 |
| BN254 scalar | **25.4** / **22.0** | 179 / 179 | – | 144 / 144 | – |
| secp256k1 base | **4.0** / **6.9** | 190 / 177 | 3.4\* / 3.0\* | 123 / 125 | – |
| P-256 base | **16.2** / **16.1** | 178 / 179 | 81.3 / 78.8 | 120 / 119 | – |
| P-256 scalar | 24.0 / 22.8 | 186 / 182 | **6.5** / **12.3** | 149 / 150 | – |
| Tom-256 base | **23.0** / **23.9** | 182 / 182 | – | 140 / 147 | – |
| BLS12-381 base | **36.5** / **34.8** | 360 / 352 | – | 253 / 254 | – |
| P-521 base | **9.6** / **11.0** | 576 / 598 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **4.7** / **4.5** | 16.5 / 17.9 | – | – | – |
| BabyBear | **5.2** / **4.8** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.3** / **4.2** | 26.8 / 22.3 | – | – | 4.9 / 5.3 |
| BN254 scalar | **77.7** / **75.6** | 182 / 183 | – | 110 / 103 | – |
| secp256k1 base | **7.2** / **7.7** | 184 / 178 | – | 80.7 / 82.8 | – |
| P-256 base | **53.7** / **56.8** | 181 / 173 | – | 78.6 / 78.0 | – |
| P-256 scalar | 80.0 / 79.1 | 187 / 177 | **43.4** / **36.5** | 106 / 102 | – |
| Tom-256 base | **75.8** / **72.8** | 174 / 187 | – | 103 / 96.0 | – |
| BLS12-381 base | **155** / **156** | 363 / 366 | – | 208 / 203 | – |
| P-521 base | **19.2** / **19.6** | 588 / 601 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.8** / – | 12.3 / – | – | – | – |
| BabyBear | **4.9** / – | 13.1 / – | – | – | – |
| Goldilocks | **3.4** / – | 17.5 / – | – | – | **3.4** / – |
| BN254 scalar | **26.3** / – | 119 / – | – | 126 / – | – |
| secp256k1 base | **2.1** / – | 118 / – | 2.3 / – | 110 / – | – |
| P-256 base | **18.5** / – | 116 / – | 59.0 / – | 102 / – | – |
| P-256 scalar | 27.2 / – | 118 / – | **2.4** / – | 133 / – | – |
| Tom-256 base | **24.2** / – | 117 / – | – | 128 / – | – |
| BLS12-381 base | **41.9** / – | 261 / – | – | 227 / – | – |
| P-521 base | **5.0** / – | 436 / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **6,846** / **6,843** | 6,884 / 6,878 | 19,639 / 19,577 | – | 46,052 / 46,051 | 55,077 / 55,438 |
| secp256k1 base | **6,779** / **6,775** | 6,832 / 6,826 | 19,680 / 19,650 | 24,850 / 24,764 | 46,776 / 46,779 | 69,486 / 69,305 |
| P-256 base | **6,666** / **6,667** | 6,709 / 6,712 | 19,669 / 19,588 | 29,992 / 28,737 | 33,196 / 33,208 | 34,746 / 34,440 |
| P-256 scalar | **6,885** / **6,890** | 6,911 / 6,901 | 19,648 / 19,595 | 78,933 / 78,927 | 49,134 / 48,697 | 60,224 / 60,534 |
| Tom-256 base | **7,274** / **7,277** | 7,323 / 7,319 | 19,669 / 19,533 | – | 44,304 / 43,329 | 51,709 / 52,326 |
| BLS12-381 base | **12,390** / **12,385** | 12,425 / 12,430 | 34,826 / 34,876 | – | 140,307 / 139,978 | 180,825 / 176,618 |
| P-521 base | **20,386** / **20,383** | 20,423 / 20,414 | 66,386 / 64,660 | – | – | 495,073 / 494,008 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **253** / **251** | 272 / 266 | 3,895 / 3,895 | – | – | – | 993 / 993 |
| BabyBear | **403** / **403** | 413 / 412 | 3,896 / 3,891 | – | – | – | 993 / 993 |
| Goldilocks | 668 / **665** | **666** / 668 | 3,897 / 3,895 | – | – | 704 / 706 | 2,115 / 2,114 |
| BN254 scalar | **6,895** / **6,894** | 6,939 / 6,935 | 19,628 / 19,603 | – | 46,000 / 46,011 | – | 55,024 / 55,775 |
| secp256k1 base | **6,778** / **6,774** | 6,831 / 6,823 | 19,656 / 19,576 | 27,011 / 25,406 | 46,710 / 46,717 | – | 69,349 / 70,126 |
| P-256 base | **6,702** / **6,706** | 6,753 / 6,755 | 19,641 / 19,503 | 28,368 / 29,191 | 33,156 / 33,132 | – | 34,678 / 34,447 |
| P-256 scalar | **6,952** / **6,947** | 6,985 / 6,989 | 19,607 / 19,586 | 78,962 / 78,466 | 49,074 / 49,077 | – | 60,243 / 60,953 |
| Tom-256 base | **7,330** / **7,333** | 7,383 / 7,370 | 19,638 / 19,571 | – | 44,270 / 43,466 | – | 51,797 / 52,563 |
| BLS12-381 base | **12,499** / 12,504 | 12,546 / **12,484** | 34,756 / 34,950 | – | 140,266 / 140,254 | – | 181,646 / 177,574 |
| P-521 base | **20,379** / **20,377** | 20,420 / 20,423 | 64,592 / 64,727 | – | – | – | 497,935 / 494,731 |

## WebAssembly

`prime-field` at commit `40558f9` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, built for `wasm32-unknown-unknown` and run in headless browsers, in cycles per operation, measured on 2026-11-07.

- The bench module runs in a dedicated worker of the browser, on the measuring CPU. A driver counts that thread with perf_event, and holds the CPU while the thread sleeps.
- A second worker carries the start and the end of each count to the driver while the measuring worker sleeps.
- The browser's flags compile every function with its optimizing tier when the module loads.
- A counted loop runs behind a function pointer, which keeps V8 from inlining it into the code around the count.
- A batch loop reads its step and its end by volatile loads, so LLVM unrolls it for no operation, whatever the operation's size. A chain through locals over operand sets of more than 32 bytes reads its step the same way.
- A run passes over its operand sets, of up to 256 KiB, as many times as it takes to count 4 million instructions. They start at a 4 KiB boundary in every case.
- A figure of cycles is the lower rate of the fewest run of n operations and the fewest of 2n, each less the fewest empty count: under load, a run of each length seldom meets an idle sibling hardware thread.
- A case whose 2n runs counted no more than its n runs, or whose chain step counted under 0.75 cycles, ran 31 more repetitions, up to 3 times. A form whose difference counted under 50 times the noise of the fixed cost of a count ran 31 repetitions of twice its operations, up to 3 times. A case that stays so stops the sweep. Each such round ran every case again, in the order of the first round, and kept the counts of those forms alone.
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
| Cost of a count, in WebAssembly, Chrome | 4,672 to 16,624 cycles and 3,081 instructions; its noise in a figure, 201 to 3,004 cycles |
| Browser, in WebAssembly with wide arithmetic, Chrome | Google Chrome for Testing 156.0.8078.4; `--js-flags=--no-liftoff --wasm-wide-arithmetic` |
| Target features, in WebAssembly with wide arithmetic, Chrome | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Chrome | 4,353 to 13,383 cycles and 3,081 instructions; its noise in a figure, 43.0 to 2,461 cycles |
| Browser, in WebAssembly, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=false` |
| Target features, in WebAssembly, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext` |
| Cost of a count, in WebAssembly, Firefox | 6,937 to 15,919 cycles and 1,885 instructions; its noise in a figure, 347 to 3,014 cycles |
| Browser, in WebAssembly with wide arithmetic, Firefox | Mozilla Firefox 158.0.1; `javascript.options.wasm_baselinejit=false`, `javascript.options.wasm_wide_arithmetic=true` |
| Target features, in WebAssembly with wide arithmetic, Firefox | `bulk-memory`, `bulk-memory-opt`, `call-indirect-overlong`, `multivalue`, `mutable-globals`, `nontrapping-fptoint`, `reference-types`, `sign-ext`, `wide-arithmetic` |
| Cost of a count, in WebAssembly with wide arithmetic, Firefox | 4,768 to 12,520 cycles and 1,885 instructions; its noise in a figure, 95.0 to 2,775 cycles |
| Run 1, in WebAssembly, Chrome | `5d2b0e1`, 2026-11-07: 891 cells; CPU 12; 1-minute load 24.48 before, 32.08 after; its SMT sibling, CPU 28, 33% busy |
| Run 1, in WebAssembly with wide arithmetic, Chrome | `5d2b0e1`, 2026-11-07: 891 cells; CPU 12; 1-minute load 32.08 before, 27.01 after; its SMT sibling, CPU 28, 29% busy |
| Run 1, in WebAssembly, Firefox | `5d2b0e1`, 2026-11-07: 891 cells; CPU 12; 1-minute load 27.01 before, 30.75 after; its SMT sibling, CPU 28, 41% busy |
| Run 1, in WebAssembly with wide arithmetic, Firefox | `5d2b0e1`, 2026-11-07: 891 cells; CPU 12; 1-minute load 30.75 before, 27.08 after; its SMT sibling, CPU 28, 29% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |


### Against the best competitor, throughput

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 2.8 vs runtime-algebra 11.3 | −8.5 (0.25×) | 2.8 vs runtime-algebra 11.3 | −8.5 (0.25×) | 2.7 vs crypto-bigint 11.9 | −9.2 (0.23×) | 2.7 vs crypto-bigint 11.6 | −8.9 (0.23×) |
| add | BabyBear | 2.6 vs plonky3 2.6 | +0.0 (1.00×) | 2.6 vs plonky3 2.6 | +0.0 (1.00×) | 2.7 vs plonky3 2.6 | +0.1 (1.04×) | 2.7 vs plonky3 2.6 | +0.1 (1.04×) |
| add | Goldilocks | 3.5 vs winter-math 3.3 | +0.2 (1.06×) | 3.5 vs winter-math 3.3 | +0.2 (1.06×) | 3.1 vs winter-math 3.5 | −0.4 (0.89×) | 3.1 vs winter-math 3.5 | −0.4 (0.89×) |
| add | BN254 scalar | 14.6 vs ark-ff 15.2 | −0.6 (0.96×) | 12.4 vs ark-ff 12.7 | −0.3 (0.98×) | 13.7 vs ark-ff 13.7 | +0.0 (1.00×) | 15.6 vs ark-ff 16.9 | −1.3 (0.92×) |
| add | secp256k1 base | 15.0 vs ark-ff 16.5 | −1.5 (0.91×) | 11.9 vs ark-ff 12.2 | −0.3 (0.98×) | 14.4 vs ark-ff 16.2 | −1.8 (0.89×) | 18.2 vs ark-ff 18.5 | −0.3 (0.98×) |
| add | P-256 base | 13.9 vs ark-ff 16.8 | −2.9 (0.83×) | 12.3 vs ark-ff 12.8 | −0.5 (0.96×) | 14.1 vs ark-ff 16.7 | −2.6 (0.84×) | 18.0 vs ark-ff 20.0 | −2.0 (0.90×) |
| add | P-256 scalar | 15.2 vs ark-ff 16.5 | −1.3 (0.92×) | 13.2 vs ark-ff 13.3 | −0.1 (0.99×) | 15.0 vs ark-ff 16.6 | −1.6 (0.90×) | 18.6 vs ark-ff 18.5 | +0.1 (1.01×) |
| add | Tom-256 base | 14.1 vs halo2curves 39.5 | −25.4 (0.36×) | 12.4 vs halo2curves 20.9 | −8.5 (0.59×) | 14.0 vs halo2curves 38.7 | −24.7 (0.36×) | 17.7 vs halo2curves 33.1 | −15.4 (0.53×) |
| add | BLS12-381 base | 25.1 vs ark-ff 39.6 | −14.5 (0.63×) | 19.6 vs ark-ff 34.9 | −15.3 (0.56×) | 23.5 vs ark-ff 40.0 | −16.5 (0.59×) | 24.5 vs ark-ff 41.4 | −16.9 (0.59×) |
| add | P-521 base | 44.8 vs crypto-bigint 124 | −79.2 (0.36×) | 28.8 vs crypto-bigint 107 | −78.2 (0.27×) | 44.2 vs crypto-bigint 170 | −126 (0.26×) | 45.5 vs crypto-bigint 177 | −132 (0.26×) |
| sub | Mersenne-31 | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.6 vs crypto-bigint 4.6 | −2.0 (0.57×) | 2.5 vs crypto-bigint 4.6 | −2.1 (0.54×) |
| sub | BabyBear | 2.6 vs plonky3 2.8 | −0.2 (0.93×) | 2.6 vs plonky3 2.8 | −0.2 (0.93×) | 2.7 vs plonky3 2.7 | +0.0 (1.00×) | 2.6 vs plonky3 2.7 | −0.1 (0.96×) |
| sub | Goldilocks | 2.8 vs winter-math 2.8 | +0.0 (1.00×) | 2.8 vs winter-math 2.8 | +0.0 (1.00×) | 2.7 vs winter-math 2.7 | +0.0 (1.00×) | 2.7 vs winter-math 2.7 | +0.0 (1.00×) |
| sub | BN254 scalar | 15.9 vs ark-ff 17.9 | −2.0 (0.89×) | 13.0 vs halo2curves 16.1 | −3.1 (0.81×) | 13.4 vs ark-ff 17.5 | −4.1 (0.77×) | 17.3 vs ark-ff 22.7 | −5.4 (0.76×) |
| sub | secp256k1 base | 17.5 vs ark-ff 18.2 | −0.7 (0.96×) | 12.6 vs halo2curves 14.9 | −2.3 (0.85×) | 14.2 vs ark-ff 18.1 | −3.9 (0.78×) | 17.4 vs halo2curves 21.7 | −4.3 (0.80×) |
| sub | P-256 base | 15.7 vs ark-ff 17.0 | −1.3 (0.92×) | 12.0 vs halo2curves 13.6 | −1.6 (0.88×) | 13.2 vs ark-ff 16.9 | −3.7 (0.78×) | 17.0 vs halo2curves 20.4 | −3.4 (0.83×) |
| sub | P-256 scalar | 16.3 vs ark-ff 17.9 | −1.6 (0.91×) | 12.6 vs halo2curves 15.1 | −2.5 (0.83×) | 14.4 vs ark-ff 17.6 | −3.2 (0.82×) | 17.2 vs ark-ff 22.5 | −5.3 (0.76×) |
| sub | Tom-256 base | 15.9 vs halo2curves 24.4 | −8.5 (0.65×) | 13.0 vs halo2curves 16.4 | −3.4 (0.79×) | 13.7 vs halo2curves 23.9 | −10.2 (0.57×) | 17.0 vs halo2curves 23.1 | −6.1 (0.74×) |
| sub | BLS12-381 base | 24.0 vs ark-ff 34.7 | −10.7 (0.69×) | 20.4 vs halo2curves 26.4 | −6.0 (0.77×) | 21.0 vs ark-ff 30.8 | −9.8 (0.68×) | 27.0 vs halo2curves 36.1 | −9.1 (0.75×) |
| sub | P-521 base | 42.7 vs crypto-bigint 94.5 | −51.8 (0.45×) | 28.8 vs crypto-bigint 71.0 | −42.2 (0.41×) | 42.8 vs crypto-bigint 126 | −83.2 (0.34×) | 42.1 vs crypto-bigint 144 | −102 (0.29×) |
| mul | Mersenne-31 | 3.7 vs crypto-bigint 30.4 | −26.7 (0.12×) | 3.7 vs crypto-bigint 16.8 | −13.1 (0.22×) | 3.6 vs crypto-bigint 39.0 | −35.4 (0.09×) | 3.5 vs crypto-bigint 18.4 | −14.9 (0.19×) |
| mul | BabyBear | 5.2 vs plonky3 4.5 | +0.7 (1.16×) | 3.7 vs plonky3 4.5 | −0.8 (0.82×) | 5.4 vs plonky3 5.9 | −0.5 (0.92×) | 5.9 vs plonky3 5.9 | +0.0 (1.00×) |
| mul | Goldilocks | 11.9 vs winter-math 12.4 | −0.5 (0.96×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.8 vs winter-math 16.1 | −2.3 (0.86×) | 6.8 vs winter-math 6.9 | −0.1 (0.99×) |
| mul | BN254 scalar | 231 vs ark-ff 315 | −84.0 (0.73×) | 85.1 vs halo2curves 90.5 | −5.4 (0.94×) | 381 vs ark-ff 394 | −13.0 (0.97×) | 146 vs halo2curves 173 | −27.0 (0.84×) |
| mul | secp256k1 base | 160 vs ark-ff 275 | −115 (0.58×) | 58.8 vs halo2curves 86.5 | −27.7 (0.68×) | 226 vs ark-ff 408 | −182 (0.55×) | 105 vs halo2curves 154 | −49.0 (0.68×) |
| mul | P-256 base | 178 vs k256/p256 244 | −66.0 (0.73×) | 63.0 vs halo2curves 70.0 | −7.0 (0.90×) | 271 vs k256/p256 448 | −177 (0.60×) | 119 vs k256/p256 126 | −7.0 (0.94×) |
| mul | P-256 scalar | 241 vs ark-ff 313 | −72.0 (0.77×) | 87.4 vs halo2curves 91.7 | −4.3 (0.95×) | 315 vs ark-ff 417 | −102 (0.76×) | 156 vs halo2curves 171 | −15.0 (0.91×) |
| mul | Tom-256 base | 241 vs halo2curves 342 | −101 (0.70×) | 85.8 vs halo2curves 90.5 | −4.7 (0.95×) | 316 vs halo2curves 601 | −285 (0.53×) | 151 vs halo2curves 166 | −15.0 (0.91×) |
| mul | BLS12-381 base | 714 vs ark-ff 763 | −49.0 (0.94×) | 185 vs halo2curves 190 | −5.0 (0.97×) | 883 vs ark-ff 1,039 | −156 (0.85×) | 334 vs halo2curves 369 | −35.0 (0.91×) |
| mul | P-521 base | 669 vs crypto-bigint 2,118 | −1,449 (0.32×) | 244 vs crypto-bigint 598 | −354 (0.41×) | 1,323 vs crypto-bigint 2,733 | −1,410 (0.48×) | 433 vs crypto-bigint 1,053 | −620 (0.41×) |
| square | Mersenne-31 | 3.6 vs crypto-bigint 29.5 | −25.9 (0.12×) | 3.6 vs crypto-bigint 16.3 | −12.7 (0.22×) | 3.4 vs crypto-bigint 38.4 | −35.0 (0.09×) | 3.4 vs crypto-bigint 18.3 | −14.9 (0.19×) |
| square | BabyBear | 5.1 vs plonky3 5.1 | +0.0 (1.00×) | 3.4 vs plonky3 5.1 | −1.7 (0.67×) | 5.4 vs plonky3 5.1 | +0.3 (1.06×) | 4.6 vs plonky3 5.2 | −0.6 (0.88×) |
| square | Goldilocks | 9.5 vs winter-math 11.9 | −2.4 (0.80×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.0 vs winter-math 16.0 | −3.0 (0.81×) | 6.8 vs winter-math 6.8 | +0.0 (1.00×) |
| square | BN254 scalar | 221 vs ark-ff 263 | −42.0 (0.84×) | 76.8 vs halo2curves 84.1 | −7.3 (0.91×) | 340 vs ark-ff 355 | −15.0 (0.96×) | 132 vs halo2curves 143 | −11.0 (0.92×) |
| square | secp256k1 base | 142 vs ark-ff 225 | −83.0 (0.63×) | 47.5 vs halo2curves 78.2 | −30.7 (0.61×) | 189 vs ark-ff 297 | −108 (0.64×) | 85.8 vs halo2curves 126 | −40.2 (0.68×) |
| square | P-256 base | 154 vs halo2curves 186 | −32.0 (0.83×) | 55.3 vs halo2curves 58.2 | −2.9 (0.95×) | 233 vs ark-ff 328 | −95.0 (0.71×) | 102 vs halo2curves 110 | −8.0 (0.93×) |
| square | P-256 scalar | 220 vs ark-ff 259 | −39.0 (0.85×) | 78.0 vs halo2curves 82.1 | −4.1 (0.95×) | 291 vs ark-ff 350 | −59.0 (0.83×) | 134 vs halo2curves 143 | −9.0 (0.94×) |
| square | Tom-256 base | 210 vs halo2curves 280 | −70.0 (0.75×) | 74.2 vs halo2curves 79.4 | −5.2 (0.93×) | 277 vs halo2curves 478 | −201 (0.58×) | 129 vs halo2curves 137 | −8.0 (0.94×) |
| square | BLS12-381 base | 553 vs ark-ff 619 | −66.0 (0.89×) | 157 vs halo2curves 162 | −5.0 (0.97×) | 813 vs ark-ff 823 | −10.0 (0.99×) | 317 vs halo2curves 308 | +9.0 (1.03×) |
| square | P-521 base | 556 vs crypto-bigint 2,142 | −1,586 (0.26×) | 203 vs crypto-bigint 612 | −409 (0.33×) | 914 vs crypto-bigint 2,811 | −1,897 (0.33×) | 381 vs crypto-bigint 1,101 | −720 (0.35×) |
| invert | Mersenne-31 | 266 vs plonky3 664 | −398 (0.40×) | 266 vs plonky3 667 | −401 (0.40×) | 277 vs plonky3 718 | −441 (0.39×) | 271 vs plonky3 721 | −450 (0.38×) |
| invert | BabyBear | 269 vs plonky3 674 | −405 (0.40×) | 269 vs plonky3 687 | −418 (0.39×) | 274 vs plonky3 686 | −412 (0.40×) | 280 vs plonky3 703 | −423 (0.40×) |
| invert | Goldilocks | 539 vs winter-math 1,770 | −1,231 (0.30×) | 540 vs winter-math 900 | −360 (0.60×) | 567 vs winter-math 1,821 | −1,254 (0.31×) | 563 vs winter-math 1,000 | −437 (0.56×) |
| invert | BN254 scalar | 8,459 vs halo2curves 14,258 | −5,799 (0.59×) | 6,468 vs halo2curves 8,006 | −1,538 (0.81×) | 11,937 vs halo2curves 21,586 | −9,649 (0.55×) | 7,922 vs halo2curves 13,859 | −5,937 (0.57×) |
| invert | secp256k1 base | 8,498 vs halo2curves 14,300 | −5,802 (0.59×) | 6,026 vs halo2curves 8,032 | −2,006 (0.75×) | 11,695 vs halo2curves 21,669 | −9,974 (0.54×) | 7,981 vs halo2curves 14,049 | −6,068 (0.57×) |
| invert | P-256 base | 8,121 vs halo2curves 14,321 | −6,200 (0.57×) | 6,069 vs halo2curves 8,028 | −1,959 (0.76×) | 11,769 vs halo2curves 21,614 | −9,845 (0.54×) | 7,686 vs halo2curves 14,006 | −6,320 (0.55×) |
| invert | P-256 scalar | 8,527 vs halo2curves 14,289 | −5,762 (0.60×) | 6,384 vs halo2curves 8,047 | −1,663 (0.79×) | 11,721 vs halo2curves 21,562 | −9,841 (0.54×) | 8,063 vs halo2curves 14,073 | −6,010 (0.57×) |
| invert | Tom-256 base | 8,765 vs halo2curves 14,285 | −5,520 (0.61×) | 6,455 vs halo2curves 8,042 | −1,587 (0.80×) | 11,502 vs halo2curves 21,700 | −10,198 (0.53×) | 7,946 vs halo2curves 14,016 | −6,070 (0.57×) |
| invert | BLS12-381 base | 19,342 vs halo2curves 25,465 | −6,123 (0.76×) | 11,777 vs halo2curves 13,157 | −1,380 (0.90×) | 25,202 vs halo2curves 38,197 | −12,995 (0.66×) | 15,431 vs halo2curves 23,619 | −8,188 (0.65×) |
| invert | P-521 base | 33,489 vs crypto-bigint 65,060 | −31,571 (0.51×) | 18,076 vs crypto-bigint 50,092 | −32,016 (0.36×) | 47,853 vs crypto-bigint 82,240 | −34,387 (0.58×) | 29,358 vs crypto-bigint 63,910 | −34,552 (0.46×) |
| sqrt | Mersenne-31 | 160 vs plonky3 322 | −162 (0.50×) | 160 vs plonky3 322 | −162 (0.50×) | 162 vs plonky3 318 | −156 (0.51×) | 163 vs plonky3 324 | −161 (0.50×) |
| sqrt | BabyBear | 2,340 vs plonky3 3,083 | −743 (0.76×) | 1,959 vs plonky3 3,085 | −1,126 (0.64×) | 2,555 vs plonky3 3,018 | −463 (0.85×) | 2,381 vs plonky3 3,000 | −619 (0.79×) |
| sqrt | Goldilocks | 4,854 vs crypto-bigint 48,688 | −43,834 (0.10×) | 3,955 vs crypto-bigint 28,833 | −24,878 (0.14×) | 5,966 vs crypto-bigint 54,504 | −48,538 (0.11×) | 4,503 vs crypto-bigint 32,891 | −28,388 (0.14×) |
| sqrt | BN254 scalar | 104,669 vs ark-ff 149,458 | −44,789 (0.70×) | 47,254 vs halo2curves 102,199 | −54,945 (0.46×) | 166,300 vs ark-ff 205,421 | −39,121 (0.81×) | 80,543 vs ark-ff 145,361 | −64,818 (0.55×) |
| sqrt | secp256k1 base | 41,129 vs ark-ff 118,775 | −77,646 (0.35×) | 16,750 vs halo2curves 43,540 | −26,790 (0.38×) | 65,663 vs ark-ff 173,719 | −108,056 (0.38×) | 24,059 vs crypto-bigint 79,772 | −55,713 (0.30×) |
| sqrt | P-256 base | 43,055 vs ark-ff 59,757 | −16,702 (0.72×) | 18,745 vs k256/p256 21,613 | −2,868 (0.87×) | 74,559 vs ark-ff 99,066 | −24,507 (0.75×) | 29,394 vs k256/p256 33,709 | −4,315 (0.87×) |
| sqrt | P-256 scalar | 73,944 vs ark-ff 118,777 | −44,833 (0.62×) | 30,038 vs halo2curves 49,879 | −19,841 (0.60×) | 109,756 vs ark-ff 158,523 | −48,767 (0.69×) | 59,861 vs halo2curves 72,406 | −12,545 (0.83×) |
| sqrt | Tom-256 base | 66,189 vs crypto-bigint 151,019 | −84,830 (0.44×) | 25,634 vs halo2curves 45,925 | −20,291 (0.56×) | 85,847 vs crypto-bigint 224,942 | −139,095 (0.38×) | 49,758 vs crypto-bigint 74,780 | −25,022 (0.67×) |
| sqrt | BLS12-381 base | 296,985 vs ark-ff 423,373 | −126,388 (0.70×) | 90,304 vs crypto-bigint 145,150 | −54,846 (0.62×) | 390,419 vs ark-ff 573,485 | −183,066 (0.68×) | 213,166 vs crypto-bigint 268,956 | −55,790 (0.79×) |
| sqrt | P-521 base | 318,749 vs crypto-bigint 1,526,546 | −1,207,797 (0.21×) | 123,473 vs crypto-bigint 296,852 | −173,379 (0.42×) | 461,539 vs crypto-bigint 1,574,170 | −1,112,631 (0.29×) | 255,934 vs crypto-bigint 591,157 | −335,223 (0.43×) |
| mul_u64 | Mersenne-31 | 7.4 vs crypto-bigint 63.6 | −56.2 (0.12×) | 7.4 vs crypto-bigint 33.5 | −26.1 (0.22×) | 7.6 vs crypto-bigint 70.2 | −62.6 (0.11×) | 7.6 vs crypto-bigint 39.5 | −31.9 (0.19×) |
| mul_u64 | BabyBear | 15.5 vs plonky3 35.2 | −19.7 (0.44×) | 11.4 vs plonky3 12.7 | −1.3 (0.90×) | 17.5 vs plonky3 44.6 | −27.1 (0.39×) | 15.0 vs plonky3 19.0 | −4.0 (0.79×) |
| mul_u64 | Goldilocks | 18.7 vs winter-math 24.9 | −6.2 (0.75×) | 7.1 vs winter-math 11.5 | −4.4 (0.62×) | 26.3 vs winter-math 31.1 | −4.8 (0.85×) | 8.7 vs winter-math 13.8 | −5.1 (0.63×) |
| mul_u64 | BN254 scalar | 94.1 vs ark-ff 668 | −574 (0.14×) | 30.8 vs crypto-bigint 262 | −231 (0.12×) | 120 vs ark-ff 826 | −706 (0.15×) | 43.2 vs crypto-bigint 407 | −364 (0.11×) |
| mul_u64 | secp256k1 base | 47.3 vs ark-ff 435 | −388 (0.11×) | 13.3 vs crypto-bigint 262 | −249 (0.05×) | 63.3 vs ark-ff 609 | −546 (0.10×) | 22.0 vs halo2curves 373 | −351 (0.06×) |
| mul_u64 | P-256 base | 73.7 vs k256/p256 488 | −414 (0.15×) | 28.3 vs k256/p256 199 | −171 (0.14×) | 94.1 vs ark-ff 833 | −739 (0.11×) | 41.2 vs k256/p256 256 | −215 (0.16×) |
| mul_u64 | P-256 scalar | 82.7 vs k256/p256 511 | −428 (0.16×) | 35.8 vs crypto-bigint 261 | −225 (0.14×) | 109 vs k256/p256 824 | −715 (0.13×) | 45.9 vs k256/p256 337 | −291 (0.14×) |
| mul_u64 | Tom-256 base | 83.3 vs crypto-bigint 848 | −765 (0.10×) | 30.1 vs crypto-bigint 261 | −231 (0.12×) | 108 vs halo2curves 1,287 | −1,179 (0.08×) | 45.0 vs crypto-bigint 407 | −362 (0.11×) |
| mul_u64 | BLS12-381 base | 136 vs ark-ff 1,650 | −1,514 (0.08×) | 43.0 vs crypto-bigint 512 | −469 (0.08×) | 177 vs ark-ff 2,080 | −1,903 (0.09×) | 61.5 vs crypto-bigint 967 | −906 (0.06×) |
| mul_u64 | P-521 base | 99.9 vs crypto-bigint 3,017 | −2,917 (0.03×) | 33.5 vs crypto-bigint 988 | −955 (0.03×) | 133 vs crypto-bigint 3,980 | −3,847 (0.03×) | 51.0 vs crypto-bigint 1,743 | −1,692 (0.03×) |
| mul_uint | Mersenne-31 | 7.4 vs crypto-bigint 63.7 | −56.3 (0.12×) | 7.4 vs crypto-bigint 33.5 | −26.1 (0.22×) | 7.6 vs crypto-bigint 70.3 | −62.7 (0.11×) | 7.6 vs crypto-bigint 39.5 | −31.9 (0.19×) |
| mul_uint | BabyBear | 15.5 vs plonky3 23.1 | −7.6 (0.67×) | 11.4 vs plonky3 12.8 | −1.4 (0.89×) | 17.5 vs plonky3 34.3 | −16.8 (0.51×) | 15.0 vs plonky3 19.0 | −4.0 (0.79×) |
| mul_uint | Goldilocks | 18.7 vs winter-math 24.9 | −6.2 (0.75×) | 7.1 vs winter-math 11.5 | −4.4 (0.62×) | 26.3 vs winter-math 31.1 | −4.8 (0.85×) | 8.7 vs winter-math 13.8 | −5.1 (0.63×) |
| mul_uint | BN254 scalar | 434 vs halo2curves 765 | −331 (0.57×) | 180 vs halo2curves 181 | −1.0 (0.99×) | 540 vs halo2curves 1,297 | −757 (0.42×) | 296 vs halo2curves 361 | −65.0 (0.82×) |
| mul_uint | secp256k1 base | 257 vs halo2curves 518 | −261 (0.50×) | 64.9 vs halo2curves 154 | −89.1 (0.42×) | 327 vs halo2curves 950 | −623 (0.34×) | 105 vs halo2curves 264 | −159 (0.40×) |
| mul_uint | P-256 base | 345 vs halo2curves 462 | −117 (0.75×) | 129 vs halo2curves 142 | −13.0 (0.91×) | 431 vs halo2curves 995 | −564 (0.43×) | 229 vs halo2curves 294 | −65.0 (0.78×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 563 | −138 (0.75×) | 190 vs halo2curves 185 | +5.0 (1.03×) | 484 vs k256/p256 884 | −400 (0.55×) | 308 vs halo2curves 345 | −37.0 (0.89×) |
| mul_uint | Tom-256 base | 411 vs halo2curves 694 | −283 (0.59×) | 181 vs halo2curves 180 | +1.0 (1.01×) | 487 vs halo2curves 1,186 | −699 (0.41×) | 297 vs halo2curves 355 | −58.0 (0.84×) |
| mul_uint | BLS12-381 base | 972 vs crypto-bigint 1,883 | −911 (0.52×) | 378 vs halo2curves 384 | −6.0 (0.98×) | 1,131 vs crypto-bigint 2,499 | −1,368 (0.45×) | 919 vs crypto-bigint 948 | −29.0 (0.97×) |
| mul_uint | P-521 base | 1,535 vs crypto-bigint 2,984 | −1,449 (0.51×) | 260 vs crypto-bigint 982 | −722 (0.26×) | 2,351 vs crypto-bigint 4,033 | −1,682 (0.58×) | 447 vs crypto-bigint 1,737 | −1,290 (0.26×) |
| add_u64 | Mersenne-31 | 5.4 vs crypto-bigint 37.0 | −31.6 (0.15×) | 5.4 vs crypto-bigint 29.3 | −23.9 (0.18×) | 5.5 vs crypto-bigint 40.8 | −35.3 (0.13×) | 5.5 vs crypto-bigint 29.6 | −24.1 (0.19×) |
| add_u64 | BabyBear | 13.0 vs plonky3 31.9 | −18.9 (0.41×) | 9.0 vs plonky3 10.3 | −1.3 (0.87×) | 12.9 vs plonky3 43.0 | −30.1 (0.30×) | 12.2 vs plonky3 13.7 | −1.5 (0.89×) |
| add_u64 | Goldilocks | 9.3 vs winter-math 13.6 | −4.3 (0.68×) | 7.5 vs winter-math 8.1 | −0.6 (0.93×) | 11.8 vs winter-math 18.2 | −6.4 (0.65×) | 8.5 vs winter-math 9.8 | −1.3 (0.87×) |
| add_u64 | BN254 scalar | 118 vs ark-ff 347 | −229 (0.34×) | 41.5 vs crypto-bigint 223 | −182 (0.19×) | 140 vs ark-ff 405 | −265 (0.35×) | 62.5 vs halo2curves 295 | −233 (0.21×) |
| add_u64 | secp256k1 base | 8.8 vs ark-ff 158 | −149 (0.06×) | 8.4 vs ark-ff 128 | −120 (0.07×) | 8.3 vs ark-ff 225 | −217 (0.04×) | 10.7 vs ark-ff 180 | −169 (0.06×) |
| add_u64 | P-256 base | 72.0 vs ark-ff 264 | −192 (0.27×) | 37.2 vs k256/p256 113 | −75.8 (0.33×) | 78.9 vs ark-ff 409 | −330 (0.19×) | 52.5 vs k256/p256 157 | −105 (0.33×) |
| add_u64 | P-256 scalar | 67.9 vs k256/p256 46.4 | +21.5 (1.46×) | 39.9 vs k256/p256 46.3 | −6.4 (0.86×) | 73.3 vs k256/p256 54.5 | +18.8 (1.34×) | 62.3 vs k256/p256 54.6 | +7.7 (1.14×) |
| add_u64 | Tom-256 base | 89.4 vs crypto-bigint 502 | −413 (0.18×) | 43.6 vs crypto-bigint 224 | −180 (0.19×) | 105 vs halo2curves 701 | −596 (0.15×) | 59.0 vs halo2curves 289 | −230 (0.20×) |
| add_u64 | BLS12-381 base | 153 vs ark-ff 849 | −696 (0.18×) | 62.1 vs halo2curves 360 | −298 (0.17×) | 248 vs ark-ff 1,075 | −827 (0.23×) | 84.2 vs crypto-bigint 640 | −556 (0.13×) |
| add_u64 | P-521 base | 22.3 vs crypto-bigint 1,673 | −1,651 (0.01×) | 18.5 vs crypto-bigint 649 | −631 (0.03×) | 18.5 vs crypto-bigint 2,187 | −2,169 (0.01×) | 22.8 vs crypto-bigint 1,157 | −1,134 (0.02×) |
| add_uint | Mersenne-31 | 5.4 vs crypto-bigint 37.0 | −31.6 (0.15×) | 5.4 vs crypto-bigint 29.0 | −23.6 (0.19×) | 5.5 vs crypto-bigint 39.5 | −34.0 (0.14×) | 5.5 vs crypto-bigint 29.8 | −24.3 (0.18×) |
| add_uint | BabyBear | 13.0 vs plonky3 22.1 | −9.1 (0.59×) | 9.0 vs plonky3 10.3 | −1.3 (0.87×) | 12.9 vs plonky3 32.2 | −19.3 (0.40×) | 12.2 vs plonky3 13.7 | −1.5 (0.89×) |
| add_uint | Goldilocks | 9.3 vs winter-math 13.6 | −4.3 (0.68×) | 7.5 vs winter-math 8.1 | −0.6 (0.93×) | 11.8 vs winter-math 18.2 | −6.4 (0.65×) | 8.5 vs winter-math 9.8 | −1.3 (0.87×) |
| add_uint | BN254 scalar | 250 vs halo2curves 419 | −169 (0.60×) | 101 vs halo2curves 111 | −10.0 (0.91×) | 357 vs halo2curves 668 | −311 (0.53×) | 177 vs halo2curves 204 | −27.0 (0.87×) |
| add_uint | secp256k1 base | 19.5 vs halo2curves 224 | −205 (0.09×) | 13.7 vs halo2curves 86.0 | −72.3 (0.16×) | 16.1 vs halo2curves 410 | −394 (0.04×) | 19.8 vs halo2curves 139 | −119 (0.14×) |
| add_uint | P-256 base | 182 vs halo2curves 248 | −66.0 (0.73×) | 76.2 vs halo2curves 90.0 | −13.8 (0.85×) | 324 vs halo2curves 504 | −180 (0.64×) | 141 vs halo2curves 152 | −11.0 (0.93×) |
| add_uint | P-256 scalar | 262 vs k256/p256 84.7 | +177 (3.09×) | 104 vs k256/p256 84.2 | +19.8 (1.24×) | 321 vs k256/p256 115 | +206 (2.79×) | 181 vs k256/p256 115 | +66.0 (1.57×) |
| add_uint | Tom-256 base | 257 vs halo2curves 376 | −119 (0.68×) | 104 vs halo2curves 111 | −7.0 (0.94×) | 327 vs halo2curves 619 | −292 (0.53×) | 175 vs halo2curves 195 | −20.0 (0.90×) |
| add_uint | BLS12-381 base | 767 vs halo2curves 961 | −194 (0.80×) | 208 vs halo2curves 222 | −14.0 (0.94×) | 952 vs crypto-bigint 1,364 | −412 (0.70×) | 379 vs halo2curves 422 | −43.0 (0.90×) |
| add_uint | P-521 base | 63.5 vs crypto-bigint 1,673 | −1,610 (0.04×) | 41.7 vs crypto-bigint 641 | −599 (0.07×) | 71.8 vs crypto-bigint 2,218 | −2,146 (0.03×) | 60.5 vs crypto-bigint 1,155 | −1,095 (0.05×) |
| sub_u64 | Mersenne-31 | 5.4 vs crypto-bigint 24.5 | −19.1 (0.22×) | 5.4 vs crypto-bigint 18.1 | −12.7 (0.30×) | 5.4 vs crypto-bigint 30.6 | −25.2 (0.18×) | 5.4 vs crypto-bigint 20.3 | −14.9 (0.27×) |
| sub_u64 | BabyBear | 11.1 vs crypto-bigint 29.5 | −18.4 (0.38×) | 9.0 vs plonky3 11.3 | −2.3 (0.80×) | 13.1 vs crypto-bigint 41.1 | −28.0 (0.32×) | 11.9 vs plonky3 14.1 | −2.2 (0.84×) |
| sub_u64 | Goldilocks | 9.4 vs winter-math 13.1 | −3.7 (0.72×) | 7.4 vs winter-math 7.4 | +0.0 (1.00×) | 11.6 vs winter-math 17.6 | −6.0 (0.66×) | 8.5 vs winter-math 8.6 | −0.1 (0.99×) |
| sub_u64 | BN254 scalar | 112 vs ark-ff 349 | −237 (0.32×) | 45.0 vs crypto-bigint 151 | −106 (0.30×) | 137 vs ark-ff 396 | −259 (0.35×) | 63.3 vs crypto-bigint 231 | −168 (0.27×) |
| sub_u64 | secp256k1 base | 9.4 vs ark-ff 161 | −152 (0.06×) | 8.6 vs ark-ff 130 | −121 (0.07×) | 7.8 vs ark-ff 230 | −222 (0.03×) | 9.4 vs ark-ff 190 | −181 (0.05×) |
| sub_u64 | P-256 base | 72.3 vs ark-ff 265 | −193 (0.27×) | 36.0 vs k256/p256 99.3 | −63.3 (0.36×) | 76.5 vs ark-ff 401 | −325 (0.19×) | 50.6 vs k256/p256 147 | −96.4 (0.34×) |
| sub_u64 | P-256 scalar | 67.2 vs k256/p256 22.2 | +45.0 (3.03×) | 36.8 vs k256/p256 22.0 | +14.8 (1.67×) | 71.4 vs k256/p256 23.7 | +47.7 (3.01×) | 58.7 vs k256/p256 23.8 | +34.9 (2.47×) |
| sub_u64 | Tom-256 base | 88.5 vs crypto-bigint 497 | −409 (0.18×) | 42.3 vs crypto-bigint 151 | −109 (0.28×) | 103 vs halo2curves 683 | −580 (0.15×) | 59.9 vs crypto-bigint 231 | −171 (0.26×) |
| sub_u64 | BLS12-381 base | 161 vs ark-ff 836 | −675 (0.19×) | 62.7 vs halo2curves 345 | −282 (0.18×) | 238 vs ark-ff 1,053 | −815 (0.23×) | 86.1 vs halo2curves 568 | −482 (0.15×) |
| sub_u64 | P-521 base | 24.3 vs crypto-bigint 1,666 | −1,642 (0.01×) | 19.1 vs crypto-bigint 621 | −602 (0.03×) | 21.3 vs crypto-bigint 2,159 | −2,138 (0.01×) | 20.0 vs crypto-bigint 1,145 | −1,125 (0.02×) |
| sub_uint | Mersenne-31 | 5.4 vs crypto-bigint 24.5 | −19.1 (0.22×) | 5.4 vs crypto-bigint 18.1 | −12.7 (0.30×) | 5.4 vs crypto-bigint 30.2 | −24.8 (0.18×) | 5.4 vs crypto-bigint 20.3 | −14.9 (0.27×) |
| sub_uint | BabyBear | 11.1 vs plonky3 22.1 | −11.0 (0.50×) | 9.0 vs plonky3 11.1 | −2.1 (0.81×) | 13.1 vs plonky3 32.4 | −19.3 (0.40×) | 11.9 vs plonky3 14.1 | −2.2 (0.84×) |
| sub_uint | Goldilocks | 9.4 vs winter-math 13.1 | −3.7 (0.72×) | 7.4 vs winter-math 7.4 | +0.0 (1.00×) | 11.6 vs winter-math 17.6 | −6.0 (0.66×) | 8.5 vs winter-math 8.6 | −0.1 (0.99×) |
| sub_uint | BN254 scalar | 249 vs halo2curves 397 | −148 (0.63×) | 99.4 vs halo2curves 105 | −5.6 (0.95×) | 371 vs halo2curves 657 | −286 (0.56×) | 173 vs halo2curves 193 | −20.0 (0.90×) |
| sub_uint | secp256k1 base | 19.9 vs halo2curves 210 | −190 (0.09×) | 14.3 vs halo2curves 80.3 | −66.0 (0.18×) | 15.4 vs halo2curves 397 | −382 (0.04×) | 19.6 vs halo2curves 128 | −108 (0.15×) |
| sub_uint | P-256 base | 174 vs halo2curves 233 | −59.0 (0.75×) | 75.1 vs halo2curves 84.8 | −9.7 (0.89×) | 280 vs halo2curves 490 | −210 (0.57×) | 134 vs halo2curves 142 | −8.0 (0.94×) |
| sub_uint | P-256 scalar | 248 vs k256/p256 70.7 | +177 (3.51×) | 101 vs k256/p256 70.0 | +31.0 (1.44×) | 322 vs k256/p256 83.5 | +239 (3.86×) | 174 vs k256/p256 75.2 | +98.8 (2.31×) |
| sub_uint | Tom-256 base | 250 vs halo2curves 366 | −116 (0.68×) | 103 vs halo2curves 103 | +0.0 (1.00×) | 323 vs halo2curves 613 | −290 (0.53×) | 171 vs halo2curves 185 | −14.0 (0.92×) |
| sub_uint | BLS12-381 base | 763 vs halo2curves 944 | −181 (0.81×) | 202 vs halo2curves 214 | −12.0 (0.94×) | 950 vs crypto-bigint 1,371 | −421 (0.69×) | 374 vs halo2curves 406 | −32.0 (0.92×) |
| sub_uint | P-521 base | 68.4 vs crypto-bigint 1,653 | −1,585 (0.04×) | 40.0 vs crypto-bigint 615 | −575 (0.07×) | 69.8 vs crypto-bigint 2,190 | −2,120 (0.03×) | 61.8 vs crypto-bigint 1,146 | −1,084 (0.05×) |
| from_u64 | Mersenne-31 | 4.0 vs plonky3 4.0 | +0.0 (1.00×) | 4.0 vs plonky3 4.0 | +0.0 (1.00×) | 3.8 vs plonky3 4.4 | −0.6 (0.86×) | 3.8 vs plonky3 4.3 | −0.5 (0.88×) |
| from_u64 | BabyBear | 11.9 vs crypto-bigint 23.0 | −11.1 (0.52×) | 7.5 vs plonky3 9.3 | −1.8 (0.81×) | 11.0 vs crypto-bigint 33.4 | −22.4 (0.33×) | 10.0 vs plonky3 11.7 | −1.7 (0.85×) |
| from_u64 | Goldilocks | 7.9 vs winter-math 11.2 | −3.3 (0.71×) | 5.6 vs winter-math 5.6 | +0.0 (1.00×) | 8.6 vs winter-math 16.0 | −7.4 (0.54×) | 6.9 vs winter-math 6.9 | +0.0 (1.00×) |
| from_u64 | BN254 scalar | 104 vs ark-ff 319 | −215 (0.33×) | 34.1 vs crypto-bigint 129 | −94.9 (0.26×) | 124 vs ark-ff 374 | −250 (0.33×) | 49.1 vs crypto-bigint 204 | −155 (0.24×) |
| from_u64 | secp256k1 base | 3.3 vs k256/p256 5.3 | −2.0 (0.62×) | 3.3 vs k256/p256 5.3 | −2.0 (0.62×) | 3.2 vs k256/p256 5.0 | −1.8 (0.64×) | 3.2 vs k256/p256 5.0 | −1.8 (0.64×) |
| from_u64 | P-256 base | 58.1 vs ark-ff 238 | −180 (0.24×) | 29.0 vs k256/p256 94.9 | −65.9 (0.31×) | 70.1 vs ark-ff 381 | −311 (0.18×) | 41.3 vs k256/p256 128 | −86.7 (0.32×) |
| from_u64 | P-256 scalar | 55.4 vs k256/p256 3.3 | +52.1 (16.79×) | 29.2 vs k256/p256 3.3 | +25.9 (8.85×) | 60.2 vs k256/p256 2.9 | +57.3 (20.76×) | 47.1 vs k256/p256 3.1 | +44.0 (15.19×) |
| from_u64 | Tom-256 base | 81.1 vs crypto-bigint 432 | −351 (0.19×) | 32.2 vs crypto-bigint 129 | −96.8 (0.25×) | 86.9 vs halo2curves 653 | −566 (0.13×) | 48.1 vs crypto-bigint 204 | −156 (0.24×) |
| from_u64 | BLS12-381 base | 147 vs ark-ff 808 | −661 (0.18×) | 51.7 vs crypto-bigint 257 | −205 (0.20×) | 182 vs ark-ff 1,011 | −829 (0.18×) | 67.1 vs halo2curves 468 | −401 (0.14×) |
| from_u64 | P-521 base | 6.0 vs crypto-bigint 1,448 | −1,442 (0.00×) | 6.0 vs crypto-bigint 481 | −475 (0.01×) | 6.0 vs crypto-bigint 1,980 | −1,974 (0.00×) | 6.0 vs crypto-bigint 844 | −838 (0.01×) |
| div_u64 | BN254 scalar | 1,205 vs halo2curves 13,431 | −12,226 (0.09×) | 861 vs halo2curves 6,999 | −6,138 (0.12×) | 1,374 vs halo2curves 19,349 | −17,975 (0.07×) | 1,092 vs halo2curves 12,104 | −11,012 (0.09×) |
| div_u64 | secp256k1 base | 992 vs halo2curves 13,071 | −12,079 (0.08×) | 709 vs halo2curves 6,951 | −6,242 (0.10×) | 1,220 vs halo2curves 18,863 | −17,643 (0.06×) | 804 vs halo2curves 11,854 | −11,050 (0.07×) |
| div_u64 | P-256 base | 1,080 vs halo2curves 13,108 | −12,028 (0.08×) | 798 vs halo2curves 6,892 | −6,094 (0.12×) | 1,341 vs halo2curves 18,994 | −17,653 (0.07×) | 987 vs halo2curves 11,831 | −10,844 (0.08×) |
| div_u64 | P-256 scalar | 1,182 vs halo2curves 13,363 | −12,181 (0.09×) | 864 vs halo2curves 6,944 | −6,080 (0.12×) | 1,444 vs halo2curves 19,327 | −17,883 (0.07×) | 1,119 vs halo2curves 12,029 | −10,910 (0.09×) |
| div_u64 | Tom-256 base | 1,192 vs halo2curves 13,284 | −12,092 (0.09×) | 867 vs halo2curves 6,948 | −6,081 (0.12×) | 1,461 vs halo2curves 19,062 | −17,601 (0.08×) | 1,086 vs halo2curves 11,973 | −10,887 (0.09×) |
| div_u64 | BLS12-381 base | 1,780 vs halo2curves 23,780 | −22,000 (0.07×) | 1,100 vs halo2curves 11,031 | −9,931 (0.10×) | 2,461 vs halo2curves 32,737 | −30,276 (0.08×) | 1,757 vs halo2curves 18,829 | −17,072 (0.09×) |
| div_u64 | P-521 base | 2,425 vs crypto-bigint 62,014 | −59,589 (0.04×) | 990 vs crypto-bigint 45,291 | −44,301 (0.02×) | 3,597 vs crypto-bigint 77,586 | −73,989 (0.05×) | 1,489 vs crypto-bigint 57,754 | −56,265 (0.03×) |
| div_uint | Mersenne-31 | 273 vs crypto-bigint 1,813 | −1,540 (0.15×) | 273 vs crypto-bigint 1,593 | −1,320 (0.17×) | 279 vs runtime-algebra 2,100 | −1,821 (0.13×) | 279 vs crypto-bigint 1,974 | −1,695 (0.14×) |
| div_uint | BabyBear | 276 vs plonky3 665 | −389 (0.42×) | 279 vs plonky3 674 | −395 (0.41×) | 280 vs plonky3 719 | −439 (0.39×) | 283 vs plonky3 727 | −444 (0.39×) |
| div_uint | Goldilocks | 552 vs winter-math 1,812 | −1,260 (0.30×) | 544 vs winter-math 936 | −392 (0.58×) | 586 vs winter-math 1,890 | −1,304 (0.31×) | 567 vs winter-math 1,041 | −474 (0.54×) |
| div_uint | BN254 scalar | 9,142 vs halo2curves 15,017 | −5,875 (0.61×) | 6,657 vs halo2curves 8,159 | −1,502 (0.82×) | 12,687 vs halo2curves 22,906 | −10,219 (0.55×) | 8,439 vs halo2curves 14,905 | −6,466 (0.57×) |
| div_uint | secp256k1 base | 8,531 vs halo2curves 14,895 | −6,364 (0.57×) | 6,054 vs halo2curves 8,223 | −2,169 (0.74×) | 12,135 vs halo2curves 22,614 | −10,479 (0.54×) | 8,101 vs halo2curves 14,840 | −6,739 (0.55×) |
| div_uint | P-256 base | 8,781 vs halo2curves 14,827 | −6,046 (0.59×) | 6,171 vs halo2curves 8,219 | −2,048 (0.75×) | 12,190 vs halo2curves 22,729 | −10,539 (0.54×) | 7,832 vs halo2curves 14,802 | −6,970 (0.53×) |
| div_uint | P-256 scalar | 9,135 vs halo2curves 15,029 | −5,894 (0.61×) | 6,501 vs halo2curves 8,219 | −1,718 (0.79×) | 12,571 vs halo2curves 23,096 | −10,525 (0.54×) | 8,614 vs halo2curves 14,921 | −6,307 (0.58×) |
| div_uint | Tom-256 base | 8,983 vs halo2curves 15,003 | −6,020 (0.60×) | 6,660 vs halo2curves 8,219 | −1,559 (0.81×) | 12,416 vs halo2curves 22,814 | −10,398 (0.54×) | 8,412 vs halo2curves 14,829 | −6,417 (0.57×) |
| div_uint | BLS12-381 base | 20,754 vs halo2curves 27,235 | −6,481 (0.76×) | 12,071 vs halo2curves 13,648 | −1,577 (0.88×) | 26,417 vs halo2curves 40,297 | −13,880 (0.66×) | 16,860 vs halo2curves 24,802 | −7,942 (0.68×) |
| div_uint | P-521 base | 34,607 vs crypto-bigint 67,859 | −33,252 (0.51×) | 18,467 vs crypto-bigint 50,956 | −32,489 (0.36×) | 49,642 vs crypto-bigint 85,802 | −36,160 (0.58×) | 30,158 vs crypto-bigint 63,755 | −33,597 (0.47×) |
| add, constant time | Mersenne-31 | 2.8 vs runtime-algebra 11.3 | −8.5 (0.25×) | 2.8 vs runtime-algebra 11.3 | −8.5 (0.25×) | 2.7 vs crypto-bigint 11.9 | −9.2 (0.23×) | 2.7 vs crypto-bigint 11.6 | −8.9 (0.23×) |
| add, constant time | BabyBear | 2.6 vs runtime-algebra 11.3 | −8.7 (0.23×) | 2.6 vs runtime-algebra 11.3 | −8.7 (0.23×) | 2.7 vs crypto-bigint 11.9 | −9.2 (0.23×) | 2.7 vs crypto-bigint 11.6 | −8.9 (0.23×) |
| add, constant time | Goldilocks | 3.5 vs winter-math 3.3 | +0.2 (1.06×) | 3.5 vs winter-math 3.3 | +0.2 (1.06×) | 3.1 vs winter-math 3.5 | −0.4 (0.89×) | 3.1 vs winter-math 3.5 | −0.4 (0.89×) |
| add, constant time | BN254 scalar | 24.8 vs halo2curves 40.4 | −15.6 (0.61×) | 16.2 vs halo2curves 21.5 | −5.3 (0.75×) | 21.6 vs halo2curves 38.6 | −17.0 (0.56×) | 25.1 vs halo2curves 34.1 | −9.0 (0.74×) |
| add, constant time | secp256k1 base | 28.0 vs halo2curves 35.4 | −7.4 (0.79×) | 16.3 vs halo2curves 19.7 | −3.4 (0.83×) | 25.7 vs halo2curves 36.1 | −10.4 (0.71×) | 27.4 vs halo2curves 32.1 | −4.7 (0.85×) |
| add, constant time | P-256 base | 24.5 vs halo2curves 33.4 | −8.9 (0.73×) | 16.0 vs k256/p256 19.0 | −3.0 (0.84×) | 24.4 vs k256/p256 32.6 | −8.2 (0.75×) | 26.6 vs k256/p256 30.1 | −3.5 (0.88×) |
| add, constant time | P-256 scalar | 27.8 vs halo2curves 38.5 | −10.7 (0.72×) | 16.6 vs halo2curves 21.3 | −4.7 (0.78×) | 26.7 vs halo2curves 38.2 | −11.5 (0.70×) | 28.1 vs halo2curves 33.6 | −5.5 (0.84×) |
| add, constant time | Tom-256 base | 27.4 vs halo2curves 39.5 | −12.1 (0.69×) | 16.3 vs halo2curves 20.9 | −4.6 (0.78×) | 26.8 vs halo2curves 38.7 | −11.9 (0.69×) | 28.1 vs halo2curves 33.1 | −5.0 (0.85×) |
| add, constant time | BLS12-381 base | 44.3 vs halo2curves 71.1 | −26.8 (0.62×) | 24.2 vs halo2curves 35.0 | −10.8 (0.69×) | 38.2 vs halo2curves 67.2 | −29.0 (0.57×) | 38.6 vs halo2curves 54.1 | −15.5 (0.71×) |
| add, constant time | P-521 base | 75.1 vs crypto-bigint 124 | −48.9 (0.61×) | 40.7 vs crypto-bigint 107 | −66.3 (0.38×) | 76.7 vs crypto-bigint 170 | −93.3 (0.45×) | 71.1 vs crypto-bigint 177 | −106 (0.40×) |
| sub, constant time | Mersenne-31 | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.6 vs crypto-bigint 4.6 | −2.0 (0.57×) | 2.5 vs crypto-bigint 4.6 | −2.1 (0.54×) |
| sub, constant time | BabyBear | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.6 vs crypto-bigint 4.8 | −2.2 (0.54×) | 2.8 vs crypto-bigint 4.6 | −1.8 (0.61×) | 2.6 vs crypto-bigint 4.5 | −1.9 (0.58×) |
| sub, constant time | Goldilocks | 2.8 vs winter-math 2.8 | +0.0 (1.00×) | 2.8 vs winter-math 2.8 | +0.0 (1.00×) | 2.7 vs winter-math 2.7 | +0.0 (1.00×) | 2.7 vs winter-math 2.7 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 21.9 vs halo2curves 24.4 | −2.5 (0.90×) | 14.6 vs halo2curves 16.1 | −1.5 (0.91×) | 21.1 vs halo2curves 23.8 | −2.7 (0.89×) | 23.1 vs halo2curves 23.1 | +0.0 (1.00×) |
| sub, constant time | secp256k1 base | 20.9 vs halo2curves 23.0 | −2.1 (0.91×) | 13.8 vs halo2curves 14.9 | −1.1 (0.93×) | 18.5 vs halo2curves 23.0 | −4.5 (0.80×) | 21.1 vs halo2curves 21.7 | −0.6 (0.97×) |
| sub, constant time | P-256 base | 18.7 vs halo2curves 21.0 | −2.3 (0.89×) | 12.9 vs halo2curves 13.6 | −0.7 (0.95×) | 18.0 vs halo2curves 20.6 | −2.6 (0.87×) | 20.0 vs halo2curves 20.4 | −0.4 (0.98×) |
| sub, constant time | P-256 scalar | 21.2 vs k256/p256 24.1 | −2.9 (0.88×) | 14.1 vs halo2curves 15.1 | −1.0 (0.93×) | 21.5 vs halo2curves 22.4 | −0.9 (0.96×) | 22.0 vs halo2curves 23.1 | −1.1 (0.95×) |
| sub, constant time | Tom-256 base | 23.0 vs halo2curves 24.4 | −1.4 (0.94×) | 14.2 vs halo2curves 16.4 | −2.2 (0.87×) | 20.8 vs halo2curves 23.9 | −3.1 (0.87×) | 22.6 vs halo2curves 23.1 | −0.5 (0.98×) |
| sub, constant time | BLS12-381 base | 38.1 vs halo2curves 43.5 | −5.4 (0.88×) | 23.1 vs halo2curves 26.4 | −3.3 (0.88×) | 36.6 vs halo2curves 42.6 | −6.0 (0.86×) | 35.6 vs halo2curves 36.1 | −0.5 (0.99×) |
| sub, constant time | P-521 base | 56.5 vs crypto-bigint 94.5 | −38.0 (0.60×) | 33.5 vs crypto-bigint 71.0 | −37.5 (0.47×) | 56.3 vs crypto-bigint 126 | −69.7 (0.45×) | 50.1 vs crypto-bigint 144 | −93.9 (0.35×) |
| mul, constant time | Mersenne-31 | 3.7 vs crypto-bigint 30.4 | −26.7 (0.12×) | 3.7 vs crypto-bigint 16.8 | −13.1 (0.22×) | 3.6 vs crypto-bigint 39.0 | −35.4 (0.09×) | 3.5 vs crypto-bigint 18.4 | −14.9 (0.19×) |
| mul, constant time | BabyBear | 5.2 vs crypto-bigint 30.5 | −25.3 (0.17×) | 3.7 vs crypto-bigint 16.8 | −13.1 (0.22×) | 5.4 vs crypto-bigint 39.0 | −33.6 (0.14×) | 5.9 vs crypto-bigint 19.2 | −13.3 (0.31×) |
| mul, constant time | Goldilocks | 11.8 vs winter-math 12.4 | −0.6 (0.95×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 13.7 vs winter-math 16.1 | −2.4 (0.85×) | 6.8 vs winter-math 6.9 | −0.1 (0.99×) |
| mul, constant time | BN254 scalar | 240 vs halo2curves 378 | −138 (0.63×) | 85.1 vs halo2curves 90.5 | −5.4 (0.94×) | 406 vs halo2curves 694 | −288 (0.59×) | 147 vs halo2curves 173 | −26.0 (0.85×) |
| mul, constant time | secp256k1 base | 160 vs halo2curves 307 | −147 (0.52×) | 58.9 vs halo2curves 86.5 | −27.6 (0.68×) | 221 vs halo2curves 574 | −353 (0.39×) | 105 vs halo2curves 154 | −49.0 (0.68×) |
| mul, constant time | P-256 base | 178 vs k256/p256 244 | −66.0 (0.73×) | 62.9 vs halo2curves 70.0 | −7.1 (0.90×) | 271 vs k256/p256 448 | −177 (0.60×) | 119 vs k256/p256 126 | −7.0 (0.94×) |
| mul, constant time | P-256 scalar | 241 vs halo2curves 350 | −109 (0.69×) | 87.4 vs halo2curves 91.7 | −4.3 (0.95×) | 314 vs halo2curves 675 | −361 (0.47×) | 158 vs halo2curves 171 | −13.0 (0.92×) |
| mul, constant time | Tom-256 base | 243 vs halo2curves 342 | −99.0 (0.71×) | 85.8 vs halo2curves 90.5 | −4.7 (0.95×) | 316 vs halo2curves 601 | −285 (0.53×) | 151 vs halo2curves 166 | −15.0 (0.91×) |
| mul, constant time | BLS12-381 base | 715 vs halo2curves 886 | −171 (0.81×) | 185 vs halo2curves 190 | −5.0 (0.97×) | 888 vs crypto-bigint 1,307 | −419 (0.68×) | 331 vs halo2curves 369 | −38.0 (0.90×) |
| mul, constant time | P-521 base | 668 vs crypto-bigint 2,118 | −1,450 (0.32×) | 244 vs crypto-bigint 598 | −354 (0.41×) | 1,321 vs crypto-bigint 2,733 | −1,412 (0.48×) | 432 vs crypto-bigint 1,053 | −621 (0.41×) |
| square, constant time | Mersenne-31 | 3.6 vs crypto-bigint 29.5 | −25.9 (0.12×) | 3.6 vs crypto-bigint 16.3 | −12.7 (0.22×) | 3.4 vs crypto-bigint 38.4 | −35.0 (0.09×) | 3.4 vs crypto-bigint 18.3 | −14.9 (0.19×) |
| square, constant time | BabyBear | 5.0 vs crypto-bigint 29.6 | −24.6 (0.17×) | 3.4 vs crypto-bigint 16.4 | −13.0 (0.21×) | 5.4 vs crypto-bigint 38.1 | −32.7 (0.14×) | 4.6 vs crypto-bigint 18.1 | −13.5 (0.25×) |
| square, constant time | Goldilocks | 9.5 vs winter-math 11.9 | −2.4 (0.80×) | 6.0 vs winter-math 6.0 | +0.0 (1.00×) | 12.9 vs winter-math 16.0 | −3.1 (0.81×) | 6.7 vs winter-math 6.8 | −0.1 (0.99×) |
| square, constant time | BN254 scalar | 223 vs halo2curves 305 | −82.0 (0.73×) | 76.9 vs halo2curves 84.1 | −7.2 (0.91×) | 296 vs halo2curves 547 | −251 (0.54×) | 132 vs halo2curves 143 | −11.0 (0.92×) |
| square, constant time | secp256k1 base | 142 vs halo2curves 244 | −102 (0.58×) | 47.5 vs halo2curves 78.2 | −30.7 (0.61×) | 189 vs halo2curves 449 | −260 (0.42×) | 85.5 vs halo2curves 126 | −40.5 (0.68×) |
| square, constant time | P-256 base | 154 vs halo2curves 186 | −32.0 (0.83×) | 55.3 vs halo2curves 58.2 | −2.9 (0.95×) | 234 vs halo2curves 334 | −100 (0.70×) | 103 vs halo2curves 110 | −7.0 (0.94×) |
| square, constant time | P-256 scalar | 218 vs halo2curves 286 | −68.0 (0.76×) | 79.1 vs halo2curves 82.1 | −3.0 (0.96×) | 292 vs halo2curves 542 | −250 (0.54×) | 135 vs halo2curves 143 | −8.0 (0.94×) |
| square, constant time | Tom-256 base | 210 vs halo2curves 280 | −70.0 (0.75×) | 74.1 vs halo2curves 79.4 | −5.3 (0.93×) | 273 vs halo2curves 478 | −205 (0.57×) | 129 vs halo2curves 137 | −8.0 (0.94×) |
| square, constant time | BLS12-381 base | 554 vs halo2curves 691 | −137 (0.80×) | 157 vs halo2curves 162 | −5.0 (0.97×) | 815 vs halo2curves 1,290 | −475 (0.63×) | 317 vs halo2curves 308 | +9.0 (1.03×) |
| square, constant time | P-521 base | 559 vs crypto-bigint 2,142 | −1,583 (0.26×) | 204 vs crypto-bigint 612 | −408 (0.33×) | 917 vs crypto-bigint 2,811 | −1,894 (0.33×) | 381 vs crypto-bigint 1,101 | −720 (0.35×) |
| invert, constant time | Mersenne-31 | 209 vs runtime-algebra 3,309 | −3,100 (0.06×) | 209 vs runtime-algebra 3,308 | −3,099 (0.06×) | 211 vs runtime-algebra 3,316 | −3,105 (0.06×) | 209 vs runtime-algebra 3,345 | −3,136 (0.06×) |
| invert, constant time | BabyBear | 449 vs runtime-algebra 3,306 | −2,857 (0.14×) | 351 vs runtime-algebra 3,306 | −2,955 (0.11×) | 485 vs runtime-algebra 3,313 | −2,828 (0.15×) | 415 vs runtime-algebra 3,322 | −2,907 (0.12×) |
| invert, constant time | Goldilocks | 1,090 vs winter-math 1,770 | −680 (0.62×) | 775 vs winter-math 900 | −125 (0.86×) | 1,346 vs winter-math 1,821 | −475 (0.74×) | 897 vs winter-math 1,000 | −103 (0.90×) |
| invert, constant time | BN254 scalar | 11,892 vs crypto-bigint 28,698 | −16,806 (0.41×) | 9,425 vs crypto-bigint 25,188 | −15,763 (0.37×) | 16,902 vs crypto-bigint 36,162 | −19,260 (0.47×) | 12,573 vs crypto-bigint 30,534 | −17,961 (0.41×) |
| invert, constant time | secp256k1 base | 11,779 vs crypto-bigint 28,968 | −17,189 (0.41×) | 8,956 vs crypto-bigint 25,192 | −16,236 (0.36×) | 16,561 vs crypto-bigint 36,193 | −19,632 (0.46×) | 12,676 vs crypto-bigint 30,471 | −17,795 (0.42×) |
| invert, constant time | P-256 base | 11,536 vs crypto-bigint 29,131 | −17,595 (0.40×) | 9,029 vs k256/p256 22,163 | −13,134 (0.41×) | 16,390 vs crypto-bigint 36,171 | −19,781 (0.45×) | 12,280 vs crypto-bigint 30,506 | −18,226 (0.40×) |
| invert, constant time | P-256 scalar | 11,874 vs crypto-bigint 28,609 | −16,735 (0.42×) | 9,414 vs crypto-bigint 25,178 | −15,764 (0.37×) | 17,018 vs crypto-bigint 36,211 | −19,193 (0.47×) | 12,825 vs crypto-bigint 30,510 | −17,685 (0.42×) |
| invert, constant time | Tom-256 base | 11,864 vs crypto-bigint 28,813 | −16,949 (0.41×) | 9,191 vs crypto-bigint 25,177 | −15,986 (0.37×) | 16,644 vs crypto-bigint 36,156 | −19,512 (0.46×) | 12,891 vs crypto-bigint 30,479 | −17,588 (0.42×) |
| invert, constant time | BLS12-381 base | 23,941 vs crypto-bigint 49,228 | −25,287 (0.49×) | 16,479 vs crypto-bigint 42,718 | −26,239 (0.39×) | 33,467 vs crypto-bigint 66,446 | −32,979 (0.50×) | 24,433 vs crypto-bigint 53,228 | −28,795 (0.46×) |
| invert, constant time | P-521 base | 41,931 vs crypto-bigint 102,511 | −60,580 (0.41×) | 25,872 vs crypto-bigint 83,918 | −58,046 (0.31×) | 63,129 vs crypto-bigint 127,660 | −64,531 (0.49×) | 41,698 vs crypto-bigint 102,847 | −61,149 (0.41×) |
| sqrt, constant time | Mersenne-31 | 159 vs runtime-algebra 1,734 | −1,575 (0.09×) | 159 vs crypto-bigint 1,646 | −1,487 (0.10×) | 170 vs runtime-algebra 1,627 | −1,457 (0.10×) | 169 vs runtime-algebra 1,738 | −1,569 (0.10×) |
| sqrt, constant time | BabyBear | 6,226 vs crypto-bigint 34,101 | −27,875 (0.18×) | 6,351 vs crypto-bigint 20,204 | −13,853 (0.31×) | 7,424 vs crypto-bigint 38,362 | −30,938 (0.19×) | 9,099 vs crypto-bigint 23,049 | −13,950 (0.39×) |
| sqrt, constant time | Goldilocks | 10,899 vs crypto-bigint 48,688 | −37,789 (0.22×) | 8,281 vs crypto-bigint 28,833 | −20,552 (0.29×) | 14,075 vs crypto-bigint 54,504 | −40,429 (0.26×) | 10,357 vs crypto-bigint 32,891 | −22,534 (0.31×) |
| sqrt, constant time | BN254 scalar | 179,977 vs crypto-bigint 331,012 | −151,035 (0.54×) | 74,303 vs halo2curves 102,199 | −27,896 (0.73×) | 292,908 vs halo2curves 521,306 | −228,398 (0.56×) | 136,322 vs halo2curves 181,448 | −45,126 (0.75×) |
| sqrt, constant time | secp256k1 base | 40,950 vs halo2curves 158,360 | −117,410 (0.26×) | 16,311 vs halo2curves 43,540 | −27,229 (0.37×) | 64,212 vs crypto-bigint 247,476 | −183,264 (0.26×) | 26,969 vs crypto-bigint 79,772 | −52,803 (0.34×) |
| sqrt, constant time | P-256 base | 43,215 vs k256/p256 64,672 | −21,457 (0.67×) | 17,364 vs k256/p256 21,613 | −4,249 (0.80×) | 78,056 vs k256/p256 119,979 | −41,923 (0.65×) | 31,002 vs k256/p256 33,709 | −2,707 (0.92×) |
| sqrt, constant time | P-256 scalar | 77,702 vs halo2curves 151,225 | −73,523 (0.51×) | 31,388 vs halo2curves 49,879 | −18,491 (0.63×) | 114,043 vs crypto-bigint 243,430 | −129,387 (0.47×) | 60,595 vs halo2curves 72,406 | −11,811 (0.84×) |
| sqrt, constant time | Tom-256 base | 65,822 vs crypto-bigint 151,019 | −85,197 (0.44×) | 26,921 vs halo2curves 45,925 | −19,004 (0.59×) | 101,342 vs crypto-bigint 224,942 | −123,600 (0.45×) | 49,422 vs crypto-bigint 74,780 | −25,358 (0.66×) |
| sqrt, constant time | BLS12-381 base | 296,743 vs crypto-bigint 551,113 | −254,370 (0.54×) | 90,254 vs crypto-bigint 145,150 | −54,896 (0.62×) | 390,900 vs crypto-bigint 627,471 | −236,571 (0.62×) | 213,009 vs crypto-bigint 268,956 | −55,947 (0.79×) |
| sqrt, constant time | P-521 base | 320,738 vs crypto-bigint 1,526,546 | −1,205,808 (0.21×) | 124,536 vs crypto-bigint 296,852 | −172,316 (0.42×) | 466,219 vs crypto-bigint 1,574,170 | −1,107,951 (0.30×) | 256,351 vs crypto-bigint 591,157 | −334,806 (0.43×) |
| mul_u64, constant time | Mersenne-31 | 7.4 vs crypto-bigint 63.6 | −56.2 (0.12×) | 7.4 vs crypto-bigint 33.5 | −26.1 (0.22×) | 7.6 vs crypto-bigint 70.2 | −62.6 (0.11×) | 7.6 vs crypto-bigint 39.5 | −31.9 (0.19×) |
| mul_u64, constant time | BabyBear | 15.5 vs crypto-bigint 69.5 | −54.0 (0.22×) | 11.4 vs crypto-bigint 32.9 | −21.5 (0.35×) | 17.5 vs crypto-bigint 84.6 | −67.1 (0.21×) | 15.0 vs crypto-bigint 41.4 | −26.4 (0.36×) |
| mul_u64, constant time | Goldilocks | 18.7 vs winter-math 24.9 | −6.2 (0.75×) | 7.1 vs winter-math 11.5 | −4.4 (0.62×) | 26.3 vs winter-math 31.1 | −4.8 (0.85×) | 8.7 vs winter-math 13.8 | −5.1 (0.63×) |
| mul_u64, constant time | BN254 scalar | 106 vs crypto-bigint 860 | −754 (0.12×) | 38.9 vs crypto-bigint 262 | −223 (0.15×) | 124 vs halo2curves 1,436 | −1,312 (0.09×) | 55.3 vs crypto-bigint 407 | −352 (0.14×) |
| mul_u64, constant time | secp256k1 base | 60.3 vs halo2curves 698 | −638 (0.09×) | 22.2 vs crypto-bigint 262 | −240 (0.08×) | 77.1 vs halo2curves 1,028 | −951 (0.07×) | 35.4 vs halo2curves 373 | −338 (0.09×) |
| mul_u64, constant time | P-256 base | 82.9 vs k256/p256 488 | −405 (0.17×) | 38.1 vs k256/p256 199 | −161 (0.19×) | 99.9 vs k256/p256 911 | −811 (0.11×) | 54.6 vs k256/p256 256 | −201 (0.21×) |
| mul_u64, constant time | P-256 scalar | 97.1 vs k256/p256 511 | −414 (0.19×) | 41.7 vs crypto-bigint 261 | −219 (0.16×) | 117 vs k256/p256 824 | −707 (0.14×) | 60.6 vs k256/p256 337 | −276 (0.18×) |
| mul_u64, constant time | Tom-256 base | 101 vs crypto-bigint 848 | −747 (0.12×) | 41.7 vs crypto-bigint 261 | −219 (0.16×) | 120 vs halo2curves 1,287 | −1,167 (0.09×) | 59.7 vs crypto-bigint 407 | −347 (0.15×) |
| mul_u64, constant time | BLS12-381 base | 150 vs crypto-bigint 1,919 | −1,769 (0.08×) | 63.3 vs crypto-bigint 512 | −449 (0.12×) | 192 vs crypto-bigint 2,504 | −2,312 (0.08×) | 82.6 vs crypto-bigint 967 | −884 (0.09×) |
| mul_u64, constant time | P-521 base | 138 vs crypto-bigint 3,017 | −2,879 (0.05×) | 64.3 vs crypto-bigint 988 | −924 (0.07×) | 172 vs crypto-bigint 3,980 | −3,808 (0.04×) | 88.4 vs crypto-bigint 1,743 | −1,655 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 7.4 vs crypto-bigint 63.7 | −56.3 (0.12×) | 7.4 vs crypto-bigint 33.5 | −26.1 (0.22×) | 7.6 vs crypto-bigint 70.3 | −62.7 (0.11×) | 7.6 vs crypto-bigint 39.5 | −31.9 (0.19×) |
| mul_uint, constant time | BabyBear | 15.5 vs crypto-bigint 69.5 | −54.0 (0.22×) | 11.4 vs crypto-bigint 32.9 | −21.5 (0.35×) | 17.5 vs crypto-bigint 82.3 | −64.8 (0.21×) | 15.0 vs crypto-bigint 41.1 | −26.1 (0.36×) |
| mul_uint, constant time | Goldilocks | 18.7 vs winter-math 24.9 | −6.2 (0.75×) | 7.1 vs winter-math 11.5 | −4.4 (0.62×) | 26.3 vs winter-math 31.1 | −4.8 (0.85×) | 8.7 vs winter-math 13.8 | −5.1 (0.63×) |
| mul_uint, constant time | BN254 scalar | 479 vs halo2curves 765 | −286 (0.63×) | 180 vs halo2curves 181 | −1.0 (0.99×) | 598 vs halo2curves 1,297 | −699 (0.46×) | 296 vs halo2curves 361 | −65.0 (0.82×) |
| mul_uint, constant time | secp256k1 base | 316 vs halo2curves 518 | −202 (0.61×) | 68.1 vs halo2curves 154 | −85.9 (0.44×) | 387 vs halo2curves 950 | −563 (0.41×) | 115 vs halo2curves 264 | −149 (0.44×) |
| mul_uint, constant time | P-256 base | 405 vs halo2curves 462 | −57.0 (0.88×) | 125 vs halo2curves 142 | −17.0 (0.88×) | 488 vs halo2curves 995 | −507 (0.49×) | 231 vs halo2curves 294 | −63.0 (0.79×) |
| mul_uint, constant time | P-256 scalar | 464 vs k256/p256 563 | −99.0 (0.82×) | 190 vs halo2curves 185 | +5.0 (1.03×) | 555 vs k256/p256 884 | −329 (0.63×) | 308 vs halo2curves 345 | −37.0 (0.89×) |
| mul_uint, constant time | Tom-256 base | 464 vs halo2curves 694 | −230 (0.67×) | 181 vs halo2curves 180 | +1.0 (1.01×) | 556 vs halo2curves 1,186 | −630 (0.47×) | 305 vs halo2curves 355 | −50.0 (0.86×) |
| mul_uint, constant time | BLS12-381 base | 1,067 vs crypto-bigint 1,883 | −816 (0.57×) | 378 vs halo2curves 384 | −6.0 (0.98×) | 1,679 vs crypto-bigint 2,499 | −820 (0.67×) | 920 vs crypto-bigint 948 | −28.0 (0.97×) |
| mul_uint, constant time | P-521 base | 1,983 vs crypto-bigint 2,984 | −1,001 (0.66×) | 290 vs crypto-bigint 982 | −692 (0.30×) | 2,889 vs crypto-bigint 4,033 | −1,144 (0.72×) | 580 vs crypto-bigint 1,737 | −1,157 (0.33×) |
| add_u64, constant time | Mersenne-31 | 5.4 vs crypto-bigint 37.0 | −31.6 (0.15×) | 5.4 vs crypto-bigint 29.3 | −23.9 (0.18×) | 5.5 vs crypto-bigint 40.8 | −35.3 (0.13×) | 5.5 vs crypto-bigint 29.6 | −24.1 (0.19×) |
| add_u64, constant time | BabyBear | 11.9 vs crypto-bigint 40.8 | −28.9 (0.29×) | 7.4 vs crypto-bigint 31.5 | −24.1 (0.23×) | 14.1 vs crypto-bigint 52.4 | −38.3 (0.27×) | 11.4 vs crypto-bigint 31.5 | −20.1 (0.36×) |
| add_u64, constant time | Goldilocks | 9.3 vs winter-math 13.6 | −4.3 (0.68×) | 7.5 vs winter-math 8.1 | −0.6 (0.93×) | 11.8 vs winter-math 18.2 | −6.4 (0.65×) | 8.5 vs winter-math 9.8 | −1.3 (0.87×) |
| add_u64, constant time | BN254 scalar | 92.3 vs crypto-bigint 501 | −409 (0.18×) | 41.3 vs crypto-bigint 223 | −182 (0.19×) | 111 vs halo2curves 749 | −638 (0.15×) | 68.5 vs halo2curves 295 | −227 (0.23×) |
| add_u64, constant time | secp256k1 base | 22.5 vs halo2curves 401 | −379 (0.06×) | 14.6 vs crypto-bigint 224 | −209 (0.07×) | 21.1 vs halo2curves 480 | −459 (0.04×) | 23.1 vs halo2curves 243 | −220 (0.10×) |
| add_u64, constant time | P-256 base | 71.8 vs k256/p256 276 | −204 (0.26×) | 36.7 vs k256/p256 113 | −76.3 (0.32×) | 78.0 vs k256/p256 488 | −410 (0.16×) | 57.1 vs k256/p256 157 | −99.9 (0.36×) |
| add_u64, constant time | P-256 scalar | 93.1 vs k256/p256 46.4 | +46.7 (2.01×) | 41.9 vs k256/p256 46.3 | −4.4 (0.90×) | 112 vs k256/p256 54.5 | +57.5 (2.06×) | 71.7 vs k256/p256 54.6 | +17.1 (1.31×) |
| add_u64, constant time | Tom-256 base | 97.1 vs crypto-bigint 502 | −405 (0.19×) | 42.0 vs crypto-bigint 224 | −182 (0.19×) | 110 vs halo2curves 701 | −591 (0.16×) | 70.3 vs halo2curves 289 | −219 (0.24×) |
| add_u64, constant time | BLS12-381 base | 150 vs crypto-bigint 1,084 | −934 (0.14×) | 64.3 vs halo2curves 360 | −296 (0.18×) | 172 vs crypto-bigint 1,374 | −1,202 (0.13×) | 106 vs crypto-bigint 640 | −534 (0.17×) |
| add_u64, constant time | P-521 base | 64.2 vs crypto-bigint 1,673 | −1,609 (0.04×) | 38.4 vs crypto-bigint 649 | −611 (0.06×) | 69.1 vs crypto-bigint 2,187 | −2,118 (0.03×) | 61.1 vs crypto-bigint 1,157 | −1,096 (0.05×) |
| add_uint, constant time | Mersenne-31 | 5.4 vs crypto-bigint 37.0 | −31.6 (0.15×) | 5.4 vs crypto-bigint 29.0 | −23.6 (0.19×) | 5.5 vs crypto-bigint 39.5 | −34.0 (0.14×) | 5.5 vs crypto-bigint 29.8 | −24.3 (0.18×) |
| add_uint, constant time | BabyBear | 11.9 vs crypto-bigint 40.8 | −28.9 (0.29×) | 7.4 vs crypto-bigint 31.4 | −24.0 (0.24×) | 14.1 vs crypto-bigint 52.3 | −38.2 (0.27×) | 11.4 vs crypto-bigint 31.4 | −20.0 (0.36×) |
| add_uint, constant time | Goldilocks | 9.3 vs winter-math 13.6 | −4.3 (0.68×) | 7.5 vs winter-math 8.1 | −0.6 (0.93×) | 11.8 vs winter-math 18.2 | −6.4 (0.65×) | 8.5 vs winter-math 9.8 | −1.3 (0.87×) |
| add_uint, constant time | BN254 scalar | 248 vs halo2curves 419 | −171 (0.59×) | 101 vs halo2curves 111 | −10.0 (0.91×) | 331 vs halo2curves 668 | −337 (0.50×) | 177 vs halo2curves 204 | −27.0 (0.87×) |
| add_uint, constant time | secp256k1 base | 40.1 vs halo2curves 224 | −184 (0.18×) | 27.1 vs halo2curves 86.0 | −58.9 (0.32×) | 37.6 vs halo2curves 410 | −372 (0.09×) | 40.5 vs halo2curves 139 | −98.5 (0.29×) |
| add_uint, constant time | P-256 base | 179 vs halo2curves 248 | −69.0 (0.72×) | 77.8 vs halo2curves 90.0 | −12.2 (0.86×) | 281 vs halo2curves 504 | −223 (0.56×) | 138 vs halo2curves 152 | −14.0 (0.91×) |
| add_uint, constant time | P-256 scalar | 261 vs k256/p256 84.7 | +176 (3.08×) | 104 vs k256/p256 84.2 | +19.8 (1.24×) | 324 vs k256/p256 115 | +209 (2.82×) | 186 vs k256/p256 115 | +71.0 (1.62×) |
| add_uint, constant time | Tom-256 base | 257 vs halo2curves 376 | −119 (0.68×) | 106 vs halo2curves 111 | −5.0 (0.95×) | 327 vs halo2curves 619 | −292 (0.53×) | 175 vs halo2curves 195 | −20.0 (0.90×) |
| add_uint, constant time | BLS12-381 base | 764 vs halo2curves 961 | −197 (0.80×) | 208 vs halo2curves 222 | −14.0 (0.94×) | 952 vs crypto-bigint 1,364 | −412 (0.70×) | 379 vs halo2curves 422 | −43.0 (0.90×) |
| add_uint, constant time | P-521 base | 142 vs crypto-bigint 1,673 | −1,531 (0.08×) | 82.0 vs crypto-bigint 641 | −559 (0.13×) | 154 vs crypto-bigint 2,218 | −2,064 (0.07×) | 129 vs crypto-bigint 1,155 | −1,026 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 5.4 vs crypto-bigint 24.5 | −19.1 (0.22×) | 5.4 vs crypto-bigint 18.1 | −12.7 (0.30×) | 5.4 vs crypto-bigint 30.6 | −25.2 (0.18×) | 5.2 vs crypto-bigint 20.3 | −15.1 (0.26×) |
| sub_u64, constant time | BabyBear | 10.9 vs crypto-bigint 29.5 | −18.6 (0.37×) | 7.3 vs crypto-bigint 19.7 | −12.4 (0.37×) | 13.0 vs crypto-bigint 41.1 | −28.1 (0.32×) | 11.2 vs crypto-bigint 21.1 | −9.9 (0.53×) |
| sub_u64, constant time | Goldilocks | 9.4 vs winter-math 13.1 | −3.7 (0.72×) | 7.4 vs winter-math 7.4 | +0.0 (1.00×) | 11.8 vs winter-math 17.6 | −5.8 (0.67×) | 8.5 vs winter-math 8.6 | −0.1 (0.99×) |
| sub_u64, constant time | BN254 scalar | 92.6 vs crypto-bigint 490 | −397 (0.19×) | 39.8 vs crypto-bigint 151 | −111 (0.26×) | 111 vs halo2curves 737 | −626 (0.15×) | 66.0 vs crypto-bigint 231 | −165 (0.29×) |
| sub_u64, constant time | secp256k1 base | 16.5 vs halo2curves 388 | −372 (0.04×) | 11.9 vs crypto-bigint 150 | −138 (0.08×) | 15.4 vs halo2curves 464 | −449 (0.03×) | 17.0 vs crypto-bigint 231 | −214 (0.07×) |
| sub_u64, constant time | P-256 base | 63.8 vs k256/p256 265 | −201 (0.24×) | 33.2 vs k256/p256 99.3 | −66.1 (0.33×) | 74.1 vs k256/p256 466 | −392 (0.16×) | 52.0 vs k256/p256 147 | −95.0 (0.35×) |
| sub_u64, constant time | P-256 scalar | 96.1 vs k256/p256 22.2 | +73.9 (4.33×) | 38.8 vs k256/p256 22.0 | +16.8 (1.76×) | 108 vs k256/p256 23.7 | +84.3 (4.56×) | 66.3 vs k256/p256 23.8 | +42.5 (2.79×) |
| sub_u64, constant time | Tom-256 base | 92.6 vs crypto-bigint 497 | −404 (0.19×) | 41.9 vs crypto-bigint 151 | −109 (0.28×) | 106 vs halo2curves 683 | −577 (0.16×) | 64.3 vs crypto-bigint 231 | −167 (0.28×) |
| sub_u64, constant time | BLS12-381 base | 144 vs crypto-bigint 1,077 | −933 (0.13×) | 62.9 vs halo2curves 345 | −282 (0.18×) | 185 vs crypto-bigint 1,369 | −1,184 (0.14×) | 103 vs halo2curves 568 | −465 (0.18×) |
| sub_u64, constant time | P-521 base | 43.2 vs crypto-bigint 1,666 | −1,623 (0.03×) | 27.9 vs crypto-bigint 621 | −593 (0.04×) | 52.6 vs crypto-bigint 2,159 | −2,106 (0.02×) | 39.1 vs crypto-bigint 1,145 | −1,106 (0.03×) |
| sub_uint, constant time | Mersenne-31 | 5.4 vs crypto-bigint 24.5 | −19.1 (0.22×) | 5.4 vs crypto-bigint 18.1 | −12.7 (0.30×) | 5.4 vs crypto-bigint 30.2 | −24.8 (0.18×) | 5.2 vs crypto-bigint 20.3 | −15.1 (0.26×) |
| sub_uint, constant time | BabyBear | 10.9 vs crypto-bigint 29.5 | −18.6 (0.37×) | 7.3 vs crypto-bigint 19.6 | −12.3 (0.37×) | 13.1 vs crypto-bigint 41.1 | −28.0 (0.32×) | 11.2 vs crypto-bigint 21.2 | −10.0 (0.53×) |
| sub_uint, constant time | Goldilocks | 9.4 vs winter-math 13.1 | −3.7 (0.72×) | 7.3 vs winter-math 7.4 | −0.1 (0.99×) | 11.8 vs winter-math 17.6 | −5.8 (0.67×) | 8.5 vs winter-math 8.6 | −0.1 (0.99×) |
| sub_uint, constant time | BN254 scalar | 249 vs halo2curves 397 | −148 (0.63×) | 99.4 vs halo2curves 105 | −5.6 (0.95×) | 329 vs halo2curves 657 | −328 (0.50×) | 173 vs halo2curves 193 | −20.0 (0.90×) |
| sub_uint, constant time | secp256k1 base | 34.1 vs halo2curves 210 | −176 (0.16×) | 23.5 vs halo2curves 80.3 | −56.8 (0.29×) | 32.5 vs halo2curves 397 | −365 (0.08×) | 35.7 vs halo2curves 128 | −92.3 (0.28×) |
| sub_uint, constant time | P-256 base | 174 vs halo2curves 233 | −59.0 (0.75×) | 74.2 vs halo2curves 84.8 | −10.6 (0.88×) | 280 vs halo2curves 490 | −210 (0.57×) | 133 vs halo2curves 142 | −9.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 247 vs k256/p256 70.7 | +176 (3.49×) | 101 vs k256/p256 70.0 | +31.0 (1.44×) | 320 vs k256/p256 83.5 | +237 (3.83×) | 174 vs k256/p256 75.2 | +98.8 (2.31×) |
| sub_uint, constant time | Tom-256 base | 251 vs halo2curves 366 | −115 (0.69×) | 103 vs halo2curves 103 | +0.0 (1.00×) | 323 vs halo2curves 613 | −290 (0.53×) | 171 vs halo2curves 185 | −14.0 (0.92×) |
| sub_uint, constant time | BLS12-381 base | 763 vs halo2curves 944 | −181 (0.81×) | 203 vs halo2curves 214 | −11.0 (0.95×) | 944 vs crypto-bigint 1,371 | −427 (0.69×) | 373 vs halo2curves 406 | −33.0 (0.92×) |
| sub_uint, constant time | P-521 base | 124 vs crypto-bigint 1,653 | −1,529 (0.08×) | 66.6 vs crypto-bigint 615 | −548 (0.11×) | 121 vs crypto-bigint 2,190 | −2,069 (0.06×) | 107 vs crypto-bigint 1,146 | −1,039 (0.09×) |
| from_u64, constant time | Mersenne-31 | 4.4 vs crypto-bigint 18.4 | −14.0 (0.24×) | 4.4 vs crypto-bigint 12.4 | −8.0 (0.35×) | 5.6 vs crypto-bigint 23.1 | −17.5 (0.24×) | 5.4 vs crypto-bigint 13.1 | −7.7 (0.41×) |
| from_u64, constant time | BabyBear | 11.7 vs crypto-bigint 23.0 | −11.3 (0.51×) | 6.4 vs crypto-bigint 12.9 | −6.5 (0.50×) | 12.6 vs crypto-bigint 33.4 | −20.8 (0.38×) | 9.6 vs crypto-bigint 15.1 | −5.5 (0.64×) |
| from_u64, constant time | Goldilocks | 7.9 vs winter-math 11.2 | −3.3 (0.71×) | 5.6 vs winter-math 5.6 | +0.0 (1.00×) | 8.6 vs winter-math 16.0 | −7.4 (0.54×) | 6.9 vs winter-math 6.9 | +0.0 (1.00×) |
| from_u64, constant time | BN254 scalar | 88.1 vs crypto-bigint 424 | −336 (0.21×) | 35.3 vs crypto-bigint 129 | −93.7 (0.27×) | 98.8 vs halo2curves 703 | −604 (0.14×) | 49.3 vs crypto-bigint 204 | −155 (0.24×) |
| from_u64, constant time | secp256k1 base | 3.3 vs k256/p256 5.3 | −2.0 (0.62×) | 3.3 vs k256/p256 5.3 | −2.0 (0.62×) | 3.2 vs k256/p256 5.0 | −1.8 (0.64×) | 3.2 vs k256/p256 5.0 | −1.8 (0.64×) |
| from_u64, constant time | P-256 base | 56.4 vs k256/p256 240 | −184 (0.23×) | 26.0 vs k256/p256 94.9 | −68.9 (0.27×) | 67.1 vs k256/p256 460 | −393 (0.15×) | 37.1 vs k256/p256 128 | −90.9 (0.29×) |
| from_u64, constant time | P-256 scalar | 86.4 vs k256/p256 3.3 | +83.1 (26.18×) | 36.0 vs k256/p256 3.3 | +32.7 (10.91×) | 108 vs k256/p256 2.9 | +105 (37.24×) | 49.2 vs k256/p256 3.1 | +46.1 (15.87×) |
| from_u64, constant time | Tom-256 base | 79.1 vs crypto-bigint 432 | −353 (0.18×) | 37.1 vs crypto-bigint 129 | −91.9 (0.29×) | 94.1 vs halo2curves 653 | −559 (0.14×) | 48.1 vs crypto-bigint 204 | −156 (0.24×) |
| from_u64, constant time | BLS12-381 base | 140 vs crypto-bigint 917 | −777 (0.15×) | 56.9 vs crypto-bigint 257 | −200 (0.22×) | 170 vs crypto-bigint 1,245 | −1,075 (0.14×) | 79.8 vs halo2curves 468 | −388 (0.17×) |
| from_u64, constant time | P-521 base | 6.0 vs crypto-bigint 1,448 | −1,442 (0.00×) | 6.0 vs crypto-bigint 481 | −475 (0.01×) | 6.0 vs crypto-bigint 1,980 | −1,974 (0.00×) | 6.0 vs crypto-bigint 844 | −838 (0.01×) |
| div_u64, constant time | BN254 scalar | 12,195 vs crypto-bigint 28,198 | −16,003 (0.43×) | 9,444 vs crypto-bigint 24,247 | −14,803 (0.39×) | 17,395 vs crypto-bigint 38,049 | −20,654 (0.46×) | 12,955 vs crypto-bigint 30,808 | −17,853 (0.42×) |
| div_u64, constant time | secp256k1 base | 11,907 vs crypto-bigint 28,259 | −16,352 (0.42×) | 9,056 vs crypto-bigint 24,316 | −15,260 (0.37×) | 16,824 vs crypto-bigint 38,093 | −21,269 (0.44×) | 12,825 vs crypto-bigint 30,817 | −17,992 (0.42×) |
| div_u64, constant time | P-256 base | 11,709 vs crypto-bigint 28,329 | −16,620 (0.41×) | 9,122 vs k256/p256 22,943 | −13,821 (0.40×) | 17,079 vs crypto-bigint 38,091 | −21,012 (0.45×) | 12,504 vs crypto-bigint 30,786 | −18,282 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,209 vs crypto-bigint 28,392 | −16,183 (0.43×) | 9,412 vs crypto-bigint 24,386 | −14,974 (0.39×) | 17,601 vs crypto-bigint 37,986 | −20,385 (0.46×) | 13,189 vs crypto-bigint 30,714 | −17,525 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,097 vs crypto-bigint 28,388 | −16,291 (0.43×) | 9,336 vs crypto-bigint 24,259 | −14,923 (0.38×) | 16,838 vs crypto-bigint 38,097 | −21,259 (0.44×) | 12,896 vs crypto-bigint 30,818 | −17,922 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,660 vs crypto-bigint 50,746 | −26,086 (0.49×) | 16,947 vs crypto-bigint 43,256 | −26,309 (0.39×) | 34,592 vs crypto-bigint 68,265 | −33,673 (0.51×) | 24,245 vs crypto-bigint 53,490 | −29,245 (0.45×) |
| div_u64, constant time | P-521 base | 42,042 vs crypto-bigint 106,272 | −64,230 (0.40×) | 25,899 vs crypto-bigint 82,997 | −57,098 (0.31×) | 64,757 vs crypto-bigint 127,918 | −63,161 (0.51×) | 43,412 vs crypto-bigint 100,134 | −56,722 (0.43×) |
| div_uint, constant time | Mersenne-31 | 225 vs runtime-algebra 3,497 | −3,272 (0.06×) | 226 vs runtime-algebra 3,513 | −3,287 (0.06×) | 229 vs runtime-algebra 3,447 | −3,218 (0.07×) | 229 vs runtime-algebra 3,420 | −3,191 (0.07×) |
| div_uint, constant time | BabyBear | 478 vs runtime-algebra 3,548 | −3,070 (0.13×) | 374 vs runtime-algebra 3,547 | −3,173 (0.11×) | 523 vs runtime-algebra 3,447 | −2,924 (0.15×) | 441 vs runtime-algebra 3,420 | −2,979 (0.13×) |
| div_uint, constant time | Goldilocks | 1,121 vs winter-math 1,812 | −691 (0.62×) | 807 vs winter-math 936 | −129 (0.86×) | 1,368 vs winter-math 1,890 | −522 (0.72×) | 952 vs winter-math 1,041 | −89.0 (0.91×) |
| div_uint, constant time | BN254 scalar | 12,459 vs crypto-bigint 28,406 | −15,947 (0.44×) | 9,650 vs crypto-bigint 24,823 | −15,173 (0.39×) | 17,690 vs crypto-bigint 38,027 | −20,337 (0.47×) | 13,299 vs crypto-bigint 30,760 | −17,461 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,920 vs crypto-bigint 28,098 | −16,178 (0.42×) | 9,070 vs crypto-bigint 24,935 | −15,865 (0.36×) | 16,879 vs crypto-bigint 37,968 | −21,089 (0.44×) | 12,848 vs crypto-bigint 30,705 | −17,857 (0.42×) |
| div_uint, constant time | P-256 base | 11,924 vs crypto-bigint 29,412 | −17,488 (0.41×) | 9,148 vs k256/p256 22,643 | −13,495 (0.40×) | 17,392 vs crypto-bigint 38,030 | −20,638 (0.46×) | 12,558 vs crypto-bigint 30,692 | −18,134 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,423 vs crypto-bigint 28,070 | −15,647 (0.44×) | 9,551 vs crypto-bigint 24,795 | −15,244 (0.39×) | 17,828 vs crypto-bigint 37,991 | −20,163 (0.47×) | 13,324 vs crypto-bigint 30,701 | −17,377 (0.43×) |
| div_uint, constant time | Tom-256 base | 12,345 vs crypto-bigint 28,506 | −16,161 (0.43×) | 9,489 vs crypto-bigint 24,779 | −15,290 (0.38×) | 17,040 vs crypto-bigint 38,020 | −20,980 (0.45×) | 13,116 vs crypto-bigint 30,680 | −17,564 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,235 vs crypto-bigint 51,293 | −26,058 (0.49×) | 17,014 vs crypto-bigint 43,408 | −26,394 (0.39×) | 35,308 vs crypto-bigint 68,362 | −33,054 (0.52×) | 24,755 vs crypto-bigint 53,386 | −28,631 (0.46×) |
| div_uint, constant time | P-521 base | 42,376 vs crypto-bigint 106,142 | −63,766 (0.40×) | 26,228 vs crypto-bigint 82,966 | −56,738 (0.32×) | 64,633 vs crypto-bigint 130,050 | −65,417 (0.50×) | 43,327 vs crypto-bigint 100,302 | −56,975 (0.43×) |

### Against the best competitor, latency

Cycles per operation of `prime-field` against the fewest of the crates whose functions keep the row's definition. Δ: ours − best, then ours ÷ best; below 1 we lead.

| Operation | Field | Chrome | Δ | Chrome, wide arithmetic | Δ | Firefox | Δ | Firefox, wide arithmetic | Δ |
|---|---|---|---:|---|---:|---|---:|---|---:|
| add | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.6 | −14.6 (0.17×) | 3.0 vs runtime-algebra 17.9 | −14.9 (0.17×) |
| add | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| add | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add | BN254 scalar | 13.0 vs ark-ff 13.4 | −0.4 (0.97×) | 10.1 vs ark-ff 10.7 | −0.6 (0.94×) | 13.5 vs ark-ff 13.9 | −0.4 (0.97×) | 15.3 vs ark-ff 16.3 | −1.0 (0.94×) |
| add | secp256k1 base | 14.2 vs ark-ff 15.5 | −1.3 (0.92×) | 9.8 vs ark-ff 10.7 | −0.9 (0.92×) | 14.1 vs ark-ff 15.6 | −1.5 (0.90×) | 17.4 vs ark-ff 18.4 | −1.0 (0.95×) |
| add | P-256 base | 12.8 vs ark-ff 16.2 | −3.4 (0.79×) | 9.9 vs ark-ff 11.4 | −1.5 (0.87×) | 13.3 vs ark-ff 16.3 | −3.0 (0.82×) | 17.3 vs ark-ff 18.2 | −0.9 (0.95×) |
| add | P-256 scalar | 13.9 vs ark-ff 16.1 | −2.2 (0.86×) | 10.9 vs ark-ff 11.2 | −0.3 (0.97×) | 13.7 vs ark-ff 15.9 | −2.2 (0.86×) | 17.9 vs ark-ff 17.8 | +0.1 (1.01×) |
| add | Tom-256 base | 13.3 vs halo2curves 41.1 | −27.8 (0.32×) | 10.7 vs halo2curves 24.7 | −14.0 (0.43×) | 13.4 vs halo2curves 42.5 | −29.1 (0.32×) | 16.4 vs halo2curves 32.6 | −16.2 (0.50×) |
| add | BLS12-381 base | 22.1 vs ark-ff 43.4 | −21.3 (0.51×) | 15.9 vs halo2curves 34.1 | −18.2 (0.47×) | 22.1 vs ark-ff 45.9 | −23.8 (0.48×) | 25.9 vs ark-ff 44.4 | −18.5 (0.58×) |
| add | P-521 base | 41.7 vs crypto-bigint 182 | −140 (0.23×) | 24.0 vs crypto-bigint 179 | −155 (0.13×) | 49.7 vs crypto-bigint 245 | −195 (0.20×) | 48.9 vs crypto-bigint 243 | −194 (0.20×) |
| sub | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub | BabyBear | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) | 3.0 vs plonky3 3.0 | +0.0 (1.00×) |
| sub | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub | BN254 scalar | 14.0 vs ark-ff 16.6 | −2.6 (0.84×) | 11.3 vs ark-ff 14.5 | −3.2 (0.78×) | 12.3 vs ark-ff 16.5 | −4.2 (0.75×) | 16.3 vs ark-ff 21.9 | −5.6 (0.74×) |
| sub | secp256k1 base | 15.8 vs ark-ff 17.6 | −1.8 (0.90×) | 11.2 vs ark-ff 14.4 | −3.2 (0.78×) | 13.5 vs ark-ff 17.4 | −3.9 (0.78×) | 17.8 vs halo2curves 21.0 | −3.2 (0.85×) |
| sub | P-256 base | 13.4 vs ark-ff 15.3 | −1.9 (0.88×) | 10.3 vs ark-ff 13.4 | −3.1 (0.77×) | 12.4 vs ark-ff 15.6 | −3.2 (0.79×) | 16.3 vs halo2curves 19.5 | −3.2 (0.84×) |
| sub | P-256 scalar | 14.9 vs ark-ff 16.5 | −1.6 (0.90×) | 11.2 vs ark-ff 14.9 | −3.7 (0.75×) | 13.5 vs ark-ff 16.9 | −3.4 (0.80×) | 17.4 vs halo2curves 22.1 | −4.7 (0.79×) |
| sub | Tom-256 base | 14.0 vs halo2curves 28.9 | −14.9 (0.48×) | 11.3 vs halo2curves 15.7 | −4.4 (0.72×) | 12.5 vs halo2curves 26.5 | −14.0 (0.47×) | 16.4 vs halo2curves 22.0 | −5.6 (0.75×) |
| sub | BLS12-381 base | 20.9 vs ark-ff 30.6 | −9.7 (0.68×) | 18.8 vs halo2curves 24.5 | −5.7 (0.77×) | 19.9 vs ark-ff 31.3 | −11.4 (0.64×) | 28.2 vs halo2curves 36.1 | −7.9 (0.78×) |
| sub | P-521 base | 42.8 vs crypto-bigint 161 | −118 (0.27×) | 25.5 vs crypto-bigint 145 | −120 (0.18×) | 41.9 vs crypto-bigint 195 | −153 (0.21×) | 43.8 vs crypto-bigint 173 | −129 (0.25×) |
| mul | Mersenne-31 | 7.0 vs crypto-bigint 45.7 | −38.7 (0.15×) | 7.0 vs crypto-bigint 26.8 | −19.8 (0.26×) | 7.0 vs crypto-bigint 48.9 | −41.9 (0.14×) | 7.0 vs crypto-bigint 30.6 | −23.6 (0.23×) |
| mul | BabyBear | 13.0 vs plonky3 12.2 | +0.8 (1.07×) | 11.0 vs plonky3 12.2 | −1.2 (0.90×) | 14.0 vs plonky3 13.0 | +1.0 (1.08×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) |
| mul | Goldilocks | 16.5 vs winter-math 18.1 | −1.6 (0.91×) | 12.7 vs winter-math 12.7 | +0.0 (1.00×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul | BN254 scalar | 222 vs ark-ff 316 | −94.0 (0.70×) | 91.7 vs halo2curves 93.6 | −1.9 (0.98×) | 367 vs ark-ff 393 | −26.0 (0.93×) | 149 vs halo2curves 178 | −29.0 (0.84×) |
| mul | secp256k1 base | 170 vs ark-ff 276 | −106 (0.62×) | 67.1 vs halo2curves 89.7 | −22.6 (0.75×) | 237 vs ark-ff 403 | −166 (0.59×) | 105 vs halo2curves 158 | −53.0 (0.66×) |
| mul | P-256 base | 184 vs k256/p256 242 | −58.0 (0.76×) | 62.2 vs halo2curves 74.3 | −12.1 (0.84×) | 294 vs k256/p256 461 | −167 (0.64×) | 118 vs halo2curves 130 | −12.0 (0.91×) |
| mul | P-256 scalar | 239 vs ark-ff 312 | −73.0 (0.77×) | 91.0 vs halo2curves 95.4 | −4.4 (0.95×) | 325 vs ark-ff 406 | −81.0 (0.80×) | 157 vs halo2curves 176 | −19.0 (0.89×) |
| mul | Tom-256 base | 235 vs halo2curves 343 | −108 (0.69×) | 87.9 vs halo2curves 93.2 | −5.3 (0.94×) | 325 vs halo2curves 600 | −275 (0.54×) | 152 vs halo2curves 171 | −19.0 (0.89×) |
| mul | BLS12-381 base | 716 vs ark-ff 781 | −65.0 (0.92×) | 185 vs halo2curves 193 | −8.0 (0.96×) | 880 vs ark-ff 1,026 | −146 (0.86×) | 335 vs halo2curves 377 | −42.0 (0.89×) |
| mul | P-521 base | 688 vs crypto-bigint 2,141 | −1,453 (0.32×) | 240 vs crypto-bigint 619 | −379 (0.39×) | 1,291 vs crypto-bigint 2,773 | −1,482 (0.47×) | 440 vs crypto-bigint 1,104 | −664 (0.40×) |
| square | Mersenne-31 | 7.0 vs crypto-bigint 44.2 | −37.2 (0.16×) | 7.0 vs crypto-bigint 26.8 | −19.8 (0.26×) | 7.0 vs crypto-bigint 49.8 | −42.8 (0.14×) | 7.0 vs crypto-bigint 31.0 | −24.0 (0.23×) |
| square | BabyBear | 12.3 vs plonky3 12.1 | +0.2 (1.02×) | 11.0 vs plonky3 12.1 | −1.1 (0.91×) | 13.0 vs plonky3 13.0 | +0.0 (1.00×) | 12.0 vs plonky3 13.0 | −1.0 (0.92×) |
| square | Goldilocks | 15.0 vs winter-math 19.8 | −4.8 (0.76×) | 13.9 vs winter-math 13.9 | +0.0 (1.00×) | 18.3 vs winter-math 20.2 | −1.9 (0.91×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) |
| square | BN254 scalar | 209 vs ark-ff 265 | −56.0 (0.79×) | 84.2 vs halo2curves 96.6 | −12.4 (0.87×) | 323 vs ark-ff 353 | −30.0 (0.92×) | 134 vs halo2curves 144 | −10.0 (0.93×) |
| square | secp256k1 base | 152 vs ark-ff 222 | −70.0 (0.68×) | 59.0 vs halo2curves 84.3 | −25.3 (0.70×) | 186 vs ark-ff 292 | −106 (0.64×) | 84.7 vs halo2curves 127 | −42.3 (0.67×) |
| square | P-256 base | 161 vs halo2curves 193 | −32.0 (0.83×) | 58.2 vs halo2curves 62.8 | −4.6 (0.93×) | 232 vs ark-ff 327 | −95.0 (0.71×) | 102 vs halo2curves 107 | −5.0 (0.95×) |
| square | P-256 scalar | 217 vs ark-ff 259 | −42.0 (0.84×) | 81.7 vs halo2curves 97.0 | −15.3 (0.84×) | 294 vs ark-ff 344 | −50.0 (0.85×) | 136 vs halo2curves 142 | −6.0 (0.96×) |
| square | Tom-256 base | 209 vs halo2curves 289 | −80.0 (0.72×) | 81.1 vs halo2curves 88.2 | −7.1 (0.92×) | 286 vs halo2curves 474 | −188 (0.60×) | 130 vs halo2curves 135 | −5.0 (0.96×) |
| square | BLS12-381 base | 565 vs ark-ff 620 | −55.0 (0.91×) | 153 vs halo2curves 162 | −9.0 (0.94×) | 789 vs ark-ff 828 | −39.0 (0.95×) | 295 vs halo2curves 310 | −15.0 (0.95×) |
| square | P-521 base | 571 vs crypto-bigint 2,165 | −1,594 (0.26×) | 230 vs crypto-bigint 627 | −397 (0.37×) | 920 vs crypto-bigint 2,853 | −1,933 (0.32×) | 386 vs crypto-bigint 1,144 | −758 (0.34×) |
| mul_u64 | Mersenne-31 | 9.0 vs crypto-bigint 59.7 | −50.7 (0.15×) | 9.0 vs crypto-bigint 31.3 | −22.3 (0.29×) | 9.0 vs crypto-bigint 71.9 | −62.9 (0.13×) | 9.0 vs crypto-bigint 38.6 | −29.6 (0.23×) |
| mul_u64 | BabyBear | 14.0 vs plonky3 34.3 | −20.3 (0.41×) | 11.1 vs plonky3 13.5 | −2.4 (0.82×) | 17.0 vs plonky3 45.0 | −28.0 (0.38×) | 14.4 vs plonky3 17.1 | −2.7 (0.84×) |
| mul_u64 | Goldilocks | 17.6 vs winter-math 25.1 | −7.5 (0.70×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) | 25.6 vs winter-math 32.3 | −6.7 (0.79×) | 15.1 vs winter-math 14.8 | +0.3 (1.02×) |
| mul_u64 | BN254 scalar | 99.2 vs ark-ff 639 | −540 (0.16×) | 37.8 vs crypto-bigint 259 | −221 (0.15×) | 119 vs ark-ff 833 | −714 (0.14×) | 44.8 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | secp256k1 base | 49.1 vs ark-ff 435 | −386 (0.11×) | 17.0 vs crypto-bigint 267 | −250 (0.06×) | 65.1 vs ark-ff 609 | −544 (0.11×) | 23.0 vs halo2curves 394 | −371 (0.06×) |
| mul_u64 | P-256 base | 75.4 vs k256/p256 488 | −413 (0.15×) | 34.0 vs k256/p256 220 | −186 (0.15×) | 90.8 vs ark-ff 840 | −749 (0.11×) | 42.1 vs k256/p256 258 | −216 (0.16×) |
| mul_u64 | P-256 scalar | 82.0 vs k256/p256 514 | −432 (0.16×) | 37.9 vs crypto-bigint 259 | −221 (0.15×) | 108 vs k256/p256 827 | −719 (0.13×) | 47.6 vs k256/p256 338 | −290 (0.14×) |
| mul_u64 | Tom-256 base | 84.2 vs crypto-bigint 866 | −782 (0.10×) | 35.0 vs crypto-bigint 259 | −224 (0.14×) | 105 vs halo2curves 1,278 | −1,173 (0.08×) | 45.5 vs crypto-bigint 410 | −365 (0.11×) |
| mul_u64 | BLS12-381 base | 130 vs ark-ff 1,632 | −1,502 (0.08×) | 48.4 vs halo2curves 557 | −509 (0.09×) | 182 vs ark-ff 2,116 | −1,934 (0.09×) | 62.7 vs crypto-bigint 1,041 | −978 (0.06×) |
| mul_u64 | P-521 base | 94.5 vs crypto-bigint 3,043 | −2,949 (0.03×) | 38.6 vs crypto-bigint 997 | −958 (0.04×) | 139 vs crypto-bigint 3,967 | −3,828 (0.04×) | 53.4 vs crypto-bigint 1,875 | −1,822 (0.03×) |
| mul_uint | Mersenne-31 | 9.0 vs crypto-bigint 59.8 | −50.8 (0.15×) | 9.0 vs crypto-bigint 31.4 | −22.4 (0.29×) | 9.0 vs crypto-bigint 72.1 | −63.1 (0.12×) | 9.0 vs crypto-bigint 38.6 | −29.6 (0.23×) |
| mul_uint | BabyBear | 14.0 vs plonky3 24.1 | −10.1 (0.58×) | 11.1 vs plonky3 13.5 | −2.4 (0.82×) | 17.0 vs plonky3 33.7 | −16.7 (0.50×) | 14.4 vs plonky3 16.7 | −2.3 (0.86×) |
| mul_uint | Goldilocks | 17.5 vs winter-math 25.0 | −7.5 (0.70×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) | 25.6 vs winter-math 32.2 | −6.6 (0.80×) | 15.1 vs winter-math 15.1 | +0.0 (1.00×) |
| mul_uint | BN254 scalar | 435 vs halo2curves 753 | −318 (0.58×) | 171 vs halo2curves 181 | −10.0 (0.94×) | 526 vs halo2curves 1,320 | −794 (0.40×) | 295 vs halo2curves 348 | −53.0 (0.85×) |
| mul_uint | secp256k1 base | 266 vs halo2curves 536 | −270 (0.50×) | 68.1 vs halo2curves 153 | −84.9 (0.45×) | 317 vs halo2curves 953 | −636 (0.33×) | 107 vs halo2curves 295 | −188 (0.36×) |
| mul_uint | P-256 base | 352 vs halo2curves 457 | −105 (0.77×) | 127 vs halo2curves 146 | −19.0 (0.87×) | 424 vs halo2curves 988 | −564 (0.43×) | 231 vs halo2curves 252 | −21.0 (0.92×) |
| mul_uint | P-256 scalar | 425 vs k256/p256 558 | −133 (0.76×) | 181 vs halo2curves 184 | −3.0 (0.98×) | 482 vs k256/p256 877 | −395 (0.55×) | 308 vs halo2curves 346 | −38.0 (0.89×) |
| mul_uint | Tom-256 base | 414 vs halo2curves 697 | −283 (0.59×) | 169 vs halo2curves 179 | −10.0 (0.94×) | 478 vs halo2curves 1,172 | −694 (0.41×) | 296 vs halo2curves 334 | −38.0 (0.89×) |
| mul_uint | BLS12-381 base | 963 vs crypto-bigint 1,938 | −975 (0.50×) | 365 vs halo2curves 379 | −14.0 (0.96×) | 1,134 vs crypto-bigint 2,514 | −1,380 (0.45×) | 897 vs halo2curves 1,019 | −122 (0.88×) |
| mul_uint | P-521 base | 1,566 vs crypto-bigint 3,023 | −1,457 (0.52×) | 244 vs crypto-bigint 994 | −750 (0.25×) | 2,390 vs crypto-bigint 3,991 | −1,601 (0.60×) | 450 vs crypto-bigint 1,893 | −1,443 (0.24×) |
| add_u64 | Mersenne-31 | 4.6 vs crypto-bigint 35.2 | −30.6 (0.13×) | 4.6 vs crypto-bigint 29.5 | −24.9 (0.16×) | 5.0 vs crypto-bigint 43.5 | −38.5 (0.11×) | 5.0 vs crypto-bigint 30.1 | −25.1 (0.17×) |
| add_u64 | BabyBear | 12.1 vs plonky3 31.0 | −18.9 (0.39×) | 8.4 vs plonky3 9.7 | −1.3 (0.87×) | 12.8 vs plonky3 41.8 | −29.0 (0.31×) | 11.0 vs plonky3 13.6 | −2.6 (0.81×) |
| add_u64 | Goldilocks | 8.1 vs winter-math 12.9 | −4.8 (0.63×) | 6.8 vs winter-math 8.2 | −1.4 (0.83×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 8.8 vs winter-math 9.1 | −0.3 (0.97×) |
| add_u64 | BN254 scalar | 106 vs ark-ff 344 | −238 (0.31×) | 41.0 vs halo2curves 225 | −184 (0.18×) | 137 vs ark-ff 402 | −265 (0.34×) | 59.3 vs halo2curves 299 | −240 (0.20×) |
| add_u64 | secp256k1 base | 7.4 vs ark-ff 157 | −150 (0.05×) | 6.0 vs ark-ff 130 | −124 (0.05×) | 7.1 vs ark-ff 224 | −217 (0.03×) | 11.0 vs ark-ff 184 | −173 (0.06×) |
| add_u64 | P-256 base | 68.7 vs ark-ff 260 | −191 (0.26×) | 34.5 vs k256/p256 123 | −88.5 (0.28×) | 76.7 vs ark-ff 404 | −327 (0.19×) | 52.4 vs k256/p256 158 | −106 (0.33×) |
| add_u64 | P-256 scalar | 68.9 vs k256/p256 56.3 | +12.6 (1.22×) | 38.5 vs k256/p256 56.1 | −17.6 (0.69×) | 74.1 vs k256/p256 62.7 | +11.4 (1.18×) | 61.8 vs k256/p256 62.1 | −0.3 (1.00×) |
| add_u64 | Tom-256 base | 85.6 vs crypto-bigint 539 | −453 (0.16×) | 39.7 vs halo2curves 228 | −188 (0.17×) | 96.5 vs halo2curves 709 | −613 (0.14×) | 59.8 vs halo2curves 292 | −232 (0.20×) |
| add_u64 | BLS12-381 base | 149 vs ark-ff 841 | −692 (0.18×) | 61.9 vs halo2curves 348 | −286 (0.18×) | 228 vs ark-ff 1,068 | −840 (0.21×) | 84.4 vs halo2curves 636 | −552 (0.13×) |
| add_u64 | P-521 base | 22.2 vs crypto-bigint 1,677 | −1,655 (0.01×) | 14.1 vs crypto-bigint 659 | −645 (0.02×) | 19.5 vs crypto-bigint 2,191 | −2,172 (0.01×) | 27.2 vs crypto-bigint 1,172 | −1,145 (0.02×) |
| add_uint | Mersenne-31 | 4.6 vs crypto-bigint 35.2 | −30.6 (0.13×) | 4.6 vs crypto-bigint 29.4 | −24.8 (0.16×) | 5.0 vs crypto-bigint 43.6 | −38.6 (0.11×) | 5.0 vs crypto-bigint 30.1 | −25.1 (0.17×) |
| add_uint | BabyBear | 12.1 vs plonky3 21.5 | −9.4 (0.56×) | 8.4 vs plonky3 9.5 | −1.1 (0.88×) | 12.8 vs plonky3 31.3 | −18.5 (0.41×) | 11.0 vs plonky3 13.6 | −2.6 (0.81×) |
| add_uint | Goldilocks | 8.1 vs winter-math 12.8 | −4.7 (0.63×) | 6.8 vs winter-math 7.5 | −0.7 (0.91×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 8.7 vs winter-math 9.1 | −0.4 (0.96×) |
| add_uint | BN254 scalar | 229 vs halo2curves 413 | −184 (0.55×) | 99.4 vs halo2curves 110 | −10.6 (0.90×) | 361 vs halo2curves 677 | −316 (0.53×) | 178 vs halo2curves 206 | −28.0 (0.86×) |
| add_uint | secp256k1 base | 17.3 vs halo2curves 222 | −205 (0.08×) | 13.5 vs halo2curves 86.0 | −72.5 (0.16×) | 17.0 vs halo2curves 417 | −400 (0.04×) | 21.9 vs halo2curves 140 | −118 (0.16×) |
| add_uint | P-256 base | 178 vs halo2curves 251 | −73.0 (0.71×) | 73.6 vs halo2curves 90.8 | −17.2 (0.81×) | 320 vs halo2curves 514 | −194 (0.62×) | 140 vs halo2curves 153 | −13.0 (0.92×) |
| add_uint | P-256 scalar | 258 vs k256/p256 83.8 | +174 (3.08×) | 101 vs k256/p256 84.3 | +16.7 (1.20×) | 324 vs k256/p256 115 | +209 (2.82×) | 183 vs k256/p256 114 | +69.0 (1.61×) |
| add_uint | Tom-256 base | 254 vs halo2curves 377 | −123 (0.67×) | 99.6 vs halo2curves 109 | −9.4 (0.91×) | 320 vs halo2curves 633 | −313 (0.51×) | 177 vs halo2curves 197 | −20.0 (0.90×) |
| add_uint | BLS12-381 base | 763 vs halo2curves 971 | −208 (0.79×) | 204 vs halo2curves 223 | −19.0 (0.91×) | 946 vs crypto-bigint 1,398 | −452 (0.68×) | 383 vs halo2curves 428 | −45.0 (0.89×) |
| add_uint | P-521 base | 61.1 vs crypto-bigint 1,670 | −1,609 (0.04×) | 38.7 vs crypto-bigint 649 | −610 (0.06×) | 78.3 vs crypto-bigint 2,188 | −2,110 (0.04×) | 67.0 vs crypto-bigint 1,171 | −1,104 (0.06×) |
| sub_u64 | Mersenne-31 | 4.7 vs crypto-bigint 22.3 | −17.6 (0.21×) | 4.6 vs crypto-bigint 18.7 | −14.1 (0.25×) | 5.1 vs crypto-bigint 30.5 | −25.4 (0.17×) | 5.1 vs crypto-bigint 20.2 | −15.1 (0.25×) |
| sub_u64 | BabyBear | 12.0 vs crypto-bigint 28.7 | −16.7 (0.42×) | 8.3 vs plonky3 10.3 | −2.0 (0.81×) | 12.5 vs plonky3 41.8 | −29.3 (0.30×) | 11.1 vs plonky3 13.4 | −2.3 (0.83×) |
| sub_u64 | Goldilocks | 8.9 vs winter-math 11.4 | −2.5 (0.78×) | 6.3 vs winter-math 7.4 | −1.1 (0.85×) | 11.4 vs winter-math 18.1 | −6.7 (0.63×) | 9.2 vs winter-math 8.5 | +0.7 (1.08×) |
| sub_u64 | BN254 scalar | 103 vs ark-ff 345 | −242 (0.30×) | 41.4 vs crypto-bigint 188 | −147 (0.22×) | 137 vs ark-ff 409 | −272 (0.33×) | 62.7 vs crypto-bigint 288 | −225 (0.22×) |
| sub_u64 | secp256k1 base | 9.0 vs ark-ff 161 | −152 (0.06×) | 6.0 vs ark-ff 126 | −120 (0.05×) | 8.0 vs ark-ff 226 | −218 (0.04×) | 9.5 vs ark-ff 187 | −178 (0.05×) |
| sub_u64 | P-256 base | 70.5 vs ark-ff 265 | −195 (0.27×) | 34.2 vs k256/p256 118 | −83.8 (0.29×) | 75.9 vs ark-ff 404 | −328 (0.19×) | 51.4 vs k256/p256 149 | −97.6 (0.34×) |
| sub_u64 | P-256 scalar | 69.6 vs k256/p256 22.6 | +47.0 (3.08×) | 36.6 vs k256/p256 22.5 | +14.1 (1.63×) | 71.9 vs k256/p256 25.0 | +46.9 (2.88×) | 58.6 vs k256/p256 25.0 | +33.6 (2.34×) |
| sub_u64 | Tom-256 base | 88.2 vs crypto-bigint 495 | −407 (0.18×) | 40.2 vs crypto-bigint 188 | −148 (0.21×) | 99.5 vs halo2curves 689 | −590 (0.14×) | 59.2 vs crypto-bigint 285 | −226 (0.21×) |
| sub_u64 | BLS12-381 base | 150 vs ark-ff 822 | −672 (0.18×) | 60.9 vs halo2curves 337 | −276 (0.18×) | 255 vs ark-ff 1,049 | −794 (0.24×) | 88.2 vs halo2curves 621 | −533 (0.14×) |
| sub_u64 | P-521 base | 22.4 vs crypto-bigint 1,666 | −1,644 (0.01×) | 14.7 vs crypto-bigint 630 | −615 (0.02×) | 21.9 vs crypto-bigint 2,178 | −2,156 (0.01×) | 20.7 vs crypto-bigint 1,161 | −1,140 (0.02×) |
| sub_uint | Mersenne-31 | 4.7 vs crypto-bigint 22.3 | −17.6 (0.21×) | 4.6 vs crypto-bigint 18.7 | −14.1 (0.25×) | 5.1 vs crypto-bigint 30.5 | −25.4 (0.17×) | 5.1 vs crypto-bigint 20.3 | −15.2 (0.25×) |
| sub_uint | BabyBear | 12.0 vs plonky3 22.0 | −10.0 (0.55×) | 8.4 vs plonky3 10.1 | −1.7 (0.83×) | 12.5 vs plonky3 31.3 | −18.8 (0.40×) | 11.1 vs plonky3 13.4 | −2.3 (0.83×) |
| sub_uint | Goldilocks | 8.9 vs winter-math 11.4 | −2.5 (0.78×) | 6.3 vs winter-math 6.2 | +0.1 (1.02×) | 11.4 vs winter-math 18.1 | −6.7 (0.63×) | 9.2 vs winter-math 8.5 | +0.7 (1.08×) |
| sub_uint | BN254 scalar | 241 vs halo2curves 400 | −159 (0.60×) | 97.0 vs halo2curves 105 | −8.0 (0.92×) | 360 vs halo2curves 652 | −292 (0.55×) | 173 vs halo2curves 194 | −21.0 (0.89×) |
| sub_uint | secp256k1 base | 19.0 vs halo2curves 211 | −192 (0.09×) | 12.8 vs halo2curves 79.7 | −66.9 (0.16×) | 16.1 vs halo2curves 400 | −384 (0.04×) | 20.7 vs halo2curves 130 | −109 (0.16×) |
| sub_uint | P-256 base | 175 vs halo2curves 234 | −59.0 (0.75×) | 72.2 vs halo2curves 85.3 | −13.1 (0.85×) | 271 vs halo2curves 498 | −227 (0.54×) | 135 vs halo2curves 143 | −8.0 (0.94×) |
| sub_uint | P-256 scalar | 253 vs k256/p256 67.1 | +186 (3.77×) | 98.5 vs k256/p256 66.7 | +31.8 (1.48×) | 320 vs k256/p256 96.0 | +224 (3.33×) | 178 vs k256/p256 95.5 | +82.5 (1.86×) |
| sub_uint | Tom-256 base | 249 vs halo2curves 365 | −116 (0.68×) | 96.6 vs halo2curves 103 | −6.4 (0.94×) | 320 vs halo2curves 612 | −292 (0.52×) | 172 vs halo2curves 188 | −16.0 (0.91×) |
| sub_uint | BLS12-381 base | 762 vs halo2curves 947 | −185 (0.80×) | 197 vs halo2curves 216 | −19.0 (0.91×) | 945 vs crypto-bigint 1,375 | −430 (0.69×) | 378 vs halo2curves 412 | −34.0 (0.92×) |
| sub_uint | P-521 base | 62.5 vs crypto-bigint 1,664 | −1,602 (0.04×) | 37.0 vs crypto-bigint 627 | −590 (0.06×) | 71.5 vs crypto-bigint 2,186 | −2,115 (0.03×) | 64.2 vs crypto-bigint 1,161 | −1,097 (0.06×) |
| div_u64 | BN254 scalar | 1,203 vs halo2curves 13,439 | −12,236 (0.09×) | 865 vs halo2curves 6,980 | −6,115 (0.12×) | 1,381 vs halo2curves 19,374 | −17,993 (0.07×) | 1,105 vs halo2curves 12,075 | −10,970 (0.09×) |
| div_u64 | secp256k1 base | 1,008 vs halo2curves 13,016 | −12,008 (0.08×) | 716 vs halo2curves 6,945 | −6,229 (0.10×) | 1,231 vs halo2curves 18,780 | −17,549 (0.07×) | 808 vs halo2curves 11,822 | −11,014 (0.07×) |
| div_u64 | P-256 base | 1,081 vs halo2curves 13,051 | −11,970 (0.08×) | 801 vs halo2curves 6,893 | −6,092 (0.12×) | 1,346 vs halo2curves 18,929 | −17,583 (0.07×) | 985 vs halo2curves 11,791 | −10,806 (0.08×) |
| div_u64 | P-256 scalar | 1,186 vs halo2curves 13,386 | −12,200 (0.09×) | 869 vs halo2curves 6,938 | −6,069 (0.13×) | 1,446 vs halo2curves 19,332 | −17,886 (0.07×) | 1,120 vs halo2curves 11,935 | −10,815 (0.09×) |
| div_u64 | Tom-256 base | 1,201 vs halo2curves 13,283 | −12,082 (0.09×) | 870 vs halo2curves 6,965 | −6,095 (0.12×) | 1,460 vs halo2curves 19,045 | −17,585 (0.08×) | 1,096 vs halo2curves 11,946 | −10,850 (0.09×) |
| div_u64 | BLS12-381 base | 1,785 vs halo2curves 23,690 | −21,905 (0.08×) | 1,105 vs halo2curves 11,016 | −9,911 (0.10×) | 2,470 vs halo2curves 32,620 | −30,150 (0.08×) | 1,754 vs halo2curves 18,784 | −17,030 (0.09×) |
| div_u64 | P-521 base | 2,450 vs crypto-bigint 61,624 | −59,174 (0.04×) | 1,002 vs crypto-bigint 45,253 | −44,251 (0.02×) | 3,600 vs crypto-bigint 77,629 | −74,029 (0.05×) | 1,532 vs crypto-bigint 57,524 | −55,992 (0.03×) |
| div_uint | Mersenne-31 | 268 vs crypto-bigint 1,822 | −1,554 (0.15×) | 269 vs crypto-bigint 1,603 | −1,334 (0.17×) | 280 vs runtime-algebra 2,103 | −1,823 (0.13×) | 278 vs crypto-bigint 1,989 | −1,711 (0.14×) |
| div_uint | BabyBear | 273 vs plonky3 672 | −399 (0.41×) | 273 vs plonky3 646 | −373 (0.42×) | 278 vs plonky3 712 | −434 (0.39×) | 280 vs plonky3 729 | −449 (0.38×) |
| div_uint | Goldilocks | 549 vs winter-math 1,788 | −1,239 (0.31×) | 539 vs winter-math 927 | −388 (0.58×) | 579 vs winter-math 1,895 | −1,316 (0.31×) | 573 vs winter-math 1,033 | −460 (0.55×) |
| div_uint | BN254 scalar | 9,008 vs halo2curves 14,999 | −5,991 (0.60×) | 6,718 vs halo2curves 8,190 | −1,472 (0.82×) | 12,609 vs halo2curves 22,851 | −10,242 (0.55×) | 8,438 vs halo2curves 14,854 | −6,416 (0.57×) |
| div_uint | secp256k1 base | 8,513 vs halo2curves 14,816 | −6,303 (0.57×) | 6,064 vs halo2curves 8,237 | −2,173 (0.74×) | 12,043 vs halo2curves 22,575 | −10,532 (0.53×) | 8,158 vs halo2curves 14,731 | −6,573 (0.55×) |
| div_uint | P-256 base | 8,692 vs halo2curves 14,798 | −6,106 (0.59×) | 6,159 vs halo2curves 8,195 | −2,036 (0.75×) | 12,174 vs halo2curves 22,672 | −10,498 (0.54×) | 7,871 vs halo2curves 14,752 | −6,881 (0.53×) |
| div_uint | P-256 scalar | 9,049 vs halo2curves 15,023 | −5,974 (0.60×) | 6,543 vs halo2curves 8,215 | −1,672 (0.80×) | 12,535 vs halo2curves 23,064 | −10,529 (0.54×) | 8,575 vs halo2curves 14,905 | −6,330 (0.58×) |
| div_uint | Tom-256 base | 8,896 vs halo2curves 14,971 | −6,075 (0.59×) | 6,454 vs halo2curves 8,210 | −1,756 (0.79×) | 12,386 vs halo2curves 22,737 | −10,351 (0.54×) | 8,407 vs halo2curves 14,872 | −6,465 (0.57×) |
| div_uint | BLS12-381 base | 20,737 vs halo2curves 27,222 | −6,485 (0.76×) | 12,085 vs halo2curves 13,609 | −1,524 (0.89×) | 26,397 vs halo2curves 40,298 | −13,901 (0.66×) | 16,914 vs halo2curves 24,791 | −7,877 (0.68×) |
| div_uint | P-521 base | 34,505 vs crypto-bigint 67,758 | −33,253 (0.51×) | 18,162 vs crypto-bigint 51,106 | −32,944 (0.36×) | 49,283 vs crypto-bigint 84,087 | −34,804 (0.59×) | 30,231 vs crypto-bigint 63,757 | −33,526 (0.47×) |
| add, constant time | Mersenne-31 | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.6 | −14.6 (0.17×) | 3.0 vs runtime-algebra 17.9 | −14.9 (0.17×) |
| add, constant time | BabyBear | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 18.0 | −15.0 (0.17×) | 3.0 vs runtime-algebra 17.6 | −14.6 (0.17×) | 3.0 vs runtime-algebra 17.9 | −14.9 (0.17×) |
| add, constant time | Goldilocks | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.0 | −1.0 (0.75×) | 3.0 vs winter-math 4.7 | −1.7 (0.64×) | 3.0 vs winter-math 4.4 | −1.4 (0.68×) |
| add, constant time | BN254 scalar | 24.6 vs halo2curves 43.5 | −18.9 (0.57×) | 16.9 vs halo2curves 24.5 | −7.6 (0.69×) | 22.2 vs halo2curves 45.3 | −23.1 (0.49×) | 24.1 vs halo2curves 34.1 | −10.0 (0.71×) |
| add, constant time | secp256k1 base | 28.1 vs halo2curves 39.8 | −11.7 (0.71×) | 16.1 vs halo2curves 22.5 | −6.4 (0.72×) | 25.5 vs halo2curves 38.0 | −12.5 (0.67×) | 27.1 vs halo2curves 32.0 | −4.9 (0.85×) |
| add, constant time | P-256 base | 24.9 vs k256/p256 40.3 | −15.4 (0.62×) | 15.9 vs k256/p256 20.5 | −4.6 (0.78×) | 24.0 vs k256/p256 39.3 | −15.3 (0.61×) | 25.6 vs k256/p256 29.6 | −4.0 (0.86×) |
| add, constant time | P-256 scalar | 26.2 vs halo2curves 42.7 | −16.5 (0.61×) | 17.2 vs halo2curves 23.2 | −6.0 (0.74×) | 26.3 vs halo2curves 42.8 | −16.5 (0.61×) | 27.1 vs halo2curves 33.1 | −6.0 (0.82×) |
| add, constant time | Tom-256 base | 27.0 vs halo2curves 41.1 | −14.1 (0.66×) | 17.0 vs halo2curves 24.7 | −7.7 (0.69×) | 26.3 vs halo2curves 42.5 | −16.2 (0.62×) | 27.1 vs halo2curves 32.6 | −5.5 (0.83×) |
| add, constant time | BLS12-381 base | 41.9 vs halo2curves 63.5 | −21.6 (0.66×) | 24.3 vs halo2curves 34.1 | −9.8 (0.71×) | 39.6 vs halo2curves 69.0 | −29.4 (0.57×) | 40.1 vs halo2curves 54.4 | −14.3 (0.74×) |
| add, constant time | P-521 base | 76.7 vs crypto-bigint 182 | −105 (0.42×) | 39.5 vs crypto-bigint 179 | −140 (0.22×) | 82.0 vs crypto-bigint 245 | −163 (0.33×) | 72.2 vs crypto-bigint 243 | −171 (0.30×) |
| sub, constant time | Mersenne-31 | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | BabyBear | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) | 3.0 vs crypto-bigint 8.0 | −5.0 (0.38×) |
| sub, constant time | Goldilocks | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) | 3.0 vs winter-math 3.0 | +0.0 (1.00×) |
| sub, constant time | BN254 scalar | 22.2 vs halo2curves 29.2 | −7.0 (0.76×) | 14.6 vs halo2curves 15.7 | −1.1 (0.93×) | 20.1 vs halo2curves 25.6 | −5.5 (0.79×) | 22.1 vs halo2curves 23.1 | −1.0 (0.96×) |
| sub, constant time | secp256k1 base | 20.3 vs halo2curves 25.2 | −4.9 (0.81×) | 14.0 vs halo2curves 15.9 | −1.9 (0.88×) | 19.1 vs halo2curves 26.4 | −7.3 (0.72×) | 20.4 vs halo2curves 21.0 | −0.6 (0.97×) |
| sub, constant time | P-256 base | 19.5 vs halo2curves 24.3 | −4.8 (0.80×) | 12.1 vs halo2curves 14.5 | −2.4 (0.83×) | 19.8 vs halo2curves 24.0 | −4.2 (0.83×) | 19.1 vs halo2curves 19.5 | −0.4 (0.98×) |
| sub, constant time | P-256 scalar | 21.6 vs k256/p256 24.6 | −3.0 (0.88×) | 14.6 vs halo2curves 16.0 | −1.4 (0.91×) | 21.1 vs k256/p256 24.8 | −3.7 (0.85×) | 21.7 vs halo2curves 22.1 | −0.4 (0.98×) |
| sub, constant time | Tom-256 base | 21.9 vs halo2curves 28.9 | −7.0 (0.76×) | 14.4 vs halo2curves 15.7 | −1.3 (0.92×) | 19.6 vs halo2curves 26.5 | −6.9 (0.74×) | 22.1 vs halo2curves 22.0 | +0.1 (1.00×) |
| sub, constant time | BLS12-381 base | 37.3 vs halo2curves 44.5 | −7.2 (0.84×) | 22.1 vs halo2curves 24.5 | −2.4 (0.90×) | 36.1 vs halo2curves 44.1 | −8.0 (0.82×) | 36.4 vs halo2curves 36.1 | +0.3 (1.01×) |
| sub, constant time | P-521 base | 56.4 vs crypto-bigint 161 | −105 (0.35×) | 31.3 vs crypto-bigint 145 | −114 (0.22×) | 59.9 vs crypto-bigint 195 | −135 (0.31×) | 51.2 vs crypto-bigint 173 | −122 (0.30×) |
| mul, constant time | Mersenne-31 | 7.0 vs crypto-bigint 45.7 | −38.7 (0.15×) | 7.0 vs crypto-bigint 26.8 | −19.8 (0.26×) | 7.0 vs crypto-bigint 48.9 | −41.9 (0.14×) | 7.0 vs crypto-bigint 30.6 | −23.6 (0.23×) |
| mul, constant time | BabyBear | 13.0 vs crypto-bigint 45.7 | −32.7 (0.28×) | 11.0 vs crypto-bigint 26.8 | −15.8 (0.41×) | 14.0 vs crypto-bigint 49.9 | −35.9 (0.28×) | 12.0 vs crypto-bigint 29.1 | −17.1 (0.41×) |
| mul, constant time | Goldilocks | 16.5 vs winter-math 18.1 | −1.6 (0.91×) | 12.7 vs winter-math 12.7 | +0.0 (1.00×) | 18.0 vs winter-math 19.0 | −1.0 (0.95×) | 14.0 vs winter-math 14.0 | +0.0 (1.00×) |
| mul, constant time | BN254 scalar | 245 vs halo2curves 387 | −142 (0.63×) | 91.5 vs halo2curves 93.6 | −2.1 (0.98×) | 414 vs halo2curves 697 | −283 (0.59×) | 149 vs halo2curves 178 | −29.0 (0.84×) |
| mul, constant time | secp256k1 base | 170 vs halo2curves 308 | −138 (0.55×) | 67.1 vs halo2curves 89.7 | −22.6 (0.75×) | 237 vs halo2curves 580 | −343 (0.41×) | 105 vs halo2curves 158 | −53.0 (0.66×) |
| mul, constant time | P-256 base | 184 vs k256/p256 242 | −58.0 (0.76×) | 62.2 vs halo2curves 74.3 | −12.1 (0.84×) | 292 vs k256/p256 461 | −169 (0.63×) | 118 vs halo2curves 130 | −12.0 (0.91×) |
| mul, constant time | P-256 scalar | 240 vs halo2curves 364 | −124 (0.66×) | 91.0 vs halo2curves 95.4 | −4.4 (0.95×) | 327 vs halo2curves 691 | −364 (0.47×) | 157 vs halo2curves 176 | −19.0 (0.89×) |
| mul, constant time | Tom-256 base | 238 vs halo2curves 343 | −105 (0.69×) | 88.2 vs halo2curves 93.2 | −5.0 (0.95×) | 325 vs halo2curves 600 | −275 (0.54×) | 152 vs halo2curves 171 | −19.0 (0.89×) |
| mul, constant time | BLS12-381 base | 719 vs halo2curves 899 | −180 (0.80×) | 185 vs halo2curves 193 | −8.0 (0.96×) | 879 vs crypto-bigint 1,321 | −442 (0.67×) | 336 vs halo2curves 377 | −41.0 (0.89×) |
| mul, constant time | P-521 base | 688 vs crypto-bigint 2,141 | −1,453 (0.32×) | 240 vs crypto-bigint 619 | −379 (0.39×) | 1,292 vs crypto-bigint 2,773 | −1,481 (0.47×) | 439 vs crypto-bigint 1,104 | −665 (0.40×) |
| square, constant time | Mersenne-31 | 7.0 vs crypto-bigint 44.2 | −37.2 (0.16×) | 7.0 vs crypto-bigint 26.8 | −19.8 (0.26×) | 7.0 vs crypto-bigint 49.8 | −42.8 (0.14×) | 7.0 vs crypto-bigint 31.0 | −24.0 (0.23×) |
| square, constant time | BabyBear | 12.2 vs crypto-bigint 43.8 | −31.6 (0.28×) | 11.0 vs crypto-bigint 26.8 | −15.8 (0.41×) | 13.0 vs crypto-bigint 51.1 | −38.1 (0.25×) | 12.0 vs crypto-bigint 31.0 | −19.0 (0.39×) |
| square, constant time | Goldilocks | 15.0 vs winter-math 19.8 | −4.8 (0.76×) | 13.9 vs winter-math 13.9 | +0.0 (1.00×) | 18.3 vs winter-math 20.2 | −1.9 (0.91×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) |
| square, constant time | BN254 scalar | 214 vs halo2curves 315 | −101 (0.68×) | 84.2 vs halo2curves 96.6 | −12.4 (0.87×) | 286 vs halo2curves 546 | −260 (0.52×) | 134 vs halo2curves 144 | −10.0 (0.93×) |
| square, constant time | secp256k1 base | 152 vs halo2curves 245 | −93.0 (0.62×) | 59.0 vs halo2curves 84.3 | −25.3 (0.70×) | 185 vs halo2curves 455 | −270 (0.41×) | 84.6 vs halo2curves 127 | −42.4 (0.67×) |
| square, constant time | P-256 base | 161 vs halo2curves 193 | −32.0 (0.83×) | 58.2 vs halo2curves 62.8 | −4.6 (0.93×) | 235 vs halo2curves 340 | −105 (0.69×) | 102 vs halo2curves 107 | −5.0 (0.95×) |
| square, constant time | P-256 scalar | 217 vs halo2curves 290 | −73.0 (0.75×) | 81.5 vs halo2curves 97.0 | −15.5 (0.84×) | 294 vs halo2curves 550 | −256 (0.53×) | 136 vs halo2curves 142 | −6.0 (0.96×) |
| square, constant time | Tom-256 base | 209 vs halo2curves 289 | −80.0 (0.72×) | 81.1 vs halo2curves 88.2 | −7.1 (0.92×) | 288 vs halo2curves 474 | −186 (0.61×) | 130 vs halo2curves 135 | −5.0 (0.96×) |
| square, constant time | BLS12-381 base | 565 vs halo2curves 927 | −362 (0.61×) | 153 vs halo2curves 162 | −9.0 (0.94×) | 795 vs halo2curves 1,228 | −433 (0.65×) | 295 vs halo2curves 310 | −15.0 (0.95×) |
| square, constant time | P-521 base | 570 vs crypto-bigint 2,165 | −1,595 (0.26×) | 231 vs crypto-bigint 627 | −396 (0.37×) | 924 vs crypto-bigint 2,853 | −1,929 (0.32×) | 387 vs crypto-bigint 1,144 | −757 (0.34×) |
| mul_u64, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.7 | −50.7 (0.15×) | 9.0 vs crypto-bigint 31.3 | −22.3 (0.29×) | 9.0 vs crypto-bigint 71.9 | −62.9 (0.13×) | 9.0 vs crypto-bigint 38.6 | −29.6 (0.23×) |
| mul_u64, constant time | BabyBear | 14.0 vs crypto-bigint 65.1 | −51.1 (0.22×) | 11.1 vs crypto-bigint 32.4 | −21.3 (0.34×) | 17.0 vs crypto-bigint 86.3 | −69.3 (0.20×) | 14.4 vs crypto-bigint 40.5 | −26.1 (0.36×) |
| mul_u64, constant time | Goldilocks | 17.5 vs winter-math 25.1 | −7.6 (0.70×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) | 25.6 vs winter-math 32.3 | −6.7 (0.79×) | 15.1 vs winter-math 14.8 | +0.3 (1.02×) |
| mul_u64, constant time | BN254 scalar | 109 vs crypto-bigint 868 | −759 (0.13×) | 48.9 vs crypto-bigint 259 | −210 (0.19×) | 138 vs halo2curves 1,441 | −1,303 (0.10×) | 57.5 vs crypto-bigint 410 | −353 (0.14×) |
| mul_u64, constant time | secp256k1 base | 69.4 vs halo2curves 690 | −621 (0.10×) | 30.6 vs crypto-bigint 267 | −236 (0.11×) | 85.1 vs halo2curves 1,025 | −940 (0.08×) | 40.8 vs halo2curves 394 | −353 (0.10×) |
| mul_u64, constant time | P-256 base | 88.6 vs k256/p256 488 | −399 (0.18×) | 45.0 vs k256/p256 220 | −175 (0.20×) | 108 vs k256/p256 915 | −807 (0.12×) | 62.8 vs k256/p256 258 | −195 (0.24×) |
| mul_u64, constant time | P-256 scalar | 107 vs k256/p256 514 | −407 (0.21×) | 47.2 vs crypto-bigint 259 | −212 (0.18×) | 127 vs k256/p256 827 | −700 (0.15×) | 68.8 vs k256/p256 338 | −269 (0.20×) |
| mul_u64, constant time | Tom-256 base | 101 vs crypto-bigint 866 | −765 (0.12×) | 45.2 vs crypto-bigint 259 | −214 (0.17×) | 122 vs halo2curves 1,278 | −1,156 (0.10×) | 68.5 vs crypto-bigint 410 | −342 (0.17×) |
| mul_u64, constant time | BLS12-381 base | 154 vs crypto-bigint 1,937 | −1,783 (0.08×) | 70.6 vs halo2curves 557 | −486 (0.13×) | 197 vs crypto-bigint 2,481 | −2,284 (0.08×) | 84.5 vs crypto-bigint 1,041 | −957 (0.08×) |
| mul_u64, constant time | P-521 base | 139 vs crypto-bigint 3,043 | −2,904 (0.05×) | 62.1 vs crypto-bigint 997 | −935 (0.06×) | 170 vs crypto-bigint 3,967 | −3,797 (0.04×) | 90.3 vs crypto-bigint 1,875 | −1,785 (0.05×) |
| mul_uint, constant time | Mersenne-31 | 9.0 vs crypto-bigint 59.8 | −50.8 (0.15×) | 9.0 vs crypto-bigint 31.4 | −22.4 (0.29×) | 9.0 vs crypto-bigint 72.1 | −63.1 (0.12×) | 9.0 vs crypto-bigint 38.6 | −29.6 (0.23×) |
| mul_uint, constant time | BabyBear | 14.0 vs crypto-bigint 64.6 | −50.6 (0.22×) | 11.1 vs crypto-bigint 32.4 | −21.3 (0.34×) | 17.0 vs crypto-bigint 90.7 | −73.7 (0.19×) | 14.4 vs crypto-bigint 41.3 | −26.9 (0.35×) |
| mul_uint, constant time | Goldilocks | 17.5 vs winter-math 25.0 | −7.5 (0.70×) | 14.3 vs winter-math 14.3 | +0.0 (1.00×) | 25.6 vs winter-math 32.2 | −6.6 (0.80×) | 15.1 vs winter-math 15.1 | +0.0 (1.00×) |
| mul_uint, constant time | BN254 scalar | 480 vs halo2curves 753 | −273 (0.64×) | 171 vs halo2curves 181 | −10.0 (0.94×) | 589 vs halo2curves 1,320 | −731 (0.45×) | 298 vs halo2curves 348 | −50.0 (0.86×) |
| mul_uint, constant time | secp256k1 base | 321 vs halo2curves 536 | −215 (0.60×) | 67.5 vs halo2curves 153 | −85.5 (0.44×) | 387 vs halo2curves 953 | −566 (0.41×) | 117 vs halo2curves 295 | −178 (0.40×) |
| mul_uint, constant time | P-256 base | 410 vs halo2curves 457 | −47.0 (0.90×) | 124 vs halo2curves 146 | −22.0 (0.85×) | 487 vs halo2curves 988 | −501 (0.49×) | 229 vs halo2curves 252 | −23.0 (0.91×) |
| mul_uint, constant time | P-256 scalar | 477 vs k256/p256 558 | −81.0 (0.85×) | 180 vs halo2curves 184 | −4.0 (0.98×) | 552 vs k256/p256 877 | −325 (0.63×) | 311 vs halo2curves 346 | −35.0 (0.90×) |
| mul_uint, constant time | Tom-256 base | 466 vs halo2curves 697 | −231 (0.67×) | 172 vs halo2curves 179 | −7.0 (0.96×) | 548 vs halo2curves 1,172 | −624 (0.47×) | 297 vs halo2curves 334 | −37.0 (0.89×) |
| mul_uint, constant time | BLS12-381 base | 1,076 vs crypto-bigint 1,938 | −862 (0.56×) | 367 vs halo2curves 379 | −12.0 (0.97×) | 1,691 vs crypto-bigint 2,514 | −823 (0.67×) | 900 vs halo2curves 1,019 | −119 (0.88×) |
| mul_uint, constant time | P-521 base | 2,016 vs crypto-bigint 3,023 | −1,007 (0.67×) | 281 vs crypto-bigint 994 | −713 (0.28×) | 2,908 vs crypto-bigint 3,991 | −1,083 (0.73×) | 494 vs crypto-bigint 1,893 | −1,399 (0.26×) |
| add_u64, constant time | Mersenne-31 | 4.6 vs crypto-bigint 35.2 | −30.6 (0.13×) | 4.6 vs crypto-bigint 29.5 | −24.9 (0.16×) | 5.0 vs crypto-bigint 43.5 | −38.5 (0.11×) | 5.0 vs crypto-bigint 30.1 | −25.1 (0.17×) |
| add_u64, constant time | BabyBear | 11.8 vs crypto-bigint 40.2 | −28.4 (0.29×) | 7.1 vs crypto-bigint 31.0 | −23.9 (0.23×) | 13.1 vs crypto-bigint 53.0 | −39.9 (0.25×) | 11.5 vs crypto-bigint 32.4 | −20.9 (0.35×) |
| add_u64, constant time | Goldilocks | 8.1 vs winter-math 12.9 | −4.8 (0.63×) | 6.8 vs winter-math 8.2 | −1.4 (0.83×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 8.7 vs winter-math 9.1 | −0.4 (0.96×) |
| add_u64, constant time | BN254 scalar | 97.7 vs crypto-bigint 538 | −440 (0.18×) | 40.7 vs halo2curves 225 | −184 (0.18×) | 108 vs halo2curves 764 | −656 (0.14×) | 69.9 vs halo2curves 299 | −229 (0.23×) |
| add_u64, constant time | secp256k1 base | 25.9 vs halo2curves 394 | −368 (0.07×) | 15.0 vs halo2curves 209 | −194 (0.07×) | 25.5 vs halo2curves 485 | −460 (0.05×) | 24.0 vs halo2curves 245 | −221 (0.10×) |
| add_u64, constant time | P-256 base | 69.1 vs k256/p256 279 | −210 (0.25×) | 36.6 vs k256/p256 123 | −86.4 (0.30×) | 81.4 vs k256/p256 499 | −418 (0.16×) | 58.6 vs k256/p256 158 | −99.4 (0.37×) |
| add_u64, constant time | P-256 scalar | 105 vs k256/p256 56.3 | +48.7 (1.87×) | 43.0 vs k256/p256 56.1 | −13.1 (0.77×) | 109 vs k256/p256 62.7 | +46.3 (1.74×) | 72.9 vs k256/p256 62.1 | +10.8 (1.17×) |
| add_u64, constant time | Tom-256 base | 98.3 vs crypto-bigint 539 | −441 (0.18×) | 40.8 vs halo2curves 228 | −187 (0.18×) | 110 vs halo2curves 709 | −599 (0.16×) | 71.1 vs halo2curves 292 | −221 (0.24×) |
| add_u64, constant time | BLS12-381 base | 136 vs crypto-bigint 1,089 | −953 (0.12×) | 63.0 vs halo2curves 348 | −285 (0.18×) | 170 vs crypto-bigint 1,368 | −1,198 (0.12×) | 107 vs halo2curves 636 | −529 (0.17×) |
| add_u64, constant time | P-521 base | 66.6 vs crypto-bigint 1,677 | −1,610 (0.04×) | 38.7 vs crypto-bigint 659 | −620 (0.06×) | 68.0 vs crypto-bigint 2,191 | −2,123 (0.03×) | 62.1 vs crypto-bigint 1,172 | −1,110 (0.05×) |
| add_uint, constant time | Mersenne-31 | 4.6 vs crypto-bigint 35.2 | −30.6 (0.13×) | 4.6 vs crypto-bigint 29.4 | −24.8 (0.16×) | 5.0 vs crypto-bigint 43.6 | −38.6 (0.11×) | 5.0 vs crypto-bigint 30.1 | −25.1 (0.17×) |
| add_uint, constant time | BabyBear | 11.8 vs crypto-bigint 40.2 | −28.4 (0.29×) | 7.1 vs crypto-bigint 31.0 | −23.9 (0.23×) | 13.1 vs crypto-bigint 52.4 | −39.3 (0.25×) | 11.5 vs crypto-bigint 31.7 | −20.2 (0.36×) |
| add_uint, constant time | Goldilocks | 8.1 vs winter-math 12.8 | −4.7 (0.63×) | 6.8 vs winter-math 7.5 | −0.7 (0.91×) | 11.8 vs winter-math 19.1 | −7.3 (0.62×) | 8.7 vs winter-math 9.1 | −0.4 (0.96×) |
| add_uint, constant time | BN254 scalar | 247 vs halo2curves 413 | −166 (0.60×) | 99.6 vs halo2curves 110 | −10.4 (0.91×) | 345 vs halo2curves 677 | −332 (0.51×) | 178 vs halo2curves 206 | −28.0 (0.86×) |
| add_uint, constant time | secp256k1 base | 39.3 vs halo2curves 222 | −183 (0.18×) | 25.2 vs halo2curves 86.0 | −60.8 (0.29×) | 38.7 vs halo2curves 417 | −378 (0.09×) | 40.6 vs halo2curves 140 | −99.4 (0.29×) |
| add_uint, constant time | P-256 base | 177 vs halo2curves 251 | −74.0 (0.71×) | 75.6 vs halo2curves 90.8 | −15.2 (0.83×) | 281 vs halo2curves 514 | −233 (0.55×) | 140 vs halo2curves 153 | −13.0 (0.92×) |
| add_uint, constant time | P-256 scalar | 256 vs k256/p256 83.8 | +172 (3.05×) | 102 vs k256/p256 84.3 | +17.7 (1.21×) | 318 vs k256/p256 115 | +203 (2.77×) | 183 vs k256/p256 114 | +69.0 (1.61×) |
| add_uint, constant time | Tom-256 base | 253 vs halo2curves 377 | −124 (0.67×) | 99.0 vs halo2curves 109 | −10.0 (0.91×) | 328 vs halo2curves 633 | −305 (0.52×) | 177 vs halo2curves 197 | −20.0 (0.90×) |
| add_uint, constant time | BLS12-381 base | 762 vs halo2curves 971 | −209 (0.78×) | 204 vs halo2curves 223 | −19.0 (0.91×) | 947 vs crypto-bigint 1,398 | −451 (0.68×) | 382 vs halo2curves 428 | −46.0 (0.89×) |
| add_uint, constant time | P-521 base | 138 vs crypto-bigint 1,670 | −1,532 (0.08×) | 80.0 vs crypto-bigint 649 | −569 (0.12×) | 139 vs crypto-bigint 2,188 | −2,049 (0.06×) | 130 vs crypto-bigint 1,171 | −1,041 (0.11×) |
| sub_u64, constant time | Mersenne-31 | 4.6 vs crypto-bigint 22.3 | −17.7 (0.21×) | 4.6 vs crypto-bigint 18.7 | −14.1 (0.25×) | 5.1 vs crypto-bigint 30.5 | −25.4 (0.17×) | 5.1 vs crypto-bigint 20.2 | −15.1 (0.25×) |
| sub_u64, constant time | BabyBear | 12.0 vs crypto-bigint 28.7 | −16.7 (0.42×) | 7.4 vs crypto-bigint 19.4 | −12.0 (0.38×) | 13.1 vs crypto-bigint 42.4 | −29.3 (0.31×) | 10.6 vs crypto-bigint 21.8 | −11.2 (0.49×) |
| sub_u64, constant time | Goldilocks | 8.9 vs winter-math 11.4 | −2.5 (0.78×) | 6.3 vs winter-math 7.4 | −1.1 (0.85×) | 11.4 vs winter-math 18.1 | −6.7 (0.63×) | 9.2 vs winter-math 8.5 | +0.7 (1.08×) |
| sub_u64, constant time | BN254 scalar | 95.1 vs crypto-bigint 494 | −399 (0.19×) | 39.0 vs crypto-bigint 188 | −149 (0.21×) | 104 vs halo2curves 750 | −646 (0.14×) | 66.9 vs crypto-bigint 288 | −221 (0.23×) |
| sub_u64, constant time | secp256k1 base | 18.0 vs halo2curves 381 | −363 (0.05×) | 11.4 vs crypto-bigint 189 | −178 (0.06×) | 15.0 vs halo2curves 472 | −457 (0.03×) | 16.0 vs halo2curves 238 | −222 (0.07×) |
| sub_u64, constant time | P-256 base | 64.0 vs k256/p256 266 | −202 (0.24×) | 33.4 vs k256/p256 118 | −84.6 (0.28×) | 73.2 vs k256/p256 483 | −410 (0.15×) | 53.1 vs k256/p256 149 | −95.9 (0.36×) |
| sub_u64, constant time | P-256 scalar | 97.1 vs k256/p256 22.6 | +74.5 (4.30×) | 39.0 vs k256/p256 22.5 | +16.5 (1.73×) | 107 vs k256/p256 25.0 | +82.0 (4.28×) | 67.6 vs k256/p256 25.0 | +42.6 (2.70×) |
| sub_u64, constant time | Tom-256 base | 91.5 vs crypto-bigint 495 | −404 (0.18×) | 38.1 vs crypto-bigint 188 | −150 (0.20×) | 105 vs halo2curves 689 | −584 (0.15×) | 65.6 vs crypto-bigint 285 | −219 (0.23×) |
| sub_u64, constant time | BLS12-381 base | 137 vs crypto-bigint 1,089 | −952 (0.13×) | 59.5 vs halo2curves 337 | −278 (0.18×) | 177 vs crypto-bigint 1,360 | −1,183 (0.13×) | 104 vs halo2curves 621 | −517 (0.17×) |
| sub_u64, constant time | P-521 base | 44.7 vs crypto-bigint 1,666 | −1,621 (0.03×) | 27.9 vs crypto-bigint 630 | −602 (0.04×) | 49.0 vs crypto-bigint 2,178 | −2,129 (0.02×) | 40.2 vs crypto-bigint 1,161 | −1,121 (0.03×) |
| sub_uint, constant time | Mersenne-31 | 4.6 vs crypto-bigint 22.3 | −17.7 (0.21×) | 4.6 vs crypto-bigint 18.7 | −14.1 (0.25×) | 5.1 vs crypto-bigint 30.5 | −25.4 (0.17×) | 5.1 vs crypto-bigint 20.3 | −15.2 (0.25×) |
| sub_uint, constant time | BabyBear | 12.1 vs crypto-bigint 28.8 | −16.7 (0.42×) | 7.4 vs crypto-bigint 19.4 | −12.0 (0.38×) | 13.1 vs crypto-bigint 42.4 | −29.3 (0.31×) | 10.6 vs crypto-bigint 21.8 | −11.2 (0.49×) |
| sub_uint, constant time | Goldilocks | 8.9 vs winter-math 11.4 | −2.5 (0.78×) | 6.3 vs winter-math 6.2 | +0.1 (1.02×) | 11.4 vs winter-math 18.1 | −6.7 (0.63×) | 9.2 vs winter-math 8.5 | +0.7 (1.08×) |
| sub_uint, constant time | BN254 scalar | 243 vs halo2curves 400 | −157 (0.61×) | 96.9 vs halo2curves 105 | −8.1 (0.92×) | 336 vs halo2curves 652 | −316 (0.52×) | 173 vs halo2curves 194 | −21.0 (0.89×) |
| sub_uint, constant time | secp256k1 base | 33.3 vs halo2curves 211 | −178 (0.16×) | 21.1 vs halo2curves 79.7 | −58.6 (0.26×) | 32.0 vs halo2curves 400 | −368 (0.08×) | 35.4 vs halo2curves 130 | −94.6 (0.27×) |
| sub_uint, constant time | P-256 base | 175 vs halo2curves 234 | −59.0 (0.75×) | 72.3 vs halo2curves 85.3 | −13.0 (0.85×) | 269 vs halo2curves 498 | −229 (0.54×) | 134 vs halo2curves 143 | −9.0 (0.94×) |
| sub_uint, constant time | P-256 scalar | 253 vs k256/p256 67.1 | +186 (3.77×) | 98.7 vs k256/p256 66.7 | +32.0 (1.48×) | 323 vs k256/p256 96.0 | +227 (3.36×) | 178 vs k256/p256 95.5 | +82.5 (1.86×) |
| sub_uint, constant time | Tom-256 base | 249 vs halo2curves 365 | −116 (0.68×) | 95.6 vs halo2curves 103 | −7.4 (0.93×) | 321 vs halo2curves 612 | −291 (0.52×) | 172 vs halo2curves 188 | −16.0 (0.91×) |
| sub_uint, constant time | BLS12-381 base | 760 vs halo2curves 947 | −187 (0.80×) | 197 vs halo2curves 216 | −19.0 (0.91×) | 950 vs crypto-bigint 1,375 | −425 (0.69×) | 377 vs halo2curves 412 | −35.0 (0.92×) |
| sub_uint, constant time | P-521 base | 122 vs crypto-bigint 1,664 | −1,542 (0.07×) | 66.1 vs crypto-bigint 627 | −561 (0.11×) | 120 vs crypto-bigint 2,186 | −2,066 (0.05×) | 109 vs crypto-bigint 1,161 | −1,052 (0.09×) |
| div_u64, constant time | BN254 scalar | 12,333 vs crypto-bigint 30,054 | −17,721 (0.41×) | 9,408 vs crypto-bigint 25,960 | −16,552 (0.36×) | 17,290 vs crypto-bigint 38,009 | −20,719 (0.45×) | 12,886 vs crypto-bigint 30,510 | −17,624 (0.42×) |
| div_u64, constant time | secp256k1 base | 11,899 vs crypto-bigint 30,078 | −18,179 (0.40×) | 9,069 vs crypto-bigint 26,032 | −16,963 (0.35×) | 16,801 vs crypto-bigint 37,907 | −21,106 (0.44×) | 12,797 vs crypto-bigint 30,538 | −17,741 (0.42×) |
| div_u64, constant time | P-256 base | 11,795 vs crypto-bigint 30,097 | −18,302 (0.39×) | 9,138 vs k256/p256 22,521 | −13,383 (0.41×) | 17,094 vs crypto-bigint 37,914 | −20,820 (0.45×) | 12,485 vs crypto-bigint 30,579 | −18,094 (0.41×) |
| div_u64, constant time | P-256 scalar | 12,226 vs crypto-bigint 30,093 | −17,867 (0.41×) | 9,364 vs crypto-bigint 25,964 | −16,600 (0.36×) | 17,559 vs crypto-bigint 37,857 | −20,298 (0.46×) | 13,154 vs crypto-bigint 30,563 | −17,409 (0.43×) |
| div_u64, constant time | Tom-256 base | 12,170 vs crypto-bigint 30,105 | −17,935 (0.40×) | 9,302 vs crypto-bigint 26,008 | −16,706 (0.36×) | 16,864 vs crypto-bigint 37,892 | −21,028 (0.45×) | 12,887 vs crypto-bigint 30,585 | −17,698 (0.42×) |
| div_u64, constant time | BLS12-381 base | 24,651 vs crypto-bigint 50,896 | −26,245 (0.48×) | 16,807 vs crypto-bigint 43,208 | −26,401 (0.39×) | 34,582 vs crypto-bigint 68,188 | −33,606 (0.51×) | 24,196 vs crypto-bigint 53,401 | −29,205 (0.45×) |
| div_u64, constant time | P-521 base | 42,815 vs crypto-bigint 104,235 | −61,420 (0.41×) | 26,209 vs crypto-bigint 83,452 | −57,243 (0.31×) | 64,651 vs crypto-bigint 128,016 | −63,365 (0.51×) | 43,315 vs crypto-bigint 100,135 | −56,820 (0.43×) |
| div_uint, constant time | Mersenne-31 | 226 vs runtime-algebra 3,522 | −3,296 (0.06×) | 226 vs runtime-algebra 3,539 | −3,313 (0.06×) | 227 vs runtime-algebra 3,449 | −3,222 (0.07×) | 227 vs runtime-algebra 3,420 | −3,193 (0.07×) |
| div_uint, constant time | BabyBear | 469 vs runtime-algebra 3,618 | −3,149 (0.13×) | 374 vs runtime-algebra 3,550 | −3,176 (0.11×) | 524 vs runtime-algebra 3,452 | −2,928 (0.15×) | 441 vs runtime-algebra 3,420 | −2,979 (0.13×) |
| div_uint, constant time | Goldilocks | 1,120 vs winter-math 1,788 | −668 (0.63×) | 811 vs winter-math 927 | −116 (0.87×) | 1,368 vs winter-math 1,895 | −527 (0.72×) | 943 vs winter-math 1,033 | −90.0 (0.91×) |
| div_uint, constant time | BN254 scalar | 12,458 vs crypto-bigint 29,352 | −16,894 (0.42×) | 9,486 vs crypto-bigint 26,269 | −16,783 (0.36×) | 17,629 vs crypto-bigint 37,909 | −20,280 (0.47×) | 13,264 vs crypto-bigint 30,634 | −17,370 (0.43×) |
| div_uint, constant time | secp256k1 base | 11,875 vs crypto-bigint 29,381 | −17,506 (0.40×) | 9,058 vs crypto-bigint 26,267 | −17,209 (0.34×) | 16,873 vs crypto-bigint 37,936 | −21,063 (0.44×) | 12,900 vs crypto-bigint 30,528 | −17,628 (0.42×) |
| div_uint, constant time | P-256 base | 12,004 vs crypto-bigint 29,391 | −17,387 (0.41×) | 9,094 vs k256/p256 22,485 | −13,391 (0.40×) | 17,366 vs crypto-bigint 37,951 | −20,585 (0.46×) | 12,491 vs crypto-bigint 30,662 | −18,171 (0.41×) |
| div_uint, constant time | P-256 scalar | 12,406 vs crypto-bigint 29,378 | −16,972 (0.42×) | 9,535 vs crypto-bigint 26,265 | −16,730 (0.36×) | 17,784 vs crypto-bigint 37,939 | −20,155 (0.47×) | 13,340 vs crypto-bigint 30,578 | −17,238 (0.44×) |
| div_uint, constant time | Tom-256 base | 12,340 vs crypto-bigint 29,434 | −17,094 (0.42×) | 9,436 vs crypto-bigint 26,278 | −16,842 (0.36×) | 17,061 vs crypto-bigint 37,827 | −20,766 (0.45×) | 13,108 vs crypto-bigint 30,602 | −17,494 (0.43×) |
| div_uint, constant time | BLS12-381 base | 25,228 vs crypto-bigint 51,027 | −25,799 (0.49×) | 16,946 vs crypto-bigint 43,331 | −26,385 (0.39×) | 35,362 vs crypto-bigint 68,282 | −32,920 (0.52×) | 24,616 vs crypto-bigint 53,827 | −29,211 (0.46×) |
| div_uint, constant time | P-521 base | 42,912 vs crypto-bigint 104,674 | −61,762 (0.41×) | 26,024 vs crypto-bigint 83,085 | −57,061 (0.31×) | 64,672 vs crypto-bigint 127,879 | −63,207 (0.51×) | 43,343 vs crypto-bigint 100,526 | −57,183 (0.43×) |

### Against native, throughput

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 1.9 | 2.8 (1.5×) | 2.8 (1.5×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add | BabyBear | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add | Goldilocks | 2.0 | 3.5 (1.8×) | 3.5 (1.8×) | 3.1 (1.6×) | 3.1 (1.6×) |
| add | BN254 scalar | 4.9 | 14.6 (3.0×) | 12.4 (2.5×) | 13.7 (2.8×) | 15.6 (3.2×) |
| add | secp256k1 base | 5.9 | 15.0 (2.5×) | 11.9 (2.0×) | 14.4 (2.4×) | 18.2 (3.1×) |
| add | P-256 base | 5.9 | 13.9 (2.4×) | 12.3 (2.1×) | 14.1 (2.4×) | 18.0 (3.1×) |
| add | P-256 scalar | 5.8 | 15.2 (2.6×) | 13.2 (2.3×) | 15.0 (2.6×) | 18.6 (3.2×) |
| add | Tom-256 base | 6.0 | 14.1 (2.4×) | 12.4 (2.1×) | 14.0 (2.3×) | 17.7 (2.9×) |
| add | BLS12-381 base | 9.6 | 25.1 (2.6×) | 19.6 (2.0×) | 23.5 (2.4×) | 24.5 (2.6×) |
| add | P-521 base | 12.2 | 44.8 (3.7×) | 28.8 (2.4×) | 44.2 (3.6×) | 45.5 (3.7×) |
| sub | Mersenne-31 | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.6 (1.4×) | 2.5 (1.3×) |
| sub | BabyBear | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.6 (1.4×) |
| sub | Goldilocks | 1.9 | 2.8 (1.5×) | 2.8 (1.5×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub | BN254 scalar | 5.0 | 15.9 (3.2×) | 13.0 (2.6×) | 13.4 (2.7×) | 17.3 (3.5×) |
| sub | secp256k1 base | 5.2 | 17.5 (3.4×) | 12.6 (2.4×) | 14.2 (2.7×) | 17.4 (3.3×) |
| sub | P-256 base | 5.1 | 15.7 (3.1×) | 12.0 (2.4×) | 13.2 (2.6×) | 17.0 (3.3×) |
| sub | P-256 scalar | 5.0 | 16.3 (3.3×) | 12.6 (2.5×) | 14.4 (2.9×) | 17.2 (3.4×) |
| sub | Tom-256 base | 5.5 | 15.9 (2.9×) | 13.0 (2.4×) | 13.7 (2.5×) | 17.0 (3.1×) |
| sub | BLS12-381 base | 9.7 | 24.0 (2.5×) | 20.4 (2.1×) | 21.0 (2.2×) | 27.0 (2.8×) |
| sub | P-521 base | 11.7 | 42.7 (3.6×) | 28.8 (2.5×) | 42.8 (3.7×) | 42.1 (3.6×) |
| mul | Mersenne-31 | 2.7 | 3.7 (1.4×) | 3.7 (1.4×) | 3.6 (1.3×) | 3.5 (1.3×) |
| mul | BabyBear | 3.4 | 5.2 (1.5×) | 3.7 (1.1×) | 5.4 (1.6×) | 5.9 (1.7×) |
| mul | Goldilocks | 3.8 | 11.9 (3.1×) | 6.0 (1.6×) | 13.8 (3.6×) | 6.8 (1.8×) |
| mul | BN254 scalar | 68.6 | 231 (3.4×) | 85.1 (1.2×) | 381 (5.6×) | 146 (2.1×) |
| mul | secp256k1 base | 44.5 | 160 (3.6×) | 58.8 (1.3×) | 226 (5.1×) | 105 (2.4×) |
| mul | P-256 base | 49.6 | 178 (3.6×) | 63.0 (1.3×) | 271 (5.5×) | 119 (2.4×) |
| mul | P-256 scalar | 74.9 | 241 (3.2×) | 87.4 (1.2×) | 315 (4.2×) | 156 (2.1×) |
| mul | Tom-256 base | 70.9 | 241 (3.4×) | 85.8 (1.2×) | 316 (4.5×) | 151 (2.1×) |
| mul | BLS12-381 base | 138 | 714 (5.2×) | 185 (1.3×) | 883 (6.4×) | 334 (2.4×) |
| mul | P-521 base | 176 | 669 (3.8×) | 244 (1.4×) | 1,323 (7.5×) | 433 (2.5×) |
| square | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 3.4 (1.3×) | 3.4 (1.3×) |
| square | BabyBear | 3.4 | 5.1 (1.5×) | 3.4 (1.0×) | 5.4 (1.6×) | 4.6 (1.4×) |
| square | Goldilocks | 3.8 | 9.5 (2.5×) | 6.0 (1.6×) | 13.0 (3.4×) | 6.8 (1.8×) |
| square | BN254 scalar | 63.2 | 221 (3.5×) | 76.8 (1.2×) | 340 (5.4×) | 132 (2.1×) |
| square | secp256k1 base | 33.9 | 142 (4.2×) | 47.5 (1.4×) | 189 (5.6×) | 85.8 (2.5×) |
| square | P-256 base | 39.7 | 154 (3.9×) | 55.3 (1.4×) | 233 (5.9×) | 102 (2.6×) |
| square | P-256 scalar | 63.9 | 220 (3.4×) | 78.0 (1.2×) | 291 (4.6×) | 134 (2.1×) |
| square | Tom-256 base | 57.0 | 210 (3.7×) | 74.2 (1.3×) | 277 (4.9×) | 129 (2.3×) |
| square | BLS12-381 base | 131 | 553 (4.2×) | 157 (1.2×) | 813 (6.2×) | 317 (2.4×) |
| square | P-521 base | 118 | 556 (4.7×) | 203 (1.7×) | 914 (7.7×) | 381 (3.2×) |
| invert | Mersenne-31 | 221 | 266 (1.2×) | 266 (1.2×) | 277 (1.3×) | 271 (1.2×) |
| invert | BabyBear | 210 | 269 (1.3×) | 269 (1.3×) | 274 (1.3×) | 280 (1.3×) |
| invert | Goldilocks | 434 | 539 (1.2×) | 540 (1.2×) | 567 (1.3×) | 563 (1.3×) |
| invert | BN254 scalar | 5,041 | 8,459 (1.7×) | 6,468 (1.3×) | 11,937 (2.4×) | 7,922 (1.6×) |
| invert | secp256k1 base | 4,922 | 8,498 (1.7×) | 6,026 (1.2×) | 11,695 (2.4×) | 7,981 (1.6×) |
| invert | P-256 base | 4,833 | 8,121 (1.7×) | 6,069 (1.3×) | 11,769 (2.4×) | 7,686 (1.6×) |
| invert | P-256 scalar | 5,033 | 8,527 (1.7×) | 6,384 (1.3×) | 11,721 (2.3×) | 8,063 (1.6×) |
| invert | Tom-256 base | 5,024 | 8,765 (1.7×) | 6,455 (1.3×) | 11,502 (2.3×) | 7,946 (1.6×) |
| invert | BLS12-381 base | 8,807 | 19,342 (2.2×) | 11,777 (1.3×) | 25,202 (2.9×) | 15,431 (1.8×) |
| invert | P-521 base | 14,723 | 33,489 (2.3×) | 18,076 (1.2×) | 47,853 (3.3×) | 29,358 (2.0×) |
| sqrt | Mersenne-31 | 135 | 160 (1.2×) | 160 (1.2×) | 162 (1.2×) | 163 (1.2×) |
| sqrt | BabyBear | 1,971 | 2,340 (1.2×) | 1,959 (1.0×) | 2,555 (1.3×) | 2,381 (1.2×) |
| sqrt | Goldilocks | 2,829 | 4,854 (1.7×) | 3,955 (1.4×) | 5,966 (2.1×) | 4,503 (1.6×) |
| sqrt | BN254 scalar | 33,510 | 104,669 (3.1×) | 47,254 (1.4×) | 166,300 (5.0×) | 80,543 (2.4×) |
| sqrt | secp256k1 base | 12,289 | 41,129 (3.3×) | 16,750 (1.4×) | 65,663 (5.3×) | 24,059 (2.0×) |
| sqrt | P-256 base | 11,790 | 43,055 (3.7×) | 18,745 (1.6×) | 74,559 (6.3×) | 29,394 (2.5×) |
| sqrt | P-256 scalar | 24,058 | 73,944 (3.1×) | 30,038 (1.2×) | 109,756 (4.6×) | 59,861 (2.5×) |
| sqrt | Tom-256 base | 21,265 | 66,189 (3.1×) | 25,634 (1.2×) | 85,847 (4.0×) | 49,758 (2.3×) |
| sqrt | BLS12-381 base | 65,953 | 296,985 (4.5×) | 90,304 (1.4×) | 390,419 (5.9×) | 213,166 (3.2×) |
| sqrt | P-521 base | 68,259 | 318,749 (4.7×) | 123,473 (1.8×) | 461,539 (6.8×) | 255,934 (3.7×) |
| mul_u64 | Mersenne-31 | 5.9 | 7.4 (1.3×) | 7.4 (1.3×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_u64 | BabyBear | 8.9 | 15.5 (1.7×) | 11.4 (1.3×) | 17.5 (2.0×) | 15.0 (1.7×) |
| mul_u64 | Goldilocks | 6.9 | 18.7 (2.7×) | 7.1 (1.0×) | 26.3 (3.8×) | 8.7 (1.3×) |
| mul_u64 | BN254 scalar | 20.7 | 94.1 (4.5×) | 30.8 (1.5×) | 120 (5.8×) | 43.2 (2.1×) |
| mul_u64 | secp256k1 base | 9.7 | 47.3 (4.9×) | 13.3 (1.4×) | 63.3 (6.5×) | 22.0 (2.3×) |
| mul_u64 | P-256 base | 18.8 | 73.7 (3.9×) | 28.3 (1.5×) | 94.1 (5.0×) | 41.2 (2.2×) |
| mul_u64 | P-256 scalar | 22.2 | 82.7 (3.7×) | 35.8 (1.6×) | 109 (4.9×) | 45.9 (2.1×) |
| mul_u64 | Tom-256 base | 20.4 | 83.3 (4.1×) | 30.1 (1.5×) | 108 (5.3×) | 45.0 (2.2×) |
| mul_u64 | BLS12-381 base | 28.6 | 136 (4.8×) | 43.0 (1.5×) | 177 (6.2×) | 61.5 (2.2×) |
| mul_u64 | P-521 base | 19.7 | 99.9 (5.1×) | 33.5 (1.7×) | 133 (6.8×) | 51.0 (2.6×) |
| mul_uint | Mersenne-31 | 5.9 | 7.4 (1.3×) | 7.4 (1.3×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_uint | BabyBear | 8.9 | 15.5 (1.7×) | 11.4 (1.3×) | 17.5 (2.0×) | 15.0 (1.7×) |
| mul_uint | Goldilocks | 6.9 | 18.7 (2.7×) | 7.1 (1.0×) | 26.3 (3.8×) | 8.7 (1.3×) |
| mul_uint | BN254 scalar | 103 | 434 (4.2×) | 180 (1.7×) | 540 (5.2×) | 296 (2.9×) |
| mul_uint | secp256k1 base | 57.0 | 257 (4.5×) | 64.9 (1.1×) | 327 (5.7×) | 105 (1.8×) |
| mul_uint | P-256 base | 102 | 345 (3.4×) | 129 (1.3×) | 431 (4.2×) | 229 (2.2×) |
| mul_uint | P-256 scalar | 110 | 425 (3.9×) | 190 (1.7×) | 484 (4.4×) | 308 (2.8×) |
| mul_uint | Tom-256 base | 108 | 411 (3.8×) | 181 (1.7×) | 487 (4.5×) | 297 (2.8×) |
| mul_uint | BLS12-381 base | 214 | 972 (4.5×) | 378 (1.8×) | 1,131 (5.3×) | 919 (4.3×) |
| mul_uint | P-521 base | 359 | 1,535 (4.3×) | 260 (0.7×) | 2,351 (6.5×) | 447 (1.2×) |
| add_u64 | Mersenne-31 | 4.6 | 5.4 (1.2×) | 5.4 (1.2×) | 5.5 (1.2×) | 5.5 (1.2×) |
| add_u64 | BabyBear | 5.9 | 13.0 (2.2×) | 9.0 (1.5×) | 12.9 (2.2×) | 12.2 (2.1×) |
| add_u64 | Goldilocks | 4.8 | 9.3 (1.9×) | 7.5 (1.6×) | 11.8 (2.5×) | 8.5 (1.8×) |
| add_u64 | BN254 scalar | 24.9 | 118 (4.7×) | 41.5 (1.7×) | 140 (5.6×) | 62.5 (2.5×) |
| add_u64 | secp256k1 base | 5.8 | 8.8 (1.5×) | 8.4 (1.4×) | 8.3 (1.4×) | 10.7 (1.8×) |
| add_u64 | P-256 base | 25.8 | 72.0 (2.8×) | 37.2 (1.4×) | 78.9 (3.1×) | 52.5 (2.0×) |
| add_u64 | P-256 scalar | 25.4 | 67.9 (2.7×) | 39.9 (1.6×) | 73.3 (2.9×) | 62.3 (2.5×) |
| add_u64 | Tom-256 base | 25.9 | 89.4 (3.5×) | 43.6 (1.7×) | 105 (4.1×) | 59.0 (2.3×) |
| add_u64 | BLS12-381 base | 38.2 | 153 (4.0×) | 62.1 (1.6×) | 248 (6.5×) | 84.2 (2.2×) |
| add_u64 | P-521 base | 10.8 | 22.3 (2.1×) | 18.5 (1.7×) | 18.5 (1.7×) | 22.8 (2.1×) |
| add_uint | Mersenne-31 | 4.6 | 5.4 (1.2×) | 5.4 (1.2×) | 5.5 (1.2×) | 5.5 (1.2×) |
| add_uint | BabyBear | 5.9 | 13.0 (2.2×) | 9.0 (1.5×) | 12.9 (2.2×) | 12.2 (2.1×) |
| add_uint | Goldilocks | 4.7 | 9.3 (2.0×) | 7.5 (1.6×) | 11.8 (2.5×) | 8.5 (1.8×) |
| add_uint | BN254 scalar | 71.9 | 250 (3.5×) | 101 (1.4×) | 357 (5.0×) | 177 (2.5×) |
| add_uint | secp256k1 base | 6.7 | 19.5 (2.9×) | 13.7 (2.0×) | 16.1 (2.4×) | 19.8 (3.0×) |
| add_uint | P-256 base | 57.2 | 182 (3.2×) | 76.2 (1.3×) | 324 (5.7×) | 141 (2.5×) |
| add_uint | P-256 scalar | 81.0 | 262 (3.2×) | 104 (1.3×) | 321 (4.0×) | 181 (2.2×) |
| add_uint | Tom-256 base | 73.8 | 257 (3.5×) | 104 (1.4×) | 327 (4.4×) | 175 (2.4×) |
| add_uint | BLS12-381 base | 151 | 767 (5.1×) | 208 (1.4×) | 952 (6.3×) | 379 (2.5×) |
| add_uint | P-521 base | 23.1 | 63.5 (2.7×) | 41.7 (1.8×) | 71.8 (3.1×) | 60.5 (2.6×) |
| sub_u64 | Mersenne-31 | 4.7 | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) |
| sub_u64 | BabyBear | 5.8 | 11.1 (1.9×) | 9.0 (1.6×) | 13.1 (2.3×) | 11.9 (2.1×) |
| sub_u64 | Goldilocks | 4.3 | 9.4 (2.2×) | 7.4 (1.7×) | 11.6 (2.7×) | 8.5 (2.0×) |
| sub_u64 | BN254 scalar | 24.5 | 112 (4.6×) | 45.0 (1.8×) | 137 (5.6×) | 63.3 (2.6×) |
| sub_u64 | secp256k1 base | 4.0 | 9.4 (2.4×) | 8.6 (2.1×) | 7.8 (1.9×) | 9.4 (2.4×) |
| sub_u64 | P-256 base | 20.2 | 72.3 (3.6×) | 36.0 (1.8×) | 76.5 (3.8×) | 50.6 (2.5×) |
| sub_u64 | P-256 scalar | 24.8 | 67.2 (2.7×) | 36.8 (1.5×) | 71.4 (2.9×) | 58.7 (2.4×) |
| sub_u64 | Tom-256 base | 24.7 | 88.5 (3.6×) | 42.3 (1.7×) | 103 (4.2×) | 59.9 (2.4×) |
| sub_u64 | BLS12-381 base | 37.8 | 161 (4.3×) | 62.7 (1.7×) | 238 (6.3×) | 86.1 (2.3×) |
| sub_u64 | P-521 base | 9.5 | 24.3 (2.6×) | 19.1 (2.0×) | 21.3 (2.2×) | 20.0 (2.1×) |
| sub_uint | Mersenne-31 | 4.7 | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) |
| sub_uint | BabyBear | 5.9 | 11.1 (1.9×) | 9.0 (1.5×) | 13.1 (2.2×) | 11.9 (2.0×) |
| sub_uint | Goldilocks | 4.3 | 9.4 (2.2×) | 7.4 (1.7×) | 11.6 (2.7×) | 8.5 (2.0×) |
| sub_uint | BN254 scalar | 73.7 | 249 (3.4×) | 99.4 (1.3×) | 371 (5.0×) | 173 (2.3×) |
| sub_uint | secp256k1 base | 5.4 | 19.9 (3.7×) | 14.3 (2.6×) | 15.4 (2.9×) | 19.6 (3.6×) |
| sub_uint | P-256 base | 53.7 | 174 (3.2×) | 75.1 (1.4×) | 280 (5.2×) | 134 (2.5×) |
| sub_uint | P-256 scalar | 80.0 | 248 (3.1×) | 101 (1.3×) | 322 (4.0×) | 174 (2.2×) |
| sub_uint | Tom-256 base | 75.4 | 250 (3.3×) | 103 (1.4×) | 323 (4.3×) | 171 (2.3×) |
| sub_uint | BLS12-381 base | 155 | 763 (4.9×) | 202 (1.3×) | 950 (6.1×) | 374 (2.4×) |
| sub_uint | P-521 base | 20.2 | 68.4 (3.4×) | 40.0 (2.0×) | 69.8 (3.5×) | 61.8 (3.1×) |
| from_u64 | Mersenne-31 | 3.2 | 4.0 (1.2×) | 4.0 (1.2×) | 3.8 (1.2×) | 3.8 (1.2×) |
| from_u64 | BabyBear | 6.3 | 11.9 (1.9×) | 7.5 (1.2×) | 11.0 (1.7×) | 10.0 (1.6×) |
| from_u64 | Goldilocks | 3.3 | 7.9 (2.4×) | 5.6 (1.7×) | 8.6 (2.6×) | 6.9 (2.1×) |
| from_u64 | BN254 scalar | 22.0 | 104 (4.7×) | 34.1 (1.6×) | 124 (5.6×) | 49.1 (2.2×) |
| from_u64 | secp256k1 base | 2.0 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.6×) | 3.2 (1.6×) |
| from_u64 | P-256 base | 18.0 | 58.1 (3.2×) | 29.0 (1.6×) | 70.1 (3.9×) | 41.3 (2.3×) |
| from_u64 | P-256 scalar | 18.0 | 55.4 (3.1×) | 29.2 (1.6×) | 60.2 (3.3×) | 47.1 (2.6×) |
| from_u64 | Tom-256 base | 21.0 | 81.1 (3.9×) | 32.2 (1.5×) | 86.9 (4.1×) | 48.1 (2.3×) |
| from_u64 | BLS12-381 base | 29.1 | 147 (5.1×) | 51.7 (1.8×) | 182 (6.3×) | 67.1 (2.3×) |
| from_u64 | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64 | BN254 scalar | 609 | 1,205 (2.0×) | 861 (1.4×) | 1,374 (2.3×) | 1,092 (1.8×) |
| div_u64 | secp256k1 base | 565 | 992 (1.8×) | 709 (1.3×) | 1,220 (2.2×) | 804 (1.4×) |
| div_u64 | P-256 base | 602 | 1,080 (1.8×) | 798 (1.3×) | 1,341 (2.2×) | 987 (1.6×) |
| div_u64 | P-256 scalar | 618 | 1,182 (1.9×) | 864 (1.4×) | 1,444 (2.3×) | 1,119 (1.8×) |
| div_u64 | Tom-256 base | 609 | 1,192 (2.0×) | 867 (1.4×) | 1,461 (2.4×) | 1,086 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,780 (2.4×) | 1,100 (1.5×) | 2,461 (3.3×) | 1,757 (2.3×) |
| div_u64 | P-521 base | 922 | 2,425 (2.6×) | 990 (1.1×) | 3,597 (3.9×) | 1,489 (1.6×) |
| div_uint | Mersenne-31 | 224 | 273 (1.2×) | 273 (1.2×) | 279 (1.2×) | 279 (1.2×) |
| div_uint | BabyBear | 214 | 276 (1.3×) | 279 (1.3×) | 280 (1.3×) | 283 (1.3×) |
| div_uint | Goldilocks | 435 | 552 (1.3×) | 544 (1.3×) | 586 (1.3×) | 567 (1.3×) |
| div_uint | BN254 scalar | 4,983 | 9,142 (1.8×) | 6,657 (1.3×) | 12,687 (2.5×) | 8,439 (1.7×) |
| div_uint | secp256k1 base | 4,886 | 8,531 (1.7×) | 6,054 (1.2×) | 12,135 (2.5×) | 8,101 (1.7×) |
| div_uint | P-256 base | 5,031 | 8,781 (1.7×) | 6,171 (1.2×) | 12,190 (2.4×) | 7,832 (1.6×) |
| div_uint | P-256 scalar | 4,964 | 9,135 (1.8×) | 6,501 (1.3×) | 12,571 (2.5×) | 8,614 (1.7×) |
| div_uint | Tom-256 base | 5,056 | 8,983 (1.8×) | 6,660 (1.3×) | 12,416 (2.5×) | 8,412 (1.7×) |
| div_uint | BLS12-381 base | 9,044 | 20,754 (2.3×) | 12,071 (1.3×) | 26,417 (2.9×) | 16,860 (1.9×) |
| div_uint | P-521 base | 14,901 | 34,607 (2.3×) | 18,467 (1.2×) | 49,642 (3.3×) | 30,158 (2.0×) |
| add, constant time | Mersenne-31 | 1.9 | 2.8 (1.5×) | 2.8 (1.5×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add, constant time | BabyBear | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.7 (1.4×) | 2.7 (1.4×) |
| add, constant time | Goldilocks | 2.0 | 3.5 (1.8×) | 3.5 (1.8×) | 3.1 (1.6×) | 3.1 (1.6×) |
| add, constant time | BN254 scalar | 4.9 | 24.8 (5.1×) | 16.2 (3.3×) | 21.6 (4.4×) | 25.1 (5.1×) |
| add, constant time | secp256k1 base | 6.0 | 28.0 (4.7×) | 16.3 (2.7×) | 25.7 (4.3×) | 27.4 (4.6×) |
| add, constant time | P-256 base | 5.8 | 24.5 (4.2×) | 16.0 (2.8×) | 24.4 (4.2×) | 26.6 (4.6×) |
| add, constant time | P-256 scalar | 5.8 | 27.8 (4.8×) | 16.6 (2.9×) | 26.7 (4.6×) | 28.1 (4.8×) |
| add, constant time | Tom-256 base | 5.8 | 27.4 (4.7×) | 16.3 (2.8×) | 26.8 (4.6×) | 28.1 (4.8×) |
| add, constant time | BLS12-381 base | 9.6 | 44.3 (4.6×) | 24.2 (2.5×) | 38.2 (4.0×) | 38.6 (4.0×) |
| add, constant time | P-521 base | 12.2 | 75.1 (6.2×) | 40.7 (3.3×) | 76.7 (6.3×) | 71.1 (5.8×) |
| sub, constant time | Mersenne-31 | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.6 (1.4×) | 2.5 (1.3×) |
| sub, constant time | BabyBear | 1.9 | 2.6 (1.4×) | 2.6 (1.4×) | 2.8 (1.5×) | 2.6 (1.4×) |
| sub, constant time | Goldilocks | 1.9 | 2.8 (1.5×) | 2.8 (1.5×) | 2.7 (1.4×) | 2.7 (1.4×) |
| sub, constant time | BN254 scalar | 4.9 | 21.9 (4.5×) | 14.6 (3.0×) | 21.1 (4.3×) | 23.1 (4.7×) |
| sub, constant time | secp256k1 base | 5.3 | 20.9 (3.9×) | 13.8 (2.6×) | 18.5 (3.5×) | 21.1 (4.0×) |
| sub, constant time | P-256 base | 5.3 | 18.7 (3.5×) | 12.9 (2.4×) | 18.0 (3.4×) | 20.0 (3.8×) |
| sub, constant time | P-256 scalar | 5.0 | 21.2 (4.2×) | 14.1 (2.8×) | 21.5 (4.3×) | 22.0 (4.4×) |
| sub, constant time | Tom-256 base | 5.5 | 23.0 (4.2×) | 14.2 (2.6×) | 20.8 (3.8×) | 22.6 (4.1×) |
| sub, constant time | BLS12-381 base | 9.7 | 38.1 (3.9×) | 23.1 (2.4×) | 36.6 (3.8×) | 35.6 (3.7×) |
| sub, constant time | P-521 base | 11.7 | 56.5 (4.8×) | 33.5 (2.9×) | 56.3 (4.8×) | 50.1 (4.3×) |
| mul, constant time | Mersenne-31 | 2.7 | 3.7 (1.4×) | 3.7 (1.4×) | 3.6 (1.3×) | 3.5 (1.3×) |
| mul, constant time | BabyBear | 3.5 | 5.2 (1.5×) | 3.7 (1.1×) | 5.4 (1.5×) | 5.9 (1.7×) |
| mul, constant time | Goldilocks | 3.8 | 11.8 (3.1×) | 6.0 (1.6×) | 13.7 (3.6×) | 6.8 (1.8×) |
| mul, constant time | BN254 scalar | 68.0 | 240 (3.5×) | 85.1 (1.3×) | 406 (6.0×) | 147 (2.2×) |
| mul, constant time | secp256k1 base | 44.7 | 160 (3.6×) | 58.9 (1.3×) | 221 (4.9×) | 105 (2.3×) |
| mul, constant time | P-256 base | 49.5 | 178 (3.6×) | 62.9 (1.3×) | 271 (5.5×) | 119 (2.4×) |
| mul, constant time | P-256 scalar | 74.9 | 241 (3.2×) | 87.4 (1.2×) | 314 (4.2×) | 158 (2.1×) |
| mul, constant time | Tom-256 base | 71.0 | 243 (3.4×) | 85.8 (1.2×) | 316 (4.5×) | 151 (2.1×) |
| mul, constant time | BLS12-381 base | 141 | 715 (5.1×) | 185 (1.3×) | 888 (6.3×) | 331 (2.3×) |
| mul, constant time | P-521 base | 176 | 668 (3.8×) | 244 (1.4×) | 1,321 (7.5×) | 432 (2.5×) |
| square, constant time | Mersenne-31 | 2.7 | 3.6 (1.3×) | 3.6 (1.3×) | 3.4 (1.3×) | 3.4 (1.3×) |
| square, constant time | BabyBear | 3.4 | 5.0 (1.5×) | 3.4 (1.0×) | 5.4 (1.6×) | 4.6 (1.4×) |
| square, constant time | Goldilocks | 3.8 | 9.5 (2.5×) | 6.0 (1.6×) | 12.9 (3.4×) | 6.7 (1.8×) |
| square, constant time | BN254 scalar | 64.9 | 223 (3.4×) | 76.9 (1.2×) | 296 (4.6×) | 132 (2.0×) |
| square, constant time | secp256k1 base | 33.8 | 142 (4.2×) | 47.5 (1.4×) | 189 (5.6×) | 85.5 (2.5×) |
| square, constant time | P-256 base | 40.3 | 154 (3.8×) | 55.3 (1.4×) | 234 (5.8×) | 103 (2.6×) |
| square, constant time | P-256 scalar | 63.9 | 218 (3.4×) | 79.1 (1.2×) | 292 (4.6×) | 135 (2.1×) |
| square, constant time | Tom-256 base | 57.0 | 210 (3.7×) | 74.1 (1.3×) | 273 (4.8×) | 129 (2.3×) |
| square, constant time | BLS12-381 base | 134 | 554 (4.1×) | 157 (1.2×) | 815 (6.1×) | 317 (2.4×) |
| square, constant time | P-521 base | 118 | 559 (4.7×) | 204 (1.7×) | 917 (7.8×) | 381 (3.2×) |
| invert, constant time | Mersenne-31 | 248 | 209 (0.8×) | 209 (0.8×) | 211 (0.9×) | 209 (0.8×) |
| invert, constant time | BabyBear | 383 | 449 (1.2×) | 351 (0.9×) | 485 (1.3×) | 415 (1.1×) |
| invert, constant time | Goldilocks | 646 | 1,090 (1.7×) | 775 (1.2×) | 1,346 (2.1×) | 897 (1.4×) |
| invert, constant time | BN254 scalar | 6,761 | 11,892 (1.8×) | 9,425 (1.4×) | 16,902 (2.5×) | 12,573 (1.9×) |
| invert, constant time | secp256k1 base | 6,858 | 11,779 (1.7×) | 8,956 (1.3×) | 16,561 (2.4×) | 12,676 (1.8×) |
| invert, constant time | P-256 base | 6,516 | 11,536 (1.8×) | 9,029 (1.4×) | 16,390 (2.5×) | 12,280 (1.9×) |
| invert, constant time | P-256 scalar | 6,911 | 11,874 (1.7×) | 9,414 (1.4×) | 17,018 (2.5×) | 12,825 (1.9×) |
| invert, constant time | Tom-256 base | 6,985 | 11,864 (1.7×) | 9,191 (1.3×) | 16,644 (2.4×) | 12,891 (1.8×) |
| invert, constant time | BLS12-381 base | 12,456 | 23,941 (1.9×) | 16,479 (1.3×) | 33,467 (2.7×) | 24,433 (2.0×) |
| invert, constant time | P-521 base | 19,938 | 41,931 (2.1×) | 25,872 (1.3×) | 63,129 (3.2×) | 41,698 (2.1×) |
| sqrt, constant time | Mersenne-31 | 142 | 159 (1.1×) | 159 (1.1×) | 170 (1.2×) | 169 (1.2×) |
| sqrt, constant time | BabyBear | 5,592 | 6,226 (1.1×) | 6,351 (1.1×) | 7,424 (1.3×) | 9,099 (1.6×) |
| sqrt, constant time | Goldilocks | 7,152 | 10,899 (1.5×) | 8,281 (1.2×) | 14,075 (2.0×) | 10,357 (1.4×) |
| sqrt, constant time | BN254 scalar | 58,556 | 179,977 (3.1×) | 74,303 (1.3×) | 292,908 (5.0×) | 136,322 (2.3×) |
| sqrt, constant time | secp256k1 base | 12,274 | 40,950 (3.3×) | 16,311 (1.3×) | 64,212 (5.2×) | 26,969 (2.2×) |
| sqrt, constant time | P-256 base | 12,039 | 43,215 (3.6×) | 17,364 (1.4×) | 78,056 (6.5×) | 31,002 (2.6×) |
| sqrt, constant time | P-256 scalar | 25,620 | 77,702 (3.0×) | 31,388 (1.2×) | 114,043 (4.5×) | 60,595 (2.4×) |
| sqrt, constant time | Tom-256 base | 20,051 | 65,822 (3.3×) | 26,921 (1.3×) | 101,342 (5.1×) | 49,422 (2.5×) |
| sqrt, constant time | BLS12-381 base | 69,785 | 296,743 (4.3×) | 90,254 (1.3×) | 390,900 (5.6×) | 213,009 (3.1×) |
| sqrt, constant time | P-521 base | 68,436 | 320,738 (4.7×) | 124,536 (1.8×) | 466,219 (6.8×) | 256,351 (3.7×) |
| mul_u64, constant time | Mersenne-31 | 5.9 | 7.4 (1.3×) | 7.4 (1.3×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_u64, constant time | BabyBear | 8.9 | 15.5 (1.7×) | 11.4 (1.3×) | 17.5 (2.0×) | 15.0 (1.7×) |
| mul_u64, constant time | Goldilocks | 7.0 | 18.7 (2.7×) | 7.1 (1.0×) | 26.3 (3.8×) | 8.7 (1.2×) |
| mul_u64, constant time | BN254 scalar | 21.9 | 106 (4.8×) | 38.9 (1.8×) | 124 (5.7×) | 55.3 (2.5×) |
| mul_u64, constant time | secp256k1 base | 11.0 | 60.3 (5.5×) | 22.2 (2.0×) | 77.1 (7.0×) | 35.4 (3.2×) |
| mul_u64, constant time | P-256 base | 24.4 | 82.9 (3.4×) | 38.1 (1.6×) | 99.9 (4.1×) | 54.6 (2.2×) |
| mul_u64, constant time | P-256 scalar | 22.8 | 97.1 (4.3×) | 41.7 (1.8×) | 117 (5.1×) | 60.6 (2.7×) |
| mul_u64, constant time | Tom-256 base | 22.5 | 101 (4.5×) | 41.7 (1.9×) | 120 (5.3×) | 59.7 (2.7×) |
| mul_u64, constant time | BLS12-381 base | 34.6 | 150 (4.3×) | 63.3 (1.8×) | 192 (5.5×) | 82.6 (2.4×) |
| mul_u64, constant time | P-521 base | 26.3 | 138 (5.2×) | 64.3 (2.4×) | 172 (6.5×) | 88.4 (3.4×) |
| mul_uint, constant time | Mersenne-31 | 5.9 | 7.4 (1.3×) | 7.4 (1.3×) | 7.6 (1.3×) | 7.6 (1.3×) |
| mul_uint, constant time | BabyBear | 8.9 | 15.5 (1.7×) | 11.4 (1.3×) | 17.5 (2.0×) | 15.0 (1.7×) |
| mul_uint, constant time | Goldilocks | 6.9 | 18.7 (2.7×) | 7.1 (1.0×) | 26.3 (3.8×) | 8.7 (1.3×) |
| mul_uint, constant time | BN254 scalar | 120 | 479 (4.0×) | 180 (1.5×) | 598 (5.0×) | 296 (2.5×) |
| mul_uint, constant time | secp256k1 base | 78.0 | 316 (4.1×) | 68.1 (0.9×) | 387 (5.0×) | 115 (1.5×) |
| mul_uint, constant time | P-256 base | 118 | 405 (3.4×) | 125 (1.1×) | 488 (4.1×) | 231 (2.0×) |
| mul_uint, constant time | P-256 scalar | 131 | 464 (3.5×) | 190 (1.5×) | 555 (4.2×) | 308 (2.4×) |
| mul_uint, constant time | Tom-256 base | 122 | 464 (3.8×) | 181 (1.5×) | 556 (4.6×) | 305 (2.5×) |
| mul_uint, constant time | BLS12-381 base | 256 | 1,067 (4.2×) | 378 (1.5×) | 1,679 (6.6×) | 920 (3.6×) |
| mul_uint, constant time | P-521 base | 426 | 1,983 (4.7×) | 290 (0.7×) | 2,889 (6.8×) | 580 (1.4×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 5.4 (1.2×) | 5.4 (1.2×) | 5.5 (1.2×) | 5.5 (1.2×) |
| add_u64, constant time | BabyBear | 5.4 | 11.9 (2.2×) | 7.4 (1.4×) | 14.1 (2.6×) | 11.4 (2.1×) |
| add_u64, constant time | Goldilocks | 4.7 | 9.3 (2.0×) | 7.5 (1.6×) | 11.8 (2.5×) | 8.5 (1.8×) |
| add_u64, constant time | BN254 scalar | 22.8 | 92.3 (4.0×) | 41.3 (1.8×) | 111 (4.9×) | 68.5 (3.0×) |
| add_u64, constant time | secp256k1 base | 5.8 | 22.5 (3.9×) | 14.6 (2.5×) | 21.1 (3.6×) | 23.1 (4.0×) |
| add_u64, constant time | P-256 base | 17.0 | 71.8 (4.2×) | 36.7 (2.2×) | 78.0 (4.6×) | 57.1 (3.4×) |
| add_u64, constant time | P-256 scalar | 23.9 | 93.1 (3.9×) | 41.9 (1.8×) | 112 (4.7×) | 71.7 (3.0×) |
| add_u64, constant time | Tom-256 base | 22.3 | 97.1 (4.4×) | 42.0 (1.9×) | 110 (4.9×) | 70.3 (3.2×) |
| add_u64, constant time | BLS12-381 base | 36.3 | 150 (4.1×) | 64.3 (1.8×) | 172 (4.7×) | 106 (2.9×) |
| add_u64, constant time | P-521 base | 10.8 | 64.2 (5.9×) | 38.4 (3.6×) | 69.1 (6.4×) | 61.1 (5.7×) |
| add_uint, constant time | Mersenne-31 | 4.6 | 5.4 (1.2×) | 5.4 (1.2×) | 5.5 (1.2×) | 5.5 (1.2×) |
| add_uint, constant time | BabyBear | 5.4 | 11.9 (2.2×) | 7.4 (1.4×) | 14.1 (2.6×) | 11.4 (2.1×) |
| add_uint, constant time | Goldilocks | 4.7 | 9.3 (2.0×) | 7.5 (1.6×) | 11.8 (2.5×) | 8.5 (1.8×) |
| add_uint, constant time | BN254 scalar | 75.9 | 248 (3.3×) | 101 (1.3×) | 331 (4.4×) | 177 (2.3×) |
| add_uint, constant time | secp256k1 base | 9.3 | 40.1 (4.3×) | 27.1 (2.9×) | 37.6 (4.0×) | 40.5 (4.4×) |
| add_uint, constant time | P-256 base | 57.2 | 179 (3.1×) | 77.8 (1.4×) | 281 (4.9×) | 138 (2.4×) |
| add_uint, constant time | P-256 scalar | 81.0 | 261 (3.2×) | 104 (1.3×) | 324 (4.0×) | 186 (2.3×) |
| add_uint, constant time | Tom-256 base | 73.8 | 257 (3.5×) | 106 (1.4×) | 327 (4.4×) | 175 (2.4×) |
| add_uint, constant time | BLS12-381 base | 157 | 764 (4.9×) | 208 (1.3×) | 952 (6.1×) | 379 (2.4×) |
| add_uint, constant time | P-521 base | 22.5 | 142 (6.3×) | 82.0 (3.6×) | 154 (6.8×) | 129 (5.7×) |
| sub_u64, constant time | Mersenne-31 | 4.7 | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) | 5.2 (1.1×) |
| sub_u64, constant time | BabyBear | 5.2 | 10.9 (2.1×) | 7.3 (1.4×) | 13.0 (2.5×) | 11.2 (2.2×) |
| sub_u64, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 7.4 (1.7×) | 11.8 (2.7×) | 8.5 (2.0×) |
| sub_u64, constant time | BN254 scalar | 25.4 | 92.6 (3.6×) | 39.8 (1.6×) | 111 (4.4×) | 66.0 (2.6×) |
| sub_u64, constant time | secp256k1 base | 4.0 | 16.5 (4.1×) | 11.9 (3.0×) | 15.4 (3.9×) | 17.0 (4.2×) |
| sub_u64, constant time | P-256 base | 16.2 | 63.8 (3.9×) | 33.2 (2.0×) | 74.1 (4.6×) | 52.0 (3.2×) |
| sub_u64, constant time | P-256 scalar | 24.0 | 96.1 (4.0×) | 38.8 (1.6×) | 108 (4.5×) | 66.3 (2.8×) |
| sub_u64, constant time | Tom-256 base | 23.0 | 92.6 (4.0×) | 41.9 (1.8×) | 106 (4.6×) | 64.3 (2.8×) |
| sub_u64, constant time | BLS12-381 base | 36.5 | 144 (3.9×) | 62.9 (1.7×) | 185 (5.1×) | 103 (2.8×) |
| sub_u64, constant time | P-521 base | 9.6 | 43.2 (4.5×) | 27.9 (2.9×) | 52.6 (5.5×) | 39.1 (4.1×) |
| sub_uint, constant time | Mersenne-31 | 4.7 | 5.4 (1.1×) | 5.4 (1.1×) | 5.4 (1.1×) | 5.2 (1.1×) |
| sub_uint, constant time | BabyBear | 5.2 | 10.9 (2.1×) | 7.3 (1.4×) | 13.1 (2.5×) | 11.2 (2.2×) |
| sub_uint, constant time | Goldilocks | 4.3 | 9.4 (2.2×) | 7.3 (1.7×) | 11.8 (2.7×) | 8.5 (2.0×) |
| sub_uint, constant time | BN254 scalar | 77.7 | 249 (3.2×) | 99.4 (1.3×) | 329 (4.2×) | 173 (2.2×) |
| sub_uint, constant time | secp256k1 base | 7.2 | 34.1 (4.7×) | 23.5 (3.3×) | 32.5 (4.5×) | 35.7 (5.0×) |
| sub_uint, constant time | P-256 base | 53.7 | 174 (3.2×) | 74.2 (1.4×) | 280 (5.2×) | 133 (2.5×) |
| sub_uint, constant time | P-256 scalar | 80.0 | 247 (3.1×) | 101 (1.3×) | 320 (4.0×) | 174 (2.2×) |
| sub_uint, constant time | Tom-256 base | 75.8 | 251 (3.3×) | 103 (1.4×) | 323 (4.3×) | 171 (2.3×) |
| sub_uint, constant time | BLS12-381 base | 155 | 763 (4.9×) | 203 (1.3×) | 944 (6.1×) | 373 (2.4×) |
| sub_uint, constant time | P-521 base | 19.2 | 124 (6.5×) | 66.6 (3.5×) | 121 (6.3×) | 107 (5.6×) |
| from_u64, constant time | Mersenne-31 | 3.8 | 4.4 (1.2×) | 4.4 (1.2×) | 5.6 (1.5×) | 5.4 (1.4×) |
| from_u64, constant time | BabyBear | 4.9 | 11.7 (2.4×) | 6.4 (1.3×) | 12.6 (2.6×) | 9.6 (2.0×) |
| from_u64, constant time | Goldilocks | 3.4 | 7.9 (2.3×) | 5.6 (1.6×) | 8.6 (2.5×) | 6.9 (2.0×) |
| from_u64, constant time | BN254 scalar | 26.3 | 88.1 (3.3×) | 35.3 (1.3×) | 98.8 (3.8×) | 49.3 (1.9×) |
| from_u64, constant time | secp256k1 base | 2.1 | 3.3 (1.6×) | 3.3 (1.6×) | 3.2 (1.5×) | 3.2 (1.5×) |
| from_u64, constant time | P-256 base | 18.5 | 56.4 (3.0×) | 26.0 (1.4×) | 67.1 (3.6×) | 37.1 (2.0×) |
| from_u64, constant time | P-256 scalar | 27.2 | 86.4 (3.2×) | 36.0 (1.3×) | 108 (4.0×) | 49.2 (1.8×) |
| from_u64, constant time | Tom-256 base | 24.2 | 79.1 (3.3×) | 37.1 (1.5×) | 94.1 (3.9×) | 48.1 (2.0×) |
| from_u64, constant time | BLS12-381 base | 41.9 | 140 (3.3×) | 56.9 (1.4×) | 170 (4.1×) | 79.8 (1.9×) |
| from_u64, constant time | P-521 base | 5.0 | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) | 6.0 (1.2×) |
| div_u64, constant time | BN254 scalar | 6,846 | 12,195 (1.8×) | 9,444 (1.4×) | 17,395 (2.5×) | 12,955 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,779 | 11,907 (1.8×) | 9,056 (1.3×) | 16,824 (2.5×) | 12,825 (1.9×) |
| div_u64, constant time | P-256 base | 6,666 | 11,709 (1.8×) | 9,122 (1.4×) | 17,079 (2.6×) | 12,504 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,885 | 12,209 (1.8×) | 9,412 (1.4×) | 17,601 (2.6×) | 13,189 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,274 | 12,097 (1.7×) | 9,336 (1.3×) | 16,838 (2.3×) | 12,896 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,390 | 24,660 (2.0×) | 16,947 (1.4×) | 34,592 (2.8×) | 24,245 (2.0×) |
| div_u64, constant time | P-521 base | 20,386 | 42,042 (2.1×) | 25,899 (1.3×) | 64,757 (3.2×) | 43,412 (2.1×) |
| div_uint, constant time | Mersenne-31 | 253 | 225 (0.9×) | 226 (0.9×) | 229 (0.9×) | 229 (0.9×) |
| div_uint, constant time | BabyBear | 403 | 478 (1.2×) | 374 (0.9×) | 523 (1.3×) | 441 (1.1×) |
| div_uint, constant time | Goldilocks | 668 | 1,121 (1.7×) | 807 (1.2×) | 1,368 (2.0×) | 952 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,895 | 12,459 (1.8×) | 9,650 (1.4×) | 17,690 (2.6×) | 13,299 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,778 | 11,920 (1.8×) | 9,070 (1.3×) | 16,879 (2.5×) | 12,848 (1.9×) |
| div_uint, constant time | P-256 base | 6,702 | 11,924 (1.8×) | 9,148 (1.4×) | 17,392 (2.6×) | 12,558 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,952 | 12,423 (1.8×) | 9,551 (1.4×) | 17,828 (2.6×) | 13,324 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,330 | 12,345 (1.7×) | 9,489 (1.3×) | 17,040 (2.3×) | 13,116 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,499 | 25,235 (2.0×) | 17,014 (1.4×) | 35,308 (2.8×) | 24,755 (2.0×) |
| div_uint, constant time | P-521 base | 20,379 | 42,376 (2.1×) | 26,228 (1.3×) | 64,633 (3.2×) | 43,327 (2.1×) |

### Against native, latency

Cycles per operation of `prime-field` in each build, then its ratio to the native figure.

| Operation | Field | Native | Chrome | Chrome, wide arithmetic | Firefox | Firefox, wide arithmetic |
|---|---|---:|---:|---:|---:|---:|
| add | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add | BN254 scalar | 6.8 | 13.0 (1.9×) | 10.1 (1.5×) | 13.5 (2.0×) | 15.3 (2.2×) |
| add | secp256k1 base | 6.9 | 14.2 (2.1×) | 9.8 (1.4×) | 14.1 (2.0×) | 17.4 (2.5×) |
| add | P-256 base | 6.9 | 12.8 (1.9×) | 9.9 (1.4×) | 13.3 (1.9×) | 17.3 (2.5×) |
| add | P-256 scalar | 6.9 | 13.9 (2.0×) | 10.9 (1.6×) | 13.7 (2.0×) | 17.9 (2.6×) |
| add | Tom-256 base | 6.9 | 13.3 (1.9×) | 10.7 (1.6×) | 13.4 (1.9×) | 16.4 (2.4×) |
| add | BLS12-381 base | 8.9 | 22.1 (2.5×) | 15.9 (1.8×) | 22.1 (2.5×) | 25.9 (2.9×) |
| add | P-521 base | 14.8 | 41.7 (2.8×) | 24.0 (1.6×) | 49.7 (3.4×) | 48.9 (3.3×) |
| sub | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub | BN254 scalar | 6.7 | 14.0 (2.1×) | 11.3 (1.7×) | 12.3 (1.8×) | 16.3 (2.4×) |
| sub | secp256k1 base | 8.0 | 15.8 (2.0×) | 11.2 (1.4×) | 13.5 (1.7×) | 17.8 (2.2×) |
| sub | P-256 base | 6.5 | 13.4 (2.1×) | 10.3 (1.6×) | 12.4 (1.9×) | 16.3 (2.5×) |
| sub | P-256 scalar | 6.9 | 14.9 (2.2×) | 11.2 (1.6×) | 13.5 (2.0×) | 17.4 (2.5×) |
| sub | Tom-256 base | 8.1 | 14.0 (1.7×) | 11.3 (1.4×) | 12.5 (1.5×) | 16.4 (2.0×) |
| sub | BLS12-381 base | 9.9 | 20.9 (2.1×) | 18.8 (1.9×) | 19.9 (2.0×) | 28.2 (2.8×) |
| sub | P-521 base | 12.1 | 42.8 (3.5×) | 25.5 (2.1×) | 41.9 (3.5×) | 43.8 (3.6×) |
| mul | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul | BabyBear | 10.9 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 12.0 (1.1×) |
| mul | Goldilocks | 9.0 | 16.5 (1.8×) | 12.7 (1.4×) | 18.0 (2.0×) | 14.0 (1.6×) |
| mul | BN254 scalar | 72.5 | 222 (3.1×) | 91.7 (1.3×) | 367 (5.1×) | 149 (2.1×) |
| mul | secp256k1 base | 54.8 | 170 (3.1×) | 67.1 (1.2×) | 237 (4.3×) | 105 (1.9×) |
| mul | P-256 base | 52.5 | 184 (3.5×) | 62.2 (1.2×) | 294 (5.6×) | 118 (2.2×) |
| mul | P-256 scalar | 84.0 | 239 (2.8×) | 91.0 (1.1×) | 325 (3.9×) | 157 (1.9×) |
| mul | Tom-256 base | 76.0 | 235 (3.1×) | 87.9 (1.2×) | 325 (4.3×) | 152 (2.0×) |
| mul | BLS12-381 base | 141 | 716 (5.1×) | 185 (1.3×) | 880 (6.2×) | 335 (2.4×) |
| mul | P-521 base | 191 | 688 (3.6×) | 240 (1.3×) | 1,291 (6.8×) | 440 (2.3×) |
| square | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square | BabyBear | 11.0 | 12.3 (1.1×) | 11.0 (1.0×) | 13.0 (1.2×) | 12.0 (1.1×) |
| square | Goldilocks | 9.0 | 15.0 (1.7×) | 13.9 (1.5×) | 18.3 (2.0×) | 14.3 (1.6×) |
| square | BN254 scalar | 66.5 | 209 (3.1×) | 84.2 (1.3×) | 323 (4.9×) | 134 (2.0×) |
| square | secp256k1 base | 42.9 | 152 (3.5×) | 59.0 (1.4×) | 186 (4.3×) | 84.7 (2.0×) |
| square | P-256 base | 44.0 | 161 (3.7×) | 58.2 (1.3×) | 232 (5.3×) | 102 (2.3×) |
| square | P-256 scalar | 73.0 | 217 (3.0×) | 81.7 (1.1×) | 294 (4.0×) | 136 (1.9×) |
| square | Tom-256 base | 64.5 | 209 (3.2×) | 81.1 (1.3×) | 286 (4.4×) | 130 (2.0×) |
| square | BLS12-381 base | 136 | 565 (4.2×) | 153 (1.1×) | 789 (5.8×) | 295 (2.2×) |
| square | P-521 base | 133 | 571 (4.3×) | 230 (1.7×) | 920 (6.9×) | 386 (2.9×) |
| mul_u64 | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64 | BabyBear | 10.9 | 14.0 (1.3×) | 11.1 (1.0×) | 17.0 (1.6×) | 14.4 (1.3×) |
| mul_u64 | Goldilocks | 9.1 | 17.6 (1.9×) | 14.3 (1.6×) | 25.6 (2.8×) | 15.1 (1.7×) |
| mul_u64 | BN254 scalar | 25.8 | 99.2 (3.8×) | 37.8 (1.5×) | 119 (4.6×) | 44.8 (1.7×) |
| mul_u64 | secp256k1 base | 14.9 | 49.1 (3.3×) | 17.0 (1.1×) | 65.1 (4.4×) | 23.0 (1.5×) |
| mul_u64 | P-256 base | 24.9 | 75.4 (3.0×) | 34.0 (1.4×) | 90.8 (3.6×) | 42.1 (1.7×) |
| mul_u64 | P-256 scalar | 27.9 | 82.0 (2.9×) | 37.9 (1.4×) | 108 (3.9×) | 47.6 (1.7×) |
| mul_u64 | Tom-256 base | 27.9 | 84.2 (3.0×) | 35.0 (1.3×) | 105 (3.8×) | 45.5 (1.6×) |
| mul_u64 | BLS12-381 base | 34.5 | 130 (3.8×) | 48.4 (1.4×) | 182 (5.3×) | 62.7 (1.8×) |
| mul_u64 | P-521 base | 32.9 | 94.5 (2.9×) | 38.6 (1.2×) | 139 (4.2×) | 53.4 (1.6×) |
| mul_uint | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint | BabyBear | 10.9 | 14.0 (1.3×) | 11.1 (1.0×) | 17.0 (1.6×) | 14.4 (1.3×) |
| mul_uint | Goldilocks | 9.1 | 17.5 (1.9×) | 14.3 (1.6×) | 25.6 (2.8×) | 15.1 (1.7×) |
| mul_uint | BN254 scalar | 114 | 435 (3.8×) | 171 (1.5×) | 526 (4.6×) | 295 (2.6×) |
| mul_uint | secp256k1 base | 68.0 | 266 (3.9×) | 68.1 (1.0×) | 317 (4.7×) | 107 (1.6×) |
| mul_uint | P-256 base | 110 | 352 (3.2×) | 127 (1.2×) | 424 (3.9×) | 231 (2.1×) |
| mul_uint | P-256 scalar | 116 | 425 (3.7×) | 181 (1.6×) | 482 (4.2×) | 308 (2.7×) |
| mul_uint | Tom-256 base | 114 | 414 (3.6×) | 169 (1.5×) | 478 (4.2×) | 296 (2.6×) |
| mul_uint | BLS12-381 base | 222 | 963 (4.3×) | 365 (1.6×) | 1,134 (5.1×) | 897 (4.0×) |
| mul_uint | P-521 base | 394 | 1,566 (4.0×) | 244 (0.6×) | 2,390 (6.1×) | 450 (1.1×) |
| add_u64 | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 5.0 (1.1×) | 5.0 (1.1×) |
| add_u64 | BabyBear | 7.0 | 12.1 (1.7×) | 8.4 (1.2×) | 12.8 (1.8×) | 11.0 (1.6×) |
| add_u64 | Goldilocks | 4.4 | 8.1 (1.8×) | 6.8 (1.5×) | 11.8 (2.7×) | 8.8 (2.0×) |
| add_u64 | BN254 scalar | 24.8 | 106 (4.3×) | 41.0 (1.7×) | 137 (5.5×) | 59.3 (2.4×) |
| add_u64 | secp256k1 base | 7.4 | 7.4 (1.0×) | 6.0 (0.8×) | 7.1 (1.0×) | 11.0 (1.5×) |
| add_u64 | P-256 base | 22.1 | 68.7 (3.1×) | 34.5 (1.6×) | 76.7 (3.5×) | 52.4 (2.4×) |
| add_u64 | P-256 scalar | 22.7 | 68.9 (3.0×) | 38.5 (1.7×) | 74.1 (3.3×) | 61.8 (2.7×) |
| add_u64 | Tom-256 base | 26.0 | 85.6 (3.3×) | 39.7 (1.5×) | 96.5 (3.7×) | 59.8 (2.3×) |
| add_u64 | BLS12-381 base | 39.4 | 149 (3.8×) | 61.9 (1.6×) | 228 (5.8×) | 84.4 (2.1×) |
| add_u64 | P-521 base | 12.7 | 22.2 (1.7×) | 14.1 (1.1×) | 19.5 (1.5×) | 27.2 (2.1×) |
| add_uint | Mersenne-31 | 4.6 | 4.6 (1.0×) | 4.6 (1.0×) | 5.0 (1.1×) | 5.0 (1.1×) |
| add_uint | BabyBear | 7.0 | 12.1 (1.7×) | 8.4 (1.2×) | 12.8 (1.8×) | 11.0 (1.6×) |
| add_uint | Goldilocks | 4.4 | 8.1 (1.8×) | 6.8 (1.5×) | 11.8 (2.7×) | 8.7 (2.0×) |
| add_uint | BN254 scalar | 73.2 | 229 (3.1×) | 99.4 (1.4×) | 361 (4.9×) | 178 (2.4×) |
| add_uint | secp256k1 base | 8.5 | 17.3 (2.0×) | 13.5 (1.6×) | 17.0 (2.0×) | 21.9 (2.6×) |
| add_uint | P-256 base | 58.0 | 178 (3.1×) | 73.6 (1.3×) | 320 (5.5×) | 140 (2.4×) |
| add_uint | P-256 scalar | 78.1 | 258 (3.3×) | 101 (1.3×) | 324 (4.1×) | 183 (2.3×) |
| add_uint | Tom-256 base | 75.3 | 254 (3.4×) | 99.6 (1.3×) | 320 (4.2×) | 177 (2.4×) |
| add_uint | BLS12-381 base | 152 | 763 (5.0×) | 204 (1.3×) | 946 (6.2×) | 383 (2.5×) |
| add_uint | P-521 base | 23.3 | 61.1 (2.6×) | 38.7 (1.7×) | 78.3 (3.4×) | 67.0 (2.9×) |
| sub_u64 | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64 | BabyBear | 7.0 | 12.0 (1.7×) | 8.3 (1.2×) | 12.5 (1.8×) | 11.1 (1.6×) |
| sub_u64 | Goldilocks | 4.2 | 8.9 (2.1×) | 6.3 (1.5×) | 11.4 (2.7×) | 9.2 (2.2×) |
| sub_u64 | BN254 scalar | 24.8 | 103 (4.2×) | 41.4 (1.7×) | 137 (5.5×) | 62.7 (2.5×) |
| sub_u64 | secp256k1 base | 6.9 | 9.0 (1.3×) | 6.0 (0.9×) | 8.0 (1.2×) | 9.5 (1.4×) |
| sub_u64 | P-256 base | 21.3 | 70.5 (3.3×) | 34.2 (1.6×) | 75.9 (3.6×) | 51.4 (2.4×) |
| sub_u64 | P-256 scalar | 23.6 | 69.6 (2.9×) | 36.6 (1.6×) | 71.9 (3.0×) | 58.6 (2.5×) |
| sub_u64 | Tom-256 base | 25.0 | 88.2 (3.5×) | 40.2 (1.6×) | 99.5 (4.0×) | 59.2 (2.4×) |
| sub_u64 | BLS12-381 base | 38.6 | 150 (3.9×) | 60.9 (1.6×) | 255 (6.6×) | 88.2 (2.3×) |
| sub_u64 | P-521 base | 11.0 | 22.4 (2.0×) | 14.7 (1.3×) | 21.9 (2.0×) | 20.7 (1.9×) |
| sub_uint | Mersenne-31 | 4.5 | 4.7 (1.0×) | 4.6 (1.0×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_uint | BabyBear | 6.9 | 12.0 (1.7×) | 8.4 (1.2×) | 12.5 (1.8×) | 11.1 (1.6×) |
| sub_uint | Goldilocks | 4.2 | 8.9 (2.1×) | 6.3 (1.5×) | 11.4 (2.7×) | 9.2 (2.2×) |
| sub_uint | BN254 scalar | 73.2 | 241 (3.3×) | 97.0 (1.3×) | 360 (4.9×) | 173 (2.4×) |
| sub_uint | secp256k1 base | 6.8 | 19.0 (2.8×) | 12.8 (1.9×) | 16.1 (2.4×) | 20.7 (3.0×) |
| sub_uint | P-256 base | 56.8 | 175 (3.1×) | 72.2 (1.3×) | 271 (4.8×) | 135 (2.4×) |
| sub_uint | P-256 scalar | 79.2 | 253 (3.2×) | 98.5 (1.2×) | 320 (4.0×) | 178 (2.2×) |
| sub_uint | Tom-256 base | 72.8 | 249 (3.4×) | 96.6 (1.3×) | 320 (4.4×) | 172 (2.4×) |
| sub_uint | BLS12-381 base | 154 | 762 (4.9×) | 197 (1.3×) | 945 (6.1×) | 378 (2.5×) |
| sub_uint | P-521 base | 15.6 | 62.5 (4.0×) | 37.0 (2.4×) | 71.5 (4.6×) | 64.2 (4.1×) |
| div_u64 | BN254 scalar | 604 | 1,203 (2.0×) | 865 (1.4×) | 1,381 (2.3×) | 1,105 (1.8×) |
| div_u64 | secp256k1 base | 566 | 1,008 (1.8×) | 716 (1.3×) | 1,231 (2.2×) | 808 (1.4×) |
| div_u64 | P-256 base | 602 | 1,081 (1.8×) | 801 (1.3×) | 1,346 (2.2×) | 985 (1.6×) |
| div_u64 | P-256 scalar | 618 | 1,186 (1.9×) | 869 (1.4×) | 1,446 (2.3×) | 1,120 (1.8×) |
| div_u64 | Tom-256 base | 604 | 1,201 (2.0×) | 870 (1.4×) | 1,460 (2.4×) | 1,096 (1.8×) |
| div_u64 | BLS12-381 base | 751 | 1,785 (2.4×) | 1,105 (1.5×) | 2,470 (3.3×) | 1,754 (2.3×) |
| div_u64 | P-521 base | 923 | 2,450 (2.7×) | 1,002 (1.1×) | 3,600 (3.9×) | 1,532 (1.7×) |
| div_uint | Mersenne-31 | 218 | 268 (1.2×) | 269 (1.2×) | 280 (1.3×) | 278 (1.3×) |
| div_uint | BabyBear | 213 | 273 (1.3×) | 273 (1.3×) | 278 (1.3×) | 280 (1.3×) |
| div_uint | Goldilocks | 434 | 549 (1.3×) | 539 (1.2×) | 579 (1.3×) | 573 (1.3×) |
| div_uint | BN254 scalar | 4,994 | 9,008 (1.8×) | 6,718 (1.3×) | 12,609 (2.5×) | 8,438 (1.7×) |
| div_uint | secp256k1 base | 4,880 | 8,513 (1.7×) | 6,064 (1.2×) | 12,043 (2.5×) | 8,158 (1.7×) |
| div_uint | P-256 base | 5,016 | 8,692 (1.7×) | 6,159 (1.2×) | 12,174 (2.4×) | 7,871 (1.6×) |
| div_uint | P-256 scalar | 4,951 | 9,049 (1.8×) | 6,543 (1.3×) | 12,535 (2.5×) | 8,575 (1.7×) |
| div_uint | Tom-256 base | 5,051 | 8,896 (1.8×) | 6,454 (1.3×) | 12,386 (2.5×) | 8,407 (1.7×) |
| div_uint | BLS12-381 base | 9,025 | 20,737 (2.3×) | 12,085 (1.3×) | 26,397 (2.9×) | 16,914 (1.9×) |
| div_uint | P-521 base | 14,953 | 34,505 (2.3×) | 18,162 (1.2×) | 49,283 (3.3×) | 30,231 (2.0×) |
| add, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BabyBear | 3.0 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| add, constant time | BN254 scalar | 6.8 | 24.6 (3.6×) | 16.9 (2.5×) | 22.2 (3.3×) | 24.1 (3.5×) |
| add, constant time | secp256k1 base | 6.8 | 28.1 (4.1×) | 16.1 (2.4×) | 25.5 (3.8×) | 27.1 (4.0×) |
| add, constant time | P-256 base | 6.9 | 24.9 (3.6×) | 15.9 (2.3×) | 24.0 (3.5×) | 25.6 (3.7×) |
| add, constant time | P-256 scalar | 6.7 | 26.2 (3.9×) | 17.2 (2.6×) | 26.3 (3.9×) | 27.1 (4.0×) |
| add, constant time | Tom-256 base | 6.9 | 27.0 (3.9×) | 17.0 (2.5×) | 26.3 (3.8×) | 27.1 (3.9×) |
| add, constant time | BLS12-381 base | 8.9 | 41.9 (4.7×) | 24.3 (2.7×) | 39.6 (4.4×) | 40.1 (4.5×) |
| add, constant time | P-521 base | 14.7 | 76.7 (5.2×) | 39.5 (2.7×) | 82.0 (5.6×) | 72.2 (4.9×) |
| sub, constant time | Mersenne-31 | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BabyBear | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | Goldilocks | 2.9 | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) | 3.0 (1.0×) |
| sub, constant time | BN254 scalar | 6.8 | 22.2 (3.3×) | 14.6 (2.1×) | 20.1 (3.0×) | 22.1 (3.3×) |
| sub, constant time | secp256k1 base | 8.0 | 20.3 (2.5×) | 14.0 (1.8×) | 19.1 (2.4×) | 20.4 (2.5×) |
| sub, constant time | P-256 base | 6.8 | 19.5 (2.9×) | 12.1 (1.8×) | 19.8 (2.9×) | 19.1 (2.8×) |
| sub, constant time | P-256 scalar | 6.8 | 21.6 (3.2×) | 14.6 (2.1×) | 21.1 (3.1×) | 21.7 (3.2×) |
| sub, constant time | Tom-256 base | 8.1 | 21.9 (2.7×) | 14.4 (1.8×) | 19.6 (2.4×) | 22.1 (2.7×) |
| sub, constant time | BLS12-381 base | 9.7 | 37.3 (3.8×) | 22.1 (2.3×) | 36.1 (3.7×) | 36.4 (3.8×) |
| sub, constant time | P-521 base | 12.2 | 56.4 (4.6×) | 31.3 (2.6×) | 59.9 (4.9×) | 51.2 (4.2×) |
| mul, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| mul, constant time | BabyBear | 11.0 | 13.0 (1.2×) | 11.0 (1.0×) | 14.0 (1.3×) | 12.0 (1.1×) |
| mul, constant time | Goldilocks | 8.9 | 16.5 (1.9×) | 12.7 (1.4×) | 18.0 (2.0×) | 14.0 (1.6×) |
| mul, constant time | BN254 scalar | 79.0 | 245 (3.1×) | 91.5 (1.2×) | 414 (5.2×) | 149 (1.9×) |
| mul, constant time | secp256k1 base | 55.0 | 170 (3.1×) | 67.1 (1.2×) | 237 (4.3×) | 105 (1.9×) |
| mul, constant time | P-256 base | 52.5 | 184 (3.5×) | 62.2 (1.2×) | 292 (5.6×) | 118 (2.2×) |
| mul, constant time | P-256 scalar | 84.0 | 240 (2.9×) | 91.0 (1.1×) | 327 (3.9×) | 157 (1.9×) |
| mul, constant time | Tom-256 base | 76.0 | 238 (3.1×) | 88.2 (1.2×) | 325 (4.3×) | 152 (2.0×) |
| mul, constant time | BLS12-381 base | 154 | 719 (4.7×) | 185 (1.2×) | 879 (5.7×) | 336 (2.2×) |
| mul, constant time | P-521 base | 191 | 688 (3.6×) | 240 (1.3×) | 1,292 (6.8×) | 439 (2.3×) |
| square, constant time | Mersenne-31 | 7.0 | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) | 7.0 (1.0×) |
| square, constant time | BabyBear | 11.0 | 12.2 (1.1×) | 11.0 (1.0×) | 13.0 (1.2×) | 12.0 (1.1×) |
| square, constant time | Goldilocks | 9.0 | 15.0 (1.7×) | 13.9 (1.5×) | 18.3 (2.0×) | 14.3 (1.6×) |
| square, constant time | BN254 scalar | 66.9 | 214 (3.2×) | 84.2 (1.3×) | 286 (4.3×) | 134 (2.0×) |
| square, constant time | secp256k1 base | 43.0 | 152 (3.5×) | 59.0 (1.4×) | 185 (4.3×) | 84.6 (2.0×) |
| square, constant time | P-256 base | 44.0 | 161 (3.7×) | 58.2 (1.3×) | 235 (5.3×) | 102 (2.3×) |
| square, constant time | P-256 scalar | 73.0 | 217 (3.0×) | 81.5 (1.1×) | 294 (4.0×) | 136 (1.9×) |
| square, constant time | Tom-256 base | 64.5 | 209 (3.2×) | 81.1 (1.3×) | 288 (4.5×) | 130 (2.0×) |
| square, constant time | BLS12-381 base | 145 | 565 (3.9×) | 153 (1.1×) | 795 (5.5×) | 295 (2.0×) |
| square, constant time | P-521 base | 133 | 570 (4.3×) | 231 (1.7×) | 924 (6.9×) | 387 (2.9×) |
| mul_u64, constant time | Mersenne-31 | 8.9 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_u64, constant time | BabyBear | 10.9 | 14.0 (1.3×) | 11.1 (1.0×) | 17.0 (1.6×) | 14.4 (1.3×) |
| mul_u64, constant time | Goldilocks | 9.1 | 17.5 (1.9×) | 14.3 (1.6×) | 25.6 (2.8×) | 15.1 (1.7×) |
| mul_u64, constant time | BN254 scalar | 30.4 | 109 (3.6×) | 48.9 (1.6×) | 138 (4.5×) | 57.5 (1.9×) |
| mul_u64, constant time | secp256k1 base | 19.9 | 69.4 (3.5×) | 30.6 (1.5×) | 85.1 (4.3×) | 40.8 (2.1×) |
| mul_u64, constant time | P-256 base | 31.9 | 88.6 (2.8×) | 45.0 (1.4×) | 108 (3.4×) | 62.8 (2.0×) |
| mul_u64, constant time | P-256 scalar | 33.9 | 107 (3.2×) | 47.2 (1.4×) | 127 (3.7×) | 68.8 (2.0×) |
| mul_u64, constant time | Tom-256 base | 32.9 | 101 (3.1×) | 45.2 (1.4×) | 122 (3.7×) | 68.5 (2.1×) |
| mul_u64, constant time | BLS12-381 base | 42.1 | 154 (3.7×) | 70.6 (1.7×) | 197 (4.7×) | 84.5 (2.0×) |
| mul_u64, constant time | P-521 base | 45.3 | 139 (3.1×) | 62.1 (1.4×) | 170 (3.8×) | 90.3 (2.0×) |
| mul_uint, constant time | Mersenne-31 | 9.0 | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) | 9.0 (1.0×) |
| mul_uint, constant time | BabyBear | 10.9 | 14.0 (1.3×) | 11.1 (1.0×) | 17.0 (1.6×) | 14.4 (1.3×) |
| mul_uint, constant time | Goldilocks | 8.8 | 17.5 (2.0×) | 14.3 (1.6×) | 25.6 (2.9×) | 15.1 (1.7×) |
| mul_uint, constant time | BN254 scalar | 141 | 480 (3.4×) | 171 (1.2×) | 589 (4.2×) | 298 (2.1×) |
| mul_uint, constant time | secp256k1 base | 83.8 | 321 (3.8×) | 67.5 (0.8×) | 387 (4.6×) | 117 (1.4×) |
| mul_uint, constant time | P-256 base | 134 | 410 (3.1×) | 124 (0.9×) | 487 (3.6×) | 229 (1.7×) |
| mul_uint, constant time | P-256 scalar | 146 | 477 (3.3×) | 180 (1.2×) | 552 (3.8×) | 311 (2.1×) |
| mul_uint, constant time | Tom-256 base | 135 | 466 (3.5×) | 172 (1.3×) | 548 (4.1×) | 297 (2.2×) |
| mul_uint, constant time | BLS12-381 base | 266 | 1,076 (4.0×) | 367 (1.4×) | 1,691 (6.4×) | 900 (3.4×) |
| mul_uint, constant time | P-521 base | 472 | 2,016 (4.3×) | 281 (0.6×) | 2,908 (6.2×) | 494 (1.0×) |
| add_u64, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 5.0 (1.1×) | 5.0 (1.1×) |
| add_u64, constant time | BabyBear | 4.9 | 11.8 (2.4×) | 7.1 (1.4×) | 13.1 (2.7×) | 11.5 (2.3×) |
| add_u64, constant time | Goldilocks | 4.8 | 8.1 (1.7×) | 6.8 (1.4×) | 11.8 (2.5×) | 8.7 (1.8×) |
| add_u64, constant time | BN254 scalar | 21.9 | 97.7 (4.5×) | 40.7 (1.9×) | 108 (4.9×) | 69.9 (3.2×) |
| add_u64, constant time | secp256k1 base | 7.4 | 25.9 (3.5×) | 15.0 (2.0×) | 25.5 (3.4×) | 24.0 (3.2×) |
| add_u64, constant time | P-256 base | 17.1 | 69.1 (4.0×) | 36.6 (2.1×) | 81.4 (4.8×) | 58.6 (3.4×) |
| add_u64, constant time | P-256 scalar | 24.7 | 105 (4.3×) | 43.0 (1.7×) | 109 (4.4×) | 72.9 (3.0×) |
| add_u64, constant time | Tom-256 base | 22.3 | 98.3 (4.4×) | 40.8 (1.8×) | 110 (4.9×) | 71.1 (3.2×) |
| add_u64, constant time | BLS12-381 base | 35.8 | 136 (3.8×) | 63.0 (1.8×) | 170 (4.7×) | 107 (3.0×) |
| add_u64, constant time | P-521 base | 12.7 | 66.6 (5.2×) | 38.7 (3.0×) | 68.0 (5.4×) | 62.1 (4.9×) |
| add_uint, constant time | Mersenne-31 | 4.6 | 4.6 (1.0×) | 4.6 (1.0×) | 5.0 (1.1×) | 5.0 (1.1×) |
| add_uint, constant time | BabyBear | 4.9 | 11.8 (2.4×) | 7.1 (1.4×) | 13.1 (2.7×) | 11.5 (2.3×) |
| add_uint, constant time | Goldilocks | 4.0 | 8.1 (2.0×) | 6.8 (1.7×) | 11.8 (3.0×) | 8.7 (2.2×) |
| add_uint, constant time | BN254 scalar | 74.8 | 247 (3.3×) | 99.6 (1.3×) | 345 (4.6×) | 178 (2.4×) |
| add_uint, constant time | secp256k1 base | 8.7 | 39.3 (4.5×) | 25.2 (2.9×) | 38.7 (4.4×) | 40.6 (4.7×) |
| add_uint, constant time | P-256 base | 57.8 | 177 (3.1×) | 75.6 (1.3×) | 281 (4.9×) | 140 (2.4×) |
| add_uint, constant time | P-256 scalar | 78.1 | 256 (3.3×) | 102 (1.3×) | 318 (4.1×) | 183 (2.3×) |
| add_uint, constant time | Tom-256 base | 75.2 | 253 (3.4×) | 99.0 (1.3×) | 328 (4.4×) | 177 (2.4×) |
| add_uint, constant time | BLS12-381 base | 152 | 762 (5.0×) | 204 (1.3×) | 947 (6.2×) | 382 (2.5×) |
| add_uint, constant time | P-521 base | 22.8 | 138 (6.1×) | 80.0 (3.5×) | 139 (6.1×) | 130 (5.7×) |
| sub_u64, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_u64, constant time | BabyBear | 4.8 | 12.0 (2.5×) | 7.4 (1.5×) | 13.1 (2.7×) | 10.6 (2.2×) |
| sub_u64, constant time | Goldilocks | 4.2 | 8.9 (2.1×) | 6.3 (1.5×) | 11.4 (2.7×) | 9.2 (2.2×) |
| sub_u64, constant time | BN254 scalar | 22.0 | 95.1 (4.3×) | 39.0 (1.8×) | 104 (4.7×) | 66.9 (3.0×) |
| sub_u64, constant time | secp256k1 base | 6.9 | 18.0 (2.6×) | 11.4 (1.7×) | 15.0 (2.2×) | 16.0 (2.3×) |
| sub_u64, constant time | P-256 base | 16.1 | 64.0 (4.0×) | 33.4 (2.1×) | 73.2 (4.5×) | 53.1 (3.3×) |
| sub_u64, constant time | P-256 scalar | 22.8 | 97.1 (4.3×) | 39.0 (1.7×) | 107 (4.7×) | 67.6 (3.0×) |
| sub_u64, constant time | Tom-256 base | 23.9 | 91.5 (3.8×) | 38.1 (1.6×) | 105 (4.4×) | 65.6 (2.7×) |
| sub_u64, constant time | BLS12-381 base | 34.8 | 137 (3.9×) | 59.5 (1.7×) | 177 (5.1×) | 104 (3.0×) |
| sub_u64, constant time | P-521 base | 11.0 | 44.7 (4.1×) | 27.9 (2.5×) | 49.0 (4.5×) | 40.2 (3.7×) |
| sub_uint, constant time | Mersenne-31 | 4.5 | 4.6 (1.0×) | 4.6 (1.0×) | 5.1 (1.1×) | 5.1 (1.1×) |
| sub_uint, constant time | BabyBear | 4.8 | 12.1 (2.5×) | 7.4 (1.5×) | 13.1 (2.7×) | 10.6 (2.2×) |
| sub_uint, constant time | Goldilocks | 4.2 | 8.9 (2.1×) | 6.3 (1.5×) | 11.4 (2.7×) | 9.2 (2.2×) |
| sub_uint, constant time | BN254 scalar | 75.6 | 243 (3.2×) | 96.9 (1.3×) | 336 (4.4×) | 173 (2.3×) |
| sub_uint, constant time | secp256k1 base | 7.7 | 33.3 (4.3×) | 21.1 (2.7×) | 32.0 (4.2×) | 35.4 (4.6×) |
| sub_uint, constant time | P-256 base | 56.8 | 175 (3.1×) | 72.3 (1.3×) | 269 (4.7×) | 134 (2.4×) |
| sub_uint, constant time | P-256 scalar | 79.1 | 253 (3.2×) | 98.7 (1.2×) | 323 (4.1×) | 178 (2.3×) |
| sub_uint, constant time | Tom-256 base | 72.8 | 249 (3.4×) | 95.6 (1.3×) | 321 (4.4×) | 172 (2.4×) |
| sub_uint, constant time | BLS12-381 base | 156 | 760 (4.9×) | 197 (1.3×) | 950 (6.1×) | 377 (2.4×) |
| sub_uint, constant time | P-521 base | 19.6 | 122 (6.2×) | 66.1 (3.4×) | 120 (6.1×) | 109 (5.6×) |
| div_u64, constant time | BN254 scalar | 6,843 | 12,333 (1.8×) | 9,408 (1.4×) | 17,290 (2.5×) | 12,886 (1.9×) |
| div_u64, constant time | secp256k1 base | 6,775 | 11,899 (1.8×) | 9,069 (1.3×) | 16,801 (2.5×) | 12,797 (1.9×) |
| div_u64, constant time | P-256 base | 6,667 | 11,795 (1.8×) | 9,138 (1.4×) | 17,094 (2.6×) | 12,485 (1.9×) |
| div_u64, constant time | P-256 scalar | 6,890 | 12,226 (1.8×) | 9,364 (1.4×) | 17,559 (2.5×) | 13,154 (1.9×) |
| div_u64, constant time | Tom-256 base | 7,277 | 12,170 (1.7×) | 9,302 (1.3×) | 16,864 (2.3×) | 12,887 (1.8×) |
| div_u64, constant time | BLS12-381 base | 12,385 | 24,651 (2.0×) | 16,807 (1.4×) | 34,582 (2.8×) | 24,196 (2.0×) |
| div_u64, constant time | P-521 base | 20,383 | 42,815 (2.1×) | 26,209 (1.3×) | 64,651 (3.2×) | 43,315 (2.1×) |
| div_uint, constant time | Mersenne-31 | 251 | 226 (0.9×) | 226 (0.9×) | 227 (0.9×) | 227 (0.9×) |
| div_uint, constant time | BabyBear | 403 | 469 (1.2×) | 374 (0.9×) | 524 (1.3×) | 441 (1.1×) |
| div_uint, constant time | Goldilocks | 665 | 1,120 (1.7×) | 811 (1.2×) | 1,368 (2.1×) | 943 (1.4×) |
| div_uint, constant time | BN254 scalar | 6,894 | 12,458 (1.8×) | 9,486 (1.4×) | 17,629 (2.6×) | 13,264 (1.9×) |
| div_uint, constant time | secp256k1 base | 6,774 | 11,875 (1.8×) | 9,058 (1.3×) | 16,873 (2.5×) | 12,900 (1.9×) |
| div_uint, constant time | P-256 base | 6,706 | 12,004 (1.8×) | 9,094 (1.4×) | 17,366 (2.6×) | 12,491 (1.9×) |
| div_uint, constant time | P-256 scalar | 6,947 | 12,406 (1.8×) | 9,535 (1.4×) | 17,784 (2.6×) | 13,340 (1.9×) |
| div_uint, constant time | Tom-256 base | 7,333 | 12,340 (1.7×) | 9,436 (1.3×) | 17,061 (2.3×) | 13,108 (1.8×) |
| div_uint, constant time | BLS12-381 base | 12,504 | 25,228 (2.0×) | 16,946 (1.4×) | 35,362 (2.8×) | 24,616 (2.0×) |
| div_uint, constant time | P-521 base | 20,377 | 42,912 (2.1×) | 26,024 (1.3×) | 64,672 (3.2×) | 43,343 (2.1×) |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- bench-wasm --chrome <chrome> --firefox <firefox> [--cpu 8,9 --browser-cpu 10,11]   # the WebAssembly builds, on a stream per measuring CPU
cargo run -p prime-field-bench-report -- bench-wasm --changed --chrome <chrome> --firefox <firefox>   # the cells whose code changed or that a stopped sweep left, with the figures they are compared with
cargo run -p prime-field-bench-report -- report   # this file
```
