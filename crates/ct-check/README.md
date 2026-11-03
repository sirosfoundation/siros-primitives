# ct-check

Checks that the operations of `ConstantTime` in `biguint`, `prime-field` and `binary-field` never
branch on a wrapped operand and never compute a memory address from one.


Checks that the `ConstantTime` operations of `biguint`, `prime-field` and `binary-field` never
branch on a wrapped value and never compute a memory address from one. Runs on x86-64 Linux with
`valgrind` in `PATH`.

```sh
cargo run -p ct-check
```

It runs every operation under Valgrind's memcheck with the wrapped operands marked undefined, in an
`x86-64` and an `x86-64-v3` release build, and fails on any memcheck error:

- "Conditional jump or move depends on uninitialised value(s)": a branch on a wrapped value.
- "Use of uninitialised value of size 8": a memory address computed from a wrapped value.

A new `ConstantTime` operation gets a line in `src/operations.rs`. A fact an operation must
disclose, such as whether an inverse exists, goes through its crate's `declassify`, which
`memcheck.supp` exempts.
