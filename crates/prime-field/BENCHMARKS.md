# Benchmarks

`prime-field` at commit `3264a60` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`: add_vartime, add_ct, sub_vartime, sub_ct, mul_vartime, mul_ct, square_vartime, square_ct, invert_vartime, invert_ct, sqrt_vartime and sqrt_ct over Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base, in cycles and instructions per operation, measured on 2026-11-02.

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
| Counters | one Linux perf_event group: user-mode cycles, instructions and branch misses |
| Load | 1-minute load average 9.73 before the sweep and 13.72 after; CPU 31, the SMT sibling of CPU 15, busy 20% of the sweep |
| Crates | ark-ff 0.6.0, crypto-bigint 0.7.10, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

A sweep measures every case in 31 repetitions; repetition r of every case runs before repetition r + 1 of any case, so a drift of the machine reaches all cases alike. Each run draws fresh operand sets from the case's distribution, outside the counted region: no operand set runs twice, so the branch predictor never learns a sequence. A repetition runs n operations uncounted, then counts a run of n operations and a run of 2n. Before the repetitions, n starts at 16 operations and doubles until a run of n, counted after an uncounted run of as many, takes 1,048,576 cycles; n stops earlier where the operand sets of 2n operations fill 256 KiB.

One perf_event group on the measuring thread, pinned to one CPU, counts user-mode cycles, instructions and branch misses; a run during which the kernel shares the counters with other groups stops the sweep. A figure per operation is the median, over the 31 repetitions, of the long run less the short run, divided by n: the fixed cost of a run cancels, as in nanoBench's differencing, and each difference pairs two runs taken back to back. A case whose long runs count no more cycles than its short runs stops the sweep, and so does a chain step under one cycle, the latency of a single instruction.

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

Per operation, each row gives throughput cycles, latency cycles for the operations that have chains, and instructions. Bold marks the fewest in each row among the functions that meet the row's definition. A `\*` marks the figure of a function that deviates from it where the figure is below `prime-field`'s; the note under the table says how the function deviates.
The charts show the first 8 crates; `compile-algebra` appears in the tables only.

### add_vartime: a + b, variable time

![add_vartime: a + b, variable time](benchmarks/add_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.5** | – | – | – | – | 2.2\* | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | 4.0 | – | – | – |
|  | instructions | **10.0** | – | – | – | – | 10.0 | – | – | – |
| BabyBear | throughput cycles | 2.3 | – | – | – | – | **2.2** | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | 4.0 | – | – | – |
|  | instructions | **11.0** | – | – | – | – | **11.0** | – | – | – |
| Goldilocks | throughput cycles | **2.2** | – | – | – | – | 2.2\* | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | 3.0 | – | – | – |
|  | instructions | **12.0** | – | – | – | – | 12.0 | – | – | – |
| BN254 scalar | throughput cycles | **6.1** | 18.9 | – | – | – | – | – | – | – |
|  | latency cycles | **6.5** | 13.9 | – | – | – | – | – | – | – |
|  | instructions | 30.0 | **26.5** | – | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **6.1** | 17.9 | – | – | – | – | – | – | – |
|  | latency cycles | **7.0** | 14.1 | – | – | – | – | – | – | – |
|  | instructions | 31.0 | **30.0** | – | – | – | – | – | – | – |
| P-256 base | throughput cycles | **6.1** | 19.5 | – | – | – | – | – | – | 369 |
|  | latency cycles | **7.0** | 14.7 | – | – | – | – | – | – | – |
|  | instructions | 31.0 | **29.0** | – | – | – | – | – | – | 1,423 |
| P-256 scalar | throughput cycles | **7.0** | 18.6 | – | – | – | – | – | – | 357 |
|  | latency cycles | **7.0** | 14.5 | – | – | – | – | – | – | – |
|  | instructions | 32.0 | **28.0** | – | – | – | – | – | – | 1,423 |
| BLS12-381 base | throughput cycles | **11.0** | 21.5 | – | – | – | – | – | – | – |
|  | latency cycles | **10.0** | 15.7 | – | – | – | – | – | – | – |
|  | instructions | 48.0 | **34.5** | – | – | – | – | – | – | – |
| P-521 base | throughput cycles | **12.8** | – | – | – | – | – | – | – | – |
|  | latency cycles | **21.7** | – | – | – | – | – | – | – | – |
|  | instructions | **53.0** | – | – | – | – | – | – | – | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### add_ct: a + b, constant time

![add_ct: a + b, constant time](benchmarks/add_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.2** | – | 9.8 | – | – | – | – | 2.5 | – |
|  | latency cycles | **3.0** | – | 17.9 | – | – | – | – | 5.0 | – |
|  | instructions | **10.0** | – | 38.0 | – | – | – | – | 16.0 | – |
| BabyBear | throughput cycles | **2.3** | – | 9.8 | – | – | – | – | 2.5 | – |
|  | latency cycles | **3.0** | – | 18.8 | – | – | – | – | 5.0 | – |
|  | instructions | **11.0** | – | 38.0 | – | – | – | – | 16.0 | – |
| Goldilocks | throughput cycles | **2.2** | – | 10.3 | – | – | – | 3.0 | 2.5 | – |
|  | latency cycles | **3.0** | – | 19.0 | – | – | – | 3.0 | 5.0 | – |
|  | instructions | **12.0** | – | 39.0 | – | – | – | 16.0 | 16.0 | – |
| BN254 scalar | throughput cycles | **6.1** | – | 66.3 | – | 12.9 | – | – | 7.1 | – |
|  | latency cycles | **6.5** | – | 80.7 | – | 19.8 | – | – | 9.4 | – |
|  | instructions | **30.0** | – | 147 | – | 57.0 | – | – | 36.0 | – |
| secp256k1 base | throughput cycles | **6.1** | – | 66.2 | 6.2 | 15.4 | – | – | 7.1 | – |
|  | latency cycles | **7.0** | – | 80.6 | 4.9\* | 21.3 | – | – | 9.4 | – |
|  | instructions | **31.0** | – | 147 | 11.0\* | 58.0 | – | – | 36.0 | – |
| P-256 base | throughput cycles | **6.2** | – | 66.6 | 29.4 | 10.6 | – | – | 7.1 | – |
|  | latency cycles | **7.0** | – | 80.5 | 27.5 | 19.6 | – | – | 9.4 | – |
|  | instructions | **31.0** | – | 147 | 61.0 | 46.0 | – | – | 36.0 | – |
| P-256 scalar | throughput cycles | **7.0** | – | 66.2 | 32.0 | 15.1 | – | – | 7.1 | – |
|  | latency cycles | **7.0** | – | 83.3 | 29.4 | 21.1 | – | – | 9.4 | – |
|  | instructions | **32.0** | – | 147 | 87.0 | 61.0 | – | – | 36.0 | – |
| BLS12-381 base | throughput cycles | **11.0** | – | 87.6 | – | 21.6 | – | – | 40.2 | – |
|  | latency cycles | **10.0** | – | 115 | – | 26.0 | – | – | 13.9 | – |
|  | instructions | **48.0** | – | 180 | – | 88.0 | – | – | 73.0 | – |
| P-521 base | throughput cycles | **12.7** | – | 128 | – | – | – | – | 38.7 | – |
|  | latency cycles | **21.8** | – | 223 | – | – | – | – | 66.9 | – |
|  | instructions | **53.0** | – | 252 | – | – | – | – | 114 | – |

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### sub_vartime: a − b, variable time

![sub_vartime: a − b, variable time](benchmarks/sub_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.2** | – | – | – | – | – | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | – | – | – | – |
|  | instructions | **10.0** | – | – | – | – | – | – | – | – |
| BabyBear | throughput cycles | 2.2 | – | – | – | – | **2.2** | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | 4.0 | – | – | – |
|  | instructions | **10.0** | – | – | – | – | **10.0** | – | – | – |
| Goldilocks | throughput cycles | **2.2** | – | – | – | – | 2.2\* | – | – | – |
|  | latency cycles | **3.0** | – | – | – | – | 3.0 | – | – | – |
|  | instructions | **10.0** | – | – | – | – | 12.0 | – | – | – |
| BN254 scalar | throughput cycles | **6.1** | 17.2 | – | – | – | – | – | – | – |
|  | latency cycles | **6.5** | 14.1 | – | – | – | – | – | – | – |
|  | instructions | **30.0** | 36.6 | – | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **6.4** | 17.3 | – | – | – | – | – | – | – |
|  | latency cycles | **8.0** | 14.2 | – | – | – | – | – | – | – |
|  | instructions | **24.0** | 35.0 | – | – | – | – | – | – | – |
| P-256 base | throughput cycles | **6.3** | 16.7 | – | – | – | – | – | – | 169 |
|  | latency cycles | **7.0** | 12.9 | – | – | – | – | – | – | – |
|  | instructions | **25.0** | 31.1 | – | – | – | – | – | – | 599 |
| P-256 scalar | throughput cycles | **6.4** | 17.2 | – | – | – | – | – | – | 168 |
|  | latency cycles | **6.9** | 13.4 | – | – | – | – | – | – | – |
|  | instructions | **28.0** | 34.5 | – | – | – | – | – | – | 600 |
| BLS12-381 base | throughput cycles | **10.1** | 23.2 | – | – | – | – | – | – | – |
|  | latency cycles | **9.6** | 19.5 | – | – | – | – | – | – | – |
|  | instructions | **49.0** | 53.1 | – | – | – | – | – | – | – |
| P-521 base | throughput cycles | **13.1** | – | – | – | – | – | – | – | – |
|  | latency cycles | **21.3** | – | – | – | – | – | – | – | – |
|  | instructions | **44.0** | – | – | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sub_ct: a − b, constant time

![sub_ct: a − b, constant time](benchmarks/sub_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.2** | – | 4.6 | – | – | 2.2\* | – | 2.4 | – |
|  | latency cycles | **3.0** | – | 12.4 | – | – | 4.0 | – | 4.0 | – |
|  | instructions | **10.0** | – | 19.0 | – | – | 10.0 | – | 13.0 | – |
| BabyBear | throughput cycles | **2.2** | – | 4.6 | – | – | – | – | 2.5 | – |
|  | latency cycles | **3.0** | – | 12.3 | – | – | – | – | 4.0 | – |
|  | instructions | **10.0** | – | 19.0 | – | – | – | – | 13.0 | – |
| Goldilocks | throughput cycles | 2.2 | – | 4.6 | – | – | – | **2.2** | 2.4 | – |
|  | latency cycles | 3.0 | – | 12.5 | – | – | – | **3.0** | 4.0 | – |
|  | instructions | **10.0** | – | 19.0 | – | – | – | **10.0** | 13.0 | – |
| BN254 scalar | throughput cycles | **6.1** | – | 49.4 | – | 10.0 | – | – | 7.0 | – |
|  | latency cycles | **6.5** | – | 52.3 | – | 11.5 | – | – | 9.0 | – |
|  | instructions | **30.0** | – | 94.0 | – | 45.0 | – | – | 35.0 | – |
| secp256k1 base | throughput cycles | **6.4** | – | 49.3 | 6.0\* | 8.0 | – | – | 7.0 | – |
|  | latency cycles | **8.0** | – | 48.9 | 4.7\* | 12.0 | – | – | 9.0 | – |
|  | instructions | **24.0** | – | 94.0 | 13.0\* | 39.0 | – | – | 35.0 | – |
| P-256 base | throughput cycles | **6.2** | – | 49.5 | 28.3 | 8.0 | – | – | 7.0 | – |
|  | latency cycles | **7.0** | – | 49.7 | 15.0 | 12.0 | – | – | 9.0 | – |
|  | instructions | **25.0** | – | 94.0 | 55.0 | 40.0 | – | – | 35.0 | – |
| P-256 scalar | throughput cycles | **6.2** | – | 49.3 | 28.7 | 9.0 | – | – | 7.0 | – |
|  | latency cycles | **7.0** | – | 48.5 | 15.3 | 13.0 | – | – | 9.0 | – |
|  | instructions | **28.0** | – | 94.0 | 56.0 | 43.0 | – | – | 35.0 | – |
| BLS12-381 base | throughput cycles | **10.0** | – | 81.8 | – | 16.0 | – | – | 39.7 | – |
|  | latency cycles | **9.6** | – | 87.0 | – | 19.6 | – | – | 12.0 | – |
|  | instructions | **49.0** | – | 138 | – | 74.0 | – | – | 72.0 | – |
| P-521 base | throughput cycles | **13.1** | – | 131 | – | – | – | – | 35.6 | – |
|  | latency cycles | **21.2** | – | 219 | – | – | – | – | 62.0 | – |
|  | instructions | **44.0** | – | 186 | – | – | – | – | 115 | – |

\* `plonky3` over Mersenne-31: returns p for a zero result, which is not canonical.

\* `k256/p256` over secp256k1 base: skips the final reduction; the result is not canonical.

### mul_vartime: a × b, variable time

![mul_vartime: a × b, variable time](benchmarks/mul_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.9** | – | – | – | – | 3.0 | – | – | – |
|  | latency cycles | **7.0** | – | – | – | – | 8.0 | – | – | – |
|  | instructions | **14.0** | – | – | – | – | 15.0 | – | – | – |
| BabyBear | throughput cycles | **3.0** | – | – | – | – | 4.8 | – | – | – |
|  | latency cycles | **11.0** | – | – | – | – | 14.0 | – | – | – |
|  | instructions | **13.0** | – | – | – | – | 17.0 | – | – | – |
| Goldilocks | throughput cycles | **4.3** | – | – | – | – | 8.0 | – | – | – |
|  | latency cycles | **9.0** | – | – | – | – | 10.1 | – | – | – |
|  | instructions | **18.0** | – | – | – | – | 18.0 | – | – | – |
| BN254 scalar | throughput cycles | **68.7** | 77.8 | – | – | – | – | – | – | – |
|  | latency cycles | **71.0** | 72.4 | – | – | – | – | – | – | – |
|  | instructions | **202** | 238 | – | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **43.9** | 93.1 | – | – | – | – | – | – | – |
|  | latency cycles | **56.4** | 89.6 | – | – | – | – | – | – | – |
|  | instructions | **144** | 221 | – | – | – | – | – | – | – |
| P-256 base | throughput cycles | **49.3** | 66.7 | – | – | – | – | – | – | 597 |
|  | latency cycles | **56.0** | 64.2 | – | – | – | – | – | – | – |
|  | instructions | **175** | 185 | – | – | – | – | – | – | 2,378 |
| P-256 scalar | throughput cycles | **76.8** | 93.5 | – | – | – | – | – | – | 664 |
|  | latency cycles | **82.1** | 91.1 | – | – | – | – | – | – | – |
|  | instructions | 228 | **224** | – | – | – | – | – | – | 2,385 |
| BLS12-381 base | throughput cycles | **156** | 167 | – | – | – | – | – | – | – |
|  | latency cycles | **156** | 161 | – | – | – | – | – | – | – |
|  | instructions | **444** | 502 | – | – | – | – | – | – | – |
| P-521 base | throughput cycles | **195** | – | – | – | – | – | – | – | – |
|  | latency cycles | **212** | – | – | – | – | – | – | – | – |
|  | instructions | **554** | – | – | – | – | – | – | – | – |

### mul_ct: a × b, constant time

![mul_ct: a × b, constant time](benchmarks/mul_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.9** | – | 17.1 | – | – | – | – | 7.0 | – |
|  | latency cycles | **7.0** | – | 35.3 | – | – | – | – | 17.0 | – |
|  | instructions | **14.0** | – | 40.0 | – | – | – | – | 30.0 | – |
| BabyBear | throughput cycles | **3.0** | – | 16.5 | – | – | – | – | 7.0 | – |
|  | latency cycles | **11.0** | – | 35.3 | – | – | – | – | 17.0 | – |
|  | instructions | **13.0** | – | 40.0 | – | – | – | – | 30.0 | – |
| Goldilocks | throughput cycles | 4.3 | – | 18.3 | – | – | – | **4.0** | 7.0 | – |
|  | latency cycles | **9.0** | – | 35.1 | – | – | – | 11.0 | 17.0 | – |
|  | instructions | **18.0** | – | 44.0 | – | – | – | **18.0** | 30.0 | – |
| BN254 scalar | throughput cycles | **71.5** | – | 124 | – | 91.0 | – | – | 93.0 | – |
|  | latency cycles | **74.1** | – | 166 | – | 99.1 | – | – | 127 | – |
|  | instructions | **212** | – | 354 | – | 248 | – | – | 304 | – |
| secp256k1 base | throughput cycles | **44.0** | – | 125 | 100 | 88.9 | – | – | 93.0 | – |
|  | latency cycles | **56.1** | – | 168 | 89.0 | 93.1 | – | – | 127 | – |
|  | instructions | **144** | – | 354 | 248 | 248 | – | – | 304 | – |
| P-256 base | throughput cycles | **49.5** | – | 124 | 72.5 | 69.4 | – | – | 53.2 | – |
|  | latency cycles | **56.1** | – | 167 | 69.2 | 73.1 | – | – | 56.3 | – |
|  | instructions | **175** | – | 353 | 206 | 200 | – | – | 178 | – |
| P-256 scalar | throughput cycles | **76.8** | – | 124 | 176 | 90.7 | – | – | 93.1 | – |
|  | latency cycles | **82.1** | – | 166 | 151 | 102 | – | – | 127 | – |
|  | instructions | **228** | – | 354 | 430 | 253 | – | – | 304 | – |
| BLS12-381 base | throughput cycles | **160** | – | 287 | – | 191 | – | – | 256 | – |
|  | latency cycles | **159** | – | 302 | – | 193 | – | – | 252 | – |
|  | instructions | **466** | – | 758 | – | 501 | – | – | 684 | – |
| P-521 base | throughput cycles | **195** | – | 570 | – | – | – | – | 464 | – |
|  | latency cycles | **212** | – | 625 | – | – | – | – | 496 | – |
|  | instructions | **554** | – | 1,522 | – | – | – | – | 1,251 | – |

### square_vartime: a², variable time

![square_vartime: a², variable time](benchmarks/square_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.8** | – | – | – | – | 2.8 | – | – | – |
|  | latency cycles | **7.0** | – | – | – | – | 8.0 | – | – | – |
|  | instructions | **14.0** | – | – | – | – | 14.0 | – | – | – |
| BabyBear | throughput cycles | **3.0** | – | – | – | – | 5.0 | – | – | – |
|  | latency cycles | **11.0** | – | – | – | – | 14.0 | – | – | – |
|  | instructions | **13.0** | – | – | – | – | 16.0 | – | – | – |
| Goldilocks | throughput cycles | **4.3** | – | – | – | – | 3.6\* | – | – | – |
|  | latency cycles | **9.0** | – | – | – | – | 10.0 | – | – | – |
|  | instructions | **18.0** | – | – | – | – | 18.0 | – | – | – |
| BN254 scalar | throughput cycles | **64.2** | 74.9 | – | – | – | – | – | – | – |
|  | latency cycles | **69.1** | 72.1 | – | – | – | – | – | – | – |
|  | instructions | **175** | 192 | – | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **32.7** | 82.8 | – | – | – | – | – | – | – |
|  | latency cycles | **44.2** | 80.4 | – | – | – | – | – | – | – |
|  | instructions | **104** | 211 | – | – | – | – | – | – | – |
| P-256 base | throughput cycles | **39.2** | 65.6 | – | – | – | – | – | – | – |
|  | latency cycles | **48.3** | 59.5 | – | – | – | – | – | – | – |
|  | instructions | **140** | 176 | – | – | – | – | – | – | – |
| P-256 scalar | throughput cycles | **63.0** | 89.9 | – | – | – | – | – | – | – |
|  | latency cycles | **73.0** | 85.0 | – | – | – | – | – | – | – |
|  | instructions | **187** | 217 | – | – | – | – | – | – | – |
| BLS12-381 base | throughput cycles | **135** | 144 | – | – | – | – | – | – | – |
|  | latency cycles | **138** | 138 | – | – | – | – | – | – | – |
|  | instructions | **383** | 403 | – | – | – | – | – | – | – |
| P-521 base | throughput cycles | **122** | – | – | – | – | – | – | – | – |
|  | latency cycles | **137** | – | – | – | – | – | – | – | – |
|  | instructions | **365** | – | – | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### square_ct: a², constant time

![square_ct: a², constant time](benchmarks/square_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **2.8** | – | 17.0 | – | – | – | – | – | – |
|  | latency cycles | **7.0** | – | 37.1 | – | – | – | – | – | – |
|  | instructions | **14.0** | – | 40.0 | – | – | – | – | – | – |
| BabyBear | throughput cycles | **3.0** | – | 17.0 | – | – | – | – | – | – |
|  | latency cycles | **11.0** | – | 36.6 | – | – | – | – | – | – |
|  | instructions | **13.0** | – | 40.0 | – | – | – | – | – | – |
| Goldilocks | throughput cycles | 4.3 | – | 18.2 | – | – | – | **3.9** | – | – |
|  | latency cycles | **9.0** | – | 34.9 | – | – | – | 11.0 | – | – |
|  | instructions | **18.0** | – | 44.0 | – | – | – | **18.0** | – | – |
| BN254 scalar | throughput cycles | **65.1** | – | 112 | – | 77.0 | – | – | – | – |
|  | latency cycles | **75.0** | – | 154 | – | 83.0 | – | – | – | – |
|  | instructions | **187** | – | 315 | – | 207 | – | – | – | – |
| secp256k1 base | throughput cycles | **32.7** | – | 119 | 72.9 | 70.4 | – | – | – | – |
|  | latency cycles | **44.2** | – | 153 | 88.0 | 81.0 | – | – | – | – |
|  | instructions | **104** | – | 315 | 177 | 220 | – | – | – | – |
| P-256 base | throughput cycles | **39.1** | – | 116 | 63.7 | 49.9 | – | – | – | – |
|  | latency cycles | **48.0** | – | 157 | 70.0 | 60.0 | – | – | – | – |
|  | instructions | **140** | – | 314 | 201 | 175 | – | – | – | – |
| P-256 scalar | throughput cycles | **63.0** | – | 113 | 141 | 81.1 | – | – | – | – |
|  | latency cycles | **73.0** | – | 153 | 147 | 89.0 | – | – | – | – |
|  | instructions | **187** | – | 315 | 424 | 231 | – | – | – | – |
| BLS12-381 base | throughput cycles | **141** | – | 290 | – | 149 | – | – | – | – |
|  | latency cycles | **138** | – | 298 | – | 164 | – | – | – | – |
|  | instructions | **408** | – | 762 | – | 442 | – | – | – | – |
| P-521 base | throughput cycles | **122** | – | 570 | – | – | – | – | – | – |
|  | latency cycles | **136** | – | 624 | – | – | – | – | – | – |
|  | instructions | **365** | – | 1,521 | – | – | – | – | – | – |

### invert_vartime: a⁻¹, variable time

![invert_vartime: a⁻¹, variable time](benchmarks/invert_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **224** | – | 1,430 | – | – | 648 | – | 905 | – |
|  | instructions | **179** | – | 2,429 | – | – | 557 | – | 993 | – |
| BabyBear | throughput cycles | **222** | – | 1,408 | – | – | 634 | – | 1,065 | – |
|  | instructions | **180** | – | 2,373 | – | – | 555 | – | 984 | – |
| Goldilocks | throughput cycles | **438** | – | 2,863 | – | – | 1,353 | – | 1,744 | – |
|  | instructions | **511** | – | 4,513 | – | – | 1,200 | – | 1,985 | – |
| BN254 scalar | throughput cycles | **5,705** | 16,237 | 14,403 | – | 6,902 | – | – | 15,835 | – |
|  | instructions | **14,817** | 25,928 | 24,812 | – | 16,840 | – | – | 33,141 | – |
| secp256k1 base | throughput cycles | **5,646** | 15,392 | 14,503 | – | 6,924 | – | – | 16,032 | – |
|  | instructions | **14,548** | 27,255 | 24,841 | – | 16,907 | – | – | 33,487 | – |
| P-256 base | throughput cycles | **5,520** | 15,961 | 14,465 | – | 6,927 | – | – | 15,607 | 94,410 |
|  | instructions | **14,204** | 24,520 | 24,832 | – | 16,896 | – | – | 33,268 | 357,678 |
| P-256 scalar | throughput cycles | **5,734** | 16,568 | 14,510 | 32,958 | 6,930 | – | – | 16,028 | 95,327 |
|  | instructions | **15,007** | 24,107 | 24,837 | 61,478 | 16,893 | – | – | 33,457 | 358,020 |
| BLS12-381 base | throughput cycles | **10,066** | 32,896 | 25,742 | – | 11,776 | – | – | 33,987 | – |
|  | instructions | **27,033** | 59,327 | 44,561 | – | 29,291 | – | – | 78,843 | – |
| P-521 base | throughput cycles | **16,205** | – | 45,254 | – | – | – | – | 66,178 | – |
|  | instructions | **44,153** | – | 73,771 | – | – | – | – | 174,674 | – |

### invert_ct: a⁻¹, constant time

![invert_ct: a⁻¹, constant time](benchmarks/invert_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **262** | – | 3,860 | – | – | – | – | – | – |
|  | instructions | **496** | – | 15,956 | – | – | – | – | – | – |
| BabyBear | throughput cycles | **387** | – | 3,891 | – | – | – | – | – | – |
|  | instructions | **543** | – | 15,956 | – | – | – | – | – | – |
| Goldilocks | throughput cycles | **656** | – | 3,845 | – | – | – | 687 | – | – |
|  | instructions | 1,081 | – | 15,958 | – | – | – | **825** | – | – |
| BN254 scalar | throughput cycles | **6,826** | – | 19,433 | – | – | – | – | – | – |
|  | instructions | **27,910** | – | 70,410 | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **6,909** | – | 19,540 | 24,614 | – | – | – | – | – |
|  | instructions | **27,830** | – | 70,410 | 47,559 | – | – | – | – | – |
| P-256 base | throughput cycles | **6,885** | – | 19,479 | 29,497 | – | – | – | – | – |
|  | instructions | **27,909** | – | 70,409 | 52,761 | – | – | – | – | – |
| P-256 scalar | throughput cycles | **6,924** | – | 19,470 | 82,729 | – | – | – | – | – |
|  | instructions | **28,040** | – | 70,410 | 179,737 | – | – | – | – | – |
| BLS12-381 base | throughput cycles | **12,930** | – | 34,383 | – | – | – | – | – | – |
|  | instructions | **42,306** | – | 115,421 | – | – | – | – | – | – |
| P-521 base | throughput cycles | **20,097** | – | 65,807 | – | – | – | – | – | – |
|  | instructions | **68,386** | – | 190,060 | – | – | – | – | – | – |

### sqrt_vartime: √a, variable time

![sqrt_vartime: √a, variable time](benchmarks/sqrt_vartime.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **139** | – | – | – | – | 143 | – | – | – |
|  | instructions | **224** | – | – | – | – | 230 | – | – | – |
| BabyBear | throughput cycles | **2,060** | – | – | – | – | 2,857 | – | – | – |
|  | instructions | **2,307** | – | – | – | – | 4,480 | – | – | – |
| Goldilocks | throughput cycles | **3,064** | – | – | – | – | 3,400 | – | – | – |
|  | instructions | **6,777** | – | – | – | – | 5,604\* | – | – | – |
| BN254 scalar | throughput cycles | **33,939** | 45,047 | – | – | – | – | – | – | – |
|  | instructions | **85,724** | 115,786 | – | – | – | – | – | – | – |
| secp256k1 base | throughput cycles | **12,480** | 43,030 | – | – | – | – | – | – | – |
|  | instructions | **24,034** | 107,630 | – | – | – | – | – | – | – |
| P-256 base | throughput cycles | **11,896** | 18,342 | – | – | – | – | – | – | – |
|  | instructions | **33,268** | 52,524 | – | – | – | – | – | – | – |
| P-256 scalar | throughput cycles | **23,759** | 39,532 | – | – | – | – | – | – | – |
|  | instructions | **56,724** | 93,875 | – | – | – | – | – | – | – |
| BLS12-381 base | throughput cycles | **65,055** | 90,502 | – | – | – | – | – | – | – |
|  | instructions | **176,944** | 265,886 | – | – | – | – | – | – | – |
| P-521 base | throughput cycles | **71,644** | – | – | – | – | – | – | – | – |
|  | instructions | **171,685** | – | – | – | – | – | – | – | – |

\* `plonky3` over Goldilocks: skips the final reduction; the result is not canonical.

### sqrt_ct: √a, constant time

![sqrt_ct: √a, constant time](benchmarks/sqrt_ct.svg)

| Field | Per operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | throughput cycles | **143** | – | 1,654 | – | – | – | – | – | – |
|  | instructions | **236** | – | 1,973 | – | – | – | – | – | – |
| BabyBear | throughput cycles | **5,580** | – | 18,705 | – | – | – | – | – | – |
|  | instructions | **8,097** | – | 33,620 | – | – | – | – | – | – |
| Goldilocks | throughput cycles | **7,154** | – | 27,028 | – | – | – | – | – | – |
|  | instructions | **15,008** | – | 47,926 | – | – | – | – | – | – |
| BN254 scalar | throughput cycles | **58,427** | – | 134,420 | – | 106,041 | – | – | – | – |
|  | instructions | **153,528** | – | 289,275 | – | 226,527 | – | – | – | – |
| secp256k1 base | throughput cycles | **12,060** | – | 53,278 | 24,306 | 45,737 | – | – | – | – |
|  | instructions | **24,101** | – | 123,520 | 46,986 | 116,568 | – | – | – | – |
| P-256 base | throughput cycles | **12,441** | – | 43,468 | 29,466 | 34,918 | – | – | – | – |
|  | instructions | **33,273** | – | 104,904 | 51,581 | 90,140 | – | – | – | – |
| P-256 scalar | throughput cycles | **25,493** | – | 52,750 | 86,078 | 41,719 | – | – | – | – |
|  | instructions | **59,570** | – | 121,877 | 188,053 | 100,484 | – | – | – | – |
| BLS12-381 base | throughput cycles | **67,414** | – | 133,836 | – | 139,796 | – | – | – | – |
|  | instructions | **187,782** | – | 341,926 | – | 398,850 | – | – | – | – |
| P-521 base | throughput cycles | **70,543** | – | 268,613 | – | – | – | – | – | – |
|  | instructions | **178,208** | – | 767,496 | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # the CPU's counts of the native build
cargo run -p prime-field-bench-report -- report   # this file and benchmarks/*.svg
```
