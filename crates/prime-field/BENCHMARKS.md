# Benchmarks

`prime-field` at commit `652cc46` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-04.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 12; 1-minute load 20.88 before, 18.81 after; its SMT sibling, CPU 28, 43% busy |
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
| Mersenne-31 | **1.9** / **2.9** | – | 9.7 / 18.3 | – | – | 2.0 / 3.0 | – | 2.6 / 4.9 | – |
| BabyBear | **1.9** / **2.9** | – | 9.7 / 18.9 | – | – | 2.0 / 3.0 | – | 2.6 / 4.9 | – |
| Goldilocks | **1.9** / 3.0 | – | 9.8 / 19.4 | – | – | 2.0 / 2.9\* | 2.4 / **2.9** | 2.5 / 4.9 | – |
| BN254 scalar | **5.1** / **6.8** | 17.5 / 12.8 | 66.6 / 93.5 | – | 12.8 / 21.7 | – | – | 6.8 / 10.7 | – |
| secp256k1 base | **5.8** / **6.8** | 17.4 / 13.1 | 66.6 / 94.0 | 4.4\* / 3.5\* | 14.4 / 21.2 | – | – | 6.8 / 10.7 | – |
| P-256 base | **5.9** / **6.8** | 18.7 / 14.0 | 66.8 / 93.1 | 10.6 / 18.5 | 10.7 / 20.7 | – | – | 6.9 / 10.6 | 333 / – |
| P-256 scalar | **5.8** / **6.8** | 18.1 / 13.7 | 66.4 / 92.6 | 17.4 / 22.7 | 14.5 / 21.0 | – | – | 6.8 / 10.7 | 343 / – |
| Tom-256 base | **5.9** / **6.8** | – | 66.6 / 93.4 | – | 11.7 / 20.6 | – | – | 6.9 / 10.7 | – |
| BLS12-381 base | **10.0** / **9.0** | 20.4 / 14.8 | 87.6 / 113 | – | 20.3 / 27.3 | – | – | 16.8 / 12.6 | – |
| P-521 base | **12.1** / **14.8** | – | 125 / 155 | – | – | – | – | 25.1 / 52.8 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 2.0 / **2.9** | – | 4.2 / 12.0 | – | – | 2.0 / 3.0 | – | **1.9** / 3.9 | – |
| BabyBear | **1.9** / **3.0** | – | 4.3 / 12.0 | – | – | 2.0 / **3.0** | – | 2.0 / 4.0 | – |
| Goldilocks | **1.9** / **2.9** | – | 4.2 / 12.6 | – | – | 1.9 / 2.9 | **1.9** / **2.9** | 2.0 / 4.0 | – |
| BN254 scalar | **5.1** / **7.0** | 16.7 / 13.4 | 49.8 / 51.2 | – | 8.3 / 11.7 | – | – | 6.9 / 7.8 | – |
| secp256k1 base | **5.5** / 8.0 | 17.6 / 13.8 | 49.9 / 48.2 | 4.6\* / 4.1\* | 7.5 / 12.8 | – | – | 6.8 / **7.8** | – |
| P-256 base | **5.5** / **7.5** | 16.7 / 12.8 | 49.9 / 43.6 | 7.9 / 12.8 | 7.5 / 11.8 | – | – | 6.8 / 7.8 | 153 / – |
| P-256 scalar | **5.1** / **6.8** | 16.7 / 13.7 | 50.0 / 48.7 | 8.3 / 12.8 | 7.9 / 12.8 | – | – | 6.8 / 7.8 | 164 / – |
| Tom-256 base | **5.6** / 8.0 | – | 49.9 / 55.1 | – | 8.3 / 11.8 | – | – | 6.8 / **7.8** | – |
| BLS12-381 base | **10.0** / **10.0** | 22.2 / 16.3 | 83.1 / 88.1 | – | 16.6 / 19.7 | – | – | 16.8 / 11.1 | – |
| P-521 base | **12.1** / **12.3** | – | 122 / 139 | – | – | – | – | 28.0 / 51.0 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.9 / 37.8 | – | – | 2.8 / 7.0 | – | 6.9 / 17.0 | – |
| BabyBear | **3.4** / **11.0** | – | 16.9 / 37.8 | – | – | 4.7 / 13.0 | – | 6.9 / 17.0 | – |
| Goldilocks | 3.8 / **8.9** | – | 17.6 / 34.9 | – | – | 3.2\* / 9.9 | **3.7** / 10.9 | 6.9 / 17.0 | – |
| BN254 scalar | **69.2** / **72.5** | 74.7 / 104 | 119 / 156 | – | 92.2 / 93.8 | – | – | 94.0 / 130 | – |
| secp256k1 base | **44.5** / **54.8** | 89.9 / 115 | 119 / 156 | 85.0 / 88.4 | 87.9 / 96.0 | – | – | 93.8 / 129 | – |
| P-256 base | **49.7** / 60.0 | 66.8 / 87.7 | 119 / 152 | 68.1 / 72.8 | 68.0 / 72.8 | – | – | 51.0 / **57.9** | 569 / – |
| P-256 scalar | **74.9** / **83.8** | 92.0 / 113 | 119 / 160 | 142 / 141 | 94.5 / 100 | – | – | 94.0 / 129 | 635 / – |
| Tom-256 base | **70.9** / **76.0** | – | 119 / 155 | – | 91.1 / 94.9 | – | – | 94.1 / 129 | – |
| BLS12-381 base | **139** / **141** | 165 / 195 | 272 / 294 | – | 189 / 195 | – | – | 256 / 272 | – |
| P-521 base | **177** / **192** | – | 530 / 548 | – | – | – | – | 466 / 475 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.7 / 34.0 | – | – | 2.7 / 7.0 | – | 7.5 / 17.0 | – |
| BabyBear | **3.7** / **11.0** | – | 16.7 / 34.1 | – | – | 4.8 / 13.0 | – | 7.5 / 17.2 | – |
| Goldilocks | 3.8 / **9.0** | – | 17.5 / 36.4 | – | – | 3.2\* / 10.0 | **3.7** / 11.0 | 7.5 / 17.0 | – |
| BN254 scalar | **63.7** / **66.7** | 74.6 / 106 | 117 / 152 | – | 74.2 / 89.4 | – | – | 93.5 / 129 | – |
| secp256k1 base | **33.9** / **42.9** | 84.9 / 102 | 116 / 152 | 57.9 / 79.9 | 75.9 / 84.9 | – | – | 93.5 / 130 | – |
| P-256 base | **40.3** / **43.9** | 65.2 / 82.1 | 117 / 152 | 60.9 / 71.9 | 54.9 / 58.0 | – | – | 50.6 / 57.9 | 566 / – |
| P-256 scalar | **63.9** / **72.9** | 90.4 / 109 | 117 / 152 | 134 / 147 | 77.5 / 93.5 | – | – | 93.5 / 129 | 638 / – |
| Tom-256 base | **57.0** / **64.4** | – | 117 / 152 | – | 72.0 / 81.0 | – | – | 93.5 / 130 | – |
| BLS12-381 base | **133** / **137** | 141 / 167 | 269 / 291 | – | 152 / 166 | – | – | 255 / 277 | – |
| P-521 base | **121** / **134** | – | 537 / 566 | – | – | – | – | 467 / 473 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **222** / – | – | 1,446 / – | – | – | 633 / – | – | 892 / – | – |
| BabyBear | **221** / – | – | 1,410 / – | – | – | 624 / – | – | 1,066 / – | – |
| Goldilocks | **436** / – | – | 2,867 / – | – | – | 1,298 / – | 691 / – | 1,744 / – | – |
| BN254 scalar | **5,068** / – | 16,507 / – | 14,356 / – | – | 7,184 / – | – | – | 14,178 / – | – |
| secp256k1 base | **4,927** / – | 15,630 / – | 14,430 / – | 24,727 / – | 7,199 / – | – | – | 14,295 / – | – |
| P-256 base | **4,863** / – | 15,359 / – | 14,452 / – | 30,751 / – | 7,199 / – | – | – | 14,877 / – | 89,381 / – |
| P-256 scalar | **5,044** / – | 16,618 / – | 14,408 / – | 30,114 / – | 7,230 / – | – | – | 14,330 / – | 89,576 / – |
| Tom-256 base | **5,049** / – | – | 14,473 / – | – | 7,217 / – | – | – | 14,271 / – | – |
| BLS12-381 base | **8,923** / – | 36,612 / – | 25,399 / – | – | 12,154 / – | – | – | 35,067 / – | – |
| P-521 base | **14,225** / – | – | 44,298 / – | – | – | – | – | 71,983 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **135** / – | – | 1,653 / – | – | – | 143 / – | 446 / – | – |
| BabyBear | **1,993** / – | – | 18,706 / – | – | – | 2,784 / – | – | – |
| Goldilocks | **2,858** / – | – | 26,968 / – | – | – | 3,196 / – | – | – |
| BN254 scalar | **34,470** / – | 45,055 / – | 133,575 / – | – | 84,892 / – | – | – | – |
| secp256k1 base | **12,320** / – | 43,387 / – | 52,897 / – | 24,433 / – | 47,483 / – | – | 71,194 / – | – |
| P-256 base | **11,927** / – | 18,871 / – | 43,337 / – | 30,427 / – | 33,661 / – | – | 29,408 / – | 168,765 / – |
| P-256 scalar | **24,197** / – | 39,592 / – | 52,075 / – | 82,032 / – | 42,078 / – | – | – | – |
| Tom-256 base | **19,820** / – | – | 49,058 / – | – | 44,136 / – | – | 50,914 / – | – |
| BLS12-381 base | **66,892** / – | 90,388 / – | 134,717 / – | – | 141,084 / – | – | 179,388 / – | – |
| P-521 base | **68,697** / – | – | 265,815 / – | – | – | – | 260,548 / – | – |

