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
| Load | CPU 15; 1-minute load 26.8 before, 21.21 after; its SMT sibling, CPU 31, 39% busy |
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
| add | `+` | `+` | `+`° | `+`° | `+`° | `+` | `+`° | `addf`° | `addf` |
| sub | `-` | `-` | `-`° | `-`° | `-`° | `-`°, `-` | `-`° | `subf`° | `subf` |
| mul | `*` | `*` | `*`° | `*`° | `*`° | `*` | `*`° | `mulf`° | `mulf` |
| square | `square` | `square` | `square`° | `square`° | `square`° | `square` | `square`° | `mulf(a, a)`° | `mulf(a, a)` |
| invert | `invert` | `inverse` | `invert_vartime` | `invert`°, `invert_vartime` | `invert` | `try_inverse` | `inv`° | `invert` | `invert` |
| sqrt | `sqrt` | `sqrt` | `sqrt`° | `sqrt`° | `sqrt`° | `try_sqrt` | – | `mulf to (p + 1) / 4`° | `mulf to (p + 1) / 4` |

° The crate's constant-time function, where it has no variable-time one.

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
| Mersenne-31 | **2.4** / **3.3** | – | 9.8° / 18.6° | – | – | 2.4\* / 3.0\* | – | 2.6° / 5.0° | – |
| BabyBear | **2.0** / **3.0** | – | 9.7° / 18.3° | – | – | **2.0** / 3.1 | – | 2.6° / 5.0° | – |
| Goldilocks | **2.1** / **3.0** | – | 9.9° / 19.7° | – | – | 2.0\* / 3.0 | 2.5° / **3.0°** | 2.7° / 5.0° | – |
| BN254 scalar | **5.2** / **7.0** | 17.6 / 12.8 | 67.3° / 94.3° | – | 13.0° / 22.0° | – | – | 7.0° / 11.0° | – |
| secp256k1 base | **6.1** / **7.0** | 18.0 / 13.3 | 66.7° / 93.3° | 5.8°\* / 5.1°\* | 14.6° / 21.3° | – | – | 7.1° / 11.0° | – |
| P-256 base | **6.1** / **7.0** | 19.0 / 13.8 | 66.7° / 92.7° | 10.9° / 18.8° | 10.9° / 21.0° | – | – | 7.1° / 11.0° | 352 / – |
| P-256 scalar | **6.3** / **7.0** | 18.4 / 13.9 | 66.7° / 94.0° | 17.7° / 23.0° | 14.6° / 21.0° | – | – | 7.1° / 11.0° | 356 / – |
| Tom-256 base | **6.2** / **7.0** | – | 66.7° / 93.2° | – | 12.1° / 20.7° | – | – | 7.1° / 11.0° | – |
| BLS12-381 base | **10.0** / **9.0** | 21.2 / 16.0 | 87.8° / 113° | – | 20.6° / 27.4° | – | – | 17.0° / 13.0° | – |
| P-521 base | **12.9** / **15.3** | – | 118° / 155° | – | – | – | – | 25.2° / 53.0° | – |

