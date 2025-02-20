# implicit-external-procedures (C053)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks if `implicit none` is missing `external`

## Why is this bad?
`implicit none` disables implicit types of variables but still allows
implicit interfaces for procedures. Fortran 2018 added the ability to also
forbid implicit interfaces through `implicit none (external)`, enabling the
compiler to check the number and type of arguments and return values.

`implicit none` is equivalent to `implicit none (type)`, so the full
statement should be `implicit none (type, external)`.