### mul_u64: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **9.5** / **9.2** | – | 41.8 / 37.1 | – | – | 7.5\* / 7.0\* | – | – |
| BabyBear | **9.0** / **10.9** | – | 41.0 / 38.4 | – | – | 12.6 / 14.9 | – | – |
| Goldilocks | **7.1** / **10.1** | – | 49.4 / 45.3 | – | – | 3.2\* / 9.9\* | 8.2 / 11.0 | – |
| BN254 scalar | **20.9** / **25.9** | 241 / 244 | 336 / 286 | – | 273 / 286 | – | – | 259 / 253 |
| secp256k1 base | **10.1** / **14.9** | 233 / 235 | 313 / 296 | 96.0 / 105 | 260 / 260 | – | – | 291 / 289 |
| P-256 base | **23.0** / **25.8** | 208 / 211 | 328 / 330 | 165 / 160 | 230 / 209 | – | – | 162 / 156 |
| P-256 scalar | **25.9** / **28.4** | 265 / 263 | 327 / 325 | 163 / 170 | 298 / 303 | – | – | 284 / 292 |
| Tom-256 base | **23.8** / **28.6** | – | 335 / 329 | – | 270 / 253 | – | – | 295 / 293 |
| BLS12-381 base | **31.5** / **34.5** | 444 / 429 | 639 / 608 | – | 464 / 480 | – | – | 635 / 612 |
| P-521 base | **19.8** / **33.7** | – | 997 / 947 | – | – | – | – | 1,230 / 1,117 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### mul_uint: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.8** / **8.9** | 36.6 / 40.6 | – | – | 5.5\* / 7.0\* | – |
| BabyBear | **9.0** / **11.0** | 37.5 / 36.5 | – | – | 12.7 / 14.9 | – |
| Goldilocks | **7.1** / **10.0** | 45.0 / 40.3 | – | – | 3.2\* / 9.9\* | 7.4 / 11.0 |
| BN254 scalar | **115** / **122** | 312 / 300 | – | 211 / 191 | – | – |
| secp256k1 base | **64.8** / **68.6** | 338 / 325 | – | 167 / 163 | – | – |
| P-256 base | **105** / **117** | 297 / 298 | – | 156 / 148 | – | – |
| P-256 scalar | **134** / **117** | 320 / 292 | 196 / 202 | 199 / 189 | – | – |
| Tom-256 base | **124** / **127** | 293 / 301 | – | 191 / 178 | – | – |
| BLS12-381 base | **215** / **251** | 639 / 626 | – | 396 / 406 | – | – |
| P-521 base | **361** / **411** | 1,121 / 1,039 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_u64: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **7.3** | – | 33.8 / 33.0 | – | – | 5.6\* / 5.4\* | – | – |
| BabyBear | **8.9** / 8.2 | – | 36.6 / 33.2 | – | – | 9.1 / **8.0** | – | – |
| Goldilocks | **5.8** / 6.3 | – | 40.1 / 35.7 | – | – | 2.0\* / 3.0\* | 6.1 / **6.1** | – |
| BN254 scalar | **32.2** / **29.1** | 134 / 134 | 238 / 242 | – | 170 / 166 | – | – | 162 / 159 |
| secp256k1 base | **6.6** / **7.6** | 107 / 108 | 252 / 246 | 3.7\* / 2.3\* | 174 / 152 | – | – | 161 / 122 |
| P-256 base | **27.8** / **25.6** | 116 / 112 | 233 / 237 | 76.6 / 91.0 | 130 / 136 | – | – | 68.3 / 70.9 |
| P-256 scalar | 25.5 / 26.3 | 140 / 137 | 250 / 240 | **14.3** / **20.7** | 192 / 172 | – | – | 165 / 154 |
| Tom-256 base | **30.3** / **29.8** | – | 239 / 235 | – | 181 / 169 | – | – | 150 / 123 |
| BLS12-381 base | **46.2** / **39.2** | 241 / 231 | 423 / 418 | – | 325 / 291 | – | – | 353 / 342 |
| P-521 base | **11.5** / **12.8** | – | 663 / 632 | – | – | – | – | 577 / 579 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.1** / **5.5** | 33.0 / 30.3 | – | – | 4.1\* / 5.4\* | – |
| BabyBear | **8.3** / **7.9** | 32.7 / 31.4 | – | – | 9.1 / **7.9** | – |
| Goldilocks | **6.3** / **5.6** | 42.5 / 38.4 | – | – | 2.0\* / 3.0\* | 7.9 / 6.2 |
| BN254 scalar | 121 / 130 | 257 / 239 | – | **117** / **124** | – | – |
| secp256k1 base | **87.2** / **78.6** | 251 / 250 | – | 92.3 / 99.5 | – | – |
| P-256 base | 123 / 123 | 237 / 244 | – | **88.3** / **85.0** | – | – |
| P-256 scalar | 138 / 129 | 253 / 257 | **50.5** / **51.7** | 122 / 127 | – | – |
| Tom-256 base | 126 / 119 | 242 / 236 | – | **103** / **111** | – | – |
| BLS12-381 base | 273 / 231 | 400 / 405 | – | **210** / **220** | – | – |
| P-521 base | **380** / **404** | 649 / 610 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_u64: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **6.5** / **5.8** | – | 16.5 / 18.1 | – | – | 4.6\* / 4.8\* | – | – |
| BabyBear | **7.8** / **8.0** | – | 17.6 / 18.4 | – | – | 8.9 / 8.9 | – | – |
| Goldilocks | **4.7** / **4.4** | – | 27.0 / 22.3 | – | – | 2.0\* / 3.0\* | 4.9 / 5.9 | – |
| BN254 scalar | **25.4** / **24.8** | 127 / 124 | 195 / 190 | – | 156 / 150 | – | – | 113 / 111 |
| secp256k1 base | **4.2** / **8.0** | 101 / 104 | 194 / 185 | 4.3 / 3.0\* | 130 / 129 | – | – | 129 / 136 |
| P-256 base | **26.7** / **23.6** | 106 / 101 | 182 / 189 | 81.7 / 85.3 | 123 / 118 | – | – | 62.6 / 64.7 |
| P-256 scalar | 25.3 / 25.9 | 138 / 129 | 190 / 184 | **6.7** / **12.4** | 156 / 161 | – | – | 136 / 128 |
| Tom-256 base | **25.0** / **25.1** | – | 192 / 218 | – | 167 / 157 | – | – | 152 / 138 |
| BLS12-381 base | **38.5** / **45.7** | 242 / 240 | 420 / 372 | – | 265 / 258 | – | – | 347 / 342 |
| P-521 base | **9.5** / **16.9** | – | 648 / 623 | – | – | – | – | 632 / 567 |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.7** / **5.5** | 16.6 / 19.1 | – | – | 4.6\* / 5.3\* | – |
| BabyBear | **7.4** / **7.0** | 18.5 / 19.6 | – | – | 9.0 / 8.9 | – |
| Goldilocks | 5.1 / **4.4** | 26.9 / 23.2 | – | – | 2.0\* / 3.0\* | **4.9** / 5.7 |
| BN254 scalar | 134 / 122 | 216 / 198 | – | **114** / **115** | – | – |
| secp256k1 base | **82.1** / **81.2** | 206 / 195 | – | 93.7 / 90.7 | – | – |
| P-256 base | 112 / 118 | 204 / 201 | – | **87.0** / **81.9** | – | – |
| P-256 scalar | 125 / 128 | 212 / 212 | **47.9** / **47.0** | 117 / 115 | – | – |
| Tom-256 base | 121 / 121 | 215 / 214 | – | **110** / **102** | – | – |
| BLS12-381 base | 233 / 231 | 398 / 381 | – | **219** / **225** | – | – |
| P-521 base | **387** / **381** | 687 / 697 | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### from_u64: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 5.9 / – | – | 14.6 / – | – | – | **4.3** / – | – | – |
| BabyBear | **7.4** / – | – | 15.3 / – | – | – | 8.0 / – | – | – |
| Goldilocks | 4.5 / – | – | 18.7 / – | – | – | 1.9\* / – | **3.6** / – | – |
| BN254 scalar | **25.5** / – | 136 / – | 154 / – | – | 151 / – | – | – | 140 / – |
| secp256k1 base | **2.4** / – | 121 / – | 128 / – | 3.2 / – | 140 / – | – | – | 106 / – |
| P-256 base | **20.6** / – | 117 / – | 156 / – | 67.3 / – | 143 / – | – | – | 61.4 / – |
| P-256 scalar | 21.6 / – | 141 / – | 149 / – | **2.3** / – | 178 / – | – | – | 141 / – |
| Tom-256 base | **22.6** / – | – | 137 / – | – | 150 / – | – | – | 107 / – |
| BLS12-381 base | **34.0** / – | 251 / – | 323 / – | – | 269 / – | – | – | 354 / – |
| P-521 base | **5.1** / – | – | 539 / – | – | – | – | – | 575 / – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **2.9** | 9.7 / 18.3 | – | – | – | 2.6 / 4.9 |
| BabyBear | **1.9** / **2.9** | 9.7 / 18.9 | – | – | – | 2.6 / 4.9 |
| Goldilocks | **2.0** / **2.9** | 9.8 / 19.4 | – | – | 2.4 / **2.9** | 2.5 / 4.9 |
| BN254 scalar | **5.0** / **6.8** | 66.6 / 93.5 | – | 12.8 / 21.7 | – | 6.8 / 10.7 |
| secp256k1 base | **5.8** / **6.8** | 66.6 / 94.0 | 4.4\* / 3.5\* | 14.4 / 21.2 | – | 6.8 / 10.7 |
| P-256 base | **5.8** / **6.8** | 66.8 / 93.1 | 10.6 / 18.5 | 10.7 / 20.7 | – | 6.9 / 10.6 |
| P-256 scalar | **5.9** / **6.7** | 66.4 / 92.6 | 17.4 / 22.7 | 14.5 / 21.0 | – | 6.8 / 10.7 |
| Tom-256 base | **5.8** / **6.8** | 66.6 / 93.4 | – | 11.7 / 20.6 | – | 6.9 / 10.7 |
| BLS12-381 base | **9.7** / **8.7** | 87.6 / 113 | – | 20.3 / 27.3 | – | 16.8 / 12.6 |
| P-521 base | **12.2** / **14.9** | 125 / 155 | – | – | – | 25.1 / 52.8 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **1.9** / **2.9** | 4.2 / 12.0 | – | – | 2.0 / 3.0 | – | **1.9** / 3.9 |
| BabyBear | **1.9** / **3.0** | 4.3 / 12.0 | – | – | – | – | 2.0 / 4.0 |
| Goldilocks | **1.9** / 3.0 | 4.2 / 12.6 | – | – | – | **1.9** / **2.9** | 2.0 / 4.0 |
| BN254 scalar | **5.1** / **6.8** | 49.8 / 51.2 | – | 8.3 / 11.7 | – | – | 6.9 / 7.8 |
| secp256k1 base | **5.3** / 9.8 | 49.9 / 48.2 | 4.6\* / 4.1\* | 7.5 / 12.8 | – | – | 6.8 / **7.8** |
| P-256 base | **5.4** / **6.7** | 49.9 / 43.6 | 7.9 / 12.8 | 7.5 / 11.8 | – | – | 6.8 / 7.8 |
| P-256 scalar | **5.2** / **6.8** | 50.0 / 48.7 | 8.3 / 12.8 | 7.9 / 12.8 | – | – | 6.8 / 7.8 |
| Tom-256 base | **5.4** / 8.0 | 49.9 / 55.1 | – | 8.3 / 11.8 | – | – | 6.8 / **7.8** |
| BLS12-381 base | **9.7** / **9.7** | 83.1 / 88.1 | – | 16.6 / 19.7 | – | – | 16.8 / 11.1 |
| P-521 base | **11.9** / **12.3** | 122 / 139 | – | – | – | – | 28.0 / 51.0 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **6.9** | 16.9 / 37.8 | – | – | – | 6.9 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.9 / 37.8 | – | – | – | 6.9 / 17.0 |
| Goldilocks | 3.8 / **8.9** | 17.6 / 34.9 | – | – | **3.7** / 10.9 | 6.9 / 17.0 |
| BN254 scalar | **69.3** / **78.9** | 119 / 156 | – | 92.2 / 93.8 | – | 94.0 / 130 |
| secp256k1 base | **44.7** / **54.8** | 119 / 156 | 85.0 / 88.4 | 87.9 / 96.0 | – | 93.8 / 129 |
| P-256 base | **50.1** / 59.8 | 119 / 152 | 68.1 / 72.8 | 68.0 / 72.8 | – | 51.0 / **57.9** |
| P-256 scalar | **74.9** / **83.9** | 119 / 160 | 142 / 141 | 94.5 / 100 | – | 94.0 / 129 |
| Tom-256 base | **70.8** / **75.9** | 119 / 155 | – | 91.1 / 94.9 | – | 94.1 / 129 |
| BLS12-381 base | **144** / **154** | 272 / 294 | – | 189 / 195 | – | 256 / 272 |
| P-521 base | **177** / **192** | 530 / 548 | – | – | – | 466 / 475 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.7 / 34.0 | – | – | – | 7.5 / 17.0 |
| BabyBear | **3.5** / **11.0** | 16.7 / 34.1 | – | – | – | 7.5 / 17.2 |
| Goldilocks | 3.8 / **9.0** | 17.5 / 36.4 | – | – | **3.7** / 11.0 | 7.5 / 17.0 |
| BN254 scalar | **64.9** / **66.9** | 117 / 152 | – | 74.2 / 89.4 | – | 93.5 / 129 |
| secp256k1 base | **33.9** / **43.0** | 116 / 152 | 57.9 / 79.9 | 75.9 / 84.9 | – | 93.5 / 130 |
| P-256 base | **40.2** / **43.9** | 117 / 152 | 60.9 / 71.9 | 54.9 / 58.0 | – | 50.6 / 57.9 |
| P-256 scalar | **63.9** / **72.9** | 117 / 152 | 134 / 147 | 77.5 / 93.5 | – | 93.5 / 129 |
| Tom-256 base | **56.9** / **64.4** | 117 / 152 | – | 72.0 / 81.0 | – | 93.5 / 130 |
| BLS12-381 base | **138** / **146** | 269 / 291 | – | 152 / 166 | – | 255 / 277 |
| P-521 base | **121** / **134** | 537 / 566 | – | – | – | 467 / 473 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **253** / – | 3,871 / – | – | – | – | 953 / – |
| BabyBear | **384** / – | 3,847 / – | – | – | – | 948 / – |
| Goldilocks | **647** / – | 3,847 / – | – | – | 691 / – | 2,069 / – |
| BN254 scalar | **6,764** / – | 19,590 / – | – | 46,625 / – | – | 54,198 / – |
| secp256k1 base | **6,871** / – | 19,694 / – | 24,727 / – | 46,610 / – | – | 72,083 / – |
| P-256 base | **6,534** / – | 19,531 / – | 30,751 / – | 33,572 / – | – | 34,561 / – |
| P-256 scalar | **6,942** / – | 19,617 / – | 78,798 / – | 49,048 / – | – | 59,444 / – |
| Tom-256 base | **7,033** / – | 19,644 / – | – | 44,829 / – | – | 51,349 / – |
| BLS12-381 base | **12,470** / – | 34,529 / – | – | 143,260 / – | – | 179,657 / – |
| P-521 base | **20,046** / – | 65,283 / – | – | – | – | 497,732 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **142** / – | 1,653 / – | – | – | 446 / – |
| BabyBear | **5,597** / – | 18,706 / – | – | – | – |
| Goldilocks | **7,153** / – | 26,968 / – | – | – | – |
| BN254 scalar | **58,590** / – | 133,575 / – | – | 84,892 / – | – |
| secp256k1 base | **12,283** / – | 52,897 / – | 24,433 / – | 47,483 / – | 71,194 / – |
| P-256 base | **11,996** / – | 43,337 / – | 30,427 / – | 33,661 / – | 29,408 / – |
| P-256 scalar | **25,452** / – | 52,075 / – | 82,032 / – | 42,078 / – | – |
| Tom-256 base | **20,054** / – | 49,058 / – | – | 44,136 / – | 50,914 / – |
| BLS12-381 base | **69,848** / – | 134,717 / – | – | 141,084 / – | 179,388 / – |
| P-521 base | **69,179** / – | 265,815 / – | – | – | 260,548 / – |

