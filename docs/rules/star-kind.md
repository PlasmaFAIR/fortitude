# star-kind (T021)
Fix is always available.

## What it does
Checks for non-standard kind specifiers such as `int*4` or `real*8`

## Why is this bad?
Types such as 'real*8' or 'integer*4' are not standard Fortran and should be
avoided. For these cases, consider instead using 'real(real64)' or
'integer(int32)', where 'real64' and 'int32' may be found in the intrinsic
module 'iso_fortran_env'. You may also wish to determine kinds using the
built-in functions 'selected_real_kind' and 'selected_int_kind'.

Also prefers the use of `character(len=*)` to
`character*(*)`, as although the latter is permitted by the standard, the former is
more explicit.