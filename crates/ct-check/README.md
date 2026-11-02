# ct-check

Checks that the constant-time operations of `prime-field` and `binary-field` never branch on a
secret operand and never compute a memory address from one.

```sh
cargo run -p ct-check
```

It builds the operations for two target CPUs, runs each build under Valgrind's memcheck with the
operands marked undefined, and prints one line per build. It fails when memcheck reports an error:
"Conditional jump or move depends on uninitialised value(s)" is a branch on a secret, and "Use of
uninitialised value of size 8" is a memory address computed from one. It needs `valgrind` in
`PATH` on x86-64 Linux.

A conditional move on a secret passes, as does an instruction whose latency depends on its
operands, such as a division.

## Coverage

Every operation of `Secret` (the sum, the difference and the product with a `Secret` or a public
element on either side and their assigning forms, `neg`, `square`, `double`, `pow` with a public
exponent, `invert`, `sqrt`) in `prime-field`, over the nine fields of its `compare` bench, and in
`binary-field`, over the seven fields of its `compare` bench.

| Build | `RUSTFLAGS` |
|---|---|
| `x86-64` | `-C target-cpu=x86-64` |
| `x86-64-v3` | `-C target-cpu=x86-64-v3 -C target-feature=+adx,+pclmulqdq` |

Each build is the release profile in `target/ct-check/<build>`, with line tables.

## Exemption

The `Option` that `invert` returns discloses whether the input is zero, and the one of
`prime-field`'s `sqrt` whether the input is a square. `memcheck.supp` exempts a conditional jump
whose innermost frame is `invert::declassify`. A report may name the other library's
`declassify`: LLVM merges identical instances.
