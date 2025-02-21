# literal-kind (PORT011)
This rule is turned on by default.

## What it does
Checks for use of raw number literals as kinds

## Why is this bad?
Rather than setting an intrinsic type's kind using an integer literal, such as
`real(8)` or `integer(kind=4)`, consider setting kinds using parameters in the
intrinsic module `iso_fortran_env` such as `real64` and `int32`. For
C-compatible types, consider instead `iso_c_binding` types such as
`real(c_double)`.

Although it is widely believed that `real(8)` represents an 8-byte floating
point (and indeed, this is the case for most compilers and architectures),
there is nothing in the standard to mandate this, and compiler vendors are free
to choose any mapping between kind numbers and machine precision. This may lead
to surprising results if your code is ported to another machine or compiler.

For floating point variables, we recommended using `real(sp)` (single
precision), `real(dp)` (double precision), and `real(qp)` (quadruple precision),
using:

```f90
use, intrinsic :: iso_fortran_env, only: sp => real32, &
                                         dp => real64, &
                                         qp => real128
```

Or alternatively:

```f90
integer, parameter :: sp = selected_real_kind(6, 37)
integer, parameter :: dp = selected_real_kind(15, 307)
integer, parameter :: qp = selected_real_kind(33, 4931)
```

Some prefer to set one precision parameter `wp` (working precision), which is
set in one module and used throughout a project.

Integer sizes may be set similarly:

```f90
integer, parameter :: i1 = selected_int_kind(2)  ! 8 bits
integer, parameter :: i2 = selected_int_kind(4)  ! 16 bits
integer, parameter :: i4 = selected_int_kind(9)  ! 32 bits
integer, parameter :: i8 = selected_int_kind(18) ! 64 bits
```

Or:

```f90
use, intrinsic :: iso_fortran_env, only: i1 => int8, &
                                         i2 => int16, &
                                         i4 => int32, &
                                         i8 => int64
```