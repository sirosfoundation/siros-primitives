# Benchmarks

`prime-field` at commit `e58febb` against `prime-field × invert`, `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 13; 1-minute load 32.54 before, 4.84 after; its SMT sibling, CPU 29, 46% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: the median over 31 repetitions of the cycles of 2n operations less those of n, over n.
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
| Mersenne-31 | **1.9** / **2.9** | – | 10.1 / 18.8 | – | – | 2.2 / 3.0 | – | 2.6 / 5.0 | – |
| BabyBear | **1.9** / **2.9** | – | 9.7 / 18.6 | – | – | 2.0 / 3.0 | – | 2.6 / 5.0 | – |
| Goldilocks | **2.0** / **2.9** | – | 9.8 / 19.6 | – | – | 2.0\* / 3.0 | 2.4 / 3.0 | 2.6 / 4.9 | – |
| BN254 scalar | **5.1** / **6.7** | 17.4 / 12.6 | 66.6 / 92.6 | – | 14.5 / 21.7 | – | – | 7.2 / 10.7 | – |
| secp256k1 base | **6.6** / **7.0** | 17.6 / 13.2 | 75.9 / 93.9 | 3.4\* / 3.1\* | 14.6 / 21.3 | – | – | 7.0 / 10.8 | – |
| P-256 base | **6.1** / **6.9** | 18.7 / 14.3 | 66.6 / 92.7 | 10.7 / 18.5 | 10.8 / 20.8 | – | – | 7.1 / 10.8 | 349 / – |
| P-256 scalar | **5.8** / **7.0** | 17.8 / 13.8 | 66.6 / 94.0 | 17.8 / 22.8 | 19.7 / 21.8 | – | – | 7.7 / 10.8 | 351 / – |
| Tom-256 base | **6.0** / **7.0** | – | 71.7 / 91.4 | – | 15.8 / 21.5 | – | – | 7.1 / 10.8 | – |
| BLS12-381 base | **10.2** / **9.0** | 20.3 / 15.8 | 95.5 / 112 | – | 20.6 / 27.4 | – | – | 17.0 / 12.9 | – |
| P-521 base | **15.7** / **20.2** | – | 117 / 155 | – | – | – | – | 25.1 / 52.6 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 12.9 | – | – | 2.2 / 3.0 | – | 2.0 / 4.0 | – |
| BabyBear | 2.1 / **2.9** | – | 4.3 / 12.3 | – | – | **2.0** / 3.1 | – | **2.0** / 4.0 | – |
| Goldilocks | **1.9** / 3.0 | – | 4.2 / 12.6 | – | – | 2.0 / 2.9\* | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.3** / **7.0** | 17.8 / 14.5 | 51.6 / 51.1 | – | 8.8 / 11.8 | – | – | 7.2 / 8.0 | – |
| secp256k1 base | **5.6** / 9.6 | 17.3 / 13.9 | 50.6 / 48.4 | 3.8\* / 4.5\* | 10.4 / 12.8 | – | – | 6.7 / **8.0** | – |
| P-256 base | **5.6** / **6.9** | 16.8 / 12.7 | 49.8 / 43.6 | 7.9 / 12.8 | 7.8 / 11.8 | – | – | 6.8 / 8.0 | 158 / – |
| P-256 scalar | **5.1** / **6.8** | 16.1 / 13.5 | 49.8 / 49.3 | 8.7 / 12.9 | 8.1 / 12.8 | – | – | 6.9 / 7.8 | 155 / – |
| Tom-256 base | **5.6** / 8.2 | – | 54.3 / 55.0 | – | 11.1 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **10.0** / **10.0** | 22.3 / 17.3 | 83.9 / 95.1 | – | 15.5 / 19.9 | – | – | 16.9 / 11.0 | – |
| P-521 base | **12.2** / **12.7** | – | 122 / 140 | – | – | – | – | 28.1 / 51.5 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | – | 16.9 / 37.6 | – | – | 2.8 / 7.0 | – | 7.0 / 17.0 | – |
| BabyBear | **3.5** / **11.0** | – | 16.9 / 37.8 | – | – | 4.8 / 13.0 | – | 7.0 / 17.0 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.6 / 34.8 | – | – | 3.2\* / 9.9 | **3.7** / 10.9 | 7.0 / 17.0 | – |
| BN254 scalar | **69.6** / **73.1** | 75.6 / 105 | 130 / 167 | – | 96.8 / 94.2 | – | – | 131 / 130 | – |
| secp256k1 base | **44.7** / **54.8** | 91.3 / 115 | 119 / 163 | 85.0 / 88.1 | 88.2 / 96.0 | – | – | 94.7 / 130 | – |
| P-256 base | **50.7** / 60.0 | 67.4 / 87.4 | 119 / 158 | 67.9 / 73.7 | 68.8 / 73.0 | – | – | 51.0 / **58.1** | 618 / – |
| P-256 scalar | **74.7** / **83.8** | 92.8 / 114 | 119 / 166 | 145 / 141 | 96.1 / 104 | – | – | 95.2 / 131 | 712 / – |
| Tom-256 base | **70.9** / **76.0** | – | 120 / 166 | – | 95.6 / 97.3 | – | – | 99.4 / 131 | – |
| BLS12-381 base | **139** / **141** | 167 / 195 | 272 / 296 | – | 195 / 195 | – | – | 255 / 276 | – |
| P-521 base | **178** / **192** | – | 534 / 550 | – | – | – | – | 520 / 506 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 17.0 / 34.5 | – | – | 2.7 / 7.0 | – | 7.6 / 17.2 | – |
| BabyBear | **3.7** / **11.0** | – | 16.7 / 34.0 | – | – | 4.9 / 13.0 | – | 7.6 / 17.0 | – |
| Goldilocks | **3.8** / **9.0** | – | 17.5 / 36.3 | – | – | 3.2\* / 10.0 | 4.2 / 11.0 | 7.6 / 17.0 | – |
| BN254 scalar | **63.9** / **67.1** | 74.5 / 105 | 120 / 155 | – | 74.2 / 89.4 | – | – | 94.6 / 130 | – |
| secp256k1 base | **33.9** / **43.0** | 84.7 / 102 | 118 / 154 | 57.9 / 79.9 | 76.0 / 84.9 | – | – | 94.5 / 132 | – |
| P-256 base | **40.6** / **44.0** | 64.8 / 82.1 | 135 / 161 | 61.1 / 72.0 | 55.0 / 58.1 | – | – | 50.6 / 58.0 | 644 / – |
| P-256 scalar | **63.9** / **73.6** | 89.7 / 109 | 119 / 154 | 135 / 150 | 77.5 / 93.5 | – | – | 93.9 / 129 | 662 / – |
| Tom-256 base | **57.0** / **64.5** | – | 117 / 154 | – | 79.8 / 85.6 | – | – | 122 / 132 | – |
| BLS12-381 base | **133** / **137** | 140 / 167 | 272 / 294 | – | 152 / 166 | – | – | 256 / 274 | – |
| P-521 base | **121** / **133** | – | 539 / 563 | – | – | – | – | 534 / 497 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **220** / – | – | 1,461 / – | – | – | 645 / – | – | 909 / – | – |
| BabyBear | **219** / – | – | 1,438 / – | – | – | 629 / – | – | 1,084 / – | – |
| Goldilocks | **436** / – | – | 2,991 / – | – | – | 1,330 / – | 693 / – | 1,765 / – | – |
| BN254 scalar | **5,135** / – | 17,953 / – | 16,237 / – | – | 8,144 / – | – | – | 15,906 / – | – |
| secp256k1 base | **5,290** / – | 17,209 / – | 15,623 / – | 27,073 / – | 7,803 / – | – | – | 16,119 / – | – |
| P-256 base | **5,092** / – | 16,149 / – | 14,953 / – | 30,714 / – | 7,336 / – | – | – | 14,605 / – | 93,870 / – |
| P-256 scalar | **5,087** / – | 16,693 / – | 15,645 / – | 30,527 / – | 7,279 / – | – | – | 14,722 / – | 90,516 / – |
| Tom-256 base | **5,039** / – | – | 14,658 / – | – | 7,221 / – | – | – | 14,301 / – | – |
| BLS12-381 base | **8,931** / – | 36,738 / – | 25,260 / – | – | 12,243 / – | – | – | 35,236 / – | – |
| P-521 base | **14,279** / – | – | 44,633 / – | – | – | – | – | 71,904 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **137** / – | – | 1,653 / – | – | – | 145 / – | 450 / – | – |
| BabyBear | **2,005** / – | – | 18,676 / – | – | – | 2,787 / – | – | – |
| Goldilocks | **2,833** / – | – | 26,889 / – | – | – | 3,233 / – | – | – |
| BN254 scalar | **34,566** / – | 45,313 / – | 137,847 / – | – | 85,849 / – | – | – | – |
| secp256k1 base | **12,331** / – | 43,290 / – | 53,651 / – | 24,062 / – | 47,599 / – | – | 72,876 / – | – |
| P-256 base | **11,962** / – | 18,715 / – | 43,436 / – | 30,193 / – | 34,276 / – | – | 29,830 / – | 168,078 / – |
| P-256 scalar | **24,237** / – | 39,419 / – | 58,366 / – | 83,118 / – | 42,215 / – | – | – | – |
| Tom-256 base | **21,275** / – | – | 49,768 / – | – | 51,276 / – | – | 55,692 / – | – |
| BLS12-381 base | **66,158** / – | 90,684 / – | 150,971 / – | – | 149,107 / – | – | 199,014 / – | – |
| P-521 base | **68,482** / – | – | 305,738 / – | – | – | – | 283,560 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **8.9** | – | 34.7 / 36.6 | – | – | 5.5\* / 6.9\* | – | – |
| BabyBear | **8.9** / **11.0** | – | 34.4 / 36.5 | – | – | 12.3 / 15.0 | – | – |
| Goldilocks | **7.1** / **10.1** | – | 38.2 / 39.2 | – | – | 3.2\* / 9.9\* | 7.3 / 10.9 | – |
| BN254 scalar | **20.9** / **25.9** | 237 / 235 | 281 / 278 | – | 241 / 236 | – | – | 255 / 254 |
| secp256k1 base | **9.8** / **14.9** | 234 / 236 | 285 / 282 | 97.1 / 97.4 | 224 / 220 | – | – | 256 / 253 |
| P-256 base | **19.0** / **25.0** | 209 / 208 | 284 / 276 | 147 / 147 | 191 / 196 | – | – | 132 / 140 |
| P-256 scalar | **22.8** / **27.9** | 260 / 260 | 282 / 278 | 139 / 148 | 248 / 242 | – | – | 255 / 254 |
| Tom-256 base | **20.6** / **28.0** | – | 283 / 285 | – | 230 / 256 | – | – | 257 / 253 |
| BLS12-381 base | **28.9** / **34.5** | 431 / 426 | 561 / 553 | – | 440 / 432 | – | – | 624 / 603 |
| P-521 base | **19.9** / **33.1** | – | 961 / 935 | – | – | – | – | 994 / 987 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **9.0** | 34.7 / 39.0 | – | – | 5.5\* / 7.0\* | – |
| BabyBear | **8.9** / **10.9** | 35.0 / 36.5 | – | – | 12.2 / 15.0 | – |
| Goldilocks | **7.1** / **9.9** | 38.1 / 39.2 | – | – | 3.2\* / 9.9\* | 7.3 / 10.9 |
| BN254 scalar | **106** / **114** | 288 / 283 | – | 186 / 186 | – | – |
| secp256k1 base | **57.3** / **68.4** | 289 / 287 | – | 158 / 155 | – | – |
| P-256 base | **107** / **110** | 285 / 284 | – | 130 / 134 | – | – |
| P-256 scalar | **112** / **117** | 292 / 286 | 194 / 187 | 197 / 189 | – | – |
| Tom-256 base | **110** / **115** | 284 / 285 | – | 170 / 173 | – | – |
| BLS12-381 base | **216** / **223** | 552 / 554 | – | 372 / 378 | – | – |
| P-521 base | **359** / **410** | 922 / 929 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | – | 33.1 / 26.7 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **6.0** / **7.9** | – | 32.7 / 30.8 | – | – | 9.0 / 8.0 | – | – |
| Goldilocks | **5.2** / **4.8** | – | 39.1 / 35.2 | – | – | 1.9\* / 3.0\* | 6.1 / 6.1 | – |
| BN254 scalar | **24.9** / **24.9** | 124 / 129 | 233 / 232 | – | 144 / 153 | – | – | 115 / 112 |
| secp256k1 base | **5.9** / **7.4** | 101 / 105 | 245 / 230 | 3.0\* / 2.2\* | 137 / 152 | – | – | 115 / 112 |
| P-256 base | **25.9** / **22.4** | 105 / 104 | 241 / 234 | 76.6 / 86.1 | 119 / 132 | – | – | 59.0 / 66.6 |
| P-256 scalar | 25.4 / 22.8 | 132 / 131 | 232 / 230 | **14.4** / **20.8** | 152 / 167 | – | – | 115 / 112 |
| Tom-256 base | **26.6** / **26.5** | – | 232 / 231 | – | 177 / 162 | – | – | 115 / 112 |
| BLS12-381 base | **38.5** / **38.9** | 230 / 225 | 382 / 386 | – | 285 / 261 | – | – | 331 / 338 |
| P-521 base | **11.4** / **12.7** | – | 597 / 604 | – | – | – | – | 538 / 545 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 34.4 / 26.8 | – | – | 4.1\* / 4.1\* | – |
| BabyBear | **6.0** / **7.0** | 32.8 / 32.1 | – | – | 9.0 / 8.0 | – |
| Goldilocks | **5.2** / **4.8** | 39.2 / 35.8 | – | – | 1.9\* / 3.0\* | 6.1 / 6.1 |
| BN254 scalar | **88.0** / **83.9** | 238 / 235 | – | 101 / 108 | – | – |
| secp256k1 base | **6.8** / **8.5** | 239 / 234 | – | 83.8 / 94.8 | – | – |
| P-256 base | **62.7** / **62.4** | 234 / 239 | – | 76.0 / 85.6 | – | – |
| P-256 scalar | 88.9 / 89.8 | 248 / 250 | **48.8** / **45.9** | 114 / 119 | – | – |
| Tom-256 base | **87.4** / **83.5** | 239 / 239 | – | 93.9 / 108 | – | – |
| BLS12-381 base | **170** / **170** | 377 / 382 | – | 206 / 214 | – | – |
| P-521 base | **23.6** / **23.9** | 598 / 606 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | – | 16.5 / 18.0 | – | – | 4.6\* / 4.1\* | – | – |
| BabyBear | **6.8** / **7.0** | – | 17.6 / 18.6 | – | – | 9.0 / 9.0 | – | – |
| Goldilocks | **4.7** / **4.4** | – | 27.0 / 22.4 | – | – | 2.0\* / 2.9\* | 4.9 / 5.6 | – |
| BN254 scalar | **24.9** / **24.9** | 126 / 121 | 183 / 187 | – | 146 / 144 | – | – | 113 / 110 |
| secp256k1 base | **4.2** / **8.9** | 102 / 104 | 191 / 182 | 3.1\* / 3.0\* | 126 / 126 | – | – | 113 / 110 |
| P-256 base | **20.2** / **21.9** | 100.0 / 97.1 | 180 / 183 | 78.5 / 83.4 | 123 / 121 | – | – | 61.9 / 61.9 |
| P-256 scalar | 24.9 / 23.7 | 130 / 128 | 199 / 184 | **6.6** / **12.4** | 155 / 153 | – | – | 113 / 110 |
| Tom-256 base | **25.0** / **25.5** | – | 186 / 191 | – | 146 / 149 | – | – | 122 / 123 |
| BLS12-381 base | **38.3** / **39.1** | 229 / 226 | 364 / 366 | – | 255 / 257 | – | – | 330 / 322 |
| P-521 base | **9.7** / **11.1** | – | 589 / 607 | – | – | – | – | 529 / 546 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.1 | – | – | 4.6\* / 4.2\* | – |
| BabyBear | **7.8** / **7.1** | 17.9 / 18.4 | – | – | 9.0 / 9.0 | – |
| Goldilocks | **4.7** / **4.4** | 27.2 / 25.9 | – | – | 1.9\* / 2.9\* | 4.9 / 5.6 |
| BN254 scalar | **87.9** / **84.1** | 185 / 189 | – | 111 / 105 | – | – |
| secp256k1 base | **5.8** / **6.8** | 193 / 183 | – | 81.1 / 82.8 | – | – |
| P-256 base | **62.4** / **62.0** | 181 / 179 | – | 79.5 / 78.6 | – | – |
| P-256 scalar | 88.7 / 87.0 | 189 / 178 | **43.5** / **38.0** | 106 / 104 | – | – |
| Tom-256 base | **84.1** / **82.0** | 187 / 188 | – | 103 / 96.4 | – | – |
| BLS12-381 base | **174** / **170** | 366 / 369 | – | 209 / 210 | – | – |
| P-521 base | **21.0** / **16.1** | 592 / 613 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.7 / – | – | 13.2 / – | – | – | **3.3** / – | – | – |
| BabyBear | **6.3** / – | – | 13.3 / – | – | – | 8.0 / – | – | – |
| Goldilocks | 4.1 / – | – | 17.6 / – | – | – | 1.0\* / – | **3.5** / – | – |
| BN254 scalar | **22.1** / – | 138 / – | 119 / – | – | 128 / – | – | – | 94.7 / – |
| secp256k1 base | 2.3 / – | 128 / – | 121 / – | **2.2** / – | 110 / – | – | – | 94.6 / – |
| P-256 base | **17.9** / – | 121 / – | 117 / – | 59.7 / – | 111 / – | – | – | 55.1 / – |
| P-256 scalar | 18.1 / – | 148 / – | 119 / – | **2.3** / – | 134 / – | – | – | 94.4 / – |
| Tom-256 base | **21.0** / – | – | 119 / – | – | 129 / – | – | – | 94.8 / – |
| BLS12-381 base | **29.1** / – | 230 / – | 265 / – | – | 227 / – | – | – | 340 / – |
| P-521 base | **5.1** / – | – | 448 / – | – | – | – | – | 533 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### div_u64: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – | – |
| BN254 scalar | **617** / **625** | 4,165 / 4,159 | 16,570 / 16,645 | 14,268 / 13,539 | – | 6,240 / 6,246 | 11,517 / 10,654 |
| secp256k1 base | **574** / **586** | 4,351 / 4,204 | 15,376 / 14,585 | 16,183 / 15,680 | 25,294 / 24,998 | 6,884 / 7,064 | 12,508 / 10,702 |
| P-256 base | **629** / **624** | 4,207 / 4,192 | 15,070 / 15,006 | 14,263 / 13,506 | 30,532 / 30,436 | 6,191 / 6,172 | 10,494 / 10,527 |
| P-256 scalar | **633** / **631** | 4,123 / 4,101 | 16,326 / 16,214 | 14,332 / 13,597 | 22,338 / 22,121 | 6,840 / 6,540 | 12,432 / 11,804 |
| Tom-256 base | **630** / **626** | 4,364 / 4,440 | – | 14,736 / 13,807 | – | 6,253 / 6,201 | 11,508 / 10,700 |
| BLS12-381 base | **768** / **771** | 7,141 / 7,137 | 35,254 / 34,967 | 23,488 / 23,049 | – | 9,881 / 9,881 | 26,249 / 25,796 |
| P-521 base | **949** / **949** | 12,055 / 12,089 | – | 40,293 / 40,756 | – | – | 62,578 / 63,090 |

### div_uint: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **223** / **221** | 225 / 225 | – | 1,478 / 1,458 | – | – | 661 / 652 | – | 952 / 956 |
| BabyBear | **218** / **220** | 237 / 235 | – | 1,487 / 1,452 | – | – | 635 / 628 | – | 1,076 / 1,076 |
| Goldilocks | **440** / **438** | 454 / 455 | – | 2,926 / 2,921 | – | – | 1,302 / 1,292 | 706 / 706 | 1,756 / 1,758 |
| BN254 scalar | **4,981** / **4,982** | 5,054 / 5,044 | 16,941 / 16,909 | 16,189 / 16,310 | – | 7,997 / 7,980 | – | – | 16,808 / 16,397 |
| secp256k1 base | 5,108 / **5,055** | **4,932** / 5,233 | 15,995 / 15,906 | 16,714 / 14,912 | 27,547 / 25,456 | 7,425 / 7,408 | – | – | 15,652 / 14,561 |
| P-256 base | 5,191 / 5,166 | **5,133** / **5,112** | 15,638 / 15,624 | 15,520 / 14,776 | 31,069 / 31,255 | 7,370 / 7,377 | – | – | 14,338 / 14,316 |
| P-256 scalar | 5,027 / **5,021** | **4,999** / 5,057 | 17,972 / 17,647 | 16,745 / 17,234 | 34,490 / 32,917 | 7,485 / 7,666 | – | – | 16,043 / 15,328 |
| Tom-256 base | **5,252** / 5,521 | 5,372 / **5,142** | – | 15,560 / 15,027 | – | 7,418 / 7,413 | – | – | 15,333 / 14,502 |
| BLS12-381 base | 9,299 / 9,253 | **9,144** / **9,162** | 37,037 / 37,091 | 26,266 / 25,843 | – | 12,400 / 12,403 | – | – | 36,352 / 35,025 |
| P-521 base | **15,086** / 15,692 | 16,117 / **15,266** | – | 47,628 / 46,302 | – | – | – | – | 85,526 / 85,384 |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 10.1 / 18.8 | – | – | – | 2.6 / 5.0 |
| BabyBear | **2.0** / **3.0** | 9.7 / 18.6 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **1.9** / **3.0** | 9.8 / 19.6 | – | – | 2.4 / **3.0** | 2.6 / 4.9 |
| BN254 scalar | **5.2** / **6.8** | 66.6 / 92.6 | – | 14.5 / 21.7 | – | 7.2 / 10.7 |
| secp256k1 base | **6.1** / **6.8** | 75.9 / 93.9 | 3.4\* / 3.1\* | 14.6 / 21.3 | – | 7.0 / 10.8 |
| P-256 base | **5.9** / **6.9** | 66.6 / 92.7 | 10.7 / 18.5 | 10.8 / 20.8 | – | 7.1 / 10.8 |
| P-256 scalar | **5.8** / **6.5** | 66.6 / 94.0 | 17.8 / 22.8 | 19.7 / 21.8 | – | 7.7 / 10.8 |
| Tom-256 base | 7.6 / **6.8** | 71.7 / 91.4 | – | 15.8 / 21.5 | – | **7.1** / 10.8 |
| BLS12-381 base | **10.0** / **8.7** | 95.5 / 112 | – | 20.6 / 27.4 | – | 17.0 / 12.9 |
| P-521 base | **12.6** / **15.1** | 117 / 155 | – | – | – | 25.1 / 52.6 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 4.3 / 12.9 | – | – | 2.2 / 3.0 | – | **2.0** / 4.0 |
| BabyBear | **2.0** / **2.9** | 4.3 / 12.3 | – | – | – | – | **2.0** / 4.0 |
| Goldilocks | **1.9** / **2.9** | 4.2 / 12.6 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.2** / **6.8** | 51.6 / 51.1 | – | 8.8 / 11.8 | – | – | 7.2 / 8.0 |
| secp256k1 base | **5.7** / 9.3 | 50.6 / 48.4 | 3.8\* / 4.5\* | 10.4 / 12.8 | – | – | 6.7 / **8.0** |
| P-256 base | **5.4** / **7.1** | 49.8 / 43.6 | 7.9 / 12.8 | 7.8 / 11.8 | – | – | 6.8 / 8.0 |
| P-256 scalar | **5.1** / **6.8** | 49.8 / 49.3 | 8.7 / 12.9 | 8.1 / 12.8 | – | – | 6.9 / 7.8 |
| Tom-256 base | **5.7** / **8.0** | 54.3 / 55.0 | – | 11.1 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **11.1** / **10.8** | 83.9 / 95.1 | – | 15.5 / 19.9 | – | – | 16.9 / 11.0 |
| P-521 base | **12.4** / **18.6** | 122 / 140 | – | – | – | – | 28.1 / 51.5 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 37.6 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.9 / 37.8 | – | – | – | 7.0 / 17.0 |
| Goldilocks | 3.8 / **9.0** | 17.6 / 34.8 | – | – | **3.7** / 10.9 | 7.0 / 17.0 |
| BN254 scalar | **69.5** / **79.0** | 130 / 167 | – | 96.8 / 94.2 | – | 131 / 130 |
| secp256k1 base | **45.1** / **57.6** | 119 / 163 | 85.0 / 88.1 | 88.2 / 96.0 | – | 94.7 / 130 |
| P-256 base | **50.4** / 59.8 | 119 / 158 | 67.9 / 73.7 | 68.8 / 73.0 | – | 51.0 / **58.1** |
| P-256 scalar | **76.4** / **84.2** | 119 / 166 | 145 / 141 | 96.1 / 104 | – | 95.2 / 131 |
| Tom-256 base | **71.0** / **76.1** | 120 / 166 | – | 95.6 / 97.3 | – | 99.4 / 131 |
| BLS12-381 base | **150** / **154** | 272 / 296 | – | 195 / 195 | – | 255 / 276 |
| P-521 base | **179** / **192** | 534 / 550 | – | – | – | 520 / 506 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 17.0 / 34.5 | – | – | – | 7.6 / 17.2 |
| BabyBear | **3.7** / **11.0** | 16.7 / 34.0 | – | – | – | 7.6 / 17.0 |
| Goldilocks | **3.8** / **9.0** | 17.5 / 36.3 | – | – | 4.2 / 11.0 | 7.6 / 17.0 |
| BN254 scalar | **66.2** / **67.0** | 120 / 155 | – | 74.2 / 89.4 | – | 94.6 / 130 |
| secp256k1 base | **33.9** / **42.9** | 118 / 154 | 57.9 / 79.9 | 76.0 / 84.9 | – | 94.5 / 132 |
| P-256 base | **40.4** / **44.2** | 135 / 161 | 61.1 / 72.0 | 55.0 / 58.1 | – | 50.6 / 58.0 |
| P-256 scalar | **64.1** / **73.1** | 119 / 154 | 135 / 150 | 77.5 / 93.5 | – | 93.9 / 129 |
| Tom-256 base | **57.0** / **64.5** | 117 / 154 | – | 79.8 / 85.6 | – | 122 / 132 |
| BLS12-381 base | **140** / **147** | 272 / 294 | – | 152 / 166 | – | 256 / 274 |
| P-521 base | **121** / **133** | 539 / 563 | – | – | – | 534 / 497 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **258** / – | 3,855 / – | – | – | – | 953 / – |
| BabyBear | **383** / – | 3,850 / – | – | – | – | 949 / – |
| Goldilocks | **648** / – | 3,923 / – | – | – | 693 / – | 2,069 / – |
| BN254 scalar | **6,842** / – | 20,063 / – | – | 46,753 / – | – | 55,465 / – |
| secp256k1 base | **6,898** / – | 19,488 / – | 27,073 / – | 46,898 / – | – | 70,364 / – |
| P-256 base | **6,561** / – | 19,491 / – | 30,714 / – | 33,596 / – | – | 34,575 / – |
| P-256 scalar | **7,004** / – | 19,493 / – | 78,544 / – | 49,551 / – | – | 62,217 / – |
| Tom-256 base | **7,406** / – | 20,681 / – | – | 45,382 / – | – | 54,253 / – |
| BLS12-381 base | **12,626** / – | 36,197 / – | – | 144,292 / – | – | 190,160 / – |
| P-521 base | **20,741** / – | 69,920 / – | – | – | – | 557,124 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,653 / – | – | – | 450 / – |
| BabyBear | **5,602** / – | 18,676 / – | – | – | – |
| Goldilocks | **7,156** / – | 26,889 / – | – | – | – |
| BN254 scalar | **59,275** / – | 137,847 / – | – | 85,849 / – | – |
| secp256k1 base | **12,326** / – | 53,651 / – | 24,062 / – | 47,599 / – | 72,876 / – |
| P-256 base | **11,996** / – | 43,436 / – | 30,193 / – | 34,276 / – | 29,830 / – |
| P-256 scalar | **26,730** / – | 58,366 / – | 83,118 / – | 42,215 / – | – |
| Tom-256 base | **21,265** / – | 49,768 / – | – | 51,276 / – | 55,692 / – |
| BLS12-381 base | **74,737** / – | 150,971 / – | – | 149,107 / – | 199,014 / – |
| P-521 base | **70,636** / – | 305,738 / – | – | – | 283,560 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **9.0** | 34.7 / 36.6 | – | – | – |
| BabyBear | **9.0** / **10.9** | 34.4 / 36.5 | – | – | – |
| Goldilocks | **7.1** / **10.0** | 38.2 / 39.2 | – | – | 7.3 / 10.9 |
| BN254 scalar | **22.0** / **30.4** | 281 / 278 | – | 241 / 236 | – |
| secp256k1 base | **11.0** / **19.9** | 285 / 282 | 97.1 / 97.4 | 224 / 220 | – |
| P-256 base | **24.5** / **32.9** | 284 / 276 | 147 / 147 | 191 / 196 | – |
| P-256 scalar | **22.9** / **33.9** | 282 / 278 | 139 / 148 | 248 / 242 | – |
| Tom-256 base | **22.6** / **33.0** | 283 / 285 | – | 230 / 256 | – |
| BLS12-381 base | **38.0** / **42.2** | 561 / 553 | – | 440 / 432 | – |
| P-521 base | **26.5** / **45.8** | 961 / 935 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **9.0** | 34.7 / 39.0 | – | – | – |
| BabyBear | **9.0** / **11.0** | 35.0 / 36.5 | – | – | – |
| Goldilocks | **7.1** / **9.9** | 38.1 / 39.2 | – | – | 7.3 / 10.9 |
| BN254 scalar | **123** / **141** | 288 / 283 | – | 186 / 186 | – |
| secp256k1 base | **78.3** / **83.9** | 289 / 287 | – | 158 / 155 | – |
| P-256 base | **123** / 137 | 285 / 284 | – | 130 / **134** | – |
| P-256 scalar | **132** / **146** | 292 / 286 | 194 / 187 | 197 / 189 | – |
| Tom-256 base | **122** / **136** | 284 / 285 | – | 170 / 173 | – |
| BLS12-381 base | **255** / **267** | 552 / 554 | – | 372 / 378 | – |
| P-521 base | **428** / **475** | 922 / 929 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **7.4** | 33.1 / 26.7 | – | – | – |
| BabyBear | **6.0** / **7.0** | 32.7 / 30.8 | – | – | – |
| Goldilocks | **5.3** / **4.8** | 39.1 / 35.2 | – | – | 6.1 / 6.1 |
| BN254 scalar | **27.4** / **28.9** | 233 / 232 | – | 144 / 153 | – |
| secp256k1 base | **5.9** / **7.4** | 245 / 230 | 3.0\* / 2.2\* | 137 / 152 | – |
| P-256 base | **27.0** / **24.4** | 241 / 234 | 76.6 / 86.1 | 119 / 132 | – |
| P-256 scalar | 28.3 / 27.8 | 232 / 230 | **14.4** / **20.8** | 152 / 167 | – |
| Tom-256 base | **28.6** / **27.4** | 232 / 231 | – | 177 / 162 | – |
| BLS12-381 base | **43.9** / **45.1** | 382 / 386 | – | 285 / 261 | – |
| P-521 base | **11.4** / **12.9** | 597 / 604 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 34.4 / 26.8 | – | – | – |
| BabyBear | **6.0** / **7.0** | 32.8 / 32.1 | – | – | – |
| Goldilocks | **5.2** / **4.8** | 39.2 / 35.8 | – | – | 6.1 / 6.1 |
| BN254 scalar | **92.8** / **91.8** | 238 / 235 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **8.7** | 239 / 234 | – | 83.8 / 94.8 | – |
| P-256 base | **70.3** / **69.0** | 234 / 239 | – | 76.0 / 85.6 | – |
| P-256 scalar | 95.8 / 89.8 | 248 / 250 | **48.8** / **45.9** | 114 / 119 | – |
| Tom-256 base | **84.9** / **86.7** | 239 / 239 | – | 93.9 / 108 | – |
| BLS12-381 base | **178** / **179** | 377 / 382 | – | 206 / 214 | – |
| P-521 base | **22.5** / **24.0** | 598 / 606 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.0 | – | – | – |
| BabyBear | **7.5** / **7.1** | 17.6 / 18.6 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 27.0 / 22.4 | – | – | 4.9 / 5.6 |
| BN254 scalar | **27.9** / **29.9** | 183 / 187 | – | 146 / 144 | – |
| secp256k1 base | **4.2** / **7.9** | 191 / 182 | 3.1\* / 3.0\* | 126 / 126 | – |
| P-256 base | **23.4** / **23.0** | 180 / 183 | 78.5 / 83.4 | 123 / 121 | – |
| P-256 scalar | 26.9 / 28.2 | 199 / 184 | **6.6** / **12.4** | 155 / 153 | – |
| Tom-256 base | **28.3** / **28.1** | 186 / 191 | – | 146 / 149 | – |
| BLS12-381 base | **42.9** / **45.3** | 364 / 366 | – | 255 / 257 | – |
| P-521 base | **9.7** / **11.1** | 589 / 607 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 16.5 / 18.1 | – | – | – |
| BabyBear | **7.5** / **7.2** | 17.9 / 18.4 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 27.2 / 25.9 | – | – | 4.9 / 5.6 |
| BN254 scalar | **92.0** / **93.2** | 185 / 189 | – | 111 / 105 | – |
| secp256k1 base | **7.3** / **7.8** | 193 / 183 | – | 81.1 / 82.8 | – |
| P-256 base | **64.8** / **68.3** | 181 / 179 | – | 79.5 / 78.6 | – |
| P-256 scalar | 90.9 / 94.2 | 189 / 178 | **43.5** / **38.0** | 106 / 104 | – |
| Tom-256 base | **91.6** / **86.5** | 187 / 188 | – | 103 / 96.4 | – |
| BLS12-381 base | **178** / **174** | 366 / 369 | – | 209 / 210 | – |
| P-521 base | **19.3** / **20.0** | 592 / 613 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **13.2** / – | – | – | – |
| BabyBear | **13.3** / – | – | – | – |
| Goldilocks | 17.6 / – | – | – | **3.5** / – |
| BN254 scalar | **119** / – | – | 128 / – | – |
| secp256k1 base | 121 / – | **2.2** / – | 110 / – | – |
| P-256 base | 117 / – | **59.7** / – | 111 / – | – |
| P-256 scalar | 119 / – | **2.3** / – | 134 / – | – |
| Tom-256 base | **119** / – | – | 129 / – | – |
| BLS12-381 base | 265 / – | – | **227** / – | – |
| P-521 base | **448** / – | – | – | – |

### div_u64, constant time: a / k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | – | – | – | – | – | – |
| BabyBear | – | – | – | – | – | – |
| Goldilocks | – | – | – | – | – | – |
| BN254 scalar | **7,085** / 7,306 | 7,632 / **6,974** | 20,933 / 21,251 | – | 46,561 / 46,132 | 55,559 / 55,978 |
| secp256k1 base | **6,879** / **6,867** | 6,892 / 6,891 | 20,931 / 19,845 | 25,294 / 24,998 | 46,817 / 46,822 | 69,578 / 69,891 |
| P-256 base | **6,721** / **6,712** | 6,787 / 6,770 | 20,594 / 19,796 | 30,532 / 30,436 | 34,687 / 35,208 | 34,945 / 35,049 |
| P-256 scalar | **7,026** / 7,340 | 7,305 / **6,984** | 21,535 / 20,176 | 79,282 / 78,856 | 49,297 / 49,218 | 60,574 / 60,801 |
| Tom-256 base | **6,788** / **6,842** | 7,346 / 7,345 | 20,618 / 19,879 | – | 44,307 / 43,974 | 51,986 / 52,211 |
| BLS12-381 base | 12,568 / 12,559 | **12,499** / **12,463** | 35,810 / 35,775 | – | 144,970 / 143,530 | 212,277 / 180,495 |
| P-521 base | 20,882 / **20,049** | **20,594** / 20,465 | 71,903 / 65,502 | – | – | 537,116 / 499,569 |

### div_uint, constant time: a / k, k of one bit less than p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | prime-field × invert | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **272** / **259** | 305 / 289 | 4,052 / 3,910 | – | – | – | 993 / 993 |
| BabyBear | **405** / **403** | 421 / 416 | 3,900 / 3,892 | – | – | – | 993 / 994 |
| Goldilocks | 669 / 676 | **652** / **671** | 3,911 / 3,910 | – | – | 706 / 706 | 2,117 / 2,117 |
| BN254 scalar | **6,956** / **6,891** | 7,080 / 7,082 | 22,809 / 23,919 | – | 50,245 / 49,169 | – | 60,171 / 56,457 |
| secp256k1 base | 7,265 / **6,892** | **7,256** / 6,912 | 21,039 / 19,913 | 27,547 / 25,456 | 46,770 / 46,794 | – | 69,797 / 69,117 |
| P-256 base | **6,719** / **6,708** | 6,805 / 6,809 | 20,583 / 19,818 | 31,069 / 31,255 | 33,191 / 33,178 | – | 34,681 / 34,921 |
| P-256 scalar | **7,071** / **7,146** | 7,699 / 7,196 | 21,447 / 23,097 | 80,976 / 80,135 | 52,062 / 51,580 | – | 65,698 / 64,842 |
| Tom-256 base | **7,040** / **6,859** | 7,423 / 7,411 | 20,715 / 19,879 | – | 44,346 / 43,799 | – | 51,925 / 51,887 |
| BLS12-381 base | **12,565** / **12,579** | 12,637 / 12,580 | 35,829 / 35,251 | – | 142,031 / 140,641 | – | 182,951 / 187,407 |
| P-521 base | **22,181** / **21,895** | 22,584 / 22,045 | 77,149 / 72,844 | – | – | – | 580,004 / 500,370 |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
