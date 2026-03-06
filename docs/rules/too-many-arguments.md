# too-many-arguments (S911)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for procedures with a large number of arguments.

## Why is this bad?
Procedures with many arguments are harder to understand, maintain, call,
and test. They often indicate that a procedure is doing too much and should
be refactored into smaller, more focused procedures, or that a derived type
should be used to group related arguments together.

The equivalent rule in Pylint has a default threshold of 5, but we set it to
10 for Fortran to account for the fact that Fortran procedures often have
more arguments than Python functions.

## Example

If we set the threshold to 6 or lower, then the following procedure would be
flagged for having too many arguments:
```f90
subroutine update_position(x, y, z, vx, vy, vz, dt)
  real, intent(inout) :: x, y, z
  real, intent(in) :: vx, vy, vz, dt
  x = x + vx * dt
  y = y + vy * dt
  z = z + vz * dt
end subroutine update_position
```

Use instead:
```f90
subroutine update_position(position, velocity, dt)
  type(vector), intent(inout) :: position
  type(vector), intent(in) :: velocity
  real, intent(in) :: dt
  position%x = position%x + velocity%x * dt
  position%y = position%y + velocity%y * dt
  position%z = position%z + velocity%z * dt
end subroutine update_position
```

where `vector` is a derived type defined as:
```f90
type :: vector
 real :: x
 real :: y
 real :: z
end type vector

## Options
- [`check.complexity.max-args`][check.complexity.max-args]


[check.complexity.max-args]: ../settings.md#check_complexity_max-args

