# implicit-external-procedures (C003)
## What it does
Checks if `implicit none` is missing `external`

## Why is this bad?
`implicit none` disables implicit types of variables but still allows
implicit interfaces for procedures. Fortran 2018 added the ability to also
forbid implicit interfaces through `implicit none (external)`, enabling the
compiler to check the number and type of arguments and return values.

`implicit none` is equivalent to `implicit none (type)`, so the full
statement should be `implicit none (type, external)`.