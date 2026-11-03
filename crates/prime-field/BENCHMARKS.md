# Benchmarks

`prime-field` at commit `35cb27c` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`, in cycles per operation, measured on 2026-11-03.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1 |
| Profile | `bench`: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1` |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1 -C llvm-args=-align-loops=64` |
| Load | CPU 15; 1-minute load 23.27 before, 23.66 after; its SMT sibling, CPU 31, 46% busy |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

- Each cell: throughput / latency, in user-mode cycles per operation.
- Throughput: independent operations on fresh operands.
- Latency: the chain `x = op(x, y)`, with `x` in registers or in memory, whichever is faster.
- Each figure: the median over 31 repetitions of the cycles of 2n operations less those of n, over n.
- A sweep ran at a 1-minute load of 2 or more: compare figures within a table only.
- Operands: canonical residues, the same for every crate.
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

- `ark-ff`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base.
- `k256/p256`: over secp256k1 base, P-256 base and P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31, BabyBear and Goldilocks.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base.
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

- `k256/p256`: over secp256k1 base, P-256 base and P-256 scalar.
- `halo2curves`: over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, Tom-256 base and BLS12-381 base.
- `plonky3`: over Mersenne-31.
- `winter-math`: over Goldilocks.
- `runtime-algebra`: sqrt over Mersenne-31, secp256k1 base, P-256 base, Tom-256 base, BLS12-381 base and P-521 base.

## Results

### add: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 3.2 / **3.1** | – | 9.7 / 18.2 | – | – | 2.0\* / 3.0\* | – | **2.6** / 5.0 | – |
| BabyBear | **2.0** / **3.0** | – | 9.7 / 18.5 | – | – | **2.0** / **3.0** | – | 2.6 / 5.0 | – |
| Goldilocks | **2.1** / **3.0** | – | 9.9 / 19.5 | – | – | 2.0\* / 3.0 | 2.5 / **3.0** | 2.7 / 5.0 | – |
| BN254 scalar | **5.3** / **7.0** | 18.0 / 12.3 | 71.5 / 85.3 | – | 13.2 / 22.0 | – | – | 7.1 / 11.0 | – |
| secp256k1 base | **6.1** / **7.0** | 17.8 / 13.5 | 66.9 / 91.3 | 4.5\* / 3.8\* | 14.6 / 21.4 | – | – | 7.1 / 11.0 | – |
| P-256 base | **6.1** / **7.0** | 19.3 / 14.0 | 66.8 / 92.4 | 15.1 / 18.7 | 10.9 / 21.0 | – | – | 7.1 / 11.0 | 353 / – |
| P-256 scalar | **6.2** / **7.0** | 18.6 / 13.7 | 70.0 / 93.1 | 17.8 / 23.0 | 14.7 / 21.0 | – | – | 7.1 / 11.0 | 355 / – |
| Tom-256 base | **6.2** / **7.0** | – | 69.5 / 92.8 | – | 12.6 / 21.1 | – | – | 7.1 / 11.0 | – |
| BLS12-381 base | **10.0** / **9.0** | 20.5 / 15.0 | 88.1 / 113 | – | 20.7 / 27.4 | – | – | 17.1 / 12.9 | – |
| P-521 base | **12.9** / **15.4** | – | 118 / 155 | – | – | – | – | 25.4 / 52.8 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | – | 4.3 / 12.4 | – | – | 2.0 / 3.0 | – | **2.0** / 4.0 | – |
| BabyBear | **2.0** / **3.0** | – | 5.0 / 12.4 | – | – | **2.0** / **3.0** | – | 2.2 / 4.0 | – |
| Goldilocks | **2.0** / **3.0** | – | 4.3 / 12.7 | – | – | 2.0\* / 3.0\* | 2.1 / **3.0** | **2.0** / 4.0 | – |
| BN254 scalar | **5.3** / **7.0** | 17.1 / 13.9 | 50.2 / 51.2 | – | 8.6 / 12.0 | – | – | 7.0 / 8.0 | – |
| secp256k1 base | **5.7** / 8.1 | 17.5 / 13.7 | 54.6 / 48.5 | 4.0\* / 4.3\* | 9.0 / 13.0 | – | – | 7.3 / **8.0** | – |
| P-256 base | **5.7** / **7.3** | 16.9 / 12.8 | 52.4 / 43.7 | 8.1 / 13.0 | 7.8 / 12.0 | – | – | 7.1 / 8.0 | 200 / – |
| P-256 scalar | **6.4** / **7.0** | 16.3 / 13.8 | 50.0 / 49.2 | 8.4 / 13.1 | 8.2 / 13.0 | – | – | 7.0 / 8.0 | 163 / – |
| Tom-256 base | **6.4** / 8.3 | – | 52.0 / 55.5 | – | 9.5 / 12.0 | – | – | 7.0 / **8.0** | – |
| BLS12-381 base | **10.1** / **10.0** | 23.9 / 17.3 | 83.3 / 88.6 | – | 15.6 / 20.0 | – | – | 17.0 / 11.3 | – |
| P-521 base | **12.5** / **12.8** | – | 123 / 139 | – | – | – | – | 31.5 / 51.0 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.9 / 37.8 | – | – | 2.9 / 7.0 | – | 7.0 / 17.0 | – |
| BabyBear | **3.6** / **11.0** | – | 17.0 / 37.8 | – | – | 4.7 / 13.0 | – | 7.0 / 17.1 | – |
| Goldilocks | **3.8** / **9.0** | – | 17.7 / 34.9 | – | – | 3.3\* / 10.0 | **3.8** / 11.0 | 7.0 / 17.0 | – |
| BN254 scalar | **73.1** / **72.9** | 75.4 / 105 | 120 / 159 | – | 98.6 / 94.2 | – | – | 94.6 / 142 | – |
| secp256k1 base | **45.2** / **55.1** | 97.4 / 115 | 119 / 170 | 101 / 88.7 | 97.8 / 100 | – | – | 94.3 / 141 | – |
| P-256 base | 51.5 / **60.1** | 67.8 / 87.6 | 120 / 162 | 69.0 / 73.1 | 70.1 / 73.2 | – | – | **51.1** / 63.8 | 615 / – |
| P-256 scalar | **75.2** / **84.1** | 95.2 / 114 | 120 / 168 | 146 / 143 | 94.4 / 102 | – | – | 94.7 / 136 | 737 / – |
| Tom-256 base | **71.1** / **76.1** | – | 128 / 156 | – | 95.0 / 100 | – | – | 95.0 / 141 | – |
| BLS12-381 base | **139** / **142** | 167 / 195 | 301 / 298 | – | 192 / 197 | – | – | 263 / 277 | – |
| P-521 base | **192** / **196** | – | 546 / 551 | – | – | – | – | 540 / 503 | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | – | 16.8 / 34.3 | – | – | 2.8\* / 7.0\* | – | 7.7 / 17.1 | – |
| BabyBear | **3.8** / **11.0** | – | 16.8 / 34.5 | – | – | 4.9 / 13.0 | – | 8.1 / 17.2 | – |
| Goldilocks | 3.9 / **9.0** | – | 18.5 / 36.2 | – | – | 3.6\* / 10.0 | **3.8** / 11.0 | 7.6 / 17.2 | – |
| BN254 scalar | **64.1** / **67.1** | 79.1 / 106 | 123 / 155 | – | 83.4 / 90.6 | – | – | 130 / 132 | – |
| secp256k1 base | **37.3** / **43.9** | 87.8 / 104 | 124 / 156 | 59.3 / 80.4 | 80.4 / 85.2 | – | – | 110 / 142 | – |
| P-256 base | **42.6** / **44.2** | 71.6 / 82.0 | 125 / 157 | 61.3 / 72.0 | 55.1 / 63.5 | – | – | 54.5 / 59.4 | 640 / – |
| P-256 scalar | **64.4** / **73.0** | 91.1 / 112 | 137 / 157 | 129 / 147 | 77.5 / 94.2 | – | – | 93.7 / 137 | 702 / – |
| Tom-256 base | **58.9** / **67.1** | – | 122 / 158 | – | 73.7 / 86.1 | – | – | 107 / 131 | – |
| BLS12-381 base | **142** / **144** | 151 / 174 | 284 / 296 | – | 178 / 166 | – | – | 293 / 290 | – |
| P-521 base | **122** / **140** | – | 594 / 602 | – | – | – | – | 502 / 487 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **222** / – | – | 1,552 / – | – | – | 661 / – | – | 953 / – | – |
| BabyBear | **220** / – | – | 1,491 / – | – | – | 650 / – | – | 1,095 / – | – |
| Goldilocks | **441** / – | – | 3,101 / – | – | – | 1,368 / – | 701 / – | 1,792 / – | – |
| BN254 scalar | **5,122** / – | 16,793 / – | 14,783 / – | – | 7,453 / – | – | – | 15,762 / – | – |
| secp256k1 base | **5,061** / – | 16,292 / – | 15,293 / – | 24,902 / – | 7,298 / – | – | – | 14,911 / – | – |
| P-256 base | **4,899** / – | 15,853 / – | 14,696 / – | 30,315 / – | 7,427 / – | – | – | 14,544 / – | 114,779 / – |
| P-256 scalar | **5,265** / – | 17,874 / – | 15,146 / – | 32,674 / – | 7,573 / – | – | – | 15,433 / – | 101,820 / – |
| Tom-256 base | **5,148** / – | – | 15,035 / – | – | 7,761 / – | – | – | 15,618 / – | – |
| BLS12-381 base | **9,600** / – | 39,943 / – | 26,788 / – | – | 12,348 / – | – | – | 38,199 / – | – |
| P-521 base | **15,816** / – | – | 47,100 / – | – | – | – | – | 79,791 / – | – |

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **148** / – | – | 1,725 / – | – | – | **148** / – | 453 / – | – |
| BabyBear | **2,023** / – | – | 18,658 / – | – | – | 2,822 / – | – | – |
| Goldilocks | **2,877** / – | – | 26,944 / – | – | – | 3,259 / – | – | – |
| BN254 scalar | **35,165** / – | 46,049 / – | 136,969 / – | – | 88,414 / – | – | – | – |
| secp256k1 base | **12,310** / – | 44,343 / – | 53,090 / – | 27,160 / – | 47,640 / – | – | 71,205 / – | – |
| P-256 base | **12,142** / – | 19,099 / – | 44,489 / – | 29,606 / – | 34,900 / – | – | 29,589 / – | 168,721 / – |
| P-256 scalar | **24,548** / – | 40,213 / – | 52,303 / – | 82,587 / – | 42,230 / – | – | – | – |
| Tom-256 base | **19,950** / – | – | 49,974 / – | – | 46,703 / – | – | 53,796 / – | – |
| BLS12-381 base | **67,610** / – | 92,794 / – | 142,529 / – | – | 143,917 / – | – | 185,099 / – | – |
| P-521 base | **69,063** / – | – | 277,776 / – | – | – | – | 272,229 / – | – |

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 9.7 / 18.2 | – | – | – | 2.6 / 5.0 |
| BabyBear | **2.0** / **3.0** | 9.7 / 18.5 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **2.1** / **3.0** | 9.9 / 19.5 | – | – | 2.5 / **3.0** | 2.7 / 5.0 |
| BN254 scalar | **5.3** / **7.0** | 71.5 / 85.3 | – | 13.2 / 22.0 | – | 7.1 / 11.0 |
| secp256k1 base | **6.1** / **7.0** | 66.9 / 91.3 | 4.5\* / 3.8\* | 14.6 / 21.4 | – | 7.1 / 11.0 |
| P-256 base | **6.1** / **7.0** | 66.8 / 92.4 | 15.1 / 18.7 | 10.9 / 21.0 | – | 7.1 / 11.0 |
| P-256 scalar | **6.1** / **7.0** | 70.0 / 93.1 | 17.8 / 23.0 | 14.7 / 21.0 | – | 7.1 / 11.0 |
| Tom-256 base | **6.2** / **7.0** | 69.5 / 92.8 | – | 12.6 / 21.1 | – | 7.1 / 11.0 |
| BLS12-381 base | **10.2** / **9.0** | 88.1 / 113 | – | 20.7 / 27.4 | – | 17.1 / 12.9 |
| P-521 base | **13.0** / **15.5** | 118 / 155 | – | – | – | 25.4 / 52.8 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 4.3 / 12.4 | – | – | 2.0 / 3.0 | – | **2.0** / 4.0 |
| BabyBear | **2.0** / **3.0** | 5.0 / 12.4 | – | – | – | – | 2.2 / 4.0 |
| Goldilocks | **2.0** / **3.0** | 4.3 / 12.7 | – | – | – | 2.1 / **3.0** | **2.0** / 4.0 |
| BN254 scalar | **5.2** / **7.0** | 50.2 / 51.2 | – | 8.6 / 12.0 | – | – | 7.0 / 8.0 |
| secp256k1 base | **5.7** / **8.0** | 54.6 / 48.5 | 4.0\* / 4.3\* | 9.0 / 13.0 | – | – | 7.3 / **8.0** |
| P-256 base | **5.7** / **7.9** | 52.4 / 43.7 | 8.1 / 13.0 | 7.8 / 12.0 | – | – | 7.1 / 8.0 |
| P-256 scalar | **5.4** / **7.0** | 50.0 / 49.2 | 8.4 / 13.1 | 8.2 / 13.0 | – | – | 7.0 / 8.0 |
| Tom-256 base | **5.6** / 8.3 | 52.0 / 55.5 | – | 9.5 / 12.0 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **10.1** / **10.0** | 83.3 / 88.6 | – | 15.6 / 20.0 | – | – | 17.0 / 11.3 |
| P-521 base | **12.4** / **13.0** | 123 / 139 | – | – | – | – | 31.5 / 51.0 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 37.8 | – | – | – | 7.0 / 17.0 |
| BabyBear | **3.6** / **11.0** | 17.0 / 37.8 | – | – | – | 7.0 / 17.1 |
| Goldilocks | **3.8** / **9.0** | 17.7 / 34.9 | – | – | **3.8** / 11.0 | 7.0 / 17.0 |
| BN254 scalar | **72.0** / **79.2** | 120 / 159 | – | 98.6 / 94.2 | – | 94.6 / 142 |
| secp256k1 base | **46.9** / **56.1** | 119 / 170 | 101 / 88.7 | 97.8 / 100 | – | 94.3 / 141 |
| P-256 base | 56.1 / **60.6** | 120 / 162 | 69.0 / 73.1 | 70.1 / 73.2 | – | **51.1** / 63.8 |
| P-256 scalar | **77.7** / **84.7** | 120 / 168 | 146 / 143 | 94.4 / 102 | – | 94.7 / 136 |
| Tom-256 base | **71.4** / **76.0** | 128 / 156 | – | 95.0 / 100 | – | 95.0 / 141 |
| BLS12-381 base | **144** / **159** | 301 / 298 | – | 192 / 197 | – | 263 / 277 |
| P-521 base | **178** / **192** | 546 / 551 | – | – | – | 540 / 503 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | 16.8 / 34.3 | – | – | – | 7.7 / 17.1 |
| BabyBear | **3.6** / **11.0** | 16.8 / 34.5 | – | – | – | 8.1 / 17.2 |
| Goldilocks | 3.9 / **9.0** | 18.5 / 36.2 | – | – | **3.8** / 11.0 | 7.6 / 17.2 |
| BN254 scalar | **66.1** / **69.2** | 123 / 155 | – | 83.4 / 90.6 | – | 130 / 132 |
| secp256k1 base | **34.1** / **43.0** | 124 / 156 | 59.3 / 80.4 | 80.4 / 85.2 | – | 110 / 142 |
| P-256 base | **40.4** / **44.0** | 125 / 157 | 61.3 / 72.0 | 55.1 / 63.5 | – | 54.5 / 59.4 |
| P-256 scalar | **64.8** / **73.8** | 137 / 157 | 129 / 147 | 77.5 / 94.2 | – | 93.7 / 137 |
| Tom-256 base | **58.9** / **65.9** | 122 / 158 | – | 73.7 / 86.1 | – | 107 / 131 |
| BLS12-381 base | **140** / **150** | 284 / 296 | – | 178 / 166 | – | 293 / 290 |
| P-521 base | **121** / **135** | 594 / 602 | – | – | – | 502 / 487 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **261** / – | 4,171 / – | – | – | – | 968 / – |
| BabyBear | **397** / – | 4,128 / – | – | – | – | 962 / – |
| Goldilocks | **656** / – | 3,978 / – | – | – | 701 / – | 2,095 / – |
| BN254 scalar | **6,898** / – | 20,097 / – | – | 47,725 / – | – | 58,200 / – |
| secp256k1 base | **7,117** / – | 20,580 / – | 24,902 / – | 47,255 / – | – | 71,358 / – |
| P-256 base | **6,829** / – | 20,066 / – | 30,315 / – | 34,850 / – | – | 34,573 / – |
| P-256 scalar | **7,003** / – | 20,432 / – | 79,488 / – | 49,457 / – | – | 61,969 / – |
| Tom-256 base | **7,134** / – | 20,034 / – | – | 45,128 / – | – | 51,819 / – |
| BLS12-381 base | **12,308** / – | 35,078 / – | – | 145,923 / – | – | 184,596 / – |
| P-521 base | **20,631** / – | 68,209 / – | – | – | – | 513,253 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **154** / – | 1,725 / – | – | – | 453 / – |
| BabyBear | **5,604** / – | 18,658 / – | – | – | – |
| Goldilocks | **7,235** / – | 26,944 / – | – | – | – |
| BN254 scalar | **59,631** / – | 136,969 / – | – | 88,414 / – | – |
| secp256k1 base | **12,342** / – | 53,090 / – | 27,160 / – | 47,640 / – | 71,205 / – |
| P-256 base | **12,216** / – | 44,489 / – | 29,606 / – | 34,900 / – | 29,589 / – |
| P-256 scalar | **25,764** / – | 52,303 / – | 82,587 / – | 42,230 / – | – |
| Tom-256 base | **20,168** / – | 49,974 / – | – | 46,703 / – | 53,796 / – |
| BLS12-381 base | **69,907** / – | 142,529 / – | – | 143,917 / – | 185,099 / – |
| P-521 base | **69,859** / – | 277,776 / – | – | – | 272,229 / – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
