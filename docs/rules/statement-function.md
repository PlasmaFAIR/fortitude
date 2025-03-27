# statement-function (OB001)
**Warning: This rule has been removed and its documentation is only available for historical reasons.**

This rule is turned on by default.

## What it does
Checks for statement functions. This rule has been temporarily removed
while we investigate false positives.

## Why is this bad?
Statement functions are an obsolescent feature from Fortran 77,
and have been entirely supplanted by internal
procedures. Statement functions are much more limited in what they
can do. They were declared obsolescent in Fortran 90 and removed
in Fortran 95.

## Examples
Statement functions are easily replaced with internal procedures:

```f90
real :: f, x
f(x) = x**2 + x
```
becomes:

```f90
contains
  real function f(x)
    real, intent(in) :: x
    f = x**2 + x
  end function f
```

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Appendix B
  'Obsolescent and Deleted Features'
