# Benchmarks

`prime-field` at commit `5bbb9f2` against `ark-ff`, `crypto-bigint`, `k256/p256`, `halo2curves`, `plonky3`, `winter-math`, `runtime-algebra` and `compile-algebra`: add, sub, mul, square, invert, invert_vartime and sqrt over Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base, in instructions per operation, counted on 2026-11-02. The counted files had uncommitted changes.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile, `-C target-cpu=native -C target-feature=-sse4a` |
| Counter | Valgrind 3.22.0, Callgrind |
| Crates | ark-ff 0.6.0, crypto-bigint 0.5.5, k256 0.13.4, p256 0.13.2, halo2curves 0.10.0, p3-field 0.9.0-rc.2, winter-math 0.13.1, runtime-algebra 0.1.0, compile-algebra 0.1.0 |

## Method

Each case runs one crate's function over 256 operand sets, the same canonical residues for every crate in a row, with operands and results behind `core::hint::black_box`. Callgrind counts the instructions of the loop over the batch, including the functions it calls, such as the allocator of `compile-algebra`; each count is that total divided by 256. The loop's own loads and stores are part of every count. Every instruction counts as one, whatever its latency. The counts are exact at any machine load. Before a case is counted, every result is checked against `num-bigint`: the exact residue, `a^(p - 2)` for the inverses, and either root for `sqrt`. Elements enter and leave each crate in its own representation; the conversions lie outside the count.

| Operation | Operands |
|---|---|
| add, sub, mul | `a` and `b` uniform below `p` |
| square, invert, invert_vartime | `a` uniform in `[1, p)` |
| sqrt | `a = r²` for `r` uniform in `[1, p)`, so that a root exists |

## Functions

Each cell counts the crate's own function:

| Operation | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---|---|---|---|---|---|---|---|---|---|
| add | `+` | `+` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `+` | `+` over secp256k1 base, P-256 base and P-256 scalar | `+` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `+` over Mersenne-31, BabyBear and Goldilocks | `+` over Goldilocks | `addf` | `addf` over P-256 base and P-256 scalar |
| sub | `-` | `-` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `-` | `-` over secp256k1 base, P-256 base and P-256 scalar | `-` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `-` over Mersenne-31, BabyBear and Goldilocks | `-` over Goldilocks | `subf` | `subf` over P-256 base and P-256 scalar |
| mul | `*` | `*` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `*` | `*` over secp256k1 base, P-256 base and P-256 scalar | `*` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `*` over Mersenne-31, BabyBear and Goldilocks | `*` over Goldilocks | `mulf` | `mulf` over P-256 base and P-256 scalar |
| square | `square` | `square` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `square` | `square` over secp256k1 base, P-256 base and P-256 scalar | `square` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `square` over Mersenne-31, BabyBear and Goldilocks | `square` over Goldilocks | – | – |
| invert | `invert` | – | `invert` | `invert` over secp256k1 base, P-256 base and P-256 scalar | `invert` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `try_inverse` over Mersenne-31, BabyBear and Goldilocks | `inv` over Goldilocks | – | – |
| invert_vartime | `invert_vartime` | `inverse` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `invert_vartime` | – | – | – | – | `invert` | `invert` over P-256 base and P-256 scalar |
| sqrt | `sqrt` | `sqrt` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `sqrt` | `sqrt` over secp256k1 base, P-256 base and P-256 scalar | `sqrt` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base | `tonelli_shanks` over Mersenne-31, BabyBear and Goldilocks | – | – | – |

