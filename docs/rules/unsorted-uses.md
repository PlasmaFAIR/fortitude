# unsorted-uses (S271)
Fix is always available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks that `use` statements are sorted alphabetically within contiguous blocks.
Intrinsic modules (`use, intrinsic ::`) are always placed first.

## Why is this bad?
Sorted imports are easier to scan, reduce cognitive load when reviewing code,
and help avoid merge conflicts when multiple developers add imports to the same block.

Blocks of `use` statements separated by blank lines are sorted independently.

## Example
```f90
! Not recommended
use module_c, only: fun_c
use, intrinsic :: iso_fortran_env, only: int32
use module_a, only: fun_a
use module_b, only: fun_b

! Better
use, intrinsic :: iso_fortran_env, only: int32
use module_a, only: fun_a
use module_b, only: fun_b
use module_c, only: fun_c
```