### mul_u64, constant time: a × k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **8.2** / **8.9** | 41.8 / 37.1 | – | – | – |
| BabyBear | **10.0** / **11.1** | 41.0 / 38.4 | – | – | – |
| Goldilocks | **8.0** / **9.9** | 49.4 / 45.3 | – | – | 8.2 / 11.0 |
| BN254 scalar | **27.1** / **30.4** | 336 / 286 | – | 273 / 286 | – |
| secp256k1 base | **12.4** / **19.9** | 313 / 296 | 96.0 / 105 | 260 / 260 | – |
| P-256 base | **25.4** / **32.9** | 328 / 330 | 165 / 160 | 230 / 209 | – |
| P-256 scalar | **28.3** / **33.9** | 327 / 325 | 163 / 170 | 298 / 303 | – |
| Tom-256 base | **25.3** / **34.0** | 335 / 329 | – | 270 / 253 | – |
| BLS12-381 base | **35.0** / **42.2** | 639 / 608 | – | 464 / 480 | – |
| P-521 base | **26.5** / **45.7** | 997 / 947 | – | – | – |

### mul_uint, constant time: a × k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **7.0** / **9.0** | 36.6 / 40.6 | – | – | – |
| BabyBear | **8.9** / **11.0** | 37.5 / 36.5 | – | – | – |
| Goldilocks | 8.4 / **10.2** | 45.0 / 40.3 | – | – | **7.4** / 11.0 |
| BN254 scalar | **139** / **146** | 312 / 300 | – | 211 / 191 | – |
| secp256k1 base | **91.9** / **96.6** | 338 / 325 | – | 167 / 163 | – |
| P-256 base | **126** / **144** | 297 / 298 | – | 156 / 148 | – |
| P-256 scalar | **143** / **150** | 320 / 292 | 196 / 202 | 199 / 189 | – |
| Tom-256 base | **138** / **144** | 293 / 301 | – | 191 / 178 | – |
| BLS12-381 base | **279** / **300** | 639 / 626 | – | 396 / 406 | – |
| P-521 base | **427** / **482** | 1,121 / 1,039 | – | – | – |

