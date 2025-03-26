# magic-io-unit (C032)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for literal integers as units in IO statements.

## Why is this bad?
Hardcoding unit numbers makes programs more brittle as it becomes harder to
verify units have been opened before reading/writing. Instead, units should
be passed in to procedures as arguments, or the `newunit=` argument used for
`open` statements. Having a named variable also makes it much clearer what a
given IO statement is for, and allows tools like LSP and IDEs to find all
references.

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
