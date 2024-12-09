# literal-kind-suffix (T012)
## What it does
Checks for using an integer literal as a kind suffix

## Why is this bad?
Using an integer literal as a kind specifier gives no guarantees regarding the
precision of the type, as kind numbers are not specified in the Fortran
standards. It is recommended to use parameter types from `iso_fortran_env`:

```f90
use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
```

or alternatively:

```f90
integer, parameter :: sp => selected_real_kind(6, 37)
integer, parameter :: dp => selected_real_kind(15, 307)
```

Floating point constants can then be specified as follows:

```f90
real(sp), parameter :: sqrt2 = 1.41421_sp
real(dp), parameter :: pi = 3.14159265358979_dp
```