# Benchmarks

`biguint` at commit `d068959` against `native`, `ruint`, `crypto-bigint`, `bnum`, `uint` and `num-bigint`: add, mul, div, mod and mulmod at 64, 128, 256, 384, 512, 1024, 2048 and 4096 bits, in instructions per operation, counted on 2026-11-02.

## Setup

|  |  |
|---|---|
| CPU | AMD Ryzen 9 5950X 16-Core Processor (`znver3`) |
| Compiler | rustc 1.99.1 (58cf8f9c3 2026-10-13), LLVM 23.1.1, `bench` profile, `-C target-cpu=native -C target-feature=-sse4a,-vpclmulqdq -C codegen-units=1` |
| Counter | Valgrind 3.22.0, Callgrind |
| Crates | ruint 1.21.1, crypto-bigint 0.7.10, bnum 0.14.4, uint 0.10.2, num-bigint 0.5.1 |

## Method

Each case runs one crate's function over 256 operand sets, the same for every crate in a row, with operands and results behind `core::hint::black_box`. Callgrind counts the instructions of the loop over the batch, including the functions it calls, such as the allocator of `num-bigint`; each count is that total divided by 256. The loop's own loads and stores are part of every count. Every instruction counts as one, whatever its latency: a hardware divide weighs as much as an addition. The counts are exact at any machine load. Every result equals `num-bigint`'s, reduced modulo 2^N for the crates that wrap.

| Operation | Operands at width N |
|---|---|
| add, mul | `a` and `b` uniform below 2^N |
| div, mod | a dividend with bit N − 1 set; a divisor below 2^(N/2) with bit N/2 − 1 set, held in an N-bit integer |
| mulmod | a modulus `m` with bit N − 1 set; `a` and `b` uniform below `m` |

## Functions

Each cell counts the crate's own function:

| Operation | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---|---|---|---|---|---|---|---|
| add | `+` | `wrapping_add` at 64 and 128 bits | `wrapping_add` | `wrapping_add` | `wrapping_add` | `overflowing_add` | `&a + &b` |
| mul | `*` | `wrapping_mul` at 64 and 128 bits | `wrapping_mul` | `wrapping_mul` | `wrapping_mul` | `overflowing_mul` | `&a * &b` |
| div | `div_rem` | `/ NonZero` at 64 and 128 bits | `wrapping_div` | `wrapping_div_vartime` | `wrapping_div` | `/` | `&a / &b` |
| mod | `div_rem` | `% NonZero` at 64 and 128 bits | `wrapping_rem` | `rem_vartime` | `wrapping_rem` | `%` | `&a % &b` |
| mulmod | `mul_mod` | – | `mul_mod` | `mul_mod_vartime` | – | – | – |

- `native` is `u64` at 64 bits and `u128` at 128 bits.
- `crypto-bigint` is constant-time; its div, mod and mulmod cases run its `*_vartime` functions.
- `num-bigint` allocates each result.
- `uint` is `construct_uint!`, as in `primitive-types`.
- No primitive of this width: `native` at 256, 384, 512, 1024, 2048 and 4096 bits.
- No modular product: `native` mulmod at 64 and 128 bits, `bnum` mulmod, `uint` mulmod and `num-bigint` mulmod.

## Results

Each cell: instructions per operation. Bold marks the fewest in each row.

### add: a + b

![add: a + b](benchmarks/add.svg)

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | 199 |
| 128 | **11.0** | **11.0** | **11.0** | **11.0** | **11.0** | **11.0** | 502 |
| 256 | **17.0** | – | **17.0** | **17.0** | 19.0 | 31.9 | 553 |
| 384 | **23.1** | – | **23.1** | 36.1 | 31.1 | 53.1 | 541 |
| 512 | **29.1** | – | **29.1** | 52.1 | 43.1 | 84.9 | 619 |
| 1024 | **66.1** | – | 74.1 | 104 | 103 | 187 | 594 |
| 2048 | **143** | – | 167 | 211 | 244 | 396 | 693 |
| 4096 | **406** | – | 472 | 463 | 600 | 806 | 814 |

### mul: a × b

![mul: a × b](benchmarks/mul.svg)

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | **8.0** | 276 |
| 128 | **14.0** | 16.0 | **14.0** | **14.0** | 16.0 | **14.0** | 611 |
| 256 | **46.1** | – | 47.1 | 47.1 | 51.1 | 57.1 | 756 |
| 384 | 128 | – | 118 | **112** | 128 | 153 | 1,092 |
| 512 | 232 | – | **218** | 335 | 254 | 369 | 1,417 |
| 1024 | **988** | – | 1,496 | 1,370 | 1,255 | 1,478 | 3,255 |
| 2048 | **3,614** | – | 4,406 | 5,722 | 4,369 | 7,454 | 9,967 |
| 4096 | **12,008** | – | 14,786 | 22,297 | 16,511 | 33,161 | 31,242 |

### div: a / b

![div: a / b](benchmarks/div.svg)

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 20.0 | **14.0** | 16.0 | 451 | 23.0 | 21.0 | 144 |
| 128 | 31.1 | **26.1** | 58.6 | 518 | 86.1 | 85.1 | 342 |
| 256 | 212 | – | **201** | 1,312 | 540 | 558 | 1,075 |
| 384 | **427** | – | 592 | 1,789 | 766 | 775 | 1,244 |
| 512 | **564** | – | 769 | 2,262 | 1,194 | 1,050 | 1,375 |
| 1024 | **1,328** | – | 1,659 | 4,908 | 3,134 | 2,359 | 2,278 |
| 2048 | **3,362** | – | 4,158 | 13,654 | 7,928 | 6,564 | 5,235 |
| 4096 | **10,442** | – | 12,291 | 45,588 | 29,214 | 27,204 | 15,040 |

### mod: a mod b

![mod: a mod b](benchmarks/mod.svg)

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 20.0 | **14.0** | 16.0 | 451 | 23.1 | 21.0 | 49.1 |
| 128 | 34.4 | **29.4** | 58.8 | 518 | 86.1 | 85.1 | 367 |
| 256 | 212 | – | **203** | 1,312 | 540 | 558 | 1,099 |
| 384 | **428** | – | 573 | 1,789 | 766 | 775 | 1,269 |
| 512 | **567** | – | 749 | 2,262 | 1,194 | 1,050 | 1,399 |
| 1024 | **1,325** | – | 1,598 | 4,908 | 3,133 | 2,358 | 2,303 |
| 2048 | **3,366** | – | 4,110 | 13,654 | 7,928 | 6,564 | 5,259 |
| 4096 | **10,437** | – | 12,241 | 45,588 | 29,214 | 27,207 | 15,069 |

### mulmod: a × b mod m

![mulmod: a × b mod m](benchmarks/mulmod.svg)

| Bits | biguint | native | ruint | crypto-bigint | bnum | uint | num-bigint |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | **30.1** | – | 84.1 | 494 | – | – | – |
| 128 | **188** | – | 191 | 971 | – | – | – |
| 256 | 449 | – | **439** | 2,296 | – | – | – |
| 384 | **788** | – | 830 | 3,810 | – | – | – |
| 512 | **1,181** | – | 1,250 | 5,749 | – | – | – |
| 1024 | **3,422** | – | 3,929 | 18,252 | – | – | – |
| 2048 | **12,893** | – | 16,343 | 65,542 | – | – | – |
| 4096 | **44,604** | – | 63,913 | 242,216 | – | – | – |

## Reproduce

```sh
cargo run -p biguint-bench-report -- bench    # Callgrind counts of the native build
cargo run -p biguint-bench-report -- report   # this file and benchmarks/*.svg
```