° The crate's constant-time function, where it has no variable-time one.

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | – | 4.3° / 12.7° | – | – | 2.0° / 3.0° | – | **2.0°** / 4.0° | – |
| BabyBear | **2.0** / **3.0** | – | 4.3° / 12.0° | – | – | 2.1 / 3.1 | – | **2.0°** / 4.0° | – |
| Goldilocks | **2.0** / **3.0** | – | 4.3° / 12.1° | – | – | 2.0 / 3.0\* | **2.0°** / **3.0°** | **2.0°** / 4.0° | – |
| BN254 scalar | **5.3** / **7.0** | 16.6 / 14.2 | 50.1° / 51.0° | – | 8.6° / 12.0° | – | – | 7.0° / 8.0° | – |
| secp256k1 base | **5.7** / 8.7 | 17.7 / 13.9 | 50.0° / 48.3° | 5.2°\* / 4.7°\* | 7.8° / 13.0° | – | – | 7.0° / **8.0°** | – |
| P-256 base | **5.6** / **8.0** | 17.1 / 13.1 | 50.1° / 43.7° | 8.1° / 13.0° | 7.8° / 12.0° | – | – | 7.0° / **8.0°** | 163 / – |
| P-256 scalar | **5.4** / **7.0** | 16.6 / 14.1 | 50.1° / 50.6° | 8.4° / 13.0° | 8.2° / 13.0° | – | – | 7.1° / 8.0° | 154 / – |
| Tom-256 base | **5.8** / 8.2 | – | 50.1° / 55.5° | – | 8.6° / 12.0° | – | – | 7.0° / **8.0°** | – |
| BLS12-381 base | **10.0** / **10.0** | 22.6 / 17.8 | 83.1° / 88.5° | – | 15.6° / 20.0° | – | – | 17.0° / 11.2° | – |
| P-521 base | **12.6** / **13.0** | – | 122° / 139° | – | – | – | – | 27.9° / 50.7° | – |

° The crate's constant-time function, where it has no variable-time one.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | – | 16.9° / 37.8° | – | – | 2.9 / 7.0 | – | 7.0° / 17.1° | – |
| BabyBear | **3.6** / **11.0** | – | 17.3° / 37.8° | – | – | 4.7 / 13.0 | – | 7.0° / 17.0° | – |
| Goldilocks | **3.8** / **9.0** | – | 17.7° / 34.9° | – | – | 3.3\* / 10.0 | **3.8°** / 11.0° | 7.0° / 17.0° | – |
| BN254 scalar | **69.3** / **72.8** | 75.1 / 104 | 121° / 158° | – | 95.8° / 94.2° | – | – | 94.7° / 130° | – |
| secp256k1 base | **44.7** / **55.0** | 90.9 / 115 | 118° / 155° | 85.1° / 88.6° | 97.7° / 96.0° | – | – | 94.7° / 130° | – |
| P-256 base | **50.1** / 60.0 | 68.6 / 87.4 | 119° / 162° | 68.9° / 73.7° | 71.2° / 73.0° | – | – | 51.5° / **58.1°** | 569 / – |
| P-256 scalar | **75.0** / **84.1** | 92.7 / 114 | 119° / 155° | 143° / 141° | 95.4° / 101° | – | – | 94.2° / 131° | 649 / – |
| Tom-256 base | **71.2** / **76.3** | – | 119° / 156° | – | 91.9° / 95.1° | – | – | 94.0° / 130° | – |
| BLS12-381 base | **140** / **141** | 188 / 195 | 272° / 296° | – | 190° / 196° | – | – | 256° / 273° | – |
| P-521 base | **178** / **199** | – | 536° / 549° | – | – | – | – | 466° / 481° | – |

° The crate's constant-time function, where it has no variable-time one.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | – | 16.8° / 34.0° | – | – | 2.8\* / 7.0\* | – | 7.6° / 17.2° | – |
| BabyBear | **3.6** / **11.0** | – | 16.8° / 34.4° | – | – | 5.1 / 13.0 | – | 7.6° / 17.3° | – |
| Goldilocks | **3.8** / **9.0** | – | 17.7° / 36.2° | – | – | 3.3\* / 10.0 | **3.8°** / 11.0° | 7.6° / 17.2° | – |
| BN254 scalar | **63.7** / **67.1** | 75.4 / 106 | 120° / 154° | – | 74.6° / 89.5° | – | – | 93.7° / 132° | – |
| secp256k1 base | **34.1** / **43.0** | 86.1 / 102 | 119° / 154° | 58.0° / 80.3° | 76.0° / 85.0° | – | – | 93.7° / 131° | – |
| P-256 base | **40.0** / **44.0** | 65.1 / 82.3 | 118° / 155° | 61.1° / 72.0° | 54.9° / 58.0° | – | – | 50.9° / 58.0° | 570 / – |
| P-256 scalar | **66.3** / **73.0** | 90.1 / 110 | 119° / 154° | 131° / 147° | 77.5° / 93.5° | – | – | 93.5° / 130° | 640 / – |
| Tom-256 base | **57.2** / **66.0** | – | 118° / 154° | – | 72.7° / 81.0° | – | – | 93.7° / 131° | – |
| BLS12-381 base | **134** / **138** | 141 / 167 | 270° / 294° | – | 152° / 166° | – | – | 255° / 273° | – |
| P-521 base | **121** / **136** | – | 543° / 566° | – | – | – | – | 468° / 473° | – |

