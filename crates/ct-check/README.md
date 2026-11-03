# ct-check

Checks that the operations of `ConstantTime` in `biguint`, `prime-field` and `binary-field` never
branch on a wrapped operand and never compute a memory address from one.


Checks that every `ConstantTime` operation of `biguint`, `prime-field` and `binary-field` runs
without a branch or a memory address that depends on a wrapped value. Needs `valgrind` on x86-64
Linux.

```sh
cargo run -p ct-check
```

It runs each operation under Valgrind's memcheck with the wrapped operands marked undefined, in an
`x86-64` and an `x86-64-v3` release build, and fails on any memcheck error. `memcheck.supp` exempts
`declassify`, the one place where `invert` and `sqrt` disclose whether a result exists.
