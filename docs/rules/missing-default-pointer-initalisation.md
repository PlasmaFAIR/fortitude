# missing-default-pointer-initalisation (T071)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

This rule is turned on by default.

## What it does
Checks for uninitialised pointer variables inside derived types

## Why is this bad?
Pointers inside derived types are undefined by default, and their
status cannot be tested by intrinsics such as `associated`. Pointer
variables should be initialised by either associating them with another
variable, or associating to `null()`.


## Examples
For example, this derived type:

```f90
type mytype
    real :: val1
    integer :: val2

    real, pointer :: pReal1

    integer, pointer :: pInt1 => null()
    integer, pointer :: pI1
    integer, pointer :: pI2 => null(), pI3
end mytype
```
will have the pointers `pReal1`, `pI1`, and `pI3` uninitialised
whenever it is created. Instead, they should be initialised like:
```f90
type mytype
    real :: val1
    integer :: val2

    real, pointer :: pReal1

    integer, pointer :: pInt1 => null()
    integer, pointer :: pI1 => null()
    integer, pointer :: pI2 => null(), pI3  => null()
end mytype
```

## References
- Metcalf, M., Reid, J. and Cohen, M., 2018, _Modern Fortran Explained:
  Incorporating Fortran 2018_, Oxford University Press, Section 8.5.3/8.5.4.
- Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
  University Press, Rule 136, p. 189.