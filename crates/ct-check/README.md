# ct-check

Checks that the constant-time operations of `prime-field` and `binary-field` never branch on a
secret operand and never compute a memory address from one.

## Usage

```sh
cargo run -p ct-check
```

The check builds the operations for two target CPUs and runs each build under Valgrind's memcheck.
It prints one line per build. It fails when memcheck reports an error, which memcheck prints with
its stack. The check needs `valgrind` in `PATH` on x86-64 Linux. A memcheck run takes under a
second.

## Method

A client request marks the operands undefined before the operations run, as in libsecp256k1's
`ctime_tests` and BoringSSL's constant-time validation. Memcheck then tracks every bit computed from
them and reports two kinds of error:

- "Conditional jump or move depends on uninitialised value(s)": a branch on a secret;
- "Use of uninitialised value of size 8": a memory address computed from a secret.

The origin of each report names the client request. Memcheck reports a dependent branch whichever
way it goes. One run of each operation therefore covers it. Memcheck propagates undefined bits
through arithmetic and through `cmov`. A conditional move on a secret passes, as does an
instruction whose latency depends on its operands, such as a division.

## Coverage

- `prime-field`: every operation of `Secret` (the sum, the difference and the product with a
  `Secret` or a public `Fp` on either side and their assigning forms, `neg`, `square`, `double`,
  `pow` with a public exponent, `invert` and `sqrt`), over the nine fields of its `compare` bench.
- `binary-field`: the sum, the product, `square`, `sqrt`, `trace`, `times_x`, `over_x` and `invert`,
  over the seven fields of its `compare` bench. `invert` runs in the x86-64-v3 build alone. Without
  hardware carry-less products, every one of these fields inverts by the Euclidean algorithm, whose
  time depends on the value.

| Build | `RUSTFLAGS` |
|---|---|
| `x86-64` | `-C target-cpu=x86-64` |
| `x86-64-v3` | `-C target-cpu=x86-64-v3 -C target-feature=+adx,+pclmulqdq` |

Because the flags name the CPU, the CPU of the build machine changes no code. Each build is the
release profile in `target/ct-check/<build>`, with line tables that name the inlined functions in
memcheck's reports. The line tables change the machine code of `main` alone. The operations inline
into each caller. The check covers the code of its own calls.

## The exemption

The `Option` that `invert` returns discloses whether the input is zero, and the one of
`prime-field`'s `sqrt` whether the input is a square. Each library builds that `Option` in
`invert::declassify`, a function with a symbol of its own. `memcheck.supp` exempts a
conditional jump whose innermost frame is that function. In the `compare` benches the boundary
changes an inversion by −150 to +63 instructions, at most 1.9%. Because LLVM merges the instances
of `declassify` that compile to the same machine code, a report may name the other library's.
