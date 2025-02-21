# star-kind (PORT021)
Fix is sometimes available.

This rule is turned on by default.

## What it does
Checks for non-standard kind specifiers such as `int*4` or `real*8`

## Why is this bad?
Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
avoided. For these cases, consider instead using 'real(real64)' or
'integer(int32)', where 'real64' and 'int32' may be found in the intrinsic
module 'iso_fortran_env'. You may also wish to determine kinds using the
built-in functions 'selected_real_kind' and 'selected_int_kind'.

Fixes to this rule are considered unsafe, as while `dtype*N` is generally
understood to mean a `dtype` that occupied `N` bytes, this does not necessarily
correspond to `dtype(N)`, which is a `dtype` of 'kind' `N`. For example, the NAG
compiler may be conigured to use a sequential kind system in which `real*8`
corresponds to `real(2)`

In a future version, we hope to upgrade this to a safe fix by use of parameters
in `iso_fortran_env`, as `real*8` should always correspond to `real(real64)`.