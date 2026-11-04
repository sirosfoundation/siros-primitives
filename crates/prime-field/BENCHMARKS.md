# Benchmarks

`prime-field` at commit `d04683c` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 12; 1-minute load 34.39 before, 35.91 after; its SMT sibling, CPU 28, 78% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: the median over 31 repetitions of the cycles of 2n operations less those of n, over n.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands: canonical residues, the same for every crate.
- Integer operands `k`: uniform of exactly 64 bits, or of exactly the bit length of `p`, which may reach `p`. A crate without an operation on an integer runs its own conversion of `k` and then its operation on two elements.
- Every result matches `num-bigint` before it is measured.
- `prime-field` runs `Fp` in the tables without a label and `ConstantTime` in the constant-time tables.

## Functions

| Operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---|---|---|---|---|---|---|---|---|
| add | `+` | `+` | `+` | `+` | `+` | `+` | `+` | `addf` | `addf` |
| sub | `-` | `-` | `-` | `-` | `-` | `-` | `-` | `subf` | `subf` |
| mul | `*` | `*` | `*` | `*` | `*` | `*` | `*` | `mulf` | `mulf` |
| square | `square` | `square` | `square` | `square` | `square` | `square` | `square` | `mulf(a, a)` | `mulf(a, a)` |
| invert | `invert` | `inverse` | `invert_vartime` | `invert`, `invert_vartime` | `invert` | `try_inverse` | `inv` | `invert` | `invert` |
| sqrt | `sqrt` | `sqrt` | `sqrt` | `sqrt` | `sqrt` | `try_sqrt` | – | `mulf to (p + 1) / 4` | `mulf to (p + 1) / 4` |
| mul_u64 | `* Uint<U64>` | `* From<u64>` | `* new` | `* from_u64`, `* From<u64>` | `* From<u64>` | `* from_int` | `* new` | `mulf of words64_to_element` | – |
| mul_uint | `* Uint` | – | `* new` | `* Reduce<U256>` | `* from_raw` | `* from_int` | `* new` | – | – |
| add_u64 | `+ Uint<U64>` | `+ From<u64>` | `+ new` | `+ from_u64`, `+ From<u64>` | `+ From<u64>` | `+ from_int` | `+ new` | `addf of words64_to_element` | – |
| add_uint | `+ Uint` | – | `+ new` | `+ Reduce<U256>` | `+ from_raw` | `+ from_int` | `+ new` | – | – |
| sub_u64 | `- Uint<U64>` | `- From<u64>` | `- new` | `- from_u64`, `- From<u64>` | `- From<u64>` | `- from_int` | `- new` | `subf of words64_to_element` | – |
| sub_uint | `- Uint` | – | `- new` | `- Reduce<U256>` | `- from_raw` | `- from_int` | `- new` | – | – |
| from_u64 | `From<u64>` | `From<u64>` | `new` | `from_u64`, `From<u64>` | `From<u64>` | `from_int` | `new` | `words64_to_element` | – |

- `ark-ff`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base.
- `k256/p256`: add, sub, mul, square, invert, sqrt, mul_u64, add_u64, sub_u64 and from_u64 over secp256k1 base, P-256 base and P-256 scalar; mul_uint, add_uint and sub_uint over P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31, BabyBear and Goldilocks.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base; mul_u64, add_u64, sub_u64 and from_u64 over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base, BLS12-381 base and P-521 base.
- `compile-algebra`: add, sub, mul, square and invert over P-256 base and P-256 scalar; sqrt over P-256 base.

## Functions, constant time

| Operation | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---|---|---|---|---|---|---|
| add | `ConstantTime +` | `+` | `+` | `+` | – | `+` | `addf` |
| sub | `ConstantTime -` | `-` | `-` | `-` | `-` | `-` | `subf` |
| mul | `ConstantTime *` | `*` | `*` | `*` | – | `*` | `mulf` |
| square | `ConstantTime::square` | `square` | `square` | `square` | – | `square` | `mulf(a, a)` |
| invert | `ConstantTime::invert` | `invert` | `invert` | `pow(p − 2)` | – | `inv` | `mulf to p − 2` |
| sqrt | `ConstantTime::sqrt` | `sqrt` | `sqrt` | `sqrt` | – | – | `mulf to (p + 1) / 4` |
| mul_u64 | `ConstantTime * ConstantTime<Uint<U64>>` | `* new` | `* from_u64`, `* From<u64>` | `* From<u64>` | – | `* new` | – |
| mul_uint | `ConstantTime * ConstantTime<Uint>` | `* new` | `* Reduce<U256>` | `* from_raw` | – | `* new` | – |
| add_u64 | `ConstantTime + ConstantTime<Uint<U64>>` | `+ new` | `+ from_u64`, `+ From<u64>` | `+ From<u64>` | – | `+ new` | – |
| add_uint | `ConstantTime + ConstantTime<Uint>` | `+ new` | `+ Reduce<U256>` | `+ from_raw` | – | `+ new` | – |
| sub_u64 | `ConstantTime - ConstantTime<Uint<U64>>` | `- new` | `- from_u64`, `- From<u64>` | `- From<u64>` | – | `- new` | – |
| sub_uint | `ConstantTime - ConstantTime<Uint>` | `- new` | `- Reduce<U256>` | `- from_raw` | – | `- new` | – |
| from_u64 | – | `new` | `from_u64`, `From<u64>` | `From<u64>` | – | `new` | – |