° The crate's constant-time function, where it has no variable-time one.

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### invert: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **221** / – | – | 1,469 / – | – | – | 642 / – | – | 886 / – | – |
| BabyBear | **223** / – | – | 1,426 / – | – | – | 624 / – | – | 1,073 / – | – |
| Goldilocks | **440** / – | – | 2,885 / – | – | – | 1,308 / – | 692° / – | 1,725 / – | – |
| BN254 scalar | **4,992** / – | 16,532 / – | 14,445 / – | – | 7,178 / – | – | – | 14,197 / – | – |
| secp256k1 base | **4,976** / – | 15,741 / – | 14,556 / – | 24,886° / – | 7,230 / – | – | – | 14,284 / – | – |
| P-256 base | **4,845** / – | 15,471 / – | 14,533 / – | 31,189° / – | 7,263 / – | – | – | 14,158 / – | 89,807 / – |
| P-256 scalar | **5,026** / – | 16,574 / – | 14,482 / – | 30,048 / – | 7,255 / – | – | – | 14,256 / – | 92,323 / – |
| Tom-256 base | **5,018** / – | – | 14,525 / – | – | 7,240 / – | – | – | 14,343 / – | – |
| BLS12-381 base | **9,573** / – | 36,965 / – | 25,413 / – | – | 12,144 / – | – | – | 35,251 / – | – |
| P-521 base | **15,464** / – | – | 44,556 / – | – | – | – | – | 72,264 / – | – |

° The crate's constant-time function, where it has no variable-time one.

### sqrt: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **136** / – | – | 1,672° / – | – | – | 143 / – | 446° / – | – |
| BabyBear | **2,001** / – | – | 18,653° / – | – | – | 2,767 / – | – | – |
| Goldilocks | **2,856** / – | – | 26,949° / – | – | – | 3,214 / – | – | – |
| BN254 scalar | **34,395** / – | 45,219 / – | 133,870° / – | – | 85,142° / – | – | – | – |
| secp256k1 base | **12,312** / – | 43,519 / – | 53,053° / – | 26,959° / – | 47,477° / – | – | 68,798° / – | – |
| P-256 base | **11,928** / – | 18,782 / – | 43,563° / – | 30,310° / – | 33,662° / – | – | 31,921° / – | 165,439 / – |
| P-256 scalar | **24,190** / – | 39,451 / – | 52,451° / – | 81,303° / – | 42,128° / – | – | – | – |
| Tom-256 base | **19,730** / – | – | 48,990° / – | – | 44,671° / – | – | 52,057° / – | – |
| BLS12-381 base | **67,004** / – | 90,513 / – | 135,513° / – | – | 141,266° / – | – | 178,598° / – | – |
| P-521 base | **68,717** / – | – | 268,615° / – | – | – | – | 259,186° / – | – |

° The crate's constant-time function, where it has no variable-time one.

