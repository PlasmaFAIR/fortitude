# missing-kind-argument (C023)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks calls to certain intrinsic functions that return numbers for a missing
explicit `kind` argument.

## Why is this bad?
Without an explicit `kind` argument, conversions done by the `CMPLX`, `REAL`, `INT`,
`AINT`, `ANINT`, `CEILING`, and `FLOOR` intrinsics use a compiler-dependent default
kind for their return value. That can silently reduce precision or change the integer
kind used by a conversion, which can lead to unexpected results and potentially
non-portable behavior.

## Example
In the following example, While `x` and `y` are declared as real64 variables, the
`REAL` intrinsic will return a real32 value when called without an explicit `kind`
argument on many compilers, silently truncating the value of `x` and losing precision
when it is assigned to `y`.
```f90
use, intrinsic :: iso_fortran_env, only: dp => real64, i8 => int64

real(dp) :: x, y

x = 1e-10_dp
y = real(x)
print *, int(y)
```

Use instead:
```f90
use, intrinsic :: iso_fortran_env, only: dp => real64, i8 => int64

real(dp) :: x, y

x = 1e-10_dp
y = real(x, kind=dp)
print *, int(y, kind=i8)
```