- `k256/p256`: add, sub, mul, square, invert, sqrt, mul_u64, add_u64, sub_u64 and from_u64 over secp256k1 base, P-256 base and P-256 scalar; mul_uint, add_uint and sub_uint over P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.9** / **2.9** | – | 12.0 / 18.9 | – | – | 3.0 / 3.0 | – | 3.2 / 5.0 | – |
| BabyBear | **2.1** / **3.0** | – | 13.0 / 19.0 | – | – | 2.7 / **3.0** | – | 3.4 / 5.0 | – |
| Goldilocks | **2.6** / **3.0** | – | 13.0 / 19.6 | – | – | 2.8 / 3.1 | 3.7 / 3.2 | 3.3 / 5.1 | – |
| BN254 scalar | **7.3** / **7.5** | 20.8 / 13.7 | 80.2 / 92.5 | – | 18.4 / 22.0 | – | – | 10.1 / 10.7 | – |
| secp256k1 base | **7.3** / **7.2** | 19.3 / 13.5 | 81.1 / 94.1 | 4.6\* / 4.2\* | 19.9 / 22.9 | – | – | 10.9 / 10.8 | – |
| P-256 base | **8.0** / **7.3** | 19.8 / 14.4 | 79.0 / 92.1 | 15.1 / 19.2 | 15.1 / 19.3 | – | – | 9.5 / 10.8 | 539 / – |
| P-256 scalar | **8.3** / **7.4** | 20.0 / 15.6 | 76.9 / 92.2 | 23.3 / 23.3 | 21.0 / 21.7 | – | – | 9.7 / 10.8 | 513 / – |
| Tom-256 base | **9.3** / **6.9** | – | 77.0 / 91.0 | – | 17.5 / 21.6 | – | – | **9.3** / 10.7 | – |
| BLS12-381 base | **13.7** / **13.0** | 21.6 / 16.2 | 110 / 124 | – | 27.7 / 27.9 | – | – | 19.4 / 13.2 | – |
| P-521 base | **17.9** / **23.5** | – | 148 / 162 | – | – | – | – | 34.8 / 54.9 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 2.7 / **3.0** | – | 5.4 / 13.0 | – | – | 2.8 / 3.0 | – | **2.5** / 4.0 | – |
| BabyBear | **2.6** / **3.0** | – | 5.4 / 12.7 | – | – | 2.7 / **3.0** | – | 2.7 / 4.0 | – |
| Goldilocks | 2.8 / **3.0** | – | 5.4 / 12.8 | – | – | 2.7\* / 2.9\* | **2.4** / **3.0** | 2.5 / 4.0 | – |
| BN254 scalar | **8.0** / **7.2** | 17.2 / 16.0 | 55.8 / 52.9 | – | 12.0 / 11.9 | – | – | 8.7 / 8.9 | – |
| secp256k1 base | **5.9** / **8.8** | 19.1 / 15.8 | 55.1 / 49.2 | 4.3\* / 4.9\* | 10.9 / 12.7 | – | – | 9.2 / **8.8** | – |
| P-256 base | **5.9** / **7.0** | 18.4 / 13.5 | 56.2 / 44.1 | 13.0 / 12.9 | 10.7 / 11.9 | – | – | 8.6 / 8.4 | 207 / – |
| P-256 scalar | **6.5** / **6.9** | 17.6 / 13.8 | 56.5 / 48.7 | 11.7 / 12.8 | 11.9 / 12.9 | – | – | 8.5 / 8.7 | 225 / – |
| Tom-256 base | **6.4** / **8.0** | – | 56.7 / 55.0 | – | 12.1 / 12.2 | – | – | 9.3 / 8.9 | – |
| BLS12-381 base | **12.7** / **10.9** | 24.9 / 19.4 | 90.7 / 104 | – | 21.9 / 21.5 | – | – | 23.9 / 12.2 | – |
| P-521 base | **13.9** / **19.0** | – | 141 / 154 | – | – | – | – | 37.1 / 54.7 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.6** / **7.0** | – | 21.2 / 35.2 | – | – | 4.2 / 7.1 | – | 9.9 / 17.2 | – |
| BabyBear | **3.8** / **11.0** | – | 21.5 / 35.4 | – | – | 5.8 / 13.1 | – | 9.8 / 17.2 | – |
| Goldilocks | **4.9** / **9.1** | – | 22.0 / 35.3 | – | – | 4.8\* / 10.0 | 5.5 / 11.0 | 9.9 / 17.3 | – |
| BN254 scalar | **80.9** / **79.3** | 99.9 / 113 | 154 / 182 | – | 110 / 111 | – | – | 144 / 154 | – |
| secp256k1 base | **57.6** / **58.5** | 106 / 121 | 156 / 184 | 116 / 115 | 111 / 112 | – | – | 150 / 153 | – |
| P-256 base | **63.1** / **62.8** | 86.0 / 92.3 | 155 / 182 | 85.7 / 84.5 | 86.6 / 84.7 | – | – | 67.8 / 82.6 | 934 / – |
| P-256 scalar | **90.5** / **88.0** | 107 / 122 | 155 / 183 | 169 / 168 | 116 / 117 | – | – | 148 / 155 | 957 / – |
| Tom-256 base | **84.6** / **82.7** | – | 155 / 185 | – | 110 / 109 | – | – | 149 / 155 | – |
| BLS12-381 base | **164** / **158** | 208 / 222 | 342 / 361 | – | 228 / 232 | – | – | 339 / 344 | – |
| P-521 base | **230** / **228** | – | 669 / 688 | – | – | – | – | 639 / 615 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.7** / **7.1** | – | 21.1 / 35.1 | – | – | 3.7 / 7.0\* | – | 10.7 / 17.3 | – |
| BabyBear | **3.9** / **11.0** | – | 20.1 / 35.0 | – | – | 5.6 / 13.1 | – | 9.9 / 17.2 | – |
| Goldilocks | **5.2** / **9.1** | – | 22.7 / 36.5 | – | – | 4.8\* / 10.1 | 5.3 / 11.1 | 9.9 / 17.2 | – |
| BN254 scalar | **72.6** / **72.0** | 91.8 / 107 | 143 / 170 | – | 90.3 / 96.0 | – | – | 147 / 155 | – |
| secp256k1 base | **40.7** / **45.6** | 102 / 109 | 146 / 170 | 80.0 / 101 | 90.4 / 95.2 | – | – | 147 / 157 | – |
| P-256 base | **52.6** / **50.3** | 79.8 / 86.4 | 144 / 169 | 78.4 / 84.4 | 65.0 / 66.6 | – | – | 66.9 / 78.1 | 886 / – |
| P-256 scalar | **75.7** / **76.4** | 103 / 116 | 142 / 170 | 166 / 174 | 101 / 102 | – | – | 147 / 153 | 965 / – |
| Tom-256 base | **69.7** / **69.4** | – | 144 / 168 | – | 95.5 / 92.4 | – | – | 150 / 155 | – |
| BLS12-381 base | **151** / **152** | 177 / 196 | 347 / 360 | – | 200 / 195 | – | – | 341 / 337 | – |
| P-521 base | **151** / **155** | – | 688 / 695 | – | – | – | – | 634 / 637 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **215** / – | – | 1,739 / – | – | – | 721 / – | – | 1,024 / – | – |
| BabyBear | **214** / – | – | 1,748 / – | – | – | 735 / – | – | 1,170 / – | – |
| Goldilocks | **440** / – | – | 3,328 / – | – | – | 1,494 / – | 733 / – | 1,981 / – | – |
| BN254 scalar | **5,700** / – | 21,222 / – | 17,031 / – | – | 8,675 / – | – | – | 17,629 / – | – |
| secp256k1 base | **5,682** / – | 19,861 / – | 16,988 / – | 26,949 / – | 8,490 / – | – | – | 17,895 / – | – |
| P-256 base | **5,562** / – | 20,103 / – | 17,264 / – | 29,918 / – | 8,455 / – | – | – | 18,867 / – | 152,343 / – |
| P-256 scalar | **5,939** / – | 21,653 / – | 17,471 / – | 40,337 / – | 8,552 / – | – | – | 19,151 / – | 147,528 / – |
| Tom-256 base | **5,881** / – | – | 17,420 / – | – | 8,715 / – | – | – | 18,952 / – | – |
| BLS12-381 base | **10,666** / – | 48,189 / – | 30,313 / – | – | 15,439 / – | – | – | 52,495 / – | – |
| P-521 base | **18,741** / – | – | 50,885 / – | – | – | – | – | 104,147 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **184** / – | – | 1,758 / – | – | – | 192 / – | 477 / – | – |
| BabyBear | **2,217** / – | – | 18,722 / – | – | – | 3,156 / – | – | – |
| Goldilocks | **3,095** / – | – | 26,834 / – | – | – | 3,500 / – | – | – |
| BN254 scalar | **38,892** / – | 52,644 / – | 146,345 / – | – | 101,992 / – | – | – | – |
| secp256k1 base | **12,536** / – | 52,180 / – | 60,808 / – | 26,498 / – | 50,770 / – | – | 81,398 / – | – |
| P-256 base | **13,240** / – | 24,353 / – | 50,742 / – | 29,162 / – | 39,183 / – | – | 32,067 / – | 248,661 / – |
| P-256 scalar | **26,050** / – | 45,862 / – | 59,771 / – | 87,568 / – | 46,335 / – | – | – | – |
| Tom-256 base | **21,772** / – | – | 56,617 / – | – | 53,202 / – | – | 59,216 / – | – |
| BLS12-381 base | **73,310** / – | 108,307 / – | 162,994 / – | – | 177,081 / – | – | 220,218 / – | – |
| P-521 base | **77,680** / – | – | 334,035 / – | – | – | – | 343,983 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **10.0** / **17.2** | – | 37.3 / 40.8 | – | – | 5.5\* / 7.0\* | – | – |
| BabyBear | **10.6** / 22.0 | – | 43.6 / 43.0 | – | – | 12.4 / **14.9** | – | – |
| Goldilocks | **4.5** / 13.0 | – | 47.9 / 47.8 | – | – | 3.3\* / 9.9\* | 9.3 / **11.0** | – |
| BN254 scalar | **20.9** / **25.9** | 241 / 252 | 335 / 334 | – | 286 / 287 | – | – | 302 / 299 |
| secp256k1 base | **11.2** / **14.9** | 240 / 240 | 339 / 333 | 97.6 / 110 | 265 / 274 | – | – | 295 / 295 |
| P-256 base | **22.2** / **25.9** | 212 / 208 | 337 / 336 | 167 / 166 | 239 / 247 | – | – | 160 / 159 |
| P-256 scalar | **23.7** / **27.9** | 262 / 271 | 332 / 328 | 166 / 169 | 319 / 312 | – | – | 300 / 299 |
| Tom-256 base | **24.4** / **28.3** | – | 329 / 332 | – | 296 / 300 | – | – | 293 / 297 |
| BLS12-381 base | **33.3** / **34.5** | 516 / 465 | 676 / 679 | – | 523 / 545 | – | – | 654 / 684 |
| P-521 base | **26.6** / **34.0** | – | 1,142 / 1,145 | – | – | – | – | 1,148 / 1,162 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **10.1** / **17.2** | 44.1 / 43.8 | – | – | 7.7\* / 7.4\* | – |
| BabyBear | **13.7** / 22.2 | 43.6 / 43.3 | – | – | 13.9 / **15.0** | – |
| Goldilocks | **6.2** / 13.0 | 49.5 / 46.7 | – | – | 4.5\* / 9.9\* | 9.3 / **11.0** |
| BN254 scalar | **129** / **132** | 331 / 339 | – | 214 / 216 | – | – |
| secp256k1 base | **78.7** / **85.7** | 336 / 334 | – | 190 / 188 | – | – |
| P-256 base | **124** / **125** | 338 / 336 | – | 164 / 160 | – | – |
| P-256 scalar | **137** / **141** | 340 / 339 | 206 / 207 | 219 / 215 | – | – |
| Tom-256 base | **131** / **131** | 339 / 338 | – | 201 / 203 | – | – |
| BLS12-381 base | **270** / **266** | 670 / 660 | – | 400 / 416 | – | – |
| P-521 base | **425** / **430** | 1,122 / 1,120 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.8** / **8.3** | – | 38.6 / 36.9 | – | – | 6.0\* / 5.4\* | – | – |
| BabyBear | 13.9 / 14.3 | – | 41.5 / 36.1 | – | – | **9.5** / **8.6** | – | – |
| Goldilocks | **6.9** / **6.2** | – | 45.0 / 41.2 | – | – | 2.3\* / 3.0\* | 8.3 / 7.1 | – |
| BN254 scalar | **33.4** / **29.1** | 134 / 135 | 259 / 263 | – | 198 / 205 | – | – | 169 / 167 |
| secp256k1 base | **5.9** / **7.8** | 108 / 107 | 260 / 247 | 4.1\* / 2.6\* | 186 / 177 | – | – | 174 / 163 |
| P-256 base | **29.7** / **28.4** | 117 / 113 | 261 / 266 | 96.7 / 97.5 | 179 / 170 | – | – | 78.8 / 82.5 |
| P-256 scalar | 32.6 / 29.1 | 136 / 134 | 257 / 240 | **17.0** / **21.4** | 200 / 193 | – | – | 165 / 169 |
| Tom-256 base | **31.7** / **28.4** | – | 261 / 262 | – | 198 / 196 | – | – | 167 / 161 |
| BLS12-381 base | **50.9** / **47.2** | 257 / 251 | 453 / 459 | – | 323 / 316 | – | – | 391 / 372 |
| P-521 base | **15.9** / **22.1** | – | 724 / 737 | – | – | – | – | 670 / 675 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.0** / **7.4** | 38.7 / 35.4 | – | – | 6.2\* / 4.8\* | – |
| BabyBear | 13.9 / 12.3 | 39.3 / 37.3 | – | – | **9.4** / **8.3** | – |
| Goldilocks | **5.3** / **4.8** | 39.3 / 35.3 | – | – | 2.0\* / 3.0\* | 6.1 / 6.9 |
| BN254 scalar | 127 / **123** | 240 / 252 | – | **118** / 126 | – | – |
| secp256k1 base | **83.0** / **82.5** | 260 / 259 | – | 97.7 / 102 | – | – |
| P-256 base | 129 / 123 | 265 / 263 | – | **89.6** / **94.7** | – | – |
| P-256 scalar | 129 / 141 | 259 / 263 | **53.2** / **53.0** | 126 / 131 | – | – |
| Tom-256 base | 136 / 136 | 259 / 258 | – | **110** / **111** | – | – |
| BLS12-381 base | 270 / 260 | 378 / 441 | – | **231** / **226** | – | – |
| P-521 base | **434** / **418** | 716 / 730 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.5** / **7.1** | – | 20.4 / 19.8 | – | – | 5.1\* / 5.3\* | – | – |
| BabyBear | 12.7 / 12.5 | – | 21.0 / 20.6 | – | – | **9.2** / **9.1** | – | – |
| Goldilocks | **6.0** / **5.4** | – | 27.6 / 29.5 | – | – | 2.0\* / 3.0\* | 6.4 / 5.7 | – |
| BN254 scalar | **30.0** / **25.3** | 133 / 130 | 215 / 191 | – | 148 / 155 | – | – | 167 / 112 |
| secp256k1 base | **5.0** / **8.5** | 108 / 105 | 196 / 185 | 4.8\* / 3.0\* | 160 / 162 | – | – | 160 / 161 |
| P-256 base | **25.7** / **24.4** | 109 / 98.8 | 219 / 210 | 91.5 / 94.2 | 162 / 160 | – | – | 74.2 / 69.4 |
| P-256 scalar | 28.8 / 24.0 | 135 / 136 | 214 / 215 | **9.9** / **12.5** | 189 / 190 | – | – | 169 / 159 |
| Tom-256 base | **30.0** / **28.3** | – | 209 / 212 | – | 190 / 180 | – | – | 125 / 149 |
| BLS12-381 base | **44.8** / **45.6** | 253 / 228 | 419 / 435 | – | 291 / 298 | – | – | 327 / 329 |
| P-521 base | **12.6** / **11.2** | – | 697 / 711 | – | – | – | – | 589 / 550 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.5** / **7.0** | 20.9 / 21.0 | – | – | 5.5\* / 4.4\* | – |
| BabyBear | 11.5 / 12.5 | 21.8 / 21.1 | – | – | **9.0** / **9.0** | – |
| Goldilocks | **4.7** / **4.5** | 31.2 / 23.1 | – | – | 2.1\* / 3.0\* | 6.6 / 6.0 |
| BN254 scalar | 122 / 122 | 186 / 190 | – | **116** / **105** | – | – |
| secp256k1 base | **68.6** / **80.1** | 212 / 211 | – | 94.3 / 92.4 | – | – |
| P-256 base | 118 / 125 | 212 / 211 | – | **90.4** / **88.0** | – | – |
| P-256 scalar | 136 / 128 | 222 / 219 | **48.0** / **48.8** | 120 / 118 | – | – |
| Tom-256 base | 138 / 129 | 219 / 220 | – | **111** / **112** | – | – |
| BLS12-381 base | 263 / 265 | 375 / 415 | – | **212** / **213** | – | – |
| P-521 base | **420** / **424** | 700 / 701 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 6.3 / – | – | 15.0 / – | – | – | **4.7** / – | – | – |
| BabyBear | 12.3 / – | – | 16.2 / – | – | – | **8.2** / – | – | – |
| Goldilocks | 4.9 / – | – | 21.2 / – | – | – | 1.9\* / – | **4.2** / – | – |
| BN254 scalar | **26.1** / – | 146 / – | 157 / – | – | 168 / – | – | – | 149 / – |
| secp256k1 base | **2.6** / – | 130 / – | 129 / – | 3.1 / – | 147 / – | – | – | 146 / – |
| P-256 base | **21.7** / – | 125 / – | 162 / – | 80.5 / – | 144 / – | – | – | 81.3 / – |
| P-256 scalar | 22.6 / – | 150 / – | 159 / – | **2.4** / – | 160 / – | – | – | 147 / – |
| Tom-256 base | **21.0** / – | – | 159 / – | – | 168 / – | – | – | 105 / – |
| BLS12-381 base | **29.1** / – | 261 / – | 324 / – | – | 293 / – | – | – | 357 / – |
| P-521 base | **5.0** / – | – | 569 / – | – | – | – | – | 611 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.1** / **3.0** | 12.0 / 18.9 | – | – | – | 3.2 / 5.0 |
| BabyBear | **2.6** / **3.2** | 13.0 / 19.0 | – | – | – | 3.4 / 5.0 |
| Goldilocks | **2.9** / **3.1** | 13.0 / 19.6 | – | – | 3.7 / 3.2 | 3.3 / 5.1 |
| BN254 scalar | **7.7** / **7.5** | 80.2 / 92.5 | – | 18.4 / 22.0 | – | 10.1 / 10.7 |
| secp256k1 base | **8.0** / **7.5** | 81.1 / 94.1 | 4.6\* / 4.2\* | 19.9 / 22.9 | – | 10.9 / 10.8 |
| P-256 base | **8.3** / **7.0** | 79.0 / 92.1 | 15.1 / 19.2 | 15.1 / 19.3 | – | 9.5 / 10.8 |
| P-256 scalar | **8.6** / **7.7** | 76.9 / 92.2 | 23.3 / 23.3 | 21.0 / 21.7 | – | 9.7 / 10.8 |
| Tom-256 base | **7.4** / **7.8** | 77.0 / 91.0 | – | 17.5 / 21.6 | – | 9.3 / 10.7 |
| BLS12-381 base | **11.9** / **10.5** | 110 / 124 | – | 27.7 / 27.9 | – | 19.4 / 13.2 |
| P-521 base | **15.5** / **23.4** | 148 / 162 | – | – | – | 34.8 / 54.9 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 2.8 / **2.9** | 5.4 / 13.0 | – | – | 2.8 / 3.0 | – | **2.5** / 4.0 |
| BabyBear | 2.8 / **2.9** | 5.4 / 12.7 | – | – | – | – | **2.7** / 4.0 |
| Goldilocks | **2.2** / **3.0** | 5.4 / 12.8 | – | – | – | 2.4 / **3.0** | 2.5 / 4.0 |
| BN254 scalar | **8.2** / **6.9** | 55.8 / 52.9 | – | 12.0 / 11.9 | – | – | 8.7 / 8.9 |
| secp256k1 base | **6.4** / **8.8** | 55.1 / 49.2 | 4.3\* / 4.9\* | 10.9 / 12.7 | – | – | 9.2 / **8.8** |
| P-256 base | **7.0** / **7.3** | 56.2 / 44.1 | 13.0 / 12.9 | 10.7 / 11.9 | – | – | 8.6 / 8.4 |
| P-256 scalar | **7.1** / **7.0** | 56.5 / 48.7 | 11.7 / 12.8 | 11.9 / 12.9 | – | – | 8.5 / 8.7 |
| Tom-256 base | **6.8** / **8.0** | 56.7 / 55.0 | – | 12.1 / 12.2 | – | – | 9.3 / 8.9 |
| BLS12-381 base | **13.7** / **11.5** | 90.7 / 104 | – | 21.9 / 21.5 | – | – | 23.9 / 12.2 |
| P-521 base | **15.4** / **19.9** | 141 / 154 | – | – | – | – | 37.1 / 54.7 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.6** / **7.0** | 21.2 / 35.2 | – | – | – | 9.9 / 17.2 |
| BabyBear | **3.9** / **11.0** | 21.5 / 35.4 | – | – | – | 9.8 / 17.2 |
| Goldilocks | **5.1** / **9.0** | 22.0 / 35.3 | – | – | 5.5 / 11.0 | 9.9 / 17.3 |
| BN254 scalar | **79.8** / **85.1** | 154 / 182 | – | 110 / 111 | – | 144 / 154 |
| secp256k1 base | **57.8** / **58.5** | 156 / 184 | 116 / 115 | 111 / 112 | – | 150 / 153 |
| P-256 base | **61.0** / **62.3** | 155 / 182 | 85.7 / 84.5 | 86.6 / 84.7 | – | 67.8 / 82.6 |
| P-256 scalar | **88.3** / **88.2** | 155 / 183 | 169 / 168 | 116 / 117 | – | 148 / 155 |
| Tom-256 base | **84.7** / **82.6** | 155 / 185 | – | 110 / 109 | – | 149 / 155 |
| BLS12-381 base | **172** / **173** | 342 / 361 | – | 228 / 232 | – | 339 / 344 |
| P-521 base | **229** / **225** | 669 / 688 | – | – | – | 639 / 615 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.6** / **7.0** | 21.1 / 35.1 | – | – | – | 10.7 / 17.3 |
| BabyBear | **4.0** / **11.0** | 20.1 / 35.0 | – | – | – | 9.9 / 17.2 |
| Goldilocks | **5.0** / **9.1** | 22.7 / 36.5 | – | – | 5.3 / 11.1 | 9.9 / 17.2 |
| BN254 scalar | **74.7** / **77.1** | 143 / 170 | – | 90.3 / 96.0 | – | 147 / 155 |
| secp256k1 base | **41.6** / **45.7** | 146 / 170 | 80.0 / 101 | 90.4 / 95.2 | – | 147 / 157 |
| P-256 base | **53.2** / **50.5** | 144 / 169 | 78.4 / 84.4 | 65.0 / 66.6 | – | 66.9 / 78.1 |
| P-256 scalar | **74.6** / **75.7** | 142 / 170 | 166 / 174 | 101 / 102 | – | 147 / 153 |
| Tom-256 base | **70.6** / **69.8** | 144 / 168 | – | 95.5 / 92.4 | – | 150 / 155 |
| BLS12-381 base | **158** / **160** | 347 / 360 | – | 200 / 195 | – | 341 / 337 |
| P-521 base | **156** / **157** | 688 / 695 | – | – | – | 634 / 637 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **255** / – | 5,720 / – | – | – | – | 999 / – |
| BabyBear | **424** / – | 5,001 / – | – | – | – | 1,001 / – |
| Goldilocks | **669** / – | 5,498 / – | – | – | 733 / – | 2,133 / – |
| BN254 scalar | **8,239** / – | 26,433 / – | – | 53,030 / – | – | 63,317 / – |
| secp256k1 base | **8,271** / – | 25,737 / – | 26,949 / – | 50,876 / – | – | 81,335 / – |
| P-256 base | **7,551** / – | 24,805 / – | 29,918 / – | 37,979 / – | – | 37,322 / – |
| P-256 scalar | **7,836** / – | 25,861 / – | 82,683 / – | 54,314 / – | – | 68,712 / – |
| Tom-256 base | **8,504** / – | 27,361 / – | – | 50,515 / – | – | 60,003 / – |
| BLS12-381 base | **15,457** / – | 45,321 / – | – | 169,002 / – | – | 219,622 / – |
| P-521 base | **25,661** / – | 78,121 / – | – | – | – | 671,553 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **189** / – | 1,758 / – | – | – | 477 / – |
| BabyBear | **5,634** / – | 18,722 / – | – | – | – |
| Goldilocks | **7,635** / – | 26,834 / – | – | – | – |
| BN254 scalar | **65,816** / – | 146,345 / – | – | 101,992 / – | – |
| secp256k1 base | **12,709** / – | 60,808 / – | 26,498 / – | 50,770 / – | 81,398 / – |
| P-256 base | **13,400** / – | 50,742 / – | 29,162 / – | 39,183 / – | 32,067 / – |
| P-256 scalar | **27,382** / – | 59,771 / – | 87,568 / – | 46,335 / – | – |
| Tom-256 base | **22,811** / – | 56,617 / – | – | 53,202 / – | 59,216 / – |
| BLS12-381 base | **82,913** / – | 162,994 / – | – | 177,081 / – | 220,218 / – |
| P-521 base | **77,179** / – | 334,035 / – | – | – | 343,983 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.8** / **17.2** | 37.3 / 40.8 | – | – | – |
| BabyBear | **12.8** / **22.1** | 43.6 / 43.0 | – | – | – |
| Goldilocks | **6.1** / 13.0 | 47.9 / 47.8 | – | – | 9.3 / **11.0** |
| BN254 scalar | **27.8** / **30.9** | 335 / 334 | – | 286 / 287 | – |
| secp256k1 base | **13.2** / **20.3** | 339 / 333 | 97.6 / 110 | 265 / 274 | – |
| P-256 base | **26.3** / **33.3** | 337 / 336 | 167 / 166 | 239 / 247 | – |
| P-256 scalar | **26.4** / **34.4** | 332 / 328 | 166 / 169 | 319 / 312 | – |
| Tom-256 base | **25.9** / **34.1** | 329 / 332 | – | 296 / 300 | – |
| BLS12-381 base | **42.1** / **42.3** | 676 / 679 | – | 523 / 545 | – |
| P-521 base | **34.0** / **45.8** | 1,142 / 1,145 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **11.2** / **17.2** | 44.1 / 43.8 | – | – | – |
| BabyBear | **14.0** / **22.2** | 43.6 / 43.3 | – | – | – |
| Goldilocks | **5.8** / 13.0 | 49.5 / 46.7 | – | – | 9.3 / **11.0** |
| BN254 scalar | **142** / **145** | 331 / 339 | – | 214 / 216 | – |
| secp256k1 base | **88.8** / **96.1** | 336 / 334 | – | 190 / 188 | – |
| P-256 base | **139** / **147** | 338 / 336 | – | 164 / 160 | – |
| P-256 scalar | **148** / **160** | 340 / 339 | 206 / 207 | 219 / 215 | – |
| Tom-256 base | **140** / **146** | 339 / 338 | – | 201 / 203 | – |
| BLS12-381 base | **306** / **296** | 670 / 660 | – | 400 / 416 | – |
| P-521 base | **490** / **485** | 1,122 / 1,120 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **7.5** | 38.6 / 36.9 | – | – | – |
| BabyBear | **13.4** / **14.2** | 41.5 / 36.1 | – | – | – |
| Goldilocks | **6.5** / **5.7** | 45.0 / 41.2 | – | – | 8.3 / 7.1 |
| BN254 scalar | **31.7** / **35.4** | 259 / 263 | – | 198 / 205 | – |
| secp256k1 base | **8.7** / **7.4** | 260 / 247 | 4.1\* / 2.6\* | 186 / 177 | – |
| P-256 base | **32.7** / **30.8** | 261 / 266 | 96.7 / 97.5 | 179 / 170 | – |
| P-256 scalar | 32.4 / 34.4 | 257 / 240 | **17.0** / **21.4** | 200 / 193 | – |
| Tom-256 base | **33.8** / **31.8** | 261 / 262 | – | 198 / 196 | – |
| BLS12-381 base | **53.3** / **57.2** | 453 / 459 | – | 323 / 316 | – |
| P-521 base | **15.8** / **22.6** | 724 / 737 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **7.7** | 38.7 / 35.4 | – | – | – |
| BabyBear | **13.8** / **14.3** | 39.3 / 37.3 | – | – | – |
| Goldilocks | 6.4 / **6.0** | 39.3 / 35.3 | – | – | **6.1** / 6.9 |
| BN254 scalar | 138 / 143 | 240 / 252 | – | **118** / **126** | – |
| secp256k1 base | 98.1 / **96.7** | 260 / 259 | – | **97.7** / 102 | – |
| P-256 base | 153 / 151 | 265 / 263 | – | **89.6** / **94.7** | – |
| P-256 scalar | 163 / 160 | 259 / 263 | **53.2** / **53.0** | 126 / 131 | – |
| Tom-256 base | 151 / 149 | 259 / 258 | – | **110** / **111** | – |
| BLS12-381 base | 297 / 296 | 378 / 441 | – | **231** / **226** | – |
| P-521 base | **483** / **490** | 716 / 730 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.5** / **7.5** | 20.4 / 19.8 | – | – | – |
| BabyBear | **13.2** / **12.4** | 21.0 / 20.6 | – | – | – |
| Goldilocks | **5.8** / **5.7** | 27.6 / 29.5 | – | – | 6.4 / **5.7** |
| BN254 scalar | **32.2** / **30.2** | 215 / 191 | – | 148 / 155 | – |
| secp256k1 base | **4.2** / **7.1** | 196 / 185 | 4.8 / 3.0\* | 160 / 162 | – |
| P-256 base | **29.0** / **29.6** | 219 / 210 | 91.5 / 94.2 | 162 / 160 | – |
| P-256 scalar | 26.8 / 29.1 | 214 / 215 | **9.9** / **12.5** | 189 / 190 | – |
| Tom-256 base | **31.3** / **28.9** | 209 / 212 | – | 190 / 180 | – |
| BLS12-381 base | **49.5** / **53.7** | 419 / 435 | – | 291 / 298 | – |
| P-521 base | **13.3** / **18.4** | 697 / 711 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.6** / **7.9** | 20.9 / 21.0 | – | – | – |
| BabyBear | **12.9** / **13.9** | 21.8 / 21.1 | – | – | – |
| Goldilocks | **4.8** / **5.9** | 31.2 / 23.1 | – | – | 6.6 / 6.0 |
| BN254 scalar | 139 / 140 | 186 / 190 | – | **116** / **105** | – |
| secp256k1 base | 94.9 / 95.7 | 212 / 211 | – | **94.3** / **92.4** | – |
| P-256 base | 147 / 145 | 212 / 211 | – | **90.4** / **88.0** | – |
| P-256 scalar | 162 / 161 | 222 / 219 | **48.0** / **48.8** | 120 / 118 | – |
| Tom-256 base | 153 / 149 | 219 / 220 | – | **111** / **112** | – |
| BLS12-381 base | 305 / 282 | 375 / 415 | – | **212** / **213** | – |
| P-521 base | **465** / **474** | 700 / 701 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **15.0** / – | – | – | – |
| BabyBear | **16.2** / – | – | – | – |
| Goldilocks | 21.2 / – | – | – | **4.2** / – |
| BN254 scalar | **157** / – | – | 168 / – | – |
| secp256k1 base | 129 / – | **3.1** / – | 147 / – | – |
| P-256 base | 162 / – | **80.5** / – | 144 / – | – |
| P-256 scalar | 159 / – | **2.4** / – | 160 / – | – |
| Tom-256 base | **159** / – | – | 168 / – | – |
| BLS12-381 base | 324 / – | – | **293** / – | – |
| P-521 base | **569** / – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