### add, constant time: a + b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 9.8 / 18.6 | – | – | – | 2.6 / 5.0 |
| BabyBear | **2.0** / **3.0** | 9.7 / 18.3 | – | – | – | 2.6 / 5.0 |
| Goldilocks | **2.1** / **3.0** | 9.9 / 19.7 | – | – | 2.5 / **3.0** | 2.7 / 5.0 |
| BN254 scalar | **5.5** / **7.0** | 67.3 / 94.3 | – | 13.0 / 22.0 | – | 7.0 / 11.0 |
| secp256k1 base | **6.1** / **7.0** | 66.7 / 93.3 | 5.8\* / 5.1\* | 14.6 / 21.3 | – | 7.1 / 11.0 |
| P-256 base | **6.2** / **7.0** | 66.7 / 92.7 | 10.9 / 18.8 | 10.9 / 21.0 | – | 7.1 / 11.0 |
| P-256 scalar | **6.1** / **7.0** | 66.7 / 94.0 | 17.7 / 23.0 | 14.6 / 21.0 | – | 7.1 / 11.0 |
| Tom-256 base | **6.1** / **7.0** | 66.7 / 93.2 | – | 12.1 / 20.7 | – | 7.1 / 11.0 |
| BLS12-381 base | **10.0** / **9.0** | 87.8 / 113 | – | 20.6 / 27.4 | – | 17.0 / 13.0 |
| P-521 base | **12.9** / **15.4** | 118 / 155 | – | – | – | 25.2 / 53.0 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub, constant time: a − b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.0** / **3.0** | 4.3 / 12.7 | – | – | 2.0 / 3.0 | – | **2.0** / 4.0 |
| BabyBear | **2.0** / **3.0** | 4.3 / 12.0 | – | – | – | – | **2.0** / 4.0 |
| Goldilocks | **2.0** / **3.0** | 4.3 / 12.1 | – | – | – | **2.0** / **3.0** | **2.0** / 4.0 |
| BN254 scalar | **5.2** / **7.0** | 50.1 / 51.0 | – | 8.6 / 12.0 | – | – | 7.0 / 8.0 |
| secp256k1 base | **5.6** / **8.0** | 50.0 / 48.3 | 5.2\* / 4.7\* | 7.8 / 13.0 | – | – | 7.0 / **8.0** |
| P-256 base | **5.6** / **7.4** | 50.1 / 43.7 | 8.1 / 13.0 | 7.8 / 12.0 | – | – | 7.0 / 8.0 |
| P-256 scalar | **5.3** / **7.0** | 50.1 / 50.6 | 8.4 / 13.0 | 8.2 / 13.0 | – | – | 7.1 / 8.0 |
| Tom-256 base | **5.6** / 8.2 | 50.1 / 55.5 | – | 8.6 / 12.0 | – | – | 7.0 / **8.0** |
| BLS12-381 base | **10.1** / **10.0** | 83.1 / 88.5 | – | 15.6 / 20.0 | – | – | 17.0 / 11.2 |
| P-521 base | **12.6** / **12.8** | 122 / 139 | – | – | – | – | 27.9 / 50.7 |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul, constant time: a × b

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.7** / **7.0** | 16.9 / 37.8 | – | – | – | 7.0 / 17.1 |
| BabyBear | **3.7** / **11.0** | 17.3 / 37.8 | – | – | – | 7.0 / 17.0 |
| Goldilocks | **3.8** / **9.1** | 17.7 / 34.9 | – | – | **3.8** / 11.0 | 7.0 / 17.0 |
| BN254 scalar | **69.2** / **79.1** | 121 / 158 | – | 95.8 / 94.2 | – | 94.7 / 130 |
| secp256k1 base | **44.7** / **55.0** | 118 / 155 | 85.1 / 88.6 | 97.7 / 96.0 | – | 94.7 / 130 |
| P-256 base | **51.0** / 60.0 | 119 / 162 | 68.9 / 73.7 | 71.2 / 73.0 | – | 51.5 / **58.1** |
| P-256 scalar | **75.1** / **84.1** | 119 / 155 | 143 / 141 | 95.4 / 101 | – | 94.2 / 131 |
| Tom-256 base | **71.1** / **76.1** | 119 / 156 | – | 91.9 / 95.1 | – | 94.0 / 130 |
| BLS12-381 base | **144** / **154** | 272 / 296 | – | 190 / 196 | – | 256 / 273 |
| P-521 base | **179** / **192** | 536 / 549 | – | – | – | 466 / 481 |

