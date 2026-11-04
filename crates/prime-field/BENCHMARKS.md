# Benchmarks

`prime-field` at commit `a159b9c` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 12; 1-minute load 11.58 before, 20.06 after; its SMT sibling, CPU 28, 49% busy |
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
| Mersenne-31 | **1.9** / **2.9** | – | 9.7 / 18.1 | – | – | 2.0 / 3.0 | – | 2.6 / 5.0 | – |
| BabyBear | **1.9** / **2.9** | – | 9.7 / 18.4 | – | – | 2.0 / 3.0 | – | 2.6 / 5.0 | – |
| Goldilocks | **2.0** / 3.0 | – | 9.8 / 19.4 | – | – | 1.9\* / 2.9\* | 2.4 / **2.9** | 2.6 / 5.0 | – |
| BN254 scalar | **5.1** / **7.0** | 17.3 / 12.7 | 66.6 / 93.7 | – | 12.8 / 21.8 | – | – | 7.0 / 11.0 | – |
| secp256k1 base | **6.0** / **7.0** | 17.5 / 13.0 | 66.7 / 92.8 | 3.3\* / 3.3\* | 14.4 / 21.2 | – | – | 7.1 / 10.8 | – |
| P-256 base | **6.0** / **7.0** | 18.5 / 13.7 | 66.6 / 93.1 | 10.7 / 18.7 | 10.7 / 20.8 | – | – | 6.9 / 10.8 | 341 / – |
| P-256 scalar | **5.7** / **6.8** | 17.9 / 13.6 | 66.6 / 92.9 | 17.5 / 22.8 | 14.4 / 20.9 | – | – | 6.9 / 10.8 | 345 / – |
| Tom-256 base | **5.9** / **6.9** | – | 66.6 / 94.1 | – | 11.7 / 21.6 | – | – | 7.0 / 10.8 | – |
| BLS12-381 base | **10.0** / **9.0** | 20.3 / 14.5 | 87.8 / 113 | – | 20.5 / 27.4 | – | – | 16.9 / 12.7 | – |
| P-521 base | **12.3** / **15.2** | – | 117 / 155 | – | – | – | – | 25.1 / 52.6 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | – | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| BabyBear | **1.9** / **2.9** | – | 4.2 / 11.9 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 | – |
| Goldilocks | **1.9** / 3.0 | – | 4.2 / 12.2 | – | – | 2.0 / 2.9\* | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.1** / **7.0** | 16.3 / 13.3 | 49.7 / 51.0 | – | 8.4 / 11.8 | – | – | 7.0 / 8.0 | – |
| secp256k1 base | **5.5** / 9.8 | 17.7 / 13.8 | 49.8 / 48.3 | 4.2\* / 3.9\* | 7.6 / 12.8 | – | – | 6.9 / **7.8** | – |
| P-256 base | **5.5** / **7.0** | 16.6 / 12.4 | 50.0 / 43.5 | 8.0 / 12.9 | 7.7 / 12.0 | – | – | 7.0 / 8.0 | 150 / – |
| P-256 scalar | **5.0** / **6.8** | 15.9 / 13.3 | 49.8 / 48.8 | 8.3 / 12.8 | 8.1 / 12.8 | – | – | 7.0 / 7.8 | 152 / – |
| Tom-256 base | **5.6** / 8.1 | – | 49.8 / 55.4 | – | 8.4 / 11.8 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **10.0** / **10.0** | 22.1 / 16.2 | 83.3 / 88.3 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 | – |
| P-521 base | **12.1** / **12.6** | – | 122 / 139 | – | – | – | – | 27.7 / 50.9 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / **6.9** | – | 16.9 / 37.7 | – | – | 3.2\* / 7.0 | – | 8.0 / 17.1 | – |
| BabyBear | **3.6** / **11.0** | – | 16.9 / 36.6 | – | – | 5.4 / 13.0 | – | 8.0 / 17.0 | – |
| Goldilocks | **4.8** / **9.1** | – | 17.8 / 34.9 | – | – | 3.9\* / 10.0 | 4.9 / 11.0 | 9.3 / 17.1 | – |
| BN254 scalar | **75.8** / **73.0** | 92.8 / 110 | 151 / 178 | – | 98.1 / 98.2 | – | – | 137 / 144 | – |
| secp256k1 base | **52.7** / **55.4** | 93.8 / 115 | 154 / 172 | 110 / 106 | 104 / 102 | – | – | 143 / 153 | – |
| P-256 base | **52.4** / **60.6** | 76.9 / 89.1 | 135 / 179 | 83.8 / 80.6 | 84.8 / 82.1 | – | – | 65.9 / 78.4 | 801 / – |
| P-256 scalar | **82.8** / **88.1** | 105 / 119 | 149 / 173 | 156 / 142 | 94.3 / 100 | – | – | 117 / 139 | 884 / – |
| Tom-256 base | **79.8** / **76.9** | – | 127 / 163 | – | 92.4 / 95.8 | – | – | 94.8 / 147 | – |
| BLS12-381 base | **157** / **157** | 166 / 199 | 277 / 316 | – | 203 / 196 | – | – | 297 / 308 | – |
| P-521 base | **213** / **192** | – | 651 / 645 | – | – | – | – | 602 / 614 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.2** / **7.0** | – | 19.8 / 34.8 | – | – | 3.2\* / 7.0 | – | 7.5 / 17.2 | – |
| BabyBear | **3.7** / **11.0** | – | 20.4 / 34.9 | – | – | 5.7 / 13.1 | – | 9.1 / 17.4 | – |
| Goldilocks | 4.9 / **9.0** | – | 18.5 / 36.5 | – | – | 4.6\* / 10.0 | **4.5** / 11.0 | 9.3 / 17.2 | – |
| BN254 scalar | **70.8** / **71.5** | 86.2 / 106 | 137 / 165 | – | 83.7 / 92.6 | – | – | 142 / 142 | – |
| secp256k1 base | **36.0** / **45.5** | 84.8 / 111 | 137 / 157 | 58.8 / 90.4 | 85.2 / 91.4 | – | – | 148 / 150 | – |
| P-256 base | **48.0** / **47.6** | 65.7 / 84.5 | 137 / 165 | 70.6 / 72.3 | 55.2 / 61.0 | – | – | 58.9 / 78.2 | 734 / – |
| P-256 scalar | **75.0** / **74.5** | 99.5 / 114 | 120 / 156 | 152 / 152 | 87.9 / 96.3 | – | – | 142 / 146 | 889 / – |
| Tom-256 base | **67.3** / **67.3** | – | 122 / 154 | – | 86.6 / 91.1 | – | – | 93.3 / 139 | – |
| BLS12-381 base | **136** / **138** | 143 / 169 | 311 / 320 | – | 178 / 180 | – | – | 309 / 295 | – |
| P-521 base | **143** / **151** | – | 664 / 583 | – | – | – | – | 575 / 542 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **215** / – | – | 1,718 / – | – | – | 705 / – | – | 951 / – | – |
| BabyBear | **217** / – | – | 1,504 / – | – | – | 659 / – | – | 1,108 / – | – |
| Goldilocks | **438** / – | – | 3,247 / – | – | – | 1,379 / – | 728 / – | 1,813 / – | – |
| BN254 scalar | **5,106** / – | 16,733 / – | 15,230 / – | – | 7,436 / – | – | – | 15,024 / – | – |
| secp256k1 base | **5,153** / – | 16,738 / – | 14,724 / – | 27,205 / – | 7,720 / – | – | – | 14,678 / – | – |
| P-256 base | **5,059** / – | 15,583 / – | 14,916 / – | 30,345 / – | 7,528 / – | – | – | 15,451 / – | 98,043 / – |
| P-256 scalar | **5,321** / – | 17,183 / – | 14,762 / – | 32,445 / – | 7,348 / – | – | – | 14,794 / – | 93,731 / – |
| Tom-256 base | **5,130** / – | – | 16,167 / – | – | 7,458 / – | – | – | 14,952 / – | – |
| BLS12-381 base | **9,725** / – | 41,940 / – | 28,566 / – | – | 13,922 / – | – | – | 40,046 / – | – |
| P-521 base | **16,054** / – | – | 44,869 / – | – | – | – | – | 87,200 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 177 / – | – | 1,747 / – | – | – | **148** / – | 480 / – | – |
| BabyBear | **2,033** / – | – | 18,652 / – | – | – | 2,874 / – | – | – |
| Goldilocks | **3,021** / – | – | 26,941 / – | – | – | 3,405 / – | – | – |
| BN254 scalar | **36,521** / – | 49,250 / – | 144,019 / – | – | 94,413 / – | – | – | – |
| secp256k1 base | **12,441** / – | 48,759 / – | 59,372 / – | 26,875 / – | 49,507 / – | – | 75,192 / – | – |
| P-256 base | **12,754** / – | 20,583 / – | 44,233 / – | 30,299 / – | 35,744 / – | – | 29,813 / – | 238,276 / – |
| P-256 scalar | **25,780** / – | 42,830 / – | 52,743 / – | 84,389 / – | 44,471 / – | – | – | – |
| Tom-256 base | **21,266** / – | – | 52,460 / – | – | 47,823 / – | – | 56,093 / – | – |
| BLS12-381 base | **72,934** / – | 107,470 / – | 153,349 / – | – | 156,150 / – | – | 196,603 / – | – |
| P-521 base | **75,351** / – | – | 308,977 / – | – | – | – | 324,413 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.8** / **9.0** | – | 34.7 / 38.9 | – | – | 5.5\* / 7.0\* | – | – |
| BabyBear | **9.0** / **11.0** | – | 35.8 / 36.9 | – | – | 12.4 / 14.9 | – | – |
| Goldilocks | **7.3** / **10.0** | – | 38.3 / 39.3 | – | – | 3.2\* / 9.9\* | **7.3** / 10.9 | – |
| BN254 scalar | **20.9** / **25.9** | 236 / 237 | 288 / 277 | – | 240 / 236 | – | – | 263 / 269 |
| secp256k1 base | **10.8** / **14.9** | 233 / 234 | 289 / 275 | 82.0 / 91.4 | 214 / 222 | – | – | 259 / 259 |
| P-256 base | **19.6** / **24.9** | 207 / 208 | 317 / 278 | 147 / 147 | 187 / 185 | – | – | 135 / 141 |
| P-256 scalar | **22.4** / **27.9** | 262 / 262 | 281 / 277 | 148 / 146 | 258 / 251 | – | – | 298 / 275 |
| Tom-256 base | **23.0** / **28.1** | – | 284 / 280 | – | 230 / 245 | – | – | 279 / 257 |
| BLS12-381 base | **28.9** / **34.5** | 428 / 425 | 566 / 565 | – | 440 / 441 | – | – | 588 / 616 |
| P-521 base | **19.9** / **33.2** | – | 927 / 931 | – | – | – | – | 980 / 1,004 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **9.0** | 34.7 / 36.5 | – | – | 5.6\* / 7.0\* | – |
| BabyBear | **10.2** / **11.0** | 34.5 / 36.8 | – | – | 12.4 / 14.9 | – |
| Goldilocks | **7.1** / **10.0** | 41.2 / 39.5 | – | – | 3.2\* / 9.9\* | 7.3 / 10.9 |
| BN254 scalar | **105** / **114** | 287 / 287 | – | 191 / 200 | – | – |
| secp256k1 base | **57.2** / **68.5** | 284 / 291 | – | 159 / 156 | – | – |
| P-256 base | **104** / **111** | 292 / 286 | – | 128 / 134 | – | – |
| P-256 scalar | **111** / **125** | 307 / 293 | 198 / 195 | 191 / 188 | – | – |
| Tom-256 base | **108** / **115** | 290 / 285 | – | 170 / 174 | – | – |
| BLS12-381 base | **215** / **222** | 553 / 559 | – | 389 / 382 | – | – |
| P-521 base | **366** / **409** | 917 / 954 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **5.5** | – | 33.1 / 32.0 | – | – | 4.1\* / 4.1\* | – | – |
| BabyBear | **6.0** / **7.0** | – | 32.5 / 30.7 | – | – | 9.0 / 7.9 | – | – |
| Goldilocks | **5.2** / **4.8** | – | 39.0 / 35.5 | – | – | 1.9\* / 2.9\* | 6.2 / 6.1 | – |
| BN254 scalar | **24.9** / **24.9** | 128 / 127 | 235 / 238 | – | 145 / 158 | – | – | 115 / 136 |
| secp256k1 base | **6.2** / **7.4** | 107 / 105 | 259 / 238 | 3.5\* / 2.1\* | 139 / 138 | – | – | 115 / 112 |
| P-256 base | **25.8** / **22.3** | 114 / 113 | 245 / 237 | 76.4 / 86.1 | 120 / 131 | – | – | 67.1 / 71.2 |
| P-256 scalar | 25.4 / 22.9 | 138 / 135 | 240 / 235 | **14.4** / **20.7** | 161 / 170 | – | – | 115 / 113 |
| Tom-256 base | **26.7** / **26.4** | – | 255 / 236 | – | 164 / 156 | – | – | 115 / 113 |
| BLS12-381 base | **46.0** / **41.0** | 227 / 220 | 442 / 386 | – | 282 / 267 | – | – | 349 / 352 |
| P-521 base | **14.6** / **12.8** | – | 599 / 613 | – | – | – | – | 610 / 581 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **5.5** | 33.4 / 26.7 | – | – | 4.1\* / 4.7\* | – |
| BabyBear | **7.9** / **7.3** | 34.6 / 33.3 | – | – | 9.0 / 7.9 | – |
| Goldilocks | 6.3 / **4.8** | 42.6 / 35.4 | – | – | 2.0\* / 3.0\* | **6.1** / 6.1 |
| BN254 scalar | **94.1** / **88.9** | 240 / 240 | – | 101 / 108 | – | – |
| secp256k1 base | **6.8** / **8.5** | 258 / 240 | – | 96.9 / 101 | – | – |
| P-256 base | **63.2** / **62.4** | 242 / 257 | – | 83.2 / 86.3 | – | – |
| P-256 scalar | 88.9 / 93.3 | 237 / 238 | **49.4** / **52.6** | 108 / 118 | – | – |
| Tom-256 base | **89.9** / **84.1** | 241 / 236 | – | 94.8 / 110 | – | – |
| BLS12-381 base | **186** / **170** | 388 / 387 | – | 208 / 214 | – | – |
| P-521 base | **23.0** / **23.4** | 620 / 608 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | – | 16.5 / 18.0 | – | – | 5.1\* / 5.0\* | – | – |
| BabyBear | **7.6** / **8.7** | – | 17.6 / 18.5 | – | – | 8.9 / 9.0 | – | – |
| Goldilocks | 5.7 / **4.5** | – | 27.2 / 22.3 | – | – | 2.3\* / 3.0\* | **4.9** / 5.6 | – |
| BN254 scalar | **29.6** / **27.3** | 126 / 123 | 216 / 212 | – | 172 / 160 | – | – | 113 / 111 |
| secp256k1 base | **4.2** / **8.8** | 101 / 104 | 214 / 201 | 4.4 / 3.2\* | 129 / 131 | – | – | 113 / 110 |
| P-256 base | **20.9** / **21.7** | 105 / 109 | 182 / 195 | 81.7 / 84.2 | 122 / 123 | – | – | 74.1 / 72.1 |
| P-256 scalar | 31.1 / 27.5 | 136 / 128 | 206 / 201 | **8.1** / **12.3** | 181 / 171 | – | – | 113 / 110 |
| Tom-256 base | **24.9** / **25.2** | – | 188 / 188 | – | 171 / 172 | – | – | 127 / 112 |
| BLS12-381 base | **38.3** / **39.2** | 241 / 233 | 394 / 374 | – | 267 / 257 | – | – | 353 / 336 |
| P-521 base | **10.1** / **18.5** | – | 597 / 611 | – | – | – | – | 559 / 560 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.1 | – | – | 4.5\* / 4.1\* | – |
| BabyBear | **7.3** / **7.0** | 17.6 / 18.4 | – | – | 9.0 / 8.9 | – |
| Goldilocks | **4.7** / **4.4** | 27.0 / 22.4 | – | – | 2.3\* / 2.9\* | 4.9 / 5.7 |
| BN254 scalar | **92.7** / **84.1** | 211 / 194 | – | 116 / 111 | – | – |
| secp256k1 base | **5.8** / **6.8** | 190 / 189 | – | 81.3 / 86.8 | – | – |
| P-256 base | **63.0** / **62.4** | 182 / 183 | – | 86.2 / 79.3 | – | – |
| P-256 scalar | 91.2 / 87.9 | 190 / 179 | **43.7** / **36.8** | 106 / 103 | – | – |
| Tom-256 base | **93.2** / **84.1** | 182 / 188 | – | 103 / 96.1 | – | – |
| BLS12-381 base | **172** / **170** | 366 / 368 | – | 209 / 206 | – | – |
| P-521 base | **20.5** / **15.8** | 620 / 618 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 4.6 / – | – | 13.3 / – | – | – | **3.3** / – | – | – |
| BabyBear | **6.3** / – | – | 15.3 / – | – | – | 8.0 / – | – | – |
| Goldilocks | **4.2** / – | – | 18.8 / – | – | – | 1.0\* / – | 4.4 / – | – |
| BN254 scalar | **22.1** / – | 135 / – | 130 / – | – | 143 / – | – | – | 116 / – |
| secp256k1 base | **2.2** / – | 122 / – | 155 / – | 3.0 / – | 120 / – | – | – | 125 / – |
| P-256 base | **18.9** / – | 113 / – | 125 / – | 63.8 / – | 112 / – | – | – | 56.3 / – |
| P-256 scalar | 18.1 / – | 140 / – | 120 / – | **2.2** / – | 143 / – | – | – | 96.0 / – |
| Tom-256 base | **21.0** / – | – | 122 / – | – | 128 / – | – | – | 100 / – |
| BLS12-381 base | **29.5** / – | 231 / – | 266 / – | – | 235 / – | – | – | 334 / – |
| P-521 base | **5.1** / – | – | 484 / – | – | – | – | – | 544 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 9.7 / 18.1 | – | – | – | 2.6 / 5.0 |
| BabyBear | **1.9** / **2.9** | 9.7 / 18.4 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 19.4 | – | – | 2.4 / **2.9** | 2.6 / 5.0 |
| BN254 scalar | **5.0** / **6.8** | 66.6 / 93.7 | – | 12.8 / 21.8 | – | 7.0 / 11.0 |
| secp256k1 base | **6.0** / **6.8** | 66.7 / 92.8 | 3.3\* / 3.3\* | 14.4 / 21.2 | – | 7.1 / 10.8 |
| P-256 base | **6.0** / **6.8** | 66.6 / 93.1 | 10.7 / 18.7 | 10.7 / 20.8 | – | 6.9 / 10.8 |
| P-256 scalar | **5.8** / **6.8** | 66.6 / 92.9 | 17.5 / 22.8 | 14.4 / 20.9 | – | 6.9 / 10.8 |
| Tom-256 base | **5.9** / **6.8** | 66.6 / 94.1 | – | 11.7 / 21.6 | – | 7.0 / 10.8 |
| BLS12-381 base | **9.7** / **8.9** | 87.8 / 113 | – | 20.5 / 27.4 | – | 16.9 / 12.7 |
| P-521 base | **12.3** / **15.2** | 117 / 155 | – | – | – | 25.1 / 52.6 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.3 / 12.0 | – | – | 2.0 / 3.0 | – | 2.0 / 4.0 |
| BabyBear | **1.9** / **2.9** | 4.2 / 11.9 | – | – | – | – | 2.0 / 4.0 |
| Goldilocks | **1.9** / **2.9** | 4.2 / 12.2 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.0** / **6.8** | 49.7 / 51.0 | – | 8.4 / 11.8 | – | – | 7.0 / 8.0 |
| secp256k1 base | **5.5** / 9.0 | 49.8 / 48.3 | 4.2\* / 3.9\* | 7.6 / 12.8 | – | – | 6.9 / **7.8** |
| P-256 base | **5.4** / **6.0** | 50.0 / 43.5 | 8.0 / 12.9 | 7.7 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.0** / **6.8** | 49.8 / 48.8 | 8.3 / 12.8 | 8.1 / 12.8 | – | – | 7.0 / 7.8 |
| Tom-256 base | **5.5** / 8.1 | 49.8 / 55.4 | – | 8.4 / 11.8 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **10.0** / **10.0** | 83.3 / 88.3 | – | 15.5 / 19.9 | – | – | 16.9 / 11.1 |
| P-521 base | **11.9** / **12.6** | 122 / 139 | – | – | – | – | 27.7 / 50.9 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 37.7 | – | – | – | 8.0 / 17.1 |
| BabyBear | **3.5** / **11.0** | 16.9 / 36.6 | – | – | – | 8.0 / 17.0 |
| Goldilocks | **3.8** / **9.0** | 17.8 / 34.9 | – | – | 4.9 / 11.0 | 9.3 / 17.1 |
| BN254 scalar | **75.0** / **85.9** | 151 / 178 | – | 98.1 / 98.2 | – | 137 / 144 |
| secp256k1 base | **55.8** / **57.2** | 154 / 172 | 110 / 106 | 104 / 102 | – | 143 / 153 |
| P-256 base | **58.2** / **62.7** | 135 / 179 | 83.8 / 80.6 | 84.8 / 82.1 | – | 65.9 / 78.4 |
| P-256 scalar | **79.2** / **86.9** | 149 / 173 | 156 / 142 | 94.3 / 100 | – | 117 / 139 |
| Tom-256 base | **71.1** / **75.9** | 127 / 163 | – | 92.4 / 95.8 | – | 94.8 / 147 |
| BLS12-381 base | **149** / **160** | 277 / 316 | – | 203 / 196 | – | 297 / 308 |
| P-521 base | **212** / **205** | 651 / 645 | – | – | – | 602 / 614 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | 19.8 / 34.8 | – | – | – | 7.5 / 17.2 |
| BabyBear | **3.9** / **11.0** | 20.4 / 34.9 | – | – | – | 9.1 / 17.4 |
| Goldilocks | **3.8** / **9.0** | 18.5 / 36.5 | – | – | 4.5 / 11.0 | 9.3 / 17.2 |
| BN254 scalar | **73.2** / **74.3** | 137 / 165 | – | 83.7 / 92.6 | – | 142 / 142 |
| secp256k1 base | **40.0** / **43.7** | 137 / 157 | 58.8 / 90.4 | 85.2 / 91.4 | – | 148 / 150 |
| P-256 base | **46.8** / **48.3** | 137 / 165 | 70.6 / 72.3 | 55.2 / 61.0 | – | 58.9 / 78.2 |
| P-256 scalar | **70.9** / **72.9** | 120 / 156 | 152 / 152 | 87.9 / 96.3 | – | 142 / 146 |
| Tom-256 base | **63.8** / **67.8** | 122 / 154 | – | 86.6 / 91.1 | – | 93.3 / 139 |
| BLS12-381 base | **141** / **153** | 311 / 320 | – | 178 / 180 | – | 309 / 295 |
| P-521 base | **125** / **146** | 664 / 583 | – | – | – | 575 / 542 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **257** / – | 4,117 / – | – | – | – | 989 / – |
| BabyBear | **409** / – | 4,803 / – | – | – | – | 994 / – |
| Goldilocks | **667** / – | 4,812 / – | – | – | 728 / – | 2,133 / – |
| BN254 scalar | **7,727** / – | 23,978 / – | – | 51,787 / – | – | 62,204 / – |
| secp256k1 base | **7,653** / – | 23,862 / – | 27,205 / – | 50,457 / – | – | 80,194 / – |
| P-256 base | **7,278** / – | 24,336 / – | 30,345 / – | 36,278 / – | – | 35,913 / – |
| P-256 scalar | **7,699** / – | 22,945 / – | 79,037 / – | 51,487 / – | – | 68,121 / – |
| Tom-256 base | **7,778** / – | 21,795 / – | – | 48,273 / – | – | 53,944 / – |
| BLS12-381 base | **13,225** / – | 36,030 / – | – | 150,996 / – | – | 204,593 / – |
| P-521 base | **22,352** / – | 74,307 / – | – | – | – | 572,776 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **191** / – | 1,747 / – | – | – | 480 / – |
| BabyBear | **5,610** / – | 18,652 / – | – | – | – |
| Goldilocks | **7,551** / – | 26,941 / – | – | – | – |
| BN254 scalar | **63,370** / – | 144,019 / – | – | 94,413 / – | – |
| secp256k1 base | **12,725** / – | 59,372 / – | 26,875 / – | 49,507 / – | 75,192 / – |
| P-256 base | **12,368** / – | 44,233 / – | 30,299 / – | 35,744 / – | 29,813 / – |
| P-256 scalar | **25,879** / – | 52,743 / – | 84,389 / – | 44,471 / – | – |
| Tom-256 base | **21,221** / – | 52,460 / – | – | 47,823 / – | 56,093 / – |
| BLS12-381 base | **72,366** / – | 153,349 / – | – | 156,150 / – | 196,603 / – |
| P-521 base | **72,312** / – | 308,977 / – | – | – | 324,413 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.9** / **8.9** | 34.7 / 38.9 | – | – | – |
| BabyBear | **9.2** / **11.0** | 35.8 / 36.9 | – | – | – |
| Goldilocks | 8.2 / **9.9** | 38.3 / 39.3 | – | – | **7.3** / 10.9 |
| BN254 scalar | **21.9** / **30.4** | 288 / 277 | – | 240 / 236 | – |
| secp256k1 base | **11.1** / **20.0** | 289 / 275 | 82.0 / 91.4 | 214 / 222 | – |
| P-256 base | **24.6** / **32.8** | 317 / 278 | 147 / 147 | 187 / 185 | – |
| P-256 scalar | **22.9** / **33.9** | 281 / 277 | 148 / 146 | 258 / 251 | – |
| Tom-256 base | **23.9** / **32.9** | 284 / 280 | – | 230 / 245 | – |
| BLS12-381 base | **42.8** / **42.2** | 566 / 565 | – | 440 / 441 | – |
| P-521 base | **26.6** / **45.6** | 927 / 931 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.8** / **8.9** | 34.7 / 36.5 | – | – | – |
| BabyBear | **8.9** / **11.0** | 34.5 / 36.8 | – | – | – |
| Goldilocks | **7.1** / **10.1** | 41.2 / 39.5 | – | – | 7.3 / 10.9 |
| BN254 scalar | **122** / **141** | 287 / 287 | – | 191 / 200 | – |
| secp256k1 base | **80.8** / **84.2** | 284 / 291 | – | 159 / 156 | – |
| P-256 base | **119** / 136 | 292 / 286 | – | 128 / **134** | – |
| P-256 scalar | **134** / **152** | 307 / 293 | 198 / 195 | 191 / 188 | – |
| Tom-256 base | **122** / **135** | 290 / 285 | – | 170 / 174 | – |
| BLS12-381 base | **258** / **266** | 553 / 559 | – | 389 / 382 | – |
| P-521 base | **427** / **475** | 917 / 954 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.9** / **5.5** | 33.1 / 32.0 | – | – | – |
| BabyBear | **6.0** / **7.0** | 32.5 / 30.7 | – | – | – |
| Goldilocks | **5.2** / **4.8** | 39.0 / 35.5 | – | – | 6.2 / 6.1 |
| BN254 scalar | **27.5** / **28.9** | 235 / 238 | – | 145 / 158 | – |
| secp256k1 base | **6.9** / **7.4** | 259 / 238 | 3.5\* / 2.1\* | 139 / 138 | – |
| P-256 base | **25.7** / **25.1** | 245 / 237 | 76.4 / 86.1 | 120 / 131 | – |
| P-256 scalar | 28.3 / 28.0 | 240 / 235 | **14.4** / **20.7** | 161 / 170 | – |
| Tom-256 base | **29.4** / **27.7** | 255 / 236 | – | 164 / 156 | – |
| BLS12-381 base | **47.8** / **50.6** | 442 / 386 | – | 282 / 267 | – |
| P-521 base | **11.4** / **12.7** | 599 / 613 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **5.5** | 33.4 / 26.7 | – | – | – |
| BabyBear | **6.0** / **7.1** | 34.6 / 33.3 | – | – | – |
| Goldilocks | **5.3** / **5.7** | 42.6 / 35.4 | – | – | 6.1 / 6.1 |
| BN254 scalar | **92.8** / **91.8** | 240 / 240 | – | 101 / 108 | – |
| secp256k1 base | **9.3** / **9.5** | 258 / 240 | – | 96.9 / 101 | – |
| P-256 base | **76.1** / **79.7** | 242 / 257 | – | 83.2 / 86.3 | – |
| P-256 scalar | 96.2 / 90.0 | 237 / 238 | **49.4** / **52.6** | 108 / 118 | – |
| Tom-256 base | **84.4** / **87.8** | 241 / 236 | – | 94.8 / 110 | – |
| BLS12-381 base | **183** / **179** | 388 / 387 | – | 208 / 214 | – |
| P-521 base | **22.7** / **23.8** | 620 / 608 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.6** / **7.3** | 16.5 / 18.0 | – | – | – |
| BabyBear | **7.5** / **7.0** | 17.6 / 18.5 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 27.2 / 22.3 | – | – | 4.9 / 5.6 |
| BN254 scalar | **27.9** / **29.9** | 216 / 212 | – | 172 / 160 | – |
| secp256k1 base | **5.8** / **8.1** | 214 / 201 | 4.4\* / 3.2\* | 129 / 131 | – |
| P-256 base | **23.3** / **25.1** | 182 / 195 | 81.7 / 84.2 | 122 / 123 | – |
| P-256 scalar | 26.5 / 28.2 | 206 / 201 | **8.1** / **12.3** | 181 / 171 | – |
| Tom-256 base | **32.6** / **31.7** | 188 / 188 | – | 171 / 172 | – |
| BLS12-381 base | **53.3** / **45.6** | 394 / 374 | – | 267 / 257 | – |
| P-521 base | **9.7** / **11.0** | 597 / 611 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.5** | 16.5 / 18.1 | – | – | – |
| BabyBear | **7.5** / **7.0** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.7** / **4.4** | 27.0 / 22.4 | – | – | 4.9 / 5.7 |
| BN254 scalar | **103** / **94.2** | 211 / 194 | – | 116 / 111 | – |
| secp256k1 base | **7.2** / **7.9** | 190 / 189 | – | 81.3 / 86.8 | – |
| P-256 base | **78.0** / **71.0** | 182 / 183 | – | 86.2 / 79.3 | – |
| P-256 scalar | 94.9 / 97.1 | 190 / 179 | **43.7** / **36.8** | 106 / 103 | – |
| Tom-256 base | **91.4** / **86.4** | 182 / 188 | – | 103 / 96.1 | – |
| BLS12-381 base | **178** / **173** | 366 / 368 | – | 209 / 206 | – |
| P-521 base | **19.2** / **19.9** | 620 / 618 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **13.3** / – | – | – | – |
| BabyBear | **15.3** / – | – | – | – |
| Goldilocks | 18.8 / – | – | – | **4.4** / – |
| BN254 scalar | **130** / – | – | 143 / – | – |
| secp256k1 base | 155 / – | **3.0** / – | 120 / – | – |
| P-256 base | 125 / – | **63.8** / – | 112 / – | – |
| P-256 scalar | 120 / – | **2.2** / – | 143 / – | – |
| Tom-256 base | **122** / – | – | 128 / – | – |
| BLS12-381 base | 266 / – | – | **235** / – | – |
| P-521 base | **484** / – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