### add_u64, constant time: a + k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.5** / **7.9** | 33.8 / 33.0 | – | – | – |
| BabyBear | **6.6** / **7.6** | 36.6 / 33.2 | – | – | – |
| Goldilocks | **5.2** / **5.4** | 40.1 / 35.7 | – | – | 6.1 / 6.1 |
| BN254 scalar | **27.7** / **30.8** | 238 / 242 | – | 170 / 166 | – |
| secp256k1 base | **6.7** / **7.4** | 252 / 246 | 3.7\* / 2.3\* | 174 / 152 | – |
| P-256 base | **26.0** / **27.6** | 233 / 237 | 76.6 / 91.0 | 130 / 136 | – |
| P-256 scalar | 29.3 / 30.1 | 250 / 240 | **14.3** / **20.7** | 192 / 172 | – |
| Tom-256 base | **31.8** / **31.5** | 239 / 235 | – | 181 / 169 | – |
| BLS12-381 base | **47.8** / **50.3** | 423 / 418 | – | 325 / 291 | – |
| P-521 base | **14.0** / **18.0** | 663 / 632 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### add_uint, constant time: a + k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **7.0** | 33.0 / 30.3 | – | – | – |
| BabyBear | **8.1** / **8.1** | 32.7 / 31.4 | – | – | – |
| Goldilocks | **5.7** / **5.7** | 42.5 / 38.4 | – | – | 7.9 / 6.2 |
| BN254 scalar | 147 / 140 | 257 / 239 | – | **117** / **124** | – |
| secp256k1 base | 98.8 / **93.6** | 251 / 250 | – | **92.3** / 99.5 | – |
| P-256 base | 148 / 143 | 237 / 244 | – | **88.3** / **85.0** | – |
| P-256 scalar | 155 / 153 | 253 / 257 | **50.5** / **51.7** | 122 / 127 | – |
| Tom-256 base | 143 / 139 | 242 / 236 | – | **103** / **111** | – |
| BLS12-381 base | 273 / 281 | 400 / 405 | – | **210** / **220** | – |
| P-521 base | **471** / **466** | 649 / 610 | – | – | – |

