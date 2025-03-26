# implicit-real-kind (C022)
## What it does
Checks for `real` variables that don't have their kind explicitly specified.

## Why is this bad?
Real variable declarations without an explicit kind will have a compiler/platform
dependent precision, which hurts portability and may lead to surprising loss of
precision in some cases. Although the default `real` will map to a 32-bit floating
point number on most systems, this is not guaranteed.

It is recommended to always be explicit about the precision required by `real`
variables. This can be done by setting their 'kinds' using integer parameters
chosen in one of the following ways:

```f90
! Set using iso_fortran_env
use, intrinsic :: iso_fortran_env, only: sp => real32, dp => real64
! Using selected_real_kind
integer, parameter :: sp = selected_real_kind(6, 37)
integer, parameter :: dp = selected_real_kind(15, 307)
! For C-compatibility:
use, intrinsic :: iso_c_binding, only: sp => c_float, dp => c_double

! Declaring real variables:
real(sp) :: single
real(dp) :: double
```

It is also common for Fortran developers to set a 'working precision' `wp`,
which is set to either `sp` or `dp` and used throughout a project. This can
then be easily toggled depending on the user's needs.

## References
- [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)
