# too-many-arguments (S902)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for procedures with a large number of arguments.

## Why is this bad?
Procedures with many arguments are harder to understand, maintain, call,
and test. They often indicate that a procedure is doing too much and should
be refactored into smaller, more focused procedures, or that a derived type
should be used to group related arguments together.

For type-bound procedures, the first argument is not counted towards the
total number of arguments. It is recommended to name this argument `this` or
`self` to make it clear that the routine is type-bound, or else this rule
may flag routines that are actually compliant.

## Example

The following procedure would be flagged for having too many arguments:
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
```

## Options
- [`check.complexity.max-args`][check.complexity.max-args]


[check.complexity.max-args]: ../settings.md#check_complexity_max-args

