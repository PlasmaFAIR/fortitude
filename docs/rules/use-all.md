# use-all (M011)
This rule is turned on by default.

## What it does
Checks whether `use` statements are used correctly.

## Why is this bad?
When using a module, it is recommended to add an 'only' clause to specify which
components you intend to use:

## Example
```f90
! Not recommended
use, intrinsic :: iso_fortran_env

! Better
use, intrinsic :: iso_fortran_env, only: int32, real64
```

This makes it easier for programmers to understand where the symbols in your
code have come from, and avoids introducing many unneeded components to your
local scope.