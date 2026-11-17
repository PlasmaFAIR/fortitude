# unsorted-use-statement (S272)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks that entities imported by a `use ... only:` statement are sorted.

## Why is this bad?
Sorted imports are easier to scan, reduce cognitive load when reviewing
code, and help avoid merge conflicts when multiple developers add entities
to the same statement.

## Example
```f90
! Not recommended
use my_module, only: zeta, alpha, beta

! Better
use my_module, only: alpha, beta, zeta
```
