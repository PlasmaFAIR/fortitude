# magic-number-in-array-size (R001)
This rule is unstable and in [preview](../preview.md). The `--preview` flag is required for use.

## What it does
Checks for use of literals when specifying array sizes

## Why is this bad?
Prefer named constants to literal integers when declaring arrays. This makes
it easier to find similarly sized arrays in the codebase, as well as ensuring
they are consistently sized when specified in different places. Named
parameters also make it easier for readers to understand your code.

The values `0, 1, 2, 3, 4` are ignored by default.

TODO: Add user settings

## Examples
Instead of:
```f90
integer, dimension(10) :: x, y
```
prefer:
```f90
integer, parameter :: NUM_SPLINE_POINTS = 10
integer, dimension(NUM_SPLINE_POINTS) :: x, y
```