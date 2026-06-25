# unary-following-arithmetic (PORT051)
Fix is sometimes available.

This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of a unary expression following an arithmetic operator.

## Why is this bad?
The use of a unary operator (`+`, `-` but *not* user-defined) following an arithmetic operator (
`+`, `-`, `*`, `/`, `**` but *not* user-defined) can be ambiguous and is not part of the Fortran
standard. The order of operations does not necessarily follow typical mathematical order. See
(the Doctor Fortran
article)[https://stevelionel.com/drfortran/2021/04/03/doctor-fortran-in-order-order/].

Some compilers may warn users of this code smell, but only via extensions. The use of a unary
operator following an arithmetic operator may result in unexpected behaviour and/or differences
in output between compilers. Use parentheses to remove ambiguity of user expected output; the
fix may change prior behaviour on some compilers.

## Example
```f90
x = 10 ** -2 * 2
! Would expected x = 0.02 but some compilers may give x = 0.0001.
```

Use instead:
```f90
x = 10 ** (-2) * 2
! Result is unambiguously x = 0.02.
```
