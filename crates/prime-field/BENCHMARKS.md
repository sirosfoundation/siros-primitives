# Benchmarks

`prime-field` at commit `3264a60` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`: add_vartime, add_ct, sub_vartime, sub_ct, mul_vartime, mul_ct, square_vartime, square_ct, invert_vartime, invert_ct, sqrt_vartime and sqrt_ct over Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base, in cycles per operation, measured on 2026-11-02.

The CPU is an AMD Ryzen 9 5950X 16-Core Processor, measured on its CPU 15.
Its counters give each figure per operation as the median, over 31 repetitions, of a run of 2n operations on fresh operands less a run of n, over n.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile |
| Features | `prime-field` with its default features |
| RUSTFLAGS | `-C target-cpu=native -C codegen-units=1` |
| Extensions | adx, aes, avx, avx2, bmi1, bmi2, f16c, fma, lzcnt, movbe, pclmulqdq, popcnt, sha, sse, sse2, sse3, sse4.1, sse4.2, sse4a, ssse3, vaes, vpclmulqdq |
| Counters | one Linux perf_event group: user-mode cycles and branch misses |
| Load | 1-minute load average 9.73 before the sweep and 13.72 after; CPU 31, the SMT sibling of CPU 15, busy 20% of the sweep |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

A sweep measures every case in 31 repetitions; repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine reaches all cases alike. Each run draws fresh operand sets from the case's distribution, outside the counted region: no operand set runs twice, so the branch predictor never learns a sequence. A repetition runs n operations uncounted, then counts a run of n operations and a run of 2n. Before the repetitions, n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as many, takes 1,048,576 cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.

One perf_event group on the measuring thread, pinned to one CPU, counts user-mode cycles and branch misses; a run during which the kernel shares the counters with other groups stops the sweep. A figure per operation is the median, over the 31 repetitions, of the long run less the short run, divided by n: the fixed cost of a run cancels, as in nanoBench's differencing, and each difference pairs two runs taken back to back. A case whose long runs count no more cycles than its short runs stops the sweep, and so does a chain step under one cycle.

Throughput runs independent operations, with operands and results behind `core::hint::black_box`. Latency runs the chain `x = op(x, y)`: from the second operand set on, each result is the first operand of the next operation, so each operation waits for the one before. Each result passes through an empty inline-assembly block, which keeps the compiler from regrouping the chain. The block holds a result of up to 64 bytes in general-purpose registers in one chain and in vector registers in another, each chain a case of its own in the sweep; it can only add moves between the two, so the latency is the chain with fewer cycles. A larger result passes through memory, in a single chain. The chain applies where the result has the type of the first operand and the other operands are plain values: a step copies them, and the copy of an allocated value would allocate.

The sweep ran at a 1-minute load of 2 or more, so its cycles compare within each table only.

The operands are canonical residues, drawn from one distribution for every crate in both rows of an operation. Before a case is measured, its results on 256 operand sets are checked against `num-bigint`: the exact residue, `a^(p - 2)` for the inverses, and either root for `sqrt`. Elements enter and leave each crate in its own representation; the conversions lie outside the counted region. The functions that a case calls, such as the allocator of `compile-algebra`, are part of its counts. The chains are those of add, sub, mul and square, except `compile-algebra`'s, whose step would copy an allocated operand.

| Operation | Operands |
|---|---|
| add, sub, mul | `a` and `b` uniform below `p` |
| square, invert | `a` uniform in `[1, p)` |
| sqrt | `a = r²` for `r` uniform in `[1, p)`, so that a root exists |

## Functions

Each cell measures the crate's own function:

| Operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---|---|---|---|---|---|---|---|---|
| add_vartime | `+` | `+` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | `+` over Mersenne-31, BabyBear and Goldilocks | – | – | `addf` over P-256 base and P-256 scalar |
| add_ct | `Secret +` | – | `+` | `+` over secp256k1 base, P-256 base and P-256 scalar | `+` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | `+` over Goldilocks | `addf` | – |
| sub_vartime | `-` | `-` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | `-` over BabyBear and Goldilocks | – | – | `subf` over P-256 base and P-256 scalar |
| sub_ct | `Secret -` | – | `-` | `-` over secp256k1 base, P-256 base and P-256 scalar | `-` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `-` over Mersenne-31 | `-` over Goldilocks | `subf` | – |
| mul_vartime | `*` | `*` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | `*` over Mersenne-31, BabyBear and Goldilocks | – | – | `mulf` over P-256 base and P-256 scalar |
| mul_ct | `Secret *` | – | `*` | `*` over secp256k1 base, P-256 base and P-256 scalar | `*` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | `*` over Goldilocks | `mulf` | – |
| square_vartime | `square` | `square` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | `square` over Mersenne-31, BabyBear and Goldilocks | – | – | – |
| square_ct | `Secret::square` | – | `square` | `square` over secp256k1 base, P-256 base and P-256 scalar | `square` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | `square` over Goldilocks | – | – |
| invert_vartime | `invert` | `inverse` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `invert_vartime` | `invert_vartime` over P-256 scalar | `invert` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `try_inverse` over Mersenne-31, BabyBear and Goldilocks | – | `invert` | `invert` over P-256 base and P-256 scalar |
| invert_ct | `Secret::invert` | – | `invert` | `invert` over secp256k1 base, P-256 base and P-256 scalar | – | – | `inv` over Goldilocks | – | – |
| sqrt_vartime | `sqrt` | `sqrt` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | `try_sqrt` over Mersenne-31, BabyBear and Goldilocks | – | – | – |
| sqrt_ct | `Secret::sqrt` | – | `sqrt` | `sqrt` over secp256k1 base, P-256 base and P-256 scalar | `sqrt` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | – | – | – | – |

- A variable-time row compares the functions whose time may depend on the values; a constant-time row those whose time depends on no value. Each function sits in the row of its class, as its source shows it: a branch, a loop bound or a memory address that depends on a value makes it variable time.
- `prime-field` runs `Fp` in the variable-time rows and `Secret` in the constant-time rows.
- `prime-field`, `ark-ff`, `crypto-bigint`, `halo2curves` and `runtime-algebra` are generic over the modulus; `k256`, `p256`, `p3-*` and `winter-math` are written for their field.
- `crypto-bigint` is `ConstMontyForm` with a `const_monty_params!` modulus: masks and constant-time selections, except `invert_vartime`.
- `ark-ff` is `Fp` with `MontBackend` from the arkworks curve crates: `+` and `*` branch on their final subtraction, `-` on its borrow, `inverse` and `sqrt` throughout.
- `k256`'s `+` and `-` add limbs without carries and leave the reduction to a later `normalize`. `p256`'s scalar adds `invert_vartime`, Stein's algorithm.
- `halo2curves` selects by masks in `+`, `-` and `*`, and runs the constant-time `sqrt` of `ff`; its `invert` runs variable-time division steps.
- `plonky3` is `p3-mersenne-31`, `p3-baby-bear` and `p3-goldilocks` on their field: `+`, `*`, `square`, `try_inverse` and `try_sqrt` decide with `if` on the value; Mersenne-31's `-` is arithmetic alone.
- `winter-math` corrects by masks, and inverts by a fixed addition chain.
- `runtime-algebra` and `compile-algebra` are longfellow-zk's prover-side and compile-side fields: `FpGenericField` is their generic Montgomery field with the modulus held at run time, whose `addf`, `subf` and `mulf` select by `cmov` in inline assembly and whose `invert` branches; `P256Field` adds a hand-written P-256 strategy; `compile-algebra`'s `FpField` computes on `num-bigint`.
- No arkworks crate declares the field: `ark-ff` over Mersenne-31, BabyBear, Goldilocks and P-521 base.
- No constant-time function: `ark-ff` add_ct, sub_ct, mul_ct, square_ct, invert_ct and sqrt_ct over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base, `halo2curves` invert_ct over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base, `plonky3` add_ct over Mersenne-31, BabyBear and Goldilocks, sub_ct over BabyBear and Goldilocks, mul_ct over Mersenne-31, BabyBear and Goldilocks, square_ct over Mersenne-31, BabyBear and Goldilocks, invert_ct over Mersenne-31, BabyBear and Goldilocks and sqrt_ct over Mersenne-31, BabyBear and Goldilocks, `runtime-algebra` invert_ct and `compile-algebra` add_ct, sub_ct, mul_ct and invert_ct over P-256 base and P-256 scalar.
- No variable-time function: `crypto-bigint` add_vartime, sub_vartime, mul_vartime, square_vartime and sqrt_vartime, `k256/p256` add_vartime over secp256k1 base, P-256 base and P-256 scalar, sub_vartime over secp256k1 base, P-256 base and P-256 scalar, mul_vartime over secp256k1 base, P-256 base and P-256 scalar, square_vartime over secp256k1 base, P-256 base and P-256 scalar, invert_vartime over secp256k1 base and P-256 base and sqrt_vartime over secp256k1 base, P-256 base and P-256 scalar, `halo2curves` add_vartime, sub_vartime, mul_vartime, square_vartime and sqrt_vartime over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base, `plonky3` sub_vartime over Mersenne-31, `winter-math` add_vartime, sub_vartime, mul_vartime, square_vartime and invert_vartime over Goldilocks and `runtime-algebra` add_vartime, sub_vartime and mul_vartime.
- No RustCrypto crate declares the field: `k256/p256` over Mersenne-31, BabyBear, Goldilocks, BN254 scalar and BLS12-381 base.
- P521 keeps its field element private: `k256/p256` over P-521 base.
- Halo2curves declares no such field: `halo2curves` over Mersenne-31, BabyBear, Goldilocks and P-521 base.
- Plonky3 declares no such field: `plonky3` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base.
- Winter-math declares no such field: `winter-math` over Mersenne-31, BabyBear, BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base.
- No square root: `winter-math` sqrt_vartime and sqrt_ct over Goldilocks, `runtime-algebra` sqrt_vartime and sqrt_ct and `compile-algebra` sqrt_vartime and sqrt_ct over P-256 base and P-256 scalar.
- No squaring function: `runtime-algebra` square_vartime and square_ct and `compile-algebra` square_vartime and square_ct over P-256 base and P-256 scalar.
- Compile-algebra declares P-256 base and scalar only: `compile-algebra` over Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1 base, BLS12-381 base and P-521 base.

## Results

The charts show the first 8 crates; `compile-algebra` appears in the tables only.

### add_vartime: a + b, variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.5** / **3.0** | – | – | – | – | 2.2\* / 4.0 | – | – | – |
| BabyBear | 2.3 / **3.0** | – | – | – | – | **2.2** / 4.0 | – | – | – |
| Goldilocks | **2.2** / **3.0** | – | – | – | – | 2.2\* / 3.0 | – | – | – |
| BN254 scalar | **6.1** / **6.5** | 18.9 / 13.9 | – | – | – | – | – | – | – |
| secp256k1 base | **6.1** / **7.0** | 17.9 / 14.1 | – | – | – | – | – | – | – |
| P-256 base | **6.1** / **7.0** | 19.5 / 14.7 | – | – | – | – | – | – | 369 / – |
| P-256 scalar | **7.0** / **7.0** | 18.6 / 14.5 | – | – | – | – | – | – | 357 / – |
| BLS12-381 base | **11.0** / **10.0** | 21.5 / 15.7 | – | – | – | – | – | – | – |
| P-521 base | **12.8** / **21.7** | – | – | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_ct: a + b, constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.2** / **3.0** | – | 9.8 / 17.9 | – | – | – | – | 2.5 / 5.0 | – |
| BabyBear | **2.3** / **3.0** | – | 9.8 / 18.8 | – | – | – | – | 2.5 / 5.0 | – |
| Goldilocks | **2.2** / **3.0** | – | 10.3 / 19.0 | – | – | – | 3.0 / 3.0 | 2.5 / 5.0 | – |
| BN254 scalar | **6.1** / **6.5** | – | 66.3 / 80.7 | – | 12.9 / 19.8 | – | – | 7.1 / 9.4 | – |
| secp256k1 base | **6.1** / **7.0** | – | 66.2 / 80.6 | 6.2 / 4.9\* | 15.4 / 21.3 | – | – | 7.1 / 9.4 | – |
| P-256 base | **6.2** / **7.0** | – | 66.6 / 80.5 | 29.4 / 27.5 | 10.6 / 19.6 | – | – | 7.1 / 9.4 | – |
| P-256 scalar | **7.0** / **7.0** | – | 66.2 / 83.3 | 32.0 / 29.4 | 15.1 / 21.1 | – | – | 7.1 / 9.4 | – |
| BLS12-381 base | **11.0** / **10.0** | – | 87.6 / 115 | – | 21.6 / 26.0 | – | – | 40.2 / 13.9 | – |
| P-521 base | **12.7** / **21.8** | – | 128 / 223 | – | – | – | – | 38.7 / 66.9 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_vartime: a − b, variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.2** / **3.0** | – | – | – | – | – | – | – | – |
| BabyBear | 2.2 / **3.0** | – | – | – | – | **2.2** / 4.0 | – | – | – |
| Goldilocks | **2.2** / **3.0** | – | – | – | – | 2.2\* / 3.0 | – | – | – |
| BN254 scalar | **6.1** / **6.5** | 17.2 / 14.1 | – | – | – | – | – | – | – |
| secp256k1 base | **6.4** / **8.0** | 17.3 / 14.2 | – | – | – | – | – | – | – |
| P-256 base | **6.3** / **7.0** | 16.7 / 12.9 | – | – | – | – | – | – | 169 / – |
| P-256 scalar | **6.4** / **6.9** | 17.2 / 13.4 | – | – | – | – | – | – | 168 / – |
| BLS12-381 base | **10.1** / **9.6** | 23.2 / 19.5 | – | – | – | – | – | – | – |
| P-521 base | **13.1** / **21.3** | – | – | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_ct: a − b, constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.2** / **3.0** | – | 4.6 / 12.4 | – | – | 2.2\* / 4.0 | – | 2.4 / 4.0 | – |
| BabyBear | **2.2** / **3.0** | – | 4.6 / 12.3 | – | – | – | – | 2.5 / 4.0 | – |
| Goldilocks | 2.2 / 3.0 | – | 4.6 / 12.5 | – | – | – | **2.2** / **3.0** | 2.4 / 4.0 | – |
| BN254 scalar | **6.1** / **6.5** | – | 49.4 / 52.3 | – | 10.0 / 11.5 | – | – | 7.0 / 9.0 | – |
| secp256k1 base | **6.4** / **8.0** | – | 49.3 / 48.9 | 6.0\* / 4.7\* | 8.0 / 12.0 | – | – | 7.0 / 9.0 | – |
| P-256 base | **6.2** / **7.0** | – | 49.5 / 49.7 | 28.3 / 15.0 | 8.0 / 12.0 | – | – | 7.0 / 9.0 | – |
| P-256 scalar | **6.2** / **7.0** | – | 49.3 / 48.5 | 28.7 / 15.3 | 9.0 / 13.0 | – | – | 7.0 / 9.0 | – |
| BLS12-381 base | **10.0** / **9.6** | – | 81.8 / 87.0 | – | 16.0 / 19.6 | – | – | 39.7 / 12.0 | – |
| P-521 base | **13.1** / **21.2** | – | 131 / 219 | – | – | – | – | 35.6 / 62.0 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul_vartime: a × b, variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.9** / **7.0** | – | – | – | – | 3.0 / 8.0 | – | – | – |
| BabyBear | **3.0** / **11.0** | – | – | – | – | 4.8 / 14.0 | – | – | – |
| Goldilocks | **4.3** / **9.0** | – | – | – | – | 8.0 / 10.1 | – | – | – |
| BN254 scalar | **68.7** / **71.0** | 77.8 / 72.4 | – | – | – | – | – | – | – |
| secp256k1 base | **43.9** / **56.4** | 93.1 / 89.6 | – | – | – | – | – | – | – |
| P-256 base | **49.3** / **56.0** | 66.7 / 64.2 | – | – | – | – | – | – | 597 / – |
| P-256 scalar | **76.8** / **82.1** | 93.5 / 91.1 | – | – | – | – | – | – | 664 / – |
| BLS12-381 base | **156** / **156** | 167 / 161 | – | – | – | – | – | – | – |
| P-521 base | **195** / **212** | – | – | – | – | – | – | – | – |

### mul_ct: a × b, constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.9** / **7.0** | – | 17.1 / 35.3 | – | – | – | – | 7.0 / 17.0 | – |
| BabyBear | **3.0** / **11.0** | – | 16.5 / 35.3 | – | – | – | – | 7.0 / 17.0 | – |
| Goldilocks | 4.3 / **9.0** | – | 18.3 / 35.1 | – | – | – | **4.0** / 11.0 | 7.0 / 17.0 | – |
| BN254 scalar | **71.5** / **74.1** | – | 124 / 166 | – | 91.0 / 99.1 | – | – | 93.0 / 127 | – |
| secp256k1 base | **44.0** / **56.1** | – | 125 / 168 | 100 / 89.0 | 88.9 / 93.1 | – | – | 93.0 / 127 | – |
| P-256 base | **49.5** / **56.1** | – | 124 / 167 | 72.5 / 69.2 | 69.4 / 73.1 | – | – | 53.2 / 56.3 | – |
| P-256 scalar | **76.8** / **82.1** | – | 124 / 166 | 176 / 151 | 90.7 / 102 | – | – | 93.1 / 127 | – |
| BLS12-381 base | **160** / **159** | – | 287 / 302 | – | 191 / 193 | – | – | 256 / 252 | – |
| P-521 base | **195** / **212** | – | 570 / 625 | – | – | – | – | 464 / 496 | – |

### square_vartime: a², variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form. A `\*` marks the figure of a function that deviates from the row's definition where the figure is below `prime-field`'s; the note under the table says how the function deviates.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | – | – | – | – | 2.8 / 8.0 | – | – | – |
| BabyBear | **3.0** / **11.0** | – | – | – | – | 5.0 / 14.0 | – | – | – |
| Goldilocks | **4.3** / **9.0** | – | – | – | – | 3.6\* / 10.0 | – | – | – |
| BN254 scalar | **64.2** / **69.1** | 74.9 / 72.1 | – | – | – | – | – | – | – |
| secp256k1 base | **32.7** / **44.2** | 82.8 / 80.4 | – | – | – | – | – | – | – |
| P-256 base | **39.2** / **48.3** | 65.6 / 59.5 | – | – | – | – | – | – | – |
| P-256 scalar | **63.0** / **73.0** | 89.9 / 85.0 | – | – | – | – | – | – | – |
| BLS12-381 base | **135** / **138** | 144 / 138 | – | – | – | – | – | – | – |
| P-521 base | **122** / **137** | – | – | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square_ct: a², constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **2.8** / **7.0** | – | 17.0 / 37.1 | – | – | – | – | – | – |
| BabyBear | **3.0** / **11.0** | – | 17.0 / 36.6 | – | – | – | – | – | – |
| Goldilocks | 4.3 / **9.0** | – | 18.2 / 34.9 | – | – | – | **3.9** / 11.0 | – | – |
| BN254 scalar | **65.1** / **75.0** | – | 112 / 154 | – | 77.0 / 83.0 | – | – | – | – |
| secp256k1 base | **32.7** / **44.2** | – | 119 / 153 | 72.9 / 88.0 | 70.4 / 81.0 | – | – | – | – |
| P-256 base | **39.1** / **48.0** | – | 116 / 157 | 63.7 / 70.0 | 49.9 / 60.0 | – | – | – | – |
| P-256 scalar | **63.0** / **73.0** | – | 113 / 153 | 141 / 147 | 81.1 / 89.0 | – | – | – | – |
| BLS12-381 base | **141** / **138** | – | 290 / 298 | – | 149 / 164 | – | – | – | – |
| P-521 base | **122** / **136** | – | 570 / 624 | – | – | – | – | – | – |

### invert_vartime: a⁻¹, variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **224** / – | – | 1,430 / – | – | – | 648 / – | – | 905 / – | – |
| BabyBear | **222** / – | – | 1,408 / – | – | – | 634 / – | – | 1,065 / – | – |
| Goldilocks | **438** / – | – | 2,863 / – | – | – | 1,353 / – | – | 1,744 / – | – |
| BN254 scalar | **5,705** / – | 16,237 / – | 14,403 / – | – | 6,902 / – | – | – | 15,835 / – | – |
| secp256k1 base | **5,646** / – | 15,392 / – | 14,503 / – | – | 6,924 / – | – | – | 16,032 / – | – |
| P-256 base | **5,520** / – | 15,961 / – | 14,465 / – | – | 6,927 / – | – | – | 15,607 / – | 94,410 / – |
| P-256 scalar | **5,734** / – | 16,568 / – | 14,510 / – | 32,958 / – | 6,930 / – | – | – | 16,028 / – | 95,327 / – |
| BLS12-381 base | **10,066** / – | 32,896 / – | 25,742 / – | – | 11,776 / – | – | – | 33,987 / – | – |
| P-521 base | **16,205** / – | – | 45,254 / – | – | – | – | – | 66,178 / – | – |

### invert_ct: a⁻¹, constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **262** / – | – | 3,860 / – | – | – | – | – | – | – |
| BabyBear | **387** / – | – | 3,891 / – | – | – | – | – | – | – |
| Goldilocks | **656** / – | – | 3,845 / – | – | – | – | 687 / – | – | – |
| BN254 scalar | **6,826** / – | – | 19,433 / – | – | – | – | – | – | – |
| secp256k1 base | **6,909** / – | – | 19,540 / – | 24,614 / – | – | – | – | – | – |
| P-256 base | **6,885** / – | – | 19,479 / – | 29,497 / – | – | – | – | – | – |
| P-256 scalar | **6,924** / – | – | 19,470 / – | 82,729 / – | – | – | – | – | – |
| BLS12-381 base | **12,930** / – | – | 34,383 / – | – | – | – | – | – | – |
| P-521 base | **20,097** / – | – | 65,807 / – | – | – | – | – | – | – |

### sqrt_vartime: √a, variable time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **139** / – | – | – | – | – | 143 / – | – | – | – |
| BabyBear | **2,060** / – | – | – | – | – | 2,857 / – | – | – | – |
| Goldilocks | **3,064** / – | – | – | – | – | 3,400 / – | – | – | – |
| BN254 scalar | **33,939** / – | 45,047 / – | – | – | – | – | – | – | – |
| secp256k1 base | **12,480** / – | 43,030 / – | – | – | – | – | – | – | – |
| P-256 base | **11,896** / – | 18,342 / – | – | – | – | – | – | – | – |
| P-256 scalar | **23,759** / – | 39,532 / – | – | – | – | – | – | – | – |
| BLS12-381 base | **65,055** / – | 90,502 / – | – | – | – | – | – | – | – |
| P-521 base | **71,644** / – | – | – | – | – | – | – | – | – |

### sqrt_ct: √a, constant time

Each cell: throughput cycles / latency cycles per operation. Bold: the fewest of the row. –: no such function or no chain form.

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | **143** / – | – | 1,654 / – | – | – | – | – | – | – |
| BabyBear | **5,580** / – | – | 18,705 / – | – | – | – | – | – | – |
| Goldilocks | **7,154** / – | – | 27,028 / – | – | – | – | – | – | – |
| BN254 scalar | **58,427** / – | – | 134,420 / – | – | 106,041 / – | – | – | – | – |
| secp256k1 base | **12,060** / – | – | 53,278 / – | 24,306 / – | 45,737 / – | – | – | – | – |
| P-256 base | **12,441** / – | – | 43,468 / – | 29,466 / – | 34,918 / – | – | – | – | – |
| P-256 scalar | **25,493** / – | – | 52,750 / – | 86,078 / – | 41,719 / – | – | – | – | – |
| BLS12-381 base | **67,414** / – | – | 133,836 / – | – | 139,796 / – | – | – | – | – |
| P-521 base | **70,543** / – | – | 268,613 / – | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file
```
