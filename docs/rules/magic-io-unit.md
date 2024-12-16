# magic-io-unit (IO011)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for literal integers as units in IO statements.

## Why is this bad?
Hardcoding unit numbers makes programs more brittle as it becomes harder to
verify units have been opened before reading/writing. Instead, units should
be passed in to procedures as arguments, or the `newunit=` argument used for
`open` statements.

Bad:
```f90
open(10, file="example.txt", action="read")
read(10, fmt=*) int
close(10)
```

Good:
```f90
open(newunit=example_unit, file="example.txt", action="read")
read(example_unit, fmt=*) int
close(example_unit)
```