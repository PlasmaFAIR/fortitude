# whole-array-indexing (S312)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for redundant whole-array indexing.

## Why is this bad?
Adding `(:)` to reference an entire array is redundant. Omitting the
subscript makes the same whole-array reference clearer and avoids unnecessary
parser/compiler work.

## Example
```f90
x = x(:)
y = y(:, :)
```

Use instead:
```f90
x = x
y = y
```

## References
- [Doctor Fortran: "Doctor, it hurts when I do this!"](https://stevelionel.com/drfortran/2008/03/31/doctor-it-hurts-when-i-do-this/)
