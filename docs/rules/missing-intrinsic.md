# missing-intrinsic (M012)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks whether `use` statements for intrinic modules specify `intrinsic` or
`non_intrinsic`.

## Why is this bad?
The compiler will default to using a non-intrinsic module, if there is one,
so not specifying the `intrinsic` modifier on intrinsic modules may lead to
the compiler version being shadowed by a different module with the same name.

## Example
```f90
! Not recommended
use :: iso_fortran_env, only: int32, real64

! Better
use, intrinsic :: iso_fortran_env, only: int32, real64
```

This ensures the compiler will use the built-in module instead of a different
module with the same name.