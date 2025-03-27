# function-missing-result (S221)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does

Checks that every function explicitly declares which variable contains the result.

## Why is this bad?
Specifying the `result()` clause allows for the variable inside the function containing the
result to be named differently from the function.

Allowing the variable to be named different from the function allows for better naming of both
the function and the internal variables, and also it can help when creating functions that might
need to be duplicated to have versions supporting various types. Since declaring the result in
the `result()` clause allows for easier duplication of the function because then less lines must
be changed.

## Example
```f90
interface distance
    module procedure distance_real32, distance_real64
end interface distance

function distance_real32(a, b)
    real(kind=real32), intent(in) :: a, b
    real(kind=real32)             :: distance_real32

    distance_real32 = abs(a-b)
end function distance_real

function distance_real64(a, b)
    real(kind=real64), intent(in) :: a, b
    real(kind=real64)             :: distance_real64

    distance_real64 = abs(a-b)
end function distance_real
```

Use instead:
```f90
interface distance
    module procedure distance_real32, distance_real64
end interface distance

function distance_real32(a, b) result(distance)
    integer, parameter               :: func_type = real32

    real(kind=func_type), intent(in) :: a, b
    real(kind=func_type)             :: distance

    distance = abs(a-b)
end function distance_real

function distance_real64(a, b) result(distance)
    integer, parameter               :: func_type = real64

    real(kind=func_type), intent(in) :: a, b
    real(kind=func_type)             :: distance

    distance = abs(a-b)
end function distance_real
```

## References
- Clerman, N. Spector, W., 2012, _Modern Fortran: Style and Usage_, Cambridge
  University Press, Rule 129, p. 178.
