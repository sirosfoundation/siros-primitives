# Benchmarks

`prime-field` at commit `e2d3450` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 12; 1-minute load 30.81 before, 37.94 after; its SMT sibling, CPU 28, 83% busy |
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
| Mersenne-31 | **2.5** / **3.0** | – | 12.5 / 18.9 | – | – | 2.9 / 3.1 | – | 3.1 / 5.0 | – |
| BabyBear | **1.9** / **3.0** | – | 12.5 / 18.7 | – | – | 2.6 / 3.1 | – | 3.2 / 4.9 | – |
| Goldilocks | **2.5** / **3.0** | – | 12.9 / 19.6 | – | – | 2.2\* / 3.0 | 3.4 / 3.2 | 3.0 / 4.9 | – |
| BN254 scalar | **6.9** / **7.0** | 19.0 / 13.2 | 81.1 / 93.7 | – | 17.8 / 21.9 | – | – | 8.8 / 10.7 | – |
| secp256k1 base | **8.1** / **7.2** | 18.4 / 14.3 | 75.2 / 92.8 | 4.3\* / 4.0\* | 19.3 / 22.1 | – | – | 8.2 / 10.6 | – |
| P-256 base | **7.3** / **6.9** | 19.2 / 15.2 | 77.2 / 93.9 | 14.1 / 18.5 | 14.1 / 19.0 | – | – | 9.1 / 10.4 | 474 / – |
| P-256 scalar | **7.8** / **7.1** | 18.6 / 14.7 | 78.1 / 92.2 | 22.5 / 22.9 | 19.2 / 21.8 | – | – | 9.1 / 10.5 | 457 / – |
| Tom-256 base | **6.0** / **7.2** | – | 78.8 / 92.5 | – | 17.7 / 21.3 | – | – | 9.1 / 10.6 | – |
| BLS12-381 base | **12.4** / **9.4** | 22.1 / 15.7 | 110 / 125 | – | 27.3 / 28.8 | – | – | 22.1 / 13.0 | – |
| P-521 base | **16.7** / **23.4** | – | 146 / 163 | – | – | – | – | 33.0 / 55.3 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 2.9 / **2.9** | – | 5.8 / 13.0 | – | – | 2.8\* / 3.0 | – | **2.7** / 4.0 | – |
| BabyBear | **2.5** / **2.9** | – | 5.6 / 12.8 | – | – | 3.0 / 3.1 | – | 3.2 / 4.0 | – |
| Goldilocks | **2.5** / **3.0** | – | 5.5 / 12.7 | – | – | 2.8 / 3.0 | 2.7 / 3.1 | 2.7 / 4.0 | – |
| BN254 scalar | **7.3** / **7.1** | 19.5 / 15.3 | 56.5 / 52.5 | – | 12.2 / 12.0 | – | – | 9.4 / 8.4 | – |
| secp256k1 base | **6.4** / 8.5 | 19.9 / 15.0 | 56.2 / 49.2 | 4.8\* / 5.3\* | 11.2 / 12.8 | – | – | 9.3 / **8.3** | – |
| P-256 base | **7.1** / **7.1** | 18.6 / 14.0 | 56.3 / 44.3 | 10.6 / 13.0 | 11.1 / 11.9 | – | – | 9.4 / 8.3 | 238 / – |
| P-256 scalar | **7.3** / **7.0** | 19.7 / 15.2 | 57.3 / 48.0 | 12.3 / 13.0 | 11.7 / 12.7 | – | – | 9.3 / 8.5 | 239 / – |
| Tom-256 base | **7.9** / **8.2** | – | 56.2 / 55.2 | – | 12.4 / 12.6 | – | – | 8.3 / 8.6 | – |
| BLS12-381 base | **13.9** / 11.9 | 25.1 / 19.8 | 87.7 / 103 | – | 19.4 / 19.8 | – | – | 17.9 / **11.2** | – |
| P-521 base | **14.9** / **19.1** | – | 137 / 151 | – | – | – | – | 34.9 / 54.0 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.5** / **7.0** | – | 20.9 / 35.1 | – | – | 3.7 / 7.1 | – | 9.8 / 17.3 | – |
| BabyBear | **4.0** / **11.0** | – | 21.7 / 35.1 | – | – | 5.5 / 13.2 | – | 9.9 / 17.2 | – |
| Goldilocks | 5.0 / **9.1** | – | 21.8 / 35.0 | – | – | 4.7\* / 10.0 | **4.9** / 11.0 | 9.6 / 17.1 | – |
| BN254 scalar | **79.2** / **77.0** | 96.2 / 112 | 153 / 182 | – | 111 / 111 | – | – | 149 / 155 | – |
| secp256k1 base | **56.2** / **56.8** | 106 / 121 | 154 / 181 | 107 / 109 | 107 / 109 | – | – | 143 / 155 | – |
| P-256 base | **58.2** / **62.4** | 80.8 / 92.2 | 152 / 180 | 83.0 / 82.4 | 86.4 / 80.8 | – | – | 69.8 / 83.0 | 845 / – |
| P-256 scalar | **86.3** / **88.5** | 104 / 121 | 151 / 180 | 167 / 168 | 116 / 116 | – | – | 145 / 155 | 922 / – |
| Tom-256 base | **84.5** / **81.1** | – | 151 / 179 | – | 106 / 111 | – | – | 147 / 155 | – |
| BLS12-381 base | **162** / **160** | 199 / 207 | 344 / 354 | – | 223 / 220 | – | – | 324 / 321 | – |
| P-521 base | **222** / **224** | – | 660 / 680 | – | – | – | – | 638 / 625 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.3** / **7.1** | – | 20.9 / 35.0 | – | – | 3.5 / 7.0\* | – | 9.7 / 17.2 | – |
| BabyBear | **4.1** / **11.0** | – | 20.5 / 35.1 | – | – | 5.4 / 13.2 | – | 10.6 / 17.2 | – |
| Goldilocks | 5.1 / **9.1** | – | 21.7 / 36.2 | – | – | 4.8\* / 10.1 | **5.0** / 11.1 | 10.4 / 17.2 | – |
| BN254 scalar | **72.7** / **69.9** | 95.7 / 106 | 141 / 166 | – | 91.7 / 95.4 | – | – | 150 / 153 | – |
| secp256k1 base | **40.8** / **45.3** | 103 / 112 | 143 / 169 | 80.6 / 94.7 | 88.1 / 92.2 | – | – | 148 / 149 | – |
| P-256 base | **48.2** / **48.7** | 81.7 / 88.3 | 143 / 169 | 73.7 / 84.3 | 65.5 / 66.0 | – | – | 65.0 / 77.6 | 827 / – |
| P-256 scalar | **75.6** / **75.4** | 107 / 117 | 141 / 166 | 162 / 170 | 96.1 / 100 | – | – | 148 / 152 | 894 / – |
| Tom-256 base | **70.3** / **68.8** | – | 141 / 169 | – | 92.7 / 92.4 | – | – | 150 / 152 | – |
| BLS12-381 base | **148** / **153** | 181 / 188 | 345 / 360 | – | 193 / 190 | – | – | 320 / 326 | – |
| P-521 base | **151** / **153** | – | 680 / 688 | – | – | – | – | 611 / 613 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **215** / – | – | 1,739 / – | – | – | 719 / – | – | 966 / – | – |
| BabyBear | **214** / – | – | 1,760 / – | – | – | 698 / – | – | 1,191 / – | – |
| Goldilocks | **439** / – | – | 3,327 / – | – | – | 1,503 / – | 728 / – | 1,889 / – | – |
| BN254 scalar | **5,577** / – | 19,830 / – | 16,447 / – | – | 8,336 / – | – | – | 16,973 / – | – |
| secp256k1 base | **5,571** / – | 19,011 / – | 16,810 / – | 26,486 / – | 8,397 / – | – | – | 17,577 / – | – |
| P-256 base | **5,468** / – | 18,933 / – | 16,861 / – | 29,567 / – | 8,283 / – | – | – | 17,370 / – | 138,414 / – |
| P-256 scalar | **5,585** / – | 19,071 / – | 16,960 / – | 37,000 / – | 8,387 / – | – | – | 17,279 / – | 135,997 / – |
| Tom-256 base | **5,746** / – | – | 16,356 / – | – | 8,270 / – | – | – | 16,938 / – | – |
| BLS12-381 base | **10,340** / – | 46,626 / – | 29,623 / – | – | 14,619 / – | – | – | 46,505 / – | – |
| P-521 base | **17,155** / – | – | 50,763 / – | – | – | – | – | 95,864 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **180** / – | – | 1,733 / – | – | – | 186 / – | 479 / – | – |
| BabyBear | **2,185** / – | – | 18,600 / – | – | – | 3,070 / – | – | – |
| Goldilocks | **3,083** / – | – | 26,811 / – | – | – | 3,432 / – | – | – |
| BN254 scalar | **38,080** / – | 52,852 / – | 147,737 / – | – | 98,845 / – | – | – | – |
| secp256k1 base | **12,398** / – | 48,997 / – | 59,524 / – | 29,071 / – | 50,352 / – | – | 80,573 / – | – |
| P-256 base | **13,024** / – | 22,318 / – | 49,296 / – | 28,960 / – | 37,915 / – | – | 31,259 / – | 249,561 / – |
| P-256 scalar | **26,185** / – | 44,386 / – | 57,943 / – | 85,620 / – | 44,987 / – | – | – | – |
| Tom-256 base | **21,827** / – | – | 55,836 / – | – | 50,955 / – | – | 58,870 / – | – |
| BLS12-381 base | **73,900** / – | 111,371 / – | 158,230 / – | – | 161,716 / – | – | 215,646 / – | – |
| P-521 base | **76,828** / – | – | 331,373 / – | – | – | – | 326,369 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **10.0** / **9.6** | – | 43.4 / 41.1 | – | – | 6.8\* / 7.0\* | – | – |
| BabyBear | 16.8 / 16.4 | – | 42.6 / 41.0 | – | – | **13.8** / **14.9** | – | – |
| Goldilocks | **8.9** / **10.3** | – | 48.1 / 44.6 | – | – | 4.9\* / 9.9\* | 9.4 / 11.1 | – |
| BN254 scalar | **23.0** / **26.2** | 256 / 251 | 334 / 325 | – | 286 / 287 | – | – | 300 / 297 |
| secp256k1 base | **10.7** / **14.8** | 237 / 236 | 340 / 322 | 102 / 98.9 | 256 / 277 | – | – | 282 / 304 |
| P-256 base | **23.8** / **26.5** | 213 / 215 | 335 / 307 | 163 / 163 | 239 / 237 | – | – | 160 / 173 |
| P-256 scalar | **26.2** / **28.6** | 268 / 268 | 329 / 317 | 157 / 164 | 302 / 296 | – | – | 290 / 290 |
| Tom-256 base | **24.4** / **28.2** | – | 333 / 335 | – | 289 / 285 | – | – | 274 / 293 |
| BLS12-381 base | **31.7** / **34.5** | 481 / 467 | 664 / 677 | – | 534 / 516 | – | – | 704 / 653 |
| P-521 base | **27.7** / **33.6** | – | 1,118 / 1,087 | – | – | – | – | 1,173 / 1,244 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.1** / **9.0** | 40.4 / 42.1 | – | – | 6.3\* / 7.2\* | – |
| BabyBear | 16.9 / 15.5 | 44.8 / 43.7 | – | – | **13.8** / **14.9** | – |
| Goldilocks | 9.1 / **10.1** | 47.2 / 46.5 | – | – | 4.6\* / 10.0\* | **8.9** / 10.9 |
| BN254 scalar | **130** / **134** | 339 / 332 | – | 213 / 216 | – | – |
| secp256k1 base | **79.1** / **84.2** | 342 / 340 | – | 188 / 188 | – | – |
| P-256 base | **124** / **124** | 309 / 294 | – | 153 / 157 | – | – |
| P-256 scalar | **132** / **137** | 330 / 332 | 199 / 204 | 212 / 218 | – | – |
| Tom-256 base | **132** / **129** | 340 / 336 | – | 200 / 200 | – | – |
| BLS12-381 base | **270** / **268** | 663 / 672 | – | 431 / 418 | – | – |
| P-521 base | **416** / **425** | 1,097 / 1,127 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.1** / **7.6** | – | 39.5 / 35.5 | – | – | 5.4\* / 5.0\* | – | – |
| BabyBear | 14.1 / 13.8 | – | 40.6 / 33.6 | – | – | **9.0** / **7.9** | – | – |
| Goldilocks | **5.7** / **5.7** | – | 44.6 / 40.5 | – | – | 2.0\* / 3.0\* | 7.5 / 6.5 | – |
| BN254 scalar | **31.2** / **26.8** | 136 / 136 | 254 / 263 | – | 194 / 199 | – | – | 171 / 162 |
| secp256k1 base | **8.3** / **7.7** | 107 / 107 | 260 / 263 | 4.1\* / 2.8\* | 180 / 180 | – | – | 171 / 163 |
| P-256 base | **29.7** / **26.4** | 113 / 114 | 262 / 260 | 93.7 / 93.2 | 164 / 161 | – | – | 76.4 / 79.6 |
| P-256 scalar | 30.1 / 26.1 | 139 / 139 | 248 / 265 | **18.8** / **22.3** | 210 / 203 | – | – | 172 / 164 |
| Tom-256 base | **31.8** / **31.5** | – | 261 / 262 | – | 207 / 201 | – | – | 172 / 169 |
| BLS12-381 base | **51.8** / **47.0** | 263 / 250 | 453 / 456 | – | 314 / 318 | – | – | 376 / 354 |
| P-521 base | **17.0** / **21.8** | – | 703 / 713 | – | – | – | – | 708 / 669 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.2** / **7.1** | 38.5 / 33.3 | – | – | 5.5\* / 4.8\* | – |
| BabyBear | 13.7 / 13.8 | 33.0 / 31.4 | – | – | **9.0** / **8.9** | – |
| Goldilocks | **6.9** / **6.7** | 43.5 / 37.0 | – | – | 2.3\* / 3.0\* | 7.7 / 7.2 |
| BN254 scalar | 140 / 132 | 257 / 262 | – | **121** / **126** | – | – |
| secp256k1 base | **83.0** / **85.0** | 263 / 259 | – | 96.6 / 102 | – | – |
| P-256 base | 133 / 130 | 256 / 254 | – | **88.7** / **94.6** | – | – |
| P-256 scalar | 140 / 140 | 250 / 245 | **49.4** / **45.1** | 119 / 120 | – | – |
| Tom-256 base | 137 / 136 | 237 / 245 | – | **94.4** / **113** | – | – |
| BLS12-381 base | 270 / 251 | 449 / 434 | – | **228** / **231** | – | – |
| P-521 base | **414** / **400** | 715 / 723 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **7.4** | – | 20.6 / 19.7 | – | – | 6.0\* / 5.6\* | – | – |
| BabyBear | 14.0 / 13.2 | – | 20.9 / 21.2 | – | – | **9.4** / **9.4** | – | – |
| Goldilocks | 6.0 / **5.5** | – | 30.6 / 29.1 | – | – | 2.0\* / 3.0\* | **5.5** / 6.9 | – |
| BN254 scalar | **30.8** / **30.6** | 125 / 128 | 218 / 216 | – | 195 / 192 | – | – | 171 / 168 |
| secp256k1 base | **5.6** / **8.9** | 112 / 107 | 216 / 211 | 4.2\* / 3.1\* | 179 / 173 | – | – | 165 / 161 |
| P-256 base | **27.4** / **26.8** | 113 / 109 | 221 / 209 | 96.1 / 95.4 | 178 / 158 | – | – | 77.3 / 63.8 |
| P-256 scalar | 29.0 / 24.7 | 138 / 135 | 218 / 213 | **10.1** / **12.5** | 189 / 188 | – | – | 167 / 163 |
| Tom-256 base | **29.9** / **31.0** | – | 219 / 219 | – | 195 / 194 | – | – | 163 / 165 |
| BLS12-381 base | **47.8** / **51.2** | 249 / 241 | 429 / 439 | – | 316 / 319 | – | – | 369 / 363 |
| P-521 base | **13.1** / **18.9** | – | 693 / 715 | – | – | – | – | 671 / 650 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.0** / **6.9** | 19.8 / 20.6 | – | – | 5.9\* / 5.2\* | – |
| BabyBear | 13.8 / 14.5 | 22.4 / 21.3 | – | – | **9.3** / **9.5** | – |
| Goldilocks | 6.5 / **5.6** | 30.2 / 26.1 | – | – | 2.2\* / 2.9\* | **6.3** / 6.6 |
| BN254 scalar | 141 / 141 | 217 / 219 | – | **122** / **119** | – | – |
| secp256k1 base | **90.3** / **83.8** | 214 / 213 | – | 93.2 / 93.2 | – | – |
| P-256 base | 133 / 127 | 216 / 215 | – | **88.8** / **89.8** | – | – |
| P-256 scalar | 142 / 142 | 213 / 218 | **47.5** / **49.6** | 116 / 117 | – | – |
| Tom-256 base | 138 / 136 | 211 / 212 | – | **107** / **111** | – | – |
| BLS12-381 base | 282 / 271 | 420 / 429 | – | **231** / **226** | – | – |
| P-521 base | **423** / **425** | 692 / 714 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 6.1 / – | – | 15.5 / – | – | – | **4.4** / – | – | – |
| BabyBear | 11.5 / – | – | 16.4 / – | – | – | **8.1** / – | – | – |
| Goldilocks | 4.9 / – | – | 21.1 / – | – | – | 1.9\* / – | **4.4** / – | – |
| BN254 scalar | **25.8** / – | 146 / – | 158 / – | – | 183 / – | – | – | 149 / – |
| secp256k1 base | **2.5** / – | 131 / – | 155 / – | 2.6 / – | 152 / – | – | – | 148 / – |
| P-256 base | **21.3** / – | 124 / – | 160 / – | 78.0 / – | 154 / – | – | – | 81.3 / – |
| P-256 scalar | 21.0 / – | 149 / – | 161 / – | **2.5** / – | 170 / – | – | – | 128 / – |
| Tom-256 base | **23.2** / – | – | 157 / – | – | 161 / – | – | – | 145 / – |
| BLS12-381 base | **32.2** / – | 249 / – | 328 / – | – | 275 / – | – | – | 358 / – |
| P-521 base | **5.2** / – | – | 555 / – | – | – | – | – | 639 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | 12.5 / 18.9 | – | – | – | 3.1 / 5.0 |
| BabyBear | **2.6** / **2.9** | 12.5 / 18.7 | – | – | – | 3.2 / 4.9 |
| Goldilocks | **2.0** / **3.0** | 12.9 / 19.6 | – | – | 3.4 / 3.2 | 3.0 / 4.9 |
| BN254 scalar | **7.0** / **6.8** | 81.1 / 93.7 | – | 17.8 / 21.9 | – | 8.8 / 10.7 |
| secp256k1 base | **7.3** / **6.8** | 75.2 / 92.8 | 4.3\* / 4.0\* | 19.3 / 22.1 | – | 8.2 / 10.6 |
| P-256 base | **8.1** / **7.1** | 77.2 / 93.9 | 14.1 / 18.5 | 14.1 / 19.0 | – | 9.1 / 10.4 |
| P-256 scalar | **7.9** / **7.0** | 78.1 / 92.2 | 22.5 / 22.9 | 19.2 / 21.8 | – | 9.1 / 10.5 |
| Tom-256 base | **8.2** / **6.8** | 78.8 / 92.5 | – | 17.7 / 21.3 | – | 9.1 / 10.6 |
| BLS12-381 base | **13.0** / **10.0** | 110 / 125 | – | 27.3 / 28.8 | – | 22.1 / 13.0 |
| P-521 base | **19.0** / **23.1** | 146 / 163 | – | – | – | 33.0 / 55.3 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 3.1 / **2.9** | 5.8 / 13.0 | – | – | 2.8\* / 3.0 | – | **2.7** / 4.0 |
| BabyBear | **2.4** / **3.0** | 5.6 / 12.8 | – | – | – | – | 3.2 / 4.0 |
| Goldilocks | **2.3** / **3.0** | 5.5 / 12.7 | – | – | – | 2.7 / 3.1 | 2.7 / 4.0 |
| BN254 scalar | **8.2** / **7.5** | 56.5 / 52.5 | – | 12.2 / 12.0 | – | – | 9.4 / 8.4 |
| secp256k1 base | **6.5** / 8.6 | 56.2 / 49.2 | 4.8\* / 5.3\* | 11.2 / 12.8 | – | – | 9.3 / **8.3** |
| P-256 base | **5.9** / **7.2** | 56.3 / 44.3 | 10.6 / 13.0 | 11.1 / 11.9 | – | – | 9.4 / 8.3 |
| P-256 scalar | **7.2** / **7.1** | 57.3 / 48.0 | 12.3 / 13.0 | 11.7 / 12.7 | – | – | 9.3 / 8.5 |
| Tom-256 base | **6.9** / **8.0** | 56.2 / 55.2 | – | 12.4 / 12.6 | – | – | 8.3 / 8.6 |
| BLS12-381 base | **12.1** / **11.0** | 87.7 / 103 | – | 19.4 / 19.8 | – | – | 17.9 / 11.2 |
| P-521 base | **13.9** / **19.4** | 137 / 151 | – | – | – | – | 34.9 / 54.0 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.4** / **7.0** | 20.9 / 35.1 | – | – | – | 9.8 / 17.3 |
| BabyBear | **3.9** / **11.0** | 21.7 / 35.1 | – | – | – | 9.9 / 17.2 |
| Goldilocks | **4.8** / **9.0** | 21.8 / 35.0 | – | – | 4.9 / 11.0 | 9.6 / 17.1 |
| BN254 scalar | **77.1** / **82.2** | 153 / 182 | – | 111 / 111 | – | 149 / 155 |
| secp256k1 base | **55.9** / **57.6** | 154 / 181 | 107 / 109 | 107 / 109 | – | 143 / 155 |
| P-256 base | **58.9** / **62.3** | 152 / 180 | 83.0 / 82.4 | 86.4 / 80.8 | – | 69.8 / 83.0 |
| P-256 scalar | **84.4** / **88.5** | 151 / 180 | 167 / 168 | 116 / 116 | – | 145 / 155 |
| Tom-256 base | **82.7** / **79.5** | 151 / 179 | – | 106 / 111 | – | 147 / 155 |
| BLS12-381 base | **169** / **169** | 344 / 354 | – | 223 / 220 | – | 324 / 321 |
| P-521 base | **224** / **219** | 660 / 680 | – | – | – | 638 / 625 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **3.5** / **7.1** | 20.9 / 35.0 | – | – | – | 9.7 / 17.2 |
| BabyBear | **4.0** / **11.0** | 20.5 / 35.1 | – | – | – | 10.6 / 17.2 |
| Goldilocks | 5.1 / **9.1** | 21.7 / 36.2 | – | – | **5.0** / 11.1 | 10.4 / 17.2 |
| BN254 scalar | **73.4** / **74.3** | 141 / 166 | – | 91.7 / 95.4 | – | 150 / 153 |
| secp256k1 base | **41.0** / **45.5** | 143 / 169 | 80.6 / 94.7 | 88.1 / 92.2 | – | 148 / 149 |
| P-256 base | **51.7** / **48.4** | 143 / 169 | 73.7 / 84.3 | 65.5 / 66.0 | – | 65.0 / 77.6 |
| P-256 scalar | **75.6** / **76.8** | 141 / 166 | 162 / 170 | 96.1 / 100 | – | 148 / 152 |
| Tom-256 base | **69.1** / **68.8** | 141 / 169 | – | 92.7 / 92.4 | – | 150 / 152 |
| BLS12-381 base | **159** / **153** | 345 / 360 | – | 193 / 190 | – | 320 / 326 |
| P-521 base | **151** / **153** | 680 / 688 | – | – | – | 611 / 613 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **254** / – | 4,911 / – | – | – | – | 1,000 / – |
| BabyBear | **421** / – | 4,962 / – | – | – | – | 997 / – |
| Goldilocks | **664** / – | 4,960 / – | – | – | 728 / – | 2,131 / – |
| BN254 scalar | **7,990** / – | 25,221 / – | – | 52,225 / – | – | 63,771 / – |
| secp256k1 base | **8,121** / – | 24,357 / – | 26,486 / – | 50,135 / – | – | 81,581 / – |
| P-256 base | **7,601** / – | 24,217 / – | 29,567 / – | 37,128 / – | – | 37,455 / – |
| P-256 scalar | **7,849** / – | 26,916 / – | 82,958 / – | 54,710 / – | – | 69,687 / – |
| Tom-256 base | **8,367** / – | 25,567 / – | – | 50,330 / – | – | 60,076 / – |
| BLS12-381 base | **14,581** / – | 43,729 / – | – | 163,898 / – | – | 221,687 / – |
| P-521 base | **24,711** / – | 79,903 / – | – | – | – | 666,082 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **190** / – | 1,733 / – | – | – | 479 / – |
| BabyBear | **5,644** / – | 18,600 / – | – | – | – |
| Goldilocks | **7,587** / – | 26,811 / – | – | – | – |
| BN254 scalar | **64,315** / – | 147,737 / – | – | 98,845 / – | – |
| secp256k1 base | **12,710** / – | 59,524 / – | 29,071 / – | 50,352 / – | 80,573 / – |
| P-256 base | **13,220** / – | 49,296 / – | 28,960 / – | 37,915 / – | 31,259 / – |
| P-256 scalar | **26,976** / – | 57,943 / – | 85,620 / – | 44,987 / – | – |
| Tom-256 base | **22,291** / – | 55,836 / – | – | 50,955 / – | 58,870 / – |
| BLS12-381 base | **75,968** / – | 158,230 / – | – | 161,716 / – | 215,646 / – |
| P-521 base | **76,969** / – | 331,373 / – | – | – | 326,369 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.9** / **9.3** | 43.4 / 41.1 | – | – | – |
| BabyBear | **15.8** / **17.2** | 42.6 / 41.0 | – | – | – |
| Goldilocks | **9.1** / **10.4** | 48.1 / 44.6 | – | – | 9.4 / 11.1 |
| BN254 scalar | **28.5** / **30.8** | 334 / 325 | – | 286 / 287 | – |
| secp256k1 base | **13.7** / **20.5** | 340 / 322 | 102 / 98.9 | 256 / 277 | – |
| P-256 base | **26.4** / **33.5** | 335 / 307 | 163 / 163 | 239 / 237 | – |
| P-256 scalar | **27.9** / **33.9** | 329 / 317 | 157 / 164 | 302 / 296 | – |
| Tom-256 base | **28.6** / **34.3** | 333 / 335 | – | 289 / 285 | – |
| BLS12-381 base | **42.8** / **43.6** | 664 / 677 | – | 534 / 516 | – |
| P-521 base | **32.7** / **45.6** | 1,118 / 1,087 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.4** / **9.6** | 40.4 / 42.1 | – | – | – |
| BabyBear | **16.8** / **16.6** | 44.8 / 43.7 | – | – | – |
| Goldilocks | **8.9** / **10.0** | 47.2 / 46.5 | – | – | **8.9** / 10.9 |
| BN254 scalar | **146** / **152** | 339 / 332 | – | 213 / 216 | – |
| secp256k1 base | **93.0** / **98.2** | 342 / 340 | – | 188 / 188 | – |
| P-256 base | **133** / **146** | 309 / 294 | – | 153 / 157 | – |
| P-256 scalar | **145** / **158** | 330 / 332 | 199 / 204 | 212 / 218 | – |
| Tom-256 base | **141** / **150** | 340 / 336 | – | 200 / 200 | – |
| BLS12-381 base | **303** / **305** | 663 / 672 | – | 431 / 418 | – |
| P-521 base | **487** / **488** | 1,097 / 1,127 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.6** / **7.6** | 39.5 / 35.5 | – | – | – |
| BabyBear | **13.9** / **14.4** | 40.6 / 33.6 | – | – | – |
| Goldilocks | **6.7** / **6.3** | 44.6 / 40.5 | – | – | 7.5 / 6.5 |
| BN254 scalar | **33.7** / **36.6** | 254 / 263 | – | 194 / 199 | – |
| secp256k1 base | **8.6** / **7.4** | 260 / 263 | 4.1\* / 2.8\* | 180 / 180 | – |
| P-256 base | **33.0** / **32.0** | 262 / 260 | 93.7 / 93.2 | 164 / 161 | – |
| P-256 scalar | 28.6 / 29.1 | 248 / 265 | **18.8** / **22.3** | 210 / 203 | – |
| Tom-256 base | **35.7** / **34.1** | 261 / 262 | – | 207 / 201 | – |
| BLS12-381 base | **53.5** / **56.0** | 453 / 456 | – | 314 / 318 | – |
| P-521 base | **16.2** / **21.9** | 703 / 713 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.5** / **7.0** | 38.5 / 33.3 | – | – | – |
| BabyBear | **13.2** / **13.8** | 33.0 / 31.4 | – | – | – |
| Goldilocks | **5.3** / **4.9** | 43.5 / 37.0 | – | – | 7.7 / 7.2 |
| BN254 scalar | 151 / 142 | 257 / 262 | – | **121** / **126** | – |
| secp256k1 base | 102 / **101** | 263 / 259 | – | **96.6** / 102 | – |
| P-256 base | 144 / 145 | 256 / 254 | – | **88.7** / **94.6** | – |
| P-256 scalar | 160 / 157 | 250 / 245 | **49.4** / **45.1** | 119 / 120 | – |
| Tom-256 base | 152 / 148 | 237 / 245 | – | **94.4** / **113** | – |
| BLS12-381 base | 301 / 299 | 449 / 434 | – | **228** / **231** | – |
| P-521 base | **485** / **489** | 715 / 723 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.6** / **7.5** | 20.6 / 19.7 | – | – | – |
| BabyBear | **12.0** / **14.0** | 20.9 / 21.2 | – | – | – |
| Goldilocks | 6.4 / **5.8** | 30.6 / 29.1 | – | – | **5.5** / 6.9 |
| BN254 scalar | **36.2** / **37.5** | 218 / 216 | – | 195 / 192 | – |
| secp256k1 base | **5.8** / **7.3** | 216 / 211 | 4.2\* / 3.1\* | 179 / 173 | – |
| P-256 base | **30.9** / **31.0** | 221 / 209 | 96.1 / 95.4 | 178 / 158 | – |
| P-256 scalar | 32.0 / 34.0 | 218 / 213 | **10.1** / **12.5** | 189 / 188 | – |
| Tom-256 base | **33.9** / **34.8** | 219 / 219 | – | 195 / 194 | – |
| BLS12-381 base | **56.2** / **57.2** | 429 / 439 | – | 316 / 319 | – |
| P-521 base | **13.3** / **18.0** | 693 / 715 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.6** / **7.4** | 19.8 / 20.6 | – | – | – |
| BabyBear | **14.3** / **13.5** | 22.4 / 21.3 | – | – | – |
| Goldilocks | **5.9** / **5.4** | 30.2 / 26.1 | – | – | 6.3 / 6.6 |
| BN254 scalar | 153 / 151 | 217 / 219 | – | **122** / **119** | – |
| secp256k1 base | 98.2 / 96.8 | 214 / 213 | – | **93.2** / **93.2** | – |
| P-256 base | 148 / 143 | 216 / 215 | – | **88.8** / **89.8** | – |
| P-256 scalar | 161 / 158 | 213 / 218 | **47.5** / **49.6** | 116 / 117 | – |
| Tom-256 base | 151 / 151 | 211 / 212 | – | **107** / **111** | – |
| BLS12-381 base | 317 / 312 | 420 / 429 | – | **231** / **226** | – |
| P-521 base | **482** / **482** | 692 / 714 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **15.5** / – | – | – | – |
| BabyBear | **16.4** / – | – | – | – |
| Goldilocks | 21.1 / – | – | – | **4.4** / – |
| BN254 scalar | **158** / – | – | 183 / – | – |
| secp256k1 base | 155 / – | **2.6** / – | 152 / – | – |
| P-256 base | 160 / – | **78.0** / – | 154 / – | – |
| P-256 scalar | 161 / – | **2.5** / – | 170 / – | – |
| Tom-256 base | **157** / – | – | 161 / – | – |
| BLS12-381 base | 328 / – | – | **275** / – | – |
| P-521 base | **555** / – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
