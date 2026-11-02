# ct-check

Checks that the operations of `ConstantTime` in `biguint`, `prime-field` and `binary-field` never
branch on a wrapped operand and never compute a memory address from one.

```sh
cargo run -p ct-check
```

It builds the operations for two target CPUs, runs each build under Valgrind's memcheck with the
wrapped operands marked undefined, and prints one line per build. It fails when memcheck reports
an error: "Conditional jump or move depends on uninitialised value(s)" is a branch on a wrapped
operand, and "Use of uninitialised value of size 8" is a memory address computed from one. It
needs `valgrind` in `PATH` on x86-64 Linux.

A conditional move on a wrapped operand passes, as does an instruction whose latency depends on
its operands, such as a division.

## Coverage

- `biguint`, at 64, 256, 521 and 4096 bits: `+ - * & | ^` between two `ConstantTime<Uint>`s and
  with a public `Uint` on either side, with their assigning forms; `!`, `<<`, `>>`, `<<=`, `>>=`,
  `square`, `double`, `pow` by a wrapped 256-bit exponent; `div_rem` and `%` by a 64-bit divisor;
  and under an odd and an even modulus, `add_mod`, `sub_mod`, `neg_mod`, `mul_mod`, `div_rem`,
  `/`, `%`, and every operation of `ConstantTimeResidue`: `reduce_constant_time`, `get`,
  `square`, `pow` by a wrapped exponent, `-`, and `+ - *` with a wrapped or a public residue on
  either side, with their assigning forms.
- `prime-field`, over the ten fields of its `compare` bench: `+ - *` between two
  `ConstantTime<Fp>`s and with a public element on either side, with their assigning forms, `-`,
  `square`, `double`, `pow` by a public exponent, `invert` and `sqrt`.
- `binary-field`, over the seven fields of its `compare` bench: `+ - *` in the same five forms,
  `-`, `square`, `sqrt` and `invert`.

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