### sub_u64, constant time: a − k, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **7.5** | 16.5 / 18.1 | – | – | – |
| BabyBear | **8.3** / **7.1** | 17.6 / 18.4 | – | – | – |
| Goldilocks | **4.9** / **4.6** | 27.0 / 22.3 | – | – | **4.9** / 5.9 |
| BN254 scalar | **27.9** / **30.3** | 195 / 190 | – | 156 / 150 | – |
| secp256k1 base | **4.2** / **10.5** | 194 / 185 | 4.3 / 3.0\* | 130 / 129 | – |
| P-256 base | **24.0** / **22.9** | 182 / 189 | 81.7 / 85.3 | 123 / 118 | – |
| P-256 scalar | 26.5 / 28.1 | 190 / 184 | **6.7** / **12.4** | 156 / 161 | – |
| Tom-256 base | **28.3** / **29.6** | 192 / 218 | – | 167 / 157 | – |
| BLS12-381 base | **42.9** / **47.1** | 420 / 372 | – | 265 / 258 | – |
| P-521 base | **9.5** / **16.0** | 648 / 623 | – | – | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_uint, constant time: a − k, k of the bit length of p

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **5.6** / **5.8** | 16.6 / 19.1 | – | – | – |
| BabyBear | **7.7** / **7.0** | 18.5 / 19.6 | – | – | – |
| Goldilocks | 5.1 / 5.8 | 26.9 / 23.2 | – | – | **4.9** / **5.7** |
| BN254 scalar | 144 / 141 | 216 / 198 | – | **114** / **115** | – |
| secp256k1 base | 98.2 / 95.1 | 206 / 195 | – | **93.7** / **90.7** | – |
| P-256 base | 146 / 134 | 204 / 201 | – | **87.0** / **81.9** | – |
| P-256 scalar | 157 / 159 | 212 / 212 | **47.9** / **47.0** | 117 / 115 | – |
| Tom-256 base | 152 / 144 | 215 / 214 | – | **110** / **102** | – |
| BLS12-381 base | 286 / 289 | 398 / 381 | – | **219** / **225** | – |
| P-521 base | **468** / **476** | 687 / 697 | – | – | – |

### from_u64, constant time: k mod p, k of 64 bits

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | crypto-bigint | k256/p256 | halo2curves | winter-math |
|---|---:|---:|---:|---:|
| Mersenne-31 | **14.6** / – | – | – | – |
| BabyBear | **15.3** / – | – | – | – |
| Goldilocks | 18.7 / – | – | – | **3.6** / – |
| BN254 scalar | 154 / – | – | **151** / – | – |
| secp256k1 base | 128 / – | **3.2** / – | 140 / – | – |
| P-256 base | 156 / – | **67.3** / – | 143 / – | – |
| P-256 scalar | 149 / – | **2.3** / – | 178 / – | – |
| Tom-256 base | **137** / – | – | 150 / – | – |
| BLS12-381 base | 323 / – | – | **269** / – | – |
| P-521 base | **539** / – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
