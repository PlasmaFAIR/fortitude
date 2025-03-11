# no-real-suffix (C021)
## What it does
Checks for floating point literal constants that don't have their kinds
explicitly specified.

## Why is this bad?
Floating point literals use the default 'real' kind unless given an explicit
kind suffix. This can cause surprising loss of precision:

```f90
use, intrinsic :: iso_fortran_env, only: dp => real64

real(dp), parameter :: pi_1 = 3.14159265358979
real(dp), parameter :: pi_2 = 3.14159265358979_dp

print *, pi_1  ! Gives: 3.1415927410125732
print *, pi_2  ! Gives: 3.1415926535897900
```

There are cases where the difference in precision doesn't matter, such
as:

```f90
real(dp) :: x, y

x = 1.0
y = real(2.0, kind=dp)
```

A case where a missing suffix may be intentional is when using a `kind`
statement:

```f90
integer, parameter :: sp = kind(0.0)
```

This rule will try to avoid catching these case. However, even for 'nice'
numbers, it's possible to accidentally lose precision in surprising ways:

```f90
real(dp) :: x

x = sqrt(2.0)
```

This rule will therefore require an explicit kind statement in the majority
of cases where a floating point literal is found in an expression.

## References
- [Fortran-Lang Best Practices on Floating Point Numbers](https://fortran-lang.org/learn/best_practices/floating_point/)