- `prime-field`, `ark-ff`, `crypto-bigint`, `halo2curves` and `runtime-algebra` are generic over the modulus; `k256`, `p256`, `p521`, `p3-*` and `winter-math` are written for their field.
- `crypto-bigint` is `ConstMontyForm` with a `const_monty_params!` modulus.
- `ark-ff` is `Fp` with `MontBackend` from the arkworks curve crates; its `inverse` and `sqrt` run in variable time.
- `plonky3` is `p3-mersenne-31`, `p3-baby-bear` and `p3-goldilocks` on their field; its inverse is a Fermat exponentiation.
- `runtime-algebra` and `compile-algebra` are longfellow-zk's prover-side and compile-side fields: `FpGenericField` is their generic Montgomery field with the modulus held at run time; `P256Field` adds a hand-written P-256 strategy; `compile-algebra`'s `FpField` computes on `num-bigint`.
- No arkworks crate declares the field: `ark-ff` over Mersenne-31, BabyBear, Goldilocks and P-521 base.
- The inverse runs in variable time: `ark-ff` invert over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base and `runtime-algebra` invert.
- No RustCrypto crate declares the field: `k256/p256` over Mersenne-31, BabyBear, Goldilocks, BN254 scalar and BLS12-381 base.
- P521 keeps its field element private: `k256/p256` over P-521 base.
- No variable-time inverse: `k256/p256` invert_vartime over secp256k1 base, P-256 base and P-256 scalar, `halo2curves` invert_vartime over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar and BLS12-381 base, `plonky3` invert_vartime over Mersenne-31, BabyBear and Goldilocks and `winter-math` invert_vartime over Goldilocks.
- Halo2curves declares no such field: `halo2curves` over Mersenne-31, BabyBear, Goldilocks and P-521 base.
- Plonky3 declares no such field: `plonky3` over BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base.
- Winter-math declares no such field: `winter-math` over Mersenne-31, BabyBear, BN254 scalar, secp256k1 base, P-256 base, P-256 scalar, BLS12-381 base and P-521 base.
- No square root: `winter-math` sqrt over Goldilocks, `runtime-algebra` sqrt and `compile-algebra` sqrt over P-256 base and P-256 scalar.
- No squaring function: `runtime-algebra` square and `compile-algebra` square over P-256 base and P-256 scalar.
- Compile-algebra declares P-256 base and scalar only: `compile-algebra` over Mersenne-31, BabyBear, Goldilocks, BN254 scalar, secp256k1 base, BLS12-381 base and P-521 base.
- The inverse is a variable-time `modpow`: `compile-algebra` invert over P-256 base and P-256 scalar.

## Results

Each cell: instructions per operation. Bold marks the fewest in each row.
The charts show the first 8 crates; `compile-algebra` appears in the tables only.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 11.0 | – | 38.1 | – | – | **10.0** | – | 16.0 | – |
| BabyBear | 12.0 | – | 38.1 | – | – | **11.0** | – | 16.0 | – |
| Goldilocks | 15.0 | – | 39.1 | – | – | **12.0** | 16.0 | 16.0 | – |
| BN254 scalar | 44.1 | **25.7** | 130 | – | 57.1 | – | – | 36.1 | – |
| secp256k1 base | 53.1 | 30.0 | 130 | **11.0** | 58.1 | – | – | 36.1 | – |
| P-256 base | 50.6 | **28.6** | 130 | 59.1 | 46.1 | – | – | 36.1 | 1,439 |
| P-256 scalar | 55.1 | **29.1** | 130 | 85.1 | 61.1 | – | – | 36.1 | 1,418 |
| BLS12-381 base | 84.0 | 78.8 | 180 | – | 88.1 | – | – | **67.1** | – |
| P-521 base | 112 | – | 265 | – | – | – | – | **106** | – |

### sub: a − b