### square, constant time: a²

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | 16.8 / 34.0 | – | – | – | 7.6 / 17.2 |
| BabyBear | **3.9** / **11.0** | 16.8 / 34.4 | – | – | – | 7.6 / 17.3 |
| Goldilocks | 3.9 / **9.0** | 17.7 / 36.2 | – | – | **3.8** / 11.0 | 7.6 / 17.2 |
| BN254 scalar | **65.1** / **67.0** | 120 / 154 | – | 74.6 / 89.5 | – | 93.7 / 132 |
| secp256k1 base | **34.0** / **43.0** | 119 / 154 | 58.0 / 80.3 | 76.0 / 85.0 | – | 93.7 / 131 |
| P-256 base | **40.4** / **44.0** | 118 / 155 | 61.1 / 72.0 | 54.9 / 58.0 | – | 50.9 / 58.0 |
| P-256 scalar | **64.0** / **73.2** | 119 / 154 | 131 / 147 | 77.5 / 93.5 | – | 93.5 / 130 |
| Tom-256 base | **57.1** / **64.5** | 118 / 154 | – | 72.7 / 81.0 | – | 93.7 / 131 |
| BLS12-381 base | **140** / **146** | 270 / 294 | – | 152 / 166 | – | 255 / 273 |
| P-521 base | **121** / **134** | 543 / 566 | – | – | – | 468 / 473 |

### invert, constant time: a⁻¹

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | winter-math | runtime-algebra |
|---|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **261** / – | 3,855 / – | – | – | – | 956 / – |
| BabyBear | **396** / – | 3,857 / – | – | – | – | 945 / – |
| Goldilocks | **654** / – | 3,855 / – | – | – | 692 / – | 2,069 / – |
| BN254 scalar | **6,899** / – | 20,388 / – | – | 46,642 / – | – | 55,310 / – |
| secp256k1 base | **6,794** / – | 19,680 / – | 24,886 / – | 46,660 / – | – | 70,058 / – |
| P-256 base | **6,583** / – | 19,692 / – | 31,189 / – | 33,540 / – | – | 34,448 / – |
| P-256 scalar | **6,857** / – | 19,847 / – | 78,977 / – | 48,962 / – | – | 60,299 / – |
| Tom-256 base | **7,007** / – | 19,656 / – | – | 44,981 / – | – | 52,387 / – |
| BLS12-381 base | **12,166** / – | 34,610 / – | – | 143,281 / – | – | 181,054 / – |
| P-521 base | **19,907** / – | 64,637 / – | – | – | – | 506,146 / – |

### sqrt, constant time: √a

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row, every tie bold. –: no such function or no chain form.

| Field | prime-field | crypto-bigint | k256/p256 | halo2curves | runtime-algebra |
|---|---:|---:|---:|---:|---:|
| Mersenne-31 | **143** / – | 1,672 / – | – | – | 446 / – |
| BabyBear | **5,597** / – | 18,653 / – | – | – | – |
| Goldilocks | **7,154** / – | 26,949 / – | – | – | – |
| BN254 scalar | **58,606** / – | 133,870 / – | – | 85,142 / – | – |
| secp256k1 base | **12,285** / – | 53,053 / – | 26,959 / – | 47,477 / – | 68,798 / – |
| P-256 base | **12,006** / – | 43,563 / – | 30,310 / – | 33,662 / – | 31,921 / – |
| P-256 scalar | **25,489** / – | 52,451 / – | 81,303 / – | 42,128 / – | – |
| Tom-256 base | **20,086** / – | 48,990 / – | – | 44,671 / – | 52,057 / – |
| BLS12-381 base | **69,918** / – | 135,513 / – | – | 141,266 / – | 178,598 / – |
| P-521 base | **69,239** / – | 268,615 / – | – | – | 259,186 / – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