![sub: a − b](benchmarks/sub.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 14.0 | – | 19.0 | – | – | **10.0** | – | 13.0 | – |
| BabyBear | 14.0 | – | 19.0 | – | – | **10.0** | – | 13.0 | – |
| Goldilocks | **10.0** | – | 19.0 | – | – | 12.0 | **10.0** | 13.0 | – |
| BN254 scalar | 36.2 | 35.3 | 85.1 | – | 45.1 | – | – | **35.1** | – |
| secp256k1 base | 27.7 | 35.0 | 85.1 | **13.0** | 39.1 | – | – | 35.1 | – |
| P-256 base | **29.3** | 30.8 | 85.1 | 53.1 | 40.1 | – | – | 35.1 | 632 |
| P-256 scalar | **30.7** | 33.6 | 85.1 | 54.1 | 43.1 | – | – | 35.1 | 626 |
| BLS12-381 base | 67.8 | 67.2 | 123 | – | 74.1 | – | – | **66.1** | – |
| P-521 base | **55.9** | – | 181 | – | – | – | – | 108 | – |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 29.0 | – | 40.1 | – | – | **15.0** | – | 31.1 | – |
| BabyBear | 17.1 | – | 40.1 | – | – | **17.0** | – | 31.1 | – |
| Goldilocks | 20.1 | – | 44.1 | – | – | **18.0** | 18.0 | 31.1 | – |
| BN254 scalar | 333 | **232** | 363 | – | 254 | – | – | 302 | – |
| secp256k1 base | 431 | **229** | 363 | 244 | 251 | – | – | 302 | – |
| P-256 base | 289 | **189** | 363 | 204 | 204 | – | – | 210 | 2,438 |
| P-256 scalar | 372 | **231** | 363 | 428 | 263 | – | – | 302 | 2,445 |
| BLS12-381 base | 673 | **498** | 746 | – | 532 | – | – | 687 | – |
| P-521 base | **733** | – | 1,498 | – | – | – | – | 1,230 | – |

### square: a²

![square: a²](benchmarks/square.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 90.1 | – | 40.1 | – | – | **14.0** | – | – | – |
| BabyBear | 81.1 | – | 40.1 | – | – | **16.0** | – | – | – |
| Goldilocks | 85.1 | – | 44.1 | – | – | **18.0** | 18.0 | – | – |
| BN254 scalar | 514 | **185** | 323 | – | 217 | – | – | – | – |
| secp256k1 base | 462 | 206 | 323 | **173** | 233 | – | – | – | – |
| P-256 base | 498 | **168** | 323 | 198 | 184 | – | – | – | – |
| P-256 scalar | 529 | **213** | 323 | 421 | 243 | – | – | – | – |
| BLS12-381 base | 1,026 | **401** | 747 | – | 450 | – | – | – | – |
| P-521 base | **1,480** | – | 1,483 | – | – | – | – | – | – |

### invert: a⁻¹, constant time

![invert: a⁻¹, constant time](benchmarks/invert.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 2,195 | – | 16,049 | – | – | **556** | – | – | – |
| BabyBear | 2,190 | – | 16,049 | – | – | **554** | – | – | – |
| Goldilocks | 5,561 | – | 16,051 | – | – | 1,201 | **825** | – | – |
| BN254 scalar | 28,191 | – | 70,349 | – | **17,208** | – | – | – | – |
| secp256k1 base | 27,461 | – | 70,349 | 47,560 | **17,225** | – | – | – | – |
| P-256 base | 27,595 | – | 70,348 | 52,756 | **17,273** | – | – | – | – |
| P-256 scalar | 27,767 | – | 70,349 | 179,604 | **17,335** | – | – | – | – |
| BLS12-381 base | 54,593 | – | 115,623 | – | **30,045** | – | – | – | – |
| P-521 base | **93,817** | – | 188,473 | – | – | – | – | – | – |

### invert_vartime: a⁻¹, variable time

![invert_vartime: a⁻¹, variable time](benchmarks/invert_vartime.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 1,191 | – | 2,469 | – | – | – | – | **1,062** | – |
| BabyBear | 1,186 | – | 2,435 | – | – | – | – | **1,064** | – |
| Goldilocks | 2,179 | – | 4,577 | – | – | – | – | **2,131** | – |
| BN254 scalar | **15,829** | 24,120 | 24,779 | – | – | – | – | 33,637 | – |
| secp256k1 base | **14,892** | 25,608 | 24,786 | – | – | – | – | 34,025 | – |
| P-256 base | **15,152** | 24,322 | 24,784 | – | – | – | – | 33,834 | 357,137 |
| P-256 scalar | **15,490** | 23,458 | 24,796 | – | – | – | – | 33,964 | 357,614 |
| BLS12-381 base | **30,135** | 58,510 | 44,686 | – | – | – | – | 94,003 | – |
| P-521 base | **55,552** | – | 72,645 | – | – | – | – | 206,553 | – |

### sqrt: √a

![sqrt: √a](benchmarks/sqrt.svg)

| Field | prime-field | ark-ff | crypto-bigint | k256/p256 | halo2curves | plonky3 | winter-math | runtime-algebra | compile-algebra |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Mersenne-31 | 6,500 | – | 2,032 | – | – | **1,869** | – | – | – |
| BabyBear | 22,117 | – | 34,518 | – | – | **4,673** | – | – | – |
| Goldilocks | 30,281 | – | 49,082 | – | – | **8,158** | – | – | – |
| BN254 scalar | 342,834 | **116,356** | 290,231 | – | 232,852 | – | – | – | – |
| secp256k1 base | 232,952 | 107,893 | 121,475 | **46,980** | 115,287 | – | – | – | – |
| P-256 base | 205,924 | 52,496 | 102,495 | **51,576** | 88,858 | – | – | – | – |
| P-256 scalar | 239,246 | **95,495** | 119,957 | 187,781 | 101,056 | – | – | – | – |
| BLS12-381 base | 656,226 | **267,181** | 347,908 | – | 373,122 | – | – | – | – |
| P-521 base | 1,268,745 | – | **748,951** | – | – | – | – | – | – |

## Reproduce

```sh
cargo run -p prime-field-bench-report -- bench    # Callgrind counts of the native build
cargo run -p prime-field-bench-report -- report   # this file and benchmarks/*.svg
```
